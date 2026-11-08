//! Independent D_056_03 graph oracles. Expected topology is specified as
//! hand-enumerated node partitions, not reproduced from production DFS.
use boardsi_import::electrical_graph_qualification::{
    qualify, BarrelProcessWitness, OpenDisposition, QualifiedEdgeKind, RequiredNetPath,
    SourceCoverage,
};
use boardsi_import::layered_source_graph::{
    ContactKind, CopperFootprint, CopperNode, DeclaredNetCheck, GeometryContact,
    LayeredSourceGraph, UnverifiedBarrel,
};
use boardsi_import::via_shapes::ViaLayerShape;

const SHA: [u8; 32] = [0x56; 32];
const VIA: &str = "via:fixture:line:7";

fn node(key: &str, layer: u32, net: &str, via: bool) -> CopperNode {
    CopperNode {
        source_key: key.to_owned(),
        source_line: Some(7),
        physical_copper_layer: layer,
        declared_net: Some(net.to_owned()),
        footprint: if via {
            CopperFootprint::ViaPad {
                centre: [100, 100],
                outer_ticks: 40,
                drill_ticks: 10,
                shape: ViaLayerShape::Round,
            }
        } else {
            CopperFootprint::Stroke { width_ticks: 10 }
        },
    }
}

fn local(first: usize, second: usize, layer: u32) -> GeometryContact {
    GeometryContact {
        first_node: first,
        second_node: second,
        layer,
        kind: ContactKind::TraceViaPad,
        declared_net_check: DeclaredNetCheck::Same,
    }
}

fn graph(layers: u32, first: u32, last: u32) -> LayeredSourceGraph {
    LayeredSourceGraph {
        source_sha256: SHA,
        declared_copper_layers: layers,
        nodes: vec![
            node("trace:start", first, "NET_A", false),
            node(&format!("{VIA}:L{first}"), first, "NET_A", true),
            node(&format!("{VIA}:L{last}"), last, "NET_A", true),
            node("trace:end", last, "NET_A", false),
        ],
        geometry_contacts: vec![local(0, 1, first), local(2, 3, last)],
        unverified_barrels: vec![UnverifiedBarrel {
            via_key: VIA.into(),
            source_line: 7,
            declared_first_layer: first,
            declared_last_layer: last,
            conductive_pad_nodes: vec![1, 2],
            process_qualified: false,
        }],
        verified_electrical_edges: vec![],
    }
}

fn path() -> RequiredNetPath {
    RequiredNetPath {
        net: "NET_A".into(),
        endpoint_source_keys: vec!["trace:start".into(), "trace:end".into()],
    }
}

fn proof(first: u32, last: u32) -> BarrelProcessWitness {
    BarrelProcessWitness {
        sha256: SHA,
        via_key: VIA.into(),
        first_layer: first,
        last_layer: last,
        documented_process_reference: "fixture:process-evidence-reviewed".into(),
        qualified_transition_reference: "fixture:continuous-barrel-in-design".into(),
    }
}

fn partial() -> SourceCoverage {
    SourceCoverage::Partial
}

#[test]
fn universal_layers_have_two_local_components_without_plating_evidence() {
    for layers in [2, 4, 6, 8, 64] {
        for (first, last) in [(1, layers), (2, 3)] {
            if last > layers {
                continue;
            }
            let board = graph(layers, first, last);
            let report = qualify(&board, SHA, &partial(), &[], &[path()]).unwrap();
            assert_eq!(report.components, vec![vec![0, 1], vec![2, 3]]);
            assert_eq!(report.component_by_node, vec![0, 0, 1, 1]);
            assert_eq!(report.edges.len(), 2);
            assert!(report
                .edges
                .iter()
                .all(|e| e.kind == QualifiedEdgeKind::SourceLocalPositiveArea));
            assert_eq!(report.open_witnesses.len(), 1);
            assert_eq!(
                report.open_witnesses[0].disposition,
                OpenDisposition::CandidateOnly
            );
            assert_eq!(report.unverified_barrel_keys, vec![VIA]);
            assert_eq!(report.bridge_edges, vec![(0, 1), (2, 3)]);
            assert!(report.articulation_nodes.is_empty());
            assert!(!report.source_scope_qualified);
        }
    }
}

#[test]
fn explicit_process_proof_joins_only_actual_pad_layers_and_has_cut_witnesses() {
    for (layers, first, last) in [
        (2, 1, 2),
        (4, 1, 4),
        (6, 2, 3),
        (6, 2, 4),
        (8, 3, 7),
        (64, 17, 41),
    ] {
        let board = graph(layers, first, last);
        let report = qualify(&board, SHA, &partial(), &[proof(first, last)], &[path()]).unwrap();
        assert_eq!(report.components, vec![vec![0, 1, 2, 3]]);
        assert_eq!(report.edges.len(), 3);
        assert_eq!(
            report
                .edges
                .iter()
                .filter(|e| e.kind == QualifiedEdgeKind::EvidenceQualifiedBarrel)
                .count(),
            1
        );
        assert_eq!(report.bridge_edges, vec![(0, 1), (1, 2), (2, 3)]);
        assert_eq!(report.articulation_nodes, vec![1, 2]);
        assert!(report.open_witnesses.is_empty());
        assert!(report.unverified_barrel_keys.is_empty());
        assert!(!report.source_scope_qualified); // Still not global/fabrication proof.
    }
}

