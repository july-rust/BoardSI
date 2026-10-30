//! C_021: public PADS V9.4 MILS synthetic source rectangle relationship tests.
//! A SOURCE-ONLY overlap candidate is never electrical Return Current/P009.
use boardsi_import::reference_area_v13::{encode, rectangle_relation, strict_route_union, SourceState};

const HEAD: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const ROUTES: &str = "*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n150 150 1 10 0\n350 150 1 10 0\n*SIGNAL* NET_REF\nR1.1 R2.2\n120 180 2 10 0\n180 180 2 10 0\n";

fn area(name: &str, bounds: [i64; 4], net: &str, hole: bool) -> String {
    let [x0, y0, x1, y1] = bounds;
    let pieces = if hole { 2 } else { 1 };
    let mut text = format!("{name} COPPER 0 0 {pieces} 1 0 {net}\nCOPCLS 5 0.01 0 2\n{x0} {y0}\n{x1} {y0}\n{x1} {y1}\n{x0} {y1}\n{x0} {y0}\n");
    if hole {
        text.push_str("COPCUT 5 0.01 0 2\n210 120\n220 120\n220 130\n210 130\n210 120\n");
    }
    text
}
fn source(a: [i64; 4], b: [i64; 4], net_b: &str, hole: bool) -> Vec<u8> {
    format!("{HEAD}*LINES*\n{}{}{ROUTES}*END*\n",
        area("RA", a, "NET_REF", false), area("RB", b, net_b, hole)).into_bytes()
}
const A: [i64; 4] = [100, 100, 250, 200];
const B: [i64; 4] = [200, 100, 400, 200];

fn result(bytes: &[u8], trace: Option<usize>) -> Result<String, &'static str> {
    encode(bytes, ["RA", "RB"], "NET_REF", 2, trace)
}
fn state(wire: &str) -> (&str, &str) {
    let fields: Vec<_> = wire.lines().next().unwrap().split('|').collect();
    assert_eq!(fields.len(), 8);
    assert_eq!(fields[0], "BOARDSI_REFERENCE_AREA_V13");
    assert_eq!(&fields[4..], &["UNSUPPORTED", "NOT_RUN", "0", "0"]);
    assert_eq!(wire.lines().count(), 4);
    assert_eq!(wire.lines().last(), Some("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL"));
    (fields[2], fields[3])
}

#[test]
fn positive_area_overlap_is_only_source_candidate() {
    let wire = result(&source(A, B, "NET_REF", false), Some(0)).unwrap();
    assert_eq!(state(&wire), ("POSITIVE_AREA_OVERLAP_CANDIDATE", "STRICT_UNION_CENTRELINE_CANDIDATE"));
    assert!(wire.lines().nth(1).unwrap().starts_with("AREA|0|5241|"));
    assert!(wire.lines().nth(2).unwrap().starts_with("AREA|1|5242|"));
    assert!(!wire.contains("PHYSICAL_PASS"));
}
#[test]
fn one_tick_gap_edge_and_corner_touches_are_not_connections() {
    for (other, expected) in [
        ([251, 100, 400, 200], "DISCONNECTED_ISLAND_RISK"),
        ([250, 100, 400, 200], "BOUNDARY_ONLY_RISK"),
        ([250, 200, 400, 300], "BOUNDARY_ONLY_RISK"),
    ] {
        let wire = result(&source(A, other, "NET_REF", false), Some(0)).unwrap();
        assert_eq!(state(&wire), (expected, "REFERENCE_CORRIDOR_UNPROVEN"));
    }
}
#[test]
fn wrong_net_layer_names_or_cutouts_fail_closed() {
    assert!(result(&source(A, B, "NET_OTHER", false), None).is_err());
    assert!(result(&source(A, B, "NET_REF", true), None).is_err());
    let bytes = source(A, B, "NET_REF", false);
    assert!(encode(&bytes, ["RA", "RB"], "NET_REF", 3, None).is_err());
    assert!(encode(&bytes, ["RA", "RA"], "NET_REF", 2, None).is_err());
    assert!(encode(&bytes, ["RA", "MISSING"], "NET_REF", 2, None).is_err());
    assert!(encode(&bytes, ["RA", "RB"], "NET_WRONG", 2, None).is_err());
}
#[test]
fn canonical_source_sha_changes_and_trace_selection_is_bounded() {
    let bytes = source(A, B, "NET_REF", false);
    let mut modified = bytes.clone();
    modified.extend_from_slice(b"REM EXTRA\n");
    // Source byte mutation after the terminator must never return the old proof.
    assert!(result(&modified, None).is_err() || result(&modified, None).unwrap() != result(&bytes, None).unwrap());
    assert!(result(&bytes, Some(100)).is_err());
    assert_eq!(state(&result(&bytes, None).unwrap()).1, "NOT_REQUESTED");
}
#[test]
fn route_coverage_rejects_l_shape_and_boundary_even_when_area_overlaps() {
    assert_eq!(rectangle_relation(A, B), Ok(SourceState::PositiveAreaOverlap));
    assert!(strict_route_union(A, B, [150, 150], [350, 150]));
    assert!(!strict_route_union([0, 0, 100, 100], [50, 50, 150, 150], [10, 10], [110, 10]));
    assert!(!strict_route_union(A, B, [150, 100], [350, 100]));
    assert!(!strict_route_union(A, B, [150, 150], [350, 170]));
    assert!(!strict_route_union(A, B, [0, 150], [350, 150]));
}
#[test]
fn nonrectangle_and_third_source_area_are_not_accepted() {
    let bytes = source(A, B, "NET_REF", false);
    let text = String::from_utf8(bytes).unwrap();
    let extra = area("RC", [500, 100, 700, 200], "NET_REF", false);
    let extended = text.replace("*ROUTE*", &format!("{extra}*ROUTE*"));
    assert!(result(extended.as_bytes(), None).is_err());
}
