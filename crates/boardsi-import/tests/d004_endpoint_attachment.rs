//! D_004 strictly bounded Source-only multi-area Trace endpoint regression.
use boardsi_import::d004_endpoint_attachment::{attach, encode, Attachment};
type R = [i64; 4];
fn area(name: &str, r: R, hole: Option<R>, net: &str, layer: i32) -> String {
    let [x, y, z, w] = r;
    let mut s = format!("{name} COPPER 0 0 {} 1 0 {net}\nCOPCLS 5 0.01 0 {layer}\n{x} {y}\n{z} {y}\n{z} {w}\n{x} {w}\n{x} {y}\n",
        if hole.is_some() { 2 } else { 1 });
    if let Some([a, b, c, d]) = hole {
        s.push_str(&format!(
            "COPCUT 5 0.01 0 {layer}\n{a} {b}\n{c} {b}\n{c} {d}\n{a} {d}\n{a} {b}\n"
        ));
    }
    s
}
fn source(p: [i64; 4], hole: Option<R>) -> String {
    format!("!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*LINES*\n{}{}{}*ROUTE*\n*SIGNAL* NET_REF\nR1.1 R2.2\n{} {} 2 10 0\n{} {} 2 10 0\n*END*\n",
        area("RA", [100,100,240,240], hole, "NET_REF",2),
        area("RB", [200,200,320,320], None,"NET_REF",2),
        area("RC", [300,300,390,390],None,"NET_REF",2),
        p[0],p[1],p[2],p[3])
}
fn run(src: &str) -> String {
    encode(src.as_bytes(), ["RA", "RB", "RC"], "NET_REF", 2, 0).unwrap()
}
#[test]
fn positive_candidate_and_transitive_group_are_not_electrical() {
    let wire = run(&source([120, 180, 350, 350], None));
    assert!(wire.starts_with("BOARDSI_D004_ENDPOINT_V1|"));
    assert!(wire.contains("POINT|0|SOURCE_ENDPOINT_ATTACHMENT_CANDIDATE|0|0\n"));
    assert!(wire.contains("POINT|1|SOURCE_ENDPOINT_ATTACHMENT_CANDIDATE|2|0\n"));
    assert!(wire.contains("END|2|SOURCE_ONLY_NOT_PHYSICAL\n"));
    assert!(wire.contains("PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|0|0"));
    assert!(!wire.contains("PHYSICAL_PASS"));
}
#[test]
fn cutout_and_boundary_and_ambiguous_owners_fail_closed() {
    let hole = [130, 130, 170, 170];
    let src = source([150, 150, 210, 210], Some(hole));
    let wire = run(&src);
    assert!(wire.contains("POINT|0|SOURCE_COPCUT_BLOCKED|-|-\n"));
    assert!(wire.contains("POINT|1|SOURCE_AMBIGUOUS_OWNER|-|-\n"));
    let boundary = run(&source([130, 150, 100, 180], Some(hole)));
    assert!(boundary.contains("POINT|0|SOURCE_BOUNDARY_RISK|-|-\n"));
    assert!(boundary.contains("POINT|1|SOURCE_BOUNDARY_RISK|-|-\n"));
    let outside = run(&source([90, 180, 500, 500], None));
    assert_eq!(outside.matches("SOURCE_SEPARATION_RISK|-|-").count(), 2);
}
#[test]
fn isolated_component_is_never_net_label_join() {
    let r = attach(
        [350, 350],
        [
            [100, 100, 200, 200],
            [300, 300, 400, 400],
            [500, 500, 600, 600],
        ],
        [None; 3],
        [0, 1, 2],
    );
    assert_eq!(
        r,
        Attachment {
            state: "SOURCE_ENDPOINT_ATTACHMENT_CANDIDATE",
            owner: Some(1),
            component: Some(1)
        }
    );
    assert_eq!(
        attach(
            [200, 150],
            [
                [100, 100, 200, 200],
                [300, 300, 400, 400],
                [500, 500, 600, 600]
            ],
            [None; 3],
            [0, 1, 2]
        )
        .state,
        "SOURCE_BOUNDARY_RISK"
    );
}
#[test]
fn reject_wrong_source_or_selection_or_trace_inventory() {
    let src = source([120, 180, 350, 350], None);
    assert!(encode(src.as_bytes(), ["RA", "RB", "RC"], "NET_REF", 2, 1).is_err());
    assert!(encode(src.as_bytes(), ["RA", "RB", "RC"], "OTHER", 2, 0).is_err());
    assert!(encode(src.as_bytes(), ["RA", "RB", "RC"], "NET_REF", 3, 0).is_err());
    let twice = src.replace(
        "*END*",
        "*SIGNAL* NET_REF\nR1.1 R2.2\n420 430 2 10 0\n430 440 2 10 0\n*END*",
    );
    assert!(encode(twice.as_bytes(), ["RA", "RB", "RC"], "NET_REF", 2, 0).is_err());
    let area = src.replace("RC COPPER 0 0 1 1 0 NET_REF", "RC COPPER 0 0 1 1 0 OTHER");
    assert!(encode(area.as_bytes(), ["RA", "RB", "RC"], "NET_REF", 2, 0).is_err());
    let old = run(&src);
    let moved = run(&source([122, 181, 350, 350], None));
    assert_ne!(old, moved);
}
