//! D_056_06: PADS 9.4 RT/ST source thermal geometry diagnostics.
//! PADS Layout ASCII Specification 9.4, pp. 38/40:
//! level size RT|ST spokeori outsize spokewidth spokenum.
//! Geometry-only: no filled plane, manufactured barrel, net or SI inference.
use std::collections::BTreeSet;
const MAX_TICKS: i64 = 5_600_000;
const MAX_SPOKES: u32 = 64; // Resource bound, NOT a PADS vendor limit.
const EDGE_GUARD: f64 = 1.0e-5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThermalKind { Round, Square }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThermalPad {
    pub layer_selector: i64, // -2/-1/0 are relative SOURCE selectors.
    pub kind: ThermalKind,
    pub size_ticks: i64,
    pub first_spoke_degrees: u16,
    pub outer_size_ticks: i64,
    pub spoke_width_ticks: i64,
    pub spoke_count: u32,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ThermalZone { Copper, PlaneClearance, Outside, DrillVoid, BoundaryUncertain }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceThermal {
    pub via_name: String,
    pub source_line: usize,
    pub pad: ThermalPad,
}

/// Parse exactly the seven RT/ST fields. Nonthermal rows remain owned by the
/// existing Via parser; no legacy round/square downcast is permitted.
pub fn parse_thermal_row(row: &str) -> Result<Option<ThermalPad>, &'static str> {
    let w: Vec<_> = row.split_ascii_whitespace().collect();
    if w.len() < 3 { return Err("D05606_INVALID_THERMAL_ROW"); }
    let kind = match w[2] {
        "RT" => ThermalKind::Round, "ST" => ThermalKind::Square,
        _ => return Ok(None),
    };
    if w.len() != 7 { return Err("D05606_INVALID_THERMAL_FIELD_COUNT"); }
    let layer = w[0].parse::<i64>().map_err(|_| "D05606_INVALID_THERMAL_LAYER")?;
    if !matches!(layer, -2 | -1 | 0 | 1..=64) {
        return Err("D05606_INVALID_THERMAL_LAYER");
    }
    // The V9.4 VIA grammar specifies integer-degree spoke orientation.
    let angle = w[3].parse::<u16>().map_err(|_| "D05606_INVALID_THERMAL_ANGLE")?;
    if angle >= 360 { return Err("D05606_INVALID_THERMAL_ANGLE"); }
    let count = w[6].parse::<u32>().map_err(|_| "D05606_INVALID_THERMAL_SPOKE_COUNT")?;
    let pad = ThermalPad {
        layer_selector: layer, kind,
        size_ticks: crate::route_preview::parse_fixed(w[1])
            .map_err(|_| "D05606_INVALID_THERMAL_SIZE")?,
        first_spoke_degrees: angle,
        outer_size_ticks: crate::route_preview::parse_fixed(w[4])
            .map_err(|_| "D05606_INVALID_THERMAL_OUTER_SIZE")?,
        spoke_width_ticks: crate::route_preview::parse_fixed(w[5])
            .map_err(|_| "D05606_INVALID_THERMAL_SPOKE_WIDTH")?,
        spoke_count: count,
    };
    pad.validate()?;
    Ok(Some(pad))
}

impl ThermalPad {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !(1..=MAX_TICKS).contains(&self.size_ticks)
            || self.outer_size_ticks <= self.size_ticks
            || self.outer_size_ticks > MAX_TICKS
            || self.spoke_width_ticks <= 0
            || self.spoke_width_ticks > self.size_ticks
            || !(1..=MAX_SPOKES).contains(&self.spoke_count)
            || self.first_spoke_degrees >= 360
            || !matches!(self.layer_selector, -2 | -1 | 0 | 1..=64)
        { return Err("D05606_INVALID_THERMAL_DIMENSIONS"); }
        Ok(())
    }

    pub fn angles_degrees(&self) -> impl Iterator<Item = f64> + '_ {
        (0..self.spoke_count).map(|n| f64::from(self.first_spoke_degrees)
            + 360.0 * f64::from(n) / f64::from(self.spoke_count))
    }

    /// Local diagnostic only, never a verified plane-to-spoke electrical edge.
    /// Spokes are clipped to the SOURCE outer RT circle / ST square.
    pub fn zone(&self, x: f64, y: f64, drill_ticks: i64) -> ThermalZone {
        if self.validate().is_err() || !x.is_finite() || !y.is_finite()
            || drill_ticks < 0 || drill_ticks >= self.size_ticks {
            return ThermalZone::BoundaryUncertain;
        }
        let radius = x.hypot(y);
        let hole = drill_ticks as f64 / 2.0;
        if drill_ticks > 0 && (radius - hole).abs() <= EDGE_GUARD {
            return ThermalZone::BoundaryUncertain;
        }
        if radius < hole { return ThermalZone::DrillVoid; }
        let bound = match self.kind {
            ThermalKind::Round => radius,
            ThermalKind::Square => x.abs().max(y.abs()),
        };
        let inner = self.size_ticks as f64 / 2.0;
        let outer = self.outer_size_ticks as f64 / 2.0;
        if (bound - outer).abs() <= EDGE_GUARD { return ThermalZone::BoundaryUncertain; }
        if bound > outer { return ThermalZone::Outside; }
        if (bound - inner).abs() <= EDGE_GUARD { return ThermalZone::BoundaryUncertain; }
        if bound < inner { return ThermalZone::Copper; }
        let half_width = self.spoke_width_ticks as f64 / 2.0;
        for deg in self.angles_degrees() {
            let (sn, cs) = deg.to_radians().sin_cos();
            let along = x * cs + y * sn;
            let across = -x * sn + y * cs;
            if along < -EDGE_GUARD { continue; }
            if (across.abs() - half_width).abs() <= EDGE_GUARD
                || along.abs() <= EDGE_GUARD {
                return ThermalZone::BoundaryUncertain;
            }
            if along > 0.0 && across.abs() < half_width {
                return ThermalZone::Copper;
            }
        }
        ThermalZone::PlaneClearance
    }
}

