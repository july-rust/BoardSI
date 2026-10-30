//! C_015 experimental opt-in mixed COPCUT/COPCCO source-only V11.
//! V8 legacy is included ONLY when genuinely produced from original bytes.
//! Circular inputs use a genuine original-source V4 witness, NOT fake V8.
//! Return current, electrical connectivity, DRC and physics remain UNSUPPORTED.
pub mod geometry;

use crate::{copper_preview, reference_route_v4, reference_route_v8, route_preview};
use copper_preview::CopperGeometry;
use geometry::{PadRisk, SourceHole};
use std::fmt::Write;

const MAX_WIRE: usize = 280_000;

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("hex formatting");
    }
    out
}

/// Source geometry only. On a circle, it is intentionally impossible to
/// claim the historic V8/Stitch-contact receipt until V5-V7 truly support it.
pub fn encode(
    bytes: &[u8], indices: &[usize], split: usize, refs: [&str; 2],
    layers: [usize; 2], stitch_index: usize,
) -> Result<String, &'static str> {
    if indices.len() < 2 || indices.len() > 16 || split == 0 || split >= indices.len()
        || refs[0].is_empty() || refs[0] != refs[1] || layers[0] == layers[1]
    {
        return Err("V11_INVALID_SELECTION");
    }
    let v4 = reference_route_v4::encode(bytes, indices, split, refs, layers)?;
    let v4lines: Vec<_> = v4.lines().collect();
    let header: Vec<_> = v4lines.first().ok_or("V11_V4_EMPTY")?.split('|').collect();
    if header.len() != 9 || header[0] != "BOARDSI_REFERENCE_ROUTE_V4"
        || header[1].len() != 64 || header[2] != indices.len().to_string()
        || header[3] != split.to_string()
    {
        return Err("V11_V4_MISMATCH");
    }
    let via_fields: Vec<_> = v4lines.get(1).ok_or("V11_V4_VIA")?.split('|').collect();
    if via_fields.len() != 10 || via_fields[0] != "VIA" {
        return Err("V11_V4_VIA");
    }
    let signal_index: usize = via_fields[1].parse().map_err(|_| "V11_SIGNAL_INDEX")?;
    let parsed = route_preview::parse(bytes)?;
    let mut board = route_preview::into_canonical(bytes, &parsed)?;
    let areas = copper_preview::parse(bytes, parsed.max_layer)?;
    copper_preview::augment(&mut board, &areas)?;
    if board.validate().has_errors() || board.areas.len() != areas.len() {
        return Err("V11_CANONICAL_INVALID");
    }
    let signal = parsed.vias.get(signal_index).ok_or("V11_SIGNAL_MISSING")?;
    let stitch = parsed.vias.get(stitch_index).ok_or("V11_STITCH_MISSING")?;
    if signal_index == stitch_index || signal.net.is_empty()
        || signal.net == refs[0] || stitch.net != refs[0]
        || via_fields[4].parse::<i64>().ok() != signal.point[0].checked_mul(254)
        || via_fields[5].parse::<i64>().ok() != signal.point[1].checked_mul(254)
    {
        return Err("V11_VIA_IDENTITY");
    }
    let mut rows = String::new();
    let mut severity = PadRisk::GapObserved;
    let mut has_circles = false;
    for &layer in &layers {
        let diameter = reference_route_v8::verify_round_pad(&board, signal, signal_index, layer)?;
        reference_route_v8::verify_round_pad(&board, stitch, stitch_index, layer)?;
        let layer_id = &board.stackup.layers.get(
            layer.checked_sub(1).ok_or("V11_LAYER")?
        ).ok_or("V11_LAYER")?.meta.id;
        let matches: Vec<_> = areas.iter().enumerate().filter(|(i, area)| {
            area.layer as usize == layer && area.net_label.as_deref() == Some(refs[0])
                && board.areas.get(*i).is_some_and(|c| c.layer == *layer_id)
        }).collect();
        let [(idx, area)] = matches.as_slice() else {
            return Err("V11_AMBIGUOUS_REFERENCE_AREA");
        };
        let CopperGeometry::Polygon(outer_ring) = &area.geometry else {
            return Err("V11_OUTER_UNSUPPORTED");
        };
        let outer = copper_preview::rectangle_bounds(outer_ring).ok_or("V11_OUTER_NONRECT")?;
        if !geometry::pad_inside_outer(signal.point, diameter, outer) {
            return Err("V11_PAD_OUTSIDE_OUTER");
        }
        let mut holes = Vec::with_capacity(area.cutouts.len());
        for cut in &area.cutouts {
            let shape = match cut {
                CopperGeometry::Polygon(p) =>
                    SourceHole::Rectangle(copper_preview::rectangle_bounds(p)
                        .ok_or("V11_HOLE_NONRECT")?),
                CopperGeometry::Circle { centre, radius } => {
                    has_circles = true;
                    SourceHole::Circle { centre: *centre, radius: *radius }
                }
                CopperGeometry::Curved { .. } => return Err("V11_HOLE_ARC_UNSUPPORTED"),
            };
            holes.push(shape);
        }
        if !geometry::validate_holes(&holes, outer) {
            return Err("V11_INVALID_HOLE_TOPOLOGY");
        }
        let (state, selected) = geometry::classify(signal.point, diameter, &holes)?;
        if state.severity() > severity.severity() {
            severity = state;
        }
        writeln!(&mut rows,
            "LAYER|{layer}|{signal_index}|{}|{}|{}|{diameter}|{}|{}|{}",
            hex(board.areas[*idx].meta.id.source_object.as_bytes()),
            signal.point[0], signal.point[1], holes.len(), state.wire(),
            selected.map_or_else(|| "-".to_owned(), |n| n.to_string()),
        ).map_err(|_| "V11_FORMAT")?;
        for (ordinal, hole) in holes.iter().enumerate() {
            match hole {
                SourceHole::Rectangle([a,b,c,d]) =>
                    writeln!(&mut rows, "HOLE|{layer}|{ordinal}|R|{a}|{b}|{c}|{d}")
                        .map_err(|_| "V11_FORMAT")?,
                SourceHole::Circle { centre:[x,y], radius } =>
                    writeln!(&mut rows, "HOLE|{layer}|{ordinal}|C|{x}|{y}|{radius}")
                        .map_err(|_| "V11_FORMAT")?,
            }
        }
    }
    let (kind, legacy, contact) = if has_circles {
        ("V4", v4.as_str(), "SOURCE_CONTACT_UNVERIFIED")
    } else {
        // Only a genuine independently produced V8 is an acceptable V8 legacy.
        let v8 = reference_route_v8::encode(bytes, indices, split, refs, layers, stitch_index)?;
        return finish(header[1], severity, indices.len(), split, stitch_index,
            signal_index, &rows, "V8", &v8, "V8_SOURCE_CONTACT_CANDIDATE");
    };
    finish(header[1], severity, indices.len(), split, stitch_index,
        signal_index, &rows, kind, legacy, contact)
}
fn finish(
    sha: &str, state: PadRisk, count: usize, split: usize, stitch: usize,
    signal: usize, rows: &str, kind: &str, legacy: &str, contact: &str
) -> Result<String, &'static str> {
    let mut wire = format!(
        "BOARDSI_REFERENCE_ROUTE_V11|{sha}|{}|UNSUPPORTED|NOT_RUN|0|REFERENCE_TRANSFER_UNVERIFIED|2|{count}|{split}|{stitch}|{signal}|{kind}|{contact}\n{rows}LEGACY|{kind}|{}\n",
        state.wire(), legacy.lines().count()
    );
    wire.push_str(legacy);
    wire.push_str("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n");
    if wire.len() > MAX_WIRE { return Err("V11_OUTPUT_LIMIT"); }
    Ok(wire)
}
