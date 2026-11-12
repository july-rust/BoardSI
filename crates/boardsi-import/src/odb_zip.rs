//! D_056_09 / E_013: strictly bounded ZIP STORED/DEFLATE ODB++ transport.
//!
//! Only methods 0 and 8 are supported; data descriptors, Zip64, split disks,
//! encryption, extra fields and executable/symlink members remain unsupported.
//! ZIP is only a transport; Canonical source/electrical qualification remains
//! in the existing ODB++ importer and cannot be inferred from extraction.
use crate::odb_source::{OdbError, OdbFileMap};
use std::collections::BTreeSet;
use flate2::{Decompress, FlushDecompress, Status};

const MAX_ARCHIVE: usize = 256 * 1024 * 1024;
const MAX_FILE: usize = 16 * 1024 * 1024;
const MAX_TOTAL: usize = 128 * 1024 * 1024;
const MAX_ENTRIES: usize = 8192;
const LOCAL: usize = 0x0403_4b50;
const CENTRAL: usize = 0x0201_4b50;
const END: usize = 0x0605_4b50;

fn bad(code: &'static str) -> OdbError {
    OdbError {
        path: "product.zip".into(),
        line: 0,
        code,
    }
}

fn u16_at(bytes: &[u8], offset: usize) -> usize {
    usize::from(u16::from_le_bytes([bytes[offset], bytes[offset + 1]]))
}

fn u32_at(bytes: &[u8], offset: usize) -> usize {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]) as usize
}

fn checked_end(start: usize, n: usize, maximum: usize) -> Result<usize, OdbError> {
    let end = start
        .checked_add(n)
        .ok_or_else(|| bad("ODB_ZIP_SIZE_OVERFLOW"))?;
    if end > maximum {
        return Err(bad("ODB_ZIP_TRUNCATED"));
    }
    Ok(end)
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let bit = crc & 1;
            crc = (crc >> 1) ^ (0xedb8_8320u32 & (0u32.wrapping_sub(bit)));
        }
    }
    !crc
}


fn inflate_raw_bounded(compressed: &[u8], declared_size: usize) -> Result<Vec<u8>, OdbError> {
    if declared_size > MAX_FILE {
        return Err(bad("ODB_ZIP_FILE_LIMIT"));
    }
    // ZIP method 8 is raw DEFLATE (no zlib header / checksum). Never decode
    // an entire untrusted entry first and check its length afterwards.
    let mut decoder = Decompress::new(false);
    let mut result = Vec::with_capacity(declared_size.min(16 * 1024));
    let mut input_offset = 0usize;
    let mut chunk = [0u8; 8192];
    loop {
        let previous_in = decoder.total_in();
        let previous_out = decoder.total_out();
        let status = decoder
            .decompress(&compressed[input_offset..], &mut chunk, FlushDecompress::None)
            .map_err(|_| bad("ODB_ZIP_DEFLATE_INVALID"))?;
        let consumed = usize::try_from(decoder.total_in() - previous_in)
            .map_err(|_| bad("ODB_ZIP_SIZE_OVERFLOW"))?;
        let produced = usize::try_from(decoder.total_out() - previous_out)
            .map_err(|_| bad("ODB_ZIP_SIZE_OVERFLOW"))?;
        input_offset = input_offset
            .checked_add(consumed)
            .ok_or_else(|| bad("ODB_ZIP_SIZE_OVERFLOW"))?;
        let end = result
            .len()
            .checked_add(produced)
            .ok_or_else(|| bad("ODB_ZIP_SIZE_OVERFLOW"))?;
        if end > declared_size || end > MAX_FILE {
            return Err(bad("ODB_ZIP_DEFLATE_SIZE_MISMATCH"));
        }
        result.extend_from_slice(&chunk[..produced]);
        if status == Status::StreamEnd {
            if input_offset != compressed.len() {
                return Err(bad("ODB_ZIP_DEFLATE_TRAILING_DATA"));
            }
            if result.len() != declared_size {
                return Err(bad("ODB_ZIP_DEFLATE_SIZE_MISMATCH"));
            }
            return Ok(result);
        }
        if consumed == 0 && produced == 0 {
            return Err(bad("ODB_ZIP_DEFLATE_INVALID"));
        }
    }
}

