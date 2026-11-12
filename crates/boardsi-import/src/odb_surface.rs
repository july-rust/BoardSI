//! D_056_08: exact bounded straight-edge ODB++ surface topology preflight.
//! Explicitly does not recognize circular OC contours, negative S or a fabricated Net.
//! All geometry uses source-owned 1e-9 units with bounded i128 predicates.
pub(crate) type Tick = [i64; 2];
const MAX_ABS: i64 = 1_000_000_000_000;
const MAX_VERTICES: usize = 512;
const MAX_HOLES: usize = 32;

const MAX_ISLANDS: usize = 16;

/// One source OB island. ob_line witnesses its independent source contour.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SurfaceIsland {
    pub(crate) outer: Vec<Tick>,
    pub(crate) holes: Vec<Vec<Tick>>,
    pub(crate) ob_line: usize,
}
#[derive(Debug, Default)]
pub(crate) struct SurfaceBuilder {
    islands: Vec<SurfaceIsland>,
    active: Option<(bool, Vec<Tick>, usize)>,
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
    pub(crate) fn begin(&mut self, start: Tick, hole: bool, ob_line: usize) -> Result<(), &'static str> {
        if self.active.is_some() {
            return Err("ODB_SURFACE_NESTED_OB");
        }
        if hole {
            if self.islands.is_empty() {
                return Err("ODB_SURFACE_HOLE_BEFORE_ISLAND");
            }
            if self.islands.len() > 1 {
                return Err("ODB_SURFACE_MULTI_ISLAND_HOLE_UNSUPPORTED");
            }
            if self.islands[0].holes.len() >= MAX_HOLES {
                return Err("ODB_SURFACE_TOO_MANY_HOLES");
            }
        } else {
            if self.islands.first().map(|v| !v.holes.is_empty()).unwrap_or(false) {
                return Err("ODB_SURFACE_MULTI_ISLAND_HOLE_UNSUPPORTED");
            }
            if self.islands.len() >= MAX_ISLANDS {
                return Err("ODB_SURFACE_TOO_MANY_ISLANDS");
            }
        }
        self.active = Some((hole, vec![start], ob_line));
        Ok(())
    }
    pub(crate) fn line(&mut self, endpoint: Tick) -> Result<(), &'static str> {
        let Some((_, ring, _)) = self.active.as_mut() else {
            return Err("ODB_SURFACE_OS_WITHOUT_OB");
        };
        if ring.len() >= MAX_VERTICES {
            return Err("ODB_SURFACE_TOO_MANY_VERTICES");
        }
        ring.push(endpoint);
        Ok(())
    }
    pub(crate) fn close(&mut self) -> Result<(), &'static str> {
        let Some((hole, ring, ob_line)) = self.active.take() else {
            return Err("ODB_SURFACE_OE_WITHOUT_OB");
        };
        validate_ring(&ring)?;
        if hole {
            self.islands[0].holes.push(ring);
        } else {
            self.islands.push(SurfaceIsland {
                outer: ring,
                holes: Vec::new(),
                ob_line,
            });
        }
        Ok(())
    }
    pub(crate) fn finish(self) -> Result<Vec<SurfaceIsland>, &'static str> {
        if self.active.is_some() {
            return Err("ODB_SURFACE_MISSING_OE");
        }
        if self.islands.is_empty() {
            return Err("ODB_SURFACE_NO_ISLAND");
        }
        let total: usize = self.islands.iter().map(|island| {
            island.outer.len() + island.holes.iter().map(Vec::len).sum::<usize>()
        }).sum();
        if total > MAX_VERTICES {
            return Err("ODB_SURFACE_TOTAL_VERTEX_LIMIT");
        }
        if self.islands.len() > 1 {
            // E_016 narrow subset: disconnected, strictly clockwise positive
            // islands, without holes. Existing single-island winding remains compatible.
            for (i, island) in self.islands.iter().enumerate() {
                if !island.holes.is_empty() {
                    return Err("ODB_SURFACE_MULTI_ISLAND_HOLE_UNSUPPORTED");
                }
                let signed_area2: i128 = island.outer.windows(2).map(|edge| {
                    i128::from(edge[0][0]) * i128::from(edge[1][1])
                        - i128::from(edge[1][0]) * i128::from(edge[0][1])
                }).sum();
                if signed_area2 >= 0 {
                    return Err("ODB_SURFACE_ISLAND_WINDING");
                }
                for previous in &self.islands[..i] {
                    if rings_touch(&previous.outer, &island.outer)
                        || strictly_inside(island.outer[0], &previous.outer)
                        || strictly_inside(previous.outer[0], &island.outer)
                    {
                        return Err("ODB_SURFACE_ISLAND_CONFLICT");
                    }
                }
            }
        } else {
            let island = &self.islands[0];
            let outer_area2 = validate_ring(&island.outer)?;
            let mut hole_area2 = 0i128;
            for (i, hole) in island.holes.iter().enumerate() {
                hole_area2 += validate_ring(hole)?;
                if rings_touch(&island.outer, hole)
                    || !strictly_inside(hole[0], &island.outer)
                {
                    return Err("ODB_SURFACE_HOLE_OUTSIDE_OR_TOUCHING");
                }
                for previous in &island.holes[..i] {
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
        }
        Ok(self.islands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn rectangle(s: &mut SurfaceBuilder, x: i64, y: i64, w: i64, clockwise: bool) {
        s.begin([x,y], false, 1).unwrap();
        let points = if clockwise {
            [[x,y+w],[x+w,y+w],[x+w,y],[x,y]]
        } else {
            [[x+w,y],[x+w,y+w],[x,y+w],[x,y]]
        };
        for p in points { s.line(p).unwrap(); }
        s.close().unwrap();
    }
    #[test]
    fn original_single_island_and_hole_are_compatible() {
        let mut s=SurfaceBuilder::default();
        rectangle(&mut s,0,0,4,false);
        s.begin([1,1],true,10).unwrap();
        for p in [[1,2],[2,2],[2,1],[1,1]] {s.line(p).unwrap();}
        s.close().unwrap();
        let islands=s.finish().unwrap();
        assert_eq!(islands.len(),1);
        assert_eq!(islands[0].outer.len(),5);
        assert_eq!(islands[0].holes.len(),1);
    }
    #[test]
    fn two_and_three_separate_clockwise_islands_are_exact() {
        for count in [2,3] {
            let mut s=SurfaceBuilder::default();
            for i in 0..count {rectangle(&mut s,i*10,0,4,true);}
            let islands=s.finish().unwrap();
            assert_eq!(islands.len(),count as usize);
            assert_eq!(islands[0].outer[0],[0,0]);
            assert_eq!(islands[1].outer[0],[10,0]);
        }
    }
    #[test]
    fn intersect_touch_contain_wrong_winding_and_hole_fail_closed() {
        for (x,y,w,expected) in [
            (3,0,4,"ODB_SURFACE_ISLAND_CONFLICT"), // overlap
            (4,0,4,"ODB_SURFACE_ISLAND_CONFLICT"), // touch
            (1,1,1,"ODB_SURFACE_ISLAND_CONFLICT"), // containment
        ] {
            let mut s=SurfaceBuilder::default();
            rectangle(&mut s,0,0,4,true);
            rectangle(&mut s,x,y,w,true);
            assert_eq!(s.finish().unwrap_err(),expected);
        }
        let mut winding=SurfaceBuilder::default();
        rectangle(&mut winding,0,0,4,true);
        rectangle(&mut winding,10,0,4,false);
        assert_eq!(winding.finish().unwrap_err(),"ODB_SURFACE_ISLAND_WINDING");
        let mut mix=SurfaceBuilder::default();
        rectangle(&mut mix,0,0,4,true);
        rectangle(&mut mix,10,0,4,true);
        assert_eq!(mix.begin([1,1],true,13),Err("ODB_SURFACE_MULTI_ISLAND_HOLE_UNSUPPORTED"));
        let mut mix_reverse=SurfaceBuilder::default();
        rectangle(&mut mix_reverse,0,0,4,true);
        mix_reverse.begin([1,1],true,8).unwrap();
        for p in [[1,2],[2,2],[2,1],[1,1]] {mix_reverse.line(p).unwrap();}
        mix_reverse.close().unwrap();
        assert_eq!(mix_reverse.begin([10,0],false,14),Err("ODB_SURFACE_MULTI_ISLAND_HOLE_UNSUPPORTED"));
    }
    #[test]
    fn bounded_islands_vertices_and_malformed_lines() {
        let mut s=SurfaceBuilder::default();
        for i in 0..16 {rectangle(&mut s, i*10, 0, 4, true);}
        assert_eq!(s.begin([200,0],false,99),Err("ODB_SURFACE_TOO_MANY_ISLANDS"));
        assert_eq!(s.finish().unwrap().len(),16);
        let mut missing=SurfaceBuilder::default();
        missing.begin([0,0],false,1).unwrap();
        assert_eq!(missing.finish().unwrap_err(),"ODB_SURFACE_MISSING_OE");
        let mut bow=SurfaceBuilder::default();
        bow.begin([0,0],false,1).unwrap();
        for p in [[3,3],[0,3],[3,0],[0,0]] {bow.line(p).unwrap();}
        assert_eq!(bow.close(),Err("ODB_SURFACE_SELF_INTERSECTION"));
    }
}
