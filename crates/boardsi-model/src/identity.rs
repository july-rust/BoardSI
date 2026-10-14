//! IDs use the complete identity tuple rather than a truncated hash.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SourceFormat {
    PadsAsc,
    Odb,
    Ipc2581,
    KiCad,
    Gerber,
    Synthetic,
    Other(String),
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceFile {
    /// Stable logical relative path, never an absolute workstation path.
    pub logical_path: String,
    pub sha256: [u8; 32],
}
/// Shared complete source identity; equality is structural, never pointer-based.
pub type SourceId = std::sync::Arc<SourceIdentity>;
/// The importer must supply verified file SHA-256 values. No file hashing/parser here.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceIdentity {
    pub format: SourceFormat,
    pub format_version: String,
    /// Canonically sorted by logical_path; checked at validation.
    pub files: Vec<SourceFile>,
    pub importer_version: String,
    /// Canonically ordered key/value import conditions (step, instance, encoding, etc.).
    pub conditions: Vec<(String, String)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportSource {
    pub id: SourceId,
    pub producer: String,
    pub producer_version: String,
    pub capability: ImportCapability,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectKind {
    Frame,
    Layer,
    Material,
    Net,
    Component,
    Pin,
    Trace,
    Arc,
    Area,
    Cutout,
    Padstack,
    Pad,
    Via,
    Drill,
    Outline,
    Node,
    Edge,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId {
    pub source: SourceId,
    pub kind: ObjectKind,
    pub source_object: String,
    /// Distinguishes generated primitives from one source object; deterministic, not a counter.
    pub part: String,
}
impl ObjectId {
    pub fn new(source: &SourceId, kind: ObjectKind, source_object: &str, part: &str) -> Self {
        Self {
            source: source.clone(),
            kind,
            source_object: source_object.into(),
            part: part.into(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConversionQuality {
    Exact,
    Approximate,
    Lossy,
    Pending,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ConversionStep {
    pub operation: String,
    pub quality: ConversionQuality,
    pub max_error_m: Option<crate::Rational>,
    pub note: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryProvenance {
    pub source_object_type: String,
    pub history: Vec<ConversionStep>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectMeta {
    pub id: ObjectId,
    pub provenance: GeometryProvenance,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapabilityState {
    Complete,
    Partial,
    Missing,
    Conflicting,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportCapability {
    pub geometry: CapabilityState,
    pub net: CapabilityState,
    pub pin_pad: CapabilityState,
    pub stackup: CapabilityState,
    pub reference: CapabilityState,
    pub model: CapabilityState,
    pub object_coverage: Vec<String>,
}
impl Default for ImportCapability {
    fn default() -> Self {
        Self {
            geometry: CapabilityState::Missing,
            net: CapabilityState::Missing,
            pin_pad: CapabilityState::Missing,
            stackup: CapabilityState::Missing,
            reference: CapabilityState::Missing,
            model: CapabilityState::Missing,
            object_coverage: Vec::new(),
        }
    }
}
