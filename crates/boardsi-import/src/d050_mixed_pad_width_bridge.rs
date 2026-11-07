//! D_050: SOURCE-local finite-width bridge connecting D_049 local components.
//! This is strictly bounded geometric evidence, NOT physical or global electrical proof.
use crate::{d049_mixed_pad_dual_width, digest, route_preview};
use std::fmt::Write;

fn remove_candidate(rows: &[&str], index: usize) -> Result<Vec<u8>, &'static str> {
    if index + 2 >= rows.len()
        || rows[index + 1].split_ascii_whitespace().count() != 5
        || rows[index + 2].split_ascii_whitespace().count() != 5
    {
        return Err("D050_ROUTE_VERTICES");
    }
    let mut reduced = String::new();
    for (i, row) in rows.iter().enumerate() {
        if (index..index + 3).contains(&i) {
            continue;
        }
        reduced.push_str(row);
        reduced.push('\n');
    }
    Ok(reduced.into_bytes())
}

fn match_track(a: &route_preview::Segment, b: &route_preview::Segment) -> bool {
    a.layer == b.layer
        && a.net == b.net
        && a.width_ticks == b.width_ticks
        && ((a.a == b.a && a.b == b.b) || (a.a == b.b && a.b == b.a))
}

/// Specialized exact squared-distance geometry for the fixed horizontal
/// 8-mil bridge. Bounds exclude any extra U1 round-pad contact, and require
/// that the centerline remains outside U2's round Pad.
fn positive_bridge(track: &route_preview::Segment, net: &str) -> bool {
    if track.layer != 1 || track.net != net || track.width_ticks != 800 {
        return false;
    }
    let (a, b) = if track.a[0] < track.b[0] {
        (track.a, track.b)
    } else {
        (track.b, track.a)
    };
    if a[1] != 30_000 || b[1] != 30_000
        || a[0] <= 22_400 || a[0] >= 23_000
        || b[0] <= 37_600 || b[0] >= 38_000
    {
        return false;
    }
    // Shortest centerline distance to vertical x=22200 y=28500..31500
    // equals a.x-22200; the halfwidth sum is 800 SOURCE ticks.
    let trace_distance = i128::from(a[0] - 22_200);
    let to_u1_pad = i128::from(a[0] - 20_000);
    // Nearest bridge endpoint to U2.1 circular SMD pad center=(40000,30000).
    let pad_distance = i128::from(40_000 - b[0]);
    trace_distance * trace_distance < 800_i128 * 800
        && to_u1_pad * to_u1_pad > 2_400_i128 * 2_400
        && pad_distance * pad_distance > 2_000_i128 * 2_000
        && pad_distance * pad_distance < 2_400_i128 * 2_400
}

fn receipt_track(row: &str) -> Result<route_preview::Segment, &'static str> {
    let f: Vec<_> = row.split('|').collect();
    if f.len() != 9 || f[0] != "T" {
        return Err("D050_D049_TRACK_RECEIPT");
    }
    Ok(route_preview::Segment {
        layer: f[2].parse().map_err(|_| "D050_RECEIPT_LAYER")?,
        net: f[3].to_owned(),
        a: [
            f[4].parse().map_err(|_| "D050_RECEIPT_X0")?,
            f[5].parse().map_err(|_| "D050_RECEIPT_Y0")?,
        ],
        b: [
            f[6].parse().map_err(|_| "D050_RECEIPT_X1")?,
            f[7].parse().map_err(|_| "D050_RECEIPT_Y1")?,
        ],
        width_ticks: f[8].parse().map_err(|_| "D050_RECEIPT_WIDTH")?,
        source_line: 0,
    })
}