#[test]
fn source_scope_disconnected_endpoints_are_open_only_when_independently_qualified() {
    let board = graph(6, 2, 4);
    let c = SourceCoverage::QualifiedSourceScope {
        sha256: SHA,
        evidence_reference: "independent-source-coverage-attestation".into(),
    };
    let report = qualify(&board, SHA, &c, &[], &[path()]).unwrap();
    assert_eq!(report.open_witnesses.len(), 1);
    // Without process evidence for this via, a positive Open is still unsafe.
    assert_eq!(
        report.open_witnesses[0].disposition,
        OpenDisposition::CandidateOnly
    );
    assert!(report.source_scope_qualified);

    // Even a missing barrel inventory cannot turn a partial pad-contact
    // search into proof that no alternate pad-to-pad path exists.
    let mut isolated = board;
    isolated.unverified_barrels.clear();
    let report = qualify(&isolated, SHA, &c, &[], &[path()]).unwrap();
    assert_eq!(
        report.open_witnesses[0].disposition,
        OpenDisposition::CandidateOnly
    );
    assert!(report
        .diagnostics
        .contains(&"PAD_CONTACT_COVERAGE_PARTIAL".to_owned()));

    // The separately attested complete source scope contains only two
    // disconnected stroke nodes, with no pad/area/process ambiguities.
    isolated.nodes = vec![
        node("trace:start", 2, "NET_A", false),
        node("trace:end", 4, "NET_A", false),
    ];
    isolated.geometry_contacts.clear();
    let report = qualify(&isolated, SHA, &c, &[], &[path()]).unwrap();
    assert_eq!(
        report.open_witnesses[0].disposition,
        OpenDisposition::QualifiedSourceScope
    );
}

#[test]
fn cross_net_conflict_requires_a_real_source_contact_or_proved_transition() {
    let mut board = graph(6, 2, 4);
    board.nodes[2].declared_net = Some("NET_B".into());
    board.nodes[3].declared_net = Some("NET_B".into());
    assert!(qualify(&board, SHA, &partial(), &[], &[])
        .unwrap()
        .net_conflicts
        .is_empty());
    let report = qualify(&board, SHA, &partial(), &[proof(2, 4)], &[]).unwrap();
    assert_eq!(report.net_conflicts.len(), 1);
    let conflict = &report.net_conflicts[0];
    assert_eq!(conflict.first_net, "NET_A");
    assert_eq!(conflict.second_net, "NET_B");
    assert_eq!(conflict.path_nodes, vec![0, 1, 2]);

    // Cross-net but separated geometry is no conflict.
    board.geometry_contacts.clear();
    let report = qualify(&board, SHA, &partial(), &[], &[]).unwrap();
    assert!(report.net_conflicts.is_empty());
}

#[test]
fn three_node_redundant_loop_has_no_bridges_or_articulation() {
    let mut board = graph(4, 1, 4);
    board.unverified_barrels.clear();
    board.nodes = vec![
        node("a", 1, "NET_A", false),
        node("b", 1, "NET_A", false),
        node("c", 1, "NET_A", false),
    ];
    board.geometry_contacts = vec![
        GeometryContact {
            first_node: 0,
            second_node: 1,
            layer: 1,
            kind: ContactKind::TraceTrace,
            declared_net_check: DeclaredNetCheck::Same,
        },
        GeometryContact {
            first_node: 1,
            second_node: 2,
            layer: 1,
            kind: ContactKind::TraceTrace,
            declared_net_check: DeclaredNetCheck::Same,
        },
        GeometryContact {
            first_node: 2,
            second_node: 0,
            layer: 1,
            kind: ContactKind::TraceTrace,
            declared_net_check: DeclaredNetCheck::Same,
        },
    ];
    let r = qualify(&board, SHA, &partial(), &[], &[]).unwrap();
    assert_eq!(r.components, vec![vec![0, 1, 2]]);
    assert_eq!(r.edges.len(), 3);
    assert!(r.bridge_edges.is_empty());
    assert!(r.articulation_nodes.is_empty());
}

