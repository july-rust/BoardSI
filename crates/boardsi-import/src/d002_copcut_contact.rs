//! D_002: exactly two source COPCLS rectangles and at most one same-owner COPCUT.
//! This is a geometry-only hypothesis, never an electrical contact graph edge.
use crate::{copper_preview, digest, pour_preview, route_preview};
use copper_preview::CopperGeometry;
use std::fmt::Write;

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("hex output");
    }
    out
}

/// Strictly positive area in the intersection (A minus own cutout) AND
/// (B minus own cutout), using all exact source-grid change coordinates.
/// No rounding, midpoint float, bounding-box-only or edge-only promotion.
pub fn classify(
    rectangles: [[i64; 4]; 2],
    holes: [Option<[i64; 4]>; 2],
) -> Result<&'static str, &'static str> {
    if holes.iter().filter(|h| h.is_some()).count() > 1 {
        return Err("D002_MORE_THAN_ONE_HOLE");
    }
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for (r, h) in rectangles.iter().zip(holes.iter()) {
        if r[0] >= r[2] || r[1] >= r[3] {
            return Err("D002_BAD_RECTANGLE");
        }
        xs.extend([r[0], r[2]]);
        ys.extend([r[1], r[3]]);
        if let Some(c) = h {
            if c[0] >= c[2] || c[1] >= c[3] ||
                c[0] <= r[0] || c[1] <= r[1] ||
                c[2] >= r[2] || c[3] >= r[3] {
                return Err("D002_BAD_CUTOUT");
            }
            xs.extend([c[0], c[2]]);
            ys.extend([c[1], c[3]]);
        }
    }
    xs.sort_unstable();
    xs.dedup();
    ys.sort_unstable();
    ys.dedup();
    for x in xs.windows(2) {
        for y in ys.windows(2) {
            let mx = i128::from(x[0]) + i128::from(x[1]);
            let my = i128::from(y[0]) + i128::from(y[1]);
            let inside = |r: [i64; 4]| {
                i128::from(r[0]) * 2 < mx && mx < i128::from(r[2]) * 2 &&
                i128::from(r[1]) * 2 < my && my < i128::from(r[3]) * 2
            };
            let retained = |index: usize| {
                inside(rectangles[index]) &&
                !holes[index].map(inside).unwrap_or(false)
            };
            if retained(0) && retained(1) {
                return Ok("SOURCE_CONTACT_CANDIDATE");
            }
        }
    }
    Ok("SOURCE_SEPARATION_RISK")
}

fn matching_ring(
    segments: &[boardsi_model::BoundarySegment],
    vertices: &[[i64; 2]],
) -> bool {
    if vertices.len() != 5 || segments.len() != vertices.len() - 1 {
        return false;
    }
    segments.iter().zip(vertices.windows(2)).all(|(s, pair)| {
        match s {
            boardsi_model::BoundarySegment::Line { start, end } =>
                start.ticks == pair[0] && end.ticks == pair[1] &&
                start.frame == end.frame,
            _ => false,
        }
    })
}

/// Opt-in, full-source reparse and Source/Canonical parity; never default wire.
pub fn encode(
    bytes: &[u8],
    names: [&str; 2],
    reference_net: &str,
    reference_layer: usize,
) -> Result<String, &'static str> {
    if names[0].is_empty() || names[1].is_empty() ||
        names[0] == names[1] || names.iter().any(|n| n.len() > 16) ||
        reference_net.is_empty() || reference_net.len() > 47 ||
        reference_layer == 0 {
        return Err("D002_INVALID_SELECTION");
    }
    let parsed = route_preview::parse(bytes)?;
    // An unfilled pour is not validated copper. Refuse mixed input rather than
    // silently inferring manufactured metal from a source association.
    if !pour_preview::parse(bytes, parsed.max_layer)?.is_empty() {
        return Err("D002_UNVERIFIED_POUR");
    }
    let mut board = route_preview::into_canonical(bytes, &parsed)?;
    let areas = copper_preview::parse(bytes, parsed.max_layer)?;
    copper_preview::augment(&mut board, &areas)?;
    if board.validate().has_errors() || areas.len() != 2 ||
        board.areas.len() != areas.len() {
        return Err("D002_INCOMPLETE_SOURCE_INVENTORY");
    }
    let layer = &board.stackup.layers
        .get(reference_layer - 1)
        .ok_or("D002_LAYER_INVALID")?.meta.id;
    let mut boxes = [[0i64; 4]; 2];
    let mut holes = [None; 2];
    let mut identities = [String::new(), String::new()];
    for (slot, name) in names.iter().enumerate() {
        let index = areas.iter().position(|a| a.name == *name)
            .ok_or("D002_AREA_UNKNOWN")?;
        let source = &areas[index];
        let canonical = &board.areas[index];
        if source.layer as usize != reference_layer ||
            canonical.layer != *layer ||
            source.net_label.as_deref() != Some(reference_net) ||
            canonical.net.is_some() ||
            source.cutouts.len() != canonical.cutouts.len() ||
            source.cutouts.len() > 1 {
            return Err("D002_UNSUPPORTED_OWNERSHIP");
        }
        let CopperGeometry::Polygon(outer) = &source.geometry else {
            return Err("D002_UNSUPPORTED_OUTER");
        };
        let rect = copper_preview::rectangle_bounds(outer)
            .ok_or("D002_OUTER_NOT_RECTANGLE")?;
        if !matching_ring(&canonical.outer.segments, outer) {
            return Err("D002_OUTER_CANONICAL_MISMATCH");
        }
        boxes[slot] = rect;
        identities[slot] = hex(canonical.meta.id.source_object.as_bytes());
        if let Some(void) = source.cutouts.first() {
            let CopperGeometry::Polygon(vertices) = void else {
                return Err("D002_UNSUPPORTED_VOID");
            };
            let cut = copper_preview::rectangle_bounds(vertices)
                .ok_or("D002_VOID_NOT_RECTANGLE")?;
            if !matching_ring(&canonical.cutouts[0].boundary.segments, vertices) {
                return Err("D002_VOID_CANONICAL_MISMATCH");
            }
            holes[slot] = Some(cut);
        }
    }
    let state = classify(boxes, holes)?;
    let sha = hex(&digest::sha256(bytes));
    let mut wire = format!(
        "BOARDSI_D002_COPCUT_CONTACT_V1|{sha}|{state}|PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|0|0\n"
    );
    for (slot, name) in names.iter().enumerate() {
        let r = boxes[slot];
        writeln!(&mut wire,
            "AREA|{slot}|{}|{}|{reference_layer}|{}|{}|{}|{}|{}",
            hex(name.as_bytes()), identities[slot], r[0], r[1], r[2], r[3],
            hex(reference_net.as_bytes())
        ).map_err(|_| "D002_WRITE_FAILED")?;
    }
    if let Some((owner, h)) = holes.iter().enumerate().find_map(|(i, h)| h.map(|r| (i, r))) {
        writeln!(&mut wire, "HOLE|{owner}|{}|{}|{}|{}", h[0], h[1], h[2], h[3])
            .map_err(|_| "D002_WRITE_FAILED")?;
    } else {
        wire.push_str("HOLE|NONE\n");
    }
    wire.push_str("END|2|SOURCE_CONTACT_HYPOTHESIS_ONLY\n");
    if wire.len() > 4096 {
        return Err("D002_WIRE_LIMIT");
    }
    Ok(wire)
}
