//! D_015 bounded perpendicular source-tick T-junction on D_014 two-Via topology.
//! These are geometry CONTACT CANDIDATES, never fabricated electrical edges.
use boardsi_import::{digest, route_preview};
use std::fmt::Write;

fn orient(a: [i64; 2], b: [i64; 2], c: [i64; 2]) -> i128 {
    let x = i128::from(b[0]) - i128::from(a[0]);
    let y = i128::from(b[1]) - i128::from(a[1]);
    x * (i128::from(c[1]) - i128::from(a[1])) - y * (i128::from(c[0]) - i128::from(a[0]))
}
pub(super) fn on_segment(p: [i64; 2], a: [i64; 2], b: [i64; 2]) -> bool {
    orient(a, b, p) == 0
        && p[0] >= a[0].min(b[0])
        && p[0] <= a[0].max(b[0])
        && p[1] >= a[1].min(b[1])
        && p[1] <= a[1].max(b[1])
}
fn strict_interior(p: [i64; 2], a: [i64; 2], b: [i64; 2]) -> bool {
    p != a && p != b && on_segment(p, a, b)
}
pub(super) fn segments_touch(a: [i64; 2], b: [i64; 2], c: [i64; 2], d: [i64; 2]) -> bool {
    let (x, y, z, w) = (
        orient(a, b, c),
        orient(a, b, d),
        orient(c, d, a),
        orient(c, d, b),
    );
    let opposite = |a: i128, b: i128| (a < 0 && b > 0) || (a > 0 && b < 0);
    (x == 0 && on_segment(c, a, b))
        || (y == 0 && on_segment(d, a, b))
        || (z == 0 && on_segment(a, c, d))
        || (w == 0 && on_segment(b, c, d))
        || (opposite(x, y) && opposite(z, w))
}
fn root(parent: &[usize], mut i: usize) -> usize {
    while parent[i] != i {
        i = parent[i];
    }
    i
}

// Deliberately limited *complete* public Source vocabulary. Route-only parsing
// must not silently ignore COPCLS, *LINES*, PARTS, other conductive artwork
// or a second (possibly conflicting) VIA definition.
// PADS V9.4 MILS lexical fixed-point subset: no floating-point rounding,
// no exponent/NaN, at most 2 fractional digits (one source tick = .01 mil).
fn mils_lexeme(text: &str) -> bool {
    let digits = text
        .strip_prefix('-')
        .or_else(|| text.strip_prefix('+'))
        .unwrap_or(text);
    let mut parts = digits.split('.');
    let whole = parts.next().unwrap_or("");
    let fraction = parts.next();
    !whole.is_empty()
        && whole.as_bytes().iter().all(u8::is_ascii_digit)
        && match fraction {
            None => true,
            Some(f) => !f.is_empty() && f.len() <= 2 && f.as_bytes().iter().all(u8::is_ascii_digit),
        }
        && parts.next().is_none()
}
pub(super) fn bounded_source_inventory(
    bytes: &[u8],
    via_name: &str,
    expected_signals: usize,
    expected_points: usize,
) -> Result<(), &'static str> {
    let source = std::str::from_utf8(bytes).map_err(|_| "D015_SOURCE_TEXT")?;
    let mut section = "";
    let mut header = false;
    let mut max_layer = 0;
    let mut via_defs = 0;
    let mut pad_rows = 0;
    let mut bottom_pad = false;
    let mut top_pad = false;
    let mut signals = 0;
    let mut route_pairs = 0;
    let mut route_points = 0;
    let mut ended = false;
    for raw in source.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if ended {
            return Err("D015_TRAILING_SOURCE");
        }
        if !header {
            if !line.starts_with("!PADS-POWERPCB-V9.4-MILS!") {
                return Err("D015_HEADER");
            }
            header = true;
            continue;
        }
        let words: Vec<_> = line.split_ascii_whitespace().collect();
        let first = words[0];
        if first == "*REMARK*" || first == "REMARK" {
            continue;
        }
        match first {
            "*PCB*" if section.is_empty() => {
                section = "pcb";
                continue;
            }
            "*VIA*" if section == "pcb" && max_layer == 1 => {
                section = "via";
                continue;
            }
            "*ROUTE*" if section == "via" && via_defs == 1 && pad_rows == 2 => {
                section = "route";
                continue;
            }
            "*END*" if section == "route" && words.len() == 1 => {
                ended = true;
                continue;
            }
            "*SIGNAL*" if section == "route" && words.len() == 2 => {
                signals += 1;
                continue;
            }
            _ => {}
        }
        if first.starts_with('*') {
            return Err("D015_UNMODELED_SECTION");
        }
        match section {
            "pcb" if words.len() == 2 && words[0] == "MAXLAYER" && words[1] == "4" => {
                max_layer += 1;
                if max_layer != 1 {
                    return Err("D015_DUPLICATE_LAYER");
                }
            }
            "via" if first == via_name && words.len() == 3 && words[2] == "2" => {
                via_defs += 1;
                if via_defs != 1 {
                    return Err("D015_DUPLICATE_VIA_DEF");
                }
            }
            "via" if words.len() == 3 && (first == "-2" || first == "0") => {
                if first == "-2" {
                    if bottom_pad {
                        return Err("D015_DUPLICATE_PAD_ROW");
                    }
                    bottom_pad = true;
                } else {
                    if top_pad {
                        return Err("D015_DUPLICATE_PAD_ROW");
                    }
                    top_pad = true;
                }
                pad_rows += 1;
            }
            "route" if signals > 0 && words.len() == 2 && words.iter().all(|w| w.contains('.')) => {
                route_pairs += 1;
            }
            "route"
                if signals > 0
                    && (words.len() == 5 || words.len() == 6)
                    && mils_lexeme(words[0])
                    && mils_lexeme(words[1])
                    && words[2].parse::<u32>().is_ok()
                    && mils_lexeme(words[3])
                    && words[4].parse::<i64>().is_ok() =>
            {
                if words.len() == 6 && words[5] != via_name {
                    return Err("D015_EXTRA_ROUTE_VIA");
                }
                route_points += 1;
            }
            _ => return Err("D015_UNMODELED_SOURCE"),
        }
    }
    if !header
        || !ended
        || max_layer != 1
        || via_defs != 1
        || pad_rows != 2
        || !bottom_pad
        || !top_pad
        || signals != expected_signals
        || route_pairs != expected_signals
        || route_points != expected_points
    {
        return Err("D015_SOURCE_STRUCTURE");
    }
    Ok(())
}

