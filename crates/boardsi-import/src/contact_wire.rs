//! A_032_2 opt-in source-linked local contact records for Qt.
//! Never certifies entire-board shorts/opens or areas/pins.
//! Default legacy artwork stdout V1-V14 is byte-for-byte unaffected.
use boardsi_model::{
    BoardModel, ConnectionEvidence, ConnectivityNode, ContactSite, ObjectId, PadRole, ViaKind,
};
use std::collections::BTreeMap;
use std::fmt::Write as _;

const NM_PER_TICK: i64 = 254;
const MAX_RECORDS: usize = 100_000;
const MAX_RECORD_BYTES: usize = 1024 * 1024;
const MAX_SITES_PER_POINT: usize = 32;

#[derive(Clone)]
struct Endpoint {
    kind: char,
    index: usize,
    site: usize,
    layer: usize,
    ticks: [i64; 2],
    net: ObjectId,
}

fn resolve(
    board: &BoardModel,
    objects: &BTreeMap<ObjectId, (char, usize)>,
    layers: &BTreeMap<ObjectId, usize>,
    node: &ConnectivityNode,
) -> Result<Endpoint, &'static str> {
    let (kind, index) = *objects
        .get(&node.conductor)
        .ok_or("CONTACT_WIRE_UNKNOWN_OBJECT")?;
    let layer = *layers
        .get(&node.layer)
        .ok_or("CONTACT_WIRE_UNKNOWN_LAYER")?;
    let (site, ticks, net) = match (kind, &node.site) {
        ('T', ContactSite::TraceVertex(v)) if *v < 2 => {
            let item = &board.traces[index];
            if item.layer != node.layer || item.centreline.len() != 2 {
                return Err("CONTACT_WIRE_TRACE_MISMATCH");
            }
            (*v, item.centreline[*v].ticks, item.net.as_ref())
        }
        ('A', ContactSite::ArcStart) => {
            let item = &board.arcs[index];
            if item.layer != node.layer {
                return Err("CONTACT_WIRE_ARC_MISMATCH");
            }
            (0, item.start.ticks, item.net.as_ref())
        }
        ('A', ContactSite::ArcEnd) => {
            let item = &board.arcs[index];
            if item.layer != node.layer {
                return Err("CONTACT_WIRE_ARC_MISMATCH");
            }
            (1, item.end.ticks, item.net.as_ref())
        }
        ('V', ContactSite::ViaCentre) => {
            let item = &board.vias[index];
            if !item.connected_layers.contains(&node.layer) {
                return Err("CONTACT_WIRE_VIA_UNQUALIFIED");
            }
            (0, item.position.ticks, item.net.as_ref())
        }
        _ => return Err("CONTACT_WIRE_UNSUPPORTED_SITE"),
    };
    let net = net.cloned().ok_or("CONTACT_WIRE_NO_NET")?;
    Ok(Endpoint {
        kind,
        index,
        site,
        layer,
        ticks,
        net,
    })
}

fn bounded_push(
    buckets: &mut BTreeMap<(usize, [i64; 2]), Vec<Endpoint>>,
    item: Endpoint,
) -> Result<(), &'static str> {
    let sites = buckets.entry((item.layer, item.ticks)).or_default();
    if sites.len() >= MAX_SITES_PER_POINT {
        return Err("CONTACT_WIRE_BUCKET_LIMIT");
    }
    sites.push(item);
    Ok(())
}

