#![cfg(feature = "f003-qualification")]
//! F004 process-boundary crash and real I/O qualification, never release hooks.
use boardsi_model::{BoardKind, BoardModel};
use boardsi_native::{load, migrate_to_current, recover_owned_staging, save};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn unique(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "boardsi-f004-resilience-{}-{}-{label}.bsi",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
fn fixture(p: &Path, kind: BoardKind) -> Vec<u8> {
    save(p, &BoardModel::empty(kind)).unwrap();
    fs::read(p).unwrap()
}
fn children_stage(pid: u32, dir: &Path) -> Vec<PathBuf> {
    let prefix = format!(".bsi-staging-{pid}-");
    fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| {
            let e = e.ok()?;
            e.file_name()
                .to_str()?
                .starts_with(&prefix)
                .then_some(e.path())
        })
        .collect()
}
fn child(src: &Path, dst: &Path, mode: &str, marker: &Path) -> Child {
    Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "f004_child_copy"])
        .env("BOARDSI_F004_SOURCE", src)
        .env("BOARDSI_F004_DEST", dst)
        .env("BOARDSI_F004_QUAL_MODE", mode)
        .env("BOARDSI_F004_QUAL_MARKER", marker)
        .spawn()
        .unwrap()
}
#[test]
#[ignore]
fn f004_child_copy() {
    let src = PathBuf::from(std::env::var_os("BOARDSI_F004_SOURCE").unwrap());
    let dst = PathBuf::from(std::env::var_os("BOARDSI_F004_DEST").unwrap());
    migrate_to_current(&src, &dst).expect("qualifier may block or fail before replace");
}
#[test]
fn process_kill_preserves_both_files_and_recovers_orphan() {
    let dir = unique("kill-dir");
    fs::create_dir(&dir).unwrap();
    let src = dir.join("old.bsi");
    let dst = dir.join("existing.bsi");
    let original_src = fixture(&src, BoardKind::Pcb);
    let original_dst = fixture(&dst, BoardKind::BgaSubstrate);
    let marker = dir.join("pause-marker");
    let mut worker = child(&src, &dst, "pause", &marker);
    let start = Instant::now();
    while !marker.exists() {
        assert!(
            worker.try_wait().unwrap().is_none(),
            "migration child exited early"
        );
        assert!(
            start.elapsed() < Duration::from_secs(40),
            "migration stage timeout"
        );
        std::thread::sleep(Duration::from_millis(30));
    }
    let live = children_stage(worker.id(), &dir);
    assert_eq!(
        live.iter()
            .filter(|p| !p.to_string_lossy().ends_with(".owner"))
            .count(),
        1
    );
    assert_eq!(recover_owned_staging(&dir).unwrap(), 0, "active owner lock");
    worker.kill().unwrap();
    assert!(!worker.wait().unwrap().success());
    assert_eq!(fs::read(&src).unwrap(), original_src);
    assert_eq!(fs::read(&dst).unwrap(), original_dst);
    assert_eq!(recover_owned_staging(&dir).unwrap(), 1, "orphan cleanup");
    assert!(children_stage(worker.id(), &dir).is_empty());
    assert_eq!(
        recover_owned_staging(&dir).unwrap(),
        0,
        "idempotent cleanup"
    );
    assert!(load(&dst).is_ok(), "last-good target must remain readable");
    fs::remove_file(src).unwrap();
    fs::remove_file(dst).unwrap();
    fs::remove_file(marker).unwrap();
    fs::remove_dir(dir).unwrap();
}
#[test]
fn injected_failure_cleans_stage_without_replacing_existing() {
    let dir = unique("fail-dir");
    fs::create_dir(&dir).unwrap();
    let src = dir.join("old.bsi");
    let dst = dir.join("existing.bsi");
    let source = fixture(&src, BoardKind::Pcb);
    let dest = fixture(&dst, BoardKind::BgaSubstrate);
    let marker = dir.join("fail-marker");
    let mut worker = child(&src, &dst, "fail", &marker);
    assert!(!worker.wait().unwrap().success());
    assert!(
        marker.exists(),
        "failure was reached after independent validation"
    );
    assert!(children_stage(worker.id(), &dir).is_empty());
    assert_eq!(fs::read(&src).unwrap(), source);
    assert_eq!(fs::read(&dst).unwrap(), dest);
    fs::remove_file(src).unwrap();
    fs::remove_file(dst).unwrap();
    fs::remove_file(marker).unwrap();
    fs::remove_dir(dir).unwrap();
}
#[test]
fn destination_exclusive_sqlite_lock_does_not_replace_file() {
    let src = unique("lock-source");
    let dst = unique("lock-target");
    let source = fixture(&src, BoardKind::Pcb);
    let dest = fixture(&dst, BoardKind::BgaSubstrate);
    let conn = Connection::open(&dst).unwrap();
    conn.execute_batch("BEGIN EXCLUSIVE").unwrap();
    assert!(migrate_to_current(&src, &dst).is_err());
    conn.execute_batch("ROLLBACK").unwrap();
    drop(conn);
    assert_eq!(fs::read(&src).unwrap(), source);
    assert_eq!(fs::read(&dst).unwrap(), dest);
    fs::remove_file(src).unwrap();
    fs::remove_file(dst).unwrap();
}
#[cfg(target_os = "linux")]
#[test]
fn actual_tmpfs_enospc_preserves_last_good_migration_target() {
    let Some(root) = std::env::var_os("F004_ENOSPC_DIR") else {
        return;
    };
    let dir = PathBuf::from(root);
    let src = unique("enospc-source");
    fixture(&src, BoardKind::Pcb);
    let dest = dir.join("last-good.bsi");
    let before = fixture(&dest, BoardKind::BgaSubstrate);
    let filler = dir.join("filler");
    let status = Command::new("dd")
        .arg("if=/dev/zero")
        .arg(format!("of={}", filler.display()))
        .args(["bs=1048576", "count=32", "conv=fsync"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap();
    assert!(!status.success(), "tiny tmpfs must actually fill");
    assert!(
        migrate_to_current(&src, &dest).is_err(),
        "disk-full migration must fail"
    );
    assert_eq!(fs::read(&dest).unwrap(), before);
    assert!(load(&dest).is_ok());
    fs::remove_file(filler).unwrap();
    fs::remove_file(src).unwrap();
    fs::remove_file(dest).unwrap();
}
