//! F_005: real CAD source -> independent native worker -> v1 SQLite -> Canonical.
//! Synthetic E_021 source contract; no manufactured connectivity or SI claim.
use boardsi_import::{odb_qt_wire, odb_source, route_preview};
use boardsi_model::{PadShape, Rational};
use boardsi_native::{canonical_semantics_equal, load};
use rusqlite::{Connection, OpenFlags};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_boardsi-native-file")
}
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
fn temp() -> PathBuf {
    // Windows/macOS wall-clock resolution may be coarser than Rust's parallel
    // test execution. Each fixture must be unique even within the same PID
    // and timestamp, without erasing another test's existing directory.
    for _ in 0..64 {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let serial = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let p =
            std::env::temp_dir().join(format!("boardsi-f005-{}-{n}-{serial}", std::process::id()));
        match fs::create_dir(&p) {
            Ok(()) => return p,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => panic!("F005 fixture creation failed: {e}"),
        }
    }
    panic!("F005 unique test fixture path exhausted");
}
#[test]
fn f005_parallel_temporary_fixture_names_do_not_collide() {
    let workers = (0..24)
        .map(|_| std::thread::spawn(temp))
        .collect::<Vec<_>>();
    let directories = workers
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect::<Vec<_>>();
    let unique = directories
        .iter()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique.len(), directories.len());
    for dir in directories {
        fs::remove_dir_all(dir).unwrap();
    }
}
fn file(root: &Path, path: &str, data: &[u8]) {
    let full = root.join(path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full, data).unwrap();
}
const MATRIX: &str="STEP {\nCOL=1\nNAME=pcb\n}\nLAYER {\nROW=1\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=sig1\nPOLARITY=POSITIVE\n}\nLAYER {\nROW=2\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=sig2\nPOLARITY=POSITIVE\n}\n";
const FEATURES: &str =
    "UNITS=MM\nF 2\n$0 r100\n$1 rect200x100 M\nL 0 0 1.5 0 0 P 0\nP 1.5 0 1 P 0 0\n";
