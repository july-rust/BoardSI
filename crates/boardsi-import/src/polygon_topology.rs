//! A_025 P2-P: exact-source-tick polygon topology preflight.
//!
//! This isolated validator does NOT enable diagonal COPCUT import or rendering.
//! The existing orthogonal importer remains fail-closed until a matching,
//! independently validated Qt CPU/RHI tessellator and picking path exist.
//! All predicates and signed areas use i128; no approximate containment.

pub type TickPoint = [i64; 2];

const MAX_HOLES: usize = 8;
const MAX_VERTICES_TOTAL: usize = 128;
const MAX_RING_VERTICES: usize = 65;
const MAX_SOURCE_TICKS: i64 = 5_600_000; // Match PADS V9.4 restricted importer.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologyError {
    Complexity,
    UnclosedOrDegenerate,
    SelfIntersection,
    BoundaryContact,
    HoleOutside,
    NestedHoles,
    EmptyFill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AreaProof {
    /// Twice the exact polygon source-tick area.
    pub outer_area2: i128,
    pub hole_area2: i128,
    pub filled_area2: i128,
}

fn orient(a: TickPoint, b: TickPoint, c: TickPoint) -> i128 {
    let dx1 = i128::from(b[0]) - i128::from(a[0]);
    let dy1 = i128::from(b[1]) - i128::from(a[1]);
    let dx2 = i128::from(c[0]) - i128::from(a[0]);
    let dy2 = i128::from(c[1]) - i128::from(a[1]);
    dx1 * dy2 - dy1 * dx2
}

fn on_segment(a: TickPoint, b: TickPoint, p: TickPoint) -> bool {
    orient(a, b, p) == 0
        && (a[0].min(b[0])..=a[0].max(b[0])).contains(&p[0])
        && (a[1].min(b[1])..=a[1].max(b[1])).contains(&p[1])
}

/// True for crossing, tangency and collinear overlap; contact is forbidden
/// between independent contours because the fill would be ambiguous.
fn segments_touch(a: TickPoint, b: TickPoint, c: TickPoint, d: TickPoint) -> bool {
    let x = orient(a, b, c);
    let y = orient(a, b, d);
    let z = orient(c, d, a);
    let w = orient(c, d, b);
    if (x == 0 && on_segment(a, b, c))
        || (y == 0 && on_segment(a, b, d))
        || (z == 0 && on_segment(c, d, a))
        || (w == 0 && on_segment(c, d, b))
    {
        return true;
    }
    (x > 0) != (y > 0) && (z > 0) != (w > 0)
}

fn signed_area2(ring: &[TickPoint]) -> i128 {
    ring.windows(2)
        .map(|pair| {
            i128::from(pair[0][0]) * i128::from(pair[1][1])
                - i128::from(pair[1][0]) * i128::from(pair[0][1])
        })
        .sum()
}

fn simple_ring_area2(ring: &[TickPoint]) -> Result<i128, TopologyError> {
    if !(4..=MAX_RING_VERTICES).contains(&ring.len()) || ring.first() != ring.last() {
        return Err(TopologyError::UnclosedOrDegenerate);
    }
    let edges = ring.len() - 1;
    for i in 0..edges {
        let prev = ring[(i + edges - 1) % edges];
        let curr = ring[i];
        let next = ring[i + 1];
        // Reject repeated points, spikes and redundant collinear vertices
        // before a future tessellator can silently discard source geometry.
        if curr == next || orient(prev, curr, next) == 0 {
            return Err(TopologyError::UnclosedOrDegenerate);
        }
        for j in (i + 1)..edges {
            if j == i + 1 || (i == 0 && j == edges - 1) {
                continue; // Neighbouring edges share one intentional vertex.
            }
            if segments_touch(ring[i], ring[i + 1], ring[j], ring[j + 1]) {
                return Err(TopologyError::SelfIntersection);
            }
        }
    }
    let area2 = signed_area2(ring).abs();
    if area2 == 0 {
        return Err(TopologyError::UnclosedOrDegenerate);
    }
    Ok(area2)
}

