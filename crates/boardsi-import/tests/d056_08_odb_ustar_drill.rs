//! Source-only USTAR/directory Drill evidence parity, not plated continuity or SI.
use boardsi_import::odb_source::{
    audit_path_drill_source, audit_tar_drill_source, audit_unpacked_drill_hits, OdbFileMap,
};

const BLOCK: usize = 512;

fn source() -> OdbFileMap {
    let mut f = OdbFileMap::new();
    f.insert(
        "matrix/matrix".into(),
        b"STEP {\nCOL=1\nNAME=pcb\n}\nLAYER {\nROW=1\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=top\nPOLARITY=POSITIVE\n}\nLAYER {\nROW=2\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=bottom\nPOLARITY=POSITIVE\n}\nLAYER {\nROW=3\nCONTEXT=BOARD\nTYPE=DRILL\nNAME=drill\nPOLARITY=POSITIVE\nSTART_NAME=top\nEND_NAME=bottom\n}\n".to_vec(),
    );
    for layer in ["top", "bottom"] {
        f.insert(
            format!("steps/pcb/layers/{layer}/features"),
            b"UNITS=MM\nF 1\n$0 r100\nL 0 0 1 0 0 P 0\n".to_vec(),
        );
    }
    f.insert(
        "steps/pcb/layers/drill/tools".into(),
        b"UNITS=MM\nTOOLS {\nNUM=7\nTYPE=VIA\nTYPE2=STANDARD\nFINISH_SIZE=110\n}\n".to_vec(),
    );
    f.insert(
        "steps/pcb/layers/drill/features".into(),
        b"UNITS=MM\nF 1\n$0 r110\nP 1.25 -0.25 0 P 7 0;ID=hole1\n".to_vec(),
    );
    f
}

fn member(name: &str, data: &[u8]) -> Vec<u8> {
    let mut h = [0u8; BLOCK];
    assert!(name.len() <= 100);
    h[..name.len()].copy_from_slice(name.as_bytes());
    h[100..108].copy_from_slice(b"0000644\0");
    h[108..116].copy_from_slice(b"0000000\0");
    h[116..124].copy_from_slice(b"0000000\0");
    let size = format!("{:011o}\0", data.len());
    h[124..136].copy_from_slice(size.as_bytes());
    h[136..148].copy_from_slice(b"00000000000\0");
    h[156] = b'0';
    h[257..263].copy_from_slice(b"ustar\0");
    h[263..265].copy_from_slice(b"00");
    for b in &mut h[148..156] {
        *b = b' ';
    }
    let sum: u64 = h.iter().map(|&b| u64::from(b)).sum();
    let check = format!("{:06o}\0 ", sum);
    h[148..156].copy_from_slice(check.as_bytes());
    let mut out = h.to_vec();
    out.extend_from_slice(data);
    out.resize(BLOCK + data.len().div_ceil(BLOCK) * BLOCK, 0);
    out
}

fn archive(source: &OdbFileMap, prefix: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, data) in source {
        out.extend(member(&format!("{prefix}{name}"), data));
    }
    out.extend([0u8; BLOCK * 2]);
    out
}

#[test]
fn same_source_snapshot_in_tar_and_directory_yields_identical_evidence() {
    let files = source();
    let baseline = audit_unpacked_drill_hits(&files, "pcb").unwrap();
    assert_eq!(baseline.len(), 1);
    for prefix in ["", "product/", "steps/"] {
        let tar = archive(&files, prefix);
        let full = audit_tar_drill_source(&tar, "pcb", true).unwrap();
        assert_eq!(full.layers.len(), 1);
        assert_eq!(full.hits.as_ref().unwrap(), &baseline);
        assert_eq!(full.layers[0].explicit_start_layer.as_deref(), Some("top"));
        assert_eq!(full.layers[0].explicit_end_layer.as_deref(), Some("bottom"));
        let tooling = audit_tar_drill_source(&tar, "pcb", false).unwrap();
        assert!(tooling.hits.is_none());
        assert_eq!(tooling.layers, full.layers);
    }

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let base = std::env::temp_dir().join(format!(
        "boardsi-d05608-odb-ustar-drill-{}-{unique}",
        std::process::id()
    ));
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    std::fs::create_dir_all(&base).unwrap();
    let _cleanup = Cleanup(base.clone());
    let dir = base.join("product");
    for (name, bytes) in &files {
        let file = dir.join(name);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, bytes).unwrap();
    }
    let tar_path = base.join("source.tar");
    std::fs::write(&tar_path, archive(&files, "product/")).unwrap();
    let folder = audit_path_drill_source(&dir, "pcb", true).unwrap();
    let from_tar = audit_path_drill_source(&tar_path, "pcb", true).unwrap();
    assert_eq!(folder.layers, from_tar.layers);
    assert_eq!(folder.hits, from_tar.hits);
    assert_eq!(folder.hits.unwrap(), baseline);
}

#[test]
fn malformed_drill_source_and_archive_fail_closed() {
    let mut files = source();
    let hits_path = "steps/pcb/layers/drill/features";
    files.insert(
        hits_path.into(),
        String::from_utf8(files[hits_path].clone())
            .unwrap()
            .replace("P 7 0", "P 8 0")
            .into_bytes(),
    );
    assert_eq!(
        audit_tar_drill_source(&archive(&files, ""), "pcb", true)
            .unwrap_err()
            .code,
        "ODB_DRILL_HIT_UNKNOWN_TOOL"
    );
    // Tool-only mode explicitly does not claim the P hit inventory was audited.
    let tool_only = audit_tar_drill_source(&archive(&files, ""), "pcb", false).unwrap();
    assert!(tool_only.hits.is_none());

    let original = source();
    let mut mixed = archive(&original, "");
    mixed.truncate(mixed.len() - BLOCK * 2);
    mixed.extend(member("second_product/matrix/matrix", b"foreign product"));
    mixed.extend([0u8; BLOCK * 2]);
    assert_eq!(
        audit_tar_drill_source(&mixed, "pcb", true)
            .unwrap_err()
            .code,
        "ODB_TAR_MULTIPLE_ROOTS"
    );

    let mut corrupted = archive(&original, "");
    corrupted[0] ^= 1;
    assert_eq!(
        audit_tar_drill_source(&corrupted, "pcb", false)
            .unwrap_err()
            .code,
        "ODB_TAR_CHECKSUM"
    );
}
