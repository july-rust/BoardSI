//! D_051: SOURCE-local second finite-width mixed-pad bridge and virtual cut-set.
//! Candidate same-layer copper geometry only; NOT manufactured/global electrical proof.
use crate::{d050_mixed_pad_width_bridge, digest, route_preview};
use std::fmt::Write;

fn reduced_without_route(rows: &[&str], index: usize) -> Result<Vec<u8>, &'static str> {
    if index + 2 >= rows.len()
        || rows[index + 1].split_ascii_whitespace().count() != 5
        || rows[index + 2].split_ascii_whitespace().count() != 5
    {
        return Err("D051_ROUTE_VERTICES");
    }
    let mut out = String::new();
    for (i, row) in rows.iter().enumerate() {
        if (index..index + 3).contains(&i) {
            continue;
        }
        out.push_str(row);
        out.push('\n');
    }
    Ok(out.into_bytes())
}

fn track_from_receipt(row: &str) -> Result<route_preview::Segment, &'static str> {
    let f: Vec<_> = row.split('|').collect();
    if f.len() != 9 || f[0] != "T" {
        return Err("D051_BASE_TRACK_RECEIPT");
    }
    Ok(route_preview::Segment {
        layer: f[2].parse().map_err(|_| "D051_TRACK_LAYER")?,
        net: f[3].to_owned(),
        a: [
            f[4].parse().map_err(|_| "D051_TRACK_X0")?,
            f[5].parse().map_err(|_| "D051_TRACK_Y0")?,
        ],
        b: [
            f[6].parse().map_err(|_| "D051_TRACK_X1")?,
            f[7].parse().map_err(|_| "D051_TRACK_Y1")?,
        ],
        width_ticks: f[8].parse().map_err(|_| "D051_TRACK_WIDTH")?,
        source_line: 0,
    })
}

fn same_track(a: &route_preview::Segment, b: &route_preview::Segment) -> bool {
    a.layer == b.layer
        && a.net == b.net
        && a.width_ticks == b.width_ticks
        && ((a.a == b.a && a.b == b.b) || (a.a == b.b && a.b == b.a))
}

/// The horizontal segment is bounded to one round-capped contact on the
/// inherited square supplementary trace and one on the U2.2 square pad.
/// All coordinates are integer 254-nm SOURCE ticks, not AABB approximations.
fn second_bridge_geometry(t: &route_preview::Segment, net: &str) -> bool {
    if t.layer != 1 || t.net != net || t.width_ticks != 800 {
        return false;
    }
    let (a, b) = if t.a[0] < t.b[0] {
        (t.a, t.b)
    } else {
        (t.b, t.a)
    };
    if a[1] != 40_000 || b[1] != 40_000 || a[0] >= b[0] {
        return false;
    }
    let left_distance = i128::from(a[0] - 21_700);
    let square_distance = i128::from(38_500 - b[0]);
    // Strict left end: NO direct U1.2 pad contact; supplementary
    // x=21700 width=800 contact is positive only when distance <800.
    // Strict right end: nearest U2.2 pad rectangle x=38500;
    // round-cap radius 400 must cross the rectangular left boundary.
    a[0] > 21_900
        && a[0] < 22_500
        && b[0] > 38_100
        && b[0] < 38_500
        && left_distance * left_distance < 800_i128 * 800
        && square_distance * square_distance < 400_i128 * 400
}

