//! C_012: exact Source Signal Via / Reference Stitch Via circular pad separation.
//! Source geometry ONLY; never manufactured clearance, P009, electrical SI or solver readiness.
use crate::{reference_route_v7, route_preview};
use boardsi_model::{BoardModel, PadRole, PadShape, Rational, ViaKind};
use std::fmt::Write;

const MAX_WIRE: usize = 220_000;

fn verify_round_pad(
    board: &BoardModel,
    source: &route_preview::ViaInstance,
    index: usize,
    layer_index: usize,
) -> Result<i64, &'static str> {
    let via = board.vias.get(index).ok_or("V8_CANONICAL_VIA_MISSING")?;
    if via.kind != ViaKind::Through
        || via.position.ticks != source.point
        || source.first_layer as usize > layer_index
        || (source.last_layer as usize) < layer_index
        || source.diameter_ticks <= source.drill_ticks
        || source.drill_ticks <= 0
    {
        return Err("V8_VIA_SOURCE_OR_SPAN_MISMATCH");
    }
    let net = board
        .nets
        .iter()
        .find(|n| n.name == source.net)
        .ok_or("V8_NET_MISSING")?;
    if via.net.as_ref() != Some(&net.meta.id) {
        return Err("V8_CANONICAL_NET_MISMATCH");
    }
    let drill = board
        .drills
        .iter()
        .find(|d| d.meta.id == via.drill)
        .ok_or("V8_DRILL_MISSING")?;
    let expected_drill = Rational::new(i128::from(source.drill_ticks) * 127, 500_000_000)
        .map_err(|_| "V8_DRILL_UNIT")?;
    if !drill.plated || drill.span != via.span || drill.diameter_m != expected_drill {
        return Err("V8_UNPLATED_OR_DRILL_MISMATCH");
    }
    let padstack = board
        .padstacks
        .iter()
        .find(|p| p.meta.id == via.padstack)
        .ok_or("V8_PADSTACK_MISSING")?;
    if padstack
        .entries
        .iter()
        .any(|e| e.role != PadRole::Conductive)
    {
        return Err("V8_UNSUPPORTED_PAD_ROLE");
    }
    let layer = board
        .stackup
        .layers
        .get(layer_index.checked_sub(1).ok_or("V8_LAYER_INVALID")?)
        .ok_or("V8_LAYER_INVALID")?;
    // C_012 inspects the explicitly described metal Pad, not a proven
    // electrical contact. Canonical connected_layers only contains verified
    // local contacts and must NOT be required or fabricated for inner pads.
    let pads: Vec<_> = padstack
        .entries
        .iter()
        .filter(|p| p.layer == layer.meta.id)
        .collect();
    let expected_diameter = Rational::new(i128::from(source.diameter_ticks) * 127, 500_000_000)
        .map_err(|_| "V8_PAD_UNIT")?;
    if pads.len() != 1
        || !matches!(&pads[0].shape,
        PadShape::Circle { diameter_m } if *diameter_m == expected_diameter)
    {
        return Err("V8_UNSUPPORTED_ROUND_PAD");
    }
    Ok(source.diameter_ticks)
}

