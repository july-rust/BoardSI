//! D_052: bounded real SOURCE midspan breaks of either finite-width bridge.
//! Local same-layer candidate contacts only, never global electrical proof.
use crate::{d051_dual_bridge_loop, digest, route_preview};
use std::fmt::Write;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Piece {
    row: usize,
    lo: i64,
    hi: i64,
}
#[derive(Clone, Debug)]
struct Path {
    pieces: Vec<Piece>,
    gap: Option<i64>,
}
fn ticks(token: &str) -> Option<i64> {
    let token = token.strip_prefix('+').unwrap_or(token);
    let (neg, token) = if let Some(rest) = token.strip_prefix('-') {
        (true, rest)
    } else {
        (false, token)
    };
    let (whole, fractional) = token.split_once('.').unwrap_or((token, ""));
    if whole.is_empty() || !whole.bytes().all(|b| b.is_ascii_digit())
        || !fractional.bytes().all(|b| b.is_ascii_digit())
        || fractional.len() > 8
    {
        return None;
    }
    let whole: i64 = whole.parse().ok()?;
    let mut frac = fractional.to_owned();
    while frac.len() < 2 {
        frac.push('0');
    }
    if frac.as_bytes().get(2..)?.iter().any(|&b| b != b'0') {
        return None;
    }
    let remainder: i64 = frac.get(..2).unwrap_or("00").parse().ok()?;
    let result = whole.checked_mul(100)?.checked_add(remainder)?;
    if neg { result.checked_neg() } else { Some(result) }
}
fn source_piece(rows: &[&str], i: usize, pair: (&str, &str), y: i64,
                left: i64, right: i64) -> Option<Piece> {
    if i + 2 >= rows.len() {
        return None;
    }
    let owners: Vec<_> = rows[i].split_ascii_whitespace().collect();
    if owners != [pair.0, pair.1] {
        return None;
    }
    let v0: Vec<_> = rows[i + 1].split_ascii_whitespace().collect();
    let v1: Vec<_> = rows[i + 2].split_ascii_whitespace().collect();
    if v0.len() != 5 || v1.len() != 5 {
        return None;
    }
    let x0 = ticks(v0[0])?;
    let x1 = ticks(v1[0])?;
    if ticks(v0[1])? != y || ticks(v1[1])? != y
        || !["1", "65"].contains(&v0[2]) || !["1", "65"].contains(&v1[2])
        || ticks(v0[3])? != 800 || ticks(v1[3])? != 800
        || !["0", "128"].contains(&v0[4]) || !["0", "128"].contains(&v1[4])
    {
        return None;
    }
    let (lo, hi) = (x0.min(x1), x0.max(x1));
    if lo < left || hi > right || lo >= hi {
        return None;
    }
    Some(Piece { row: i, lo, hi })
}
fn qualified(mut pieces: Vec<Piece>, left: i64, right: i64)
    -> Result<Path, &'static str> {
    pieces.sort_by_key(|p| p.lo);
    match pieces.as_slice() {
        [only] if only.lo == left && only.hi == right => {
            Ok(Path { pieces, gap: None })
        }
        [a, b] if a.lo == left && b.hi == right
            && (28_000..=30_500).contains(&a.hi)
            && (29_500..=32_000).contains(&b.lo)
            && b.lo - a.hi > 800 => {
            let gap = b.lo - a.hi;
            Ok(Path { pieces, gap: Some(gap) })
        }
        _ => Err("D052_BRIDGE_PARTITION"),
    }
}
fn segment_matches(t: &route_preview::Segment, net: &str,
                   a: [i64; 2], b: [i64; 2], width: i64) -> bool {
    t.layer == 1 && t.net == net && t.width_ticks == width
        && ((t.a == a && t.b == b) || (t.a == b && t.b == a))
}
fn read_track(line: &str) -> Result<([i64; 2], [i64; 2], i64), &'static str> {
    let f: Vec<_> = line.split('|').collect();
    if f.len() != 9 || f[0] != "T" || f[2] != "1" {
        return Err("D052_BASE_TRACK");
    }
    Ok((
        [f[4].parse().map_err(|_| "D052_X0")?, f[5].parse().map_err(|_| "D052_Y0")?],
        [f[6].parse().map_err(|_| "D052_X1")?, f[7].parse().map_err(|_| "D052_Y1")?],
        f[8].parse().map_err(|_| "D052_WIDTH")?,
    ))
}
fn graph(edges: &[(usize, usize)], n: usize)
    -> Result<(usize, usize, bool), &'static str> {
    if !(12..=14).contains(&n) || edges.len() != 12 {
        return Err("D052_GRAPH_SIZE");
    }
    let mut parent: Vec<_> = (0..n).collect();
    fn root(parent: &[usize], mut x: usize) -> usize {
        while parent[x] != x { x = parent[x]; }
        x
    }
    for &(a, b) in edges {
        if a >= n || b >= n || a == b { return Err("D052_GRAPH_EDGE"); }
        let ra = root(&parent, a);
        let rb = root(&parent, b);
        if ra != rb { parent[ra] = rb; }
    }
    let mut roots = Vec::new();
    for i in 0..n {
        let root = root(&parent, i);
        if !roots.contains(&root) { roots.push(root); }
    }
    let cycles = edges.len() + roots.len() - n;
    Ok((roots.len(), cycles, root(&parent, 0) == root(&parent, 2)))
}
fn track_row(index: usize, net: &str, a: i64, b: i64, y: i64) -> String {
    format!("T|{index}|1|{net}|{a}|{y}|{b}|{y}|800\n")
}
/// Encode a diagnostic bound to the actual split SOURCE SHA, independently
/// matching all imported segments against a requalified D_051 baseline.
pub fn encode(bytes: &[u8]) -> Result<String, &'static str> {
    if bytes.is_empty() || bytes.len() > route_preview::MAX_FILE_BYTES {
        return Err("D052_SIZE");
    }
    let source = std::str::from_utf8(bytes).map_err(|_| "D052_UTF8")?;
    let rows: Vec<_> = source.lines().collect();
    let mut signal = false;
    let (mut a, mut b) = (Vec::new(), Vec::new());
    let mut i = 0;
    while i < rows.len() {
        let fields: Vec<_> = rows[i].split_ascii_whitespace().collect();
        if fields.first() == Some(&"*SIGNAL*") { signal = true; }
        if fields.first() == Some(&"*END*") { signal = false; }
        if signal {
            if let Some(piece) = source_piece(&rows, i, ("U1.1", "U2.1"),
                                               30_000, 22_600, 37_800) {
                a.push(piece); i += 3; continue;
            }
            if let Some(piece) = source_piece(&rows, i, ("U1.2", "U2.2"),
                                               40_000, 22_200, 38_200) {
                b.push(piece); i += 3; continue;
            }
        }
        i += 1;
    }
    let a = qualified(a, 22_600, 37_800)?;
    let b = qualified(b, 22_200, 38_200)?;
    // Build a D_051 SOURCE only as an *additional baseline oracle*; never
    // substitute its virtual graph for the real, independently parsed input.
    let mut restored = String::new();
    for (i, row) in rows.iter().enumerate() {
        if i == a.pieces[0].row {
            restored.push_str("U1.1 U2.1\n226 300 1 8 0\n378 300 1 8 0\n");
        }
        if i == b.pieces[0].row {
            restored.push_str("U1.2 U2.2\n222 400 1 8 0\n382 400 1 8 0\n");
        }
        if a.pieces.iter().chain(b.pieces.iter())
            .any(|p| (p.row..p.row + 3).contains(&i)) {
            continue;
        }
        restored.push_str(row);
        restored.push('\n');
    }
    let wire = d051_dual_bridge_loop::encode(restored.as_bytes())
        .map_err(|_| "D052_RESTORED_BASELINE")?;
    let imported = route_preview::parse(bytes).map_err(|_| "D052_IMPORT")?;
    let n_extra = usize::from(a.gap.is_some()) + usize::from(b.gap.is_some());
    if imported.skipped != 0 || !imported.vias.is_empty()
        || !imported.arcs.is_empty() || imported.segments.len() != 8 + n_extra {
        return Err("D052_ACTUAL_INVENTORY");
    }
    let tracks: Vec<_> = wire.lines().filter(|row| row.starts_with("T|")).collect();
    if tracks.len() != 8 { return Err("D052_BASE_INVENTORY"); }
    let net = tracks[0].split('|').nth(3).ok_or("D052_NET")?;
    let mut expected = Vec::<([i64; 2], [i64; 2], i64)>::new();
    for row in &tracks[..6] { expected.push(read_track(row)?); }
    for (path, y) in [(&a, 30_000), (&b, 40_000)] {
        for part in &path.pieces {
            expected.push(([part.lo, y], [part.hi, y], 800));
        }
    }
    let mut used = vec![false; imported.segments.len()];
    for &(begin, end, width) in &expected {
        let matches: Vec<_> = imported.segments.iter().enumerate()
            .filter(|(i, t)| !used[*i] && segment_matches(t, net, begin, end, width))
            .map(|(i, _)| i).collect();
        if matches.len() != 1 { return Err("D052_NOT_ONE_TO_ONE"); }
        used[matches[0]] = true;
    }
    if used.iter().any(|ok| !ok) { return Err("D052_EXTRA_TRACK"); }
    let mut edges: Vec<(usize, usize, String)> = wire.lines()
        .filter(|s| s.starts_with("E|"))
        .map(|s| {
            let f: Vec<_> = s.split('|').collect();
            if f.len() != 5 { return Err("D052_EDGE_ROW"); }
            Ok((f[2].parse().map_err(|_| "D052_EDGE_A")?,
                f[3].parse().map_err(|_| "D052_EDGE_B")?, f[4].to_owned()))
        })
        .collect::<Result<_, &'static str>>()?;
    if edges.len() != 12 || edges[9].0 != 2 || edges[9].1 != 10
        || edges[11].0 != 3 || edges[11].1 != 11 {
        return Err("D052_BASE_EDGES");
    }
    let mut next = 12;
    let a_right = if a.gap.is_some() { let id = next; next += 1; id } else { 10 };
    let b_right = if b.gap.is_some() { let id = next; next += 1; id } else { 11 };
    edges[9].1 = a_right;
    edges[11].1 = b_right;
    let links: Vec<_> = edges.iter().map(|(u, v, _)| (*u, *v)).collect();
    let (components, cycles, reachable) = graph(&links, next)?;
    let state = match (a.gap.is_some(), b.gap.is_some()) {
        (false, false) => "INTACT",
        (true, false) => "BREAK_A",
        (false, true) => "BREAK_B",
        (true, true) => "BREAK_BOTH",
    };
    let correct = match state {
        "INTACT" => (12, 1, 1, true),
        "BREAK_A" | "BREAK_B" => (13, 1, 0, true),
        _ => (14, 2, 0, false),
    };
    if (next, components, cycles, reachable) != correct {
        return Err("D052_BREAK_TOPOLOGY");
    }
    let mut sha = String::new();
    for byte in digest::sha256(bytes) {
        write!(&mut sha, "{byte:02x}").map_err(|_| "D052_FORMAT")?;
    }
    let mut out = format!("BOARDSI_D052_DUAL_BRIDGE_MIDSPAN_BREAK_V1|{sha}|1|SOURCE_LOCAL_DUAL_BRIDGE_MIDSPAN_BREAK_ONLY|P009_UNSUPPORTED|SI_NOT_RUN|GLOBAL_ELECTRICAL_NOT_VERIFIED\nC|0|U1_U2|{net}|1|{next}|12\n");
    for row in wire.lines().filter(|s| s.starts_with("P|")) {
        writeln!(&mut out, "{row}").map_err(|_| "D052_FORMAT")?;
    }
    for row in &tracks[..6] {
        writeln!(&mut out, "{row}").map_err(|_| "D052_FORMAT")?;
    }
    for (index, path, y) in [(6, &a, 30_000), (7, &b, 40_000)] {
        let p = path.pieces[0];
        out.push_str(&track_row(index, net, p.lo, p.hi, y));
    }
    if a.gap.is_some() {
        let p = a.pieces[1];
        out.push_str(&track_row(8, net, p.lo, p.hi, 30_000));
    }
    if b.gap.is_some() {
        let p = b.pieces[1];
        out.push_str(&track_row(8 + usize::from(a.gap.is_some()), net, p.lo, p.hi, 40_000));
    }
    for (index, (u, v, label)) in edges.iter().enumerate() {
        writeln!(&mut out, "E|{index}|{u}|{v}|{label}").map_err(|_| "D052_FORMAT")?;
    }
    for (id, path) in [("A", &a), ("B", &b)] {
        match path.gap {
            Some(gap) => writeln!(&mut out, "S|{id}|BREAK|CENTER_GAP_TICKS|{gap}|COPPER_GAP_TICKS|{}", gap - 800),
            None => writeln!(&mut out, "S|{id}|INTACT"),
        }.map_err(|_| "D052_FORMAT")?;
    }
    writeln!(&mut out, "D|{state}|{next}|12|{components}|{cycles}|{}", if reachable { "REACHABLE" } else { "SEPARATED" })
        .map_err(|_| "D052_FORMAT")?;
    out.push_str(&format!("COUNT|NODES|{next}|EDGES|12|LOCAL_COMPONENTS|{components}|CYCLE_RANK|{cycles}|GLOBAL_ELECTRICAL|NOT_VERIFIED\nEND|SOURCE_LOCAL_DUAL_BRIDGE_MIDSPAN_BREAK_ONLY_NO_GLOBAL_PASS\n"));
    Ok(out)
}
