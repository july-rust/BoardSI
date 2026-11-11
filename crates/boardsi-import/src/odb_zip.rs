//! D_056_09: strictly bounded ZIP *stored-member* ODB++ transport.
//!
//! This deliberately does not support DEFLATE, data descriptors, Zip64,
//! split disks, encryption, extra fields or executable/symlink members.
//! ZIP is only a transport; Canonical source/electrical qualification remains
//! in the existing ODB++ importer and cannot be inferred from extraction.
use crate::odb_source::{OdbError, OdbFileMap};
use std::collections::BTreeSet;

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

/// Read a single-disk, zero-comment ZIP with STORED (method 0) members.
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
        if method != 0 {
            return Err(bad("ODB_ZIP_COMPRESSION_UNSUPPORTED"));
        }
        if compressed != uncompressed || compressed > MAX_FILE {
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
        let data = &bytes[data_start..data_end];
        if crc32(data) != expected_crc {
            return Err(bad("ODB_ZIP_CRC32_MISMATCH"));
        }
        if is_directory {
            if !data.is_empty() || !directory_names.insert(clean.to_owned()) {
                return Err(bad("ODB_ZIP_DUPLICATE_OR_NONEMPTY_DIRECTORY"));
            }
        } else {
            total = total
                .checked_add(data.len())
                .ok_or_else(|| bad("ODB_ZIP_TOTAL_LIMIT"))?;
            if total > MAX_TOTAL || raw.insert(name.to_owned(), data.to_vec()).is_some() {
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
        std::fs::write(&path, &archive).unwrap();
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
        assert_eq!(
            read_zip(&compressed).unwrap_err().code,
            "ODB_ZIP_COMPRESSION_UNSUPPORTED"
        );

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
}
