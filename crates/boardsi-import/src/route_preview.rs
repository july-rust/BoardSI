//! Restricted PADS Layout 9.4 MILS ROUTE importer for the developer preview.
//! This is a PARTIAL artwork view, never physical electrical connectivity or SI.
//! Grammar basis: Mentor PADS Layout ASCII Format Specification 9.4 pp. 51-55.
//! Only straight, same-copper-layer segments of equal vertex width are retained.
use boardsi_model::*;
use std::collections::BTreeMap;
use std::sync::Arc;

pub const MAX_FILE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_SEGMENTS: usize = 10_000;
// One canonical tick is 0.01 mil = exactly 254 nm.
const NM_PER_TICK: i64 = 254;
// PADS 9.4 MILS source-domain limit is +/-56,000 mil (exactly +/-5,600,000 ticks).
const MAX_COORD_TICKS: i64 = 5_600_000;
const SUPPORTED_STRAIGHT_FLAGS: i64 = 0x0080 | 0x0200 | 0x0400;

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
pub struct ViaInstance {
    pub name: String,
    pub net: String,
    pub point: [i64; 2],
    pub diameter_ticks: i64,
    pub drill_ticks: i64,
    pub first_layer: u32,
    pub last_layer: u32,
    pub inner_pad: bool,
    pub source_line: usize,
}
#[derive(Debug)]
pub struct Artwork {
    pub vias: Vec<ViaInstance>,
    pub segments: Vec<Segment>,
    pub max_layer: u32,
    pub skipped: u32,
}
#[derive(Clone, Copy)]
struct SimpleViaDef {
    drill_ticks: i64,
    diameter_ticks: i64,
    inner_pad: bool,
}
// Deliberately narrow PADS 9.4 subset: only two or three explicit equal
// round conductive pads (-2 = top, 0 = bottom, optional -1 = interior).
// No blind/buried/microvia, thermal, antipad or implicit pad geometry.
fn scan_simple_via_defs(source: &str) -> BTreeMap<String, SimpleViaDef> {
    let lines: Vec<&str> = source.lines().collect();
    let mut definitions = BTreeMap::new();
    let mut in_via = false;
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        let keyword = line.split_ascii_whitespace().next().unwrap_or("");
        // Like the actual ROUTE parser, stop at the first database terminator.
        // Subsequent bytes are not part of the PADS board and must not define
        // geometry referenced by records before *END*.
        if keyword == "*END*" {
            break;
        }
        // A remark is not a section boundary; preserve VIA mode.
        if keyword == "*REMARK*" {
            i += 1;
            continue;
        }
        if line.starts_with('*') {
            in_via = keyword == "*VIA*";
            i += 1;
            continue;
        }
        if !in_via || line.is_empty() || line.starts_with("REMARK ") {
            i += 1;
            continue;
        }
        let header = parts(line);
        if header.len() != 3
            || !header[0].is_ascii()
            || header[0].len() > 64
            || header[0].contains('|')
        {
            i += 1;
            continue;
        }
        let (Ok(drill), Ok(count)) = (parse_fixed(header[1]), header[2].parse::<usize>()) else {
            i += 1;
            continue;
        };
        if !(2..=3).contains(&count) || i + count >= lines.len() {
            i += 1;
            continue;
        }
        let mut top = None;
        let mut bottom = None;
        let mut interior = None;
        let mut valid = drill > 0 && drill <= MAX_COORD_TICKS;
        for entry in &lines[i + 1..=i + count] {
            let row = parts(entry.trim());
            if row.len() != 3 || row[2] != "R" {
                valid = false;
                continue;
            }
            let (Ok(level), Ok(diameter)) = (parse_i64(row[0]), parse_fixed(row[1])) else {
                valid = false;
                continue;
            };
            if diameter <= drill || diameter > MAX_COORD_TICKS {
                valid = false;
            }
            match level {
                -2 if top.replace(diameter).is_none() => {}
                -1 if interior.replace(diameter).is_none() => {}
                0 if bottom.replace(diameter).is_none() => {}
                _ => valid = false,
            }
        }
        if valid && top.is_some() && top == bottom && (count == 2 || interior == top) {
            definitions.insert(
                header[0].to_string(),
                SimpleViaDef {
                    drill_ticks: drill,
                    diameter_ticks: top.unwrap_or_default(),
                    inner_pad: interior.is_some(),
                },
            );
        }
        i += count + 1;
    }
    definitions
}
fn parts(line: &str) -> Vec<&str> {
    line.split_ascii_whitespace().collect()
}
fn parse_i64(value: &str) -> Result<i64, &'static str> {
    value.parse::<i64>().map_err(|_| "INVALID_NUMBER")
}
// PADS 9.4 MILS coordinates are decimal, not integer-only. Preserve
// hundredth-mil precision without f64 rounding or truncation.
fn parse_fixed(value: &str) -> Result<i64, &'static str> {
    let (negative, digits) = if let Some(rest) = value.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = value.strip_prefix('+') {
        (false, rest)
    } else {
        (false, value)
    };
    let (whole, fraction) = match digits.split_once('.') {
        Some((whole, fraction)) => {
            if fraction.is_empty() || fraction.len() > 2 {
                return Err("UNSUPPORTED_PRECISION");
            }
            (whole, fraction)
        }
        None => (digits, ""),
    };
    if whole.is_empty()
        || !whole.bytes().all(|c| c.is_ascii_digit())
        || !fraction.bytes().all(|c| c.is_ascii_digit())
    {
        return Err("INVALID_NUMBER");
    }
    let base = whole.parse::<i64>().map_err(|_| "INVALID_NUMBER")?;
    let cents = match fraction.len() {
        0 => 0,
        1 => fraction.parse::<i64>().map_err(|_| "INVALID_NUMBER")? * 10,
        2 => fraction.parse::<i64>().map_err(|_| "INVALID_NUMBER")?,
        _ => return Err("UNSUPPORTED_PRECISION"),
    };
    let ticks = base
        .checked_mul(100)
        .and_then(|n| n.checked_add(cents))
        .ok_or("COORDINATE_OVERFLOW")?;
    if negative {
        ticks.checked_neg().ok_or("COORDINATE_OVERFLOW")
    } else {
        Ok(ticks)
    }
}
fn pin_token(value: &str) -> bool {
    let Some((reference, pin)) = value.rsplit_once('.') else {
        return false;
    };
    !reference.is_empty()
        && reference.len() <= 15
        && !pin.is_empty()
        && pin.len() <= 7
        && reference
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_')
        && pin.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
}
fn pin_pair(fields: &[&str]) -> bool {
    // PADS 9.4 Layout ASCII spec, ROUTE Definition (p. 52): the ROUTE
    // pin-pair record has EXACTLY two REF.PIN tokens. The 5-field reuse
    // metadata grammar belongs to the separate Connection section and must
    // not accidentally qualify malformed ROUTE records as copper paths.
    fields.len() == 2 && pin_token(fields[0]) && pin_token(fields[1])
}
fn mul_nm(n: i64) -> Result<i64, &'static str> {
    n.checked_mul(NM_PER_TICK).ok_or("COORDINATE_OVERFLOW")
}
pub fn parse(source: &[u8]) -> Result<Artwork, &'static str> {
    if source.len() > MAX_FILE_BYTES {
        return Err("INPUT_TOO_LARGE");
    }
    // Header controls are ASCII even when ignored CAD sections use CP1252.
    // Accept only the documented V9.4 MILS/CP1252 source subset; never infer
    // that a different declared code page has been decoded correctly.
    let first = source
        .split(|&byte| byte == b'\n')
        .next()
        .ok_or("EMPTY_FILE")?;
    let header = std::str::from_utf8(first).map_err(|_| "INVALID_HEADER_ENCODING")?;
    let tag = header.split_ascii_whitespace().next().unwrap_or("");
    if !matches!(
        tag,
        "!PADS-POWERPCB-V9.4-MILS!"
            | "!PADS-POWERPCB-V9.4-MILS-CP1252!"
            | "!PADS-POWERPCB-V9.4-MILS-250L!"
            | "!PADS-POWERPCB-V9.4-MILS-250L-CP1252!"
    ) {
        return Err("UNSUPPORTED_REVISION_OR_UNIT");
    }
    // Do not decode high bytes into route grammar: the supported subset has
    // ASCII-only route/pin/net tokens. Non-UTF8 CP1252 bytes elsewhere are
    // explicitly ignored, never mapped into copper geometry.
    let source = String::from_utf8_lossy(source);
    let via_defs = scan_simple_via_defs(&source);
    let mut vias = Vec::new();
    let mut max_layer = 0u32;
    let mut in_pcb = false;
    let mut in_route = false;
    let mut found_route = false;
    let mut found_end = false;
    let mut net: Option<&str> = None;
    let mut have_pin_pair = false;
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
        let keyword = line.split_ascii_whitespace().next().unwrap_or("");
        if keyword == "*REMARK*" {
            continue;
        }
        if keyword == "*END*" {
            found_end = true;
            break;
        }
        if line.starts_with('*') {
            if keyword == "*PCB*" {
                in_pcb = true;
                in_route = false;
            } else if keyword == "*ROUTE*" {
                in_route = true;
                in_pcb = false;
                found_route = true;
                net = None;
                have_pin_pair = false;
                previous = None;
            } else if in_route && keyword == "*SIGNAL*" {
                let fields = parts(line);
                if fields.len() < 2
                    || fields[1].len() > 64
                    || !fields[1].bytes().all(|b| b.is_ascii_graphic() && b != b'|')
                {
                    return Err("INVALID_NET");
                }
                net = Some(fields[1]);
                have_pin_pair = false;
                previous = None;
            } else {
                in_pcb = false;
                in_route = false;
                net = None;
                have_pin_pair = false;
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
                    if !(1..=64).contains(&value) {
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
        if !line.is_ascii() {
            return Err("NON_ASCII_ROUTE_UNSUPPORTED");
        }
        let fields = parts(line);
        if fields.is_empty() {
            continue;
        }
        if pin_pair(&fields) {
            if net.is_none() {
                return Err("PIN_PAIR_WITHOUT_SIGNAL");
            }
            have_pin_pair = true;
            previous = None;
            continue;
        }
        // Coordinate tokens may contain '.', so integer probing is unsafe:
        // 100.25 must never be silently mistaken for a pin-pair boundary.
        let numeric_start = fields[0]
            .as_bytes()
            .first()
            .is_some_and(|c| c.is_ascii_digit() || matches!(*c, b'-' | b'+' | b'.'));
        if !numeric_start {
            return Err("INVALID_PIN_PAIR_OR_ROUTE_RECORD");
        }
        if net.is_none() {
            return Err("ROUTE_WITHOUT_SIGNAL");
        }
        if !have_pin_pair {
            return Err("MISSING_PIN_PAIR");
        }
        if fields.len() < 5 {
            return Err("INVALID_ROUTE_RECORD");
        }
        let x = parse_fixed(fields[0])?;
        let y = parse_fixed(fields[1])?;
        let layer = parse_i64(fields[2])?;
        let width = parse_fixed(fields[3])?;
        let flag = parse_i64(fields[4])?;
        if !(-MAX_COORD_TICKS..=MAX_COORD_TICKS).contains(&x)
            || !(-MAX_COORD_TICKS..=MAX_COORD_TICKS).contains(&y)
        {
            return Err("COORDINATE_OUT_OF_RANGE");
        }
        // PADS V9.4 MILS source-domain width must be positive and <= 56000 mil.
        // Reject it BEFORE Canonical/Qt so a malformed trace cannot be drawn.
        if !(1..=MAX_COORD_TICKS).contains(&width) {
            return Err("WIDTH_OUT_OF_RANGE");
        }
        if !(0..=32767).contains(&flag) {
            return Err("INVALID_ROUTE_FIELD");
        }
        let p = [x, y];
        mul_nm(x)?;
        mul_nm(y)?;
        mul_nm(width)?;
        // A named via is a separate placement at this exact route vertex,
        // NOT proof of electrical contact with other copper layers.
        if fields.len() == 6 && flag & !SUPPORTED_STRAIGHT_FLAGS == 0 {
            if let Some(definition) = via_defs.get(fields[5]) {
                if (1..=i64::from(max_layer)).contains(&layer) && max_layer >= 2 {
                    if vias.len() >= MAX_SEGMENTS {
                        return Err("VIA_LIMIT");
                    }
                    // The explicit via vertex terminates the incoming copper on
                    // the preceding layer and begins outgoing copper on the
                    // newly declared layer. Never connect unrelated pin pairs.
                    if let Some((before, old_layer, old_width)) = previous {
                        if old_width == width && before != p {
                            if segments.len() >= MAX_SEGMENTS {
                                return Err("SEGMENT_LIMIT");
                            }
                            segments.push(Segment {
                                layer: old_layer as u32,
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
                    vias.push(ViaInstance {
                        name: fields[5].to_string(),
                        net: net.unwrap_or_default().to_string(),
                        point: p,
                        diameter_ticks: definition.diameter_ticks,
                        drill_ticks: definition.drill_ticks,
                        first_layer: 1,
                        last_layer: max_layer,
                        inner_pad: definition.inner_pad,
                        source_line: line_no + 1,
                    });
                    previous = Some((p, layer, width));
                    continue;
                }
            }
        }
        // Arc/miter/via extensions never become fictitious straight chords.
        if fields.len() != 5 || flag & !SUPPORTED_STRAIGHT_FLAGS != 0 {
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
    if segments.is_empty() && vias.is_empty() {
        return Err("NO_SUPPORTED_TRACES");
    }
    Ok(Artwork {
        vias,
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
        importer_version: "preview-p2a-straight-v2-centimil".into(),
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
        metres_per_tick: Rational::new(127, 500_000_000).map_err(|_| "FRAME_OVERFLOW")?,
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
    for via in &parsed.vias {
        if !nets.contains_key(&via.net) {
            let m = meta(&source, ObjectKind::Net, &via.net);
            nets.insert(via.net.clone(), m.id.clone());
            board.nets.push(Net {
                meta: m,
                name: via.net.clone(),
                role: NetRole::Unknown,
            });
        }
    }
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
            width_m: Rational::new(i128::from(s.width_ticks) * 127, 500_000_000)
                .map_err(|_| "WIDTH_OVERFLOW")?,
        });
    }
    for via in &parsed.vias {
        let start = ObjectId::new(
            &source,
            ObjectKind::Layer,
            &format!("layer-{}", via.first_layer),
            "",
        );
        let end = ObjectId::new(
            &source,
            ObjectKind::Layer,
            &format!("layer-{}", via.last_layer),
            "",
        );
        let span = ViaSpan {
            first_layer: start.clone(),
            last_layer: end.clone(),
        };
        let padstack_key = format!("simple-via-{}", via.name);
        let padstack_id = ObjectId::new(&source, ObjectKind::Padstack, &padstack_key, "");
        let drill_key = format!("simple-via-drill-{}", via.name);
        let drill_id = ObjectId::new(&source, ObjectKind::Drill, &drill_key, "");
        if !board.padstacks.iter().any(|p| p.meta.id == padstack_id) {
            let diameter_m = Rational::new(i128::from(via.diameter_ticks) * 127, 500_000_000)
                .map_err(|_| "VIA_DIMENSION_OVERFLOW")?;
            let mut entries = vec![
                PadstackLayer {
                    layer: start.clone(),
                    role: PadRole::Conductive,
                    shape: PadShape::Circle { diameter_m },
                },
                PadstackLayer {
                    layer: end.clone(),
                    role: PadRole::Conductive,
                    shape: PadShape::Circle { diameter_m },
                },
            ];
            if via.inner_pad {
                for n in 2..via.last_layer {
                    entries.push(PadstackLayer {
                        layer: ObjectId::new(&source, ObjectKind::Layer, &format!("layer-{n}"), ""),
                        role: PadRole::Conductive,
                        shape: PadShape::Circle { diameter_m },
                    });
                }
            }
            board.padstacks.push(Padstack {
                meta: meta(&source, ObjectKind::Padstack, &padstack_key),
                entries,
            });
            board.drills.push(Drill {
                meta: meta(&source, ObjectKind::Drill, &drill_key),
                diameter_m: Rational::new(i128::from(via.drill_ticks) * 127, 500_000_000)
                    .map_err(|_| "VIA_DIMENSION_OVERFLOW")?,
                plated: true,
                span: span.clone(),
            });
        }
        board.vias.push(Via {
            meta: meta(
                &source,
                ObjectKind::Via,
                &format!("via-line-{}", via.source_line),
            ),
            position: Point {
                frame: frame_id.clone(),
                ticks: via.point,
            },
            padstack: padstack_id,
            drill: drill_id,
            span,
            kind: ViaKind::Through,
            net: nets.get(&via.net).cloned(),
            // Never infer electrical contacts solely from same net or drill span.
            connected_layers: vec![],
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
    const SAMPLE: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n300 200 1 10 0\n300 400 1 10 0\n*END*\n";
    #[test]
    fn parses_exact_partial_artwork_with_canonical_links() {
        let parsed = parse(SAMPLE.as_bytes()).unwrap();
        assert_eq!(parsed.segments.len(), 2);
        assert_eq!(parsed.segments[0].a, [10_000, 20_000]);
        assert_eq!(mul_nm(parsed.segments[0].width_ticks).unwrap(), 254_000);
        assert_eq!(parsed.segments[0].width_ticks, 1000);
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
    #[test]
    fn parses_annotated_controls_remarks_and_exact_signed_decimal_values() {
        let sample = SAMPLE
            .replace("*PCB*", "*PCB* GENERAL PARAMETERS OF THE PCB DESIGN")
            .replace("*ROUTE*", "*ROUTE* ROUTE DEFINITIONS")
            .replace("MAXLAYER 4", "*REMARK* comment in PCB\nMAXLAYER 4")
            .replace(
                "*SIGNAL* NET_A",
                "*REMARK* comment in route\n*SIGNAL* NET_A",
            )
            .replace("100 200 1 10 0", "-100.25 200.01 1 10.50 0")
            .replace("300 200 1 10 0", "300.75 200.01 1 10.50 0")
            .replace("300 400 1 10 0", "300.75 400.50 1 10.50 0");
        let art = parse(sample.as_bytes()).unwrap();
        assert_eq!(art.segments.len(), 2);
        assert_eq!(art.segments[0].a, [-10_025, 20_001]);
        assert_eq!(art.segments[0].b, [30_075, 20_001]);
        assert_eq!(art.segments[0].width_ticks, 1050);
        assert_eq!(mul_nm(art.segments[0].a[0]).unwrap(), -2_546_350);
        let board = into_canonical(sample.as_bytes(), &art).unwrap();
        assert!(!board.validate().has_errors());
        assert_eq!(
            board.frames[0].metres_per_tick,
            Rational::new(127, 500_000_000).unwrap()
        );
        assert_eq!(board.traces.len(), 2);
    }
    #[test]
    fn fail_closed_on_decimal_precision_and_missing_or_corrupt_pin_pair() {
        let bad_precision = SAMPLE.replace("300 200 1 10 0", "300.001 200 1 10 0");
        assert_eq!(
            parse(bad_precision.as_bytes()).unwrap_err(),
            "UNSUPPORTED_PRECISION"
        );
        let no_pair = SAMPLE.replace("U1.1 U2.2\n", "");
        assert_eq!(parse(no_pair.as_bytes()).unwrap_err(), "MISSING_PIN_PAIR");
        let bad_pair = SAMPLE.replace("U1.1 U2.2", "U1.1 GARBAGE R 0 0");
        assert_eq!(
            parse(bad_pair.as_bytes()).unwrap_err(),
            "INVALID_PIN_PAIR_OR_ROUTE_RECORD"
        );
        let over = SAMPLE.replace("300 200 1 10 0", "92233720368547758.08 200 1 10 0");
        assert_eq!(parse(over.as_bytes()).unwrap_err(), "COORDINATE_OVERFLOW");
    }
    #[test]
    fn miter_arc_and_extensions_never_invent_chords() {
        for replacement in [
            "300 200 1 10 24576",
            "300 200 1 10 28672 CW",
            "300 200 1 10 0 STANDARDVIA",
            "300 200 1 10 0 TEARDROP P 80 80",
        ] {
            let candidate = SAMPLE.replace("300 200 1 10 0", replacement);
            assert_eq!(
                parse(candidate.as_bytes()).unwrap_err(),
                "NO_SUPPORTED_TRACES"
            );
        }
        let candidate = SAMPLE
            .replace("300 200 1 10 0", "300 200 1 10 24576")
            .replace("300 400 1 10 0", "300 400 1 10 0\n400 400 1 10 0");
        let art = parse(candidate.as_bytes()).unwrap();
        assert_eq!(art.skipped, 1);
        assert_eq!(art.segments.len(), 1);
        assert_eq!(art.segments[0].a, [30_000, 40_000]);
        assert_eq!(art.segments[0].b, [40_000, 40_000]);
    }

    #[test]
    fn accepts_complete_pads_v94_maximum_copper_layer_domain() {
        let high = SAMPLE
            .replace("MAXLAYER 4", "MAXIMUMLAYER 64")
            .replace("100 200 1 10 0", "100 200 64 10 0")
            .replace("300 200 1 10 0", "300 200 64 10 0")
            .replace("300 400 1 10 0", "300 400 64 10 0");
        let art = parse(high.as_bytes()).unwrap();
        assert_eq!(art.max_layer, 64);
        assert_eq!(art.segments.len(), 2);
        assert!(art.segments.iter().all(|segment| segment.layer == 64));
        let board = into_canonical(high.as_bytes(), &art).unwrap();
        assert_eq!(board.stackup.layers.len(), 64);
        assert!(!board.validate().has_errors());

        let one_layer = SAMPLE.replace("MAXLAYER 4", "MAXIMUMLAYER 1");
        let art = parse(one_layer.as_bytes()).unwrap();
        assert_eq!(art.max_layer, 1);
        assert_eq!(art.segments.len(), 2);
        assert!(!into_canonical(one_layer.as_bytes(), &art)
            .unwrap()
            .validate()
            .has_errors());

        let invalid = SAMPLE.replace("MAXLAYER 4", "MAXIMUMLAYER 65");
        assert_eq!(
            parse(invalid.as_bytes()).unwrap_err(),
            "UNSUPPORTED_LAYER_COUNT"
        );
    }

    #[test]
    fn enforces_v94_mils_coordinate_domain_without_rounding() {
        let high = SAMPLE.replace("100 200 1 10 0", "56000.01 200 1 10 0");
        assert_eq!(
            parse(high.as_bytes()).unwrap_err(),
            "COORDINATE_OUT_OF_RANGE"
        );
        let low = SAMPLE.replace("100 200 1 10 0", "-56000.01 200 1 10 0");
        assert_eq!(
            parse(low.as_bytes()).unwrap_err(),
            "COORDINATE_OUT_OF_RANGE"
        );
        let boundary = SAMPLE.replace("100 200 1 10 0", "56000 -56000 1 10 0");
        assert_eq!(parse(boundary.as_bytes()).unwrap().segments.len(), 2);
    }
    #[test]
    fn enforces_v94_mils_trace_width_domain() {
        for bad_width in ["56000.01", "-0.01", "0", "-56000", "56001"] {
            let candidate = SAMPLE.replace("300 200 1 10 0", &format!("300 200 1 {bad_width} 0"));
            assert_eq!(
                parse(candidate.as_bytes()).unwrap_err(),
                "WIDTH_OUT_OF_RANGE",
                "invalid PADS width: {bad_width}"
            );
        }
        // The documented positive upper boundary must not be rounded away.
        let boundary = SAMPLE.replace(" 1 10 0", " 1 56000 0");
        let parsed = parse(boundary.as_bytes()).unwrap();
        assert_eq!(parsed.segments.len(), 2);
        assert_eq!(parsed.segments[0].width_ticks, MAX_COORD_TICKS);
    }
    #[test]
    fn cp1252_control_header_and_ignored_high_bytes_preserve_copper_geometry() {
        let annotated = SAMPLE
            .replace(
                "!PADS-POWERPCB-V9.4-MILS!",
                "!PADS-POWERPCB-V9.4-MILS-CP1252!",
            )
            .replace("*PCB*", "*TEXT*\ncafe@\n*PCB*")
            .replace("*ROUTE*", "*REMARK* cafe@\n*ROUTE*");
        let bytes = annotated.into_bytes();
        // CP1252 high byte in a non-route section and remark; not valid UTF-8.
        let bytes: Vec<u8> = bytes
            .into_iter()
            .flat_map(|b| if b == b'@' { vec![0xe9] } else { vec![b] })
            .collect();
        let parsed = parse(&bytes).unwrap();
        assert_eq!(parsed.segments.len(), 2);
        assert_eq!(parsed.segments[0].a, [10_000, 20_000]);

        let default = SAMPLE.replace("*ROUTE*", "*REMARK* @\n*ROUTE*");
        let bytes: Vec<u8> = default
            .into_bytes()
            .into_iter()
            .map(|b| if b == b'@' { 0xe9 } else { b })
            .collect();
        assert_eq!(parse(&bytes).unwrap().segments.len(), 2);
    }
    #[test]
    fn reject_other_code_pages_and_nonascii_active_routes() {
        let cp932 = SAMPLE.replace(
            "!PADS-POWERPCB-V9.4-MILS!",
            "!PADS-POWERPCB-V9.4-MILS-CP932!",
        );
        assert_eq!(
            parse(cp932.as_bytes()).unwrap_err(),
            "UNSUPPORTED_REVISION_OR_UNIT"
        );
        let mut active = SAMPLE
            .replace("*SIGNAL* NET_A", "*SIGNAL* NET_@")
            .into_bytes();
        for byte in &mut active {
            if *byte == b'@' {
                *byte = 0xe9;
            }
        }
        assert_eq!(parse(&active).unwrap_err(), "INVALID_NET");

        // A non-ASCII byte in an active numeric ROUTE record is rejected too.
        let mut geometry = SAMPLE
            .replace("300 200 1 10 0", "300 @ 1 10 0")
            .into_bytes();
        for byte in &mut geometry {
            if *byte == b'@' {
                *byte = 0xe9;
            }
        }
        assert_eq!(parse(&geometry).unwrap_err(), "NON_ASCII_ROUTE_UNSUPPORTED");
    }
    #[test]
    fn official_two_field_route_pin_pair_is_required() {
        let art = parse(SAMPLE.as_bytes()).unwrap();
        assert_eq!(art.segments.len(), 2);
        assert_eq!(art.segments[0].net, "NET_A");
        // Connection-section reuse suffixes do not belong in ROUTE entries.
        // Previously, these were accepted by the synthetic-only importer.
        for suffix in [" R 0 0", " P 0 0", " U 0 0"] {
            let candidate = SAMPLE.replace("U1.1 U2.2", &format!("U1.1 U2.2{suffix}"));
            assert_eq!(
                parse(candidate.as_bytes()).unwrap_err(),
                "INVALID_PIN_PAIR_OR_ROUTE_RECORD"
            );
        }
        let annotated_signal = SAMPLE.replace("*SIGNAL* NET_A", "*SIGNAL* NET_A 1024 0");
        assert_eq!(
            parse(annotated_signal.as_bytes()).unwrap().segments.len(),
            2
        );
    }
    #[test]
    fn pair_and_signal_boundaries_do_not_connect_unrelated_routes() {
        let candidate = SAMPLE.replace("300 400 1 10 0", "300 400 1 10 0\nU2.2 U3.3\n400 400 1 10 0\n500 400 1 10 0\n*SIGNAL* NET_B\nU4.1 U5.2\n600 400 2 10 0\n700 400 2 10 0");
        let art = parse(candidate.as_bytes()).unwrap();
        assert_eq!(art.segments.len(), 4);
        assert_eq!(art.segments[2].net, "NET_A");
        assert_eq!(art.segments[3].net, "NET_B");
        assert_eq!(art.segments[3].layer, 2);
    }
}

#[cfg(test)]
mod p2b_via_tests {
    use super::*;
    const VIA_SAMPLE: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*VIA*\nSIMPLEVIA 15 2\n-2 35 R\n0 35 R\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n300 200 2 10 0 SIMPLEVIA\n500 200 2 10 0\n*END*\n";
    #[test]
    fn imports_named_through_via_without_inferred_contacts() {
        let artwork = parse(VIA_SAMPLE.as_bytes()).unwrap();
        assert_eq!(artwork.vias.len(), 1);
        assert_eq!(artwork.segments.len(), 2);
        assert_eq!(artwork.segments[0].layer, 1);
        assert_eq!(artwork.segments[1].layer, 2);
        assert_eq!(artwork.segments[0].b, artwork.vias[0].point);
        assert_eq!(artwork.segments[1].a, artwork.vias[0].point);
        assert_eq!(artwork.vias[0].point, [30_000, 20_000]);
        assert_eq!(artwork.vias[0].diameter_ticks, 3500);
        assert_eq!(artwork.vias[0].drill_ticks, 1500);
        let board = into_canonical(VIA_SAMPLE.as_bytes(), &artwork).unwrap();
        assert_eq!(board.vias.len(), 1);
        assert_eq!(board.padstacks.len(), 1);
        assert_eq!(board.drills.len(), 1);
        assert!(board.vias[0].connected_layers.is_empty());
        assert!(!board.validate().has_errors());
    }
    #[test]
    fn unknown_and_unsupported_vias_are_not_fabricated() {
        let source = VIA_SAMPLE.replace("SIMPLEVIA", "OTHER_VIA");
        // The renamed definition and reference still match.
        assert_eq!(parse(source.as_bytes()).unwrap().vias.len(), 1);
        let bad = VIA_SAMPLE.replace("0 35 R", "0 35 S");
        // The one adjoining section is not enough to form a safe copper
        // segment once the unsupported via breaks continuity.
        assert_eq!(parse(bad.as_bytes()).unwrap_err(), "NO_SUPPORTED_TRACES");
    }
}

#[cfg(test)]
mod p2b_via_regressions {
    use super::*;
    const BASE: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*VIA*\n*REMARK* keep section\nSIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n300 200 2 10 0 SIMPLEVIA\n500 200 2 10 0\n*END*\n";
    #[test]
    fn inner_pad_and_remark_are_preserved() {
        let artwork = parse(BASE.as_bytes()).unwrap();
        assert_eq!(artwork.vias.len(), 1);
        assert!(artwork.vias[0].inner_pad);
        let board = into_canonical(BASE.as_bytes(), &artwork).unwrap();
        assert_eq!(board.padstacks[0].entries.len(), 4);
        assert_eq!(board.vias[0].connected_layers.len(), 0);
        assert!(!board.validate().has_errors());
    }
    #[test]
    fn via_definitions_after_first_end_do_not_create_geometry() {
        const TRAILING: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n300 200 1 10 0 POSTEND\n400 200 1 10 0\n500 200 1 10 0\n*END*\n*VIA*\nPOSTEND 15 2\n-2 35 R\n0 35 R\n*END*\n";
        let artwork = parse(TRAILING.as_bytes()).unwrap();
        assert!(artwork.vias.is_empty());
        assert_eq!(artwork.segments.len(), 1);
        assert_eq!(artwork.segments[0].a, [40_000, 20_000]);
        let board = into_canonical(TRAILING.as_bytes(), &artwork).unwrap();
        assert!(board.vias.is_empty());
        assert!(board.padstacks.is_empty());
        assert!(board.drills.is_empty());
        assert!(!board.validate().has_errors());
    }

    #[test]
    fn via_only_model_does_not_need_invented_trace() {
        let source = BASE
            .replace("100 200 1 10 0\n", "")
            .replace("500 200 2 10 0\n", "");
        let artwork = parse(source.as_bytes()).unwrap();
        assert!(artwork.segments.is_empty());
        assert_eq!(artwork.vias.len(), 1);
        assert_eq!(
            into_canonical(source.as_bytes(), &artwork)
                .unwrap()
                .vias
                .len(),
            1
        );
    }
}
