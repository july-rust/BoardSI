//! E_015: deterministic UNIX-compress .Z source path, identity, and fail-closed
//! integration. This is Source-only qualification, not complete ODB++ or SI.
use boardsi_import::odb_source::{
    audit_unpacked_drill_hits, import_tar, import_unpacked, OdbFileMap,
};

const FEATURE: &[u8] = b"UNITS=MM\nF 1\n$0 r100\nL 0 0 1 0 0 P 0\n";
// Independently validated with GNU gzip -dc (UNIX-compress compatibility).
const FEATURE_Z: &[u8] = &[
    0x1f, 0x9d, 0x8c, 0x55, 0x9c, 0x24, 0xa1, 0x32, 0xa5, 0x47, 0x93, 0x26, 0x0a, 0x8c, 0x80, 0x88,
    0xa1, 0x80, 0x04, 0x0c, 0x10, 0x72, 0x62, 0xc0, 0x80, 0xa1, 0x80, 0x09, 0x08, 0x18, 0x20, 0x60,
    0x80, 0x88, 0x01, 0x02, 0x06, 0x08, 0x18, 0x20, 0xa0, 0x80, 0x80, 0xa1, 0x00,
];
const MATRIX: &[u8] = b"STEP {\nCOL=1\nNAME=pcb\n}\n\
LAYER {\nROW=1\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=top\nPOLARITY=POSITIVE\n}\n\
LAYER {\nROW=2\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=bottom\nPOLARITY=POSITIVE\n}\n";

fn board(z: bool) -> OdbFileMap {
    let mut files = OdbFileMap::new();
    files.insert("matrix/matrix".into(), MATRIX.to_vec());
    for layer in ["top", "bottom"] {
        files.insert(
            format!(
                "steps/pcb/layers/{layer}/{}",
                if z { "features.Z" } else { "features" }
            ),
            if z { FEATURE_Z } else { FEATURE }.to_vec(),
        );
    }
    files
}

