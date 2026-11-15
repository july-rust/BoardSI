//! F0031 standalone process integration; no fake Qt IPC or source bytes.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf, process::Command, time::{SystemTime, UNIX_EPOCH}};

fn binary() -> &'static str { env!("CARGO_BIN_EXE_boardsi-native-file") }
fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/boardsi-import/tests/fixtures/pads_via_round_through_v94.asc")
}
fn temp() -> PathBuf {
    let n=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir=std::env::temp_dir().join(format!("boardsi-f0031-worker-{}-{n}",std::process::id()));
    fs::create_dir(&dir).unwrap();
    dir
}
#[test]
fn actual_process_save_open_copy_and_source_sha_guard() {
    let dir=temp();
    let board=fixture();
    let good=dir.join("source.bsi");
    let next=dir.join("copy.bsi");
    let hash=format!("{:x}",Sha256::digest(&fs::read(&board).unwrap()));
    let saved=Command::new(binary()).arg("save-pads").arg(&board).arg(&good).arg(&hash)
        .output().unwrap();
    assert!(saved.status.success(),"{}",String::from_utf8_lossy(&saved.stderr));
    let json: Value=serde_json::from_slice(&saved.stdout).unwrap();
    assert_eq!(json["protocol"],"BSI_NATIVE_FILE_V1");
    assert_eq!(json["action"],"save-pads");
    assert_eq!(json["physical_si"],"NOT_RUN");
    assert_eq!(json["read_only"],false);
    assert!(json["traces"].as_u64().unwrap()>0);
    let opened=Command::new(binary()).arg("open").arg(&good).output().unwrap();
    assert!(opened.status.success(),"{}",String::from_utf8_lossy(&opened.stderr));
    let loaded:Value=serde_json::from_slice(&opened.stdout).unwrap();
    assert_eq!(loaded["traces"],json["traces"]);
    let copied=Command::new(binary()).arg("save-loaded").arg(&good).arg(&next).output().unwrap();
    assert!(copied.status.success(),"{}",String::from_utf8_lossy(&copied.stderr));
    assert!(fs::metadata(&next).unwrap().len()>4096);
    let original=fs::read(&good).unwrap();
    let wrong=Command::new(binary()).arg("save-pads").arg(&board).arg(&good).arg("0".repeat(64))
        .output().unwrap();
    assert!(!wrong.status.success());
    assert_eq!(fs::read(&good).unwrap(),original,"SHA error must preserve last-good bytes");
    let corrupt=dir.join("corrupt.bsi");
    fs::write(&corrupt,b"no sqlite").unwrap();
    let rejected=Command::new(binary()).arg("open").arg(&corrupt).output().unwrap();
    assert!(!rejected.status.success());
    assert_eq!(fs::read(&good).unwrap(),original);
    fs::remove_dir_all(&dir).unwrap();
}
