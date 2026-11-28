//! Exercise the real worker using a small independently specified KiCad board.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, process::Command, time::{SystemTime, UNIX_EPOCH}};

#[test]
fn source_free_snapshot_inspection_is_read_only_and_exact() {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("boardsi-f012-{}-{nanos}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let source = dir.join("input.kicad_pcb");
    let native = dir.join("input.bsi");
    let bytes = br#"(kicad_pcb (version 20241229) (generator "F012 test")
      (layers (0 "F.Cu" signal) (2 "B.Cu" signal))
      (net 0 "") (net 1 "CLK")
      (segment (start 0 0) (end 0.25 0) (width 0.1)
       (layer "F.Cu") (net 1)
       (uuid "223e4567-e89b-12d3-a456-426614174000")))"#;
    fs::write(&source, bytes).unwrap();
    let source_sha = format!("{:x}", Sha256::digest(bytes));
    let bin = env!("CARGO_BIN_EXE_boardsi-native-file");
    let result = Command::new(bin).arg("save-kicad").arg(&source).arg(&native)
        .arg(&source_sha).output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    fs::remove_file(&source).unwrap();
    let before = fs::read(&native).unwrap();
    let digest = format!("{:x}", Sha256::digest(&before));
    let token = "0123456789abcdef0123456789abcdef";
    let query = |category: &str, offset: &str, limit: &str, sha: &str| {
        Command::new(bin).arg("inspect").arg(&native).args([sha,token,category,offset,limit])
            .env_remove("BOARDSI_ARTWORK_REQUEST").output().unwrap()
    };
    let result = query("traces", "0", "1", &digest);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let reply: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(reply["protocol"], "BSI_NATIVE_INSPECT_V1");
    assert_eq!(reply["request"], token);
    assert_eq!(reply["sha256"], digest);
    assert_eq!(reply["total"], 1);
    assert!(reply["next"].is_null());
    assert_eq!(reply["rows"][0]["width_m"]["numerator"], "1");
    assert_eq!(reply["rows"][0]["width_m"]["denominator"], "10000");
    assert_eq!(reply["rows"][0]["end"]["ticks"][0], "250000");
    assert_eq!(reply["physical_si"], "NOT_RUN");
    let mut selection = serde_json::json!({"category":"traces","index":0,
        "id":reply["rows"][0]["meta"]["id"]});
    let pick = |selection: &Value| {
        Command::new(bin).arg("inspect").arg(&native)
            .args([digest.as_str(),token,"traces","0","1"])
            .arg(selection.to_string()).env_remove("BOARDSI_ARTWORK_REQUEST").output().unwrap()
    };
    assert!(pick(&selection).status.success());
    selection["id"]["part"] = serde_json::json!("not-the-canonical-part");
    assert!(!pick(&selection).status.success());
    let result = query("sources", "0", "1", &digest);
    assert!(result.status.success());
    let reply: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(reply["rows"][0]["files"][0]["stored_sha256"], source_sha);
    assert_eq!(reply["rows"][0]["source_reopened"], false);
    assert!(!query("traces", "0", "129", &digest).status.success());
    assert!(!query("traces", "2", "1", &digest).status.success());
    assert!(!query("traces", "0", "1", &"0".repeat(64)).status.success());
    assert!(!query("invalid", "0", "1", &digest).status.success());
    assert_eq!(fs::read(&native).unwrap(), before);
    fs::remove_dir_all(dir).unwrap();
}
