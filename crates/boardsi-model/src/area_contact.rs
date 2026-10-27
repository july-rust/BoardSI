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
/// A boundary touch is excluded. Holes, arcs and invalid polygons fail closed.
pub fn strict_simple_area_contains(area: &CopperArea, p: &Point) -> Result<bool, ModelError> {
    let segs = &area.outer.segments;
    if !area.cutouts.is_empty() || !(3..=64).contains(&segs.len()) {
        return Err(ModelError::InvalidConnection);
    }
    let mut ring = Vec::with_capacity(segs.len());
    for segment in segs {
        let BoundarySegment::Line { start, end } = segment else {
            return Err(ModelError::InvalidConnection);
        };
        if start.frame != p.frame
            || end.frame != p.frame
            || start.ticks == end.ticks
            || ring.last().is_some_and(|&prev| prev != start.ticks)
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
            return Err(ModelError::InvalidConnection);
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
    Ok(inside)
}
