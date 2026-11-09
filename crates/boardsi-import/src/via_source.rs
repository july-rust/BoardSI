//! PADS Layout ASCII V9.4 VIA definition reader for source-backed spans.
//! The published grammar uses name drill stacklines [drill_start drill_end]
//! followed by level size shape [inner_diameter] rows. -2/-1/0 are relative
//! top/interior/bottom *pad-stack selectors*, not copper layer IDs.
//! Geometry-only: unknown pad shapes and missing/contradictory rows fail closed.
use crate::route_preview::parse_fixed;
use crate::via_shapes::{NormalizedViaShapes, ViaLayerFootprint, ViaLayerShape};
use crate::via_topology::{CircularLayerPad, NormalizedVia};
use boardsi_model::ViaKind;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SourcePad {
    size_ticks: i64,
    shape: ViaLayerShape,
}

#[derive(Clone, Debug)]
pub struct ViaDefinition {
    drill_ticks: i64,
    span: Option<(u32, u32)>,
    top: SourcePad,
    interior: Option<SourcePad>,
    bottom: SourcePad,
    explicit_layers: BTreeMap<u32, SourcePad>,
}

impl ViaDefinition {
    /// Preserve the original R-only interface for the existing safe consumers.
    /// A square or plane anti-pad must never be misrepresented as a circle.
    pub fn normalize(&self, copper_layers: u32) -> Result<NormalizedVia, &'static str> {
        let shaped = self.normalize_shapes(copper_layers)?;
        if !shaped.is_only_round_copper() {
            return Err("D056_NONCIRCULAR_VIA_REQUIRES_SHAPED_PATH");
        }
        let normalized = NormalizedVia {
            copper_layers: shaped.copper_layers,
            first_layer: shaped.first_layer,
            last_layer: shaped.last_layer,
            kind: shaped.kind,
            drill_ticks: shaped.drill_ticks,
            plated: shaped.plated,
            pads: shaped
                .pads
                .into_iter()
                .map(|p| CircularLayerPad {
                    layer: p.layer,
                    diameter_ticks: p.size_ticks,
                })
                .collect(),
        };
        normalized.validate()?;
        Ok(normalized)
    }

    /// Data-only normalization of actual PADS 9.4 conductive and plane-void
    /// shapes. R/S are copper; RA/SA are anti-pads (never copper).
    pub fn normalize_shapes(
        &self,
        copper_layers: u32,
    ) -> Result<NormalizedViaShapes, &'static str> {
        if !(2..=64).contains(&copper_layers) {
            return Err("D056_INVALID_VIA_STACKUP");
        }
        let (first, last) = self.span.unwrap_or((1, copper_layers));
        if first == 0 || last <= first || last > copper_layers {
            return Err("D056_INVALID_VIA_SPAN_OR_DRILL");
        }
        let kind = if first == 1 && last == copper_layers {
            ViaKind::Through
        } else if first == 1 || last == copper_layers {
            ViaKind::Blind
        } else {
            ViaKind::Buried
        };
        let mut layer_map = BTreeMap::<u32, SourcePad>::new();
        layer_map.insert(first, self.top);
        layer_map.insert(last, self.bottom);
        if let Some(pad) = self.interior {
            for layer in first + 1..last {
                layer_map.insert(layer, pad);
            }
        }
        for (&layer, &pad) in &self.explicit_layers {
            if layer <= first || layer >= last {
                return Err("D056_EXPLICIT_VIA_LAYER_CONFLICT");
            }
            layer_map.insert(layer, pad);
        }
        let normalized = NormalizedViaShapes {
            copper_layers,
            first_layer: first,
            last_layer: last,
            kind,
            drill_ticks: self.drill_ticks,
            plated: true,
            pads: layer_map
                .into_iter()
                .map(|(layer, pad)| ViaLayerFootprint {
                    layer,
                    size_ticks: pad.size_ticks,
                    shape: pad.shape,
                })
                .collect(),
        };
        normalized.validate()?;
        Ok(normalized)
    }
}

