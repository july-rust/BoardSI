//! Hardened SQL reader. Referential checks are independent of FK constraints.
use crate::bsig::GeometryRecord;
use crate::encoding::{decode, from_rational, from_sql_orientation, reference_dict, Decoded};
use crate::shared::*;
use crate::store::LoadedBoard;
use crate::{invalid, NativeError, Result};
use boardsi_model::*;
use rusqlite::{Connection, OptionalExtension};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc as Shared;

type GeometryBytesById = BTreeMap<ObjectId, (Vec<u8>, Vec<(ObjectId, String)>)>;
type PendingArea = (ObjectMeta, Boundary, Vec<ObjectId>, ObjectId, Option<ObjectId>);

fn rational_pair(num: &[u8], den: &[u8]) -> Result<Rational> {
    let mut b = Vec::with_capacity(32);
    b.extend_from_slice(num);
    b.extend_from_slice(den);
    from_rational(&b)
}
fn status_property(
    status: &str,
    num: Option<Vec<u8>>,
    den: Option<Vec<u8>>,
    source: Option<PropertySource>,
) -> Result<Property<Rational>> {
    match (status, num, den, source) {
        ("Missing", None, None, None) => Ok(Property::Missing),
        ("Assumed", Some(n), Some(d), Some(s)) => Ok(Property::Assumed {
            value: rational_pair(&n, &d)?,
            source: s,
        }),
        ("Confirmed", Some(n), Some(d), Some(s)) => Ok(Property::Confirmed {
            value: rational_pair(&n, &d)?,
            source: s,
        }),
        _ => Err(invalid("rational property/status/source mismatch")),
    }
}
fn property_source(conn: &Connection, id: i64, key: &str) -> Result<Option<PropertySource>> {
    let row=conn.query_row(
        "SELECT document,revision,method,reference_frequency_hz,temperature_c FROM layer_property_sources WHERE layer_id=? AND property_key=?",
        (id,key),|r|Ok(PropertySource{document:r.get(0)?,revision:r.get(1)?,
            method:r.get(2)?,reference_frequency_hz:r.get(3)?,temperature_c:r.get(4)?})
    ).optional()?;
    check_source(row.as_ref())?;
    Ok(row)
}
fn check_source(source: Option<&PropertySource>) -> Result<()> {
    if let Some(s) = source {
        if s.document.trim().is_empty()
            || s.reference_frequency_hz
                .is_some_and(|x| !x.is_finite() || x < 0.0)
            || s.temperature_c.is_some_and(|x| !x.is_finite())
        {
            return Err(invalid("invalid property source"));
        }
    }
    Ok(())
}
fn read_source_ids(conn: &Connection) -> Result<(Vec<ImportSource>, BTreeMap<i64, SourceId>)> {
    let mut out = Vec::new();
    let mut refs = BTreeMap::new();
    let mut s=conn.prepare("SELECT id,source_format,format_other,format_version,importer_version,producer,producer_version FROM source_identity ORDER BY id")?;
    let mut rows = s.query([])?;
    while let Some(r) = rows.next()? {
        let id: i64 = r.get(0)?;
        let format: String = r.get(1)?;
        let other: Option<String> = r.get(2)?;
        let format_version: String = r.get(3)?;
        let importer_version: String = r.get(4)?;
        let producer: String = r.get(5)?;
        let producer_version: String = r.get(6)?;
        let mut files = Vec::new();
        let mut stmt=conn.prepare("SELECT ordinal,logical_path,sha256,sha_verified FROM source_files WHERE source_id=? ORDER BY ordinal")?;
        let mut sub = stmt.query([id])?;
        while let Some(f) = sub.next()? {
            let ordinal: i64 = f.get(0)?;
            let path: String = f.get(1)?;
            let hash: Vec<u8> = f.get(2)?;
            let verified: i64 = f.get(3)?;
            if ordinal != files.len() as i64
                || path.is_empty()
                || path.starts_with('/')
                || path.starts_with('\\')
                || path.contains('\\')
                || path.contains(':')
                || path
                    .split('/')
                    .any(|p| p == ".." || p == "." || p.is_empty())
                || files
                    .last()
                    .is_some_and(|old: &SourceFile| old.logical_path >= path)
                || (verified != 0 && verified != 1)
            {
                return Err(invalid("source file provenance/path"));
            }
            let sha256: [u8; 32] = hash.try_into().map_err(|_| invalid("source hash length"))?;
            files.push(SourceFile {
                logical_path: path,
                sha256,
            });
        }
        if files.is_empty() {
            return Err(invalid("source files missing"));
        }
        let mut conditions = Vec::new();
        let mut stmt=conn.prepare("SELECT ordinal,condition_key,condition_value FROM source_conditions WHERE source_id=? ORDER BY ordinal")?;
        let mut sub = stmt.query([id])?;
        while let Some(c) = sub.next()? {
            let ordinal: i64 = c.get(0)?;
            let key: String = c.get(1)?;
            let value: String = c.get(2)?;
            if ordinal != conditions.len() as i64
                || key.is_empty()
                || conditions
                    .last()
                    .is_some_and(|old: &(String, String)| old.0 >= key)
            {
                return Err(invalid("source conditions order"));
            }
            conditions.push((key, value));
        }
        let source_id = Shared::new(SourceIdentity {
            format: from_source(&format, other)?,
            format_version,
            files,
            importer_version,
            conditions,
        });
        let mut cap = BTreeMap::new();
        let mut stmt = conn.prepare(
            "SELECT dimension,state FROM import_capability WHERE source_id=? ORDER BY dimension",
        )?;
        let mut sub = stmt.query([id])?;
        while let Some(c) = sub.next()? {
            let dimension: String = c.get(0)?;
            let state: String = c.get(1)?;
            if cap.insert(dimension, from_cap(&state)?).is_some() {
                return Err(invalid("duplicate capability"));
            }
        }
        if cap.len() != 6
            || !["G", "N", "P", "S", "R", "M"]
                .iter()
                .all(|k| cap.contains_key(*k))
        {
            return Err(invalid("capability dimensions"));
        }
        let mut coverage = Vec::new();
        let mut stmt=conn.prepare("SELECT ordinal,covered_kind FROM capability_coverage WHERE source_id=? ORDER BY ordinal")?;
        let mut sub = stmt.query([id])?;
        while let Some(c) = sub.next()? {
            let ordinal: i64 = c.get(0)?;
            if ordinal != coverage.len() as i64 {
                return Err(invalid("capability coverage ordinal"));
            }
            coverage.push(c.get(1)?);
        }
        let capability = ImportCapability {
            geometry: cap["G"],
            net: cap["N"],
            pin_pad: cap["P"],
            stackup: cap["S"],
            reference: cap["R"],
            model: cap["M"],
            object_coverage: coverage,
        };
        if refs.insert(id, source_id.clone()).is_some() {
            return Err(invalid("duplicate source id"));
        }
        if out.len() >= 100_000 {
            return Err(invalid("source count budget"));
        }
        out.push(ImportSource {
            id: source_id,
            producer,
            producer_version,
            capability,
        });
    }
    let mut identities = BTreeSet::new();
    for src in &out {
        if !identities.insert((*src.id).clone()) {
            return Err(invalid("duplicate SourceIdentity"));
        }
    }
    Ok((out, refs))
}
fn read_objects(
    conn: &Connection,
    sources: &BTreeMap<i64, SourceId>,
) -> Result<BTreeMap<i64, ObjectMeta>> {
    let mut out = BTreeMap::new();
    let mut stmt = conn.prepare(
        "SELECT id,source_id,kind,source_object,part,source_object_type FROM objects ORDER BY id",
    )?;
    let mut rows = stmt.query([])?;
    while let Some(r) = rows.next()? {
        let id: i64 = r.get(0)?;
        let source_id: i64 = r.get(1)?;
        let kind: String = r.get(2)?;
        let source = sources
            .get(&source_id)
            .ok_or_else(|| invalid("unknown source identity"))?;
        let object_id = ObjectId {
            source: source.clone(),
            kind: from_kind(&kind)?,
            source_object: r.get(3)?,
            part: r.get(4)?,
        };
        let meta = ObjectMeta {
            id: object_id,
            provenance: GeometryProvenance {
                source_object_type: r.get(5)?,
                history: Vec::new(),
            },
        };
        if out.insert(id, meta).is_some() {
            return Err(invalid("object duplicate key"));
        }
        if out.len() > 1_000_000 {
            return Err(invalid("object budget"));
        }
    }
    let mut semantics = BTreeSet::new();
    for obj in out.values() {
        if !semantics.insert(obj.id.clone()) {
            return Err(invalid("duplicate semantic ObjectId"));
        }
    }
    for (key, m) in &mut out {
        let mut stmt=conn.prepare("SELECT ordinal,operation,quality,error_num_le,error_den_le,note FROM conversion_history WHERE object_id=? ORDER BY ordinal")?;
        let mut rows = stmt.query([*key])?;
        while let Some(r) = rows.next()? {
            let ordinal: i64 = r.get(0)?;
            if ordinal != m.provenance.history.len() as i64 {
                return Err(invalid("history ordinal"));
            }
            let operation: String = r.get(1)?;
            if operation.is_empty() {
                return Err(invalid("empty conversion operation"));
            }
            let quality: String = r.get(2)?;
            let n: Option<Vec<u8>> = r.get(3)?;
            let d: Option<Vec<u8>> = r.get(4)?;
            let max_error_m = match (n, d) {
                (None, None) => None,
                (Some(n), Some(d)) => Some(rational_pair(&n, &d)?),
                _ => return Err(invalid("history rational pair")),
            };
            m.provenance.history.push(ConversionStep {
                operation,
                quality: from_quality(&quality)?,
                max_error_m,
                note: r.get(5)?,
            });
        }
    }
    Ok(out)
}
fn read_material_property(
    conn: &Connection,
    id: i64,
    key: &str,
) -> Result<(String, Option<Vec<u8>>, Option<PropertySource>)> {
    let data=conn.query_row(
        "SELECT status,value_blob,document,revision,method,reference_frequency_hz,temperature_c FROM material_properties WHERE material_id=? AND property_key=?",
        (id,key),
        |r|Ok((r.get::<_,String>(0)?,r.get::<_,Option<Vec<u8>>>(1)?,
            r.get::<_,Option<String>>(2)?,r.get::<_,Option<String>>(3)?,
            r.get::<_,Option<String>>(4)?,r.get::<_,Option<f64>>(5)?,
            r.get::<_,Option<f64>>(6)?))
    ).optional()?.ok_or_else(||invalid("material property missing"))?;
    let (status, blob, doc, rev, method, freq, temp) = data;
    let source = match (doc, rev, method) {
        (None, None, None) if freq.is_none() && temp.is_none() => None,
        (Some(document), Some(revision), Some(method)) => Some(PropertySource {
            document,
            revision,
            method,
            reference_frequency_hz: freq,
            temperature_c: temp,
        }),
        _ => return Err(invalid("material evidence incomplete")),
    };
    check_source(source.as_ref())?;
    Ok((status, blob, source))
}
fn material_f64(conn: &Connection, id: i64, key: &str) -> Result<Property<f64>> {
    let (s, b, source) = read_material_property(conn, id, key)?;
    match (s.as_str(), b, source) {
        ("Missing", None, None) => Ok(Property::Missing),
        ("Assumed", Some(b), Some(source)) | ("Confirmed", Some(b), Some(source)) => {
            if b.len() != 8 {
                return Err(invalid("f64 size"));
            }
            let value = f64::from_le_bytes(b.try_into().expect("length"));
            if !value.is_finite() {
                return Err(invalid("material finite"));
            }
            if s == "Assumed" {
                Ok(Property::Assumed { value, source })
            } else {
                Ok(Property::Confirmed { value, source })
            }
        }
        _ => Err(invalid("material f64 status mismatch")),
    }
}
fn material_band(conn: &Connection, id: i64) -> Result<Property<FrequencyRange>> {
    let (s, b, source) = read_material_property(conn, id, "ValidBand")?;
    match (s.as_str(), b, source) {
        ("Missing", None, None) => Ok(Property::Missing),
        ("Assumed", Some(b), Some(source)) | ("Confirmed", Some(b), Some(source)) => {
            if b.len() != 16 {
                return Err(invalid("valid band length"));
            }
            let min_hz = f64::from_le_bytes(b[..8].try_into().expect("length"));
            let max_hz = f64::from_le_bytes(b[8..].try_into().expect("length"));
            if !min_hz.is_finite() || !max_hz.is_finite() || min_hz < 0.0 || min_hz >= max_hz {
                return Err(invalid("valid band"));
            }
            let value = FrequencyRange { min_hz, max_hz };
            if s == "Assumed" {
                Ok(Property::Assumed { value, source })
            } else {
                Ok(Property::Confirmed { value, source })
            }
        }
        _ => Err(invalid("band status mismatch")),
    }
}
fn read_fixed(
    conn: &Connection,
    model: &mut BoardModel,
    objects: &BTreeMap<i64, ObjectMeta>,
) -> Result<()> {
    let mut stmt=conn.prepare("SELECT object_id,unit_label,metre_num_le,metre_den_le,origin_x_num_le,origin_x_den_le,origin_y_num_le,origin_y_den_le,orientation,quarter_turn,pending_radians,mirror_x,reverse_source_layer_order FROM coordinate_frames ORDER BY object_id")?;
    let mut rows = stmt.query([])?;
    while let Some(r) = rows.next()? {
        let id: i64 = r.get(0)?;
        let m = meta(objects, id, ObjectKind::Frame)?;
        let metre = rational_pair(&r.get::<_, Vec<u8>>(2)?, &r.get::<_, Vec<u8>>(3)?)?;
        if !metre.positive() {
            return Err(invalid("frame scale"));
        }
        model.frames.push(CoordinateFrame {
            meta: m,
            unit_label: r.get(1)?,
            metres_per_tick: metre,
            origin_m: [
                rational_pair(&r.get::<_, Vec<u8>>(4)?, &r.get::<_, Vec<u8>>(5)?)?,
                rational_pair(&r.get::<_, Vec<u8>>(6)?, &r.get::<_, Vec<u8>>(7)?)?,
            ],
            orientation: from_sql_orientation(
                &r.get::<_, String>(8)?,
                r.get(9)?,
                r.get(10)?,
                r.get(11)?,
            )?,
            source_layer_order_reversed: bool_from(r.get(12)?)?,
        });
    }
    let mut stmt = conn.prepare("SELECT object_id,name FROM materials ORDER BY object_id")?;
    let mut rows = stmt.query([])?;
    while let Some(r) = rows.next()? {
        let id: i64 = r.get(0)?;
        model.materials.push(Material {
            meta: meta(objects, id, ObjectKind::Material)?,
            name: r.get(1)?,
            dk: material_f64(conn, id, "Dk")?,
            df: material_f64(conn, id, "Df")?,
            conductivity_s_per_m: material_f64(conn, id, "Conductivity")?,
            valid_band: material_band(conn, id)?,
        });
    }
    let mut stmt=conn.prepare("SELECT object_id,layer_order,name,layer_kind,material_id,z_status,z_num_le,z_den_le,thickness_status,thickness_num_le,thickness_den_le FROM layers ORDER BY layer_order")?;
    let mut rows = stmt.query([])?;
    while let Some(r) = rows.next()? {
        let id: i64 = r.get(0)?;
        let mat: Option<i64> = r.get(4)?;
        let material = mat.map(|n| lookup(objects, n)).transpose()?;
        if material
            .as_ref()
            .is_some_and(|x| x.kind != ObjectKind::Material)
        {
            return Err(invalid("layer material kind"));
        }
        let z_source = property_source(conn, id, "z_m")?;
        let t_source = property_source(conn, id, "thickness_m")?;
        model.stackup.layers.push(Layer {
            meta: meta(objects, id, ObjectKind::Layer)?,
            order: r.get::<_, u32>(1)?,
            name: r.get(2)?,
            kind: from_layer(&r.get::<_, String>(3)?)?,
            material,
            z_m: status_property(&r.get::<_, String>(5)?, r.get(6)?, r.get(7)?, z_source)?,
            thickness_m: status_property(&r.get::<_, String>(8)?, r.get(9)?, r.get(10)?, t_source)?,
        });
    }
    let mut stmt = conn.prepare("SELECT object_id,name,role FROM nets ORDER BY object_id")?;
    let mut rows = stmt.query([])?;
    while let Some(r) = rows.next()? {
        let id: i64 = r.get(0)?;
        model.nets.push(Net {
            meta: meta(objects, id, ObjectKind::Net)?,
            name: r.get(1)?,
            role: from_net(&r.get::<_, String>(2)?)?,
        });
    }
    Ok(())
}
fn check_relation(
    objects: &BTreeMap<i64, ObjectMeta>,
    id: Option<i64>,
    kind: ObjectKind,
) -> Result<Option<ObjectId>> {
    id.map(|id| {
        let x = lookup(objects, id)?;
        if x.kind != kind {
            return Err(invalid("relation object kind"));
        }
        Ok(x)
    })
    .transpose()
}
fn exact_reencode(
    model: &BoardModel,
    original: &GeometryBytesById,
) -> Result<()> {
    use crate::encoding::Geom;
    let mut count = 0usize;
    let mut check = |g: Geom<'_>| -> Result<()> {
        let (record, refs) = g.encoded()?;
        let expected = record.encode()?;
        let (actual, dict) = original
            .get(&g.meta().id)
            .ok_or_else(|| invalid("missing native geometry"))?;
        if *actual != expected || dict.len() != refs.entries.len() {
            return Err(invalid("geometry noncanonical data"));
        }
        for ((a, role), (b, want_role)) in dict.iter().zip(&refs.entries) {
            if a != b || role != want_role {
                return Err(invalid("geometry ref mismatch"));
            }
        }
        count += 1;
        Ok(())
    };
    for x in &model.traces {
        check(Geom::Trace(x))?;
    }
    for x in &model.arcs {
        check(Geom::Arc(x))?;
    }
    for x in &model.areas {
        check(Geom::Area(x))?;
        for c in &x.cutouts {
            check(Geom::Cutout(c))?;
        }
    }
    for x in &model.padstacks {
        check(Geom::Padstack(x))?;
    }
    for x in &model.pads {
        check(Geom::Pad(x))?;
    }
    for x in &model.drills {
        check(Geom::Drill(x))?;
    }
    for x in &model.vias {
        check(Geom::Via(x))?;
    }
    for x in &model.outlines {
        check(Geom::Outline(x))?;
        for c in &x.cutouts {
            check(Geom::Cutout(c))?;
        }
    }
    if count != original.len() {
        return Err(invalid("extra or dropped geometry"));
    }
    Ok(())
}
fn read_geometry(
    conn: &Connection,
    model: &mut BoardModel,
    objects: &BTreeMap<i64, ObjectMeta>,
) -> Result<()> {
    let mut original: GeometryBytesById = BTreeMap::new();
    let mut cutouts: BTreeMap<ObjectId, (Cutout, Option<ObjectId>)> = BTreeMap::new();
    let mut areas: Vec<PendingArea> = Vec::new();
    let mut outlines: Vec<(ObjectMeta, Boundary, Vec<ObjectId>)> = Vec::new();
    let mut stmt=conn.prepare("SELECT object_id,geometry_kind,codec_version,envelope,layer_id,net_id,parent_geometry_id,padstack_id,first_copper_order,last_copper_order,plating FROM geometry ORDER BY object_id")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let kind: i64 = row.get(1)?;
        let codec: i64 = row.get(2)?;
        if codec != 1 || !(1..=9).contains(&kind) {
            return Err(NativeError::UnsupportedRequired);
        }
        let expected_kind = match kind {
            1 => ObjectKind::Trace,
            2 => ObjectKind::Arc,
            3 => ObjectKind::Area,
            4 => ObjectKind::Cutout,
            5 => ObjectKind::Padstack,
            6 => ObjectKind::Pad,
            7 => ObjectKind::Via,
            8 => ObjectKind::Drill,
            9 => ObjectKind::Outline,
            _ => return Err(invalid("geometry kind")),
        };
        let m = meta(objects, id, expected_kind)?;
        let blob: Vec<u8> = row.get(3)?;
        let record = GeometryRecord::decode(&blob, kind as u16)?;
        let layer = check_relation(objects, row.get(4)?, ObjectKind::Layer)?;
        let net = check_relation(objects, row.get(5)?, ObjectKind::Net)?;
        // Parent is checked strictly below because Area and Outline share the FK.
        let parent_raw: Option<i64> = row.get(6)?;
        let parent = if let Some(n) = parent_raw {
            let obj = lookup(objects, n)?;
            if !matches!(obj.kind, ObjectKind::Area | ObjectKind::Outline) {
                return Err(invalid("cutout parent kind"));
            }
            Some(obj)
        } else {
            None
        };
        let padstack = check_relation(objects, row.get(7)?, ObjectKind::Padstack)?;
        let first: Option<i64> = row.get(8)?;
        let last: Option<i64> = row.get(9)?;
        let plating: Option<String> = row.get(10)?;
        if kind == 4 && parent.is_none() || kind != 4 && parent.is_some() {
            return Err(invalid("cutout parent relation"));
        }
        let mut dict = Vec::new();
        let mut refs_stmt=conn.prepare("SELECT ref_index,target_object_id,ref_role FROM geometry_refs WHERE geometry_id=? ORDER BY ref_index")?;
        let mut refs_rows = refs_stmt.query([id])?;
        while let Some(r) = refs_rows.next()? {
            let idx: i64 = r.get(0)?;
            if idx != dict.len() as i64 {
                return Err(invalid("noncontiguous geometry refs"));
            }
            dict.push((lookup(objects, r.get(1)?)?, r.get::<_, String>(2)?));
        }
        if dict.len() > 65_535 {
            return Err(invalid("geometry refs budget"));
        }
        let candidates = dict
            .iter()
            .map(|(id, role)| {
                let role: &'static str = match role.as_str() {
                    "Frame" => "Frame",
                    "Boundary" => "Boundary",
                    "Hole" => "Hole",
                    "Padstack" => "Padstack",
                    "Pad" => "Pad",
                    "Via" => "Via",
                    "Drill" => "Drill",
                    "Component" => "Component",
                    "Other" => "Other",
                    _ => "INVALID",
                };
                (id.clone(), role)
            })
            .collect();
        let refdict = reference_dict(candidates)?;
        if dict.len() != refdict.entries.len()
            || dict
                .iter()
                .zip(&refdict.entries)
                .any(|((a, role), (b, want))| a != b || role != want)
        {
            return Err(invalid("noncanonical reference order/role"));
        }
        let decoded = decode(&record, &refdict)?;
        match decoded {
            Decoded::Trace(centreline, width_m) => {
                if padstack.is_some() {
                    return Err(invalid("geometry kind forbids padstack relation"));
                }
                let layer = layer.ok_or_else(|| invalid("trace layer"))?;
                model.traces.push(Trace {
                    meta: m.clone(),
                    layer,
                    net,
                    centreline,
                    width_m,
                });
            }
            Decoded::Arc(centre, start, end, clockwise, width_m) => {
                if padstack.is_some() {
                    return Err(invalid("geometry kind forbids padstack relation"));
                }
                let layer = layer.ok_or_else(|| invalid("arc layer"))?;
                model.arcs.push(Arc {
                    meta: m.clone(),
                    layer,
                    net,
                    centre,
                    start,
                    end,
                    clockwise,
                    width_m,
                });
            }
            Decoded::Area(outer, cutout_ids) => {
                if padstack.is_some() {
                    return Err(invalid("geometry kind forbids padstack relation"));
                }
                let layer = layer.ok_or_else(|| invalid("area layer"))?;
                areas.push((m.clone(), outer, cutout_ids, layer, net));
            }
            Decoded::Cutout(boundary) => {
                if layer.is_some() || net.is_some() || padstack.is_some() {
                    return Err(invalid("cutout relations"));
                }
                if cutouts
                    .insert(
                        m.id.clone(),
                        (
                            Cutout {
                                meta: m.clone(),
                                boundary,
                            },
                            parent,
                        ),
                    )
                    .is_some()
                {
                    return Err(invalid("duplicate cutout"));
                }
            }
            Decoded::Padstack(entries) => {
                if layer.is_some() || net.is_some() || padstack.is_some() {
                    return Err(invalid("padstack relations"));
                }
                model.padstacks.push(Padstack {
                    meta: m.clone(),
                    entries,
                });
            }
            Decoded::Pad(position, orientation) => {
                if layer.is_some() {
                    return Err(invalid("pad or via forbids direct layer relation"));
                }
                let padstack = padstack.ok_or_else(|| invalid("missing padstack"))?;
                model.pads.push(Pad {
                    meta: m.clone(),
                    position,
                    orientation,
                    padstack,
                    net,
                });
            }
            Decoded::Via(position, via_kind, span, connected_layers) => {
                if layer.is_some() {
                    return Err(invalid("pad or via forbids direct layer relation"));
                }
                let padstack = padstack.ok_or_else(|| invalid("via padstack"))?;
                let drill = refdict
                    .entries
                    .iter()
                    .filter(|(id, role)| id.kind == ObjectKind::Drill && *role == "Drill")
                    .map(|(id, _)| id.clone())
                    .collect::<Vec<_>>();
                if drill.len() != 1 {
                    return Err(invalid("via drill reference"));
                }
                model.vias.push(Via {
                    meta: m.clone(),
                    position,
                    padstack,
                    drill: drill[0].clone(),
                    span,
                    kind: via_kind,
                    net,
                    connected_layers,
                });
            }
            Decoded::Drill(diameter_m, plated, span) => {
                if layer.is_some() || net.is_some() || padstack.is_some() {
                    return Err(invalid("drill relations"));
                }
                model.drills.push(Drill {
                    meta: m.clone(),
                    diameter_m,
                    plated,
                    span,
                });
            }
            Decoded::Outline(outer, cutout_ids) => {
                if layer.is_some() || net.is_some() || padstack.is_some() {
                    return Err(invalid("outline relations"));
                }
                outlines.push((m.clone(), outer, cutout_ids));
            }
        }
        match kind {
            7 | 8 => {
                let (f, l) = match (first, last) {
                    (Some(a), Some(b)) if a < b => (a, b),
                    _ => return Err(invalid("via/drill copper span")),
                };
                let span = match kind {
                    7 => &model.vias.last().expect("pushed via").span,
                    _ => &model.drills.last().expect("pushed drill").span,
                };
                let layer_order = |id: &ObjectId| -> Result<i64> {
                    model
                        .stackup
                        .layers
                        .iter()
                        .find(|x| x.meta.id == *id && x.kind == LayerKind::Copper)
                        .map(|x| i64::from(x.order))
                        .ok_or_else(|| invalid("span noncopper layer"))
                };
                if f != layer_order(&span.first_layer)? || l != layer_order(&span.last_layer)? {
                    return Err(invalid("SQL/BLOB span mismatch"));
                }
                if kind == 8 {
                    let plated = model.drills.last().expect("pushed drill").plated;
                    if plating.as_deref()
                        != Some(if plated {
                            "SourceDeclaredPlated"
                        } else {
                            "NonPlated"
                        })
                    {
                        return Err(invalid("drill plating status"));
                    }
                } else if plating.as_deref() != Some("Unknown") {
                    return Err(invalid("via must not assert verified plating"));
                }
            }
            _ if first.is_some() || last.is_some() || plating.is_some() => {
                return Err(invalid("unexpected span/plating"))
            }
            _ => {}
        }
        if original.insert(m.id.clone(), (blob, dict)).is_some() {
            return Err(invalid("duplicate geometry"));
        }
    }
    let mut seen = BTreeSet::new();
    for (m, outer, ids, layer, net) in areas {
        let mut nested = Vec::new();
        for id in ids {
            let (c, parent) = cutouts
                .get(&id)
                .ok_or_else(|| invalid("area cutout missing"))?;
            if parent.as_ref() != Some(&m.id) || !seen.insert(id) {
                return Err(invalid("area cutout parent/duplicate"));
            }
            nested.push(c.clone());
        }
        model.areas.push(CopperArea {
            meta: m,
            layer,
            net,
            outer,
            cutouts: nested,
        });
    }
    for (m, outer, ids) in outlines {
        let mut nested = Vec::new();
        for id in ids {
            let (c, parent) = cutouts
                .get(&id)
                .ok_or_else(|| invalid("outline cutout missing"))?;
            if parent.as_ref() != Some(&m.id) || !seen.insert(id) {
                return Err(invalid("outline cutout parent/duplicate"));
            }
            nested.push(c.clone());
        }
        model.outlines.push(Outline {
            meta: m,
            outer,
            cutouts: nested,
        });
    }
    if seen.len() != cutouts.len() {
        return Err(invalid("orphan cutout"));
    }
    exact_reencode(model, &original)?;
    Ok(())
}
fn read_components_pins(
    conn: &Connection,
    model: &mut BoardModel,
    objects: &BTreeMap<i64, ObjectMeta>,
) -> Result<()> {
    let mut s=conn.prepare("SELECT object_id,reference,placement_frame_id,x_ticks,y_ticks,orientation,quarter_turn,pending_radians,mirror_x,bottom_side FROM components ORDER BY object_id")?;
    let mut rows = s.query([])?;
    while let Some(r) = rows.next()? {
        let id: i64 = r.get(0)?;
        let frame = lookup(objects, r.get(2)?)?;
        if frame.kind != ObjectKind::Frame {
            return Err(invalid("component position frame"));
        }
        model.components.push(Component {
            meta: meta(objects, id, ObjectKind::Component)?,
            reference: r.get(1)?,
            placement: Placement {
                position: Point {
                    frame,
                    ticks: [r.get(3)?, r.get(4)?],
                },
                orientation: from_sql_orientation(
                    &r.get::<_, String>(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                )?,
                bottom_side: bool_from(r.get(9)?)?,
            },
        });
    }
    let mut s = conn.prepare(
        "SELECT object_id,component_id,pin_number,pad_id,net_id FROM pins ORDER BY object_id",
    )?;
    let mut rows = s.query([])?;
    while let Some(r) = rows.next()? {
        let id: i64 = r.get(0)?;
        let component = lookup(objects, r.get(1)?)?;
        let pad = lookup(objects, r.get(3)?)?;
        if component.kind != ObjectKind::Component || pad.kind != ObjectKind::Pad {
            return Err(invalid("pin component/pad"));
        }
        model.pins.push(Pin {
            meta: meta(objects, id, ObjectKind::Pin)?,
            component,
            number: r.get(2)?,
            pad,
            net: check_relation(objects, r.get(4)?, ObjectKind::Net)?,
        });
    }
    Ok(())
}
fn read_graph(
    conn: &Connection,
    model: &mut BoardModel,
    objects: &BTreeMap<i64, ObjectMeta>,
) -> Result<()> {
    let mut s=conn.prepare("SELECT object_id,conductor_id,layer_id,frame_id,x_ticks,y_ticks,witness_kind,trace_vertex_index,witness_evidence FROM connectivity_nodes ORDER BY object_id")?;
    let mut rows = s.query([])?;
    let mut positions = BTreeMap::new();
    while let Some(r) = rows.next()? {
        let id: i64 = r.get(0)?;
        let conductor = lookup(objects, r.get(1)?)?;
        let layer = lookup(objects, r.get(2)?)?;
        let frame = lookup(objects, r.get(3)?)?;
        if layer.kind != ObjectKind::Layer || frame.kind != ObjectKind::Frame {
            return Err(invalid("connectivity layer/frame"));
        }
        let point = Point {
            frame,
            ticks: [r.get(4)?, r.get(5)?],
        };
        let kind: String = r.get(6)?;
        let index: Option<i64> = r.get(7)?;
        let evidence: String = r.get(8)?;
        let site = match (kind.as_str(), index) {
            ("TraceVertex", Some(i)) if i >= 0 => ContactSite::TraceVertex(i as usize),
            ("TraceInterior", None) => ContactSite::TraceInterior(point.clone()),
            ("ArcStart", None) => ContactSite::ArcStart,
            ("ArcEnd", None) => ContactSite::ArcEnd,
            ("PadCentre", None) => ContactSite::PadCentre,
            ("ViaCentre", None) => ContactSite::ViaCentre,
            ("AreaWitness", None) => ContactSite::AreaWitness {
                point: point.clone(),
                evidence: evidence.clone(),
            },
            ("AreaInterior", None) => ContactSite::AreaInterior(point.clone()),
            _ => return Err(invalid("contact witness type")),
        };
        if kind != "AreaWitness" && !evidence.is_empty() {
            return Err(invalid("unexpected contact evidence"));
        }
        positions.insert(id, (point, kind));
        model.connectivity.nodes.push(ConnectivityNode {
            meta: meta(objects, id, ObjectKind::Node)?,
            conductor,
            layer,
            site,
        });
    }
    let mut s=conn.prepare("SELECT object_id,first_node,second_node,evidence_kind,evidence_text,witness_object_id,verified_state FROM connectivity_edges ORDER BY object_id")?;
    let mut rows = s.query([])?;
    while let Some(r) = rows.next()? {
        let id: i64 = r.get(0)?;
        let a = lookup(objects, r.get(1)?)?;
        let b = lookup(objects, r.get(2)?)?;
        if a.kind != ObjectKind::Node || b.kind != ObjectKind::Node {
            return Err(invalid("edge endpoint kind"));
        }
        let kind: String = r.get(3)?;
        let evidence_text: String = r.get(4)?;
        let witness: Option<i64> = r.get(5)?;
        let state: String = r.get(6)?;
        // The current Canonical Model does not expose a verified_state field.
        if state != "Unverified" {
            return Err(NativeError::UnsupportedRequired);
        }
        let evidence = match (kind.as_str(), witness) {
            ("SameLayerContact", None) if !evidence_text.is_empty() => {
                ConnectionEvidence::SameLayerContact {
                    evidence: evidence_text,
                }
            }
            ("ViaBarrel", Some(id)) if evidence_text.is_empty() => {
                let id = lookup(objects, id)?;
                if id.kind != ObjectKind::Via {
                    return Err(invalid("barrel witness kind"));
                }
                ConnectionEvidence::ViaBarrel { via: id }
            }
            ("AreaContinuity", Some(id)) if evidence_text.is_empty() => {
                let id = lookup(objects, id)?;
                if id.kind != ObjectKind::Area {
                    return Err(invalid("area witness kind"));
                }
                ConnectionEvidence::AreaContinuity { area: id }
            }
            _ => return Err(invalid("edge evidence mismatch")),
        };
        model.connectivity.edges.push(ConnectivityEdge {
            meta: meta(objects, id, ObjectKind::Edge)?,
            a,
            b,
            evidence,
        });
    }
    let identity_ids: BTreeMap<ObjectId, i64> = objects
        .iter()
        .map(|(id, meta)| (meta.id.clone(), *id))
        .collect();
    let contact_index = crate::write::ContactLookup::new(model);
    for n in &model.connectivity.nodes {
        let id = identity_ids
            .get(&n.meta.id)
            .copied()
            .ok_or_else(|| invalid("contact ID"))?;
        let (p, kind) = positions
            .get(&id)
            .ok_or_else(|| invalid("contact position"))?;
        let (want, want_kind, _, _) = contact_index.point(n)?;
        if want != *p || want_kind != kind {
            return Err(invalid("contact witness geometry mismatch"));
        }
    }
    Ok(())
}
fn validate_coverage(conn: &Connection, objects: &BTreeMap<i64, ObjectMeta>) -> Result<()> {
    // A row in objects must have exactly one corresponding typed payload.
    for (id, m) in objects {
        let table = match m.id.kind {
            ObjectKind::Frame => "coordinate_frames",
            ObjectKind::Material => "materials",
            ObjectKind::Layer => "layers",
            ObjectKind::Net => "nets",
            ObjectKind::Component => "components",
            ObjectKind::Pin => "pins",
            ObjectKind::Node => "connectivity_nodes",
            ObjectKind::Edge => "connectivity_edges",
            _ => "geometry",
        };
        let sql = format!("SELECT COUNT(*) FROM {table} WHERE object_id=?");
        let count: i64 = conn.query_row(&sql, [id], |r| r.get(0))?;
        if count != 1 {
            return Err(invalid("orphan or extra typed object"));
        }
    }
    let diagnostics_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM diagnostics", [], |r| r.get(0))?;
    if diagnostics_count != 0 {
        return Err(NativeError::UnsupportedRequired);
    }
    Ok(())
}
pub(crate) fn read_board(conn: &Connection) -> Result<LoadedBoard> {
    let meta: Vec<(i64, i64, i64, i64, String)> = {
        let mut stmt=conn.prepare("SELECT format_major,format_minor,min_reader_major,min_reader_minor,board_kind FROM file_meta")?;
        let mut rows = stmt.query([])?;
        let mut all = Vec::new();
        while let Some(r) = rows.next()? {
            all.push((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?));
        }
        all
    };
    if meta.len() != 1 {
        return Err(invalid("file_meta singleton"));
    }
    let (major, minor, min_major, min_minor, kind) = &meta[0];
    if *major > 1 {
        return Err(NativeError::FutureSchema);
    }
    if *min_major > 1 || *min_major == 1 && *min_minor > 0 {
        return Err(NativeError::UnsupportedRequired);
    }
    if *major != 1 || *minor < 0 {
        return Err(invalid("format version"));
    }
    let mut read_only = false;
    for table in ["features", "extensions"] {
        let sql = if table == "features" {
            "SELECT required FROM features"
        } else {
            "SELECT required FROM extensions"
        };
        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query([])?;
        while let Some(r) = rows.next()? {
            let required: i64 = r.get(0)?;
            if required == 1 {
                return Err(NativeError::UnsupportedRequired);
            }
            if required != 0 {
                return Err(invalid("invalid feature flag"));
            }
            read_only = true;
        }
    }
    let board_kind = match kind.as_str() {
        "Pcb" => BoardKind::Pcb,
        "BgaSubstrate" => BoardKind::BgaSubstrate,
        _ => return Err(invalid("board kind")),
    };
    let mut model = BoardModel::empty(board_kind);
    let (sources, source_ids) = read_source_ids(conn)?;
    model.sources = sources;
    // A verified-source claim has no Boolean field in the current Canonical model.
    // The file remains read-only rather than silently losing this evidence on Save As.
    let verified_files: i64 = conn.query_row(
        "SELECT COUNT(*) FROM source_files WHERE sha_verified=1",
        [],
        |row| row.get(0),
    )?;
    if verified_files != 0 {
        read_only = true;
    }
    let objects = read_objects(conn, &source_ids)?;
    read_fixed(conn, &mut model, &objects)?;
    read_geometry(conn, &mut model, &objects)?;
    read_components_pins(conn, &mut model, &objects)?;
    read_graph(conn, &mut model, &objects)?;
    validate_coverage(conn, &objects)?;
    if !crate::storage_valid(&model) {
        return Err(invalid("canonical model invalid"));
    }
    Ok(LoadedBoard { model, read_only })
}
