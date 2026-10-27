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
            // Index only an actual endpoint or an exact source-tick
            // straight Trace interior witness. Idle vias remain unconnected.
            let endpoint = locations.contains_key(&(entry.layer.clone(), via.position.ticks));
            let interior = board.traces.iter().any(|trace| {
                if trace.layer != entry.layer || trace.centreline.len() != 2 {
                    return false;
                }
                let a = trace.centreline[0].ticks;
                let b = trace.centreline[1].ticks;
                let p = via.position.ticks;
                let (ax, ay) = (i128::from(a[0]), i128::from(a[1]));
                let (bx, by) = (i128::from(b[0]), i128::from(b[1]));
                let (px, py) = (i128::from(p[0]), i128::from(p[1]));
                (bx - ax) * (py - ay) == (by - ay) * (px - ax)
                    && (px - ax) * (px - bx) + (py - ay) * (py - by) < 0
            });
            if !endpoint && !interior {
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

    // Preserve an independent y-ordered lookup to prevent long horizontal
    // traces from scanning every unrelated endpoint on the board.
    let by_y: BTreeMap<(ObjectId, [i64; 2]), [i64; 2]> = locations
        .keys()
        .map(|(layer, point)| ((layer.clone(), [point[1], point[0]]), *point))
        .collect();
    // Exact source-tick T-junctions: a different conductor's original
    // endpoint strictly inside a straight centreline. Not width overlap.
    for trace in &board.traces {
        let a = trace.centreline[0].ticks;
        let b = trace.centreline[1].ticks;
        let xmin = a[0].min(b[0]);
        let xmax = a[0].max(b[0]);
        let ymin = a[1].min(b[1]);
        let ymax = a[1].max(b[1]);
        let scan_y = i128::from(ymax) - i128::from(ymin) < i128::from(xmax) - i128::from(xmin);
        let candidates: Vec<[i64; 2]> = if scan_y {
            by_y.range(
                (trace.layer.clone(), [ymin, i64::MIN])..=(trace.layer.clone(), [ymax, i64::MAX]),
            )
            .map(|(_, point)| *point)
            .collect()
        } else {
            locations
                .range(
                    (trace.layer.clone(), [xmin, i64::MIN])
                        ..=(trace.layer.clone(), [xmax, i64::MAX]),
                )
                .map(|((_, point), _)| *point)
                .collect()
        };
        let witnesses: Vec<[i64; 2]> = candidates
            .into_iter()
            .filter(|ticks| {
                if ticks[0] < xmin || ticks[0] > xmax || ticks[1] < ymin || ticks[1] > ymax {
                    return false;
                }
                if !locations
                    .get(&(trace.layer.clone(), *ticks))
                    .is_some_and(|sites| sites.iter().any(|s| s.conductor != trace.meta.id))
                {
                    return false;
                }
                let (ax, ay) = (i128::from(a[0]), i128::from(a[1]));
                let (bx, by) = (i128::from(b[0]), i128::from(b[1]));
                let (px, py) = (i128::from(ticks[0]), i128::from(ticks[1]));
                let cross = (bx - ax) * (py - ay) - (by - ay) * (px - ax);
                let dot = (px - ax) * (px - bx) + (py - ay) * (py - by);
                cross == 0 && dot < 0
            })
            .collect();
        for (index, ticks) in witnesses.into_iter().enumerate() {
            let point = boardsi_model::Point {
                frame: trace.centreline[0].frame.clone(),
                ticks,
            };
            let candidate = site(
                &source,
                trace.meta.id.clone(),
                trace.layer.clone(),
                trace.net.clone(),
                ContactSite::TraceInterior(point),
                &format!("interior-{}-{}-{}", index, ticks[0], ticks[1]),
                None,
            );
            push_site(&mut locations, &trace.layer, ticks, candidate)?;
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

/// Add only strict source-grid Pad-centre / Trace-centreline contact evidence
/// after a separately accepted ROUTE pin-pair assigned an actual Pad Net.
/// Unassigned pads and width-overlap contacts remain unqualified.
pub fn augment_pad_centres(board: &mut BoardModel) -> Result<usize, &'static str> {
    let source = board.sources.first().ok_or("MISSING_SOURCE")?.id.clone();
    let mut nodes: BTreeMap<ObjectId, ConnectivityNode> = board
        .connectivity
        .nodes
        .iter()
        .map(|n| (n.meta.id.clone(), n.clone()))
        .collect();
    let mut new_edges = Vec::new();
    let mut qualified = 0usize;
    let mut cross_net = 0usize;
    for pad in &board.pads {
        let Some(pad_net) = &pad.net else {
            continue;
        };
        let padstack = board
            .padstacks
            .iter()
            .find(|stack| stack.meta.id == pad.padstack)
            .ok_or("MISSING_PADSTACK")?;
        for entry in padstack
            .entries
            .iter()
            .filter(|entry| entry.role == PadRole::Conductive)
        {
            for trace in board.traces.iter().filter(|t| t.layer == entry.layer) {
                if trace.centreline.len() != 2 {
                    return Err("UNSUPPORTED_CONTACT_TRACE");
                }
                let a = &trace.centreline[0];
                let b = &trace.centreline[1];
                let p = &pad.position;
                let (ax, ay) = (i128::from(a.ticks[0]), i128::from(a.ticks[1]));
                let (bx, by) = (i128::from(b.ticks[0]), i128::from(b.ticks[1]));
                let (px, py) = (i128::from(p.ticks[0]), i128::from(p.ticks[1]));
                let cross = (bx - ax) * (py - ay) - (by - ay) * (px - ax);
                let dot = (px - ax) * (px - bx) + (py - ay) * (py - by);
                if a.frame != p.frame || b.frame != p.frame || cross != 0 || dot > 0 {
                    continue;
                }
                let contact = if p.ticks == a.ticks {
                    ContactSite::TraceVertex(0)
                } else if p.ticks == b.ticks {
                    ContactSite::TraceVertex(1)
                } else {
                    ContactSite::TraceInterior(p.clone())
                };
                let trace_name = if p.ticks == a.ticks {
                    "start".to_owned()
                } else if p.ticks == b.ticks {
                    "end".to_owned()
                } else {
                    format!("pad-interior-{}-{}", p.ticks[0], p.ticks[1])
                };
                let trace_site = site(
                    &source,
                    trace.meta.id.clone(),
                    trace.layer.clone(),
                    trace.net.clone(),
                    contact,
                    &trace_name,
                    None,
                );
                let pad_site = site(
                    &source,
                    pad.meta.id.clone(),
                    entry.layer.clone(),
                    pad.net.clone(),
                    ContactSite::PadCentre,
                    "centre",
                    None,
                );
                if trace.net.as_ref() != Some(pad_net) {
                    cross_net += 1;
                    continue;
                }
                nodes
                    .entry(trace_site.id.clone())
                    .or_insert_with(|| contact_node(&source, &trace_site));
                nodes
                    .entry(pad_site.id.clone())
                    .or_insert_with(|| contact_node(&source, &pad_site));
                add_edge(
                    &source,
                    &mut new_edges,
                    &trace_site.id,
                    &pad_site.id,
                    "pad-centre",
                    ConnectionEvidence::SameLayerContact {
                        evidence: "source ROUTE pin-pair Net assignment and exact PADS tick Pad centre on straight Trace centreline".into(),
                    },
                )?;
                qualified += 1;
            }
        }
    }
    if board.connectivity.edges.len() + new_edges.len() > MAX_EDGES {
        return Err("CONTACT_EDGE_LIMIT");
    }
    board.connectivity.nodes = nodes.into_values().collect();
    board.connectivity.edges.extend(new_edges);
    board.sources[0].capability.object_coverage.push(format!(
        "B_003 strict Pad centre/Trace centreline verified local edges {}; cross-Net coincidences {} (not certified shorts); arbitrary width overlap NOT_EVALUATED",
        qualified, cross_net
    ));
    Ok(qualified)
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
    fn strict_t_junction_is_connected_but_near_miss_is_not() {
        let original = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n500 200 1 10 0\nU3.1 U4.2\n300 400 1 10 0\n300 200 1 10 0\n*END*\n";
        let parsed = parse(original.as_bytes()).unwrap();
        let board = into_canonical(original.as_bytes(), &parsed).unwrap();
        assert_eq!(board.connectivity.edges.len(), 1);
        assert!(board
            .connectivity
            .nodes
            .iter()
            .any(|node| matches!(node.site, ContactSite::TraceInterior(_))));
        assert!(!board.validate().has_errors());
        let offset = original.replace("300 200 1 10 0", "300 200.01 1 10 0");
        let parsed = parse(offset.as_bytes()).unwrap();
        assert!(into_canonical(offset.as_bytes(), &parsed)
            .unwrap()
            .connectivity
            .edges
            .is_empty());
    }

    #[test]
    fn cross_net_t_junction_is_not_certified_as_connected() {
        let source = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n500 200 1 10 0\n*SIGNAL* NET_B\nU3.1 U4.2\n300 400 1 10 0\n300 200 1 10 0\n*END*\n";
        let parsed = parse(source.as_bytes()).unwrap();
        let board = into_canonical(source.as_bytes(), &parsed).unwrap();
        assert!(board.connectivity.edges.is_empty());
        assert!(board.sources[0]
            .capability
            .object_coverage
            .iter()
            .any(|s| s.contains("potential net conflicts 1")));
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
    fn plated_via_at_exact_trace_interior_proves_second_layer_contact() {
        let source = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*VIA*\nSIMPLEVIA 15 2\n-2 35 R\n0 35 R\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 400 1 10 0\n500 400 1 10 0\nU3.1 U4.2\n100 400 4 10 0\n300 400 4 10 0 SIMPLEVIA\n500 400 4 10 0\n*END*\n";
        let parsed = parse(source.as_bytes()).unwrap();
        let board = into_canonical(source.as_bytes(), &parsed).unwrap();
        assert_eq!(board.vias.len(), 1);
        assert_eq!(board.vias[0].connected_layers.len(), 2);
        assert!(board
            .connectivity
            .edges
            .iter()
            .any(|edge| { matches!(edge.evidence, ConnectionEvidence::ViaBarrel { .. }) }));
        assert!(!board.validate().has_errors());
        let shifted = source.replace("500 400 1 10 0", "500 400.01 1 10 0");
        let parsed = parse(shifted.as_bytes()).unwrap();
        let board = into_canonical(shifted.as_bytes(), &parsed).unwrap();
        assert_eq!(board.vias[0].connected_layers.len(), 1);
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
    fn ten_thousand_segment_chain_preserves_exact_local_connectivity() {
        // GUI import permits this bounded size. Exercise the formerly
        // quadratic node/edge validation with fully generated public data.
        let mut source = String::from(
            "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* NET_A\n",
        );
        for index in 0..10_000usize {
            source.push_str(&format!(
                "U1.1 U2.2\n{} 200 1 10 0\n{} 200 1 10 0\n",
                100 + index,
                101 + index
            ));
        }
        source.push_str("*END*\n");
        let started = std::time::Instant::now();
        let parsed = parse(source.as_bytes()).unwrap();
        assert_eq!(parsed.segments.len(), 10_000);
        let board = into_canonical(source.as_bytes(), &parsed).unwrap();
        assert_eq!(board.connectivity.edges.len(), 9_999);
        assert!(!board.validate().has_errors());
        eprintln!(
            "A_032_1 10000 segment chain debug-test duration: {:?} (not a release or GUI deadline benchmark)",
            started.elapsed()
        );
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
