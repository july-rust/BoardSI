//! D_043: counterfactual removal of SOURCE-local trace nodes; not a physical Open/Short test.
use crate::d042_local_closed_loop;
use std::collections::BTreeSet;
use std::fmt::Write;

fn valid_cuts(cuts: &[usize]) -> bool {
    match cuts {
        [] => true,
        [one] => (4..=9).contains(one),
        [4, detour] => (5..=9).contains(detour),
        _ => false,
    }
}

/// Requires a complete, valid D_042 SOURCE before any counterfactual scenario.
/// Trace ID Tn maps to D_042 graph node (4+n); incident edges are removed.
pub fn encode(source: &[u8], cuts: &[usize]) -> Result<String, &'static str> {
    if !valid_cuts(cuts) {
        return Err("D043_UNSUPPORTED_CUT_SET");
    }
    let full = d042_local_closed_loop::encode(source)?;
    let header = full.lines().next().ok_or("D043_HEADER")?;
    let fields: Vec<_> = header.split('|').collect();
    if fields.first() != Some(&"BOARDSI_D042_LOCAL_CLOSED_LOOP_V1")
        || fields.get(1).map(str::len) != Some(64)
    {
        return Err("D043_SOURCE_RECEIPT");
    }
    let sha = fields[1];
    let mut graph_edges = Vec::new();
    for row in full.lines().filter(|row| row.starts_with("E|")) {
        let parts: Vec<_> = row.split('|').collect();
        if parts.len() != 5 {
            return Err("D043_EDGE_SHAPE");
        }
        let id = parts[1].parse::<usize>().map_err(|_| "D043_EDGE_ID")?;
        let a = parts[2].parse::<usize>().map_err(|_| "D043_EDGE_NODE")?;
        let b = parts[3].parse::<usize>().map_err(|_| "D043_EDGE_NODE")?;
        if id != graph_edges.len() || a >= 14 || b >= 14 || a == b {
            return Err("D043_EDGE_INVALID");
        }
        graph_edges.push((a, b));
    }
    if graph_edges.len() != 14 {
        return Err("D043_EDGE_INVENTORY");
    }
    let mut enabled = [true; 14];
    for &id in cuts {
        enabled[4 + id] = false;
    }
    let nodes = enabled.iter().filter(|&&on| on).count();
    let mut parent: Vec<usize> = (0..14).collect();
    let mut edges = 0_usize;
    let mut cycles = 0_usize;
    for (a, b) in graph_edges {
        if !enabled[a] || !enabled[b] {
            continue;
        }
        edges += 1;
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
    let root = |node: usize| {
        let mut x = node;
        while parent[x] != x {
            x = parent[x];
        }
        x
    };
    let components = (0..14)
        .filter(|&n| enabled[n])
        .map(root)
        .collect::<BTreeSet<_>>()
        .len();
    let rank = edges
        .checked_add(components)
        .and_then(|n| n.checked_sub(nodes))
        .ok_or("D043_RANK_OVERFLOW")?;
    if rank != cycles {
        return Err("D043_GRAPH_RANK");
    }
    let reachable = root(0) == root(2);
    // Preserve the mandatory U1 and U2 local junctions in every supported cut.
    if root(0) != root(1) || root(2) != root(3) {
        return Err("D043_PAD_GROUP");
    }

    let mut cut_name = String::new();
    if cuts.is_empty() {
        cut_name.push_str("NONE");
    } else {
        for (index, id) in cuts.iter().enumerate() {
            if index != 0 {
                cut_name.push(',');
            }
            write!(&mut cut_name, "T{id}").map_err(|_| "D043_FORMAT")?;
        }
    }
    let mut output = format!(
        "BOARDSI_D043_LOCAL_CUTSET_V1|{sha}|SOURCE_LOCAL_COUNTERFACTUAL_ONLY|P009_UNSUPPORTED|SI_NOT_RUN|GLOBAL_ELECTRICAL_NOT_VERIFIED\nCUT|{cut_name}\n"
    );
    writeln!(
        &mut output,
        "COUNT|NODES|{nodes}|EDGES|{edges}|LOCAL_COMPONENTS|{components}|LOCAL_CYCLES|{cycles}|U1_U2_REACHABLE|{}",
        if reachable { "YES" } else { "NO" }
    )
    .map_err(|_| "D043_FORMAT")?;
    output.push_str("END|SOURCE_LOCAL_DIAGNOSTIC_NO_PHYSICAL_OPEN_SHORT\n");
    Ok(output)
}

/// A CLI token must exactly match T4..T9; aliases and leading zeros fail closed.
pub fn parse_cut_id(text: &str) -> Result<usize, &'static str> {
    let value = text.strip_prefix('T').ok_or("D043_CUT_TOKEN")?;
    let id = value.parse::<usize>().map_err(|_| "D043_CUT_TOKEN")?;
    if format!("T{id}") != text || !(4..=9).contains(&id) {
        return Err("D043_CUT_TOKEN");
    }
    Ok(id)
}
