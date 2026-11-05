//! Offline local PADS ASC (V9.4 MILS only) -> partial Canonical Model -> Qt line protocol.
//! Standard output is a bounded transport, never an SI simulation or full CAD import.
use boardsi_import::part_preview::{self, MAX_PADS};
use boardsi_import::route_preview::{into_canonical, parse, MAX_FILE_BYTES};
use boardsi_import::{copper_preview, outline_preview, pour_preview};
use std::{
    env, fs,
    io::{self, Write},
};
// 0.01 mil canonical tick = 254 nm. All Qt wire coordinates remain nm.
const NM: i64 = 254;
#[path = "boardsi-d007-via-trace-three-area.rs"]
#[allow(dead_code)]
mod d007;
#[path = "boardsi-d008-dual-layer-via-area.rs"]
#[allow(dead_code)]
mod d008;
#[path = "boardsi-d009-dual-layer-via-trace.rs"]
#[allow(dead_code)]
mod d009;
#[path = "boardsi-d011-source-contact-graph.rs"]
#[allow(dead_code)]
mod d011;
#[path = "../d012_span_witness.rs"]
#[allow(dead_code)]
mod d012_core;
#[path = "../d013_barrel_witness.rs"]
#[allow(dead_code)]
mod d013_core;
#[path = "../d014_two_via_chain.rs"]
#[allow(dead_code)]
mod d014_core;
#[path = "../d015_t_junction.rs"]
#[allow(dead_code)]
mod d015_core;
#[path = "../d016_x_junction.rs"]
#[allow(dead_code)]
mod d016_core;
#[path = "../d036_dual_mirrored_via.rs"]
#[allow(dead_code)]
mod d036_core;
#[path = "../d037_mixed_sign_dual_via.rs"]
#[allow(dead_code)]
mod d037_core;
#[path = "../d038_mixed_sign_dual_via.rs"]
#[allow(dead_code)]
mod d038_core;
fn run() -> Result<(usize, usize), &'static str> {
    let args: Vec<_> = env::args_os().collect();
    // The suffix protocol is opt-in. Existing one-path invocations keep
    // byte-identical V1-V14 stdout and their existing bounded stderr audit.
    let contacts_v1 = args.len() == 3 && args[2] == "--contact-records-v1";
    if args.len() != 2 && !contacts_v1 {
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
    let pours = pour_preview::parse(&bytes, result.max_layer)?;
    let coppers = copper_preview::parse(&bytes, result.max_layer)?;
    let mut board = into_canonical(&bytes, &result)?;
    let pad_data = part_preview::parse(&bytes, result.max_layer);
    part_preview::augment_source_bound(&mut board, &pad_data, &bytes)?;
    outline_preview::augment(&mut board, &outlines)?;
    copper_preview::augment(&mut board, &coppers)?;
    boardsi_import::copper_contact::augment_source_bound_multi(&mut board, &coppers, &bytes)?;
    // B_006: local source-bound Via/COPCLS evidence, never physical P009 or SI.
    boardsi_import::via_copper_contact::augment_multi(&mut board, &bytes)?;
    if board.arcs.len() != result.arcs.len()
        || board.traces.len() != result.segments.len()
        || board.vias.len() != result.vias.len()
        || board.pads.len() != pad_data.pads.len()
        || board.outlines.len() != usize::from(!outlines.is_empty())
        || board.areas.len() != coppers.len()
        || board.validate().has_errors()
    {
        return Err("CANONICAL_VALIDATION_FAILED");
    }
    // A_032 auxiliary diagnostic channel: stdout V1-V14 artwork wire remains
    // byte-for-byte compatible. This small, bounded stderr *success* record
    // reports only local, source-grid exact contact evidence. The GUI must
    // never interpret it as a complete electrical connectivity result.
    let (local_contacts, via_barrels) = boardsi_import::contact_wire::legacy_v1_counts(&board);
    let mut out = Vec::new();
    // Preserve the established P2A/P2B V1 wire contract whenever no pad
    // records are actually emitted. Rejected PART rows are not P2C geometry.
    let mixed = outlines.iter().any(|s| s.segments.is_some());
    let curved_count = coppers
        .iter()
        .filter(|c| matches!(c.geometry, copper_preview::CopperGeometry::Curved { .. }))
        .count();
    let curved_holes = coppers
        .iter()
        .filter(|c| {
            matches!(c.geometry, copper_preview::CopperGeometry::Curved { .. })
                && !c.cutouts.is_empty()
        })
        .count();
    if curved_count > 0
        && coppers.iter().any(|c| {
            (!c.cutouts.is_empty()
                && !matches!(c.geometry, copper_preview::CopperGeometry::Curved { .. }))
                || c.cutouts
                    .iter()
                    .any(|h| matches!(h, copper_preview::CopperGeometry::Circle { .. }))
        })
    {
        // V13 cannot carry any H record and V14 is narrowly scoped to a
        // curved owner's own single convex COPCUT. Reject independent holes
        // instead of emitting a mixed-version wire Qt cannot decode.
        return Err("UNSUPPORTED_CURVED_COMBINED_COPPER");
    }
    let mode = if curved_holes > 0 {
        "BOARDSI_P2U_PARTIAL_V14"
    } else if curved_count > 0 {
        "BOARDSI_P2T_PARTIAL_V13"
    } else if coppers.iter().any(|c| {
        c.cutouts
            .iter()
            .any(|h| matches!(h, copper_preview::CopperGeometry::Circle { .. }))
    }) {
        "BOARDSI_P2Q_PARTIAL_V12"
    } else if coppers.iter().any(|c| !c.cutouts.is_empty()) {
        "BOARDSI_P2N_PARTIAL_V11"
    } else if !coppers.is_empty() {
        "BOARDSI_P2M_PARTIAL_V10"
    } else if !pours.is_empty() {
        "BOARDSI_P2L_PARTIAL_V9"
    } else if mixed {
        "BOARDSI_P2I_PARTIAL_V8"
    } else if outlines.iter().any(|s| s.circle.is_some()) {
        "BOARDSI_P2H_PARTIAL_V7"
    } else if !outlines.is_empty() {
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
                || mode == "BOARDSI_P2H_PARTIAL_V7"
                || mode == "BOARDSI_P2I_PARTIAL_V8"
                || mode == "BOARDSI_P2L_PARTIAL_V9"
                || mode == "BOARDSI_P2M_PARTIAL_V10"
                || (mode == "BOARDSI_P2N_PARTIAL_V11"
                    || (mode == "BOARDSI_P2Q_PARTIAL_V12"
                        || (mode == "BOARDSI_P2T_PARTIAL_V13"
                            || mode == "BOARDSI_P2U_PARTIAL_V14")))
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
    if mode == "BOARDSI_P2G_PARTIAL_V6"
        || mode == "BOARDSI_P2H_PARTIAL_V7"
        || mode == "BOARDSI_P2I_PARTIAL_V8"
        || mode == "BOARDSI_P2L_PARTIAL_V9"
        || mode == "BOARDSI_P2M_PARTIAL_V10"
        || (mode == "BOARDSI_P2N_PARTIAL_V11"
            || (mode == "BOARDSI_P2Q_PARTIAL_V12"
                || (mode == "BOARDSI_P2T_PARTIAL_V13" || mode == "BOARDSI_P2U_PARTIAL_V14")))
    {
        // Non-electrical BOARD geometry. A circle is analytical (C), while
        // straight polygon segments retain the unchanged O record.
        for shape in &outlines {
            if let Some(circle) = shape.circle {
                writeln!(
                    out,
                    "C|{}|{}|{}|{}|{}",
                    if shape.cutout { "CUT" } else { "OUTER" },
                    shape.name,
                    outline_preview::wire_nm(circle.centre[0])?,
                    outline_preview::wire_nm(circle.centre[1])?,
                    outline_preview::wire_nm(circle.radius)?,
                )
                .map_err(|_| "FORMAT_ERROR")?;
            }
            if let Some(segments) = &shape.segments {
                for (n, segment) in segments.iter().enumerate() {
                    use boardsi_import::outline_topology::Segment;
                    let kind = if shape.cutout { "CUT" } else { "OUTER" };
                    match segment {
                        Segment::Line(a, b) => writeln!(
                            out,
                            "B|{}|{}|{}|L|{}|{}|{}|{}",
                            kind,
                            shape.name,
                            n,
                            outline_preview::wire_nm(a[0])?,
                            outline_preview::wire_nm(a[1])?,
                            outline_preview::wire_nm(b[0])?,
                            outline_preview::wire_nm(b[1])?
                        )
                        .map_err(|_| "FORMAT_ERROR")?,
                        Segment::Arc(a) => writeln!(
                            out,
                            "B|{}|{}|{}|A|{}|{}|{}|{}|{}|{}|{}",
                            kind,
                            shape.name,
                            n,
                            outline_preview::wire_nm(a.start[0])?,
                            outline_preview::wire_nm(a.start[1])?,
                            outline_preview::wire_nm(a.centre[0])?,
                            outline_preview::wire_nm(a.centre[1])?,
                            outline_preview::wire_nm(a.end[0])?,
                            outline_preview::wire_nm(a.end[1])?,
                            if a.clockwise { "CW" } else { "CCW" }
                        )
                        .map_err(|_| "FORMAT_ERROR")?,
                    }
                }
                continue;
            }
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
        if mode == "BOARDSI_P2L_PARTIAL_V9"
            || mode == "BOARDSI_P2M_PARTIAL_V10"
            || (mode == "BOARDSI_P2N_PARTIAL_V11"
                || (mode == "BOARDSI_P2Q_PARTIAL_V12"
                    || (mode == "BOARDSI_P2T_PARTIAL_V13" || mode == "BOARDSI_P2U_PARTIAL_V14")))
        {
            for pour in &pours {
                for (sequence, pair) in pour.vertices.windows(2).enumerate() {
                    // Q is a source-authored UNFILLED boundary, not copper fill.
                    writeln!(
                        out,
                        "Q|{}|{}|{}|{}|{}|{}|{}|{}|{}",
                        pour.layer,
                        pour.name,
                        pour.net_label.as_deref().unwrap_or("-"),
                        sequence,
                        outline_preview::wire_nm(pair[0][0])?,
                        outline_preview::wire_nm(pair[0][1])?,
                        outline_preview::wire_nm(pair[1][0])?,
                        outline_preview::wire_nm(pair[1][1])?,
                        pour.source_line
                    )
                    .map_err(|_| "FORMAT_ERROR")?;
                }
            }
            // V10 emits only explicit source-filled *LINES* COPPER geometry.
            // G record is a non-electrical preview shape, never an invented flood.
            if mode == "BOARDSI_P2M_PARTIAL_V10"
                || (mode == "BOARDSI_P2N_PARTIAL_V11"
                    || (mode == "BOARDSI_P2Q_PARTIAL_V12"
                        || (mode == "BOARDSI_P2T_PARTIAL_V13"
                            || mode == "BOARDSI_P2U_PARTIAL_V14")))
            {
                for copper in &coppers {
                    use copper_preview::CopperGeometry;
                    match &copper.geometry {
                        CopperGeometry::Polygon(vertices) => {
                            write!(
                                out,
                                "G|P|{}|{}|{}|{}|{}",
                                copper.layer,
                                copper.name,
                                copper.net_label.as_deref().unwrap_or("-"),
                                copper.source_line,
                                vertices.len()
                            )
                            .map_err(|_| "FORMAT_ERROR")?;
                            for v in vertices {
                                write!(
                                    out,
                                    "|{}|{}",
                                    outline_preview::wire_nm(v[0])?,
                                    outline_preview::wire_nm(v[1])?
                                )
                                .map_err(|_| "FORMAT_ERROR")?;
                            }
                            writeln!(out).map_err(|_| "FORMAT_ERROR")?;
                        }
                        CopperGeometry::Curved { segments, .. } => {
                            write!(
                                out,
                                "G|R|{}|{}|{}|{}|{}",
                                copper.layer,
                                copper.name,
                                copper.net_label.as_deref().unwrap_or("-"),
                                copper.source_line,
                                segments.len()
                            )
                            .map_err(|_| "FORMAT_ERROR")?;
                            for segment in segments {
                                use boardsi_import::outline_topology::Segment;
                                match segment {
                                    Segment::Line(start, end) => {
                                        write!(
                                            out,
                                            "|L|{}|{}|0|0|{}|{}|-",
                                            outline_preview::wire_nm(start[0])?,
                                            outline_preview::wire_nm(start[1])?,
                                            outline_preview::wire_nm(end[0])?,
                                            outline_preview::wire_nm(end[1])?
                                        )
                                        .map_err(|_| "FORMAT_ERROR")?;
                                    }
                                    Segment::Arc(arc) => {
                                        write!(
                                            out,
                                            "|A|{}|{}|{}|{}|{}|{}|{}",
                                            outline_preview::wire_nm(arc.start[0])?,
                                            outline_preview::wire_nm(arc.start[1])?,
                                            outline_preview::wire_nm(arc.centre[0])?,
                                            outline_preview::wire_nm(arc.centre[1])?,
                                            outline_preview::wire_nm(arc.end[0])?,
                                            outline_preview::wire_nm(arc.end[1])?,
                                            if arc.clockwise { "CW" } else { "CCW" }
                                        )
                                        .map_err(|_| "FORMAT_ERROR")?;
                                    }
                                }
                            }
                            writeln!(out).map_err(|_| "FORMAT_ERROR")?;
                        }
                        CopperGeometry::Circle { centre, radius } => {
                            writeln!(
                                out,
                                "G|C|{}|{}|{}|{}|{}|{}|{}",
                                copper.layer,
                                copper.name,
                                copper.net_label.as_deref().unwrap_or("-"),
                                copper.source_line,
                                outline_preview::wire_nm(centre[0])?,
                                outline_preview::wire_nm(centre[1])?,
                                outline_preview::wire_nm(*radius)?
                            )
                            .map_err(|_| "FORMAT_ERROR")?;
                        }
                    }
                }
                if mode == "BOARDSI_P2N_PARTIAL_V11"
                    || (mode == "BOARDSI_P2Q_PARTIAL_V12"
                        || (mode == "BOARDSI_P2T_PARTIAL_V13" || mode == "BOARDSI_P2U_PARTIAL_V14"))
                {
                    // H references a G index; the source alone defines ownership.
                    for (index, copper) in coppers.iter().enumerate() {
                        for cutout in &copper.cutouts {
                            match cutout {
                                copper_preview::CopperGeometry::Curved { .. } => {
                                    return Err("UNSUPPORTED_CURVED_COPPER_CUTOUT")
                                }
                                copper_preview::CopperGeometry::Polygon(vertices) => {
                                    write!(out, "H|{}|{}", index, vertices.len())
                                        .map_err(|_| "FORMAT_ERROR")?;
                                    for v in vertices {
                                        write!(
                                            out,
                                            "|{}|{}",
                                            outline_preview::wire_nm(v[0])?,
                                            outline_preview::wire_nm(v[1])?
                                        )
                                        .map_err(|_| "FORMAT_ERROR")?;
                                    }
                                    writeln!(out).map_err(|_| "FORMAT_ERROR")?;
                                }
                                copper_preview::CopperGeometry::Circle { centre, radius } => {
                                    if mode != "BOARDSI_P2Q_PARTIAL_V12" {
                                        return Err("UNSUPPORTED_COPPER_CUTOUT_PIECE");
                                    }
                                    writeln!(
                                        out,
                                        "J|{}|{}|{}|{}",
                                        index,
                                        outline_preview::wire_nm(centre[0])?,
                                        outline_preview::wire_nm(centre[1])?,
                                        outline_preview::wire_nm(*radius)?
                                    )
                                    .map_err(|_| "FORMAT_ERROR")?;
                                }
                            }
                        }
                    }
                }
            }
            let mixed_arcs: usize = outlines
                .iter()
                .filter_map(|s| s.segments.as_ref())
                .map(|v| {
                    v.iter()
                        .filter(|seg| {
                            matches!(seg, boardsi_import::outline_topology::Segment::Arc(_))
                        })
                        .count()
                })
                .sum();
            if mode == "BOARDSI_P2M_PARTIAL_V10"
                || (mode == "BOARDSI_P2N_PARTIAL_V11"
                    || (mode == "BOARDSI_P2Q_PARTIAL_V12"
                        || (mode == "BOARDSI_P2T_PARTIAL_V13"
                            || mode == "BOARDSI_P2U_PARTIAL_V14")))
            {
                if mode == "BOARDSI_P2U_PARTIAL_V14" {
                    writeln!(
                        out,
                        "END|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|CANONICAL_PARTIAL|SI_NOT_RUN",
                        result.segments.len(),
                        result.skipped.saturating_add(pad_data.skipped as u32),
                        pad_data.pads.len(),
                        result.arcs.len(),
                        outlines.iter().filter(|s| !s.cutout).count(),
                        outlines.iter().filter(|s| s.cutout).count(),
                        outlines.iter().filter(|s| s.circle.is_some()).count(),
                        mixed_arcs,
                        pours.len(),
                        coppers.len(),
                        curved_count,
                        coppers.iter().map(|c| c.cutouts.len()).sum::<usize>(),
                    )
                    .map_err(|_| "FORMAT_ERROR")?;
                } else if mode == "BOARDSI_P2T_PARTIAL_V13" {
                    writeln!(
                        out,
                        "END|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|CANONICAL_PARTIAL|SI_NOT_RUN",
                        result.segments.len(),
                        result.skipped.saturating_add(pad_data.skipped as u32),
                        pad_data.pads.len(),
                        result.arcs.len(),
                        outlines.iter().filter(|s| !s.cutout).count(),
                        outlines.iter().filter(|s| s.cutout).count(),
                        outlines.iter().filter(|s| s.circle.is_some()).count(),
                        mixed_arcs,
                        pours.len(),
                        coppers.len(),
                        curved_count,
                    )
                    .map_err(|_| "FORMAT_ERROR")?;
                } else if mode == "BOARDSI_P2N_PARTIAL_V11" || mode == "BOARDSI_P2Q_PARTIAL_V12" {
                    writeln!(
                        out,
                        "END|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|CANONICAL_PARTIAL|SI_NOT_RUN",
                        result.segments.len(),
                        result.skipped.saturating_add(pad_data.skipped as u32),
                        pad_data.pads.len(),
                        result.arcs.len(),
                        outlines.iter().filter(|s| !s.cutout).count(),
                        outlines.iter().filter(|s| s.cutout).count(),
                        outlines.iter().filter(|s| s.circle.is_some()).count(),
                        mixed_arcs,
                        pours.len(),
                        coppers.len(),
                        coppers.iter().map(|c| c.cutouts.len()).sum::<usize>()
                    )
                    .map_err(|_| "FORMAT_ERROR")?;
                } else {
                    writeln!(
                        out,
                        "END|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|CANONICAL_PARTIAL|SI_NOT_RUN",
                        result.segments.len(),
                        result.skipped.saturating_add(pad_data.skipped as u32),
                        pad_data.pads.len(),
                        result.arcs.len(),
                        outlines.iter().filter(|s| !s.cutout).count(),
                        outlines.iter().filter(|s| s.cutout).count(),
                        outlines.iter().filter(|s| s.circle.is_some()).count(),
                        mixed_arcs,
                        pours.len(),
                        coppers.len()
                    )
                    .map_err(|_| "FORMAT_ERROR")?;
                }
                // V10-V14 intentionally return early from their legacy
                // END serialization. The optional evidence suffix MUST be
                // emitted on this path as well, before the first stdout byte.
                if contacts_v1 {
                    let evidence = boardsi_import::contact_wire::encode(&board)?;
                    out.extend_from_slice(&evidence);
                }
                // All END records are emitted by their own protocol version.
                // Avoid emitting V9 END a second time.
                if out.len() > 2 * 1024 * 1024 {
                    return Err("OUTPUT_LIMIT");
                }
                io::stdout().write_all(&out).map_err(|_| "STDOUT_FAILED")?;
                return Ok((local_contacts, via_barrels));
            }
            writeln!(
                out,
                "END|{}|{}|{}|{}|{}|{}|{}|{}|{}|CANONICAL_PARTIAL|SI_NOT_RUN",
                result.segments.len(),
                result.skipped.saturating_add(pad_data.skipped as u32),
                pad_data.pads.len(),
                result.arcs.len(),
                outlines.iter().filter(|s| !s.cutout).count(),
                outlines.iter().filter(|s| s.cutout).count(),
                outlines.iter().filter(|s| s.circle.is_some()).count(),
                mixed_arcs,
                pours.len()
            )
        } else if mode == "BOARDSI_P2I_PARTIAL_V8" {
            let mixed_arcs: usize = outlines
                .iter()
                .filter_map(|s| s.segments.as_ref())
                .map(|v| {
                    v.iter()
                        .filter(|seg| {
                            matches!(seg, boardsi_import::outline_topology::Segment::Arc(_))
                        })
                        .count()
                })
                .sum();
            writeln!(
                out,
                "END|{}|{}|{}|{}|{}|{}|{}|{}|CANONICAL_PARTIAL|SI_NOT_RUN",
                result.segments.len(),
                result.skipped.saturating_add(pad_data.skipped as u32),
                pad_data.pads.len(),
                result.arcs.len(),
                outlines.iter().filter(|s| !s.cutout).count(),
                outlines.iter().filter(|s| s.cutout).count(),
                outlines.iter().filter(|s| s.circle.is_some()).count(),
                mixed_arcs
            )
        } else if mode == "BOARDSI_P2H_PARTIAL_V7" {
            writeln!(
                out,
                "END|{}|{}|{}|{}|{}|{}|{}|CANONICAL_PARTIAL|SI_NOT_RUN",
                result.segments.len(),
                result.skipped.saturating_add(pad_data.skipped as u32),
                pad_data.pads.len(),
                result.arcs.len(),
                outlines.iter().filter(|s| !s.cutout).count(),
                outlines.iter().filter(|s| s.cutout).count(),
                outlines.iter().filter(|s| s.circle.is_some()).count(),
            )
        } else {
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
        }
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
    if contacts_v1 {
        // Validate and format the complete suffix before any stdout write.
        // Never return a successful partial artwork if this gate fails.
        let evidence = boardsi_import::contact_wire::encode(&board)?;
        out.extend_from_slice(&evidence);
    }
    if out.len() > 2 * 1024 * 1024 {
        return Err("OUTPUT_LIMIT");
    }
    io::stdout().write_all(&out).map_err(|_| "STDOUT_FAILED")?;
    Ok((local_contacts, via_barrels))
}
fn main() {
    // Preserve the ordinary importer contract on Unix paths containing
    // non-Unicode bytes: only textual audit parameters require UTF-8.
    let args: Vec<_> = std::env::args_os().collect();
    // D_031 steep 3:4 non-45 oblique Via-A Source endpoint contact; never fabricated electrical proof.
    if args.len() == 8
        && args[2] == std::ffi::OsStr::new("--d031-non45-steep-via-a-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D031_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D031_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D031_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D031_READ")?;
            d015_core::encode_non45_steep_via_a_endpoint_contact(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D031_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D031_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D031_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D031_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D031_NON45_STEEP_VIA_B_ENDPOINT_CONTACT_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_033 steep 3:4 non-45 oblique Via-A Source endpoint contact; never fabricated electrical proof.
    if args.len() == 8
        && args[2] == std::ffi::OsStr::new("--d033-non45-steep-mirrored-via-a-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D033_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D033_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D033_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D033_READ")?;
            d015_core::encode_non45_steep_mirrored_via_a_endpoint_contact(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D033_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D033_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D033_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D033_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D033_NON45_STEEP_VIA_A_ENDPOINT_CONTACT_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_032 steep 3:4 non-45 oblique Via-B Source endpoint contact; never fabricated electrical proof.
    if args.len() == 8
        && args[2] == std::ffi::OsStr::new("--d032-non45-steep-mirrored-via-b-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D032_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D032_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D032_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D032_READ")?;
            d015_core::encode_non45_steep_mirrored_via_b_endpoint_contact(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D032_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D032_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D032_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D032_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D032_NON45_STEEP_VIA_B_ENDPOINT_CONTACT_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_030 steep 3:4 non-45 oblique Via-B Source endpoint contact; never fabricated electrical proof.
    if args.len() == 8
        && args[2] == std::ffi::OsStr::new("--d030-non45-steep-via-b-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D030_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D030_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D030_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D030_READ")?;
            d015_core::encode_non45_steep_via_b_endpoint_contact(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D030_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D030_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D030_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D030_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D030_NON45_STEEP_VIA_B_ENDPOINT_CONTACT_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_029 non-45 oblique Via-A Source endpoint contact; never fabricated electrical proof.
    if args.len() == 8
        && args[2] == std::ffi::OsStr::new("--d029-non45-oblique-via-a-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D029_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D029_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D029_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D029_READ")?;
            d015_core::encode_non45_oblique_via_a_endpoint_contact(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D029_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D029_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D029_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D029_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D029_NON45_OBLIQUE_VIA_A_ENDPOINT_CONTACT_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_036: combined Via-A/Via-B exact-negative 4:3 SOURCE candidates.
    if args.len() == 8
        && args[2] == std::ffi::OsStr::new("--d036-non45-dual-mirrored-via-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D036_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D036_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D036_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D036_READ")?;
            d036_core::encode(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D036_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D036_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D036_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D036_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D036_DUAL_MIRRORED_VIA_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_037: mixed-sign negative-A positive-B Via-A/Via-B 4:3 SOURCE candidates.
    if args.len() == 8
        && args[2] == std::ffi::OsStr::new("--d037-non45-mixed-sign-dual-via-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D037_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D037_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D037_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D037_READ")?;
            d037_core::encode(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D037_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D037_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D037_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D037_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D037_MIXED_SIGN_DUAL_VIA_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_042: source-owned exact redundant detour / closed loop, no global electrical PASS.
    if args.len() == 3 && args[2] == std::ffi::OsStr::new("--d042-local-closed-loop-v1") {
        let result = (|| -> Result<String, &'static str> {
            let path = std::path::Path::new(&args[1]);
            let metadata = fs::metadata(path).map_err(|_| "D042_FILE")?;
            if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES as u64 {
                return Err("D042_FILE_LIMIT");
            }
            let source = fs::read(path).map_err(|_| "D042_READ")?;
            boardsi_import::d042_local_closed_loop::encode(&source)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D042_LOCAL_CLOSED_LOOP_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_041: one explicit Source-owned bridge joining two local components.
    // The receipt is bounded geometry only, NOT global Electrical PASS.
    if args.len() == 3 && args[2] == std::ffi::OsStr::new("--d041-local-bridge-v1") {
        let result = (|| -> Result<String, &'static str> {
            let path = std::path::Path::new(&args[1]);
            let metadata = fs::metadata(path).map_err(|_| "D041_FILE")?;
            if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES as u64 {
                return Err("D041_FILE_LIMIT");
            }
            let source = fs::read(path).map_err(|_| "D041_READ")?;
            boardsi_import::d041_local_bridge::encode(&source)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D041_LOCAL_BRIDGE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_040: exact two local copper components, not global Electrical Open/Short.
    if args.len() == 3 && args[2] == std::ffi::OsStr::new("--d040-multi-component-v1") {
        let result = (|| -> Result<String, &'static str> {
            let path = std::path::Path::new(&args[1]);
            let metadata = fs::metadata(path).map_err(|_| "D040_FILE")?;
            if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES as u64 {
                return Err("D040_FILE_LIMIT");
            }
            let source = fs::read(path).map_err(|_| "D040_READ")?;
            boardsi_import::d040_multi_component::encode(&source)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D040_MULTI_COMPONENT_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_039: two Source-owned SMD pads and two straight same-layer traces;
    // local copper geometry only, never global electrical or physical SI.
    if args.len() == 3 && args[2] == std::ffi::OsStr::new("--d039-local-copper-v1") {
        let result = (|| -> Result<String, &'static str> {
            let path = std::path::Path::new(&args[1]);
            let metadata = fs::metadata(path).map_err(|_| "D039_FILE")?;
            if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES as u64 {
                return Err("D039_FILE_LIMIT");
            }
            let source = fs::read(path).map_err(|_| "D039_READ")?;
            boardsi_import::d039_local_copper::encode(&source, "NET_A", 1)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D039_LOCAL_COPPER_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_038: mixed-sign positive-A negative-B Via-A/Via-B 4:3 SOURCE candidates.
    if args.len() == 8
        && args[2]
            == std::ffi::OsStr::new("--d038-non45-mixed-sign-reversed-dual-via-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D038_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D038_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D038_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D038_READ")?;
            d038_core::encode(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D038_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D038_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D038_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D038_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D038_MIXED_SIGN_DUAL_VIA_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_035 non-45 oblique Via-A Source endpoint contact; never fabricated electrical proof.
    if args.len() == 8
        && args[2]
            == std::ffi::OsStr::new("--d035-non45-oblique-mirrored-via-a-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D035_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D035_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D035_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D035_READ")?;
            d015_core::encode_non45_oblique_mirrored_via_a_endpoint_contact(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D035_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D035_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D035_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D035_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D035_NON45_OBLIQUE_VIA_A_ENDPOINT_CONTACT_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_034 non-45 oblique Via-B Source endpoint contact; never fabricated electrical proof.
    if args.len() == 8
        && args[2]
            == std::ffi::OsStr::new("--d034-non45-oblique-mirrored-via-b-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D034_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D034_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D034_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D034_READ")?;
            d015_core::encode_non45_oblique_mirrored_via_b_endpoint_contact(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D034_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D034_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D034_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D034_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D034_NON45_OBLIQUE_VIA_B_ENDPOINT_CONTACT_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_028 non-45 oblique Via-B Source endpoint contact; never fabricated electrical proof.
    if args.len() == 8
        && args[2] == std::ffi::OsStr::new("--d028-non45-oblique-via-b-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D028_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D028_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D028_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D028_READ")?;
            d015_core::encode_non45_oblique_via_b_endpoint_contact(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D028_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D028_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D028_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D028_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D028_NON45_OBLIQUE_VIA_B_ENDPOINT_CONTACT_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_027 oblique Via-A Source endpoint contact; never fabricated electrical proof.
    if args.len() == 8
        && args[2] == std::ffi::OsStr::new("--d027-oblique-via-a-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D027_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D027_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D027_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D027_READ")?;
            d015_core::encode_oblique_via_a_endpoint_contact(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D027_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D027_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D027_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D027_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D027_OBLIQUE_VIA_A_ENDPOINT_CONTACT_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_026 oblique Via-B Source endpoint contact; never fabricated electrical proof.
    if args.len() == 8
        && args[2] == std::ffi::OsStr::new("--d026-oblique-via-b-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D026_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D026_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D026_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D026_READ")?;
            d015_core::encode_oblique_via_b_endpoint_contact(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D026_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D026_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D026_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D026_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D026_OBLIQUE_VIA_B_ENDPOINT_CONTACT_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_025 orthogonal Via-A Source endpoint contact; never fabricated electrical proof.
    if args.len() == 8
        && args[2] == std::ffi::OsStr::new("--d025-orthogonal-via-a-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D025_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D025_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D025_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D025_READ")?;
            d015_core::encode_orthogonal_via_a_endpoint_contact(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D025_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D025_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D025_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D025_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D025_ORTHOGONAL_VIA_A_ENDPOINT_CONTACT_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_024 orthogonal Via-B Source endpoint contact; never fabricated electrical proof.
    if args.len() == 8 && args[2] == std::ffi::OsStr::new("--d024-orthogonal-endpoint-contact-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D024_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D024_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D024_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D024_READ")?;
            d015_core::encode_orthogonal_endpoint_contact(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D024_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D024_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D024_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D024_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D024_COLLINEAR_ENDPOINT_CONTACT_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_023 opposite Via-A collinear Source endpoint contact; never fabricated electrical proof.
    if args.len() == 8
        && args[2] == std::ffi::OsStr::new("--d023-collinear-opposite-endpoint-contact-v1")
    {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D023_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D023_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D023_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D023_READ")?;
            d015_core::encode_opposite_endpoint_contact(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D023_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D023_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D023_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D023_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D023_COLLINEAR_ENDPOINT_CONTACT_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_022 proper partial collinear Source overlap; never fabricated electrical proof.
    if args.len() == 8 && args[2] == std::ffi::OsStr::new("--d022-collinear-endpoint-contact-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D022_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D022_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D022_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D022_READ")?;
            d015_core::encode_endpoint_contact(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D022_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D022_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D022_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D022_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D022_COLLINEAR_ENDPOINT_CONTACT_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_021 proper partial collinear Source overlap; never fabricated electrical proof.
    if args.len() == 8 && args[2] == std::ffi::OsStr::new("--d021-partial-collinear-overlap-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D021_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D021_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D021_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D021_READ")?;
            d015_core::encode_partial_overlap(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D021_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D021_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D021_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D021_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D021_PARTIAL_COLLINEAR_OVERLAP_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_020 opt-in Source-only contained collinear overlap, never manufactured electrical proof.
    if args.len() == 8 && args[2] == std::ffi::OsStr::new("--d020-contained-collinear-overlap-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D020_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D020_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D020_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D020_READ")?;
            d015_core::encode_contained_overlap(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D020_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D020_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D020_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D020_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D020_CONTAINED_COLLINEAR_OVERLAP_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_019 opt-in Source-only oblique T, never manufactured electrical proof.
    if args.len() == 8 && args[2] == std::ffi::OsStr::new("--d019-oblique-t-junction-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D019_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D019_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D019_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D019_READ")?;
            d015_core::encode_oblique(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D019_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D019_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D019_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D019_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D019_OBLIQUE_T_JUNCTION_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_017 opt-in Source geometry ONLY, not physical electrical continuity.
    if args.len() == 8 && args[2] == std::ffi::OsStr::new("--d017-oblique-x-junction-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D017_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D017_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D017_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D017_READ")?;
            d016_core::encode_oblique(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D017_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D017_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D017_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D017_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D017_OBLIQUE_X_JUNCTION_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_016 opt-in Source geometry ONLY, not physical electrical continuity.
    if args.len() == 8 && args[2] == std::ffi::OsStr::new("--d016-x-junction-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D016_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D016_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D016_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D016_READ")?;
            d016_core::encode(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D016_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D016_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D016_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D016_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D016_X_JUNCTION_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_015 opt-in Source geometry ONLY, not physical electrical continuity.
    if args.len() == 8 && args[2] == std::ffi::OsStr::new("--d015-t-junction-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D015_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D015_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D015_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D015_READ")?;
            d015_core::encode(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D015_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D015_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D015_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D015_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D015_T_JUNCTION_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_014: independent opt-in two-Via Source-only path, not cross-layer conduction.
    if args.len() == 8 && args[2] == std::ffi::OsStr::new("--d014-two-via-source-chain-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D014_UTF8");
            let path = std::path::Path::new(&args[1]);
            let meta = fs::metadata(path).map_err(|_| "D014_FILE")?;
            if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
                return Err("D014_FILE_LIMIT");
            }
            let bytes = fs::read(path).map_err(|_| "D014_READ")?;
            d014_core::encode(
                &bytes,
                txt(3)?,
                [
                    txt(4)?.parse::<usize>().map_err(|_| "D014_LAYER")?,
                    txt(5)?.parse::<usize>().map_err(|_| "D014_LAYER")?,
                ],
                [
                    txt(6)?.parse::<usize>().map_err(|_| "D014_VIA")?,
                    txt(7)?.parse::<usize>().map_err(|_| "D014_VIA")?,
                ],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D014_TWO_VIA_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_013 opt-in Source-only legacy barrel projection, never electrical proof.
    if args.len() == 11 && args[2] == std::ffi::OsStr::new("--d013-legacy-barrel-provenance-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D013_NON_UTF8_SELECTION");
            let names = [txt(3)?, txt(4)?, txt(5)?, txt(6)?];
            let net = txt(7)?;
            let layers = [
                txt(8)?.parse::<usize>().map_err(|_| "D013_LAYER")?,
                txt(9)?.parse::<usize>().map_err(|_| "D013_LAYER")?,
            ];
            let via = txt(10)?.parse::<usize>().map_err(|_| "D013_VIA")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "D013_FILE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("D013_FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "D013_READ")?;
            let graph = d011::encode(&bytes, names, net, layers, via)?;
            let span = d012_core::encode_from_qualified_graph(&bytes, &graph, net, layers)?;
            d013_core::encode_from_qualified_span(&bytes, &span, layers)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D013_LEGACY_BARREL_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_012: source Via Span provenance witness, no electrical barrel inference.
    if args.len() == 11 && args[2] == std::ffi::OsStr::new("--d012-via-span-provenance-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D012_NON_UTF8_SELECTION");
            let names = [txt(3)?, txt(4)?, txt(5)?, txt(6)?];
            let net = txt(7)?;
            let layers = [
                txt(8)?.parse::<usize>().map_err(|_| "D012_LAYER")?,
                txt(9)?.parse::<usize>().map_err(|_| "D012_LAYER")?,
            ];
            let via = txt(10)?.parse::<usize>().map_err(|_| "D012_VIA")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "D012_FILE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("D012_FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "D012_READ")?;
            let graph = d011::encode(&bytes, names, net, layers, via)?;
            d012_core::encode_from_qualified_graph(&bytes, &graph, net, layers)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D012_SOURCE_GRAPH_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_011: source-only dual-layer graph, 8 nodes / 6 local candidate edges.
    if args.len() == 11 && args[2] == std::ffi::OsStr::new("--d011-source-contact-graph-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D011_NON_UTF8_SELECTION");
            let names = [txt(3)?, txt(4)?, txt(5)?, txt(6)?];
            let net = txt(7)?;
            let layers = [
                txt(8)?.parse::<usize>().map_err(|_| "D011_LAYER")?,
                txt(9)?.parse::<usize>().map_err(|_| "D011_LAYER")?,
            ];
            let via = txt(10)?.parse::<usize>().map_err(|_| "D011_VIA")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "D011_FILE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("D011_FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "D011_READ")?;
            d011::encode(&bytes, names, net, layers, via)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D011_SOURCE_GRAPH_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_010: source-only two-layer, two-trace, four-source-area Via candidate.
    if args.len() == 11 && args[2] == std::ffi::OsStr::new("--d010-dual-layer-two-trace-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D010_NON_UTF8_SELECTION");
            let names = [txt(3)?, txt(4)?, txt(5)?, txt(6)?];
            let net = txt(7)?;
            let layers = [
                txt(8)?.parse::<usize>().map_err(|_| "D010_LAYER_INVALID")?,
                txt(9)?.parse::<usize>().map_err(|_| "D010_LAYER_INVALID")?,
            ];
            let index = txt(10)?.parse::<usize>().map_err(|_| "D010_VIA_INVALID")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "D010_FILE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("D010_FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "D010_READ")?;
            d011::d010::encode(&bytes, names, net, layers, index)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D010_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_009: source geometry only, one Via Pad on two layers and one Layer B Trace to C.
    if args.len() == 10 && args[2] == std::ffi::OsStr::new("--d009-dual-layer-via-trace-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D009_NON_UTF8_SELECTION");
            let names = [txt(3)?, txt(4)?, txt(5)?];
            let net = txt(6)?;
            let layers = [
                txt(7)?.parse::<usize>().map_err(|_| "D009_LAYER_INVALID")?,
                txt(8)?.parse::<usize>().map_err(|_| "D009_LAYER_INVALID")?,
            ];
            let index = txt(9)?.parse::<usize>().map_err(|_| "D009_VIA_INVALID")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "D009_FILE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("D009_FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            d009::encode(&bytes, names, net, layers, index)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D009_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_008: two separately verified Source Via pad/COPCLS layer contacts.
    if args.len() == 9 && args[2] == std::ffi::OsStr::new("--d008-dual-layer-via-area-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D008_NON_UTF8_SELECTION");
            let names = [txt(3)?, txt(4)?];
            let net = txt(5)?;
            let layers = [
                txt(6)?.parse::<usize>().map_err(|_| "D008_LAYER_INVALID")?,
                txt(7)?.parse::<usize>().map_err(|_| "D008_LAYER_INVALID")?,
            ];
            let index = txt(8)?.parse::<usize>().map_err(|_| "D008_VIA_INVALID")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("D008_FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            d008::encode(&bytes, names, net, layers, index)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D008_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_007: strict opt-in plated Via Pad -> one Trace -> different COPCLS area.
    if args.len() == 9 && args[2] == std::ffi::OsStr::new("--d007-via-trace-three-area-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D007_NON_UTF8_SELECTION");
            let names = [txt(3)?, txt(4)?, txt(5)?];
            let net = txt(6)?;
            let layer = txt(7)?.parse::<usize>().map_err(|_| "D007_LAYER_INVALID")?;
            let index = txt(8)?.parse::<usize>().map_err(|_| "D007_VIA_INVALID")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("D007_FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            d007::encode(&bytes, names, net, layer, index)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D007_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_005: source-only Pad -> one straight Trace -> bounded COPCLS component.
    // Not a physical or electrical connectivity, return-path, or solver gate.
    if args.len() == 9 && args[2] == std::ffi::OsStr::new("--d005-pad-trace-area-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D005_NON_UTF8_SELECTION");
            let names = [txt(3)?, txt(4)?, txt(5)?];
            let net = txt(6)?;
            let layer = txt(7)?.parse::<usize>().map_err(|_| "D005_LAYER_INVALID")?;
            let index = txt(8)?.parse::<usize>().map_err(|_| "D005_TRACE_INVALID")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("D005_FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::d005_pad_trace_area::encode(&bytes, names, net, layer, index)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D005_SOURCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_004: one straight Trace source endpoint to a D_003 geometry component.
    // No electrical/manufactured continuity or solver promotion.
    if args.len() == 9 && args[2] == std::ffi::OsStr::new("--d004-endpoint-attachment-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D004_NON_UTF8_SELECTION");
            let names = [txt(3)?, txt(4)?, txt(5)?];
            let net = txt(6)?;
            let layer = txt(7)?.parse::<usize>().map_err(|_| "D004_LAYER_INVALID")?;
            let index = txt(8)?.parse::<usize>().map_err(|_| "D004_TRACE_INVALID")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("D004_FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::d004_endpoint_attachment::encode(&bytes, names, net, layer, index)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D004_ENDPOINT_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_003: exactly three source-owned rectangles with at most one COPCUT.
    // WARNING: source geometry candidate only, no electrical / solver promotion.
    if args.len() == 8 && args[2] == std::ffi::OsStr::new("--d003-three-area-copcut-v1") {
        let result = (|| -> Result<String, &'static str> {
            let txt = |i: usize| args[i].to_str().ok_or("D003_NON_UTF8_SELECTION");
            let names = [txt(3)?, txt(4)?, txt(5)?];
            let net = txt(6)?;
            let layer = txt(7)?.parse::<usize>().map_err(|_| "D003_LAYER_INVALID")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("D003_FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::d003_three_area_copcut::encode(&bytes, names, net, layer)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D003_THREE_AREA_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_002: two source COPCLS rectangles with at most one same-owner COPCUT.
    // This opt-in diagnostic never modifies ordinary artwork stdout or graph.
    if args.len() == 7 && args[2] == std::ffi::OsStr::new("--d002-copcut-contact-v1") {
        let result = (|| -> Result<String, &'static str> {
            let text = |at: usize| args[at].to_str().ok_or("D002_NON_UTF8_SELECTION");
            let a = text(3)?;
            let b = text(4)?;
            let net = text(5)?;
            let layer = text(6)?
                .parse::<usize>()
                .map_err(|_| "D002_LAYER_INVALID")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("D002_FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::d002_copcut_contact::encode(&bytes, [a, b], net, layer)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D002_COPCUT_CONTACT_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // D_001: separate bounded pad-endpoint witness; no changes to default artwork wire.
    if args.len() == 3 && args[2] == std::ffi::OsStr::new("--d001-pad-endpoint-v1") {
        let result = (|| -> Result<String, &'static str> {
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("D001_FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::d001_pad_endpoint::encode(&bytes)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("D001_PAD_ENDPOINT_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // C_023: strictly opt-in source geometry corridor across V14-qualified areas.
    if args.len() == 7 && args[2] == std::ffi::OsStr::new("--reference-corridor-v15") {
        let result = (|| -> Result<String, &'static str> {
            let text = |at: usize| args[at].to_str().ok_or("C023_NON_UTF8_SELECTION");
            let raw = text(3)?;
            if raw.is_empty() || raw.len() > 390 {
                return Err("C023_AREA_SELECTION_LIMIT");
            }
            let names: Vec<&str> = raw.split(',').collect();
            let net = text(4)?;
            let layer = text(5)?.parse::<usize>().map_err(|_| "C023_LAYER")?;
            let trace = text(6)?.parse::<usize>().map_err(|_| "C023_TRACE")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::reference_corridor_v15::encode(&bytes, &names, net, layer, trace)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("C023_REFERENCE_CORRIDOR_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // C_022: explicit 3..=6 Source areas, no electrical graph promotion.
    if args.len() == 6 && args[2] == std::ffi::OsStr::new("--reference-area-v14") {
        let result = (|| -> Result<String, &'static str> {
            let text = |at: usize| args[at].to_str().ok_or("C022_NON_UTF8_SELECTION");
            let raw = text(3)?;
            if raw.is_empty() || raw.len() > 390 {
                return Err("C022_AREA_SELECTION_LIMIT");
            }
            let names: Vec<&str> = raw.split(',').collect();
            let net = text(4)?;
            let layer = text(5)?
                .parse::<usize>()
                .map_err(|_| "C022_LAYER_INVALID")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::reference_area_v14::encode(&bytes, &names, net, layer)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("C022_REFERENCE_AREA_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // C_021: strictly opt-in, source-only two COPCLS rectangle connection audit.
    // No changes to V1–V12, Canonical connectivity or ordinary PADS stdout.
    if args.len() == 7 && args[2] == std::ffi::OsStr::new("--reference-area-v13") {
        let result = (|| -> Result<String, &'static str> {
            let text = |at: usize| args[at].to_str().ok_or("C021_NON_UTF8_SELECTION");
            let a = text(3)?;
            let b = text(4)?;
            let net = text(5)?;
            let layer = text(6)?
                .parse::<usize>()
                .map_err(|_| "C021_LAYER_INVALID")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::reference_area_v13::encode(&bytes, [a, b], net, layer, None)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("C021_REFERENCE_AREA_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // C_014: bounded multiple Source COPCUT rectangles; preserve V9 or V8.
    if args.len() == 10
        && (args[2] == std::ffi::OsStr::new("--reference-route-v10")
            || args[2] == std::ffi::OsStr::new("--reference-route-v11")
            || args[2] == std::ffi::OsStr::new("--reference-route-v12"))
    {
        let result = (|| -> Result<String, &'static str> {
            let raw = args[3].to_str().ok_or("INVALID_ROUTE_INDICES")?;
            if raw.len() > 95 || raw.is_empty() {
                return Err("INVALID_ROUTE_INDICES");
            }
            let indices: Vec<usize> = raw
                .split(',')
                .map(|part| part.parse::<usize>().map_err(|_| "INVALID_ROUTE_INDEX"))
                .collect::<Result<_, _>>()?;
            let parse_usize = |index: usize| {
                args[index]
                    .to_str()
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or("INVALID_V10_INTEGER")
            };
            let split = parse_usize(4)?;
            let ref_a = args[5].to_str().ok_or("INVALID_REFERENCE_A")?;
            let ref_b = args[7].to_str().ok_or("INVALID_REFERENCE_B")?;
            let layers = [parse_usize(6)?, parse_usize(8)?];
            let candidate = parse_usize(9)?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            if args[2] == std::ffi::OsStr::new("--reference-route-v12") {
                boardsi_import::reference_route_v12::encode(
                    &bytes,
                    &indices,
                    split,
                    [ref_a, ref_b],
                    layers,
                    candidate,
                )
            } else if args[2] == std::ffi::OsStr::new("--reference-route-v11") {
                boardsi_import::reference_route_v11::encode(
                    &bytes,
                    &indices,
                    split,
                    [ref_a, ref_b],
                    layers,
                    candidate,
                )
            } else {
                boardsi_import::reference_route_v10::encode(
                    &bytes,
                    &indices,
                    split,
                    [ref_a, ref_b],
                    layers,
                    candidate,
                )
            }
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("C014_C015_C016_CUTOUT_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // C_013: source-exact Signal Via Pad vs Reference Copper Cutout; V8 untouched.
    if args.len() == 10 && args[2] == std::ffi::OsStr::new("--reference-route-v9") {
        let result = (|| -> Result<String, &'static str> {
            let raw = args[3].to_str().ok_or("INVALID_ROUTE_INDICES")?;
            if raw.len() > 95 || raw.is_empty() {
                return Err("INVALID_ROUTE_INDICES");
            }
            let indices: Vec<usize> = raw
                .split(',')
                .map(|part| part.parse::<usize>().map_err(|_| "INVALID_ROUTE_INDEX"))
                .collect::<Result<_, _>>()?;
            let parse_usize = |index: usize| {
                args[index]
                    .to_str()
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or("INVALID_V9_INTEGER")
            };
            let split = parse_usize(4)?;
            let ref_a = args[5].to_str().ok_or("INVALID_REFERENCE_A")?;
            let ref_b = args[7].to_str().ok_or("INVALID_REFERENCE_B")?;
            let layers = [parse_usize(6)?, parse_usize(8)?];
            let candidate = parse_usize(9)?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::reference_route_v9::encode(
                &bytes,
                &indices,
                split,
                [ref_a, ref_b],
                layers,
                candidate,
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("C013_CUTOUT_CLEARANCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // C_012: source-exact Signal Via / Stitch Via Pad separation; V7 untouched.
    if args.len() == 10 && args[2] == std::ffi::OsStr::new("--reference-route-v8") {
        let result = (|| -> Result<String, &'static str> {
            let raw = args[3].to_str().ok_or("INVALID_ROUTE_INDICES")?;
            if raw.len() > 95 || raw.is_empty() {
                return Err("INVALID_ROUTE_INDICES");
            }
            let indices: Vec<usize> = raw
                .split(',')
                .map(|part| part.parse::<usize>().map_err(|_| "INVALID_ROUTE_INDEX"))
                .collect::<Result<_, _>>()?;
            let parse_usize = |index: usize| {
                args[index]
                    .to_str()
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or("INVALID_V8_INTEGER")
            };
            let split = parse_usize(4)?;
            let ref_a = args[5].to_str().ok_or("INVALID_REFERENCE_A")?;
            let ref_b = args[7].to_str().ok_or("INVALID_REFERENCE_B")?;
            let layers = [parse_usize(6)?, parse_usize(8)?];
            let candidate = parse_usize(9)?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::reference_route_v8::encode(
                &bytes,
                &indices,
                split,
                [ref_a, ref_b],
                layers,
                candidate,
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("C012_PAD_CLEARANCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // C_011: source-exact Corridor/Reference Stitch Area identity, V6 untouched.
    if args.len() == 10 && args[2] == std::ffi::OsStr::new("--reference-route-v7") {
        let result = (|| -> Result<String, &'static str> {
            let raw = args[3].to_str().ok_or("INVALID_ROUTE_INDICES")?;
            if raw.len() > 95 || raw.is_empty() {
                return Err("INVALID_ROUTE_INDICES");
            }
            let indices: Vec<usize> = raw
                .split(',')
                .map(|part| part.parse::<usize>().map_err(|_| "INVALID_ROUTE_INDEX"))
                .collect::<Result<_, _>>()?;
            let parse_usize = |index: usize| {
                args[index]
                    .to_str()
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or("INVALID_V7_INTEGER")
            };
            let split = parse_usize(4)?;
            let ref_a = args[5].to_str().ok_or("INVALID_REFERENCE_A")?;
            let ref_b = args[7].to_str().ok_or("INVALID_REFERENCE_B")?;
            let layers = [parse_usize(6)?, parse_usize(8)?];
            let candidate = parse_usize(9)?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::reference_route_v7::encode(
                &bytes,
                &indices,
                split,
                [ref_a, ref_b],
                layers,
                candidate,
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("C011_STITCH_PAD_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // C_010: source-exact Corridor/Reference Stitch Area identity, V5 untouched.
    if args.len() == 10 && args[2] == std::ffi::OsStr::new("--reference-route-v6") {
        let result = (|| -> Result<String, &'static str> {
            let raw = args[3].to_str().ok_or("INVALID_ROUTE_INDICES")?;
            if raw.len() > 95 || raw.is_empty() {
                return Err("INVALID_ROUTE_INDICES");
            }
            let indices: Vec<usize> = raw
                .split(',')
                .map(|part| part.parse::<usize>().map_err(|_| "INVALID_ROUTE_INDEX"))
                .collect::<Result<_, _>>()?;
            let parse_usize = |index: usize| {
                args[index]
                    .to_str()
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or("INVALID_V6_INTEGER")
            };
            let split = parse_usize(4)?;
            let ref_a = args[5].to_str().ok_or("INVALID_REFERENCE_A")?;
            let ref_b = args[7].to_str().ok_or("INVALID_REFERENCE_B")?;
            let layers = [parse_usize(6)?, parse_usize(8)?];
            let candidate = parse_usize(9)?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::reference_route_v6::encode(
                &bytes,
                &indices,
                split,
                [ref_a, ref_b],
                layers,
                candidate,
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("C010_CORRIDOR_STITCH_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // C_009: explicit one Reference stitching via, strict source contact and V4 receipts.
    if args.len() == 10 && args[2] == std::ffi::OsStr::new("--reference-route-v5") {
        let result = (|| -> Result<String, &'static str> {
            let raw = args[3].to_str().ok_or("INVALID_ROUTE_INDICES")?;
            if raw.len() > 95 || raw.is_empty() {
                return Err("INVALID_ROUTE_INDICES");
            }
            let indices: Vec<usize> = raw
                .split(',')
                .map(|part| part.parse::<usize>().map_err(|_| "INVALID_ROUTE_INDEX"))
                .collect::<Result<_, _>>()?;
            let parse_usize = |index: usize| {
                args[index]
                    .to_str()
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or("INVALID_V5_INTEGER")
            };
            let split = parse_usize(4)?;
            let ref_a = args[5].to_str().ok_or("INVALID_REFERENCE_A")?;
            let ref_b = args[7].to_str().ok_or("INVALID_REFERENCE_B")?;
            let layers = [parse_usize(6)?, parse_usize(8)?];
            let candidate = parse_usize(9)?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::reference_route_v5::encode(
                &bytes,
                &indices,
                split,
                [ref_a, ref_b],
                layers,
                candidate,
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("C009_STITCHING_REFERENCE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // C_008: explicit two-layer split plus one plated-through source via; geometry only.
    if args.len() == 9 && args[2] == std::ffi::OsStr::new("--reference-route-v4") {
        let result = (|| -> Result<String, &'static str> {
            let raw = args[3].to_str().ok_or("INVALID_ROUTE_INDICES")?;
            if raw.len() > 95 || raw.is_empty() {
                return Err("INVALID_ROUTE_INDICES");
            }
            let indices: Vec<usize> = raw
                .split(',')
                .map(|part| part.parse::<usize>().map_err(|_| "INVALID_ROUTE_INDEX"))
                .collect::<Result<_, _>>()?;
            let split = args[4]
                .to_str()
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or("INVALID_ROUTE_SPLIT")?;
            let ref_a = args[5].to_str().ok_or("INVALID_REFERENCE_A")?;
            let layer_a = args[6]
                .to_str()
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or("INVALID_REFERENCE_LAYER_A")?;
            let ref_b = args[7].to_str().ok_or("INVALID_REFERENCE_B")?;
            let layer_b = args[8]
                .to_str()
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or("INVALID_REFERENCE_LAYER_B")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::reference_route_v4::encode(
                &bytes,
                &indices,
                split,
                [ref_a, ref_b],
                [layer_a, layer_b],
            )
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("C008_REFERENCE_ROUTE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // C_007: opt-in bounded multi-straight-route geometry-only V3.
    if args.len() == 6 && args[2] == std::ffi::OsStr::new("--reference-route-v3") {
        let result = (|| -> Result<String, &'static str> {
            let raw = args[3].to_str().ok_or("INVALID_ROUTE_INDICES")?;
            if raw.len() > 95 || raw.is_empty() {
                return Err("INVALID_ROUTE_INDICES");
            }
            let indices: Vec<usize> = raw
                .split(',')
                .map(|part| part.parse::<usize>().map_err(|_| "INVALID_ROUTE_INDEX"))
                .collect::<Result<_, _>>()?;
            let reference = args[4].to_str().ok_or("INVALID_REFERENCE_NET")?;
            let layer = args[5]
                .to_str()
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or("INVALID_REFERENCE_LAYER")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::reference_route_v3::encode(&bytes, &indices, reference, layer)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("C007_REFERENCE_ROUTE_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    // C_006 opt-in exact Reference gap intervals; C_005 V1 unchanged.
    if args.len() == 6 && args[2] == std::ffi::OsStr::new("--reference-gap-v2") {
        let result = (|| -> Result<String, &'static str> {
            let index = args[3]
                .to_str()
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or("INVALID_TRACE_INDEX")?;
            let reference = args[4].to_str().ok_or("INVALID_REFERENCE_NET")?;
            let layer = args[5]
                .to_str()
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or("INVALID_REFERENCE_LAYER")?;
            let path = std::path::Path::new(&args[1]);
            let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err("FILE_LIMIT");
            }
            let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
            boardsi_import::reference_gap_intervals::encode(&bytes, index, reference, layer)
        })();
        match result {
            Ok(wire) => print!("{wire}"),
            Err(reason) => {
                eprintln!("C006_REFERENCE_GAP_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    if args.len() >= 6 && args[2] == std::ffi::OsStr::new("--reference-risk-v1") {
        let parse_index = |index: usize| {
            args[index]
                .to_str()
                .and_then(|value| value.parse::<usize>().ok())
        };
        let selection = if args.len() == 6 {
            parse_index(3)
                .zip(parse_index(5))
                .map(|(index, layer)| (vec![index], vec![layer]))
        } else if args.len() == 8 {
            parse_index(3)
                .zip(parse_index(5))
                .zip(parse_index(6).zip(parse_index(7)))
                .map(|((i, l), (j, m))| (vec![i, j], vec![l, m]))
        } else {
            None
        };
        let result = selection
            .ok_or("INVALID_SELECTION")
            .and_then(|(ids, layers)| {
                let reference = args[4].to_str().ok_or("INVALID_REFERENCE_NET")?;
                let path = std::path::Path::new(&args[1]);
                let metadata = std::fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
                if !metadata.is_file()
                    || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
                {
                    return Err("FILE_LIMIT");
                }
                let bytes = std::fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
                boardsi_import::reference_risk_wire::encode(&bytes, &ids, reference, &layers)
            });
        match result {
            Ok(wire) => {
                print!("{wire}");
            }
            Err(reason) => {
                eprintln!("C005_REFERENCE_RISK_ERROR:{reason}");
                std::process::exit(3);
            }
        }
        return;
    }
    match run() {
        Ok((contacts, barrels)) => {
            // Emit success diagnostics only after the complete bounded artwork
            // payload was actually written, not during a failing parse.
            eprintln!(
                "BOARDSI_CONTACT_AUDIT_V1|{}|{}|CONFLICT_DETAIL_NOT_EXPOSED|LOCAL_ENDPOINT_ONLY",
                contacts, barrels
            );
        }
        Err(err) => {
            eprintln!("P2A_IMPORT_ERROR:{err}");
            std::process::exit(3);
        }
    }
}
