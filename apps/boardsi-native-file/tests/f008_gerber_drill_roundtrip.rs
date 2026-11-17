//! F_008: actual Worker, native SQLite v1, independent binary/source Oracle.
use boardsi_import::gerber_excellon_source::into_native_canonical;
use boardsi_native::{canonical_semantics_equal, load};
use rusqlite::{Connection, OpenFlags};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
const GERBER: &str="%FSLAX36Y36*%\n%MOMM*%\n%TF.FileFunction,Copper,L1,Top*%\n%ADD10C,0.600*%\n%LPD*%\nD10*\nX012345678Y002000000D03*\nM02*\n";
const DRILL: &str = "M48\nMETRIC\nT01C0.300\n%\nT01\nX12.345678Y2.000000\nM30\n";
static NONCE: AtomicU64 = AtomicU64::new(0);
struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Self {
        for _ in 0..128 {
            let ns = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let serial = NONCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("boardsi-f008-{}-{ns}-{serial}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("create temp: {e}"),
            }
        }
        panic!("temp dir collision");
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn worker() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_boardsi-native-file")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_boardsi-native-file")))
}
fn invoke(args: &[&str]) -> std::process::Output {
    Command::new(worker()).args(args).output().unwrap()
}
fn hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
#[test]
fn f008_native_roundtrip_exact_sql_extension_and_save_as() {
    let dir = TempDir::new();
    let g = dir.0.join("top.gbr");
    let d = dir.0.join("drill.drl");
    let b = dir.0.join("native.bsi");
    let copy = dir.0.join("copy.bsi");
    fs::write(&g, GERBER).unwrap();
    fs::write(&d, DRILL).unwrap();
    let sha_g = hex(GERBER.as_bytes());
    let sha_d = hex(DRILL.as_bytes());
    let g = g.to_str().unwrap();
    let d = d.to_str().unwrap();
    let b = b.to_str().unwrap();
    let copy = copy.to_str().unwrap();
    let run = invoke(&["save-gerber-drill", g, d, b, &sha_g, &sha_d]);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&run.stdout).unwrap();
    assert_eq!(receipt["source_format"], "GERBER_X2_EXCELLON_SOURCE_SUBSET");
    assert_eq!(receipt["drill_observations"], 1);
    assert_eq!(receipt["physical_si"], "NOT_RUN");
    let expected = into_native_canonical(GERBER.as_bytes(), DRILL.as_bytes()).unwrap();
    let loaded = load(b).unwrap();
    assert!(!loaded.is_read_only());
    assert!(canonical_semantics_equal(&expected, loaded.model()));
    assert!(loaded.model().vias.is_empty() && loaded.model().drills.is_empty());
    assert!(loaded.model().connectivity.edges.is_empty());
    let holes = &loaded.drill_evidence().unwrap().holes;
    assert_eq!(holes.len(), 1);
    assert_eq!(holes[0].centre_nm, [12_345_678, 2_000_000]);
    assert_eq!(holes[0].diameter_nm, 300_000);
    assert_eq!(holes[0].tool_id, 1);
    assert!(holes[0].source_offset > 0);
    let opened = invoke(&["open", b]);
    assert!(opened.status.success());
    let open: serde_json::Value = serde_json::from_slice(&opened.stdout).unwrap();
    assert_eq!(open["source_format"], "GERBER_X2_EXCELLON_SOURCE_SUBSET");
    assert_eq!(open["artwork"]["items"].as_array().unwrap().len(), 2);
    assert_eq!(open["artwork"]["alignment_verified"], false);
    // Independent SQL and byte layout inspection; no Native Reader helpers.
    let conn = Connection::open_with_flags(b, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let row:(i64,i64,Vec<u8>)=conn.query_row(
        "SELECT extension_version,required,payload FROM extensions WHERE extension_name='org.boardsi.source.gerber-excellon-drill'",
        [],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!((row.0, row.1), (1, 1));
    let payload = &row.2;
    assert_eq!(&payload[0..8], b"BDRL0001");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(
        &payload[12..44],
        Sha256::digest(DRILL.as_bytes()).as_slice()
    );
    assert_eq!(
        i64::from_le_bytes(payload[44..52].try_into().unwrap()),
        12_345_678
    );
    assert_eq!(
        i64::from_le_bytes(payload[52..60].try_into().unwrap()),
        2_000_000
    );
    assert_eq!(
        i64::from_le_bytes(payload[60..68].try_into().unwrap()),
        300_000
    );
    assert_eq!(u32::from_le_bytes(payload[68..72].try_into().unwrap()), 1);
    assert_eq!(
        &payload[payload.len() - 32..],
        Sha256::digest(&payload[..payload.len() - 32]).as_slice()
    );
    // Existing v1 BSIG shape is preserved (canonical Pad and Padstack records).
    let geom_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM geometry", [], |r| r.get(0))
        .unwrap();
    assert_eq!(geom_count, 2);
    // Independently decode the Pad BSIG Point TLV; no Native Reader/helpers.
    let pad_blob: Vec<u8> = conn
        .query_row(
            "SELECT envelope FROM geometry WHERE geometry_kind=6",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(&pad_blob[0..4], b"BSIG");
    assert_eq!(u16::from_le_bytes(pad_blob[6..8].try_into().unwrap()), 6);
    assert_eq!(
        &pad_blob[20..52],
        Sha256::digest(&pad_blob[52..]).as_slice()
    );
    let pad = &pad_blob[52..];
    assert_eq!(u16::from_le_bytes(pad[0..2].try_into().unwrap()), 1);
    assert_eq!(u16::from_le_bytes(pad[4..6].try_into().unwrap()), 1);
    assert_eq!(u16::from_le_bytes(pad[6..8].try_into().unwrap()), 6);
    assert_eq!(u32::from_le_bytes(pad[8..12].try_into().unwrap()), 20);
    assert_eq!(
        i64::from_le_bytes(pad[16..24].try_into().unwrap()),
        12_345_678
    );
    assert_eq!(
        i64::from_le_bytes(pad[24..32].try_into().unwrap()),
        2_000_000
    );
    // Independently inspect Padstack Circle Rational128: 0.600000 mm = 3/5000 m.
    let stack_blob: Vec<u8> = conn
        .query_row(
            "SELECT envelope FROM geometry WHERE geometry_kind=5",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(&stack_blob[0..4], b"BSIG");
    assert_eq!(
        &stack_blob[20..52],
        Sha256::digest(&stack_blob[52..]).as_slice()
    );
    let stack = &stack_blob[52..];
    assert_eq!(u16::from_le_bytes(stack[4..6].try_into().unwrap()), 1);
    assert_eq!(u16::from_le_bytes(stack[6..8].try_into().unwrap()), 11);
    assert_eq!(u32::from_le_bytes(stack[12..16].try_into().unwrap()), 1);
    assert_eq!((stack[20], stack[21]), (0, 0));
    assert_eq!(u32::from_le_bytes(stack[24..28].try_into().unwrap()), 32);
    assert_eq!(i128::from_le_bytes(stack[28..44].try_into().unwrap()), 3);
    assert_eq!(i128::from_le_bytes(stack[44..60].try_into().unwrap()), 5000);
    drop(conn);
    let copied = invoke(&["save-loaded", b, copy, &hex(&fs::read(b).unwrap())]);
    assert!(
        copied.status.success(),
        "{}",
        String::from_utf8_lossy(&copied.stderr)
    );
    let orig = load(b).unwrap();
    let clone = load(copy).unwrap();
    assert_eq!(orig.drill_evidence(), clone.drill_evidence());
    assert!(canonical_semantics_equal(orig.model(), clone.model()));
}

#[test]
fn f008_bad_source_and_changed_sha_do_not_clobber_existing_native() {
    let dir = TempDir::new();
    let g = dir.0.join("top.gbr");
    let d = dir.0.join("drill.drl");
    let b = dir.0.join("native.bsi");
    fs::write(&g, GERBER).unwrap();
    fs::write(&d, DRILL).unwrap();
    let sha_g = hex(GERBER.as_bytes());
    let sha_d = hex(DRILL.as_bytes());
    let g = g.to_str().unwrap();
    let d = d.to_str().unwrap();
    let b = b.to_str().unwrap();
    assert!(invoke(&["save-gerber-drill", g, d, b, &sha_g, &sha_d])
        .status
        .success());
    let before = fs::read(b).unwrap();
    fs::write(d, DRILL.replace("0.300", "0.400")).unwrap();
    let bad = invoke(&["save-gerber-drill", g, d, b, &sha_g, &sha_d]);
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("BSI_SOURCE_CHANGED"));
    assert_eq!(fs::read(b).unwrap(), before);
    let changed_sha = hex(&fs::read(d).unwrap());
    let bad_gerber = GERBER.replace("D03", "D01");
    fs::write(g, &bad_gerber).unwrap();
    let rejected = invoke(&[
        "save-gerber-drill",
        g,
        d,
        b,
        &hex(bad_gerber.as_bytes()),
        &changed_sha,
    ]);
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("BSI_F008_SOURCE_UNSUPPORTED"));
    assert_eq!(fs::read(b).unwrap(), before);
}

#[test]
fn f008_corrupt_required_extension_fails_closed() {
    let dir = TempDir::new();
    let g = dir.0.join("top.gbr");
    let d = dir.0.join("drill.drl");
    let b = dir.0.join("native.bsi");
    fs::write(&g, GERBER).unwrap();
    fs::write(&d, DRILL).unwrap();
    let sha_g = hex(GERBER.as_bytes());
    let sha_d = hex(DRILL.as_bytes());
    assert!(invoke(&[
        "save-gerber-drill",
        g.to_str().unwrap(),
        d.to_str().unwrap(),
        b.to_str().unwrap(),
        &sha_g,
        &sha_d
    ])
    .status
    .success());
    let conn = Connection::open(&b).unwrap();
    conn.execute("UPDATE extensions SET payload=x'0001'", [])
        .unwrap();
    drop(conn);
    assert!(load(&b).is_err());
}

#[test]
fn f008_malformed_source_frame_fails_before_qt_preview() {
    let dir = TempDir::new();
    let g = dir.0.join("top.gbr");
    let d = dir.0.join("drill.drl");
    let b = dir.0.join("native.bsi");
    fs::write(&g, GERBER).unwrap();
    fs::write(&d, DRILL).unwrap();
    let gs = hex(GERBER.as_bytes());
    let ds = hex(DRILL.as_bytes());
    assert!(invoke(&[
        "save-gerber-drill",
        g.to_str().unwrap(),
        d.to_str().unwrap(),
        b.to_str().unwrap(),
        &gs,
        &ds
    ])
    .status
    .success());
    let conn = Connection::open(&b).unwrap();
    conn.execute("UPDATE coordinate_frames SET quarter_turn=1", [])
        .unwrap();
    drop(conn);
    // A structurally valid rotated frame cannot be rendered as raw nm XY.
    assert!(load(&b).is_err());
    let opened = invoke(&["open", b.to_str().unwrap()]);
    assert!(!opened.status.success());
}
