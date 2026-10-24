//! Restricted, fail-closed curved BOARD outline topology.
//! Exact integer predicates for quarter-circle/line tests. Arc/arc boxes
//! overlapping away from a single common endpoint are conservatively refused:
//! this is a documented subset, NOT general exact arbitrary-arc CAD geometry.
use crate::outline_arc::CardinalArcCorner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Segment {
    Line([i64; 2], [i64; 2]),
    Arc(CardinalArcCorner),
}
impl Segment {
    pub fn ends(self) -> ([i64; 2], [i64; 2]) {
        match self {
            Self::Line(a, b) => (a, b),
            Self::Arc(a) => (a.start, a.end),
        }
    }
    fn bbox(self) -> ([i64; 2], [i64; 2]) {
        let (a, b) = self.ends();
        ([a[0].min(b[0]), a[1].min(b[1])],
         [a[0].max(b[0]), a[1].max(b[1])])
    }
}
fn cross(a: [i64; 2], b: [i64; 2], c: [i64; 2]) -> i128 {
    (i128::from(b[0]) - i128::from(a[0]))
        * (i128::from(c[1]) - i128::from(a[1]))
        - (i128::from(b[1]) - i128::from(a[1]))
        * (i128::from(c[0]) - i128::from(a[0]))
}
fn between(a: [i64; 2], b: [i64; 2], p: [i64; 2]) -> bool {
    cross(a, b, p) == 0
        && (a[0].min(b[0])..=a[0].max(b[0])).contains(&p[0])
        && (a[1].min(b[1])..=a[1].max(b[1])).contains(&p[1])
}
fn line_touch(a: [i64; 2], b: [i64; 2], c: [i64; 2], d: [i64; 2]) -> bool {
    let (x, y, z, w) = (cross(a, b, c), cross(a, b, d),
                       cross(c, d, a), cross(c, d, b));
    if (x == 0 && between(a, b, c))
        || (y == 0 && between(a, b, d))
        || (z == 0 && between(c, d, a))
        || (w == 0 && between(c, d, b)) { return true; }
    (x > 0) != (y > 0) && (z > 0) != (w > 0)
}
fn overlap(a: ([i64; 2], [i64; 2]), b: ([i64; 2], [i64; 2])) -> bool {
    (0..2).all(|k| a.0[k] <= b.1[k] && b.0[k] <= a.1[k])
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Rat { n: i128, d: i128 }
impl Rat {
    fn new(mut n: i128, mut d: i128) -> Self {
        if d < 0 { n = -n; d = -d; }
        Self { n, d }
    }
    fn lt(self, other: Self) -> bool { self.n * other.d < other.n * self.d }
    fn gt(self, other: Self) -> bool { other.lt(self) }
}
fn arc_line_touch(arc: CardinalArcCorner, a: [i64; 2], b: [i64; 2],
                  shared: Option<[i64; 2]>) -> bool {
    // Clip the whole line to the exact cardinal-quarter bounding rectangle
    // using rational parameters. All discriminants use bounded i128 integers.
    let lo = [arc.start[0].min(arc.end[0]), arc.start[1].min(arc.end[1])];
    let hi = [arc.start[0].max(arc.end[0]), arc.start[1].max(arc.end[1])];
    let mut lower = Rat::new(0, 1);
    let mut upper = Rat::new(1, 1);
    for k in 0..2 {
        let dir = i128::from(b[k]) - i128::from(a[k]);
        if dir == 0 {
            if a[k] < lo[k] || a[k] > hi[k] { return false; }
            continue;
        }
        let mut t0 = Rat::new(i128::from(lo[k]) - i128::from(a[k]), dir);
        let mut t1 = Rat::new(i128::from(hi[k]) - i128::from(a[k]), dir);
        if t1.lt(t0) { std::mem::swap(&mut t0, &mut t1); }
        if t0.gt(lower) { lower = t0; }
        if t1.lt(upper) { upper = t1; }
        if upper.lt(lower) { return false; }
    }
    let x = i128::from(a[0]) - i128::from(arc.centre[0]);
    let y = i128::from(a[1]) - i128::from(arc.centre[1]);
    let vx = i128::from(b[0]) - i128::from(a[0]);
    let vy = i128::from(b[1]) - i128::from(a[1]);
    let r = i128::from(arc.start[0] - arc.centre[0]).abs()
        .max(i128::from(arc.start[1] - arc.centre[1]).abs());
    let f = |t: Rat| {
        let xx = x * t.d + vx * t.n;
        let yy = y * t.d + vy * t.n;
        xx * xx + yy * yy - (r * t.d) * (r * t.d)
    };
    let point_is_shared = |t: Rat| {
        shared.map(|p| {
            (i128::from(p[0]) - i128::from(a[0])) * t.d == vx * t.n
                && (i128::from(p[1]) - i128::from(a[1])) * t.d == vy * t.n
        }).unwrap_or(false)
    };
    if lower == upper { return f(lower) == 0 && !point_is_shared(lower); }
    let left = f(lower);
    let right = f(upper);
    if left < 0 && right < 0 { return false; }
    let norm = vx * vx + vy * vy;
    if norm == 0 { return left == 0 && !point_is_shared(lower); }
    let tmin = Rat::new(-(x * vx + y * vy), norm);
    let min_inside = tmin.gt(lower) && tmin.lt(upper);
    let fmin = (x * x + y * y - r * r) * norm - (x * vx + y * vy) * (x * vx + y * vy);
    let min_negative = min_inside && fmin < 0;
    if left > 0 && right > 0 {
        if min_negative { return true; }
        return min_inside && fmin == 0 && !point_is_shared(tmin);
    }
    if left == 0 && right == 0 { return true; }
    if left == 0 && right > 0 { return min_negative || !point_is_shared(lower); }
    if right == 0 && left > 0 { return min_negative || !point_is_shared(upper); }
    if left == 0 { return !point_is_shared(lower); }
    if right == 0 { return !point_is_shared(upper); }
    true
}
fn touches(a: Segment, b: Segment, shared: Option<[i64; 2]>) -> bool {
    if !overlap(a.bbox(), b.bbox()) { return false; }
    match (a, b) {
        (Segment::Line(x, y), Segment::Line(u, v)) => {
            if let Some(p) = shared {
                // The shared point must not lie in the interior of either line
                // if collinear. Crossings away from it are still forbidden.
                if cross(x, y, u) == 0 && cross(x, y, v) == 0 {
                    let other_a = if x == p { y } else { x };
                    let other_b = if u == p { v } else { u };
                    return between(p, other_a, other_b)
                        || between(p, other_b, other_a);
                }
                return false;
            }
            line_touch(x, y, u, v)
        }
        (Segment::Arc(x), Segment::Line(u, v))
        | (Segment::Line(u, v), Segment::Arc(x)) => arc_line_touch(x, u, v, shared),
        (Segment::Arc(x), Segment::Arc(y)) => {
            if let Some(p) = shared {
                let (al, ah) = Segment::Arc(x).bbox();
                let (bl, bh) = Segment::Arc(y).bbox();
                let lo = [al[0].max(bl[0]), al[1].max(bl[1])];
                let hi = [ah[0].min(bh[0]), ah[1].min(bh[1])];
                return lo != p || hi != p;
            }
            // Conservative refusal when quarter-circle boxes overlap.
            true
        }
    }
}
pub fn ring_valid(ring: &[Segment]) -> bool {
    if ring.len() < 3 || ring.len() > 256 { return false; }
    for (i, &s) in ring.iter().enumerate() {
        let (start, end) = s.ends();
        if start == end || end != ring[(i + 1) % ring.len()].ends().0 { return false; }
        for (j, &other) in ring.iter().enumerate().skip(i + 1) {
            let adjacent = j == i + 1 || (i == 0 && j == ring.len() - 1);
            let shared = if adjacent {
                Some(if j == i + 1 { end } else { start })
            } else { None };
            if touches(s, other, shared) { return false; }
        }
    }
    // Chord signed area zero cannot establish a valid oriented interior.
    let mut twice_area = 0i128;
    for &seg in ring {
        let (a, b) = seg.ends();
        twice_area += i128::from(a[0]) * i128::from(b[1])
            - i128::from(b[0]) * i128::from(a[1]);
    }
    twice_area != 0
}
fn on_arc(p: [i64; 2], a: CardinalArcCorner) -> bool {
    let dx = i128::from(p[0] - a.centre[0]);
    let dy = i128::from(p[1] - a.centre[1]);
    let rx = i128::from(a.start[0] - a.centre[0]);
    let ry = i128::from(a.start[1] - a.centre[1]);
    let r2 = rx * rx + ry * ry;
    let (lo, hi) = Segment::Arc(a).bbox();
    p[0] >= lo[0] && p[0] <= hi[0] && p[1] >= lo[1] && p[1] <= hi[1]
        && dx * dx + dy * dy == r2
}
pub fn inside_strict(p: [i64; 2], ring: &[Segment]) -> bool {
    let mut parity = false;
    for &segment in ring {
        match segment {
            Segment::Line(a, b) => {
                if between(a, b, p) { return false; }
                if (a[1] > p[1]) != (b[1] > p[1]) {
                    let v = cross(a, b, p);
                    if (v > 0) == (b[1] > a[1]) { parity = !parity; }
                }
            }
            Segment::Arc(arc) => {
                if on_arc(p, arc) { return false; }
                let (lo, hi) = segment.bbox();
                // Same y-half-open convention as line ray parity.
                if p[1] < lo[1] || p[1] >= hi[1] { continue; }
                let dy = i128::from(p[1] - arc.centre[1]);
                let r = i128::from(arc.start[0] - arc.centre[0]).abs()
                    .max(i128::from(arc.start[1] - arc.centre[1]).abs());
                let square = r * r - dy * dy;
                let dx = i128::from(p[0] - arc.centre[0]);
                let right = hi[0] > arc.centre[0];
                let to_right = if right {
                    dx < 0 || dx * dx < square
                } else {
                    dx < 0 && dx * dx > square
                };
                if to_right { parity = !parity; }
            }
        }
    }
    parity
}
pub fn strictly_contains(outer: &[Segment], cut: &[Segment]) -> bool {
    cut.iter().all(|s| inside_strict(s.ends().0, outer))
        && outer.iter().all(|&a| cut.iter().all(|&b| !touches(a, b, None)))
}
pub fn overlap_or_contain(a: &[Segment], b: &[Segment]) -> bool {
    a.iter().any(|&x| b.iter().any(|&y| touches(x, y, None)))
        || a.iter().any(|s| inside_strict(s.ends().0, b))
        || b.iter().any(|s| inside_strict(s.ends().0, a))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn arc(a:[i64;2], b:[i64;2], c:[i64;2]) -> Segment {
        Segment::Arc(CardinalArcCorner{start:a,end:b,centre:c,clockwise:false})
    }
    #[test]
    fn quarter_arc_line_contact_only_at_expected_shared_endpoint() {
        let a = arc([30,0],[40,10],[30,10]);
        assert!(!touches(a,Segment::Line([10,0],[30,0]),Some([30,0])));
        assert!(touches(a,Segment::Line([10,10],[40,10]),None));
        assert!(touches(a,Segment::Line([40,0],[40,20]),None));
    }
    #[test]
    fn rounded_rectangle_is_valid_with_exact_arc_boundary() {
        let ring = [
            Segment::Line([10,0],[30,0]),
            arc([30,0],[40,10],[30,10]),
            Segment::Line([40,10],[40,20]),
            arc([40,20],[30,30],[30,20]),
            Segment::Line([30,30],[10,30]),
            arc([10,30],[0,20],[10,20]),
            Segment::Line([0,20],[0,10]),
            arc([0,10],[10,0],[10,10]),
        ];
        assert!(ring_valid(&ring));
        assert!(inside_strict([20,15], &ring));
        assert!(!inside_strict([0,0], &ring));
        let square = [Segment::Line([15,10],[25,10]),Segment::Line([25,10],[25,20]),
                      Segment::Line([25,20],[15,20]),Segment::Line([15,20],[15,10])];
        assert!(strictly_contains(&ring,&square));
        assert!(overlap_or_contain(&ring,&square));
    }
}
