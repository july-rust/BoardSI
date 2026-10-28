//! C_005: source-bound, bounded Qt risk-audit transport. No physical SI.
use crate::{copper_preview, reference_topology, route_preview};
use std::collections::BTreeSet;
use std::fmt::Write;

const TICK_NM: i64 = 254;
fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut out, "{byte:02x}").expect("writing to String");
    }
    out
}
fn nm(tick: i64) -> Result<i64, &'static str> {
    tick.checked_mul(TICK_NM).ok_or("COORD_OVERFLOW")
}
/// Selection uses source-order segment indices, but every resulting row carries
/// its complete source-bound ObjectId parts and independently parsed coordinates.
/// The Qt consumer must compare these with its accepted imported artwork.
pub fn encode(
    bytes: &[u8],
    indices: &[usize],
    reference_net: &str,
    reference_layer_numbers: &[usize],
) -> Result<String, &'static str> {
    if bytes.is_empty()
        || bytes.len() > route_preview::MAX_FILE_BYTES
        || indices.is_empty()
        || indices.len() > 2
        || indices.len() != reference_layer_numbers.len()
        || reference_net.is_empty()
        || reference_net.len() > 64
        || reference_net
            .chars()
            .any(|c| c == '\n' || c == '\r' || c == '|')
    {
        return Err("INVALID_REQUEST");
    }
    let parsed = route_preview::parse(bytes)?;
    let mut board = route_preview::into_canonical(bytes, &parsed)?;
    let copper = copper_preview::parse(bytes, parsed.max_layer)?;
    copper_preview::augment(&mut board, &copper)?;
    if board.validate().has_errors() || board.traces.len() != parsed.segments.len() {
        return Err("CANONICAL_INVALID");
    }
    let mut unique = BTreeSet::new();
    let mut ids = Vec::with_capacity(indices.len());
    let mut layers = Vec::with_capacity(indices.len());
    for (&index, &layer_number) in indices.iter().zip(reference_layer_numbers) {
        if !unique.insert(index) || layer_number == 0 || layer_number > board.stackup.layers.len() {
            return Err("INVALID_SELECTION");
        }
        let trace = board.traces.get(index).ok_or("TRACE_NOT_FOUND")?;
        ids.push(trace.meta.id.clone());
        layers.push(board.stackup.layers[layer_number - 1].meta.id.clone());
    }
    let result = reference_topology::audit(&board, bytes, &ids, reference_net, &layers);
    let source = board.sources.first().ok_or("MISSING_SOURCE")?;
    let file = source.id.files.first().ok_or("MISSING_FILE_SHA")?;
    if source.id.files.len() != 1 {
        return Err("AMBIGUOUS_SOURCE");
    }
    let marker_count = result.risk_source_ticks.len();
    if marker_count > 1
        || (marker_count == 1
            && result.state != "REFERENCE_SPLIT_RISK"
            && result.state != "LAYER_TRANSITION_RISK")
    {
        return Err("INVALID_AUDIT_WITNESS");
    }
    let mut out = format!(
        "BOARDSI_REFERENCE_RISK_V1|{}|{}|{}|{}|0|UNSUPPORTED|NOT_RUN|0\n",
        hex(&file.sha256),
        result.state,
        result.reason,
        indices.len()
    );
    for (&index, trace) in indices.iter().zip(&ids) {
        let item = &board.traces[index];
        if item.centreline.len() != 2 {
            return Err("UNSUPPORTED_TRACE");
        }
        let a = item.centreline[0].ticks;
        let b = item.centreline[1].ticks;
        let name = board
            .nets
            .iter()
            .find(|net| Some(&net.meta.id) == item.net.as_ref())
            .ok_or("UNKNOWN_TRACE_NET")?;
        writeln!(
            &mut out,
            "T|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            index,
            board
                .stackup
                .layers
                .iter()
                .position(|layer| layer.meta.id == item.layer)
                .ok_or("MISSING_TRACE_LAYER")?
                + 1,
            hex(name.name.as_bytes()),
            hex(trace.source_object.as_bytes()),
            if trace.part.is_empty() {
                "-".to_owned()
            } else {
                hex(trace.part.as_bytes())
            },
            nm(a[0])?,
            nm(a[1])?,
            nm(b[0])?,
            nm(b[1])?
        )
        .map_err(|_| "FORMAT")?;
    }
    if let Some(point) = result.risk_source_ticks.first() {
        let via = result
            .via_id
            .as_ref()
            .map(|v| hex(v.source_object.as_bytes()))
            .unwrap_or_else(|| "-".to_string());
        writeln!(&mut out, "M|{}|{}|{}", nm(point[0])?, nm(point[1])?, via)
            .map_err(|_| "FORMAT")?;
    }
    writeln!(
        &mut out,
        "END|{}|{}|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL",
        indices.len(),
        marker_count
    )
    .map_err(|_| "FORMAT")?;
    if out.len() > 4096 {
        return Err("OUTPUT_LIMIT");
    }
    Ok(out)
}