#[test]
fn invalid_span_missing_pad_stale_sha_and_forged_preverification_fail_closed() {
    let board = graph(6, 2, 4);
    assert_eq!(
        qualify(&board, [0; 32], &partial(), &[], &[]).unwrap_err(),
        "D05603_STALE_SOURCE_SHA"
    );
    let mut bad_proof = proof(2, 4);
    bad_proof.last_layer = 5;
    assert_eq!(
        qualify(&board, SHA, &partial(), &[bad_proof], &[]).unwrap_err(),
        "D05603_PROCESS_SPAN_MISMATCH"
    );
    let mut missing = board.clone();
    missing.unverified_barrels[0].conductive_pad_nodes.pop();
    assert_eq!(
        qualify(&missing, SHA, &partial(), &[proof(2, 4)], &[]).unwrap_err(),
        "D05603_MISSING_BARREL_ENDPOINT_LANDING"
    );
    let mut promoted = board.clone();
    promoted.verified_electrical_edges.push((1, 2));
    assert_eq!(
        qualify(&promoted, SHA, &partial(), &[], &[]).unwrap_err(),
        "D05603_UNTRUSTED_PREVERIFIED_EDGES"
    );
    let mut forged = board.clone();
    forged.unverified_barrels[0].process_qualified = true;
    assert_eq!(
        qualify(&forged, SHA, &partial(), &[], &[]).unwrap_err(),
        "D05603_UNTRUSTED_BARREL_FLAG"
    );
    let mut inconsistent = board;
    inconsistent.geometry_contacts[0].declared_net_check = DeclaredNetCheck::Different;
    assert_eq!(
        qualify(&inconsistent, SHA, &partial(), &[], &[]).unwrap_err(),
        "D05603_LOCAL_NET_EVIDENCE_CONFLICT"
    );
}

#[test]
fn duplicate_and_cross_layer_source_contacts_fail_closed() {
    let mut board = graph(6, 2, 4);
    board.nodes[3].source_key = "trace:start".into();
    assert_eq!(
        qualify(&board, SHA, &partial(), &[], &[]).unwrap_err(),
        "D05603_DUPLICATE_SOURCE_KEY"
    );
    board.nodes[3].source_key = "trace:end".into();
    board.geometry_contacts[0].layer = 4;
    assert_eq!(
        qualify(&board, SHA, &partial(), &[], &[]).unwrap_err(),
        "D05603_INVALID_LOCAL_CONTACT"
    );
}

#[test]
fn iterative_cut_detection_is_bounded_for_long_trace_chain() {
    let mut board = graph(4, 1, 4);
    board.unverified_barrels.clear();
    board.nodes = (0..12_000)
        .map(|i| node(&format!("stroke:{i}"), 1, "NET_A", false))
        .collect();
    board.geometry_contacts = (0..11_999)
        .map(|i| GeometryContact {
            first_node: i,
            second_node: i + 1,
            layer: 1,
            kind: ContactKind::TraceTrace,
            declared_net_check: DeclaredNetCheck::Same,
        })
        .collect();
    let report = qualify(&board, SHA, &partial(), &[], &[]).unwrap();
    assert_eq!(report.components.len(), 1);
    assert_eq!(report.bridge_edges.len(), 11_999);
    assert_eq!(report.articulation_nodes.len(), 11_998);
}

#[test]
fn mutated_coverage_and_missing_required_source_identity_are_rejected() {
    let board = graph(6, 2, 4);
    let bad = SourceCoverage::QualifiedSourceScope {
        sha256: [0; 32],
        evidence_reference: "external".into(),
    };
    assert_eq!(
        qualify(&board, SHA, &bad, &[], &[path()]).unwrap_err(),
        "D05603_INVALID_COVERAGE_EVIDENCE"
    );
    let path = RequiredNetPath {
        net: "NET_A".into(),
        endpoint_source_keys: vec!["trace:start".into(), "no-such-source".into()],
    };
    assert_eq!(
        qualify(&board, SHA, &partial(), &[], &[path]).unwrap_err(),
        "D05603_MISSING_REQUIRED_ENDPOINT"
    );
}

#[test]
fn contact_kind_and_unbounded_conflicts_are_rejected() {
    let mut wrong = graph(6, 2, 4);
    wrong.geometry_contacts[0].kind = ContactKind::TraceTrace;
    assert_eq!(
        qualify(&wrong, SHA, &partial(), &[], &[]).unwrap_err(),
        "D05603_CONTACT_KIND_MISMATCH"
    );

    // An adversarial multi-net chain must not allocate quadratic path witnesses.
    let mut many = graph(4, 1, 4);
    many.unverified_barrels.clear();
    many.nodes = (0..500)
        .map(|i| node(&format!("net:{i}"), 1, &format!("NET_{i}"), false))
        .collect();
    many.geometry_contacts = (0..499)
        .map(|i| GeometryContact {
            first_node: i,
            second_node: i + 1,
            layer: 1,
            kind: ContactKind::TraceTrace,
            declared_net_check: DeclaredNetCheck::Different,
        })
        .collect();
    assert_eq!(
        qualify(&many, SHA, &partial(), &[], &[]).unwrap_err(),
        "D05603_CONFLICT_WITNESS_BOUNDS"
    );
}
