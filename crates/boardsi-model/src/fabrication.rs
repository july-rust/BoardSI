//! D_056_07 format-neutral fabrication *description*, never fabrication proof.
//! Consumers must not turn a geometry overlap or nominal span into an electrical
//! edge without independent conductive-process and source qualification.
use crate::{ModelError, Rational};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalLayerSpan {
    /// Physical copper-layer order, counting from top (1-based).
    pub first: u32,
    pub last: u32,
}

impl PhysicalLayerSpan {
    pub fn validate(self, copper_layers: u32) -> Result<(), ModelError> {
        if !(2..=64).contains(&copper_layers)
            || self.first == 0
            || self.first >= self.last
            || self.last > copper_layers
        {
            return Err(ModelError::InvalidStackup);
        }
        Ok(())
    }
    pub fn contains(self, layer: u32) -> bool {
        self.first <= layer && layer <= self.last
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FabricationHoleShape {
    Round {
        diameter_m: Rational,
    },
    Slot {
        width_m: Rational,
        /// Native declared slot-length parameter. Do not infer CAM endcaps.
        declared_length_m: Rational,
        orientation_millidegrees: u32,
        offset_along_axis_m: Rational,
    },
}

impl FabricationHoleShape {
    pub fn validate(&self) -> Result<(), ModelError> {
        match self {
            Self::Round { diameter_m } => {
                if !diameter_m.positive() {
                    return Err(ModelError::InvalidDimension);
                }
            }
            Self::Slot {
                width_m,
                declared_length_m,
                orientation_millidegrees,
                ..
            } => {
                if !width_m.positive()
                    || !declared_length_m.positive()
                    || *orientation_millidegrees >= 180_000
                    || declared_length_m.checked_cmp(*width_m)?
                        == std::cmp::Ordering::Less
                {
                    return Err(ModelError::InvalidGeometry);
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FabricationPlating {
    /// No SOURCE-declared metallization; never a conducting barrel.
    NonPlated,
    /// SOURCE says plated, but finished-barrel continuity is not proven.
    SourceDeclaredPlated,
    /// No sufficiently qualified SOURCE/manufacturing evidence.
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FabricationOperation {
    Drilled,
    /// Removal request, not a claim that barrel copper is physically gone.
    Backdrill {
        from_layer: u32,
        to_layer: u32,
        diameter_m: Rational,
    },
    /// Composition of recorded *transitions*, without inferred stacking.
    MicroviaTransitions {
        spans: Vec<PhysicalLayerSpan>,
        source_declared_stacked: Option<bool>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FabricationSourceEvidence {
    pub format: String,
    pub source_sha256: [u8; 32],
    pub source_line: usize,
    pub qualified_native_grammar: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FabricationHole {
    pub shape: FabricationHoleShape,
    pub plating: FabricationPlating,
    pub span: PhysicalLayerSpan,
    pub operation: FabricationOperation,
    pub evidence: FabricationSourceEvidence,
}

impl FabricationHole {
    pub fn validate(&self, copper_layers: u32) -> Result<(), ModelError> {
        self.span.validate(copper_layers)?;
        self.shape.validate()?;
        if self.evidence.format.trim().is_empty()
            || self.evidence.source_line == 0
            || self.evidence.source_sha256 == [0_u8; 32]
        {
            return Err(ModelError::InvalidProvenance);
        }
        match &self.operation {
            FabricationOperation::Drilled => {}
            FabricationOperation::Backdrill {
                from_layer,
                to_layer,
                diameter_m,
            } => {
                if !diameter_m.positive()
                    || !self.span.contains(*from_layer)
                    || !self.span.contains(*to_layer)
                    || from_layer == to_layer
                    || (*from_layer != self.span.first && *from_layer != self.span.last)
                {
                    return Err(ModelError::InvalidGeometry);
                }
                // A backdrill cutting from either end must progress inward.
                if (*from_layer == self.span.first && from_layer >= to_layer)
                    || (*from_layer == self.span.last && from_layer <= to_layer)
                {
                    return Err(ModelError::InvalidGeometry);
                }
            }
            FabricationOperation::MicroviaTransitions { spans, .. } => {
                if spans.is_empty() || spans.len() > 64 {
                    return Err(ModelError::MissingRequired);
                }
                for s in spans {
                    s.validate(copper_layers)?;
                    if s.first < self.span.first || s.last > self.span.last {
                        return Err(ModelError::InvalidConnection);
                    }
                }
                // Multiple declared hops do not prove contact at intersections.
            }
        }
        Ok(())
    }

    /// SOURCE and nominal process data alone can never certify a barrel edge.
    /// A separate fabrication/process qualifier must provide that evidence.
    pub fn has_verified_manufactured_barrel(&self) -> bool {
        false
    }

    /// Mechanical void is never a positive electrical contact.
    pub fn is_electrical_conductor(&self) -> bool {
        false
    }

    /// A plain declared PTH can be a *candidate* for later process proof;
    /// slot, backdrill, or microvia transitions need independent qualification.
    pub fn requires_process_qualification(&self) -> bool {
        !matches!(self.plating, FabricationPlating::NonPlated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence() -> FabricationSourceEvidence {
        FabricationSourceEvidence {
            format: "PADS-V9.4-MILS-PARTDECAL".into(),
            source_sha256: [42; 32],
            source_line: 6,
            qualified_native_grammar: true,
        }
    }
    fn hole(span: PhysicalLayerSpan) -> FabricationHole {
        FabricationHole {
            shape: FabricationHoleShape::Round {
                diameter_m: Rational::new(1, 5_000).unwrap(),
            },
            plating: FabricationPlating::NonPlated,
            span,
            operation: FabricationOperation::Drilled,
            evidence: evidence(),
        }
    }
    #[test]
    fn arbitrary_layer_counts_and_partial_spans_are_data_driven() {
        for layers in [2, 4, 6, 8, 64] {
            let thru = hole(PhysicalLayerSpan {
                first: 1,
                last: layers,
            });
            thru.validate(layers).unwrap();
            assert!(!thru.is_electrical_conductor());
            assert!(!thru.has_verified_manufactured_barrel());
            if layers >= 4 {
                hole(PhysicalLayerSpan { first: 2, last: 3 })
                    .validate(layers)
                    .unwrap();
            }
            assert!(hole(PhysicalLayerSpan {
                first: layers,
                last: layers
            })
            .validate(layers)
            .is_err());
        }
    }

    #[test]
    fn slots_keep_exact_dimensions_and_declared_angle() {
        let mut h = hole(PhysicalLayerSpan { first: 1, last: 8 });
        h.shape = FabricationHoleShape::Slot {
            width_m: Rational::new(1, 2_000).unwrap(),
            declared_length_m: Rational::new(3, 2_000).unwrap(),
            orientation_millidegrees: 179_999,
            offset_along_axis_m: Rational::new(-1, 100_000).unwrap(),
        };
        h.validate(8).unwrap();
        if let FabricationHoleShape::Slot { ref mut orientation_millidegrees, .. } = h.shape {
            *orientation_millidegrees = 180_000;
        }
        assert!(h.validate(8).is_err());
    }

    #[test]
    fn backdrill_and_microvia_require_separate_process_evidence() {
        let mut h = hole(PhysicalLayerSpan { first: 1, last: 8 });
        h.plating = FabricationPlating::SourceDeclaredPlated;
        h.operation = FabricationOperation::Backdrill {
            from_layer: 8,
            to_layer: 5,
            diameter_m: Rational::new(1, 4_000).unwrap(),
        };
        h.validate(8).unwrap();
        assert!(!h.has_verified_manufactured_barrel());
        assert!(h.requires_process_qualification());
        h.operation = FabricationOperation::MicroviaTransitions {
            spans: vec![
                PhysicalLayerSpan { first: 1, last: 2 },
                PhysicalLayerSpan { first: 2, last: 3 },
            ],
            source_declared_stacked: None,
        };
        h.validate(8).unwrap();
        assert!(!h.has_verified_manufactured_barrel());
        h.operation = FabricationOperation::MicroviaTransitions {
            spans: vec![PhysicalLayerSpan { first: 7, last: 9 }],
            source_declared_stacked: Some(true),
        };
        assert!(h.validate(8).is_err());
    }

    #[test]
    fn unqualified_provenance_is_not_silently_accepted() {
        let mut h = hole(PhysicalLayerSpan { first: 2, last: 4 });
        h.evidence.source_sha256 = [0; 32];
        assert_eq!(h.validate(6), Err(ModelError::InvalidProvenance));
    }
}
