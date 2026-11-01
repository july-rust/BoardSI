//! D_005 synthetic actual PADS ASCII Pad -> Trace -> COPCLS geometry candidates.
use boardsi_import::d005_pad_trace_area::{encode, qualify};
type R = [i64; 4];
const BASE: &str = include_str!("fixtures/pads_component_simple_v94.asc");
fn area(name: &str, r: R, hole: Option<R>, net: &str) -> String {
    let [a, b, c, d] = r;
    let mut s=format!("{name} COPPER 0 0 {} 1 0 {net}\nCOPCLS 5 0.01 0 1\n{a} {b}\n{c} {b}\n{c} {d}\n{a} {d}\n{a} {b}\n",if hole.is_some(){2}else{1});
    if let Some([x, y, z, w]) = hole {
        s.push_str(&format!(
            "COPCUT 5 0.01 0 1\n{x} {y}\n{z} {y}\n{z} {w}\n{x} {w}\n{x} {y}\n"
        ));
    }
    s
}
fn source(start: [i64; 2], end: [i64; 2], hole: Option<R>) -> String {
    let raw = BASE.replace("\r\n", "\n");
    let full = format!(
        "*LINES*\n{}{}{}*ROUTE*",
        area("RA", [240, 240, 350, 360], hole, "NET_A"),
        area("RB", [340, 320, 450, 420], None, "NET_A"),
        area("RC", [440, 390, 520, 480], None, "NET_A")
    );
    raw.replace("*ROUTE*", &full).replace(
        "U9.1 U9.2\n100 200 1 10 0\n300 200 1 10 0",
        &format!(
            "U1.1 U9.2\n{} {} 1 10 0\n{} {} 1 10 0",
            start[0], start[1], end[0], end[1]
        ),
    )
}
fn run(src: &str) -> String {
    encode(src.as_bytes(), ["RA", "RB", "RC"], "NET_A", 1, 0).unwrap()
}
#[test]
fn positive_pad_to_area_group_is_only_geometric() {
    let out = run(&source([204, 300], [260, 310], None));
    assert!(out.starts_with("BOARDSI_D005_PAD_TRACE_AREA_V1|"));
    assert!(
        out.contains("LINK|SOURCE_PAD_TRACE_AREA_CANDIDATE|0|0|1|0|0\n"),
        "{out}"
    );
    assert!(out.contains("PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|0|0"));
    assert!(out.ends_with("END|SOURCE_ONLY_NOT_PHYSICAL\n"));
    assert!(!out.contains("PHYSICAL_PASS"));
}
#[test]
fn area_cutout_or_boundary_refuses_link() {
    let blocked = run(&source([204, 300], [260, 310], Some([250, 300, 270, 320])));
    assert!(blocked.contains("SOURCE_COPCUT_BLOCKED"), "{blocked}");
    assert!(blocked.contains("LINK|SOURCE_CONTACT_NOT_VERIFIED|-|-|-|-|-"));
    let tangent = run(&source([204, 300], [240, 300], None));
    assert!(tangent.contains("SOURCE_BOUNDARY_RISK"));
    assert!(tangent.contains("LINK|SOURCE_CONTACT_NOT_VERIFIED"));
}
#[test]
fn wrong_pin_and_pad_boundary_do_not_promote() {
    let near = source([220, 300], [260, 310], None);
    assert!(run(&near).contains("LINK|SOURCE_CONTACT_NOT_VERIFIED"));
    let wrong = source([204, 300], [260, 310], None).replace("U1.1 U9.2", "U1.2 U9.2");
    assert!(run(&wrong).contains("LINK|SOURCE_CONTACT_NOT_VERIFIED"));
}
#[test]
fn invalid_net_layer_and_extra_trace_fail_closed() {
    let s = source([204, 300], [260, 310], None);
    assert!(encode(s.as_bytes(), ["RA", "RB", "RC"], "OTHER", 1, 0).is_err());
    assert!(encode(s.as_bytes(), ["RA", "RB", "RC"], "NET_A", 2, 0).is_err());
    let twice = s.replace(
        "*END*",
        "*SIGNAL* NET_A\nU1.1 U9.2\n300 300 1 10 0\n400 400 1 10 0\n*END*",
    );
    assert!(encode(twice.as_bytes(), ["RA", "RB", "RC"], "NET_A", 1, 0).is_err());
    let mut wrong = s.clone();
    wrong = wrong.replace("RB COPPER 0 0 1 1 0 NET_A", "RB COPPER 0 0 1 1 0 OTHER");
    assert!(encode(wrong.as_bytes(), ["RA", "RB", "RC"], "NET_A", 1, 0).is_err());
}
#[test]
fn stale_digest_changes_and_nonselected_trace_refuses() {
    let a = source([204, 300], [260, 310], None);
    let b = source([205, 300], [260, 310], None);
    assert_ne!(run(&a), run(&b));
    assert_eq!(run(&a), run(&a));
    assert!(qualify(a.as_bytes(), ["RA", "RB", "RC"], "NET_A", 1, 1).is_err());
}
