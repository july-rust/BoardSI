//! Restricted PADS Layout 9.4 MILS ROUTE importer for the developer preview.
//! This is a PARTIAL artwork view, never physical electrical connectivity or SI.
//! Grammar basis: Mentor PADS Layout ASCII Format Specification 9.4 pp. 51-55.
//! Only straight, same-copper-layer segments of equal vertex width are retained.
use boardsi_model::*;
use std::collections::BTreeMap;
use std::sync::Arc;

pub const MAX_FILE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_SEGMENTS: usize = 10_000;
const NM_PER_TICK: i64 = 25_400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub layer: u32,
    pub net: String,
    pub a: [i64; 2],
    pub b: [i64; 2],
    pub width_ticks: i64,
    pub source_line: usize,
}
#[derive(Debug)]
pub struct Artwork {
    pub segments: Vec<Segment>,
    pub max_layer: u32,
    pub skipped: u32,
}
fn parts(line: &str) -> Vec<&str> {
    line.split_ascii_whitespace().collect()
}
fn parse_i64(value: &str) -> Result<i64, &'static str> {
    value.parse::<i64>().map_err(|_| "INVALID_NUMBER")
}
fn mul_nm(n: i64) -> Result<i64, &'static str> {
    n.checked_mul(NM_PER_TICK).ok_or("COORDINATE_OVERFLOW")
}
pub fn parse(source: &[u8]) -> Result<Artwork, &'static str> {
    if source.len() > MAX_FILE_BYTES {
        return Err("INPUT_TOO_LARGE");
    }
    let source = std::str::from_utf8(source).map_err(|_| "NON_UTF8_UNSUPPORTED")?;
    let header = source.lines().next().ok_or("EMPTY_FILE")?;
    if !header.starts_with("!PADS-POWERPCB-V9.4-MILS!") {
        return Err("UNSUPPORTED_REVISION_OR_UNIT");
    }
    let mut max_layer = 0u32;
    let mut in_pcb = false;
    let mut in_route = false;
    let mut found_route = false;
    let mut found_end = false;
    let mut net: Option<&str> = None;
    // A new SIGNAL or pin pair is an explicit break in the physical route path.
    let mut previous: Option<([i64; 2], i64, i64)> = None;
    let mut segments = Vec::new();
    let mut skipped = 0u32;
    for (line_no, source_line) in source.lines().enumerate().skip(1) {
        if source_line.len() > 16_384 {
            return Err("OVERLONG_LINE");
        }
        let line = source_line.trim();
        if line.is_empty() || line.starts_with("REMARK ") {
            continue;
        }
        if line == "*END*" {
            found_end = true;
            break;
        }
        if line.starts_with('*') {
            if line == "*PCB*" {
                in_pcb = true;
                in_route = false;
            } else if line == "*ROUTE*" {
                in_route = true;
                in_pcb = false;
                found_route = true;
                net = None;
                previous = None;
            } else if in_route && line.starts_with("*SIGNAL* ") {
                let fields = parts(line);
                if fields.len() < 2
                    || fields[1].len() > 64
                    || !fields[1].bytes().all(|b| b.is_ascii_graphic() && b != b'|')
                {
                    return Err("INVALID_NET");
                }
                net = Some(fields[1]);
                previous = None;
            } else {
                in_pcb = false;
                in_route = false;
                net = None;
                previous = None;
            }
            continue;
        }
        if in_pcb {
            let fields = parts(line);
            for window in fields.windows(2) {
                if window[0] == "MAXLAYER" || window[0] == "MAXIMUMLAYER" {
                    let value = window[1]
                        .parse::<u32>()
                        .map_err(|_| "INVALID_LAYER_COUNT")?;
                    if !(2..=32).contains(&value) {
                        return Err("UNSUPPORTED_LAYER_COUNT");
                    }
                    if max_layer != 0 && max_layer != value {
                        return Err("CONFLICTING_LAYERS");
                    }
                    max_layer = value;
                }
            }
            continue;
        }
        if !in_route {
            continue;
        }
        let fields = parts(line);
        if fields.is_empty() {
            continue;
        }
        // The pin-pair header is not coordinate geometry.
        if fields.len() < 5 || fields[0].parse::<i64>().is_err() {
            previous = None;
            continue;
        }
        if net.is_none() {
            return Err("ROUTE_WITHOUT_SIGNAL");
        }
        let x = parse_i64(fields[0])?;
        let y = parse_i64(fields[1])?;
        let layer = parse_i64(fields[2])?;
        let width = parse_i64(fields[3])?;
        let flag = parse_i64(fields[4])?;
        // Any optional fields (via, arc, teardrop, reuse) are outside this subset.
        // Do not silently draw a chord for an unsupported arc or across a via.
        if width <= 0 || flag < 0 || flag > 32767 {
            return Err("INVALID_ROUTE_FIELD");
        }
        let p = [x, y];
        mul_nm(x)?;
        mul_nm(y)?;
        mul_nm(width)?;
        if fields.len() != 5 || flag & 0x1000 != 0 {
            skipped = skipped.saturating_add(1);
            previous = None;
            continue;
        }
        // 0 = unrouted, 65 = pin termination. Neither is a copper layer.
        let effective_layer = if layer == 65 {
            previous.map(|(_, prior, _)| prior).unwrap_or(0)
        } else {
            layer
        };
        if effective_layer <= 0 || max_layer == 0 || effective_layer > i64::from(max_layer) {
            skipped = skipped.saturating_add(1);
            previous = None;
            continue;
        }
        if let Some((before, prev_layer, prev_width)) = previous {
            if prev_layer == effective_layer && prev_width == width && before != p {
                if segments.len() >= MAX_SEGMENTS {
                    return Err("SEGMENT_LIMIT");
                }
                segments.push(Segment {
                    layer: effective_layer as u32,
                    net: net.unwrap_or_default().to_string(),
                    a: before,
                    b: p,
                    width_ticks: width,
                    source_line: line_no + 1,
                });
            } else {
                skipped = skipped.saturating_add(1);
            }
        }
        previous = if layer == 65 {
            None
        } else {
            Some((p, effective_layer, width))
        };
    }
    if !found_end {
        return Err("MISSING_END_MARKER");
    }
    if !found_route {
        return Err("MISSING_ROUTE_SECTION");
    }
    if max_layer == 0 {
        return Err("MISSING_LAYER_COUNT");
    }
    if segments.is_empty() {
        return Err("NO_SUPPORTED_TRACES");
    }
    Ok(Artwork {
        segments,
        max_layer,
        skipped,
    })
}
fn meta(source: &SourceId, kind: ObjectKind, object: &str) -> ObjectMeta {
    ObjectMeta {
        id: ObjectId::new(source, kind, object, ""),
        provenance: GeometryProvenance {
            source_object_type: "PADS Layout ROUTE (partial straight segments)".into(),
            history: vec![],
        },
    }
}
pub fn into_canonical(bytes: &[u8], parsed: &Artwork) -> Result<BoardModel, &'static str> {
    let sha = crate::digest::sha256(bytes);
    let source: SourceId = Arc::new(SourceIdentity {
        format: SourceFormat::PadsAsc,
        format_version: "V9.4-MILS".into(),
        files: vec![SourceFile {
            logical_path: "local/source.asc".into(),
            sha256: sha,
        }],
        importer_version: "preview-p2a-straight-v1".into(),
        conditions: vec![("route_subset".into(), "straight-same-layer".into())],
    });
    let mut board = BoardModel::empty(BoardKind::Pcb);
    board.sources.push(ImportSource {
        id: source.clone(),
        producer: "PADS Layout ASCII local importer".into(),
        producer_version: "V9.4-MILS".into(),
        capability: ImportCapability {
            geometry: CapabilityState::Partial,
            net: CapabilityState::Partial,
            pin_pad: CapabilityState::Missing,
            stackup: CapabilityState::Missing,
            reference: CapabilityState::Missing,
            model: CapabilityState::Missing,
            object_coverage: vec!["ROUTE straight copper segment only".into()],
        },
    });
    let frame_id = ObjectId::new(&source, ObjectKind::Frame, "xy", "");
    board.frames.push(CoordinateFrame {
        meta: meta(&source, ObjectKind::Frame, "xy"),
        unit_label: "PADS 9.4 MILS".into(),
        metres_per_tick: Rational::new(127, 5_000_000).map_err(|_| "FRAME_OVERFLOW")?,
        origin_m: [Rational::integer(0); 2],
        orientation: Orientation::Orthogonal {
            turn: QuarterTurn::Zero,
            mirror_x: false,
        },
        source_layer_order_reversed: false,
    });
    for n in 1..=parsed.max_layer {
        board.stackup.layers.push(Layer {
            meta: meta(&source, ObjectKind::Layer, &format!("layer-{n}")),
            name: if n == 1 {
                "Top".into()
            } else if n == parsed.max_layer {
                "Bottom".into()
            } else {
                format!("Inner {n}")
            },
            order: n - 1,
            kind: LayerKind::Copper,
            z_m: Property::Missing,
            thickness_m: Property::Missing,
            material: None,
        });
    }
    let mut nets = BTreeMap::<String, ObjectId>::new();
    for s in &parsed.segments {
        if !nets.contains_key(&s.net) {
            let m = meta(&source, ObjectKind::Net, &s.net);
            nets.insert(s.net.clone(), m.id.clone());
            board.nets.push(Net {
                meta: m,
                name: s.net.clone(),
                role: NetRole::Unknown,
            });
        }
        let id = format!("route-line-{}", s.source_line);
        board.traces.push(Trace {
            meta: meta(&source, ObjectKind::Trace, &id),
            layer: ObjectId::new(
                &source,
                ObjectKind::Layer,
                &format!("layer-{}", s.layer),
                "",
            ),
            net: nets.get(&s.net).cloned(),
            centreline: vec![
                Point {
                    frame: frame_id.clone(),
                    ticks: s.a,
                },
                Point {
                    frame: frame_id.clone(),
                    ticks: s.b,
                },
            ],
            width_m: Rational::new(i128::from(s.width_ticks) * 127, 5_000_000)
                .map_err(|_| "WIDTH_OVERFLOW")?,
        });
    }
    let report = board.validate();
    if report.has_errors() {
        return Err("CANONICAL_MODEL_INVALID");
    }
    Ok(board)
}
#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2 R 0 0\n100 200 1 10 0\n300 200 1 10 0\n300 400 1 10 0\n*END*\n";
    #[test]
    fn parses_exact_partial_artwork_with_canonical_links() {
        let parsed = parse(SAMPLE.as_bytes()).unwrap();
        assert_eq!(parsed.segments.len(), 2);
        assert_eq!(parsed.segments[0].a, [100, 200]);
        assert_eq!(mul_nm(parsed.segments[0].width_ticks).unwrap(), 254_000);
        let model = into_canonical(SAMPLE.as_bytes(), &parsed).unwrap();
        assert_eq!(model.traces.len(), 2);
        assert_eq!(model.stackup.layers.len(), 4);
        assert_eq!(
            model.sources[0].capability.geometry,
            CapabilityState::Partial
        );
        assert_eq!(
            model.sources[0].id.files[0].sha256,
            crate::digest::sha256(SAMPLE.as_bytes())
        );
    }
    #[test]
    fn rejects_unsupported_or_corrupt_and_never_draws_arc_chord() {
        assert_eq!(
            parse(SAMPLE.replace("V9.4-MILS", "V9.5-MM").as_bytes()).unwrap_err(),
            "UNSUPPORTED_REVISION_OR_UNIT"
        );
        assert_eq!(
            parse(SAMPLE.replace("*END*", "").as_bytes()).unwrap_err(),
            "MISSING_END_MARKER"
        );
        assert_eq!(
            parse(
                SAMPLE
                    .replace("300 200 1 10 0", "300 200 1 10 4096 CW")
                    .as_bytes()
            )
            .unwrap_err(),
            "NO_SUPPORTED_TRACES"
        );
        assert_eq!(
            parse(
                SAMPLE
                    .replace("300 200 1 10 0", "300 200 0 10 0")
                    .as_bytes()
            )
            .unwrap_err(),
            "NO_SUPPORTED_TRACES"
        );
    }
}
