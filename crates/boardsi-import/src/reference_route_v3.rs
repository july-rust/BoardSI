//! C_007 opt-in multi-straight-route source Reference gap witness.
//! A route is an explicit ordered list of source trace indices, NOT a net-name walk.
//! This is geometry-only: P009 UNSUPPORTED, solver_ready=false, physical_si=NOT_RUN.
use crate::{reference_gap_intervals, route_preview};
use std::collections::BTreeSet;
use std::fmt::Write;

const MAX_ROUTE_SEGMENTS: usize = 16;
const MAX_OUTPUT: usize = 140_000;

fn cross(a: [i64; 2], b: [i64; 2], c: [i64; 2]) -> i128 {
    (i128::from(b[0]) - i128::from(a[0])) * (i128::from(c[1]) - i128::from(a[1]))
        - (i128::from(b[1]) - i128::from(a[1])) * (i128::from(c[0]) - i128::from(a[0]))
}
fn on(a: [i64; 2], b: [i64; 2], p: [i64; 2]) -> bool {
    cross(a, b, p) == 0
        && (a[0].min(b[0])..=a[0].max(b[0])).contains(&p[0])
        && (a[1].min(b[1])..=a[1].max(b[1])).contains(&p[1])
}
fn intersects(a: [i64; 2], b: [i64; 2], c: [i64; 2], d: [i64; 2]) -> bool {
    let (p, q, r, s) = (cross(a, b, c), cross(a, b, d), cross(c, d, a), cross(c, d, b));
    (p == 0 && on(a, b, c))
        || (q == 0 && on(a, b, d))
        || (r == 0 && on(c, d, a))
        || (s == 0 && on(c, d, b))
        || ((p > 0) != (q > 0) && (r > 0) != (s > 0))
}
/// Only forward source-order, end-to-start joints are accepted. Non-selected
/// branches, crossings and same-net vias/arcs are intentionally fail-closed.
pub fn encode(
    bytes: &[u8],
    indices: &[usize],
    reference: &str,
    reference_layer: usize,
) -> Result<String, &'static str> {
    if !(2..=MAX_ROUTE_SEGMENTS).contains(&indices.len()) {
        return Err("ROUTE_SEGMENT_LIMIT");
    }
    if bytes.len() > route_preview::MAX_FILE_BYTES || bytes.is_empty() {
        return Err("SOURCE_SIZE_UNSUPPORTED");
    }
    let parsed = route_preview::parse(bytes)?;
    let mut seen = BTreeSet::new();
    let mut selected = Vec::new();
    for &index in indices {
        if !seen.insert(index) {
            return Err("DUPLICATE_ROUTE_TRACE");
        }
        let segment = parsed.segments.get(index).ok_or("UNKNOWN_ROUTE_TRACE")?;
        if segment.a == segment.b {
            return Err("DEGENERATE_ROUTE_SEGMENT");
        }
        if let Some(first) = selected.first() {
            let first: &&route_preview::Segment = first;
            if segment.layer != first.layer || segment.net != first.net {
                return Err("MIXED_ROUTE_NET_OR_LAYER");
            }
        }
        if let Some(previous) = selected.last() {
            let previous: &&route_preview::Segment = previous;
            if previous.b != segment.a {
                return Err("ROUTE_ENDPOINT_DISCONNECTED_OR_REVERSED");
            }
            if cross(previous.a, previous.b, segment.b) == 0
                && on(previous.a, previous.b, segment.b)
            {
                return Err("OVERLAPPING_ROUTE_SEGMENTS");
            }
        }
        for earlier in selected.iter().take(selected.len().saturating_sub(1)) {
            let earlier: &&route_preview::Segment = earlier;
            if intersects(earlier.a, earlier.b, segment.a, segment.b) {
                return Err("SELF_INTERSECTING_ROUTE");
            }
        }
        selected.push(segment);
    }
    let first = selected[0];
    // A same-net trace touching even the interior of this selected route
    // changes its topology; no assumed straight-through traversal.
    for (index, segment) in parsed.segments.iter().enumerate() {
        if seen.contains(&index) || segment.net != first.net || segment.layer != first.layer {
            continue;
        }
        if selected
            .iter()
            .any(|s| intersects(s.a, s.b, segment.a, segment.b))
        {
            return Err("UNSELECTED_BRANCH_OR_CROSSING");
        }
    }
    // Conservative: a same-net via or arc on this layer could introduce a
    // branch or unsupported connection even if the artwork subset is incomplete.
    if parsed.arcs.iter().any(|a| a.net == first.net && a.layer == first.layer)
        || parsed.vias.iter().any(|v| {
            v.net == first.net
                && first.layer >= v.first_layer
                && first.layer <= v.last_layer
        })
    {
        return Err("ROUTE_ARC_OR_VIA_UNSUPPORTED");
    }
    let mut v2s = Vec::with_capacity(indices.len());
    let mut source_hash = String::new();
    let mut total_gaps = 0usize;
    let mut unsupported = false;
    for &index in indices {
        let wire = reference_gap_intervals::encode(bytes, index, reference, reference_layer)?;
        let header = wire.lines().next().ok_or("INVALID_V2_HEADER")?;
        let fields: Vec<_> = header.split('|').collect();
        if fields.len() != 9 || fields[0] != "BOARDSI_REFERENCE_GAP_V2" {
            return Err("INVALID_V2_HEADER");
        }
        if source_hash.is_empty() {
            source_hash = fields[1].to_owned();
        } else if source_hash != fields[1] {
            return Err("MULTIPLE_SOURCES");
        }
        if fields[2] == "UNSUPPORTED" {
            unsupported = true;
        } else if fields[2] != "REFERENCE_SPLIT_RISK"
            && fields[2] != "SOURCE_GEOMETRY_ONLY"
        {
            return Err("INVALID_V2_STATE");
        }
        total_gaps += wire.lines().filter(|line| line.starts_with("I|")).count();
        if total_gaps > 16 * 32 {
            return Err("ROUTE_INTERVAL_LIMIT");
        }
        v2s.push(wire);
    }
    if source_hash.len() != 64 {
        return Err("SOURCE_HASH_INVALID");
    }
    let status = if unsupported {
        "UNSUPPORTED"
    } else if total_gaps > 0 {
        "REFERENCE_SPLIT_RISK"
    } else {
        "SOURCE_GEOMETRY_ONLY"
    };
    let mut out = format!(
        "BOARDSI_REFERENCE_ROUTE_V3|{source_hash}|{}|{status}|UNSUPPORTED|NOT_RUN|0\n",
        indices.len()
    );
    for (&index, v2) in indices.iter().zip(&v2s) {
        writeln!(&mut out, "SEG|{index}|{}", v2.lines().count()).map_err(|_| "FORMAT")?;
        out.push_str(v2);
    }
    writeln!(
        &mut out,
        "END|{}|{total_gaps}|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL",
        indices.len()
    )
    .map_err(|_| "FORMAT")?;
    if out.len() > MAX_OUTPUT {
        return Err("ROUTE_OUTPUT_LIMIT");
    }
    Ok(out)
}
