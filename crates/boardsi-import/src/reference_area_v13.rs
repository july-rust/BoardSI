//! C_021: opt-in two-rectangle Source-geometry audit, NOT a connectivity-graph edge.
//! Same PADS Net text never proves electrical continuity or Return Current.
use crate::{copper_preview, digest, route_preview};
use copper_preview::CopperGeometry;
use std::fmt::Write;

const MAX_WIRE_BYTES: usize = 4096;

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("write to string");
    }
    out
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceState {
    PositiveAreaOverlap,
    DisconnectedIsland,
    BoundaryOnly,
}

impl SourceState {
    pub fn wire(self) -> &'static str {
        match self {
            Self::PositiveAreaOverlap => "POSITIVE_AREA_OVERLAP_CANDIDATE",
            Self::DisconnectedIsland => "DISCONNECTED_ISLAND_RISK",
            Self::BoundaryOnly => "BOUNDARY_ONLY_RISK",
        }
    }
}

/// This is an exact rectangle-predicate on source ticks, NOT IPC, physical SI,
/// fabricated fill, clearance DRC or electrically verified AreaContinuity.
pub fn rectangle_relation(a: [i64; 4], b: [i64; 4]) -> Result<SourceState, &'static str> {
    if a[0] >= a[2] || a[1] >= a[3] || b[0] >= b[2] || b[1] >= b[3] {
        return Err("C021_INVALID_RECTANGLE");
    }
    let ox = i128::from(a[2].min(b[2])) - i128::from(a[0].max(b[0]));
    let oy = i128::from(a[3].min(b[3])) - i128::from(a[1].max(b[1]));
    Ok(if ox > 0 && oy > 0 {
        SourceState::PositiveAreaOverlap
    } else if ox >= 0 && oy >= 0 {
        SourceState::BoundaryOnly
    } else {
        SourceState::DisconnectedIsland
    })
}

/// A straight *axis-aligned* centreline, strictly covered by the union of the
/// source rectangles. Positive 2D overlap alone is not enough (an L-shape can
/// leave a line gap). Both rectangles must contribute a positive line interval.
pub fn strict_route_union(a: [i64; 4], b: [i64; 4], start: [i64; 2], end: [i64; 2]) -> bool {
    let (horizontal, fixed, lo, hi) = if start[1] == end[1] && start[0] != end[0] {
        (true, start[1], start[0].min(end[0]), start[0].max(end[0]))
    } else if start[0] == end[0] && start[1] != end[1] {
        (false, start[0], start[1].min(end[1]), start[1].max(end[1]))
    } else {
        return false;
    };
    let interval = |r: [i64; 4]| {
        if horizontal {
            (r[1] < fixed && fixed < r[3]).then_some((r[0], r[2]))
        } else {
            (r[0] < fixed && fixed < r[2]).then_some((r[1], r[3]))
        }
    };
    let (Some(mut x), Some(mut y)) = (interval(a), interval(b)) else {
        return false;
    };
    if x.0 > y.0 {
        std::mem::swap(&mut x, &mut y);
    }
    x.0 < lo && y.1.max(x.1) > hi && x.1 > y.0 && x.1 > lo && y.1 > lo && x.0 < hi && y.0 < hi
}