/// Only one complete 7-trace SOURCE candidate whose removal passes D_049
/// may qualify. Full artwork must match all six old tracks one-to-one.
pub fn encode(bytes: &[u8]) -> Result<String, &'static str> {
    if bytes.is_empty() || bytes.len() > route_preview::MAX_FILE_BYTES {
        return Err("D050_SOURCE_SIZE");
    }
    let complete = route_preview::parse(bytes)?;
    if complete.skipped != 0 || !complete.vias.is_empty() || !complete.arcs.is_empty()
        || complete.segments.len() != 7
    {
        return Err("D050_SOURCE_INVENTORY");
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "D050_UTF8")?;
    let rows: Vec<_> = text.lines().collect();
    let mut signal = false;
    let mut selected: Option<(Vec<u8>, String)> = None;
    for i in 0..rows.len() {
        let fields: Vec<_> = rows[i].split_ascii_whitespace().collect();
        if fields.first() == Some(&"*SIGNAL*") {
            signal = true;
        } else if fields.first() == Some(&"*END*") {
            signal = false;
        }
        if signal && fields == ["U1.1", "U2.1"] {
            let reduced = remove_candidate(&rows, i)?;
            if let Ok(d049) = d049_mixed_pad_dual_width::encode(&reduced) {
                if selected.is_some() {
                    return Err("D050_AMBIGUOUS_SOURCE_ROUTE");
                }
                selected = Some((reduced, d049));
            }
        }
    }
    let (reduced, old_wire) = selected.ok_or("D050_BRIDGE_MISSING")?;
    let accepted = route_preview::parse(&reduced)?;
    if accepted.segments.len() != 6 || accepted.skipped != 0 {
        return Err("D050_REDUCED_INVENTORY");
    }
    let mut old_tracks = Vec::new();
    for row in old_wire.lines().filter(|r| r.starts_with("T|")) {
        old_tracks.push(receipt_track(row)?);
    }
    if old_tracks.len() != 6 {
        return Err("D050_RECEIPT_INVENTORY");
    }
    // D_050 deliberately fixes the inherited supplementary traces: D_049
    // permits alternate round-cap witnesses that would no longer touch this
    // bridge, so those cannot be promoted to a D_050 connectivity edge.
    let round = &old_tracks[4];
    let square = &old_tracks[5];
    let exact_ends = |t: &route_preview::Segment, a: [i64; 2], b: [i64; 2]| {
        (t.a == a && t.b == b) || (t.a == b && t.b == a)
    };
    if round.width_ticks != 800
        || square.width_ticks != 800
        || !exact_ends(round, [22_200, 28_500], [22_200, 31_500])
        || !exact_ends(square, [21_700, 39_000], [21_700, 41_000])
    {
        return Err("D050_INHERITED_TRACE_GEOMETRY");
    }
    let mut used = [false; 7];
    for expected in &old_tracks {
        let matches: Vec<_> = complete.segments.iter().enumerate()
            .filter(|(i, actual)| !used[*i] && match_track(actual, expected))
            .map(|(i, _)| i).collect();
        if matches.len() != 1 {
            return Err("D050_SOURCE_NOT_ONE_TO_ONE");
        }
        used[matches[0]] = true;
    }
    let bridge_indices: Vec<_> = used.iter().enumerate()
        .filter_map(|(i, matched)| if !matched { Some(i) } else { None }).collect();
    if bridge_indices.len() != 1 {
        return Err("D050_BRIDGE_IDENTITY");
    }
    let bridge = &complete.segments[bridge_indices[0]];
    let net = &old_tracks[0].net;
    if !positive_bridge(bridge, net) {
        return Err("D050_BRIDGE_GEOMETRY");
    }
    // All six inherited geometric edges are requalified by D_049 on the
    // independent reduced fixture, not by trusting any supplied receipt.
    let mut sha = String::new();
    for octet in digest::sha256(bytes) {
        write!(&mut sha, "{octet:02x}").map_err(|_| "D050_FORMAT")?;
    }
    let mut wire = format!(
        "BOARDSI_D050_MIXED_PAD_WIDTH_BRIDGE_V1|{sha}|1|SOURCE_LOCAL_MIXED_PAD_FINITE_WIDTH_BRIDGE_ONLY|P009_UNSUPPORTED|SI_NOT_RUN|GLOBAL_ELECTRICAL_NOT_VERIFIED\n"
    );
    writeln!(&mut wire, "C|0|U1_U2|{net}|1|11|10").map_err(|_| "D050_FORMAT")?;
    for row in old_wire.lines().filter(|r| r.starts_with("P|") || r.starts_with("T|")) {
        writeln!(&mut wire, "{row}").map_err(|_| "D050_FORMAT")?;
    }
    let (a, b) = if bridge.a[0] < bridge.b[0] {
        (bridge.a, bridge.b)
    } else {
        (bridge.b, bridge.a)
    };
    writeln!(
        &mut wire, "T|6|1|{net}|{}|{}|{}|{}|800",
        a[0], a[1], b[0], b[1]
    ).map_err(|_| "D050_FORMAT")?;
    for row in old_wire.lines().filter(|r| r.starts_with("E|")) {
        writeln!(&mut wire, "{row}").map_err(|_| "D050_FORMAT")?;
    }
    wire.push_str(
        "E|8|8|10|POSITIVE_FINITE_WIDTH_TRACE_TRACE_BRIDGE\n\
         E|9|2|10|POSITIVE_FINITE_WIDTH_TRACE_ROUND_PAD_BRIDGE\n\
         COUNT|NODES|11|EDGES|10|LOCAL_COMPONENTS|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n\
         END|SOURCE_LOCAL_MIXED_PAD_FINITE_WIDTH_BRIDGE_ONLY_NO_GLOBAL_PASS\n",
    );
    Ok(wire)
}
