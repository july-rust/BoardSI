//! C_015: exact PADS source-grid circular/rectangular void geometry.
//! This module makes no physical clearance, fabricated-copper, SI or P009 claim.
//! All coordinates are validated source ticks, NOT Qt display tessellations.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceHole {
    Rectangle([i64; 4]),
    Circle { centre: [i64; 2], radius: i64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadRisk {
    GapObserved,
    TangencyRisk,
    CopperOverlapRisk,
    CutoutMissingRisk,
}

impl PadRisk {
    pub fn wire(self) -> &'static str {
        match self {
            Self::GapObserved => "SOURCE_CUTOUT_GAP_OBSERVED",
            Self::TangencyRisk => "SOURCE_CUTOUT_TANGENCY_RISK",
            Self::CopperOverlapRisk => "SOURCE_REFERENCE_COPPER_OVERLAP_RISK",
            Self::CutoutMissingRisk => "SOURCE_CUTOUT_MISSING_RISK",
        }
    }
    pub fn severity(self) -> u8 {
        match self {
            Self::GapObserved => 0,
            Self::CutoutMissingRisk => 1,
            Self::TangencyRisk => 2,
            Self::CopperOverlapRisk => 3,
        }
    }
}

fn sq(n: i128) -> i128 {
    // Source ticks are bounded to +/- 5_600_000 by the PADS importer.
    n * n
}
fn dist2(a: [i64; 2], b: [i64; 2]) -> i128 {
    sq(i128::from(a[0]) - i128::from(b[0]))
        + sq(i128::from(a[1]) - i128::from(b[1]))
}

fn valid_rect(r: [i64; 4]) -> bool {
    r[0] < r[2] && r[1] < r[3]
}
fn valid_circle(radius: i64) -> bool {
    radius > 0
}
fn inside_rect(p: [i64; 2], r: [i64; 4]) -> bool {
    p[0] >= r[0] && p[0] <= r[2] && p[1] >= r[1] && p[1] <= r[3]
}
fn disjoint_rects(a: [i64; 4], b: [i64; 4]) -> bool {
    a[2] < b[0] || b[2] < a[0] || a[3] < b[1] || b[3] < a[1]
}
fn rect_circle_disjoint(r: [i64; 4], centre: [i64; 2], radius: i64) -> bool {
    let x = centre[0].clamp(r[0], r[2]);
    let y = centre[1].clamp(r[1], r[3]);
    dist2([x, y], centre) > sq(i128::from(radius))
}

pub fn strictly_disjoint(a: SourceHole, b: SourceHole) -> bool {
    match (a, b) {
        (SourceHole::Rectangle(a), SourceHole::Rectangle(b)) =>
            valid_rect(a) && valid_rect(b) && disjoint_rects(a, b),
        (SourceHole::Circle { centre: a, radius: ra },
         SourceHole::Circle { centre: b, radius: rb }) =>
            valid_circle(ra) && valid_circle(rb)
                && dist2(a, b) > sq(i128::from(ra) + i128::from(rb)),
        (SourceHole::Rectangle(r), SourceHole::Circle { centre, radius })
        | (SourceHole::Circle { centre, radius }, SourceHole::Rectangle(r)) =>
            valid_rect(r) && valid_circle(radius)
                && rect_circle_disjoint(r, centre, radius),
    }
}

pub fn inside_outer(h: SourceHole, outer: [i64; 4]) -> bool {
    if !valid_rect(outer) {
        return false;
    }
    match h {
        SourceHole::Rectangle(r) =>
            valid_rect(r) && r[0] > outer[0] && r[1] > outer[1]
                && r[2] < outer[2] && r[3] < outer[3],
        SourceHole::Circle { centre: [x, y], radius } =>
            valid_circle(radius)
                && i128::from(x) - i128::from(radius) > i128::from(outer[0])
                && i128::from(y) - i128::from(radius) > i128::from(outer[1])
                && i128::from(x) + i128::from(radius) < i128::from(outer[2])
                && i128::from(y) + i128::from(radius) < i128::from(outer[3]),
    }
}

pub fn validate_holes(holes: &[SourceHole], outer: [i64; 4]) -> bool {
    holes.len() <= 8
        && holes.iter().all(|&h| inside_outer(h, outer))
        && holes.iter().enumerate().all(|(i, &h)| {
            holes.iter().skip(i + 1).all(|&other| strictly_disjoint(h, other))
        })
}

/// The conductive Via pad must itself be STRICTLY contained by the outer
/// source-owned rectangle, including the circular perimeter.
pub fn pad_inside_outer(p: [i64; 2], diameter: i64, outer: [i64; 4]) -> bool {
    valid_rect(outer) && diameter > 0
        && [
            i128::from(p[0]) - i128::from(outer[0]),
            i128::from(outer[2]) - i128::from(p[0]),
            i128::from(p[1]) - i128::from(outer[1]),
            i128::from(outer[3]) - i128::from(p[1]),
        ].iter().all(|&margin| 2 * margin > i128::from(diameter))
}

