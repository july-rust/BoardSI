//! F_005: real CAD source -> independent native worker -> v1 SQLite -> Canonical.
//! Synthetic E_021 source contract; no manufactured connectivity or SI claim.
use boardsi_import::{odb_qt_wire, odb_source, route_preview};
use boardsi_model::Rational;
use boardsi_native::{canonical_semantics_equal, load};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::{Path,PathBuf}, process::Command, time::{SystemTime,UNIX_EPOCH}};

fn binary() -> &'static str { env!("CARGO_BIN_EXE_boardsi-native-file") }
fn temp() -> PathBuf {
    let n=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let p=std::env::temp_dir().join(format!("boardsi-f005-{}-{n}",std::process::id()));
    fs::create_dir(&p).unwrap();
    p
}
fn file(root: &Path, path: &str, data: &[u8]) {
    let full=root.join(path);
    fs::create_dir_all(full.parent().unwrap()).unwrap();
    fs::write(full,data).unwrap();
}
const MATRIX: &str="STEP {\nCOL=1\nNAME=pcb\n}\nLAYER {\nROW=1\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=sig1\nPOLARITY=POSITIVE\n}\nLAYER {\nROW=2\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=sig2\nPOLARITY=POSITIVE\n}\n";
const FEATURES: &str="UNITS=MM\nF 2\n$0 r100\n$1 rect200x100 M\nL 0 0 1.5 0 0 P 0\nP 1.5 0 1 P 0 0\n";
const FILES: [&str;3]=["matrix/matrix","steps/pcb/layers/sig1/features","steps/pcb/layers/sig2/features"];
fn seed(root: &Path) {
    file(root,FILES[0],MATRIX.as_bytes());
    for path in &FILES[1..] { file(root,path,FEATURES.as_bytes()); }
}
fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b|format!("{b:02x}")).collect() }
/// Independent Qt E_021 manifest: hex(utf8 logical path)|hex(SHA256 raw bytes)\n.
fn manifest(root: &Path) -> String {
    let mut hash=Sha256::new();
    for path in FILES {
        let payload=fs::read(root.join(path)).unwrap();
        let file_hash=Sha256::digest(payload);
        hash.update(format!("{}|{}\n",hex(path.as_bytes()),hex(&file_hash)).as_bytes());
    }
    hex(&hash.finalize())
}
fn run(args: &[&std::ffi::OsStr]) -> std::process::Output {
    Command::new(binary()).args(args).output().expect("worker must run")
}
fn ok(output: std::process::Output) -> Value {
    assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).expect("JSON worker receipt")
}
#[test]
fn f005_odb_actual_worker_exact_round_trip_and_independent_source_oracle() {
    let temp=temp();
    let root=temp.join("vendor-synthetic-odb");
    seed(&root);
    let expected=manifest(&root);
    let model=odb_source::import_directory(&root,"pcb").unwrap();
    assert!(odb_qt_wire::render(&model).unwrap().starts_with("BOARDSI_ODB_QT_V1|3|2|4|"));
    assert_eq!(model.sources.len(),1);
    assert_eq!((model.traces.len(),model.pads.len(),model.stackup.layers.len()),(2,2,2));
    let frame=model.frames.iter().find(|f| f.meta.id == model.traces[0].centreline[1].frame).unwrap();
    // Independent physical witness: original literal 1.5 mm = 3/2000 metre.
    assert_eq!(frame.exact_m(&model.traces[0].centreline[1]).unwrap()[0],
               Rational::new(3,2000).unwrap());
    let native=temp.join("from-odb.bsi");
    let args=[
        std::ffi::OsStr::new("save-odb"),root.as_os_str(),std::ffi::OsStr::new("pcb"),
        native.as_os_str(),std::ffi::OsStr::new(&expected)];
    let receipt=ok(run(&args));
    assert_eq!(receipt["action"],"save-odb");
    assert_eq!(receipt["source_format"],"ODB_SOURCE_SUBSET");
    assert_eq!(receipt["source_manifest_sha256"],expected);
    assert_eq!(receipt["physical_si"],"NOT_RUN");
    assert_eq!(receipt["traces"],2);
    assert_eq!(receipt["pads"],2);
    let loaded=load(&native).unwrap();
    assert!(!loaded.is_read_only());
    assert!(canonical_semantics_equal(&model,loaded.model()),
            "all SourceIdentity, frame rational, net, padstack, geometry and capability fields");
    let reopened=ok(run(&[std::ffi::OsStr::new("open"),native.as_os_str()]));
    assert_eq!(reopened["sha256"],receipt["sha256"]);
    let copy=temp.join("copy.bsi");
    let sha=reopened["sha256"].as_str().unwrap();
    ok(run(&[std::ffi::OsStr::new("save-loaded"),native.as_os_str(),copy.as_os_str(),
        std::ffi::OsStr::new(sha)]));
    assert!(canonical_semantics_equal(&model,load(&copy).unwrap().model()));
    fs::remove_dir_all(temp).unwrap();
}
#[test]
fn f005_odb_stale_manifest_bad_step_and_unsupported_feature_preserve_last_good() {
    let temp=temp();
    let root=temp.join("odb");seed(&root);
    let target=temp.join("keep.bsi");
    let accepted=manifest(&root);
    let good=ok(run(&[
        std::ffi::OsStr::new("save-odb"),root.as_os_str(),std::ffi::OsStr::new("pcb"),
        target.as_os_str(),std::ffi::OsStr::new(&accepted)]));
    assert_eq!(good["action"],"save-odb");
    let original=fs::read(&target).unwrap();
    file(&root,FILES[1],FEATURES.replace("1.5 0","1.8 0").as_bytes());
    let stale=run(&[
        std::ffi::OsStr::new("save-odb"),root.as_os_str(),std::ffi::OsStr::new("pcb"),
        target.as_os_str(),std::ffi::OsStr::new(&accepted)]);
    assert!(!stale.status.success());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("BSI_SOURCE_CHANGED"));
    assert_eq!(fs::read(&target).unwrap(),original);
    let fresh=manifest(&root);
    let bad_step=run(&[
        std::ffi::OsStr::new("save-odb"),root.as_os_str(),std::ffi::OsStr::new("missing"),
        target.as_os_str(),std::ffi::OsStr::new(&fresh)]);
    assert!(!bad_step.status.success());
    assert_eq!(fs::read(&target).unwrap(),original);
    file(&root,FILES[1],FEATURES.replace("L 0 0","X 0 0").as_bytes());
    let unsupported=manifest(&root);
    let bad=run(&[
        std::ffi::OsStr::new("save-odb"),root.as_os_str(),std::ffi::OsStr::new("pcb"),
        target.as_os_str(),std::ffi::OsStr::new(&unsupported)]);
    assert!(!bad.status.success());
    assert_eq!(fs::read(&target).unwrap(),original);
    assert!(load(&target).is_ok());
    fs::remove_dir_all(temp).unwrap();
}
#[test]
fn f005_pads_source_and_native_reader_semantics_are_identical() {
    let temp=temp();
    let src=Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/boardsi-import/tests/fixtures/pads_via_round_through_v94.asc");
    let bytes=fs::read(&src).unwrap();
    let hash=hex(&Sha256::digest(&bytes));
    let parsed=route_preview::parse_generalized(&bytes).unwrap();
    assert_eq!(parsed.skipped,0);
    let canonical=route_preview::into_canonical(&bytes,&parsed).unwrap();
    let target=temp.join("pads.bsi");
    let receipt=ok(run(&[
        std::ffi::OsStr::new("save-pads"),src.as_os_str(),target.as_os_str(),
        std::ffi::OsStr::new(&hash)]));
    assert_eq!(receipt["action"],"save-pads");
    assert!(canonical_semantics_equal(&canonical,load(&target).unwrap().model()));
    fs::remove_dir_all(temp).unwrap();
}
