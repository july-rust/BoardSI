//! E_015: bounded UNIX compress (.Z) LZW decoder for ODB++ layer features.
//!
//! This is *not* gzip, zlib, or ZIP. UNIX compress has no CRC, ISIZE, or
//! unambiguous end-of-stream code: successful decoding alone cannot certify
//! original exporter bytes or detect every truncated/extended bit stream.
//! No external process, filesystem extraction, or unsafe code is used.
use crate::odb_source::OdbError;

fn bad(path: &str, code: &'static str) -> OdbError {
    OdbError {
        path: path.to_owned(),
        line: 0,
        code,
    }
}

/// Classic compress stores up to eight LSB-first codes in a block containing
/// exactly 'width' bytes. On a code-width change or CLEAR, the encoder pads
/// the previous block; the decoder must discard its remaining bits.
struct CodeGroups<'a> {
    packed: &'a [u8],
    offset: usize,
    group: [u8; 16],
    available: usize,
    used: usize,
}
impl<'a> CodeGroups<'a> {
    fn new(packed: &'a [u8]) -> Self {
        Self {
            packed,
            offset: 0,
            group: [0; 16],
            available: 0,
            used: 0,
        }
    }
    fn read(&mut self, width: usize, realign: bool) -> Result<Option<usize>, &'static str> {
        if realign {
            self.available = 0;
            self.used = 0;
        }
        if self.used == self.available {
            if self.offset == self.packed.len() {
                return Ok(None);
            }
            let count = width.min(self.packed.len() - self.offset);
            if count * 8 < width {
                return Err("ODB_Z_TRUNCATED_CODE");
            }
            self.group[..count].copy_from_slice(&self.packed[self.offset..self.offset + count]);
            self.offset += count;
            self.available = (count * 8) / width;
            self.used = 0;
        }
        let start = self.used * width;
        let mut code = 0usize;
        for bit in 0..width {
            let position = start + bit;
            let value = (self.group[position / 8] >> (position % 8)) & 1;
            code |= usize::from(value) << bit;
        }
        self.used += 1;
        Ok(Some(code))
    }
}

/// Expand a dictionary entry in reversed byte order without recursion. The
/// prefix chain is bounded by the dictionary size even for malicious input.
fn expand(
    mut code: usize,
    next: usize,
    prefix: &[u16],
    suffix: &[u8],
    stack: &mut Vec<u8>,
) -> Result<u8, &'static str> {
    stack.clear();
    let mut count = 0usize;
    while code >= 256 {
        if code == 256 || code >= next || count >= prefix.len() {
            return Err("ODB_Z_INVALID_DICTIONARY");
        }
        stack.push(suffix[code]);
        code = usize::from(prefix[code]);
        count += 1;
    }
    stack.push(code as u8);
    Ok(code as u8)
}

