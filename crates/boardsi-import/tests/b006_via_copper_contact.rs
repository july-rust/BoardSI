//! B_006 synthetic PADS V9.4 MILS Via-to-Copper strict local contact regression.
use boardsi_import::{
    contact_wire, copper_contact, copper_preview, route_preview, via_copper_contact,
};
use boardsi_model::{ConnectionEvidence, ContactSite};

const HEADER: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const VIA: &str = "*VIA*\nSIMPLEVIA 15 2\n-2 35 R\n0 35 R\n";
const AREA: &str = "*LINES*\nZONE COPPER 100 200 1 1 0 NET_A\nCOPCLS 5 0.01 0 4\n0 0\n100 0\n100 100\n0 100\n0 0\n";
const ROUTE: &str = "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 250 1 10 0\n150 250 1 10 0 SIMPLEVIA\n200 250 1 10 0\n";

fn source(area: &str, route: &str) -> String {
    format!("{HEADER}{VIA}{area}{route}*END*\n")
}

fn assemble(input: &str) -> boardsi_model::BoardModel {
    let route = route_preview::parse(input.as_bytes()).unwrap();
    let mut board = route_preview::into_canonical(input.as_bytes(), &route).unwrap();
    let areas = copper_preview::parse(input.as_bytes(), route.max_layer).unwrap();
    copper_preview::augment(&mut board, &areas).unwrap();
    copper_contact::augment(&mut board, &areas).unwrap();
    assert!(
        !board.validate().has_errors(),
        "{:#?}",
        board.validate().issues
    );
    board
}

#[test]
fn plated_through_via_local_area_witness_joins_existing_trace_layer() {
    let text = source(AREA, ROUTE);
    let mut board = assemble(&text);
    assert_eq!(board.vias.len(), 1);
    assert_eq!(board.vias[0].connected_layers.len(), 1);
    let before = contact_wire::legacy_v1_counts(&board);
    let audit = via_copper_contact::augment(&mut board, text.as_bytes()).unwrap();
    assert_eq!(audit.local_contacts, 1);
    assert_eq!(audit.new_barrel_edges, 1);
    assert_eq!(board.vias[0].connected_layers.len(), 2);
    assert!(
        !board.validate().has_errors(),
        "{:#?}",
        board.validate().issues
    );
    assert!(board
        .connectivity
        .edges
        .iter()
        .any(|e| matches!(e.evidence, ConnectionEvidence::ViaBarrel { .. })));
    let area = board
        .connectivity
        .nodes
        .iter()
        .find(|n| matches!(n.site, ContactSite::AreaInterior(_)))
        .unwrap();
    let via_top = board
        .connectivity
        .nodes
        .iter()
        .find(|n| n.conductor == board.vias[0].meta.id && n.layer == board.vias[0].span.first_layer)
        .unwrap();
    assert!(board
        .validated_view()
        .unwrap()
        .connected(&area.meta.id, &via_top.meta.id)
        .unwrap());
    assert_eq!(before, contact_wire::legacy_v1_counts(&board));
}

#[test]
fn exact_boundary_outside_and_other_layer_are_not_contacts() {
    for area in [
        AREA.replace("ZONE COPPER 100 200", "ZONE COPPER 150 200"),
        AREA.replace("ZONE COPPER 100 200", "ZONE COPPER 300 200"),
        AREA.replace("COPCLS 5 0.01 0 4", "COPCLS 5 0.01 0 2"),
    ] {
        let text = source(&area, ROUTE);
        let mut board = assemble(&text);
        let result = via_copper_contact::augment(&mut board, text.as_bytes()).unwrap();
        assert_eq!(result.local_contacts, 0);
        assert!(board.validate().issues.is_empty() || !board.validate().has_errors());
    }
}

#[test]
fn different_net_and_unknown_net_never_create_edges() {
    let another = format!("{ROUTE}*SIGNAL* NET_B\nU3.1 U4.2\n500 500 1 10 0\n600 500 1 10 0\n");
    for net in ["NET_B", "NOT_IN_ROUTE"] {
        let area = AREA.replace("0 NET_A", &format!("0 {net}"));
        let text = source(&area, &another);
        let mut board = assemble(&text);
        let result = via_copper_contact::augment(&mut board, text.as_bytes()).unwrap();
        assert_eq!(result.local_contacts, 0);
        assert_eq!(board.vias[0].connected_layers.len(), 1);
    }
}

