//! D_014 two-Via source geometry witness, NOT electrical continuity.
use boardsi_import::{digest, route_preview};
use std::fmt::Write;

// Exact integer-grid intersection gate: the fixed three-component graph
// excludes any unmodeled contact between the two layer-1 arms.
fn orient(a: [i64; 2], b: [i64; 2], c: [i64; 2]) -> i128 {
    let dx = i128::from(b[0]) - i128::from(a[0]);
    let dy = i128::from(b[1]) - i128::from(a[1]);
    dx * (i128::from(c[1]) - i128::from(a[1])) - dy * (i128::from(c[0]) - i128::from(a[0]))
}
fn on_segment(p: [i64; 2], a: [i64; 2], b: [i64; 2]) -> bool {
    orient(a, b, p) == 0
        && p[0] >= a[0].min(b[0])
        && p[0] <= a[0].max(b[0])
        && p[1] >= a[1].min(b[1])
        && p[1] <= a[1].max(b[1])
}
fn opposite(a: i128, b: i128) -> bool {
    (a < 0 && b > 0) || (a > 0 && b < 0)
}
fn segments_touch(a: [i64; 2], b: [i64; 2], c: [i64; 2], d: [i64; 2]) -> bool {
    let x = orient(a, b, c);
    let y = orient(a, b, d);
    let z = orient(c, d, a);
    let w = orient(c, d, b);
    (x == 0 && on_segment(c, a, b))
        || (y == 0 && on_segment(d, a, b))
        || (z == 0 && on_segment(a, c, d))
        || (w == 0 && on_segment(b, c, d))
        || (opposite(x, y) && opposite(z, w))
}

pub fn encode(
    bytes: &[u8],
    net: &str,
    layers: [usize; 2],
    vias: [usize; 2],
) -> Result<String, &'static str> {
    if bytes.is_empty()
        || bytes.len() > route_preview::MAX_FILE_BYTES
        || net.is_empty()
        || net.len() > 47
        || layers != [1, 4]
        || vias != [0, 1]
    {
        return Err("D014_SELECTION");
    }
    let p = route_preview::parse(bytes)?;
    if p.max_layer != 4
        || p.skipped != 0
        || p.vias.len() != 2
        || p.segments.len() != 3
        || !p.arcs.is_empty()
    {
        return Err("D014_ROUTE_INCOMPLETE");
    }
    let (a, b) = (&p.vias[0], &p.vias[1]);
    if a.net != net
        || b.net != net
        || a.name != b.name
        || a.point == b.point
        || a.first_layer != 1
        || a.last_layer != 4
        || b.first_layer != 1
        || b.last_layer != 4
        || a.inner_pad
        || b.inner_pad
        || a.drill_ticks <= 0
        || a.diameter_ticks <= a.drill_ticks
        || a.diameter_ticks != b.diameter_ticks
        || a.drill_ticks != b.drill_ticks
    {
        return Err("D014_VIA_SOURCE");
    }
    let s = &p.segments;
    if s.iter()
        .any(|t| t.net != net || t.width_ticks <= 0 || t.a == t.b)
        || [s[0].layer, s[1].layer, s[2].layer] != [1, 4, 1]
        || s[0].b != a.point
        || s[1].a != a.point
        || s[1].b != b.point
        || s[2].a != b.point
        || s[0].a == b.point
        || s[2].b == a.point
        || s[0].a == s[2].b
    {
        return Err("D014_CONTACT_PATH");
    }
    if on_segment(b.point, s[0].a, s[0].b)
        || on_segment(a.point, s[2].a, s[2].b)
        || segments_touch(s[0].a, s[0].b, s[2].a, s[2].b)
    {
        return Err("D014_UNMODELED_LAYER1_CONTACT");
    }
    let m = route_preview::into_canonical(bytes, &p)?;
    if m.validate().has_errors()
        || m.vias.len() != 2
        || m.traces.len() != 3
        || m.padstacks.len() != 1
        || m.drills.len() != 1
        || m.stackup.layers.len() != 4
    {
        return Err("D014_CANONICAL");
    }
    for (i, v) in m.vias.iter().enumerate() {
        if v.position.ticks != p.vias[i].point
            || v.net.is_none()
            || v.span.first_layer != m.stackup.layers[0].meta.id
            || v.span.last_layer != m.stackup.layers[3].meta.id
        {
            return Err("D014_CANONICAL_VIA");
        }
    }
    for (i, t) in m.traces.iter().enumerate() {
        if t.centreline.len() != 2
            || t.centreline[0].ticks != s[i].a
            || t.centreline[1].ticks != s[i].b
            || t.layer != m.stackup.layers[(s[i].layer - 1) as usize].meta.id
            || t.net != m.vias[0].net
        {
            return Err("D014_CANONICAL_TRACE");
        }
    }
    let mut sha = String::new();
    for b in digest::sha256(bytes) {
        write!(&mut sha, "{b:02x}").map_err(|_| "D014_FMT")?;
    }
    let mut out=format!("BOARDSI_D014_TWO_VIA_CHAIN_V1|{sha}|SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE\n");
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
        .map_err(|_| "D014_FMT")?;
    }
    for (i, t) in s.iter().enumerate() {
        writeln!(
            &mut out,
            "TRACE|{i}|{}|{}|{}|{}|{}|{}",
            t.layer, t.a[0], t.a[1], t.b[0], t.b[1], t.width_ticks
        )
        .map_err(|_| "D014_FMT")?;
    }
    for (id, kind, layer, index) in [
        (0, "VIA_PAD", 1, 0),
        (1, "VIA_PAD", 4, 0),
        (2, "VIA_PAD", 4, 1),
        (3, "VIA_PAD", 1, 1),
        (4, "TRACE", 1, 0),
        (5, "TRACE", 4, 1),
        (6, "TRACE", 1, 2),
    ] {
        writeln!(&mut out, "NODE|{id}|{kind}|{layer}|{index}").map_err(|_| "D014_FMT")?;
    }
    for (id, a, b, layer) in [(0, 0, 4, 1), (1, 1, 5, 4), (2, 2, 5, 4), (3, 3, 6, 1)] {
        writeln!(
            &mut out,
            "EDGE|{id}|{a}|{b}|SOURCE_ENDPOINT_CONTACT_CANDIDATE|{layer}"
        )
        .map_err(|_| "D014_FMT")?;
    }
    out.push_str("COUNT|NODES|7|LAYER_LOCAL_EDGES|4|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0\nOVERALL|TWO_VIA_SOURCE_CHAIN_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    if out.len() > 16384 {
        return Err("D014_LIMIT");
    }
    Ok(out)
}
