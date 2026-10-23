//! Offline local PADS ASC (V9.4 MILS only) -> partial Canonical Model -> Qt line protocol.
//! Standard output is a bounded transport, never an SI simulation or full CAD import.
use boardsi_import::outline_preview;
use boardsi_import::part_preview::{self, MAX_PADS};
use boardsi_import::route_preview::{into_canonical, parse, MAX_FILE_BYTES};
use std::{
    env, fs,
    io::{self, Write},
};
// 0.01 mil canonical tick = 254 nm. All Qt wire coordinates remain nm.
const NM: i64 = 254;
fn run() -> Result<(), &'static str> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 2 {
        return Err("USAGE");
    }
    let path = &args[1];
    let meta = fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
    if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
        return Err("INVALID_FILE_SIZE");
    }
    let bytes = fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
    let result = parse(&bytes)?;
    let outlines = outline_preview::parse(&bytes)?;
    let mut board = into_canonical(&bytes, &result)?;
    let pad_data = part_preview::parse(&bytes, result.max_layer);
    part_preview::augment(&mut board, &pad_data)?;
    outline_preview::augment(&mut board, &outlines)?;
    if board.arcs.len() != result.arcs.len()
        || board.traces.len() != result.segments.len()
        || board.vias.len() != result.vias.len()
        || board.pads.len() != pad_data.pads.len()
        || board.outlines.len() != usize::from(!outlines.is_empty())
        || board.validate().has_errors()
    {
        return Err("CANONICAL_VALIDATION_FAILED");
    }
    let mut out = Vec::new();
    // Preserve the established P2A/P2B V1 wire contract whenever no pad
    // records are actually emitted. Rejected PART rows are not P2C geometry.
    let mode = if !outlines.is_empty() {
        "BOARDSI_P2G_PARTIAL_V6"
    } else if !result.arcs.is_empty() {
        "BOARDSI_P2F_PARTIAL_V5"
    } else if pad_data.pads.iter().any(|pad| pad.bottom) {
        "BOARDSI_P2E_PARTIAL_V4"
    } else if pad_data.pads.is_empty() {
        "BOARDSI_P2A_PARTIAL_V1"
    } else if pad_data.pads.iter().any(|pad| pad.rectangular) {
        "BOARDSI_P2D_PARTIAL_V3"
    } else {
        "BOARDSI_P2C_PARTIAL_V2"
    };
    writeln!(out, "{}|V9.4-MILS|{}", mode, result.max_layer).map_err(|_| "FORMAT_ERROR")?;
    for layer in &board.stackup.layers {
        writeln!(out, "L|{}|{}", layer.order + 1, layer.name).map_err(|_| "FORMAT_ERROR")?;
    }
    for segment in &result.segments {
        let a = [
            segment.a[0].checked_mul(NM).ok_or("COORD_OVERFLOW")?,
            segment.a[1].checked_mul(NM).ok_or("COORD_OVERFLOW")?,
        ];
        let b = [
            segment.b[0].checked_mul(NM).ok_or("COORD_OVERFLOW")?,
            segment.b[1].checked_mul(NM).ok_or("COORD_OVERFLOW")?,
        ];
        writeln!(
            out,
            "T|{}|{}|{}|{}|{}|{}|{}",
            segment.layer,
            segment.net,
            a[0],
            a[1],
            b[0],
            b[1],
            segment
                .width_ticks
                .checked_mul(NM)
                .ok_or("WIDTH_OVERFLOW")?
        )
        .map_err(|_| "FORMAT_ERROR")?;
    }
    for arc in &result.arcs {
        let coord = |x: i64| x.checked_mul(NM).ok_or("ARC_COORD_OVERFLOW");
        writeln!(
            out,
            "A|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            arc.layer,
            arc.net,
            coord(arc.start[0])?,
            coord(arc.start[1])?,
            coord(arc.centre[0])?,
            coord(arc.centre[1])?,
            coord(arc.end[0])?,
            coord(arc.end[1])?,
            if arc.clockwise { "CW" } else { "CCW" },
            coord(arc.width_ticks)?,
            arc.source_line
        )
        .map_err(|_| "FORMAT_ERROR")?;
    }
    for via in &result.vias {
        writeln!(
            out,
            "V|{}|{}|{}|{}|{}|{}|{}|{}",
            via.net,
            via.name,
            via.point[0].checked_mul(NM).ok_or("COORD_OVERFLOW")?,
            via.point[1].checked_mul(NM).ok_or("COORD_OVERFLOW")?,
            via.diameter_ticks
                .checked_mul(NM)
                .ok_or("VIA_DIMENSION_OVERFLOW")?,
            via.drill_ticks
                .checked_mul(NM)
                .ok_or("VIA_DIMENSION_OVERFLOW")?,
            via.first_layer,
            via.last_layer,
        )
        .map_err(|_| "FORMAT_ERROR")?;
    }
    if pad_data.pads.len() > MAX_PADS {
        return Err("PAD_LIMIT");
    }
    for pad in &pad_data.pads {
        // Geometry is restricted and axis-aligned after component rotation;
        // unknown custom shapes are never interpreted as RF.
        writeln!(
            out,
            "P|{}|{}|{}|{}|{}|{}|{}|{}|NET_UNVERIFIED{}",
            pad.reference,
            pad.pin,
            pad.layer,
            if pad.rectangular {
                "RF"
            } else if pad.square {
                "S"
            } else {
                "R"
            },
            pad.point[0].checked_mul(NM).ok_or("PAD_COORD_OVERFLOW")?,
            pad.point[1].checked_mul(NM).ok_or("PAD_COORD_OVERFLOW")?,
            pad.width_ticks.checked_mul(NM).ok_or("PAD_SIZE_OVERFLOW")?,
            pad.height_ticks
                .checked_mul(NM)
                .ok_or("PAD_SIZE_OVERFLOW")?,
            if mode == "BOARDSI_P2E_PARTIAL_V4"
                || mode == "BOARDSI_P2F_PARTIAL_V5"
                || mode == "BOARDSI_P2G_PARTIAL_V6"
            {
                format!(
                    "|{}|{}",
                    if pad.bottom { "B" } else { "T" },
                    u32::from(pad.rotation) * 90
                )
            } else {
                String::new()
            }
        )
        .map_err(|_| "FORMAT_ERROR")?;
    }
    if mode == "BOARDSI_P2G_PARTIAL_V6" {
        // Non-electrical BOARD perimeter edges. Never mix them with copper T/A.
        for shape in &outlines {
            for (n, pair) in shape.vertices.windows(2).enumerate() {
                writeln!(
                    out,
                    "O|{}|{}|{}|{}|{}|{}|{}",
                    if shape.cutout { "CUT" } else { "OUTER" },
                    shape.name,
                    n,
                    outline_preview::wire_nm(pair[0][0])?,
                    outline_preview::wire_nm(pair[0][1])?,
                    outline_preview::wire_nm(pair[1][0])?,
                    outline_preview::wire_nm(pair[1][1])?,
                )
                .map_err(|_| "FORMAT_ERROR")?;
            }
        }
        writeln!(
            out,
            "END|{}|{}|{}|{}|{}|{}|CANONICAL_PARTIAL|SI_NOT_RUN",
            result.segments.len(),
            result.skipped.saturating_add(pad_data.skipped as u32),
            pad_data.pads.len(),
            result.arcs.len(),
            outlines.iter().filter(|s| !s.cutout).count(),
            outlines.iter().filter(|s| s.cutout).count()
        )
    } else if mode == "BOARDSI_P2F_PARTIAL_V5" {
        writeln!(
            out,
            "END|{}|{}|{}|{}|CANONICAL_PARTIAL|SI_NOT_RUN",
            result.segments.len(),
            result.skipped.saturating_add(pad_data.skipped as u32),
            pad_data.pads.len(),
            result.arcs.len()
        )
    } else if mode == "BOARDSI_P2A_PARTIAL_V1" {
        writeln!(
            out,
            "END|{}|{}|CANONICAL_PARTIAL|SI_NOT_RUN",
            result.segments.len(),
            result.skipped
        )
    } else {
        writeln!(
            out,
            "END|{}|{}|{}|CANONICAL_PARTIAL|SI_NOT_RUN",
            result.segments.len(),
            result.skipped.saturating_add(pad_data.skipped as u32),
            pad_data.pads.len()
        )
    }
    .map_err(|_| "FORMAT_ERROR")?;
    if out.len() > 2 * 1024 * 1024 {
        return Err("OUTPUT_LIMIT");
    }
    io::stdout().write_all(&out).map_err(|_| "STDOUT_FAILED")?;
    Ok(())
}
fn main() {
    if let Err(err) = run() {
        eprintln!("P2A_IMPORT_ERROR:{err}");
        std::process::exit(3);
    }
}
