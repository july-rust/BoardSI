//! B_004: strictly bounded source-grid polygon containment, not SI readiness.
use crate::{BoundarySegment, CopperArea, ModelError, Point};

fn cross(a: [i64; 2], b: [i64; 2], p: [i64; 2]) -> i128 {
    (i128::from(b[0]) - i128::from(a[0])) * (i128::from(p[1]) - i128::from(a[1]))
        - (i128::from(b[1]) - i128::from(a[1])) * (i128::from(p[0]) - i128::from(a[0]))
}
fn on(a: [i64; 2], b: [i64; 2], p: [i64; 2]) -> bool {
    cross(a, b, p) == 0
        && (a[0].min(b[0])..=a[0].max(b[0])).contains(&p[0])
        && (a[1].min(b[1])..=a[1].max(b[1])).contains(&p[1])
}
fn touch(a: [i64; 2], b: [i64; 2], c: [i64; 2], d: [i64; 2]) -> bool {
    let (x, y, z, w) = (
        cross(a, b, c),
        cross(a, b, d),
        cross(c, d, a),
        cross(c, d, b),
    );
    (x == 0 && on(a, b, c))
        || (y == 0 && on(a, b, d))
        || (z == 0 && on(c, d, a))
        || (w == 0 && on(c, d, b))
        || ((x > 0) != (y > 0) && (z > 0) != (w > 0))
}
/// C_001: strict integer containment for up to eight source-owned polygon holes.
/// Every boundary touch is excluded; malformed topology and unsupported holes fail closed.
pub fn strict_simple_area_contains(area: &CopperArea, p: &Point) -> Result<bool, ModelError> {
    let segs = &area.outer.segments;
    if area.cutouts.len() > 8 || !(3..=64).contains(&segs.len()) {
        return Err(ModelError::InvalidConnection);
    }
    let mut ring = Vec::with_capacity(segs.len());
    let mut previous_end = None;
    for segment in segs {
        let BoundarySegment::Line { start, end } = segment else {
            return Err(ModelError::InvalidConnection);
        };
        if start.frame != p.frame
            || end.frame != p.frame
            || start.ticks == end.ticks
            || previous_end.is_some_and(|prior| prior != start.ticks)
            || start
                .ticks
                .iter()
                .chain(end.ticks.iter())
                .chain(p.ticks.iter())
                .any(|n| !(-5_600_000..=5_600_000).contains(n))
        {
            return Err(ModelError::InvalidConnection);
        }
        ring.push(start.ticks);
        previous_end = Some(end.ticks);
    }
    let BoundarySegment::Line { end, .. } = segs.last().unwrap() else {
        return Err(ModelError::InvalidConnection);
    };
    if end.ticks != ring[0] {
        return Err(ModelError::InvalidConnection);
    }
    let n = ring.len();
    let mut area2 = 0i128;
    for i in 0..n {
        let (a, b, c) = (ring[i], ring[(i + 1) % n], ring[(i + 2) % n]);
        if cross(a, b, c) == 0 {
            // Forward collinear source vertices preserve the same filled area.
            // Reject a segment that reverses or doubles back over copper edges.
            let ab = [
                i128::from(b[0]) - i128::from(a[0]),
                i128::from(b[1]) - i128::from(a[1]),
            ];
            let bc = [
                i128::from(c[0]) - i128::from(b[0]),
                i128::from(c[1]) - i128::from(b[1]),
            ];
            if ab[0] * bc[0] + ab[1] * bc[1] <= 0 {
                return Err(ModelError::InvalidConnection);
            }
        }
        area2 += i128::from(a[0]) * i128::from(b[1]) - i128::from(b[0]) * i128::from(a[1]);
        for j in i + 1..n {
            if j != i + 1 && !(i == 0 && j == n - 1) && touch(a, b, ring[j], ring[(j + 1) % n]) {
                return Err(ModelError::InvalidConnection);
            }
        }
    }
    if area2 == 0 {
        return Err(ModelError::InvalidConnection);
    }
    let mut inside = false;
    for i in 0..n {
        let (a, b) = (ring[i], ring[(i + 1) % n]);
        if on(a, b, p.ticks) {
            return Ok(false);
        }
        if (a[1] > p.ticks[1]) != (b[1] > p.ticks[1]) {
            let side = cross(a, b, p.ticks);
            if (b[1] > a[1] && side > 0) || (b[1] < a[1] && side < 0) {
                inside = !inside;
            }
        }
    }
    let mut in_void = false;
    for (hole_index, cutout) in area.cutouts.iter().enumerate() {
        if !(3..=64).contains(&cutout.boundary.segments.len()) {
            return Err(ModelError::InvalidConnection);
        }
        // Revalidate both rings in the Canonical model, not only the importer.
        // Strictly enclosed mutually disjoint simple holes cannot split a simple outer fill.
        let mut outer_only = area.clone();
        outer_only.cutouts.clear();
        let mut hole_only = outer_only.clone();
        hole_only.outer = cutout.boundary.clone();
        // Reject invalid/self-touching hole rings even when p is outside.
        strict_simple_area_contains(&hole_only, p)?;
        for segment in &hole_only.outer.segments {
            let BoundarySegment::Line { start, end } = segment else {
                return Err(ModelError::InvalidConnection);
            };
            if !strict_simple_area_contains(&outer_only, start)?
                || !strict_simple_area_contains(&outer_only, end)?
            {
                return Err(ModelError::InvalidConnection);
            }
            for outer_segment in &outer_only.outer.segments {
                let BoundarySegment::Line {
                    start: outer_start,
                    end: outer_end,
                } = outer_segment
                else {
                    return Err(ModelError::InvalidConnection);
                };
                if touch(start.ticks, end.ticks, outer_start.ticks, outer_end.ticks) {
                    return Err(ModelError::InvalidConnection);
                }
            }
        }
        // Fail closed on any pairwise cutout intersection, overlap or nesting.
        for earlier in &area.cutouts[..hole_index] {
            let mut earlier_only = outer_only.clone();
            earlier_only.outer = earlier.boundary.clone();
            strict_simple_area_contains(&earlier_only, p)?;
            for first in &earlier_only.outer.segments {
                let BoundarySegment::Line { start: a, end: b } = first else {
                    return Err(ModelError::InvalidConnection);
                };
                if strict_simple_area_contains(&hole_only, a)? {
                    return Err(ModelError::InvalidConnection);
                }
                for second in &hole_only.outer.segments {
                    let BoundarySegment::Line { start: c, end: d } = second else {
                        return Err(ModelError::InvalidConnection);
                    };
                    if touch(a.ticks, b.ticks, c.ticks, d.ticks) {
                        return Err(ModelError::InvalidConnection);
                    }
                }
            }
            if let BoundarySegment::Line { start, .. } = &hole_only.outer.segments[0] {
                if strict_simple_area_contains(&earlier_only, start)? {
                    return Err(ModelError::InvalidConnection);
                }
            }
        }
        // All hole interiors and exact hole boundaries are electrically void.
        in_void |= strict_simple_area_contains(&hole_only, p)?
            || hole_only.outer.segments.iter().any(|segment| {
                matches!(segment, BoundarySegment::Line { start, end } if on(start.ticks, end.ticks, p.ticks))
            });
    }
    Ok(inside && !in_void)
}
