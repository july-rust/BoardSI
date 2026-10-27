//! B_007 public synthetic PADS V9.4 MILS source-owned COPCUT contact tests.
use boardsi_import::{
    contact_wire, copper_contact, copper_preview, route_preview, via_copper_contact,
};
use boardsi_model::{BoundarySegment, ConnectionEvidence, ContactSite};

const HEADER: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const VIA: &str = "*VIA*\nSIMPLEVIA 15 2\n-2 35 R\n0 35 R\n";
const AREA: &str = "*LINES*\nZONE COPPER 100 200 2 1 0 NET_A\nCOPCLS 5 0.01 0 1\n0 0\n100 0\n100 100\n0 100\n0 0\nCOPCUT 5 0.01 0 1\n20 20\n40 20\n40 40\n20 40\n20 20\n";
const ROUTE_VIA_FILL: &str = "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 250 1 10 0\n150 250 1 10 0 SIMPLEVIA\n200 250 1 10 0\n";
const ROUTE_VIA_HOLE: &str = "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 230 1 10 0\n130 230 1 10 0 SIMPLEVIA\n200 230 1 10 0\n";
fn text(area: &str, route: &str) -> String {
    format!("{HEADER}{VIA}{area}{route}*END*\n")
}
fn pre_audit(input: &str) -> (boardsi_model::BoardModel, Vec<copper_preview::CopperArea>) {
    let route = route_preview::parse(input.as_bytes()).unwrap();
    let mut board = route_preview::into_canonical(input.as_bytes(), &route).unwrap();
    let areas = copper_preview::parse(input.as_bytes(), route.max_layer).unwrap();
    copper_preview::augment(&mut board, &areas).unwrap();
    (board, areas)
}
fn run(
    input: &str,
) -> (
    boardsi_model::BoardModel,
    copper_contact::AreaContactAudit,
    via_copper_contact::ViaAreaAudit,
) {
    let (mut board, areas) = pre_audit(input);
    let original = contact_wire::legacy_v1_counts(&board);
    let contacts =
        copper_contact::augment_source_bound(&mut board, &areas, input.as_bytes()).unwrap();
    let vias = via_copper_contact::augment(&mut board, input.as_bytes()).unwrap();
    assert_eq!(original, contact_wire::legacy_v1_counts(&board));
    assert!(
        !board.validate().has_errors(),
        "{:#?}",
        board.validate().issues
    );
    (board, contacts, vias)
}

#[test]
fn plated_via_in_remaining_copper_connects_and_preserves_barrel() {
    let (board, trace, via) = run(&text(AREA, ROUTE_VIA_FILL));
    assert_eq!(trace.exact_contacts, 1);
    assert_eq!(via.local_contacts, 1);
    assert_eq!(via.new_barrel_edges, 1);
    assert!(board
        .connectivity
        .edges
        .iter()
        .any(|e| matches!(e.evidence, ConnectionEvidence::ViaBarrel { .. })));
    assert_eq!(board.vias[0].connected_layers.len(), 2);
}

#[test]
fn via_and_trace_in_cutout_are_not_connected_despite_matching_net() {
    let (board, contacts, via) = run(&text(AREA, ROUTE_VIA_HOLE));
    assert_eq!(contacts.exact_contacts, 0);
    assert_eq!(via.local_contacts, 0);
    assert_eq!(board.vias[0].connected_layers.len(), 1);
    assert!(board.areas[0].net.is_none());
}

#[test]
fn trace_sites_around_enclosed_cutout_retain_same_area_reachability() {
    let route = "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n110 210 1 10 0\n80 210 1 10 0\nU3.1 U4.2\n180 280 1 10 0\n210 280 1 10 0\n";
    let (board, contacts, via) = run(&text(AREA, route));
    assert_eq!(contacts.exact_contacts, 2);
    assert_eq!(contacts.same_area_links, 1);
    assert_eq!(via.local_contacts, 0);
    let nodes: Vec<_> = board
        .connectivity
        .nodes
        .iter()
        .filter(|n| matches!(n.site, ContactSite::AreaInterior(_)))
        .collect();
    assert_eq!(nodes.len(), 2);
    assert!(board
        .validated_view()
        .unwrap()
        .connected(&nodes[0].meta.id, &nodes[1].meta.id)
        .unwrap());
}

#[test]
fn cutout_boundary_blocks_and_one_tick_outside_hole_is_valid_copper() {
    let cases = [
        ("130 220", 0usize), // exact cutout edge
        ("120 230", 0),      // exact cutout vertex/edge
        ("130 219.99", 1),   // 1 PADS tick outside cutout
        ("140.01 230", 1),   // 1 PADS tick beyond its right edge
    ];
    for (position, expected) in cases {
        let route =
            format!("*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n{position} 1 10 0\n80 230 1 10 0\n");
        let (_board, contacts, _) = run(&text(AREA, &route));
        assert_eq!(contacts.exact_contacts, expected, "point {position}");
    }
}

