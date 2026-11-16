//! Actual F004 file worker process tests. Enable f004-qualification explicitly
//! to exercise real process termination without touching the production binary.
use boardsi_native::{load, recover_owned_staging};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(feature = "f004-qualification")]
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "f004-qualification")]
use std::time::Duration;

static NEXT: AtomicU64 = AtomicU64::new(0);

fn folder(label: &str) -> PathBuf {
    let name = format!(
        "boardsi-f004-worker-{}-{}-{}",
        label,
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let path = std::env::temp_dir().join(name);
    fs::create_dir(&path).unwrap();
    path
}
fn source_in(folder: &Path) -> PathBuf {
    let path = folder.join("source.bsi");
    fs::write(
        &path,
        include_bytes!("../../../schemas/bsi/golden-v1-empty.bsi"),
    )
    .unwrap();
    path
}
fn worker() -> Command {
    Command::new(env!("CARGO_BIN_EXE_boardsi-native-file"))
}

#[test]
fn exact_byte_copy_and_noop_are_idempotent_and_never_clobber() {
    let dir = folder("identity");
    let source = source_in(&dir);
    let target = dir.join("copy.bsi");
    let original = fs::read(&source).unwrap();
    let self_report = worker()
        .args([
            "migrate-v1",
            source.to_str().unwrap(),
            source.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(self_report.status.success(), "{:?}", self_report.stderr);
    assert!(String::from_utf8_lossy(&self_report.stdout).contains("\"copied\":false"));
    let first = worker()
        .args([
            "migrate-v1",
            source.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(first.status.success(), "{:?}", first.stderr);
    assert!(String::from_utf8_lossy(&first.stdout).contains("\"steps\":0"));
    assert_eq!(fs::read(&source).unwrap(), original);
    assert_eq!(fs::read(&target).unwrap(), original);
    assert_eq!(
        load(&target).unwrap().model(),
        load(&source).unwrap().model()
    );

    let second = worker()
        .args([
            "migrate-v1",
            source.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!second.status.success(), "must refuse existing destination");
    assert_eq!(fs::read(&target).unwrap(), original);
    assert_eq!(fs::read(&source).unwrap(), original);
    assert_eq!(recover_owned_staging(&dir).unwrap(), 0);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn read_only_source_permissions_do_not_make_stage_unwritable() {
    let dir = folder("read-only-source");
    let source = source_in(&dir);
    let target = dir.join("target.bsi");
    let original = fs::read(&source).unwrap();
    let permissions = fs::metadata(&source).unwrap().permissions();
    let mut readonly = permissions.clone();
    readonly.set_readonly(true);
    fs::set_permissions(&source, readonly).unwrap();
    let result = worker()
        .args([
            "migrate-v1",
            source.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    fs::set_permissions(&source, permissions).unwrap();
    assert!(result.status.success(), "{:?}", result.stderr);
    assert_eq!(fs::read(&source).unwrap(), original);
    assert_eq!(fs::read(&target).unwrap(), original);
    assert_eq!(recover_owned_staging(&dir).unwrap(), 0);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn unknown_optional_and_invalid_v1_cannot_be_migrated() {
    let dir = folder("readonly");
    let source = source_in(&dir);
    let target = dir.join("target.bsi");
    let conn = Connection::open(&source).unwrap();
    conn.execute_batch(
        "INSERT INTO extensions(extension_name,extension_version,required,payload) \
         VALUES('new-unknown',1,0,X'DEADBEEF');",
    )
    .unwrap();
    drop(conn);
    let before = fs::read(&source).unwrap();
    let result = worker()
        .args([
            "migrate-v1",
            source.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!target.exists());
    assert_eq!(fs::read(&source).unwrap(), before);
    let conn = Connection::open(&source).unwrap();
    conn.execute_batch("PRAGMA user_version=2;").unwrap();
    drop(conn);
    let future = fs::read(&source).unwrap();
    let result = worker()
        .args([
            "migrate-v1",
            source.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!target.exists());
    assert_eq!(fs::read(&source).unwrap(), future);
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(feature = "f004-qualification")]
#[test]
fn injected_io_failure_preserves_source_and_does_not_publish() {
    let dir = folder("fault");
    let source = source_in(&dir);
    let target = dir.join("target.bsi");
    let marker = dir.join("fault-marker");
    let before = fs::read(&source).unwrap();
    let output = worker()
        .args([
            "migrate-v1",
            source.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .env("BOARDSI_F004_QUAL_MODE", "fail")
        .env("BOARDSI_F004_QUAL_MARKER", &marker)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read(&marker).unwrap(), b"f004-validated-staging");
    assert!(!target.exists());
    assert_eq!(fs::read(&source).unwrap(), before);
    assert_eq!(recover_owned_staging(&dir).unwrap(), 0);
    fs::remove_dir_all(dir).unwrap();
}

#[cfg(feature = "f004-qualification")]
#[test]
fn actual_worker_kill_leaves_recoverable_locked_staging_and_source_intact() {
    let dir = folder("kill");
    let source = source_in(&dir);
    let target = dir.join("target.bsi");
    let marker = dir.join("pause-marker");
    let before = fs::read(&source).unwrap();
    let mut child = worker()
        .args([
            "migrate-v1",
            source.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .env("BOARDSI_F004_QUAL_MODE", "pause")
        .env("BOARDSI_F004_QUAL_MARKER", &marker)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut reached = false;
    for _ in 0..200 {
        if marker.exists() {
            reached = true;
            break;
        }
        if let Some(status) = child.try_wait().unwrap() {
            panic!("worker exited before pause marker: {status}");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(
        reached,
        "migration did not reach validated staging faultpoint"
    );
    assert!(!target.exists());
    assert_eq!(fs::read(&source).unwrap(), before);
    assert_eq!(recover_owned_staging(&dir).unwrap(), 1);
    assert_eq!(recover_owned_staging(&dir).unwrap(), 0);
    let normal = worker()
        .args([
            "migrate-v1",
            source.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(normal.status.success(), "{:?}", normal.stderr);
    assert_eq!(fs::read(&target).unwrap(), before);
    assert_eq!(fs::read(&source).unwrap(), before);
    fs::remove_dir_all(dir).unwrap();
}
