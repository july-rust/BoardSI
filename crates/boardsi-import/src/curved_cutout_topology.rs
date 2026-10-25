//! A_030 / P2-U — exact source-tick containment gate for a single convex
//! straight COPCUT in a previously validated A_029 convex outward-arc COPCLS.
//! Rust source arcs remain analytical. Only the conservative, inscribed A_029
//! polygon is used for these extra display-clearance checks.
//! A pass is NOT an electrical connection or a flood computation.
use crate::polygon_topology::validate_disjoint_polygon_holes;

pub type TickPoint = [i64; 2];
const MAX_OUTER: usize = 65;
const MAX_HOLE: usize = 65;
const MAX_COMBINED: usize = 128;
const LIMIT: i64 = 5_600_000;

fn cross(a: TickPoint, b: TickPoint, c: TickPoint) -> i128 {
    let x = i128::from(b[0]) - i128::from(a[0]);
    let y = i128::from(b[1]) - i128::from(a[1]);
    x * (i128::from(c[1]) - i128::from(a[1]))
        - y * (i128::from(c[0]) - i128::from(a[0]))
}
fn area2(ring: &[TickPoint]) -> i128 {
    ring.windows(2).map(|e|
        i128::from(e[0][0]) * i128::from(e[1][1])
        - i128::from(e[1][0]) * i128::from(e[0][1])
    ).sum()
}

/// Validate only an explicitly source-owned single straight convex hole.
/// The caller MUST have already validated `outer_display` with A_029's
/// exact analytical outer + inward conservative preview oracle.
pub fn validate(outer_display: &[TickPoint], hole: &[TickPoint]) -> Result<(), &'static str> {
    if !(4..=MAX_OUTER).contains(&outer_display.len())
        || !(4..=MAX_HOLE).contains(&hole.len())
        || outer_display.len() + hole.len() > MAX_COMBINED
        || outer_display.first() != outer_display.last()
        || hole.first() != hole.last()
    { return Err("CURVED_CUTOUT_COMPLEXITY_OR_CLOSURE"); }
    if outer_display.iter().chain(hole).flat_map(|p| p.iter())
        .any(|v| !(-LIMIT..=LIMIT).contains(v))
    { return Err("CURVED_CUTOUT_RANGE"); }
    // Validated original source ring contains any segment of this strictly
    // convex display polygon; collinear proxy vertices are permitted.
    let direction = area2(outer_display).signum();
    if direction == 0 { return Err("CURVED_CUTOUT_OUTER_DEGENERATE"); }
    let edges = outer_display.len() - 1;
    for i in 0..edges {
        let a=outer_display[i];
        let b=outer_display[i+1];
        let c=outer_display[(i+2)%edges];
        if a==b || cross(a,b,c)*direction < 0 {
            return Err("CURVED_CUTOUT_OUTER_NOT_CONVEX");
        }
    }
    // The already-independent A_025 exact polygon oracle excludes repeated
    // hole vertices, zero-area corners, self-intersections and invalid range.
    validate_disjoint_polygon_holes(&[hole.to_vec()])
        .map_err(|_| "CURVED_CUTOUT_INVALID_HOLE")?;
    let hd=area2(hole).signum();
    if hd==0 { return Err("CURVED_CUTOUT_INVALID_HOLE"); }
    let hn=hole.len()-1;
    for i in 0..hn {
        if cross(hole[i],hole[(i+1)%hn],hole[(i+2)%hn])*hd <= 0 {
            return Err("CURVED_CUTOUT_NONCONVEX_HOLE");
        }
    }
    // On a convex outer, checking every vertex against every strictly
    // inward oriented half-plane also certifies *all* hole edges stay
    // strictly within the conservative display polygon. No sampling.
    for p in &hole[..hn] {
        for e in outer_display.windows(2) {
            if cross(e[0],e[1],*p)*direction <= 0 {
                return Err("CURVED_CUTOUT_OUTSIDE_OR_TOUCHING_PROXY");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn close(mut v:Vec<TickPoint>)->Vec<TickPoint> {
        v.push(v[0]);v
    }
    fn outer()->Vec<TickPoint> {
        close(vec![[0,0],[1000,0],[2000,0],[2000,2000],[0,2000]])
    }
    fn hole()->Vec<TickPoint> {
        close(vec![[700,700],[1300,700],[1300,1300],[700,1300]])
    }
    #[test]
    fn accepts_only_strictly_inside_convex_polygon() {
        assert!(validate(&outer(),&hole()).is_ok());
        let mut reverse=hole();
        reverse.reverse();
        assert!(validate(&outer(),&reverse).is_ok());
        let mut reverse_outer=outer();
        reverse_outer.reverse();
        assert!(validate(&reverse_outer,&hole()).is_ok());
    }
    #[test]
    fn fail_closed_for_contact_exterior_concavity_and_repeated_vertex() {
        for bad in [
            close(vec![[0,700],[200,700],[200,900],[0,900]]),
            close(vec![[-10,700],[200,700],[200,900],[-10,900]]),
            close(vec![[700,700],[1300,700],[1000,900],[1300,1300],[700,1300]]),
            close(vec![[700,700],[1300,700],[1300,700],[700,1300]]),
        ] {
            assert!(validate(&outer(),&bad).is_err(),"{bad:?}");
        }
    }
    #[test]
    fn rejects_wrong_bounds_and_outer() {
        let mut invalid=outer();invalid[1]=invalid[0];
        assert!(validate(&invalid,&hole()).is_err());
        let mut invalid=hole();invalid[1]=[5_600_001,700];
        assert!(validate(&outer(),&invalid).is_err());
    }
}
