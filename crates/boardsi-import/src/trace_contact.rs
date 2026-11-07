//! Source-grid finite-width trace proximity for any straight vector and circular route arc.
//! Geometry candidate only: this does not establish manufactured continuity, an
//! electrical Open/Short, reference return paths, or solver readiness.
use crate::route_preview;
use std::f64::consts::TAU;

const MAX_TICK: i64 = 5_600_000;
// > worst f64 operation noise on the bounded source grid. A candidate within
// this guard band of exact tangency is UNKNOWN, never silently connected.
const BOUNDARY_GUARD_TICKS: f64 = 1.0e-5;
const MAX_BROADPHASE_PAIRS: usize = 1_000_000;

#[derive(Clone, Debug)]
pub enum Centreline {
    Straight {
        start: [i64; 2],
        end: [i64; 2],
    },
    Circular {
        centre: [i64; 2],
        start: [i64; 2],
        end: [i64; 2],
        clockwise: bool,
    },
}
#[derive(Clone, Debug)]
pub struct Stroke {
    pub layer: u32,
    pub net: String,
    pub width_ticks: i64,
    pub source_line: usize,
    pub centreline: Centreline,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Contact {
    PositiveArea,
    Separated,
    BoundaryUncertain,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetRelation {
    SameDeclaredNet,
    DifferentDeclaredNets,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub first: usize,
    pub second: usize,
    pub first_source_line: usize,
    pub second_source_line: usize,
    pub net_relation: NetRelation,
}
#[derive(Clone, Copy, Debug)]
struct P {
    x: f64,
    y: f64,
}
impl P {
    fn from_ticks(t: [i64; 2]) -> Self {
        Self {
            x: t[0] as f64,
            y: t[1] as f64,
        }
    }
    fn plus(self, o: Self) -> Self {
        Self {
            x: self.x + o.x,
            y: self.y + o.y,
        }
    }
    fn minus(self, o: Self) -> Self {
        Self {
            x: self.x - o.x,
            y: self.y - o.y,
        }
    }
    fn mul(self, k: f64) -> Self {
        Self {
            x: self.x * k,
            y: self.y * k,
        }
    }
    fn dot(self, o: Self) -> f64 {
        self.x * o.x + self.y * o.y
    }
    fn norm2(self) -> f64 {
        self.dot(self)
    }
    fn norm(self) -> f64 {
        self.norm2().sqrt()
    }
    fn distance(self, o: Self) -> f64 {
        self.minus(o).norm()
    }
}
#[derive(Clone, Copy)]
struct Line {
    a: P,
    b: P,
}
#[derive(Clone, Copy)]
struct CircleArc {
    c: P,
    a: P,
    b: P,
    r: f64,
    clockwise: bool,
}
impl CircleArc {
    fn contains(self, p: P) -> bool {
        let start = (self.a.y - self.c.y).atan2(self.a.x - self.c.x);
        let end = (self.b.y - self.c.y).atan2(self.b.x - self.c.x);
        let theta = (p.y - self.c.y).atan2(p.x - self.c.x);
        let sweep = if self.clockwise {
            (start - end).rem_euclid(TAU)
        } else {
            (end - start).rem_euclid(TAU)
        };
        let delta = if self.clockwise {
            (start - theta).rem_euclid(TAU)
        } else {
            (theta - start).rem_euclid(TAU)
        };
        delta <= sweep + 1.0e-12 || (TAU - delta) <= 1.0e-12
    }
    fn closest(self, p: P) -> f64 {
        let radial = p.minus(self.c);
        let mut best = p.distance(self.a).min(p.distance(self.b));
        if radial.norm2() > 0.0 {
            let nearest = self.c.plus(radial.mul(self.r / radial.norm()));
            if self.contains(nearest) {
                best = best.min((radial.norm() - self.r).abs());
            }
        } else {
            best = best.min(self.r);
        }
        best
    }
}
fn valid_point(p: [i64; 2]) -> bool {
    p.iter().all(|v| (-MAX_TICK..=MAX_TICK).contains(v))
}
fn valid_stroke(s: &Stroke) -> Result<(), &'static str> {
    if s.layer == 0
        || s.layer > 64
        || s.net.is_empty()
        || s.width_ticks <= 0
        || s.width_ticks > MAX_TICK
    {
        return Err("INVALID_STROKE_METADATA");
    }
    match &s.centreline {
        Centreline::Straight { start, end } => {
            if !valid_point(*start) || !valid_point(*end) || start == end {
                return Err("INVALID_STRAIGHT_CENTRELINE");
            }
        }
        Centreline::Circular {
            centre, start, end, ..
        } => {
            if !valid_point(*centre) || !valid_point(*start) || !valid_point(*end) || start == end {
                return Err("INVALID_ARC_CENTRELINE");
            }
            let sq = |p: &[i64; 2]| -> i128 {
                let x = i128::from(p[0]) - i128::from(centre[0]);
                let y = i128::from(p[1]) - i128::from(centre[1]);
                x * x + y * y
            };
            if sq(start) == 0 || sq(start) != sq(end) {
                return Err("INVALID_ARC_RADIUS");
            }
        }
    }
    Ok(())
}
fn arc(centre: [i64; 2], start: [i64; 2], end: [i64; 2], clockwise: bool) -> CircleArc {
    let c = P::from_ticks(centre);
    let a = P::from_ticks(start);
    CircleArc {
        c,
        a,
        b: P::from_ticks(end),
        r: a.distance(c),
        clockwise,
    }
}
fn point_line_distance(p: P, line: Line) -> f64 {
    let v = line.b.minus(line.a);
    let t = (p.minus(line.a).dot(v) / v.norm2()).clamp(0.0, 1.0);
    p.distance(line.a.plus(v.mul(t)))
}
fn line_intersection(a: Line, b: Line) -> bool {
    let o = |p: P, q: P, r: P| -> i128 {
        // All points originate from SOURCE integers; int conversion is exact.
        let (px, py, qx, qy, rx, ry) = (
            p.x as i128,
            p.y as i128,
            q.x as i128,
            q.y as i128,
            r.x as i128,
            r.y as i128,
        );
        (qx - px) * (ry - py) - (qy - py) * (rx - px)
    };
    let on = |p: P, q: P, r: P| -> bool {
        r.x >= p.x.min(q.x) && r.x <= p.x.max(q.x) && r.y >= p.y.min(q.y) && r.y <= p.y.max(q.y)
    };
    let p = o(a.a, a.b, b.a);
    let q = o(a.a, a.b, b.b);
    let r = o(b.a, b.b, a.a);
    let s = o(b.a, b.b, a.b);
    (p == 0 && on(a.a, a.b, b.a))
        || (q == 0 && on(a.a, a.b, b.b))
        || (r == 0 && on(b.a, b.b, a.a))
        || (s == 0 && on(b.a, b.b, a.b))
        || ((p > 0 && q < 0 || p < 0 && q > 0) && (r > 0 && s < 0 || r < 0 && s > 0))
}
fn line_line(a: Line, b: Line) -> f64 {
    if line_intersection(a, b) {
        return 0.0;
    }
    point_line_distance(a.a, b)
        .min(point_line_distance(a.b, b))
        .min(point_line_distance(b.a, a))
        .min(point_line_distance(b.b, a))
}
fn line_arc(line: Line, a: CircleArc) -> f64 {
    let v = line.b.minus(line.a);
    let len2 = v.norm2();
    let mut best = a
        .closest(line.a)
        .min(a.closest(line.b))
        .min(point_line_distance(a.a, line))
        .min(point_line_distance(a.b, line));
    // All roots of segment vs circle, respecting actual directed arc sweep.
    let rel = line.a.minus(a.c);
    let b = 2.0 * rel.dot(v);
    let c = rel.norm2() - a.r * a.r;
    let discriminant = b * b - 4.0 * len2 * c;
    if discriminant >= 0.0 {
        let sqrt = discriminant.sqrt();
        for t in [(-b - sqrt) / (2.0 * len2), (-b + sqrt) / (2.0 * len2)] {
            if (0.0..=1.0).contains(&t) && a.contains(line.a.plus(v.mul(t))) {
                return 0.0;
            }
        }
    }
    // Stationary distance to infinite line occurs at circle's +/- normal.
    let normal = P { x: -v.y, y: v.x }.mul(1.0 / len2.sqrt());
    for sign in [-1.0, 1.0] {
        let p = a.c.plus(normal.mul(sign * a.r));
        let t = p.minus(line.a).dot(v) / len2;
        if (0.0..=1.0).contains(&t) && a.contains(p) {
            best = best.min(point_line_distance(p, line));
        }
    }
    best
}
fn arc_arc(a: CircleArc, b: CircleArc) -> f64 {
    let mut best = b
        .closest(a.a)
        .min(b.closest(a.b))
        .min(a.closest(b.a))
        .min(a.closest(b.b));
    let delta = b.c.minus(a.c);
    let d = delta.norm();
    if d == 0.0 {
        return best;
    }
    let u = delta.mul(1.0 / d);
    // Interior stationary candidates are collinear with both circle centres.
    for x in [-1.0, 1.0] {
        for y in [-1.0, 1.0] {
            let p = a.c.plus(u.mul(a.r * x));
            let q = b.c.plus(u.mul(b.r * y));
            if a.contains(p) && b.contains(q) {
                best = best.min(p.distance(q));
            }
        }
    }
    // Circle-circle crossings; their presence alone does not mean arc overlap.
    if d <= a.r + b.r && d >= (a.r - b.r).abs() {
        let along = (a.r * a.r - b.r * b.r + d * d) / (2.0 * d);
        let h2 = a.r * a.r - along * along;
        if h2 >= 0.0 {
            let foot = a.c.plus(u.mul(along));
            let perpendicular = P { x: -u.y, y: u.x }.mul(h2.sqrt());
            for p in [foot.plus(perpendicular), foot.minus(perpendicular)] {
                if a.contains(p) && b.contains(p) {
                    return 0.0;
                }
            }
        }
    }
    best
}
fn envelope(s: &Stroke) -> [f64; 4] {
    let r = s.width_ticks as f64 / 2.0;
    match s.centreline {
        Centreline::Straight { start, end } => [
            start[0].min(end[0]) as f64 - r,
            start[1].min(end[1]) as f64 - r,
            start[0].max(end[0]) as f64 + r,
            start[1].max(end[1]) as f64 + r,
        ],
        Centreline::Circular { centre, start, .. } => {
            let radial = P::from_ticks(start).distance(P::from_ticks(centre)) + r;
            [
                centre[0] as f64 - radial,
                centre[1] as f64 - radial,
                centre[0] as f64 + radial,
                centre[1] as f64 + radial,
            ]
        }
    }
}
fn distance(a: &Stroke, b: &Stroke) -> f64 {
    match (&a.centreline, &b.centreline) {
        (
            Centreline::Straight { start: aa, end: ab },
            Centreline::Straight { start: ba, end: bb },
        ) => line_line(
            Line {
                a: P::from_ticks(*aa),
                b: P::from_ticks(*ab),
            },
            Line {
                a: P::from_ticks(*ba),
                b: P::from_ticks(*bb),
            },
        ),
        (
            Centreline::Straight { start, end },
            Centreline::Circular {
                centre,
                start: as_,
                end: ae,
                clockwise,
            },
        ) => line_arc(
            Line {
                a: P::from_ticks(*start),
                b: P::from_ticks(*end),
            },
            arc(*centre, *as_, *ae, *clockwise),
        ),
        (
            Centreline::Circular {
                centre,
                start: as_,
                end: ae,
                clockwise,
            },
            Centreline::Straight { start, end },
        ) => line_arc(
            Line {
                a: P::from_ticks(*start),
                b: P::from_ticks(*end),
            },
            arc(*centre, *as_, *ae, *clockwise),
        ),
        (
            Centreline::Circular {
                centre: ac,
                start: as_,
                end: ae,
                clockwise: aw,
            },
            Centreline::Circular {
                centre: bc,
                start: bs,
                end: be,
                clockwise: bw,
            },
        ) => arc_arc(arc(*ac, *as_, *ae, *aw), arc(*bc, *bs, *be, *bw)),
    }
}
/// A finite-width candidate requires *strictly positive* copper-area overlap.
/// Cross-layer pairs are separated; touching exactly at the boundary is UNKNOWN.
/// No electrical connectivity edge is manufactured by this predicate.
pub fn contact(a: &Stroke, b: &Stroke) -> Result<Contact, &'static str> {
    valid_stroke(a)?;
    valid_stroke(b)?;
    if a.layer != b.layer {
        return Ok(Contact::Separated);
    }
    let ea = envelope(a);
    let eb = envelope(b);
    if ea[0] > eb[2] || eb[0] > ea[2] || ea[1] > eb[3] || eb[1] > ea[3] {
        return Ok(Contact::Separated);
    }
    let d = distance(a, b);
    if !d.is_finite() {
        return Err("NONFINITE_GEOMETRY");
    }
    let limit = (a.width_ticks as f64 + b.width_ticks as f64) / 2.0;
    Ok(if d < limit - BOUNDARY_GUARD_TICKS {
        Contact::PositiveArea
    } else if d > limit + BOUNDARY_GUARD_TICKS {
        Contact::Separated
    } else {
        Contact::BoundaryUncertain
    })
}
/// Reuse actual imported straight and circular PADS SOURCE objects, rather than
/// an angle-specific fixture; report only source-local contact hypotheses.
pub fn from_artwork(art: &route_preview::Artwork) -> Result<Vec<Stroke>, &'static str> {
    if art.skipped != 0 {
        return Err("UNSUPPORTED_ROUTE_GEOMETRY");
    }
    let mut strokes = Vec::with_capacity(art.segments.len() + art.arcs.len());
    for s in &art.segments {
        strokes.push(Stroke {
            layer: s.layer,
            net: s.net.clone(),
            width_ticks: s.width_ticks,
            source_line: s.source_line,
            centreline: Centreline::Straight {
                start: s.a,
                end: s.b,
            },
        });
    }
    for a in &art.arcs {
        strokes.push(Stroke {
            layer: a.layer,
            net: a.net.clone(),
            width_ticks: a.width_ticks,
            source_line: a.source_line,
            centreline: Centreline::Circular {
                centre: a.centre,
                start: a.start,
                end: a.end,
                clockwise: a.clockwise,
            },
        });
    }
    strokes.sort_by_key(|s| s.source_line);
    for s in &strokes {
        valid_stroke(s)?;
    }
    Ok(strokes)
}
/// Bounded sweep over finite-width conservative envelopes. Reject ambiguous
/// tangencies (fail closed) and pathologically dense pair sets, not false PASS.
pub fn contact_candidates(art: &route_preview::Artwork) -> Result<Vec<Candidate>, &'static str> {
    let strokes = from_artwork(art)?;
    let boxes: Vec<_> = strokes.iter().map(envelope).collect();
    let mut order: Vec<_> = (0..strokes.len()).collect();
    order.sort_by(|&a, &b| boxes[a][0].total_cmp(&boxes[b][0]));
    let mut candidates = Vec::new();
    let mut tested = 0usize;
    for (i, &lhs) in order.iter().enumerate() {
        for &rhs in order.iter().skip(i + 1) {
            if boxes[rhs][0] > boxes[lhs][2] {
                break;
            }
            if strokes[lhs].layer != strokes[rhs].layer
                || boxes[lhs][1] > boxes[rhs][3]
                || boxes[rhs][1] > boxes[lhs][3]
            {
                continue;
            }
            tested += 1;
            if tested > MAX_BROADPHASE_PAIRS {
                return Err("CONTACT_PAIR_BUDGET");
            }
            match contact(&strokes[lhs], &strokes[rhs])? {
                Contact::PositiveArea => {
                    let (a, b) = (lhs.min(rhs), lhs.max(rhs));
                    candidates.push(Candidate {
                        first: a,
                        second: b,
                        first_source_line: strokes[a].source_line,
                        second_source_line: strokes[b].source_line,
                        net_relation: if strokes[a].net == strokes[b].net {
                            NetRelation::SameDeclaredNet
                        } else {
                            NetRelation::DifferentDeclaredNets
                        },
                    });
                }
                Contact::BoundaryUncertain => return Err("UNCERTAIN_COPPER_TANGENCY"),
                Contact::Separated => {}
            }
        }
    }
    candidates.sort_by_key(|c| (c.first, c.second));
    Ok(candidates)
}
/// Fail-closed source parsing: any skipped PADS route primitive prevents
/// claiming completeness. Neither this nor the returned candidates is an SI result.
pub fn source_candidates(bytes: &[u8]) -> Result<Vec<Candidate>, &'static str> {
    let art = route_preview::parse(bytes)?;
    contact_candidates(&art)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn line(a: [i64; 2], b: [i64; 2], width: i64) -> Stroke {
        Stroke {
            layer: 1,
            net: "N".into(),
            width_ticks: width,
            source_line: 1,
            centreline: Centreline::Straight { start: a, end: b },
        }
    }
    fn round(c: [i64; 2], a: [i64; 2], b: [i64; 2], clockwise: bool, width: i64) -> Stroke {
        Stroke {
            layer: 1,
            net: "N".into(),
            width_ticks: width,
            source_line: 2,
            centreline: Centreline::Circular {
                centre: c,
                start: a,
                end: b,
                clockwise,
            },
        }
    }
    #[test]
    fn arbitrary_slopes_and_endpoint_reversal_are_not_angle_cases() {
        let rail = line([0, 0], [246, 914], 10);
        for dx in -20..=20 {
            let crossing = line([23 + dx, 257], [223 - dx, 657], 7);
            assert_eq!(contact(&rail, &crossing).unwrap(), Contact::PositiveArea);
            assert_eq!(contact(&crossing, &rail).unwrap(), Contact::PositiveArea);
            assert_eq!(
                contact(&rail, &line([223 - dx, 657], [23 + dx, 257], 7)).unwrap(),
                Contact::PositiveArea
            );
        }
        let other = line([200, 0], [300, 30], 10);
        assert_eq!(contact(&rail, &other).unwrap(), Contact::Separated);
    }
    #[test]
    fn tangent_is_not_positive_contact_and_one_tick_changes_classification() {
        let a = line([0, 0], [100, 0], 10);
        assert_eq!(
            contact(&a, &line([0, 10], [100, 10], 10)).unwrap(),
            Contact::BoundaryUncertain
        );
        assert_eq!(
            contact(&a, &line([0, 9], [100, 9], 10)).unwrap(),
            Contact::PositiveArea
        );
        assert_eq!(
            contact(&a, &line([0, 11], [100, 11], 10)).unwrap(),
            Contact::Separated
        );
    }
    #[test]
    fn straight_arc_cw_ccw_and_major_sweeps() {
        let arc = round([0, 0], [100, 0], [0, 100], false, 10);
        let close = line([68, 68], [75, 75], 10);
        assert_eq!(contact(&arc, &close).unwrap(), Contact::PositiveArea);
        assert_eq!(contact(&close, &arc).unwrap(), Contact::PositiveArea);
        assert_eq!(
            contact(&arc, &line([-75, -75], [-68, -68], 10)).unwrap(),
            Contact::Separated
        );
        let major = round([0, 0], [100, 0], [0, 100], true, 10);
        assert_eq!(
            contact(&major, &line([-75, -75], [-68, -68], 10)).unwrap(),
            Contact::PositiveArea
        );
    }
    #[test]
    fn arc_arc_contacts_away_from_endpoints_and_non_contacts() {
        let a = round([0, 0], [100, 0], [0, 100], false, 10);
        let b = round([0, 0], [108, 0], [0, 108], false, 10);
        assert_eq!(contact(&a, &b).unwrap(), Contact::PositiveArea);
        let c = round([0, 0], [112, 0], [0, 112], false, 10);
        assert_eq!(contact(&a, &c).unwrap(), Contact::Separated);
        let d = round([200, 0], [100, 0], [200, 100], true, 10);
        assert_eq!(contact(&a, &d).unwrap(), Contact::PositiveArea);
    }
    #[test]
    fn invalid_radius_and_zero_length_rejected_not_drawn_as_chord() {
        assert_eq!(
            contact(
                &round([0, 0], [100, 0], [0, 101], false, 10),
                &line([0, 0], [10, 10], 10)
            ),
            Err("INVALID_ARC_RADIUS")
        );
        assert_eq!(
            contact(&line([0, 0], [0, 0], 10), &line([0, 0], [1, 0], 10)),
            Err("INVALID_STRAIGHT_CENTRELINE")
        );
        let mut other = line([0, 0], [10, 10], 10);
        other.layer = 2;
        assert_eq!(
            contact(&other, &line([0, 0], [10, 10], 10)).unwrap(),
            Contact::Separated
        );
    }
    #[test]
    fn real_pads_parser_handles_arbitrary_line_and_arc_in_one_source() {
        let source=b"!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 2\n*ROUTE*\n*SIGNAL* N\nU1.1 U2.1\n0 0 1 10 0\n40 40 1 10 0\nU1.2 U2.2\n20 0 1 10 0\n20 40 1 10 0\nU1.3 U2.3\n100 0 1 8 0\n0 0 1 8 4096 CCW\n0 100 1 8 0\nU1.4 U2.4\n68 68 1 10 0\n75 75 1 10 0\n*END*\n";
        let art = route_preview::parse(source).unwrap();
        assert_eq!((art.segments.len(), art.arcs.len(), art.skipped), (3, 1, 0));
        let candidates = source_candidates(source).unwrap();
        assert!(candidates
            .iter()
            .any(|c| c.first_source_line == 8 && c.second_source_line == 11));
        assert!(candidates
            .iter()
            .any(|c| c.first_source_line == 15 && c.second_source_line == 18));
        assert_eq!(source_candidates(source).unwrap().len(), candidates.len());
    }
}