/// Do not expand or merge a truncated VIA record with post-*END* bytes.
/// Each declared definition either normalizes, or has a source-specific error.
pub fn scan_definitions(text: &str) -> BTreeMap<String, Result<ViaDefinition, &'static str>> {
    let lines: Vec<_> = text
        .lines()
        .take_while(|line| line.split_ascii_whitespace().next() != Some("*END*"))
        .collect();
    let mut result = BTreeMap::new();
    let mut in_via = false;
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        let key = line.split_ascii_whitespace().next().unwrap_or("");
        if key == "*REMARK*" || line.is_empty() || line.starts_with("REMARK ") {
            i += 1;
            continue;
        }
        if line.starts_with('*') {
            in_via = key == "*VIA*";
            i += 1;
            continue;
        }
        if !in_via {
            i += 1;
            continue;
        }
        let header: Vec<_> = line.split_ascii_whitespace().collect();
        if header.len() != 3 && header.len() != 5 {
            i += 1;
            continue;
        }
        let name = header[0];
        if !name.is_ascii() || name.len() > 64 || name.contains('|') {
            i += 1;
            continue;
        }
        let count = header[2].parse::<usize>().ok();
        let drill = parse_fixed(header[1]);
        let span = if header.len() == 5 {
            match (header[3].parse::<u32>(), header[4].parse::<u32>()) {
                (Ok(first), Ok(last)) => Some((first, last)),
                _ => {
                    result.insert(name.into(), Err("D056_INVALID_VIA_SPAN"));
                    i += 1;
                    continue;
                }
            }
        } else {
            None
        };
        let definition = match (drill, count) {
            (Ok(drill_ticks), Some(count)) if (2..=66).contains(&count) => {
                if i + count >= lines.len() {
                    Err("D056_TRUNCATED_VIA_DEFINITION")
                } else {
                    parse_rows(&lines[i + 1..=i + count], drill_ticks, span)
                }
            }
            _ => Err("D056_INVALID_VIA_DEFINITION"),
        };
        if result.insert(name.into(), definition).is_some() {
            result.insert(name.into(), Err("D056_DUPLICATE_VIA_DEFINITION"));
        }
        // Never consume a subsequent section or *END* as via pad data.
        // In case of malformed rows, advance by one to preserve strict
        // recognition of the following definition rather than invent pads.
        if let Some(count) = count {
            if (2..=66).contains(&count)
                && i + count < lines.len()
                && lines[i + 1..=i + count]
                    .iter()
                    .all(|row| !row.trim_start().starts_with('*'))
            {
                i += count + 1;
                continue;
            }
        }
        i += 1;
    }
    result
}