// Use exactly the supported source endpoints. For a potential cross-Net
// diagnostic, an otherwise unconnected Via still qualifies as a candidate
// only on explicitly conductive plated-padstack layers. It is NOT a Short.
fn potential_conflicts(
    board: &BoardModel,
    layers: &BTreeMap<ObjectId, usize>,
) -> Result<Vec<(Endpoint, Endpoint)>, &'static str> {
    let mut buckets: BTreeMap<(usize, [i64; 2]), Vec<Endpoint>> = BTreeMap::new();
    for (index, t) in board.traces.iter().enumerate() {
        if t.centreline.len() != 2 {
            return Err("CONTACT_WIRE_TRACE_GEOMETRY");
        }
        let Some(net) = &t.net else {
            continue;
        };
        let layer = *layers.get(&t.layer).ok_or("CONTACT_WIRE_UNKNOWN_LAYER")?;
        for (site, point) in t.centreline.iter().enumerate() {
            bounded_push(
                &mut buckets,
                Endpoint {
                    kind: 'T',
                    index,
                    site,
                    layer,
                    ticks: point.ticks,
                    net: net.clone(),
                },
            )?;
        }
    }
    for (index, a) in board.arcs.iter().enumerate() {
        let Some(net) = &a.net else {
            continue;
        };
        let layer = *layers.get(&a.layer).ok_or("CONTACT_WIRE_UNKNOWN_LAYER")?;
        for (site, point) in [(0, &a.start), (1, &a.end)] {
            bounded_push(
                &mut buckets,
                Endpoint {
                    kind: 'A',
                    index,
                    site,
                    layer,
                    ticks: point.ticks,
                    net: net.clone(),
                },
            )?;
        }
    }
    let drills: BTreeMap<_, _> = board
        .drills
        .iter()
        .map(|x| (x.meta.id.clone(), x))
        .collect();
    let padstacks: BTreeMap<_, _> = board
        .padstacks
        .iter()
        .map(|x| (x.meta.id.clone(), x))
        .collect();
    for (index, v) in board.vias.iter().enumerate() {
        let Some(net) = &v.net else {
            continue;
        };
        let drill = drills.get(&v.drill).ok_or("CONTACT_WIRE_MISSING_DRILL")?;
        if v.kind != ViaKind::Through || !drill.plated {
            continue;
        }
        let padstack = padstacks
            .get(&v.padstack)
            .ok_or("CONTACT_WIRE_MISSING_PADSTACK")?;
        for entry in &padstack.entries {
            if entry.role != PadRole::Conductive {
                continue;
            }
            let layer = *layers
                .get(&entry.layer)
                .ok_or("CONTACT_WIRE_UNKNOWN_LAYER")?;
            // Add the Via only if a supported endpoint already exists at
            // precisely the centre on this conductive layer.
            if buckets.contains_key(&(layer, v.position.ticks)) {
                bounded_push(
                    &mut buckets,
                    Endpoint {
                        kind: 'V',
                        index,
                        site: 0,
                        layer,
                        ticks: v.position.ticks,
                        net: net.clone(),
                    },
                )?;
            }
        }
    }
    let mut conflicts = Vec::new();
    for (_, mut sites) in buckets {
        sites.sort_by_key(|s| (s.kind, s.index, s.site, s.layer));
        for i in 0..sites.len() {
            for j in i + 1..sites.len() {
                let a = &sites[i];
                let b = &sites[j];
                if (a.kind == b.kind && a.index == b.index)
                    || (a.kind == 'V' && b.kind == 'V')
                    || a.net == b.net
                {
                    continue;
                }
                if conflicts.len() >= MAX_RECORDS {
                    return Err("CONTACT_WIRE_RECORD_LIMIT");
                }
                conflicts.push((a.clone(), b.clone()));
            }
        }
    }
    Ok(conflicts)
}

