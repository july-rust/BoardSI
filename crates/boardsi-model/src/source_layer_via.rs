//! E_004: SOURCE-bound, read-only Layer/Via manufacturing-hole observation.
//!
//! This is a *non-promoting* contract. A source-declared span or plated flag
//! does not establish a manufactured barrel, connected copper, a Canonical
//! `Drill.plated` value, an electrical graph edge, or solver readiness.
//! The narrower SOURCE contract deliberately does not change legacy importers.
use crate::{
    BoardModel, FabricationHole, FabricationPlating, LayerKind, ModelError, ObjectId, Property,
    SourceId,
};

/// This describes SOURCE grammar and declarations, not verified conductivity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceViaEvidenceState {
    UnsupportedGrammar,
    UnknownPlating,
    SourceDeclaredPlated,
    SourceDeclaredNonPlated,
}

/// Source properties only: `Explicit` is NOT a materials/physics approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceStackupEvidence {
    Missing,
    Assumed,
    Explicit,
}

/// A read-only observation. Envelope layers are NOT a connected-layer set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceViaHoleObservation {
    pub source_file: String,
    pub source_line: usize,
    pub first_layer: ObjectId,
    pub last_layer: ObjectId,
    /// Copper layers within the declared span, including layers with no pad.
    /// Never copy these directly into `Via.connected_layers`.
    pub physical_envelope_layers: Vec<ObjectId>,
    pub via_evidence: SourceViaEvidenceState,
    pub stackup_evidence: SourceStackupEvidence,
}

