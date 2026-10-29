//! C_011: exact source-grid round Reference Stitch Via pad-footprint containment.
//! A geometric candidate only. Never proof of fabricated copper, P009 or SI.
use crate::{copper_preview, reference_route_v6, route_preview};
use boardsi_model::{PadRole, PadShape, Rational, ViaKind};
use std::fmt::Write;

const MAX_V7_BYTES: usize = 200_000;

fn hex(input: &str) -> String {
    let mut text = String::with_capacity(input.len() * 2);
    for value in input.as_bytes() {
        write!(&mut text, "{value:02x}").expect("hex formatter");
    }
    text
}

/// Strict circle/closed polygon separation, including an exact boundary touch.
/// PADS V9.4 coordinates and diameters are centimil integer ticks. Integer
/// cross products are compared against four times the squared circle radius;
/// no sqrt, f32, f64 or rounded display coordinate is used for acceptance.
fn circle_strictly_inside_edges(
    centre: [i64; 2],
    diameter: i64,
    ring: &[[i64; 2]],
) -> Result<bool, &'static str> {
    if diameter <= 0 || ring.len() < 4 || ring.first() != ring.last() {
        return Err("UNSUPPORTED_V7_RING");
    }
    let radius4 = i128::from(diameter)
        .checked_mul(i128::from(diameter))
        .ok_or("V7_OVERFLOW")?;
    for edge in ring.windows(2) {
        let ax = i128::from(edge[0][0]);
        let ay = i128::from(edge[0][1]);
        let dx = i128::from(edge[1][0]) - ax;
        let dy = i128::from(edge[1][1]) - ay;
        let px = i128::from(centre[0]) - ax;
        let py = i128::from(centre[1]) - ay;
        let len2 = dx * dx + dy * dy;
        if len2 == 0 {
            return Err("UNSUPPORTED_V7_ZERO_EDGE");
        }
        let dot = px * dx + py * dy;
        let clear = if dot <= 0 {
            4 * (px * px + py * py) > radius4
        } else if dot >= len2 {
            let qx = px - dx;
            let qy = py - dy;
            4 * (qx * qx + qy * qy) > radius4
        } else {
            let cross = px * dy - py * dx;
            4 * cross * cross > radius4 * len2
        };
        if !clear {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn encode(
    bytes: &[u8],
    indices: &[usize],
    split: usize,
    references: [&str; 2],
    reference_layers: [usize; 2],
    reference_via_index: usize,
) -> Result<String, &'static str> {
    // The existing V6 source, exact Area identity, via centre and legacy
    // source receipts are independently re-evaluated, not supplied by caller.
    let v6 = reference_route_v6::encode(
        bytes,
        indices,
        split,
        references,
        reference_layers,
        reference_via_index,
    )?;
    let h: Vec<_> = v6
        .lines()
        .next()
        .ok_or("V6_HEADER_MISSING")?
        .split('|')
        .collect();
    if h.len() != 11 || h[0] != "BOARDSI_REFERENCE_ROUTE_V6" || h[1].len() != 64 {
        return Err("V6_HEADER_INVALID");
    }

    let parsed = route_preview::parse(bytes)?;
    let mut board = route_preview::into_canonical(bytes, &parsed)?;
    let copper = copper_preview::parse(bytes, parsed.max_layer)?;
    copper_preview::augment(&mut board, &copper)?;
    if board.validate().has_errors() || board.areas.len() != copper.len() {
        return Err("V7_SOURCE_UNSUPPORTED");
    }

    let source_via = parsed.vias.get(reference_via_index).ok_or("V7_VIA_INDEX")?;
    let via = board
        .vias
        .get(reference_via_index)
        .ok_or("V7_CANONICAL_VIA")?;
    if via.kind != ViaKind::Through
        || via.position.ticks != source_via.point
        || source_via.net != references[0]
        || source_via.diameter_ticks <= source_via.drill_ticks
        || source_via.drill_ticks <= 0
    {
        return Err("V7_SOURCE_PAD_MISMATCH");
    }
    let padstack = board
        .padstacks
        .iter()
        .find(|p| p.meta.id == via.padstack)
        .ok_or("V7_PADSTACK_UNSUPPORTED")?;
    let drill = board
        .drills
        .iter()
        .find(|d| d.meta.id == via.drill)
        .ok_or("V7_DRILL_UNSUPPORTED")?;
    if !drill.plated || drill.span != via.span {
        return Err("V7_DRILL_NOT_PLATED");
    }
    let expected_diameter = Rational::new(i128::from(source_via.diameter_ticks) * 127, 500_000_000)
        .map_err(|_| "V7_PAD_DIMENSION")?;
    let expected_drill = Rational::new(i128::from(source_via.drill_ticks) * 127, 500_000_000)
        .map_err(|_| "V7_DRILL_DIMENSION")?;
    if drill.diameter_m != expected_drill
        || padstack
            .entries
            .iter()
            .any(|e| e.role != PadRole::Conductive)
    {
        return Err("V7_UNSUPPORTED_PAD_ROLE");
    }

    let mut rows = Vec::with_capacity(2);
    let mut overall = "PAD_FOOTPRINT_CONTAINED_CANDIDATE";
    for (leg, layer_number) in reference_layers.iter().copied().enumerate() {
        let corridor: Vec<_> = v6
            .lines()
            .nth(1 + leg)
            .ok_or("V7_CORRIDOR_MISSING")?
            .split('|')
            .collect();
        if corridor.len() != 5
            || corridor[0] != "CORRIDOR"
            || corridor[1] != layer_number.to_string()
        {
            return Err("V7_CORRIDOR_INVALID");
        }
        let layer = board
            .stackup
            .layers
            .get(layer_number.checked_sub(1).ok_or("V7_LAYER")?)
            .ok_or("V7_LAYER")?;
        let pads: Vec<_> = padstack
            .entries
            .iter()
            .filter(|entry| entry.layer == layer.meta.id)
            .collect();
        if pads.len() != 1
            || !matches!(&pads[0].shape,
            PadShape::Circle { diameter_m } if *diameter_m == expected_diameter)
        {
            return Err("V7_ROUND_PAD_UNSUPPORTED");
        }
        let mut state = "PAD_FOOTPRINT_CONTAINED_CANDIDATE";
        if corridor[4] != "SAME_SOURCE_AREA" || corridor[2] != corridor[3] {
            state = "V6_CORRIDOR_RISK";
        } else {
            let matches: Vec<_> = board
                .areas
                .iter()
                .enumerate()
                .filter(|(_, area)| {
                    area.layer == layer.meta.id && hex(&area.meta.id.source_object) == corridor[2]
                })
                .map(|(i, _)| i)
                .collect();
            if matches.len() != 1 {
                return Err("V7_SOURCE_AREA_AMBIGUOUS");
            }
            let source_area = &copper[matches[0]];
            let copper_preview::CopperGeometry::Polygon(outer) = &source_area.geometry else {
                return Err("V7_OUTER_UNSUPPORTED");
            };
            if !circle_strictly_inside_edges(source_via.point, source_via.diameter_ticks, outer)? {
                state = "PAD_BOUNDARY_RISK";
            }
            for cutout in &source_area.cutouts {
                let copper_preview::CopperGeometry::Polygon(hole) = cutout else {
                    return Err("V7_CUTOUT_UNSUPPORTED");
                };
                if !circle_strictly_inside_edges(source_via.point, source_via.diameter_ticks, hole)?
                    && state == "PAD_FOOTPRINT_CONTAINED_CANDIDATE"
                {
                    state = "PAD_CUTOUT_RISK";
                }
            }
        }
        if overall == "PAD_FOOTPRINT_CONTAINED_CANDIDATE"
            && state != "PAD_FOOTPRINT_CONTAINED_CANDIDATE"
        {
            overall = state;
        }
        rows.push((layer_number, corridor[3].to_string(), state));
    }
    let mut out = format!(
        "BOARDSI_REFERENCE_ROUTE_V7|{}|{}|UNSUPPORTED|NOT_RUN|0|REFERENCE_TRANSFER_UNVERIFIED|2|{}|{}|{}\n",
        h[1], overall, indices.len(), split, reference_via_index
    );
    for (layer, area, state) in rows {
        writeln!(
            &mut out,
            "PAD|{layer}|{area}|{}|{}|{state}",
            source_via.diameter_ticks, source_via.drill_ticks
        )
        .map_err(|_| "V7_FORMAT")?;
    }
    writeln!(&mut out, "V6|{}", v6.lines().count()).map_err(|_| "V7_FORMAT")?;
    out.push_str(&v6);
    out.push_str("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n");
    if out.len() > MAX_V7_BYTES {
        return Err("V7_OUTPUT_LIMIT");
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_tangent_and_half_tick_radius_are_not_fuzzy() {
        let rectangle = [[0, 0], [100, 0], [100, 100], [0, 100], [0, 0]];
        assert_eq!(
            circle_strictly_inside_edges([50, 50], 99, &rectangle),
            Ok(true)
        );
        assert_eq!(
            circle_strictly_inside_edges([50, 50], 100, &rectangle),
            Ok(false)
        );
        assert_eq!(
            circle_strictly_inside_edges([50, 50], 101, &rectangle),
            Ok(false)
        );
        assert_eq!(
            circle_strictly_inside_edges([50, 50], 1, &rectangle),
            Ok(true)
        );
    }
}
