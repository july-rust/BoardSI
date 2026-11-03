//! D_016 bounded perpendicular source-tick X-junction on D_014 two-Via topology.
//! These are geometry CONTACT CANDIDATES, never fabricated electrical edges.
use boardsi_import::{digest, route_preview};
use std::fmt::Write;

fn orient(a: [i64; 2], b: [i64; 2], c: [i64; 2]) -> i128 {
    let x = i128::from(b[0]) - i128::from(a[0]);
    let y = i128::from(b[1]) - i128::from(a[1]);
    x * (i128::from(c[1]) - i128::from(a[1])) - y * (i128::from(c[0]) - i128::from(a[0]))
}
fn on_segment(p: [i64; 2], a: [i64; 2], b: [i64; 2]) -> bool {
    orient(a, b, p) == 0
        && p[0] >= a[0].min(b[0])
        && p[0] <= a[0].max(b[0])
        && p[1] >= a[1].min(b[1])
        && p[1] <= a[1].max(b[1])
}
fn segments_touch(a: [i64; 2], b: [i64; 2], c: [i64; 2], d: [i64; 2]) -> bool {
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
fn bounded_source_inventory(bytes: &[u8], via_name: &str) -> Result<(), &'static str> {
    let source = std::str::from_utf8(bytes).map_err(|_| "D016_SOURCE_TEXT")?;
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
            return Err("D016_TRAILING_SOURCE");
        }
        if !header {
            if !line.starts_with("!PADS-POWERPCB-V9.4-MILS!") {
                return Err("D016_HEADER");
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
            return Err("D016_UNMODELED_SECTION");
        }
        match section {
            "pcb" if words.len() == 2 && words[0] == "MAXLAYER" && words[1] == "4" => {
                max_layer += 1;
                if max_layer != 1 {
                    return Err("D016_DUPLICATE_LAYER");
                }
            }
            "via" if first == via_name && words.len() == 3 && words[2] == "2" => {
                via_defs += 1;
                if via_defs != 1 {
                    return Err("D016_DUPLICATE_VIA_DEF");
                }
            }
            "via" if words.len() == 3 && (first == "-2" || first == "0") => {
                if first == "-2" {
                    if bottom_pad {
                        return Err("D016_DUPLICATE_PAD_ROW");
                    }
                    bottom_pad = true;
                } else {
                    if top_pad {
                        return Err("D016_DUPLICATE_PAD_ROW");
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
                    return Err("D016_EXTRA_ROUTE_VIA");
                }
                route_points += 1;
            }
            _ => return Err("D016_UNMODELED_SOURCE"),
        }
    }
    if !header
        || !ended
        || max_layer != 1
        || via_defs != 1
        || pad_rows != 2
        || !bottom_pad
        || !top_pad
        || signals != 2
        || route_pairs != 2
        || route_points != 6
    {
        return Err("D016_SOURCE_STRUCTURE");
    }
    Ok(())
}

