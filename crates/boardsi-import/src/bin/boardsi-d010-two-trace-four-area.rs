//! D_010: opt-in dual-layer Via Pad/Trace/third-Area SOURCE geometry only, not electrical connectivity.
use boardsi_import::{
    copper_preview::{self, CopperGeometry},
    digest, pour_preview, route_preview,
};
use boardsi_model::{BoundarySegment, PadRole, PadShape, Rational, ViaKind};
use std::fmt::Write;
type Rect = [i64; 4];

fn inside(p: [i64; 2], diameter: i64, r: Rect) -> bool {
    let x = i128::from(p[0]) * 2;
    let y = i128::from(p[1]) * 2;
    let d = i128::from(diameter);
    d > 0
        && x > 2 * i128::from(r[0]) + d
        && x < 2 * i128::from(r[2]) - d
        && y > 2 * i128::from(r[1]) + d
        && y < 2 * i128::from(r[3]) - d
}
fn intersects(p: [i64; 2], diameter: i64, r: Rect) -> bool {
    let x = i128::from(p[0]);
    let y = i128::from(p[1]);
    let dx = if x < i128::from(r[0]) {
        i128::from(r[0]) - x
    } else if x > i128::from(r[2]) {
        x - i128::from(r[2])
    } else {
        0
    };
    let dy = if y < i128::from(r[1]) {
        i128::from(r[1]) - y
    } else if y > i128::from(r[3]) {
        y - i128::from(r[3])
    } else {
        0
    };
    4 * (dx * dx + dy * dy) <= i128::from(diameter) * i128::from(diameter)
}
fn classify(p: [i64; 2], d: i64, r: Rect, hole: Option<Rect>) -> &'static str {
    if hole.is_some_and(|h| intersects(p, d, h)) {
        "SOURCE_COPCUT_RISK"
    } else if inside(p, d, r) {
        "SOURCE_VIA_PAD_AREA_CANDIDATE"
    } else if intersects(p, d, r) {
        "SOURCE_PAD_BOUNDARY_RISK"
    } else {
        "SOURCE_SEPARATION_RISK"
    }
}
fn hex(b: &[u8]) -> String {
    let mut s = String::new();
    for c in b {
        write!(s, "{c:02x}").expect("hex");
    }
    s
}
fn exact_ring(segments: &[BoundarySegment], vertices: &[[i64; 2]]) -> bool {
    vertices.len()==5 && segments.len()==4 && segments.iter().zip(vertices.windows(2)).all(|(s,p)| {
        matches!(s,BoundarySegment::Line{start,end} if start.ticks==p[0] && end.ticks==p[1] && start.frame==end.frame)
    })
}
pub fn encode(
    bytes: &[u8],
    names: [&str; 4],
    net: &str,
    layers: [usize; 2],
    index: usize,
) -> Result<String, &'static str> {
    if index != 0
        || layers[0] == 0
        || layers[0] >= layers[1]
        || names.iter().any(|n| n.is_empty() || n.len() > 16)
        || names.iter().enumerate().any(|(i, n)| names.iter().skip(i + 1).any(|m| m == n))
        || net.is_empty()
        || net.len() > 47
    {
        return Err("D010_UNSUPPORTED_SELECTION");
    }
    let parsed = route_preview::parse(bytes)?;
    if parsed.skipped != 0
        || parsed.vias.len() != 1
        || parsed.segments.len() != 2
        || !parsed.arcs.is_empty()
    {
        return Err("D010_INCOMPLETE_ROUTE");
    }
    if !pour_preview::parse(bytes, parsed.max_layer)?.is_empty() {
        return Err("D010_UNVERIFIED_POUR");
    }
    let src = &parsed.vias[0];
    if src.net != net
        || src.first_layer as usize > layers[0]
        || (src.last_layer as usize) < layers[1]
    {
        return Err("D010_VIA_NET_LAYER");
    }
    let mut board = route_preview::into_canonical(bytes, &parsed)?;
    let areas = copper_preview::parse(bytes, parsed.max_layer)?;
    copper_preview::augment(&mut board, &areas)?;
    if board.validate().has_errors()
        || board.sources.len() != 1
        || board.vias.len() != 1
        || board.traces.len() != parsed.segments.len()
        || areas.len() != 4
        || board.areas.len() != 4
    {
        return Err("D010_CANONICAL_INVENTORY");
    }
    let via = &board.vias[0];
    let net_id = board
        .nets
        .iter()
        .find(|n| n.name == net)
        .ok_or("D010_NET")?
        .meta
        .id
        .clone();
    if via.kind != ViaKind::Through
        || via.net.as_ref() != Some(&net_id)
        || via.position.ticks != src.point
        || src.drill_ticks <= 0
        || src.diameter_ticks <= src.drill_ticks
    {
        return Err("D010_VIA_SOURCE");
    }
    let drill = board
        .drills
        .iter()
        .find(|d| d.meta.id == via.drill)
        .ok_or("D010_DRILL")?;
    let expected_drill = Rational::new(i128::from(src.drill_ticks) * 127, 500_000_000)
        .map_err(|_| "D010_DRILL_RATIO")?;
    if !drill.plated || drill.span != via.span || drill.diameter_m != expected_drill {
        return Err("D010_DRILL_MISMATCH");
    }
    let stack = board
        .padstacks
        .iter()
        .find(|s| s.meta.id == via.padstack)
        .ok_or("D010_PADSTACK")?;
    let expected = Rational::new(i128::from(src.diameter_ticks) * 127, 500_000_000)
        .map_err(|_| "D010_PAD_RATIO")?;
    if stack.entries.iter().any(|e| e.role != PadRole::Conductive) {
        return Err("D010_NON_CONDUCTIVE_STACK");
    }
    for layer in layers {
        let selected = board.stackup.layers.get(layer - 1).ok_or("D010_LAYER")?;
        let entries: Vec<_> = stack
            .entries
            .iter()
            .filter(|e| e.layer == selected.meta.id)
            .collect();
        if entries.len() != 1
            || !matches!(&entries[0].shape,PadShape::Circle{diameter_m} if *diameter_m==expected)
        {
            return Err("D010_MISSING_LAYER_PAD");
        }
    }
    // Four rectangular source areas: A/C on first layer, B/D on second.
    let mut rects = [[0_i64; 4]; 4];
    let mut holes: [Option<Rect>; 4] = [None; 4];
    let mut pad_states = ["SOURCE_SEPARATION_RISK"; 2];
    for slot in 0..4 {
        let name = names[slot];
        let idx = areas.iter().position(|a| a.name == name)
            .ok_or("D010_AREA_UNKNOWN")?;
        let area = &areas[idx];
        let canonical = &board.areas[idx];
        let layer = layers[slot % 2];
        let selected = &board.stackup.layers[layer - 1].meta.id;
        if area.layer as usize != layer
            || area.net_label.as_deref() != Some(net)
            || canonical.layer != *selected
            || canonical.net.is_some()
            || area.cutouts.len() > 1
            || canonical.cutouts.len() != area.cutouts.len()
        {
            return Err("D010_AREA_OWNERSHIP");
        }
        let CopperGeometry::Polygon(vertices) = &area.geometry else {
            return Err("D010_UNSUPPORTED_OUTER");
        };
        if !exact_ring(&canonical.outer.segments, vertices) {
            return Err("D010_CANONICAL_OUTER");
        }
        rects[slot] = copper_preview::rectangle_bounds(vertices)
            .ok_or("D010_NONRECT_AREA")?;
        if let Some(cut) = area.cutouts.first() {
            let CopperGeometry::Polygon(v) = cut else {
                return Err("D010_UNSUPPORTED_HOLE");
            };
            if !exact_ring(&canonical.cutouts[0].boundary.segments, v) {
                return Err("D010_CANONICAL_HOLE");
            }
            holes[slot] = Some(copper_preview::rectangle_bounds(v)
                .ok_or("D010_NONRECT_HOLE")?);
        }
        if slot < 2 {
            pad_states[slot] = classify(src.point, src.diameter_ticks, rects[slot], holes[slot]);
        }
    }
    let in_bounds = |v: i64| (-5_600_000..=5_600_000).contains(&v);
    if !src.point.iter().copied().all(in_bounds)
        || src.diameter_ticks <= 0
        || src.diameter_ticks > 5_600_000
    {
        return Err("D010_SOURCE_DOMAIN");
    }
    let strict = |p: [i64; 2], r: Rect| {
        p[0] > r[0] && p[0] < r[2] && p[1] > r[1] && p[1] < r[3]
    };
    let touched = |p: [i64; 2], r: Rect| {
        p[0] >= r[0] && p[0] <= r[2] && p[1] >= r[1] && p[1] <= r[3]
    };
    let in_pad = |p: [i64; 2]| {
        let dx = i128::from(p[0]) - i128::from(src.point[0]);
        let dy = i128::from(p[1]) - i128::from(src.point[1]);
        4 * (dx * dx + dy * dy)
            < i128::from(src.diameter_ticks) * i128::from(src.diameter_ticks)
    };
    let mut indexes = [0_usize; 2];
    let mut via_end = [-1_i32; 2];
    let mut area_end = [-1_i32; 2];
    let mut link = ["SOURCE_TRACE_AREA_NOT_VERIFIED"; 2];
    for slot in 0..2 {
        let (idx, seg) = parsed.segments.iter().enumerate()
            .find(|(_, s)| s.layer as usize == layers[slot])
            .ok_or("D010_MISSING_TRACE_LAYER")?;
        indexes[slot] = idx;
        if seg.net != net || seg.width_ticks <= 0 || seg.a == seg.b
            || seg.width_ticks > 5_600_000
            || !seg.a.iter().copied().all(in_bounds)
            || !seg.b.iter().copied().all(in_bounds)
        {
            return Err("D010_TRACE_NET_DOMAIN");
        }
        let a = in_pad(seg.a);
        let b = in_pad(seg.b);
        let target = slot + 2;
        if pad_states[slot] == "SOURCE_VIA_PAD_AREA_CANDIDATE" && a != b {
            let endpoint = if a { 0 } else { 1 };
            let other = if a { seg.b } else { seg.a };
            if strict(other, rects[target])
                && !holes[target].is_some_and(|h| touched(other, h))
                && !touched(other, rects[slot])
                && !intersects(src.point, src.diameter_ticks, rects[target])
            {
                link[slot] = "SOURCE_VIA_TRACE_TARGET_AREA_CANDIDATE";
                via_end[slot] = endpoint;
                area_end[slot] = 1 - endpoint;
            } else if holes[target].is_some_and(|h| touched(other, h)) {
                link[slot] = "SOURCE_COPCUT_RISK";
            } else if touched(other, rects[slot]) {
                link[slot] = "SOURCE_AMBIGUOUS_AREA_OWNER";
            } else if touched(other, rects[target]) {
                link[slot] = "SOURCE_BOUNDARY_RISK";
            }
        } else if a && b {
            link[slot] = "SOURCE_AMBIGUOUS_ENDPOINT";
        } else if !a && !b {
            link[slot] = "SOURCE_VIA_TRACE_DISCONNECTED";
        }
    }
    if indexes[0] == indexes[1] {
        return Err("D010_DUPLICATE_TRACE");
    }
    let success = pad_states.iter().all(|s| *s == "SOURCE_VIA_PAD_AREA_CANDIDATE")
        && link.iter().all(|s| *s == "SOURCE_VIA_TRACE_TARGET_AREA_CANDIDATE");
    let mut out = format!(
        "BOARDSI_D010_DUAL_LAYER_TWO_TRACE_FOUR_AREA_V1|{}|SOURCE_GEOMETRY_ONLY|PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|0|0\n",
        hex(&digest::sha256(bytes))
    );
    writeln!(&mut out, "VIA|0|{}|{}|{}|{}|{}|{}|{}",
        hex(net.as_bytes()), src.point[0], src.point[1], src.diameter_ticks,
        src.drill_ticks, src.first_layer, src.last_layer
    ).map_err(|_| "D010_WIRE")?;
    for slot in 0..4 {
        let name = names[slot];
        let idx = areas.iter().position(|a| a.name == name)
            .ok_or("D010_AREA_UNKNOWN")?;
        let area = &board.areas[idx];
        let layer = layers[slot % 2];
        let r = rects[slot];
        writeln!(&mut out, "AREA|{slot}|{}|{}|{layer}|{}|{}|{}|{}",
            hex(name.as_bytes()),
            hex(area.meta.id.source_object.as_bytes()),
            r[0], r[1], r[2], r[3]
        ).map_err(|_| "D010_WIRE")?;
        if let Some(h) = holes[slot] {
            writeln!(&mut out, "HOLE|{slot}|{}|{}|{}|{}",
                h[0], h[1], h[2], h[3]
            ).map_err(|_| "D010_WIRE")?;
        } else {
            writeln!(&mut out, "HOLE|{slot}|NONE").map_err(|_| "D010_WIRE")?;
        }
        let state = if slot < 2 {
            pad_states[slot]
        } else if via_end[slot-2] >= 0 {
            "SOURCE_TRACE_AREA_ENDPOINT_CANDIDATE"
        } else {
            "SOURCE_TRACE_AREA_NOT_VERIFIED"
        };
        writeln!(&mut out, "CONTACT|{slot}|{state}")
            .map_err(|_| "D010_WIRE")?;
    }
    for slot in 0..2 {
        let seg = &parsed.segments[indexes[slot]];
        writeln!(&mut out, "TRACE|{slot}|{}|{}|{}|{}|{}|{}|{}",
            layers[slot], hex(net.as_bytes()),
            seg.a[0], seg.a[1], seg.b[0], seg.b[1], seg.width_ticks
        ).map_err(|_| "D010_WIRE")?;
    }
    for slot in 0..2 {
        let ve = if via_end[slot] < 0 {
            "-".to_string()
        } else {
            via_end[slot].to_string()
        };
        let ae = if area_end[slot] < 0 {
            "-".to_string()
        } else {
            area_end[slot].to_string()
        };
        writeln!(&mut out, "LINK|{slot}|{}|{ve}|{ae}|0|{}|{}",
            link[slot], slot+2, if via_end[slot]>=0 {1} else {0}
        ).map_err(|_| "D010_WIRE")?;
    }
    writeln!(&mut out, "OVERALL|{}",
        if success {
            "SOURCE_DUAL_LAYER_TWO_TRACE_FOUR_AREA_CANDIDATE"
        } else {
            "SOURCE_DUAL_LAYER_TWO_TRACE_NOT_VERIFIED"
        }
    ).map_err(|_| "D010_WIRE")?;
    out.push_str("END|SOURCE_ONLY_NOT_ELECTRICAL\n");
    if out.len() > 32768 {
        return Err("D010_WIRE_LIMIT");
    }
    Ok(out)
}
fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 10 {
        eprintln!("D010_USAGE: board.asc AREA_A AREA_B AREA_C AREA_D NET LAYER_A LAYER_B VIA_INDEX");
        std::process::exit(2);
    }
    let txt = |i: usize| args[i].to_str().ok_or("D010_UTF8");
    let result = (|| -> Result<String, &'static str> {
        let path = std::path::Path::new(&args[1]);
        let m = std::fs::metadata(path).map_err(|_| "D010_FILE")?;
        if !m.is_file() || m.len() > route_preview::MAX_FILE_BYTES as u64 {
            return Err("D010_FILE_LIMIT");
        }
        let bytes = std::fs::read(path).map_err(|_| "D010_READ")?;
        encode(
            &bytes, [txt(2)?, txt(3)?, txt(4)?, txt(5)?], txt(6)?,
            [txt(7)?.parse().map_err(|_| "D010_LAYER")?,
             txt(8)?.parse().map_err(|_| "D010_LAYER")?],
            txt(9)?.parse().map_err(|_| "D010_INDEX")?
        )
    })();
    match result {
        Ok(s) => print!("{s}"),
        Err(e) => { eprintln!("{e}"); std::process::exit(3); }
    }
}
