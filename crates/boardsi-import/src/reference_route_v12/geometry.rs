//! C_016: integer source pad boundary analysis, nonphysical.
use crate::reference_route_v11::geometry::{validate_holes, SourceHole};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Risk {
    Full,
    CutoutTangent,
    CutoutIntersect,
    OuterTangent,
    OuterCross,
}
impl Risk {
    pub fn wire(self) -> &'static str {
        match self {
            Self::Full => "STITCH_PAD_FULL_COPPER_FOOTPRINT_CANDIDATE",
            Self::CutoutTangent => "STITCH_PAD_CUTOUT_TANGENCY_RISK",
            Self::CutoutIntersect => "STITCH_PAD_CUTOUT_INTERSECTION_RISK",
            Self::OuterTangent => "STITCH_PAD_OUTER_TANGENCY_RISK",
            Self::OuterCross => "STITCH_PAD_OUTER_CROSSING_RISK",
        }
    }
    pub fn rank(self) -> u8 {
        match self {
            Self::Full => 0,
            Self::CutoutTangent | Self::OuterTangent => 1,
            _ => 2,
        }
    }
}
fn sq(x: i128) -> Result<i128, &'static str> {
    x.checked_mul(x).ok_or("V12_OVERFLOW")
}
fn d2(a: [i64; 2], b: [i64; 2]) -> Result<i128, &'static str> {
    sq(i128::from(a[0]) - i128::from(b[0]))?
        .checked_add(sq(i128::from(a[1]) - i128::from(b[1]))?)
        .ok_or("V12_OVERFLOW")
}
/// Full conductive round Pad versus *source* outer and holes, no physical SI.
pub fn classify(
    p: [i64; 2],
    diameter: i64,
    outer: [i64; 4],
    holes: &[SourceHole],
) -> Result<(Risk, Option<usize>), &'static str> {
    if diameter <= 0
        || diameter > 5_600_000
        || p.iter().any(|&v| !(-5_600_000..=5_600_000).contains(&v))
        || !validate_holes(holes, outer)
        || p[0] <= outer[0]
        || p[0] >= outer[2]
        || p[1] <= outer[1]
        || p[1] >= outer[3]
    {
        return Err("V12_UNSUPPORTED_SOURCE_GEOMETRY");
    }
    let distances = [
        i128::from(p[0]) - i128::from(outer[0]),
        i128::from(outer[2]) - i128::from(p[0]),
        i128::from(p[1]) - i128::from(outer[1]),
        i128::from(outer[3]) - i128::from(p[1]),
    ];
    let m = *distances.iter().min().ok_or("V12_OUTER")?;
    let d = i128::from(diameter);
    let mut risk = match (2 * m).cmp(&d) {
        std::cmp::Ordering::Less => Risk::OuterCross,
        std::cmp::Ordering::Equal => Risk::OuterTangent,
        std::cmp::Ordering::Greater => Risk::Full,
    };
    let mut chosen = None;
    for (i, h) in holes.iter().enumerate() {
        let (v, limit) = match *h {
            SourceHole::Rectangle(r) => {
                let q = [p[0].clamp(r[0], r[2]), p[1].clamp(r[1], r[3])];
                (4 * d2(p, q)?, sq(d)?)
            }
            SourceHole::Circle { centre, radius } => {
                let radius2 = sq(i128::from(radius))?;
                let dist = d2(p, centre)?;
                if dist <= radius2 {
                    return Err("V12_STITCH_CENTRE_IN_VOID");
                }
                (4 * dist, sq(2 * i128::from(radius) + d)?)
            }
        };
        if v == 0 {
            return Err("V12_STITCH_CENTRE_IN_VOID");
        }
        let state = match v.cmp(&limit) {
            std::cmp::Ordering::Less => Some(Risk::CutoutIntersect),
            std::cmp::Ordering::Equal => Some(Risk::CutoutTangent),
            std::cmp::Ordering::Greater => None,
        };
        if let Some(next) = state {
            if next.rank() > risk.rank() {
                risk = next;
                chosen = Some(i)
            }
        }
    }
    Ok((risk, chosen))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outer_edge_exact_gap_tangent_cross() {
        let o = [0, 0, 100, 100];
        assert_eq!(classify([50, 50], 20, o, &[]).unwrap().0, Risk::Full);
        assert_eq!(
            classify([10, 50], 20, o, &[]).unwrap().0,
            Risk::OuterTangent
        );
        assert_eq!(classify([9, 50], 20, o, &[]).unwrap().0, Risk::OuterCross);
    }
    #[test]
    fn rectangle_edge_exact_and_centre_rejected() {
        let o = [0, 0, 100, 100];
        let h = [SourceHole::Rectangle([40, 40, 60, 60])];
        assert_eq!(classify([20, 50], 20, o, &h).unwrap().0, Risk::Full);
        assert_eq!(
            classify([30, 50], 20, o, &h).unwrap().0,
            Risk::CutoutTangent
        );
        assert_eq!(
            classify([31, 50], 20, o, &h).unwrap().0,
            Risk::CutoutIntersect
        );
        assert!(classify([50, 50], 20, o, &h).is_err());
    }
    #[test]
    fn mixed_circle_and_rectangle_are_analytic() {
        let o = [0, 0, 200, 200];
        let h = [
            SourceHole::Circle {
                centre: [100, 100],
                radius: 20,
            },
            SourceHole::Rectangle([140, 80, 160, 120]),
        ];
        assert_eq!(classify([64, 100], 30, o, &h).unwrap().0, Risk::Full);
        assert_eq!(
            classify([65, 100], 30, o, &h).unwrap().0,
            Risk::CutoutTangent
        );
        assert_eq!(
            classify([66, 100], 30, o, &h).unwrap().0,
            Risk::CutoutIntersect
        );
        assert_eq!(
            classify([130, 100], 20, o, &h).unwrap().0,
            Risk::CutoutTangent
        );
    }
    #[test]
    fn exactly_eight_non_touching_source_cutouts_accepted() {
        let outer = [0, 0, 200, 200];
        let holes: Vec<_> = (0..8)
            .map(|i| SourceHole::Rectangle([10 + 20 * i, 10, 15 + 20 * i, 15]))
            .collect();
        assert_eq!(holes.len(), 8);
        assert_eq!(classify([100, 100], 20, outer, &holes).unwrap().0, Risk::Full);
    }
    #[test]
    fn rejects_unsupported_and_out_of_range() {
        let o = [0, 0, 100, 100];
        assert!(classify([50, 50], 0, o, &[]).is_err());
        assert!(classify([i64::MAX, 50], 20, o, &[]).is_err());
        assert!(classify(
            [50, 50],
            20,
            o,
            &[SourceHole::Rectangle([40, 40, 60, 60]); 9]
        )
        .is_err());
    }
}
