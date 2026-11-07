//! D_053: actual finite-width SOURCE repair of a D_052 midspan copper break.
//! This is a bounded local geometry candidate, NOT manufactured/global continuity.
use crate::{d052_bridge_midspan_break, digest, route_preview};
use std::fmt::Write;

#[derive(Clone, Copy, Debug)]
struct Repair {
    row: usize,
    lo: i64,
    hi: i64,
    y: i64,
}
fn ticks(token: &str) -> Option<i64> {
    let token = token.strip_prefix('+').unwrap_or(token);
    let (neg, token) = if let Some(s) = token.strip_prefix('-') { (true, s) } else { (false, token) };
    let (whole, fraction) = token.split_once('.').unwrap_or((token, ""));
    if whole.is_empty() || !whole.bytes().all(|c| c.is_ascii_digit())
        || fraction.len() > 8 || !fraction.bytes().all(|c| c.is_ascii_digit()) { return None; }
    let mut frac = fraction.to_owned();
    while frac.len() < 2 { frac.push('0'); }
    if frac.as_bytes().get(2..)?.iter().any(|&c| c != b'0') { return None; }
    let abs = whole.parse::<i64>().ok()?.checked_mul(100)?
        .checked_add(frac[..2].parse::<i64>().ok()?)?;
    if neg { abs.checked_neg() } else { Some(abs) }
}
fn candidate(rows: &[&str], i: usize, owner: (&str, &str), y: i64) -> Option<Repair> {
    if i + 2 >= rows.len() || rows[i].split_ascii_whitespace().collect::<Vec<_>>() != [owner.0, owner.1] {
        return None;
    }
    let a: Vec<_> = rows[i + 1].split_ascii_whitespace().collect();
    let b: Vec<_> = rows[i + 2].split_ascii_whitespace().collect();
    if a.len() != 5 || b.len() != 5
        || ticks(a[1])? != y || ticks(b[1])? != y
        || !["1", "65"].contains(&a[2]) || !["1", "65"].contains(&b[2])
        || ticks(a[3])? != 800 || ticks(b[3])? != 800
        || !["0", "128"].contains(&a[4]) || !["0", "128"].contains(&b[4]) { return None; }
    let x0 = ticks(a[0])?;
    let x1 = ticks(b[0])?;
    let (lo, hi) = (x0.min(x1), x0.max(x1));
    if !(28_000..=30_000).contains(&lo)
        || !(30_000..=32_000).contains(&hi) || lo >= hi { return None; }
    Some(Repair { row: i, lo, hi, y })
}
fn strip_repairs(source: &str) -> Result<(String, [Option<Repair>; 2]), &'static str> {
    let rows: Vec<_> = source.lines().collect();
    let mut repairs: [Option<Repair>; 2] = [None, None];
    let mut signal = false;
    let mut i = 0;
    while i < rows.len() {
        let f: Vec<_> = rows[i].split_ascii_whitespace().collect();
        if f.first() == Some(&"*SIGNAL*") { signal = true; }
        if f.first() == Some(&"*END*") { signal = false; }
        if signal {
            let side = if let Some(r) = candidate(&rows, i, ("U1.1", "U2.1"), 30_000) {
                Some((0, r))
            } else {
                candidate(&rows, i, ("U1.2", "U2.2"), 40_000).map(|r| (1, r))
            };
            if let Some((side, repair)) = side {
                if repairs[side].replace(repair).is_some() { return Err("D053_DUPLICATE_REPAIR"); }
                i += 3;
                continue;
            }
        }
        i += 1;
    }
    let mut bare = String::new();
    for (i, row) in rows.iter().enumerate() {
        if repairs.iter().flatten().any(|r| (r.row..r.row + 3).contains(&i)) { continue; }
        bare.push_str(row);
        bare.push('\n');
    }
    Ok((bare, repairs))
}
#[derive(Clone, Debug)]
struct Trace {
    a: [i64; 2], b: [i64; 2], width: i64, net: String,
}
fn read_trace(row: &str) -> Result<Trace, &'static str> {
    let f: Vec<_> = row.split('|').collect();
    if f.len() != 9 || f[0] != "T" || f[2] != "1" { return Err("D053_BASE_TRACE"); }
    Ok(Trace {
        net: f[3].to_owned(),
        a: [f[4].parse().map_err(|_| "D053_X")?, f[5].parse().map_err(|_| "D053_Y")?],
        b: [f[6].parse().map_err(|_| "D053_X")?, f[7].parse().map_err(|_| "D053_Y")?],
        width: f[8].parse().map_err(|_| "D053_WIDTH")?,
    })
}
fn graph(n: usize, edges: &[(usize, usize)]) -> Result<(usize, usize, bool), &'static str> {
    if !(14..=16).contains(&n) || !(12..=16).contains(&edges.len()) { return Err("D053_GRAPH_SIZE"); }
    let mut parent: Vec<_> = (0..n).collect();
    fn root(par: &[usize], mut x: usize) -> usize {
        while par[x] != x { x = par[x]; } x
    }
    for &(u, v) in edges {
        if u >= n || v >= n || u == v { return Err("D053_GRAPH_EDGE"); }
        let (a, b) = (root(&parent, u), root(&parent, v));
        if a != b { parent[a] = b; }
    }
    let mut roots = Vec::new();
    for i in 0..n {
        let r = root(&parent, i);
        if !roots.contains(&r) { roots.push(r); }
    }
    Ok((roots.len(), edges.len() + roots.len() - n, root(&parent, 0) == root(&parent, 2)))
}
/// The original SOURCE, including both independently authored repair rows, is
/// hashed and imported. D052 requalifies only a freshly constructed bare input.
pub fn encode(bytes: &[u8]) -> Result<String, &'static str> {
    if bytes.is_empty() || bytes.len() > route_preview::MAX_FILE_BYTES { return Err("D053_SIZE"); }
    let source = std::str::from_utf8(bytes).map_err(|_| "D053_UTF8")?;
    let (bare, repair) = strip_repairs(source)?;
    let baseline = d052_bridge_midspan_break::encode(bare.as_bytes()).map_err(|_| "D053_BARE_BASELINE")?;
    let split: Vec<bool> = ["A", "B"].iter().map(|id| {
        baseline.lines().any(|s| s.starts_with(&format!("S|{id}|BREAK|")))
    }).collect();
    let ra = repair[0].is_some();
    let rb = repair[1].is_some();
    let (state, expected) = match (split[0], split[1], ra, rb) {
        (true, true, false, false) => ("BREAK_BOTH", (14, 12, 2, 0, false)),
        (true, true, true, false) => ("REPAIR_A_ONLY", (15, 14, 1, 0, true)),
        (true, true, false, true) => ("REPAIR_B_ONLY", (15, 14, 1, 0, true)),
        (true, true, true, true) => ("REPAIR_BOTH", (16, 16, 1, 1, true)),
        (true, false, true, false) => ("BREAK_A_REPAIR_A", (14, 14, 1, 1, true)),
        (false, true, false, true) => ("BREAK_B_REPAIR_B", (14, 14, 1, 1, true)),
        _ => return Err("D053_UNSUPPORTED_STATE"),
    };
    let traces: Vec<_> = baseline.lines().filter(|s| s.starts_with("T|")).map(read_trace)
        .collect::<Result<_, _>>()?;
    let pads: Vec<_> = baseline.lines().filter(|s| s.starts_with("P|")).collect();
    if pads.len() != 4 || traces.len() != 8 + usize::from(split[0]) + usize::from(split[1])
       || !traces.iter().all(|t| t.net == traces[0].net) { return Err("D053_BASE_INVENTORY"); }
    let net = &traces[0].net;
    let mut extra = Vec::new();
    let mut witnesses = Vec::new();
    for (side, rep) in repair.iter().enumerate() {
        if let Some(r) = rep {
            if !split[side] { return Err("D053_REPAIR_WITHOUT_BREAK"); }
            let left = &traces[6 + side];
            let right = &traces[8 + if side == 0 { 0 } else { usize::from(split[0]) }];
            if left.width != 800 || right.width != 800
               || left.b[1] != r.y || right.a[1] != r.y
               || left.b[0] >= right.a[0]
               || r.lo <= left.a[0] || r.hi >= right.b[0] {
                return Err("D053_REPAIR_GEOMETRY");
            }
            let left_distance = (r.lo - left.b[0]).max(0);
            let right_distance = (right.a[0] - r.hi).max(0);
            if left_distance >= 800 || right_distance >= 800 {
                return Err("D053_NO_POSITIVE_CONTACT");
            }
            witnesses.push((side, 800 - left_distance, 800 - right_distance));
            extra.push(Trace { a: [r.lo, r.y], b: [r.hi, r.y], width: 800, net: net.clone() });
        }
    }
    let actual = route_preview::parse(bytes).map_err(|_| "D053_IMPORT")?;
    if actual.skipped != 0 || !actual.arcs.is_empty() || !actual.vias.is_empty()
        || actual.segments.len() != traces.len() + extra.len() {
        return Err("D053_ACTUAL_INVENTORY");
    }
    let all: Vec<_> = traces.iter().chain(extra.iter()).collect();
    let mut matched = vec![false; actual.segments.len()];
    for t in &all {
        let hits: Vec<_> = actual.segments.iter().enumerate().filter(|(i, a)| {
            !matched[*i] && a.layer == 1 && a.net == t.net && a.width_ticks == t.width &&
            ((a.a == t.a && a.b == t.b) || (a.a == t.b && a.b == t.a))
        }).map(|(i, _)| i).collect();
        if hits.len() != 1 { return Err("D053_TRACE_NOT_ONE_TO_ONE"); }
        matched[hits[0]] = true;
    }
    if matched.iter().any(|v| !v) { return Err("D053_EXTRA_TRACE"); }
    let mut edges = Vec::<(usize, usize, String)>::new();
    for s in baseline.lines().filter(|s| s.starts_with("E|")) {
        let f: Vec<_> = s.split('|').collect();
        if f.len() != 5 || f[1] != edges.len().to_string() { return Err("D053_BASE_EDGE"); }
        edges.push((f[2].parse().map_err(|_| "D053_EDGE")?,
                    f[3].parse().map_err(|_| "D053_EDGE")?, f[4].to_owned()));
    }
    if edges.len() != 12 { return Err("D053_BASE_EDGES"); }
    let base_nodes = 12 + usize::from(split[0]) + usize::from(split[1]);
    for (index, (side, _, _)) in witnesses.iter().enumerate() {
        let id = base_nodes + index;
        let left = 6 + side;
        let right = 12 + if *side == 0 { 0 } else { usize::from(split[0]) };
        edges.push((left, id, format!("POSITIVE_FINITE_WIDTH_REPAIR_{}_LEFT", if *side == 0 {"A"} else {"B"})));
        edges.push((right, id, format!("POSITIVE_FINITE_WIDTH_REPAIR_{}_RIGHT", if *side == 0 {"A"} else {"B"})));
    }
    let nodes = base_nodes + extra.len();
    let links: Vec<_> = edges.iter().map(|(u,v,_)| (*u,*v)).collect();
    let (components, cycles, reachable) = graph(nodes, &links)?;
    if (nodes, edges.len(), components, cycles, reachable) != expected {
        return Err("D053_REPAIR_TOPOLOGY");
    }
    let mut sha = String::new();
    for b in digest::sha256(bytes) { write!(&mut sha, "{b:02x}").map_err(|_| "D053_SHA")?; }
    let mut out = format!("BOARDSI_D053_SOURCE_MIDSPAN_REPAIR_V1|{sha}|1|SOURCE_LOCAL_DUAL_BRIDGE_REPAIR_ONLY|P009_UNSUPPORTED|SI_NOT_RUN|GLOBAL_ELECTRICAL_NOT_VERIFIED\nC|0|U1_U2|{net}|1|{nodes}|{}\n", edges.len());
    for row in pads { writeln!(&mut out, "{row}").map_err(|_| "D053_FORMAT")?; }
    for (i, t) in all.iter().enumerate() {
        writeln!(&mut out, "T|{i}|1|{}|{}|{}|{}|{}|{}", t.net, t.a[0],t.a[1],t.b[0],t.b[1],t.width)
            .map_err(|_| "D053_FORMAT")?;
    }
    for (id, (u,v,label)) in edges.iter().enumerate() {
        writeln!(&mut out, "E|{id}|{u}|{v}|{label}").map_err(|_| "D053_FORMAT")?;
    }
    for row in baseline.lines().filter(|s| s.starts_with("S|")) {
        writeln!(&mut out, "{row}").map_err(|_| "D053_FORMAT")?;
    }
    for (side, l, r) in &witnesses {
        writeln!(&mut out, "R|{}|LEFT_COPPER_OVERLAP_TICKS|{l}|RIGHT_COPPER_OVERLAP_TICKS|{r}",
                 if *side == 0 {"A"} else {"B"}).map_err(|_| "D053_FORMAT")?;
    }
    writeln!(&mut out, "D|{state}|{nodes}|{}|{components}|{cycles}|{}",
             edges.len(), if reachable {"REACHABLE"} else {"SEPARATED"})
        .map_err(|_| "D053_FORMAT")?;
    writeln!(&mut out, "COUNT|NODES|{nodes}|EDGES|{}|LOCAL_COMPONENTS|{components}|CYCLE_RANK|{cycles}|GLOBAL_ELECTRICAL|NOT_VERIFIED",
             edges.len()).map_err(|_| "D053_FORMAT")?;
    out.push_str("END|SOURCE_LOCAL_DUAL_BRIDGE_REPAIR_ONLY_NO_GLOBAL_PASS\n");
    Ok(out)
}