/// An exact odd-even ray test. A point on any boundary is NOT strictly inside.
/// The winding of the input ring does not affect the result.
fn strictly_inside(point: TickPoint, ring: &[TickPoint]) -> bool {
    let mut inside = false;
    for e in ring.windows(2) {
        let (a, b) = (e[0], e[1]);
        if on_segment(a, b, point) {
            return false;
        }
        if (a[1] > point[1]) != (b[1] > point[1]) {
            let side = orient(a, b, point);
            if (b[1] > a[1] && side > 0) || (b[1] < a[1] && side < 0) {
                inside = !inside;
            }
        }
    }
    inside
}

fn any_contact(a: &[TickPoint], b: &[TickPoint]) -> bool {
    a.windows(2)
        .any(|e| b.windows(2).any(|f| segments_touch(e[0], e[1], f[0], f[1])))
}

/// Validates simple non-orthogonal rings with strictly disjoint, unnested holes.
/// Does not triangulate or modify geometry; the returned integer area is an
/// independent oracle for subsequent source-to-mesh acceptance tests.
pub fn validate_polygon_with_holes(
    outer: &[TickPoint],
    holes: &[Vec<TickPoint>],
) -> Result<AreaProof, TopologyError> {
    // Gate complexity and source range before any i128 products.
    if holes.len() > MAX_HOLES || outer.len() > MAX_RING_VERTICES {
        return Err(TopologyError::Complexity);
    }
    let mut total = outer.len();
    for ring in std::iter::once(outer).chain(holes.iter().map(Vec::as_slice)) {
        if ring.len() > MAX_RING_VERTICES
            || ring
                .iter()
                .flatten()
                .any(|v| !(-MAX_SOURCE_TICKS..=MAX_SOURCE_TICKS).contains(v))
        {
            return Err(TopologyError::Complexity);
        }
        if !std::ptr::eq(ring, outer) {
            total = total
                .checked_add(ring.len())
                .ok_or(TopologyError::Complexity)?;
        }
    }
    if total > MAX_VERTICES_TOTAL {
        return Err(TopologyError::Complexity);
    }
    let outer_area2 = simple_ring_area2(outer)?;
    let mut hole_area2 = 0i128;
    for (i, hole) in holes.iter().enumerate() {
        let area2 = simple_ring_area2(hole)?;
        if any_contact(outer, hole) {
            return Err(TopologyError::BoundaryContact);
        }
        if !strictly_inside(hole[0], outer) {
            return Err(TopologyError::HoleOutside);
        }
        for previous in &holes[..i] {
            if any_contact(previous, hole) {
                return Err(TopologyError::BoundaryContact);
            }
            if strictly_inside(hole[0], previous) || strictly_inside(previous[0], hole) {
                return Err(TopologyError::NestedHoles);
            }
        }
        hole_area2 += area2;
    }
    if outer_area2 <= hole_area2 {
        return Err(TopologyError::EmptyFill);
    }
    Ok(AreaProof {
        outer_area2,
        hole_area2,
        filled_area2: outer_area2 - hole_area2,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn closed(mut p: Vec<TickPoint>) -> Vec<TickPoint> {
        p.push(p[0]);
        p
    }

    #[test]
    fn diagonal_outer_and_cutouts_have_exact_area_and_winding_parity() {
        let outer = closed(vec![[0, 0], [20, 0], [10, 20]]);
        let hole = closed(vec![[9, 5], [11, 5], [10, 7]]);
        let a = validate_polygon_with_holes(&outer, std::slice::from_ref(&hole)).unwrap();
        assert_eq!(a.outer_area2, 400);
        assert_eq!(a.hole_area2, 4);
        assert_eq!(a.filled_area2, 396);
        let reversed_outer = closed(outer[..outer.len() - 1].iter().rev().copied().collect());
        let reversed_hole = closed(hole[..hole.len() - 1].iter().rev().copied().collect());
        assert_eq!(
            validate_polygon_with_holes(&reversed_outer, &[reversed_hole]),
            Ok(a)
        );
    }

    #[test]
    fn concave_diagonal_outer_and_multiple_holes() {
        let outer = closed(vec![
            [0, 0],
            [40, 0],
            [40, 40],
            [25, 40],
            [20, 28],
            [15, 40],
            [0, 40],
        ]);
        let hole_a = closed(vec![[5, 5], [10, 5], [10, 10], [5, 10]]);
        let hole_b = closed(vec![[27, 10], [32, 10], [30, 14]]);
        let proof = validate_polygon_with_holes(&outer, &[hole_a, hole_b]).unwrap();
        assert!(proof.filled_area2 > 0);
        assert_eq!(proof.filled_area2, proof.outer_area2 - proof.hole_area2);
    }

    #[test]
    fn rejects_self_crossing_touching_outside_and_nested_holes() {
        let outer = closed(vec![[0, 0], [20, 0], [20, 20], [0, 20]]);
        let crossing = closed(vec![[3, 3], [10, 10], [3, 10], [10, 3]]);
        assert_eq!(
            validate_polygon_with_holes(&outer, &[crossing]),
            Err(TopologyError::SelfIntersection)
        );
        let touching = closed(vec![[0, 4], [5, 4], [4, 8]]);
        assert_eq!(
            validate_polygon_with_holes(&outer, &[touching]),
            Err(TopologyError::BoundaryContact)
        );
        let outside = closed(vec![[21, 3], [24, 3], [22, 6]]);
        assert_eq!(
            validate_polygon_with_holes(&outer, &[outside]),
            Err(TopologyError::HoleOutside)
        );
        let big = closed(vec![[2, 2], [15, 2], [15, 15], [2, 15]]);
        let nested = closed(vec![[4, 4], [6, 4], [5, 6]]);
        assert_eq!(
            validate_polygon_with_holes(&outer, &[big, nested]),
            Err(TopologyError::NestedHoles)
        );
    }

    #[test]
    fn rejects_overlap_redundant_points_and_complexity() {
        let outer = closed(vec![[0, 0], [20, 0], [20, 20], [0, 20]]);
        let a = closed(vec![[2, 2], [12, 2], [10, 12]]);
        let b = closed(vec![[9, 5], [17, 5], [15, 15]]);
        assert_eq!(
            validate_polygon_with_holes(&outer, &[a, b]),
            Err(TopologyError::BoundaryContact)
        );
        let degenerate = closed(vec![[0, 0], [10, 0], [15, 0], [10, 20]]);
        assert_eq!(
            validate_polygon_with_holes(&degenerate, &[]),
            Err(TopologyError::UnclosedOrDegenerate)
        );
        let small = closed(vec![[2, 2], [3, 2], [2, 3]]);
        assert_eq!(
            validate_polygon_with_holes(&outer, &vec![small; 9]),
            Err(TopologyError::Complexity)
        );
    }

    #[test]
    fn rejects_extreme_coordinates_before_exact_predicates() {
        let extreme = closed(vec![[i64::MIN, 0], [i64::MAX, 1], [0, i64::MAX]]);
        assert_eq!(
            validate_polygon_with_holes(&extreme, &[]),
            Err(TopologyError::Complexity)
        );
    }

    #[test]
    fn rejects_nonadjacent_collinear_edge_contact_and_spikes() {
        let repeated = closed(vec![[0, 0], [10, 0], [10, 10], [10, 0], [0, 10]]);
        assert!(validate_polygon_with_holes(&repeated, &[]).is_err());
        let reversed = closed(vec![[0, 0], [0, 10], [10, 10], [10, 0]]);
        assert_eq!(
            validate_polygon_with_holes(&reversed, &[])
                .unwrap()
                .filled_area2,
            200
        );
    }
}