/// Graph diagnostics use SOURCE-qualified edges, not net labels.
/// Node 10 = first bridge; node 11 = second bridge. Both are virtual
/// failures and do not claim physical copper removal.
fn graph_diagnostic(
    edges: &[(usize, usize)],
    remove_first: bool,
    remove_second: bool,
) -> Result<(usize, usize, usize, usize, bool), &'static str> {
    let removed = |node: usize| {
        (remove_first && node == 10) || (remove_second && node == 11)
    };
    let mut roots: [usize; 12] = std::array::from_fn(|i| i);
    let find = |roots: &[usize; 12], mut x: usize| {
        while roots[x] != x {
            x = roots[x];
        }
        x
    };
    let mut count_edges = 0_usize;
    for &(a, b) in edges {
        if a >= 12 || b >= 12 || a == b {
            return Err("D051_GRAPH_EDGE");
        }
        if removed(a) || removed(b) {
            continue;
        }
        let ra = find(&roots, a);
        let rb = find(&roots, b);
        if ra != rb {
            roots[ra] = rb;
        }
        count_edges += 1;
    }
    let mut seen = Vec::new();
    for node in 0..12 {
        if !removed(node) {
            let root = find(&roots, node);
            if !seen.contains(&root) {
                seen.push(root);
            }
        }
    }
    let count_nodes = 12 - usize::from(remove_first) - usize::from(remove_second);
    let cycle_rank = count_edges
        .checked_add(seen.len())
        .and_then(|sum| sum.checked_sub(count_nodes))
        .ok_or("D051_GRAPH_CYCLE")?;
    Ok((
        count_nodes,
        count_edges,
        seen.len(),
        cycle_rank,
        find(&roots, 0) == find(&roots, 2),
    ))
}