/// Read a single-disk, zero-comment ZIP with STORED (0) / DEFLATE (8) members.
/// Cross-check every central-directory record against its local header and
/// CRC before making any file visible to the Canonical importer.
pub fn read_zip(bytes: &[u8]) -> Result<OdbFileMap, OdbError> {
    if bytes.len() < 22 || bytes.len() > MAX_ARCHIVE {
        return Err(bad("ODB_ZIP_SIZE"));
    }
    // No trailing bytes or comments: avoid searching for a spoofed EOCD.
    let eocd = bytes.len() - 22;
    if u32_at(bytes, eocd) != END {
        return Err(bad("ODB_ZIP_END_MISSING"));
    }
    if u16_at(bytes, eocd + 4) != 0 || u16_at(bytes, eocd + 6) != 0 || u16_at(bytes, eocd + 20) != 0
    {
        return Err(bad("ODB_ZIP_MULTIDISK_OR_COMMENT_UNSUPPORTED"));
    }
    let entries = u16_at(bytes, eocd + 10);
    if entries == 0
        || entries > MAX_ENTRIES
        || u16_at(bytes, eocd + 8) != entries
        || entries == 0xffff
    {
        return Err(bad("ODB_ZIP_ENTRY_LIMIT_OR_SPANNED"));
    }
    let central_size = u32_at(bytes, eocd + 12);
    let central_start = u32_at(bytes, eocd + 16);
    if central_start == 0xffff_ffff
        || central_size == 0xffff_ffff
        || checked_end(central_start, central_size, eocd)? != eocd
    {
        return Err(bad("ODB_ZIP_BAD_CENTRAL_DIRECTORY"));
    }

    let mut cursor = central_start;
    let mut total = 0usize;
    let mut raw = OdbFileMap::new();
    let mut occupied = Vec::with_capacity(entries);
    let mut directory_names = BTreeSet::new();
    for _ in 0..entries {
        if checked_end(cursor, 46, eocd)? > eocd || u32_at(bytes, cursor) != CENTRAL {
            return Err(bad("ODB_ZIP_BAD_CENTRAL_DIRECTORY"));
        }
        let creator = u16_at(bytes, cursor + 4);
        let version = u16_at(bytes, cursor + 6);
        let flags = u16_at(bytes, cursor + 8);
        let method = u16_at(bytes, cursor + 10);
        let expected_crc = u32_at(bytes, cursor + 16) as u32;
        let compressed = u32_at(bytes, cursor + 20);
        let uncompressed = u32_at(bytes, cursor + 24);
        let name_len = u16_at(bytes, cursor + 28);
        let extra_len = u16_at(bytes, cursor + 30);
        let comment_len = u16_at(bytes, cursor + 32);
        let disk = u16_at(bytes, cursor + 34);
        let external = u32_at(bytes, cursor + 38) as u32;
        let local_offset = u32_at(bytes, cursor + 42);
        let central_end = checked_end(
            cursor,
            46usize
                .checked_add(name_len)
                .and_then(|n| n.checked_add(extra_len))
                .and_then(|n| n.checked_add(comment_len))
                .ok_or_else(|| bad("ODB_ZIP_SIZE_OVERFLOW"))?,
            eocd,
        )?;
        if version > 20
            || flags & !0x0800 != 0
            || extra_len != 0
            || comment_len != 0
            || disk != 0
            || local_offset == 0xffff_ffff
        {
            return Err(bad("ODB_ZIP_UNSUPPORTED_EXTENSION"));
        }
        if method != 0 && method != 8 {
            return Err(bad("ODB_ZIP_COMPRESSION_UNSUPPORTED"));
        }
        if uncompressed > MAX_FILE || (method == 0 && compressed != uncompressed) {
            return Err(bad("ODB_ZIP_FILE_LIMIT"));
        }
        let name_bytes = &bytes[cursor + 46..cursor + 46 + name_len];
        let name = std::str::from_utf8(name_bytes).map_err(|_| bad("ODB_ZIP_NONUTF8_PATH"))?;
        let is_directory = name.ends_with('/');
        let clean = if is_directory {
            &name[..name.len() - 1]
        } else {
            name
        };
        if !crate::odb_tar::safe_name(clean) {
            return Err(bad("ODB_ZIP_UNSAFE_PATH"));
        }

        // A UNIX-origin ZIP must not hide symlinks or device nodes.
        if creator >> 8 == 3 {
            let file_type = (external >> 16) & 0o170000;
            let wanted = if is_directory { 0o040000 } else { 0o100000 };
            if file_type != 0 && file_type != wanted {
                return Err(bad("ODB_ZIP_UNSUPPORTED_MEMBER"));
            }
        }

        let local_min = checked_end(local_offset, 30, central_start)?;
        if local_min > central_start || u32_at(bytes, local_offset) != LOCAL {
            return Err(bad("ODB_ZIP_BAD_LOCAL_HEADER"));
        }
        let local_flags = u16_at(bytes, local_offset + 6);
        let local_method = u16_at(bytes, local_offset + 8);
        let local_name_len = u16_at(bytes, local_offset + 26);
        let local_extra = u16_at(bytes, local_offset + 28);
        if local_flags != flags
            || local_method != method
            || local_extra != 0
            || local_name_len != name_len
            || u32_at(bytes, local_offset + 14) as u32 != expected_crc
            || u32_at(bytes, local_offset + 18) != compressed
            || u32_at(bytes, local_offset + 22) != uncompressed
        {
            return Err(bad("ODB_ZIP_HEADER_MISMATCH"));
        }
        let data_start = checked_end(local_offset + 30, local_name_len, central_start)?;
        if bytes[local_offset + 30..data_start] != *name_bytes {
            return Err(bad("ODB_ZIP_NAME_MISMATCH"));
        }
        let data_end = checked_end(data_start, compressed, central_start)?;
        if !is_directory && total.checked_add(uncompressed).map_or(true, |n| n > MAX_TOTAL) {
            return Err(bad("ODB_ZIP_TOTAL_LIMIT"));
        }
        let payload = &bytes[data_start..data_end];
        let decoded = if method == 8 {
            inflate_raw_bounded(payload, uncompressed)?
        } else {
            payload.to_vec()
        };
        if crc32(&decoded) != expected_crc {
            return Err(bad("ODB_ZIP_CRC32_MISMATCH"));
        }
        if is_directory {
            if !decoded.is_empty() || !directory_names.insert(clean.to_owned()) {
                return Err(bad("ODB_ZIP_DUPLICATE_OR_NONEMPTY_DIRECTORY"));
            }
        } else {
            total += decoded.len(); // checked against MAX_TOTAL before decompression
            if raw.insert(name.to_owned(), decoded).is_some() {
                return Err(bad("ODB_ZIP_TOTAL_OR_DUPLICATE_FILE"));
            }
        }
        occupied.push((local_offset, data_end));
        cursor = central_end;
    }
    if cursor != eocd || raw.is_empty() {
        return Err(bad("ODB_ZIP_CENTRAL_TRAILING_DATA"));
    }
    // Disallow overlapping, aliased or unlisted local entries, prefixes and
    // hidden payload between local data and the central directory.
    occupied.sort_unstable();
    let mut next = 0usize;
    for (start, end) in occupied {
        if start != next {
            return Err(bad("ODB_ZIP_LOCAL_LAYOUT"));
        }
        next = end;
    }
    if next != central_start {
        return Err(bad("ODB_ZIP_LOCAL_LAYOUT"));
    }

    crate::odb_tar::normalize_product_files(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push16(v: &mut Vec<u8>, x: u16) {
        v.extend_from_slice(&x.to_le_bytes());
    }
    fn push32(v: &mut Vec<u8>, x: u32) {
        v.extend_from_slice(&x.to_le_bytes());
    }

    fn zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut central = Vec::new();
        for &(name, data) in entries {
            let crc = crc32(data);
            let offset = out.len() as u32;
            push32(&mut out, LOCAL as u32);
            push16(&mut out, 10);
            push16(&mut out, 0);
            push16(&mut out, 0);
            push16(&mut out, 0);
            push16(&mut out, 0);
            push32(&mut out, crc);
            push32(&mut out, data.len() as u32);
            push32(&mut out, data.len() as u32);
            push16(&mut out, name.len() as u16);
            push16(&mut out, 0);
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(data);

            push32(&mut central, CENTRAL as u32);
            push16(&mut central, 10);
            push16(&mut central, 10);
            push16(&mut central, 0);
            push16(&mut central, 0);
            push16(&mut central, 0);
            push16(&mut central, 0);
            push32(&mut central, crc);
            push32(&mut central, data.len() as u32);
            push32(&mut central, data.len() as u32);
            push16(&mut central, name.len() as u16);
            push16(&mut central, 0);
            push16(&mut central, 0);
            push16(&mut central, 0);
            push16(&mut central, 0);
            push32(&mut central, 0);
            push32(&mut central, offset);
            central.extend_from_slice(name.as_bytes());
        }
        let start = out.len() as u32;
        out.extend_from_slice(&central);
        push32(&mut out, END as u32);
        push16(&mut out, 0);
        push16(&mut out, 0);
        push16(&mut out, entries.len() as u16);
        push16(&mut out, entries.len() as u16);
        push32(&mut out, central.len() as u32);
        push32(&mut out, start);
        push16(&mut out, 0);
        out
    }


    /// Test fixture generator uses the standard DEFLATE encoder. The decoder
    /// also has an independent, precomputed Python-zlib raw-DEFLATE vector.
    fn zip_methods(entries: &[(&str, &[u8], u16)]) -> Vec<u8> {
        use std::io::Write;
        let mut out = Vec::new();
        let mut central = Vec::new();
        for &(name, data, method) in entries {
            assert!(method == 0 || method == 8);
            let packed = if method == 8 {
                let mut encoder =
                    flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
                encoder.write_all(data).unwrap();
                encoder.finish().unwrap()
            } else {
                data.to_vec()
            };
            let crc = crc32(data);
            let offset = out.len() as u32;
            let version = if method == 8 { 20 } else { 10 };
            push32(&mut out, LOCAL as u32);
            push16(&mut out, version);
            push16(&mut out, 0);
            push16(&mut out, method);
            push16(&mut out, 0);
            push16(&mut out, 0);
            push32(&mut out, crc);
            push32(&mut out, packed.len() as u32);
            push32(&mut out, data.len() as u32);
            push16(&mut out, name.len() as u16);
            push16(&mut out, 0);
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(&packed);

            push32(&mut central, CENTRAL as u32);
            push16(&mut central, version);
            push16(&mut central, version);
            push16(&mut central, 0);
            push16(&mut central, method);
            push16(&mut central, 0);
            push16(&mut central, 0);
            push32(&mut central, crc);
            push32(&mut central, packed.len() as u32);
            push32(&mut central, data.len() as u32);
            push16(&mut central, name.len() as u16);
            push16(&mut central, 0);
            push16(&mut central, 0);
            push16(&mut central, 0);
            push16(&mut central, 0);
            push32(&mut central, 0);
            push32(&mut central, offset);
            central.extend_from_slice(name.as_bytes());
        }
        let start = out.len() as u32;
        out.extend_from_slice(&central);
        push32(&mut out, END as u32);
        push16(&mut out, 0);
        push16(&mut out, 0);
        push16(&mut out, entries.len() as u16);
        push16(&mut out, entries.len() as u16);
        push32(&mut out, central.len() as u32);
        push32(&mut out, start);
        push16(&mut out, 0);
        out
    }

    fn zip_source(methods: [u16; 3]) -> Vec<u8> {
        let matrix = b"STEP {\nCOL=1\nNAME=pcb\n}\nLAYER {\nROW=1\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=top\nPOLARITY=POSITIVE\n}\nLAYER {\nROW=2\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=bottom\nPOLARITY=POSITIVE\n}\n";
        let copper = b"UNITS=MM\nF 1\n$0 r100\nL 0 0 1 0 0 P 0\n";
        zip_methods(&[
            ("product/matrix/matrix", matrix, methods[0]),
            ("product/steps/pcb/layers/top/features", copper, methods[1]),
            ("product/steps/pcb/layers/bottom/features", copper, methods[2]),
        ])
    }

    fn example() -> Vec<u8> {
        let matrix = b"STEP {\nCOL=1\nNAME=pcb\n}\nLAYER {\nROW=1\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=top\nPOLARITY=POSITIVE\n}\nLAYER {\nROW=2\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=bottom\nPOLARITY=POSITIVE\n}\n";
        let copper = b"UNITS=MM\nF 1\n$0 r100\nL 0 0 1 0 0 P 0\n";
        zip(&[
            ("product/matrix/matrix", matrix),
            ("product/steps/pcb/layers/top/features", copper),
            ("product/steps/pcb/layers/bottom/features", copper),
        ])
    }

    #[test]
    fn stored_zip_to_canonical_has_source_geometry() {
        let archive = example();
        let map = read_zip(&archive).unwrap();
        assert_eq!(map.len(), 3);
        assert!(map.contains_key("matrix/matrix"));
        let board = crate::odb_source::import_unpacked(&map, "pcb").unwrap();
        assert_eq!(board.stackup.layers.len(), 2);
        assert_eq!(board.traces.len(), 2);
        assert!(!board.validate().has_errors());
        assert!(board.connectivity.edges.is_empty());
    }

    #[test]
    fn stored_zip_path_reuses_canonical_and_source_drill_snapshot() {
        let archive = example();
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("boardsi-d05609-{}-{stamp}.zip", std::process::id()));
        std::fs::write(&path, archive).unwrap();
        let checked = (|| -> Result<(), crate::odb_source::OdbError> {
            let board = crate::odb_source::import_archive_file(&path, "pcb", false)?;
            let source = crate::odb_fabrication::import_path_with_holes(&path, "pcb", false)?;
            let audit = crate::odb_source::audit_path_drill_source(&path, "pcb", true)?;
            assert_eq!(board.traces.len(), 2);
            assert_eq!(board.stackup.layers.len(), 2);
            assert_eq!(board.sources[0].id.files, source.board.sources[0].id.files);
            assert!(source.source_holes.is_empty());
            assert!(audit.layers.is_empty());
            assert!(audit.hits.unwrap().is_empty());
            Ok(())
        })();
        std::fs::remove_file(&path).unwrap();
        checked.unwrap();
    }

    #[test]
    fn stored_zip_rejects_invalid_crc_and_unsafe_inputs() {
        let valid = example();
        let mut corrupt = valid.clone();
        // Flip one matrix data byte while keeping headers unchanged.
        let start = corrupt
            .windows(b"STEP".len())
            .position(|w| w == b"STEP")
            .unwrap();
        corrupt[start] ^= 1;
        assert_eq!(
            read_zip(&corrupt).unwrap_err().code,
            "ODB_ZIP_CRC32_MISMATCH"
        );

        let duplicate = zip(&[("matrix/matrix", b"a"), ("matrix/matrix", b"b")]);
        assert_eq!(
            read_zip(&duplicate).unwrap_err().code,
            "ODB_ZIP_TOTAL_OR_DUPLICATE_FILE"
        );
        let traversal = zip(&[("../matrix/matrix", b"a")]);
        assert_eq!(
            read_zip(&traversal).unwrap_err().code,
            "ODB_ZIP_UNSAFE_PATH"
        );

        let mut compressed = valid.clone();
        // First local compression method and its central-directory twin.
        compressed[8] = 8;
        let eocd = compressed.len() - 22;
        let cd = u32_at(&compressed, eocd + 16);
        compressed[cd + 10] = 8;
        assert!(matches!(
            read_zip(&compressed).unwrap_err().code,
            "ODB_ZIP_DEFLATE_INVALID"
                | "ODB_ZIP_DEFLATE_SIZE_MISMATCH"
                | "ODB_ZIP_DEFLATE_TRAILING_DATA"
        ));

        let mut trailing = valid.clone();
        trailing.push(0);
        assert_eq!(read_zip(&trailing).unwrap_err().code, "ODB_ZIP_END_MISSING");
        let mut local_mismatch = valid.clone();
        local_mismatch[0] = b'X';
        assert_eq!(
            read_zip(&local_mismatch).unwrap_err().code,
            "ODB_ZIP_BAD_LOCAL_HEADER"
        );
        let mut descriptor = valid.clone();
        descriptor[6] = 8;
        let eocd = descriptor.len() - 22;
        let cd = u32_at(&descriptor, eocd + 16);
        descriptor[cd + 8] = 8;
        assert_eq!(
            read_zip(&descriptor).unwrap_err().code,
            "ODB_ZIP_UNSUPPORTED_EXTENSION"
        );
    }

    #[test]
    fn stored_zip_rejects_multiple_roots_and_unsupported_unix_symlinks() {
        let mixed = zip(&[
            ("product/matrix/matrix", b"a"),
            ("other/matrix/matrix", b"b"),
        ]);
        assert_eq!(read_zip(&mixed).unwrap_err().code, "ODB_TAR_MULTIPLE_ROOTS");
        let mut link = zip(&[("matrix/matrix", b"a")]);
        let eocd = link.len() - 22;
        let cd = u32_at(&link, eocd + 16);
        // Source platform Unix, symlink mode 0120000 in external attributes.
        link[cd + 5] = 3;
        link[cd + 40..cd + 42].copy_from_slice(&(0o120000u16).to_le_bytes());
        assert_eq!(
            read_zip(&link).unwrap_err().code,
            "ODB_ZIP_UNSUPPORTED_MEMBER"
        );
    }

    #[test]
    fn independent_raw_deflate_oracle_and_exact_stream_consumption() {
        // CPython zlib.compressobj(wbits=-15) on b"hello": cb48cdc9c90700.
        let reference = [0xcb, 0x48, 0xcd, 0xc9, 0xc9, 0x07, 0x00];
        assert_eq!(inflate_raw_bounded(&reference, 5).unwrap(), b"hello");
        assert_eq!(
            inflate_raw_bounded(&reference, 4).unwrap_err().code,
            "ODB_ZIP_DEFLATE_SIZE_MISMATCH"
        );
        assert_eq!(
            inflate_raw_bounded(&reference, 6).unwrap_err().code,
            "ODB_ZIP_DEFLATE_SIZE_MISMATCH"
        );
        let mut trailing = reference.to_vec();
        trailing.push(0xaa);
        assert_eq!(
            inflate_raw_bounded(&trailing, 5).unwrap_err().code,
            "ODB_ZIP_DEFLATE_TRAILING_DATA"
        );
        assert_eq!(
            inflate_raw_bounded(&reference[..reference.len() - 1], 5)
                .unwrap_err()
                .code,
            "ODB_ZIP_DEFLATE_INVALID"
        );
    }

    #[test]
    fn method8_and_mixed_method0_method8_preserve_canonical_source() {
        let reference_map = read_zip(&example()).unwrap();
        let reference_board = crate::odb_source::import_unpacked(&reference_map, "pcb").unwrap();
        for methods in [[8, 8, 8], [0, 8, 0], [8, 0, 8]] {
            let actual = read_zip(&zip_source(methods)).unwrap();
            assert_eq!(actual, reference_map);
            let board = crate::odb_source::import_unpacked(&actual, "pcb").unwrap();
            assert_eq!(board.sources[0].id.files, reference_board.sources[0].id.files);
            assert_eq!(board.traces.len(), reference_board.traces.len());
            assert_eq!(board.stackup.layers.len(), reference_board.stackup.layers.len());
            assert!(board.vias.is_empty());
            assert!(board.connectivity.edges.is_empty());
            assert!(!board.validate().has_errors());
        }
    }

    #[test]
    fn method8_file_route_preserves_canonical_and_drill_audit() {
        let archive = zip_source([8, 8, 8]);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("boardsi-e013-{}-{stamp}.zip", std::process::id()));
        std::fs::write(&path, archive).unwrap();
        let checked = (|| -> Result<(), crate::odb_source::OdbError> {
            let board = crate::odb_source::import_archive_file(&path, "pcb", false)?;
            let holes = crate::odb_fabrication::import_path_with_holes(&path, "pcb", false)?;
            let drill = crate::odb_source::audit_path_drill_source(&path, "pcb", true)?;
            assert_eq!(board.traces.len(), 2);
            assert_eq!(board.sources[0].id.files, holes.board.sources[0].id.files);
            assert!(holes.source_holes.is_empty());
            assert!(drill.layers.is_empty());
            assert!(drill.hits.unwrap().is_empty());
            Ok(())
        })();
        std::fs::remove_file(&path).unwrap();
        checked.unwrap();
    }

    #[test]
    fn deflate_negative_crc_header_declared_sizes_and_unsupported_method() {
        let archive = zip_source([8, 8, 8]);
        let end = archive.len() - 22;
        let central = u32_at(&archive, end + 16);
        let mut crc = archive.clone();
        crc[14] ^= 1;
        crc[central + 16] ^= 1;
        assert_eq!(read_zip(&crc).unwrap_err().code, "ODB_ZIP_CRC32_MISMATCH");

        let mut short = archive.clone();
        let first_length = u32_at(&short, 22);
        short[22..26].copy_from_slice(&(first_length - 1).to_le_bytes());
        short[central + 24..central + 28].copy_from_slice(&(first_length - 1).to_le_bytes());
        assert_eq!(
            read_zip(&short).unwrap_err().code,
            "ODB_ZIP_DEFLATE_SIZE_MISMATCH"
        );

        let mut large = archive.clone();
        large[22..26].copy_from_slice(&(MAX_FILE as u32 + 1).to_le_bytes());
        large[central + 24..central + 28]
            .copy_from_slice(&(MAX_FILE as u32 + 1).to_le_bytes());
        assert_eq!(read_zip(&large).unwrap_err().code, "ODB_ZIP_FILE_LIMIT");

        let mut method = archive.clone();
        method[8..10].copy_from_slice(&99u16.to_le_bytes());
        method[central + 10..central + 12].copy_from_slice(&99u16.to_le_bytes());
        assert_eq!(
            read_zip(&method).unwrap_err().code,
            "ODB_ZIP_COMPRESSION_UNSUPPORTED"
        );

        let mut corrupt = archive.clone();
        let first_data = 30 + u16_at(&corrupt, 26);
        corrupt[first_data] ^= 0xff;
        assert!(read_zip(&corrupt).is_err());
    }

    #[test]
    fn deflate_bomb_uses_bounded_decompression_not_untrusted_headers() {
        let zeros = vec![0u8; 1_048_576];
        let archive = zip_methods(&[("matrix/matrix", &zeros, 8)]);
        let end = archive.len() - 22;
        let central = u32_at(&archive, end + 16);
        let mut lied = archive;
        lied[22..26].copy_from_slice(&32u32.to_le_bytes());
        lied[central + 24..central + 28].copy_from_slice(&32u32.to_le_bytes());
        assert_eq!(
            read_zip(&lied).unwrap_err().code,
            "ODB_ZIP_DEFLATE_SIZE_MISMATCH"
        );
    }


    fn ustar_of(map: &OdbFileMap) -> Vec<u8> {
        const BLOCK: usize = 512;
        let mut archive = Vec::new();
        for (name, data) in map {
            assert!(name.len() <= 100);
            let mut header = [0u8; BLOCK];
            header[..name.len()].copy_from_slice(name.as_bytes());
            header[100..108].copy_from_slice(b"0000644\0");
            header[108..116].copy_from_slice(b"0000000\0");
            header[116..124].copy_from_slice(b"0000000\0");
            let size = format!("{:011o}\0", data.len());
            header[124..136].copy_from_slice(size.as_bytes());
            header[136..148].copy_from_slice(b"00000000000\0");
            header[156] = b'0';
            header[257..263].copy_from_slice(b"ustar\0");
            header[263..265].copy_from_slice(b"00");
            header[148..156].fill(b' ');
            let sum: u64 = header.iter().map(|&b| u64::from(b)).sum();
            let checksum = format!("{:06o}\0 ", sum);
            header[148..156].copy_from_slice(checksum.as_bytes());
            archive.extend_from_slice(&header);
            archive.extend_from_slice(data);
            archive.resize(archive.len().div_ceil(BLOCK) * BLOCK, 0);
        }
        archive.extend_from_slice(&[0u8; BLOCK * 2]);
        archive
    }

    #[test]
    fn method8_directory_ustar_stored_and_mixed_fixtures_match() {
        let expected = read_zip(&zip_source([8, 8, 8])).unwrap();
        assert_eq!(read_zip(&zip_source([0, 8, 0])).unwrap(), expected);
        assert_eq!(read_zip(&zip_source([0, 0, 0])).unwrap(), expected);
        assert_eq!(crate::odb_tar::read_tar(&ustar_of(&expected)).unwrap(), expected);

        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir()
            .join(format!("boardsi-e013-directory-{}-{stamp}", std::process::id()));
        for (relative, data) in &expected {
            let target = root.join(relative);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::write(target, data).unwrap();
        }
        let actual = crate::odb_source::read_source_path_file_map(&root);
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn deflate_traversal_duplicate_and_multiple_roots_stay_closed() {
        assert_eq!(
            read_zip(&zip_methods(&[("../matrix/matrix", b"x", 8)]))
                .unwrap_err()
                .code,
            "ODB_ZIP_UNSAFE_PATH"
        );
        assert_eq!(
            read_zip(&zip_methods(&[
                ("matrix/matrix", b"x", 8),
                ("matrix/matrix", b"x", 0)
            ]))
            .unwrap_err()
            .code,
            "ODB_ZIP_TOTAL_OR_DUPLICATE_FILE"
        );
        assert_eq!(
            read_zip(&zip_methods(&[
                ("rootA/matrix/matrix", b"x", 8),
                ("rootB/matrix/matrix", b"x", 8)
            ]))
            .unwrap_err()
            .code,
            "ODB_TAR_MULTIPLE_ROOTS"
        );
    }

}
