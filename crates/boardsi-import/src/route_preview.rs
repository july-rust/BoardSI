//! Restricted PADS Layout 9.4 MILS ROUTE importer for the developer preview.
//! This is a PARTIAL artwork view, never physical electrical connectivity or SI.
//! Grammar basis: Mentor PADS Layout ASCII Format Specification 9.4 pp. 51-55.
//! Accepts straight traces and strict, analytically preserved PADS route arcs.
use boardsi_model::*;
use std::collections::BTreeMap;
use std::sync::Arc;

pub const MAX_FILE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_SEGMENTS: usize = 10_000;
// One canonical tick is 0.01 mil = exactly 254 nm.
const NM_PER_TICK: i64 = 254;
// PADS 9.4 MILS source-domain limit is +/-56,000 mil (exactly +/-5,600,000 ticks).
const MAX_COORD_TICKS: i64 = 5_600_000;
// PADS 9.4 padstack/VIA drill limit is 1000 mil = 100,000 centimil ticks.
const MAX_VIA_DRILL_TICKS: i64 = 100_000;
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
pub struct RouteArc {
    pub layer: u32,
    pub net: String,
    pub start: [i64; 2],
    pub centre: [i64; 2],
    pub end: [i64; 2],
    pub clockwise: bool,
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
    /// Explicit SOURCE-backed per-layer pad geometry for the opt-in V9.4 via
    /// adapter. None denotes the unchanged legacy equal-round Through subset.
    pub topology: Option<crate::via_topology::NormalizedVia>,
    /// Independent SOURCE-layer geometry for non-round copper and plane voids.
    /// Never use a uniform round diameter for these layers.
    pub shaped: Option<crate::via_shapes::NormalizedViaShapes>,
    pub source_line: usize,
}
#[derive(Clone, Copy)]
struct PendingArc {
    start: [i64; 2],
    centre: [i64; 2],
    layer: i64,
    width: i64,
    clockwise: bool,
}
#[derive(Debug)]
pub struct Artwork {
    pub vias: Vec<ViaInstance>,
    pub segments: Vec<Segment>,
    pub arcs: Vec<RouteArc>,
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
    // Limit the entire definition reader to the database body FIRST.
    // A malformed definition claiming missing rows must never consume *END*
    // and then resume at a post-terminator *VIA* section.
    let lines: Vec<&str> = source
        .lines()
        .take_while(|line| line.split_ascii_whitespace().next() != Some("*END*"))
        .collect();
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
        let mut valid = drill > 0 && drill <= MAX_VIA_DRILL_TICKS;
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
pub(crate) fn parse_fixed(value: &str) -> Result<i64, &'static str> {
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
    parse_with_via_mode(source, false)
}

/// Explicit, opt-in PADS V9.4 circular via source support including verified
/// partial drill spans. Existing Qt V1-V14 and historical source receipts
/// continue using parse() and cannot misrepresent these extra structures.
pub fn parse_generalized(source: &[u8]) -> Result<Artwork, &'static str> {
    parse_with_via_mode(source, true)
}

