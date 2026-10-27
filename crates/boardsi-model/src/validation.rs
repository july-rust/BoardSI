use crate::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Missing,
    Assumption,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub error: ModelError,
    pub object: Option<ObjectId>,
    pub context: String,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
}
impl ValidationReport {
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|x| x.severity == Severity::Error)
    }
    pub fn has_missing(&self) -> bool {
        self.issues.iter().any(|x| x.severity == Severity::Missing)
    }
    pub fn contains(&self, error: ModelError) -> bool {
        self.issues.iter().any(|x| x.error == error)
    }
    fn add(
        &mut self,
        severity: Severity,
        error: ModelError,
        object: Option<&ObjectId>,
        context: &str,
    ) {
        self.issues.push(ValidationIssue {
            severity,
            error,
            object: object.cloned(),
            context: context.into(),
        });
    }
    fn check(&mut self, result: Result<(), ModelError>, id: &ObjectId, context: &str) {
        if let Err(error) = result {
            self.add(Severity::Error, error, Some(id), context);
        }
    }
}
fn require(condition: bool, error: ModelError) -> Result<(), ModelError> {
    if condition {
        Ok(())
    } else {
        Err(error)
    }
}
fn orientation(value: &Orientation) -> Result<(), ModelError> {
    match value {
        Orientation::Orthogonal { .. } => Ok(()),
        Orientation::PendingRotation { radians, .. } if !radians.is_finite() => {
            Err(ModelError::InvalidGeometry)
        }
        Orientation::PendingRotation { .. } => Err(ModelError::UnsupportedTransform),
    }
}
fn property<T>(value: &Property<T>, id: &ObjectId, label: &str, report: &mut ValidationReport) {
    match value {
        Property::Missing => report.add(
            Severity::Missing,
            ModelError::MissingRequired,
            Some(id),
            label,
        ),
        Property::Assumed { .. } => report.add(
            Severity::Assumption,
            ModelError::AssumedValue,
            Some(id),
            label,
        ),
        Property::Confirmed { .. } => {}
    }
    if let Some(s) = value.source() {
        report.check(
            require(
                !s.document.trim().is_empty()
                    && s.reference_frequency_hz
                        .map_or(true, |v| v.is_finite() && v >= 0.0)
                    && s.temperature_c.map_or(true, f64::is_finite),
                ModelError::InvalidProvenance,
            ),
            id,
            label,
        );
    }
}
// Built once per board validation. A contact graph must not turn every
// edge into a linear scan over all imported conductors and contact nodes.
struct ContactValidationIndex<'a> {
    nodes: BTreeMap<&'a ObjectId, &'a ConnectivityNode>,
    traces: BTreeMap<&'a ObjectId, &'a Trace>,
    arcs: BTreeMap<&'a ObjectId, &'a Arc>,
    pads: BTreeMap<&'a ObjectId, &'a Pad>,
    vias: BTreeMap<&'a ObjectId, &'a Via>,
    areas: BTreeMap<&'a ObjectId, &'a CopperArea>,
    nets: BTreeSet<&'a ObjectId>,
    conductive: BTreeSet<(&'a ObjectId, &'a ObjectId)>,
}

impl<'a> ContactValidationIndex<'a> {
    fn new(board: &'a BoardModel) -> Self {
        Self {
            nodes: board
                .connectivity
                .nodes
                .iter()
                .map(|x| (&x.meta.id, x))
                .collect(),
            traces: board.traces.iter().map(|x| (&x.meta.id, x)).collect(),
            arcs: board.arcs.iter().map(|x| (&x.meta.id, x)).collect(),
            pads: board.pads.iter().map(|x| (&x.meta.id, x)).collect(),
            vias: board.vias.iter().map(|x| (&x.meta.id, x)).collect(),
            areas: board.areas.iter().map(|x| (&x.meta.id, x)).collect(),
            nets: board.nets.iter().map(|x| &x.meta.id).collect(),
            conductive: board
                .padstacks
                .iter()
                .flat_map(|p| {
                    p.entries.iter().filter_map(move |entry| {
                        (entry.role == PadRole::Conductive).then_some((&p.meta.id, &entry.layer))
                    })
                })
                .collect(),
        }
    }
    fn known_net(&self, net: &Option<ObjectId>) -> Result<(), ModelError> {
        require(
            net.as_ref().map_or(true, |id| self.nets.contains(id)),
            ModelError::InvalidReference,
        )
    }
    fn conductive(&self, stack: &ObjectId, layer: &ObjectId) -> Result<(), ModelError> {
        require(
            self.conductive.contains(&(stack, layer)),
            ModelError::InvalidConnection,
        )
    }
}

