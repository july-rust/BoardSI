//! C_022 synthetic PADS source geometry only. No electrical continuity.
use boardsi_import::reference_area_v14::{components, encode};

const HEAD: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const TAIL: &str = "*ROUTE*\n*SIGNAL* NET_REF\nR1.1 R2.2\n120 180 2 10 0\n180 180 2 10 0\n*END*\n";
fn area(name: &str, b: [i64; 4], net: &str, cutout: bool, layer: usize) -> String {
    let [x0, y0, x1, y1] = b;
    let mut result = format!(
        "{name} COPPER 0 0 {} 1 0 {net}\nCOPCLS 5 0.01 0 {layer}\n{x0} {y0}\n{x1} {y0}\n{x1} {y1}\n{x0} {y1}\n{x0} {y0}\n",
        if cutout { 2 } else { 1 }
    );
    if cutout {
        result.push_str("COPCUT 5 0.01 0 2\n210 120\n220 120\n220 130\n210 130\n210 120\n");
    }
    result
}
fn source(boxes: &[[i64; 4]], net_b: &str, hole: bool, layer: usize) -> Vec<u8> {
    let mut s = format!("{HEAD}*LINES*\n");
    for (i, b) in boxes.iter().enumerate() {
        s.push_str(&area(&format!("R{}", (b'A' + i as u8) as char), *b,
            if i == 1 { net_b } else { "NET_REF" }, i == 1 && hole,
            if i == 1 { layer } else { 2 }));
    }
    s.push_str(TAIL);
    s.into_bytes()
}
fn run(rects: &[[i64; 4]]) -> String {
    let names = ["RA", "RB", "RC", "RD", "RE", "RF"];
    encode(&source(rects, "NET_REF", false, 2), &names[..rects.len()], "NET_REF", 2)
        .expect("valid full source")
}
fn summary(wire: &str) -> Vec<&str> {
    let parts: Vec<_> = wire.lines().next().unwrap().split('|').collect();
    assert_eq!(parts.len(), 10);
    assert_eq!(parts[0], "BOARDSI_REFERENCE_AREA_V14");
    assert_eq!(&parts[6..], ["UNSUPPORTED", "NOT_RUN", "0", "0"]);
    parts
}
fn ids(wire: &str) -> Vec<usize> {
    wire.lines().skip(1).take(wire.lines().count() - 2)
        .map(|line| line.split('|').last().unwrap().parse().unwrap()).collect()
}
#[test]
fn transitive_chain_without_direct_a_c_overlap() {
    let s = run(&[[100, 100, 250, 200], [200, 100, 350, 200], [300, 100, 450, 200]]);
    assert_eq!(summary(&s)[3], "1");
    assert_eq!(ids(&s), [0, 0, 0]);
    assert!(s.ends_with("END|3|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n"));
}
#[test]
fn pair_and_island_and_all_islands() {
    let pair = run(&[[100, 100, 250, 200], [200, 100, 350, 200], [450, 100, 550, 200]]);
    assert_eq!(ids(&pair), [0, 0, 2]);
    assert_eq!(summary(&pair)[4], "DISCONNECTED_ISLAND_RISK");
    let lone = run(&[[100, 100, 200, 200], [300, 100, 400, 200], [500, 100, 600, 200]]);
    assert_eq!(ids(&lone), [0, 1, 2]);
    assert_eq!(summary(&lone)[3], "3");
}
#[test]
fn boundary_only_never_unions() {
    let b = run(&[[100, 100, 200, 200], [200, 100, 300, 200], [300, 200, 400, 300]]);
    assert_eq!(summary(&b)[3], "3");
    assert_eq!(summary(&b)[5], "2");
    assert_eq!(ids(&b), [0, 1, 2]);
}
#[test]
fn six_groups_are_deterministic_and_order_sensitive_only_for_ids() {
    let r = [[100, 100, 230, 200], [200, 100, 330, 200],
        [300, 100, 430, 200], [700, 100, 800, 200],
        [900, 100, 1000, 200], [950, 100, 1050, 200]];
    let a = run(&r);
    assert_eq!(ids(&a), [0, 0, 0, 3, 4, 4]);
    assert_eq!(summary(&a)[3], "3");
    assert_eq!(a, run(&r));
}
#[test]
fn net_layer_cutout_duplicate_and_incomplete_inventory_fail_closed() {
    let r = [[100, 100, 250, 200], [200, 100, 350, 200], [300, 100, 450, 200]];
    let names = ["RA", "RB", "RC"];
    for (net, hole, layer) in [("NET_OTHER", false, 2), ("NET_REF", true, 2), ("NET_REF", false, 3)] {
        assert!(encode(&source(&r, net, hole, layer), &names, "NET_REF", 2).is_err());
    }
    let valid = source(&r, "NET_REF", false, 2);
    assert!(encode(&valid, &["RA", "RA", "RC"], "NET_REF", 2).is_err());
    assert!(encode(&valid, &["RA", "RB", "MISSING"], "NET_REF", 2).is_err());
    assert!(encode(&valid, &["RA", "RB"], "NET_REF", 2).is_err());
    assert!(encode(&valid, &["RA", "RB", "RC", "RD"], "NET_REF", 2).is_err());
    assert!(encode(&valid, &names, "NET_REF", 3).is_err());
    assert!(encode(&valid, &names, "NET_OTHER", 2).is_err());
}
#[test]
fn original_sha_changes_and_invalid_rectangles_are_rejected() {
    let r = [[100, 100, 250, 200], [200, 100, 350, 200], [300, 100, 450, 200]];
    let names = ["RA", "RB", "RC"];
    let before = source(&r, "NET_REF", false, 2);
    let mut after = before.clone();
    after.extend_from_slice(b"REM SOME COMMENT\n");
    let a = encode(&before, &names, "NET_REF", 2).unwrap();
    let b = encode(&after, &names, "NET_REF", 2);
    assert!(b.is_err() || a != b.unwrap());
    assert!(components(&[[100, 100, 100, 200], r[1], r[2]]).is_err());
    assert!(components(&[r[0], r[1]]).is_err());
    assert!(components(&[r[0]; 7]).is_err());
}
