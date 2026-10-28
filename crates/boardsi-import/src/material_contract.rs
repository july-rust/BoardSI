//! C_004: source-bounded Canonical Material/Layer INPUT contract, not a loss solver.
//! The import/source identity is a caller claim here. No physical or cache authorization.
use crate::digest::sha256;
use boardsi_model::{FrequencyRange, Layer, LayerKind, Material, Property, PropertySource};

const MATERIAL_DOMAIN: &[u8] = b"BoardSI C004 canonical material input v1\0";
const AUDIT_DOMAIN: &[u8] = b"BoardSI C004 frequency audit v1\0";
const MAX_FIELD_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialAuditError {
    MissingRequiredData,
    InvalidMaterial,
    OutsideValidBand,
    UnsupportedLossModel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeConvention {
    ExpPlusJ,
    ExpMinusJ,
    Unspecified,
}

/// Signed relative complex permittivity: imag follows its *source* time convention.
/// Conductivity already folded into epsilon'' cannot also be supplied separately.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RelativePermittivity {
    pub real: f64,
    pub signed_imag: f64,
    pub source_convention: TimeConvention,
    pub includes_bulk_conduction: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoughnessMetric {
    Rq,
    Ra,
    Rz,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CopperSurface {
    Unknown,
    ExplicitSmooth {
        evidence: PropertySource,
    },
    /// Measurement is recorded but cannot enable an unqualified Huray/Hammerstad
    /// correction. RMS, Ra and Rz are not interchangeable.
    Measured {
        metric: RoughnessMetric,
        height_m: f64,
        evidence: PropertySource,
        band_hz: FrequencyRange,
        model_id: String,
        model_revision: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct MaterialAudit {
    /// This means valid *assumed* inputs, not a measured or qualified material.
    pub material_revision_sha256: String,
    /// Recomputed when analysis frequency changes.
    pub audit_revision_sha256: String,
    pub resistivity_ohm_m: Option<f64>,
    pub normalized_relative_epsilon_imag: Option<f64>,
    pub roughness_qualified_for_loss: bool,
    pub solver_ready: bool,
    pub cache_reuse_eligible: bool,
}

fn valid_text(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= MAX_FIELD_BYTES && !value.contains('\0')
}

fn valid_positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn valid_band(band: FrequencyRange, hz: f64) -> bool {
    band.min_hz.is_finite()
        && band.max_hz.is_finite()
        && band.min_hz >= 0.0
        && band.max_hz >= band.min_hz
        && hz >= band.min_hz
        && hz <= band.max_hz
}

fn validate_evidence(
    source: &PropertySource,
    band: FrequencyRange,
) -> Result<(), MaterialAuditError> {
    if !valid_text(&source.document) || !valid_text(&source.revision) || !valid_text(&source.method)
    {
        return Err(MaterialAuditError::MissingRequiredData);
    }
    let reference = source
        .reference_frequency_hz
        .ok_or(MaterialAuditError::MissingRequiredData)?;
    if !valid_positive(reference) || !valid_band(band, reference) {
        return Err(MaterialAuditError::OutsideValidBand);
    }
    if source.temperature_c.is_some_and(|t| !t.is_finite()) {
        return Err(MaterialAuditError::InvalidMaterial);
    }
    Ok(())
}

fn assumed<T>(prop: &Property<T>) -> Result<(&T, &PropertySource), MaterialAuditError> {
    match prop {
        Property::Assumed { value, source } => Ok((value, source)),
        Property::Missing => Err(MaterialAuditError::MissingRequiredData),
        // A Canonical "Confirmed" claim cannot be authenticated by this narrow auditor.
        Property::Confirmed { .. } => Err(MaterialAuditError::UnsupportedLossModel),
    }
}

fn empty<T>(prop: &Property<T>) -> bool {
    matches!(prop, Property::Missing)
}

fn add_text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn add_number(bytes: &mut Vec<u8>, value: f64) {
    let normalized = if value == 0.0 { 0.0 } else { value };
    bytes.extend_from_slice(&normalized.to_bits().to_be_bytes());
}

fn add_source(bytes: &mut Vec<u8>, source: &PropertySource) {
    add_text(bytes, &source.document);
    add_text(bytes, &source.revision);
    add_text(bytes, &source.method);
    add_number(
        bytes,
        source.reference_frequency_hz.expect("validated source"),
    );
    match source.temperature_c {
        Some(t) => {
            bytes.push(1);
            add_number(bytes, t);
        }
        None => bytes.push(0),
    }
}

fn hash(domain: &[u8], payload: &[u8]) -> String {
    let mut data = Vec::with_capacity(domain.len() + payload.len());
    data.extend_from_slice(domain);
    data.extend_from_slice(payload);
    let raw = sha256(&data);
    let mut hex = String::with_capacity(64);
    for byte in raw {
        use std::fmt::Write;
        write!(&mut hex, "{byte:02x}").expect("writing to String");
    }
    hex
}

/// Checks actual Canonical Layer + Material properties (not unbound user values).
/// Does not independently authenticate PADS bytes, material measurements or models.
pub fn audit_material(
    layer: &Layer,
    material: &Material,
    frequency_hz: f64,
    surface: &CopperSurface,
    complex: Option<RelativePermittivity>,
    explicit_lossless_dielectric: bool,
) -> Result<MaterialAudit, MaterialAuditError> {
    if !frequency_hz.is_finite() || frequency_hz < 0.0 {
        return Err(MaterialAuditError::InvalidMaterial);
    }
    if layer.material.as_ref() != Some(&material.meta.id)
        || layer.meta.id.source != material.meta.id.source
        || !valid_text(&material.name)
    {
        return Err(MaterialAuditError::InvalidMaterial);
    }
    let layer_identity = format!("{:?}", layer.meta.id);
    let material_identity = format!("{:?}", material.meta.id);
    if layer_identity.len() > 4096 || material_identity.len() > 4096 {
        return Err(MaterialAuditError::InvalidMaterial);
    }
    let (band, band_source) = assumed(&material.valid_band)?;
    if !band.min_hz.is_finite()
        || !band.max_hz.is_finite()
        || band.min_hz < 0.0
        || band.max_hz < band.min_hz
    {
        return Err(MaterialAuditError::InvalidMaterial);
    }
    if !valid_band(*band, frequency_hz) {
        return Err(MaterialAuditError::OutsideValidBand);
    }
    validate_evidence(band_source, *band)?;
    let mut bytes = Vec::new();
    add_text(&mut bytes, "boardsi-c004-material-input-v1");
    add_text(&mut bytes, &layer_identity);
    add_text(&mut bytes, &material_identity);
    add_text(&mut bytes, &material.name);
    add_number(&mut bytes, band.min_hz);
    add_number(&mut bytes, band.max_hz);
    add_source(&mut bytes, band_source);

    let mut resistivity = None;
    let mut epsilon_imag = None;
    match layer.kind {
        LayerKind::Copper => {
            if !empty(&material.dk)
                || !empty(&material.df)
                || complex.is_some()
                || explicit_lossless_dielectric
            {
                return Err(MaterialAuditError::InvalidMaterial);
            }
            let (sigma, evidence) = assumed(&material.conductivity_s_per_m)?;
            if !valid_positive(*sigma) {
                return Err(MaterialAuditError::InvalidMaterial);
            }
            validate_evidence(evidence, *band)?;
            bytes.push(1);
            add_number(&mut bytes, *sigma);
            add_source(&mut bytes, evidence);
            resistivity = Some(1.0 / *sigma);
            if !resistivity.is_some_and(valid_positive) {
                return Err(MaterialAuditError::InvalidMaterial);
            }
            match surface {
                CopperSurface::Unknown => bytes.push(0),
                CopperSurface::ExplicitSmooth { evidence } => {
                    validate_evidence(evidence, *band)?;
                    bytes.push(1);
                    add_source(&mut bytes, evidence);
                }
                CopperSurface::Measured {
                    metric,
                    height_m,
                    evidence,
                    band_hz,
                    model_id,
                    model_revision,
                } => {
                    if !height_m.is_finite()
                        || *height_m < 0.0
                        || !valid_text(model_id)
                        || !valid_text(model_revision)
                    {
                        return Err(MaterialAuditError::InvalidMaterial);
                    }
                    if !valid_band(*band_hz, frequency_hz) {
                        return Err(MaterialAuditError::OutsideValidBand);
                    }
                    validate_evidence(evidence, *band_hz)?;
                    bytes.push(2);
                    bytes.push(match metric {
                        RoughnessMetric::Rq => 1,
                        RoughnessMetric::Ra => 2,
                        RoughnessMetric::Rz => 3,
                    });
                    add_number(&mut bytes, *height_m);
                    add_number(&mut bytes, band_hz.min_hz);
                    add_number(&mut bytes, band_hz.max_hz);
                    add_source(&mut bytes, evidence);
                    add_text(&mut bytes, model_id);
                    add_text(&mut bytes, model_revision);
                    // All correction models remain unqualified: metadata accepted only.
                }
            }
        }
        LayerKind::Dielectric(_) | LayerKind::Mask => {
            if !empty(&material.conductivity_s_per_m) || !matches!(surface, CopperSurface::Unknown)
            {
                return Err(MaterialAuditError::InvalidMaterial);
            }
            let (dk, dk_source) = assumed(&material.dk)?;
            let (df, df_source) = assumed(&material.df)?;
            if !valid_positive(*dk)
                || !df.is_finite()
                || *df < 0.0
                || (*df == 0.0 && !explicit_lossless_dielectric)
            {
                return Err(MaterialAuditError::InvalidMaterial);
            }
            validate_evidence(dk_source, *band)?;
            validate_evidence(df_source, *band)?;
            bytes.push(2);
            add_number(&mut bytes, *dk);
            add_number(&mut bytes, *df);
            add_source(&mut bytes, dk_source);
            add_source(&mut bytes, df_source);
            bytes.push(u8::from(explicit_lossless_dielectric));
            if let Some(value) = complex {
                if !valid_positive(value.real)
                    || !value.signed_imag.is_finite()
                    || value.includes_bulk_conduction
                {
                    return Err(MaterialAuditError::InvalidMaterial);
                }
                let normalized = match value.source_convention {
                    TimeConvention::ExpPlusJ if value.signed_imag <= 0.0 => -value.signed_imag,
                    TimeConvention::ExpMinusJ if value.signed_imag >= 0.0 => value.signed_imag,
                    _ => return Err(MaterialAuditError::UnsupportedLossModel),
                };
                // Comparison is only valid when the supplied values represent the SAME frequency.
                // Exact bit matching is not appropriate for measured decimal datasets.
                let expected_imag = *dk * *df;
                if !expected_imag.is_finite() {
                    return Err(MaterialAuditError::InvalidMaterial);
                }
                let tolerance = 1e-10_f64.max(1e-8 * dk.abs().max(expected_imag.abs()));
                if (value.real - dk).abs() > tolerance || (normalized - expected_imag).abs() > tolerance {
                    return Err(MaterialAuditError::InvalidMaterial);
                }
                bytes.push(1);
                add_number(&mut bytes, value.real);
                add_number(&mut bytes, value.signed_imag);
                bytes.push(match value.source_convention {
                    TimeConvention::ExpPlusJ => 1,
                    TimeConvention::ExpMinusJ => 2,
                    TimeConvention::Unspecified => 0,
                });
                epsilon_imag = Some(normalized);
            } else {
                bytes.push(0);
            }
        }
        LayerKind::Other => return Err(MaterialAuditError::InvalidMaterial),
    }
    let material_revision_sha256 = hash(MATERIAL_DOMAIN, &bytes);
    add_number(&mut bytes, frequency_hz);
    let audit_revision_sha256 = hash(AUDIT_DOMAIN, &bytes);
    Ok(MaterialAudit {
        material_revision_sha256,
        audit_revision_sha256,
        resistivity_ohm_m: resistivity,
        normalized_relative_epsilon_imag: epsilon_imag,
        roughness_qualified_for_loss: false,
        solver_ready: false,
        cache_reuse_eligible: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use boardsi_model::{
        ConversionQuality, ConversionStep, DielectricKind, GeometryProvenance, ObjectId,
        ObjectKind, ObjectMeta, SourceFormat, SourceIdentity,
    };
    use std::sync::Arc;

    fn source() -> PropertySource {
        PropertySource {
            document: "measured material fixture".into(),
            revision: "r1".into(),
            method: "test-method".into(),
            reference_frequency_hz: Some(1e9),
            temperature_c: Some(23.0),
        }
    }

    fn property(x: f64) -> Property<f64> {
        Property::Assumed {
            value: x,
            source: source(),
        }
    }

    fn fixture(copper: bool) -> (Layer, Material) {
        let source_id = Arc::new(SourceIdentity {
            format: SourceFormat::Synthetic,
            format_version: "1".into(),
            files: vec![],
            importer_version: "test".into(),
            conditions: vec![],
        });
        let layer_id = ObjectId::new(&source_id, ObjectKind::Layer, "L1", "");
        let material_id = ObjectId::new(&source_id, ObjectKind::Material, "M1", "");
        let meta = |id: ObjectId| ObjectMeta {
            id,
            provenance: GeometryProvenance {
                source_object_type: "synthetic".into(),
                history: vec![ConversionStep {
                    operation: "fixture".into(),
                    quality: ConversionQuality::Pending,
                    max_error_m: None,
                    note: "unqualified".into(),
                }],
            },
        };
        let band = FrequencyRange {
            min_hz: 1e6,
            max_hz: 10e9,
        };
        let material = Material {
            meta: meta(material_id.clone()),
            name: if copper { "Cu" } else { "dielectric" }.into(),
            dk: if copper {
                Property::Missing
            } else {
                property(4.0)
            },
            df: if copper {
                Property::Missing
            } else {
                property(0.02)
            },
            conductivity_s_per_m: if copper {
                property(5.8e7)
            } else {
                Property::Missing
            },
            valid_band: Property::Assumed {
                value: band,
                source: source(),
            },
        };
        let layer = Layer {
            meta: meta(layer_id),
            name: "L1".into(),
            order: 0,
            kind: if copper {
                LayerKind::Copper
            } else {
                LayerKind::Dielectric(DielectricKind::Core)
            },
            z_m: Property::Missing,
            thickness_m: Property::Missing,
            material: Some(material_id),
        };
        (layer, material)
    }

    fn run(
        layer: &Layer,
        material: &Material,
        freq: f64,
    ) -> Result<MaterialAudit, MaterialAuditError> {
        audit_material(layer, material, freq, &CopperSurface::Unknown, None, false)
    }

    #[test]
    fn c004_canonical_dielectric_and_frequency_bounds() {
        let (layer, material) = fixture(false);
        let good = run(&layer, &material, 1e9).unwrap();
        assert_eq!(good.resistivity_ohm_m, None);
        assert!(
            !good.solver_ready && !good.cache_reuse_eligible && !good.roughness_qualified_for_loss
        );
        assert!(run(&layer, &material, 1e6).is_ok());
        assert!(run(&layer, &material, 10e9).is_ok());
        assert_eq!(
            run(&layer, &material, 1e3),
            Err(MaterialAuditError::OutsideValidBand)
        );
        assert_eq!(
            run(&layer, &material, 11e9),
            Err(MaterialAuditError::OutsideValidBand)
        );
    }

    #[test]
    fn c004_copper_resistivity_and_explicit_smooth_assumption() {
        let (layer, material) = fixture(true);
        let audit = run(&layer, &material, 1e9).unwrap();
        let rho = audit.resistivity_ohm_m.unwrap();
        assert!((rho * 5.8e7 - 1.0).abs() < 1e-12);
        let smooth = audit_material(
            &layer,
            &material,
            1e9,
            &CopperSurface::ExplicitSmooth { evidence: source() },
            None,
            false,
        )
        .unwrap();
        assert_ne!(
            audit.material_revision_sha256,
            smooth.material_revision_sha256
        );
        assert!(!smooth.roughness_qualified_for_loss);
    }

    #[test]
    fn c004_complex_sign_normalization_and_rejection() {
        let (layer, material) = fixture(false);
        let minus = RelativePermittivity {
            real: 4.0,
            signed_imag: 0.08,
            source_convention: TimeConvention::ExpMinusJ,
            includes_bulk_conduction: false,
        };
        let plus = RelativePermittivity {
            signed_imag: -0.08,
            source_convention: TimeConvention::ExpPlusJ,
            ..minus
        };
        let r1 = audit_material(
            &layer,
            &material,
            1e9,
            &CopperSurface::Unknown,
            Some(minus),
            false,
        )
        .unwrap();
        let r2 = audit_material(
            &layer,
            &material,
            1e9,
            &CopperSurface::Unknown,
            Some(plus),
            false,
        )
        .unwrap();
        assert_eq!(r1.normalized_relative_epsilon_imag, Some(0.08));
        assert_eq!(r2.normalized_relative_epsilon_imag, Some(0.08));
        assert_ne!(r1.material_revision_sha256, r2.material_revision_sha256);
        for bad in [
            RelativePermittivity {
                source_convention: TimeConvention::Unspecified,
                ..minus
            },
            RelativePermittivity {
                signed_imag: -0.08,
                ..minus
            },
        ] {
            assert_eq!(
                audit_material(
                    &layer,
                    &material,
                    1e9,
                    &CopperSurface::Unknown,
                    Some(bad),
                    false
                ),
                Err(MaterialAuditError::UnsupportedLossModel)
            );
        }
        for bad in [
            RelativePermittivity {
                includes_bulk_conduction: true,
                ..minus
            },
            RelativePermittivity { real: 4.1, ..minus },
            RelativePermittivity {
                signed_imag: 0.09,
                ..minus
            },
        ] {
            assert_eq!(
                audit_material(
                    &layer,
                    &material,
                    1e9,
                    &CopperSurface::Unknown,
                    Some(bad),
                    false
                ),
                Err(MaterialAuditError::InvalidMaterial)
            );
        }
    }

    #[test]
    fn c004_reject_invalid_property_values() {
        let (layer, mut mat) = fixture(false);
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            mat.dk = property(bad);
            assert_eq!(
                run(&layer, &mat, 1e9),
                Err(MaterialAuditError::InvalidMaterial)
            );
        }
        mat.dk = property(4.0);
        for bad in [-0.01, f64::NAN, f64::INFINITY] {
            mat.df = property(bad);
            assert_eq!(
                run(&layer, &mat, 1e9),
                Err(MaterialAuditError::InvalidMaterial)
            );
        }
        mat.df = property(0.0);
        assert_eq!(
            run(&layer, &mat, 1e9),
            Err(MaterialAuditError::InvalidMaterial)
        );
        assert!(audit_material(&layer, &mat, 1e9, &CopperSurface::Unknown, None, true).is_ok());
        let (cl, mut cm) = fixture(true);
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            cm.conductivity_s_per_m = property(bad);
            assert_eq!(run(&cl, &cm, 1e9), Err(MaterialAuditError::InvalidMaterial));
        }
    }

    #[test]
    fn c004_fail_closed_metadata_and_canonical_link() {
        let (mut layer, mut material) = fixture(false);
        material.df = Property::Missing;
        assert_eq!(
            run(&layer, &material, 1e9),
            Err(MaterialAuditError::MissingRequiredData)
        );
        material.df = property(0.02);
        if let Property::Assumed { source, .. } = &mut material.df {
            source.document = " ".into();
        }
        assert_eq!(
            run(&layer, &material, 1e9),
            Err(MaterialAuditError::MissingRequiredData)
        );
        material.df = property(0.02);
        if let Property::Assumed { source, .. } = &mut material.df {
            source.reference_frequency_hz = Some(11e9);
        }
        assert_eq!(
            run(&layer, &material, 1e9),
            Err(MaterialAuditError::OutsideValidBand)
        );
        material.df = property(0.02);
        material.df = Property::Confirmed {
            value: 0.02,
            source: source(),
        };
        assert_eq!(
            run(&layer, &material, 1e9),
            Err(MaterialAuditError::UnsupportedLossModel)
        );
        material.df = property(0.02);
        layer.material = None;
        assert_eq!(
            run(&layer, &material, 1e9),
            Err(MaterialAuditError::InvalidMaterial)
        );
    }

    #[test]
    fn c004_roughness_provenance_and_range() {
        let (layer, material) = fixture(true);
        let surf = CopperSurface::Measured {
            metric: RoughnessMetric::Rq,
            height_m: 1e-6,
            evidence: source(),
            band_hz: FrequencyRange {
                min_hz: 1e7,
                max_hz: 10e9,
            },
            model_id: "model declaration only".into(),
            model_revision: "unqualified".into(),
        };
        let audit = audit_material(&layer, &material, 1e9, &surf, None, false).unwrap();
        assert!(!audit.roughness_qualified_for_loss && !audit.solver_ready);
        let mut alt = surf.clone();
        if let CopperSurface::Measured { metric, .. } = &mut alt {
            *metric = RoughnessMetric::Ra;
        }
        let alt_audit = audit_material(&layer, &material, 1e9, &alt, None, false).unwrap();
        assert_ne!(
            audit.material_revision_sha256,
            alt_audit.material_revision_sha256
        );
        assert_eq!(
            audit_material(&layer, &material, 1e6, &surf, None, false),
            Err(MaterialAuditError::OutsideValidBand)
        );
        let mut bad = surf.clone();
        if let CopperSurface::Measured { height_m, .. } = &mut bad {
            *height_m = f64::NAN;
        }
        assert_eq!(
            audit_material(&layer, &material, 1e9, &bad, None, false),
            Err(MaterialAuditError::InvalidMaterial)
        );
    }

    #[test]
    fn c004_revision_is_deterministic_and_frequency_bound() {
        let (layer, mut material) = fixture(false);
        let initial = run(&layer, &material, 1e9).unwrap();
        assert_eq!(initial, run(&layer, &material, 1e9).unwrap());
        let another_frequency = run(&layer, &material, 2e9).unwrap();
        assert_eq!(
            initial.material_revision_sha256,
            another_frequency.material_revision_sha256
        );
        assert_ne!(
            initial.audit_revision_sha256,
            another_frequency.audit_revision_sha256
        );
        material.df = property(0.03);
        let edit = run(&layer, &material, 1e9).unwrap();
        assert_ne!(
            initial.material_revision_sha256,
            edit.material_revision_sha256
        );
        material.df = property(0.02);
        assert_eq!(initial, run(&layer, &material, 1e9).unwrap());
        if let Property::Assumed { source, .. } = &mut material.df {
            source.revision = "r2".into();
        }
        assert_ne!(
            initial.material_revision_sha256,
            run(&layer, &material, 1e9)
                .unwrap()
                .material_revision_sha256
        );
    }

    #[test]
    fn c004_prevent_copper_dielectric_confusion() {
        let (layer, mut material) = fixture(true);
        material.dk = property(4.0);
        assert_eq!(
            run(&layer, &material, 1e9),
            Err(MaterialAuditError::InvalidMaterial)
        );
        let (layer, mut material) = fixture(false);
        material.conductivity_s_per_m = property(1.0);
        assert_eq!(
            run(&layer, &material, 1e9),
            Err(MaterialAuditError::InvalidMaterial)
        );
    }
}
