//! Format-neutral, dependency-free M0 data contract. Not an importer or physical validator.
mod coordinates;
mod identity;
mod stackup_configuration;
mod validation;
pub use coordinates::*;
pub use identity::*;
pub use stackup_configuration::*;
pub use validation::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelError {
    InvalidRational,
    CoordinateOverflow,
    PrecisionLoss,
    UnsupportedTransform,
    InvalidDimension,
    InvalidReference,
    DuplicateId,
    InvalidIdentity,
    InvalidGeometry,
    InvalidStackup,
    InvalidMaterial,
    InvalidConnection,
    InvalidProvenance,
    MissingRequired,
    AssumedValue,
    OutsideMaterialBand,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PropertySource {
    pub document: String,
    pub revision: String,
    pub method: String,
    pub reference_frequency_hz: Option<f64>,
    pub temperature_c: Option<f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Property<T> {
    Missing,
    Assumed { value: T, source: PropertySource },
    Confirmed { value: T, source: PropertySource },
}
impl<T> Property<T> {
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Missing => None,
            Self::Assumed { value, .. } | Self::Confirmed { value, .. } => Some(value),
        }
    }
    pub fn source(&self) -> Option<&PropertySource> {
        match self {
            Self::Missing => None,
            Self::Assumed { source, .. } | Self::Confirmed { source, .. } => Some(source),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrequencyRange {
    pub min_hz: f64,
    pub max_hz: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub meta: ObjectMeta,
    pub name: String,
    pub dk: Property<f64>,
    pub df: Property<f64>,
    pub conductivity_s_per_m: Property<f64>,
    /// Constant property model only; valid band must be supplied for analysis readiness.
    pub valid_band: Property<FrequencyRange>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DielectricKind {
    Core,
    Prepreg,
    Other,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerKind {
    Copper,
    Dielectric(DielectricKind),
    Mask,
    Other,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Layer {
    pub meta: ObjectMeta,
    pub name: String,
    pub order: u32,
    pub kind: LayerKind,
    /// Board z increases top-to-bottom; z is the upper face, not the centre.
    pub z_m: Property<Rational>,
    pub thickness_m: Property<Rational>,
    pub material: Option<ObjectId>,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stackup {
    pub layers: Vec<Layer>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetRole {
    Signal,
    Ground,
    Power,
    Floating,
    Unknown,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Net {
    pub meta: ObjectMeta,
    pub name: String,
    pub role: NetRole,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardKind {
    Pcb,
    BgaSubstrate,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    pub position: Point,
    pub orientation: Orientation,
    pub bottom_side: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Component {
    pub meta: ObjectMeta,
    pub reference: String,
    pub placement: Placement,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Pin {
    pub meta: ObjectMeta,
    pub component: ObjectId,
    pub number: String,
    pub pad: ObjectId,
    pub net: Option<ObjectId>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Trace {
    pub meta: ObjectMeta,
    pub layer: ObjectId,
    pub net: Option<ObjectId>,
    pub centreline: Vec<Point>,
    pub width_m: Rational,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Arc {
    pub meta: ObjectMeta,
    pub layer: ObjectId,
    pub net: Option<ObjectId>,
    pub centre: Point,
    pub start: Point,
    pub end: Point,
    pub clockwise: bool,
    pub width_m: Rational,
}
/// Curved edges are retained analytically, never tessellated for solver storage.
#[derive(Clone, Debug, PartialEq)]
pub enum BoundarySegment {
    Line {
        start: Point,
        end: Point,
    },
    Arc {
        centre: Point,
        start: Point,
        end: Point,
        clockwise: bool,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct Boundary {
    pub segments: Vec<BoundarySegment>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Cutout {
    pub meta: ObjectMeta,
    pub boundary: Boundary,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CopperArea {
    pub meta: ObjectMeta,
    pub layer: ObjectId,
    pub net: Option<ObjectId>,
    pub outer: Boundary,
    pub cutouts: Vec<Cutout>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum PadShape {
    Circle {
        diameter_m: Rational,
    },
    Rectangle {
        size_m: [Rational; 2],
    },
    /// Reusable pad-local geometry; its frame maps source ticks to local metres.
    /// Apply Pad position/orientation once when instantiating it in board space.
    Custom(Boundary),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadRole {
    Conductive,
    AntiPad,
    Clearance,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PadstackLayer {
    pub layer: ObjectId,
    pub role: PadRole,
    pub shape: PadShape,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Padstack {
    pub meta: ObjectMeta,
    pub entries: Vec<PadstackLayer>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Pad {
    pub meta: ObjectMeta,
    pub position: Point,
    pub orientation: Orientation,
    pub padstack: ObjectId,
    pub net: Option<ObjectId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViaSpan {
    pub first_layer: ObjectId,
    pub last_layer: ObjectId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViaKind {
    Through,
    Blind,
    Buried,
    Microvia,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Drill {
    pub meta: ObjectMeta,
    pub diameter_m: Rational,
    pub plated: bool,
    pub span: ViaSpan,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Via {
    pub meta: ObjectMeta,
    pub position: Point,
    pub padstack: ObjectId,
    pub drill: ObjectId,
    pub span: ViaSpan,
    pub kind: ViaKind,
    pub net: Option<ObjectId>,
    /// Actual connected copper layers, not inferred from the span or same net names.
    pub connected_layers: Vec<ObjectId>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Outline {
    pub meta: ObjectMeta,
    pub outer: Boundary,
    pub cutouts: Vec<Cutout>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ContactSite {
    TraceVertex(usize),
    ArcStart,
    ArcEnd,
    PadCentre,
    ViaCentre,
    /// Geometry engine assertion only: containment/clearance proof is NOT performed here.
    AreaWitness {
        point: Point,
        evidence: String,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectivityNode {
    pub meta: ObjectMeta,
    pub conductor: ObjectId,
    pub layer: ObjectId,
    pub site: ContactSite,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionEvidence {
    /// Same layer and exact position plus importer/topology evidence required.
    SameLayerContact { evidence: String },
    /// Only one plated via's explicitly connected conductive layers may be joined.
    ViaBarrel { via: ObjectId },
}
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectivityEdge {
    pub meta: ObjectMeta,
    pub a: ObjectId,
    pub b: ObjectId,
    pub evidence: ConnectionEvidence,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConnectivityGraph {
    pub nodes: Vec<ConnectivityNode>,
    pub edges: Vec<ConnectivityEdge>,
}
#[derive(Debug, PartialEq)]
pub struct BoardModel {
    pub kind: BoardKind,
    pub sources: Vec<ImportSource>,
    pub frames: Vec<CoordinateFrame>,
    pub stackup: Stackup,
    pub materials: Vec<Material>,
    pub nets: Vec<Net>,
    pub components: Vec<Component>,
    pub pins: Vec<Pin>,
    pub traces: Vec<Trace>,
    pub arcs: Vec<Arc>,
    pub areas: Vec<CopperArea>,
    pub padstacks: Vec<Padstack>,
    pub pads: Vec<Pad>,
    pub drills: Vec<Drill>,
    pub vias: Vec<Via>,
    pub outlines: Vec<Outline>,
    pub connectivity: ConnectivityGraph,
}
impl BoardModel {
    pub fn empty(kind: BoardKind) -> Self {
        Self {
            kind,
            sources: Vec::new(),
            frames: Vec::new(),
            stackup: Stackup::default(),
            materials: Vec::new(),
            nets: Vec::new(),
            components: Vec::new(),
            pins: Vec::new(),
            traces: Vec::new(),
            arcs: Vec::new(),
            areas: Vec::new(),
            padstacks: Vec::new(),
            pads: Vec::new(),
            drills: Vec::new(),
            vias: Vec::new(),
            outlines: Vec::new(),
            connectivity: ConnectivityGraph::default(),
        }
    }
}