pub fn encode(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, ContactMode::Perpendicular)
}

#[allow(dead_code)]
pub fn encode_oblique(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, ContactMode::Oblique)
}

/// D_020 strict source-owned positive-length contained collinear overlap.
/// Never a manufactured copper or electrical continuity guarantee.
#[allow(dead_code)]
pub fn encode_contained_overlap(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, ContactMode::Contained)
}

/// D_021 proper partial same-layer collinear Trace overlap: geometry candidate only.
#[allow(dead_code)]
pub fn encode_partial_overlap(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, ContactMode::Partial)
}

/// D_022 exact collinear endpoint contact; Source geometry candidate only.
#[allow(dead_code)]
pub fn encode_endpoint_contact(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, ContactMode::Endpoint)
}

/// D_023 opposite Via-A Source endpoint mode; NEVER electrical barrel proof.
#[allow(dead_code)]
pub fn encode_opposite_endpoint_contact(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, ContactMode::OppositeEndpoint)
}

/// D_024 orthogonal 90-degree Via-B Source endpoint; NEVER electrical proof.
#[allow(dead_code)]
pub fn encode_orthogonal_endpoint_contact(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, ContactMode::OrthogonalEndpoint)
}

/// D_025 exact-tick Via-A orthogonal source contact; never fabricated copper proof.
#[allow(dead_code)]
pub fn encode_orthogonal_via_a_endpoint_contact(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, ContactMode::OrthogonalViaA)
}

/// D_026 Via-B 45-degree oblique endpoint Source contact, never electrical proof.
#[allow(dead_code)]
pub fn encode_oblique_via_b_endpoint_contact(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, ContactMode::ObliqueViaB)
}

/// D_027 Via-A 45-degree oblique exterior endpoint Source contact; never electrical proof.
#[allow(dead_code)]
pub fn encode_oblique_via_a_endpoint_contact(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, ContactMode::ObliqueViaA)
}

/// D_035 distinct signed-negative 4:3 exterior Via-A SOURCE, not electrical proof.
#[allow(dead_code)]
pub fn encode_non45_oblique_mirrored_via_a_endpoint_contact(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(
        bytes,
        net,
        layers,
        vias,
        ContactMode::Non45ObliqueMirroredViaA,
    )
}

/// D_034 distinct signed-negative 4:3 exterior Via-B SOURCE, not electrical proof.
#[allow(dead_code)]
pub fn encode_non45_oblique_mirrored_via_b_endpoint_contact(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(
        bytes,
        net,
        layers,
        vias,
        ContactMode::Non45ObliqueMirroredViaB,
    )
}

/// D_028 bounded exact 4:3 non-45-degree exterior Via-B endpoint; source only.
#[allow(dead_code)]
pub fn encode_non45_oblique_via_b_endpoint_contact(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, ContactMode::Non45ObliqueViaB)
}

/// D_029 exact 4:3 non-45-degree Via-A Source endpoint; no barrel proof.
#[allow(dead_code)]
pub fn encode_non45_oblique_via_a_endpoint_contact(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, ContactMode::Non45ObliqueViaA)
}

/// D_030 exact steep 3:4 non-45-degree Via-B Source endpoint; no electrical proof.
#[allow(dead_code)]
pub fn encode_non45_steep_via_b_endpoint_contact(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, ContactMode::Non45SteepViaB)
}

/// D_032 exact negative-slope mirrored steep 3:4 Via-B SOURCE endpoint only.
#[allow(dead_code)]
pub fn encode_non45_steep_mirrored_via_b_endpoint_contact(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(
        bytes,
        net,
        layers,
        vias,
        ContactMode::Non45SteepMirroredViaB,
    )
}