/// Additive V8 receipt embeds byte-exact V7, including V6/V5/V4/V2.
pub fn encode(
    bytes: &[u8],
    indices: &[usize],
    split: usize,
    references: [&str; 2],
    reference_layers: [usize; 2],
    reference_via_index: usize,
) -> Result<String, &'static str> {
    let v7 = reference_route_v7::encode(
        bytes,
        indices,
        split,
        references,
        reference_layers,
        reference_via_index,
    )?;
    let v7_fields: Vec<_> = v7.lines().next().ok_or("V7_EMPTY")?.split('|').collect();
    if v7_fields.len() != 11
        || v7_fields[0] != "BOARDSI_REFERENCE_ROUTE_V7"
        || v7_fields[1].len() != 64
    {
        return Err("V8_V7_HEADER_MISMATCH");
    }
    let parsed = route_preview::parse(bytes)?;
    let board = route_preview::into_canonical(bytes, &parsed)?;
    if board.validate().has_errors() || board.vias.len() != parsed.vias.len() {
        return Err("V8_CANONICAL_INVALID");
    }
    // V7 independently verifies a supported consecutive two-layer signal route.
    // Do not guess by proximity or use an arbitrary same-Net via.
    let before = parsed
        .segments
        .get(
            *indices
                .get(split.checked_sub(1).ok_or("V8_SPLIT")?)
                .ok_or("V8_SPLIT")?,
        )
        .ok_or("V8_SIGNAL_SEGMENT")?;
    let after = parsed
        .segments
        .get(*indices.get(split).ok_or("V8_SPLIT")?)
        .ok_or("V8_SIGNAL_SEGMENT")?;
    if before.b != after.a
        || before.net != after.net
        || before.net.is_empty()
        || before.net == references[0]
    {
        return Err("V8_UNSUPPORTED_SIGNAL_TRANSITION");
    }
    let matching: Vec<_> = parsed
        .vias
        .iter()
        .enumerate()
        .filter(|(_, v)| {
            v.net == before.net
                && v.point == before.b
                && v.first_layer <= before.layer
                && v.last_layer >= before.layer
                && v.first_layer <= after.layer
                && v.last_layer >= after.layer
        })
        .collect();
    if matching.len() != 1 {
        return Err("V8_SIGNAL_VIA_NOT_UNIQUE");
    }
    let (signal_index, signal) = matching[0];
    let reference = parsed
        .vias
        .get(reference_via_index)
        .ok_or("V8_REFERENCE_VIA_MISSING")?;
    if signal_index == reference_via_index
        || reference.net != references[0]
        || reference.net == signal.net
    {
        return Err("V8_NOT_DIFFERENT_NET_VIAS");
    }
    let mut overall = "SOURCE_PAD_SEPARATE_OBSERVED";
    if v7_fields[2] != "PAD_FOOTPRINT_CONTAINED_CANDIDATE" {
        overall = "V7_REFERENCE_FOOTPRINT_RISK";
    }
    let mut rows = String::new();
    for &layer in &reference_layers {
        let signal_d = verify_round_pad(&board, signal, signal_index, layer)?;
        let reference_d = verify_round_pad(&board, reference, reference_via_index, layer)?;
        let dx = i128::from(signal.point[0]) - i128::from(reference.point[0]);
        let dy = i128::from(signal.point[1]) - i128::from(reference.point[1]);
        let squared = dx
            .checked_mul(dx)
            .and_then(|x| dy.checked_mul(dy).and_then(|y| x.checked_add(y)))
            .ok_or("V8_DISTANCE_OVERFLOW")?;
        let four_dist = squared.checked_mul(4).ok_or("V8_DISTANCE_OVERFLOW")?;
        let diameter_sum = i128::from(signal_d)
            .checked_add(i128::from(reference_d))
            .ok_or("V8_DIAMETER_OVERFLOW")?;
        let diameter_sum_squared = diameter_sum
            .checked_mul(diameter_sum)
            .ok_or("V8_DIAMETER_OVERFLOW")?;
        let state = match four_dist.cmp(&diameter_sum_squared) {
            std::cmp::Ordering::Less => "SOURCE_PAD_OVERLAP_RISK",
            std::cmp::Ordering::Equal => "SOURCE_PAD_TANGENT_RISK",
            std::cmp::Ordering::Greater => "SOURCE_PAD_SEPARATE_OBSERVED",
        };
        if overall == "SOURCE_PAD_SEPARATE_OBSERVED" && state != "SOURCE_PAD_SEPARATE_OBSERVED" {
            overall = state;
        }
        writeln!(&mut rows,
            "CLEARANCE|{layer}|{signal_index}|{reference_via_index}|{}|{}|{}|{}|{signal_d}|{reference_d}|{}|{}|{state}",
            signal.point[0], signal.point[1], reference.point[0], reference.point[1],
            signal.drill_ticks, reference.drill_ticks
        ).map_err(|_| "V8_FORMAT")?;
    }
    let mut wire = format!(
        "BOARDSI_REFERENCE_ROUTE_V8|{}|{overall}|UNSUPPORTED|NOT_RUN|0|REFERENCE_TRANSFER_UNVERIFIED|2|{}|{split}|{reference_via_index}|{signal_index}|{}\n{rows}V7|{}\n",
        v7_fields[1], indices.len(), v7_fields[2], v7.lines().count()
    );
    wire.push_str(&v7);
    wire.push_str("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n");
    if wire.len() > MAX_WIRE {
        return Err("V8_OUTPUT_LIMIT");
    }
    Ok(wire)
}

#[cfg(test)]
mod tests {
    #[test]
    fn exact_separation_uses_integer_comparison() {
        let judge = |dx: i128, dy: i128, sum_diameter: i128| {
            4 * (dx * dx + dy * dy) - sum_diameter * sum_diameter
        };
        assert!(judge(3501, 0, 7000) > 0);
        assert_eq!(judge(3500, 0, 7000), 0);
        assert!(judge(3499, 0, 7000) < 0);
    }
}