/// SOURCE-only diagnostic inventory. The existing production importer keeps
/// RT/ST fail-closed until geometric contact, versioned IPC and Qt are proved.
pub fn scan_via_thermals(source: &str) -> Result<Vec<SourceThermal>, &'static str> {
    if source.len() > 4 * 1024 * 1024
        || !source.lines().next().is_some_and(|l|
            l.starts_with("!PADS-POWERPCB-V9.4-MILS!")) {
        return Err("D05606_UNSUPPORTED_SOURCE_OR_SIZE");
    }
    let lines: Vec<_> = source.lines().collect();
    let (mut in_via, mut i) = (false, 1usize);
    let (mut result, mut seen) = (Vec::new(), BTreeSet::new());
    while i < lines.len() {
        let line = lines[i].trim();
        let key = line.split_ascii_whitespace().next();
        if key == Some("*END*") { break; }
        if line.starts_with('*') {
            in_via = key == Some("*VIA*");
            i += 1;
            continue;
        }
        if !in_via || line.is_empty() || line.starts_with("REMARK ") {
            i += 1;
            continue;
        }
        let h: Vec<_> = line.split_ascii_whitespace().collect();
        if (h.len() != 3 && h.len() != 5)
            || h[0].is_empty() || h[0].len() > 64 || !h[0].is_ascii()
            || h[0].contains('|') {
            return Err("D05606_INVALID_VIA_HEADER");
        }
        if h.len() == 5
            && (h[3].parse::<u32>().is_err() || h[4].parse::<u32>().is_err()) {
            return Err("D05606_INVALID_VIA_SPAN");
        }
        let count = h[2].parse::<usize>()
            .map_err(|_| "D05606_INVALID_VIA_ROW_COUNT")?;
        if !(2..=66).contains(&count) || i + count >= lines.len() {
            return Err("D05606_TRUNCATED_VIA_RECORD");
        }
        for j in 1..=count {
            let row = lines[i + j].trim();
            if row.starts_with('*') { return Err("D05606_TRUNCATED_VIA_RECORD"); }
            if let Some(pad) = parse_thermal_row(row)? {
                if !seen.insert((h[0].to_owned(), pad.layer_selector)) {
                    return Err("D05606_DUPLICATE_THERMAL_LAYER");
                }
                if result.len() >= 100_000 { return Err("D05606_THERMAL_LIMIT"); }
                result.push(SourceThermal {
                    via_name: h[0].to_owned(), source_line: i + j + 1, pad,
                });
            }
        }
        i += count + 1;
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pads_v94_page_40_rt_example_is_not_solid_copper() {
        let p = parse_thermal_row("-2 60 RT 45 84 15 4").unwrap().unwrap();
        assert_eq!((p.size_ticks, p.outer_size_ticks, p.spoke_width_ticks),
                   (6000, 8400, 1500));
        assert_eq!(p.angles_degrees().collect::<Vec<_>>(),
                   vec![45., 135., 225., 315.]);
        assert_eq!(p.zone(2800., 2800., 1500), ThermalZone::Copper);
        assert_eq!(p.zone(3700., 0., 1500), ThermalZone::PlaneClearance);
        assert_eq!(p.zone(0., 0., 1500), ThermalZone::DrillVoid);
        assert_eq!(p.zone(5000., 0., 1500), ThermalZone::Outside);
    }
    #[test]
    fn square_spokes_and_void_are_not_interchangeable() {
        let p = parse_thermal_row("-1 60 ST 0 84 15 4").unwrap().unwrap();
        assert_eq!(p.zone(3800., 0., 1500), ThermalZone::Copper);
        assert_eq!(p.zone(3700., 3700., 1500), ThermalZone::PlaneClearance);
        assert_eq!(p.zone(4200., 0., 1500), ThermalZone::BoundaryUncertain);
    }
    #[test]
    fn invalid_parameters_and_original_sections_fail_closed() {
        for bad in [
            "-2 60 RT 45 84 0 4", "-2 60 RT 45 84 15 0",
            "-2 60 RT 45 84 15 65", "-2 60 RT 45 50 15 4",
            "-2 60 RT 360 84 15 4", "-2 60 RT 45.5 84 15 4",
            "-2 60 RT 45 84 15", "-3 60 RT 45 84 15 4"
        ] { assert!(parse_thermal_row(bad).is_err(), "{bad}"); }
        assert_eq!(parse_thermal_row("-2 60 R").unwrap(), None);
        let src = "!PADS-POWERPCB-V9.4-MILS!\n*VIA*\nA 15 2\n-2 60 RT 45 84 15 4\n0 50 R\n*END*\n";
        let entries = scan_via_thermals(src).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!((entries[0].via_name.as_str(), entries[0].source_line), ("A", 4));
        assert!(scan_via_thermals(&src.replace("0 50 R", "*END*")).is_err());
        assert!(scan_via_thermals(&src.replace("*VIA*", "*PART*")).unwrap().is_empty());
        assert_eq!(scan_via_thermals(
            &src.replace("*END*", "*END*\n*VIA*\nB 15 2\n-2 60 RT 0 84 15 4\n0 50 R"))
            .unwrap().len(), 1);
    }
}