const FILES: [&str; 3] = [
    "matrix/matrix",
    "steps/pcb/layers/sig1/features",
    "steps/pcb/layers/sig2/features",
];
fn seed(root: &Path) {
    file(root, FILES[0], MATRIX.as_bytes());
    for path in &FILES[1..] {
        file(root, path, FEATURES.as_bytes());
    }
}
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("string write");
    }
    out
}
/// Independent Qt E_021 manifest: hex(utf8 logical path)|hex(SHA256 raw bytes)\n.
fn manifest(root: &Path) -> String {
    let mut hash = Sha256::new();
    for path in FILES {
        let payload = fs::read(root.join(path)).unwrap();
        let file_hash = Sha256::digest(payload);
        hash.update(format!("{}|{}\n", hex(path.as_bytes()), hex(&file_hash)).as_bytes());
    }
    hex(&hash.finalize())
}
fn run(args: &[&std::ffi::OsStr]) -> std::process::Output {
    Command::new(binary())
        .args(args)
        .output()
        .expect("worker must run")
}
fn ok(output: std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("JSON worker receipt")
}
// Deliberately independent of boardsi_native BSIG encoder/decoder/SQL read code.
fn independent_sqlite_oracle(path: &Path, root: &Path) {
    let db = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let app: i64 = db
        .pragma_query_value(None, "application_id", |r| r.get(0))
        .unwrap();
    let schema: i64 = db
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!((app, schema), (0x42534931, 1));
    let mut stmt = db
        .prepare("SELECT logical_path,sha256 FROM source_files ORDER BY ordinal")
        .unwrap();
    let files: Vec<(String, Vec<u8>)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(files.len(), FILES.len());
    for (i, (path, sha)) in files.iter().enumerate() {
        assert_eq!(path, FILES[i]);
        assert_eq!(
            sha.as_slice(),
            &Sha256::digest(fs::read(root.join(path)).unwrap())[..]
        );
    }
    let data: Vec<u8> = db
        .query_row(
            "SELECT envelope FROM geometry WHERE geometry_kind=1 LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(data.len() > 52 && &data[..4] == b"BSIG");
    let ver = u16::from_le_bytes(data[4..6].try_into().unwrap());
    let kind = u16::from_le_bytes(data[6..8].try_into().unwrap());
    assert_eq!((ver, kind), (1, 1));
    assert_eq!(
        u64::from_le_bytes(data[12..20].try_into().unwrap()) as usize,
        data.len() - 52
    );
    assert_eq!(&Sha256::digest(&data[52..])[..], &data[20..52]);
    // Trace TLV #1, type=7: count u32 and two points(frame ref,u64 x,u64 y).
    let p = &data[52..];
    assert_eq!(u16::from_le_bytes(p[4..6].try_into().unwrap()), 1);
    assert_eq!(u16::from_le_bytes(p[6..8].try_into().unwrap()), 7);
    assert_eq!(u32::from_le_bytes(p[12..16].try_into().unwrap()), 2);
    let point = |start: usize| -> (i64, i64) {
        (
            i64::from_le_bytes(p[start + 4..start + 12].try_into().unwrap()),
            i64::from_le_bytes(p[start + 12..start + 20].try_into().unwrap()),
        )
    };
    assert_eq!(point(16), (0, 0));
    assert_eq!(point(36), (1_500_000_000, 0));
}

#[test]
fn f005_odb_actual_worker_exact_round_trip_and_independent_source_oracle() {
    let temp = temp();
    let root = temp.join("vendor-synthetic-odb");
    seed(&root);
    let expected = manifest(&root);
    let model = odb_source::import_directory(&root, "pcb").unwrap();
    assert!(odb_qt_wire::render(&model)
        .unwrap()
        .starts_with("BOARDSI_ODB_QT_V1|3|2|4|"));
    assert_eq!(model.sources.len(), 1);
    assert_eq!(
        (
            model.traces.len(),
            model.pads.len(),
            model.stackup.layers.len()
        ),
        (2, 2, 2)
    );
    let frame = model
        .frames
        .iter()
        .find(|f| f.meta.id == model.traces[0].centreline[1].frame)
        .unwrap();
    // Independent physical witness: original literal 1.5 mm = 3/2000 metre.
    assert_eq!(
        frame.exact_m(&model.traces[0].centreline[1]).unwrap()[0],
        Rational::new(3, 2000).unwrap()
    );
    let native = temp.join("from-odb.bsi");
    let args = [
        std::ffi::OsStr::new("save-odb"),
        root.as_os_str(),
        std::ffi::OsStr::new("pcb"),
        native.as_os_str(),
        std::ffi::OsStr::new(&expected),
    ];
    let receipt = ok(run(&args));
    assert_eq!(receipt["action"], "save-odb");
    assert_eq!(receipt["source_format"], "ODB_SOURCE_SUBSET");
    assert_eq!(receipt["source_manifest_sha256"], expected);
    assert_eq!(receipt["physical_si"], "NOT_RUN");
    assert_eq!(receipt["traces"], 2);
    assert_eq!(receipt["pads"], 2);
    independent_sqlite_oracle(&native, &root);
    let loaded = load(&native).unwrap();
    assert!(!loaded.is_read_only());
    assert!(
        canonical_semantics_equal(&model, loaded.model()),
        "all SourceIdentity, frame rational, net, padstack, geometry and capability fields"
    );
    let reopened = ok(run(&[std::ffi::OsStr::new("open"), native.as_os_str()]));
    assert_eq!(reopened["sha256"], receipt["sha256"]);
    let copy = temp.join("copy.bsi");
    let sha = reopened["sha256"].as_str().unwrap();
    ok(run(&[
        std::ffi::OsStr::new("save-loaded"),
        native.as_os_str(),
        copy.as_os_str(),
        std::ffi::OsStr::new(sha),
    ]));
    assert!(canonical_semantics_equal(
        &model,
        load(&copy).unwrap().model()
    ));
    fs::remove_dir_all(temp).unwrap();
}
#[test]
fn f005_odb_stale_manifest_bad_step_and_unsupported_feature_preserve_last_good() {
    let temp = temp();
    let root = temp.join("odb");
    seed(&root);
    let target = temp.join("keep.bsi");
    let accepted = manifest(&root);
    let good = ok(run(&[
        std::ffi::OsStr::new("save-odb"),
        root.as_os_str(),
        std::ffi::OsStr::new("pcb"),
        target.as_os_str(),
        std::ffi::OsStr::new(&accepted),
    ]));
    assert_eq!(good["action"], "save-odb");
    let original = fs::read(&target).unwrap();
    file(
        &root,
        FILES[1],
        FEATURES.replace("1.5 0", "1.8 0").as_bytes(),
    );
    let stale = run(&[
        std::ffi::OsStr::new("save-odb"),
        root.as_os_str(),
        std::ffi::OsStr::new("pcb"),
        target.as_os_str(),
        std::ffi::OsStr::new(&accepted),
    ]);
    assert!(!stale.status.success());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("BSI_SOURCE_CHANGED"));
    assert_eq!(fs::read(&target).unwrap(), original);
    let fresh = manifest(&root);
    let bad_step = run(&[
        std::ffi::OsStr::new("save-odb"),
        root.as_os_str(),
        std::ffi::OsStr::new("missing"),
        target.as_os_str(),
        std::ffi::OsStr::new(&fresh),
    ]);
    assert!(!bad_step.status.success());
    assert_eq!(fs::read(&target).unwrap(), original);
    file(
        &root,
        FILES[1],
        FEATURES.replace("L 0 0", "X 0 0").as_bytes(),
    );
    let unsupported = manifest(&root);
    let bad = run(&[
        std::ffi::OsStr::new("save-odb"),
        root.as_os_str(),
        std::ffi::OsStr::new("pcb"),
        target.as_os_str(),
        std::ffi::OsStr::new(&unsupported),
    ]);
    assert!(!bad.status.success());
    assert_eq!(fs::read(&target).unwrap(), original);
    assert!(load(&target).is_ok());
    fs::remove_dir_all(temp).unwrap();
}
#[test]
fn f005_pads_source_and_native_reader_semantics_are_identical() {
    let temp = temp();
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/boardsi-import/tests/fixtures/pads_via_round_through_v94.asc");
    let bytes = fs::read(&src).unwrap();
    let hash = hex(&Sha256::digest(&bytes));
    let parsed = route_preview::parse_generalized(&bytes).unwrap();
    assert_eq!(parsed.skipped, 0);
    let canonical = route_preview::into_canonical(&bytes, &parsed).unwrap();
    let target = temp.join("pads.bsi");
    let receipt = ok(run(&[
        std::ffi::OsStr::new("save-pads"),
        src.as_os_str(),
        target.as_os_str(),
        std::ffi::OsStr::new(&hash),
    ]));
    assert_eq!(receipt["action"], "save-pads");
    assert!(canonical_semantics_equal(
        &canonical,
        load(&target).unwrap().model()
    ));
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn f005_shared_gate_rejects_qt_numeric_overflow_and_source_variants() {
    let temp = temp();
    let root = temp.join("odb");
    seed(&root);
    let valid = odb_source::import_directory(&root, "pcb").unwrap();
    assert!(odb_qt_wire::render(&valid).is_ok());
    let target = temp.join("keep.bsi");
    let accepted = manifest(&root);
    ok(run(&[
        std::ffi::OsStr::new("save-odb"),
        root.as_os_str(),
        std::ffi::OsStr::new("pcb"),
        target.as_os_str(),
        std::ffi::OsStr::new(&accepted),
    ]));
    let original = fs::read(&target).unwrap();

    // First, enforce numeric gates on already-valid Canonical models. These
    // must be rejected by the common Rust gate, not merely by a Qt decoder.
    let mut too_wide = odb_source::import_directory(&root, "pcb").unwrap();
    too_wide.traces[0].width_m = Rational::integer(2);
    assert_eq!(
        odb_qt_wire::render(&too_wide),
        Err("E021_QT_DIMENSION_LIMIT")
    );
    let mut too_big_pad = odb_source::import_directory(&root, "pcb").unwrap();
    too_big_pad.padstacks[0].entries[0].shape = PadShape::Circle {
        diameter_m: Rational::integer(2),
    };
    assert_eq!(
        odb_qt_wire::render(&too_big_pad),
        Err("E021_QT_DIMENSION_LIMIT")
    );
    let mut too_far = odb_source::import_directory(&root, "pcb").unwrap();
    too_far.frames[0].origin_m[0] = Rational::integer(1001);
    assert!(odb_qt_wire::render(&too_far).is_err());
    // Positive exact 1e-21 metre pad width at a normal 1.5 mm coordinate
    // collapses to a zero-width QRectF after conversion to Qt double mm.
    // Keep the Canonical frame unchanged to isolate Qt display precision.
    let mut collapsed = odb_source::import_directory(&root, "pcb").unwrap();
    collapsed.padstacks[0].entries[0].shape = PadShape::Rectangle {
        size_m: [
            Rational::new(1, 1_000_000_000_000_000_000_000).unwrap(),
            Rational::integer(1),
        ],
    };
    assert_eq!(
        odb_qt_wire::render(&collapsed),
        Err("E021_QT_PROJECTION_COLLAPSED")
    );

    // Independently verify that newly hashed oversize source files never
    // overwrite the last-good .bsi (whether importer or shared gate rejects).
    for source in [
        FEATURES.replace("$0 r100", "$0 r2000000 M"),
        FEATURES.replace("rect200x100 M", "rect2000000x100 M"),
        FEATURES.replace("L 0 0 1.5 0", "L 1000001 0 1.5 0"),
        FEATURES
            .replace("rect200x100 M", "rect0.000000001x100 M")
            .replace("P 1.5 0 1 P 0 0", "P 1000000 0 1 P 0 0"),
    ] {
        seed(&root);
        file(&root, FILES[1], source.as_bytes());
        let hash = manifest(&root);
        let bad = run(&[
            std::ffi::OsStr::new("save-odb"),
            root.as_os_str(),
            std::ffi::OsStr::new("pcb"),
            target.as_os_str(),
            std::ffi::OsStr::new(&hash),
        ]);
        assert!(
            !bad.status.success(),
            "oversized source was accepted: {}",
            String::from_utf8_lossy(&bad.stdout)
        );
        assert_eq!(fs::read(&target).unwrap(), original);
    }
    fs::remove_dir_all(temp).unwrap();
}

#[cfg(feature = "f004-qualification")]
#[test]
fn f005_real_source_mutation_during_staging_aborts_before_publish() {
    use std::{process::Stdio, thread, time::Duration};
    let temp = temp();
    let root = temp.join("odb");
    seed(&root);
    let target = temp.join("last-good.bsi");
    let expected = manifest(&root);
    ok(run(&[
        std::ffi::OsStr::new("save-odb"),
        root.as_os_str(),
        std::ffi::OsStr::new("pcb"),
        target.as_os_str(),
        std::ffi::OsStr::new(&expected),
    ]));
    let original = fs::read(&target).unwrap();

    let marker = temp.join("before-replace.marker");
    let resume = marker.with_extension("resume");
    let mut worker = Command::new(binary())
        .arg("save-odb")
        .arg(&root)
        .arg("pcb")
        .arg(&target)
        .arg(&expected)
        .env("BOARDSI_F003_QUAL_MODE", "pause-until-resume")
        .env("BOARDSI_F003_QUAL_MARKER", &marker)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    for _ in 0..200 {
        if marker.exists() {
            break;
        }
        if worker.try_wait().unwrap().is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    if !marker.exists() {
        let _ = worker.kill();
        let _ = worker.wait();
        panic!("native worker never reached the validated staging gate");
    }
    // Mutation takes place while the worker has a fully validated staging
    // SQLite file and has not yet published the replacement.
    file(
        &root,
        FILES[1],
        FEATURES.replace("1.5 0", "1.8 0").as_bytes(),
    );
    fs::write(&resume, b"resume").unwrap();
    let output = worker.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("BSI_SOURCE_CHANGED"),
        "actual error: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read(&target).unwrap(), original);
    assert!(load(&target).is_ok());
    fs::remove_dir_all(temp).unwrap();
}
