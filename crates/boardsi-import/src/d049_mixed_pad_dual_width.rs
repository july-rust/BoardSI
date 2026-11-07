//! D_049: compose two independently qualified same-layer finite-width contacts.
//! SOURCE-local synthetic geometry only; NOT a physical or global electrical proof.
use crate::{
    d040_multi_component, d047_trace_square_pad_width, d048_trace_round_pad_width, digest,
    route_preview,
};
use std::fmt::Write;

fn without_pair(bytes: &[u8], a: &str, b: &str) -> Result<Vec<u8>, &'static str> {
    let source = std::str::from_utf8(bytes).map_err(|_| "D049_UTF8")?;
    let lines: Vec<_> = source.lines().collect();
    let mut result = String::new();
    let mut count = 0;
    let mut in_signal = false;
    let mut i = 0;
    while i < lines.len() {
        let fields: Vec<_> = lines[i].split_ascii_whitespace().collect();
        if fields.first() == Some(&"*SIGNAL*") {
            in_signal = true;
        } else if fields.first() == Some(&"*END*") {
            in_signal = false;
        }
        if fields.as_slice() == [a, b] {
            if !in_signal
                || count != 0
                || i + 2 >= lines.len()
                || lines[i + 1].split_ascii_whitespace().count() != 5
                || lines[i + 2].split_ascii_whitespace().count() != 5
            {
                return Err("D049_PAIR_INVENTORY");
            }
            count += 1;
            i += 3;
        } else {
            result.push_str(lines[i]);
            result.push('\n');
            i += 1;
        }
    }
    if count != 1 {
        return Err("D049_MISSING_OR_DUPLICATE_PAIR");
    }
    Ok(result.into_bytes())
}

fn equal_track(a: &route_preview::Segment, b: &route_preview::Segment) -> bool {
    a.layer == b.layer
        && a.net == b.net
        && a.width_ticks == b.width_ticks
        && ((a.a == b.a && a.b == b.b) || (a.a == b.b && a.b == b.a))
}

fn exact_extra<'a>(
    source: &'a route_preview::Artwork,
    baseline: &route_preview::Artwork,
) -> Result<&'a route_preview::Segment, &'static str> {
    if source.skipped != 0
        || !source.vias.is_empty()
        || !source.arcs.is_empty()
        || source.segments.len() != 5
    {
        return Err("D049_SINGLE_SOURCE_INVENTORY");
    }
    let extras: Vec<_> = source
        .segments
        .iter()
        .filter(|t| !baseline.segments.iter().any(|b| equal_track(t, b)))
        .collect();
    if extras.len() != 1 {
        return Err("D049_SINGLE_CONTACT_INVENTORY");
    }
    Ok(extras[0])
}

/// Exact bounded D_047 + D_048 SOURCE evidence, rebuilt into one fresh graph.
/// Neither legacy wire is concatenated or trusted as a combined receipt.
pub fn encode(bytes: &[u8]) -> Result<String, &'static str> {
    if bytes.is_empty() || bytes.len() > route_preview::MAX_FILE_BYTES {
        return Err("D049_SOURCE_SIZE");
    }
    let round_only = without_pair(bytes, "U1.2", "U2.2")?;
    let square_only = without_pair(bytes, "U1.1", "U2.1")?;
    let baseline_bytes = without_pair(&round_only, "U1.1", "U2.1")?;
    // Full inventory, exact positive-area geometry and pin-pair provenance
    // are separately proven by the existing fixed-support validators.
    d048_trace_round_pad_width::encode(&round_only)?;
    d047_trace_square_pad_width::encode(&square_only)?;
    let baseline_wire = d040_multi_component::encode(&baseline_bytes)?;
    let base = route_preview::parse(&baseline_bytes)?;
    let round_source = route_preview::parse(&round_only)?;
    let square_source = route_preview::parse(&square_only)?;
    let complete = route_preview::parse(bytes)?;
    if base.skipped != 0
        || !base.arcs.is_empty()
        || !base.vias.is_empty()
        || base.segments.len() != 4
        || complete.skipped != 0
        || !complete.arcs.is_empty()
        || !complete.vias.is_empty()
        || complete.segments.len() != 6
    {
        return Err("D049_COMPLETE_INVENTORY");
    }
    let round = exact_extra(&round_source, &base)?;
    let square = exact_extra(&square_source, &base)?;
    if equal_track(round, square) {
        return Err("D049_DUPLICATE_CONTACT");
    }
    let mut used = [false; 6];
    for expected in base
        .segments
        .iter()
        .chain(std::iter::once(round))
        .chain(std::iter::once(square))
    {
        let candidates: Vec<_> = complete
            .segments
            .iter()
            .enumerate()
            .filter(|(i, actual)| !used[*i] && equal_track(actual, expected))
            .map(|(i, _)| i)
            .collect();
        if candidates.len() != 1 {
            return Err("D049_MISMATCHED_FULL_SOURCE");
        }
        used[candidates[0]] = true;
    }
    if used.iter().any(|matched| !matched) {
        return Err("D049_EXTRA_SOURCE");
    }
    let net = &base.segments[0].net;
    let mut sha = String::new();
    for byte in digest::sha256(bytes) {
        write!(&mut sha, "{byte:02x}").map_err(|_| "D049_FORMAT")?;
    }
    let mut result = format!(
        "BOARDSI_D049_MIXED_PAD_DUAL_WIDTH_V1|{sha}|1|SOURCE_LOCAL_MIXED_PAD_DUAL_FINITE_WIDTH_ONLY|P009_UNSUPPORTED|SI_NOT_RUN|GLOBAL_ELECTRICAL_NOT_VERIFIED\n"
    );
    writeln!(&mut result, "C|0|U1|{net}|1|6|5").map_err(|_| "D049_FORMAT")?;
    writeln!(&mut result, "C|1|U2|{net}|1|4|3").map_err(|_| "D049_FORMAT")?;
    for row in baseline_wire
        .lines()
        .filter(|r| r.starts_with("P|") || r.starts_with("T|"))
    {
        writeln!(&mut result, "{row}").map_err(|_| "D049_FORMAT")?;
    }
    for (index, track) in [(4, round), (5, square)] {
        writeln!(
            &mut result,
            "T|{index}|{}|{}|{}|{}|{}|{}|{}",
            track.layer,
            track.net,
            track.a[0],
            track.a[1],
            track.b[0],
            track.b[1],
            track.width_ticks
        )
        .map_err(|_| "D049_FORMAT")?;
    }
    for row in baseline_wire.lines().filter(|r| r.starts_with("E|")) {
        writeln!(&mut result, "{row}").map_err(|_| "D049_FORMAT")?;
    }
    result.push_str(
        "E|6|0|8|POSITIVE_FINITE_WIDTH_TRACE_ROUND_PAD_CONTACT\n\
         E|7|1|9|POSITIVE_FINITE_WIDTH_TRACE_PAD_CONTACT\n\
         COUNT|NODES|10|EDGES|8|LOCAL_COMPONENTS|2|GLOBAL_ELECTRICAL|NOT_VERIFIED\n\
         END|SOURCE_LOCAL_MIXED_PAD_DUAL_FINITE_WIDTH_ONLY_NO_GLOBAL_PASS\n",
    );
    Ok(result)
}
