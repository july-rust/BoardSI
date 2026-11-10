//! D_056_08: exact bounded straight-edge ODB++ surface topology preflight.
//! Explicitly does not recognize circular OC contours, negative S or a fabricated Net.
//! All geometry uses source-owned 1e-9 units with bounded i128 predicates.
pub(crate) type Tick = [i64; 2];
const MAX_ABS: i64 = 1_000_000_000_000;
const MAX_VERTICES: usize = 512;
const MAX_HOLES: usize = 32;

#[derive(Debug, Default)]
pub(crate) struct SurfaceBuilder {
    outer: Option<Vec<Tick>>,
    holes: Vec<Vec<Tick>>,
    active: Option<(bool, Vec<Tick>)>,
}

fn orient(a: Tick, b: Tick, c: Tick) -> i128 {
    let ax = i128::from(b[0]) - i128::from(a[0]);
    let ay = i128::from(b[1]) - i128::from(a[1]);
    let bx = i128::from(c[0]) - i128::from(a[0]);
    let by = i128::from(c[1]) - i128::from(a[1]);
    // MAX_ABS bounds ensure all i128 products and subtractions cannot overflow.
    ax * by - ay * bx
}
fn on_segment(a: Tick, b: Tick, p: Tick) -> bool {
    orient(a, b, p) == 0
        && (a[0].min(b[0])..=a[0].max(b[0])).contains(&p[0])
        && (a[1].min(b[1])..=a[1].max(b[1])).contains(&p[1])
}
fn segments_touch(a: Tick, b: Tick, c: Tick, d: Tick) -> bool {
    let s1 = orient(a, b, c);
    let s2 = orient(a, b, d);
    let s3 = orient(c, d, a);
    let s4 = orient(c, d, b);
    if (s1 == 0 && on_segment(a, b, c))
        || (s2 == 0 && on_segment(a, b, d))
        || (s3 == 0 && on_segment(c, d, a))
        || (s4 == 0 && on_segment(c, d, b))
    {
        return true;
    }
    (s1 > 0) != (s2 > 0) && (s3 > 0) != (s4 > 0)
}
fn validate_ring(ring: &[Tick]) -> Result<i128, &'static str> {
    if ring.len() < 4 || ring.len() > MAX_VERTICES || ring.first() != ring.last() {
        return Err("ODB_SURFACE_UNCLOSED_OR_COMPLEX");
    }
    if ring
        .iter()
        .flat_map(|p| p.iter())
        .any(|v| !(-MAX_ABS..=MAX_ABS).contains(v))
    {
        return Err("ODB_SURFACE_COORDINATE_RANGE");
    }
    let edges = ring.len() - 1;
    let mut area2 = 0i128;
    for i in 0..edges {
        let a = ring[i];
        let b = ring[i + 1];
        if a == b || orient(ring[(i + edges - 1) % edges], a, b) == 0 {
            return Err("ODB_SURFACE_DEGENERATE_EDGE");
        }
        area2 += i128::from(a[0]) * i128::from(b[1]) - i128::from(b[0]) * i128::from(a[1]);
        for j in (i + 1)..edges {
            if j == i + 1 || (i == 0 && j == edges - 1) {
                continue;
            }
            if segments_touch(a, b, ring[j], ring[j + 1]) {
                return Err("ODB_SURFACE_SELF_INTERSECTION");
            }
        }
    }
    if area2 == 0 {
        return Err("ODB_SURFACE_ZERO_AREA");
    }
    Ok(area2.abs())
}
fn strictly_inside(point: Tick, ring: &[Tick]) -> bool {
    let mut inside = false;
    for edge in ring.windows(2) {
        let (a, b) = (edge[0], edge[1]);
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
fn rings_touch(a: &[Tick], b: &[Tick]) -> bool {
    a.windows(2)
        .any(|x| b.windows(2).any(|y| segments_touch(x[0], x[1], y[0], y[1])))
}

impl SurfaceBuilder {
    pub(crate) fn begin(&mut self, start: Tick, hole: bool) -> Result<(), &'static str> {
        if self.active.is_some() {
            return Err("ODB_SURFACE_NESTED_OB");
        }
        if hole && self.outer.is_none() {
            return Err("ODB_SURFACE_HOLE_BEFORE_ISLAND");
        }
        if !hole && self.outer.is_some() {
            // A single feature can carry disconnected islands; safely reject
            // until the Canonical mapper supports multi-island partitioning.
            return Err("ODB_SURFACE_MULTI_ISLAND_UNSUPPORTED");
        }
        self.active = Some((hole, vec![start]));
        Ok(())
    }
    pub(crate) fn line(&mut self, endpoint: Tick) -> Result<(), &'static str> {
        let Some((_, ring)) = self.active.as_mut() else {
            return Err("ODB_SURFACE_OS_WITHOUT_OB");
        };
        if ring.len() >= MAX_VERTICES {
            return Err("ODB_SURFACE_TOO_MANY_VERTICES");
        }
        ring.push(endpoint);
        Ok(())
    }
    pub(crate) fn close(&mut self) -> Result<(), &'static str> {
        let Some((hole, ring)) = self.active.take() else {
            return Err("ODB_SURFACE_OE_WITHOUT_OB");
        };
        validate_ring(&ring)?;
        if hole {
            if self.holes.len() >= MAX_HOLES {
                return Err("ODB_SURFACE_TOO_MANY_HOLES");
            }
            self.holes.push(ring);
        } else {
            self.outer = Some(ring);
        }
        Ok(())
    }
    pub(crate) fn finish(self) -> Result<(Vec<Tick>, Vec<Vec<Tick>>), &'static str> {
        if self.active.is_some() {
            return Err("ODB_SURFACE_MISSING_OE");
        }
        let outer = self.outer.ok_or("ODB_SURFACE_NO_ISLAND")?;
        let total = self.holes.iter().map(Vec::len).sum::<usize>() + outer.len();
        if total > MAX_VERTICES {
            return Err("ODB_SURFACE_TOTAL_VERTEX_LIMIT");
        }
        let outer_area2 = validate_ring(&outer)?;
        let mut hole_area2 = 0i128;
        for (i, hole) in self.holes.iter().enumerate() {
            hole_area2 += validate_ring(hole)?;
            if rings_touch(&outer, hole) || !strictly_inside(hole[0], &outer) {
                return Err("ODB_SURFACE_HOLE_OUTSIDE_OR_TOUCHING");
            }
            for previous in &self.holes[..i] {
                if rings_touch(previous, hole)
                    || strictly_inside(hole[0], previous)
                    || strictly_inside(previous[0], hole)
                {
                    return Err("ODB_SURFACE_HOLE_CONFLICT");
                }
            }
        }
        if hole_area2 >= outer_area2 {
            return Err("ODB_SURFACE_EMPTY_FILL");
        }
        Ok((outer, self.holes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_island_and_hole_exact_integer_topology() {
        let mut s = SurfaceBuilder::default();
        let m = 1_000_000_000;
        s.begin([0, 0], false).unwrap();
        for p in [[4 * m, 0], [4 * m, 4 * m], [0, 4 * m], [0, 0]] {
            s.line(p).unwrap();
        }
        s.close().unwrap();
        s.begin([m, m], true).unwrap();
        for p in [[m, 2 * m], [2 * m, 2 * m], [2 * m, m], [m, m]] {
            s.line(p).unwrap();
        }
        s.close().unwrap();
        let (outer, holes) = s.finish().unwrap();
        assert_eq!(outer.len(), 5);
        assert_eq!(holes.len(), 1);
    }
    #[test]
    fn rejects_bowtie_hole_outside_multi_island_and_missing_oe() {
        let mut bow = SurfaceBuilder::default();
        bow.begin([0, 0], false).unwrap();
        for p in [[3, 3], [0, 3], [3, 0], [0, 0]] {
            bow.line(p).unwrap();
        }
        assert_eq!(bow.close(), Err("ODB_SURFACE_SELF_INTERSECTION"));
        let mut incomplete = SurfaceBuilder::default();
        incomplete.begin([0, 0], false).unwrap();
        assert_eq!(incomplete.finish(), Err("ODB_SURFACE_MISSING_OE"));
        let mut multi = SurfaceBuilder::default();
        multi.begin([0, 0], false).unwrap();
        for p in [[3, 0], [3, 3], [0, 3], [0, 0]] {
            multi.line(p).unwrap();
        }
        multi.close().unwrap();
        assert_eq!(
            multi.begin([10, 10], false),
            Err("ODB_SURFACE_MULTI_ISLAND_UNSUPPORTED")
        );
        multi.begin([10, 10], true).unwrap();
        for p in [[11, 10], [11, 11], [10, 11], [10, 10]] {
            multi.line(p).unwrap();
        }
        multi.close().unwrap();
        assert_eq!(multi.finish(), Err("ODB_SURFACE_HOLE_OUTSIDE_OR_TOUCHING"));
    }
}
