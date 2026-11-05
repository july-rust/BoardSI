//! D_038: independently parsed two-branch mixed-sign 4:3 Via-A/Via-B SOURCE
//! contacts in ONE PADS V9.4 MILS board. Geometry candidates, not conduction.
use boardsi_import::{digest, route_preview};
use std::fmt::Write;

fn candidate(
    shared: [i64; 2],
    a: [i64; 2],
    b: [i64; 2],
    main_a: [i64; 2],
    main_b: [i64; 2],
    via_a: bool,
) -> bool {
    // Canonical Via->exterior orientation is independent of SOURCE record order.
    let exterior = if a == shared && b != shared {
        b
    } else if b == shared && a != shared {
        a
    } else {
        return false;
    };
    let mx = i128::from(main_b[0]) - i128::from(main_a[0]);
    let my = i128::from(main_b[1]) - i128::from(main_a[1]);
    let dx = i128::from(exterior[0]) - i128::from(shared[0]);
    let dy = i128::from(exterior[1]) - i128::from(shared[1]);
    let dot = mx * dx + my * dy;
    let cross = mx * dy - my * dx;
    let start_x = i128::from(exterior[0]) - i128::from(main_a[0]);
    let start_y = i128::from(exterior[1]) - i128::from(main_a[1]);
    let projection = start_x * mx + start_y * my;
    let len2 = mx * mx + my * my;
    dot != 0
        && (if via_a { cross > 0 } else { cross < 0 })
        && 4 * cross.abs() == 3 * dot.abs()
        && if via_a {
            shared == main_a && projection < 0
        } else {
            shared == main_b && projection > len2
        }
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
        return Err("D038_SELECTION");
    }
    let p = route_preview::parse(bytes)?;
    if p.max_layer != 4
        || p.skipped != 0
        || p.vias.len() != 2
        || p.segments.len() != 5
        || !p.arcs.is_empty()
    {
        return Err("D038_SOURCE_INCOMPLETE");
    }
    let (a, b) = (&p.vias[0], &p.vias[1]);
    super::d015_core::bounded_source_inventory(bytes, &a.name, 3, 8)?;
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
        return Err("D038_VIA_SOURCE");
    }
    let s = &p.segments;
    if s.iter().any(|t| {
        t.net != net
            || t.width_ticks <= 0
            || t.a == t.b
            || [t.a[0], t.a[1], t.b[0], t.b[1]]
                .iter()
                .any(|&v| !(-5_600_000..=5_600_000).contains(&v))
    }) || s.iter().map(|t| t.layer).collect::<Vec<_>>() != [1, 4, 1, 4, 4]
        || s[0].b != a.point
        || s[1].a != a.point
        || s[1].b != b.point
        || s[2].a != b.point
        || s[0].a == b.point
        || s[2].b == a.point
        || s[0].a == s[2].b
    {
        return Err("D038_BASE_PATH");
    }
    // Full exact source-tick intersection, not a horizontal bounding-box proxy.
    // The two Layer-1 arms remain separate components in the candidate graph.
    if super::d015_core::on_segment(b.point, s[0].a, s[0].b)
        || super::d015_core::on_segment(a.point, s[2].a, s[2].b)
        || super::d015_core::segments_touch(s[0].a, s[0].b, s[2].a, s[2].b)
    {
        return Err("D038_UNMODELED_LAYER1_CONTACT");
    }
    if !candidate(a.point, s[3].a, s[3].b, s[1].a, s[1].b, true)
        || !candidate(b.point, s[4].a, s[4].b, s[1].a, s[1].b, false)
    {
        return Err("D038_SIGNED_DUAL_ENDPOINT");
    }
    let m = route_preview::into_canonical(bytes, &p)?;
    if m.validate().has_errors()
        || m.vias.len() != 2
        || m.traces.len() != 5
        || m.padstacks.len() != 1
        || m.drills.len() != 1
        || m.stackup.layers.len() != 4
    {
        return Err("D038_CANONICAL");
    }
    for (i, v) in m.vias.iter().enumerate() {
        if v.position.ticks != p.vias[i].point
            || v.net.is_none()
            || v.span.first_layer != m.stackup.layers[0].meta.id
            || v.span.last_layer != m.stackup.layers[3].meta.id
        {
            return Err("D038_CANONICAL_VIA");
        }
    }
    for (i, t) in m.traces.iter().enumerate() {
        if t.centreline.len() != 2
            || t.centreline[0].ticks != s[i].a
            || t.centreline[1].ticks != s[i].b
            || t.layer != m.stackup.layers[(s[i].layer - 1) as usize].meta.id
            || t.net != m.vias[0].net
        {
            return Err("D038_CANONICAL_TRACE");
        }
    }
    let mut sha = String::new();
    for byte in digest::sha256(bytes) {
        write!(&mut sha, "{byte:02x}").map_err(|_| "D038_FORMAT")?;
    }
    let mut out = format!(
        "BOARDSI_D038_DUAL_MIXED_SIGN_VIA_ENDPOINT_CONTACT_V1|{sha}|SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE\n"
    );
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
        .map_err(|_| "D038_FORMAT")?;
    }
    for (i, t) in s.iter().enumerate() {
        writeln!(
            &mut out,
            "TRACE|{i}|{}|{}|{}|{}|{}|{}",
            t.layer, t.a[0], t.a[1], t.b[0], t.b[1], t.width_ticks
        )
        .map_err(|_| "D038_FORMAT")?;
    }
    writeln!(
        &mut out,
        "CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|VIA|A|X|{}|Y|{}",
        a.point[0], a.point[1]
    )
    .map_err(|_| "D038_FORMAT")?;
    writeln!(
        &mut out,
        "CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|4|VIA|B|X|{}|Y|{}",
        b.point[0], b.point[1]
    )
    .map_err(|_| "D038_FORMAT")?;
    // ID is source-primitive identity (not an electrical node).
    let nodes = [
        ("VIA_PAD", 1, 0),
        ("VIA_PAD", 4, 0),
        ("VIA_PAD", 4, 1),
        ("VIA_PAD", 1, 1),
        ("TRACE", 1, 0),
        ("TRACE", 4, 1),
        ("TRACE", 1, 2),
        ("TRACE", 4, 3),
        ("TRACE", 4, 4),
    ];
    let edges = [
        (0, 4, "SOURCE_ENDPOINT_CONTACT_CANDIDATE", 1),
        (1, 5, "SOURCE_ENDPOINT_CONTACT_CANDIDATE", 4),
        (2, 5, "SOURCE_ENDPOINT_CONTACT_CANDIDATE", 4),
        (3, 6, "SOURCE_ENDPOINT_CONTACT_CANDIDATE", 1),
        (
            5,
            7,
            "SOURCE_NON45_OBLIQUE_VIA_A_ENDPOINT_CONTACT_CANDIDATE",
            4,
        ),
        (1, 7, "SOURCE_ENDPOINT_CONTACT_CANDIDATE", 4),
        (
            5,
            8,
            "SOURCE_NON45_OBLIQUE_MIRRORED_VIA_B_ENDPOINT_CONTACT_CANDIDATE",
            4,
        ),
        (2, 8, "SOURCE_ENDPOINT_CONTACT_CANDIDATE", 4),
    ];
    let mut parent: Vec<usize> = (0..nodes.len()).collect();
    for &(u, v, _, _) in &edges {
        fn root(parent: &[usize], mut id: usize) -> usize {
            while parent[id] != id {
                id = parent[id];
            }
            id
        }
        let ru = root(&parent, u);
        let rv = root(&parent, v);
        if ru != rv {
            parent[rv] = ru;
        }
    }
    let components = (0..nodes.len())
        .filter(|&i| {
            let mut k = i;
            while parent[k] != k {
                k = parent[k];
            }
            k == i
        })
        .count();
    if components != 3 {
        return Err("D038_COMPONENT_INVARIANT");
    }
    for (id, (kind, layer, index)) in nodes.iter().enumerate() {
        writeln!(&mut out, "NODE|{id}|{kind}|{layer}|{index}").map_err(|_| "D038_FORMAT")?;
    }
    for (id, (u, v, kind, layer)) in edges.iter().enumerate() {
        writeln!(&mut out, "EDGE|{id}|{u}|{v}|{kind}|{layer}").map_err(|_| "D038_FORMAT")?;
    }
    writeln!(
        &mut out,
        "COUNT|NODES|{}|LAYER_LOCAL_EDGES|{}|LOCAL_COMPONENTS|{}|ELECTRICAL_CROSS_LAYER_EDGES|0",
        nodes.len(),
        edges.len(),
        components
    )
    .map_err(|_| "D038_FORMAT")?;
    out.push_str("OVERALL|DUAL_MIXED_SIGN_POS_A_NEG_B_4_3_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n");
    if out.len() > 16384 {
        return Err("D038_LIMIT");
    }
    Ok(out)
}
