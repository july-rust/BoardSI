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
    // The historical nine-geometry fixture has no stored nets. Do not
    // manufacture a parent at index zero; use a separate net-bearing fixture.
    query["category"] = json!("nets");
    writeln!(input, "{query}").unwrap();
    input.flush().unwrap();
    let nets = receive();
    let conn =
        rusqlite::Connection::open_with_flags(&native, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .unwrap();
    let net_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM nets", [], |row| row.get(0))
        .unwrap();
    assert_eq!(nets["total"], net_count);
    drop(conn);
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

#[test]
fn independent_sql_membership_oracle_and_mixed_geometry() {
    use rusqlite::{Connection, OpenFlags};
    use std::collections::BTreeSet;

    let dir = std::env::temp_dir().join(format!(
        "boardsi-f013-members-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&dir).unwrap();
    let source = dir.join("input.kicad_pcb");
    let native = dir.join("input.bsi");
    let source_bytes = br#"(kicad_pcb (version 20241229) (generator "F013 membership oracle")
      (layers (0 "F.Cu" signal) (2 "B.Cu" signal))
      (net 0 "") (net 1 "CLK")
      (segment (start 0 0) (end 0.25 0) (width 0.1)
       (layer "F.Cu") (net 1)
       (uuid "223e4567-e89b-12d3-a456-426614174000"))
      (segment (start 0 1) (end 0.5 1) (width 0.1)
       (layer "B.Cu") (net 1)
       (uuid "323e4567-e89b-12d3-a456-426614174000"))
      (via (at 0 1) (size 0.6) (drill 0.3) (layers "F.Cu" "B.Cu") (net 1)
       (uuid "123e4567-e89b-12d3-a456-426614174000")))"#;
    fs::write(&source, source_bytes).unwrap();
    let bin = env!("CARGO_BIN_EXE_boardsi-native-file");
    let saved = Command::new(bin)
        .arg("save-kicad")
        .arg(&source)
        .arg(&native)
        .arg(format!("{:x}", Sha256::digest(source_bytes)))
        .env_remove("BOARDSI_ARTWORK_REQUEST")
        .output()
        .unwrap();
    assert!(
        saved.status.success(),
        "{}",
        String::from_utf8_lossy(&saved.stderr)
    );
    fs::remove_file(&source).unwrap();
    let before = fs::read(&native).unwrap();
    let digest = format!("{:x}", Sha256::digest(&before));
    let conn = Connection::open_with_flags(&native, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let expected: BTreeSet<(String, String, String)> = conn
        .prepare("SELECT o.kind,o.source_object,o.part FROM geometry g JOIN objects o ON o.id=g.object_id JOIN nets n ON n.object_id=g.net_id WHERE n.name='CLK' UNION ALL SELECT o.kind,o.source_object,o.part FROM pins p JOIN objects o ON o.id=p.object_id JOIN nets n ON n.object_id=p.net_id WHERE n.name='CLK'")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(expected.iter().filter(|v| v.0 == "Trace").count(), 2);
    assert_eq!(expected.iter().filter(|v| v.0 == "Via").count(), 1);
    let document = "33333333333333333333333333333333";
    let session = "44444444444444444444444444444444";
    let generation = "55555555555555555555555555555555";
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
    let mut read = || {
        let mut line = String::new();
        assert!(output.read_line(&mut line).unwrap() > 0);
        assert!(line.len() < 60_000);
        serde_json::from_str::<Value>(&line).unwrap()
    };
    assert_eq!(read()["native_loads"], 1);
    let mut sequence = 0_u128;
    let mut ask = |category: &str, offset: usize, limit: usize| {
        sequence += 1;
        let request = format!("{sequence:032x}");
        let frame = json!({"protocol":"BSI_NATIVE_SESSION_V1","session":session,
            "document":document,"generation":generation,"sha256":digest,"request":request,
            "action":"query","category":category,"offset":offset,"limit":limit});
        writeln!(input, "{frame}").unwrap();
        input.flush().unwrap();
        let reply = read();
        assert_eq!(reply["session"], session);
        assert_eq!(reply["request"], request);
        assert_eq!(reply["physical_contact"], "NOT_VERIFIED");
        reply
    };
    let nets = ask("nets", 0, 128);
    let parent = nets["rows"]
        .as_array()
        .unwrap()
        .iter()
        .position(|row| row["name"] == "CLK")
        .unwrap();
    let category = format!("net-members:{parent}");
    let mut actual = BTreeSet::new();
    for i in 0..expected.len() {
        let page = ask(&category, i, 1);
        assert_eq!(page["total"], expected.len());
        let row = &page["rows"][0];
        assert_eq!(row["basis"], "stored_net");
        let id = &row["reference"]["id"];
        let key = (
            id["kind"].as_str().unwrap().to_owned(),
            id["source_object"].as_str().unwrap().to_owned(),
            id["part"].as_str().unwrap().to_owned(),
        );
        assert!(actual.insert(key));
        let detail = ask(
            row["reference"]["category"].as_str().unwrap(),
            row["reference"]["index"].as_u64().unwrap() as usize,
            1,
        );
        assert_eq!(detail["rows"][0]["meta"]["id"], *id);
    }
    assert_eq!(actual, expected);
    assert!(ask(&category, expected.len(), 1)["rows"]
        .as_array()
        .unwrap()
        .is_empty());
    let layers = ask("layers", 0, 128);
    for (parent, layer) in layers["rows"].as_array().unwrap().iter().enumerate() {
        let name = layer["name"].as_str().unwrap();
        let expected: BTreeSet<String> = conn
            .prepare("SELECT o.source_object FROM geometry g JOIN objects o ON o.id=g.object_id JOIN layers l ON l.object_id=g.layer_id WHERE l.name=? AND o.kind='Trace'")
            .unwrap()
            .query_map([name], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(expected.len(), 1);
        let members = ask(&format!("layer-members:{parent}"), 0, 128);
        let actual: BTreeSet<String> = members["rows"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["reference"]["id"]["kind"] == "Trace")
            .map(|row| {
                assert_eq!(row["basis"], "stored_layer");
                row["reference"]["id"]["source_object"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect();
        assert_eq!(actual, expected);
    }
    let traces = ask("traces", 0, 128);
    assert_eq!(traces["total"], 2);
    let lengths: BTreeSet<i64> = traces["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["geometric_length_m_approx"].as_f64().unwrap() * 1e9).round() as i64)
        .collect();
    assert_eq!(lengths, BTreeSet::from([250_000, 500_000]));
    let via = ask("vias", 0, 1);
    assert_eq!(
        via["rows"][0]["drill_details"]["diameter_m"],
        json!({"numerator":"3","denominator":"10000"})
    );
    drop(input);
    assert!(child.wait().unwrap().success());
    drop(conn);
    assert_eq!(fs::read(&native).unwrap(), before);
    fs::remove_dir_all(dir).unwrap();
}
