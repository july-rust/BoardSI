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
    sync::atomic::{AtomicU64, Ordering},
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
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
fn temp() -> PathBuf {
    // Windows' wall clock may have a coarser granularity than the parallel
    // Rust test harness. A monotonic process-local nonce plus an exclusive
    // create avoids identical PID/timestamp names and stale fixture reuse.
    for _ in 0..64 {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("boardsi-f006-{}-{n}-{serial}", std::process::id()));
        match fs::create_dir(&path) {
            Ok(()) => return path,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => panic!("F006 temporary test fixture cannot be created: {e}"),
        }
    }
    panic!("F006 unique temporary fixture name exhausted");
}

fn independent_sqlite_bsig_oracle(path: &std::path::Path, hash: &str) {
    use rusqlite::{Connection, OpenFlags};
    // Deliberately independent from the BoardSI Native Reader and BSIG
    // encoder: query raw SQLite and parse little-endian envelope bytes.
    let db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let app: i64 = db
        .pragma_query_value(None, "application_id", |r| r.get(0))
        .unwrap();
    let ver: i64 = db
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!((app, ver), (0x42534931, 1));
    let raw_sha: Vec<u8> = db
        .query_row("SELECT sha256 FROM source_files WHERE ordinal=0", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(
        raw_sha.as_slice(),
        Sha256::digest(SOURCE.as_bytes()).as_slice()
    );
    assert_eq!(hex_lower(&raw_sha), hash);
    let blob: Vec<u8> = db
        .query_row(
            "SELECT envelope FROM geometry WHERE geometry_kind=1 LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(blob.len() > 52 && &blob[0..4] == b"BSIG");
    assert_eq!(u16::from_le_bytes(blob[4..6].try_into().unwrap()), 1);
    assert_eq!(u16::from_le_bytes(blob[6..8].try_into().unwrap()), 1);
    assert_eq!(
        u64::from_le_bytes(blob[12..20].try_into().unwrap()) as usize,
        blob.len() - 52
    );
    assert_eq!(&blob[20..52], Sha256::digest(&blob[52..]).as_slice());
    let body = &blob[52..];
    assert_eq!(u16::from_le_bytes(body[4..6].try_into().unwrap()), 1);
    assert_eq!(u16::from_le_bytes(body[6..8].try_into().unwrap()), 7);
    assert_eq!(u32::from_le_bytes(body[12..16].try_into().unwrap()), 2);
    let read_point = |pos: usize| -> (i64, i64) {
        (
            i64::from_le_bytes(body[pos + 4..pos + 12].try_into().unwrap()),
            i64::from_le_bytes(body[pos + 12..pos + 20].try_into().unwrap()),
        )
    };
    assert_eq!(read_point(16), (1_123_456, -2_000_001));
    assert_eq!(read_point(36), (3_000_001, 5_000_000));
}
fn hex_lower(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    use std::fmt::Write as _;
    for byte in bytes {
        write!(&mut output, "{byte:02x}").unwrap();
    }
    output
}

#[test]
fn f006_parallel_temp_names_never_collide() {
    let mut workers = Vec::new();
    for _ in 0..24 {
        workers.push(std::thread::spawn(temp));
    }
    let directories = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect::<Vec<_>>();
    let unique = directories
        .iter()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique.len(), directories.len());
    for directory in directories {
        fs::remove_dir_all(directory).unwrap();
    }
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
    independent_sqlite_bsig_oracle(&target, &hash);
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

#[cfg(feature = "f004-qualification")]
#[test]
fn f006_real_source_mutation_during_staging_aborts_before_publish() {
    use std::{process::Stdio, thread, time::Duration};
    let dir = temp();
    let source = dir.join("candidate.kicad_pcb");
    let target = dir.join("previous.bsi");
    fs::write(&source, SOURCE).unwrap();
    let hash = format!("{:x}", Sha256::digest(SOURCE.as_bytes()));
    assert!(invoke(&[
        std::ffi::OsStr::new("save-kicad"),
        source.as_os_str(),
        target.as_os_str(),
        std::ffi::OsStr::new(&hash),
    ])
    .status
    .success());
    let good = fs::read(&target).unwrap();
    let marker = dir.join("f006-pre-publish.marker");
    let resume = marker.with_extension("resume");
    let mut worker = Command::new(env!("CARGO_BIN_EXE_boardsi-native-file"))
        .arg("save-kicad")
        .arg(&source)
        .arg(&target)
        .arg(&hash)
        .env("BOARDSI_F003_QUAL_MODE", "pause-until-resume")
        .env("BOARDSI_F003_QUAL_MARKER", &marker)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    for _ in 0..200 {
        if marker.exists() || worker.try_wait().unwrap().is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    if !marker.exists() {
        let _ = worker.kill();
        let _ = worker.wait();
        panic!("F006 staged native source guard marker was not reached");
    }
    fs::write(
        &source,
        SOURCE.replace("(net 1 \"CLK\")", "(net 1 \"MUTATED\")"),
    )
    .unwrap();
    fs::write(resume, b"resume").unwrap();
    let result = worker.wait_with_output().unwrap();
    assert!(
        !result.status.success()
            && String::from_utf8_lossy(&result.stderr).contains("BSI_SOURCE_CHANGED"),
        "F006 source changed before publication: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(fs::read(&target).unwrap(), good);
    assert!(load(&target).is_ok());
    fs::remove_dir_all(dir).unwrap();
}
