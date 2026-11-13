//! E_015 .Z-inside-archive cross-transport and legacy compatibility regression.
//! No fabricated electrical connectivity, layer material or manufactured vias.
use boardsi_import::odb_source::{import_archive_file, import_directory};
use std::io::Write;
use std::path::Path;

const FEATURE_Z: &[u8] = &[
    0x1f, 0x9d, 0x8c, 0x55, 0x9c, 0x24, 0xa1, 0x32, 0xa5, 0x47, 0x93, 0x26, 0x0a, 0x8c, 0x80, 0x88,
    0xa1, 0x80, 0x04, 0x0c, 0x10, 0x72, 0x62, 0xc0, 0x80, 0xa1, 0x80, 0x09, 0x08, 0x18, 0x20, 0x60,
    0x80, 0x88, 0x01, 0x02, 0x06, 0x08, 0x18, 0x20, 0xa0, 0x80, 0x80, 0xa1, 0x00,
];
const MATRIX: &[u8] = b"STEP {\nCOL=1\nNAME=pcb\n}\n\
LAYER {\nROW=1\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=top\nPOLARITY=POSITIVE\n}\n\
LAYER {\nROW=2\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=bottom\nPOLARITY=POSITIVE\n}\n";

fn entries() -> Vec<(String, Vec<u8>)> {
    vec![
        ("product/matrix/matrix".into(), MATRIX.to_vec()),
        (
            "product/steps/pcb/layers/top/features.Z".into(),
            FEATURE_Z.to_vec(),
        ),
        (
            "product/steps/pcb/layers/bottom/features.Z".into(),
            FEATURE_Z.to_vec(),
        ),
    ]
}
fn tar(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut data = Vec::new();
    for (name, body) in entries {
        let mut h = [0u8; 512];
        h[..name.len()].copy_from_slice(name.as_bytes());
        h[100..108].copy_from_slice(b"0000644\0");
        h[108..116].copy_from_slice(b"0000000\0");
        h[116..124].copy_from_slice(b"0000000\0");
        h[124..136].copy_from_slice(format!("{:011o}\0", body.len()).as_bytes());
        h[136..148].copy_from_slice(b"00000000000\0");
        h[148..156].fill(b' ');
        h[156] = b'0';
        h[257..263].copy_from_slice(b"ustar\0");
        h[263..265].copy_from_slice(b"00");
        let checksum: u64 = h.iter().map(|&b| u64::from(b)).sum();
        h[148..156].copy_from_slice(format!("{checksum:06o}\0 ").as_bytes());
        data.extend_from_slice(&h);
        data.extend_from_slice(body);
        data.resize(data.len().div_ceil(512) * 512, 0);
    }
    data.extend_from_slice(&[0u8; 1024]);
    data
}
fn put16(out: &mut Vec<u8>, n: u16) {
    out.extend_from_slice(&n.to_le_bytes());
}
fn put32(out: &mut Vec<u8>, n: u32) {
    out.extend_from_slice(&n.to_le_bytes());
}
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &value in bytes {
        crc ^= u32::from(value);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
fn zip(entries: &[(String, Vec<u8>)], method: u16) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, input) in entries {
        let packed = if method == 8 {
            let mut deflate =
                flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
            deflate.write_all(input).unwrap();
            deflate.finish().unwrap()
        } else {
            input.clone()
        };
        let version = if method == 8 { 20 } else { 10 };
        let offset = out.len() as u32;
        let crc = crc32(input);
        put32(&mut out, 0x0403_4b50);
        put16(&mut out, version);
        put16(&mut out, 0);
        put16(&mut out, method);
        put16(&mut out, 0);
        put16(&mut out, 0);
        put32(&mut out, crc);
        put32(&mut out, packed.len() as u32);
        put32(&mut out, input.len() as u32);
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
        put32(&mut central, input.len() as u32);
        put16(&mut central, name.len() as u16);
        for _ in 0..4 {
            put16(&mut central, 0);
        }
        put32(&mut central, 0);
        put32(&mut central, offset);
        central.extend_from_slice(name.as_bytes());
    }
    let offset = out.len() as u32;
    out.extend_from_slice(&central);
    put32(&mut out, 0x0605_4b50);
    for _ in 0..2 {
        put16(&mut out, 0);
    }
    put16(&mut out, entries.len() as u16);
    put16(&mut out, entries.len() as u16);
    put32(&mut out, central.len() as u32);
    put32(&mut out, offset);
    put16(&mut out, 0);
    out
}
fn write(path: &Path, bytes: &[u8]) {
    std::fs::write(path, bytes).unwrap();
}

