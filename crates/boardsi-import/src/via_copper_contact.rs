//! B_006/B_007 bounded source-exact plated via-to-COPCLS with one verified COPCUT.
//! Geometry-only local graph proof; NEVER physical return path or P009.
use crate::{copper_preview, route_preview};
use boardsi_model::{
    strict_simple_area_contains, BoardModel, ConnectionEvidence, ConnectivityEdge,
    ConnectivityNode, ContactSite, GeometryProvenance, ObjectId, ObjectKind, ObjectMeta, PadRole,
    Point, SourceId, ViaKind,
};
use std::collections::{BTreeMap, BTreeSet};

const MAX_NEW_EDGES: usize = 10_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViaAreaAudit {
    pub local_contacts: usize,
    pub new_barrel_edges: usize,
    pub unsupported_candidates: usize,
}

fn meta(source: &SourceId, kind: ObjectKind, key: &str) -> ObjectMeta {
    ObjectMeta {
        id: ObjectId::new(source, kind, key, ""),
        provenance: GeometryProvenance {
            source_object_type:
                "B_006 PADS source-exact local Via/COPCLS centre witness; no physical SI".into(),
            history: vec![],
        },
    }
}

fn append_edge(
    source: &SourceId,
    edges: &mut Vec<ConnectivityEdge>,
    kind: &str,
    a: &ObjectId,
    b: &ObjectId,
    evidence: ConnectionEvidence,
) -> Result<(), &'static str> {
    if edges.len() >= MAX_NEW_EDGES {
        return Err("B006_EDGE_LIMIT");
    }
    let (a, b) = if a < b { (a, b) } else { (b, a) };
    edges.push(ConnectivityEdge {
        meta: meta(
            source,
            ObjectKind::Edge,
            &format!("b006-{kind}:{}--{}", a.source_object, b.source_object),
        ),
        a: a.clone(),
        b: b.clone(),
        evidence,
    });
    Ok(())
}

