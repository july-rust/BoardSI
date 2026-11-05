//! D_039: opt-in, strictly bounded local same-layer copper geometry graph.
//! Two source-owned SMD pads joined by two positive-width straight traces.
//! This does NOT prove manufactured, global electrical or SI continuity.
use crate::{d001_pad_endpoint, digest, part_preview, route_preview};
use std::fmt::Write;

fn bounded_sections(source: &[u8]) -> Result<(), &'static str> {
    let text = std::str::from_utf8(source).map_err(|_| "D039_UTF8")?;
    let mut counts = [0usize; 7];
    let mut ended = false;
    for line in text.lines().skip(1) {
        let trimmed = line.trim();
        if ended {
            if !trimmed.is_empty() {
                return Err("D039_AFTER_END");
            }
            continue;
        }
        let tag = trimmed.split_ascii_whitespace().next().unwrap_or("");
        if !tag.starts_with('*') {
            continue;
        }
        let index = match tag {
            "*PCB*" => 0,
            "*PARTDECAL*" => 1,
            "*PARTTYPE*" => 2,
            "*PART*" => 3,
            "*ROUTE*" => 4,
            "*SIGNAL*" => 5,
            "*END*" => 6,
            "*REMARK*" => continue,
            _ => return Err("D039_UNSUPPORTED_SECTION"),
        };
        counts[index] += 1;
        if counts[index] != 1 {
            return Err("D039_DUPLICATE_SECTION");
        }
        if tag == "*END*" {
            ended = true;
        }
    }
    if counts != [1, 1, 1, 1, 1, 1, 1] {
        return Err("D039_SOURCE_INVENTORY");
    }
    Ok(())
}

fn strict_between(a: i64, m: i64, b: i64) -> bool {
    (a < m && m < b) || (b < m && m < a)
}

