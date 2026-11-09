//! D_056_06 conservative local SOURCE Copper thermal-to-trace witnesses.
//! Positive requires a strict interior point on both finite-width trace and
//! source RT/ST pad/spoke. Missing witness = UNKNOWN, not a fabricated OPEN.
//! Never promote plane-fill continuity, barrel process, net or global SI.
use crate::thermal_source::{ThermalPad, ThermalZone};
use crate::trace_contact::{self, Centreline, Contact, Stroke, BOUNDARY_GUARD_TICKS};
use std::f64::consts::TAU;

fn sample_thermal(
    thermal: &ThermalPad,
    point: [i64; 2],
    drill_ticks: i64,
    centre: [f64; 2],
    tangent: [f64; 2],
    trace_half_width: f64,
) -> bool {
    let norm = tangent[0].hypot(tangent[1]);
    if norm <= 0.0 || !norm.is_finite() {
        return false;
    }
    let normal = [-tangent[1] / norm, tangent[0] / norm];
    // Every witness is strictly interior to the true finite-width stroke.
    for scale in [-0.5, 0.0, 0.5] {
        let x = centre[0] + normal[0] * scale * trace_half_width - point[0] as f64;
        let y = centre[1] + normal[1] * scale * trace_half_width - point[1] as f64;
        if thermal.zone(x, y, drill_ticks) == ThermalZone::Copper {
            return true;
        }
    }
    false
}

/// SOUND-POSITIVE/BROADPHASE-NEGATIVE test. Unknown boundaries, unresolved
/// spokes and all unavailable plane pour remain BoundaryUncertain.
pub fn trace_thermal_candidate(
    trace: &Stroke,
    point: [i64; 2],
    drill_ticks: i64,
    thermal: &ThermalPad,
) -> Result<Contact, &'static str> {
    thermal.validate()?;
    let _ = trace_contact::centreline_envelope(trace)?;
    if trace.layer as i64 != thermal.layer_selector
        || point
            .iter()
            .any(|&v| !(-5_600_000..=5_600_000).contains(&v))
        || drill_ticks < 0
        || drill_ticks >= thermal.size_ticks
    {
        return Err("D05606_THERMAL_TRACE_METADATA");
    }
    let half = trace.width_ticks as f64 / 2.0;
    let nearest = trace_contact::centreline_point_distance(trace, point)?;
    // Square's circumscribing circle is larger than a round pad of same outsize.
    let bound = thermal.outer_size_ticks as f64 / 2.0
        * if thermal.kind == crate::thermal_source::ThermalKind::Square {
            std::f64::consts::SQRT_2
        } else {
            1.0
        };
    if nearest > bound + half + BOUNDARY_GUARD_TICKS {
        return Ok(Contact::Separated);
    }
    let farthest = trace_contact::centreline_point_farthest(trace, point)?;
    if farthest + half < drill_ticks as f64 / 2.0 - BOUNDARY_GUARD_TICKS {
        return Ok(Contact::Separated);
    }
    const SAMPLES: usize = 128; // strict interior proof, not a negative oracle
    match &trace.centreline {
        Centreline::Straight { start, end } => {
            let dx = (end[0] - start[0]) as f64;
            let dy = (end[1] - start[1]) as f64;
            for k in 0..=SAMPLES {
                let t = k as f64 / SAMPLES as f64;
                let c = [start[0] as f64 + t * dx, start[1] as f64 + t * dy];
                if sample_thermal(thermal, point, drill_ticks, c, [dx, dy], half) {
                    return Ok(Contact::PositiveArea);
                }
            }
        }
        Centreline::Circular {
            centre,
            start,
            end,
            clockwise,
        } => {
            let sx = (start[0] - centre[0]) as f64;
            let sy = (start[1] - centre[1]) as f64;
            let ex = (end[0] - centre[0]) as f64;
            let ey = (end[1] - centre[1]) as f64;
            let from = sy.atan2(sx);
            let to = ey.atan2(ex);
            let sweep = if *clockwise {
                -(from - to).rem_euclid(TAU)
            } else {
                (to - from).rem_euclid(TAU)
            };
            let radius = sx.hypot(sy);
            if radius <= 0.0 {
                return Err("D05606_INVALID_SOURCE_ARC");
            }
            for k in 0..=SAMPLES {
                let a = from + sweep * k as f64 / SAMPLES as f64;
                let c = [
                    centre[0] as f64 + radius * a.cos(),
                    centre[1] as f64 + radius * a.sin(),
                ];
                if sample_thermal(thermal, point, drill_ticks, c, [-a.sin(), a.cos()], half) {
                    return Ok(Contact::PositiveArea);
                }
            }
        }
    }
    Ok(Contact::BoundaryUncertain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thermal_source::ThermalKind;
    fn thermal() -> ThermalPad {
        ThermalPad {
            layer_selector: 2,
            kind: ThermalKind::Round,
            size_ticks: 6000,
            outer_size_ticks: 8400,
            spoke_width_ticks: 1500,
            first_spoke_degrees: 45,
            spoke_count: 4,
        }
    }
    fn line(a: [i64; 2], b: [i64; 2], width: i64, layer: u32) -> Stroke {
        Stroke {
            layer,
            net: "NET_A".into(),
            width_ticks: width,
            source_line: 9,
            centreline: Centreline::Straight { start: a, end: b },
        }
    }
    #[test]
    fn spoke_and_pad_interior_positive_but_clearance_never_forced_positive() {
        let t = thermal();
        let yes = line([2500, 2500], [3000, 3000], 100, 2);
        assert_eq!(
            trace_thermal_candidate(&yes, [0, 0], 1500, &t),
            Ok(Contact::PositiveArea)
        );
        let mid = line([3600, 0], [3800, 0], 20, 2);
        assert_ne!(
            trace_thermal_candidate(&mid, [0, 0], 1500, &t),
            Ok(Contact::PositiveArea)
        );
        let out = line([5000, 0], [6000, 0], 20, 2);
        assert_eq!(
            trace_thermal_candidate(&out, [0, 0], 1500, &t),
            Ok(Contact::Separated)
        );
        let centre = line([900, 0], [2000, 0], 10, 2);
        assert_eq!(
            trace_thermal_candidate(&centre, [0, 0], 1500, &t),
            Ok(Contact::PositiveArea)
        );
    }
}