impl BoardModel {
    /// Qualify ONE SOURCE-bound fabrication hole without inserting a Via,
    /// Drill, Pad or graph edge. Physical copper ordinals, not arbitrary
    /// `Layer.order` values, are matched to the source-declared drill span.
    ///
    /// Missing stackup properties and unknown plating are returned distinctly.
    /// Contradictory/ambiguous source identity or layer/span mapping is an
    /// explicit error; no fallback through net names, display layers or vias.
    pub fn observe_source_via_hole(
        &self,
        source: &SourceId,
        hole: &FabricationHole,
    ) -> Result<SourceViaHoleObservation, ModelError> {
        if self.validate().has_errors()
            || !self.sources.iter().any(|item| &item.id == source)
            || source.files.iter().any(|f| f.sha256 == [0_u8; 32])
        {
            return Err(ModelError::InvalidProvenance);
        }

        // A SHA alone is not a file locator when two logical files have
        // identical bytes. Preserve that ambiguity instead of choosing one.
        let matching_files: Vec<_> = source
            .files
            .iter()
            .filter(|f| f.sha256 == hole.evidence.source_sha256)
            .collect();
        if matching_files.len() != 1 {
            return Err(ModelError::InvalidProvenance);
        }

        let mut copper: Vec<_> = self
            .stackup
            .layers
            .iter()
            .filter(|layer| layer.kind == LayerKind::Copper)
            .collect();
        copper.sort_by_key(|layer| layer.order);
        if !(2..=64).contains(&copper.len())
            || copper.iter().any(|layer| &layer.meta.id.source != source)
        {
            return Err(ModelError::InvalidStackup);
        }
        // Existing manufacturing validation checks dimensions, reversed or
        // out-of-bounds spans, source SHA presence, line and operation.
        hole.validate(copper.len() as u32)?;

        let start = (hole.span.first - 1) as usize;
        let end = hole.span.last as usize;
        let envelope = &copper[start..end];
        let stackup_evidence = if envelope.iter().any(|layer| {
            layer.material.is_none()
                || matches!(&layer.z_m, Property::Missing)
                || matches!(&layer.thickness_m, Property::Missing)
        }) {
            SourceStackupEvidence::Missing
        } else if envelope.iter().any(|layer| {
            matches!(&layer.z_m, Property::Assumed { .. })
                || matches!(&layer.thickness_m, Property::Assumed { .. })
        }) {
            SourceStackupEvidence::Assumed
        } else {
            SourceStackupEvidence::Explicit
        };
        let via_evidence = if !hole.evidence.qualified_native_grammar {
            SourceViaEvidenceState::UnsupportedGrammar
        } else {
            match hole.plating {
                FabricationPlating::Unknown => SourceViaEvidenceState::UnknownPlating,
                FabricationPlating::SourceDeclaredPlated => {
                    SourceViaEvidenceState::SourceDeclaredPlated
                }
                FabricationPlating::NonPlated => SourceViaEvidenceState::SourceDeclaredNonPlated,
            }
        };
        Ok(SourceViaHoleObservation {
            source_file: matching_files[0].logical_path.clone(),
            source_line: hole.evidence.source_line,
            first_layer: envelope[0].meta.id.clone(),
            last_layer: envelope[envelope.len() - 1].meta.id.clone(),
            physical_envelope_layers: envelope.iter().map(|layer| layer.meta.id.clone()).collect(),
            via_evidence,
            stackup_evidence,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        BoardKind, CapabilityState, FabricationHoleShape, FabricationOperation,
        FabricationSourceEvidence, GeometryProvenance, ImportCapability, ImportSource, Layer,
        ObjectKind, ObjectMeta, PhysicalLayerSpan, Rational, SourceFile, SourceFormat,
        SourceIdentity, Stackup,
    };
    use std::sync::Arc;

    fn fixture(copper_count: usize) -> (BoardModel, SourceId) {
        let source = Arc::new(SourceIdentity {
            format: SourceFormat::Synthetic,
            format_version: "e004-v1".into(),
            files: vec![SourceFile {
                logical_path: "fixture/board.asc".into(),
                sha256: [7; 32],
            }],
            importer_version: "e004-test".into(),
            conditions: vec![],
        });
        let mut board = BoardModel::empty(BoardKind::Pcb);
        board.sources.push(ImportSource {
            id: source.clone(),
            producer: "synthetic-fixture".into(),
            producer_version: "1".into(),
            capability: ImportCapability {
                stackup: CapabilityState::Missing,
                ..ImportCapability::default()
            },
        });
        board.stackup = Stackup {
            layers: (0..copper_count)
                .map(|index| Layer {
                    meta: ObjectMeta {
                        id: ObjectId::new(
                            &source,
                            ObjectKind::Layer,
                            &format!("L{}", index + 1),
                            "",
                        ),
                        provenance: GeometryProvenance {
                            source_object_type: "synthetic-copper-layer".into(),
                            history: vec![],
                        },
                    },
                    name: format!("L{}", index + 1),
                    order: (index + 1) as u32,
                    kind: LayerKind::Copper,
                    z_m: Property::Missing,
                    thickness_m: Property::Missing,
                    material: None,
                })
                .collect(),
        };
        (board, source)
    }

    fn hole(first: u32, last: u32) -> FabricationHole {
        FabricationHole {
            shape: FabricationHoleShape::Round {
                diameter_m: Rational::new(1, 5000).unwrap(),
            },
            plating: FabricationPlating::Unknown,
            span: PhysicalLayerSpan { first, last },
            operation: FabricationOperation::Drilled,
            evidence: FabricationSourceEvidence {
                format: "synthetic-fixture".into(),
                source_sha256: [7; 32],
                source_line: 12,
                qualified_native_grammar: true,
            },
        }
    }

    #[test]
    fn layer_count_and_arbitrary_partial_span_use_one_algorithm() {
        for count in [2_usize, 4, 6, 8, 64] {
            let (board, source) = fixture(count);
            let full = board
                .observe_source_via_hole(&source, &hole(1, count as u32))
                .unwrap();
            assert_eq!(full.physical_envelope_layers.len(), count);
            assert_eq!(full.first_layer, board.stackup.layers[0].meta.id);
            assert_eq!(full.last_layer, board.stackup.layers[count - 1].meta.id);
            assert_eq!(full.source_file, "fixture/board.asc");
            assert_eq!(full.source_line, 12);
            assert_eq!(full.stackup_evidence, SourceStackupEvidence::Missing);
            assert_eq!(full.via_evidence, SourceViaEvidenceState::UnknownPlating);
            if count >= 4 {
                let partial = board.observe_source_via_hole(&source, &hole(2, 3)).unwrap();
                assert_eq!(partial.physical_envelope_layers.len(), 2);
                assert_eq!(partial.first_layer, board.stackup.layers[1].meta.id);
                assert_eq!(partial.last_layer, board.stackup.layers[2].meta.id);
            }
            // Observations may not silently create geometry or graph edges.
            assert!(board.vias.is_empty());
            assert!(board.drills.is_empty());
            assert!(board.connectivity.edges.is_empty());
        }
    }

    #[test]
    fn unknown_plating_and_unsupported_grammar_never_become_connections() {
        let (board, source) = fixture(6);
        let mut h = hole(2, 4);
        let before = board.observe_source_via_hole(&source, &h).unwrap();
        assert_eq!(before.via_evidence, SourceViaEvidenceState::UnknownPlating);
        h.plating = FabricationPlating::SourceDeclaredPlated;
        let declared = board.observe_source_via_hole(&source, &h).unwrap();
        assert_eq!(
            declared.via_evidence,
            SourceViaEvidenceState::SourceDeclaredPlated
        );
        assert!(!h.has_verified_manufactured_barrel());
        assert!(!h.is_electrical_conductor());
        h.evidence.qualified_native_grammar = false;
        assert_eq!(
            board
                .observe_source_via_hole(&source, &h)
                .unwrap()
                .via_evidence,
            SourceViaEvidenceState::UnsupportedGrammar
        );
        h.evidence.qualified_native_grammar = true;
        h.plating = FabricationPlating::NonPlated;
        assert_eq!(
            board
                .observe_source_via_hole(&source, &h)
                .unwrap()
                .via_evidence,
            SourceViaEvidenceState::SourceDeclaredNonPlated
        );
        assert!(board.vias.is_empty());
        assert!(board.connectivity.edges.is_empty());
    }

    #[test]
    fn malformed_or_ambiguous_source_never_falls_back() {
        let (mut board, source) = fixture(8);
        let mut h = hole(1, 8);
        h.evidence.source_sha256 = [99; 32];
        assert_eq!(
            board.observe_source_via_hole(&source, &h),
            Err(ModelError::InvalidProvenance)
        );
        h = hole(1, 8);
        board.sources[0].id = Arc::new(SourceIdentity {
            files: vec![
                SourceFile {
                    logical_path: "a".into(),
                    sha256: [7; 32],
                },
                SourceFile {
                    logical_path: "b".into(),
                    sha256: [7; 32],
                },
            ],
            ..(*source).clone()
        });
        assert_eq!(
            board.observe_source_via_hole(&board.sources[0].id, &h),
            Err(ModelError::InvalidProvenance)
        );
        let (mut board, source) = fixture(8);
        board.stackup.layers[3].meta.id.source = Arc::new(SourceIdentity {
            importer_version: "unmapped".into(),
            ..(*source).clone()
        });
        assert!(board.observe_source_via_hole(&source, &h).is_err());
    }

    #[test]
    fn invalid_span_and_duplicate_layer_order_fail_closed() {
        let (mut board, source) = fixture(64);
        for span in [(0, 64), (4, 4), (9, 8), (1, 65)] {
            assert!(board
                .observe_source_via_hole(&source, &hole(span.0, span.1))
                .is_err());
        }
        board.stackup.layers[2].order = 2;
        assert!(board
            .observe_source_via_hole(&source, &hole(1, 64))
            .is_err());
    }

    #[test]
    fn envelope_is_not_a_connected_layer_set_even_for_same_net() {
        let (board, source) = fixture(6);
        let audit = board.observe_source_via_hole(&source, &hole(2, 5)).unwrap();
        assert_eq!(audit.physical_envelope_layers.len(), 4);
        assert!(board.connectivity.nodes.is_empty());
        assert!(board.connectivity.edges.is_empty());
    }
}
