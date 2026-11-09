//! D_056_03: independently hand-constructed source Polygon cross witnesses.
//! This specifically catches a real electrical graph blind spot: two long,
//! thin COPCLS bars cross at midspan but NO vertex lies inside the other.
use boardsi_import::electrical_graph_qualification::{
    qualify_pads_source, SourceCoverage,
};
use boardsi_import::layered_source_graph::{ContactKind, from_pads_v94_source};

fn source_with_vertical(x0: i64, x1: i64) -> Vec<u8> {
    let fixture = String::from_utf8(
        include_bytes!("fixtures/pads_copper_filled_v94.asc").to_vec(),
    )
    .unwrap()
    .replace("\r\n", "\n");
    assert!(fixture.contains("COP_B COPPER"));
    let a = "CROSS_A COPPER 1000 1000 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n0 40\n100 40\n100 60\n0 60\n0 40";
    let b = format!(
        "CROSS_B COPPER 1000 1000 1 1 0 NET_B\nCOPCLS 5 0.01 0 1\n{x0} 0\n{x1} 0\n{x1} 100\n{x0} 100\n{x0} 0"
    );
    fixture
        .replacen("COP_B COPPER", &format!("{a}\n{b}\nCOP_B COPPER"), 1)
        .into_bytes()
}

fn graph_area_contact_count(source: &[u8]) -> usize {
    let graph = from_pads_v94_source(source).unwrap();
    graph
        .geometry_contacts
        .iter()
        .filter(|e| e.kind == ContactKind::CopperAreaCopperArea)
        .count()
}

#[test]
fn crossed_midspan_polygon_bars_connect_despite_no_vertex_containment() {
    let bytes = source_with_vertical(40, 60);
    let graph = from_pads_v94_source(&bytes).unwrap();
    assert_eq!(graph_area_contact_count(&bytes), 1);
    let candidates: Vec<_> = graph
        .geometry_contacts
        .iter()
        .filter(|e| e.kind == ContactKind::CopperAreaCopperArea)
        .collect();
    assert_eq!(candidates[0].layer, 1);
    assert!(graph.verified_electrical_edges.is_empty());
    let report = qualify_pads_source(&bytes, &SourceCoverage::Partial, &[], &[]).unwrap();
    assert!(report.net_conflicts.iter().any(|c|
        (c.first_net == "NET_A" && c.second_net == "NET_B")
        || (c.first_net == "NET_B" && c.second_net == "NET_A")));
    assert!(!report.source_scope_qualified);
}

#[test]
fn nonoverlap_polygon_pairs_stay_disconnected() {
    let bytes = source_with_vertical(120, 140);
    assert_eq!(graph_area_contact_count(&bytes), 0);
    let report = qualify_pads_source(&bytes, &SourceCoverage::Partial, &[], &[]).unwrap();
    assert!(report.net_conflicts.is_empty());
}

#[test]
fn shared_boundary_only_is_not_a_positive_area_contact() {
    let bytes = source_with_vertical(100, 120);
    assert_eq!(graph_area_contact_count(&bytes), 0);
}

#[test]
fn source_reimport_changes_digest_and_qualified_net_conflicts() {
    let connected = source_with_vertical(40, 60);
    let disconnected = source_with_vertical(120, 140);
    let yes = qualify_pads_source(&connected, &SourceCoverage::Partial, &[], &[]).unwrap();
    let no = qualify_pads_source(&disconnected, &SourceCoverage::Partial, &[], &[]).unwrap();
    assert_ne!(yes.source_sha256, no.source_sha256);
    assert!(yes.net_conflicts.len() > no.net_conflicts.len());
}

#[test]
fn stale_external_coverage_attestation_cannot_authorize_changed_source() {
    let initial = source_with_vertical(40, 60);
    let changed = source_with_vertical(120, 140);
    let first = qualify_pads_source(&initial, &SourceCoverage::Partial, &[], &[]).unwrap();
    let stale = SourceCoverage::QualifiedSourceScope {
        sha256: first.source_sha256,
        evidence_reference: "source-verification-by-independent-reviewer".into(),
    };
    assert_eq!(
        qualify_pads_source(&changed, &stale, &[], &[]).unwrap_err(),
        "D05603_INVALID_COVERAGE_EVIDENCE"
    );
}

#[test]
fn invented_barrel_proof_is_rejected_on_original_source() {
    use boardsi_import::electrical_graph_qualification::BarrelProcessWitness;
    let bytes = source_with_vertical(40, 60);
    let first = qualify_pads_source(&bytes, &SourceCoverage::Partial, &[], &[]).unwrap();
    let fake = BarrelProcessWitness {
        sha256: first.source_sha256,
        via_key: "fictitious:via".into(),
        first_layer: 1,
        last_layer: 4,
        documented_process_reference: "not-a-vendor-certificate".into(),
        qualified_transition_reference: "no-actual-source-identity".into(),
    };
    assert_eq!(
        qualify_pads_source(&bytes, &SourceCoverage::Partial, &[fake], &[]).unwrap_err(),
        "D05603_UNKNOWN_BARREL_PROOF"
    );
}
