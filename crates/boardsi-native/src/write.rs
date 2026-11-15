//! SQL writer for the existing canonical model; no new competing board model.
use crate::encoding::{i128_bytes, sql_orientation, Geom};
use crate::shared::*;
use crate::{invalid, Result};
use boardsi_model::*;
use rusqlite::{params, Transaction};
use std::collections::BTreeMap;

fn metas(board: &BoardModel) -> Vec<(&ObjectMeta, ObjectKind)> {
    let mut v = Vec::new();
    for x in &board.frames {
        v.push((&x.meta, ObjectKind::Frame));
    }
    for x in &board.materials {
        v.push((&x.meta, ObjectKind::Material));
    }
    for x in &board.stackup.layers {
        v.push((&x.meta, ObjectKind::Layer));
    }
    for x in &board.nets {
        v.push((&x.meta, ObjectKind::Net));
    }
    for x in &board.components {
        v.push((&x.meta, ObjectKind::Component));
    }
    for x in &board.pins {
        v.push((&x.meta, ObjectKind::Pin));
    }
    for x in &board.traces {
        v.push((&x.meta, ObjectKind::Trace));
    }
    for x in &board.arcs {
        v.push((&x.meta, ObjectKind::Arc));
    }
    for x in &board.areas {
        v.push((&x.meta, ObjectKind::Area));
        for c in &x.cutouts {
            v.push((&c.meta, ObjectKind::Cutout));
        }
    }
    for x in &board.padstacks {
        v.push((&x.meta, ObjectKind::Padstack));
    }
    for x in &board.pads {
        v.push((&x.meta, ObjectKind::Pad));
    }
    for x in &board.drills {
        v.push((&x.meta, ObjectKind::Drill));
    }
    for x in &board.vias {
        v.push((&x.meta, ObjectKind::Via));
    }
    for x in &board.outlines {
        v.push((&x.meta, ObjectKind::Outline));
        for c in &x.cutouts {
            v.push((&c.meta, ObjectKind::Cutout));
        }
    }
    let contact_index = ContactLookup::new(board);
    for x in &board.connectivity.nodes {
        v.push((&x.meta, ObjectKind::Node));
    }
    for x in &board.connectivity.edges {
        v.push((&x.meta, ObjectKind::Edge));
    }
    v
}
fn source_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains('\\')
        && !path.contains(':')
        && !path
            .split('/')
            .any(|s| s == ".." || s == "." || s.is_empty())
}
fn sources(tx: &Transaction<'_>, board: &BoardModel) -> Result<BTreeMap<SourceIdentity, i64>> {
    if board.sources.len() > 100_000 {
        return Err(invalid("source count"));
    }
    let mut indexed: BTreeMap<SourceIdentity, &ImportSource> = BTreeMap::new();
    for src in &board.sources {
        if indexed.insert((*src.id).clone(), src).is_some() {
            return Err(invalid("duplicate SourceIdentity"));
        }
    }
    let mut out = BTreeMap::new();
    for (i, (identity, src)) in indexed.into_iter().enumerate() {
        let source_id = (i + 1) as i64;
        out.insert(identity.clone(), source_id);
        let (fmt, other) = source_name(&identity.format);
        tx.execute("INSERT INTO source_identity(id,source_format,format_other,format_version,importer_version,producer,producer_version) VALUES(?,?,?,?,?,?,?)",
            params![source_id,fmt,other,identity.format_version,identity.importer_version,src.producer,src.producer_version])?;
        if identity.files.is_empty() {
            return Err(invalid("empty SourceIdentity files"));
        }
        let mut last_path = None;
        for (index, f) in identity.files.iter().enumerate() {
            if !source_path(&f.logical_path)
                || last_path.is_some_and(|p: &str| p >= f.logical_path.as_str())
            {
                return Err(invalid("source path order/absolute path"));
            }
            last_path = Some(&f.logical_path);
            // The model carries a hash, not independent proof that file bytes were verified.
            tx.execute(
                "INSERT INTO source_files VALUES(?,?,?,?,?)",
                params![
                    source_id,
                    index as i64,
                    f.logical_path,
                    f.sha256.to_vec(),
                    0i64
                ],
            )?;
        }
        let mut last_key = None;
        for (index, (key, value)) in identity.conditions.iter().enumerate() {
            if key.is_empty() || last_key.is_some_and(|k: &str| k >= key.as_str()) {
                return Err(invalid("source condition order"));
            }
            last_key = Some(key.as_str());
            tx.execute(
                "INSERT INTO source_conditions VALUES(?,?,?,?)",
                params![source_id, index as i64, key, value],
            )?;
        }
        let cap = &src.capability;
        for (dimension, state) in [
            ("G", cap.geometry),
            ("N", cap.net),
            ("P", cap.pin_pad),
            ("S", cap.stackup),
            ("R", cap.reference),
            ("M", cap.model),
        ] {
            tx.execute(
                "INSERT INTO import_capability VALUES(?,?,?)",
                params![source_id, dimension, cap_name(state)],
            )?;
        }
        for (index, covered) in cap.object_coverage.iter().enumerate() {
            tx.execute(
                "INSERT INTO capability_coverage VALUES(?,?,?)",
                params![source_id, index as i64, covered],
            )?;
        }
    }
    Ok(out)
}
fn objects(
    tx: &Transaction<'_>,
    board: &BoardModel,
    source_ids: &BTreeMap<SourceIdentity, i64>,
) -> Result<BTreeMap<ObjectId, i64>> {
    let list = metas(board);
    if list.len() > 1_000_000 {
        return Err(invalid("object count"));
    }
    let mut ordered: BTreeMap<ObjectId, &ObjectMeta> = BTreeMap::new();
    for (meta, kind) in list {
        if meta.id.kind != kind || ordered.insert(meta.id.clone(), meta).is_some() {
            return Err(invalid("duplicate or wrong object kind"));
        }
    }
    let mut out = BTreeMap::new();
    for (i, (id, meta)) in ordered.into_iter().enumerate() {
        let object_id = (i + 1) as i64;
        out.insert(id.clone(), object_id);
        let source_id = *source_ids
            .get(id.source.as_ref())
            .ok_or_else(|| invalid("source not declared"))?;
        tx.execute("INSERT INTO objects(id,source_id,kind,source_object,part,source_object_type) VALUES(?,?,?,?,?,?)",
            params![object_id,source_id,kind_name(id.kind),id.source_object,id.part,
                meta.provenance.source_object_type])?;
        for (index, step) in meta.provenance.history.iter().enumerate() {
            let (num, den) = step
                .max_error_m
                .map(|x| {
                    (
                        Some(i128_bytes(x.numerator())),
                        Some(i128_bytes(x.denominator())),
                    )
                })
                .unwrap_or((None, None));
            tx.execute(
                "INSERT INTO conversion_history VALUES(?,?,?,?,?,?,?)",
                params![
                    object_id,
                    index as i64,
                    step.operation,
                    quality_name(step.quality),
                    num,
                    den,
                    step.note
                ],
            )?;
        }
    }
    Ok(out)
}
fn property_source(
    tx: &Transaction<'_>,
    layer_id: i64,
    key: &str,
    p: &Property<Rational>,
) -> Result<()> {
    if let Some(s) = p.source() {
        if s.reference_frequency_hz
            .is_some_and(|n| !n.is_finite() || n < 0.0)
            || s.temperature_c.is_some_and(|n| !n.is_finite())
        {
            return Err(invalid("property source nonfinite"));
        }
        tx.execute(
            "INSERT INTO layer_property_sources VALUES(?,?,?,?,?,?,?)",
            params![
                layer_id,
                key,
                s.document,
                s.revision,
                s.method,
                s.reference_frequency_hz,
                s.temperature_c
            ],
        )?;
    }
    Ok(())
}
fn rational_property(p: &Property<Rational>) -> (&'static str, Option<Vec<u8>>, Option<Vec<u8>>) {
    match p {
        Property::Missing => ("Missing", None, None),
        Property::Assumed { value, .. } => (
            "Assumed",
            Some(i128_bytes(value.numerator())),
            Some(i128_bytes(value.denominator())),
        ),
        Property::Confirmed { value, .. } => (
            "Confirmed",
            Some(i128_bytes(value.numerator())),
            Some(i128_bytes(value.denominator())),
        ),
    }
}
fn material_property(tx: &Transaction<'_>, id: i64, key: &str, p: &Property<f64>) -> Result<()> {
    let (status, blob, source) = match p {
        Property::Missing => ("Missing", None, None),
        Property::Assumed { value, source } if value.is_finite() => {
            ("Assumed", Some(value.to_le_bytes().to_vec()), Some(source))
        }
        Property::Confirmed { value, source } if value.is_finite() => (
            "Confirmed",
            Some(value.to_le_bytes().to_vec()),
            Some(source),
        ),
        _ => return Err(invalid("material property nonfinite")),
    };
    put_material_property(tx, id, key, status, blob, source)
}
fn put_material_property(
    tx: &Transaction<'_>,
    id: i64,
    key: &str,
    status: &str,
    blob: Option<Vec<u8>>,
    source: Option<&PropertySource>,
) -> Result<()> {
    if let Some(s) = source {
        if s.reference_frequency_hz
            .is_some_and(|x| !x.is_finite() || x < 0.0)
            || s.temperature_c.is_some_and(|x| !x.is_finite())
        {
            return Err(invalid("property evidence"));
        }
    }
    tx.execute("INSERT INTO material_properties(material_id,property_key,status,value_blob,document,revision,method,reference_frequency_hz,temperature_c) VALUES(?,?,?,?,?,?,?,?,?)",
        params![id,key,status,blob,source.map(|s|s.document.as_str()),
            source.map(|s|s.revision.as_str()),source.map(|s|s.method.as_str()),
            source.and_then(|s|s.reference_frequency_hz),source.and_then(|s|s.temperature_c)])?;
    Ok(())
}
fn valid_band(tx: &Transaction<'_>, id: i64, p: &Property<FrequencyRange>) -> Result<()> {
    let (status, blob, source) = match p {
        Property::Missing => ("Missing", None, None),
        Property::Assumed { value, source } | Property::Confirmed { value, source } => {
            if !value.min_hz.is_finite()
                || !value.max_hz.is_finite()
                || value.min_hz < 0.0
                || value.min_hz >= value.max_hz
            {
                return Err(invalid("valid band"));
            }
            let mut b = value.min_hz.to_le_bytes().to_vec();
            b.extend_from_slice(&value.max_hz.to_le_bytes());
            (
                if matches!(p, Property::Assumed { .. }) {
                    "Assumed"
                } else {
                    "Confirmed"
                },
                Some(b),
                Some(source),
            )
        }
    };
    put_material_property(tx, id, "ValidBand", status, blob, source)
}
fn layer_order(board: &BoardModel, id: &ObjectId) -> Result<i64> {
    let x = board
        .stackup
        .layers
        .iter()
        .find(|x| x.meta.id == *id)
        .ok_or_else(|| invalid("missing span layer"))?;
    if x.kind != LayerKind::Copper {
        return Err(invalid("noncopper span"));
    }
    Ok(i64::from(x.order))
}
fn span_order(board: &BoardModel, span: &ViaSpan) -> Result<(i64, i64)> {
    let a = layer_order(board, &span.first_layer)?;
    let b = layer_order(board, &span.last_layer)?;
    if a >= b {
        return Err(invalid("span order"));
    }
    Ok((a, b))
}
/// Indexed contact geometry lookup. BTreeMap avoids O(nodes × conductors) work.
pub(crate) struct ContactLookup<'a> {
    traces: BTreeMap<ObjectId, &'a Trace>,
    arcs: BTreeMap<ObjectId, &'a Arc>,
    pads: BTreeMap<ObjectId, &'a Pad>,
    vias: BTreeMap<ObjectId, &'a Via>,
}
impl<'a> ContactLookup<'a> {
    pub(crate) fn new(board: &'a BoardModel) -> Self {
        Self {
            traces: board.traces.iter().map(|x| (x.meta.id.clone(), x)).collect(),
            arcs: board.arcs.iter().map(|x| (x.meta.id.clone(), x)).collect(),
            pads: board.pads.iter().map(|x| (x.meta.id.clone(), x)).collect(),
            vias: board.vias.iter().map(|x| (x.meta.id.clone(), x)).collect(),
        }
    }
    pub(crate) fn point(
        &self,
        n: &ConnectivityNode,
    ) -> Result<(Point, &'static str, Option<i64>, String)> {
        let (p, kind, index, evidence) = match &n.site {
            ContactSite::TraceVertex(i) => {
                let trace = self.traces.get(&n.conductor).ok_or_else(|| invalid("trace vertex conductor"))?;
                let p = trace.centreline.get(*i).ok_or_else(|| invalid("trace vertex bounds"))?;
                (p.clone(), "TraceVertex", Some(*i as i64), String::new())
            }
            ContactSite::TraceInterior(p) => (p.clone(), "TraceInterior", None, String::new()),
            ContactSite::ArcStart | ContactSite::ArcEnd => {
                let a = self.arcs.get(&n.conductor).ok_or_else(|| invalid("arc contact conductor"))?;
                (
                    if matches!(n.site, ContactSite::ArcStart) { a.start.clone() } else { a.end.clone() },
                    if matches!(n.site, ContactSite::ArcStart) { "ArcStart" } else { "ArcEnd" },
                    None, String::new(),
                )
            }
            ContactSite::PadCentre => {
                let x = self.pads.get(&n.conductor).ok_or_else(|| invalid("pad conductor"))?;
                (x.position.clone(), "PadCentre", None, String::new())
            }
            ContactSite::ViaCentre => {
                let x = self.vias.get(&n.conductor).ok_or_else(|| invalid("via conductor"))?;
                (x.position.clone(), "ViaCentre", None, String::new())
            }
            ContactSite::AreaWitness { point, evidence } =>
                (point.clone(), "AreaWitness", None, evidence.clone()),
            ContactSite::AreaInterior(p) => (p.clone(), "AreaInterior", None, String::new()),
        };
        Ok((p, kind, index, evidence))
    }
}
pub(crate) fn write_board(tx: &Transaction<'_>, board: &BoardModel, uuid: &str) -> Result<()> {
    if !crate::storage_valid(board) {
        return Err(invalid("canonical model validation"));
    }
    tx.execute(
        "INSERT INTO file_meta VALUES(1,?,1,0,1,0,?,'BoardSI','F_003',strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
        params![
            uuid,
            match board.kind {
                BoardKind::Pcb => "Pcb",
                BoardKind::BgaSubstrate => "BgaSubstrate",
            }
        ],
    )?;
    let sources = sources(tx, board)?;
    let ids = objects(tx, board, &sources)?;
    for x in &board.frames {
        let id = id_of(&ids, &x.meta.id)?;
        let (mode, q, r, mirror) = sql_orientation(&x.orientation)?;
        tx.execute(
            "INSERT INTO coordinate_frames VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)",
            params![
                id,
                x.unit_label,
                i128_bytes(x.metres_per_tick.numerator()),
                i128_bytes(x.metres_per_tick.denominator()),
                i128_bytes(x.origin_m[0].numerator()),
                i128_bytes(x.origin_m[0].denominator()),
                i128_bytes(x.origin_m[1].numerator()),
                i128_bytes(x.origin_m[1].denominator()),
                mode,
                q,
                r,
                mirror,
                x.source_layer_order_reversed as i64
            ],
        )?;
    }
    for x in &board.materials {
        let id = id_of(&ids, &x.meta.id)?;
        tx.execute("INSERT INTO materials VALUES(?,?)", params![id, x.name])?;
        material_property(tx, id, "Dk", &x.dk)?;
        material_property(tx, id, "Df", &x.df)?;
        material_property(tx, id, "Conductivity", &x.conductivity_s_per_m)?;
        valid_band(tx, id, &x.valid_band)?;
    }
    for x in &board.stackup.layers {
        let id = id_of(&ids, &x.meta.id)?;
        let (zs, zn, zd) = rational_property(&x.z_m);
        let (ts, tn, td) = rational_property(&x.thickness_m);
        tx.execute(
            "INSERT INTO layers VALUES(?,?,?,?,?,?,?,?,?,?,?)",
            params![
                id,
                i64::from(x.order),
                x.name,
                layer_name(x.kind),
                opt_id(&ids, &x.material)?,
                zs,
                zn,
                zd,
                ts,
                tn,
                td
            ],
        )?;
        property_source(tx, id, "z_m", &x.z_m)?;
        property_source(tx, id, "thickness_m", &x.thickness_m)?;
    }
    for x in &board.nets {
        tx.execute(
            "INSERT INTO nets VALUES(?,?,?)",
            params![id_of(&ids, &x.meta.id)?, x.name, net_name(x.role)],
        )?;
    }
    let mut geoms = Vec::new();
    for x in &board.traces {
        geoms.push((Geom::Trace(x), None));
    }
    for x in &board.arcs {
        geoms.push((Geom::Arc(x), None));
    }
    for x in &board.areas {
        geoms.push((Geom::Area(x), None));
    }
    for x in &board.padstacks {
        geoms.push((Geom::Padstack(x), None));
    }
    for x in &board.pads {
        geoms.push((Geom::Pad(x), None));
    }
    for x in &board.drills {
        geoms.push((Geom::Drill(x), None));
    }
    for x in &board.vias {
        geoms.push((Geom::Via(x), None));
    }
    for x in &board.outlines {
        geoms.push((Geom::Outline(x), None));
    }
    // Parent area or outline has to exist before child Cutout: FK is immediate.
    for x in &board.areas {
        for c in &x.cutouts {
            geoms.push((Geom::Cutout(c), Some(x.meta.id.clone())));
        }
    }
    for x in &board.outlines {
        for c in &x.cutouts {
            geoms.push((Geom::Cutout(c), Some(x.meta.id.clone())));
        }
    }
    for (geom, parent) in geoms {
        let id = id_of(&ids, &geom.meta().id)?;
        let (record, refs) = geom.encoded()?;
        let blob = record.encode()?;
        let (layer, net, padstack, span, plating) = match &geom {
            Geom::Trace(x) => (
                Some(id_of(&ids, &x.layer)?),
                opt_id(&ids, &x.net)?,
                None,
                None,
                None,
            ),
            Geom::Arc(x) => (
                Some(id_of(&ids, &x.layer)?),
                opt_id(&ids, &x.net)?,
                None,
                None,
                None,
            ),
            Geom::Area(x) => (
                Some(id_of(&ids, &x.layer)?),
                opt_id(&ids, &x.net)?,
                None,
                None,
                None,
            ),
            Geom::Pad(x) => (
                None,
                opt_id(&ids, &x.net)?,
                Some(id_of(&ids, &x.padstack)?),
                None,
                None,
            ),
            Geom::Via(x) => (
                None,
                opt_id(&ids, &x.net)?,
                Some(id_of(&ids, &x.padstack)?),
                Some(span_order(board, &x.span)?),
                Some("Unknown"),
            ),
            Geom::Drill(x) => (
                None,
                None,
                None,
                Some(span_order(board, &x.span)?),
                Some(if x.plated {
                    "SourceDeclaredPlated"
                } else {
                    "NonPlated"
                }),
            ),
            _ => (None, None, None, None, None),
        };
        let (first, last) = span
            .map(|(a, b)| (Some(a), Some(b)))
            .unwrap_or((None, None));
        tx.execute("INSERT INTO geometry(object_id,geometry_kind,codec_version,envelope,layer_id,net_id,parent_geometry_id,padstack_id,first_copper_order,last_copper_order,plating) VALUES(?,?,?,?,?,?,?,?,?,?,?)",
            params![id,record.kind,1i64,blob,layer,net,parent.as_ref().map(|p|id_of(&ids,p)).transpose()?,
                padstack,first,last,plating])?;
        for (idx, (ref_id, role)) in refs.entries.iter().enumerate() {
            tx.execute(
                "INSERT INTO geometry_refs VALUES(?,?,?,?)",
                params![id, idx as i64, id_of(&ids, ref_id)?, role],
            )?;
        }
    }
    for x in &board.components {
        let (mode, q, r, mirror) = sql_orientation(&x.placement.orientation)?;
        tx.execute(
            "INSERT INTO components VALUES(?,?,?,?,?,?,?,?,?,?)",
            params![
                id_of(&ids, &x.meta.id)?,
                x.reference,
                id_of(&ids, &x.placement.position.frame)?,
                x.placement.position.ticks[0],
                x.placement.position.ticks[1],
                mode,
                q,
                r,
                mirror,
                x.placement.bottom_side as i64
            ],
        )?;
    }
    for x in &board.pins {
        tx.execute(
            "INSERT INTO pins VALUES(?,?,?,?,?)",
            params![
                id_of(&ids, &x.meta.id)?,
                id_of(&ids, &x.component)?,
                x.number,
                id_of(&ids, &x.pad)?,
                opt_id(&ids, &x.net)?
            ],
        )?;
    }
    for x in &board.connectivity.nodes {
        let (p, kind, index, evidence) = contact_index.point(x)?;
        tx.execute(
            "INSERT INTO connectivity_nodes VALUES(?,?,?,?,?,?,?,?,?)",
            params![
                id_of(&ids, &x.meta.id)?,
                id_of(&ids, &x.conductor)?,
                id_of(&ids, &x.layer)?,
                id_of(&ids, &p.frame)?,
                p.ticks[0],
                p.ticks[1],
                kind,
                index,
                evidence
            ],
        )?;
    }
    for x in &board.connectivity.edges {
        let (kind, text, witness) = match &x.evidence {
            ConnectionEvidence::SameLayerContact { evidence } => {
                ("SameLayerContact", evidence.clone(), None)
            }
            ConnectionEvidence::ViaBarrel { via } => {
                ("ViaBarrel", String::new(), Some(id_of(&ids, via)?))
            }
            ConnectionEvidence::AreaContinuity { area } => {
                ("AreaContinuity", String::new(), Some(id_of(&ids, area)?))
            }
        };
        tx.execute(
            "INSERT INTO connectivity_edges VALUES(?,?,?,?,?,?,?)",
            params![
                id_of(&ids, &x.meta.id)?,
                id_of(&ids, &x.a)?,
                id_of(&ids, &x.b)?,
                kind,
                text,
                witness,
                "Unverified"
            ],
        )?;
    }
    // There is no diagnostics vector on BoardModel: never invent claims.
    // Unknown required fields/features are intentionally NOT materialized as native v1.
    Ok(())
}