impl BoardModel {
    fn metas(&self) -> Vec<(&ObjectMeta, ObjectKind)> {
        let mut result = Vec::new();
        macro_rules! append {
            ($objects:expr, $kind:ident) => {
                result.extend($objects.iter().map(|x| (&x.meta, ObjectKind::$kind)));
            };
        }
        append!(self.frames, Frame);
        append!(self.stackup.layers, Layer);
        append!(self.materials, Material);
        append!(self.nets, Net);
        append!(self.components, Component);
        append!(self.pins, Pin);
        append!(self.traces, Trace);
        append!(self.arcs, Arc);
        append!(self.areas, Area);
        append!(self.padstacks, Padstack);
        append!(self.pads, Pad);
        append!(self.drills, Drill);
        append!(self.vias, Via);
        append!(self.outlines, Outline);
        append!(self.connectivity.nodes, Node);
        append!(self.connectivity.edges, Edge);
        for area in &self.areas {
            append!(area.cutouts, Cutout);
        }
        for outline in &self.outlines {
            append!(outline.cutouts, Cutout);
        }
        result
    }
    pub fn frame(&self, id: &ObjectId) -> Result<&CoordinateFrame, ModelError> {
        self.frames
            .iter()
            .find(|x| &x.meta.id == id)
            .ok_or(ModelError::InvalidReference)
    }
    pub fn exact_point_m(&self, point: &Point) -> Result<[Rational; 2], ModelError> {
        self.frame(&point.frame)?.exact_m(point)
    }
    pub fn solver_point_m(&self, point: &Point) -> Result<[f64; 2], ModelError> {
        self.frame(&point.frame)?.solver_m(point)
    }
    /// Convert a complete geometry set together. Distinct exact coordinates from
    /// different grids/origins must not collapse to one binary64 axis coordinate.
    pub fn solver_points_m(&self, points: &[&Point]) -> Result<Vec<[f64; 2]>, ModelError> {
        let mut axes = [BTreeMap::new(), BTreeMap::new()];
        let mut result = Vec::with_capacity(points.len());
        for point in points {
            let exact = self.exact_point_m(point)?;
            let converted = self.solver_point_m(point)?;
            for i in 0..2 {
                let bits = if converted[i] == 0.0 {
                    0
                } else {
                    converted[i].to_bits()
                };
                if let Some(previous) = axes[i].insert(bits, exact[i]) {
                    require(previous == exact[i], ModelError::PrecisionLoss)?;
                }
            }
            result.push(converted);
        }
        Ok(result)
    }
    fn all_points(&self) -> Vec<&Point> {
        fn boundary_points<'a>(boundary: &'a Boundary, points: &mut Vec<&'a Point>) {
            for segment in &boundary.segments {
                match segment {
                    BoundarySegment::Line { start, end } => points.extend([start, end]),
                    BoundarySegment::Arc {
                        centre, start, end, ..
                    } => points.extend([centre, start, end]),
                }
            }
        }
        let mut points = Vec::new();
        for c in &self.components {
            points.push(&c.placement.position);
        }
        for t in &self.traces {
            points.extend(t.centreline.iter());
        }
        for a in &self.arcs {
            points.extend([&a.centre, &a.start, &a.end]);
        }
        for a in &self.areas {
            boundary_points(&a.outer, &mut points);
            for c in &a.cutouts {
                boundary_points(&c.boundary, &mut points);
            }
        }
        for o in &self.outlines {
            boundary_points(&o.outer, &mut points);
            for c in &o.cutouts {
                boundary_points(&c.boundary, &mut points);
            }
        }
        for p in &self.pads {
            points.push(&p.position);
        }
        for v in &self.vias {
            points.push(&v.position);
        }
        for n in &self.connectivity.nodes {
            if let ContactSite::AreaWitness { point, .. } = &n.site {
                points.push(point);
            }
        }
        points
    }
    fn point(&self, point: &Point) -> Result<(), ModelError> {
        self.solver_point_m(point).map(|_| ())
    }
    fn layer(&self, id: &ObjectId) -> Result<&Layer, ModelError> {
        self.stackup
            .layers
            .iter()
            .find(|x| &x.meta.id == id)
            .ok_or(ModelError::InvalidReference)
    }
    fn copper_layer(&self, id: &ObjectId) -> Result<&Layer, ModelError> {
        let layer = self.layer(id)?;
        require(
            layer.kind == LayerKind::Copper,
            ModelError::InvalidReference,
        )?;
        Ok(layer)
    }
    fn net(&self, id: &Option<ObjectId>) -> Result<(), ModelError> {
        require(
            id.as_ref()
                .map_or(true, |id| self.nets.iter().any(|n| &n.meta.id == id)),
            ModelError::InvalidReference,
        )
    }
    fn padstack(&self, id: &ObjectId) -> Result<&Padstack, ModelError> {
        self.padstacks
            .iter()
            .find(|x| &x.meta.id == id)
            .ok_or(ModelError::InvalidReference)
    }
    fn conductive(&self, stack: &ObjectId, layer: &ObjectId) -> Result<(), ModelError> {
        require(
            self.padstack(stack)?
                .entries
                .iter()
                .any(|x| &x.layer == layer && x.role == PadRole::Conductive),
            ModelError::InvalidConnection,
        )
    }
    fn span(&self, span: &ViaSpan) -> Result<(u32, u32), ModelError> {
        let a = self.copper_layer(&span.first_layer)?.order;
        let b = self.copper_layer(&span.last_layer)?.order;
        require(a < b, ModelError::InvalidConnection)?;
        Ok((a, b))
    }
    fn arc_geometry(&self, centre: &Point, start: &Point, end: &Point) -> Result<(), ModelError> {
        self.point(centre)?;
        self.point(start)?;
        self.point(end)?;
        // Source-grid equality is exact; mixed-frame arcs require explicit normalization.
        require(
            centre.frame == start.frame && start.frame == end.frame,
            ModelError::InvalidGeometry,
        )?;
        let squared = |p: &Point| -> Result<i128, ModelError> {
            let dx = i128::from(p.ticks[0]) - i128::from(centre.ticks[0]);
            let dy = i128::from(p.ticks[1]) - i128::from(centre.ticks[1]);
            dx.checked_mul(dx)
                .and_then(|a| dy.checked_mul(dy).and_then(|b| a.checked_add(b)))
                .ok_or(ModelError::CoordinateOverflow)
        };
        let radius = squared(start)?;
        require(
            radius > 0 && radius == squared(end)? && start != end,
            ModelError::InvalidGeometry,
        )
    }
    fn boundary(&self, boundary: &Boundary) -> Result<(), ModelError> {
        require(boundary.segments.len() >= 2, ModelError::InvalidGeometry)?;
        let mut endpoints = Vec::new();
        for segment in &boundary.segments {
            let (start, end) = match segment {
                BoundarySegment::Line { start, end } => {
                    self.point(start)?;
                    self.point(end)?;
                    (start, end)
                }
                BoundarySegment::Arc {
                    centre, start, end, ..
                } => {
                    self.arc_geometry(centre, start, end)?;
                    (start, end)
                }
            };
            let a = self.exact_point_m(start)?;
            let b = self.exact_point_m(end)?;
            require(a != b, ModelError::InvalidGeometry)?;
            endpoints.push((a, b));
        }
        for i in 0..endpoints.len() {
            require(
                endpoints[i].1 == endpoints[(i + 1) % endpoints.len()].0,
                ModelError::InvalidGeometry,
            )?;
        }
        Ok(())
    }
    fn shape(&self, shape: &PadShape) -> Result<(), ModelError> {
        match shape {
            PadShape::Circle { diameter_m } => {
                require(diameter_m.positive(), ModelError::InvalidDimension)
            }
            PadShape::Rectangle { size_m } => require(
                size_m.iter().all(|x| x.positive()),
                ModelError::InvalidDimension,
            ),
            PadShape::Custom(boundary) => self.boundary(boundary),
        }
    }
    /// Checks identity, references, exact-coordinate envelopes and explicit graph contracts.
    /// Missing properties are reported separately. No polygon containment/topology proof.
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::default();
        let mut source_ids = BTreeSet::new();
        for source in &self.sources {
            if !source_ids.insert(&source.id) {
                report.add(Severity::Error, ModelError::DuplicateId, None, "source ID");
            }
            let id = &source.id;
            let valid = !id.files.is_empty()
                && !id.format_version.is_empty()
                && !id.importer_version.is_empty()
                && id.files.iter().all(|f| {
                    !f.logical_path.is_empty()
                        && !f.logical_path.starts_with('/')
                        && !f.logical_path.contains('\\')
                        && !f.logical_path.contains(':')
                        && !f
                            .logical_path
                            .split('/')
                            .any(|p| p == ".." || p == "." || p.is_empty())
                })
                && id
                    .files
                    .windows(2)
                    .all(|w| w[0].logical_path < w[1].logical_path)
                && id.conditions.iter().all(|(k, _)| !k.is_empty())
                && id.conditions.windows(2).all(|w| w[0].0 < w[1].0);
            if !valid {
                report.add(
                    Severity::Error,
                    ModelError::InvalidIdentity,
                    None,
                    "source identity ordering/fields",
                );
            }
        }
        let mut ids = BTreeSet::new();
        for (meta, kind) in self.metas() {
            let id = &meta.id;
            report.check(
                require(ids.insert(id), ModelError::DuplicateId),
                id,
                "globally unique ID",
            );
            report.check(
                require(
                    id.kind == kind
                        && !id.source_object.is_empty()
                        && source_ids.contains(&id.source),
                    ModelError::InvalidIdentity,
                ),
                id,
                "source/type binding",
            );
            report.check(
                require(
                    !meta.provenance.source_object_type.is_empty(),
                    ModelError::InvalidProvenance,
                ),
                id,
                "source object type",
            );
            for step in &meta.provenance.history {
                report.check(
                    require(
                        !step.operation.trim().is_empty()
                            && step.max_error_m.map_or(true, |x| x.numerator() >= 0)
                            && (step.quality != ConversionQuality::Exact
                                || step.max_error_m.map_or(true, |x| x.numerator() == 0))
                            && (!matches!(
                                step.quality,
                                ConversionQuality::Approximate | ConversionQuality::Lossy
                            ) || step.max_error_m.is_some()),
                        ModelError::InvalidProvenance,
                    ),
                    id,
                    "conversion quality/error bound",
                );
                if step.quality != ConversionQuality::Exact {
                    report.add(
                        Severity::Assumption,
                        ModelError::AssumedValue,
                        Some(id),
                        "non-exact conversion; no physical validation implied",
                    );
                }
            }
        }
        for frame in &self.frames {
            report.check(
                require(
                    frame.metres_per_tick.positive() && !frame.unit_label.is_empty(),
                    ModelError::InvalidDimension,
                ),
                &frame.meta.id,
                "coordinate grid",
            );
            report.check(
                orientation(&frame.orientation),
                &frame.meta.id,
                "source transform",
            );
        }
        for material in &self.materials {
            let id = &material.meta.id;
            for (p, allow_zero) in [
                (&material.dk, false),
                (&material.df, true),
                (&material.conductivity_s_per_m, false),
            ] {
                // Not every property is required for every material; layer readiness checks below.
                if let Some(v) = p.value() {
                    report.check(
                        require(
                            v.is_finite() && if allow_zero { *v >= 0.0 } else { *v > 0.0 },
                            ModelError::InvalidMaterial,
                        ),
                        id,
                        "material scalar",
                    );
                }
                if !matches!(p, Property::Missing) {
                    property(p, id, "material evidence", &mut report);
                }
            }
            if let Some(band) = material.valid_band.value() {
                report.check(
                    require(
                        band.min_hz.is_finite()
                            && band.max_hz.is_finite()
                            && band.min_hz >= 0.0
                            && band.min_hz <= band.max_hz,
                        ModelError::InvalidMaterial,
                    ),
                    id,
                    "valid material band",
                );
                property(
                    &material.valid_band,
                    id,
                    "material band evidence",
                    &mut report,
                );
            }
        }
        let mut orders = BTreeSet::new();
        for layer in &self.stackup.layers {
            let id = &layer.meta.id;
            report.check(
                require(orders.insert(layer.order), ModelError::InvalidStackup),
                id,
                "unique layer order",
            );
            property(&layer.z_m, id, "missing/assumed layer z", &mut report);
            property(
                &layer.thickness_m,
                id,
                "missing/assumed layer thickness",
                &mut report,
            );
            if let Some(t) = layer.thickness_m.value() {
                report.check(
                    require(t.positive(), ModelError::InvalidDimension),
                    id,
                    "positive thickness",
                );
            }
            if let Some(m) = &layer.material {
                report.check(
                    require(
                        self.materials.iter().any(|x| &x.meta.id == m),
                        ModelError::InvalidReference,
                    ),
                    id,
                    "material reference",
                );
            } else {
                report.add(
                    Severity::Missing,
                    ModelError::MissingRequired,
                    Some(id),
                    "layer material",
                );
            }
        }
        let mut layers: Vec<_> = self.stackup.layers.iter().collect();
        layers.sort_by_key(|x| x.order);
        for pair in layers.windows(2) {
            if let (Some(z), Some(t), Some(next_z)) = (
                pair[0].z_m.value(),
                pair[0].thickness_m.value(),
                pair[1].z_m.value(),
            ) {
                report.check(
                    z.checked_add(*t).and_then(|end| {
                        require(
                            end.checked_cmp(*next_z)? == std::cmp::Ordering::Equal,
                            ModelError::InvalidStackup,
                        )
                    }),
                    &pair[1].meta.id,
                    "contiguous top-face z and thickness",
                );
            }
        }
        for component in &self.components {
            report.check(
                self.point(&component.placement.position)
                    .and_then(|_| orientation(&component.placement.orientation)),
                &component.meta.id,
                "component placement",
            );
        }
        for pin in &self.pins {
            let result = (|| {
                require(
                    self.components.iter().any(|x| x.meta.id == pin.component),
                    ModelError::InvalidReference,
                )?;
                let pad = self
                    .pads
                    .iter()
                    .find(|x| x.meta.id == pin.pad)
                    .ok_or(ModelError::InvalidReference)?;
                self.net(&pin.net)?;
                require(
                    pin.net == pad.net && !pin.number.is_empty(),
                    ModelError::InvalidConnection,
                )
            })();
            report.check(result, &pin.meta.id, "pin/component/pad/net");
        }
        for trace in &self.traces {
            let result = (|| {
                self.copper_layer(&trace.layer)?;
                self.net(&trace.net)?;
                require(
                    trace.width_m.positive() && trace.centreline.len() >= 2,
                    ModelError::InvalidGeometry,
                )?;
                for point in &trace.centreline {
                    self.point(point)?;
                }
                for p in trace.centreline.windows(2) {
                    require(
                        self.exact_point_m(&p[0])? != self.exact_point_m(&p[1])?,
                        ModelError::InvalidGeometry,
                    )?;
                }
                Ok(())
            })();
            report.check(result, &trace.meta.id, "trace geometry/references");
        }
        for arc in &self.arcs {
            let result = (|| {
                self.copper_layer(&arc.layer)?;
                self.net(&arc.net)?;
                require(arc.width_m.positive(), ModelError::InvalidDimension)?;
                self.arc_geometry(&arc.centre, &arc.start, &arc.end)
            })();
            report.check(result, &arc.meta.id, "arc geometry/references");
        }
        for area in &self.areas {
            report.check(
                self.copper_layer(&area.layer)
                    .and_then(|_| self.net(&area.net))
                    .and_then(|_| self.boundary(&area.outer)),
                &area.meta.id,
                "area boundary/layer/net",
            );
            for cutout in &area.cutouts {
                report.check(
                    self.boundary(&cutout.boundary),
                    &cutout.meta.id,
                    "cutout boundary (not copper)",
                );
            }
        }
        for outline in &self.outlines {
            report.check(
                self.boundary(&outline.outer),
                &outline.meta.id,
                "outline boundary",
            );
            for cutout in &outline.cutouts {
                report.check(
                    self.boundary(&cutout.boundary),
                    &cutout.meta.id,
                    "outline cutout",
                );
            }
        }
        for stack in &self.padstacks {
            let mut entries = BTreeSet::new();
            for entry in &stack.entries {
                report.check(
                    self.copper_layer(&entry.layer)
                        .and_then(|_| self.shape(&entry.shape))
                        .and_then(|_| {
                            require(
                                entries.insert((&entry.layer, entry.role as u8)),
                                ModelError::InvalidGeometry,
                            )
                        }),
                    &stack.meta.id,
                    "padstack layer/role/shape",
                );
            }
            report.check(
                require(!stack.entries.is_empty(), ModelError::InvalidGeometry),
                &stack.meta.id,
                "nonempty padstack",
            );
        }
        for pad in &self.pads {
            report.check(
                self.point(&pad.position)
                    .and_then(|_| orientation(&pad.orientation))
                    .and_then(|_| self.padstack(&pad.padstack).map(|_| ()))
                    .and_then(|_| self.net(&pad.net)),
                &pad.meta.id,
                "pad placement/references",
            );
        }
        for drill in &self.drills {
            report.check(
                self.span(&drill.span).and_then(|_| {
                    require(drill.diameter_m.positive(), ModelError::InvalidDimension)
                }),
                &drill.meta.id,
                "drill span/diameter",
            );
        }
        for via in &self.vias {
            report.check(
                self.validate_via(via),
                &via.meta.id,
                "via actual connected layers/plating/span",
            );
        }
        let lookup = ContactValidationIndex::new(self);
        for node in &self.connectivity.nodes {
            report.check(
                self.node_data(node, &lookup).map(|_| ()),
                &node.meta.id,
                "contact node/conductive owner/site",
            );
        }
        for edge in &self.connectivity.edges {
            report.check(
                self.validate_edge(edge, &lookup),
                &edge.meta.id,
                "explicit connection evidence",
            );
        }
        if let Err(error) = self.solver_points_m(&self.all_points()) {
            report.add(
                Severity::Error,
                error,
                None,
                "whole-model coordinate conversion/cross-frame gap preservation",
            );
        }
        if self.sources.is_empty() {
            report.add(
                Severity::Missing,
                ModelError::MissingRequired,
                None,
                "import source missing",
            );
        }
        if self.stackup.layers.is_empty() {
            report.add(
                Severity::Missing,
                ModelError::MissingRequired,
                None,
                "stackup empty",
            );
        }
        if self.outlines.is_empty() {
            report.add(
                Severity::Missing,
                ModelError::MissingRequired,
                None,
                "outline missing",
            );
        }
        report
    }
    fn validate_via(&self, via: &Via) -> Result<(), ModelError> {
        self.point(&via.position)?;
        self.net(&via.net)?;
        self.padstack(&via.padstack)?;
        let (first, last) = self.span(&via.span)?;
        let drill = self
            .drills
            .iter()
            .find(|x| x.meta.id == via.drill)
            .ok_or(ModelError::InvalidReference)?;
        require(drill.span == via.span, ModelError::InvalidConnection)?;
        let mut unique = BTreeSet::new();
        for layer in &via.connected_layers {
            let order = self.copper_layer(layer)?.order;
            require(
                drill.plated && order >= first && order <= last && unique.insert(layer),
                ModelError::InvalidConnection,
            )?;
            self.conductive(&via.padstack, layer)?;
        }
        // Geometric pad entries, including anti-pads, cannot extend outside the via.
        for entry in &self.padstack(&via.padstack)?.entries {
            let order = self.copper_layer(&entry.layer)?.order;
            require(
                order >= first && order <= last,
                ModelError::InvalidConnection,
            )?;
        }
        let copper: Vec<_> = self
            .stackup
            .layers
            .iter()
            .filter(|x| x.kind == LayerKind::Copper)
            .collect();
        let min = copper
            .iter()
            .map(|x| x.order)
            .min()
            .ok_or(ModelError::InvalidStackup)?;
        let max = copper
            .iter()
            .map(|x| x.order)
            .max()
            .ok_or(ModelError::InvalidStackup)?;
        match via.kind {
            ViaKind::Through => require(first == min && last == max, ModelError::InvalidConnection),
            ViaKind::Blind => require(
                (first == min) != (last == max),
                ModelError::InvalidConnection,
            ),
            ViaKind::Buried => require(first > min && last < max, ModelError::InvalidConnection),
            ViaKind::Microvia => require(
                !copper.iter().any(|x| x.order > first && x.order < last),
                ModelError::InvalidConnection,
            ),
        }
    }
    fn node_data<'a>(
        &'a self,
        node: &'a ConnectivityNode,
        lookup: &ContactValidationIndex<'a>,
    ) -> Result<(Option<&'a ObjectId>, [Rational; 2]), ModelError> {
        self.copper_layer(&node.layer)?;
        let (net, point) = match &node.site {
            ContactSite::TraceVertex(i) => {
                let trace = lookup
                    .traces
                    .get(&node.conductor)
                    .copied()
                    .ok_or(ModelError::InvalidReference)?;
                require(trace.layer == node.layer, ModelError::InvalidConnection)?;
                (
                    &trace.net,
                    trace
                        .centreline
                        .get(*i)
                        .ok_or(ModelError::InvalidReference)?,
                )
            }
            ContactSite::TraceInterior(point) => {
                let trace = lookup
                    .traces
                    .get(&node.conductor)
                    .copied()
                    .ok_or(ModelError::InvalidReference)?;
                require(trace.layer == node.layer, ModelError::InvalidConnection)?;
                require(trace.centreline.len() == 2, ModelError::InvalidConnection)?;
                let a = &trace.centreline[0];
                let b = &trace.centreline[1];
                // PADS source ticks are bounded. i128 avoids i64 cross-product overflow.
                let (ax, ay) = (i128::from(a.ticks[0]), i128::from(a.ticks[1]));
                let (bx, by) = (i128::from(b.ticks[0]), i128::from(b.ticks[1]));
                let (px, py) = (i128::from(point.ticks[0]), i128::from(point.ticks[1]));
                let cross = (bx - ax) * (py - ay) - (by - ay) * (px - ax);
                let dot = (px - ax) * (px - bx) + (py - ay) * (py - by);
                require(
                    a.frame == point.frame && b.frame == point.frame && cross == 0 && dot < 0,
                    ModelError::InvalidConnection,
                )?;
                (&trace.net, point)
            }
            ContactSite::ArcStart | ContactSite::ArcEnd => {
                let arc = lookup
                    .arcs
                    .get(&node.conductor)
                    .copied()
                    .ok_or(ModelError::InvalidReference)?;
                require(arc.layer == node.layer, ModelError::InvalidConnection)?;
                (
                    &arc.net,
                    if matches!(node.site, ContactSite::ArcStart) {
                        &arc.start
                    } else {
                        &arc.end
                    },
                )
            }
            ContactSite::PadCentre => {
                let pad = lookup
                    .pads
                    .get(&node.conductor)
                    .copied()
                    .ok_or(ModelError::InvalidReference)?;
                lookup.conductive(&pad.padstack, &node.layer)?;
                (&pad.net, &pad.position)
            }
            ContactSite::ViaCentre => {
                let via = lookup
                    .vias
                    .get(&node.conductor)
                    .copied()
                    .ok_or(ModelError::InvalidReference)?;
                // The via itself was checked in the board's via-validation pass.
                // Repeat only the node-specific layer and plating checks here.
                require(
                    via.connected_layers.contains(&node.layer),
                    ModelError::InvalidConnection,
                )?;
                lookup.conductive(&via.padstack, &node.layer)?;
                (&via.net, &via.position)
            }
            ContactSite::AreaWitness { point, evidence } => {
                let area = lookup
                    .areas
                    .get(&node.conductor)
                    .copied()
                    .ok_or(ModelError::InvalidReference)?;
                require(
                    area.layer == node.layer && !evidence.trim().is_empty(),
                    ModelError::InvalidConnection,
                )?;
                self.point(point)?;
                (&area.net, point)
            }
        };
        lookup.known_net(net)?;
        Ok((net.as_ref(), self.exact_point_m(point)?))
    }
    fn validate_edge<'a>(
        &'a self,
        edge: &ConnectivityEdge,
        lookup: &ContactValidationIndex<'a>,
    ) -> Result<(), ModelError> {
        require(edge.a != edge.b, ModelError::InvalidConnection)?;
        let a = lookup
            .nodes
            .get(&edge.a)
            .copied()
            .ok_or(ModelError::InvalidReference)?;
        let b = lookup
            .nodes
            .get(&edge.b)
            .copied()
            .ok_or(ModelError::InvalidReference)?;
        let (net_a, pos_a) = self.node_data(a, lookup)?;
        let (net_b, pos_b) = self.node_data(b, lookup)?;
        // Unknown/floating data is never shorted by implication; an electrical edge needs a known matching net.
        require(
            net_a.is_some() && net_a == net_b,
            ModelError::InvalidConnection,
        )?;
        match &edge.evidence {
            ConnectionEvidence::SameLayerContact { evidence } => require(
                a.layer == b.layer && pos_a == pos_b && !evidence.trim().is_empty(),
                ModelError::InvalidConnection,
            ),
            ConnectionEvidence::ViaBarrel { via } => {
                require(
                    a.conductor == *via
                        && b.conductor == *via
                        && a.layer != b.layer
                        && matches!(a.site, ContactSite::ViaCentre)
                        && matches!(b.site, ContactSite::ViaCentre),
                    ModelError::InvalidConnection,
                )?;
                let via = lookup
                    .vias
                    .get(via)
                    .copied()
                    .ok_or(ModelError::InvalidReference)?;
                // Whole-board via validation checks plated drill, span and
                // conductive padstack. Here ensure the edge uses its two
                // independently proven contact layers.
                require(
                    via.connected_layers.contains(&a.layer)
                        && via.connected_layers.contains(&b.layer),
                    ModelError::InvalidConnection,
                )
            }
        }
    }
    /// Material/stackup readiness ONLY. This never certifies geometry, reference or solver support.
    pub fn validate_materials_at(&self, frequency_hz: f64) -> ValidationReport {
        let mut report = self.validate();
        if !frequency_hz.is_finite() || frequency_hz < 0.0 {
            report.add(
                Severity::Error,
                ModelError::InvalidMaterial,
                None,
                "analysis frequency",
            );
            return report;
        }
        for layer in &self.stackup.layers {
            let material = layer
                .material
                .as_ref()
                .and_then(|id| self.materials.iter().find(|x| &x.meta.id == id));
            if let Some(m) = material {
                match layer.kind {
                    LayerKind::Copper => property(
                        &m.conductivity_s_per_m,
                        &m.meta.id,
                        "conductivity required",
                        &mut report,
                    ),
                    LayerKind::Dielectric(_) | LayerKind::Mask => {
                        property(&m.dk, &m.meta.id, "Dk required", &mut report);
                        property(
                            &m.df,
                            &m.meta.id,
                            "Df required (RF, not DC leakage)",
                            &mut report,
                        );
                    }
                    LayerKind::Other => report.add(
                        Severity::Missing,
                        ModelError::MissingRequired,
                        Some(&layer.meta.id),
                        "unknown layer material model",
                    ),
                }
                property(
                    &m.valid_band,
                    &m.meta.id,
                    "valid frequency band required",
                    &mut report,
                );
                if let Some(band) = m.valid_band.value() {
                    if frequency_hz < band.min_hz || frequency_hz > band.max_hz {
                        report.add(
                            Severity::Error,
                            ModelError::OutsideMaterialBand,
                            Some(&m.meta.id),
                            "extrapolation prohibited",
                        );
                    }
                }
            }
        }
        report
    }
    /// An immutable shared borrow, preventing divergent source geometry in consumers.
    pub fn validated_view(&self) -> Result<BoardView<'_>, ValidationReport> {
        let report = self.validate();
        if report.has_errors() {
            Err(report)
        } else {
            Ok(BoardView {
                model: self,
                report,
            })
        }
    }
}
/// Structural validation only. Missing/assumed data remains visible; not physical Validated.
pub struct BoardView<'a> {
    model: &'a BoardModel,
    report: ValidationReport,
}
impl<'a> BoardView<'a> {
    pub fn model(&self) -> &'a BoardModel {
        self.model
    }
    pub fn report(&self) -> &ValidationReport {
        &self.report
    }
    /// Reachability over explicit edges only. No geometric/net-name auto connection.
    pub fn connected(&self, a: &ObjectId, b: &ObjectId) -> Result<bool, ModelError> {
        let nodes = &self.model.connectivity.nodes;
        require(
            nodes.iter().any(|n| &n.meta.id == a) && nodes.iter().any(|n| &n.meta.id == b),
            ModelError::InvalidReference,
        )?;
        let mut adjacency: BTreeMap<&ObjectId, Vec<&ObjectId>> = BTreeMap::new();
        for edge in &self.model.connectivity.edges {
            adjacency.entry(&edge.a).or_default().push(&edge.b);
            adjacency.entry(&edge.b).or_default().push(&edge.a);
        }
        let mut queue = VecDeque::from([a]);
        let mut seen = BTreeSet::new();
        while let Some(node) = queue.pop_front() {
            if node == b {
                return Ok(true);
            }
            if !seen.insert(node) {
                continue;
            }
            if let Some(neighbors) = adjacency.get(node) {
                queue.extend(neighbors.iter().copied());
            }
        }
        Ok(false)
    }
}
