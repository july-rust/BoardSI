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
    // C_014: bounded multiple Source COPCUT rectangles; preserve V9 or V8.
    if args.len() == 10 && args[2] == std::ffi::OsStr::new("--reference-route-v10") {
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
            boardsi_import::reference_route_v10::encode(
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
                eprintln!("C014_MULTI_CUTOUT_ERROR:{reason}");
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
