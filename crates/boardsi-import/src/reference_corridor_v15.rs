//! C_023 V15: source-exact rectangle union corridor RISK, never electrical continuity.
use crate::{reference_area_v14, route_preview};
use std::fmt::Write;

pub const MAX_WIRE_BYTES: usize = 12_288;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coverage {
    pub gaps: Vec<[i64; 2]>,
    pub boundary_only: bool,
}

/// Bounds are PADS source ticks. Only strict interior on the transverse axis counts.
/// A point/edge-only join between intervals is never promoted into a covered corridor.
pub fn corridor(rects: &[[i64; 4]], a: [i64; 2], b: [i64; 2]) -> Result<Coverage, &'static str> {
    if !(3..=6).contains(&rects.len()) {
        return Err("C023_AREA_COUNT");
    }
    let (horizontal, fixed, lo, hi) = if a[1] == b[1] && a[0] != b[0] {
        (true, a[1], a[0].min(b[0]), a[0].max(b[0]))
    } else if a[0] == b[0] && a[1] != b[1] {
        (false, a[0], a[1].min(b[1]), a[1].max(b[1]))
    } else {
        return Err("C023_NON_STRAIGHT_ROUTE");
    };
    let mut intervals = Vec::new();
    for &r in rects {
        if r[0] >= r[2] || r[1] >= r[3] {
            return Err("C023_INVALID_RECTANGLE");
        }
        let inside = if horizontal {
            r[1] < fixed && fixed < r[3]
        } else {
            r[0] < fixed && fixed < r[2]
        };
        if inside {
            let start = lo.max(if horizontal { r[0] } else { r[1] });
            let end = hi.min(if horizontal { r[2] } else { r[3] });
            if end > start {
                intervals.push([start, end]);
            }
        }
    }
    intervals.sort_unstable();
    let mut cursor = lo;
    let mut gaps = Vec::new();
    let mut boundary_only = false;
    for [start, end] in intervals {
        if start > cursor {
            gaps.push([cursor, start]);
        } else if start == cursor && cursor != lo {
            // Two open copper interiors meet only at a line/point.
            boundary_only = true;
        }
        cursor = cursor.max(end);
    }
    if cursor < hi {
        gaps.push([cursor, hi]);
    }
    Ok(Coverage {
        gaps,
        boundary_only,
    })
}

fn hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut result, "{byte:02x}").expect("string");
    }
    result
}

pub fn encode(
    bytes: &[u8],
    area_names: &[&str],
    reference_net: &str,
    reference_layer: usize,
    trace_index: usize,
) -> Result<String, &'static str> {
    // V14 is the authoritative, independently tested source + Canonical Area
    // inventory gate; V15 cannot bypass any V14 rejection or add graph edges.
    let prior = reference_area_v14::encode(bytes, area_names, reference_net, reference_layer)?;
    let lines: Vec<_> = prior.lines().collect();
    if lines.len() != area_names.len() + 2 {
        return Err("C023_V14_RECEIPT");
    }
    let header: Vec<_> = lines[0].split('|').collect();
    if header.len() != 10 {
        return Err("C023_V14_HEADER");
    }
    let mut rects = Vec::new();
    for (i, row) in lines[1..=area_names.len()].iter().enumerate() {
        let fields: Vec<_> = row.split('|').collect();
        if fields.len() != 11 || fields[0] != "AREA" || fields[1].parse::<usize>().ok() != Some(i) {
            return Err("C023_V14_AREA");
        }
        let mut r = [0i64; 4];
        for (index, slot) in r.iter_mut().enumerate() {
            *slot = fields[5 + index].parse().map_err(|_| "C023_V14_BOUNDS")?;
        }
        rects.push(r);
    }
    let parsed = route_preview::parse(bytes)?;
    if parsed.skipped != 0 || trace_index >= parsed.segments.len() || !parsed.arcs.is_empty() {
        return Err("C023_INCOMPLETE_ROUTE_INVENTORY");
    }
    let trace = &parsed.segments[trace_index];
    if trace.net == reference_net || trace.net.is_empty() || trace.layer as usize == reference_layer
    {
        // Signal and reference metal must be on distinct layers in this bounded case.
        return Err("C023_SIGNAL_REFERENCE_SELECTION");
    }
    let coverage = corridor(&rects, trace.a, trace.b)?;
    let groups = header[3].parse::<usize>().map_err(|_| "C023_V14_GROUPS")?;
    let boundary_pairs = header[5]
        .parse::<usize>()
        .map_err(|_| "C023_V14_BOUNDARIES")?;
    let state = if !coverage.gaps.is_empty() {
        "REFERENCE_GAP_RISK"
    } else if groups > 1 {
        "DISCONNECTED_COMPONENT_RISK"
    } else if coverage.boundary_only || boundary_pairs > 0 {
        "BOUNDARY_ONLY_RISK"
    } else {
        "COVERED_SOURCE_CANDIDATE"
    };
    let mut result = format!(
        "BOARDSI_REFERENCE_CORRIDOR_V15|{}|{}|{trace_index}|{groups}|{boundary_pairs}|{state}|{}|UNSUPPORTED|NOT_RUN|0|0\n",
        header[1],area_names.len(),coverage.gaps.len());
    for row in &lines[1..=area_names.len()] {
        writeln!(&mut result, "{row}").map_err(|_| "C023_OUTPUT")?;
    }
    writeln!(
        &mut result,
        "TRACE|{trace_index}|{}|{}|{}|{}|{}|{}",
        trace.layer,
        hex(trace.net.as_bytes()),
        trace.a[0],
        trace.a[1],
        trace.b[0],
        trace.b[1]
    )
    .map_err(|_| "C023_OUTPUT")?;
    for [a, b] in coverage.gaps {
        writeln!(&mut result, "GAP|{a}|{b}").map_err(|_| "C023_OUTPUT")?;
    }
    writeln!(
        &mut result,
        "END|{}|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL",
        area_names.len()
    )
    .map_err(|_| "C023_OUTPUT")?;
    if result.len() > MAX_WIRE_BYTES {
        return Err("C023_OUTPUT_LIMIT");
    }
    Ok(result)
}
