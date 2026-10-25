//! A_026 isolated exact source-tick predicates for circular copper voids.
//! This module is intentionally NOT wired into production parse/Qt until
//! Canonical, lossless IPC and CPU/RHI hollow-mesh gates all pass.
//! PADS V9.4 MILS input source tick = 0.01 mil. Not a physical SI model.

const SOURCE_BOUND: i64 = 5_600_000;
type Point = [i64; 2];

fn bounded(p: Point) -> bool {
    p.iter().all(|v| (-SOURCE_BOUND..=SOURCE_BOUND).contains(v))
}

fn ring_valid_input(ring: &[Point]) -> bool {
    (4..=65).contains(&ring.len())
        && ring.first() == ring.last()
        && ring.iter().copied().all(bounded)
}

fn cross(a: Point, b: Point, p: Point) -> i128 {
    let dx = i128::from(b[0]) - i128::from(a[0]);
    let dy = i128::from(b[1]) - i128::from(a[1]);
    let px = i128::from(p[0]) - i128::from(a[0]);
    let py = i128::from(p[1]) - i128::from(a[1]);
    dx * py - dy * px
}

fn squared_distance(a: Point, b: Point) -> i128 {
    let x = i128::from(a[0]) - i128::from(b[0]);
    let y = i128::from(a[1]) - i128::from(b[1]);
    x * x + y * y
}

fn on_segment(a: Point, b: Point, p: Point) -> bool {
    cross(a, b, p) == 0
        && (a[0].min(b[0])..=a[0].max(b[0])).contains(&p[0])
        && (a[1].min(b[1])..=a[1].max(b[1])).contains(&p[1])
}

/// Exact open-interior ray crossing; ring simplicity is checked separately
/// by the existing polygon_topology oracle at integration.
fn strictly_inside(ring: &[Point], p: Point) -> bool {
    let mut inside = false;
    for edge in ring.windows(2) {
        let a = edge[0];
        let b = edge[1];
        if on_segment(a, b, p) {
            return false;
        }
        if (a[1] > p[1]) != (b[1] > p[1]) {
            let side = cross(a, b, p);
            if (side > 0) == (b[1] > a[1]) {
                inside = !inside;
            }
        }
    }
    inside
}

/// True only when an entire finite edge is strictly outside the closed
/// source circle. No squared roots, floating point or polygon sampling.
fn segment_clear(a: Point, b: Point, center: Point, radius: i64) -> bool {
    let abx = i128::from(b[0]) - i128::from(a[0]);
    let aby = i128::from(b[1]) - i128::from(a[1]);
    let acx = i128::from(center[0]) - i128::from(a[0]);
    let acy = i128::from(center[1]) - i128::from(a[1]);
    let len2 = abx * abx + aby * aby;
    if len2 == 0 {
        return false;
    }
    let proj = acx * abx + acy * aby;
    let r2 = i128::from(radius) * i128::from(radius);
    if proj <= 0 {
        squared_distance(a, center) > r2
    } else if proj >= len2 {
        squared_distance(b, center) > r2
    } else {
        let area2 = abx * acy - aby * acx;
        area2 * area2 > r2 * len2
    }
}

/// Check a strictly interior circle within a simple closed polygon.
/// A concave outer outline is valid; any tangent/contact is rejected.
/// Caller MUST independently establish that the polygon is simple.
pub fn circle_strictly_in_polygon(outer: &[[i64; 2]], center: [i64; 2], radius: i64) -> bool {
    if !ring_valid_input(outer)
        || !bounded(center)
        || !(1..=SOURCE_BOUND).contains(&radius)
        || !strictly_inside(outer, center)
    {
        return false;
    }
    outer
        .windows(2)
        .all(|edge| segment_clear(edge[0], edge[1], center, radius))
}

/// Neither a polygonal hole enclosing the source circle nor any circle
/// crossing or contacting a polygonal hole is allowed.
/// Caller MUST independently establish that the polygon is simple.
pub fn circle_strictly_disjoint_from_polygon(
    polygon_hole: &[[i64; 2]],
    center: [i64; 2],
    radius: i64,
) -> bool {
    if !ring_valid_input(polygon_hole)
        || !bounded(center)
        || !(1..=SOURCE_BOUND).contains(&radius)
        || strictly_inside(polygon_hole, center)
    {
        return false;
    }
    polygon_hole
        .windows(2)
        .all(|edge| segment_clear(edge[0], edge[1], center, radius))
}

