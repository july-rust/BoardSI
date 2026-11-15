use boardsi_model::*;
use boardsi_native::{load,save};
use std::sync::Arc;
use std::time::{SystemTime,UNIX_EPOCH};

fn path(name:&str)->std::path::PathBuf{
    let now=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("boardsi-f003-{}-{}-{}.bsi",std::process::id(),now,name))
}
fn fixture()->BoardModel{
    let source=Arc::new(SourceIdentity{format:SourceFormat::Synthetic,format_version:"1".into(),
        files:vec![SourceFile{logical_path:"synthetic/board.asc".into(),sha256:[7;32]}],
        importer_version:"F_003-test".into(),conditions:vec![("step".into(),"main".into())]});
    let mut board=BoardModel::empty(BoardKind::Pcb);
    board.sources.push(ImportSource{id:source.clone(),producer:"test".into(),
        producer_version:"1".into(),capability:ImportCapability::default()});
    let make=|kind,name:&str|ObjectMeta{
        id:ObjectId::new(&source,kind,name,""),
        provenance:GeometryProvenance{source_object_type:"test-source".into(),history:vec![]}};
    let frame=make(ObjectKind::Frame,"f");
    board.frames.push(CoordinateFrame{meta:frame.clone(),unit_label:"um".into(),
        metres_per_tick:Rational::new(1,1_000_000).unwrap(),
        origin_m:[Rational::integer(0),Rational::integer(0)],
        orientation:Orientation::Orthogonal{turn:QuarterTurn::Zero,mirror_x:false},
        source_layer_order_reversed:false});
    let layer=make(ObjectKind::Layer,"L1");
    board.stackup.layers.push(Layer{meta:layer.clone(),name:"L1".into(),order:0,
        kind:LayerKind::Copper,z_m:Property::Missing,thickness_m:Property::Missing,material:None});
    board.traces.push(Trace{meta:make(ObjectKind::Trace,"T1"),layer:layer.id,net:None,
        centreline:vec![Point{frame:frame.id.clone(),ticks:[i64::MIN,0]},
            Point{frame:frame.id,ticks:[i64::MAX,0]}],
        width_m:Rational::new(1,100_000).unwrap()});
    board
}
#[test]
fn empty_board_round_trip() {
    let p=path("empty");let b=BoardModel::empty(BoardKind::Pcb);
    save(&p,&b).unwrap();let got=load(&p).unwrap();
    assert!(!got.read_only);assert_eq!(got.model,b);
    std::fs::remove_file(&p).unwrap();
}
#[test]
fn exact_i64_geometry_source_round_trip() {
    let p=path("trace");let b=fixture();
    save(&p,&b).unwrap();let got=load(&p).unwrap();
    assert_eq!(got.model,b);
    std::fs::remove_file(&p).unwrap();
}
#[test]
fn reject_non_native_and_bad_digest_without_modifying_input(){
    let p=path("corrupt");let b=fixture();
    save(&p,&b).unwrap();
    let conn=rusqlite::Connection::open(&p).unwrap();
    conn.execute("UPDATE geometry SET envelope=? WHERE geometry_kind=1",[vec![0u8;52]]).unwrap();
    drop(conn);
    assert!(load(&p).is_err());
    std::fs::remove_file(&p).unwrap();
    let q=path("other");
    std::fs::write(&q,b"Not native").unwrap();
    assert!(load(&q).is_err());
    std::fs::remove_file(&q).unwrap();
}
#[test]
fn replace_protects_existing_on_invalid_model(){
    let p=path("protect");
    let good=BoardModel::empty(BoardKind::Pcb);
    save(&p,&good).unwrap();
    let before=std::fs::read(&p).unwrap();
    let mut bad=fixture();bad.traces[0].width_m=Rational::integer(-1);
    assert!(save(&p,&bad).is_err());
    assert_eq!(std::fs::read(&p).unwrap(),before);
    std::fs::remove_file(&p).unwrap();
}
