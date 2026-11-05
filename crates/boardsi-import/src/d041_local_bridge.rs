//! D_041 bounded Source bridge. Not a global electrical connectivity certificate.
use crate::{d040_multi_component, digest, route_preview};
use std::fmt::Write;

fn without_bridge(bytes: &[u8]) -> Result<Vec<u8>, &'static str> {
    let rows: Vec<_> = std::str::from_utf8(bytes).map_err(|_| "D041_UTF8")?.lines().collect();
    let mut out = String::new();
    let (mut i, mut signals, mut bridges) = (0, 0, 0);
    let mut in_signal = false;
    while i < rows.len() {
        let words: Vec<_> = rows[i].split_ascii_whitespace().collect();
        if words.first() == Some(&"*SIGNAL*") {
            signals += 1;
            in_signal = true;
        } else if words.first() == Some(&"*END*") {
            in_signal = false;
        }
        if words == ["U1.1", "U2.1"] {
            if !in_signal || bridges != 0 || i + 2 >= rows.len()
                || rows[i + 1].split_ascii_whitespace().count() != 5
                || rows[i + 2].split_ascii_whitespace().count() != 5
            {
                return Err("D041_BRIDGE_INVENTORY");
            }
            bridges += 1;
            i += 3;
        } else {
            out.push_str(rows[i]);
            out.push('\n');
            i += 1;
        }
    }
    if bridges != 1 || signals != 1 {
        return Err("D041_BRIDGE_SECTION");
    }
    Ok(out.into_bytes())
}

#[derive(Clone)]
struct Trace {
    net: String,
    a: [i64; 2],
    b: [i64; 2],
    width: i64,
}

fn parse_t(row: &str) -> Result<Trace, &'static str> {
    let f: Vec<_> = row.split('|').collect();
    if f.len() != 9 || f[0] != "T" || f[2] != "1" {
        return Err("D041_BASE_T");
    }
    let n = |i: usize| f[i].parse::<i64>().map_err(|_| "D041_BASE_T_VALUE");
    Ok(Trace {
        net: f[3].to_string(),
        a: [n(4)?, n(5)?],
        b: [n(6)?, n(7)?],
        width: n(8)?,
    })
}
fn same_pair(a: [i64; 2], b: [i64; 2], x: [i64; 2], y: [i64; 2]) -> bool {
    (a == x && b == y) || (a == y && b == x)
}

pub fn encode(bytes: &[u8]) -> Result<String, &'static str> {
    if bytes.is_empty() || bytes.len() > route_preview::MAX_FILE_BYTES {
        return Err("D041_SOURCE_SIZE");
    }
    // D_040 qualifies ALL four old traces/pads, source inventory and isolation.
    let baseline = d040_multi_component::encode(&without_bridge(bytes)?)?;
    let old: Vec<_> = baseline.lines().filter(|l| l.starts_with("T|"))
        .map(parse_t).collect::<Result<_, _>>()?;
    if old.len() != 4 || old.iter().any(|t| t.net != old[0].net) {
        return Err("D041_CROSS_NET_OR_BASE");
    }
    let (j0, j1) = (old[0].b, old[2].b);
    let horizontal = j0[1] == j1[1] && j0[0] != j1[0]
        && old.iter().all(|t| t.a[0] == t.b[0]);
    let vertical = j0[0] == j1[0] && j0[1] != j1[1]
        && old.iter().all(|t| t.a[1] == t.b[1]);
    if !horizontal && !vertical {
        return Err("D041_BRIDGE_ORTHOGONAL");
    }
    let artwork = route_preview::parse(bytes)?;
    if artwork.skipped != 0 || artwork.segments.len() != 5
        || !artwork.vias.is_empty() || !artwork.arcs.is_empty()
    {
        return Err("D041_ROUTE_INVENTORY");
    }
    let mut used = [false; 4];
    let mut bridge = false;
    for t in &artwork.segments {
        if let Some((i, _)) = old.iter().enumerate().find(|(i, o)| {
            !used[*i] && t.layer == 1 && t.net == o.net
                && t.width_ticks == o.width && same_pair(t.a, t.b, o.a, o.b)
        }) {
            used[i] = true;
        } else {
            if bridge || t.layer != 1 || t.net != old[0].net
                || t.width_ticks != old[0].width || t.width_ticks != old[2].width
                || !same_pair(t.a, t.b, j0, j1)
            {
                return Err("D041_BRIDGE_ENDPOINT_NET_WIDTH");
            }
            bridge = true;
        }
    }
    if !bridge || used.iter().any(|v| !v) {
        return Err("D041_BRIDGE_MISSING");
    }
    // Reject pads crossed by the bridge's conservative copper envelope.
    let w = i128::from(old[0].width);
    let (xlo, xhi) = (i128::from(j0[0].min(j1[0])) * 2 - w,
                      i128::from(j0[0].max(j1[0])) * 2 + w);
    let (ylo, yhi) = (i128::from(j0[1].min(j1[1])) * 2 - w,
                      i128::from(j0[1].max(j1[1])) * 2 + w);
    for row in baseline.lines().filter(|l| l.starts_with("P|")) {
        let f: Vec<_> = row.split('|').collect();
        if f.len() != 10 { return Err("D041_BASE_PAD"); }
        let n = |i: usize| f[i].parse::<i128>().map_err(|_| "D041_BASE_PAD_VALUE");
        let (x, y, pw, ph) = (n(5)? * 2, n(6)? * 2, n(7)?, n(8)?);
        if xlo <= x + pw && x - pw <= xhi && ylo <= y + ph && y - ph <= yhi {
            return Err("D041_BRIDGE_PAD_ENVELOPE");
        }
    }
    let mut sha = String::new();
    for b in digest::sha256(bytes) {
        write!(&mut sha, "{b:02x}").map_err(|_| "D041_FORMAT")?;
    }
    let mut out = format!(
        "BOARDSI_D041_LOCAL_BRIDGE_V1|{sha}|1|LOCAL_COPPER_BRIDGE_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|GLOBAL_ELECTRICAL_NOT_VERIFIED\n"
    );
    writeln!(&mut out, "C|0|U1+U2|{}|1|9|8", old[0].net).map_err(|_| "D041_FORMAT")?;
    for row in baseline.lines().filter(|l| l.starts_with("P|") || l.starts_with("T|")) {
        writeln!(&mut out, "{row}").map_err(|_| "D041_FORMAT")?;
    }
    writeln!(&mut out, "T|4|1|{}|{}|{}|{}|{}|{}",
        old[0].net, j0[0], j0[1], j1[0], j1[1], old[0].width)
        .map_err(|_| "D041_FORMAT")?;
    for row in baseline.lines().filter(|l| l.starts_with("E|")) {
        writeln!(&mut out, "{row}").map_err(|_| "D041_FORMAT")?;
    }
    out.push_str("E|6|4|8|EXACT_BRIDGE_JUNCTION_LOCAL_COPPER\n");
    out.push_str("E|7|8|6|EXACT_BRIDGE_JUNCTION_LOCAL_COPPER\n");
    out.push_str("COUNT|NODES|9|EDGES|8|LOCAL_COMPONENTS|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n");
    out.push_str("END|SOURCE_LOCAL_BRIDGE_ONLY_NO_GLOBAL_PASS\n");
    Ok(out)
}
