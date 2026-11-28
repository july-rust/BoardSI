//! Real F013 process: immutable session, full identity and bounded frames.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn persistent_native_session_reuses_snapshot_and_rejects_wrong_identity() {
    let dir = std::env::temp_dir().join(format!(
        "boardsi-f013-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&dir).unwrap();
    let native = dir.join("golden.bsi");
    let bytes = include_bytes!("../../../schemas/bsi/golden-v1-nine-geometry.bsi");
    fs::write(&native, bytes).unwrap();
    let digest = format!("{:x}", Sha256::digest(bytes));
    let document = "11111111111111111111111111111111";
    let session = "0123456789abcdef0123456789abcdef";
    let generation = "22222222222222222222222222222222";
    let request = "fedcba9876543210fedcba9876543210";
    let bin = env!("CARGO_BIN_EXE_boardsi-native-file");
    let mut child = Command::new(bin)
        .arg("inspect-session")
        .arg(&native)
        .args([digest.as_str(), document, session, generation])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut receive = || {
        let mut line = String::new();
        assert!(output.read_line(&mut line).unwrap() > 0);
        assert!(line.len() < 60_000);
        serde_json::from_str::<Value>(&line).unwrap()
    };
    let ready = receive();
    assert_eq!(ready["action"], "ready");
    assert_eq!(ready["native_loads"], 1);
    assert_eq!(ready["sha256"], digest);
    assert_eq!(ready["document"], document);
    assert_eq!(ready["generation"], generation);
    let mut query = json!({"protocol":"BSI_NATIVE_SESSION_V1","session":session,
        "document":document,"generation":generation,"sha256":digest,"request":request,"action":"query","category":"arcs",
        "offset":0,"limit":1});
    for _ in 0..100 {
        writeln!(input, "{query}").unwrap();
        input.flush().unwrap();
        let reply = receive();
        assert_eq!(reply["session"], session);
        assert_eq!(reply["document"], document);
        assert_eq!(reply["generation"], generation);
        assert_eq!(reply["request"], request);
        assert_eq!(reply["rows"][0]["centre"]["ticks"], json!(["3000", "3000"]));
        query["id"] = reply["rows"][0]["meta"]["id"].clone();
    }
    query["action"] = json!("lookup");
    writeln!(input, "{query}").unwrap();
    input.flush().unwrap();
    let direct = receive();
    assert_eq!(direct["category"], "arcs");
    assert_eq!(direct["rows"][0]["meta"]["id"], query["id"]);
    query["action"] = json!("query");
    query["category"] = json!("layer-members:0");
    query["limit"] = json!(128);
    writeln!(input, "{query}").unwrap();
    input.flush().unwrap();
    let members = receive();
    assert_eq!(members["category"], "layer-members:0");
    assert_eq!(members["physical_contact"], "NOT_VERIFIED");
    assert!(members["rows"].is_array());
    query["category"] = json!("net-members:0");
    writeln!(input, "{query}").unwrap();
    input.flush().unwrap();
    let members = receive();
    assert_eq!(members["category"], "net-members:0");
    assert_eq!(members["physical_si"], "NOT_RUN");
    assert!(members["rows"].is_array());
    query["session"] = json!("00000000000000000000000000000000");
    writeln!(input, "{query}").unwrap();
    input.flush().unwrap();
    drop(input);
    assert!(!child.wait().unwrap().success());
    assert_eq!(fs::read(&native).unwrap(), bytes);

    // A separate real session must deny an ordinary writer on Windows,
    // or reject a same-size/mtime-restored mutation on Unix.
    let mut changed = Command::new(bin)
        .arg("inspect-session")
        .arg(&native)
        .args([digest.as_str(), document, session, generation])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let changed_in = changed.stdin.take().unwrap();
    let mut changed_out = BufReader::new(changed.stdout.take().unwrap());
    let mut line = String::new();
    assert!(changed_out.read_line(&mut line).unwrap() > 0);
    let ready: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(ready["action"], "ready");
    #[cfg(windows)]
    {
        assert!(fs::OpenOptions::new().write(true).open(&native).is_err());
        drop(changed_in);
        assert!(changed.wait().unwrap().success());
    }
    #[cfg(unix)]
    {
        use std::io::{Seek, SeekFrom};
        let original_modified = fs::metadata(&native).unwrap().modified().unwrap();
        let mut writer = fs::OpenOptions::new().write(true).open(&native).unwrap();
        writer.seek(SeekFrom::End(-1)).unwrap();
        writer.write_all(&[bytes[bytes.len() - 1] ^ 1]).unwrap();
        writer
            .set_times(fs::FileTimes::new().set_modified(original_modified))
            .unwrap();
        drop(writer);
        let mut changed_in = changed_in;
        query["session"] = json!(session);
        writeln!(changed_in, "{query}").unwrap();
        changed_in.flush().unwrap();
        drop(changed_in);
        assert!(!changed.wait().unwrap().success());
    }
    fs::remove_dir_all(dir).unwrap();
}
