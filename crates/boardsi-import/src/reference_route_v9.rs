//! C_013: source-exact Signal Via round Pad versus explicit Reference COPCUT.
//! Deliberately bounded to a single straight rectangular Source Copper area and
//! at most one same-owner rectangular COPCUT on each selected reference layer.
//! Source geometry only, never manufactured clearance, P009 or SI qualification.
use crate::{copper_preview, reference_route_v8, route_preview};
use std::fmt::Write;

const MAX_WIRE: usize = 250_000;

fn hex(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for byte in s.bytes() {
        write!(&mut out, "{byte:02x}").expect("infallible string formatter");
    }
    out
}

fn strictly_in_rectangle(pos: [i64; 2], diameter: i64, rect: [i64; 4]) -> bool {
    if diameter <= 0 { return false; }
    let margins = [
        i128::from(pos[0]) - i128::from(rect[0]),
        i128::from(rect[2]) - i128::from(pos[0]),
        i128::from(pos[1]) - i128::from(rect[1]),
        i128::from(rect[3]) - i128::from(pos[1]),
    ];
    margins.iter().all(|m| *m > 0 && *m * 2 > i128::from(diameter))
}

fn classify_cutout(pos: [i64; 2], diameter: i64, rect: [i64; 4]) -> &'static str {
    let margins = [
        i128::from(pos[0]) - i128::from(rect[0]),
        i128::from(rect[2]) - i128::from(pos[0]),
        i128::from(pos[1]) - i128::from(rect[1]),
        i128::from(rect[3]) - i128::from(pos[1]),
    ];
    if margins.iter().any(|m| *m <= 0) {
        return "SOURCE_REFERENCE_COPPER_OVERLAP_RISK";
    }
    let minimum = *margins.iter().min().expect("four margins");
    match (minimum * 2).cmp(&i128::from(diameter)) {
        std::cmp::Ordering::Greater => "SOURCE_CUTOUT_GAP_OBSERVED",
        std::cmp::Ordering::Equal => "SOURCE_CUTOUT_TANGENCY_RISK",
        std::cmp::Ordering::Less => "SOURCE_REFERENCE_COPPER_OVERLAP_RISK",
    }
}
fn severity(state: &str) -> usize {
    match state {
        "SOURCE_REFERENCE_COPPER_OVERLAP_RISK" => 3,
        "SOURCE_CUTOUT_TANGENCY_RISK" => 2,
        "SOURCE_CUTOUT_MISSING_RISK" => 1,
        _ => 0,
    }
}

