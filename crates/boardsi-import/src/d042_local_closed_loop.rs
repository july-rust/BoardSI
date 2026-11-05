//! D_042: an exact synthetic SOURCE local closed-loop witness; never global electrical proof.
use crate::{d041_local_bridge, digest, route_preview};
use std::fmt::Write;

fn strip_detour(bytes: &[u8]) -> Result<Vec<u8>, &'static str> {
    let text = std::str::from_utf8(bytes).map_err(|_| "D042_UTF8")?;
    let rows: Vec<_> = text.lines().collect();
    let (mut i, mut signals, mut detours) = (0, 0, 0);
    let mut in_signal = false;
    let mut base = String::new();
    while i < rows.len() {
        let words: Vec<_> = rows[i].split_ascii_whitespace().collect();
        if words.first() == Some(&"*SIGNAL*") {
            signals += 1;
            in_signal = true;
        } else if words.first() == Some(&"*END*") {
            in_signal = false;
        }
        if words == ["U1.2", "U2.2"] {
            if !in_signal
                || i + 2 >= rows.len()
                || rows[i + 1].split_ascii_whitespace().count() != 5
                || rows[i + 2].split_ascii_whitespace().count() != 5
            {
                return Err("D042_DETOUR_SOURCE");
            }
            detours += 1;
            i += 3;
        } else {
            base.push_str(rows[i]);
            base.push('\n');
            i += 1;
        }
    }
    if signals != 1 || detours != 5 {
        return Err("D042_DETOUR_INVENTORY");
    }
    Ok(base.into_bytes())
}

#[derive(Clone)]
struct Track {
    net: String,
    a: [i64; 2],
    b: [i64; 2],
    width: i64,
}

fn parse_track(row: &str) -> Result<Track, &'static str> {
    let f: Vec<_> = row.split('|').collect();
    if f.len() != 9 || f[0] != "T" || f[2] != "1" {
        return Err("D042_BASE_TRACK");
    }
    let integer = |i: usize| f[i].parse::<i64>().map_err(|_| "D042_BASE_INTEGER");
    Ok(Track {
        net: f[3].to_owned(),
        a: [integer(4)?, integer(5)?],
        b: [integer(6)?, integer(7)?],
        width: integer(8)?,
    })
}

fn same_pair(a: [i64; 2], b: [i64; 2], x: [i64; 2], y: [i64; 2]) -> bool {
    (a == x && b == y) || (a == y && b == x)
}

