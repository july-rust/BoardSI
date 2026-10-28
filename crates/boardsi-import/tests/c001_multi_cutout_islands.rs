//! C_001 source-owned straight multi-COPCUT and disconnected Area regression.
use boardsi_import::{contact_wire, copper_contact, copper_preview, route_preview, via_copper_contact};
use boardsi_model::{BoundarySegment, ContactSite};

const HEADER: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const VIA: &str = "*VIA*\nSIMPLEVIA 15 2\n-2 35 R\n0 35 R\n";
const AREA: &str = "*LINES*\nZONE COPPER 100 200 3 1 0 NET_A\nCOPCLS 5 0.01 0 1\n0 0\n100 0\n100 100\n0 100\n0 0\nCOPCUT 5 0.01 0 1\n20 20\n40 20\n40 40\n20 40\n20 20\nCOPCUT 5 0.01 0 1\n60 60\n80 60\n80 80\n60 80\n60 60\n";
const ROUTE: &str = "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n110 210 1 10 0\n80 210 1 10 0\n";
const VIA_ROUTE: &str = "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 250 1 10 0\n150 250 1 10 0 SIMPLEVIA\n200 250 1 10 0\n";
fn text(area: &str, route: &str) -> String { format!("{HEADER}{VIA}{area}{route}*END*\n") }
fn load(input: &str) -> (boardsi_model::BoardModel, Vec<copper_preview::CopperArea>) {
    let parsed = route_preview::parse(input.as_bytes()).unwrap();
    let mut board = route_preview::into_canonical(input.as_bytes(), &parsed).unwrap();
    let areas = copper_preview::parse(input.as_bytes(), parsed.max_layer).unwrap();
    copper_preview::augment(&mut board, &areas).unwrap();
    (board, areas)
}
fn run(input: &str) -> (boardsi_model::BoardModel, copper_contact::AreaContactAudit) {
    let (mut board, areas) = load(input);
    let old_wire = contact_wire::legacy_v1_counts(&board);
    let contacts = copper_contact::augment_source_bound_multi(&mut board, &areas, input.as_bytes()).unwrap();
    assert_eq!(old_wire, contact_wire::legacy_v1_counts(&board));
    assert!(!board.validate().has_errors(), "{:#?}", board.validate().issues);
    (board, contacts)
}
#[test]
fn multi_hole_positive_and_legacy_one_hole_limit() {
    let input = text(AREA, ROUTE);
    let (mut legacy, areas) = load(&input);
    assert_eq!(copper_contact::augment_source_bound(&mut legacy, &areas, input.as_bytes()).unwrap().unsupported_areas, 1);
    let (board, audit) = run(&input);
    assert_eq!(audit.exact_contacts, 1);
    assert_eq!(audit.unsupported_areas, 0);
    assert_eq!(board.areas[0].net, board.traces[0].net);
}
#[test]
fn both_hole_interiors_and_boundaries_block_contacts() {
    for site in ["130 230", "120 230", "170 270", "180 280", "160 260"] {
        let route = format!("*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n{site} 1 10 0\n80 230 1 10 0\n");
        let (_, audit) = run(&text(AREA, &route));
        assert_eq!(audit.exact_contacts, 0, "{site}");
    }
}
#[test]
fn two_disjoint_same_net_areas_do_not_merge() {
    let second = "SECOND COPPER 400 200 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n0 0\n100 0\n100 100\n0 100\n0 0\n";
    let route = "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n110 210 1 10 0\n80 210 1 10 0\nU3.1 U4.2\n410 210 1 10 0\n380 210 1 10 0\n";
    let (board, audit) = run(&text(&format!("{AREA}{second}"), route));
    assert_eq!(audit.exact_contacts, 2);
    assert_eq!(audit.same_area_links, 0);
    let nodes: Vec<_> = board.connectivity.nodes.iter().filter(|n| matches!(n.site, ContactSite::AreaInterior(_))).collect();
    assert_eq!(nodes.len(), 2);
    assert!(!board.validated_view().unwrap().connected(&nodes[0].meta.id, &nodes[1].meta.id).unwrap());
}
#[test]
fn overlapping_source_area_bounds_are_conservatively_unsupported() {
    let second = "SECOND COPPER 150 200 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n0 0\n100 0\n100 100\n0 100\n0 0\n";
    let (_, audit) = run(&text(&format!("{AREA}{second}"), ROUTE));
    assert_eq!(audit.exact_contacts, 0);
    assert_eq!(audit.unsupported_areas, 2);
}
#[test]
fn changed_original_bytes_or_canonical_second_hole_fail_closed() {
    let input = text(AREA, ROUTE);
    let (mut board, areas) = load(&input);
    let changed = input.replace("60 60", "61 60");
    assert_eq!(copper_contact::augment_source_bound_multi(&mut board, &areas, changed.as_bytes()).unwrap_err(), "B007_SOURCE_SHA_MISMATCH");
    let (mut board, areas) = load(&input);
    if let BoundarySegment::Line { ref mut start, .. } = board.areas[0].cutouts[1].boundary.segments[0] {
        start.ticks[0] += 1;
    }
    assert_eq!(copper_contact::augment_source_bound_multi(&mut board, &areas, input.as_bytes()).unwrap_err(), "B007_SOURCE_INVENTORY_MISMATCH");
}
#[test]
fn cross_net_trace_never_joins_same_layer_area() {
    let route = "*ROUTE*\n*SIGNAL* NET_B\nU1.1 U2.2\n110 210 1 10 0\n80 210 1 10 0\n";
    let (board, audit) = run(&text(AREA, route));
    assert_eq!(audit.exact_contacts, 0);
    assert_eq!(audit.cross_net_coincidences, 1);
    assert!(board.areas[0].net.is_none());
}
#[test]
fn nested_source_holes_are_rejected() {
    let nested = AREA.replace("60 60\n80 60\n80 80\n60 80\n60 60", "25 25\n35 25\n35 35\n25 35\n25 25");
    let input = text(&nested, ROUTE);
    assert!(copper_preview::parse(input.as_bytes(), 4).is_err());
}
#[test]
fn canonical_witness_moved_to_second_void_is_invalid() {
    let input = text(AREA, ROUTE);
    let (mut board, _) = run(&input);
    let node = board.connectivity.nodes.iter_mut().find(|n| matches!(n.site, ContactSite::AreaInterior(_))).unwrap();
    if let ContactSite::AreaInterior(ref mut p) = node.site { p.ticks = [17000, 27000]; }
    assert!(board.validate().has_errors());
}
#[test]
fn via_centre_on_remaining_copper_is_qualified_without_synthetic_barrel() {
    let input = text(AREA, VIA_ROUTE);
    let (mut board, areas) = load(&input);
    let contacts = copper_contact::augment_source_bound_multi(&mut board, &areas, input.as_bytes()).unwrap();
    let vias = via_copper_contact::augment_multi(&mut board, input.as_bytes()).unwrap();
    assert_eq!(contacts.exact_contacts, 2);
    assert_eq!(vias.local_contacts, 1);
    assert!(!board.validate().has_errors(), "{:#?}", board.validate().issues);
}
#[test]
fn source_via_in_hole_does_not_gain_contact() {
    let input = text(AREA, "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 230 1 10 0\n130 230 1 10 0 SIMPLEVIA\n200 230 1 10 0\n");
    let (mut board, areas) = load(&input);
    let contacts = copper_contact::augment_source_bound_multi(&mut board, &areas, input.as_bytes()).unwrap();
    let vias = via_copper_contact::augment_multi(&mut board, input.as_bytes()).unwrap();
    assert_eq!(contacts.exact_contacts, 0);
    assert_eq!(vias.local_contacts, 0);
}
