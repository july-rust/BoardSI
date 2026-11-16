//! F_007 IPC-2581C bounded source -> Canonical -> .bsi -> Canonical.
//! Does not certify full IPC XSD, manufactured connectivity or SI.
use boardsi_import::ipc2581c_source::into_native_canonical;
use boardsi_native::{canonical_semantics_equal, load};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const SOURCE: &str = r#"<?xml version="1.0"?>
<IPC-2581 revision="C" xmlns="http://webstds.ipc.org/2581">
<Content><LayerRef name="TOP"/><LayerRef name="BOTTOM"/></Content>
<LogisticHeader/><HistoryRecord/>
<Ecad><CadHeader units="MILLIMETER"/><CadData>
<Layer name="TOP" layerFunction="SIGNAL" side="TOP" polarity="POSITIVE"/>
<Layer name="BOTTOM" layerFunction="SIGNAL" side="BOTTOM" polarity="POSITIVE"/>
<Step name="BOARD"><LogicalNet name="SIG"><PinRef componentRef="U1" pin="1"/></LogicalNet>
<LayerFeature layerRef="TOP"><Set net="SIG">
<Features><Line startX="1.123456" startY="-2.000001" endX="3" endY="5">
<LineDesc lineEnd="ROUND" lineProperty="SOLID" lineWidth="0.15"/></Line></Features>
<Features><Polyline><PolyBegin x="3" y="5"/><PolyStepSegment x="4" y="6"/>
<PolyStepSegment x="5" y="6"/><LineDesc lineEnd="ROUND" lineWidth="0.2"/></Polyline></Features>
</Set></LayerFeature></Step></CadData></Ecad></IPC-2581>"#;

static NONCE: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Self {
        for _ in 0..128 {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let serial = NONCE.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!(
                "boardsi-f007-{}-{nanos}-{serial}",
                std::process::id(),
            ));
            match fs::create_dir(&dir) {
                Ok(()) => return Self(dir),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("temporary dir: {e}"),
            }
        }
        panic!("no unique temporary directory");
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

#[test]
fn native_ipc_source_exact_save_open_oracle_and_stale_guard() {
    let tmp = TempDir::new();
    let source = tmp.0.join("input.xml");
    let target = tmp.0.join("qualified.bsi");
    let invalid = tmp.0.join("stale-rejected.bsi");
    fs::write(&source, SOURCE).unwrap();
    let sha = format!("{:x}", Sha256::digest(SOURCE.as_bytes()));
    let expected = into_native_canonical(SOURCE.as_bytes()).unwrap();
    assert_eq!(expected.traces.len(), 2);
    let save = Command::new(worker())
        .args([
            "save-ipc2581",
            source.to_str().unwrap(),
            target.to_str().unwrap(),
            &sha,
        ])
        .output()
        .unwrap();
    assert!(
        save.status.success(),
        "save error: {}",
        String::from_utf8_lossy(&save.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&save.stdout).unwrap();
    assert_eq!(receipt["action"], "save-ipc2581");
    assert_eq!(receipt["source_sha256"], sha);
    assert_eq!(receipt["source_format"], "IPC2581C_SOURCE_SUBSET");
    assert_eq!(receipt["physical_si"], "NOT_RUN");
    assert_eq!(receipt["traces"], 2);
    let loaded = load(&target).unwrap();
    assert!(canonical_semantics_equal(&expected, loaded.model()));
    assert!(!loaded.is_read_only());
    let open = Command::new(worker())
        .args(["open", target.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(open.status.success());
    let read: serde_json::Value = serde_json::from_slice(&open.stdout).unwrap();
    assert_eq!(read["source_format"], "IPC2581C_SOURCE_SUBSET");
    assert_eq!(read["artwork"]["items"].as_array().unwrap().len(), 3);
    assert_eq!(read["artwork"]["units"], "integer-nm");

    // Independent SQL/BSIG format inspection; do not reuse Rust native Reader.
    use rusqlite::{Connection, OpenFlags};
    let db = Connection::open_with_flags(&target, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let raw_sha: Vec<u8> = db
        .query_row(
            "SELECT sha256 FROM source_files WHERE ordinal=0",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(raw_sha, Sha256::digest(SOURCE.as_bytes()).as_slice());
    let blob: Vec<u8> = db
        .query_row(
            "SELECT envelope FROM geometry WHERE geometry_kind=1 LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(&blob[0..4], b"BSIG");
    assert_eq!(u16::from_le_bytes([blob[4], blob[5]]), 1);
    assert_eq!(&blob[20..52], Sha256::digest(&blob[52..]).as_slice());

    // Detect an actual on-disk mutation, and never publish the stale target.
    fs::write(&source, SOURCE.replace("SIG", "CHANGED")).unwrap();
    let stale = Command::new(worker())
        .args([
            "save-ipc2581",
            source.to_str().unwrap(),
            invalid.to_str().unwrap(),
            &sha,
        ])
        .output()
        .unwrap();
    assert!(!stale.status.success());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("BSI_SOURCE_CHANGED"));
    assert!(!invalid.exists());
    assert!(canonical_semantics_equal(
        &expected,
        load(&target).unwrap().model()
    ));
}

#[test]
fn native_rejects_unsupported_geometry_without_clobber() {
    let tmp = TempDir::new();
    let source = tmp.0.join("input.xml");
    let target = tmp.0.join("existing.bsi");
    fs::write(&source, SOURCE).unwrap();
    let sha = format!("{:x}", Sha256::digest(SOURCE.as_bytes()));
    let ok = Command::new(worker())
        .args([
            "save-ipc2581",
            source.to_str().unwrap(),
            target.to_str().unwrap(),
            &sha,
        ])
        .output()
        .unwrap();
    assert!(ok.status.success());
    let before = fs::read(&target).unwrap();
    let malicious = SOURCE.replace("</Set>", "<Pad padstackDefRef=\"P\"/></Set>");
    fs::write(&source, &malicious).unwrap();
    let malicious_sha = format!("{:x}", Sha256::digest(malicious.as_bytes()));
    let rejected = Command::new(worker())
        .args([
            "save-ipc2581",
            source.to_str().unwrap(),
            target.to_str().unwrap(),
            &malicious_sha,
        ])
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("BSI_IPC2581_SOURCE_UNSUPPORTED"));
    assert_eq!(before, fs::read(&target).unwrap());
}