/// Deterministic source-owned bounded candidate graph, not a physical copper or P009 claim.
pub fn encode(bytes: &[u8]) -> Result<String, &'static str> {
    if bytes.is_empty() || bytes.len() > route_preview::MAX_FILE_BYTES {
        return Err("D042_SOURCE_SIZE");
    }
    // Full D_041 provenance, pad/trace geometry, original bridge and
    // source inventory must remain valid after removing only detour rows.
    let baseline = d041_local_bridge::encode(&strip_detour(bytes)?)?;
    let old: Vec<Track> = baseline
        .lines()
        .filter(|line| line.starts_with("T|"))
        .map(parse_track)
        .collect::<Result<_, _>>()?;
    if old.len() != 5 || old[0].width <= 0 || old.iter().any(|t| t.net != old[0].net) {
        return Err("D042_BASE_TRACKS");
    }
    let (j0, j1) = (old[4].a, old[4].b);
    // This stage qualifies exactly the documented public synthetic geometry.
    // Arbitrary translations/rotations and other loop geometries are unsupported.
    if j0 != [20_000, 35_000]
        || j1 != [40_000, 35_000]
        || old.iter().any(|t| t.width != old[4].width)
    {
        return Err("D042_UNSUPPORTED_BASE_GEOMETRY");
    }
    let expected_pad_points = [
        [20_000, 30_000],
        [20_000, 40_000],
        [40_000, 30_000],
        [40_000, 40_000],
    ];
    let pads: Vec<_> = baseline.lines().filter(|s| s.starts_with("P|")).collect();
    if pads.len() != 4 {
        return Err("D042_BASE_PAD_COUNT");
    }
    for (i, row) in pads.iter().enumerate() {
        let f: Vec<_> = row.split('|').collect();
        if f.len() != 10
            || f[5].parse::<i64>().ok() != Some(expected_pad_points[i][0])
            || f[6].parse::<i64>().ok() != Some(expected_pad_points[i][1])
        {
            return Err("D042_BASE_PAD_LOCATION");
        }
    }
    let points: [[i64; 2]; 6] = [
        j0,
        [15_000, 35_000],
        [15_000, 25_000],
        [45_000, 25_000],
        [45_000, 35_000],
        j1,
    ];
    let route = route_preview::parse(bytes)?;
    if route.skipped != 0
        || route.segments.len() != 10
        || !route.vias.is_empty()
        || !route.arcs.is_empty()
    {
        return Err("D042_ROUTE_INVENTORY");
    }
    // Every imported segment must match exactly one independent SOURCE
    // witness. Reversed direction and record order do not change topology.
    let mut used = [false; 10];
    for t in &route.segments {
        if t.layer != 1 || t.net != old[0].net || t.width_ticks != old[4].width {
            return Err("D042_NET_LAYER_WIDTH");
        }
        let mut found = None;
        for (i, base) in old.iter().enumerate() {
            if same_pair(t.a, t.b, base.a, base.b) {
                if found.is_some() {
                    return Err("D042_AMBIGUOUS_EDGE");
                }
                found = Some(i);
            }
        }
        for i in 0..5 {
            if same_pair(t.a, t.b, points[i], points[i + 1]) {
                if found.is_some() {
                    return Err("D042_AMBIGUOUS_DETOUR");
                }
                found = Some(i + 5);
            }
        }
        let index = found.ok_or("D042_UNSUPPORTED_GEOMETRY")?;
        if used[index] {
            return Err("D042_DUPLICATE_EDGE");
        }
        used[index] = true;
    }
    if used.iter().any(|u| !u) {
        return Err("D042_MISSING_EDGE");
    }

    // SOURCE edges: four pad contacts, two old local junctions, original
    // bridge endpoints, then five detour segments with six exact attachments.
    let edges: [(usize, usize); 14] = [
        (0, 4),
        (1, 5),
        (4, 5),
        (2, 6),
        (3, 7),
        (6, 7),
        (4, 8),
        (8, 6),
        (4, 9),
        (9, 10),
        (10, 11),
        (11, 12),
        (12, 13),
        (13, 6),
    ];
    let mut parent: Vec<usize> = (0..14).collect();
    let mut cycles = 0_usize;
    for (a, b) in edges {
        let mut x = a;
        while parent[x] != x {
            x = parent[x];
        }
        let mut y = b;
        while parent[y] != y {
            y = parent[y];
        }
        if x == y {
            cycles += 1;
        } else {
            parent[x] = y;
        }
    }
    let mut components = 0_usize;
    for n in 0..14 {
        let mut r = n;
        while parent[r] != r {
            r = parent[r];
        }
        if r == n {
            components += 1;
        }
    }
    if components != 1 || cycles != 1 || 14 - 14 + components != cycles {
        return Err("D042_TOPOLOGY");
    }

    let mut sha = String::with_capacity(64);
    for b in digest::sha256(bytes) {
        write!(&mut sha, "{b:02x}").map_err(|_| "D042_FORMAT")?;
    }
    let mut out = format!(
        "BOARDSI_D042_LOCAL_CLOSED_LOOP_V1|{sha}|1|LOCAL_COPPER_CLOSED_LOOP_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|GLOBAL_ELECTRICAL_NOT_VERIFIED\n"
    );
    writeln!(&mut out, "C|0|U1+U2|{}|1|14|14", old[0].net).map_err(|_| "D042_FORMAT")?;
    for row in baseline
        .lines()
        .filter(|l| l.starts_with("P|") || l.starts_with("T|"))
    {
        writeln!(&mut out, "{row}").map_err(|_| "D042_FORMAT")?;
    }
    for i in 0..5 {
        writeln!(
            &mut out,
            "T|{}|1|{}|{}|{}|{}|{}|{}",
            i + 5,
            old[0].net,
            points[i][0],
            points[i][1],
            points[i + 1][0],
            points[i + 1][1],
            old[4].width
        )
        .map_err(|_| "D042_FORMAT")?;
    }
    for row in baseline.lines().filter(|l| l.starts_with("E|")) {
        writeln!(&mut out, "{row}").map_err(|_| "D042_FORMAT")?;
    }
    for (n, (a, b)) in edges.iter().enumerate().skip(8) {
        writeln!(&mut out, "E|{n}|{a}|{b}|EXACT_DETOUR_JUNCTION_LOCAL_COPPER")
            .map_err(|_| "D042_FORMAT")?;
    }
    out.push_str("CYCLE|LOCAL|1|INDEPENDENT_PATHS|2\n");
    out.push_str("COUNT|NODES|14|EDGES|14|LOCAL_COMPONENTS|1|LOCAL_CYCLES|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n");
    out.push_str("END|SOURCE_LOCAL_CLOSED_LOOP_ONLY_NO_GLOBAL_PASS\n");
    Ok(out)
}
