//! B_004 source-grid Copper Area contact regression; synthetic PADS V9.4 MILS.
use boardsi_import::{copper_contact, copper_preview, route_preview};
use boardsi_model::{ConnectionEvidence, ContactSite};

fn assemble(source: &str) -> (boardsi_model::BoardModel, copper_contact::AreaContactAudit) {
    let parsed = route_preview::parse(source.as_bytes()).unwrap();
    let mut board = route_preview::into_canonical(source.as_bytes(), &parsed).unwrap();
    let areas = copper_preview::parse(source.as_bytes(), parsed.max_layer).unwrap();
    copper_preview::augment(&mut board, &areas).unwrap();
    let result = copper_contact::augment(&mut board, &areas).unwrap();
    assert!(
        !board.validate().has_errors(),
        "{:#?}",
        board.validate().issues
    );
    (board, result)
}
const HEADER: &str =
    "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n";
const AREA: &str = "*LINES*\nZONE COPPER 100 200 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n0 0\n100 0\n100 100\n0 100\n0 0\n";
const ROUTE: &str = "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n130 230 1 10 0\n80 230 1 10 0\n";
fn source(area: &str, route: &str) -> String {
    format!("{HEADER}{area}{route}*END*\n")
}
#[test]
fn exact_endpoint_inside_filled_polygon_connects_locally() {
    let (board, audit) = assemble(&source(AREA, ROUTE));
    assert_eq!(audit.exact_contacts, 1);
    assert_eq!(audit.same_area_links, 0);
    assert_eq!(audit.unsupported_areas, 0);
    assert_eq!(board.areas[0].net, board.traces[0].net);
    let report = copper_contact::local_reference_candidate(&board, "NET_A");
    assert_eq!(report.state, "LOCAL_CONTACT_VERIFIED");
    assert_eq!(report.area_ids.len(), 1);
    assert_eq!(report.conductor_ids.len(), 1);
    assert_eq!(report.conductor_ids[0], board.traces[0].meta.id);
    assert!(board
        .connectivity
        .edges
        .iter()
        .any(|e| matches!(e.evidence, ConnectionEvidence::SameLayerContact { .. })));
}
#[test]
fn area_connects_only_its_own_interior_witnesses() {
    let route="*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n130 230 1 10 0\n80 230 1 10 0\nU3.1 U4.2\n150 270 1 10 0\n80 270 1 10 0\n";
    let (board, a) = assemble(&source(AREA, route));
    assert_eq!(a.exact_contacts, 2);
    assert_eq!(a.same_area_links, 1);
    let root: Vec<_> = board
        .connectivity
        .nodes
        .iter()
        .filter(|n| matches!(n.site, ContactSite::AreaWitness { .. }))
        .map(|n| n.meta.id.clone())
        .collect();
    assert_eq!(root.len(), 2);
    assert!(board
        .validated_view()
        .unwrap()
        .connected(&root[0], &root[1])
        .unwrap());
}
#[test]
fn no_contact_on_boundary_outside_or_other_layer() {
    for route in [
        "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 240 1 10 0\n80 240 1 10 0\n",
        "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n99.99 240 1 10 0\n80 240 1 10 0\n",
        "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n130 230 2 10 0\n80 230 2 10 0\n",
    ] {
        let (b, a) = assemble(&source(AREA, route));
        assert_eq!(a.exact_contacts, 0);
        assert!(b.areas[0].net.is_none());
        assert_eq!(
            copper_contact::local_reference_candidate(&b, "NET_A").state,
            "UNSUPPORTED"
        );
    }
}
#[test]
fn conflicting_declared_net_records_no_electrical_contact() {
    let route="*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n300 400 1 10 0\n350 400 1 10 0\n*SIGNAL* NET_B\nU3.1 U4.2\n130 230 1 10 0\n80 230 1 10 0\n";
    let (b, a) = assemble(&source(AREA, route));
    assert_eq!(a.exact_contacts, 0);
    assert_eq!(a.cross_net_coincidences, 1);
    assert!(b.areas[0].net.is_none());
}
#[test]
fn two_disconnected_areas_with_same_name_do_not_join() {
    let a=format!("{AREA}SECOND COPPER 400 200 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n0 0\n100 0\n100 100\n0 100\n0 0\n");
    let route="*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n130 230 1 10 0\n80 230 1 10 0\nU3.1 U4.2\n430 230 1 10 0\n380 230 1 10 0\n";
    let (b, report) = assemble(&source(&a, route));
    assert_eq!(report.exact_contacts, 2);
    assert_eq!(report.same_area_links, 0);
    let area_nodes: Vec<_> = b
        .connectivity
        .nodes
        .iter()
        .filter(|n| matches!(n.site, ContactSite::AreaWitness { .. }))
        .collect();
    assert_eq!(area_nodes.len(), 2);
    assert!(!b
        .validated_view()
        .unwrap()
        .connected(&area_nodes[0].meta.id, &area_nodes[1].meta.id)
        .unwrap());
}
#[test]
fn missing_or_unsupported_area_net_stays_unknown() {
    for area in [
        AREA.replace(" 0 NET_A", " 0 UNKNOWN_NET"),
        AREA.replace(" 0 NET_A", ""),
    ] {
        let (b, a) = assemble(&source(&area, ROUTE));
        assert_eq!(a.exact_contacts, 0);
        assert!(a.unsupported_areas > 0);
        assert!(b.areas[0].net.is_none());
    }
}
#[test]
fn source_owned_cutout_is_explicitly_unsupported() {
    let hole_area = AREA
        .replace("1 1 0 NET_A", "2 1 0 NET_A")
        .replace(
            "0 100\n0 0\n",
            "0 100\n0 0\nCOPCUT 5 0.01 0 1\n20 20\n40 20\n40 40\n20 40\n20 20\n",
        );
    let (board, report) = assemble(&source(&hole_area, ROUTE));
    assert_eq!(report.exact_contacts, 0);
    assert_eq!(report.unsupported_areas, 1);
    assert!(board.areas[0].net.is_none());
}
#[test]
fn area_witness_is_independently_checked_by_canonical_model() {
    let (mut b, _) = assemble(&source(AREA, ROUTE));
    let index = b
        .connectivity
        .nodes
        .iter()
        .position(|n| matches!(n.site, ContactSite::AreaWitness { .. }))
        .unwrap();
    if let ContactSite::AreaWitness { ref mut point, .. } = b.connectivity.nodes[index].site {
        point.ticks[0] = 30000;
    }
    assert!(b.validate().has_errors());
}
#[test]
fn legacy_v1_contact_counts_are_unchanged() {
    let src = source(AREA, ROUTE);
    let parsed = route_preview::parse(src.as_bytes()).unwrap();
    let mut b = route_preview::into_canonical(src.as_bytes(), &parsed).unwrap();
    let before = boardsi_import::contact_wire::legacy_v1_counts(&b);
    let areas = copper_preview::parse(src.as_bytes(), 4).unwrap();
    copper_preview::augment(&mut b, &areas).unwrap();
    copper_contact::augment(&mut b, &areas).unwrap();
    assert_eq!(before, boardsi_import::contact_wire::legacy_v1_counts(&b));
}