/// Circular copper source area minus a circular void, strictly bounded.
/// Exact squared-distance comparison also forbids internally tangent voids.
pub fn circle_strictly_in_circle(
    outer_center: [i64; 2],
    outer_radius: i64,
    hole_center: [i64; 2],
    hole_radius: i64,
) -> bool {
    if !bounded(outer_center)
        || !bounded(hole_center)
        || !(1..=SOURCE_BOUND).contains(&outer_radius)
        || !(1..=SOURCE_BOUND).contains(&hole_radius)
        || outer_radius <= hole_radius
    {
        return false;
    }
    let clearance = i128::from(outer_radius - hole_radius);
    squared_distance(outer_center, hole_center) < clearance * clearance
}

/// Strictly disjoint circular voids, including non-touching requirement.
pub fn circles_strictly_disjoint(
    first_center: [i64; 2],
    first_radius: i64,
    second_center: [i64; 2],
    second_radius: i64,
) -> bool {
    if !bounded(first_center)
        || !bounded(second_center)
        || !(1..=SOURCE_BOUND).contains(&first_radius)
        || !(1..=SOURCE_BOUND).contains(&second_radius)
    {
        return false;
    }
    let span = i128::from(first_radius) + i128::from(second_radius);
    squared_distance(first_center, second_center) > span * span
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(x0: i64, y0: i64, x1: i64, y1: i64) -> Vec<Point> {
        vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1], [x0, y0]]
    }

    #[test]
    fn polygon_accepts_strict_circle_and_refuses_tangency() {
        let outer = square(0, 0, 100, 100);
        assert!(circle_strictly_in_polygon(&outer, [50, 50], 49));
        assert!(!circle_strictly_in_polygon(&outer, [50, 50], 50));
        assert!(!circle_strictly_in_polygon(&outer, [99, 50], 2));
        assert!(!circle_strictly_in_polygon(&outer, [150, 50], 2));
        assert!(!circle_strictly_in_polygon(&outer, [50, 50], 0));
    }

    #[test]
    fn disjoint_polygon_holes_require_strict_clearance() {
        let hole = square(20, 20, 40, 40);
        assert!(circle_strictly_disjoint_from_polygon(&hole, [60, 60], 5));
        assert!(!circle_strictly_disjoint_from_polygon(&hole, [45, 30], 5));
        assert!(!circle_strictly_disjoint_from_polygon(&hole, [30, 30], 2));
        assert!(!circle_strictly_disjoint_from_polygon(
            &square(0, 0, 100, 100),
            [50, 50],
            10
        ));
    }

    #[test]
    fn diagonal_segment_is_not_approximated_by_box() {
        let triangle = vec![[0, 0], [100, 0], [0, 100], [0, 0]];
        assert!(circle_strictly_in_polygon(&triangle, [20, 20], 10));
        assert!(!circle_strictly_in_polygon(&triangle, [49, 49], 3));
        let corner = vec![[10, 10], [90, 10], [10, 90], [10, 10]];
        assert!(!circle_strictly_disjoint_from_polygon(&corner, [20, 20], 4));
    }

    #[test]
    fn circular_pairs_reject_exact_tangency_and_overlap() {
        assert!(circle_strictly_in_circle([0, 0], 100, [10, 0], 89));
        assert!(!circle_strictly_in_circle([0, 0], 100, [10, 0], 90));
        assert!(!circle_strictly_in_circle([0, 0], 100, [0, 0], 100));
        assert!(circles_strictly_disjoint([0, 0], 10, [21, 0], 10));
        assert!(!circles_strictly_disjoint([0, 0], 10, [20, 0], 10));
        assert!(!circles_strictly_disjoint([0, 0], 10, [19, 0], 10));
    }

    #[test]
    fn malformed_bounds_and_closure_fail_closed() {
        let outer = square(0, 0, 100, 100);
        let unclosed = vec![[0, 0], [100, 0], [100, 100], [0, 100]];
        assert!(!circle_strictly_in_polygon(&unclosed, [50, 50], 1));
        assert!(!circle_strictly_in_polygon(
            &outer,
            [SOURCE_BOUND + 1, 50],
            1
        ));
        assert!(!circles_strictly_disjoint([i64::MAX, 0], 1, [0, 0], 1));
        assert!(!circle_strictly_in_circle([0, 0], 2, [0, 0], 0));
    }
}