/// Source ticks, no sqrt/no IEEE-754 in the boundary decision.
/// Returned index is only a cutout whose interior contains the pad centre.
/// A missing hole is never evidence of a safe reference separation.
pub fn classify(p: [i64; 2], diameter: i64, holes: &[SourceHole])
    -> Result<(PadRisk, Option<usize>), &'static str>
{
    if diameter <= 0 || holes.len() > 8 {
        return Err("V11_INVALID_PAD_OR_HOLE_COUNT");
    }
    if holes.is_empty() {
        return Ok((PadRisk::CutoutMissingRisk, None));
    }
    for (i, &hole) in holes.iter().enumerate() {
        match hole {
            SourceHole::Rectangle(r) => {
                if !valid_rect(r) { return Err("V11_RECT_INVALID"); }
                if inside_rect(p, r) {
                    let m = [
                        i128::from(p[0]) - i128::from(r[0]),
                        i128::from(r[2]) - i128::from(p[0]),
                        i128::from(p[1]) - i128::from(r[1]),
                        i128::from(r[3]) - i128::from(p[1]),
                    ].into_iter().min().ok_or("V11_RECT_EMPTY")?;
                    let risk = match (2 * m).cmp(&i128::from(diameter)) {
                        std::cmp::Ordering::Greater => PadRisk::GapObserved,
                        std::cmp::Ordering::Equal => PadRisk::TangencyRisk,
                        std::cmp::Ordering::Less => PadRisk::CopperOverlapRisk,
                    };
                    return Ok((risk, Some(i)));
                }
            }
            SourceHole::Circle { centre, radius } => {
                if !valid_circle(radius) { return Err("V11_CIRCLE_INVALID"); }
                let d2 = dist2(p, centre);
                let radius2 = sq(i128::from(radius));
                if d2 <= radius2 {
                    let remaining_twice = 2 * i128::from(radius) - i128::from(diameter);
                    let risk = if remaining_twice < 0 {
                        PadRisk::CopperOverlapRisk
                    } else {
                        match (4 * d2).cmp(&sq(remaining_twice)) {
                            std::cmp::Ordering::Less => PadRisk::GapObserved,
                            std::cmp::Ordering::Equal => PadRisk::TangencyRisk,
                            std::cmp::Ordering::Greater => PadRisk::CopperOverlapRisk,
                        }
                    };
                    return Ok((risk, Some(i)));
                }
            }
        }
    }
    Ok((PadRisk::CopperOverlapRisk, None))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_circle_exact_gap_tangent_overlap() {
        let h = [SourceHole::Circle { centre: [50, 50], radius: 20 }];
        assert_eq!(classify([50, 50], 39, &h).unwrap().0, PadRisk::GapObserved);
        assert_eq!(classify([50, 50], 40, &h).unwrap().0, PadRisk::TangencyRisk);
        assert_eq!(classify([50, 50], 41, &h).unwrap().0, PadRisk::CopperOverlapRisk);
        assert_eq!(classify([51, 50], 38, &h).unwrap().0, PadRisk::TangencyRisk);
        assert_eq!(classify([51, 51], 38, &h).unwrap().0, PadRisk::CopperOverlapRisk);
        assert_eq!(classify([80, 50], 1, &h).unwrap(), (PadRisk::CopperOverlapRisk, None));
    }
    #[test]
    fn source_rectangle_exact_and_missing() {
        let h = [SourceHole::Rectangle([30, 30, 70, 70])];
        assert_eq!(classify([50, 50], 39, &h).unwrap().0, PadRisk::GapObserved);
        assert_eq!(classify([50, 50], 40, &h).unwrap().0, PadRisk::TangencyRisk);
        assert_eq!(classify([50, 50], 41, &h).unwrap().0, PadRisk::CopperOverlapRisk);
        assert_eq!(classify([50, 50], 20, &[]).unwrap().0, PadRisk::CutoutMissingRisk);
    }
    #[test]
    fn strict_mixed_topology_disallows_tangency_and_nesting() {
        let outer = [0, 0, 100, 100];
        let r = SourceHole::Rectangle([5, 5, 25, 25]);
        let c = SourceHole::Circle { centre: [50, 50], radius: 10 };
        assert!(validate_holes(&[r, c], outer));
        assert!(!strictly_disjoint(r, SourceHole::Circle { centre: [30, 15], radius: 5 }));
        assert!(!strictly_disjoint(c, SourceHole::Circle { centre: [70, 50], radius: 10 }));
        assert!(!inside_outer(SourceHole::Circle { centre: [20, 50], radius: 20 }, outer));
        assert!(!validate_holes(&[r; 9], outer));
        assert!(!strictly_disjoint(r, SourceHole::Rectangle([10, 10, 15, 15])));
        assert!(pad_inside_outer([50, 50], 20, outer));
        assert!(!pad_inside_outer([50, 50], 100, outer));
    }
    #[test]
    fn bounded_i128_distance_no_float_rounding() {
        let h = [SourceHole::Circle { centre: [5_599_900, 0], radius: 50 }];
        assert_eq!(classify([5_599_900, 0], 100, &h).unwrap().0, PadRisk::TangencyRisk);
        assert_eq!(classify([5_599_901, 0], 98, &h).unwrap().0, PadRisk::TangencyRisk);
    }
}
