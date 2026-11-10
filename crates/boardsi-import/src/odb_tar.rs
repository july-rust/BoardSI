//! D_056_08: bounded, source-only USTAR product archive reader.
//! This is an uncompressed TAR reader. Gzip/ZIP, PAX, links, sparse files and
//! unknown extensions MUST fail closed rather than becoming incomplete PCB data.
use crate::odb_source::{OdbError, OdbFileMap};

const BLOCK: usize = 512;
const MAX_ARCHIVE: usize = 256 * 1024 * 1024;
const MAX_FILE: usize = 16 * 1024 * 1024;
const MAX_TOTAL: usize = 128 * 1024 * 1024;
const MAX_ENTRIES: usize = 8192;

fn bad(code: &'static str) -> OdbError {
    OdbError {
        path: "product.tar".into(),
        line: 0,
        code,
    }
}

fn field(bytes: &[u8]) -> Result<String, OdbError> {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    if bytes[end..].iter().any(|&b| b != 0) {
        return Err(bad("ODB_TAR_INVALID_STRING"));
    }
    let s = std::str::from_utf8(&bytes[..end]).map_err(|_| bad("ODB_TAR_NONUTF8_PATH"))?;
    if s.bytes().any(|b| b < 0x20 || b == 0x7f) {
        return Err(bad("ODB_TAR_INVALID_PATH"));
    }
    Ok(s.to_owned())
}

fn octal(bytes: &[u8]) -> Result<u64, OdbError> {
    // Reject base-256 extensions and malformed non-octal values.
    if bytes.first().is_some_and(|b| b & 0x80 != 0) {
        return Err(bad("ODB_TAR_UNSUPPORTED_NUMBER"));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| bad("ODB_TAR_BAD_NUMBER"))?;
    let digits = text.trim_matches(|c: char| c == '\0' || c == ' ');
    if digits.is_empty() || !digits.bytes().all(|b| (b'0'..=b'7').contains(&b)) {
        return Err(bad("ODB_TAR_BAD_NUMBER"));
    }
    u64::from_str_radix(digits, 8).map_err(|_| bad("ODB_TAR_BAD_NUMBER"))
}

