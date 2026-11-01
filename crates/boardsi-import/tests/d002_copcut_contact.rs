//! D_002 public PADS V9.4 MILS Source COPCUT geometry regressions.
use boardsi_import::d002_copcut_contact::{classify, encode};

const HEAD: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const TAIL: &str = "*ROUTE*\n*SIGNAL* NET_REF\nR1.1 R2.2\n120 180 2 10 0\n180 180 2 10 0\n*END*\n";
const A: [i64; 4] = [100, 100, 400, 400];
const H: [i64; 4] = [150, 150, 350, 350];

fn area(name: &str, r: [i64; 4], net: &str, layer: usize, hole: Option<[i64; 4]>) -> String {
    let [x0, y0, x1, y1] = r;
    let mut s = format!(
        "{name} COPPER 0 0 {} 1 0 {net}\nCOPCLS 5 0.01 0 {layer}\n{x0} {y0}\n{x1} {y0}\n{x1} {y1}\n{x0} {y1}\n{x0} {y0}\n",
        if hole.is_some() { 2 } else { 1 }
    );
    if let Some([a, b, c, d]) = hole {
        s.push_str(&format!("COPCUT 5 0.01 0 {layer}\n{a} {b}\n{c} {b}\n{c} {d}\n{a} {d}\n{a} {b}\n"));
    }
    s
}
fn source(b: [i64; 4], a_hole: Option<[i64; 4]>, net_b: &str, layer_b: usize) -> Vec<u8> {
    format!(
        "{HEAD}*LINES*\n{}{}{TAIL}",
        area("RA", A, "NET_REF", 2, a_hole),
        area("RB", b, net_b, layer_b, None)
    ).into_bytes()
}
fn run(b: [i64; 4], hole: Option<[i64; 4]>) -> String {
    encode(&source(b, hole, "NET_REF", 2), ["RA", "RB"], "NET_REF", 2).unwrap()
}
fn state(wire: &str) -> &str {
    let parts: Vec<_> = wire.lines().next().unwrap().split('|').collect();
    assert_eq!(parts.len(), 8);
    assert_eq!(parts[0], "BOARDSI_D002_COPCUT_CONTACT_V1");
    assert_eq!(&parts[3..], ["PHYSICAL_NOT_VERIFIED", "P009_UNSUPPORTED", "SI_NOT_RUN", "0", "0"]);
    assert_eq!(wire.lines().count(), 5);
    assert_eq!(wire.lines().last(), Some("END|2|SOURCE_CONTACT_HYPOTHESIS_ONLY"));
    parts[2]
}
#[test]
fn no_cutout_and_partial_cutout_are_source_candidates_only() {
    let b = [300, 300, 380, 380];
    assert_eq!(state(&run(b, None)), "SOURCE_CONTACT_CANDIDATE");
    let partial = run(b, Some(H));
    assert_eq!(state(&partial), "SOURCE_CONTACT_CANDIDATE");
    assert!(partial.lines().nth(3).unwrap().starts_with("HOLE|0|"));
    assert!(!partial.contains("PHYSICAL_PASS"));
    assert_eq!(partial, run(b, Some(H)));
}
#[test]
fn cutout_removes_entire_baseline_overlap() {
    let b = [200, 200, 300, 300];
    assert_eq!(state(&run(b, None)), "SOURCE_CONTACT_CANDIDATE");
    assert_eq!(state(&run(b, Some(H))), "SOURCE_SEPARATION_RISK");
}
#[test]
fn boundary_and_disconnected_are_not_contact() {
    for b in [[400, 100, 500, 400], [450, 100, 550, 400], [400, 400, 500, 500]] {
        assert_eq!(state(&run(b, None)), "SOURCE_SEPARATION_RISK");
    }
}
#[test]
fn fail_closed_net_layer_ownership_inventory_and_complex_void() {
    let valid = source([300, 300, 380, 380], Some(H), "NET_REF", 2);
    assert!(encode(&valid, ["RA", "RA"], "NET_REF", 2).is_err());
    assert!(encode(&valid, ["RA", "MISSING"], "NET_REF", 2).is_err());
    assert!(encode(&valid, ["RA", "RB"], "NET_OTHER", 2).is_err());
    assert!(encode(&source([300, 300, 380, 380], Some(H), "OTHER", 2),
        ["RA", "RB"], "NET_REF", 2).is_err());
    assert!(encode(&source([300, 300, 380, 380], Some(H), "NET_REF", 3),
        ["RA", "RB"], "NET_REF", 2).is_err());
    let corrupted = String::from_utf8(valid.clone()).unwrap()
        .replace("COPCUT 5 0.01 0 2", "COPCUT 4 0.01 0 2");
    assert!(encode(corrupted.as_bytes(), ["RA", "RB"], "NET_REF", 2).is_err());
    let three = String::from_utf8(valid).unwrap().replace(
        "*ROUTE*", &format!("{}*ROUTE*", area("RC", [500, 500, 600, 600], "NET_REF", 2, None))
    );
    assert!(encode(three.as_bytes(), ["RA", "RB"], "NET_REF", 2).is_err());
}
#[test]
fn invalid_and_two_hole_inputs_rejected() {
    assert!(classify([A, [200, 200, 300, 300]], [Some(H), Some([210,210,250,250])]).is_err());
    assert!(classify([A, [200, 200, 300, 300]], [Some([90, 90, 200, 200]), None]).is_err());
    assert!(classify([A, [300, 300, 300, 400]], [None, None]).is_err());
}
#[test]
fn source_sha_is_original_byte_bound_and_invalidates_old_receipt() {
    let src = source([300,300,380,380],Some(H),"NET_REF",2);
    let mut changed = src.clone();
    changed.extend_from_slice(b"REM CHANGED SOURCE\n");
    let first = encode(&src,["RA","RB"],"NET_REF",2).unwrap();
    let second = encode(&changed,["RA","RB"],"NET_REF",2);
    assert!(second.is_err() || first != second.unwrap());
}
