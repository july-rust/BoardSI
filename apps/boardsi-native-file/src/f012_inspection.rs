//! F_012 read-only, snapshot-bound Native inspection pages.
//! The existing Native Reader remains the sole structural authority.
use boardsi_model::{
    BoardModel, BoundarySegment, ObjectId, ObjectMeta, PadShape, Point, Property, Rational,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub(super) type ObjectPositions<'a> = BTreeMap<&'a ObjectId, (&'static str, usize)>;

fn sha_hex(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(64);
    for &byte in bytes {
        result.push(char::from(HEX[usize::from(byte >> 4)]));
        result.push(char::from(HEX[usize::from(byte & 15)]));
    }
    result
}

const MAX_PAGE: usize = 128;
const MAX_BYTES: usize = 60_000;
const MAX_ROW: usize = 32_000;

fn rational(value: Rational) -> Value {
    json!({"numerator": value.numerator().to_string(),
           "denominator": value.denominator().to_string()})
}

fn id(board: &BoardModel, value: &ObjectId) -> Value {
    // The source index resolves to the complete SourceIdentity in the same
    // accepted snapshot's sources page. Never identify by name/hash alone.
    json!({"source_index": board.sources.iter().position(|s| s.id == value.source),
           "kind": format!("{:?}", value.kind),
           "source_object": value.source_object, "part": value.part})
}

pub(super) fn artwork_ref(
    board: &BoardModel,
    category: &str,
    index: usize,
    object: &ObjectId,
) -> Value {
    json!({"category": category, "index": index, "id": id(board, object)})
}

fn meta(board: &BoardModel, value: &ObjectMeta) -> Value {
    json!({"id": id(board, &value.id),
           "source_object_type": value.provenance.source_object_type,
           "history_count": value.provenance.history.len()})
}

fn point(board: &BoardModel, value: &Point) -> Value {
    json!({"frame": id(board, &value.frame),
           "ticks": [value.ticks[0].to_string(), value.ticks[1].to_string()]})
}

// Do not manufacture a total from an incomplete or merely ordered stackup.
fn total_thickness(board: &BoardModel) -> Value {
    let calculate = || -> Option<(Rational, bool)> {
        let mut layers: Vec<_> = board.stackup.layers.iter().collect();
        layers.sort_by_key(|layer| layer.order);
        let first = *layers.first()?.z_m.value()?;
        let mut end = first;
        let mut assumed = false;
        for layer in layers {
            let z = *layer.z_m.value()?;
            let thickness = *layer.thickness_m.value()?;
            if z != end || !thickness.positive() {
                return None;
            }
            assumed |= matches!(layer.z_m, Property::Assumed { .. })
                || matches!(layer.thickness_m, Property::Assumed { .. });
            end = z.checked_add(thickness).ok()?;
        }
        Some((end.checked_add(first.checked_neg().ok()?).ok()?, assumed))
    };
    match calculate() {
        Some((value, assumed)) => {
            json!({"state":if assumed {"ASSUMED"} else {"CONFIRMED"},
                "value":rational(value),"basis":"derived from contiguous stored layer faces; not a measurement"})
        }
        None => {
            json!({"state":"UNAVAILABLE","reason":"missing, noncontiguous or nonrepresentable layer dimensions"})
        }
    }
}

fn segment(board: &BoardModel, value: &BoundarySegment) -> Value {
    match value {
        BoundarySegment::Line { start, end } => {
            json!({"kind":"Line","start":point(board,start),"end":point(board,end)})
        }
        BoundarySegment::Arc {
            centre,
            start,
            end,
            clockwise,
        } => {
            json!({"kind":"Arc","centre":point(board,centre),"start":point(board,start),
                   "end":point(board,end),"clockwise":clockwise})
        }
    }
}

fn length_m(
    board: &BoardModel,
    positions: Option<&ObjectPositions<'_>>,
    points: &[Point],
) -> Option<f64> {
    let frame = |id: &ObjectId| match positions {
        Some(index) => index
            .get(id)
            .filter(|(category, _)| *category == "frames")
            .and_then(|(_, index)| board.frames.get(*index)),
        None => board.frames.iter().find(|f| f.meta.id == *id),
    };
    let mut length = 0.0;
    for pair in points.windows(2) {
        let a = frame(&pair[0].frame)?;
        let b = frame(&pair[1].frame)?;
        let x = a.exact_m(&pair[0]).ok()?;
        let y = b.exact_m(&pair[1]).ok()?;
        let dx = y[0]
            .checked_add(x[0].checked_neg().ok()?)
            .ok()?
            .to_f64()
            .ok()?;
        let dy = y[1]
            .checked_add(x[1].checked_neg().ok()?)
            .ok()?
            .to_f64()
            .ok()?;
        length += dx.hypot(dy);
    }
    length.is_finite().then_some(length)
}

fn pad_shape(shape: &PadShape) -> Value {
    match shape {
        PadShape::Circle { diameter_m } => {
            json!({"kind":"Circle","diameter_m":rational(*diameter_m)})
        }
        PadShape::Rectangle { size_m } => {
            json!({"kind":"Rectangle","size_m":[rational(size_m[0]),rational(size_m[1])]})
        }
        PadShape::Annulus {
            outer_diameter_m,
            inner_diameter_m,
        } => {
            json!({"kind":"Annulus","outer_diameter_m":rational(*outer_diameter_m),"inner_diameter_m":rational(*inner_diameter_m)})
        }
        PadShape::Thermal {
            round,
            centre_size_m,
            outer_size_m,
            spoke_width_m,
            first_spoke_degrees,
            spoke_count,
        } => {
            json!({"kind":"Thermal","round":round,"centre_size_m":rational(*centre_size_m),"outer_size_m":rational(*outer_size_m),"spoke_width_m":rational(*spoke_width_m),"first_spoke_degrees":first_spoke_degrees,"spoke_count":spoke_count})
        }
        PadShape::Custom(boundary) => {
            json!({"kind":"Custom","segment_count":boundary.segments.len()})
        }
    }
}

fn prop<T>(property: &Property<T>, encode: impl Fn(&T) -> Value) -> Value {
    match property {
        Property::Missing => json!({"state": "MISSING"}),
        Property::Assumed { value, source } | Property::Confirmed { value, source } => {
            json!({
                "state": if matches!(property, Property::Assumed { .. }) {
                    "ASSUMED"
                } else {
                    "CONFIRMED"
                },
                "value": encode(value),
                "source": {"document": source.document, "revision": source.revision,
                           "method": source.method, "frequency_hz": source.reference_frequency_hz,
                           "temperature_c": source.temperature_c}
            })
        }
    }
}

fn bounded_page<T>(
    values: &[T],
    offset: usize,
    limit: usize,
    mut row: impl FnMut(&T) -> Value,
) -> Result<(usize, Vec<Value>), String> {
    if offset > values.len() {
        return Err("BSI_F012_OFFSET".into());
    }
    let mut used = 0;
    let mut rows = Vec::new();
    for item in values.iter().skip(offset).take(limit) {
        let value = row(item);
        let size = value.to_string().len() + 1;
        if size > MAX_ROW {
            return Err("BSI_F012_ROW_LIMIT".into());
        }
        if used + size > MAX_BYTES - 2048 {
            break;
        }
        used += size;
        rows.push(value);
    }
    Ok((values.len(), rows))
}

pub(super) fn query(
    path: &std::path::Path,
    digest: &str,
    request: &str,
    category: &str,
    offset: usize,
    limit: usize,
    selection: Option<&str>,
) -> Result<Value, String> {
    if digest.len() != 64
        || !digest.bytes().all(|b| b.is_ascii_hexdigit())
        || request.len() != 32
        || !request.bytes().all(|b| b.is_ascii_hexdigit())
        || !(1..=MAX_PAGE).contains(&limit)
    {
        return Err("BSI_F012_QUERY_INVALID".into());
    }
    if super::file_sha256(path)? != digest {
        return Err("BSI_SOURCE_CHANGED".into());
    }
    let loaded = boardsi_native::load(path).map_err(super::native_error)?;
    let result = query_loaded(
        (&loaded, None),
        digest,
        request,
        category,
        offset,
        limit,
        selection,
    )?;
    if super::file_sha256(path)? != digest {
        return Err("BSI_SOURCE_CHANGED".into());
    }
    Ok(result)
}

/// Query the already-validated, immutable model without opening a CAD or Native file.
pub(super) fn query_loaded(
    snapshot: (&boardsi_native::LoadedBoard, Option<&ObjectPositions<'_>>),
    digest: &str,
    request: &str,
    category: &str,
    offset: usize,
    limit: usize,
    selection: Option<&str>,
) -> Result<Value, String> {
    if digest.len() != 64
        || !digest.bytes().all(|b| b.is_ascii_hexdigit())
        || request.len() != 32
        || !request.bytes().all(|b| b.is_ascii_hexdigit())
        || !(1..=MAX_PAGE).contains(&limit)
    {
        return Err("BSI_F012_QUERY_INVALID".into());
    }
    let (loaded, positions) = snapshot;
    let b = loaded.model();
    let selection = selection
        .map(|raw| {
            if raw.len() > 16_000 {
                return Err("BSI_F012_SELECTION_LIMIT".to_string());
            }
            serde_json::from_str::<Value>(raw).map_err(|_| "BSI_F012_SELECTION_INVALID".to_string())
        })
        .transpose()?;
    if let Some(reference) = &selection {
        if reference["category"] != category
            || reference["index"].as_u64() != Some(offset as u64)
            || limit != 1
        {
            return Err("BSI_F012_SELECTION_INVALID".into());
        }
    }
    let (section, parent, child) = if let Some((section, raw)) = category.split_once(':') {
        let (raw, child) = raw
            .split_once(':')
            .map_or((raw, None), |(p, c)| (p, Some(c)));
        let parent = raw.parse::<usize>().map_err(|_| "BSI_F012_PARENT")?;
        let child = child
            .map(|c| c.parse::<usize>().map_err(|_| "BSI_F012_PARENT"))
            .transpose()?;
        if selection.is_some() {
            return Err("BSI_F012_SELECTION_INVALID".into());
        }
        (section, Some(parent), child)
    } else {
        (category, None, None)
    };
    if child.is_some()
        != matches!(
            section,
            "area-cutout-segments" | "outline-cutout-segments" | "custom-pad-segments"
        )
    {
        return Err("BSI_F012_PARENT".into());
    }
    if parent.is_some()
        && !matches!(
            section,
            "source-files"
                | "source-conditions"
                | "trace-points"
                | "padstack-entries"
                | "trace-history"
                | "via-history"
                | "pad-history"
                | "layer-history"
                | "material-history"
                | "arc-history"
                | "area-history"
                | "net-history"
                | "frame-history"
                | "drill-history"
                | "padstack-history"
                | "component-history"
                | "pin-history"
                | "outline-history"
                | "area-segments"
                | "outline-segments"
                | "area-cutouts"
                | "outline-cutouts"
                | "area-cutout-segments"
                | "outline-cutout-segments"
                | "custom-pad-segments"
        )
    {
        return Err("BSI_F012_CATEGORY".into());
    }
    let nested = || parent.ok_or("BSI_F012_PARENT");
    let page = match section {
        "source-files" => bounded_page(
            &b.sources.get(nested()?).ok_or("BSI_F012_PARENT")?.id.files,
            offset,
            limit,
            |f| json!({"logical_path":f.logical_path,"stored_sha256":sha_hex(&f.sha256),"source_reopened":false}),
        ),
        "source-conditions" => bounded_page(
            &b.sources
                .get(nested()?)
                .ok_or("BSI_F012_PARENT")?
                .id
                .conditions,
            offset,
            limit,
            |c| json!({"key":c.0,"value":c.1}),
        ),
        "trace-points" => bounded_page(
            &b.traces.get(nested()?).ok_or("BSI_F012_PARENT")?.centreline,
            offset,
            limit,
            |p| point(b, p),
        ),
        "padstack-entries" => bounded_page(
            &b.padstacks.get(nested()?).ok_or("BSI_F012_PARENT")?.entries,
            offset,
            limit,
            |e| json!({"layer":id(b,&e.layer),"role":format!("{:?}",e.role),"shape":pad_shape(&e.shape)}),
        ),
        "area-segments"
        | "outline-segments"
        | "area-cutout-segments"
        | "outline-cutout-segments"
        | "custom-pad-segments" => {
            let index = nested()?;
            let boundary = match section {
                "area-segments" => b.areas.get(index).map(|v| &v.outer),
                "outline-segments" => b.outlines.get(index).map(|v| &v.outer),
                "area-cutout-segments" => b
                    .areas
                    .get(index)
                    .and_then(|v| v.cutouts.get(child?))
                    .map(|v| &v.boundary),
                "outline-cutout-segments" => b
                    .outlines
                    .get(index)
                    .and_then(|v| v.cutouts.get(child?))
                    .map(|v| &v.boundary),
                _ => b
                    .padstacks
                    .get(index)
                    .and_then(|v| v.entries.get(child?))
                    .and_then(|v| match &v.shape {
                        PadShape::Custom(boundary) => Some(boundary),
                        _ => None,
                    }),
            }
            .ok_or("BSI_F012_PARENT")?;
            bounded_page(&boundary.segments, offset, limit, |v| segment(b, v))
        }
        "area-cutouts" | "outline-cutouts" => {
            let index = nested()?;
            let cutouts = if section == "area-cutouts" {
                b.areas.get(index).map(|v| &v.cutouts)
            } else {
                b.outlines.get(index).map(|v| &v.cutouts)
            }
            .ok_or("BSI_F012_PARENT")?;
            bounded_page(
                cutouts,
                offset,
                limit,
                |v| json!({"meta":meta(b,&v.meta),"segment_count":v.boundary.segments.len()}),
            )
        }
        "trace-history" | "via-history" | "pad-history" | "layer-history" | "material-history"
        | "arc-history" | "area-history" | "net-history" | "frame-history" | "drill-history"
        | "padstack-history" | "component-history" | "pin-history" | "outline-history" => {
            let index = nested()?;
            let object = match section {
                "trace-history" => b.traces.get(index).map(|v| &v.meta),
                "via-history" => b.vias.get(index).map(|v| &v.meta),
                "pad-history" => b.pads.get(index).map(|v| &v.meta),
                "layer-history" => b.stackup.layers.get(index).map(|v| &v.meta),
                "material-history" => b.materials.get(index).map(|v| &v.meta),
                "arc-history" => b.arcs.get(index).map(|v| &v.meta),
                "area-history" => b.areas.get(index).map(|v| &v.meta),
                "net-history" => b.nets.get(index).map(|v| &v.meta),
                "frame-history" => b.frames.get(index).map(|v| &v.meta),
                "drill-history" => b.drills.get(index).map(|v| &v.meta),
                "padstack-history" => b.padstacks.get(index).map(|v| &v.meta),
                "component-history" => b.components.get(index).map(|v| &v.meta),
                "pin-history" => b.pins.get(index).map(|v| &v.meta),
                _ => b.outlines.get(index).map(|v| &v.meta),
            }
            .ok_or("BSI_F012_PARENT")?;
            bounded_page(&object.provenance.history, offset, limit, |h| {
                json!({"operation":h.operation,"quality":format!("{:?}",h.quality),
                       "max_error_m":h.max_error_m.map(rational),"note":h.note})
            })
        }
        "board" => bounded_page(&[()], offset, limit, |_| {
            json!({"kind": format!("{:?}", b.kind), "schema": 1,
                   "read_only": loaded.is_read_only(),
                   "stackup_total_thickness_m": total_thickness(b),
                   "layers": b.stackup.layers.len(), "materials": b.materials.len(),
                   "nets": b.nets.len(), "components": b.components.len(),
                   "pins": b.pins.len(), "frames": b.frames.len(),
                   "traces": b.traces.len(), "arcs": b.arcs.len(),
                   "areas": b.areas.len(), "pads": b.pads.len(),
                   "vias": b.vias.len(), "drills": b.drills.len(),
                   "sources": b.sources.len(), "padstacks": b.padstacks.len(),
                   "outlines": b.outlines.len(),
                   "drill_observations": loaded.drill_evidence().map_or(0, |e| e.holes.len())})
        }),
        "sources" => bounded_page(&b.sources, offset, limit, |s| {
            json!({"format": format!("{:?}", s.id.format),
                   "format_version": s.id.format_version, "importer_version": s.id.importer_version,
                   "producer": s.producer, "producer_version": s.producer_version,
                   "file_count": s.id.files.len(), "files_preview_limited": s.id.files.len()>16,
                   "files": s.id.files.iter().take(16).map(|f| json!({"logical_path": f.logical_path,
                       "stored_sha256": sha_hex(&f.sha256)})).collect::<Vec<_>>(),
                   "condition_count": s.id.conditions.len(), "conditions_preview_limited": s.id.conditions.len()>16,
                   "conditions": s.id.conditions.iter().take(16).collect::<Vec<_>>(), "capability": {
                       "geometry": format!("{:?}", s.capability.geometry),
                       "net": format!("{:?}", s.capability.net),
                       "pin_pad": format!("{:?}", s.capability.pin_pad),
                       "stackup": format!("{:?}", s.capability.stackup),
                       "reference": format!("{:?}", s.capability.reference),
                       "model": format!("{:?}", s.capability.model),
                       "object_coverage": s.capability.object_coverage},
                   "source_reopened": false})
        }),
        "layers" => bounded_page(&b.stackup.layers, offset, limit, |l| {
            json!({"meta": meta(b, &l.meta), "name": l.name, "order": l.order,
                   "kind": format!("{:?}", l.kind),
                   "z_m": prop(&l.z_m, |v| rational(*v)),
                   "thickness_m": prop(&l.thickness_m, |v| rational(*v)),
                   "material": l.material.as_ref().map(|v| id(b, v))})
        }),
        "materials" => bounded_page(&b.materials, offset, limit, |m| {
            json!({"meta": meta(b, &m.meta), "name": m.name,
                   "dk": prop(&m.dk, |v| json!(v)), "df": prop(&m.df, |v| json!(v)),
                   "conductivity_s_per_m": prop(&m.conductivity_s_per_m, |v| json!(v)),
                   "valid_band_hz": prop(&m.valid_band, |v| json!([v.min_hz, v.max_hz]))})
        }),
        "nets" => bounded_page(
            &b.nets,
            offset,
            limit,
            |n| json!({"meta": meta(b, &n.meta), "name": n.name, "role": format!("{:?}", n.role)}),
        ),
        "frames" => bounded_page(&b.frames, offset, limit, |f| {
            json!({"meta": meta(b, &f.meta), "unit_label": f.unit_label,
                   "metres_per_tick": rational(f.metres_per_tick),
                   "origin_m": [rational(f.origin_m[0]), rational(f.origin_m[1])],
                   "orientation": format!("{:?}", f.orientation),
                   "source_layer_order_reversed":f.source_layer_order_reversed})
        }),
        "traces" => bounded_page(&b.traces, offset, limit, |t| {
            json!({"meta": meta(b, &t.meta), "layer": id(b, &t.layer),
                   "net": t.net.as_ref().map(|v| id(b, v)), "width_m": rational(t.width_m),
                   "point_count": t.centreline.len(),
                   "geometric_length_m_approx": length_m(b,positions,&t.centreline),
                   "length_basis":"polyline centreline only; binary64 approximation; not electrical length",
                   "start": t.centreline.first().map(|v| point(b, v)),
                   "end": t.centreline.last().map(|v| point(b, v))})
        }),
        "arcs" => bounded_page(&b.arcs, offset, limit, |a| {
            json!({"meta": meta(b, &a.meta), "layer": id(b, &a.layer),
                   "net": a.net.as_ref().map(|v| id(b, v)), "width_m": rational(a.width_m),
                   "centre": point(b, &a.centre), "start": point(b, &a.start),
                   "end": point(b, &a.end), "clockwise": a.clockwise})
        }),
        "padstacks" => bounded_page(
            &b.padstacks,
            offset,
            limit,
            |p| json!({"meta":meta(b,&p.meta),"entry_count":p.entries.len()}),
        ),
        "pads" => bounded_page(&b.pads, offset, limit, |p| {
            json!({"meta": meta(b, &p.meta), "position": point(b, &p.position),
                   "padstack": id(b, &p.padstack), "orientation": format!("{:?}", p.orientation),
                   "net": p.net.as_ref().map(|v| id(b, v))})
        }),
        "vias" => bounded_page(&b.vias, offset, limit, |v| {
            json!({"meta": meta(b, &v.meta), "position": point(b, &v.position),
                   "padstack": id(b, &v.padstack), "drill": id(b, &v.drill),
                   "kind": format!("{:?}", v.kind), "net": v.net.as_ref().map(|n| id(b, n)),
                   "drill_details":match positions {
                       Some(index) => index.get(&v.drill)
                           .filter(|(category,_)| *category=="drills")
                           .and_then(|(_,i)| b.drills.get(*i)),
                       None => b.drills.iter().find(|d|d.meta.id==v.drill),
                   }.map(|d|json!({"diameter_m":rational(d.diameter_m),"stored_plated":d.plated})),
                   "first_layer": id(b, &v.span.first_layer),
                   "last_layer": id(b, &v.span.last_layer),
                   "stored_connected_layers": v.connected_layers.iter().map(|l| id(b, l)).collect::<Vec<_>>()})
        }),
        "drills" => bounded_page(&b.drills, offset, limit, |d| {
            json!({"meta": meta(b, &d.meta), "diameter_m": rational(d.diameter_m),
                   "stored_plated": d.plated,
                   "first_layer": id(b, &d.span.first_layer),
                   "last_layer": id(b, &d.span.last_layer)})
        }),
        "areas" => bounded_page(&b.areas, offset, limit, |a| {
            json!({"meta": meta(b, &a.meta), "layer": id(b, &a.layer),
                   "net": a.net.as_ref().map(|n| id(b, n)),
                   "outer_segments": a.outer.segments.len(), "cutouts": a.cutouts.len()})
        }),
        "outlines" => bounded_page(
            &b.outlines,
            offset,
            limit,
            |o| json!({"meta":meta(b,&o.meta),"outer_segments":o.outer.segments.len(),"cutouts":o.cutouts.len()}),
        ),
        "components" => bounded_page(&b.components, offset, limit, |c| {
            json!({"meta": meta(b, &c.meta), "reference": c.reference,
                   "position": point(b, &c.placement.position), "bottom_side": c.placement.bottom_side,
                   "orientation": format!("{:?}", c.placement.orientation)})
        }),
        "pins" => bounded_page(&b.pins, offset, limit, |p| {
            json!({"meta": meta(b, &p.meta), "component": id(b, &p.component),
                   "number": p.number, "pad": id(b, &p.pad),
                   "net": p.net.as_ref().map(|n| id(b, n))})
        }),
        "drill-observations" => bounded_page(
            loaded
                .drill_evidence()
                .map_or(&[][..], |e| e.holes.as_slice()),
            offset,
            limit,
            |h| {
                json!({"centre_nm": [h.centre_nm[0].to_string(), h.centre_nm[1].to_string()],
                       "diameter_nm": h.diameter_nm.to_string(), "tool_id": h.tool_id,
                       "source_offset": h.source_offset.to_string(),
                       "plating": "UNKNOWN", "layer_span": "UNKNOWN", "alignment": "NOT_VERIFIED"})
            },
        ),
        _ => return Err("BSI_F012_CATEGORY".into()),
    }?;
    let (total, rows) = page;
    if let Some(reference) = &selection {
        let row = rows.first().ok_or("BSI_F012_SELECTION_MISSING")?;
        if category == "drill-observations" {
            if reference["source_offset"].as_str().is_none()
                || reference["source_offset"] != row["source_offset"]
                || reference["tool_id"] != row["tool_id"]
            {
                return Err("BSI_F012_SELECTION_MISMATCH".into());
            }
        } else if !reference["id"].is_object() || reference["id"] != row["meta"]["id"] {
            return Err("BSI_F012_SELECTION_MISMATCH".into());
        }
    }
    let next = offset + rows.len();
    let result = json!({
        "protocol": "BSI_NATIVE_INSPECT_V1", "request": request,
        "sha256": digest, "category": category, "offset": offset,
        "total": total, "next": if next < total { Some(next) } else { None },
        "selection": selection,
        "rows": rows, "physical_si": "NOT_RUN", "physical_contact": "NOT_VERIFIED"
    });
    if result.to_string().len() >= MAX_BYTES {
        return Err("BSI_F012_OUTPUT_LIMIT".into());
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_rational_strings_and_missing_state() {
        assert_eq!(
            rational(Rational::new(9_007_199_254_740_993, 7).unwrap()),
            json!({"numerator":"9007199254740993","denominator":"7"})
        );
        assert_eq!(
            prop(&Property::<Rational>::Missing, |v| rational(*v)),
            json!({"state":"MISSING"})
        );
    }

    #[test]
    fn page_bounds_do_not_skip_items() {
        let data = vec!["x".repeat(1000); 129];
        let (total, rows) = bounded_page(&data, 0, 128, |v| json!(v)).unwrap();
        assert_eq!(total, 129);
        assert!(!rows.is_empty() && rows.len() < 128);
        let (total, tail) = bounded_page(&data, 128, 128, |v| json!(v)).unwrap();
        assert_eq!(total, 129);
        assert_eq!(tail.len(), 1);
        assert!(bounded_page(&data, 130, 128, |v| json!(v)).is_err());
        assert!(bounded_page(&["x".repeat(MAX_ROW)], 0, 1, |v| json!(v)).is_err());
    }
}