#[test]
fn directory_ustar_gzip_and_zip_0_8_load_same_decoded_feature_geometry() {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("boardsi-e015-{}-{stamp}", std::process::id()));
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    std::fs::create_dir_all(&base).unwrap();
    let _cleanup = Cleanup(base.clone());
    let entries = entries();
    for (name, bytes) in &entries {
        let path = base.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        write(&path, bytes);
    }
    let baseline = import_directory(&base.join("product"), "pcb").unwrap();
    let archive = tar(&entries);
    let tar_path = base.join("board.tar");
    write(&tar_path, &archive);
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gz.write_all(&archive).unwrap();
    let gzip_path = base.join("board.tgz");
    write(&gzip_path, &gz.finish().unwrap());
    let zip0_path = base.join("board-stored.zip");
    write(&zip0_path, &zip(&entries, 0));
    let zip8_path = base.join("board-deflate.zip");
    write(&zip8_path, &zip(&entries, 8));
    for archive_path in [&tar_path, &gzip_path, &zip0_path, &zip8_path] {
        let board = import_archive_file(archive_path, "pcb", false).unwrap();
        assert_eq!(board.stackup.layers.len(), baseline.stackup.layers.len());
        assert_eq!(board.traces.len(), baseline.traces.len());
        for (a, b) in board.traces.iter().zip(baseline.traces.iter()) {
            assert_eq!(a.centreline, b.centreline);
            assert_eq!(a.width_m, b.width_m);
        }
        assert!(board.connectivity.edges.is_empty());
        assert_eq!(board.sources[0].id.files, baseline.sources[0].id.files);
    }
}


#[test]
fn e017_step_repeat_directory_ustar_gzip_zip0_zip8_and_features_z_exact_parity() {
    // Transport-level oracle: one STEP-REPEAT source model with compressed
    // child features.Z must produce exactly the same Canonical object graph,
    // file hashes, rational frame offsets and instance provenance regardless
    // of physical directory, TAR, GZIP, ZIP STORE or ZIP DEFLATE packaging.
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!("boardsi-e017-{}-{stamp}", std::process::id()));
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    std::fs::create_dir_all(&base).unwrap();
    let _cleanup = Cleanup(base.clone());
    let mut sources = entries();
    let matrix = String::from_utf8(sources[0].1.clone()).unwrap()
        + "STEP {\nCOL=2\nNAME=panel\n}\n";
    sources[0].1 = matrix.into_bytes();
    sources.push((
        "product/steps/pcb/stephdr".into(),
        b"UNITS=MM\nX_DATUM=0\nY_DATUM=0\nX_ORIGIN=0\nY_ORIGIN=0\n".to_vec(),
    ));
    sources.push((
        "product/steps/panel/stephdr".into(),
        b"UNITS=MM\nX_DATUM=0\nY_DATUM=0\nSTEP-REPEAT {\nNAME=pcb\nX=10\nY=20\nDX=30\nDY=40\nNX=2\nNY=2\nANGLE=0\nFLIP=NO\nMIRROR=NO\n}\n".to_vec(),
    ));
    for (name, bytes) in &sources {
        let path = base.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        write(&path, bytes);
    }
    let root = base.join("product");
    let child = import_directory(&root, "pcb").unwrap();
    let expected = import_directory(&root, "panel").unwrap();
    assert_eq!(expected.traces.len(), child.traces.len() * 4);
    assert_eq!(expected.frames.len(), child.frames.len() * 4);
    assert_eq!(expected.padstacks.len(), child.padstacks.len() * 4);
    assert!(!expected.validate().has_errors());
    assert!(expected.connectivity.edges.is_empty());
    let tar_path = base.join("board.tar");
    let tar_bytes = tar(&sources);
    write(&tar_path, &tar_bytes);
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gz.write_all(&tar_bytes).unwrap();
    let tgz_path = base.join("board.tgz");
    write(&tgz_path, &gz.finish().unwrap());
    let zip0_path = base.join("board-stored.zip");
    write(&zip0_path, &zip(&sources, 0));
    let zip8_path = base.join("board-deflate.zip");
    write(&zip8_path, &zip(&sources, 8));
    for file in [&tar_path, &tgz_path, &zip0_path, &zip8_path] {
        let received = import_archive_file(file, "panel", false).unwrap();
        assert_eq!(received, expected, "source differs for {}", file.display());
    }
    // A transport does not make unsupported rotation/mirroring safe.
    let header_path = root.join("steps/panel/stephdr");
    let rotated = std::fs::read_to_string(&header_path).unwrap()
        .replace("ANGLE=0", "ANGLE=45");
    write(&header_path, rotated.as_bytes());
    assert_eq!(
        import_directory(&root, "panel").unwrap_err().code,
        "ODB_STEP_TRANSFORM_UNSUPPORTED"
    );
}
