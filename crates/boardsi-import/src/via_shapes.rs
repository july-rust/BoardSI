//! D_056: actual per-layer via copper versus plane-clearance geometry.
//! These shapes are source-normalized. They are not evidence of plated barrel
//! continuity, a whole-board electric graph, a microvia process, or SI readiness.
use boardsi_model::{ObjectId, PadRole, PadShape, PadstackLayer, Rational, ViaKind};
use std::collections::BTreeSet;

/// PADS V9.4 R/S/RA/SA. Annular 'A' (inner-diameter) and thermal RT/ST
/// are intentionally not represented as a filled circle/square.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViaLayerShape {
    Round,
    Square,
    RoundAntiPad,
    SquareAntiPad,
}

impl ViaLayerShape {
    pub fn conductive(self) -> bool {
        matches!(self, Self::Round | Self::Square)
    }

    pub fn wire_code(self) -> &'static str {
        match self {
            Self::Round => "R",
            Self::Square => "S",
            Self::RoundAntiPad => "RA",
            Self::SquareAntiPad => "SA",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViaLayerFootprint {
    pub layer: u32,
    pub size_ticks: i64,
    pub shape: ViaLayerShape,
}

#[derive(Clone, Debug)]
pub struct NormalizedViaShapes {
    pub copper_layers: u32,
    pub first_layer: u32,
    pub last_layer: u32,
    pub kind: ViaKind,
    pub drill_ticks: i64,
    pub plated: bool,
    pub pads: Vec<ViaLayerFootprint>,
}

impl NormalizedViaShapes {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !(2..=64).contains(&self.copper_layers)
            || self.first_layer == 0
            || self.first_layer >= self.last_layer
            || self.last_layer > self.copper_layers
            || !(1..=100_000).contains(&self.drill_ticks)
        {
            return Err("D056_INVALID_VIA_SPAN_OR_DRILL");
        }
        let top = self.first_layer == 1;
        let bottom = self.last_layer == self.copper_layers;
        let valid_kind = match self.kind {
            ViaKind::Through => top && bottom,
            ViaKind::Blind => top != bottom,
            ViaKind::Buried => !top && !bottom,
            ViaKind::Microvia => self.first_layer + 1 == self.last_layer,
        };
        if !valid_kind {
            return Err("D056_VIA_KIND_SPAN_CONFLICT");
        }
        let mut seen = BTreeSet::new();
        if self.pads.is_empty() {
            return Err("D056_MISSING_LAYER_PADS");
        }
        for pad in &self.pads {
            if pad.layer < self.first_layer
                || pad.layer > self.last_layer
                || !(1..=5_600_000).contains(&pad.size_ticks)
                || pad.size_ticks <= self.drill_ticks
                || !seen.insert(pad.layer)
            {
                return Err("D056_INVALID_LAYER_PAD");
            }
        }
        if self.pads.iter().all(|p| !p.shape.conductive()) {
            // No electrical landing pad is proven. This isn't a valid
            // conductive via footprint, even if it has plane clearances.
            return Err("D056_NO_CONDUCTIVE_VIA_PAD");
        }
        Ok(())
    }

    pub fn layer_pad(&self, layer: u32) -> Option<ViaLayerFootprint> {
        self.pads.iter().find(|p| p.layer == layer).copied()
    }

    pub fn conductive_pad(&self, layer: u32) -> Option<ViaLayerFootprint> {
        self.layer_pad(layer).filter(|p| p.shape.conductive())
    }

    pub fn is_only_round_copper(&self) -> bool {
        self.pads.iter().all(|p| p.shape == ViaLayerShape::Round)
    }

    /// Constructs the actual canonical roles and shapes. Anti-pads are voids
    /// in a plane, NEVER conductive via-to-trace contacts.
    pub fn canonical_entries(
        &self,
        layer_id: impl Fn(u32) -> ObjectId,
    ) -> Result<Vec<PadstackLayer>, &'static str> {
        self.validate()?;
        self.pads
            .iter()
            .map(|p| {
                let size_m = Rational::new(i128::from(p.size_ticks) * 127, 500_000_000)
                    .map_err(|_| "D056_VIA_DIMENSION_OVERFLOW")?;
                let shape = match p.shape {
                    ViaLayerShape::Round | ViaLayerShape::RoundAntiPad => {
                        PadShape::Circle { diameter_m: size_m }
                    }
                    ViaLayerShape::Square | ViaLayerShape::SquareAntiPad => PadShape::Rectangle {
                        size_m: [size_m; 2],
                    },
                };
                Ok(PadstackLayer {
                    layer: layer_id(p.layer),
                    role: if p.shape.conductive() {
                        PadRole::Conductive
                    } else {
                        PadRole::AntiPad
                    },
                    shape,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> NormalizedViaShapes {
        NormalizedViaShapes {
            copper_layers: 6,
            first_layer: 2,
            last_layer: 4,
            kind: ViaKind::Buried,
            drill_ticks: 1500,
            plated: true,
            pads: vec![
                ViaLayerFootprint {
                    layer: 2,
                    size_ticks: 3200,
                    shape: ViaLayerShape::Square,
                },
                ViaLayerFootprint {
                    layer: 3,
                    size_ticks: 5000,
                    shape: ViaLayerShape::RoundAntiPad,
                },
                ViaLayerFootprint {
                    layer: 4,
                    size_ticks: 4000,
                    shape: ViaLayerShape::Round,
                },
            ],
        }
    }

    #[test]
    fn square_copper_and_plane_antipad_are_distinct() {
        let via = sample();
        via.validate().unwrap();
        assert_eq!(via.conductive_pad(2).unwrap().shape, ViaLayerShape::Square);
        assert_eq!(via.layer_pad(3).unwrap().shape, ViaLayerShape::RoundAntiPad);
        assert!(via.conductive_pad(3).is_none());
        assert!(via.layer_pad(1).is_none());
        assert_eq!(via.conductive_pad(4).unwrap().size_ticks, 4000);
    }

    #[test]
    fn invalid_layers_or_missing_conductive_copper_fail_closed() {
        let mut via = sample();
        via.pads.push(via.pads[0]);
        assert_eq!(via.validate(), Err("D056_INVALID_LAYER_PAD"));
        let mut via = sample();
        via.pads[0].layer = 5;
        assert_eq!(via.validate(), Err("D056_INVALID_LAYER_PAD"));
        let mut via = sample();
        via.pads[0].shape = ViaLayerShape::SquareAntiPad;
        via.pads[2].shape = ViaLayerShape::RoundAntiPad;
        assert_eq!(via.validate(), Err("D056_NO_CONDUCTIVE_VIA_PAD"));
    }

    #[test]
    fn arbitrary_stack_counts_and_buried_spans_are_not_cases() {
        for copper_layers in [4, 6, 8, 64] {
            let mut via = sample();
            via.copper_layers = copper_layers;
            via.validate().unwrap();
        }
    }
}
