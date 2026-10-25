//! A_029 P2-T: bounded candidate for explicitly owned curved COPCLS.
//! Not activated in the production copper importer or Qt transport.
//! This preflight proves only the restricted source cardinal-arc topology and
//! conservative display proxy; it does not prove real copper or physical SI.
use crate::outline_arc::preflight_closed;
use crate::outline_preview::{bounded, fixed, valid_ring};
use crate::outline_topology::{self, Segment};

const MIN_ARC_RADIUS_TICKS: i64 = 1000; // 254 um; source is 254 nm per tick.
const ARC_STEPS: usize = 8; // <= 1% radial conservative preview bound.
const MAX_DISPLAY_VERTICES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurvedCopperCandidate {
    /// Source-exact line and cardinal-quarter-arc segments, not a polygon.
    pub source_segments: Vec<Segment>,
    /// Bounded, conservative visual proxy only, source-grid closed ring.
    pub display_ring: Vec<[i64; 2]>,
}

fn area2(ring: &[[i64; 2]]) -> i128 {
    ring.windows(2)
        .map(|p| {
            i128::from(p[0][0]) * i128::from(p[1][1]) - i128::from(p[1][0]) * i128::from(p[0][1])
        })
        .sum()
}

fn cross(a: [i64; 2], b: [i64; 2], c: [i64; 2]) -> i128 {
    let x = i128::from(b[0]) - i128::from(a[0]);
    let y = i128::from(b[1]) - i128::from(a[1]);
    x * (i128::from(c[1]) - i128::from(b[1])) - y * (i128::from(c[0]) - i128::from(b[0]))
}

fn convex_chords(ring: &[[i64; 2]]) -> bool {
    if ring.len() < 4 || ring.first() != ring.last() {
        return false;
    }
    let winding = area2(ring).signum();
    if winding == 0 {
        return false;
    }
    let n = ring.len() - 1;
    (0..n).all(|i| cross(ring[i], ring[(i + 1) % n], ring[(i + 2) % n]) * winding >= 0)
}