/// Source-bound selection by TWO explicit names; whole-source SHA, Canonical
/// identifiers, Layer and declared Net names are recorded. No existing V1–V12
/// receipt or Canonical connectivity graph is mutated.
pub fn encode(
    bytes: &[u8],
    area_names: [&str; 2],
    reference_net: &str,
    reference_layer: usize,
    signal_trace: Option<usize>,
) -> Result<String, &'static str> {
    if area_names[0].is_empty()
        || area_names[0] == area_names[1]
        || reference_net.is_empty()
        || reference_layer == 0
    {
        return Err("C021_INVALID_SELECTION");
    }
    let parsed = route_preview::parse(bytes)?;
    let mut board = route_preview::into_canonical(bytes, &parsed)?;
    let areas = copper_preview::parse(bytes, parsed.max_layer)?;
    copper_preview::augment(&mut board, &areas)?;
    if board.validate().has_errors() || board.areas.len() != areas.len() {
        return Err("C021_CANONICAL_INVALID");
    }
    // No third source area: C_001's general multi-area ambiguity still fails
    // closed. The entire source is reparsed, not a caller-supplied digest.
    if areas.len() != 2 {
        return Err("C021_EXACTLY_TWO_SOURCE_AREAS_REQUIRED");
    }
    let canonical_layer = &board
        .stackup
        .layers
        .get(reference_layer - 1)
        .ok_or("C021_REFERENCE_LAYER_INVALID")?
        .meta
        .id;
    let mut bounds = [[0i64; 4]; 2];
    let mut ids = [String::new(), String::new()];
    for (slot, name) in area_names.iter().enumerate() {
        let index = areas
            .iter()
            .position(|a| &a.name == name)
            .ok_or("C021_SOURCE_AREA_UNKNOWN")?;
        let source = &areas[index];
        let canonical = &board.areas[index];
        if source.layer as usize != reference_layer
            || canonical.layer != *canonical_layer
            || source.net_label.as_deref() != Some(reference_net)
            || canonical.net.is_some()
            || !source.cutouts.is_empty()
            || canonical.cutouts.len() != 0
            || canonical.outer.segments.len() != 4
        {
            return Err("C021_UNSUPPORTED_SOURCE_GEOMETRY");
        }
        let CopperGeometry::Polygon(vertices) = &source.geometry else {
            return Err("C021_UNSUPPORTED_SOURCE_GEOMETRY");
        };
        bounds[slot] = copper_preview::rectangle_bounds(vertices).ok_or("C021_NONRECTANGLE")?;
        // Prove the canonical original ring is byte/point identical, not a
        // separate approximate/bounding-box representation of another shape.
        for (segment, points) in canonical.outer.segments.iter().zip(vertices.windows(2)) {
            let boardsi_model::BoundarySegment::Line { start, end } = segment else {
                return Err("C021_CANONICAL_RING_MISMATCH");
            };
            if start.ticks != points[0] || end.ticks != points[1] || start.frame != end.frame {
                return Err("C021_CANONICAL_RING_MISMATCH");
            }
        }
        ids[slot] = hex(canonical.meta.id.source_object.as_bytes());
    }
    let relationship = rectangle_relation(bounds[0], bounds[1])?;
    let route = if let Some(index) = signal_trace {
        let raw = parsed
            .segments
            .get(index)
            .ok_or("C021_TRACE_INDEX_INVALID")?;
        let canonical = board.traces.get(index).ok_or("C021_TRACE_MISSING")?;
        let signal_layer = board
            .stackup
            .layers
            .get(
                (raw.layer as usize)
                    .checked_sub(1)
                    .ok_or("C021_TRACE_LAYER")?,
            )
            .ok_or("C021_TRACE_LAYER")?;
        if signal_layer
            .order
            .abs_diff(board.stackup.layers[reference_layer - 1].order)
            != 1
            || canonical.layer != signal_layer.meta.id
            || canonical.centreline.len() != 2
            || canonical.centreline[0].ticks != raw.a
            || canonical.centreline[1].ticks != raw.b
        {
            return Err("C021_UNSUPPORTED_TRACE");
        }
        if relationship == SourceState::PositiveAreaOverlap
            && strict_route_union(bounds[0], bounds[1], raw.a, raw.b)
        {
            "STRICT_UNION_CENTRELINE_CANDIDATE"
        } else {
            "REFERENCE_CORRIDOR_UNPROVEN"
        }
    } else {
        "NOT_REQUESTED"
    };
    let sha = hex(&digest::sha256(bytes));
    let mut result = format!(
        "BOARDSI_REFERENCE_AREA_V13|{sha}|{}|{route}|UNSUPPORTED|NOT_RUN|0|0\n",
        relationship.wire()
    );
    for (slot, name) in area_names.iter().enumerate() {
        writeln!(
            &mut result,
            "AREA|{slot}|{}|{}|{}|{}|{}|{}|{}|{}",
            hex(name.as_bytes()),
            ids[slot],
            reference_layer,
            bounds[slot][0],
            bounds[slot][1],
            bounds[slot][2],
            bounds[slot][3],
            hex(reference_net.as_bytes())
        )
        .map_err(|_| "C021_OUTPUT_ERROR")?;
    }
    writeln!(&mut result, "END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")
        .map_err(|_| "C021_OUTPUT_ERROR")?;
    if result.len() > MAX_WIRE_BYTES {
        return Err("C021_OUTPUT_LIMIT");
    }
    Ok(result)
}
