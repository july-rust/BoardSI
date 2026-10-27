//! B_004: bounded exact interior contacts of source-filled, hole-free polygons.
//! This never qualifies physical reference return, shorts, trace width or SI.
use crate::copper_preview::{CopperArea as SourceArea, CopperGeometry};
use boardsi_model::{
    strict_simple_area_contains, BoardModel, ConnectionEvidence, ConnectivityEdge,
    ConnectivityNode, ContactSite, GeometryProvenance, ObjectId, ObjectKind, ObjectMeta, PadRole,
    Point, SourceId,
};
use std::collections::{BTreeMap, BTreeSet};

const MAX_AREA_CONTACT_EDGES: usize = 100_000;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AreaContactAudit {
    pub exact_contacts: usize,
    pub same_area_links: usize,
    pub cross_net_coincidences: usize,
    pub unsupported_areas: usize,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReferenceLocalEvidence {
    pub state: &'static str,
    pub area_ids: Vec<ObjectId>,
    pub conductor_ids: Vec<ObjectId>,
}
#[derive(Clone)]
struct Candidate {
    owner: ObjectId,
    net: ObjectId,
    layer: ObjectId,
    site: ContactSite,
    label: &'static str,
}
fn meta(source: &SourceId, kind: ObjectKind, key: &str) -> ObjectMeta {
    ObjectMeta {
        id: ObjectId::new(source, kind, key, ""),
        provenance: GeometryProvenance {
            source_object_type: "B_004 bounded PADS source-grid interior contact; not physical SI"
                .into(),
            history: vec![],
        },
    }
}
fn node(source: &SourceId, c: &Candidate) -> ConnectivityNode {
    let key = format!(
        "{:?}/{}:{}/{}:{}",
        c.owner.kind, c.owner.source_object, c.owner.part, c.layer.source_object, c.label
    );
    ConnectivityNode {
        meta: meta(source, ObjectKind::Node, &key),
        conductor: c.owner.clone(),
        layer: c.layer.clone(),
        site: c.site.clone(),
    }
}
fn add(
    source: &SourceId,
    new: &mut Vec<ConnectivityEdge>,
    a: &ObjectId,
    b: &ObjectId,
    kind: &str,
    evidence: ConnectionEvidence,
) -> Result<(), &'static str> {
    if new.len() >= MAX_AREA_CONTACT_EDGES {
        return Err("B004_EDGE_LIMIT");
    }
    let (first, second) = if a < b { (a, b) } else { (b, a) };
    new.push(ConnectivityEdge {
        meta: meta(
            source,
            ObjectKind::Edge,
            &format!(
                "b004-{kind}:{}--{}",
                first.source_object, second.source_object
            ),
        ),
        a: first.clone(),
        b: second.clone(),
        evidence,
    });
    Ok(())
}
/// Add B_004 evidence only after the actual accepted source Copper parser
/// has populated the same Canonical Area set. Never infer a missing Net.
pub fn augment(
    board: &mut BoardModel,
    source_areas: &[SourceArea],
) -> Result<AreaContactAudit, &'static str> {
    if source_areas.len() != board.areas.len() {
        return Err("B004_AREA_SOURCE_MISMATCH");
    }
    let source = board.sources.first().ok_or("MISSING_SOURCE")?.id.clone();
    let mut locations: BTreeMap<(ObjectId, [i64; 2]), Vec<Candidate>> = BTreeMap::new();
    for t in &board.traces {
        if t.centreline.len() != 2 {
            return Err("B004_TRACE_UNSUPPORTED");
        }
        if let Some(net) = &t.net {
            for (i, p) in t.centreline.iter().enumerate() {
                locations
                    .entry((t.layer.clone(), p.ticks))
                    .or_default()
                    .push(Candidate {
                        owner: t.meta.id.clone(),
                        net: net.clone(),
                        layer: t.layer.clone(),
                        site: ContactSite::TraceVertex(i),
                        label: if i == 0 { "start" } else { "end" },
                    });
            }
        }
    }
    for pad in &board.pads {
        let Some(net) = &pad.net else { continue };
        let stack = board
            .padstacks
            .iter()
            .find(|s| s.meta.id == pad.padstack)
            .ok_or("B004_MISSING_PADSTACK")?;
        for layer in stack
            .entries
            .iter()
            .filter(|s| s.role == PadRole::Conductive)
        {
            locations
                .entry((layer.layer.clone(), pad.position.ticks))
                .or_default()
                .push(Candidate {
                    owner: pad.meta.id.clone(),
                    net: net.clone(),
                    layer: layer.layer.clone(),
                    site: ContactSite::PadCentre,
                    label: "centre",
                });
        }
    }
    let mut nodes: BTreeMap<ObjectId, ConnectivityNode> = board
        .connectivity
        .nodes
        .iter()
        .map(|n| (n.meta.id.clone(), n.clone()))
        .collect();
    let mut edges = Vec::new();
    let mut audit = AreaContactAudit::default();
    let mut used_pairs = BTreeSet::new();
    for (index, input) in source_areas.iter().enumerate() {
        let area = &board.areas[index];
        let expected_id = ObjectId::new(
            &source,
            ObjectKind::Area,
            &format!("source-copper-{}-{}", input.name, input.source_line),
            "",
        );
        if area.meta.id != expected_id {
            return Err("B004_AREA_SOURCE_MISMATCH");
        }
        let CopperGeometry::Polygon(ref ring) = input.geometry else {
            audit.unsupported_areas += 1;
            continue;
        };
        if !input.cutouts.is_empty() || ring.len() < 4 || ring.len() > 65 {
            audit.unsupported_areas += 1;
            continue;
        }
        let Some(label) = &input.net_label else {
            audit.unsupported_areas += 1;
            continue;
        };
        let Some(net) = board
            .nets
            .iter()
            .find(|n| n.name == *label)
            .map(|n| n.meta.id.clone())
        else {
            audit.unsupported_areas += 1;
            continue;
        };
        let minx = ring.iter().map(|p| p[0]).min().ok_or("B004_EMPTY_RING")?;
        let maxx = ring.iter().map(|p| p[0]).max().ok_or("B004_EMPTY_RING")?;
        let mut anchor: Option<(ObjectId, [i64; 2])> = None;
        let mut matched = 0usize;
        for ((_, xy), candidates) in locations
            .range((area.layer.clone(), [minx, i64::MIN])..=(area.layer.clone(), [maxx, i64::MAX]))
        {
            let p = Point {
                frame: board.frames[0].meta.id.clone(),
                ticks: *xy,
            };
            if !strict_simple_area_contains(area, &p).unwrap_or(false) {
                continue;
            }
            for candidate in candidates {
                if candidate.net != net {
                    audit.cross_net_coincidences += 1;
                    continue;
                }
                let c_node = node(&source, candidate);
                let a_node = node(
                    &source,
                    &Candidate {
                        owner: area.meta.id.clone(),
                        net: net.clone(),
                        layer: area.layer.clone(),
                        site: ContactSite::AreaWitness {
                            point: p.clone(),
                            evidence: "source-filled single simple COPCLS exact strict interior"
                                .into(),
                        },
                        label: "area-interior",
                    },
                );
                // Distinct witness sites at distinct positions MUST have distinct IDs.
                let a_key = format!(
                    "b004-area-node:{}:{}:{}",
                    area.meta.id.source_object, xy[0], xy[1]
                );
                let a_id = ObjectId::new(&source, ObjectKind::Node, &a_key, "");
                let mut area_node = a_node;
                area_node.meta = meta(&source, ObjectKind::Node, &a_key);
                if !used_pairs.insert((a_id.clone(), c_node.meta.id.clone())) {
                    continue;
                }
                if let Some((ref root, root_xy)) = anchor {
                    if root_xy != *xy && !nodes.contains_key(&a_id) {
                        add(
                            &source,
                            &mut edges,
                            root,
                            &a_id,
                            "area-fill",
                            ConnectionEvidence::AreaContinuity {
                                area: area.meta.id.clone(),
                            },
                        )?;
                        audit.same_area_links += 1;
                    }
                } else {
                    anchor = Some((a_id.clone(), *xy));
                }
                nodes.entry(a_id.clone()).or_insert(area_node);
                nodes
                    .entry(c_node.meta.id.clone())
                    .or_insert(c_node.clone());
                add(&source,&mut edges,&a_id,&c_node.meta.id,"local",
                    ConnectionEvidence::SameLayerContact {
                        evidence:"PADS source-filled simple polygon strict interior and matching declared Net".into(),
                    })?;
                matched += 1;
                audit.exact_contacts += 1;
            }
        }
        if matched > 0 {
            board.areas[index].net = Some(net);
        }
    }
    if board
        .connectivity
        .edges
        .len()
        .checked_add(edges.len())
        .map_or(true, |count| count > MAX_AREA_CONTACT_EDGES)
    {
        return Err("B004_EDGE_LIMIT");
    }
    board.connectivity.nodes = nodes.into_values().collect();
    board.connectivity.edges.extend(edges);
    board.sources[0].capability.object_coverage.push(format!(
        "B_004 strict simple-COPCLS local contacts {}; area links {}; cross-Net {} (not certified shorts); unsupported areas {}; P009 UNSUPPORTED",
        audit.exact_contacts,audit.same_area_links,audit.cross_net_coincidences,audit.unsupported_areas
    ));
    Ok(audit)
}