/// Strict source-owned COPCLS candidate (no holes).
///
/// PADS point rows are either x y or 8-field cardinal arc records. Reject
/// unsupported arc sweeps, concavity, overlap, invalid source closure and
/// excessive geometry. All source arcs must bulge outward from a convex ring.
/// This is a **preflight only**; wire/CPU/RHI parity is a separate gate.
pub fn validate(
    rows: &[Vec<&str>],
    origin: [i64; 2],
) -> Result<CurvedCopperCandidate, &'static str> {
    if rows.len() < 4 || rows.len() > 65 {
        return Err("COPPER_ARC_ROW_LIMIT");
    }
    if !rows.iter().any(|row| row.len() == 8) {
        return Err("COPPER_ARC_MISSING");
    }
    let arcs = preflight_closed(rows, origin)?;
    let mut vertices = Vec::with_capacity(rows.len());
    for row in rows {
        if row.len() != 2 && row.len() != 8 {
            return Err("COPPER_ARC_ROW");
        }
        let x = origin[0]
            .checked_add(fixed(row[0])?)
            .ok_or("COPPER_ARC_OVERFLOW")
            .and_then(bounded)?;
        let y = origin[1]
            .checked_add(fixed(row[1])?)
            .ok_or("COPPER_ARC_OVERFLOW")
            .and_then(bounded)?;
        vertices.push([x, y]);
    }
    if !convex_chords(&vertices) {
        return Err("COPPER_ARC_NONCONVEX");
    }
    let ccw = area2(&vertices) > 0;
    let mut segments = Vec::with_capacity(arcs.len());
    let mut display = vec![vertices[0]];
    for (i, arc) in arcs.iter().enumerate() {
        match arc {
            None => {
                segments.push(Segment::Line(vertices[i], vertices[i + 1]));
                display.push(vertices[i + 1]);
            }
            Some(a) => {
                // Outward convex arc: source winding and quarter sweep agree.
                if a.clockwise == ccw {
                    return Err("COPPER_ARC_NOT_OUTWARD");
                }
                let r = (a.start[0] - a.centre[0])
                    .abs()
                    .max((a.start[1] - a.centre[1]).abs());
                if r < MIN_ARC_RADIUS_TICKS {
                    return Err("COPPER_ARC_TOO_SMALL");
                }
                segments.push(Segment::Arc(*a));
                let theta =
                    ((a.start[1] - a.centre[1]) as f64).atan2((a.start[0] - a.centre[0]) as f64);
                let signed_quarter = if a.clockwise {
                    -std::f64::consts::FRAC_PI_2
                } else {
                    std::f64::consts::FRAC_PI_2
                };
                for step in 1..=ARC_STEPS {
                    let p = if step == ARC_STEPS {
                        a.end
                    } else {
                        let angle = theta + signed_quarter * step as f64 / ARC_STEPS as f64;
                        let inner_r = (r - 2) as f64;
                        let x = a.centre[0] + (inner_r * angle.cos()).round() as i64;
                        let y = a.centre[1] + (inner_r * angle.sin()).round() as i64;
                        let dx = i128::from(x) - i128::from(a.centre[0]);
                        let dy = i128::from(y) - i128::from(a.centre[1]);
                        if dx * dx + dy * dy >= i128::from(r) * i128::from(r) {
                            return Err("COPPER_ARC_PROXY_OUTSIDE");
                        }
                        [bounded(x)?, bounded(y)?]
                    };
                    if display.last() == Some(&p) {
                        return Err("COPPER_ARC_PROXY_DEGENERATE");
                    }
                    display.push(p);
                }
            }
        }
        if display.len() > MAX_DISPLAY_VERTICES {
            return Err("COPPER_ARC_PROXY_LIMIT");
        }
    }
    if !outline_topology::ring_valid(&segments) || !valid_ring(&display) || !convex_chords(&display)
    {
        return Err("COPPER_ARC_INVALID_TOPOLOGY");
    }
    // Conservative proxy radius: eight 11.25-degree chords per quarter arc.
    // Worst underfill <= 1-cos(pi/32) + 3/r < 0.8% at r>=1000 ticks.
    // This bound is geometric only; not an exporter, electrical or GPU proof.
    Ok(CurvedCopperCandidate {
        source_segments: segments,
        display_ring: display,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROUNDED: &str =
        "100 0\n300 0 2700 900 200 0 400 200\n400 100\n400 200 0 900 200 100 400 300\n300 300\n100 300 900 900 0 100 200 300\n0 200\n0 100 1800 900 0 0 200 200\n100 0";

    fn check(source: &str) -> Result<CurvedCopperCandidate, &'static str> {
        let rows: Vec<Vec<&str>> = source
            .lines()
            .map(|line| line.split_ascii_whitespace().collect())
            .collect();
        validate(&rows, [0, 0])
    }

    #[test]
    fn rounded_rectangle_retains_four_exact_arcs_and_conservative_proxy() {
        let c = check(ROUNDED).unwrap();
        assert_eq!(c.source_segments.len(), 8);
        assert_eq!(
            c.source_segments
                .iter()
                .filter(|s| matches!(s, Segment::Arc(_)))
                .count(),
            4
        );
        assert_eq!(c.display_ring.len(), 37);
        assert_eq!(c.display_ring.first(), c.display_ring.last());
        assert!(valid_ring(&c.display_ring));
        // Source arcs are never replaced by the display chord proxies.
        assert!(matches!(c.source_segments[1], Segment::Arc(_)));
    }

    #[test]
    fn rejects_unsupported_angle_and_broken_endpoint() {
        assert!(check(&ROUNDED.replace("2700 900", "2700 450")).is_err());
        assert!(check(&ROUNDED.replace("400 100\n", "401 100\n")).is_err());
        assert!(check(&ROUNDED.replace("400 200 0 900", "400 200 90 900")).is_err());
        assert!(check(&ROUNDED.replace("100 0\n300", "100 1\n300")).is_err());
    }

    #[test]
    fn refuses_bow_tie_wrong_sweep_and_unbounded_source() {
        assert!(check(&ROUNDED.replace("300 300\n100 300", "300 300\n100 -300")).is_err());
        assert!(check(&ROUNDED.replace("2700 900", "2700 -900")).is_err());
        assert!(validate(
            &ROUNDED
                .lines()
                .map(|row| row.split_ascii_whitespace().collect())
                .collect::<Vec<Vec<&str>>>(),
            [5_590_000, 0]
        )
        .is_err());
    }

    #[test]
    fn no_arc_is_not_misrepresented_as_a_curved_source() {
        assert_eq!(
            check("0 0\n100 0\n100 100\n0 100\n0 0").unwrap_err(),
            "COPPER_ARC_MISSING"
        );
    }
}