fn safe_name(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 1024
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains(':')
        && !path.contains('\0')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn validate_product_files(files: &OdbFileMap) -> Result<(), OdbError> {
    if !files.contains_key("matrix/matrix") {
        return Err(bad("ODB_TAR_MISSING_MATRIX"));
    }
    // Both direct-root and prefix-normalized packages must reject a nested
    // second product instead of silently importing only the first board.
    if files
        .keys()
        .any(|path| path.ends_with("/matrix/matrix") || path.contains("/steps/"))
    {
        return Err(bad("ODB_TAR_MULTIPLE_ROOTS"));
    }
    Ok(())
}

/// Inspect a full TAR before publishing any source-file map. The end-of-archive
/// marker is mandatory and ignored data cannot hide extra nonzero members.
pub fn read_tar(bytes: &[u8]) -> Result<OdbFileMap, OdbError> {
    if bytes.starts_with(&[0x1f, 0x8b]) || bytes.starts_with(b"PK\x03\x04") {
        return Err(bad("ODB_TAR_COMPRESSION_UNSUPPORTED"));
    }
    if bytes.is_empty() || bytes.len() > MAX_ARCHIVE || bytes.len() % BLOCK != 0 {
        return Err(bad("ODB_TAR_SIZE"));
    }
    let mut raw = OdbFileMap::new();
    let mut offset = 0usize;
    let mut cumulative = 0usize;
    let mut entries = 0usize;
    let mut terminated = false;
    while offset < bytes.len() {
        let h = &bytes[offset..offset + BLOCK];
        if h.iter().all(|b| *b == 0) {
            if bytes.len() < offset + 2 * BLOCK || bytes[offset..].iter().any(|&b| b != 0) {
                return Err(bad("ODB_TAR_MISSING_END"));
            }
            terminated = true;
            break;
        }
        entries += 1;
        if entries > MAX_ENTRIES {
            return Err(bad("ODB_TAR_ENTRY_LIMIT"));
        }
        if &h[257..263] != b"ustar\0" && &h[257..263] != b"ustar " {
            return Err(bad("ODB_TAR_UNSUPPORTED_HEADER"));
        }
        let checksum = octal(&h[148..156])?;
        let sum: u64 = h
            .iter()
            .enumerate()
            .map(|(i, b)| {
                if (148..156).contains(&i) {
                    32u64
                } else {
                    u64::from(*b)
                }
            })
            .sum();
        if sum != checksum {
            return Err(bad("ODB_TAR_CHECKSUM"));
        }
        let filename = field(&h[..100])?;
        let prefix = field(&h[345..500])?;
        let name = if prefix.is_empty() {
            filename
        } else {
            format!("{prefix}/{filename}")
        };
        let directory = h[156] == b'5';
        let regular = h[156] == 0 || h[156] == b'0';
        if !regular && !directory {
            return Err(bad("ODB_TAR_UNSUPPORTED_MEMBER"));
        }
        let name = if directory {
            name.trim_end_matches('/')
        } else {
            &name
        }
        .to_owned();
        if !safe_name(&name) {
            return Err(bad("ODB_TAR_UNSAFE_PATH"));
        }
        let n = usize::try_from(octal(&h[124..136])?).map_err(|_| bad("ODB_TAR_FILE_LIMIT"))?;
        if n > MAX_FILE || (directory && n != 0) {
            return Err(bad("ODB_TAR_FILE_LIMIT"));
        }
        let padded = n
            .checked_add(BLOCK - 1)
            .and_then(|v| v.checked_div(BLOCK))
            .and_then(|v| v.checked_mul(BLOCK))
            .ok_or_else(|| bad("ODB_TAR_SIZE"))?;
        let start = offset
            .checked_add(BLOCK)
            .ok_or_else(|| bad("ODB_TAR_SIZE"))?;
        let end = start
            .checked_add(padded)
            .ok_or_else(|| bad("ODB_TAR_SIZE"))?;
        if end > bytes.len() {
            return Err(bad("ODB_TAR_TRUNCATED"));
        }
        if regular {
            cumulative = cumulative
                .checked_add(n)
                .ok_or_else(|| bad("ODB_TAR_TOTAL_LIMIT"))?;
            if cumulative > MAX_TOTAL || raw.len() >= MAX_ENTRIES {
                return Err(bad("ODB_TAR_TOTAL_LIMIT"));
            }
            if raw.insert(name, bytes[start..start + n].to_vec()).is_some() {
                return Err(bad("ODB_TAR_DUPLICATE_FILE"));
            }
        }
        offset = end;
    }
    if !terminated || raw.is_empty() {
        return Err(bad("ODB_TAR_MISSING_END"));
    }

    // Archives may use one product-root directory, but never silently merge
    // unrelated roots or allow a member outside the product.
    if raw.contains_key("matrix/matrix") {
        validate_product_files(&raw)?;
        return Ok(raw);
    }
    let first = raw.keys().next().ok_or_else(|| bad("ODB_TAR_EMPTY"))?;
    let root = first
        .split('/')
        .next()
        .ok_or_else(|| bad("ODB_TAR_BAD_ROOT"))?;
    let prefix = format!("{root}/");
    if root == "matrix" || root == "steps" || !raw.keys().all(|path| path.starts_with(&prefix)) {
        return Err(bad("ODB_TAR_MULTIPLE_ROOTS"));
    }
    let mut flat = OdbFileMap::new();
    for (path, data) in raw {
        let relative = path
            .strip_prefix(&prefix)
            .ok_or_else(|| bad("ODB_TAR_MULTIPLE_ROOTS"))?;
        if !safe_name(relative) || flat.insert(relative.to_owned(), data).is_some() {
            return Err(bad("ODB_TAR_UNSAFE_PATH"));
        }
    }
    validate_product_files(&flat)?;
    Ok(flat)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(name: &str, data: &[u8], member_type: u8) -> Vec<u8> {
        let mut h = [0u8; BLOCK];
        assert!(name.len() <= 100);
        h[..name.len()].copy_from_slice(name.as_bytes());
        h[100..108].copy_from_slice(b"0000644\0");
        h[108..116].copy_from_slice(b"0000000\0");
        h[116..124].copy_from_slice(b"0000000\0");
        let size = format!("{:011o}\0", data.len());
        h[124..136].copy_from_slice(size.as_bytes());
        h[136..148].copy_from_slice(b"00000000000\0");
        h[156] = member_type;
        h[257..263].copy_from_slice(b"ustar\0");
        h[263..265].copy_from_slice(b"00");
        for b in &mut h[148..156] {
            *b = b' ';
        }
        let sum: u64 = h.iter().map(|&b| u64::from(b)).sum();
        let check = format!("{:06o}\0 ", sum);
        h[148..156].copy_from_slice(check.as_bytes());
        let mut bytes = h.to_vec();
        bytes.extend_from_slice(data);
        bytes.resize(BLOCK + data.len().div_ceil(BLOCK) * BLOCK, 0);
        bytes
    }

    fn archive(root: &str) -> Vec<u8> {
        let matrix = b"STEP {\nCOL=1\nNAME=pcb\n}\nLAYER {\nROW=1\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=top\nPOLARITY=POSITIVE\n}\nLAYER {\nROW=2\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=bottom\nPOLARITY=POSITIVE\n}\n";
        let feature = b"UNITS=MM\nF 1\n$0 r100\nL 0 0 1 0 0 P 0\n";
        let mut out = member(&format!("{root}matrix/matrix"), matrix, b'0');
        for layer in ["top", "bottom"] {
            out.extend(member(
                &format!("{root}steps/pcb/layers/{layer}/features"),
                feature,
                b'0',
            ));
        }
        out.extend([0u8; BLOCK * 2]);
        out
    }
    #[test]
    fn tar_to_real_canonical_subset_and_root_normalization() {
        for prefix in ["", "pcb_product/"] {
            let bytes = archive(prefix);
            let files = read_tar(&bytes).unwrap();
            assert!(files.contains_key("matrix/matrix"));
            let board = crate::odb_source::import_unpacked(&files, "pcb").unwrap();
            assert_eq!(board.stackup.layers.len(), 2);
            assert_eq!(board.traces.len(), 2);
            assert!(!board.validate().has_errors());
        }
    }
    #[test]
    fn archive_negative_mutations_fail_closed() {
        let valid = archive("");
        let mut changed = valid.clone();
        changed[0] = b'X';
        assert_eq!(read_tar(&changed).unwrap_err().code, "ODB_TAR_CHECKSUM");
        assert_eq!(
            read_tar(&valid[..valid.len() - BLOCK]).unwrap_err().code,
            "ODB_TAR_MISSING_END"
        );
        let mut link = member("matrix/matrix", b"abc", b'2');
        link.extend([0u8; BLOCK * 2]);
        assert_eq!(
            read_tar(&link).unwrap_err().code,
            "ODB_TAR_UNSUPPORTED_MEMBER"
        );
        let mut traversal = member("../matrix/matrix", b"abc", b'0');
        traversal.extend([0u8; BLOCK * 2]);
        assert_eq!(
            read_tar(&traversal).unwrap_err().code,
            "ODB_TAR_UNSAFE_PATH"
        );
        let mut duplicate = member("matrix/matrix", b"one", b'0');
        duplicate.extend(member("matrix/matrix", b"two", b'0'));
        duplicate.extend([0u8; BLOCK * 2]);
        assert_eq!(
            read_tar(&duplicate).unwrap_err().code,
            "ODB_TAR_DUPLICATE_FILE"
        );
        let mut mixed = valid[..valid.len() - 2 * BLOCK].to_vec();
        mixed.extend(member("second_product/matrix/matrix", b"other", b'0'));
        mixed.extend([0u8; BLOCK * 2]);
        assert_eq!(read_tar(&mixed).unwrap_err().code, "ODB_TAR_MULTIPLE_ROOTS");
        let prefixed = archive("product/");
        let mut nested = prefixed[..prefixed.len() - 2 * BLOCK].to_vec();
        nested.extend(member(
            "product/second_product/matrix/matrix",
            b"other",
            b'0',
        ));
        nested.extend([0u8; BLOCK * 2]);
        assert_eq!(read_tar(&nested).unwrap_err().code, "ODB_TAR_MULTIPLE_ROOTS");
        assert_eq!(
            read_tar(&[0x1f, 0x8b, 8, 0]).unwrap_err().code,
            "ODB_TAR_COMPRESSION_UNSUPPORTED"
        );
    }
}
