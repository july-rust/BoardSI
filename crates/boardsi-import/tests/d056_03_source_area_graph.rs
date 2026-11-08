//! D_056_03 independent SOURCE fixture acceptance for filled COPPER objects.
//! Only positive local overlap candidates are established here; missing
//! witness does not mean a verified electrical open.
use boardsi_import::electrical_graph_qualification::{
    qualify, OpenDisposition, RequiredNetPath, SourceCoverage,
};
use boardsi_import::layered_source_graph::{
    from_pads_v94_source, ContactKind, CopperFootprint, DeclaredNetCheck,
};

// Git may check out fixture files with CRLF on Windows. Normalize only
// the TEST text before making deliberate source mutations; production
// import still hashes and parses the exact original bytes it receives.
fn fixture_text(bytes: &[u8]) -> Result<String, std::string::FromUtf8Error> {
    String::from_utf8(bytes.to_vec()).map(|text| text.replace("\r\n", "\n"))
}

fn import_with_route(
    original: &str,
    replacement: &str,
) -> boardsi_import::layered_source_graph::LayeredSourceGraph {
    let input =
        fixture_text(include_bytes!("fixtures/pads_copper_filled_v94.asc")).expect("ASCII fixture");
    let modified = input.replace(original, replacement);
    assert_ne!(modified, input, "fixture mutation must be applied");
    from_pads_v94_source(modified.as_bytes()).expect("parsed source areas and route")
}
fn area_contact_count(
    graph: &boardsi_import::layered_source_graph::LayeredSourceGraph,
    kind: ContactKind,
) -> usize {
    graph
        .geometry_contacts
        .iter()
        .filter(|e| e.kind == kind)
        .count()
}

