//! B_005 exact source-geometry coverage tests; all public synthetic V9.4 MILS.
use boardsi_import::{copper_contact, copper_preview, reference_geometry, route_preview};
use boardsi_model::ObjectId;

const HEADER: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const AREA: &str = "*LINES*\nREF COPPER 100 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\n";
const SIGNAL: &str = "*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n130 230 1 10 0\n150 230 1 10 0\nU2.2 U3.3\n150 230 1 10 0\n170 230 1 10 0\n*SIGNAL* NET_REF\nR1.1 R2.2\n300 400 2 10 0\n350 400 2 10 0\n";
fn source(area: &str, route: &str) -> String {
    format!("{HEADER}{area}{route}*END*\n")
}
fn assembled(input: &str) -> (boardsi_model::BoardModel, Vec<copper_preview::CopperArea>) {
    let parsed = route_preview::parse(input.as_bytes()).unwrap();
    let mut board = route_preview::into_canonical(input.as_bytes(), &parsed).unwrap();
    let areas = copper_preview::parse(input.as_bytes(), parsed.max_layer).unwrap();
    copper_preview::augment(&mut board, &areas).unwrap();
    copper_contact::augment(&mut board, &areas).unwrap();
    assert!(
        !board.validate().has_errors(),
        "{:#?}",
        board.validate().issues
    );
    (board, areas)
}
fn check(
    board: &boardsi_model::BoardModel,
    areas: &[copper_preview::CopperArea],
    ids: &[ObjectId],
    reference: &str,
    layer: usize,
) -> reference_geometry::ReferenceGeometryAudit {
    reference_geometry::audit(
        board,
        areas,
        ids,
        reference,
        &board.stackup.layers[layer].meta.id,
    )
}
#[test]
fn exact_two_segment_path_inside_source_area_is_geometry_only() {
    let (board, areas) = assembled(&source(AREA, SIGNAL));
    let ids: Vec<_> = board.traces[..2]
        .iter()
        .map(|t| t.meta.id.clone())
        .collect();
    let result = check(&board, &areas, &ids, "NET_REF", 1);
    assert_eq!(result.state, "SOURCE_ORDER_GEOMETRY_ONLY");
    assert!(result.area_id.is_some());
    assert_eq!(result.return_path_p009, "UNSUPPORTED");
    assert_eq!(result.physical_si, "NOT_RUN");
    assert!(!result.solver_ready && !result.cache_reuse_eligible);
}
#[test]
fn single_segment_inside_and_reversed_order_are_supported() {
    let one_signal = SIGNAL.replace("U2.2 U3.3\n150 230 1 10 0\n170 230 1 10 0\n", "");
    let (one_board, one_areas) = assembled(&source(AREA, &one_signal));
    assert_eq!(
        check(
            &one_board,
            &one_areas,
            &[one_board.traces[0].meta.id.clone()],
            "NET_REF",
            1
        )
        .state,
        "SOURCE_ORDER_GEOMETRY_ONLY"
    );
    let (board, areas) = assembled(&source(AREA, SIGNAL));
    let ids = [
        board.traces[1].meta.id.clone(),
        board.traces[0].meta.id.clone(),
    ];
    assert_eq!(
        check(&board, &areas, &ids, "NET_REF", 1).state,
        "SOURCE_ORDER_GEOMETRY_ONLY"
    );
}
#[test]
fn refuses_boundary_crossing_and_one_tick_outside() {
    for route in [
        SIGNAL.replace("170 230 1 10 0", "220 230 1 10 0"),
        SIGNAL.replace("130 230 1 10 0", "100 230 1 10 0"),
        SIGNAL.replace("130 230 1 10 0", "99.99 230 1 10 0"),
    ] {
        let (board, areas) = assembled(&source(AREA, &route));
        let ids: Vec<_> = board.traces[..2]
            .iter()
            .map(|t| t.meta.id.clone())
            .collect();
        assert_eq!(
            check(&board, &areas, &ids, "NET_REF", 1).state,
            "UNSUPPORTED"
        );
    }
}
#[test]
fn concave_polygon_both_endpoints_inside_but_line_exits() {
    let concave = "*LINES*\nREF COPPER 100 200 1 1 0 NET_REF\nCOPCLS 7 0.01 0 2\n0 0\n100 0\n100 40\n40 40\n40 100\n0 100\n0 0\n";
    let route = "*ROUTE*\n*SIGNAL* NET_SIG\nU1.1 U2.2\n130 270 1 10 0\n170 230 1 10 0\n*SIGNAL* NET_REF\nR1.1 R2.2\n300 400 2 10 0\n350 400 2 10 0\n";
    let (board, areas) = assembled(&source(concave, route));
    let ids = [board.traces[0].meta.id.clone()];
    assert_eq!(
        check(&board, &areas, &ids, "NET_REF", 1).state,
        "UNSUPPORTED"
    );
}
#[test]
fn no_net_alias_join_no_cross_layer_reference_or_missing_source() {
    let (board, areas) = assembled(&source(AREA, SIGNAL));
    let ids = [board.traces[0].meta.id.clone()];
    assert_eq!(
        check(&board, &areas, &ids, "NET_SIG", 1).reason,
        "SIGNAL_REFERENCE_NET_IDENTICAL"
    );
    assert_eq!(
        check(&board, &areas, &ids, "NET_REF", 3).state,
        "UNSUPPORTED"
    );
    assert_eq!(
        check(&board, &[], &ids, "NET_REF", 1).reason,
        "SOURCE_INVENTORY_MISMATCH"
    );
    assert_eq!(
        check(&board, &areas, &ids, "MISSING", 1).state,
        "UNSUPPORTED"
    );
}
#[test]
fn rejects_disjoint_same_net_areas_and_unknown_geometry() {
    let split = format!("{AREA}OTHER COPPER 300 200 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n0 0\n100 0\n100 100\n0 100\n0 0\n");
    let (board, areas) = assembled(&source(&split, SIGNAL));
    let ids = [board.traces[0].meta.id.clone(), board.traces[1].meta.id.clone()];
    assert_eq!(
        check(&board, &areas, &ids, "NET_REF", 1).state,
        "SOURCE_ORDER_GEOMETRY_ONLY"
    );
    let no_area = AREA.replace(" 0 NET_REF", " 0 NET_UNKNOWN");
    let (board, areas) = assembled(&source(&no_area, SIGNAL));
    let ids = [board.traces[0].meta.id.clone()];
    assert_eq!(
        check(&board, &areas, &ids, "NET_REF", 1).state,
        "UNSUPPORTED"
    );
}
#[test]
fn rejects_wrong_path_order_duplicates_and_modified_source_polygon() {
    let (board, areas) = assembled(&source(AREA, SIGNAL));
    let first = board.traces[0].meta.id.clone();
    let last = board.traces[2].meta.id.clone();
    assert_eq!(
        check(&board, &areas, &[first.clone(), last], "NET_REF", 1).state,
        "UNSUPPORTED"
    );
    assert_eq!(
        check(
            &board,
            &areas,
            &[first.clone(), first.clone()],
            "NET_REF",
            1
        )
        .reason,
        "DUPLICATE_TRACE"
    );
    let mut altered = areas.clone();
    if let copper_preview::CopperGeometry::Polygon(ref mut ring) = altered[0].geometry {
        ring[1][0] += 1;
    }
    assert_eq!(
        check(&board, &altered, &[first], "NET_REF", 1).state,
        "UNSUPPORTED"
    );
}
#[test]
fn unknown_hole_owner_is_not_positive_and_old_graph_stays_unmodified() {
    let (board, areas) = assembled(&source(AREA, SIGNAL));
    let edges = board.connectivity.edges.clone();
    let ids = [board.traces[0].meta.id.clone()];
    let _ = check(&board, &areas, &ids, "NET_REF", 1);
    assert_eq!(board.connectivity.edges, edges);
    assert!(!board.validate().has_errors());
}

