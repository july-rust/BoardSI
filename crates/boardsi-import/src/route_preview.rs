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
    // Standard ROUTE grammar also permits the exact two-field pin pair.
    if fields.len() == 2 {
        return pin_token(fields[0]) && pin_token(fields[1]);
    }
    fields.len() == 5
        && pin_token(fields[0])
        && pin_token(fields[1])
        && matches!(fields[2], "R" | "P" | "U")
        && fields[3].parse::<i64>().is_ok()
        && fields[4].parse::<i64>().is_ok()
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
        if width <= 0 || !(0..=32767).contains(&flag) {
            return Err("INVALID_ROUTE_FIELD");
        }
        let p = [x, y];
        mul_nm(x)?;
        mul_nm(y)?;
        mul_nm(width)?;
        // Only flag bits which cannot alter geometry are supported.
        // Any arc/miter/via/jumper/teardrop/unknown extension breaks the path.
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
        let no_pair = SAMPLE.replace("U1.1 U2.2 R 0 0\n", "");
        assert_eq!(parse(no_pair.as_bytes()).unwrap_err(), "MISSING_PIN_PAIR");
        let bad_pair = SAMPLE.replace("U1.1 U2.2 R 0 0", "U1.1 GARBAGE R 0 0");
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
    fn standard_two_field_pin_pair_is_accepted_without_invented_metadata() {
        let sample = SAMPLE.replace("U1.1 U2.2 R 0 0", "U1.1 U2.2");
        let art = parse(sample.as_bytes()).unwrap();
        assert_eq!(art.segments.len(), 2);
        assert_eq!(art.segments[0].net, "NET_A");
    }
    #[test]
    fn pair_and_signal_boundaries_do_not_connect_unrelated_routes() {
        let candidate = SAMPLE.replace("300 400 1 10 0", "300 400 1 10 0\nU2.2 U3.3 P 0 0\n400 400 1 10 0\n500 400 1 10 0\n*SIGNAL* NET_B\nU4.1 U5.2 R 0 0\n600 400 2 10 0\n700 400 2 10 0");
        let art = parse(candidate.as_bytes()).unwrap();
        assert_eq!(art.segments.len(), 4);
        assert_eq!(art.segments[2].net, "NET_A");
        assert_eq!(art.segments[3].net, "NET_B");
        assert_eq!(art.segments[3].layer, 2);
    }
}