#[test]
fn cross_net_or_cross_layer_does_not_create_cutout_contact() {
    for route in [
        "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n150 250 2 10 0\n80 250 2 10 0\n",
        "*ROUTE*\n*SIGNAL* NET_B\nU1.1 U2.2\n150 250 1 10 0\n80 250 1 10 0\n",
    ] {
        let (board, contacts, _) = run(&text(AREA, route));
        assert_eq!(contacts.exact_contacts, 0);
        assert!(board.areas[0].net.is_none());
    }
}

#[test]
fn independent_source_sha_inventory_and_cutout_geometry_are_required() {
    let source = text(AREA, ROUTE_VIA_FILL);
    let (mut board, areas) = pre_audit(&source);
    let changed = source.replace("20 20\n40 20", "21 20\n40 20");
    assert_eq!(
        copper_contact::augment_source_bound(&mut board, &areas, changed.as_bytes()).unwrap_err(),
        "B007_SOURCE_SHA_MISMATCH"
    );
    if let BoundarySegment::Line { ref mut start, .. } =
        board.areas[0].cutouts[0].boundary.segments[0]
    {
        start.ticks[0] += 1;
    }
    assert_eq!(
        copper_contact::augment_source_bound(&mut board, &areas, source.as_bytes()).unwrap_err(),
        "B007_SOURCE_INVENTORY_MISMATCH"
    );
}

#[test]
fn forged_area_witness_moved_into_void_fails_model_validation() {
    let (mut board, _, _) = run(&text(AREA, ROUTE_VIA_FILL));
    let node = board
        .connectivity
        .nodes
        .iter_mut()
        .find(|n| matches!(n.site, ContactSite::AreaInterior(_)))
        .unwrap();
    if let ContactSite::AreaInterior(ref mut p) = node.site {
        p.ticks = [13000, 23000];
    }
    assert!(board.validate().has_errors());
    assert_eq!(
        copper_contact::local_reference_candidate(&board, "NET_A").state,
        "UNSUPPORTED"
    );
}

#[test]
fn unsupported_owner_hole_shapes_fail_closed() {
    let source = text(AREA, ROUTE_VIA_FILL);
    // Owner/layer mismatch or tangent to the outer contour must not parse as filled.
    let other_layer = source.replace("COPCUT 5 0.01 0 1", "COPCUT 5 0.01 0 2");
    assert!(copper_preview::parse(other_layer.as_bytes(), 4).is_err());
    let touching = source.replace("20 20\n40 20", "0 20\n40 20");
    assert!(copper_preview::parse(touching.as_bytes(), 4).is_err());
    let extra = AREA.replace("2 1 0 NET_A", "3 1 0 NET_A")
        + "COPCUT 5 0.01 0 1\n50 50\n60 50\n60 60\n50 60\n50 50\n";
    let (board, areas) = pre_audit(&text(&extra, ROUTE_VIA_FILL));
    let mut board = board;
    let audit = copper_contact::augment_source_bound(
        &mut board,
        &areas,
        text(&extra, ROUTE_VIA_FILL).as_bytes(),
    )
    .unwrap();
    assert_eq!(audit.exact_contacts, 0);
    assert_eq!(audit.unsupported_areas, 1);
}

#[test]
fn source_declared_smd_pad_inside_copper_but_not_cutout_is_a_witness() {
    let sample = include_str!("fixtures/pads_component_simple_v94.asc").replace("\r\n", "\n");
    let existing = "*ROUTE*\n*SIGNAL* NET_A\nU9.1 U9.2\n100 200 1 10 0\n300 200 1 10 0\n";
    let replacement = format!(
        "*LINES*\nZONE COPPER 150 250 2 1 0 NET_A\nCOPCLS 5 0.01 0 1\n0 0\n100 0\n100 100\n0 100\n0 0\nCOPCUT 5 0.01 0 1\n60 60\n80 60\n80 80\n60 80\n60 60\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U9.2\n300 400 1 10 0\n350 400 1 10 0\n"
    );
    assert!(sample.contains(existing));
    let input = sample.replace(existing, &replacement);
    let parsed = route_preview::parse(input.as_bytes()).unwrap();
    let mut board = route_preview::into_canonical(input.as_bytes(), &parsed).unwrap();
    let pads = boardsi_import::part_preview::parse(input.as_bytes(), parsed.max_layer);
    boardsi_import::part_preview::augment_source_bound(&mut board, &pads, input.as_bytes())
        .unwrap();
    let areas = copper_preview::parse(input.as_bytes(), parsed.max_layer).unwrap();
    copper_preview::augment(&mut board, &areas).unwrap();
    let audit = copper_contact::augment_source_bound(&mut board, &areas, input.as_bytes()).unwrap();
    assert_eq!(audit.exact_contacts, 1);
    assert!(
        !board.validate().has_errors(),
        "{:#?}",
        board.validate().issues
    );
    assert!(board
        .connectivity
        .nodes
        .iter()
        .any(|n| matches!(n.site, ContactSite::PadCentre)
            && board.pads.iter().any(|p| p.meta.id == n.conductor)));
}