/// A strictly opt-in suffix AFTER the unchanged legacy END line. The caller
/// must not silently interpret these records using a V1-V14 decoder.
pub fn encode(board: &BoardModel) -> Result<Vec<u8>, &'static str> {
    if board.validate().has_errors() {
        return Err("CONTACT_WIRE_MODEL_INVALID");
    }
    if board.sources.len() != 1 || board.sources[0].id.files.len() != 1 {
        return Err("CONTACT_WIRE_SOURCE_UNKNOWN");
    }
    let mut objects = BTreeMap::new();
    for (i, t) in board.traces.iter().enumerate() {
        objects.insert(t.meta.id.clone(), ('T', i));
    }
    for (i, a) in board.arcs.iter().enumerate() {
        objects.insert(a.meta.id.clone(), ('A', i));
    }
    for (i, v) in board.vias.iter().enumerate() {
        objects.insert(v.meta.id.clone(), ('V', i));
    }
    let layers: BTreeMap<_, _> = board
        .stackup
        .layers
        .iter()
        .enumerate()
        .map(|(i, l)| (l.meta.id.clone(), i + 1))
        .collect();
    let nodes: BTreeMap<_, _> = board
        .connectivity
        .nodes
        .iter()
        .map(|n| (n.meta.id.clone(), n))
        .collect();
    if board.connectivity.edges.len() > MAX_RECORDS {
        return Err("CONTACT_WIRE_RECORD_LIMIT");
    }

    // Legacy V1 suffix is *endpoint-only*. New B_003 interior and Pad-centre
    // witnesses are kept in the Canonical graph, not misrepresented as V1
    // endpoint records. V1 wire must remain backward compatible.
    let legacy_edges: Vec<_> = board
        .connectivity
        .edges
        .iter()
        .filter(|edge| {
            [nodes.get(&edge.a), nodes.get(&edge.b)].iter().all(|node| {
                node.is_some_and(|node| {
                    objects.contains_key(&node.conductor)
                        && matches!(
                            node.site,
                            ContactSite::TraceVertex(_)
                                | ContactSite::ArcStart
                                | ContactSite::ArcEnd
                                | ContactSite::ViaCentre
                        )
                })
            })
        })
        .collect();
    let conflicts = potential_conflicts(board, &layers)?;
    let count = legacy_edges
        .len()
        .checked_add(conflicts.len())
        .ok_or("CONTACT_WIRE_RECORD_LIMIT")?;
    if count > MAX_RECORDS {
        return Err("CONTACT_WIRE_RECORD_LIMIT");
    }
    let mut out = String::new();
    let hash = &board.sources[0].id.files[0].sha256;
    write!(
        &mut out,
        "BOARDSI_CONTACT_RECORDS_V1|{}|LOCAL_ENDPOINT_ONLY|",
        count
    )
    .map_err(|_| "CONTACT_WIRE_FORMAT")?;
    for b in hash {
        write!(&mut out, "{b:02x}").map_err(|_| "CONTACT_WIRE_FORMAT")?;
    }
    out.push('\n');
    for edge in legacy_edges {
        let a = resolve(
            board,
            &objects,
            &layers,
            nodes.get(&edge.a).ok_or("CONTACT_WIRE_NODE_MISSING")?,
        )?;
        let b = resolve(
            board,
            &objects,
            &layers,
            nodes.get(&edge.b).ok_or("CONTACT_WIRE_NODE_MISSING")?,
        )?;
        if a.net != b.net || a.ticks != b.ticks {
            return Err("CONTACT_WIRE_GEOMETRY_MISMATCH");
        }
        let class = match &edge.evidence {
            ConnectionEvidence::SameLayerContact { .. } => {
                if a.layer != b.layer || (a.kind == 'V' && b.kind == 'V') {
                    return Err("CONTACT_WIRE_LAYER_MISMATCH");
                }
                'S'
            }
            ConnectionEvidence::ViaBarrel { via } => {
                if a.layer == b.layer
                    || a.kind != 'V'
                    || b.kind != 'V'
                    || a.index != b.index
                    || board.vias[a.index].meta.id != *via
                {
                    return Err("CONTACT_WIRE_BARREL_MISMATCH");
                }
                'B'
            }
        };
        let x = a.ticks[0]
            .checked_mul(NM_PER_TICK)
            .ok_or("CONTACT_WIRE_OVERFLOW")?;
        let y = a.ticks[1]
            .checked_mul(NM_PER_TICK)
            .ok_or("CONTACT_WIRE_OVERFLOW")?;
        writeln!(
            &mut out,
            "E|{class}|{}|{}|{}|{}|{}|{}|{}|{}|{x}|{y}",
            a.kind, a.index, a.site, a.layer, b.kind, b.index, b.site, b.layer
        )
        .map_err(|_| "CONTACT_WIRE_FORMAT")?;
        if out.len() > MAX_RECORD_BYTES {
            return Err("CONTACT_WIRE_OUTPUT_LIMIT");
        }
    }
    for (a, b) in conflicts {
        if a.layer != b.layer || a.ticks != b.ticks || a.net == b.net {
            return Err("CONTACT_WIRE_CONFLICT_MISMATCH");
        }
        let x = a.ticks[0]
            .checked_mul(NM_PER_TICK)
            .ok_or("CONTACT_WIRE_OVERFLOW")?;
        let y = a.ticks[1]
            .checked_mul(NM_PER_TICK)
            .ok_or("CONTACT_WIRE_OVERFLOW")?;
        writeln!(
            &mut out,
            "X|C|{}|{}|{}|{}|{}|{}|{}|{}|{x}|{y}",
            a.kind, a.index, a.site, a.layer, b.kind, b.index, b.site, b.layer
        )
        .map_err(|_| "CONTACT_WIRE_FORMAT")?;
        if out.len() > MAX_RECORD_BYTES {
            return Err("CONTACT_WIRE_OUTPUT_LIMIT");
        }
    }
    writeln!(&mut out, "BOARDSI_CONTACT_END_V1|{}", count).map_err(|_| "CONTACT_WIRE_FORMAT")?;
    if out.len() > MAX_RECORD_BYTES {
        return Err("CONTACT_WIRE_OUTPUT_LIMIT");
    }
    Ok(out.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route_preview::{into_canonical, parse};
    const TWO: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n300 200 1 10 0\nU2.2 U3.3\n300 200 1 10 0\n300 400 1 10 0\n*END*\n";
    #[test]
    fn opt_in_rows_are_verified_endpoint_edges() {
        let parsed = parse(TWO.as_bytes()).unwrap();
        let board = into_canonical(TWO.as_bytes(), &parsed).unwrap();
        let encoded = String::from_utf8(encode(&board).unwrap()).unwrap();
        assert!(encoded.starts_with("BOARDSI_CONTACT_RECORDS_V1|1|LOCAL_ENDPOINT_ONLY|"));
        // Stable Canonical node ID sorting need not preserve input segment order.
        // Check the exact edge class/location and two original conductors,
        // without assuming which end is emitted first.
        let rows: Vec<_> = encoded.lines().filter(|r| r.starts_with("E|S|")).collect();
        assert_eq!(rows.len(), 1);
        assert!(rows[0].contains("|7620000|5080000"));
        assert!(rows[0].contains("|T|0|1|1|") || rows[0].contains("|T|0|0|1|"));
        assert!(rows[0].contains("|T|1|0|1|") || rows[0].contains("|T|1|1|1|"));
        assert!(encoded.ends_with("BOARDSI_CONTACT_END_V1|1\n"));
    }
    #[test]
    fn cross_net_conflict_is_location_only_never_a_short_edge() {
        let with_other_net = TWO.replace(
            "U2.2 U3.3\n300 200 1 10 0",
            "*SIGNAL* NET_B\nU2.2 U3.3\n300 200 1 10 0",
        );
        let parsed = parse(with_other_net.as_bytes()).unwrap();
        let board = into_canonical(with_other_net.as_bytes(), &parsed).unwrap();
        assert!(board.connectivity.edges.is_empty());
        let wire = String::from_utf8(encode(&board).unwrap()).unwrap();
        assert!(wire.contains("X|C|T|0|1|1|T|1|0|1|"));
        assert!(!wire.contains("E|S|"));
    }
    #[test]
    fn output_is_deterministic_for_same_source() {
        let parsed = parse(TWO.as_bytes()).unwrap();
        let board1 = into_canonical(TWO.as_bytes(), &parsed).unwrap();
        let board2 = into_canonical(TWO.as_bytes(), &parsed).unwrap();
        assert_eq!(encode(&board1).unwrap(), encode(&board2).unwrap());
    }
}
