//! D_056_02: source-bound, layer-specific copper-contact graph foundation.
//!
//! This is an opt-in PADS V9.4 local geometry inventory, NOT a manufactured
//! electrical connectivity graph, open/short result, or SI solver input.
//! In particular, a declared drill span and `plated: true` in the preview
//! adapter do not certify a conducting barrel. D_056_03 owns that gate.
use crate::part_preview;
use crate::route_preview;
use crate::trace_attachment::{self, DeclaredNetRelation, Target};
use crate::trace_contact::{self, NetRelation};
use crate::via_shapes::ViaLayerShape;
use std::collections::{BTreeMap, BTreeSet};

const MAX_NODES: usize = 50_000;
const MAX_CONTACTS: usize = 100_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CopperFootprint {
    Stroke {
        width_ticks: i64,
    },
    ViaPad {
        centre: [i64; 2],
        outer_ticks: i64,
        drill_ticks: i64,
        shape: ViaLayerShape,
    },
    SmdPad {
        centre: [i64; 2],
        width_ticks: i64,
        height_ticks: i64,
        rectangular: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopperNode {
    /// Stable within a byte-identical source import; not a fabricated native ID.
    pub source_key: String,
    /// None when the restricted native adapter cannot identify the record line.
    pub source_line: Option<usize>,
    pub physical_copper_layer: u32,
    pub declared_net: Option<String>,
    pub footprint: CopperFootprint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContactKind {
    TraceTrace,
    TraceViaPad,
    TraceSmdPad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeclaredNetCheck {
    Same,
    Different,
    Unknown,
}

/// Only a geometry witness. Never serialize this as a verified electrical edge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeometryContact {
    pub first_node: usize,
    pub second_node: usize,
    pub layer: u32,
    pub kind: ContactKind,
    pub declared_net_check: DeclaredNetCheck,
}

/// A span inventory, *not* an edge between the supplied landing pads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnverifiedBarrel {
    pub via_key: String,
    pub source_line: usize,
    pub declared_first_layer: u32,
    pub declared_last_layer: u32,
    pub conductive_pad_nodes: Vec<usize>,
    /// Unknown manufacturing / plating continuity, irrespective of parser's bool.
    pub process_qualified: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayeredSourceGraph {
    /// Exact source bytes; use this digest to reject stale/reimported graph state.
    pub source_sha256: [u8; 32],
    pub declared_copper_layers: u32,
    pub nodes: Vec<CopperNode>,
    pub geometry_contacts: Vec<GeometryContact>,
    pub unverified_barrels: Vec<UnverifiedBarrel>,
    /// Deliberately absent until a separate process and full-graph evidence gate.
    /// No consumer may infer verified edges from geometry_contacts.
    pub verified_electrical_edges: Vec<(usize, usize)>,
}

fn add_node(nodes: &mut Vec<CopperNode>, node: CopperNode) -> Result<usize, &'static str> {
    if nodes.len() >= MAX_NODES {
        return Err("D05602_GRAPH_NODE_LIMIT");
    }
    let index = nodes.len();
    nodes.push(node);
    Ok(index)
}

fn net_check(a: &CopperNode, b: &CopperNode) -> DeclaredNetCheck {
    match (&a.declared_net, &b.declared_net) {
        (Some(a), Some(b)) if a == b => DeclaredNetCheck::Same,
        (Some(_), Some(_)) => DeclaredNetCheck::Different,
        _ => DeclaredNetCheck::Unknown,
    }
}

fn contact(
    nodes: &[CopperNode],
    first: usize,
    second: usize,
    kind: ContactKind,
) -> Result<GeometryContact, &'static str> {
    if first == second {
        return Err("D05602_SELF_CONTACT");
    }
    let a = nodes.get(first).ok_or("D05602_MISSING_CONTACT_NODE")?;
    let b = nodes.get(second).ok_or("D05602_MISSING_CONTACT_NODE")?;
    if a.physical_copper_layer != b.physical_copper_layer {
        return Err("D05602_CROSS_LAYER_LOCAL_CONTACT");
    }
    let (first_node, second_node) = if first < second {
        (first, second)
    } else {
        (second, first)
    };
    Ok(GeometryContact {
        first_node,
        second_node,
        layer: a.physical_copper_layer,
        kind,
        declared_net_check: net_check(a, b),
    })
}

/// Build a deterministic source-owned *local* geometry contact graph.
///
/// Only supported PADS V9.4 MILS straight/arc trace, via R/S/A conductive
/// pad and restricted SMD pad inventory participates. Antipads, bare holes,
/// and missing-layer pad entries produce no copper nodes. Unsupported records,
/// numerical boundary ambiguity and contradictory spans fail closed.
/// The result is expressly PARTIAL: no polygon/flood/thermal/whole-board proof.
pub fn from_pads_v94_source(bytes: &[u8]) -> Result<LayeredSourceGraph, &'static str> {
    let art = route_preview::parse_generalized(bytes)?;
    if art.skipped != 0 {
        return Err("D05602_INCOMPLETE_ROUTE_INVENTORY");
    }
    let strokes = trace_contact::from_artwork(&art)?;
    let pad_import = part_preview::parse(bytes, art.max_layer);
    if pad_import.skipped != 0 {
        return Err("D05602_INCOMPLETE_SMD_INVENTORY");
    }
    let pin_nets = part_preview::declared_route_pin_nets(bytes)?;
    // These are the existing bounded broadphase + shared narrow-phase
    // operations. Never redo geometry by coincident centre/Net names.
    let trace_hits = trace_contact::contact_candidates(&art)?;
    let attachment_hits =
        trace_attachment::contact_candidates(&strokes, &art.vias, &pad_import.pads, &pin_nets)?;

    let mut nodes = Vec::new();
    let mut trace_nodes = Vec::with_capacity(strokes.len());
    for (index, stroke) in strokes.iter().enumerate() {
        trace_nodes.push(add_node(
            &mut nodes,
            CopperNode {
                source_key: format!("stroke:{index}:line:{}", stroke.source_line),
                source_line: Some(stroke.source_line),
                physical_copper_layer: stroke.layer,
                declared_net: Some(stroke.net.clone()),
                footprint: CopperFootprint::Stroke {
                    width_ticks: stroke.width_ticks,
                },
            },
        )?);
    }

    let mut via_pad_nodes = BTreeMap::new();
    let mut unverified_barrels = Vec::with_capacity(art.vias.len());
    for (index, via) in art.vias.iter().enumerate() {
        let mut declared_pads = Vec::new();
        let mut footprints = Vec::new();
        if let Some(profile) = &via.shaped {
            profile.validate()?;
            if via.first_layer != profile.first_layer
                || via.last_layer != profile.last_layer
                || via.drill_ticks != profile.drill_ticks
            {
                return Err("D05602_VIA_SPAN_CONFLICT");
            }
            for pad in &profile.pads {
                if pad.shape.conductive() {
                    footprints.push((pad.layer, pad.size_ticks, pad.shape));
                }
            }
        } else if let Some(topology) = &via.topology {
            topology.validate()?;
            if via.first_layer != topology.first_layer
                || via.last_layer != topology.last_layer
                || via.drill_ticks != topology.drill_ticks
            {
                return Err("D05602_VIA_SPAN_CONFLICT");
            }
            for pad in &topology.pads {
                footprints.push((pad.layer, pad.diameter_ticks, ViaLayerShape::Round));
            }
        } else {
            // The generalized mode MUST not silently fall back to uniform
            // Through-via pad assumptions from the legacy renderer.
            return Err("D05602_MISSING_SOURCE_LAYER_PADS");
        }
        footprints.sort_by_key(|p| p.0);
        for (layer, outer_ticks, shape) in footprints {
            let node = add_node(
                &mut nodes,
                CopperNode {
                    source_key: format!(
                        "via:{index}:{}:line:{}:L{layer}",
                        via.name, via.source_line
                    ),
                    source_line: Some(via.source_line),
                    physical_copper_layer: layer,
                    declared_net: Some(via.net.clone()),
                    footprint: CopperFootprint::ViaPad {
                        centre: via.point,
                        outer_ticks,
                        drill_ticks: via.drill_ticks,
                        shape,
                    },
                },
            )?;
            if via_pad_nodes.insert((index, layer), node).is_some() {
                return Err("D05602_DUPLICATE_VIA_LAYER");
            }
            declared_pads.push(node);
        }
        unverified_barrels.push(UnverifiedBarrel {
            via_key: format!("via:{index}:{}:line:{}", via.name, via.source_line),
            source_line: via.source_line,
            declared_first_layer: via.first_layer,
            declared_last_layer: via.last_layer,
            conductive_pad_nodes: declared_pads,
            process_qualified: false,
        });
    }

    let mut smd_nodes = Vec::with_capacity(pad_import.pads.len());
    for (index, pad) in pad_import.pads.iter().enumerate() {
        let key = format!("{}.{}", pad.reference, pad.pin);
        smd_nodes.push(add_node(
            &mut nodes,
            CopperNode {
                source_key: format!("smd:{index}:{key}:L{}", pad.layer),
                source_line: None,
                physical_copper_layer: pad.layer,
                declared_net: pin_nets.get(&key).cloned(),
                footprint: CopperFootprint::SmdPad {
                    centre: pad.point,
                    width_ticks: pad.width_ticks,
                    height_ticks: pad.height_ticks,
                    rectangular: pad.square || pad.rectangular,
                },
            },
        )?);
    }

    let mut geometry_contacts = Vec::new();
    let mut seen = BTreeSet::new();
    for hit in trace_hits {
        let edge = contact(
            &nodes,
            trace_nodes[hit.first],
            trace_nodes[hit.second],
            ContactKind::TraceTrace,
        )?;
        // Fail closed on mismatched candidate metadata.
        let declared = if hit.net_relation == NetRelation::SameDeclaredNet {
            DeclaredNetCheck::Same
        } else {
            DeclaredNetCheck::Different
        };
        if edge.declared_net_check != declared {
            return Err("D05602_TRACE_NET_EVIDENCE_CONFLICT");
        }
        if seen.insert((edge.first_node, edge.second_node, edge.kind)) {
            geometry_contacts.push(edge);
        }
    }
    for hit in attachment_hits {
        let first = *trace_nodes
            .get(hit.trace_index)
            .ok_or("D05602_MISSING_TRACE")?;
        let (second, kind) = match hit.target {
            Target::Via(index) => (
                *via_pad_nodes
                    .get(&(index, nodes[first].physical_copper_layer))
                    .ok_or("D05602_MISSING_CONDUCTIVE_LANDING")?,
                ContactKind::TraceViaPad,
            ),
            Target::SmdPad(index) => (
                *smd_nodes.get(index).ok_or("D05602_MISSING_SMD_PAD")?,
                ContactKind::TraceSmdPad,
            ),
        };
        let edge = contact(&nodes, first, second, kind)?;
        let declared = match hit.net_relation {
            DeclaredNetRelation::Same => DeclaredNetCheck::Same,
            DeclaredNetRelation::Different => DeclaredNetCheck::Different,
            DeclaredNetRelation::PadNetUnverified => DeclaredNetCheck::Unknown,
        };
        if edge.declared_net_check != declared {
            return Err("D05602_ATTACHMENT_NET_EVIDENCE_CONFLICT");
        }
        if seen.insert((edge.first_node, edge.second_node, edge.kind)) {
            geometry_contacts.push(edge);
        }
    }
    if geometry_contacts.len() > MAX_CONTACTS {
        return Err("D05602_GRAPH_CONTACT_LIMIT");
    }
    geometry_contacts.sort_by_key(|e| (e.first_node, e.second_node, e.kind));
    Ok(LayeredSourceGraph {
        source_sha256: crate::digest::sha256(bytes),
        declared_copper_layers: art.max_layer,
        nodes,
        geometry_contacts,
        unverified_barrels,
        verified_electrical_edges: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Independent count oracle; no expected edges computed using production
    // geometry routines. Public synthetic PADS V9.4 source only.
    fn source(layers: u32, first: u32, last: u32) -> Vec<u8> {
        format!(
            "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n\
             *PCB*\nMAXLAYER {layers}\n*VIA*\n\
             V_TEST 15 2 {first} {last}\n-2 35 R\n0 40 R\n\
             *ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n\
             100 200 {first} 10 0\n300 200 {last} 10 0 V_TEST\n\
             500 200 {last} 10 0\n*END*\n"
        )
        .into_bytes()
    }

    #[test]
    fn same_algorithm_all_layer_counts_and_partial_spans() {
        for layers in [2, 4, 6, 8, 64] {
            for (first, last) in [(1, layers), (2, 3)] {
                if last > layers {
                    continue;
                }
                let input = source(layers, first, last);
                let graph = from_pads_v94_source(&input).unwrap();
                assert_eq!(graph.declared_copper_layers, layers);
                assert_eq!(graph.unverified_barrels.len(), 1);
                assert_eq!(graph.unverified_barrels[0].conductive_pad_nodes.len(), 2);
                assert_eq!(
                    graph
                        .geometry_contacts
                        .iter()
                        .filter(|e| e.kind == ContactKind::TraceViaPad)
                        .count(),
                    2
                );
                assert!(graph
                    .geometry_contacts
                    .iter()
                    .all(
                        |e| graph.nodes[e.first_node].physical_copper_layer == e.layer
                            && graph.nodes[e.second_node].physical_copper_layer == e.layer
                    ));
                assert!(graph.verified_electrical_edges.is_empty());
                assert!(!graph.unverified_barrels[0].process_qualified);
                assert!(!graph.nodes.iter().any(|n| n.source_line.is_none()
                    && matches!(n.footprint, CopperFootprint::ViaPad { .. })));
            }
        }
    }

    #[test]
    fn middle_layer_without_pad_is_not_a_contact_node() {
        let input = source(6, 2, 4);
        let graph = from_pads_v94_source(&input).unwrap();
        assert!(!graph.nodes.iter().any(|n| n.physical_copper_layer == 3
            && matches!(n.footprint, CopperFootprint::ViaPad { .. })));
    }

    #[test]
    fn bad_span_and_changed_source_are_not_reused() {
        let input = source(6, 2, 4);
        let valid = from_pads_v94_source(&input).unwrap();
        let wrong = String::from_utf8(input.clone())
            .unwrap()
            .replace("V_TEST 15 2 2 4", "V_TEST 15 2 2 3");
        assert!(from_pads_v94_source(wrong.as_bytes()).is_err());
        let mut changed = input;
        changed.extend_from_slice(b"\n");
        let changed_graph = from_pads_v94_source(&changed).unwrap();
        assert_ne!(valid.source_sha256, changed_graph.source_sha256);
    }

    #[test]
    fn source_graph_never_promotes_declared_span_to_electrical_edges() {
        let graph = from_pads_v94_source(&source(6, 2, 4)).unwrap();
        assert!(graph.verified_electrical_edges.is_empty());
        for barrel in &graph.unverified_barrels {
            assert!(!barrel.process_qualified);
        }
        // Candidate classification does not silently constitute a short.
        assert!(graph
            .geometry_contacts
            .iter()
            .all(|e| e.declared_net_check == DeclaredNetCheck::Same));
    }

    #[test]
    fn native_square_and_antipad_do_not_create_false_copper_nodes() {
        let raw = String::from_utf8(source(6, 2, 4)).unwrap();
        let raw = raw.replace(
            "V_TEST 15 2 2 4\n-2 35 R\n0 40 R",
            "V_TEST 15 3 2 4\n-2 35 S\n-1 60 RA\n0 40 R",
        );
        assert!(raw.contains("-1 60 RA"));
        let graph = from_pads_v94_source(raw.as_bytes()).unwrap();
        let via_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| matches!(&n.footprint, CopperFootprint::ViaPad { .. }))
            .collect();
        assert_eq!(via_nodes.len(), 2); // L3 is a plane void, not copper.
        assert_eq!(via_nodes[0].physical_copper_layer, 2);
        assert_eq!(via_nodes[1].physical_copper_layer, 4);
        assert!(matches!(
            &via_nodes[0].footprint,
            CopperFootprint::ViaPad {
                shape: ViaLayerShape::Square,
                ..
            }
        ));
        assert_eq!(
            graph
                .geometry_contacts
                .iter()
                .filter(|e| e.kind == ContactKind::TraceViaPad)
                .count(),
            2
        );
        assert!(graph.verified_electrical_edges.is_empty());
    }

    #[test]
    fn off_centre_contact_and_cross_net_are_geometry_only_not_a_short() {
        let raw = String::from_utf8(source(6, 2, 4)).unwrap();
        let raw = raw.replace(
            "*END*\n",
            "*SIGNAL* NET_B\nU3.1 U4.2\n100 215 2 10 0\n500 215 2 10 0\n\
             U5.1 U6.2\n100 230 2 10 0\n500 230 2 10 0\n*END*\n",
        );
        let graph = from_pads_v94_source(raw.as_bytes()).unwrap();
        let via_contacts: Vec<_> = graph
            .geometry_contacts
            .iter()
            .filter(|e| e.kind == ContactKind::TraceViaPad)
            .collect();
        // Two centred route landings plus one off-centre positive-area landing.
        // The y=230 trace is outside the via's (radius + trace half-width).
        assert_eq!(via_contacts.len(), 3);
        assert_eq!(
            via_contacts
                .iter()
                .filter(|e| e.declared_net_check == DeclaredNetCheck::Different)
                .count(),
            1
        );
        assert!(graph.verified_electrical_edges.is_empty());
    }
}