/// Must be invoked after the supported route/copper imports and optional B_004 contacts.
/// Caller must discard the candidate board if this function returns Err.
pub fn augment(
    board: &mut BoardModel,
    original_bytes: &[u8],
) -> Result<ViaAreaAudit, &'static str> {
    if board.validate().has_errors() {
        return Err("B006_INVALID_CANONICAL");
    }
    if board.sources.len() != 1 || board.frames.len() != 1 {
        return Err("B006_SOURCE_INVENTORY");
    }
    if board
        .connectivity
        .edges
        .iter()
        .any(|e| e.meta.id.source_object.starts_with("b006-"))
    {
        return Err("B006_ALREADY_APPLIED");
    }
    // Reparse exact source bytes, not a detached caller-supplied Via/Area list.
    let route = route_preview::parse(original_bytes).map_err(|_| "B006_SOURCE_ROUTE")?;
    let original =
        route_preview::into_canonical(original_bytes, &route).map_err(|_| "B006_SOURCE_MODEL")?;
    let source_areas =
        copper_preview::parse(original_bytes, route.max_layer).map_err(|_| "B006_SOURCE_COPPER")?;
    let mut original = original;
    copper_preview::augment(&mut original, &source_areas).map_err(|_| "B006_SOURCE_COPPER")?;
    if board.sources[0].id != original.sources[0].id {
        return Err("B006_SOURCE_SHA_MISMATCH");
    }
    if board.nets != original.nets
        || board.traces != original.traces
        || board.arcs != original.arcs
        || board.vias.len() != original.vias.len()
        || board.areas.len() != original.areas.len()
        || board.stackup.layers.len() != original.stackup.layers.len()
        || !board
            .stackup
            .layers
            .iter()
            .zip(&original.stackup.layers)
            .all(|(a, b)| a.meta.id == b.meta.id && a.order == b.order && a.kind == b.kind)
    {
        return Err("B006_SOURCE_INVENTORY_MISMATCH");
    }
    for (actual, expected) in board.areas.iter().zip(&original.areas) {
        if actual.meta.id != expected.meta.id
            || actual.layer != expected.layer
            || actual.outer != expected.outer
            || actual.cutouts != expected.cutouts
        {
            return Err("B006_AREA_GEOMETRY_MISMATCH");
        }
    }
    for expected in &original.vias {
        let actual = board
            .vias
            .iter()
            .find(|v| v.meta.id == expected.meta.id)
            .ok_or("B006_VIA_SOURCE_MISMATCH")?;
        if actual.position != expected.position
            || actual.padstack != expected.padstack
            || actual.drill != expected.drill
            || actual.span != expected.span
            || actual.kind != expected.kind
            || actual.net != expected.net
            || actual.connected_layers != expected.connected_layers
        {
            return Err("B006_VIA_SOURCE_MISMATCH");
        }
        let a_stack = board
            .padstacks
            .iter()
            .find(|s| s.meta.id == actual.padstack)
            .ok_or("B006_PADSTACK_MISSING")?;
        let o_stack = original
            .padstacks
            .iter()
            .find(|s| s.meta.id == expected.padstack)
            .ok_or("B006_PADSTACK_SOURCE_MISSING")?;
        let a_drill = board
            .drills
            .iter()
            .find(|d| d.meta.id == actual.drill)
            .ok_or("B006_DRILL_MISSING")?;
        let o_drill = original
            .drills
            .iter()
            .find(|d| d.meta.id == expected.drill)
            .ok_or("B006_DRILL_SOURCE_MISSING")?;
        if a_stack != o_stack || a_drill != o_drill {
            return Err("B006_VIA_DEFINITION_MISMATCH");
        }
    }
    let source = board.sources[0].id.clone();
    let frame = board.frames[0].meta.id.clone();
    let mut nodes: BTreeMap<ObjectId, ConnectivityNode> = board
        .connectivity
        .nodes
        .iter()
        .map(|n| (n.meta.id.clone(), n.clone()))
        .collect();
    let mut edges = Vec::new();
    let mut audit = ViaAreaAudit::default();
    let mut visited = BTreeSet::new();
    let mut area_updates = BTreeSet::new();
    // Via lookup is Layer + source-tick X indexed; only bounding-box candidates
    // on the same layer undergo the exact strict polygon predicate.
    let mut index: BTreeMap<(ObjectId, i64), Vec<usize>> = BTreeMap::new();
    for (i, via) in board.vias.iter().enumerate() {
        for pad in board
            .padstacks
            .iter()
            .filter(|s| s.meta.id == via.padstack)
            .flat_map(|s| &s.entries)
            .filter(|e| e.role == PadRole::Conductive)
        {
            index
                .entry((pad.layer.clone(), via.position.ticks[0]))
                .or_default()
                .push(i);
        }
    }
    for (area_index, area) in source_areas.iter().enumerate() {
        let Some(net_label) = &area.net_label else {
            audit.unsupported_candidates += 1;
            continue;
        };
        let Some(net) = board
            .nets
            .iter()
            .find(|n| n.name == *net_label)
            .map(|n| n.meta.id.clone())
        else {
            audit.unsupported_candidates += 1;
            continue;
        };
        let copper = &board.areas[area_index];
        let copper_source = &original.areas[area_index];
        // Exactly one source Area on the current Layer: no ambiguous overlapping,
        // split or separate island selection can be promoted into a contact.
        if source_areas
            .iter()
            .filter(|a| a.layer == area.layer)
            .count()
            != 1
            || area.cutouts.len() > 1
            || copper.cutouts.len() != area.cutouts.len()
            || area.cutouts.iter().any(|h| !matches!(h, copper_preview::CopperGeometry::Polygon(_)))
            || !matches!(&area.geometry, copper_preview::CopperGeometry::Polygon(_))
            || copper.net.as_ref().is_some_and(|current| *current != net)
            || copper_source.net.is_some()
        {
            audit.unsupported_candidates += 1;
            continue;
        }
        let copper_preview::CopperGeometry::Polygon(ring) = &area.geometry else {
            continue;
        };
        if ring.len() < 4 || ring.len() > 65 || ring.first() != ring.last() {
            audit.unsupported_candidates += 1;
            continue;
        }
        let xmin = ring.iter().map(|p| p[0]).min().ok_or("B006_AREA_RING")?;
        let xmax = ring.iter().map(|p| p[0]).max().ok_or("B006_AREA_RING")?;
        let layer = copper.layer.clone();
        // Keep one anchor for every Area ID, including already accepted B_004
        // interior nodes; never link distinct Area IDs just by Net labels.
        let mut anchor = nodes
            .values()
            .find(|n| {
                n.conductor == copper.meta.id
                    && n.layer == layer
                    && matches!(n.site, ContactSite::AreaInterior(_))
            })
            .map(|n| n.meta.id.clone());
        for ((_, x), indices) in index.range((layer.clone(), xmin)..=(layer.clone(), xmax)) {
            let _ = x;
            for &via_index in indices {
                let via = board.vias[via_index].clone();
                if !visited.insert((area_index, via_index)) {
                    continue;
                }
                if via.kind != ViaKind::Through || via.net.as_ref() != Some(&net) {
                    audit.unsupported_candidates += 1;
                    continue;
                }
                let drill = board
                    .drills
                    .iter()
                    .find(|d| d.meta.id == via.drill)
                    .ok_or("B006_DRILL_MISSING")?;
                let stack = board
                    .padstacks
                    .iter()
                    .find(|s| s.meta.id == via.padstack)
                    .ok_or("B006_PADSTACK_MISSING")?;
                if !drill.plated
                    || drill.span != via.span
                    || stack.entries.iter().any(|e| e.role != PadRole::Conductive)
                    || !stack
                        .entries
                        .iter()
                        .any(|e| e.layer == layer && e.role == PadRole::Conductive)
                {
                    audit.unsupported_candidates += 1;
                    continue;
                }
                let point = Point {
                    frame: frame.clone(),
                    ticks: via.position.ticks,
                };
                if strict_simple_area_contains(copper, &point).ok() != Some(true) {
                    audit.unsupported_candidates += 1;
                    continue;
                }
                let via_key = format!(
                    "{:?}/{}:{}/{}:centre",
                    via.meta.id.kind,
                    via.meta.id.source_object,
                    via.meta.id.part,
                    layer.source_object
                );
                let via_id = ObjectId::new(&source, ObjectKind::Node, &via_key, "");
                let area_key = format!(
                    "b004-area-node:{}:{}:{}",
                    copper.meta.id.source_object, point.ticks[0], point.ticks[1]
                );
                let area_id = ObjectId::new(&source, ObjectKind::Node, &area_key, "");
                let via_node = ConnectivityNode {
                    meta: meta(&source, ObjectKind::Node, &via_key),
                    conductor: via.meta.id.clone(),
                    layer: layer.clone(),
                    site: ContactSite::ViaCentre,
                };
                let area_node = ConnectivityNode {
                    meta: meta(&source, ObjectKind::Node, &area_key),
                    conductor: copper.meta.id.clone(),
                    layer: layer.clone(),
                    site: ContactSite::AreaInterior(point),
                };
                for (id, candidate) in [(&via_id, &via_node), (&area_id, &area_node)] {
                    if nodes.get(id).is_some_and(|existing| {
                        existing.conductor != candidate.conductor
                            || existing.layer != candidate.layer
                            || existing.site != candidate.site
                    }) {
                        return Err("B006_NODE_ID_CONFLICT");
                    }
                }
                let other_layer = nodes
                    .values()
                    .find(|n| {
                        n.conductor == via.meta.id
                            && n.layer != layer
                            && matches!(n.site, ContactSite::ViaCentre)
                            && via.connected_layers.contains(&n.layer)
                            && board
                                .connectivity
                                .edges
                                .iter()
                                .chain(edges.iter())
                                .any(|e| {
                                    matches!(
                                        e.evidence,
                                        ConnectionEvidence::SameLayerContact { .. }
                                    ) && (e.a == n.meta.id || e.b == n.meta.id)
                                })
                    })
                    .map(|n| n.meta.id.clone());
                let new_area_witness = !nodes.contains_key(&area_id);
                nodes.entry(via_id.clone()).or_insert(via_node);
                nodes.entry(area_id.clone()).or_insert(area_node);
                if let Some(ref first) = anchor {
                    if first != &area_id && new_area_witness {
                        append_edge(
                            &source,
                            &mut edges,
                            "area",
                            first,
                            &area_id,
                            ConnectionEvidence::AreaContinuity {
                                area: copper.meta.id.clone(),
                            },
                        )?;
                    }
                } else {
                    anchor = Some(area_id.clone());
                }
                append_edge(&source, &mut edges, "via-area", &via_id, &area_id,
                    ConnectionEvidence::SameLayerContact {
                        evidence: "Original PADS plated Via conductive Padstack Layer and strict COPCLS/COPCUT copper-only interior, declared same Net".into(),
                    })?;
                if !board.vias[via_index].connected_layers.contains(&layer) {
                    board.vias[via_index].connected_layers.push(layer.clone());
                    if let Some(existing) = other_layer {
                        append_edge(
                            &source,
                            &mut edges,
                            "barrel",
                            &existing,
                            &via_id,
                            ConnectionEvidence::ViaBarrel {
                                via: via.meta.id.clone(),
                            },
                        )?;
                        audit.new_barrel_edges += 1;
                    }
                }
                area_updates.insert((area_index, net.clone()));
                audit.local_contacts += 1;
            }
        }
    }
    if board
        .connectivity
        .edges
        .len()
        .checked_add(edges.len())
        .map_or(true, |total| total > 100_000)
    {
        return Err("B006_GRAPH_LIMIT");
    }
    for (area_index, net) in area_updates {
        board.areas[area_index].net = Some(net);
    }
    board.connectivity.nodes = nodes.into_values().collect();
    board.connectivity.edges.extend(edges);
    if board.validate().has_errors() {
        return Err("B006_GRAPH_VALIDATION");
    }
    board.sources[0].capability.object_coverage.push(format!(
        "B_006 exact source Via-to-COPCLS local centreline contacts {}; new barred junctions {}; unsupported {}; return_path_p009=UNSUPPORTED, solver_ready=false",
        audit.local_contacts, audit.new_barrel_edges, audit.unsupported_candidates,
    ));
    Ok(audit)
}
