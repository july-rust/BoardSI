//! C_006 opt-in exact source-rational reference gap intervals.
//! Geometry along one straight trace only; NEVER P009, EM or electrical continuity.
use crate::{copper_preview, reference_risk_wire, reference_topology, route_preview};
use boardsi_model::{strict_simple_area_contains, BoundarySegment};
use std::cmp::Ordering;
use std::fmt::Write;

const MAX_TICK: i64 = 5_600_000;
const MAX_INTERVALS: usize = 32;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio {
    pub n: i128,
    pub d: i128,
}
impl Ratio {
    fn new(mut n: i128, mut d: i128) -> Result<Self, &'static str> {
        if d == 0 {
            return Err("DEGENERATE_INTERSECTION");
        }
        if d < 0 {
            n = -n;
            d = -d;
        }
        let mut x = n.unsigned_abs();
        let mut y = d as u128;
        while y != 0 {
            let z = x % y;
            x = y;
            y = z;
        }
        let g = i128::try_from(x).map_err(|_| "GEOMETRY_OVERFLOW")?;
        Ok(Self { n: n / g, d: d / g })
    }
    fn cmp_exact(self, other: Self) -> Ordering {
        // PADS source grid is bounded above so both cross-products fit i128.
        (self.n * other.d).cmp(&(other.n * self.d))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gap {
    pub start: Ratio,
    pub end: Ratio,
}
fn cross(a: [i64; 2], b: [i64; 2]) -> i128 {
    i128::from(a[0]) * i128::from(b[1]) - i128::from(a[1]) * i128::from(b[0])
}
fn sub(a: [i64; 2], b: [i64; 2]) -> [i64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn on_segment(a: [i64; 2], b: [i64; 2], p: [i64; 2]) -> bool {
    cross(sub(b, a), sub(p, a)) == 0
        && (a[0].min(b[0])..=a[0].max(b[0])).contains(&p[0])
        && (a[1].min(b[1])..=a[1].max(b[1])).contains(&p[1])
}
fn boundary_points(segs: &[BoundarySegment]) -> Result<Vec<([i64; 2], [i64; 2])>, &'static str> {
    let mut out = Vec::with_capacity(segs.len());
    for seg in segs {
        let BoundarySegment::Line { start, end } = seg else {
            return Err("CURVED_BOUNDARY_UNSUPPORTED");
        };
        for p in [start.ticks, end.ticks] {
            if p.iter().any(|v| v.abs() > MAX_TICK) {
                return Err("COORDINATE_RANGE_UNSUPPORTED");
            }
        }
        out.push((start.ticks, end.ticks));
    }
    if out.len() < 3 {
        return Err("DEGENERATE_POLYGON");
    }
    Ok(out)
}
/// Exact cut parameters: endpoint, collinear boundary and vertex touch are
/// explicitly UNSUPPORTED instead of a forged precise gap.
pub fn uncovered(
    area: &boardsi_model::CopperArea,
    a: [i64; 2],
    b: [i64; 2],
    inside_at_start: bool,
) -> Result<Vec<Gap>, &'static str> {
    if a == b
        || [a, b]
            .iter()
            .flat_map(|p| p.iter())
            .any(|v| v.abs() > MAX_TICK)
    {
        return Err("TRACE_COORDINATES_UNSUPPORTED");
    }
    let direction = sub(b, a);
    let mut crossings = Vec::new();
    for segments in std::iter::once(&area.outer.segments)
        .chain(area.cutouts.iter().map(|cutout| &cutout.boundary.segments))
    {
        for (p, q) in boundary_points(segments)? {
            if on_segment(p, q, a) || on_segment(p, q, b) {
                return Err("TRACE_ENDPOINT_ON_BOUNDARY");
            }
            let edge = sub(q, p);
            let denominator = cross(direction, edge);
            if denominator == 0 {
                if cross(sub(p, a), direction) == 0 {
                    return Err("COLLINEAR_REFERENCE_EDGE_UNSUPPORTED");
                }
                continue;
            }
            let shift = sub(p, a);
            let t = Ratio::new(cross(shift, edge), denominator)?;
            let u = Ratio::new(cross(shift, direction), denominator)?;
            let zero = Ratio { n: 0, d: 1 };
            let one = Ratio { n: 1, d: 1 };
            if t.cmp_exact(zero) == Ordering::Less
                || t.cmp_exact(one) == Ordering::Greater
                || u.cmp_exact(zero) == Ordering::Less
                || u.cmp_exact(one) == Ordering::Greater
            {
                continue;
            }
            if t == zero || t == one || u == zero || u == one {
                return Err("BOUNDARY_VERTEX_OR_ENDPOINT_TOUCH");
            }
            crossings.push(t);
        }
    }
    crossings.sort_by(|a, b| a.cmp_exact(*b));
    if crossings.windows(2).any(|w| w[0] == w[1]) {
        return Err("AMBIGUOUS_DUPLICATE_CROSSING");
    }
    let mut gaps = Vec::new();
    let mut covered = inside_at_start;
    let mut previous = Ratio { n: 0, d: 1 };
    for next in crossings
        .into_iter()
        .chain(std::iter::once(Ratio { n: 1, d: 1 }))
    {
        if !covered {
            if let Some(last) = gaps.last_mut() {
                let last: &mut Gap = last;
                if last.end == previous {
                    last.end = next;
                } else {
                    gaps.push(Gap {
                        start: previous,
                        end: next,
                    });
                }
            } else {
                gaps.push(Gap {
                    start: previous,
                    end: next,
                });
            }
        }
        if gaps.len() > MAX_INTERVALS {
            return Err("TOO_MANY_GAP_INTERVALS");
        }
        covered = !covered;
        previous = next;
    }
    Ok(gaps)
}
fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(&mut out, "{b:02x}").expect("string write");
    }
    out
}
/// V2 is opt-in and has independent safety gates; V1 bytes and callers stay untouched.
pub fn encode(
    bytes: &[u8],
    index: usize,
    reference: &str,
    reference_layer_number: usize,
) -> Result<String, &'static str> {
    let v1 = reference_risk_wire::encode(bytes, &[index], reference, &[reference_layer_number])?;
    let mut v1lines = v1.lines();
    let header = v1lines.next().ok_or("V1_MISSING_HEADER")?;
    let row = v1lines.next().ok_or("V1_MISSING_SOURCE_TRACE")?;
    let header: Vec<_> = header.split('|').collect();
    if header.len() != 9 || header[0] != "BOARDSI_REFERENCE_RISK_V1" {
        return Err("V1_INVALID");
    }
    let mut state = header[2];
    let mut reason = header[3];
    let mut gaps = Vec::new();
    if state == "REFERENCE_SPLIT_RISK" || state == "SOURCE_GEOMETRY_ONLY" {
        let parsed = route_preview::parse(bytes)?;
        let mut board = route_preview::into_canonical(bytes, &parsed)?;
        let source_areas = copper_preview::parse(bytes, parsed.max_layer)?;
        copper_preview::augment(&mut board, &source_areas)?;
        let trace = board.traces.get(index).ok_or("UNKNOWN_TRACE")?;
        let reference_layer = board
            .stackup
            .layers
            .get(reference_layer_number - 1)
            .ok_or("UNKNOWN_REFERENCE_LAYER")?;
        let ids = [trace.meta.id.clone()];
        let result = reference_topology::audit(
            &board,
            bytes,
            &ids,
            reference,
            &[reference_layer.meta.id.clone()],
        );
        if result.state != state {
            return Err("INCONSISTENT_SOURCE_AUDIT");
        }
        let candidates: Vec<_> = board
            .areas
            .iter()
            .zip(&source_areas)
            .filter(|(a, s)| {
                a.layer == reference_layer.meta.id && s.net_label.as_deref() == Some(reference)
            })
            .map(|(a, _)| a)
            .collect();
        if candidates.len() != 1 {
            state = "UNSUPPORTED";
            reason = "MULTIPLE_REFERENCE_ISLANDS_UNSUPPORTED";
        } else {
            let area = candidates[0];
            let a = trace.centreline[0].ticks;
            let b = trace.centreline[1].ticks;
            let inside = strict_simple_area_contains(area, &trace.centreline[0])
                .map_err(|_| "REFERENCE_CONTAINS_UNSUPPORTED")?;
            match uncovered(area, a, b, inside) {
                Ok(g) => {
                    if (g.is_empty()) != (state == "SOURCE_GEOMETRY_ONLY") {
                        state = "UNSUPPORTED";
                        reason = "C002_C006_COVERAGE_MISMATCH";
                    } else {
                        gaps = g;
                    }
                }
                Err(why) => {
                    state = "UNSUPPORTED";
                    reason = why;
                }
            }
        }
    }
    let mut out = format!(
        "BOARDSI_REFERENCE_GAP_V2|{}|{}|{}|1|0|UNSUPPORTED|NOT_RUN|0\n",
        header[1], state, reason
    );
    writeln!(&mut out, "{row}").map_err(|_| "FORMAT")?;
    for g in &gaps {
        writeln!(
            &mut out,
            "I|{}|{}|{}|{}",
            g.start.n, g.start.d, g.end.n, g.end.d
        )
        .map_err(|_| "FORMAT")?;
    }
    writeln!(
        &mut out,
        "END|1|{}|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL",
        gaps.len()
    )
    .map_err(|_| "FORMAT")?;
    if out.len() > 8192 {
        return Err("OUTPUT_LIMIT");
    }
    // Original V1 source SHA must be a single parsed authenticated file SHA.
    if header[1].len() != 64
        || !header[1].bytes().all(|b| b.is_ascii_hexdigit())
        || !row.starts_with(&format!("T|{index}|"))
        || reference.is_empty()
    {
        return Err("SOURCE_WITNESS_INVALID");
    }
    let _ = hex; // source hash is provided unchanged by independently authenticated V1
    Ok(out)
}
