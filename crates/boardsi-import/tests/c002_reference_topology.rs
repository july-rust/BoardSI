//! C_002 public PADS V9.4 MILS source-bound split / transition regression.
use boardsi_import::{copper_preview, reference_topology, route_preview};
use boardsi_model::{BoardModel, BoundarySegment, ObjectId};

const HEADER: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const AREA: &str = "*LINES*\nREF COPPER 100 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\n";
const HOLE: &str = "COPCUT 5 0.01 0 2\n20 20\n40 20\n40 40\n20 40\n20 20\n";
const VIA: &str = "*VIA*\nSIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R\n";
const REF_ROUTE: &str = "*SIGNAL* NET_REF\nR1.1 R2.2\n300 400 2 10 0\n350 400 2 10 0\n";
fn input(area: &str, route: &str, via: &str) -> String {
    format!("{HEADER}{via}{area}*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n{route}\n{REF_ROUTE}*END*\n")
}
fn board(raw: &str) -> BoardModel {
    let parsed = route_preview::parse(raw.as_bytes()).unwrap();
    let mut b = route_preview::into_canonical(raw.as_bytes(), &parsed).unwrap();
    let areas = copper_preview::parse(raw.as_bytes(), parsed.max_layer).unwrap();
    copper_preview::augment(&mut b, &areas).unwrap();
    assert!(!b.validate().has_errors(), "{:#?}", b.validate().issues);
    b
}
fn check(
    b: &BoardModel,
    raw: &str,
    trace_ids: &[ObjectId],
    ref_layers: &[usize],
) -> reference_topology::TopologyRiskAudit {
    let layers: Vec<_> = ref_layers
        .iter()
        .map(|&i| b.stackup.layers[i].meta.id.clone())
        .collect();
    reference_topology::audit(b, raw.as_bytes(), trace_ids, "NET_REF", &layers)
}
fn selected(b: &BoardModel) -> Vec<ObjectId> {
    b.traces
        .iter()
        .filter(|t| t.net == b.traces[0].net)
        .map(|t| t.meta.id.clone())
        .collect()
}
#[test]
fn continuous_reference_is_only_source_geometry_not_si() {
    let raw = input(AREA, "130 250 1 10 0\n170 250 1 10 0", "");
    let b = board(&raw);
    let v = check(&b, &raw, &selected(&b), &[1]);
    assert_eq!(v.state, "SOURCE_GEOMETRY_ONLY");
    assert_eq!(v.return_path_p009, "UNSUPPORTED");
    assert_eq!(v.physical_si, "NOT_RUN");
    assert!(!v.solver_ready && !v.cache_reuse_eligible);
}
#[test]
fn cutout_crossing_with_both_endpoints_on_copper_is_split_risk() {
    let cut = AREA.replace("100 200 1 1 0", "100 200 2 1 0");
    let raw = input(
        &format!("{cut}{HOLE}"),
        "110 230 1 10 0\n170 230 1 10 0",
        "",
    );
    let b = board(&raw);
    let v = check(&b, &raw, &selected(&b), &[1]);
    assert_eq!(v.state, "REFERENCE_SPLIT_RISK");
    assert_eq!(v.risk_source_ticks.len(), 1);
}
#[test]
fn separated_source_areas_never_become_one_continuous_reference() {
    let second =
        "REF2 COPPER 230 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\n";
    let raw = input(
        &format!("{AREA}{second}"),
        "130 250 1 10 0\n270 250 1 10 0",
        "",
    );
    let b = board(&raw);
    let v = check(&b, &raw, &selected(&b), &[1]);
    assert_eq!(v.state, "REFERENCE_SPLIT_RISK");
    assert_eq!(v.reference_area_ids.len(), 2);
}
#[test]
fn original_sha_or_canonical_contour_tampering_fails_closed() {
    let raw = input(AREA, "130 250 1 10 0\n170 250 1 10 0", "");
    let mut b = board(&raw);
    let altered = raw.replace("REF COPPER", "BAD COPPER");
    assert_eq!(
        check(&b, &altered, &selected(&b), &[1]).reason,
        "SOURCE_SHA_MISMATCH"
    );
    if let BoundarySegment::Line { ref mut start, .. } = b.areas[0].outer.segments[0] {
        start.ticks[0] += 1;
    }
    assert_eq!(check(&b, &raw, &selected(&b), &[1]).state, "UNSUPPORTED");
}
#[test]
fn reference_unknown_net_and_wrong_layer_do_not_create_risk_certificates() {
    let raw = input(AREA, "130 250 1 10 0\n170 250 1 10 0", "");
    let b = board(&raw);
    let ids = selected(&b);
    assert_eq!(check(&b, &raw, &ids, &[3]).state, "UNSUPPORTED");
    let layer = [b.stackup.layers[1].meta.id.clone()];
    assert_eq!(
        reference_topology::audit(&b, raw.as_bytes(), &ids, "NET_SIG", &layer).state,
        "UNSUPPORTED"
    );
}
#[test]
fn source_plated_through_via_with_adjacent_reference_is_transition_risk_only() {
    let raw = input(
        AREA,
        "130 250 1 10 0\n150 250 3 10 0 SIMPLEVIA\n170 250 3 10 0",
        VIA,
    );
    let b = board(&raw);
    let ids = selected(&b);
    assert_eq!(ids.len(), 2, "{:#?}", b.traces);
    let result = check(&b, &raw, &ids, &[1, 1]);
    assert_eq!(result.state, "LAYER_TRANSITION_RISK", "{result:#?}");
    assert_eq!(result.risk_source_ticks.len(), 1);
    assert!(result.via_id.is_some());
    assert!(!result.solver_ready);
}
#[test]
fn via_without_source_or_plating_is_never_transition_evidence() {
    let raw = input(
        AREA,
        "130 250 1 10 0\n150 250 3 10 0 SIMPLEVIA\n170 250 3 10 0",
        VIA,
    );
    let mut b = board(&raw);
    let ids = selected(&b);
    b.drills[0].plated = false;
    assert_eq!(check(&b, &raw, &ids, &[1, 1]).state, "UNSUPPORTED");
    let b = board(&raw);
    let altered = raw.replace("SIMPLEVIA 15 3", "FAKEVIA 15 3");
    assert_eq!(check(&b, &altered, &ids, &[1, 1]).state, "UNSUPPORTED");
}
#[test]
fn overlapping_reference_bounds_are_unsupported_not_false_split() {
    let second =
        "REF2 COPPER 150 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\n";
    let raw = input(
        &format!("{AREA}{second}"),
        "130 250 1 10 0\n170 250 1 10 0",
        "",
    );
    let b = board(&raw);
    assert_eq!(
        check(&b, &raw, &selected(&b), &[1]).reason,
        "AMBIGUOUS_REFERENCE_AREA_BOUNDS"
    );
}
