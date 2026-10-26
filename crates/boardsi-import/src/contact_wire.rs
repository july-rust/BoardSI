//! A_032_2 opt-in source-linked local contact records for Qt.
//! Never certifies entire-board shorts/opens or areas/pins.
//! Default legacy artwork stdout V1-V14 is byte-for-byte unaffected.
use boardsi_model::{BoardModel, ConnectionEvidence, ConnectivityNode, ContactSite, ObjectId};
use std::collections::BTreeMap;
use std::fmt::Write as _;

const NM_PER_TICK: i64 = 254;
const MAX_RECORDS: usize = 100_000;
const MAX_RECORD_BYTES: usize = 1024 * 1024;

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
            if item.layer != node.layer { return Err("CONTACT_WIRE_ARC_MISMATCH"); }
            (0, item.start.ticks, item.net.as_ref())
        }
        ('A', ContactSite::ArcEnd) => {
            let item = &board.arcs[index];
            if item.layer != node.layer { return Err("CONTACT_WIRE_ARC_MISMATCH"); }
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
    Ok(Endpoint { kind, index, site, layer, ticks, net })
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
    let layers: BTreeMap<_, _> = board.stackup.layers.iter().enumerate()
        .map(|(i, l)| (l.meta.id.clone(), i + 1)).collect();
    let nodes: BTreeMap<_, _> = board.connectivity.nodes.iter()
        .map(|n| (n.meta.id.clone(), n)).collect();
    if board.connectivity.edges.len() > MAX_RECORDS {
        return Err("CONTACT_WIRE_RECORD_LIMIT");
    }

    let mut out = String::new();
    let hash = &board.sources[0].id.files[0].sha256;
    write!(&mut out, "BOARDSI_CONTACT_RECORDS_V1|{}|LOCAL_ENDPOINT_ONLY|",
        board.connectivity.edges.len()).map_err(|_| "CONTACT_WIRE_FORMAT")?;
    for b in hash { write!(&mut out, "{b:02x}").map_err(|_| "CONTACT_WIRE_FORMAT")?; }
    out.push('\n');
    for edge in &board.connectivity.edges {
        let a = resolve(board, &objects, &layers,
            nodes.get(&edge.a).ok_or("CONTACT_WIRE_NODE_MISSING")?)?;
        let b = resolve(board, &objects, &layers,
            nodes.get(&edge.b).ok_or("CONTACT_WIRE_NODE_MISSING")?)?;
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
                if a.layer == b.layer || a.kind != 'V' || b.kind != 'V'
                    || a.index != b.index || board.vias[a.index].meta.id != *via {
                    return Err("CONTACT_WIRE_BARREL_MISMATCH");
                }
                'B'
            }
        };
        let x = a.ticks[0].checked_mul(NM_PER_TICK).ok_or("CONTACT_WIRE_OVERFLOW")?;
        let y = a.ticks[1].checked_mul(NM_PER_TICK).ok_or("CONTACT_WIRE_OVERFLOW")?;
        writeln!(&mut out, "E|{class}|{}|{}|{}|{}|{}|{}|{}|{}|{x}|{y}",
            a.kind, a.index, a.site, a.layer,
            b.kind, b.index, b.site, b.layer)
            .map_err(|_| "CONTACT_WIRE_FORMAT")?;
        if out.len() > MAX_RECORD_BYTES {
            return Err("CONTACT_WIRE_OUTPUT_LIMIT");
        }
    }
    writeln!(&mut out, "BOARDSI_CONTACT_END_V1|{}",
        board.connectivity.edges.len()).map_err(|_| "CONTACT_WIRE_FORMAT")?;
    if out.len() > MAX_RECORD_BYTES {
        return Err("CONTACT_WIRE_OUTPUT_LIMIT");
    }
    Ok(out.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route_preview::{into_canonical, parse};
    const TWO: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n300 200 1 10 0\n300 400 1 10 0\n*END*\n";
    #[test]
    fn opt_in_rows_are_verified_endpoint_edges() {
        let parsed = parse(TWO.as_bytes()).unwrap();
        let board = into_canonical(TWO.as_bytes(), &parsed).unwrap();
        let encoded = String::from_utf8(encode(&board).unwrap()).unwrap();
        assert!(encoded.starts_with("BOARDSI_CONTACT_RECORDS_V1|1|LOCAL_ENDPOINT_ONLY|"));
        assert!(encoded.contains("E|S|T|0|1|1|T|1|0|1|"));
        assert!(encoded.ends_with("BOARDSI_CONTACT_END_V1|1\n"));
    }
    #[test]
    fn output_is_deterministic_for_same_source() {
        let parsed = parse(TWO.as_bytes()).unwrap();
        let board1 = into_canonical(TWO.as_bytes(), &parsed).unwrap();
        let board2 = into_canonical(TWO.as_bytes(), &parsed).unwrap();
        assert_eq!(encode(&board1).unwrap(), encode(&board2).unwrap());
    }
}
