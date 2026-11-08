//! D_056_03: positive-only, source-bound straight route-to-filled-area
//! witness from SOURCE polygon and circle boundary events.
//!
//! The old fixed endpoint/quarter sampling misses narrow intersected COPPER
//! polygons. Event intervals cover crossings regardless of trace angle and
//! without imposing a 2/4/6 layer assumption. A robustly strict interior
//! centreline point is sufficient for positive-area stroke copper contact;
//! missing evidence is UNKNOWN, not proof of a gap or fabricated Open.
use crate::copper_preview::{CopperArea, CopperGeometry};
use crate::trace_contact::{Centreline, Stroke};

const GUARD_TICKS: f64 = 0.0001;
const EVENT_CAP: usize = 16_384;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Position {
    Inside,
    Outside,
    Boundary,
}

fn point_segment_distance2(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let length2 = dx * dx + dy * dy;
    if length2 == 0.0 {
        return (p[0] - a[0]).powi(2) + (p[1] - a[1]).powi(2);
    }
    let t = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / length2).clamp(0.0, 1.0);
    let x = a[0] + t * dx;
    let y = a[1] + t * dy;
    (p[0] - x).powi(2) + (p[1] - y).powi(2)
}

fn region(shape: &CopperGeometry, p: [f64; 2]) -> Position {
    match shape {
        CopperGeometry::Polygon(vertices) => {
            let mut inside = false;
            for side in vertices.windows(2) {
                let a = [side[0][0] as f64, side[0][1] as f64];
                let b = [side[1][0] as f64, side[1][1] as f64];
                if point_segment_distance2(p, a, b) <= GUARD_TICKS * GUARD_TICKS {
                    return Position::Boundary;
                }
                if (a[1] > p[1]) != (b[1] > p[1])
                    && p[0] < a[0] + (p[1] - a[1]) * (b[0] - a[0]) / (b[1] - a[1])
                {
                    inside = !inside;
                }
            }
            if inside {
                Position::Inside
            } else {
                Position::Outside
            }
        }
        CopperGeometry::Circle { centre, radius } => {
            let dx = p[0] - centre[0] as f64;
            let dy = p[1] - centre[1] as f64;
            let distance = (dx * dx + dy * dy).sqrt();
            if (distance - *radius as f64).abs() <= GUARD_TICKS {
                Position::Boundary
            } else if distance < *radius as f64 {
                Position::Inside
            } else {
                Position::Outside
            }
        }
        CopperGeometry::Curved { .. } => Position::Boundary,
    }
}

fn copper_interior(area: &CopperArea, p: [f64; 2]) -> bool {
    region(&area.geometry, p) == Position::Inside
        && area
            .cutouts
            .iter()
            .all(|hole| region(hole, p) == Position::Outside)
}

fn add_event(events: &mut Vec<f64>, t: f64) -> Result<(), &'static str> {
    if t.is_finite() && (0.0..=1.0).contains(&t) {
        if events.len() >= EVENT_CAP {
            return Err("D05603_STROKE_AREA_EVENT_CAP");
        }
        events.push(t);
    }
    Ok(())
}

fn boundary_events(
    shape: &CopperGeometry,
    a: [f64; 2],
    b: [f64; 2],
    events: &mut Vec<f64>,
) -> Result<(), &'static str> {
    let d = [b[0] - a[0], b[1] - a[1]];
    match shape {
        CopperGeometry::Polygon(vertices) => {
            for edge in vertices.windows(2) {
                let p = [edge[0][0] as f64, edge[0][1] as f64];
                let e = [edge[1][0] as f64 - p[0], edge[1][1] as f64 - p[1]];
                let denominator = d[0] * e[1] - d[1] * e[0];
                if denominator == 0.0 {
                    // Collinear boundary is not strict interior evidence.
                    continue;
                }
                let q = [p[0] - a[0], p[1] - a[1]];
                let t = (q[0] * e[1] - q[1] * e[0]) / denominator;
                let u = (q[0] * d[1] - q[1] * d[0]) / denominator;
                if (0.0..=1.0).contains(&u) {
                    add_event(events, t)?;
                }
            }
        }
        CopperGeometry::Circle { centre, radius } => {
            let q = [a[0] - centre[0] as f64, a[1] - centre[1] as f64];
            let aa = d[0] * d[0] + d[1] * d[1];
            if aa == 0.0 {
                return Ok(());
            }
            let bb = 2.0 * (q[0] * d[0] + q[1] * d[1]);
            let cc = q[0] * q[0] + q[1] * q[1] - (*radius as f64).powi(2);
            let discriminant = bb * bb - 4.0 * aa * cc;
            if discriminant >= 0.0 {
                let root = discriminant.sqrt();
                add_event(events, (-bb - root) / (2.0 * aa))?;
                add_event(events, (-bb + root) / (2.0 * aa))?;
            }
        }
        CopperGeometry::Curved { .. } => return Err("D05603_STROKE_AREA_CURVED_UNSUPPORTED"),
    }
    Ok(())
}

/// A *positive* witness only: never return true at tangency, through a source
/// cutout, or by interpolating the chord of a circular arc. The caller must
/// still treat a false result as UNKNOWN rather than a verified separation.
pub fn positive_straight_area_witness(
    stroke: &Stroke,
    area: &CopperArea,
) -> Result<bool, &'static str> {
    if stroke.layer != area.layer || stroke.width_ticks <= 0 {
        return Ok(false);
    }
    let (start, end) = match &stroke.centreline {
        Centreline::Straight { start, end } => (*start, *end),
        Centreline::Circular { .. } => return Ok(false),
    };
    let a = [start[0] as f64, start[1] as f64];
    let b = [end[0] as f64, end[1] as f64];
    let mut events = vec![0.0, 1.0];
    boundary_events(&area.geometry, a, b, &mut events)?;
    for hole in &area.cutouts {
        boundary_events(hole, a, b, &mut events)?;
    }
    events.sort_by(f64::total_cmp);
    for interval in events.windows(2) {
        if interval[1] <= interval[0] {
            continue;
        }
        let t = interval[0] + (interval[1] - interval[0]) * 0.5;
        let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        if copper_interior(area, p) {
            return Ok(true);
        }
    }
    Ok(false)
}