/// D_033 exact negative-slope mirrored steep 3:4 Via-A SOURCE endpoint only.
#[allow(dead_code)]
pub fn encode_non45_steep_mirrored_via_a_endpoint_contact(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(
        bytes,
        net,
        layers,
        vias,
        ContactMode::Non45SteepMirroredViaA,
    )
}

/// D_031 exact steep 3:4 non-45-degree Via-A Source endpoint; not electrical proof.
#[allow(dead_code)]
pub fn encode_non45_steep_via_a_endpoint_contact(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, ContactMode::Non45SteepViaA)
}

#[derive(Clone, Copy)]
enum ContactMode {
    Perpendicular,
    Oblique,
    Contained,
    Partial,
    Endpoint,
    OppositeEndpoint,
    OrthogonalEndpoint,
    OrthogonalViaA,
    ObliqueViaB,
    ObliqueViaA,
    Non45ObliqueViaB,
    Non45ObliqueMirroredViaB,
    Non45ObliqueViaA,
    Non45ObliqueMirroredViaA,
    Non45SteepViaB,
    Non45SteepViaA,
    Non45SteepMirroredViaB,
    Non45SteepMirroredViaA,
}

fn encode_with_mode(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
    mode: ContactMode,
) -> Result<String, &'static str> {
    let oblique = matches!(mode, ContactMode::Oblique);
    let contained_overlap = matches!(mode, ContactMode::Contained);
    let partial_overlap = matches!(mode, ContactMode::Partial);
    let opposite_endpoint = matches!(mode, ContactMode::OppositeEndpoint);
    let orthogonal_via_a = matches!(mode, ContactMode::OrthogonalViaA);
    let non45_oblique_via_b = matches!(mode, ContactMode::Non45ObliqueViaB);
    let non45_oblique_mirrored_via_b = matches!(mode, ContactMode::Non45ObliqueMirroredViaB);
    let non45_oblique_via_a = matches!(mode, ContactMode::Non45ObliqueViaA);
    let non45_oblique_mirrored_via_a = matches!(mode, ContactMode::Non45ObliqueMirroredViaA);
    let non45_steep_via_b = matches!(mode, ContactMode::Non45SteepViaB);
    let non45_steep_via_a = matches!(mode, ContactMode::Non45SteepViaA);
    let non45_steep_mirrored_via_b = matches!(mode, ContactMode::Non45SteepMirroredViaB);
    let non45_steep_mirrored_via_a = matches!(mode, ContactMode::Non45SteepMirroredViaA);
    let oblique_via_b = matches!(mode, ContactMode::ObliqueViaB);
    let oblique_via_a = matches!(mode, ContactMode::ObliqueViaA);
    let orthogonal_endpoint = matches!(
        mode,
        ContactMode::OrthogonalEndpoint | ContactMode::OrthogonalViaA
    );
    let endpoint_contact = matches!(
        mode,
        ContactMode::Endpoint
            | ContactMode::OppositeEndpoint
            | ContactMode::OrthogonalEndpoint
            | ContactMode::OrthogonalViaA
            | ContactMode::ObliqueViaB
            | ContactMode::ObliqueViaA
            | ContactMode::Non45ObliqueViaB
            | ContactMode::Non45ObliqueMirroredViaB
            | ContactMode::Non45ObliqueViaA
            | ContactMode::Non45ObliqueMirroredViaA
            | ContactMode::Non45SteepViaB
            | ContactMode::Non45SteepViaA
            | ContactMode::Non45SteepMirroredViaB
            | ContactMode::Non45SteepMirroredViaA
    );
    if bytes.is_empty()
        || bytes.len() > route_preview::MAX_FILE_BYTES
        || net.is_empty()
        || net.len() > 47
        || layers != [1, 4]
        || vias != [0, 1]
    {
        return Err("D015_SELECTION");
    }
    let p = route_preview::parse(bytes)?;
    if p.max_layer != 4
        || p.skipped != 0
        || p.vias.len() != 2
        || p.segments.len() != 4
        || !p.arcs.is_empty()
    {
        return Err("D015_SOURCE_INCOMPLETE");
    }
    let (a, b) = (&p.vias[0], &p.vias[1]);
    bounded_source_inventory(bytes, &a.name, 2, 6)?;
    if a.net != net
        || b.net != net
        || a.name != b.name
        || a.name.is_empty()
        || a.point == b.point
        || a.first_layer != 1
        || a.last_layer != 4
        || b.first_layer != 1
        || b.last_layer != 4
        || a.inner_pad
        || b.inner_pad
        || a.drill_ticks <= 0
        || a.diameter_ticks <= a.drill_ticks
        || a.drill_ticks != b.drill_ticks
        || a.diameter_ticks != b.diameter_ticks
    {
        return Err("D015_VIA_SOURCE");
    }
    let s = &p.segments;
    if (oblique || contained_overlap || partial_overlap || endpoint_contact)
        && s.iter().any(|t| {
            [t.a[0], t.a[1], t.b[0], t.b[1]]
                .iter()
                .any(|&v| !(-5_600_000..=5_600_000).contains(&v))
        })
    {
        return Err(if endpoint_contact {
            if non45_steep_mirrored_via_a {
                "D033_COORDINATE_BOUNDS"
            } else if non45_steep_mirrored_via_b {
                "D032_COORDINATE_BOUNDS"
            } else if non45_steep_via_a {
                "D031_COORDINATE_BOUNDS"
            } else if non45_steep_via_b {
                "D030_COORDINATE_BOUNDS"
            } else if non45_oblique_mirrored_via_a {
                "D035_COORDINATE_BOUNDS"
            } else if non45_oblique_via_a {
                "D029_COORDINATE_BOUNDS"
            } else if non45_oblique_mirrored_via_b {
                "D034_COORDINATE_BOUNDS"
            } else if non45_oblique_via_b {
                "D028_COORDINATE_BOUNDS"
            } else if oblique_via_a {
                "D027_COORDINATE_BOUNDS"
            } else if oblique_via_b {
                "D026_COORDINATE_BOUNDS"
            } else if orthogonal_via_a {
                "D025_COORDINATE_BOUNDS"
            } else if orthogonal_endpoint {
                "D024_COORDINATE_BOUNDS"
            } else if opposite_endpoint {
                "D023_COORDINATE_BOUNDS"
            } else {
                "D022_COORDINATE_BOUNDS"
            }
        } else if partial_overlap {
            "D021_COORDINATE_BOUNDS"
        } else if contained_overlap {
            "D020_COORDINATE_BOUNDS"
        } else {
            "D019_COORDINATE_BOUNDS"
        });
    }
    if s.iter()
        .any(|t| t.net != net || t.width_ticks <= 0 || t.a == t.b)
        || [s[0].layer, s[1].layer, s[2].layer, s[3].layer] != [1, 4, 1, 4]
        || s[0].b != a.point
        || s[1].a != a.point
        || s[1].b != b.point
        || s[2].a != b.point
        || s[0].a == b.point
        || s[2].b == a.point
        || s[0].a == s[2].b
    {
        return Err("D015_BASE_PATH");
    }
    if on_segment(b.point, s[0].a, s[0].b)
        || on_segment(a.point, s[2].a, s[2].b)
        || segments_touch(s[0].a, s[0].b, s[2].a, s[2].b)
    {
        return Err("D015_UNMODELED_LAYER1_CONTACT");
    }
    // Exact source-tick geometry: strict endpoint-to-interior T contact only.
    let branch = &s[3];
    let mx = i128::from(s[1].b[0]) - i128::from(s[1].a[0]);
    let my = i128::from(s[1].b[1]) - i128::from(s[1].a[1]);
    let bx = i128::from(branch.b[0]) - i128::from(branch.a[0]);
    let by = i128::from(branch.b[1]) - i128::from(branch.a[1]);
    let dot = mx * bx + my * by;
    let cross = mx * by - my * bx;
    let partial_inside_a = strict_interior(branch.a, s[1].a, s[1].b);
    let partial_inside_b = strict_interior(branch.b, s[1].a, s[1].b);
    let partial_external = if partial_inside_a { branch.b } else { branch.a };
    let external_projection = (i128::from(partial_external[0]) - i128::from(s[1].a[0])) * mx
        + (i128::from(partial_external[1]) - i128::from(s[1].a[1])) * my;
    let main_length_squared = mx * mx + my * my;
    // The reused main Trace ends at Via Pad B. Endpoint contact therefore
    // has three Source-owned primitives at one coordinate; model all three.
    let shared_a = branch.a == s[1].a || branch.a == s[1].b;
    let shared_b = branch.b == s[1].a || branch.b == s[1].b;
    let shared = if shared_a { branch.a } else { branch.b };
    let exterior = if shared_a { branch.b } else { branch.a };
    let exterior_projection = (i128::from(exterior[0]) - i128::from(s[1].a[0])) * mx
        + (i128::from(exterior[1]) - i128::from(s[1].a[1])) * my;
    // Canonical cross uses the Via-shared -> exterior ray; branch-record reversal
    // must not flip the signed slope classification.
    let canonical_cross = mx * (i128::from(exterior[1]) - i128::from(shared[1]))
        - my * (i128::from(exterior[0]) - i128::from(shared[0]));
    let valid_contact = if endpoint_contact {
        shared_a != shared_b
            && (if non45_steep_mirrored_via_a {
                // Canonical Via-A -> exterior ray: negative steep 3:4 only.
                shared == s[1].a
                    && dot != 0
                    && cross != 0
                    && 3 * cross.abs() == 4 * dot.abs()
                    && canonical_cross < 0
                    && exterior_projection < 0
            } else if non45_steep_mirrored_via_b {
                shared == s[1].b
                    && dot != 0
                    && cross != 0
                    && 3 * cross.abs() == 4 * dot.abs()
                    && canonical_cross < 0
                    && exterior_projection > main_length_squared
            } else if non45_steep_via_a {
                // Exact 3:4 steep non-45 Via-A exterior ray, source tick/i128 only.
                shared == s[1].a
                    && dot != 0
                    && cross != 0
                    && 3 * cross.abs() == 4 * dot.abs()
                    && canonical_cross > 0
                    && exterior_projection < 0
            } else if non45_steep_via_b {
                // D_030 positive signed ray; D_032 alone qualifies its mirror.
                shared == s[1].b
                    && dot != 0
                    && cross != 0
                    && 3 * cross.abs() == 4 * dot.abs()
                    && canonical_cross > 0
                    && exterior_projection > main_length_squared
            } else if non45_oblique_mirrored_via_a {
                // Canonical Via-A -> exterior: exact negative shallow 4:3 only.
                shared == s[1].a
                    && dot != 0
                    && cross != 0
                    && 4 * cross.abs() == 3 * dot.abs()
                    && canonical_cross < 0
                    && exterior_projection < 0
            } else if non45_oblique_via_a {
                // D_029 positive 4:3 Via-A; D_035 is its signed mirror.
                shared == s[1].a
                    && dot != 0
                    && cross != 0
                    && 4 * cross.abs() == 3 * dot.abs()
                    && canonical_cross > 0
                    && exterior_projection < 0
            } else if non45_oblique_mirrored_via_b {
                // Reverse-safe signed Via-B -> exterior, negative 4:3 only.
                shared == s[1].b
                    && dot != 0
                    && cross != 0
                    && 4 * cross.abs() == 3 * dot.abs()
                    && canonical_cross < 0
                    && exterior_projection > main_length_squared
            } else if non45_oblique_via_b {
                // D_028 positive signed mode remains separate from D_034.
                shared == s[1].b
                    && dot != 0
                    && cross != 0
                    && 4 * cross.abs() == 3 * dot.abs()
                    && canonical_cross > 0
                    && exterior_projection > main_length_squared
            } else if oblique_via_a {
                // Exact 45-degree exterior projection beyond Via-A; reversal-safe.
                // SOURCE geometry only; refuse even one tick of angle drift.
                shared == s[1].a
                    && dot != 0
                    && cross != 0
                    && dot.abs() == cross.abs()
                    && exterior_projection < 0
            } else if oblique_via_b {
                // Exact 45-degree source-tick oblique endpoint at Via-B.
                // Reversed branch is admitted; 1-tick angle drift fails closed.
                shared == s[1].b
                    && dot != 0
                    && cross != 0
                    && dot.abs() == cross.abs()
                    && exterior_projection > main_length_squared
            } else if orthogonal_endpoint {
                // Two independent endpoint Source primitives meet at Via-B.
                // Exact source-tick dot/cross; no floating tolerance, copper proof.
                shared == (if orthogonal_via_a { s[1].a } else { s[1].b }) && dot == 0 && cross != 0
            } else {
                cross == 0
                    && dot != 0
                    && (if opposite_endpoint {
                        shared == s[1].a && exterior_projection < 0
                    } else {
                        shared == s[1].b && exterior_projection > main_length_squared
                    })
            })
    } else if partial_overlap {
        // Exactly one branch endpoint strictly inside main, the other strictly
        // beyond either main endpoint; exclude endpoint-only and containment.
        partial_inside_a != partial_inside_b
            && cross == 0
            && dot != 0
            && (external_projection < 0 || external_projection > main_length_squared)
    } else if contained_overlap {
        // Both endpoints must be *strictly* inside the original main Trace.
        // Exact source-tick orientation and nonzero dot prove collinearity.
        strict_interior(branch.a, s[1].a, s[1].b)
            && strict_interior(branch.b, s[1].a, s[1].b)
            && cross == 0
            && dot != 0
    } else {
        strict_interior(branch.b, s[1].a, s[1].b)
            && !on_segment(branch.a, s[1].a, s[1].b)
            && branch.a != a.point
            && branch.a != b.point
            && branch.b != a.point
            && branch.b != b.point
            && (if oblique {
                dot != 0 && cross != 0
            } else {
                dot == 0
            })
    };
    if !valid_contact {
        return Err(if endpoint_contact {
            if non45_steep_mirrored_via_a {
                "D033_NON45_STEEP_MIRRORED_VIA_A_ENDPOINT_CONTACT"
            } else if non45_steep_mirrored_via_b {
                "D032_NON45_STEEP_MIRRORED_VIA_B_ENDPOINT_CONTACT"
            } else if non45_steep_via_a {
                "D031_NON45_STEEP_VIA_A_ENDPOINT_CONTACT"
            } else if non45_steep_via_b {
                "D030_NON45_STEEP_VIA_B_ENDPOINT_CONTACT"
            } else if non45_oblique_mirrored_via_a {
                "D035_NON45_OBLIQUE_MIRRORED_VIA_A_ENDPOINT_CONTACT"
            } else if non45_oblique_via_a {
                "D029_NON45_OBLIQUE_VIA_A_ENDPOINT_CONTACT"
            } else if non45_oblique_via_b {
                "D028_NON45_OBLIQUE_VIA_B_ENDPOINT_CONTACT"
            } else if oblique_via_a {
                "D027_OBLIQUE_VIA_A_ENDPOINT_CONTACT"
            } else if oblique_via_b {
                "D026_OBLIQUE_VIA_B_ENDPOINT_CONTACT"
            } else if orthogonal_via_a {
                "D025_ORTHOGONAL_VIA_A_ENDPOINT_CONTACT"
            } else if orthogonal_endpoint {
                "D024_ORTHOGONAL_ENDPOINT_CONTACT"
            } else if opposite_endpoint {
                "D023_COLLINEAR_OPPOSITE_ENDPOINT_CONTACT"
            } else {
                "D022_COLLINEAR_ENDPOINT_CONTACT"
            }
        } else if partial_overlap {
            "D021_PARTIAL_COLLINEAR_OVERLAP"
        } else if contained_overlap {
            "D020_COLLINEAR_CONTAINED_OVERLAP"
        } else if oblique {
            "D019_OBLIQUE_T_JUNCTION"
        } else {
            "D015_T_JUNCTION"
        });
    }

    let m = route_preview::into_canonical(bytes, &p)?;
    if m.validate().has_errors()
        || m.vias.len() != 2
        || m.traces.len() != 4
        || m.padstacks.len() != 1
        || m.drills.len() != 1
        || m.stackup.layers.len() != 4
    {
        return Err("D015_CANONICAL");
    }
    for (i, v) in m.vias.iter().enumerate() {
        if v.position.ticks != p.vias[i].point
            || v.net.is_none()
            || v.span.first_layer != m.stackup.layers[0].meta.id
            || v.span.last_layer != m.stackup.layers[3].meta.id
        {
            return Err("D015_CANONICAL_VIA");
        }
    }
    for (i, t) in m.traces.iter().enumerate() {
        if t.centreline.len() != 2
            || t.centreline[0].ticks != s[i].a
            || t.centreline[1].ticks != s[i].b
            || t.layer != m.stackup.layers[(s[i].layer - 1) as usize].meta.id
            || t.net != m.vias[0].net
        {
            return Err("D015_CANONICAL_TRACE");
        }
    }
    let mut sha = String::new();
    for byte in digest::sha256(bytes) {
        write!(&mut sha, "{byte:02x}").map_err(|_| "D015_FMT")?;
    }
    let mode = if non45_steep_mirrored_via_a {
        "D033_NON45_STEEP_MIRRORED_VIA_A_ENDPOINT_CONTACT_V1"
    } else if non45_steep_mirrored_via_b {
        "D032_NON45_STEEP_MIRRORED_VIA_B_ENDPOINT_CONTACT_V1"
    } else if non45_steep_via_a {
        "D031_NON45_STEEP_VIA_A_ENDPOINT_CONTACT_V1"
    } else if non45_steep_via_b {
        "D030_NON45_STEEP_VIA_B_ENDPOINT_CONTACT_V1"
    } else if non45_oblique_mirrored_via_a {
        "D035_NON45_OBLIQUE_MIRRORED_VIA_A_ENDPOINT_CONTACT_V1"
    } else if non45_oblique_via_a {
        "D029_NON45_OBLIQUE_VIA_A_ENDPOINT_CONTACT_V1"
    } else if non45_oblique_mirrored_via_b {
        "D034_NON45_OBLIQUE_MIRRORED_VIA_B_ENDPOINT_CONTACT_V1"
    } else if non45_oblique_via_b {
        "D028_NON45_OBLIQUE_VIA_B_ENDPOINT_CONTACT_V1"
    } else if oblique_via_a {
        "D027_OBLIQUE_VIA_A_ENDPOINT_CONTACT_V1"
    } else if oblique_via_b {
        "D026_OBLIQUE_VIA_B_ENDPOINT_CONTACT_V1"
    } else if orthogonal_via_a {
        "D025_ORTHOGONAL_VIA_A_ENDPOINT_CONTACT_V1"
    } else if orthogonal_endpoint {
        "D024_ORTHOGONAL_ENDPOINT_CONTACT_V1"
    } else if opposite_endpoint {
        "D023_COLLINEAR_OPPOSITE_ENDPOINT_CONTACT_V1"
    } else if endpoint_contact {
        "D022_COLLINEAR_ENDPOINT_CONTACT_V1"
    } else if partial_overlap {
        "D021_PARTIAL_COLLINEAR_OVERLAP_V1"
    } else if contained_overlap {
        "D020_CONTAINED_COLLINEAR_OVERLAP_V1"
    } else if oblique {
        "D019_OBLIQUE_T_JUNCTION_V1"
    } else {
        "D015_T_JUNCTION_V1"
    };
    let mut out = format!("BOARDSI_{mode}|{sha}|SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE\n");
    for (i, v) in p.vias.iter().enumerate() {
        writeln!(
            &mut out,
            "VIA|{i}|{}|{}|{}|{}|{}|{}|{}",
            v.point[0],
            v.point[1],
            v.diameter_ticks,
            v.drill_ticks,
            v.first_layer,
            v.last_layer,
            v.name
        )
        .map_err(|_| "D015_FMT")?;
    }
    for (i, t) in s.iter().enumerate() {
        writeln!(
            &mut out,
            "TRACE|{i}|{}|{}|{}|{}|{}|{}",
            t.layer, t.a[0], t.a[1], t.b[0], t.b[1], t.width_ticks
        )
        .map_err(|_| "D015_FMT")?;
    }
    if contained_overlap {
        // Report interval endpoints in main Trace direction, never fabricated length.
        let (start, end) = if dot > 0 {
            (branch.a, branch.b)
        } else {
            (branch.b, branch.a)
        };
        writeln!(
            &mut out,
            "OVERLAP|MAIN_TRACE|1|CONTAINED_TRACE|3|START|{}|{}|END|{}|{}",
            start[0], start[1], end[0], end[1]
        )
        .map_err(|_| "D020_FMT")?;
    }
    if partial_overlap {
        // Always serialize in the main Trace's direction, regardless of branch direction.
        let interior = if partial_inside_a { branch.a } else { branch.b };
        let (start, end) = if external_projection < 0 {
            (s[1].a, interior)
        } else {
            (interior, s[1].b)
        };
        writeln!(
            &mut out,
            "OVERLAP|MAIN_TRACE|1|PARTIAL_TRACE|3|START|{}|{}|END|{}|{}",
            start[0], start[1], end[0], end[1]
        )
        .map_err(|_| "D021_FMT")?;
    }
    if endpoint_contact {
        writeln!(
            &mut out,
            "CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|X|{}|Y|{}",
            shared[0], shared[1]
        )
        .map_err(|_| {
            if non45_steep_mirrored_via_a {
                "D033_FMT"
            } else if non45_steep_mirrored_via_b {
                "D032_FMT"
            } else if non45_steep_via_a {
                "D031_FMT"
            } else if non45_steep_via_b {
                "D030_FMT"
            } else if non45_oblique_mirrored_via_a {
                "D035_FMT"
            } else if non45_oblique_via_a {
                "D029_FMT"
            } else if non45_oblique_via_b {
                "D028_FMT"
            } else if oblique_via_a {
                "D027_FMT"
            } else if oblique_via_b {
                "D026_FMT"
            } else if orthogonal_via_a {
                "D025_FMT"
            } else if orthogonal_endpoint {
                "D024_FMT"
            } else if opposite_endpoint {
                "D023_FMT"
            } else {
                "D022_FMT"
            }
        })?;
    }
    let nodes = [
        ("VIA_PAD", 1, 0),
        ("VIA_PAD", 4, 0),
        ("VIA_PAD", 4, 1),
        ("VIA_PAD", 1, 1),
        ("TRACE", 1, 0),
        ("TRACE", 4, 1),
        ("TRACE", 1, 2),
        ("TRACE", 4, 3),
    ];
    // All edges are same-layer candidate contacts. NEVER a Via barrel edge.
    let t_kind = if non45_steep_mirrored_via_a {
        "SOURCE_NON45_STEEP_MIRRORED_VIA_A_ENDPOINT_CONTACT_CANDIDATE"
    } else if non45_steep_mirrored_via_b {
        "SOURCE_NON45_STEEP_MIRRORED_VIA_B_ENDPOINT_CONTACT_CANDIDATE"
    } else if non45_steep_via_a {
        "SOURCE_NON45_STEEP_VIA_A_ENDPOINT_CONTACT_CANDIDATE"
    } else if non45_steep_via_b {
        "SOURCE_NON45_STEEP_VIA_B_ENDPOINT_CONTACT_CANDIDATE"
    } else if non45_oblique_mirrored_via_a {
        "SOURCE_NON45_OBLIQUE_MIRRORED_VIA_A_ENDPOINT_CONTACT_CANDIDATE"
    } else if non45_oblique_via_a {
        "SOURCE_NON45_OBLIQUE_VIA_A_ENDPOINT_CONTACT_CANDIDATE"
    } else if non45_oblique_mirrored_via_b {
        "SOURCE_NON45_OBLIQUE_MIRRORED_VIA_B_ENDPOINT_CONTACT_CANDIDATE"
    } else if non45_oblique_via_b {
        "SOURCE_NON45_OBLIQUE_VIA_B_ENDPOINT_CONTACT_CANDIDATE"
    } else if oblique_via_a {
        "SOURCE_OBLIQUE_VIA_A_ENDPOINT_CONTACT_CANDIDATE"
    } else if oblique_via_b {
        "SOURCE_OBLIQUE_VIA_B_ENDPOINT_CONTACT_CANDIDATE"
    } else if orthogonal_via_a {
        "SOURCE_ORTHOGONAL_VIA_A_ENDPOINT_CONTACT_CANDIDATE"
    } else if orthogonal_endpoint {
        "SOURCE_ORTHOGONAL_ENDPOINT_CONTACT_CANDIDATE"
    } else if endpoint_contact {
        "SOURCE_COLLINEAR_ENDPOINT_CONTACT_CANDIDATE"
    } else if partial_overlap {
        "SOURCE_COLLINEAR_PARTIAL_OVERLAP_CANDIDATE"
    } else if contained_overlap {
        "SOURCE_COLLINEAR_CONTAINED_OVERLAP_CANDIDATE"
    } else if oblique {
        "SOURCE_INTERIOR_OBLIQUE_T_CONTACT_CANDIDATE"
    } else {
        "SOURCE_INTERIOR_T_CONTACT_CANDIDATE"
    };
    let mut edges = vec![
        (0, 4, "SOURCE_ENDPOINT_CONTACT_CANDIDATE", 1),
        (1, 5, "SOURCE_ENDPOINT_CONTACT_CANDIDATE", 4),
        (2, 5, "SOURCE_ENDPOINT_CONTACT_CANDIDATE", 4),
        (3, 6, "SOURCE_ENDPOINT_CONTACT_CANDIDATE", 1),
        (5, 7, t_kind, 4),
    ];
    if endpoint_contact {
        // Via-A Source Pad node 1 (D023/D025); Via-B pad node 2 (D022/D024).
        // This remains layer-local Source geometry, not electrical barrel proof.
        edges.push((
            if opposite_endpoint
                || orthogonal_via_a
                || oblique_via_a
                || non45_oblique_via_a
                || non45_oblique_mirrored_via_a
                || non45_steep_via_a
                || non45_steep_mirrored_via_a
            {
                1
            } else {
                2
            },
            7,
            "SOURCE_ENDPOINT_CONTACT_CANDIDATE",
            4,
        ));
    }
    let mut parent: Vec<usize> = (0..nodes.len()).collect();
    for &(a, b, _, _) in &edges {
        let ra = root(&parent, a);
        let rb = root(&parent, b);
        if ra != rb {
            parent[rb] = ra;
        } else if !endpoint_contact {
            return Err("D015_GRAPH_CYCLE");
        }
    }
    let components = (0..nodes.len()).filter(|&i| root(&parent, i) == i).count();
    for (id, (kind, layer, index)) in nodes.iter().enumerate() {
        writeln!(&mut out, "NODE|{id}|{kind}|{layer}|{index}").map_err(|_| "D015_FMT")?;
    }
    for (id, (a, b, kind, layer)) in edges.iter().enumerate() {
        writeln!(&mut out, "EDGE|{id}|{a}|{b}|{kind}|{layer}").map_err(|_| "D015_FMT")?;
    }
    writeln!(
        &mut out,
        "COUNT|NODES|{}|LAYER_LOCAL_EDGES|{}|LOCAL_COMPONENTS|{}|ELECTRICAL_CROSS_LAYER_EDGES|0",
        nodes.len(),
        edges.len(),
        components
    )
    .map_err(|_| "D015_FMT")?;
    if non45_steep_mirrored_via_a {
        out.push_str("OVERALL|NON45_STEEP_MIRRORED_VIA_A_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if non45_steep_mirrored_via_b {
        out.push_str("OVERALL|NON45_STEEP_MIRRORED_VIA_B_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if non45_steep_via_a {
        out.push_str("OVERALL|NON45_STEEP_VIA_A_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if non45_steep_via_b {
        out.push_str("OVERALL|NON45_STEEP_VIA_B_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if non45_oblique_mirrored_via_a {
        out.push_str("OVERALL|NON45_OBLIQUE_MIRRORED_VIA_A_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if non45_oblique_via_a {
        out.push_str("OVERALL|NON45_OBLIQUE_VIA_A_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if non45_oblique_mirrored_via_b {
        out.push_str("OVERALL|NON45_OBLIQUE_MIRRORED_VIA_B_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if non45_oblique_via_b {
        out.push_str("OVERALL|NON45_OBLIQUE_VIA_B_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if oblique_via_a {
        out.push_str("OVERALL|OBLIQUE_VIA_A_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if oblique_via_b {
        out.push_str("OVERALL|OBLIQUE_VIA_B_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if orthogonal_via_a {
        out.push_str("OVERALL|ORTHOGONAL_VIA_A_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if orthogonal_endpoint {
        out.push_str("OVERALL|ORTHOGONAL_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if opposite_endpoint {
        out.push_str("OVERALL|COLLINEAR_OPPOSITE_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if endpoint_contact {
        out.push_str("OVERALL|COLLINEAR_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if partial_overlap {
        out.push_str("OVERALL|COLLINEAR_PARTIAL_OVERLAP_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if contained_overlap {
        out.push_str("OVERALL|COLLINEAR_CONTAINED_OVERLAP_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else if oblique {
        out.push_str(
            "OVERALL|OBLIQUE_T_JUNCTION_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n",
        );
    } else {
        out.push_str("OVERALL|T_JUNCTION_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    }
    if out.len() > 16384 {
        return Err("D015_LIMIT");
    }
    Ok(out)
}