fn parse_with_via_mode(
    source: &[u8],
    allow_generalized_vias: bool,
) -> Result<Artwork, &'static str> {
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
    let source_bytes = source; // Preserve CP1252 raw byte count for the outline parser.
    let source = String::from_utf8_lossy(source);
    let via_defs = scan_simple_via_defs(&source);
    let source_via_defs = if allow_generalized_vias {
        crate::via_source::scan_definitions(&source)
    } else {
        BTreeMap::new()
    };
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
    let mut arcs = Vec::new();
    // A 0x1000 vertex is an arc centre, not an actual copper vertex.
    let mut pending_arc: Option<PendingArc> = None;
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
            if pending_arc.take().is_some() {
                skipped = skipped.saturating_add(1);
            }
            found_end = true;
            break;
        }
        if line.starts_with('*') {
            if pending_arc.take().is_some() {
                skipped = skipped.saturating_add(1);
            }
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
                if window[0].eq_ignore_ascii_case("MAXLAYER")
                    || window[0].eq_ignore_ascii_case("MAXIMUMLAYER")
                {
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
            if pending_arc.take().is_some() {
                skipped = skipped.saturating_add(1);
            }
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
        // Only a PADS 9.4 centre flag plus CW/CCW is an arc centre.
        if flag & 0x1000 != 0 {
            if pending_arc.take().is_some() {
                // Adjacent centres are malformed. Never re-use the previous
                // start vertex to convert the second centre into valid copper.
                skipped = skipped.saturating_add(1);
                previous = None;
                continue;
            }
            if fields.len() == 6
                && flag & !(SUPPORTED_STRAIGHT_FLAGS | 0x1000) == 0
                && (fields[5] == "CW" || fields[5] == "CCW")
            {
                if let Some((start, old_layer, old_width)) = previous {
                    if old_layer == layer && old_width == width && start != p {
                        pending_arc = Some(PendingArc {
                            start,
                            centre: p,
                            layer,
                            width,
                            clockwise: fields[5] == "CW",
                        });
                        continue;
                    }
                }
            }
            skipped = skipped.saturating_add(1);
            previous = None;
            continue;
        }
        // A bad endpoint must not silently turn an arc into a straight chord.
        if let Some(PendingArc {
            start,
            centre,
            layer: arc_layer,
            width: arc_width,
            clockwise,
        }) = pending_arc.take()
        {
            let effective_layer = if layer == 65 { arc_layer } else { layer };
            let radius_sq = |vertex: [i64; 2]| {
                let dx = i128::from(vertex[0]) - i128::from(centre[0]);
                let dy = i128::from(vertex[1]) - i128::from(centre[1]);
                dx * dx + dy * dy
            };
            if fields.len() == 5
                && flag & !SUPPORTED_STRAIGHT_FLAGS == 0
                && effective_layer == arc_layer
                && width == arc_width
                && start != p
                && radius_sq(start) > 0
                && radius_sq(start) == radius_sq(p)
            {
                if segments.len() + arcs.len() >= MAX_SEGMENTS {
                    return Err("SEGMENT_LIMIT");
                }
                arcs.push(RouteArc {
                    layer: arc_layer as u32,
                    net: net.unwrap_or_default().to_string(),
                    start,
                    centre,
                    end: p,
                    clockwise,
                    width_ticks: width,
                    source_line: line_no + 1,
                });
                previous = if layer == 65 {
                    None
                } else {
                    Some((p, arc_layer, width))
                };
                continue;
            }
            skipped = skipped.saturating_add(1);
            previous = None;
            continue;
        }
        // A named via is a separate placement at this exact route vertex,
        // NOT proof of electrical contact with other copper layers.
        if fields.len() == 6 && flag & !SUPPORTED_STRAIGHT_FLAGS == 0 {
            // Generalized mode always checks the strict source-definition
            // inventory, including conflicting duplicate names. A matching
            // legacy equal-round through definition must not bypass it.
            let simple = if allow_generalized_vias {
                None
            } else {
                via_defs.get(fields[5])
            };
            let (topology, shaped) = if allow_generalized_vias {
                let definition = source_via_defs
                    .get(fields[5])
                    .ok_or("D056_UNDECLARED_VIA_DEFINITION")?
                    .as_ref()
                    .map_err(|error| *error)?;
                let profile = definition.normalize_shapes(max_layer)?;
                if profile.is_only_round_copper() {
                    (Some(definition.normalize(max_layer)?), None)
                } else {
                    (None, Some(profile))
                }
            } else {
                (None, None)
            };
            if simple.is_some() || topology.is_some() || shaped.is_some() {
                if !(1..=i64::from(max_layer)).contains(&layer) || max_layer < 2 {
                    return Err("D056_VIA_ROUTE_LAYER_OUT_OF_RANGE");
                }
                let (first_layer, last_layer, diameter_ticks, inner_pad, drill_ticks) =
                    if let Some(normalized) = &topology {
                        (
                            normalized.first_layer,
                            normalized.last_layer,
                            normalized
                                .diameter_on(normalized.first_layer)
                                .ok_or("D056_MISSING_TOP_VIA_PAD")?,
                            normalized.pads.iter().any(|p| {
                                p.layer != normalized.first_layer
                                    && p.layer != normalized.last_layer
                            }),
                            normalized.drill_ticks,
                        )
                    } else if let Some(profile) = &shaped {
                        (
                            profile.first_layer,
                            profile.last_layer,
                            profile
                                .layer_pad(profile.first_layer)
                                .ok_or("D056_MISSING_TOP_VIA_PAD")?
                                .size_ticks,
                            profile.pads.iter().any(|p| {
                                p.layer != profile.first_layer && p.layer != profile.last_layer
                                    && p.shape.conductive()
                            }),
                            profile.drill_ticks,
                        )
                    } else {
                        let definition = simple.ok_or("D056_MISSING_SIMPLE_VIA")?;
                        (
                            1,
                            max_layer,
                            definition.diameter_ticks,
                            definition.inner_pad,
                            definition.drill_ticks,
                        )
                    };
                // A route transition cannot pass through a layer that the
                // declared source drill does not physically span.
                if layer < i64::from(first_layer)
                    || layer > i64::from(last_layer)
                    || previous.is_some_and(|(_, old_layer, _)| {
                        old_layer < i64::from(first_layer) || old_layer > i64::from(last_layer)
                    })
                {
                    return Err("D056_ROUTE_OUTSIDE_VIA_SPAN");
                }
                if vias.len() >= MAX_SEGMENTS {
                    return Err("VIA_LIMIT");
                }
                if let Some((before, old_layer, old_width)) = previous {
                    if old_width == width && before != p {
                        if segments.len() + arcs.len() >= MAX_SEGMENTS {
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
                    diameter_ticks,
                    drill_ticks,
                    first_layer,
                    last_layer,
                    inner_pad,
                    topology,
                    shaped,
                    source_line: line_no + 1,
                });
                previous = Some((p, layer, width));
                continue;
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
                if segments.len() + arcs.len() >= MAX_SEGMENTS {
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
    if max_layer == 0 {
        return Err("MISSING_LAYER_COUNT");
    }
    // Pad-only fixtures are legitimate Preview input; route-only regression
    // behavior stays unchanged when no verified simple SMD pads exist.
    let has_pad = !crate::part_preview::parse(source.as_bytes(), max_layer)
        .pads
        .is_empty();
    let has_outline = !crate::outline_preview::parse(source_bytes)?.is_empty();
    // A valid POUROUT boundary is preview geometry even without routed copper.
    // This never marks the poured interior as an actual conductive plane.
    let has_pour = !crate::pour_preview::parse(source_bytes, max_layer)?.is_empty();
    let has_copper = !crate::copper_preview::parse(source_bytes, max_layer)?.is_empty();
    if !found_route && !has_pad && !has_outline && !has_pour && !has_copper {
        return Err("MISSING_ROUTE_SECTION");
    }
    if segments.is_empty()
        && arcs.is_empty()
        && vias.is_empty()
        && !has_pad
        && !has_outline
        && !has_pour
        && !has_copper
    {
        return Err("NO_SUPPORTED_TRACES");
    }
    Ok(Artwork {
        vias,
        segments,
        arcs,
        max_layer,
        skipped,
    })
}
fn meta_with_provenance(
    source: &SourceId,
    kind: ObjectKind,
    object: &str,
    source_type: &str,
) -> ObjectMeta {
    ObjectMeta {
        id: ObjectId::new(source, kind, object, ""),
        provenance: GeometryProvenance {
            source_object_type: source_type.into(),
            history: vec![],
        },
    }
}
fn meta(source: &SourceId, kind: ObjectKind, object: &str) -> ObjectMeta {
    meta_with_provenance(
        source,
        kind,
        object,
        "PADS Layout ROUTE (partial straight segments)",
    )
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
        importer_version: if parsed.vias.iter().any(|via| via.topology.is_some() || via.shaped.is_some()) {
            "preview-d056-source-layered-via-v1"
        } else if !parsed.arcs.is_empty() {
            "preview-p2f-strict-route-arc-v1"
        } else if parsed.vias.is_empty() {
            "preview-p2a-straight-v2-centimil"
        } else {
            "preview-p2b-round-through-via-v1"
        }
        .into(),
        conditions: {
            let mut conditions = vec![("route_subset".into(), "straight-same-layer".into())];
            if !parsed.arcs.is_empty() {
                conditions.push((
                    "arc_subset".into(),
                    "strict-centre-cw-ccw-source-grid".into(),
                ));
            }
            if parsed.vias.iter().any(|via| via.topology.is_some() || via.shaped.is_some()) {
                conditions.push((
                    "via_subset".into(),
                    "source-explicit-round-layered-via-span-no-electrical-proof".into(),
                ));
            } else if !parsed.vias.is_empty() {
                conditions.push((
                    "via_subset".into(),
                    "explicit-equal-round-through-pad-and-drill-no-connectivity".into(),
                ));
            }
            // Stable source-condition order is part of the Canonical identity contract.
            conditions.sort_by(|a: &(String, String), b| a.0.cmp(&b.0));
            conditions
        },
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
            object_coverage: {
                let mut coverage = Vec::new();
                if !parsed.segments.is_empty() {
                    coverage.push("ROUTE straight copper segment only".into());
                }
                if !parsed.arcs.is_empty() {
                    coverage.push(
                        "ROUTE strict exact-radius CW/CCW Arc geometry; electrical contact not verified".into(),
                    );
                }
                if !parsed.vias.is_empty() {
                    coverage.push(
                        "VIA explicit round through-hole padstack and drill geometry; contact not verified"
                            .into(),
                    );
                }
                coverage
            },
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
    for a in &parsed.arcs {
        if !nets.contains_key(&a.net) {
            let m = meta(&source, ObjectKind::Net, &a.net);
            nets.insert(a.net.clone(), m.id.clone());
            board.nets.push(Net {
                meta: m,
                name: a.net.clone(),
                role: NetRole::Unknown,
            });
        }
        let point = |ticks| Point {
            frame: frame_id.clone(),
            ticks,
        };
        board.arcs.push(boardsi_model::Arc {
            meta: meta_with_provenance(
                &source,
                ObjectKind::Arc,
                &format!("route-arc-{}", a.source_line),
                "PADS Layout ROUTE strict centre/CCW/CW arc",
            ),
            layer: ObjectId::new(
                &source,
                ObjectKind::Layer,
                &format!("layer-{}", a.layer),
                "",
            ),
            net: nets.get(&a.net).cloned(),
            centre: point(a.centre),
            start: point(a.start),
            end: point(a.end),
            clockwise: a.clockwise,
            width_m: Rational::new(i128::from(a.width_ticks) * 127, 500_000_000)
                .map_err(|_| "WIDTH_OVERFLOW")?,
        });
    }
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
        // Source profiles carry PadRole and exact round/square shapes. The
        // legacy normalized circular path remains only for truly round vias.
        let rounded = if via.shaped.is_some() {
            None
        } else if let Some(topology) = &via.topology {
            Some(topology.clone())
        } else {
            Some(crate::via_topology::NormalizedVia::pads_v94_through(
                parsed.max_layer,
                via.drill_ticks,
                via.diameter_ticks,
                via.inner_pad.then_some(via.diameter_ticks),
                via.diameter_ticks,
            )?)
        };
        let kind = if let Some(profile) = &via.shaped {
            profile.kind
        } else {
            rounded.as_ref().ok_or("D056_MISSING_VIA_PROFILE")?.kind
        };
        let plated = if let Some(profile) = &via.shaped {
            profile.plated
        } else {
            rounded.as_ref().ok_or("D056_MISSING_VIA_PROFILE")?.plated
        };
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
            // Canonical padstacks consume exactly the source-backed layer
            // inventory; no guessed interior conductive entry.
            let layer_id = |n| ObjectId::new(
                &source,
                ObjectKind::Layer,
                &format!("layer-{n}"),
                "",
            );
            let entries = if let Some(profile) = &via.shaped {
                profile.canonical_entries(layer_id)?
            } else {
                rounded
                    .as_ref()
                    .ok_or("D056_MISSING_VIA_PROFILE")?
                    .canonical_circular_pads(layer_id)?
            };
            board.padstacks.push(Padstack {
                meta: meta_with_provenance(
                    &source,
                    ObjectKind::Padstack,
                    &padstack_key,
                    "PADS Layout VIA definition (restricted round padstack geometry)",
                ),
                entries,
            });
            board.drills.push(Drill {
                meta: meta_with_provenance(
                    &source,
                    ObjectKind::Drill,
                    &drill_key,
                    "PADS Layout VIA definition (restricted round drill geometry)",
                ),
                diameter_m: Rational::new(i128::from(via.drill_ticks) * 127, 500_000_000)
                    .map_err(|_| "VIA_DIMENSION_OVERFLOW")?,
                plated,
                span: span.clone(),
            });
        }
        board.vias.push(Via {
            meta: meta_with_provenance(
                &source,
                ObjectKind::Via,
                &format!("via-line-{}", via.source_line),
                if via.shaped.is_some() {
                    "PADS Layout ROUTE via placement (SOURCE-declared layer-shaped padstack)"
                } else if via.topology.is_some() {
                    "PADS Layout ROUTE via placement (SOURCE-declared round layer span)"
                } else {
                    "PADS Layout ROUTE via placement (restricted round through-via)"
                },
            ),
            position: Point {
                frame: frame_id.clone(),
                ticks: via.point,
            },
            padstack: padstack_id,
            drill: drill_id,
            span,
            kind,
            net: nets.get(&via.net).cloned(),
            // Never infer electrical contacts solely from same net or drill span.
            connected_layers: vec![],
        });
    }
    crate::connectivity_preview::augment(&mut board)?;
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
    fn cp1252_ignored_rows_do_not_expand_raw_outline_file_limit() {
        let mut bytes =
            b"!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n"
                .to_vec();
        for _ in 0..720 {
            bytes.extend_from_slice(b"*REMARK* ");
            bytes.extend(std::iter::repeat(0xe9u8).take(4000));
            bytes.push(b'\n');
        }
        bytes.extend_from_slice(
            b"*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n300 200 1 10 0\n*END*\n",
        );
        assert!(bytes.len() < MAX_FILE_BYTES);
        assert!(String::from_utf8_lossy(&bytes).len() > MAX_FILE_BYTES);
        let artwork = parse(&bytes).expect("valid raw CP1252 bytes in ignored remarks");
        assert_eq!(artwork.segments.len(), 1);
        assert!(crate::outline_preview::parse(&bytes).unwrap().is_empty());
    }
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
        // A strict CW centre with equal-radius endpoints is supported in A_015.
        let arc = parse(
            SAMPLE
                .replace("300 200 1 10 0", "300 200 1 10 4096 CW")
                .as_bytes(),
        )
        .unwrap();
        assert_eq!(arc.arcs.len(), 1);
        assert!(arc.segments.is_empty());
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
        let coverage = &board.sources[0].capability.object_coverage;
        assert!(coverage
            .iter()
            .any(|scope| scope.contains("ROUTE straight")));
        assert!(coverage
            .iter()
            .any(|scope| scope.contains("VIA explicit round")));
        assert_eq!(board.padstacks.len(), 1);
        assert_eq!(board.drills.len(), 1);
        assert!(board.padstacks[0]
            .meta
            .provenance
            .source_object_type
            .contains("VIA definition"));
        assert!(board.drills[0]
            .meta
            .provenance
            .source_object_type
            .contains("VIA definition"));
        assert!(board.vias[0]
            .meta
            .provenance
            .source_object_type
            .contains("ROUTE via placement"));
        assert_eq!(board.vias[0].connected_layers.len(), 1);
        // The two-row source padstack has no conductive pad on layer 2:
        // never infer barrel-to-trace contact from the via's drilled span.
        assert_eq!(
            board
                .connectivity
                .edges
                .iter()
                .filter(|e| matches!(e.evidence, ConnectionEvidence::ViaBarrel { .. }))
                .count(),
            0
        );
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
    fn d056_same_source_via_definition_supports_multiple_board_layer_counts() {
        // The same PADS top/inner/bottom definition is expanded using each
        // board's parsed max layer, not a compiled 2-/4-/6-/8-layer branch.
        for layers in [2u32, 4, 6, 8] {
            let source = BASE.replace("MAXLAYER 4", &format!("MAXLAYER {layers}"));
            let artwork = parse(source.as_bytes()).expect("valid through-via source");
            assert_eq!(artwork.max_layer, layers);
            assert_eq!(artwork.vias.len(), 1);
            assert_eq!(artwork.vias[0].last_layer, layers);
            let model = into_canonical(source.as_bytes(), &artwork).unwrap();
            assert_eq!(model.stackup.layers.len(), layers as usize);
            assert_eq!(model.padstacks[0].entries.len(), layers as usize);
            assert_eq!(model.vias[0].kind, ViaKind::Through);
            assert!(!model.validate().has_errors());
        }
    }

    #[test]
    fn d056_no_declared_inner_pad_is_not_synthesized() {
        let without_inner = BASE
            .replace(
                "SIMPLEVIA 15 3\n-2 35 R\n-1 35 R\n0 35 R",
                "SIMPLEVIA 15 2\n-2 35 R\n0 35 R",
            )
            .replace("MAXLAYER 4", "MAXLAYER 6");
        let artwork = parse(without_inner.as_bytes()).unwrap();
        assert!(!artwork.vias[0].inner_pad);
        let model = into_canonical(without_inner.as_bytes(), &artwork).unwrap();
        assert_eq!(model.padstacks[0].entries.len(), 2);
        assert!(!model.validate().has_errors());
    }

    #[test]
    fn pcb_layer_parameter_names_accept_ascii_case_variants() {
        for keyword in ["maximumlayer", "MaximumLayer", "maxlayer", "MaxLayer"] {
            let source = BASE.replace("MAXLAYER", keyword);
            let artwork = parse(source.as_bytes()).unwrap();
            assert_eq!(artwork.max_layer, 4);
            assert_eq!(artwork.vias.len(), 1);
        }
    }

    #[test]
    fn rejects_oversize_via_drills_without_fabricating_geometry() {
        let oversized = BASE
            .replace("SIMPLEVIA 15 3", "SIMPLEVIA 2000 3")
            .replace("35 R", "3000 R");
        let scan = scan_simple_via_defs(&oversized);
        assert!(!scan.contains_key("SIMPLEVIA"));
        assert_eq!(
            parse(oversized.as_bytes()).unwrap_err(),
            "NO_SUPPORTED_TRACES"
        );

        let upper_bound = BASE
            .replace("SIMPLEVIA 15 3", "SIMPLEVIA 1000 3")
            .replace("35 R", "3000 R");
        let scan = scan_simple_via_defs(&upper_bound);
        assert_eq!(
            scan.get("SIMPLEVIA").map(|via| via.drill_ticks),
            Some(100_000)
        );
    }

    #[test]
    fn inner_pad_and_remark_are_preserved() {
        let artwork = parse(BASE.as_bytes()).unwrap();
        assert_eq!(artwork.vias.len(), 1);
        assert!(artwork.vias[0].inner_pad);
        let board = into_canonical(BASE.as_bytes(), &artwork).unwrap();
        assert_eq!(board.padstacks[0].entries.len(), 4);
        assert_eq!(board.vias[0].connected_layers.len(), 2);
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
    fn truncated_via_definition_cannot_swallow_end_and_import_trailing_data() {
        // A pre-END via header claims two round pad rows but only one exists.
        // Definitions following the first END cannot satisfy an earlier route
        // vertex, even when the malformed row count would jump over END.
        const TRUNCATED: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n300 200 1 10 0 POSTEND\n400 200 1 10 0\n500 200 1 10 0\n*VIA*\nTRUNC 15 2\n-2 35 R\n*END*\n*VIA*\nPOSTEND 15 2\n-2 35 R\n0 35 R\n*END*\n";
        let artwork = parse(TRUNCATED.as_bytes()).unwrap();
        assert!(artwork.vias.is_empty());
        assert_eq!(artwork.segments.len(), 1);
        let board = into_canonical(TRUNCATED.as_bytes(), &artwork).unwrap();
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

    #[test]
    fn a015_adjacent_centres_must_not_promote_copper() {
        let source = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n200 100 1 10 0\n100 100 1 10 4096 CCW\n200 200 1 10 4096 CW\n300 200 1 10 0\n*END*\n";
        assert_eq!(parse(source.as_bytes()).unwrap_err(), "NO_SUPPORTED_TRACES");
    }

    #[test]
    fn a015_strict_arc_canonical_and_fail_closed() {
        let source = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* ARC_NET\nU1.1 U2.2\n200 100 1 10 0\n100 100 1 10 4096 CCW\n100 200 1 10 0\n*END*\n";
        let a = parse(source.as_bytes()).unwrap();
        assert_eq!(a.arcs.len(), 1);
        assert!(a.segments.is_empty());
        assert_eq!(a.arcs[0].centre, [10000, 10000]);
        assert!(!a.arcs[0].clockwise);
        let model = into_canonical(source.as_bytes(), &a).unwrap();
        assert_eq!(model.arcs.len(), 1);
        assert!(model.traces.is_empty());
        assert!(model.sources[0]
            .capability
            .object_coverage
            .iter()
            .any(|item| item.contains("CW/CCW Arc")));
        assert!(!model.validate().has_errors());
        let cw = source.replace("4096 CCW", "4096 CW");
        assert!(parse(cw.as_bytes()).unwrap().arcs[0].clockwise);
        let bad = source.replace("100 200 1 10 0", "100 201 1 10 0");
        assert_eq!(parse(bad.as_bytes()).unwrap_err(), "NO_SUPPORTED_TRACES");
        let dangling = source.replace("100 200 1 10 0\n", "");
        assert_eq!(
            parse(dangling.as_bytes()).unwrap_err(),
            "NO_SUPPORTED_TRACES"
        );
        let unknown = source.replace("4096 CCW", "4096 UNKNOWN");
        assert_eq!(
            parse(unknown.as_bytes()).unwrap_err(),
            "NO_SUPPORTED_TRACES"
        );
    }
}

#[cfg(test)]
mod d056_native_via_source_regressions {
    use super::*;

    fn source(layers: u32, first: u32, last: u32, definition: &str) -> String {
        format!(
            "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n\
*PCB*\nMAXLAYER {layers}\n*VIA*\n{definition}\n\
*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n\
100 200 {first} 10 0\n300 200 {last} 10 0 V_TEST\n\
500 200 {last} 10 0\n*END*\n"
        )
    }

    #[test]
    fn source_declared_partial_spans_2_3_and_2_4_share_one_canonical_path() {
        for (first, last, expected) in [
            (2, 4, ViaKind::Buried),
            (2, 3, ViaKind::Buried),
            (1, 3, ViaKind::Blind),
            (3, 6, ViaKind::Blind),
            (1, 6, ViaKind::Through),
        ] {
            let definition = format!("V_TEST 15 3 {first} {last}\n-2 35 R\n-1 45 R\n0 40 R");
            let source = source(6, first, last, &definition);
            // The original Qt V1-V14 wire must not guess a diameter/span.
            assert!(parse(source.as_bytes()).is_err());
            let artwork = parse_generalized(source.as_bytes()).unwrap();
            assert_eq!(artwork.vias.len(), 1);
            assert_eq!(artwork.skipped, 0);
            let v = &artwork.vias[0];
            let topology = v.topology.as_ref().unwrap();
            assert_eq!((v.first_layer, v.last_layer), (first, last));
            assert_eq!(topology.kind, expected);
            assert_eq!(topology.diameter_on(first), Some(3500));
            assert_eq!(topology.diameter_on(last), Some(4000));
            assert_eq!(
                topology.diameter_on(first + 1),
                if last > first + 1 {
                    Some(4500)
                } else {
                    Some(4000)
                }
            );
            let board = into_canonical(source.as_bytes(), &artwork).unwrap();
            assert_eq!(board.vias[0].kind, expected);
            assert_eq!(board.padstacks[0].entries.len(), topology.pads.len());
            assert_eq!(board.vias[0].connected_layers.len(), 0);
            assert!(!board.validate().has_errors());
        }
    }

    #[test]
    fn source_declared_nonuniform_through_pads_follow_board_layer_count() {
        for layers in [2, 4, 6, 8] {
            let source = source(layers, 1, layers, "V_TEST 15 3\n-2 35 R\n-1 45 R\n0 40 R");
            let artwork = parse_generalized(source.as_bytes()).unwrap();
            let topology = artwork.vias[0].topology.as_ref().unwrap();
            assert_eq!(topology.pads.len(), layers as usize);
            assert_eq!(topology.diameter_on(layers), Some(4000));
            assert_eq!(topology.kind, ViaKind::Through);
            let board = into_canonical(source.as_bytes(), &artwork).unwrap();
            assert_eq!(board.padstacks[0].entries.len(), layers as usize);
            assert!(!board.validate().has_errors());
        }
    }

    #[test]
    fn generalized_mode_cannot_bypass_conflicting_source_via_names() {
        // Legacy scan recognizes the first equal-round Through Via, but
        // the same name has a second, contradictory L2-L4 definition.
        // Generalized import must reject rather than reuse the old match.
        let source = source(
            6,
            1,
            6,
            "V_TEST 15 2\n-2 35 R\n0 35 R\nV_TEST 15 2 2 4\n-2 35 R\n0 40 R",
        );
        assert_eq!(parse(source.as_bytes()).unwrap().vias.len(), 1);
        assert_eq!(
            parse_generalized(source.as_bytes()).unwrap_err(),
            "D056_DUPLICATE_VIA_DEFINITION"
        );
        let contact_error =
            crate::trace_attachment::source_candidates_generalized(source.as_bytes()).unwrap_err();
        assert_eq!(contact_error, "D056_DUPLICATE_VIA_DEFINITION");
    }

    #[test]
    fn d056_square_and_antipad_records_keep_canonical_roles() {
        let source = source(
            6,
            2,
            4,
            "V_TEST 15 3 2 4\n-2 35 S\n-1 65 RA\n0 40 R",
        );
        let artwork = parse_generalized(source.as_bytes()).unwrap();
        assert_eq!(artwork.vias.len(), 1);
        assert!(artwork.vias[0].topology.is_none());
        let shapes = artwork.vias[0].shaped.as_ref().unwrap();
        assert_eq!(shapes.first_layer, 2);
        assert_eq!(shapes.last_layer, 4);
        assert!(shapes.conductive_pad(2).is_some());
        assert!(shapes.conductive_pad(3).is_none());
        assert_eq!(
            shapes.layer_pad(3).unwrap().shape,
            crate::via_shapes::ViaLayerShape::RoundAntiPad
        );
        let board = into_canonical(source.as_bytes(), &artwork).unwrap();
        assert_eq!(board.vias[0].kind, ViaKind::Buried);
        assert_eq!(board.vias[0].connected_layers.len(), 0);
        assert_eq!(board.padstacks[0].entries.len(), 3);
        assert_eq!(board.padstacks[0].entries[0].role, boardsi_model::PadRole::Conductive);
        assert_eq!(board.padstacks[0].entries[1].role, boardsi_model::PadRole::AntiPad);
        assert_eq!(board.padstacks[0].entries[2].role, boardsi_model::PadRole::Conductive);
        assert!(matches!(
            board.padstacks[0].entries[0].shape,
            boardsi_model::PadShape::Rectangle { .. }
        ));
        assert!(matches!(
            board.padstacks[0].entries[1].shape,
            boardsi_model::PadShape::Circle { .. }
        ));
        assert!(!board.validate().has_errors());
    }

    #[test]
    fn missing_inner_pad_and_explicit_layer_override_not_synthesized() {
        let definition = "V_TEST 15 3 2 4\n-2 35 R\n3 46 R\n0 40 R";
        let source = source(6, 2, 4, definition);
        let artwork = parse_generalized(source.as_bytes()).unwrap();
        let via = artwork.vias[0].topology.as_ref().unwrap();
        assert_eq!(via.pads.len(), 3);
        assert_eq!(via.diameter_on(1), None);
        assert_eq!(via.diameter_on(3), Some(4600));
        assert_eq!(via.diameter_on(5), None);
        let board = into_canonical(source.as_bytes(), &artwork).unwrap();
        assert_eq!(board.padstacks[0].entries.len(), 3);
        assert!(!board.validate().has_errors());
    }

    #[test]
    fn wrong_route_span_and_unsupported_native_shape_fail_closed() {
        let invalid_route = source(6, 1, 3, "V_TEST 15 2 2 3\n-2 35 R\n0 40 R");
        assert_eq!(
            parse_generalized(invalid_route.as_bytes()).unwrap_err(),
            "D056_ROUTE_OUTSIDE_VIA_SPAN"
        );
        let unsupported = source(6, 2, 4, "V_TEST 15 2 2 4\n-2 35 S\n0 40 R");
        assert_eq!(
            parse_generalized(unsupported.as_bytes()).unwrap_err(),
            "D056_UNSUPPORTED_VIA_PAD_SHAPE"
        );
        let malformed = source(6, 2, 4, "V_TEST 15 2 4 2\n-2 35 R\n0 40 R");
        assert_eq!(
            parse_generalized(malformed.as_bytes()).unwrap_err(),
            "D056_INVALID_VIA_SPAN_OR_DRILL"
        );
    }
}
