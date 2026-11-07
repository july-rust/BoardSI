//! D_056: source-agnostic stackup-aware via geometry normalization.
//! Structural only: no proof of plating continuity, contact, or SI readiness.
use boardsi_model::{ObjectId, PadRole, PadShape, PadstackLayer, Rational, ViaKind};
use std::collections::BTreeSet;

/// Copper layer number after source-to-stackup mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CircularLayerPad {
    pub layer: u32,
    pub diameter_ticks: i64,
}

/// Data-driven circular via representation, with explicit source-confirmed span.
#[derive(Clone, Debug)]
pub struct NormalizedVia {
    pub copper_layers: u32,
    pub first_layer: u32,
    pub last_layer: u32,
    pub kind: ViaKind,
    pub drill_ticks: i64,
    pub plated: bool,
    pub pads: Vec<CircularLayerPad>,
}

impl NormalizedVia {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !(2..=64).contains(&self.copper_layers)
            || self.first_layer == 0
            || self.last_layer <= self.first_layer
            || self.last_layer > self.copper_layers
            || !(1..=100_000).contains(&self.drill_ticks)
        {
            return Err("D056_INVALID_VIA_SPAN_OR_DRILL");
        }
        let top = self.first_layer == 1;
        let bottom = self.last_layer == self.copper_layers;
        let valid = match self.kind {
            ViaKind::Through => top && bottom,
            ViaKind::Blind => top != bottom,
            ViaKind::Buried => !top && !bottom,
            ViaKind::Microvia => self.last_layer == self.first_layer + 1,
        };
        if !valid {
            return Err("D056_VIA_KIND_SPAN_CONFLICT");
        }
        if self.pads.is_empty() {
            return Err("D056_MISSING_LAYER_PADS");
        }
        let mut seen = BTreeSet::new();
        for pad in &self.pads {
            if !(self.first_layer..=self.last_layer).contains(&pad.layer)
                || pad.diameter_ticks <= self.drill_ticks
                || pad.diameter_ticks > 5_600_000
                || !seen.insert(pad.layer)
            {
                return Err("D056_INVALID_LAYER_PAD");
            }
        }
        Ok(())
    }

    pub fn diameter_on(&self, layer: u32) -> Option<i64> {
        self.pads
            .iter()
            .find(|p| p.layer == layer)
            .map(|p| p.diameter_ticks)
    }

    /// Only explicit conductive circles. No inferred intermediary connection.
    pub fn canonical_circular_pads(
        &self,
        layer_id: impl Fn(u32) -> ObjectId,
    ) -> Result<Vec<PadstackLayer>, &'static str> {
        self.validate()?;
        self.pads
            .iter()
            .map(|pad| {
                let diameter_m = Rational::new(i128::from(pad.diameter_ticks) * 127, 500_000_000)
                    .map_err(|_| "D056_VIA_DIMENSION_OVERFLOW")?;
                Ok(PadstackLayer {
                    layer: layer_id(pad.layer),
                    role: PadRole::Conductive,
                    shape: PadShape::Circle { diameter_m },
                })
            })
            .collect()
    }

    /// PADS 9.4 -2/-1/0: top, repeating inner pad, bottom.
    /// This mapping is format-specific; the generic contract above is not.
    pub fn pads_v94_through(
        copper_layers: u32,
        drill_ticks: i64,
        top_ticks: i64,
        inner_ticks: Option<i64>,
        bottom_ticks: i64,
    ) -> Result<Self, &'static str> {
        if !(2..=64).contains(&copper_layers) {
            return Err("D056_INVALID_VIA_STACKUP");
        }
        let mut pads = vec![CircularLayerPad {
            layer: 1,
            diameter_ticks: top_ticks,
        }];
        if let Some(diameter_ticks) = inner_ticks {
            for layer in 2..copper_layers {
                pads.push(CircularLayerPad {
                    layer,
                    diameter_ticks,
                });
            }
        }
        pads.push(CircularLayerPad {
            layer: copper_layers,
            diameter_ticks: bottom_ticks,
        });
        let value = Self {
            copper_layers,
            first_layer: 1,
            last_layer: copper_layers,
            kind: ViaKind::Through,
            drill_ticks,
            plated: true,
            pads,
        };
        value.validate()?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_algorithm_handles_2_4_6_8_64_layer_through_vias() {
        for count in [2, 4, 6, 8, 64] {
            let via = NormalizedVia::pads_v94_through(count, 1500, 3500, Some(3200), 3700)
                .expect("valid through via with unequal per-layer pads");
            assert_eq!(via.pads.len(), count as usize);
            assert_eq!(via.diameter_on(1), Some(3500));
            assert_eq!(via.diameter_on(count), Some(3700));
            if count > 2 {
                assert_eq!(via.diameter_on(2), Some(3200));
            }
        }
    }

    #[test]
    fn arbitrary_partial_spans_are_parameters_not_separate_code() {
        for (count, first, last, kind) in [
            (6, 2, 4, ViaKind::Buried),
            (6, 2, 3, ViaKind::Buried),
            (6, 1, 3, ViaKind::Blind),
            (6, 3, 6, ViaKind::Blind),
            (8, 4, 5, ViaKind::Microvia),
        ] {
            let via = NormalizedVia {
                copper_layers: count,
                first_layer: first,
                last_layer: last,
                kind,
                drill_ticks: 1000,
                plated: true,
                pads: vec![
                    CircularLayerPad {
                        layer: first,
                        diameter_ticks: 2600,
                    },
                    CircularLayerPad {
                        layer: last,
                        diameter_ticks: 3100,
                    },
                ],
            };
            via.validate().unwrap();
            assert_eq!(via.diameter_on(first), Some(2600));
            assert_eq!(via.diameter_on(last), Some(3100));
            if last > first + 1 {
                assert_eq!(via.diameter_on(first + 1), None);
            }
        }
    }

    #[test]
    fn through_via_does_not_invent_inner_pads() {
        let via = NormalizedVia::pads_v94_through(6, 1500, 3500, None, 3700).unwrap();
        assert_eq!(via.pads.len(), 2);
        assert_eq!(via.diameter_on(3), None);
    }

    #[test]
    fn malformed_vias_fail_closed() {
        let base = NormalizedVia {
            copper_layers: 6,
            first_layer: 2,
            last_layer: 4,
            kind: ViaKind::Buried,
            drill_ticks: 1000,
            plated: true,
            pads: vec![CircularLayerPad {
                layer: 2,
                diameter_ticks: 2500,
            }],
        };
        assert!(base.validate().is_ok());
        let mut x = base.clone();
        x.kind = ViaKind::Through;
        assert_eq!(x.validate(), Err("D056_VIA_KIND_SPAN_CONFLICT"));
        let mut x = base.clone();
        x.pads.push(x.pads[0]);
        assert_eq!(x.validate(), Err("D056_INVALID_LAYER_PAD"));
        let mut x = base.clone();
        x.pads.push(CircularLayerPad {
            layer: 5,
            diameter_ticks: 3000,
        });
        assert_eq!(x.validate(), Err("D056_INVALID_LAYER_PAD"));
        let mut x = base.clone();
        x.pads[0].diameter_ticks = 999;
        assert_eq!(x.validate(), Err("D056_INVALID_LAYER_PAD"));
        let mut x = base.clone();
        x.pads.clear();
        assert_eq!(x.validate(), Err("D056_MISSING_LAYER_PADS"));
    }
}
