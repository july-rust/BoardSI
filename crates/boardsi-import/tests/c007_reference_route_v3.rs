//! C_007 synthetic source route coverage: geometry evidence only, NEVER SI.
use boardsi_import::reference_route_v3::encode;

const HEADER: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const AREA: &str =
    "*LINES*\nREF COPPER 100 200 2 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\nCOPCUT 5 0.01 0 2\n20 20\n40 20\n40 40\n20 40\n20 20\n";
const ROUTE: &str = "110 230 1 10 0\n170 230 1 10 0\n190 230 1 10 0";
fn make(route: &str, extras: &str) -> Vec<u8> {
    format!(
        "{HEADER}{AREA}*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n{route}\n{extras}*SIGNAL* NET_REF\nR1.1 R2.2\n300 400 2 10 0\n350 400 2 10 0\n*END*\n"
    )
    .into_bytes()
}
fn route(bytes: &[u8], ids: &[usize]) -> Result<String, &'static str> {
    encode(bytes, ids, "NET_REF", 2)
}
#[test]
fn connected_two_segment_route_has_segment_indexed_v2_receipts() {
    let result = route(&make(ROUTE, ""), &[0, 1]).unwrap();
    assert!(
        result.starts_with("BOARDSI_REFERENCE_ROUTE_V3|"),
        "{result}"
    );
    assert!(
        result.contains("|2|REFERENCE_SPLIT_RISK|UNSUPPORTED|NOT_RUN|0"),
        "{result}"
    );
    assert!(
        result.contains("SEG|0|") && result.contains("SEG|1|"),
        "{result}"
    );
    assert!(result.contains("I|1|6|1|2"), "{result}");
    assert!(
        result.contains("END|2|1|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL"),
        "{result}"
    );
}
#[test]
fn every_segment_fully_covered_is_still_not_physics() {
    let bytes = make("130 250 1 10 0\n150 250 1 10 0\n170 250 1 10 0", "");
    let result = route(&bytes, &[0, 1]).unwrap();
    assert!(
        result.contains("|2|SOURCE_GEOMETRY_ONLY|UNSUPPORTED|NOT_RUN|0"),
        "{result}"
    );
    assert!(
        result.contains("END|2|0|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL"),
        "{result}"
    );
}
#[test]
fn multiple_gaps_across_segments_stay_source_ordered() {
    let bytes = make("110 230 1 10 0\n170 230 1 10 0\n110 230 1 10 0", "");
    // Returning on the same segment is a source overlap, not a valid route.
    assert!(route(&bytes, &[0, 1]).is_err());
}
#[test]
fn codex_backtracking_past_previous_start_is_rejected() {
    // 130 -> 170 -> 110 retraces the entire previous source segment.
    let bytes = make("130 250 1 10 0\n170 250 1 10 0\n110 250 1 10 0", "");
    assert_eq!(route(&bytes, &[0, 1]), Err("OVERLAPPING_ROUTE_SEGMENTS"));
}
#[test]
fn duplicate_reversed_and_missing_indices_are_rejected() {
    let bytes = make(ROUTE, "");
    for indices in [&[0, 0][..], &[1, 0][..], &[0, 999][..], &[0][..]] {
        assert!(route(&bytes, indices).is_err(), "{indices:?}");
    }
}
#[test]
fn same_name_without_physical_joint_is_not_continuity() {
    let bytes = make(
        ROUTE,
        "*SIGNAL* NET_SIG\nU3.1 U4.2\n240 270 1 10 0\n260 270 1 10 0\n",
    );
    assert_eq!(
        route(&bytes, &[0, 2]),
        Err("ROUTE_ENDPOINT_DISCONNECTED_OR_REVERSED")
    );
}
#[test]
fn cross_net_is_not_a_route() {
    let bytes = make(ROUTE, "");
    assert_eq!(route(&bytes, &[0, 2]), Err("MIXED_ROUTE_NET_OR_LAYER"));
}
#[test]
fn unselected_touching_branch_is_fail_closed() {
    let bytes = make(
        ROUTE,
        "*SIGNAL* NET_SIG\nU3.1 U4.2\n170 230 1 10 0\n170 270 1 10 0\n",
    );
    assert_eq!(route(&bytes, &[0, 1]), Err("UNSELECTED_BRANCH_OR_CROSSING"));
}
#[test]
fn unsupported_reference_never_becomes_a_positive_coverage_state() {
    let bytes = make(ROUTE, "");
    let result = encode(&bytes, &[0, 1], "NOT_A_REFERENCE", 2).unwrap();
    assert!(
        result.contains("|2|UNSUPPORTED|UNSUPPORTED|NOT_RUN|0"),
        "{result}"
    );
}
#[test]
fn source_change_recomputes_hash_and_prevents_receipt_reuse() {
    let a = make(ROUTE, "");
    let b = make("111 230 1 10 0\n170 230 1 10 0\n190 230 1 10 0", "");
    let aa = route(&a, &[0, 1]).unwrap();
    let bb = route(&b, &[0, 1]).unwrap();
    assert_ne!(aa.lines().next(), bb.lines().next());
}
#[test]
fn legacy_v2_is_not_changed_by_v3_audit() {
    let bytes = make(ROUTE, "");
    let before = boardsi_import::reference_gap_intervals::encode(&bytes, 0, "NET_REF", 2).unwrap();
    let _ = route(&bytes, &[0, 1]).unwrap();
    let after = boardsi_import::reference_gap_intervals::encode(&bytes, 0, "NET_REF", 2).unwrap();
    assert_eq!(before, after);
}