#[test]
fn source_polygon_contains_finite_width_trace_and_reports_cross_net_candidate() {
    // COP_A: L1 explicit filled notched polygon 40..140 / 40..120.
    // SIGNAL0: route endpoint 100,60 strictly in COP_A copper (not a hole).
    let graph = import_with_route("10 20 1 10 0\n100 20 1 10 0", "10 60 1 10 0\n100 60 1 10 0");
    let areas: Vec<_> = graph
        .nodes
        .iter()
        .filter(|node| matches!(node.footprint, CopperFootprint::CopperArea { .. }))
        .collect();
    assert_eq!(areas.len(), 2); // Polygon L1 and distinct Circle L4.
    assert_eq!(area_contact_count(&graph, ContactKind::TraceCopperArea), 1);
    let edge = graph
        .geometry_contacts
        .iter()
        .find(|e| e.kind == ContactKind::TraceCopperArea)
        .expect("positive area-contact witness");
    assert_eq!(edge.layer, 1);
    assert_eq!(edge.declared_net_check, DeclaredNetCheck::Different);
    assert!(graph.verified_electrical_edges.is_empty());
    let report = qualify(
        &graph,
        graph.source_sha256,
        &SourceCoverage::Partial,
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(report.net_conflicts.len(), 1);
    assert!(!report.source_scope_qualified);
}

#[test]
fn trace_wholly_inside_copcut_hole_has_no_positive_area_contact() {
    let source =
        fixture_text(include_bytes!("fixtures/pads_copper_cutout_v94.asc")).expect("ASCII source");
    // First hole: X 60..80/Y 60..80; this short, narrow trace has no
    // copper footprint outside its source-owned hole.
    let bytes = source.replace("10 20 1 10 0\n100 20 1 10 0", "68 70 1 2 0\n72 70 1 2 0");
    assert_ne!(source, bytes);
    let graph = from_pads_v94_source(bytes.as_bytes()).unwrap();
    assert_eq!(
        graph
            .nodes
            .iter()
            .filter(|n| matches!(n.footprint, CopperFootprint::CopperArea { cutout_count: 2 }))
            .count(),
        1
    );
    assert_eq!(area_contact_count(&graph, ContactKind::TraceCopperArea), 0);
    assert!(qualify(
        &graph,
        graph.source_sha256,
        &SourceCoverage::Partial,
        &[],
        &[]
    )
    .unwrap()
    .net_conflicts
    .is_empty());
}

#[test]
fn copper_circle_is_layer_specific_and_unknown_net_stays_unknown() {
    // Native COP_B is a L4 circle centred at (240, 130), radius 20.
    let graph = import_with_route(
        "10 20 1 10 0\n100 20 1 10 0",
        "230 130 4 10 0\n250 130 4 10 0",
    );
    assert_eq!(area_contact_count(&graph, ContactKind::TraceCopperArea), 1);
    let edge = graph
        .geometry_contacts
        .iter()
        .find(|e| e.kind == ContactKind::TraceCopperArea)
        .unwrap();
    assert_eq!(edge.layer, 4);
    assert_eq!(edge.declared_net_check, DeclaredNetCheck::Unknown);
    assert!(graph.nodes.iter().any(|n| n.physical_copper_layer == 4
        && n.declared_net.is_none()
        && matches!(n.footprint, CopperFootprint::CopperArea { .. })));
}

#[test]
fn identical_source_polygons_on_same_layer_form_area_to_area_short_candidate() {
    let source = fixture_text(include_bytes!("fixtures/pads_copper_filled_v94.asc")).unwrap();
    let first = "COP_A COPPER 40 40 1 1 0 GND\nCOPCLS 7 0.01 0 1\n0 0\n100 0\n100 40\n60 40\n60 80\n0 80\n0 0";
    assert!(source.contains(first));
    let duplicate = first
        .replace("COP_A COPPER", "COP_X COPPER")
        .replace("0 GND\nCOPCLS", "0 NET_X\nCOPCLS");
    let combined = source.replace("COP_B COPPER", &format!("{duplicate}\nCOP_B COPPER"));
    let graph = from_pads_v94_source(combined.as_bytes()).unwrap();
    assert_eq!(
        area_contact_count(&graph, ContactKind::CopperAreaCopperArea),
        1
    );
    let report = qualify(
        &graph,
        graph.source_sha256,
        &SourceCoverage::Partial,
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(report.net_conflicts.len(), 1);
    assert!(!report.source_scope_qualified);
}

#[test]
fn coincident_shapes_on_different_layers_are_not_merged() {
    let source = fixture_text(include_bytes!("fixtures/pads_copper_filled_v94.asc")).unwrap();
    let first = "COP_A COPPER 40 40 1 1 0 GND\nCOPCLS 7 0.01 0 1\n0 0\n100 0\n100 40\n60 40\n60 80\n0 80\n0 0";
    let duplicate = first
        .replace("COP_A COPPER", "COP_X COPPER")
        .replace("0 GND\nCOPCLS", "0 NET_X\nCOPCLS")
        .replace("COPCLS 7 0.01 0 1", "COPCLS 7 0.01 0 4");
    let combined = source.replace("COP_B COPPER", &format!("{duplicate}\nCOP_B COPPER"));
    let graph = from_pads_v94_source(combined.as_bytes()).unwrap();
    assert_eq!(
        area_contact_count(&graph, ContactKind::CopperAreaCopperArea),
        0
    );
}

#[test]
fn sampled_area_graph_cannot_promote_disconnection_to_verified_open() {
    // Even an attested complete source inventory cannot turn the limited
    // interior-sample contact search into a proof of no other positive contact.
    let raw = fixture_text(include_bytes!("fixtures/pads_copper_filled_v94.asc")).unwrap();
    let bytes = raw.replace("0 GND\nCOPCLS", "0 SIGNAL0\nCOPCLS");
    let graph = from_pads_v94_source(bytes.as_bytes()).unwrap();
    let trace = graph
        .nodes
        .iter()
        .find(|n| {
            n.declared_net.as_deref() == Some("SIGNAL0")
                && matches!(&n.footprint, CopperFootprint::Stroke { .. })
        })
        .unwrap();
    let area = graph
        .nodes
        .iter()
        .find(|n| {
            n.declared_net.as_deref() == Some("SIGNAL0")
                && matches!(&n.footprint, CopperFootprint::CopperArea { .. })
        })
        .unwrap();
    let req = RequiredNetPath {
        net: "SIGNAL0".into(),
        endpoint_source_keys: vec![trace.source_key.clone(), area.source_key.clone()],
    };
    let r = qualify(
        &graph,
        graph.source_sha256,
        &SourceCoverage::QualifiedSourceScope {
            sha256: graph.source_sha256,
            evidence_reference: "independent-complete-source-inventory".into(),
        },
        &[],
        &[req],
    )
    .unwrap();
    assert_eq!(r.open_witnesses.len(), 1);
    assert_eq!(
        r.open_witnesses[0].disposition,
        OpenDisposition::CandidateOnly
    );
    assert!(r
        .diagnostics
        .contains(&"COPPER_AREA_CONTACT_COVERAGE_PARTIAL".to_owned()));
}

#[test]
fn circular_cutout_with_independent_island_has_no_false_short() {
    let raw = fixture_text(include_bytes!("fixtures/pads_copper_circular_void_v94.asc")).unwrap();
    // COP_CIRC: outer source polygon from (50,50) to (170,170);
    // first circular cutout is centred at (90,80), radius 10.
    // COP_INNER lies entirely within that source-owned empty cutout.
    let source = raw.replace(
        "*END*",
        "COP_INNER COPPER 90 80 1 1 0 NET_INNER\n\
         COPCIR 2 0.01 0 1\n\
         -5 0\n5 0\n\
         *ROUTE*\n*SIGNAL* OTHER\nU1.1 U2.2\n\
         10 20 1 5 0\n20 20 1 5 0\n*END*",
    );
    assert_ne!(raw, source);
    let graph = from_pads_v94_source(source.as_bytes()).unwrap();
    assert_eq!(
        area_contact_count(&graph, ContactKind::CopperAreaCopperArea),
        0,
        "a circle inside an owned void must never contact its enclosing copper",
    );
    let report = qualify(
        &graph,
        graph.source_sha256,
        &SourceCoverage::Partial,
        &[],
        &[],
    )
    .unwrap();
    assert!(report.net_conflicts.is_empty());
}