fn tar(files: &OdbFileMap) -> Vec<u8> {
    let mut result = Vec::new();
    for (name, bytes) in files {
        let name = format!("product/{name}");
        assert!(name.len() <= 100);
        let mut h = [0u8; 512];
        h[..name.len()].copy_from_slice(name.as_bytes());
        h[100..108].copy_from_slice(b"0000644\0");
        h[108..116].copy_from_slice(b"0000000\0");
        h[116..124].copy_from_slice(b"0000000\0");
        h[124..136].copy_from_slice(format!("{:011o}\0", bytes.len()).as_bytes());
        h[136..148].copy_from_slice(b"00000000000\0");
        h[148..156].fill(b' ');
        h[156] = b'0';
        h[257..263].copy_from_slice(b"ustar\0");
        h[263..265].copy_from_slice(b"00");
        let sum: u64 = h.iter().map(|v| u64::from(*v)).sum();
        h[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
        result.extend_from_slice(&h);
        result.extend_from_slice(bytes);
        result.resize(result.len().div_ceil(512) * 512, 0);
    }
    result.extend_from_slice(&[0u8; 1024]);
    result
}

#[test]
fn compressed_source_is_geometry_equivalent_and_retains_both_digests() {
    let a = import_unpacked(&board(false), "pcb").unwrap();
    let b = import_unpacked(&board(true), "pcb").unwrap();
    let c = import_tar(&tar(&board(true)), "pcb").unwrap();
    for other in [&b, &c] {
        assert_eq!(a.stackup.layers.len(), other.stackup.layers.len());
        assert_eq!(a.traces.len(), other.traces.len());
        assert_eq!(a.pads.len(), other.pads.len());
        assert_eq!(a.arcs.len(), other.arcs.len());
        assert_eq!(a.areas.len(), other.areas.len());
        for (ta, tb) in a.traces.iter().zip(other.traces.iter()) {
            assert_eq!(ta.centreline.len(), tb.centreline.len());
            for (pa, pb) in ta.centreline.iter().zip(tb.centreline.iter()) {
                // Compressed and plain Source IDs intentionally differ: only
                // physical coordinates and their logical frame must match.
                assert_eq!(pa.ticks, pb.ticks);
                assert_eq!(pa.frame.kind, pb.frame.kind);
                assert_eq!(pa.frame.source_object, pb.frame.source_object);
            }
            assert_eq!(ta.width_m, tb.width_m);
        }
        assert!(other.connectivity.edges.is_empty());
        let listed = &other.sources[0].id.files;
        assert!(listed
            .iter()
            .any(|f| f.logical_path.ends_with("/features.Z")));
        assert!(listed.iter().any(|f| f.logical_path.ends_with("/features")));
        let compressed = listed
            .iter()
            .find(|f| f.logical_path == "steps/pcb/layers/top/features.Z")
            .unwrap();
        let expanded = listed
            .iter()
            .find(|f| f.logical_path == "steps/pcb/layers/top/features")
            .unwrap();
        assert_ne!(compressed.sha256, expanded.sha256);
    }
    // Legacy inputs continue to have only their original SourceFile identity.
    assert_eq!(a.sources[0].id.files.len(), 3);
    assert_eq!(b.sources[0].id.files.len(), 5);
}

#[test]
fn duplicate_wrong_path_bad_lzw_and_unsupported_features_fail_closed() {
    let mut both = board(true);
    both.insert("steps/pcb/layers/top/features".into(), FEATURE.to_vec());
    assert_eq!(
        import_unpacked(&both, "pcb").unwrap_err().code,
        "ODB_Z_DUPLICATE_FEATURES"
    );

    let mut wrong = board(false);
    wrong.insert("steps/pcb/eda/data.Z".into(), FEATURE_Z.to_vec());
    assert_eq!(
        import_unpacked(&wrong, "pcb").unwrap_err().code,
        "ODB_Z_UNSUPPORTED_MEMBER_PATH"
    );

    let mut broken = board(true);
    broken.insert(
        "steps/pcb/layers/top/features.Z".into(),
        vec![0x1f, 0x9d, 0x8c, 1],
    );
    assert!(import_unpacked(&broken, "pcb").is_err());

    let mut unsupported = board(true);
    unsupported.insert(
        "steps/pcb/layers/top/features.Z".into(),
        // Valid UNIX-compress codes but decoded content is not a valid ODB++ feature file.
        vec![0x1f, 0x9d, 0x8c, 0x41, 0x02, 0x02],
    );
    assert!(import_unpacked(&unsupported, "pcb").is_err());
}

#[test]
fn drill_tool_and_hit_audit_are_unaffected_by_compressed_copper_features() {
    let mut plain = board(false);
    plain.insert(
        "matrix/matrix".into(),
        [MATRIX, b"LAYER {\nROW=3\nCONTEXT=BOARD\nTYPE=DRILL\nNAME=drill\nPOLARITY=POSITIVE\nSTART_NAME=top\nEND_NAME=bottom\n}\n"].concat(),
    );
    plain.insert(
        "steps/pcb/layers/drill/tools".into(),
        b"UNITS=MM\nTOOLS {\nNUM=7\nTYPE=VIA\nTYPE2=STANDARD\nFINISH_SIZE=110\n}\n".to_vec(),
    );
    plain.insert(
        "steps/pcb/layers/drill/features".into(),
        b"UNITS=MM\nF 1\n$0 r110\nP 1.25 -0.25 0 P 7 0;ID=hole1\n".to_vec(),
    );
    let baseline = audit_unpacked_drill_hits(&plain, "pcb").unwrap();
    assert_eq!(baseline.len(), 1);
    for layer in ["top", "bottom"] {
        plain.remove(&format!("steps/pcb/layers/{layer}/features"));
        plain.insert(
            format!("steps/pcb/layers/{layer}/features.Z"),
            FEATURE_Z.to_vec(),
        );
    }
    let test = audit_unpacked_drill_hits(&plain, "pcb").unwrap();
    assert_eq!(test, baseline);
}
