#![cfg(feature = "f003-qualification")]
//! Real subprocess/process-limit qualification of the F003 single-file replace.
//! Never link these fault hooks into ordinary application builds.
use boardsi_model::{BoardKind, BoardModel};
use boardsi_native::{load, save};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "boardsi-f003-resilience-{}-{}-{label}.bsi",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ))
}
fn original(path: &Path) -> Vec<u8> {
    save(path, &BoardModel::empty(BoardKind::Pcb)).unwrap();
    let bytes = fs::read(path).unwrap();
    assert!(bytes.len() > 4096);
    bytes
}
fn assert_original(path: &Path, before: &[u8]) {
    assert_eq!(fs::read(path).unwrap(), before, "last good file was mutated");
    let loaded = load(path).expect("last good .bsi is still readable");
    assert!(loaded.model().traces.is_empty());
}
fn stages(pid: u32, parent: &Path) -> Vec<PathBuf> {
    let prefix = format!(".bsi-staging-{pid}-");
    fs::read_dir(parent)
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            entry.file_name().to_str()?.starts_with(&prefix).then_some(entry.path())
        })
        .collect()
}
fn child(dest: &Path, mode: &str, marker: &Path) -> Child {
    Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "qualification_child_write"])
        .env("BOARDSI_F003_QUAL_MODE", mode)
        .env("BOARDSI_F003_QUAL_MARKER", marker)
        .env("BOARDSI_F003_DEST", dest)
        .spawn()
        .unwrap()
}
#[test]
#[ignore]
fn qualification_child_write() {
    let file = PathBuf::from(std::env::var_os("BOARDSI_F003_DEST").unwrap());
    save(&file, &BoardModel::empty(BoardKind::Pcb))
        .expect("CI subprocess intentionally performs native save");
}
#[test]
fn injected_pre_replace_io_failure_preserves_original_and_cleans_stage() {
    let dest = path("inject-error");
    let before = original(&dest);
    let marker = path("marker-error");
    let mut worker = child(&dest, "fail", &marker);
    let status = worker.wait().unwrap();
    assert!(!status.success(), "injected I/O failure must reject save");
    assert!(marker.exists(), "failure was reached after staging validation");
    assert_original(&dest, &before);
    assert!(stages(worker.id(), dest.parent().unwrap()).is_empty(), "failed save left staged file");
    fs::remove_file(&marker).unwrap();
    fs::remove_file(&dest).unwrap();
}
#[test]
fn force_killed_worker_before_replace_preserves_last_good_file() {
    let dest = path("kill-before-replace");
    let before = original(&dest);
    let marker = path("marker-kill");
    let mut worker = child(&dest, "pause", &marker);
    let start = Instant::now();
    while !marker.exists() {
        if let Some(status) = worker.try_wait().unwrap() {
            panic!("worker finished before kill point: {status}");
        }
        assert!(
            start.elapsed() < Duration::from_secs(40),
            "worker did not reach validated staging within timeout"
        );
        std::thread::sleep(Duration::from_millis(30));
    }
    let leftovers = stages(worker.id(), dest.parent().unwrap());
    assert_eq!(leftovers.len(), 1, "one validated temporary file expected");
    assert!(load(&leftovers[0]).is_ok(), "temporary DB must be independently readable");
    worker.kill().expect("terminate qualification process");
    let status = worker.wait().unwrap();
    assert!(!status.success(), "forced termination must not report success");
    assert_original(&dest, &before);
    for stage in leftovers {
        fs::remove_file(stage).unwrap();
    }
    fs::remove_file(&marker).unwrap();
    fs::remove_file(&dest).unwrap();
}
#[test]
fn concurrent_sqlite_exclusive_lock_rejects_overwrite_and_preserves_file() {
    let dest = path("exclusive-lock");
    let before = original(&dest);
    let lock = rusqlite::Connection::open(&dest).unwrap();
    lock.execute_batch("BEGIN EXCLUSIVE").unwrap();
    assert!(save(&dest, &BoardModel::empty(BoardKind::Pcb)).is_err());
    lock.execute_batch("ROLLBACK").unwrap();
    drop(lock);
    assert_original(&dest, &before);
    fs::remove_file(&dest).unwrap();
}
#[cfg(unix)]
#[test]
fn real_process_file_size_quota_failure_preserves_last_good_file() {
    // This is a real OS resource limit, not a power failure or exhausted disk.
    let dest = path("os-quota");
    let before = original(&dest);
    let child_binary = std::env::current_exe().unwrap();
    let status = Command::new("/bin/sh")
        .arg("-c")
        .arg("ulimit -c 0; ulimit -f 8; exec \"$@\"")
        .arg("f003")
        .arg(child_binary)
        .args(["--ignored", "--exact", "qualification_child_write"])
        .env("BOARDSI_F003_DEST", &dest)
        .status()
        .unwrap();
    assert!(!status.success(), "file-size limit must make replacement fail");
    assert_original(&dest, &before);
    fs::remove_file(&dest).unwrap();
}
#[cfg(target_os = "linux")]
#[test]
fn sparse_oversized_file_rejected_before_sqlite_parsing() {
    let oversized = path("oversized");
    let file = File::create(&oversized).unwrap();
    file.set_len(512 * 1024 * 1024 + 1).unwrap();
    drop(file);
    assert_eq!(load(&oversized).err().unwrap().code(), "BSI_INVALID");
    fs::remove_file(&oversized).unwrap();
}
