//! F_006 source-limited KiCad9 -> Canonical -> .bsi -> Canonical.
//! Synthetic KiCad source fixtures are NOT independent KiCad vendor evidence.
use boardsi_import::kicad9_source::into_native_canonical;
use boardsi_model::Rational;
use boardsi_native::{canonical_semantics_equal, load};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const SOURCE: &str = r#"(kicad_pcb (version 20241229) (generator "F006")
  (layers (0 "F.Cu" signal) (2 "B.Cu" signal)
          (4 "In1.Cu" signal) (6 "In2.Cu" signal))
  (net 0 "") (net 1 "CLK")
  (segment (start 1.123456 -2.000001) (end 3.000001 5)
           (width 0.15) (layer "In1.Cu") (net 1)
           (uuid "223e4567-e89b-12d3-a456-426614174000"))
  (via (at 3.000001 5) (size 0.6) (drill 0.3)
       (layers "F.Cu" "B.Cu") (net 1)
       (uuid "123e4567-e89b-12d3-a456-426614174000")))"#;
fn temp() -> PathBuf {
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("boardsi-f006-{}-{n}", std::process::id()));
    fs::create_dir(&path).unwrap();
    path
}
fn invoke(parts: &[&std::ffi::OsStr]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_boardsi-native-file"))
        .args(parts)
        .output()
        .expect("actual worker launch")
}
#[test]
fn f006_worker_exact_semantic_roundtrip() {
    let dir = temp();
    let source = dir.join("candidate.kicad_pcb");
    let target = dir.join("candidate.bsi");
    fs::write(&source, SOURCE).unwrap();
    let hash = format!("{:x}", Sha256::digest(SOURCE.as_bytes()));
    let original = into_native_canonical(SOURCE.as_bytes()).unwrap();
    let result = invoke(&[
        std::ffi::OsStr::new("save-kicad"),
        source.as_os_str(),
        target.as_os_str(),
        std::ffi::OsStr::new(&hash),
    ]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(receipt["source_format"], "KICAD9_SOURCE_SUBSET");
    assert_eq!(receipt["traces"], 1);
    assert_eq!(receipt["vias"], 1);
    let loaded = load(&target).unwrap();
    assert!(canonical_semantics_equal(&original, loaded.model()));
    assert_eq!(
        loaded.model().traces[0].centreline[0].ticks,
        [1_123_456, -2_000_001]
    );
    assert_eq!(
        loaded.model().traces[0].width_m,
        Rational::new(3, 20_000).unwrap()
    );
    assert!(loaded.model().connectivity.nodes.is_empty());
    assert!(loaded.model().connectivity.edges.is_empty());
    assert!(loaded.model().vias[0].connected_layers.is_empty());
    assert!(loaded
        .model()
        .stackup
        .layers
        .iter()
        .all(|l| l.thickness_m.value().is_none()));
    fs::remove_dir_all(dir).unwrap();
}
#[test]
fn f006_source_hash_and_unsupported_geometry_preserve_last_good_file() {
    let dir = temp();
    let source = dir.join("candidate.kicad_pcb");
    let target = dir.join("candidate.bsi");
    fs::write(&source, SOURCE).unwrap();
    let good = format!("{:x}", Sha256::digest(SOURCE.as_bytes()));
    let args = |digest: &str| {
        invoke(&[
            std::ffi::OsStr::new("save-kicad"),
            source.as_os_str(),
            target.as_os_str(),
            std::ffi::OsStr::new(digest),
        ])
    };
    assert!(args(&good).status.success());
    let original = fs::read(&target).unwrap();
    let altered = SOURCE.replace("(net 1 \"CLK\")", "(net 1 \"NEW\")");
    fs::write(&source, &altered).unwrap();
    assert!(!args(&good).status.success());
    assert_eq!(original, fs::read(&target).unwrap());
    let changed = format!("{:x}", Sha256::digest(altered.as_bytes()));
    assert!(args(&changed).status.success());
    // An unsupported copper zone must FAIL rather than silently disappear.
    let stable = fs::read(&target).unwrap();
    let unsupported = altered.replace("(via (at", "(zone (net 1)) (via (at");
    fs::write(&source, &unsupported).unwrap();
    let sha = format!("{:x}", Sha256::digest(unsupported.as_bytes()));
    assert!(!args(&sha).status.success());
    assert_eq!(stable, fs::read(&target).unwrap());
    fs::remove_dir_all(dir).unwrap();
}
