//! B_008 public synthetic PADS V9.4 MILS Reference path/COPCUT geometry regression.
//! The predicate is geometrical only; it MUST NOT promote physical P009 or SI.
use boardsi_import::{copper_contact, copper_preview, reference_geometry, route_preview};
use boardsi_model::{BoundarySegment, ObjectId};

const HEADER: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const OUTER: &str = "*LINES*\nREF COPPER 100 200 2 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\n";
const HOLE: &str = "COPCUT 5 0.01 0 2\n20 20\n40 20\n40 40\n20 40\n20 20\n";
const REF_ROUTE: &str = "*SIGNAL* NET_REF\nR1.1 R2.2\n300 400 2 10 0\n350 400 2 10 0\n";
fn source(area: &str, path: &str) -> String {
    format!("{HEADER}{area}*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n{path}\n{REF_ROUTE}*END*\n")
}
fn single(start: &str, end: &str) -> String {
    format!("{start} 1 10 0\n{end} 1 10 0")
}
fn prepare(input: &str) -> (boardsi_model::BoardModel, Vec<copper_preview::CopperArea>) {
    let parsed = route_preview::parse(input.as_bytes()).unwrap();
    let mut board = route_preview::into_canonical(input.as_bytes(), &parsed).unwrap();
    let areas = copper_preview::parse(input.as_bytes(), parsed.max_layer).unwrap();
    copper_preview::augment(&mut board, &areas).unwrap();
    copper_contact::augment_source_bound(&mut board, &areas, input.as_bytes()).unwrap();
    assert!(!board.validate().has_errors(), "{:#?}", board.validate().issues);
    (board, areas)
}
fn audit(
    board: &boardsi_model::BoardModel,
    areas: &[copper_preview::CopperArea],
    input: &str,
    traces: &[ObjectId],
    reference: &str,
    layer_index: usize,
) -> reference_geometry::ReferenceGeometryAudit {
    reference_geometry::audit_cutout_aware(
        board, areas, input.as_bytes(), traces, reference,
        &board.stackup.layers[layer_index].meta.id,
    )
}
fn first_id(board: &boardsi_model::BoardModel) -> Vec<ObjectId> {
    vec![board.traces[0].meta.id.clone()]
}
#[test]
fn cutout_free_legacy_geometry_is_preserved() {
    let holefree = OUTER.replace("100 200 2 1 0", "100 200 1 1 0");
    let raw = source(&holefree, &single("130 250", "170 250"));
    let (board, areas) = prepare(&raw);
    let ids = first_id(&board);
    assert_eq!(audit(&board, &areas, &raw, &ids, "NET_REF", 1).state,
               "SOURCE_ORDER_GEOMETRY_ONLY");
    assert_eq!(reference_geometry::audit(
        &board, &areas, raw.as_bytes(), &ids, "NET_REF",
        &board.stackup.layers[1].meta.id).state,
        "SOURCE_ORDER_GEOMETRY_ONLY");
}
#[test]
fn single_hole_non_intersecting_route_is_geometry_only_not_physical() {
    let raw = source(&format!("{OUTER}{HOLE}"), &single("130 250", "170 250"));
    let (board, areas) = prepare(&raw);
    let ids = first_id(&board);
    let v = audit(&board, &areas, &raw, &ids, "NET_REF", 1);
    assert_eq!(v.state, "SOURCE_ORDER_GEOMETRY_ONLY");
    assert!(v.area_id.is_some());
    assert_eq!(v.return_path_p009, "UNSUPPORTED");
    assert_eq!(v.physical_si, "NOT_RUN");
    assert!(!v.solver_ready && !v.cache_reuse_eligible);
    // Original B_005 entrypoint deliberately remains hole-free.
    assert_eq!(reference_geometry::audit(
        &board, &areas, raw.as_bytes(), &ids, "NET_REF",
        &board.stackup.layers[1].meta.id).state, "UNSUPPORTED");
}
#[test]
fn rejects_segment_with_both_endpoints_outside_hole_but_crossing_void() {
    let raw = source(&format!("{OUTER}{HOLE}"), &single("110 230", "170 230"));
    let (board, areas) = prepare(&raw);
    assert_eq!(audit(&board, &areas, &raw, &first_id(&board), "NET_REF", 1).state,
               "UNSUPPORTED");
}
#[test]
fn rejects_vertex_touch_and_hole_boundary_even_when_endpoints_are_clear() {
    let area = format!("{OUTER}{HOLE}");
    for (start, end) in [
        ("110 210", "150 250"), // crosses two opposite cutout vertices
        ("110 220", "170 220"), // runs along a cutout edge
        ("120 230", "170 230"), // starts on cutout boundary
    ] {
        let raw = source(&area, &single(start, end));
        let (board, areas) = prepare(&raw);
        let got = audit(&board, &areas, &raw, &first_id(&board), "NET_REF", 1);
        assert_eq!(got.state, "UNSUPPORTED", "{start} -> {end}");
    }
}
#[test]
fn one_source_tick_outside_hole_stays_eligible() {
    let raw = source(&format!("{OUTER}{HOLE}"), &single("110 219.99", "170 219.99"));
    let (board, areas) = prepare(&raw);
    assert_eq!(audit(&board, &areas, &raw, &first_id(&board), "NET_REF", 1).state,
               "SOURCE_ORDER_GEOMETRY_ONLY");
}
#[test]
fn rejects_outer_boundary_and_concave_polygon_excursion() {
    let raw = source(&format!("{OUTER}{HOLE}"), &single("100 250", "170 250"));
    let (board, areas) = prepare(&raw);
    assert_eq!(audit(&board, &areas, &raw, &first_id(&board), "NET_REF", 1).state,
               "UNSUPPORTED");

    let concave = "*LINES*\nREF COPPER 100 200 2 1 0 NET_REF\nCOPCLS 7 0.01 0 2\n0 0\n100 0\n100 40\n40 40\n40 100\n0 100\n0 0\n";
    let inner = HOLE.replace("20 20", "10 10").replace("40 20", "20 10").replace("40 40", "20 20").replace("20 40", "10 20");
    let raw = source(&format!("{concave}{inner}"), &single("130 270", "170 230"));
    let (board, areas) = prepare(&raw);
    assert_eq!(audit(&board, &areas, &raw, &first_id(&board), "NET_REF", 1).state,
               "UNSUPPORTED");
}
#[test]
fn refuses_wrong_reference_net_or_source_layer_order() {
    let raw = source(&format!("{OUTER}{HOLE}"), &single("130 250", "170 250"));
    let (mut board, areas) = prepare(&raw);
    let ids = first_id(&board);
    assert_eq!(audit(&board, &areas, &raw, &ids, "NET_SIG", 1).reason,
               "SIGNAL_REFERENCE_NET_IDENTICAL");
    assert_eq!(audit(&board, &areas, &raw, &ids, "NET_REF", 3).state, "UNSUPPORTED");
    board.stackup.layers[0].order = 5;
    assert_eq!(audit(&board, &areas, &raw, &ids, "NET_REF", 1).reason,
               "ORIGINAL_LAYER_ORDER_MISMATCH");
}
#[test]
fn rejects_original_sha_and_canonical_cutout_spoof() {
    let raw = source(&format!("{OUTER}{HOLE}"), &single("130 250", "170 250"));
    let (mut board, areas) = prepare(&raw);
    let ids = first_id(&board);
    let altered = raw.replace("REF COPPER", "NEWREF COPPER");
    assert_eq!(audit(&board, &areas, &altered, &ids, "NET_REF", 1).reason,
               "ORIGINAL_SOURCE_SHA_MISMATCH");
    if let BoundarySegment::Line { ref mut start, .. } =
        board.areas[0].cutouts[0].boundary.segments[0] {
        start.ticks[0] += 1;
    }
    assert_eq!(audit(&board, &areas, &raw, &ids, "NET_REF", 1).state, "UNSUPPORTED");
}
#[test]
fn rejects_multiple_cutouts_without_claiming_reference_pass() {
    let outer_three = OUTER.replace("100 200 2 1 0", "100 200 3 1 0");
    let two = format!("{outer_three}{HOLE}COPCUT 5 0.01 0 2\n60 60\n80 60\n80 80\n60 80\n60 60\n");
    let raw = source(&two, &single("110 250", "170 250"));
    let parsed = route_preview::parse(raw.as_bytes()).unwrap();
    let mut board = route_preview::into_canonical(raw.as_bytes(), &parsed).unwrap();
    let areas = copper_preview::parse(raw.as_bytes(), parsed.max_layer).unwrap();
    copper_preview::augment(&mut board, &areas).unwrap();
    let ids = first_id(&board);
    assert_eq!(audit(&board, &areas, &raw, &ids, "NET_REF", 1).state,
               "UNSUPPORTED");
}
#[test]
fn rejects_source_trace_modified_in_canonical_model() {
    let raw = source(&format!("{OUTER}{HOLE}"), &single("130 250", "170 250"));
    let (mut board, areas) = prepare(&raw);
    let ids = first_id(&board);
    board.traces[0].centreline[0].ticks[0] += 1;
    assert_eq!(audit(&board, &areas, &raw, &ids, "NET_REF", 1).state,
               "UNSUPPORTED");
}
