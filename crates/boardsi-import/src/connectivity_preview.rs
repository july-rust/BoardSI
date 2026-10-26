//! A_032 P2-W: conservative, source-grid exact ROUTE endpoint contacts.
//! Only local endpoint coincidence is evidence; not full-board connectivity,
//! polygon contact, flooding, pad-pin assignment, or complete short/open proof.
use boardsi_model::{
    BoardModel, ConnectionEvidence, ConnectivityEdge, ConnectivityNode, ContactSite,
    GeometryProvenance, ObjectId, ObjectKind, ObjectMeta, PadRole, SourceId, ViaKind,
};
use std::collections::BTreeMap;

const MAX_SITES_PER_POINT: usize = 32;
const MAX_EDGES: usize = 100_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ContactAudit {
    pub local_edges: usize,
    pub via_barrel_edges: usize,
    /// Different declared nets at identical *supported endpoint* sites.
    /// This is not a certified copper short.
    pub potential_conflicts: usize,
}

#[derive(Clone)]
struct Site {
    id: ObjectId,
    conductor: ObjectId,
    layer: ObjectId,
    net: Option<ObjectId>,
    site: ContactSite,
    via_index: Option<usize>,
}

fn metadata(source: &SourceId, kind: ObjectKind, key: &str, note: &str) -> ObjectMeta {
    ObjectMeta {
        id: ObjectId::new(source, kind, key, ""),
        provenance: GeometryProvenance {
            source_object_type: note.to_owned(),
            history: vec![],
        },
    }
}

fn site(
    source: &SourceId,
    conductor: ObjectId,
    layer: ObjectId,
    net: Option<ObjectId>,
    contact: ContactSite,
    site_name: &str,
    via_index: Option<usize>,
) -> Site {
    let id = ObjectId::new(
        source,
        ObjectKind::Node,
        &format!(
            "{:?}/{}:{}/{}:{}",
            conductor.kind, conductor.source_object, conductor.part, layer.source_object, site_name
        ),
        "",
    );
    Site {
        id,
        conductor,
        layer,
        net,
        site: contact,
        via_index,
    }
}

fn contact_node(source: &SourceId, item: &Site) -> ConnectivityNode {
    ConnectivityNode {
        meta: metadata(
            source,
            ObjectKind::Node,
            &item.id.source_object,
            "PADS V9.4 MILS exact original ROUTE endpoint/explicit via conductive layer",
        ),
        conductor: item.conductor.clone(),
        layer: item.layer.clone(),
        site: item.site.clone(),
    }
}

fn add_edge(
    source: &SourceId,
    edges: &mut Vec<ConnectivityEdge>,
    a: &ObjectId,
    b: &ObjectId,
    kind: &str,
    evidence: ConnectionEvidence,
) -> Result<(), &'static str> {
    if edges.len() >= MAX_EDGES {
        return Err("CONTACT_EDGE_LIMIT");
    }
    let (first, second) = if a <= b { (a, b) } else { (b, a) };
    edges.push(ConnectivityEdge {
        meta: metadata(
            source,
            ObjectKind::Edge,
            &format!("{}:{}--{}", kind, first.source_object, second.source_object),
            "PADS V9.4 MILS bounded local contact evidence, not full PCB connectivity",
        ),
        a: first.clone(),
        b: second.clone(),
        evidence,
    });
    Ok(())
}

// Reject oversized buckets before allocating the next contact record.
fn push_site(
    locations: &mut BTreeMap<(ObjectId, [i64; 2]), Vec<Site>>,
    layer: &ObjectId,
    ticks: [i64; 2],
    candidate: Site,
) -> Result<(), &'static str> {
    let sites = locations.entry((layer.clone(), ticks)).or_default();
    if sites.len() >= MAX_SITES_PER_POINT {
        return Err("CONTACT_BUCKET_LIMIT");
    }
    sites.push(candidate);
    Ok(())
}

