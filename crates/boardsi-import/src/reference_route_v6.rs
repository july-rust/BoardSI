//! C_010: source-exact corridor-to-stitch Area identity; NEVER electrical Return Path.
//! V6 opt-in embeds an unchanged C_009 V5 (and its V4/V2) source receipt.
use crate::{copper_preview, reference_geometry, reference_route_v5, route_preview, via_copper_contact};
use std::fmt::Write;

const MAX_WIRE: usize = 185_000;
fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(&mut out, "{b:02x}").expect("hex");
    }
    out
}

/// Check whether each full selected signal leg is strictly supported by the
/// very same unique Source-owned Copper Area contacted by the Reference Via.
/// An Area ID/Net match alone is NEVER proof of Return Current or Solver readiness.
pub fn encode(
    bytes: &[u8],
    indices: &[usize],
    split: usize,
    references: [&str; 2],
    reference_layers: [usize; 2],
    reference_via_index: usize,
) -> Result<String, &'static str> {
    let v5 = reference_route_v5::encode(
        bytes, indices, split, references, reference_layers, reference_via_index,
    )?;
    if indices.len() < 2 || indices.len() > 16 || split == 0 || split >= indices.len() {
        return Err("INVALID_V6_ROUTE");
    }
    let parsed = route_preview::parse(bytes)?;
    let mut board = route_preview::into_canonical(bytes, &parsed)?;
    let source_areas = copper_preview::parse(bytes, parsed.max_layer)?;
    copper_preview::augment(&mut board, &source_areas)?;
    // Source COPPER labels remain unverified until strict ViaCentre/AreaInterior
    // graph witnesses establish bounded candidate Net association (C_009).
    via_copper_contact::augment_multi(&mut board, bytes)?;
    if board.validate().has_errors() || source_areas.len() != board.areas.len() {
        return Err("SOURCE_GEOMETRY_UNSUPPORTED");
    }
    let _ref_net = board.nets.iter().find(|n| n.name == references[0])
        .ok_or("REFERENCE_NET_UNKNOWN")?;
    let v5_header = v5.lines().next().ok_or("V5_HEADER_MISSING")?;
    let header: Vec<_> = v5_header.split('|').collect();
    if header.len() != 14 || header[0] != "BOARDSI_REFERENCE_ROUTE_V5"
        || header[1].len() != 64 {
        return Err("V5_SOURCE_MISMATCH");
    }
    let mut corridors = Vec::new();
    let mut overall = "SAME_SOURCE_AREA_CORRIDOR_CANDIDATE";
    for leg in 0..2 {
        let layer_number = reference_layers[leg];
        let layer = board.stackup.layers.get(
            layer_number.checked_sub(1).ok_or("REFERENCE_LAYER_INVALID")?
        ).ok_or("REFERENCE_LAYER_INVALID")?;
        let slice = if leg == 0 { &indices[..split] } else { &indices[split..] };
        let source_contact = v5.lines().nth(1 + leg).ok_or("V5_CONTACT_MISSING")?;
        let fields: Vec<_> = source_contact.split('|').collect();
        if fields.len() != 3 || fields[0] != "CONTACT"
            || fields[1] != layer_number.to_string() {
            return Err("V5_CONTACT_INVALID");
        }
        let mut selected = Vec::new();
        for &i in slice {
            let raw = parsed.segments.get(i).ok_or("TRACE_INDEX_INVALID")?;
            let trace = board.traces.get(i).ok_or("TRACE_NOT_CANONICAL")?;
            let signal_layer = board.stackup.layers.get(
                (raw.layer as usize).checked_sub(1).ok_or("SIGNAL_LAYER_INVALID")?
            ).ok_or("SIGNAL_LAYER_INVALID")?;
            if trace.layer != signal_layer.meta.id
                || trace.centreline.len() != 2
                || trace.centreline[0].ticks != raw.a
                || trace.centreline[1].ticks != raw.b
                || signal_layer.order.abs_diff(layer.order) != 1 {
                return Err("UNSUPPORTED_CORRIDOR_LAYER_OR_SOURCE");
            }
            selected.push(trace);
        }
        // An uncontacted Source Area intentionally has Canonical net=None.
        // Its original declared Net label is only a *candidate* for corridor
        // identity; never promote it to an electrical connection.
        let candidates: Vec<_> = board.areas.iter().enumerate().filter(|(i,a)| {
            a.layer == layer.meta.id
                && source_areas.get(*i)
                    .and_then(|raw| raw.net_label.as_deref()) == Some(references[leg])
                && selected.iter().all(|t| {
                    reference_geometry::segment_strictly_inside(
                        a, &t.centreline[0], &t.centreline[1]
                    )
                })
        }).map(|(_,a)|a).collect();
        let (area_hex, state) = match candidates.as_slice() {
            [] => ("-".to_string(), "REFERENCE_CORRIDOR_GAP"),
            [area] => {
                let id = hex(area.meta.id.source_object.as_bytes());
                let state = if id == fields[2] {
                    "SAME_SOURCE_AREA"
                } else {
                    "REFERENCE_AREA_MISMATCH"
                };
                (id, state)
            }
            _ => ("-".to_string(), "SOURCE_GEOMETRY_UNSUPPORTED"),
        };
        if state != "SAME_SOURCE_AREA" && overall == "SAME_SOURCE_AREA_CORRIDOR_CANDIDATE" {
            overall = state;
        }
        corridors.push((layer_number, area_hex, fields[2].to_owned(), state));
    }
    let mut out = format!(
        "BOARDSI_REFERENCE_ROUTE_V6|{}|{overall}|UNSUPPORTED|NOT_RUN|0|REFERENCE_TRANSFER_UNVERIFIED|2|{}|{split}|{reference_via_index}\n",
        header[1], indices.len()
    );
    for (layer, area, contact, state) in corridors {
        writeln!(&mut out, "CORRIDOR|{layer}|{area}|{contact}|{state}")
            .map_err(|_| "V6_FORMAT")?;
    }
    writeln!(&mut out, "V5|{}", v5.lines().count()).map_err(|_| "V6_FORMAT")?;
    out.push_str(&v5);
    out.push_str("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n");
    if out.len() > MAX_WIRE {
        return Err("V6_OUTPUT_LIMIT");
    }
    Ok(out)
}