/// Local candidate report only. No contact never proves whole-board disconnection.
pub fn local_reference_candidate(board: &BoardModel, name: &str) -> ReferenceLocalEvidence {
    let mut result = ReferenceLocalEvidence {
        state: "UNSUPPORTED",
        ..Default::default()
    };
    // Never report a verified contact from altered or forged Canonical evidence.
    if board.validate().has_errors() {
        return result;
    }
    let Some(net) = board.nets.iter().find(|n| n.name == name) else {
        return result;
    };
    let area_ids: BTreeSet<_> = board
        .areas
        .iter()
        .filter(|a| a.net.as_ref() == Some(&net.meta.id))
        .map(|a| a.meta.id.clone())
        .collect();
    let lookup: BTreeMap<_, _> = board
        .connectivity
        .nodes
        .iter()
        .map(|n| (&n.meta.id, n))
        .collect();
    let mut contacts = BTreeSet::new();
    for edge in &board.connectivity.edges {
        if !matches!(edge.evidence, ConnectionEvidence::SameLayerContact { .. }) {
            continue;
        }
        for (a, b) in [(&edge.a, &edge.b), (&edge.b, &edge.a)] {
            if let (Some(area), Some(other)) = (lookup.get(a), lookup.get(b)) {
                if area_ids.contains(&area.conductor)
                    && matches!(area.site, ContactSite::AreaWitness { .. })
                {
                    contacts.insert(other.conductor.clone());
                }
            }
        }
    }
    if !contacts.is_empty() {
        result.state = "LOCAL_CONTACT_VERIFIED";
        result.area_ids = area_ids.into_iter().collect();
        result.conductor_ids = contacts.into_iter().collect();
    }
    result
}
