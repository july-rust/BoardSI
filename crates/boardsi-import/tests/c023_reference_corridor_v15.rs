//! Source-only exact-interval regression; no material fill/return-current claim.
use boardsi_import::reference_corridor_v15::{corridor, encode};

const HEAD: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const NAMES: [&str;6] = ["RA","RB","RC","RD","RE","RF"];
fn source(rects: &[[i64;4]], net_b: &str, layer_b: usize, cutout: bool) -> Vec<u8> {
    let mut s = format!("{HEAD}*LINES*\n");
    for (i,&[x0,y0,x1,y1]) in rects.iter().enumerate() {
        let name=NAMES[i];
        let net=if i==1 {net_b} else {"NET_REF"};
        let layer=if i==1 {layer_b} else {2};
        let count=if i==1 && cutout {2} else {1};
        s.push_str(&format!("{name} COPPER 0 0 {count} 1 0 {net}\nCOPCLS 5 0.01 0 {layer}\n{x0} {y0}\n{x1} {y0}\n{x1} {y1}\n{x0} {y1}\n{x0} {y0}\n"));
        if i==1 && cutout {
            s.push_str("COPCUT 5 0.01 0 2\n210 120\n220 120\n220 130\n210 130\n210 120\n");
        }
    }
    s.push_str("*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n150 150 1 10 0\n550 150 1 10 0\n*SIGNAL* NET_REF\nR1.1 R2.2\n120 180 2 10 0\n180 180 2 10 0\n*END*\n");
    s.into_bytes()
}
fn wire(rects: &[[i64;4]]) -> String {
    encode(&source(rects,"NET_REF",2,false), &NAMES[..rects.len()],"NET_REF",2,0).unwrap()
}
fn first(w: &str) -> Vec<&str> {w.lines().next().unwrap().split('|').collect()}
#[test]
fn three_rectangles_cover_full_signal_with_positive_interiors() {
    let w=wire(&[[100,100,260,200],[240,100,410,200],[390,100,600,200]]);
    assert_eq!(first(&w)[6],"COVERED_SOURCE_CANDIDATE");
    assert_eq!(first(&w)[7],"0");
    assert!(w.contains("TRACE|0|1|"));
    assert!(w.ends_with("END|3|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
}
#[test]
fn six_rectangle_overlaps_preserve_coverage() {
    let w=wire(&[[100,100,240,200],[220,100,310,200],[290,100,370,200],
                 [350,100,440,200],[420,100,510,200],[490,100,650,200]]);
    assert_eq!(first(&w)[6],"COVERED_SOURCE_CANDIDATE");
    assert_eq!(first(&w)[4],"1");
}
#[test]
fn positive_gap_has_exact_source_tick_endpoints() {
    let w=wire(&[[100,100,260,200],[240,100,310,200],[350,100,600,200]]);
    assert_eq!(first(&w)[6],"REFERENCE_GAP_RISK");
    assert_eq!(first(&w)[7],"1");
    assert!(w.contains("\nGAP|310|350\n"));
}
#[test]
fn exact_touch_is_not_strictly_connected() {
    let w=wire(&[[100,100,260,200],[260,100,400,200],[400,100,600,200]]);
    assert_eq!(first(&w)[6],"DISCONNECTED_COMPONENT_RISK");
    assert_eq!(first(&w)[4],"3");
    let c=corridor(&[[100,100,260,200],[260,100,400,200],[400,100,600,200]],
                   [150,150],[550,150]).unwrap();
    assert!(c.gaps.is_empty());
    assert!(c.boundary_only);
}
#[test]
fn disjoint_in_2d_not_along_selected_trace_is_still_component_risk() {
    let w=wire(&[[100,100,300,200],[280,100,600,200],[700,300,800,400]]);
    assert_eq!(first(&w)[6],"DISCONNECTED_COMPONENT_RISK");
    assert_eq!(first(&w)[4],"2");
}
#[test]
fn transverse_boundary_and_zero_intervals_are_not_coverage() {
    let c=corridor(&[[100,150,260,250],[240,150,410,250],[390,150,600,250]],
                   [150,150],[550,150]).unwrap();
    assert_eq!(c.gaps,vec![[150,550]]);
    assert!(corridor(&[[100,100,260,200];3],[150,150],[150,150]).is_err());
    assert!(corridor(&[[100,100,260,200];3],[150,150],[550,160]).is_err());
    assert!(corridor(&[[100,100,100,200];3],[150,150],[550,150]).is_err());
}
#[test]
fn invalid_inventory_net_layer_cutout_trace_and_modified_source_fail_closed() {
    let rects=[[100,100,260,200],[240,100,410,200],[390,100,600,200]];
    for (net,layer,hole) in [("NET_OTHER",2,false),("NET_REF",3,false),
                             ("NET_REF",2,true)] {
        assert!(encode(&source(&rects,net,layer,hole),&NAMES[..3],"NET_REF",2,0).is_err());
    }
    let b=source(&rects,"NET_REF",2,false);
    assert!(encode(&b,&NAMES[..2],"NET_REF",2,0).is_err());
    assert!(encode(&b,&NAMES[..3],"NET_REF",2,99).is_err());
    assert!(encode(&b,&NAMES[..3],"NET_REF",2,1).is_err());
    assert!(encode(&b,&["RA","RA","RC"],"NET_REF",2,0).is_err());
    let mut changed=b.clone();changed.extend_from_slice(b"! 2026 changed\n");
    let before=encode(&b,&NAMES[..3],"NET_REF",2,0).unwrap();
    let after=encode(&changed,&NAMES[..3],"NET_REF",2,0);
    assert!(after.is_err() || before!=after.unwrap());
}