#[test]
fn collinear_backtracking_is_not_accepted_as_a_forward_path() {
    let backward = SIGNAL.replace("170 230 1 10 0", "140 230 1 10 0");
    let (board, areas) = assembled(&source(AREA, &backward));
    let ids: Vec<_> = board.traces[..2]
        .iter()
        .map(|t| t.meta.id.clone())
        .collect();
    assert_eq!(
        check(&board, &areas, &ids, "NET_REF", 1).reason,
        "COLLINEAR_BACKTRACK"
    );
}
#[test]
fn rejects_unselected_source_trace_branch_at_path_interior() {
    let branch = SIGNAL.replace(
        "*SIGNAL* NET_REF",
        "U9.1 U10.2\n140 230 1 10 0\n140 250 1 10 0\n*SIGNAL* NET_REF",
    );
    let (board, areas) = assembled(&source(AREA, &branch));
    let ids: Vec<_> = board.traces[..2]
        .iter()
        .map(|t| t.meta.id.clone())
        .collect();
    assert_eq!(
        check(&board, &areas, &ids, "NET_REF", 1).reason,
        "UNSELECTED_TRACE_TOUCHES_PATH"
    );
    assert_eq!(
        check(&board, &areas, &ids[..1], "NET_REF", 1).reason,
        "UNSELECTED_TRACE_TOUCHES_PATH"
    );
}
