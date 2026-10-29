//! C_009 opt-in exact-source Reference stitching via *candidate* witness.
//! Source-local copper contact only: no Return Current, P009 or SI qualification.
use crate::{copper_preview, reference_route_v4, route_preview, via_copper_contact};
use boardsi_model::{ConnectionEvidence, ContactSite, PadRole, ViaKind};
use std::collections::BTreeSet;
use std::fmt::Write;

const MAX_V5_BYTES: usize = 170_000;
fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("string");
    }
    out
}
fn nm(tick: i64) -> Result<i64, &'static str> {
    tick.checked_mul(254).ok_or("COORD_OVERFLOW")
}

/// Explicit Reference Via index is the *source route parser's* zero-based via index.
/// All source provenance is recomputed from the original bytes; no inferred continuity.
pub fn encode(
    bytes: &[u8],
    indices: &[usize],
    split: usize,
    references: [&str; 2],
    reference_layers: [usize; 2],
    reference_via_index: usize,
) -> Result<String, &'static str> {
    if references[0].is_empty() || references[0] != references[1] ||
        reference_layers[0] == reference_layers[1] {
        return Err("REFERENCE_NET_OR_LAYER_MISMATCH");
    }
    let v4 = reference_route_v4::encode(
        bytes, indices, split, references, reference_layers
    )?;
    let v4_head = v4.lines().next().ok_or("V4_MISSING")?;
    let v4_fields: Vec<_> = v4_head.split('|').collect();
    if v4_fields.len() != 9 ||
        v4_fields[0] != "BOARDSI_REFERENCE_ROUTE_V4" {
        return Err("V4_INVALID");
    }
    let parsed = route_preview::parse(bytes)?;
    let mut board = route_preview::into_canonical(bytes, &parsed)?;
    let source_copper = copper_preview::parse(bytes, parsed.max_layer)?;
    copper_preview::augment(&mut board, &source_copper)?;
    // The B_006/C_001 graph emitter independently reparses the original and
    // only emits a SameLayerContact for strict supported COPCLS interiors.
    via_copper_contact::augment_multi(&mut board, bytes)?;
    if board.validate().has_errors() || board.vias.len() != parsed.vias.len() {
        return Err("INVALID_CONTACT_GRAPH");
    }
    let candidate = parsed.vias.get(reference_via_index)
        .ok_or("UNKNOWN_REFERENCE_VIA")?;
    let via = board.vias.get(reference_via_index)
        .ok_or("CANONICAL_REFERENCE_VIA_MISSING")?;
    if via.kind != ViaKind::Through ||
        via.position.ticks != candidate.point ||
        via.net.is_none() ||
        candidate.net != references[0] ||
        parsed.segments.get(indices[0]).map_or(true, |s| s.net == candidate.net) {
        return Err("REFERENCE_VIA_SOURCE_MISMATCH");
    }
    let reference_net = board.nets.iter()
        .find(|n| n.name == references[0])
        .ok_or("REFERENCE_NET_NOT_FOUND")?;
    if via.net.as_ref() != Some(&reference_net.meta.id) {
        return Err("REFERENCE_CANONICAL_NET_MISMATCH");
    }
    let drill = board.drills.iter()
        .find(|d| d.meta.id == via.drill)
        .ok_or("REFERENCE_DRILL_UNKNOWN")?;
    let padstack = board.padstacks.iter()
        .find(|p| p.meta.id == via.padstack)
        .ok_or("REFERENCE_PADSTACK_UNKNOWN")?;
    if !drill.plated || drill.span != via.span ||
        padstack.entries.iter().any(|p| p.role != PadRole::Conductive) {
        return Err("REFERENCE_VIA_NOT_PLATED");
    }
    let mut contacts = Vec::new();
    let mut areas = BTreeSet::new();
    for &layer_number in &reference_layers {
        let layer = board.stackup.layers.get(layer_number.checked_sub(1)
            .ok_or("INVALID_REFERENCE_LAYER")?)
            .ok_or("INVALID_REFERENCE_LAYER")?;
        if !via.connected_layers.contains(&layer.meta.id) ||
            !padstack.entries.iter().any(|e|
                e.layer == layer.meta.id && e.role == PadRole::Conductive) {
            return Err("REFERENCE_VIA_LAYER_NOT_CONDUCTIVE");
        }
        // Only an actual ViaCentre -> AreaInterior SameLayerContact edge,
        // grounded to the same Source Via/Net/Layer, qualifies as evidence.
        let witnesses: Vec<_> = board.connectivity.edges.iter().filter_map(|edge| {
            if !matches!(edge.evidence, ConnectionEvidence::SameLayerContact { .. }) {
                return None;
            }
            let a = board.connectivity.nodes.iter()
                .find(|n| n.meta.id == edge.a)?;
            let b = board.connectivity.nodes.iter()
                .find(|n| n.meta.id == edge.b)?;
            for (via_node, area_node) in [(a, b), (b, a)] {
                if via_node.conductor != via.meta.id ||
                    via_node.layer != layer.meta.id ||
                    !matches!(via_node.site, ContactSite::ViaCentre) ||
                    area_node.layer != layer.meta.id ||
                    !matches!(area_node.site, ContactSite::AreaInterior(_)) {
                    continue;
                }
                let area = board.areas.iter()
                    .find(|area| area.meta.id == area_node.conductor)?;
                if area.net.as_ref() == Some(&reference_net.meta.id) {
                    return Some(area.meta.id.source_object.clone());
                }
            }
            None
        }).collect();
        if witnesses.len() != 1 || !areas.insert((layer_number, witnesses[0].clone())) {
            return Err("REFERENCE_CONTACT_NOT_UNIQUE_OR_UNSUPPORTED");
        }
        contacts.push((layer_number, witnesses[0].clone()));
    }
    // Exact integer tick arithmetic. The following mm is DISPLAY ONLY.
    let signal_via_line = v4.lines().nth(1).ok_or("SIGNAL_VIA_MISSING")?;
    let signal_via: Vec<_> = signal_via_line.split('|').collect();
    if signal_via.len() != 10 || signal_via[0] != "VIA" {
        return Err("SIGNAL_VIA_INVALID");
    }
    let sx: i64 = signal_via[4].parse().map_err(|_| "SIGNAL_X_INVALID")?;
    let sy: i64 = signal_via[5].parse().map_err(|_| "SIGNAL_Y_INVALID")?;
    if sx % 254 != 0 || sy % 254 != 0 {
        return Err("SIGNAL_COORD_GRID_MISMATCH");
    }
    let dx = i128::from(sx / 254) - i128::from(candidate.point[0]);
    let dy = i128::from(sy / 254) - i128::from(candidate.point[1]);
    let squared = dx.checked_mul(dx)
        .and_then(|x| dy.checked_mul(dy).and_then(|y| x.checked_add(y)))
        .ok_or("DISTANCE_OVERFLOW")?;
    if squared == 0 {
        return Err("COINCIDENT_SIGNAL_REFERENCE_VIA");
    }
    let approx_mm = (squared as f64).sqrt() * 0.000_254;
    let mut out = format!(
        "BOARDSI_REFERENCE_ROUTE_V5|{}|VERIFIED_SOURCE_STITCH_CANDIDATE|UNSUPPORTED|NOT_RUN|0|REFERENCE_TRANSFER_UNVERIFIED|{}|{}|{}|{}|{}|{squared}|{approx_mm:.6}\n",
        v4_fields[1], hex(candidate.net.as_bytes()), reference_via_index,
        hex(via.meta.id.source_object.as_bytes()),
        nm(candidate.point[0])?, nm(candidate.point[1])?
    );
    for (layer, area) in contacts {
        writeln!(&mut out, "CONTACT|{layer}|{}", hex(area.as_bytes()))
            .map_err(|_| "V5_FORMAT")?;
    }
    writeln!(&mut out, "V4|{}", v4.lines().count())
        .map_err(|_| "V5_FORMAT")?;
    out.push_str(&v4);
    out.push_str("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n");
    if out.len() > MAX_V5_BYTES {
        return Err("V5_OUTPUT_LIMIT");
    }
    Ok(out)
}