#[test]
fn owned_cutout_fails_closed_without_fabricating_conductor() {
    let area = AREA
        .replace("100 200 1 1 0 NET_A", "100 200 2 1 0 NET_A")
        .replace(
            "0 100\n0 0\n",
            "0 100\n0 0\nCOPCUT 5 0.01 0 4\n20 20\n40 20\n40 40\n20 40\n20 20\n",
        );
    let text = source(&area, ROUTE);
    let mut board = assemble(&text);
    let result = via_copper_contact::augment(&mut board, text.as_bytes()).unwrap();
    assert_eq!(result.local_contacts, 0);
    assert!(result.unsupported_candidates > 0);
}

#[test]
fn mutated_original_bytes_and_canonical_via_fail_closed() {
    let text = source(AREA, ROUTE);
    let mut board = assemble(&text);
    let changed = text.replace("150 250 1 10 0 SIMPLEVIA", "151 250 1 10 0 SIMPLEVIA");
    assert_eq!(
        via_copper_contact::augment(&mut board, changed.as_bytes()).unwrap_err(),
        "B006_SOURCE_SHA_MISMATCH"
    );
    board.vias[0].position.ticks[0] += 1;
    assert!(via_copper_contact::augment(&mut board, text.as_bytes()).is_err());
}

#[test]
fn modified_drill_or_padstack_cannot_be_source_verified() {
    let text = source(AREA, ROUTE);
    let mut a = assemble(&text);
    a.drills[0].plated = false;
    assert!(via_copper_contact::augment(&mut a, text.as_bytes()).is_err());
    let mut b = assemble(&text);
    b.padstacks[0].entries.pop();
    assert!(via_copper_contact::augment(&mut b, text.as_bytes()).is_err());
}

#[test]
fn source_area_geometry_cannot_be_tampered() {
    let text = source(AREA, ROUTE);
    let mut board = assemble(&text);
    if let boardsi_model::BoundarySegment::Line { ref mut start, .. } =
        board.areas[0].outer.segments[0]
    {
        start.ticks[0] += 1;
    }
    assert!(via_copper_contact::augment(&mut board, text.as_bytes()).is_err());
}

#[test]
fn repeated_success_is_not_silently_accepted() {
    let text = source(AREA, ROUTE);
    let mut board = assemble(&text);
    via_copper_contact::augment(&mut board, text.as_bytes()).unwrap();
    assert_eq!(
        via_copper_contact::augment(&mut board, text.as_bytes()).unwrap_err(),
        "B006_ALREADY_APPLIED"
    );
}

#[test]
fn source_unknown_extra_connected_layer_is_rejected() {
    let text = source(AREA, ROUTE);
    let mut board = assemble(&text);
    let layer = board.vias[0].span.last_layer.clone();
    board.vias[0].connected_layers.push(layer);
    assert!(!board.validate().has_errors());
    assert_eq!(
        via_copper_contact::augment(&mut board, text.as_bytes()).unwrap_err(),
        "B006_VIA_SOURCE_MISMATCH"
    );
}

#[test]
fn two_supported_areas_on_distinct_layers_get_independent_contacts() {
    let via = VIA.replace(
        "SIMPLEVIA 15 2\n-2 35 R\n0 35 R",
        "SIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R",
    );
    let second = AREA
        .replace("ZONE COPPER", "SECOND COPPER")
        .replace("COPCLS 5 0.01 0 4", "COPCLS 5 0.01 0 2");
    let text = format!("{HEADER}{via}{AREA}{second}{ROUTE}*END*\n");
    let mut board = assemble(&text);
    let audit = via_copper_contact::augment(&mut board, text.as_bytes()).unwrap();
    assert_eq!(audit.local_contacts, 2);
    assert_eq!(board.vias[0].connected_layers.len(), 3);
    assert!(!board.validate().has_errors(), "{:#?}", board.validate().issues);
    assert_eq!(board.areas.len(), 2);
}