/// Creates no global Canonical edges. The result is an isolated diagnostic
/// whose source identity, complete bounded inventory and copper footprints
/// have been independently checked via existing D_001 and canonical validators.
pub fn encode(bytes: &[u8], net: &str, layer: u32) -> Result<String, &'static str> {
    if bytes.is_empty() || bytes.len() > route_preview::MAX_FILE_BYTES {
        return Err("D039_SOURCE_SIZE");
    }
    if layer != 1
        || net.is_empty()
        || net.len() > 64
        || !net.bytes().all(|b| b.is_ascii_graphic() && b != b'|')
    {
        return Err("D039_SELECTION_UNSUPPORTED");
    }
    bounded_sections(bytes)?;
    let route = route_preview::parse(bytes)?;
    if route.skipped != 0
        || route.segments.len() != 2
        || !route.arcs.is_empty()
        || !route.vias.is_empty()
    {
        return Err("D039_ROUTE_INCOMPLETE");
    }
    let pads = part_preview::parse(bytes, route.max_layer);
    if pads.skipped != 0 || pads.pads.len() != 2 {
        return Err("D039_PAD_INVENTORY");
    }
    let audit = d001_pad_endpoint::audit(bytes)?;
    if audit.ambiguous_owners != 0
        || audit.cross_net != 0
        || audit.unproven_net != 0
        || audit.witnesses.len() != 2
    {
        return Err("D039_PAD_CONTACT_NOT_PROVEN");
    }
    // (Pad, opposite trace endpoint, source trace width). One witness per
    // trace and one unique pad owner; pad centre is strict overlap, not tangency.
    let mut paths: Vec<(&part_preview::ComponentPad, [i64; 2], i64)> = Vec::new();
    for (trace_index, segment) in route.segments.iter().enumerate() {
        if segment.layer != layer || segment.net != net || segment.width_ticks <= 0 {
            return Err("D039_TRACE_LAYER_NET");
        }
        let matches: Vec<_> = audit
            .witnesses
            .iter()
            .filter(|w| w.trace_index == trace_index)
            .collect();
        if matches.len() != 1 {
            return Err("D039_TRACE_PAD_AMBIGUOUS");
        }
        let witness = matches[0];
        let pad = &pads.pads[witness.pad_index];
        if pad.bottom
            || pad.layer != layer
            || pad.rectangular
            || pad.width_ticks <= 0
            || pad.width_ticks != pad.height_ticks
            || segment.width_ticks > pad.width_ticks / 2
        {
            return Err("D039_PAD_UNSUPPORTED");
        }
        let point = if witness.endpoint == 0 {
            segment.a
        } else {
            segment.b
        };
        let joint = if witness.endpoint == 0 {
            segment.b
        } else {
            segment.a
        };
        if witness.layer != layer || point != witness.point || point != pad.point {
            return Err("D039_PAD_SOURCE_MISMATCH");
        }
        paths.push((pad, joint, segment.width_ticks));
    }
    if paths[0].0.reference == paths[1].0.reference && paths[0].0.pin == paths[1].0.pin {
        return Err("D039_DUPLICATE_PAD_OWNER");
    }
    if paths[0].1 != paths[1].1 || paths[0].2 != paths[1].2 {
        return Err("D039_TRACE_JUNCTION_GAP_OR_WIDTH");
    }
    let a = paths[0].0.point;
    let b = paths[1].0.point;
    let middle = paths[0].1;
    if !((a[0] == b[0] && middle[0] == a[0] && strict_between(a[1], middle[1], b[1]))
        || (a[1] == b[1] && middle[1] == a[1] && strict_between(a[0], middle[0], b[0])))
    {
        return Err("D039_PATH_NOT_BOUNDED_COLLINEAR");
    }
    // The junction must be outside each footprint, so each trace contributes
    // an independently distinguishable conductive segment.
    let delta_a = (middle[0] - a[0]).abs() + (middle[1] - a[1]).abs();
    let delta_b = (middle[0] - b[0]).abs() + (middle[1] - b[1]).abs();
    if 2 * delta_a <= paths[0].0.width_ticks || 2 * delta_b <= paths[1].0.width_ticks {
        return Err("D039_JUNCTION_INSIDE_PAD");
    }
    paths.sort_by(|left, right| {
        (&left.0.reference, &left.0.pin).cmp(&(&right.0.reference, &right.0.pin))
    });
    let mut sha = String::with_capacity(64);
    for byte in digest::sha256(bytes) {
        write!(&mut sha, "{byte:02x}").map_err(|_| "D039_FORMAT")?;
    }
    let mut wire = format!(
        "BOARDSI_D039_LOCAL_COPPER_V1|{sha}|{net}|{layer}|LOCAL_COPPER_GEOMETRY_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|GLOBAL_ELECTRICAL_NOT_VERIFIED\n"
    );
    for (index, (pad, _, _)) in paths.iter().enumerate() {
        let shape = if pad.square { "S" } else { "R" };
        writeln!(
            &mut wire,
            "P|{index}|{}|{}|{}|{}|{}|{}|{}|{shape}",
            pad.reference,
            pad.pin,
            pad.layer,
            pad.point[0],
            pad.point[1],
            pad.width_ticks,
            pad.height_ticks
        )
        .map_err(|_| "D039_FORMAT")?;
    }
    for (index, (pad, junction, width)) in paths.iter().enumerate() {
        writeln!(
            &mut wire,
            "T|{index}|{layer}|{}|{}|{}|{}|{width}",
            pad.point[0], pad.point[1], junction[0], junction[1]
        )
        .map_err(|_| "D039_FORMAT")?;
    }
    wire.push_str(
        "E|0|0|2|PAD_TRACE_LOCAL_COPPER\n\
         E|1|1|3|PAD_TRACE_LOCAL_COPPER\n\
         E|2|2|3|EXACT_TRACE_JUNCTION_LOCAL_COPPER\n\
         COUNT|NODES|4|EDGES|3|LOCAL_COMPONENTS|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n\
         END|LOCAL_COPPER_GEOMETRY_ONLY_NO_GLOBAL_PASS\n",
    );
    Ok(wire)
}
