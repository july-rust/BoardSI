//! B_001: typed, explicit-assumption stackup overlay -> real Canonical BoardModel.
//! An overlay is not a PADS geometry import, netlist, return-path proof or SI solve.
use crate::*;

#[derive(Clone, Debug)]
pub struct StackupLayerEdit {
    pub source_layer_id: u32,
    pub kind: LayerKind,
    pub thickness_um: i64,
    pub material_name: String,
    pub dk: Option<f64>,
    pub df: Option<f64>,
    pub conductivity_s_per_m: Option<f64>,
    pub band_hz: Option<FrequencyRange>,
    pub evidence: PropertySource,
    pub role_hint: NetRole,
    pub reference_layer_id: Option<u32>,
}

#[derive(Debug)]
pub struct StackupConfiguration {
    pub model: BoardModel,
    pub total_thickness_m: Rational,
    pub reference_continuity: ReferenceContinuity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceContinuity {
    /// No imported area/net or topology proof was provided by this layer-only overlay.
    UnsupportedNoCopperNetEvidence,
}

fn provenance(id: ObjectId) -> ObjectMeta {
    ObjectMeta {
        id,
        provenance: GeometryProvenance {
            source_object_type: "user-stackup-overlay".into(),
            history: vec![ConversionStep {
                operation: "layer-index-bound user assumption, not a measured CAD import".into(),
                quality: ConversionQuality::Pending,
                max_error_m: None,
                note: "Source PCB copper geometry and return continuity unverified".into(),
            }],
        },
    }
}
fn assumed<T>(value: T, source: &PropertySource) -> Property<T> {
    Property::Assumed { value, source: source.clone() }
}
fn finite_positive(x: f64) -> bool { x.is_finite() && x > 0.0 }
fn valid_evidence(source: &PropertySource) -> bool {
    !source.document.trim().is_empty()
        && !source.revision.trim().is_empty()
        && !source.method.trim().is_empty()
        && source.reference_frequency_hz.map_or(false, finite_positive)
        && source.temperature_c.map_or(true, f64::is_finite)
}

/// Only the stackup/material portion is constructed. No synthetic net, copper, port,
/// via or plane is created and the resulting model is never solver-ready.
pub fn build_stackup_configuration(
    source: SourceId,
    edits: &[StackupLayerEdit],
    frequency_hz: f64,
) -> Result<StackupConfiguration, ModelError> {
    if edits.is_empty() || edits.len() > 64 || !frequency_hz.is_finite() || frequency_hz < 0.0 {
        return Err(ModelError::InvalidStackup);
    }
    let mut board = BoardModel::empty(BoardKind::Pcb);
    board.sources.push(ImportSource {
        id: source.clone(),
        producer: "boardsi-b001-user-overlay".into(),
        producer_version: "1".into(),
        capability: ImportCapability::default(), // No geometry/reference promotion.
    });
    let mut z_um = 0i64;
    for (index, edit) in edits.iter().enumerate() {
        if edit.source_layer_id != (index + 1) as u32
            || edit.thickness_um <= 0
            || edit.thickness_um > 1_000_000
            || edit.material_name.trim().is_empty()
            || !valid_evidence(&edit.evidence)
        {
            return Err(ModelError::InvalidStackup);
        }
        let is_copper = edit.kind == LayerKind::Copper;
        if edit.kind == LayerKind::Other || (!is_copper && edit.role_hint != NetRole::Unknown) {
            return Err(ModelError::InvalidMaterial);
        }
        if let Some(reference) = edit.reference_layer_id {
            let target = edits.get(reference.checked_sub(1).ok_or(ModelError::InvalidReference)? as usize)
                .ok_or(ModelError::InvalidReference)?;
            if !is_copper || target.kind != LayerKind::Copper
                || reference == edit.source_layer_id
            {
                return Err(ModelError::InvalidReference);
            }
        }
        let band = edit.band_hz.ok_or(ModelError::MissingRequired)?;
        if !band.min_hz.is_finite() || !band.max_hz.is_finite()
            || band.min_hz < 0.0 || band.max_hz < band.min_hz
            || frequency_hz < band.min_hz || frequency_hz > band.max_hz
        {
            return Err(ModelError::OutsideMaterialBand);
        }
        let reference_frequency = edit.evidence.reference_frequency_hz
            .ok_or(ModelError::MissingRequired)?;
        if reference_frequency < band.min_hz || reference_frequency > band.max_hz {
            return Err(ModelError::OutsideMaterialBand);
        }
        if is_copper {
            if !edit.conductivity_s_per_m.map_or(false, finite_positive)
                || edit.dk.is_some() || edit.df.is_some()
            {
                return Err(ModelError::InvalidMaterial);
            }
        } else if !edit.dk.map_or(false, finite_positive)
            || !edit.df.map_or(false, |v| v.is_finite() && v >= 0.0)
            || edit.conductivity_s_per_m.is_some()
        {
            return Err(ModelError::InvalidMaterial);
        }
        let layer_name = format!("PADS-layer-{}", edit.source_layer_id);
        let layer_id = ObjectId::new(&source, ObjectKind::Layer, &layer_name, "");
        let material_id = ObjectId::new(&source, ObjectKind::Material, &layer_name, "user");
        let material = Material {
            meta: provenance(material_id.clone()),
            name: edit.material_name.clone(),
            dk: edit.dk.map_or(Property::Missing, |v| assumed(v, &edit.evidence)),
            df: edit.df.map_or(Property::Missing, |v| assumed(v, &edit.evidence)),
            conductivity_s_per_m: edit.conductivity_s_per_m.map_or(
                Property::Missing, |v| assumed(v, &edit.evidence)
            ),
            valid_band: assumed(band, &edit.evidence),
        };
        board.materials.push(material);
        board.stackup.layers.push(Layer {
            meta: provenance(layer_id),
            name: layer_name,
            order: index as u32,
            kind: edit.kind,
            z_m: assumed(Rational::new(i128::from(z_um), 1_000_000)?, &edit.evidence),
            thickness_m: assumed(
                Rational::new(i128::from(edit.thickness_um), 1_000_000)?,
                &edit.evidence,
            ),
            material: Some(material_id),
        });
        z_um = z_um.checked_add(edit.thickness_um).ok_or(ModelError::CoordinateOverflow)?;
    }
    let report = board.validate_materials_at(frequency_hz);
    if report.has_errors() || report.has_missing() {
        return Err(ModelError::InvalidMaterial);
    }
    Ok(StackupConfiguration {
        model: board,
        total_thickness_m: Rational::new(i128::from(z_um), 1_000_000)?,
        reference_continuity: ReferenceContinuity::UnsupportedNoCopperNetEvidence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    fn fixture() -> (SourceId, Vec<StackupLayerEdit>) {
        let src = Arc::new(SourceIdentity {
            format: SourceFormat::Synthetic,
            format_version: "public-b001-v1".into(),
            files: vec![SourceFile { logical_path: "synthetic.asc".into(), sha256: [42;32] }],
            importer_version: "test".into(), conditions: vec![],
        });
        let source = PropertySource {
            document: "synthetic testcase only".into(), revision: "v1".into(),
            method: "manual demonstration".into(),
            reference_frequency_hz: Some(1e9), temperature_c: Some(25.0),
        };
        let copper = StackupLayerEdit {
            source_layer_id: 1, kind: LayerKind::Copper, thickness_um: 35,
            material_name: "test copper".into(), dk: None, df: None,
            conductivity_s_per_m: Some(5.8e7),
            band_hz: Some(FrequencyRange { min_hz: 1e6, max_hz: 2e9 }),
            evidence: source.clone(), role_hint: NetRole::Signal, reference_layer_id: Some(3),
        };
        let core = StackupLayerEdit {
            source_layer_id: 2, kind: LayerKind::Dielectric(DielectricKind::Core),
            thickness_um: 100, material_name: "test dielectric".into(),
            dk: Some(4.2), df: Some(0.02), conductivity_s_per_m: None,
            band_hz: copper.band_hz, evidence: source, role_hint: NetRole::Unknown,
            reference_layer_id: None,
        };
        let mut gnd = copper.clone();
        gnd.source_layer_id = 3;
        gnd.role_hint = NetRole::Ground;
        gnd.reference_layer_id = None;
        (src, vec![copper, core, gnd])
    }
    #[test]
    fn typed_assumed_stackup_is_contiguous_but_not_electrically_qualified() {
        let (src, edits) = fixture();
        let c = build_stackup_configuration(src, &edits, 1e9).unwrap();
        assert_eq!(c.model.stackup.layers.len(), 3);
        assert_eq!(c.model.materials.len(), 3);
        assert_eq!(c.total_thickness_m, Rational::new(170, 1_000_000).unwrap());
        assert_eq!(c.model.stackup.layers[1].z_m.value(), Some(&Rational::new(35,1_000_000).unwrap()));
        assert_eq!(c.reference_continuity, ReferenceContinuity::UnsupportedNoCopperNetEvidence);
        assert!(c.model.nets.is_empty() && c.model.areas.is_empty());
        assert!(!c.model.validate_materials_at(1e9).has_errors());
        assert!(matches!(c.model.materials[1].dk, Property::Assumed { .. }));
    }
    #[test]
    fn missing_bad_or_out_of_band_inputs_fail_closed() {
        let (src, mut e) = fixture();
        e[1].df = Some(-0.1);
        assert_eq!(build_stackup_configuration(src.clone(), &e, 1e9).unwrap_err(), ModelError::InvalidMaterial);
        e[1].df = Some(0.02);
        e[1].evidence.revision.clear();
        assert_eq!(build_stackup_configuration(src.clone(), &e, 1e9).unwrap_err(), ModelError::InvalidStackup);
        e[1].evidence.revision = "v1".into();
        assert_eq!(build_stackup_configuration(src.clone(), &e, 30e9).unwrap_err(), ModelError::OutsideMaterialBand);
        e[0].reference_layer_id = Some(2);
        assert_eq!(build_stackup_configuration(src.clone(), &e, 1e9).unwrap_err(), ModelError::InvalidReference);
        e[0].reference_layer_id = Some(3);
        e[1].thickness_um = 0;
        assert_eq!(build_stackup_configuration(src, &e, 1e9).unwrap_err(), ModelError::InvalidStackup);
    }
}
