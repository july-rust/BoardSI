//! C_008: opt-in source-exact two-layer straight-route / plated via warning.
//! A geometric witness, never Return Path P009, physics, or solver qualification.
use crate::{reference_gap_intervals, route_preview};
use boardsi_model::{PadRole, ViaKind};
use std::collections::BTreeSet;
use std::fmt::Write;

const MAX_SEGMENTS: usize = 16;
const MAX_WIRE: usize = 150_000;
const NM_PER_TICK: i64 = 254;

fn cross(a: [i64; 2], b: [i64; 2], c: [i64; 2]) -> i128 {
    (i128::from(b[0]) - i128::from(a[0])) * (i128::from(c[1]) - i128::from(a[1]))
        - (i128::from(b[1]) - i128::from(a[1])) * (i128::from(c[0]) - i128::from(a[0]))
}
fn on(a: [i64; 2], b: [i64; 2], p: [i64; 2]) -> bool {
    cross(a, b, p) == 0
        && (a[0].min(b[0])..=a[0].max(b[0])).contains(&p[0])
        && (a[1].min(b[1])..=a[1].max(b[1])).contains(&p[1])
}
fn intersects(a: [i64; 2], b: [i64; 2], c: [i64; 2], d: [i64; 2]) -> bool {
    let (p, q, r, s) = (
        cross(a, b, c),
        cross(a, b, d),
        cross(c, d, a),
        cross(c, d, b),
    );
    (p == 0 && on(a, b, c))
        || (q == 0 && on(a, b, d))
        || (r == 0 && on(c, d, a))
        || (s == 0 && on(c, d, b))
        || ((p > 0) != (q > 0) && (r > 0) != (s > 0))
}
fn hex(value: &[u8]) -> String {
    let mut out = String::with_capacity(value.len() * 2);
    for v in value {
        write!(&mut out, "{v:02x}").expect("string");
    }
    out
}
fn nm(tick: i64) -> Result<i64, &'static str> {
    tick.checked_mul(NM_PER_TICK).ok_or("COORD_OVERFLOW")
}

