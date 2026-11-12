//! E_014: strictly bounded single-member GZIP transport for an ODB++ USTAR product.
//!
//! No filesystem extraction or shell. The complete decoded stream must pass the
//! existing USTAR reader before any source map or Canonical object is published.
//! Optional RFC1952 header metadata is validated and discarded (never a path).
use crate::odb_source::{OdbError, OdbFileMap};
use flate2::{Decompress, FlushDecompress, Status};

const MAX_COMPRESSED: usize = 256 * 1024 * 1024;
const MAX_TAR: usize = 256 * 1024 * 1024;
const MAX_HEADER: usize = 64 * 1024;

fn bad(code: &'static str) -> OdbError {
    OdbError {
        path: "product.tar.gz".into(),
        line: 0,
        code,
    }
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320u32 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

/// Parse only the RFC1952 header. Do not use FNAME/FCOMMENT as filesystem paths.
/// Bound variable-length metadata and verify FHCRC when present.
fn payload_start(bytes: &[u8]) -> Result<usize, OdbError> {
    if bytes.len() < 18 || bytes.len() > MAX_COMPRESSED {
        return Err(bad("ODB_GZIP_SIZE"));
    }
    if bytes[..2] != [0x1f, 0x8b] || bytes[2] != 8 {
        return Err(bad("ODB_GZIP_HEADER"));
    }
    let flags = bytes[3];
    if flags & 0xe0 != 0 {
        return Err(bad("ODB_GZIP_RESERVED_FLAGS"));
    }
    // The final eight bytes are the required CRC32 and ISIZE trailer.
    let trailer = bytes.len() - 8;
    let mut at = 10usize;
    let header_limit = trailer.min(MAX_HEADER);
    if at > header_limit {
        return Err(bad("ODB_GZIP_TRUNCATED_HEADER"));
    }
    if flags & 0x04 != 0 {
        let end = at
            .checked_add(2)
            .ok_or_else(|| bad("ODB_GZIP_HEADER_LIMIT"))?;
        if end > header_limit {
            return Err(bad("ODB_GZIP_TRUNCATED_HEADER"));
        }
        let xlen = usize::from(u16::from_le_bytes([bytes[at], bytes[at + 1]]));
        at = end
            .checked_add(xlen)
            .ok_or_else(|| bad("ODB_GZIP_HEADER_LIMIT"))?;
        if at > header_limit {
            return Err(bad("ODB_GZIP_HEADER_LIMIT"));
        }
    }
    for flag in [0x08, 0x10] {
        if flags & flag != 0 {
            // FNAME and FCOMMENT are NUL-terminated; never scan past 64 KiB.
            loop {
                if at >= header_limit {
                    return Err(bad("ODB_GZIP_UNTERMINATED_HEADER"));
                }
                let value = bytes[at];
                at += 1;
                if value == 0 {
                    break;
                }
            }
        }
    }
    if flags & 0x02 != 0 {
        let end = at
            .checked_add(2)
            .ok_or_else(|| bad("ODB_GZIP_HEADER_LIMIT"))?;
        if end > header_limit {
            return Err(bad("ODB_GZIP_TRUNCATED_HEADER"));
        }
        let claimed = u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        let calculated = crc32(&bytes[..at]) as u16;
        if claimed != calculated {
            return Err(bad("ODB_GZIP_HEADER_CRC"));
        }
        at = end;
    }
    if at >= trailer {
        return Err(bad("ODB_GZIP_MISSING_STREAM"));
    }
    Ok(at)
}

/// Bounded single GZIP member decoder. Require both DEFLATE EOS and consumption
/// of all payload bytes; this deliberately rejects trailing data and concatenated
/// members instead of silently importing just the first ODB++ board.
fn inflate_gzip_with_limit(bytes: &[u8], maximum: usize) -> Result<Vec<u8>, OdbError> {
    let start = payload_start(bytes)?;
    let trailer = bytes.len() - 8;
    let packed = &bytes[start..trailer];
    let mut decoder = Decompress::new(false); // RFC1952 embeds *raw* DEFLATE.
    let mut out = Vec::new();
    let mut offset = 0usize;
    let mut chunk = [0u8; 8192];
    loop {
        let before_in = decoder.total_in();
        let before_out = decoder.total_out();
        let status = decoder
            .decompress(&packed[offset..], &mut chunk, FlushDecompress::None)
            .map_err(|_| bad("ODB_GZIP_DEFLATE_INVALID"))?;
        let consumed = usize::try_from(decoder.total_in() - before_in)
            .map_err(|_| bad("ODB_GZIP_SIZE_OVERFLOW"))?;
        let produced = usize::try_from(decoder.total_out() - before_out)
            .map_err(|_| bad("ODB_GZIP_SIZE_OVERFLOW"))?;
        offset = offset
            .checked_add(consumed)
            .ok_or_else(|| bad("ODB_GZIP_SIZE_OVERFLOW"))?;
        let end = out
            .len()
            .checked_add(produced)
            .ok_or_else(|| bad("ODB_GZIP_SIZE_OVERFLOW"))?;
        if end > maximum || end > MAX_TAR {
            return Err(bad("ODB_GZIP_EXPANSION_LIMIT"));
        }
        out.extend_from_slice(&chunk[..produced]);
        if status == Status::StreamEnd {
            if offset != packed.len() {
                return Err(bad("ODB_GZIP_TRAILING_OR_MULTIMEMBER"));
            }
            break;
        }
        if consumed == 0 && produced == 0 {
            return Err(bad("ODB_GZIP_TRUNCATED_OR_STALLED"));
        }
    }
    let expected_crc = u32::from_le_bytes(bytes[trailer..trailer + 4].try_into().unwrap());
    let expected_size = u32::from_le_bytes(bytes[trailer + 4..].try_into().unwrap());
    if crc32(&out) != expected_crc {
        return Err(bad("ODB_GZIP_CRC"));
    }
    // The limit is 256 MiB, so RFC1952 ISIZE modulo 2^32 is unambiguous.
    if usize::try_from(expected_size).ok() != Some(out.len()) {
        return Err(bad("ODB_GZIP_ISIZE"));
    }
    Ok(out)
}

/// Decode a complete bounded GZIP stream and reuse the *existing* strict USTAR
/// parser (checksum, path, entry, duplicate, roots and Source FileMap limits).
pub(crate) fn read_gzip_tar(bytes: &[u8]) -> Result<OdbFileMap, OdbError> {
    let tar = inflate_gzip_with_limit(bytes, MAX_TAR)?;
    crate::odb_tar::read_tar(&tar)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const MATRIX: &[u8] = b"STEP {\nCOL=1\nNAME=pcb\n}\nLAYER {\nROW=1\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=top\nPOLARITY=POSITIVE\n}\nLAYER {\nROW=2\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=bottom\nPOLARITY=POSITIVE\n}\n";
    const COPPER: &[u8] = b"UNITS=MM\nF 1\n$0 r100\nL 0 0 1 0 0 P 0\n";

    fn files() -> [(&'static str, &'static [u8]); 3] {
        [
            ("product/matrix/matrix", MATRIX),
            ("product/steps/pcb/layers/top/features", COPPER),
            ("product/steps/pcb/layers/bottom/features", COPPER),
        ]
    }

    fn tar(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        for &(name, data) in entries {
            assert!(name.len() <= 100);
            let mut header = [0u8; 512];
            header[..name.len()].copy_from_slice(name.as_bytes());
            header[100..108].copy_from_slice(b"0000644\0");
            header[108..116].copy_from_slice(b"0000000\0");
            header[116..124].copy_from_slice(b"0000000\0");
            header[124..136].copy_from_slice(format!("{:011o}\0", data.len()).as_bytes());
            header[136..148].copy_from_slice(b"00000000000\0");
            header[148..156].fill(b' ');
            header[156] = b'0';
            header[257..263].copy_from_slice(b"ustar\0");
            header[263..265].copy_from_slice(b"00");
            let sum: u64 = header.iter().map(|&b| u64::from(b)).sum();
            header[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
            out.extend_from_slice(&header);
            out.extend_from_slice(data);
            out.resize((out.len() + 511) / 512 * 512, 0);
        }
        out.extend_from_slice(&[0u8; 1024]);
        out
    }

    fn gzip(input: &[u8]) -> Vec<u8> {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(input).unwrap();
        encoder.finish().unwrap()
    }

    fn sample_tar() -> Vec<u8> {
        tar(&files())
    }

    fn zip_methods(entries: &[(&str, &[u8])], method: u16) -> Vec<u8> {
        assert!(method == 0 || method == 8);
        fn put16(v: &mut Vec<u8>, n: u16) {
            v.extend_from_slice(&n.to_le_bytes());
        }
        fn put32(v: &mut Vec<u8>, n: u32) {
            v.extend_from_slice(&n.to_le_bytes());
        }
        let mut out = Vec::new();
        let mut central = Vec::new();
        for &(name, data) in entries {
            let packed = if method == 8 {
                let mut encoder =
                    flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
                encoder.write_all(data).unwrap();
                encoder.finish().unwrap()
            } else {
                data.to_vec()
            };
            let offset = out.len() as u32;
            let version = if method == 8 { 20 } else { 10 };
            let crc = crc32(data);
            put32(&mut out, 0x0403_4b50);
            put16(&mut out, version);
            put16(&mut out, 0);
            put16(&mut out, method);
            put16(&mut out, 0);
            put16(&mut out, 0);
            put32(&mut out, crc);
            put32(&mut out, packed.len() as u32);
            put32(&mut out, data.len() as u32);
            put16(&mut out, name.len() as u16);
            put16(&mut out, 0);
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(&packed);

            put32(&mut central, 0x0201_4b50);
            put16(&mut central, version);
            put16(&mut central, version);
            put16(&mut central, 0);
            put16(&mut central, method);
            put16(&mut central, 0);
            put16(&mut central, 0);
            put32(&mut central, crc);
            put32(&mut central, packed.len() as u32);
            put32(&mut central, data.len() as u32);
            put16(&mut central, name.len() as u16);
            put16(&mut central, 0);
            put16(&mut central, 0);
            put16(&mut central, 0);
            put16(&mut central, 0);
            put32(&mut central, 0);
            put32(&mut central, offset);
            central.extend_from_slice(name.as_bytes());
        }
        let start = out.len() as u32;
        out.extend_from_slice(&central);
        put32(&mut out, 0x0605_4b50);
        put16(&mut out, 0);
        put16(&mut out, 0);
        put16(&mut out, entries.len() as u16);
        put16(&mut out, entries.len() as u16);
        put32(&mut out, central.len() as u32);
        put32(&mut out, start);
        put16(&mut out, 0);
        out
    }

    #[test]
    fn gzip_tar_matches_directory_and_both_zip_methods() {
        let input = sample_tar();
        let gz_map = read_gzip_tar(&gzip(&input)).unwrap();
        let tar_map = crate::odb_tar::read_tar(&input).unwrap();
        assert_eq!(gz_map, tar_map);
        for method in [0u16, 8u16] {
            let zip_map = crate::odb_zip::read_zip(&zip_methods(&files(), method)).unwrap();
            assert_eq!(gz_map, zip_map);
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("boardsi-e014-dir-{}-{stamp}", std::process::id()));
        for (path, contents) in files() {
            let relative = path.strip_prefix("product/").unwrap();
            let target = root.join(relative);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::write(target, contents).unwrap();
        }
        let directory_board = crate::odb_source::import_directory(&root, "pcb").unwrap();
        let gzip_board = crate::odb_source::import_unpacked(&gz_map, "pcb").unwrap();
        assert_eq!(directory_board.traces.len(), gzip_board.traces.len());
        assert_eq!(
            directory_board.sources[0].id.files,
            gzip_board.sources[0].id.files
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn gzip_matches_uncompressed_ustar_and_canonical_source() {
        let plain = sample_tar();
        let gz = gzip(&plain);
        let map = read_gzip_tar(&gz).unwrap();
        assert_eq!(map, crate::odb_tar::read_tar(&plain).unwrap());
        assert_eq!(map.len(), 3);
        let board = crate::odb_source::import_unpacked(&map, "pcb").unwrap();
        assert_eq!(board.stackup.layers.len(), 2);
        assert_eq!(board.traces.len(), 2);
        assert!(!board.validate().has_errors());
        assert!(board.connectivity.edges.is_empty());
    }

    #[test]
    fn real_archive_paths_and_drill_source_audit_are_shared() {
        let archive = gzip(&sample_tar());
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        for ext in ["tar.gz", "tgz"] {
            let path = std::env::temp_dir()
                .join(format!("boardsi-e014-{}-{stamp}.{ext}", std::process::id()));
            std::fs::write(&path, &archive).unwrap();
            let result = (|| -> Result<(), OdbError> {
                let board = crate::odb_source::import_archive_file(&path, "pcb", false)?;
                let source = crate::odb_fabrication::import_path_with_holes(&path, "pcb", false)?;
                let audit = crate::odb_source::audit_path_drill_source(&path, "pcb", true)?;
                assert_eq!(board.traces.len(), 2);
                assert_eq!(board.sources[0].id.files, source.board.sources[0].id.files);
                assert!(audit.layers.is_empty());
                assert!(audit.hits.as_ref().is_some_and(Vec::is_empty));
                Ok(())
            })();
            std::fs::remove_file(&path).unwrap();
            result.unwrap();
        }
    }

    #[test]
    fn rfc1952_optional_fields_and_fhcrc_are_validated() {
        let input = sample_tar();
        let original = gzip(&input);
        let mut extra = original[..10].to_vec();
        extra[3] = 0x1e; // FEXTRA + FNAME + FCOMMENT + FHCRC
        extra.extend_from_slice(&[2, 0, 0xaa, 0xbb]);
        extra.extend_from_slice(b"board.tar\0comment\0");
        let header_crc = crc32(&extra) as u16;
        extra.extend_from_slice(&header_crc.to_le_bytes());
        extra.extend_from_slice(&original[10..]);
        assert_eq!(
            read_gzip_tar(&extra).unwrap(),
            crate::odb_tar::read_tar(&input).unwrap()
        );
        extra[18] ^= 0x01;
        assert_eq!(
            read_gzip_tar(&extra).unwrap_err().code,
            "ODB_GZIP_HEADER_CRC"
        );
    }

    #[test]
    fn independent_python_gzip_vector_and_integrity() {
        // Python 3: gzip.compress(b"Hello ODB++ gzip!\n", mtime=0)
        const PYTHON_GZIP: [u8; 38] = [
            0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xff, 0xf3, 0x48, 0xcd, 0xc9,
            0xc9, 0x57, 0xf0, 0x77, 0x71, 0xd2, 0xd6, 0x56, 0x48, 0xaf, 0xca, 0x2c, 0x50, 0xe4,
            0x02, 0x00, 0xfa, 0x4d, 0x39, 0xb0, 0x12, 0x00, 0x00, 0x00,
        ];
        assert_eq!(
            inflate_gzip_with_limit(&PYTHON_GZIP, MAX_TAR).unwrap(),
            b"Hello ODB++ gzip!\n"
        );
    }

    #[test]
    fn corruption_truncation_size_bomb_and_multiple_member_fail_closed() {
        let gz = gzip(&sample_tar());
        let mut bad_flags = gz.clone();
        bad_flags[3] |= 0x80;
        assert_eq!(
            read_gzip_tar(&bad_flags).unwrap_err().code,
            "ODB_GZIP_RESERVED_FLAGS"
        );
        let mut bad_crc = gz.clone();
        let at = bad_crc.len() - 8;
        bad_crc[at] ^= 1;
        assert_eq!(read_gzip_tar(&bad_crc).unwrap_err().code, "ODB_GZIP_CRC");
        let mut bad_size = gz.clone();
        let at = bad_size.len() - 4;
        bad_size[at] ^= 1;
        assert_eq!(read_gzip_tar(&bad_size).unwrap_err().code, "ODB_GZIP_ISIZE");
        for n in [0, 1, 9, 10, gz.len() - 1, gz.len() - 3] {
            assert!(read_gzip_tar(&gz[..n]).is_err());
        }
        let mut concat = gz.clone();
        concat.extend_from_slice(&gz);
        assert!(read_gzip_tar(&concat).is_err());
        let mut trailing = gz.clone();
        trailing.extend_from_slice(b"garbage");
        assert!(read_gzip_tar(&trailing).is_err());
        let huge = gzip(&vec![b'0'; 4096]);
        assert_eq!(
            inflate_gzip_with_limit(&huge, 64).unwrap_err().code,
            "ODB_GZIP_EXPANSION_LIMIT"
        );
        assert!(read_gzip_tar(&gzip(b"not a tar")).is_err());
    }

    #[test]
    fn unsafe_tar_content_is_rejected_after_valid_gzip() {
        let bad_path = tar(&[("../matrix/matrix", MATRIX)]);
        assert!(read_gzip_tar(&gzip(&bad_path)).is_err());
        let duplicate = tar(&[
            ("product/matrix/matrix", MATRIX),
            ("product/matrix/matrix", MATRIX),
        ]);
        assert!(read_gzip_tar(&gzip(&duplicate)).is_err());
        let nested_root = tar(&[
            ("left/matrix/matrix", MATRIX),
            ("right/matrix/matrix", MATRIX),
        ]);
        assert!(read_gzip_tar(&gzip(&nested_root)).is_err());
        let mut broken_tar = sample_tar();
        broken_tar[0] ^= 1; // invalidate TAR checksum
        assert!(read_gzip_tar(&gzip(&broken_tar)).is_err());
    }
}
