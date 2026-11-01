//! D_003 source-owned three-area geometry-only regressions (public synthetic PADS).
use boardsi_import::d003_three_area_copcut::{components,encode};

const HEAD:&str="!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const TAIL:&str="*ROUTE*\n*SIGNAL* NET_REF\nR1.1 R2.2\n120 180 2 10 0\n180 180 2 10 0\n*END*\n";
type Rect=[i64;4];
fn area(name:&str,r:Rect,net:&str,layer:usize,hole:Option<Rect>)->String {
    let [x0,y0,x1,y1]=r;
    let mut s=format!("{name} COPPER 0 0 {} 1 0 {net}\nCOPCLS 5 0.01 0 {layer}\n{x0} {y0}\n{x1} {y0}\n{x1} {y1}\n{x0} {y1}\n{x0} {y0}\n",
        if hole.is_some(){2}else{1});
    if let Some([a,b,c,d])=hole {
        s.push_str(&format!("COPCUT 5 0.01 0 {layer}\n{a} {b}\n{c} {b}\n{c} {d}\n{a} {d}\n{a} {b}\n"));
    }
    s
}
fn source(boxes:[Rect;3],h:Option<Rect>,third_net:&str,third_layer:usize)->Vec<u8> {
    format!("{HEAD}*LINES*\n{}{}{}{TAIL}",
        area("RA",boxes[0],"NET_REF",2,h),
        area("RB",boxes[1],"NET_REF",2,None),
        area("RC",boxes[2],third_net,third_layer,None)).into_bytes()
}
fn run(boxes:[Rect;3],h:Option<Rect>)->String {
    encode(&source(boxes,h,"NET_REF",2),["RA","RB","RC"],"NET_REF",2).unwrap()
}
#[test]
fn three_area_positive_transitive_without_manufactured_claim() {
    let boxes=[[100,100,230,230],[200,200,320,320],[300,300,390,390]];
    let (groups,edges)=components(boxes,[None;3]).unwrap();
    assert_eq!(groups,[0,0,0]);
    assert_eq!(edges,[true,false,true]);
    let wire=run(boxes,None);
    assert!(wire.starts_with("BOARDSI_D003_THREE_AREA_V1|"));
    assert!(wire.contains("|SOURCE_GEOMETRIC_COMPONENT_CANDIDATE|1|PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|0|0\n"));
    assert!(wire.contains("EDGE|0|2|SOURCE_SEPARATION_RISK\n"));
    assert_eq!(wire.lines().count(),9);
    assert_eq!(wire.lines().last(),Some("END|3|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL"));
    assert_eq!(wire,run(boxes,None));
    assert!(!wire.contains("PHYSICAL_PASS"));
}
#[test]
fn cutout_removes_pair_contact_and_keeps_independent_component() {
    let boxes=[[100,100,400,400],[200,200,300,300],[360,360,390,390]];
    let hole=[150,150,350,350];
    assert_eq!(components(boxes,[None;3]).unwrap().0,[0,0,0]);
    let (groups,edges)=components(boxes,[Some(hole),None,None]).unwrap();
    assert_eq!(groups,[0,1,0]);
    assert_eq!(edges,[false,true,false]);
    let wire=run(boxes,Some(hole));
    assert!(wire.contains("|SOURCE_SEPARATION_RISK|2|PHYSICAL_NOT_VERIFIED|"));
    assert!(wire.contains("HOLE|0|150|150|350|350\n"));
    assert!(wire.contains("EDGE|0|1|SOURCE_SEPARATION_RISK\n"));
}
#[test]
fn disconnected_or_boundary_only_do_not_create_edge() {
    let boxes=[[100,100,200,200],[200,100,300,200],[400,400,500,500]];
    let (g,e)=components(boxes,[None;3]).unwrap();
    assert_eq!(g,[0,1,2]);
    assert_eq!(e,[false;3]);
    assert!(run(boxes,None).contains("|SOURCE_SEPARATION_RISK|3|"));
}
#[test]
fn full_inventory_net_layer_source_shape_fail_closed() {
    let boxes=[[100,100,400,400],[200,200,300,300],[360,360,390,390]];
    let raw=source(boxes,Some([150,150,350,350]),"NET_REF",2);
    for names in [["RA","RA","RC"],["RA","RB","UNKNOWN"]] {
        assert!(encode(&raw,names,"NET_REF",2).is_err());
    }
    assert!(encode(&raw,["RA","RB","RC"],"OTHER",2).is_err());
    assert!(encode(&source(boxes,None,"OTHER",2),["RA","RB","RC"],"NET_REF",2).is_err());
    assert!(encode(&source(boxes,None,"NET_REF",3),["RA","RB","RC"],"NET_REF",2).is_err());
    let four=String::from_utf8(raw.clone()).unwrap().replace("*ROUTE*",
        &format!("{}*ROUTE*",area("RD",[500,500,600,600],"NET_REF",2,None)));
    assert!(encode(four.as_bytes(),["RA","RB","RC"],"NET_REF",2).is_err());
    let malformed=String::from_utf8(raw.clone()).unwrap()
        .replace("COPCUT 5 0.01 0 2","COPCUT 4 0.01 0 2");
    assert!(encode(malformed.as_bytes(),["RA","RB","RC"],"NET_REF",2).is_err());
    assert!(components(boxes,[Some([150,150,350,350]),Some([210,210,250,250]),None]).is_err());
    assert!(components(boxes,[Some([90,90,200,200]),None,None]).is_err());
    assert!(components([[100,100,100,200],boxes[1],boxes[2]],[None;3]).is_err());
}
#[test]
fn sha_is_full_original_source_and_stale_receipt_must_not_match() {
    let boxes=[[100,100,230,230],[200,200,320,320],[300,300,390,390]];
    let original=source(boxes,None,"NET_REF",2);
    let first=encode(&original,["RA","RB","RC"],"NET_REF",2).unwrap();
    let mut changed=original.clone();
    changed.extend_from_slice(b"REM DIFFERENT SOURCE\n");
    let second=encode(&changed,["RA","RB","RC"],"NET_REF",2);
    assert!(second.is_err() || second.unwrap()!=first);
}
