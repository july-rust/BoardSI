//! F_004 cross-OS Golden acceptance against an independent Python/SQLite+BSIG
//! producer. Run with --ignored AND F004_GOLDEN_PATH from the F004 workflow.
use boardsi_model::{BoardKind, NetRole};
use boardsi_native::{load, migrate_to_current, save};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn golden() -> PathBuf {
    let p = PathBuf::from(
        std::env::var("F004_GOLDEN_PATH")
            .expect("F004_GOLDEN_PATH is mandatory for these ignored qualification tests"),
    );
    assert!(p.is_file(), "missing frozen Golden v1 fixture");
    p
}
fn scratch(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "boardsi-f004-{}-{nanos}-{tag}.bsi",
        std::process::id()
    ))
}
fn copy_golden(tag: &str) -> PathBuf {
    let to = scratch(tag);
    fs::copy(golden(), &to).unwrap();
    to
}
fn hash(p: &Path) -> Vec<u8> {
    Sha256::digest(fs::read(p).unwrap()).to_vec()
}

#[test]
#[ignore = "F004 OS matrix provides the immutable independent Golden file"]
fn t01_t02_t03_independent_golden_v1_semantics() {
    let p = golden();
    let bytes = fs::read(&p).unwrap();
    assert_eq!(&bytes[..16], b"SQLite format 3\0");
    assert_eq!(
        u32::from_be_bytes(bytes[68..72].try_into().unwrap()),
        0x42534931
    );
    let loaded = load(p).expect("independent SQL/BSIG v1 fixture must be accepted");
    assert!(!loaded.is_read_only());
    let board = loaded.model();
    assert_eq!(board.kind, BoardKind::Pcb);
    assert_eq!(board.sources.len(), 1);
    assert_eq!(board.sources[0].id.files[0].sha256, [7u8; 32]);
    assert_eq!(
        board.sources[0].id.files[0].logical_path,
        "synthetic/golden-v1.asc"
    );
    assert_eq!(board.frames.len(), 1);
    assert_eq!(board.stackup.layers.len(), 1);
    assert_eq!(board.nets.len(), 1);
    assert_eq!(board.nets[0].name, "NET_A");
    assert_eq!(board.nets[0].role, NetRole::Signal);
    assert_eq!(board.traces.len(), 1);
    assert_eq!(
        board.traces[0].centreline[0].ticks,
        [12345678901, -12345678901]
    );
    assert_eq!(
        board.traces[0].centreline[1].ticks,
        [12345679001, -12345678801]
    );
    assert_eq!(board.traces[0].width_m.numerator(), 1);
    assert_eq!(board.traces[0].width_m.denominator(), 100000);
    assert_eq!(board.traces[0].centreline[0].frame, board.frames[0].meta.id);
    assert_eq!(board.traces[0].layer, board.stackup.layers[0].meta.id);
    assert_eq!(board.traces[0].net.as_ref(), Some(&board.nets[0].meta.id));
    assert!(
        board.connectivity.edges.is_empty(),
        "do not invent electrical proof"
    );
}

#[test]
#[ignore = "F004 OS matrix provides the immutable independent Golden file"]
fn t05_t06_t07_version_and_required_optional_fail_closed() {
    let optional = copy_golden("optional");
    {
        let conn = Connection::open(&optional).unwrap();
        conn.execute(
            "INSERT INTO extensions VALUES(?,?,?,?)",
            ("future-opaque", 12, 0, vec![0u8, 255, 7, 0, 42]),
        )
        .unwrap();
    }
    let before = hash(&optional);
    let read = load(&optional).unwrap();
    assert!(read.is_read_only());
    assert!(read.into_editable().is_err());
    assert_eq!(
        load(&optional)
            .unwrap()
            .save_as(scratch("forbidden"))
            .unwrap_err()
            .code(),
        "BSI_UNSUPPORTED_REQUIRED"
    );
    // Raw migration/copy preserves unknown optional payload and file bytes.
    let copied = scratch("opaque-copy");
    assert!(!migrate_to_current(&optional, &copied).unwrap().transformed);
    assert_eq!(hash(&optional), hash(&copied));
    assert_eq!(before, hash(&copied));
    let destination = scratch("blocked-target");
    save(
        &destination,
        &boardsi_model::BoardModel::empty(BoardKind::Pcb),
    )
    .unwrap();
    let unchanged = hash(&destination);
    assert!(save(&optional, &boardsi_model::BoardModel::empty(BoardKind::Pcb)).is_err());
    assert_eq!(hash(&optional), before);
    assert!(migrate_to_current(&destination, &optional).is_err());
    assert_eq!(hash(&optional), before);
    assert_eq!(hash(&destination), unchanged);
    fs::remove_file(optional).unwrap();
    fs::remove_file(copied).unwrap();
    fs::remove_file(destination).unwrap();

    let required = copy_golden("required");
    let conn = Connection::open(&required).unwrap();
    conn.execute(
        "INSERT INTO features VALUES(?,?,?,?)",
        ("future-required", 1, 1, vec![1u8]),
    )
    .unwrap();
    drop(conn);
    assert_eq!(
        load(&required).err().unwrap().code(),
        "BSI_UNSUPPORTED_REQUIRED"
    );
    fs::remove_file(required).unwrap();

    let future = copy_golden("future");
    let conn = Connection::open(&future).unwrap();
    conn.pragma_update(None, "user_version", 2).unwrap();
    drop(conn);
    let before = hash(&future);
    assert_eq!(load(&future).err().unwrap().code(), "BSI_FUTURE_SCHEMA");
    let out = scratch("future-migration");
    assert_eq!(
        migrate_to_current(&future, &out).unwrap_err().code(),
        "BSI_FUTURE_SCHEMA"
    );
    assert!(!out.exists());
    assert_eq!(hash(&future), before);
    fs::remove_file(future).unwrap();

    let min_reader = copy_golden("minimum-reader");
    let conn = Connection::open(&min_reader).unwrap();
    conn.execute("UPDATE file_meta SET format_minor=1,min_reader_minor=1", [])
        .unwrap();
    drop(conn);
    assert_eq!(
        load(&min_reader).err().unwrap().code(),
        "BSI_UNSUPPORTED_REQUIRED"
    );
    fs::remove_file(min_reader).unwrap();
}

#[test]
#[ignore = "F004 OS matrix provides the immutable independent Golden file"]
fn t08_t09_t10_migration_copy_idempotence_and_corruption() {
    let original = golden();
    let before = hash(&original);
    let first = scratch("first");
    let second = scratch("second");
    assert!(!migrate_to_current(&original, &first).unwrap().transformed);
    assert!(!migrate_to_current(&first, &second).unwrap().transformed);
    assert_eq!(hash(&original), before);
    assert_eq!(hash(&first), before);
    assert_eq!(hash(&second), before);
    assert!(!migrate_to_current(&first, &first).unwrap().transformed);
    assert_eq!(hash(&first), before);
    fs::remove_file(first).unwrap();
    fs::remove_file(second).unwrap();

    let invalid = copy_golden("invalid-bsig");
    {
        let conn = Connection::open(&invalid).unwrap();
        conn.execute(
            "UPDATE geometry SET envelope=zeroblob(length(envelope))",
            [],
        )
        .unwrap();
    }
    let before_invalid = hash(&invalid);
    let target = scratch("failed-migration");
    assert!(migrate_to_current(&invalid, &target).is_err());
    assert!(!target.exists());
    assert_eq!(hash(&invalid), before_invalid);
    fs::remove_file(invalid).unwrap();
}
