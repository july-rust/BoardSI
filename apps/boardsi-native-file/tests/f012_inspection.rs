//! Exercise the real worker using a small independently specified KiCad board.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn source_free_snapshot_inspection_is_read_only_and_exact() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
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
    let result = Command::new(bin)
        .arg("save-kicad")
        .arg(&source)
        .arg(&native)
        .arg(&source_sha)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    fs::remove_file(&source).unwrap();
    let before = fs::read(&native).unwrap();
    let digest = format!("{:x}", Sha256::digest(&before));
    let token = "0123456789abcdef0123456789abcdef";
    let query = |category: &str, offset: &str, limit: &str, sha: &str| {
        Command::new(bin)
            .arg("inspect")
            .arg(&native)
            .args([sha, token, category, offset, limit])
            .env_remove("BOARDSI_ARTWORK_REQUEST")
            .output()
            .unwrap()
    };
    let result = query("traces", "0", "1", &digest);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
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
    let length = reply["rows"][0]["geometric_length_m_approx"]
        .as_f64()
        .unwrap();
    assert!((length - 0.00025).abs() < 1e-15);
    let points = query("trace-points:0", "1", "1", &digest);
    assert!(points.status.success());
    let points: Value = serde_json::from_slice(&points.stdout).unwrap();
    assert_eq!(points["total"], 2);
    assert_eq!(points["rows"][0]["ticks"][0], "250000");
    assert!(points["next"].is_null());
    assert!(!query("trace-points:1", "0", "1", &digest).status.success());
    assert!(!query("trace-points", "0", "1", &digest).status.success());
    let mut selection = serde_json::json!({"category":"traces","index":0,
    "id":reply["rows"][0]["meta"]["id"]});
    let pick = |selection: &Value| {
        Command::new(bin)
            .arg("inspect")
            .arg(&native)
            .args([digest.as_str(), token, "traces", "0", "1"])
            .arg(selection.to_string())
            .env_remove("BOARDSI_ARTWORK_REQUEST")
            .output()
            .unwrap()
    };
    assert!(pick(&selection).status.success());
    selection["id"]["part"] = serde_json::json!("not-the-canonical-part");
    assert!(!pick(&selection).status.success());
    let result = query("sources", "0", "1", &digest);
    assert!(result.status.success());
    let reply: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(reply["rows"][0]["files"][0]["stored_sha256"], source_sha);
    assert_eq!(reply["rows"][0]["source_reopened"], false);
    let files = query("source-files:0", "0", "1", &digest);
    assert!(files.status.success());
    let files: Value = serde_json::from_slice(&files.stdout).unwrap();
    assert_eq!(files["rows"][0]["stored_sha256"], source_sha);
    assert!(!query("sources:0", "0", "1", &digest).status.success());
    assert!(!query("traces", "0", "129", &digest).status.success());
    assert!(!query("traces", "2", "1", &digest).status.success());
    assert!(!query("traces", "0", "1", &"0".repeat(64)).status.success());
    assert!(!query("invalid", "0", "1", &digest).status.success());
    assert_eq!(fs::read(&native).unwrap(), before);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn frozen_geometry_and_independent_stackup_oracle() {
    use rusqlite::{params, Connection, OpenFlags};
    use serde_json::json;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("boardsi-f012-oracle-{}-{nanos}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let path = dir.join("golden.bsi");
    let bytes = include_bytes!("../../../schemas/bsi/golden-v1-nine-geometry.bsi");
    let frozen_sha = include_str!("../../../schemas/bsi/golden-v1-nine-geometry.sha256").trim();
    assert_eq!(format!("{:x}", Sha256::digest(bytes)), frozen_sha);
    fs::write(&path, bytes).unwrap();
    let bin = env!("CARGO_BIN_EXE_boardsi-native-file");
    let run = |path: &std::path::Path, category: &str, offset: &str, limit: &str| {
        let digest = format!("{:x}", Sha256::digest(fs::read(path).unwrap()));
        Command::new(bin)
            .arg("inspect")
            .arg(path)
            .args([
                digest.as_str(),
                "0123456789abcdef0123456789abcdef",
                category,
                offset,
                limit,
            ])
            .env_remove("BOARDSI_ARTWORK_REQUEST")
            .output()
            .unwrap()
    };
    let page = |path: &std::path::Path, category: &str, offset: &str, limit: &str| {
        let result = run(path, category, offset, limit);
        assert!(
            result.status.success(),
            "{category}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let value: Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(value["physical_si"], "NOT_RUN");
        assert_eq!(value["physical_contact"], "NOT_VERIFIED");
        value
    };
    let inspect = |category: &str| page(&path, category, "0", "128");
    let rational = |n: i128, d: i128| json!({"numerator":n.to_string(),"denominator":d.to_string()});
    assert_eq!(inspect("board")["rows"][0]["stackup_total_thickness_m"]["state"], "UNAVAILABLE");
    let arc = inspect("arcs");
    assert_eq!(arc["rows"][0]["centre"]["ticks"], json!(["3000", "3000"]));
    assert_eq!(arc["rows"][0]["start"]["ticks"], json!(["4000", "3000"]));
    assert_eq!(arc["rows"][0]["end"]["ticks"], json!(["3000", "4000"]));
    assert_eq!(arc["rows"][0]["clockwise"], false);
    assert_eq!(arc["rows"][0]["width_m"], rational(1, 100000));
    let area = inspect("area-segments:0");
    assert_eq!(area["total"], 4);
    assert_eq!(area["rows"][0]["start"]["ticks"], json!(["0", "0"]));
    assert_eq!(area["rows"][0]["end"]["ticks"], json!(["12000", "0"]));
    let hole = inspect("area-cutout-segments:0:0");
    assert_eq!(hole["rows"][0]["start"]["ticks"], json!(["1000", "1000"]));
    assert_eq!(hole["rows"][0]["end"]["ticks"], json!(["2000", "1000"]));
    let outline = inspect("outline-segments:0");
    assert_eq!(outline["rows"][0]["start"]["ticks"], json!(["-1000", "-1000"]));
    assert_eq!(outline["rows"][0]["end"]["ticks"], json!(["15000", "-1000"]));
    let outline_hole = inspect("outline-cutout-segments:0:0");
    assert_eq!(outline_hole["rows"][0]["start"]["ticks"], json!(["5000", "5000"]));
    let stack = inspect("padstack-entries:0");
    let shapes = stack["rows"].as_array().unwrap();
    assert_eq!(shapes.len(), 5);
    let shape = |kind: &str| {
        &shapes.iter().find(|r| r["shape"]["kind"] == kind).unwrap()["shape"]
    };
    assert_eq!(shape("Circle")["diameter_m"], rational(1, 10000));
    assert_eq!(shape("Rectangle")["size_m"], json!([rational(1, 10000), rational(1, 5000)]));
    assert_eq!(shape("Annulus")["inner_diameter_m"], rational(1, 10000));
    assert_eq!(shape("Annulus")["outer_diameter_m"], rational(1, 5000));
    assert_eq!(shape("Thermal")["spoke_count"], 4);
    assert_eq!(shape("Thermal")["first_spoke_degrees"], 45);
    let custom_index = shapes.iter().position(|r| r["shape"]["kind"] == "Custom").unwrap();
    let custom = inspect(&format!("custom-pad-segments:0:{custom_index}"));
    assert_eq!(custom["total"], 4);
    assert_eq!(custom["rows"][0]["end"]["ticks"], json!(["100", "0"]));
    let via = inspect("vias");
    assert_eq!(via["rows"][0]["position"]["ticks"], json!(["8000", "8000"]));
    assert_eq!(via["rows"][0]["drill_details"]["diameter_m"], rational(1, 20000));
    assert_eq!(via["rows"][0]["drill_details"]["stored_plated"], true);
    assert_eq!(via["rows"][0]["stored_connected_layers"].as_array().unwrap().len(), 2);
    assert_eq!(inspect("pins")["rows"][0]["number"], "1");
    assert_eq!(inspect("components")["rows"][0]["reference"], "U1");
    assert!(!run(&path, "area-cutout-segments:0:99", "0", "1").status.success());
    assert!(!run(&path, "area-segments:0:0", "0", "1").status.success());
    assert_eq!(fs::read(&path).unwrap(), bytes);

    // Direct SQLite mutation of a COPY of the historical fixture, independent
    // of the production Writer and the Inspector's JSON serialization.
    let sql_path = dir.join("independent-sql.bsi");
    fs::copy(&path, &sql_path).unwrap();
    let conn = Connection::open(&sql_path).unwrap();
    let source: i64 = conn.query_row("SELECT id FROM source_identity", [], |r| r.get(0)).unwrap();
    let material_id: i64 = conn.query_row("SELECT MAX(id)+1 FROM objects", [], |r| r.get(0)).unwrap();
    conn.execute("INSERT INTO objects VALUES(?,?,'Material','independent-material','','fixture')", params![material_id,source]).unwrap();
    conn.execute("INSERT INTO materials VALUES(?,'Independent dielectric')", [material_id]).unwrap();
    for (key, state, value) in [("Dk", "Confirmed", 4.4_f64), ("Df", "Assumed", 0.015_f64)] {
        conn.execute("INSERT INTO material_properties(material_id,property_key,status,value_blob,document,revision,method,reference_frequency_hz,temperature_c) VALUES(?,?,?,?,'oracle','A','declared',1000000000,23)",
            params![material_id,key,state,value.to_le_bytes().to_vec()]).unwrap();
    }
    for key in ["Conductivity", "ValidBand"] {
        conn.execute("INSERT INTO material_properties(material_id,property_key,status) VALUES(?,?,'Missing')", params![material_id,key]).unwrap();
    }
    let layer_ids: Vec<i64> = conn.prepare("SELECT object_id FROM layers ORDER BY layer_order").unwrap().query_map([], |r| r.get(0)).unwrap().collect::<Result<_,_>>().unwrap();
    for (index, layer) in layer_ids.iter().enumerate() {
        let z_num = index as i128;
        let z_den = if index == 0 { 1_i128 } else { 10000_i128 };
        conn.execute("UPDATE layers SET z_status='Confirmed',z_num_le=?,z_den_le=?,thickness_status='Assumed',thickness_num_le=?,thickness_den_le=?,material_id=? WHERE object_id=?",
            params![z_num.to_le_bytes().to_vec(),z_den.to_le_bytes().to_vec(),1_i128.to_le_bytes().to_vec(),10000_i128.to_le_bytes().to_vec(),material_id,layer]).unwrap();
        for key in ["z_m", "thickness_m"] {
            conn.execute("INSERT INTO layer_property_sources(layer_id,property_key,document,revision,method) VALUES(?,?,'oracle','A','declared')", params![layer,key]).unwrap();
        }
    }
    let trace_id: i64 = conn.query_row("SELECT id FROM objects WHERE kind='Trace'", [], |r| r.get(0)).unwrap();
    conn.execute("INSERT INTO conversion_history(object_id,ordinal,operation,quality,note) VALUES(?,0,'independent-step','Exact','retained note')", [trace_id]).unwrap();
    drop(conn);
    let before = fs::read(&sql_path).unwrap();
    let total = page(&sql_path, "board", "0", "1");
    assert_eq!(total["rows"][0]["stackup_total_thickness_m"]["state"], "ASSUMED");
    assert_eq!(total["rows"][0]["stackup_total_thickness_m"]["value"], rational(1, 5000));
    let material = page(&sql_path, "materials", "0", "1");
    assert_eq!(material["rows"][0]["dk"]["value"], 4.4);
    assert_eq!(material["rows"][0]["dk"]["state"], "CONFIRMED");
    assert_eq!(material["rows"][0]["df"]["value"], 0.015);
    assert_eq!(material["rows"][0]["df"]["state"], "ASSUMED");
    assert_eq!(material["rows"][0]["df"]["source"]["frequency_hz"], 1_000_000_000.0);
    assert_eq!(material["rows"][0]["conductivity_s_per_m"]["state"], "MISSING");
    assert_eq!(page(&sql_path, "trace-history:0", "0", "1")["rows"][0]["note"], "retained note");
    let conn = Connection::open_with_flags(&sql_path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let raw: Vec<u8> = conn.query_row("SELECT value_blob FROM material_properties WHERE property_key='Dk'", [], |r| r.get(0)).unwrap();
    assert_eq!(f64::from_le_bytes(raw.try_into().unwrap()), 4.4);
    drop(conn);
    assert_eq!(fs::read(&sql_path).unwrap(), before);
    fs::remove_dir_all(dir).unwrap();
}