/// Ordered segment IDs: indices [0..split] on first layer, remaining on second.
/// V4 embeds independently source-validated V2 receipts; V1/V2/V3 are untouched.
pub fn encode(
    bytes: &[u8],
    indices: &[usize],
    split: usize,
    references: [&str; 2],
    reference_layers: [usize; 2],
) -> Result<String, &'static str> {
    if !(2..=MAX_SEGMENTS).contains(&indices.len())
        || split == 0
        || split >= indices.len()
        || bytes.is_empty()
        || bytes.len() > route_preview::MAX_FILE_BYTES
        || references.iter().any(|r| {
            r.is_empty() || r.len() > 64 || r.bytes().any(|v| v == b'|' || v == b'\n' || v == b'\r')
        })
    {
        return Err("INVALID_V4_REQUEST");
    }
    let parsed = route_preview::parse(bytes)?;
    let board = route_preview::into_canonical(bytes, &parsed)?;
    if board.validate().has_errors() || board.vias.len() != parsed.vias.len() {
        return Err("CANONICAL_INVALID");
    }
    let mut seen = BTreeSet::new();
    let mut route = Vec::with_capacity(indices.len());
    for (order, &index) in indices.iter().enumerate() {
        if !seen.insert(index) {
            return Err("DUPLICATE_ROUTE_TRACE");
        }
        let trace = parsed.segments.get(index).ok_or("UNKNOWN_ROUTE_TRACE")?;
        if trace.a == trace.b || trace.net.is_empty() {
            return Err("INVALID_ROUTE_TRACE");
        }
        if let Some(first) = route.first() {
            let first: &&route_preview::Segment = first;
            if trace.net != first.net {
                return Err("MIXED_SIGNAL_NET");
            }
        }
        if let Some(prev) = route.last() {
            let prev: &&route_preview::Segment = prev;
            if prev.b != trace.a {
                return Err("ROUTE_ENDPOINT_DISCONNECTED_OR_REVERSED");
            }
            if order != split
                && (prev.layer != trace.layer
                    || (cross(prev.a, prev.b, trace.b) == 0
                        && (on(prev.a, prev.b, trace.b) || on(trace.a, trace.b, prev.a))))
            {
                return Err("INVALID_SAME_LAYER_JOINT");
            }
        }
        let expected_layer = if order < split {
            parsed.segments[indices[0]].layer
        } else {
            parsed.segments[indices[split]].layer
        };
        if trace.layer != expected_layer {
            return Err("MULTIPLE_LAYER_TRANSITIONS");
        }
        for (earlier_order, earlier) in route.iter().enumerate() {
            let earlier: &&route_preview::Segment = earlier;
            if earlier.layer == trace.layer
                && earlier_order + 1 != order
                && intersects(earlier.a, earlier.b, trace.a, trace.b)
            {
                return Err("SELF_INTERSECTING_ROUTE");
            }
        }
        route.push(trace);
    }
    let first_layer = route[0].layer;
    let second_layer = route[split].layer;
    if first_layer == second_layer {
        return Err("TWO_DISTINCT_LAYERS_REQUIRED");
    }
    if reference_layers
        .iter()
        .any(|&layer| layer == 0 || layer > parsed.max_layer as usize)
    {
        return Err("INVALID_REFERENCE_LAYER");
    }
    // A geometric crossing or an unselected contact on either route layer can
    // invalidate the selected straight-through path. Never walk by Net labels.
    for (index, candidate) in parsed.segments.iter().enumerate() {
        if seen.contains(&index) || candidate.net != route[0].net {
            continue;
        }
        if route.iter().any(|selected| {
            selected.layer == candidate.layer
                && intersects(selected.a, selected.b, candidate.a, candidate.b)
        }) {
            return Err("UNSELECTED_BRANCH_OR_CROSSING");
        }
    }
    if parsed
        .arcs
        .iter()
        .any(|a| a.net == route[0].net && (a.layer == first_layer || a.layer == second_layer))
    {
        return Err("SAME_NET_ARC_UNSUPPORTED");
    }
    let joint = route[split - 1].b;
    let candidates: Vec<_> = parsed
        .vias
        .iter()
        .enumerate()
        .filter(|(_, v)| {
            v.net == route[0].net
                && v.point == joint
                && v.first_layer <= first_layer
                && v.last_layer >= first_layer
                && v.first_layer <= second_layer
                && v.last_layer >= second_layer
        })
        .collect();
    if candidates.len() != 1 {
        return Err("VIA_TRANSITION_NOT_UNIQUE");
    }
    let (via_index, via) = candidates[0];
    if parsed.vias.iter().enumerate().any(|(i, v)| {
        i != via_index
            && v.net == route[0].net
            && route.iter().any(|s| {
                v.first_layer <= s.layer && v.last_layer >= s.layer && on(s.a, s.b, v.point)
            })
    }) {
        return Err("UNSELECTED_VIA_CONTACT");
    }
    let actual = &board.vias[via_index];
    if actual.kind != ViaKind::Through
        || actual.position.ticks != joint
        || actual.net != board.traces[indices[0]].net
    {
        return Err("SOURCE_VIA_CANONICAL_MISMATCH");
    }
    let padstack = board
        .padstacks
        .iter()
        .find(|p| p.meta.id == actual.padstack)
        .ok_or("UNKNOWN_VIA_PADSTACK")?;
    for &layer in &[first_layer, second_layer] {
        let id = &board.stackup.layers[(layer - 1) as usize].meta.id;
        if !actual.connected_layers.contains(id)
            || !padstack
                .entries
                .iter()
                .any(|entry| &entry.layer == id && entry.role == PadRole::Conductive)
        {
            return Err("VIA_CONDUCTIVE_PAD_MISSING");
        }
    }

    let mut receipts = Vec::with_capacity(indices.len());
    let mut source_sha = String::new();
    let mut gaps = 0usize;
    let mut unsupported = false;
    for (order, &index) in indices.iter().enumerate() {
        let leg = usize::from(order >= split);
        let wire =
            reference_gap_intervals::encode(bytes, index, references[leg], reference_layers[leg])?;
        let header: Vec<_> = wire
            .lines()
            .next()
            .ok_or("V2_HEADER_MISSING")?
            .split('|')
            .collect();
        if header.len() != 9 || header[0] != "BOARDSI_REFERENCE_GAP_V2" {
            return Err("V2_INVALID");
        }
        if source_sha.is_empty() {
            source_sha = header[1].to_owned();
        } else if source_sha != header[1] {
            return Err("SOURCE_SHA_MISMATCH");
        }
        match header[2] {
            "UNSUPPORTED" => unsupported = true,
            "REFERENCE_SPLIT_RISK" | "SOURCE_GEOMETRY_ONLY" => {}
            _ => return Err("V2_UNKNOWN_STATE"),
        }
        gaps += wire.lines().filter(|line| line.starts_with("I|")).count();
        if gaps > MAX_SEGMENTS * 32 {
            return Err("INTERVAL_LIMIT");
        }
        receipts.push(wire);
    }
    if source_sha.len() != 64 {
        return Err("SOURCE_HASH_INVALID");
    }
    let status = if unsupported {
        "UNSUPPORTED"
    } else if gaps > 0 {
        "REFERENCE_SPLIT_RISK"
    } else {
        "SOURCE_GEOMETRY_ONLY"
    };
    let mut out = format!(
        "BOARDSI_REFERENCE_ROUTE_V4|{source_sha}|{}|{split}|UNSUPPORTED|NOT_RUN|0|REFERENCE_TRANSFER_UNVERIFIED|{status}\n",
        indices.len()
    );
    writeln!(
        &mut out,
        "VIA|{via_index}|{}|{}|{}|{}|{}|{}|{first_layer}|{second_layer}",
        hex(via.name.as_bytes()),
        hex(actual.meta.id.source_object.as_bytes()),
        nm(joint[0])?,
        nm(joint[1])?,
        via.first_layer,
        via.last_layer
    )
    .map_err(|_| "FORMAT")?;
    for (&index, wire) in indices.iter().zip(&receipts) {
        writeln!(&mut out, "SEG|{index}|{}", wire.lines().count()).map_err(|_| "FORMAT")?;
        out.push_str(wire);
    }
    writeln!(
        &mut out,
        "END|{}|{gaps}|1|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL",
        indices.len()
    )
    .map_err(|_| "FORMAT")?;
    if out.len() > MAX_WIRE {
        return Err("V4_OUTPUT_LIMIT");
    }
    Ok(out)
}