fn parse_rows(
    rows: &[&str],
    drill_ticks: i64,
    span: Option<(u32, u32)>,
) -> Result<ViaDefinition, &'static str> {
    let mut top = None;
    let mut bottom = None;
    let mut interior = None;
    let mut explicit_layers = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for row in rows {
        let parts: Vec<_> = row.split_ascii_whitespace().collect();
        if parts.len() < 3 {
            return Err("D056_INVALID_VIA_PAD_RECORD");
        }
        let shape = match parts[2] {
            "R" if parts.len() == 3 => ViaLayerShape::Round,
            "S" if parts.len() == 3 => ViaLayerShape::Square,
            "RA" if parts.len() == 3 => ViaLayerShape::RoundAntiPad,
            "SA" if parts.len() == 3 => ViaLayerShape::SquareAntiPad,
            "A" if parts.len() == 4 => ViaLayerShape::Annular {
                inner_diameter_ticks: parse_fixed(parts[3])?,
            },
            "RT" | "ST" => {
                let thermal = crate::thermal_source::parse_thermal_row(row)?
                    .ok_or("D05606_THERMAL_ROW_MISSING")?;
                ViaLayerShape::Thermal {
                    kind: thermal.kind,
                    first_spoke_degrees: thermal.first_spoke_degrees,
                    outer_size_ticks: thermal.outer_size_ticks,
                    spoke_width_ticks: thermal.spoke_width_ticks,
                    spoke_count: thermal.spoke_count,
                }
            }
            _ => return Err("D056_UNSUPPORTED_VIA_PAD_SHAPE"),
        };
        let level = parts[0]
            .parse::<i64>()
            .map_err(|_| "D056_INVALID_VIA_PAD_LAYER")?;
        let width = parse_fixed(parts[1])?;
        let pad = SourcePad {
            size_ticks: width,
            shape,
        };
        if !seen.insert(level) {
            return Err("D056_DUPLICATE_VIA_PAD_LAYER");
        }
        match level {
            -2 => top = Some(pad),
            -1 => interior = Some(pad),
            0 => bottom = Some(pad),
            1..=64 => {
                explicit_layers.insert(level as u32, pad);
            }
            _ => return Err("D056_INVALID_VIA_PAD_LAYER"),
        }
    }
    Ok(ViaDefinition {
        drill_ticks,
        span,
        top: top.ok_or("D056_MISSING_TOP_VIA_PAD")?,
        interior,
        bottom: bottom.ok_or("D056_MISSING_BOTTOM_VIA_PAD")?,
        explicit_layers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_driven_partial_spans_use_actual_start_and_end() {
        for (first, last, expected) in [
            (2, 4, ViaKind::Buried),
            (2, 3, ViaKind::Buried),
            (1, 3, ViaKind::Blind),
            (3, 6, ViaKind::Blind),
            (1, 6, ViaKind::Through),
        ] {
            let source = format!(
                "*VIA*\nCUSTOM 15 3 {first} {last}\n-2 35 R\n-1 42 R\n0 39 R\n*ROUTE*\n*END*\n"
            );
            let defs = scan_definitions(&source);
            let via = defs["CUSTOM"].as_ref().unwrap().normalize(6).unwrap();
            assert_eq!(
                (via.first_layer, via.last_layer, via.kind),
                (first, last, expected)
            );
            assert_eq!(via.diameter_on(first), Some(3500));
            assert_eq!(via.diameter_on(last), Some(3900));
            if last > first + 1 {
                assert_eq!(via.diameter_on(first + 1), Some(4200));
            } else {
                assert_eq!(via.pads.len(), 2);
            }
        }
    }

    #[test]
    fn explicit_inner_layer_override_does_not_change_other_layers() {
        let defs =
            scan_definitions("*VIA*\nCUSTOM 15 4 2 5\n-2 35 R\n-1 42 R\n4 50 R\n0 39 R\n*END*\n");
        let via = defs["CUSTOM"].as_ref().unwrap().normalize(6).unwrap();
        assert_eq!(via.diameter_on(3), Some(4200));
        assert_eq!(via.diameter_on(4), Some(5000));
        assert_eq!(via.diameter_on(5), Some(3900));
        assert_eq!(via.diameter_on(1), None);
    }

    #[test]
    fn unsupported_shape_duplicate_source_and_bad_spans_fail_closed() {
        for (row, reason) in [
            ("-2 35 O", "D056_UNSUPPORTED_VIA_PAD_SHAPE"),
            ("-2 35 A", "D056_UNSUPPORTED_VIA_PAD_SHAPE"),
            ("-2 35 R\n-2 36 R", "D056_DUPLICATE_VIA_PAD_LAYER"),
        ] {
            let count = if row.contains('\n') { 3 } else { 2 };
            let source = format!("*VIA*\nA 15 {count} 2 3\n{row}\n0 40 R\n*END*\n");
            let defs = scan_definitions(&source);
            assert_eq!(defs["A"].as_ref().unwrap_err(), &reason);
        }
        let bad = scan_definitions("*VIA*\nA 15 2 4 2\n-2 35 R\n0 40 R\n*END*");
        assert_eq!(
            bad["A"].as_ref().unwrap().normalize(6).unwrap_err(),
            "D056_INVALID_VIA_SPAN_OR_DRILL"
        );
    }

    #[test]
    fn post_end_and_truncated_rows_must_not_define_copper() {
        let defs = scan_definitions(
            "*VIA*\nTRUNC 15 3 2 4\n-2 35 R\n0 35 R\n*END*\n*VIA*\nTRUNC 15 2 2 4\n-2 35 R\n0 35 R\n"
        );
        assert!(defs["TRUNC"].is_err());
    }

    #[test]
    fn square_and_plane_antipad_rows_keep_roles_on_a_buried_stack() {
        let defs = scan_definitions("*VIA*\nV_TEST 15 3 2 4\n-2 35 S\n-1 60 RA\n0 40 R\n*END*\n");
        let shapes = defs["V_TEST"]
            .as_ref()
            .unwrap()
            .normalize_shapes(6)
            .unwrap();
        assert_eq!(shapes.pads.len(), 3);
        assert_eq!(shapes.layer_pad(2).unwrap().shape, ViaLayerShape::Square);
        assert_eq!(
            shapes.layer_pad(3).unwrap().shape,
            ViaLayerShape::RoundAntiPad
        );
        assert_eq!(shapes.conductive_pad(3), None);
        assert_eq!(shapes.layer_pad(4).unwrap().shape, ViaLayerShape::Round);
        assert_eq!(
            defs["V_TEST"].as_ref().unwrap().normalize(6).unwrap_err(),
            "D056_NONCIRCULAR_VIA_REQUIRES_SHAPED_PATH"
        );
    }

    #[test]
    fn annular_a_inner_diameter_is_source_owned_and_not_drill_only() {
        let defs = scan_definitions("*VIA*\nV_TEST 15 3 2 4\n-2 35 S\n-1 65 A 50\n0 40 R\n*END*");
        let shapes = defs["V_TEST"]
            .as_ref()
            .unwrap()
            .normalize_shapes(6)
            .unwrap();
        assert_eq!(
            shapes.layer_pad(3).unwrap().shape,
            ViaLayerShape::Annular {
                inner_diameter_ticks: 5000
            }
        );
        assert_eq!(shapes.layer_pad(3).unwrap().size_ticks, 6500);
        assert!(shapes.conductive_pad(3).is_some());
        let bad = scan_definitions("*VIA*\nV_TEST 15 3 2 4\n-2 35 S\n-1 65 A 10\n0 40 R\n*END*");
        assert_eq!(
            bad["V_TEST"]
                .as_ref()
                .unwrap()
                .normalize_shapes(6)
                .unwrap_err(),
            "D056_INVALID_ANNULAR_INNER_DIAMETER"
        );
    }

    #[test]
    fn thermal_rt_st_source_fields_survive_through_and_buried_stack() {
        for (layers, first, last, kind) in [
            (2, 1, 2, ViaKind::Through),
            (4, 1, 4, ViaKind::Through),
            (6, 2, 4, ViaKind::Buried),
            (8, 2, 3, ViaKind::Buried),
            (64, 2, 4, ViaKind::Buried),
        ] {
            let src = format!(
                "*VIA*\nV_T 15 3 {first} {last}\n-2 60 RT 45 84 15 4\n-1 50 ST 0 75 10 4\n0 50 R\n*END*\n"
            );
            let def = scan_definitions(&src);
            let shaped = def["V_T"]
                .as_ref()
                .unwrap()
                .normalize_shapes(layers)
                .unwrap();
            assert_eq!(shaped.kind, kind);
            assert_eq!(shaped.first_layer, first);
            let pad = shaped.conductive_pad(first).unwrap();
            assert!(matches!(
                pad.shape,
                ViaLayerShape::Thermal {
                    kind: crate::thermal_source::ThermalKind::Round,
                    first_spoke_degrees: 45,
                    outer_size_ticks: 8400,
                    spoke_width_ticks: 1500,
                    spoke_count: 4,
                }
            ));
            assert!(def["V_T"].as_ref().unwrap().normalize(layers).is_err());
            if last > first + 1 {
                assert!(matches!(
                    shaped.conductive_pad(first + 1).unwrap().shape,
                    ViaLayerShape::Thermal {
                        kind: crate::thermal_source::ThermalKind::Square,
                        ..
                    }
                ));
            }
        }
    }

    #[test]
    fn unnamed_source_microvia_is_not_inferred_from_one_layer_step() {
        let defs = scan_definitions("*VIA*\nUNLABELED 15 2 1 2\n-2 35 R\n0 35 R\n*END*");
        assert_eq!(
            defs["UNLABELED"]
                .as_ref()
                .unwrap()
                .normalize(6)
                .unwrap()
                .kind,
            ViaKind::Blind
        );
    }
}
