//! C_015: exact bounded canonical Source circle/COPCUT point containment.
//! Specialised only for a straight 4-edge rectangular outer and strict mixed
//! rectangular / cardinal-arc circular holes. No tessellation or physical SI.
use crate::{BoundarySegment, CopperArea, ModelError, Point};

#[derive(Clone, Copy)]
enum Hole {
    Rect([i64; 4]),
    Circle([i64; 2], i64),
}
fn valid(n: i64) -> bool {
    (-5_600_000..=5_600_000).contains(&n)
}
fn square(x: i128) -> i128 {
    x * x
}
fn d2(a: [i64; 2], b: [i64; 2]) -> i128 {
    square(i128::from(a[0]) - i128::from(b[0])) + square(i128::from(a[1]) - i128::from(b[1]))
}
fn rectangle(segs: &[BoundarySegment], p: &Point) -> Result<[i64; 4], ModelError> {
    if segs.len() != 4 {
        return Err(ModelError::InvalidConnection);
    }
    let mut pts = Vec::with_capacity(5);
    for s in segs {
        let BoundarySegment::Line { start, end } = s else {
            return Err(ModelError::InvalidConnection);
        };
        if start.frame != p.frame
            || end.frame != p.frame
            || start
                .ticks
                .iter()
                .chain(end.ticks.iter())
                .any(|&n| !valid(n))
            || start.ticks == end.ticks
        {
            return Err(ModelError::InvalidConnection);
        }
        if pts.last().is_some_and(|x| *x != start.ticks) {
            return Err(ModelError::InvalidConnection);
        }
        if pts.is_empty() {
            pts.push(start.ticks);
        }
        pts.push(end.ticks);
    }
    if pts[0] != pts[4] {
        return Err(ModelError::InvalidConnection);
    }
    let mut xs = [pts[0][0], pts[1][0], pts[2][0], pts[3][0]];
    let mut ys = [pts[0][1], pts[1][1], pts[2][1], pts[3][1]];
    xs.sort_unstable();
    ys.sort_unstable();
    let r = [xs[0], ys[0], xs[3], ys[3]];
    if r[0] >= r[2] || r[1] >= r[3] {
        return Err(ModelError::InvalidConnection);
    }
    let expected = [[r[0], r[1]], [r[0], r[3]], [r[2], r[1]], [r[2], r[3]]];
    let mut four = pts[..4].to_vec();
    four.sort_unstable();
    let mut check = expected.to_vec();
    check.sort_unstable();
    if four != check {
        return Err(ModelError::InvalidConnection);
    }
    for w in pts.windows(2) {
        if (w[0][0] == w[1][0]) == (w[0][1] == w[1][1]) {
            return Err(ModelError::InvalidConnection);
        }
    }
    Ok(r)
}
fn circle(segs: &[BoundarySegment], p: &Point) -> Result<([i64; 2], i64), ModelError> {
    if segs.len() != 4 {
        return Err(ModelError::InvalidConnection);
    }
    let mut centre = None;
    let mut rad = None;
    let mut ends = Vec::with_capacity(4);
    for s in segs {
        let BoundarySegment::Arc {
            centre: c,
            start,
            end,
            clockwise,
        } = s
        else {
            return Err(ModelError::InvalidConnection);
        };
        if *clockwise
            || c.frame != p.frame
            || start.frame != p.frame
            || end.frame != p.frame
            || c.ticks
                .iter()
                .chain(start.ticks.iter())
                .chain(end.ticks.iter())
                .any(|&n| !valid(n))
        {
            return Err(ModelError::InvalidConnection);
        }
        let dx = i128::from(start.ticks[0]) - i128::from(c.ticks[0]);
        let dy = i128::from(start.ticks[1]) - i128::from(c.ticks[1]);
        let radius = dx.abs().max(dy.abs());
        if radius < 1000
            || radius > 5_600_000
            || (dx == 0) == (dy == 0)
            || centre.is_some_and(|q| q != c.ticks)
            || rad.is_some_and(|r| r != radius)
        {
            return Err(ModelError::InvalidConnection);
        }
        centre = Some(c.ticks);
        rad = Some(radius);
        ends.push((start.ticks, end.ticks));
    }
    let c = centre.ok_or(ModelError::InvalidConnection)?;
    let r = i64::try_from(rad.ok_or(ModelError::InvalidConnection)?)
        .map_err(|_| ModelError::InvalidConnection)?;
    let cardinal = [
        ([c[0] + r, c[1]], [c[0], c[1] + r]),
        ([c[0], c[1] + r], [c[0] - r, c[1]]),
        ([c[0] - r, c[1]], [c[0], c[1] - r]),
        ([c[0], c[1] - r], [c[0] + r, c[1]]),
    ];
    if ends
        .iter()
        .zip(cardinal)
        .any(|(a, b)| a.0 != b.0 || a.1 != b.1)
    {
        return Err(ModelError::InvalidConnection);
    }
    Ok((c, r))
}
fn strict_inside(h: Hole, outer: [i64; 4]) -> bool {
    match h {
        Hole::Rect(r) => r[0] > outer[0] && r[1] > outer[1] && r[2] < outer[2] && r[3] < outer[3],
        Hole::Circle(c, r) => {
            c[0] - r > outer[0] && c[1] - r > outer[1] && c[0] + r < outer[2] && c[1] + r < outer[3]
        }
    }
}
fn disjoint(a: Hole, b: Hole) -> bool {
    match (a, b) {
        (Hole::Rect(a), Hole::Rect(b)) => a[2] < b[0] || b[2] < a[0] || a[3] < b[1] || b[3] < a[1],
        (Hole::Circle(a, ar), Hole::Circle(b, br)) => {
            d2(a, b) > square(i128::from(ar) + i128::from(br))
        }
        (Hole::Rect(a), Hole::Circle(b, r)) | (Hole::Circle(b, r), Hole::Rect(a)) => {
            let closest = [b[0].clamp(a[0], a[2]), b[1].clamp(a[1], a[3])];
            d2(closest, b) > square(i128::from(r))
        }
    }
}
fn in_hole(h: Hole, p: [i64; 2]) -> bool {
    match h {
        Hole::Rect(r) => p[0] >= r[0] && p[0] <= r[2] && p[1] >= r[1] && p[1] <= r[3],
        Hole::Circle(c, r) => d2(p, c) <= square(i128::from(r)),
    }
}
/// Return None to let the legacy polygon-only algorithm run byte-exact.
/// Some(Err) rejects malformed arcs rather than guessing ownership/contact.
pub(crate) fn strict_mixed_circle_contains(
    area: &CopperArea,
    p: &Point,
) -> Option<Result<bool, ModelError>> {
    if !area.cutouts.iter().any(|h| {
        h.boundary
            .segments
            .iter()
            .any(|s| matches!(s, BoundarySegment::Arc { .. }))
    }) {
        return None;
    }
    Some((|| {
        if area.cutouts.len() > 8 || p.ticks.iter().any(|&n| !valid(n)) {
            return Err(ModelError::InvalidConnection);
        }
        let outer = rectangle(&area.outer.segments, p)?;
        let mut holes = Vec::with_capacity(area.cutouts.len());
        for cut in &area.cutouts {
            let seg = &cut.boundary.segments;
            let hole = if seg
                .iter()
                .all(|s| matches!(s, BoundarySegment::Line { .. }))
            {
                Hole::Rect(rectangle(seg, p)?)
            } else if seg.iter().all(|s| matches!(s, BoundarySegment::Arc { .. })) {
                let (c, r) = circle(seg, p)?;
                Hole::Circle(c, r)
            } else {
                return Err(ModelError::InvalidConnection);
            };
            if !strict_inside(hole, outer) || holes.iter().copied().any(|h| !disjoint(h, hole)) {
                return Err(ModelError::InvalidConnection);
            }
            holes.push(hole);
        }
        Ok(p.ticks[0] > outer[0]
            && p.ticks[0] < outer[2]
            && p.ticks[1] > outer[1]
            && p.ticks[1] < outer[3]
            && !holes.into_iter().any(|h| in_hole(h, p.ticks)))
    })())
}