/// V9 independently regenerates V8 verbatim and never changes V1-V8 transports.
/// Each CUTOUT row is an observation of original Source copper, not a DRC pass.
pub fn encode(
    bytes: &[u8],
    indices: &[usize],
    split: usize,
    references: [&str; 2],
    reference_layers: [usize; 2],
    reference_via_index: usize,
) -> Result<String, &'static str> {
    let v8 = reference_route_v8::encode(
        bytes, indices, split, references, reference_layers, reference_via_index,
    )?;
    let header: Vec<_> = v8.lines().next().ok_or("V9_V8_EMPTY")?.split('|').collect();
    if header.len() != 13 || header[0] != "BOARDSI_REFERENCE_ROUTE_V8"
        || header[1].len() != 64 || header[10] != reference_via_index.to_string()
    {
        return Err("V9_V8_HEADER_INVALID");
    }
    let signal_index = header[11].parse::<usize>().map_err(|_| "V9_SIGNAL_INDEX")?;
    let parsed = route_preview::parse(bytes)?;
    let mut board = route_preview::into_canonical(bytes, &parsed)?;
    let source_areas = copper_preview::parse(bytes, parsed.max_layer)?;
    copper_preview::augment(&mut board, &source_areas)?;
    if board.validate().has_errors() || board.areas.len() != source_areas.len() {
        return Err("V9_CANONICAL_INVALID");
    }
    let signal = parsed.vias.get(signal_index).ok_or("V9_SIGNAL_VIA_MISSING")?;
    let reference = parsed.vias.get(reference_via_index).ok_or("V9_STITCH_VIA_MISSING")?;
    if signal.net.is_empty() || signal.net == reference.net || reference.net != references[0] {
        return Err("V9_SOURCE_NET_MISMATCH");
    }

    let mut overall = "SOURCE_CUTOUT_GAP_OBSERVED";
    let mut rows = String::new();
    for &layer in &reference_layers {
        let diameter = reference_route_v8::verify_round_pad(
            &board, signal, signal_index, layer,
        )?;
        let _ref_diameter = reference_route_v8::verify_round_pad(
            &board, reference, reference_via_index, layer,
        )?;
        let layer_id = &board.stackup.layers
            .get(layer.checked_sub(1).ok_or("V9_LAYER_INVALID")?)
            .ok_or("V9_LAYER_INVALID")?.meta.id;
        let matches: Vec<_> = source_areas.iter().enumerate()
            .filter(|(i, area)| {
                area.layer as usize == layer && area.net_label.as_deref() == Some(references[0])
                    && board.areas.get(*i).is_some_and(|canonical| canonical.layer == *layer_id)
            }).collect();
        let [(area_index, area)] = matches.as_slice() else {
            return Err("V9_AMBIGUOUS_REFERENCE_AREA");
        };
        let source_identity = &board.areas[*area_index].meta.id.source_object;
        let copper_preview::CopperGeometry::Polygon(outer) = &area.geometry else {
            return Err("V9_UNSUPPORTED_OUTER");
        };
        let outer_bounds = copper_preview::rectangle_bounds(outer)
            .ok_or("V9_UNSUPPORTED_OUTER_SHAPE")?;
        // Pad must be strictly inside an unambiguous declared outer region.
        if !strictly_in_rectangle(signal.point, diameter, outer_bounds) {
            return Err("V9_SIGNAL_OUTSIDE_SOURCE_REFERENCE_AREA");
        }
        // This V9 slice intentionally excludes multiple, round, non-rectangular
        // or curved holes. Do not manufacture a void from a via's presence.
        if area.cutouts.len() > 1 { return Err("V9_MULTIPLE_CUTOUTS_UNSUPPORTED"); }
        let (bounds, state) = match area.cutouts.first() {
            None => (None, "SOURCE_CUTOUT_MISSING_RISK"),
            Some(copper_preview::CopperGeometry::Polygon(hole)) => {
                let rect = copper_preview::rectangle_bounds(hole)
                    .ok_or("V9_UNSUPPORTED_CUTOUT_SHAPE")?;
                (Some(rect), classify_cutout(signal.point, diameter, rect))
            }
            Some(_) => return Err("V9_UNSUPPORTED_CUTOUT_SHAPE"),
        };
        if severity(state) > severity(overall) { overall = state; }
        let rect_text = if let Some([x0, y0, x1, y1]) = bounds {
            format!("{x0}|{y0}|{x1}|{y1}")
        } else { "-|-|-|-".to_owned() };
        writeln!(&mut rows,
            "CUTOUT|{layer}|{signal_index}|{}|{}|{}|{diameter}|{rect_text}|{state}",
            hex(source_identity), signal.point[0], signal.point[1]
        ).map_err(|_| "V9_FORMAT")?;
    }
    let mut wire = format!(
        "BOARDSI_REFERENCE_ROUTE_V9|{}|{overall}|UNSUPPORTED|NOT_RUN|0|REFERENCE_TRANSFER_UNVERIFIED|2|{}|{split}|{reference_via_index}|{signal_index}\n{rows}V8|{}\n",
        header[1], indices.len(), v8.lines().count()
    );
    wire.push_str(&v8);
    wire.push_str("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n");
    if wire.len() > MAX_WIRE { return Err("V9_OUTPUT_LIMIT"); }
    Ok(wire)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_rectangle_tangency_and_single_tick_distinction() {
        let rect = [0, 0, 100, 100];
        assert_eq!(classify_cutout([50, 50], 99, rect), "SOURCE_CUTOUT_GAP_OBSERVED");
        assert_eq!(classify_cutout([50, 50], 100, rect), "SOURCE_CUTOUT_TANGENCY_RISK");
        assert_eq!(classify_cutout([50, 50], 101, rect), "SOURCE_REFERENCE_COPPER_OVERLAP_RISK");
        assert_eq!(classify_cutout([101, 50], 1, rect), "SOURCE_REFERENCE_COPPER_OVERLAP_RISK");
        assert!(strictly_in_rectangle([50, 50], 99, rect));
        assert!(!strictly_in_rectangle([50, 50], 100, rect));
    }
}