/// Decode classic UNIX compress (magic 1F 9D), allowing 9..16-bit dictionaries
/// and optional block mode. Enforce the output limit *before* appending data.
///
/// The .Z stream lacks a mandatory checksum or EOS. The caller additionally
/// validates the complete decoded feature grammar and source-file inventory.
pub(crate) fn decode_features_z(
    path: &str,
    bytes: &[u8],
    limit: usize,
) -> Result<Vec<u8>, OdbError> {
    if bytes.len() < 5 || bytes.get(..2) != Some(&[0x1f, 0x9d][..]) {
        return Err(bad(path, "ODB_Z_BAD_HEADER"));
    }
    let flags = bytes[2];
    if flags & 0x60 != 0 {
        return Err(bad(path, "ODB_Z_RESERVED_FLAGS"));
    }
    let max_bits = usize::from(flags & 0x1f);
    if !(9..=16).contains(&max_bits) {
        return Err(bad(path, "ODB_Z_UNSUPPORTED_CODE_WIDTH"));
    }
    let block_mode = flags & 0x80 != 0;
    let capacity = 1usize << max_bits;
    let mut prefix = vec![0u16; capacity];
    let mut suffix = vec![0u8; capacity];
    let mut stack = Vec::new();
    let mut groups = CodeGroups::new(&bytes[3..]);
    let mut output = Vec::new();
    let mut width = 9usize;
    let mut next = if block_mode { 257usize } else { 256usize };
    let mut previous: Option<usize> = None;
    let mut realign = false;
    loop {
        let code = match groups
            .read(width, realign)
            .map_err(|c| bad(path, c))?
        {
            Some(value) => value,
            None => break,
        };
        realign = false;
        if block_mode && code == 256 {
            // CLEAR resets the entire dictionary and aligns to a 9-bit block.
            next = 257;
            previous = None;
            width = 9;
            realign = true;
            continue;
        }
        let Some(old) = previous else {
            if code > 255 {
                return Err(bad(path, "ODB_Z_INVALID_FIRST_CODE"));
            }
            if output.len() >= limit {
                return Err(bad(path, "ODB_Z_EXPANSION_LIMIT"));
            }
            output.push(code as u8);
            previous = Some(code);
            continue;
        };
        if code > next || (code == next && next >= capacity) {
            return Err(bad(path, "ODB_Z_INVALID_CODE"));
        }
        let special = code == next; // LZW KwKwK case.
        let first = expand(
            if special { old } else { code },
            next,
            &prefix,
            &suffix,
            &mut stack,
        )
        .map_err(|c| bad(path, c))?;
        let wanted = output
            .len()
            .checked_add(stack.len())
            .and_then(|n| n.checked_add(usize::from(special)))
            .ok_or_else(|| bad(path, "ODB_Z_SIZE_OVERFLOW"))?;
        if wanted > limit {
            return Err(bad(path, "ODB_Z_EXPANSION_LIMIT"));
        }
        output.extend(stack.iter().rev().copied());
        if special {
            output.push(first);
        }
        if next < capacity {
            prefix[next] = u16::try_from(old).map_err(|_| bad(path, "ODB_Z_INVALID_CODE"))?;
            suffix[next] = first;
            next += 1;
        }
        previous = Some(code);
        if next > ((1usize << width) - 1) && width < max_bits {
            width += 1;
            realign = true;
        }
    }
    if previous.is_none() || output.is_empty() {
        return Err(bad(path, "ODB_Z_EMPTY_OR_INCOMPLETE"));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    const PATH: &str = "steps/pcb/layers/top/features.Z";
    // Fixed fixture: independent GNU gzip -dc validates this UNIX-compress
    // byte stream against the exact UTF-8 feature text below.
    const FEATURE_Z: &[u8] = &[
        0x1f, 0x9d, 0x8c, 0x55, 0x9c, 0x24, 0xa1, 0x32, 0xa5, 0x47, 0x93,
        0x26, 0x0a, 0x8c, 0x80, 0x88, 0xa1, 0x80, 0x04, 0x0c, 0x10, 0x72,
        0x62, 0xc0, 0x80, 0xa1, 0x80, 0x09, 0x08, 0x18, 0x20, 0x60, 0x80,
        0x88, 0x01, 0x02, 0x06, 0x08, 0x18, 0x20, 0xa0, 0x80, 0x80, 0xa1,
        0x00,
    ];
    const TEXT: &[u8] = b"UNITS=MM\nF 1\n$0 r100\nL 0 0 1 0 0 P 0\n";

    #[test]
    fn independent_unix_compress_vector_decodes() {
        assert_eq!(decode_features_z(PATH, FEATURE_Z, 16 * 1024 * 1024).unwrap(), TEXT);
        assert_eq!(
            decode_features_z(PATH, FEATURE_Z, TEXT.len() - 1)
                .unwrap_err()
                .code,
            "ODB_Z_EXPANSION_LIMIT"
        );
    }

    #[test]
    fn kwkwk_and_dictionary_clear_use_correct_lzw_semantics() {
        assert_eq!(
            decode_features_z(PATH, &[0x1f, 0x9d, 0x8c, 0x41, 0x02, 0x02], 100).unwrap(),
            b"AAA"
        );
        assert_eq!(
            decode_features_z(
                PATH,
                &[0x1f, 0x9d, 0x8c, 0x41, 0x84, 0, 4, 0, 0, 0, 0, 0, 0x43, 0x88, 0],
                100
            )
            .unwrap(),
            b"ABCD"
        );
    }

    // A deterministic *literal* LZW stream, packed in classic compress
    // groups. This builder's 9->10->11-bit outputs were independently
    // accepted and byte-decoded by GNU gzip -dc as a UNIX-compress oracle.
    fn literal_z(bytes: &[u8], maxbits: usize) -> Vec<u8> {
        fn flush(output: &mut Vec<u8>, codes: &mut Vec<u8>, width: usize, full: bool) {
            if codes.is_empty() {
                return;
            }
            let mut packed = 0u128;
            for (index, &code) in codes.iter().enumerate() {
                packed |= u128::from(code) << (index * width);
            }
            let count = if full { width } else { (codes.len() * width).div_ceil(8) };
            output.extend_from_slice(&packed.to_le_bytes()[..count]);
            codes.clear();
        }
        let mut out = vec![0x1f, 0x9d, 0x80 | maxbits as u8];
        let mut width = 9usize;
        let mut next = 257usize;
        let mut first = true;
        let mut codes = Vec::new();
        for &byte in bytes {
            if next > ((1usize << width) - 1) && width < maxbits {
                flush(&mut out, &mut codes, width, true);
                width += 1;
            }
            codes.push(byte);
            if codes.len() == 8 {
                flush(&mut out, &mut codes, width, true);
            }
            if !first && next < (1usize << maxbits) {
                next += 1;
            }
            first = false;
        }
        flush(&mut out, &mut codes, width, false);
        out
    }

    #[test]
    fn variable_width_group_alignment_and_bounded_expansion() {
        for size in [255usize, 256, 257, 260, 1000, 4000] {
            let plain: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
            let stream = literal_z(&plain, 12);
            assert_eq!(decode_features_z(PATH, &stream, size).unwrap(), plain);
            assert_eq!(
                decode_features_z(PATH, &stream, size - 1).unwrap_err().code,
                "ODB_Z_EXPANSION_LIMIT"
            );
        }
    }

    #[test]
    fn malformed_streams_fail_closed() {
        for data in [
            vec![],
            vec![0x1f, 0x8b, 0x8c, 0, 0],
            vec![0x1f, 0x9d, 0xac, 0, 0],
            vec![0x1f, 0x9d, 0x88, 0, 0],
            vec![0x1f, 0x9d, 0x91, 0, 0],
            vec![0x1f, 0x9d, 0x8c, 1],
            vec![0x1f, 0x9d, 0x8c, 0x2c, 0x01], // invalid first code=300
        ] {
            assert!(decode_features_z(PATH, &data, 100).is_err());
        }
    }
}