/// Appends *only* endpoint-supported graph evidence to the existing model.
/// Requires the V9.4 source geometry to have been validated by its parsers.
/// A failure returns an error; callers must discard the partial model.
pub fn augment(board: &mut BoardModel) -> Result<ContactAudit, &'static str> {
    if !board.connectivity.nodes.is_empty() || !board.connectivity.edges.is_empty() {
        return Err("CONTACT_GRAPH_ALREADY_PRESENT");
    }
    let source = board.sources.first().ok_or("MISSING_SOURCE")?.id.clone();
    let mut locations: BTreeMap<(ObjectId, [i64; 2]), Vec<Site>> = BTreeMap::new();
    for trace in &board.traces {
        // Current restricted importer creates straight two-point segments only.
        if trace.centreline.len() != 2 {
            return Err("UNSUPPORTED_CONTACT_TRACE");
        }
        for (index, point) in trace.centreline.iter().enumerate() {
            let candidate = site(
                &source,
                trace.meta.id.clone(),
                trace.layer.clone(),
                trace.net.clone(),
                ContactSite::TraceVertex(index),
                if index == 0 { "start" } else { "end" },
                None,
            );
            push_site(&mut locations, &trace.layer, point.ticks, candidate)?;
        }
    }
    for arc in &board.arcs {
        for (contact, point, name) in [
            (ContactSite::ArcStart, &arc.start, "start"),
            (ContactSite::ArcEnd, &arc.end, "end"),
        ] {
            let candidate = site(
                &source,
                arc.meta.id.clone(),
                arc.layer.clone(),
                arc.net.clone(),
                contact,
                name,
                None,
            );
            push_site(&mut locations, &arc.layer, point.ticks, candidate)?;
        }
    }
    for (index, via) in board.vias.iter().enumerate() {
        let drill = board
            .drills
            .iter()
            .find(|x| x.meta.id == via.drill)
            .ok_or("MISSING_CONTACT_DRILL")?;
        if !drill.plated || via.kind != ViaKind::Through {
            continue;
        }
        let padstack = board
            .padstacks
            .iter()
            .find(|x| x.meta.id == via.padstack)
            .ok_or("MISSING_CONTACT_PADSTACK")?;
        for entry in padstack
            .entries
            .iter()
            .filter(|e| e.role == PadRole::Conductive)
        {
            // No endpoint to connect on this layer: do not index an idle
            // via barrel/pad as though its span established an actual contact.
            if !locations.contains_key(&(entry.layer.clone(), via.position.ticks)) {
                continue;
            }
            let candidate = site(
                &source,
                via.meta.id.clone(),
                entry.layer.clone(),
                via.net.clone(),
                ContactSite::ViaCentre,
                "centre",
                Some(index),
            );
            push_site(&mut locations, &entry.layer, via.position.ticks, candidate)?;
        }
    }

    let mut nodes: BTreeMap<ObjectId, ConnectivityNode> = BTreeMap::new();
    let mut edges = Vec::new();
    let mut via_contacts: BTreeMap<usize, BTreeMap<ObjectId, ObjectId>> = BTreeMap::new();
    let mut audit = ContactAudit::default();

    for (_location, mut sites) in locations {
        if sites.len() > MAX_SITES_PER_POINT {
            return Err("CONTACT_BUCKET_LIMIT");
        }
        sites.sort_by(|a, b| a.id.cmp(&b.id));
        for first in 0..sites.len() {
            for second in first + 1..sites.len() {
                let a = &sites[first];
                let b = &sites[second];
                // One source conductor's two sites are not a contact with another
                // conductor; distinct via centres do not imply a barrel contact.
                if a.conductor == b.conductor || (a.via_index.is_some() && b.via_index.is_some()) {
                    continue;
                }
                let (Some(net_a), Some(net_b)) = (&a.net, &b.net) else {
                    continue;
                };
                if net_a != net_b {
                    audit.potential_conflicts += 1;
                    continue;
                }
                nodes
                    .entry(a.id.clone())
                    .or_insert_with(|| contact_node(&source, a));
                nodes
                    .entry(b.id.clone())
                    .or_insert_with(|| contact_node(&source, b));
                add_edge(
                    &source,
                    &mut edges,
                    &a.id,
                    &b.id,
                    "same-layer",
                    ConnectionEvidence::SameLayerContact {
                        evidence: "identical original PADS source-tick endpoint, same layer and declared NetId".into(),
                    },
                )?;
                audit.local_edges += 1;
                for item in [a, b] {
                    if let Some(index) = item.via_index {
                        via_contacts
                            .entry(index)
                            .or_default()
                            .insert(item.layer.clone(), item.id.clone());
                    }
                }
            }
        }
    }

    // The physical barrel evidence is only between *two* independently
    // qualified local contacts at a plated via's conductive layer sites.
    for (index, contacts) in via_contacts {
        board.vias[index].connected_layers = contacts.keys().cloned().collect();
        let ordered: Vec<_> = contacts.values().collect();
        for pair in ordered.windows(2) {
            add_edge(
                &source,
                &mut edges,
                pair[0],
                pair[1],
                "plated-via",
                ConnectionEvidence::ViaBarrel {
                    via: board.vias[index].meta.id.clone(),
                },
            )?;
            audit.via_barrel_edges += 1;
        }
    }
    board.connectivity.nodes = nodes.into_values().collect();
    board.connectivity.edges = edges;
    board.sources[0].capability.object_coverage.push(format!(
        "A_032 local endpoint-only contact edges {}; via barrel edges {}; potential net conflicts {}; non-endpoint/area contacts NOT_EVALUATED",
        audit.local_edges, audit.via_barrel_edges, audit.potential_conflicts
    ));
    Ok(audit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route_preview::{into_canonical, parse};
    const TWO: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n300 200 1 10 0\nU2.2 U3.3\n300 200 1 10 0\n500 200 1 10 0\n*END*\n";
    #[test]
    fn exact_join_is_source_backed_and_deterministic() {
        let parsed = parse(TWO.as_bytes()).unwrap();
        let a = into_canonical(TWO.as_bytes(), &parsed).unwrap();
        let b = into_canonical(TWO.as_bytes(), &parsed).unwrap();
        assert_eq!(a.connectivity, b.connectivity);
        assert_eq!(a.connectivity.edges.len(), 1);
        assert_eq!(a.connectivity.nodes.len(), 2);
        assert!(!a.validate().has_errors());
    }

    #[test]
    fn no_cross_layer_or_near_miss_contact() {
        let altered = TWO.replace("U2.2 U3.3\n300 200 1 10 0", "U2.2 U3.3\n300 200 2 10 0");
        let parsed = parse(altered.as_bytes()).unwrap();
        assert!(into_canonical(altered.as_bytes(), &parsed)
            .unwrap()
            .connectivity
            .edges
            .is_empty());
        let shifted = TWO.replace("U2.2 U3.3\n300 200 1 10 0", "U2.2 U3.3\n300.01 200 1 10 0");
        let parsed = parse(shifted.as_bytes()).unwrap();
        assert!(into_canonical(shifted.as_bytes(), &parsed)
            .unwrap()
            .connectivity
            .edges
            .is_empty());
    }

    #[test]
    fn different_source_nets_are_not_connected() {
        let changed = TWO.replace(
            "U2.2 U3.3\n300 200 1 10 0",
            "*SIGNAL* NET_B\nU2.2 U3.3\n300 200 1 10 0",
        );
        let parsed = parse(changed.as_bytes()).unwrap();
        let board = into_canonical(changed.as_bytes(), &parsed).unwrap();
        assert!(board.connectivity.edges.is_empty());
        assert!(board.sources[0]
            .capability
            .object_coverage
            .iter()
            .any(|s| s.contains("potential net conflicts 1")));
    }

    #[test]
    fn verified_arc_to_trace_endpoint_only() {
        let source = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* ARC_NET\nU1.1 U2.2\n200 100 1 10 0\n100 100 1 10 4096 CCW\n100 200 1 10 0\nU2.2 U3.3\n100 200 1 10 0\n300 200 1 10 0\n*END*\n";
        let parsed = parse(source.as_bytes()).unwrap();
        assert_eq!(parsed.arcs.len(), 1);
        assert_eq!(parsed.segments.len(), 1);
        let board = into_canonical(source.as_bytes(), &parsed).unwrap();
        assert_eq!(board.connectivity.edges.len(), 1);
        assert!(!board.validate().has_errors());
    }

    #[test]
    fn via_without_inner_conductive_pad_never_invents_layer_two_barrel() {
        let source = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*VIA*\nSIMPLEVIA 15 2\n-2 35 R\n0 35 R\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n300 200 2 10 0 SIMPLEVIA\n500 200 2 10 0\n*END*\n";
        let parsed = parse(source.as_bytes()).unwrap();
        let board = into_canonical(source.as_bytes(), &parsed).unwrap();
        assert_eq!(board.vias[0].connected_layers.len(), 1);
        assert_eq!(board.connectivity.edges.len(), 1);
        assert!(!board.validate().has_errors());
    }

    #[test]
    fn via_barrel_requires_both_layer_contacts() {
        const VIA: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*VIA*\nSIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n300 200 2 10 0 SIMPLEVIA\n500 200 2 10 0\n*END*\n";
        let parsed = parse(VIA.as_bytes()).unwrap();
        let board = into_canonical(VIA.as_bytes(), &parsed).unwrap();
        assert_eq!(board.vias[0].connected_layers.len(), 2);
        assert_eq!(
            board
                .connectivity
                .edges
                .iter()
                .filter(|e| matches!(e.evidence, ConnectionEvidence::ViaBarrel { .. }))
                .count(),
            1
        );
        assert!(!board.validate().has_errors());
    }
    #[test]
    fn oversized_contact_bucket_fails_during_insertion() {
        // Every trace starts at one exact coordinate, with a distinct end.
        // The 33rd site must be rejected without accumulating all contacts.
        let mut source = String::from(
            "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* NET_A\n",
        );
        for index in 0..(MAX_SITES_PER_POINT + 1) {
            source.push_str(&format!(
                "U1.1 U2.2\n100 200 1 10 0\n{} 300 1 10 0\n",
                200 + index
            ));
        }
        source.push_str("*END*\n");
        let parsed = parse(source.as_bytes()).unwrap();
        assert_eq!(
            into_canonical(source.as_bytes(), &parsed).unwrap_err(),
            "CONTACT_BUCKET_LIMIT"
        );
    }

}