pub fn encode(bytes: &[u8]) -> Result<String, &'static str> {
    if bytes.is_empty() || bytes.len() > route_preview::MAX_FILE_BYTES {
        return Err("D051_SOURCE_SIZE");
    }
    let complete = route_preview::parse(bytes)?;
    if complete.skipped != 0
        || complete.segments.len() != 8
        || !complete.vias.is_empty()
        || !complete.arcs.is_empty()
    {
        return Err("D051_SOURCE_INVENTORY");
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "D051_UTF8")?;
    let rows: Vec<_> = text.lines().collect();
    let mut in_signal = false;
    let mut selected: Option<(Vec<u8>, String)> = None;
    for i in 0..rows.len() {
        let fields: Vec<_> = rows[i].split_ascii_whitespace().collect();
        if fields.first() == Some(&"*SIGNAL*") {
            in_signal = true;
        } else if fields.first() == Some(&"*END*") {
            in_signal = false;
        }
        if in_signal && fields == ["U1.2", "U2.2"] {
            let reduced = reduced_without_route(&rows, i)?;
            if let Ok(d050) = d050_mixed_pad_width_bridge::encode(&reduced) {
                if selected.is_some() {
                    return Err("D051_AMBIGUOUS_SECOND_ROUTE");
                }
                selected = Some((reduced, d050));
            }
        }
    }
    let (reduced, old_wire) = selected.ok_or("D051_SECOND_BRIDGE_MISSING")?;
    let baseline = route_preview::parse(&reduced)?;
    if baseline.skipped != 0 || baseline.segments.len() != 7 {
        return Err("D051_REDUCED_INVENTORY");
    }
    let old_tracks: Vec<_> = old_wire
        .lines()
        .filter(|line| line.starts_with("T|"))
        .map(track_from_receipt)
        .collect::<Result<_, _>>()?;
    if old_tracks.len() != 7 {
        return Err("D051_BASE_INVENTORY");
    }
    let mut used = [false; 8];
    for expected in &old_tracks {
        let matches: Vec<_> = complete
            .segments
            .iter()
            .enumerate()
            .filter(|(i, actual)| !used[*i] && same_track(actual, expected))
            .map(|(i, _)| i)
            .collect();
        if matches.len() != 1 {
            return Err("D051_SOURCE_ONE_TO_ONE");
        }
        used[matches[0]] = true;
    }
    let new_indices: Vec<_> = used
        .iter()
        .enumerate()
        .filter_map(|(i, matched)| if *matched { None } else { Some(i) })
        .collect();
    if new_indices.len() != 1 {
        return Err("D051_SECOND_ROUTE_IDENTITY");
    }
    let bridge = &complete.segments[new_indices[0]];
    let net = &old_tracks[0].net;
    if !second_bridge_geometry(bridge, net) {
        return Err("D051_SECOND_BRIDGE_GEOMETRY");
    }
    let (a, b) = if bridge.a[0] < bridge.b[0] {
        (bridge.a, bridge.b)
    } else {
        (bridge.b, bridge.a)
    };

    let mut edges = Vec::<(usize, usize)>::new();
    for line in old_wire.lines().filter(|line| line.starts_with("E|")) {
        let f: Vec<_> = line.split('|').collect();
        if f.len() != 5 || f[0] != "E" || f[1] != edges.len().to_string() {
            return Err("D051_BASE_EDGE_ORDER");
        }
        edges.push((
            f[2].parse().map_err(|_| "D051_BASE_EDGE_A")?,
            f[3].parse().map_err(|_| "D051_BASE_EDGE_B")?,
        ));
    }
    if edges.len() != 10 {
        return Err("D051_BASE_EDGE_COUNT");
    }
    edges.push((9, 11));
    edges.push((3, 11));
    let expected = [
        (false, false, 12, 12, 1, 1, true),
        (true, false, 11, 10, 1, 0, true),
        (false, true, 11, 10, 1, 0, true),
        (true, true, 10, 8, 2, 0, false),
    ];
    for &(drop_a, drop_b, nodes, links, groups, cycles, reach) in &expected {
        if graph_diagnostic(&edges, drop_a, drop_b)? != (nodes, links, groups, cycles, reach) {
            return Err("D051_REDUNDANCY_TOPOLOGY");
        }
    }
    let mut sha = String::new();
    for octet in digest::sha256(bytes) {
        write!(&mut sha, "{octet:02x}").map_err(|_| "D051_SHA")?;
    }
    let mut wire = format!(
        "BOARDSI_D051_DUAL_FINITE_WIDTH_LOOP_V1|{sha}|1|SOURCE_LOCAL_DUAL_FINITE_WIDTH_REDUNDANCY_ONLY|P009_UNSUPPORTED|SI_NOT_RUN|GLOBAL_ELECTRICAL_NOT_VERIFIED\n"
    );
    writeln!(&mut wire, "C|0|U1_U2|{net}|1|12|12").map_err(|_| "D051_FORMAT")?;
    for line in old_wire
        .lines()
        .filter(|line| line.starts_with("P|") || line.starts_with("T|"))
    {
        writeln!(&mut wire, "{line}").map_err(|_| "D051_FORMAT")?;
    }
    writeln!(
        &mut wire,
        "T|7|1|{net}|{}|{}|{}|{}|800",
        a[0], a[1], b[0], b[1]
    )
    .map_err(|_| "D051_FORMAT")?;
    for line in old_wire.lines().filter(|line| line.starts_with("E|")) {
        writeln!(&mut wire, "{line}").map_err(|_| "D051_FORMAT")?;
    }
    wire.push_str(
        "E|10|9|11|POSITIVE_FINITE_WIDTH_TRACE_TRACE_SECOND_BRIDGE\n\
         E|11|3|11|POSITIVE_FINITE_WIDTH_TRACE_SQUARE_PAD_SECOND_BRIDGE\n",
    );
    for &(drop_a, drop_b, _, _, _, _, _) in &expected {
        let (n, e, c, cycles, connected) = graph_diagnostic(&edges, drop_a, drop_b)?;
        let name = match (drop_a, drop_b) {
            (false, false) => "NONE",
            (true, false) => "FIRST",
            (false, true) => "SECOND",
            (true, true) => "BOTH",
        };
        writeln!(
            &mut wire,
            "D|{name}|{n}|{e}|{c}|{cycles}|{}",
            if connected { "REACHABLE" } else { "SEPARATED" }
        )
        .map_err(|_| "D051_FORMAT")?;
    }
    wire.push_str(
        "COUNT|NODES|12|EDGES|12|LOCAL_COMPONENTS|1|CYCLE_RANK|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n\
         END|SOURCE_LOCAL_DUAL_FINITE_WIDTH_REDUNDANCY_ONLY_NO_GLOBAL_PASS\n",
    );
    Ok(wire)
}