pub fn encode(
    bytes: &[u8], net: &str, layers: [usize; 2], vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, false)
}
// D_017 reuses D_016's strict inventory/canonical/contact graph with the
// complementary non-perpendicular intersection rule and distinct receipt.
pub fn encode_oblique(
    bytes: &[u8], net: &str, layers: [usize; 2], vias: [usize; 2],
) -> Result<String, &'static str> {
    encode_with_mode(bytes, net, layers, vias, true)
}
fn encode_with_mode(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
    oblique: bool,
) -> Result<String, &'static str> {
    if bytes.is_empty()
        || bytes.len() > route_preview::MAX_FILE_BYTES
        || net.is_empty()
        || net.len() > 47
        || layers != [1, 4]
        || vias != [0, 1]
    {
        return Err("D016_SELECTION");
    }
    let p = route_preview::parse(bytes)?;
    if p.max_layer != 4
        || p.skipped != 0
        || p.vias.len() != 2
        || p.segments.len() != 4
        || !p.arcs.is_empty()
    {
        return Err("D016_SOURCE_INCOMPLETE");
    }
    let (a, b) = (&p.vias[0], &p.vias[1]);
    bounded_source_inventory(bytes, &a.name)?;
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
        return Err("D016_VIA_SOURCE");
    }
    let s = &p.segments;
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
        return Err("D016_BASE_PATH");
    }
    if on_segment(b.point, s[0].a, s[0].b)
        || on_segment(a.point, s[2].a, s[2].b)
        || segments_touch(s[0].a, s[0].b, s[2].a, s[2].b)
    {
        return Err("D016_UNMODELED_LAYER1_CONTACT");
    }
    // D_017 arithmetic remains bounded before i128 orientation/dot products.
    // The existing D_016 path and its acceptance rules are unchanged.
    if oblique && s.iter().any(|t| {
        [t.a[0], t.a[1], t.b[0], t.b[1]]
            .iter().any(|&v| !(-5_600_000..=5_600_000).contains(&v))
    }) {
        return Err("D017_COORDINATE_BOUNDS");
    }
    // D_016 perpendicular or D_017 strictly oblique interior–interior.
    // Strict orientation
    // rejects T-junction endpoints, tangency, collinearity, and near-misses.
    // This is a SOURCE GEOMETRY candidate, NEVER an electrical connection.
    let branch = &s[3];
    let opposite = |x: i128, y: i128| (x < 0 && y > 0) || (x > 0 && y < 0);
    let main_a = orient(s[1].a, s[1].b, branch.a);
    let main_b = orient(s[1].a, s[1].b, branch.b);
    let branch_a = orient(branch.a, branch.b, s[1].a);
    let branch_b = orient(branch.a, branch.b, s[1].b);
    let mx = i128::from(s[1].b[0]) - i128::from(s[1].a[0]);
    let my = i128::from(s[1].b[1]) - i128::from(s[1].a[1]);
    let bx = i128::from(branch.b[0]) - i128::from(branch.a[0]);
    let by = i128::from(branch.b[1]) - i128::from(branch.a[1]);
    if !opposite(main_a, main_b)
        || !opposite(branch_a, branch_b)
        || (if oblique { mx * bx + my * by == 0 } else { mx * bx + my * by != 0 })
        || on_segment(branch.a, s[1].a, s[1].b)
        || on_segment(branch.b, s[1].a, s[1].b)
    {
        return Err(if oblique { "D017_OBLIQUE_X_JUNCTION" } else { "D016_X_JUNCTION" });
    }

    let m = route_preview::into_canonical(bytes, &p)?;
    if m.validate().has_errors()
        || m.vias.len() != 2
        || m.traces.len() != 4
        || m.padstacks.len() != 1
        || m.drills.len() != 1
        || m.stackup.layers.len() != 4
    {
        return Err("D016_CANONICAL");
    }
    for (i, v) in m.vias.iter().enumerate() {
        if v.position.ticks != p.vias[i].point
            || v.net.is_none()
            || v.span.first_layer != m.stackup.layers[0].meta.id
            || v.span.last_layer != m.stackup.layers[3].meta.id
        {
            return Err("D016_CANONICAL_VIA");
        }
    }
    for (i, t) in m.traces.iter().enumerate() {
        if t.centreline.len() != 2
            || t.centreline[0].ticks != s[i].a
            || t.centreline[1].ticks != s[i].b
            || t.layer != m.stackup.layers[(s[i].layer - 1) as usize].meta.id
            || t.net != m.vias[0].net
        {
            return Err("D016_CANONICAL_TRACE");
        }
    }
    let mut sha = String::new();
    for byte in digest::sha256(bytes) {
        write!(&mut sha, "{byte:02x}").map_err(|_| "D016_FMT")?;
    }
    let mode = if oblique { "D017_OBLIQUE_X_JUNCTION_V1" } else { "D016_X_JUNCTION_V1" };
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
        .map_err(|_| "D016_FMT")?;
    }
    for (i, t) in s.iter().enumerate() {
        writeln!(
            &mut out,
            "TRACE|{i}|{}|{}|{}|{}|{}|{}",
            t.layer, t.a[0], t.a[1], t.b[0], t.b[1], t.width_ticks
        )
        .map_err(|_| "D016_FMT")?;
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
    let x_kind = if oblique { "SOURCE_INTERIOR_OBLIQUE_X_CONTACT_CANDIDATE" } else { "SOURCE_INTERIOR_X_CONTACT_CANDIDATE" };
    let edges = [
        (0, 4, "SOURCE_ENDPOINX_CONTACT_CANDIDATE", 1),
        (1, 5, "SOURCE_ENDPOINX_CONTACT_CANDIDATE", 4),
        (2, 5, "SOURCE_ENDPOINX_CONTACT_CANDIDATE", 4),
        (3, 6, "SOURCE_ENDPOINX_CONTACT_CANDIDATE", 1),
        (5, 7, x_kind, 4),
    ];
    let mut parent: Vec<usize> = (0..nodes.len()).collect();
    for (a, b, _, _) in edges {
        let ra = root(&parent, a);
        let rb = root(&parent, b);
        if ra == rb {
            return Err("D016_GRAPH_CYCLE");
        }
        parent[rb] = ra;
    }
    let components = (0..nodes.len()).filter(|&i| root(&parent, i) == i).count();
    for (id, (kind, layer, index)) in nodes.iter().enumerate() {
        writeln!(&mut out, "NODE|{id}|{kind}|{layer}|{index}").map_err(|_| "D016_FMT")?;
    }
    for (id, (a, b, kind, layer)) in edges.iter().enumerate() {
        writeln!(&mut out, "EDGE|{id}|{a}|{b}|{kind}|{layer}").map_err(|_| "D016_FMT")?;
    }
    writeln!(
        &mut out,
        "COUNT|NODES|{}|LAYER_LOCAL_EDGES|{}|LOCAL_COMPONENTS|{}|ELECTRICAL_CROSS_LAYER_EDGES|0",
        nodes.len(),
        edges.len(),
        components
    )
    .map_err(|_| "D016_FMT")?;
    if oblique {
        out.push_str("OVERALL|OBLIQUE_X_JUNCTION_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    } else {
        out.push_str("OVERALL|X_JUNCTION_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    }
    if out.len() > 16384 {
        return Err("D016_LIMIT");
    }
    Ok(out)
}
