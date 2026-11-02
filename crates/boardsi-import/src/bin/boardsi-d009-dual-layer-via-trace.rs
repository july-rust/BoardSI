//! D_009: opt-in dual-layer Via Pad/Trace/third-Area SOURCE geometry only, not electrical connectivity.
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
    names: [&str; 3],
    net: &str,
    layers: [usize; 2],
    index: usize,
) -> Result<String, &'static str> {
    if index != 0
        || layers[0] == 0
        || layers[0] >= layers[1]
        || names.iter().any(|n| n.is_empty() || n.len() > 16)
        || (names[0] == names[1] || names[0] == names[2] || names[1] == names[2])
        || net.is_empty()
        || net.len() > 47
    {
        return Err("D009_UNSUPPORTED_SELECTION");
    }
    let parsed = route_preview::parse(bytes)?;
    if parsed.skipped != 0 || parsed.vias.len() != 1 || parsed.segments.len() != 1 || !parsed.arcs.is_empty() {
        return Err("D009_INCOMPLETE_ROUTE");
    }
    if !pour_preview::parse(bytes, parsed.max_layer)?.is_empty() {
        return Err("D009_UNVERIFIED_POUR");
    }
    let src = &parsed.vias[0];
    if src.net != net
        || src.first_layer as usize > layers[0]
        || (src.last_layer as usize) < layers[1]
    {
        return Err("D009_VIA_NET_LAYER");
    }
    let mut board = route_preview::into_canonical(bytes, &parsed)?;
    let areas = copper_preview::parse(bytes, parsed.max_layer)?;
    copper_preview::augment(&mut board, &areas)?;
    if board.validate().has_errors()
        || board.sources.len() != 1
        || board.vias.len() != 1
        || board.traces.len() != parsed.segments.len()
        || areas.len() != 3
        || board.areas.len() != 3
    {
        return Err("D009_CANONICAL_INVENTORY");
    }
    let via = &board.vias[0];
    let net_id = board
        .nets
        .iter()
        .find(|n| n.name == net)
        .ok_or("D009_NET")?
        .meta
        .id
        .clone();
    if via.kind != ViaKind::Through
        || via.net.as_ref() != Some(&net_id)
        || via.position.ticks != src.point
        || src.drill_ticks <= 0
        || src.diameter_ticks <= src.drill_ticks
    {
        return Err("D009_VIA_SOURCE");
    }
    let drill = board
        .drills
        .iter()
        .find(|d| d.meta.id == via.drill)
        .ok_or("D009_DRILL")?;
    let expected_drill = Rational::new(i128::from(src.drill_ticks) * 127, 500_000_000)
        .map_err(|_| "D009_DRILL_RATIO")?;
    if !drill.plated || drill.span != via.span || drill.diameter_m != expected_drill {
        return Err("D009_DRILL_MISMATCH");
    }
    let stack = board
        .padstacks
        .iter()
        .find(|s| s.meta.id == via.padstack)
        .ok_or("D009_PADSTACK")?;
    let expected = Rational::new(i128::from(src.diameter_ticks) * 127, 500_000_000)
        .map_err(|_| "D009_PAD_RATIO")?;
    if stack.entries.iter().any(|e| e.role != PadRole::Conductive) {
        return Err("D009_NON_CONDUCTIVE_STACK");
    }
    for layer in layers {
        let selected = board.stackup.layers.get(layer - 1).ok_or("D009_LAYER")?;
        let entries: Vec<_> = stack
            .entries
            .iter()
            .filter(|e| e.layer == selected.meta.id)
            .collect();
        if entries.len() != 1
            || !matches!(&entries[0].shape,PadShape::Circle{diameter_m} if *diameter_m==expected)
        {
            return Err("D009_MISSING_LAYER_PAD");
        }
    }
    let mut out=format!("BOARDSI_D009_DUAL_LAYER_VIA_TRACE_THIRD_AREA_V1|{}|SOURCE_GEOMETRY_ONLY|PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|0|0\n",hex(&digest::sha256(bytes)));
    writeln!(
        &mut out,
        "VIA|0|{}|{}|{}|{}|{}|{}|{}",
        hex(net.as_bytes()),
        src.point[0],
        src.point[1],
        src.diameter_ticks,
        src.drill_ticks,
        src.first_layer,
        src.last_layer
    )
    .map_err(|_| "D009_WIRE")?;
    let mut success = true;
    let mut second_rect = None;
    for (slot, name) in names[..2].iter().enumerate() {
        let idx = areas
            .iter()
            .position(|a| a.name == *name)
            .ok_or("D009_AREA_UNKNOWN")?;
        let area = &areas[idx];
        let canonical = &board.areas[idx];
        let layer = layers[slot];
        let selected = &board.stackup.layers[layer - 1].meta.id;
        if area.layer as usize != layer
            || area.net_label.as_deref() != Some(net)
            || canonical.layer != *selected
            || canonical.net.is_some()
            || area.cutouts.len() > 1
            || canonical.cutouts.len() != area.cutouts.len()
        {
            return Err("D009_AREA_OWNERSHIP");
        }
        let CopperGeometry::Polygon(vertices) = &area.geometry else {
            return Err("D009_UNSUPPORTED_OUTER");
        };
        let rect = copper_preview::rectangle_bounds(vertices).ok_or("D009_NONRECT_AREA")?;
        if !exact_ring(&canonical.outer.segments, vertices) {
            return Err("D009_CANONICAL_OUTER");
        }
        let hole = if let Some(cut) = area.cutouts.first() {
            let CopperGeometry::Polygon(v) = cut else {
                return Err("D009_UNSUPPORTED_HOLE");
            };
            if !exact_ring(&canonical.cutouts[0].boundary.segments, v) {
                return Err("D009_CANONICAL_HOLE");
            }
            Some(copper_preview::rectangle_bounds(v).ok_or("D009_NONRECT_HOLE")?)
        } else {
            None
        };
        if slot == 1 { second_rect = Some(rect); }
        let state = classify(src.point, src.diameter_ticks, rect, hole);
        success &= state == "SOURCE_VIA_PAD_AREA_CANDIDATE";
        writeln!(
            &mut out,
            "AREA|{slot}|{}|{}|{layer}|{}|{}|{}|{}",
            hex(name.as_bytes()),
            hex(canonical.meta.id.source_object.as_bytes()),
            rect[0],
            rect[1],
            rect[2],
            rect[3]
        )
        .map_err(|_| "D009_WIRE")?;
        if let Some(h) = hole {
            writeln!(&mut out, "HOLE|{slot}|{}|{}|{}|{}", h[0], h[1], h[2], h[3])
                .map_err(|_| "D009_WIRE")?;
        } else {
            writeln!(&mut out, "HOLE|{slot}|NONE").map_err(|_| "D009_WIRE")?;
        }
        writeln!(&mut out, "CONTACT|{slot}|{state}").map_err(|_| "D009_WIRE")?;
    }
    // The third area belongs only to Layer B. It is independently source-owned.
    let third_idx = areas.iter().position(|a| a.name == names[2])
        .ok_or("D009_THIRD_AREA_UNKNOWN")?;
    let third = &areas[third_idx];
    let third_canonical = &board.areas[third_idx];
    let selected = &board.stackup.layers[layers[1]-1].meta.id;
    if third.layer as usize != layers[1] || third.net_label.as_deref() != Some(net)
        || third_canonical.layer != *selected || third_canonical.net.is_some()
        || third.cutouts.len() > 1 || third_canonical.cutouts.len() != third.cutouts.len()
    { return Err("D009_THIRD_AREA_OWNERSHIP"); }
    let CopperGeometry::Polygon(v) = &third.geometry else {
        return Err("D009_UNSUPPORTED_THIRD");
    };
    if !exact_ring(&third_canonical.outer.segments,v) {
        return Err("D009_THIRD_RING");
    }
    let rect_c = copper_preview::rectangle_bounds(v).ok_or("D009_NONRECT_THIRD")?;
    let hole_c = if let Some(cut) = third.cutouts.first() {
        let CopperGeometry::Polygon(vertices) = cut else { return Err("D009_THIRD_HOLE"); };
        if !exact_ring(&third_canonical.cutouts[0].boundary.segments,vertices) {
            return Err("D009_THIRD_HOLE_RING");
        }
        Some(copper_preview::rectangle_bounds(vertices).ok_or("D009_NONRECT_THIRD_HOLE")?)
    } else { None };
    writeln!(&mut out, "AREA|2|{}|{}|{}|{}|{}|{}|{}",
        hex(names[2].as_bytes()),hex(third_canonical.meta.id.source_object.as_bytes()),
        layers[1],rect_c[0],rect_c[1],rect_c[2],rect_c[3])
        .map_err(|_| "D009_WIRE")?;
    if let Some(h)=hole_c {
        writeln!(&mut out,"HOLE|2|{}|{}|{}|{}",h[0],h[1],h[2],h[3])
            .map_err(|_| "D009_WIRE")?;
    } else { out.push_str("HOLE|2|NONE\n"); }
    let seg = &parsed.segments[0];
    if seg.net != net || seg.layer as usize != layers[1] || seg.width_ticks <= 0
        || seg.a == seg.b { return Err("D009_TRACE_NET_LAYER"); }
    // Enforce the same bounded source tick domain before any squared distance.
    let in_bounds = |v:i64| (-5_600_000..=5_600_000).contains(&v);
    if !src.point.iter().copied().all(in_bounds)
        || !seg.a.iter().copied().all(in_bounds)
        || !seg.b.iter().copied().all(in_bounds)
        || src.diameter_ticks <= 0 || src.diameter_ticks > 5_600_000
        || seg.width_ticks > 5_600_000 { return Err("D009_SOURCE_DOMAIN"); }
    let in_pad=|p:[i64;2]| {
        let dx=i128::from(p[0])-i128::from(src.point[0]);
        let dy=i128::from(p[1])-i128::from(src.point[1]);
        4*(dx*dx+dy*dy)<i128::from(src.diameter_ticks)*i128::from(src.diameter_ticks)
    };
    let a=in_pad(seg.a); let b=in_pad(seg.b);
    let rect_b=second_rect.ok_or("D009_SECOND_AREA")?;
    let strict = |p:[i64;2],r:Rect| p[0]>r[0] && p[0]<r[2]
        && p[1]>r[1] && p[1]<r[3];
    let touched = |p:[i64;2],r:Rect| p[0]>=r[0] && p[0]<=r[2]
        && p[1]>=r[1] && p[1]<=r[3];
    let mut via_end = "-".to_string();
    let mut area_end = "-".to_string();
    let mut link = "SOURCE_TRACE_AREA_NOT_VERIFIED";
    if success && a != b {
        let endpoint=if a {0usize} else {1usize};
        let other=if a {seg.b} else {seg.a};
        if strict(other,rect_c) && !hole_c.is_some_and(|h|touched(other,h))
            && !touched(other,rect_b) && !intersects(src.point,src.diameter_ticks,rect_c) {
            link="SOURCE_DUAL_LAYER_VIA_TRACE_THIRD_AREA_CANDIDATE";
            via_end=endpoint.to_string(); area_end=(1-endpoint).to_string();
        } else if hole_c.is_some_and(|h|touched(other,h)) {
            link="SOURCE_COPCUT_RISK";
        } else if touched(other,rect_b) {
            link="SOURCE_AMBIGUOUS_AREA_OWNER";
        } else if touched(other,rect_c) {
            link="SOURCE_BOUNDARY_RISK";
        }
    } else if a && b { link="SOURCE_AMBIGUOUS_ENDPOINT"; }
    else if !a && !b { link="SOURCE_VIA_TRACE_DISCONNECTED"; }
    writeln!(&mut out,"CONTACT|2|{}",if link=="SOURCE_DUAL_LAYER_VIA_TRACE_THIRD_AREA_CANDIDATE" {
        "SOURCE_TRACE_AREA_ENDPOINT_CANDIDATE"
    } else { "SOURCE_TRACE_AREA_NOT_VERIFIED" }).map_err(|_|"D009_WIRE")?;
    writeln!(&mut out,"TRACE|0|{}|{}|{}|{}|{}|{}|{}",
        layers[1],hex(net.as_bytes()),seg.a[0],seg.a[1],seg.b[0],seg.b[1],
        seg.width_ticks).map_err(|_|"D009_WIRE")?;
    writeln!(&mut out,"LINK|{link}|{via_end}|{area_end}|1|2|{}",
        if link=="SOURCE_DUAL_LAYER_VIA_TRACE_THIRD_AREA_CANDIDATE" {"1"} else {"0"})
        .map_err(|_|"D009_WIRE")?;
    success &= link=="SOURCE_DUAL_LAYER_VIA_TRACE_THIRD_AREA_CANDIDATE";
    writeln!(
        &mut out,
        "OVERALL|{}",
        if success {
            "SOURCE_DUAL_LAYER_VIA_TRACE_THIRD_AREA_CANDIDATE"
        } else {
            "SOURCE_DUAL_LAYER_TRACE_NOT_VERIFIED"
        }
    )
    .map_err(|_| "D009_WIRE")?;
    out.push_str("END|SOURCE_ONLY_NOT_ELECTRICAL\n");
    if out.len() > 32768 {
        return Err("D009_WIRE_LIMIT");
    }
    Ok(out)
}
fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 9 {
        eprintln!("D009_USAGE: board.asc AREA_A AREA_B AREA_C NET LAYER_A LAYER_B VIA_INDEX");
        std::process::exit(2);
    }
    let txt = |i: usize| args[i].to_str().ok_or("D009_UTF8");
    let result = (|| -> Result<String, &'static str> {
        let path = std::path::Path::new(&args[1]);
        let m = std::fs::metadata(path).map_err(|_| "D009_FILE")?;
        if !m.is_file() || m.len() > route_preview::MAX_FILE_BYTES as u64 {
            return Err("D009_FILE_LIMIT");
        }
        let bytes = std::fs::read(path).map_err(|_| "D009_READ")?;
        encode(
            &bytes,
            [txt(2)?, txt(3)?, txt(4)?],
            txt(5)?,
            [
                txt(7)?.parse().map_err(|_| "D009_LAYER")?,
                txt(6)?.parse().map_err(|_| "D009_LAYER")?,
            ],
            txt(8)?.parse().map_err(|_| "D009_INDEX")?,
        )
    })();
    match result {
        Ok(s) => print!("{s}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(3);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pad_inside_cutout_boundary_and_separation() {
        let r = [100, 200, 200, 300];
        let p = [150, 250];
        assert_eq!(
            classify([5000, 5000], 20, r, None),
            "SOURCE_SEPARATION_RISK"
        );
        assert_eq!(
            classify(p, 20, [0, 0, 5000, 5000], Some([160, 240, 200, 300])),
            "SOURCE_COPCUT_RISK"
        );
        assert_eq!(
            classify(p, 20, [0, 0, 5000, 5000], None),
            "SOURCE_VIA_PAD_AREA_CANDIDATE"
        );
        assert_eq!(
            classify(p, 20, [150, 250, 5000, 5000], None),
            "SOURCE_PAD_BOUNDARY_RISK"
        );
    }
}
