//! F_012 read-only, snapshot-bound Native inspection pages.
//! The existing Native Reader remains the sole structural authority.
use boardsi_model::{BoardModel, ObjectId, ObjectMeta, Point, Property, Rational};
use serde_json::{json, Value};

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
    let page = match category {
        "board" => bounded_page(&[()], offset, limit, |_| {
            json!({"kind": format!("{:?}", b.kind), "schema": 1,
                   "read_only": loaded.is_read_only(),
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
                   "files": s.id.files.iter().map(|f| json!({"logical_path": f.logical_path,
                       "stored_sha256": sha_hex(&f.sha256)})).collect::<Vec<_>>(),
                   "conditions": s.id.conditions, "capability": {
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
                   "orientation": format!("{:?}", f.orientation)})
        }),
        "traces" => bounded_page(&b.traces, offset, limit, |t| {
            json!({"meta": meta(b, &t.meta), "layer": id(b, &t.layer),
                   "net": t.net.as_ref().map(|v| id(b, v)), "width_m": rational(t.width_m),
                   "point_count": t.centreline.len(),
                   "start": t.centreline.first().map(|v| point(b, v)),
                   "end": t.centreline.last().map(|v| point(b, v))})
        }),
        "arcs" => bounded_page(&b.arcs, offset, limit, |a| {
            json!({"meta": meta(b, &a.meta), "layer": id(b, &a.layer),
                   "net": a.net.as_ref().map(|v| id(b, v)), "width_m": rational(a.width_m),
                   "centre": point(b, &a.centre), "start": point(b, &a.start),
                   "end": point(b, &a.end), "clockwise": a.clockwise})
        }),
        "pads" => bounded_page(&b.pads, offset, limit, |p| {
            json!({"meta": meta(b, &p.meta), "position": point(b, &p.position),
                   "padstack": id(b, &p.padstack), "orientation": format!("{:?}", p.orientation),
                   "net": p.net.as_ref().map(|v| id(b, v))})
        }),
        "vias" => bounded_page(&b.vias, offset, limit, |v| {
            json!({"meta": meta(b, &v.meta), "position": point(b, &v.position),
                   "padstack": id(b, &v.padstack), "drill": id(b, &v.drill),
                   "kind": format!("{:?}", v.kind), "net": v.net.as_ref().map(|n| id(b, n)),
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
    if super::file_sha256(path)? != digest {
        return Err("BSI_SOURCE_CHANGED".into());
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
