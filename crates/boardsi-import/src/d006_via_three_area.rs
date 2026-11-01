//! D_006: one Source plated Through Via round pad against three COPCLS rectangles.
//! A conservative full-footprint geometry candidate, NEVER an electrical edge.
use crate::{copper_preview, d003_three_area_copcut, digest, reference_route_v8, route_preview};
use std::fmt::Write;

type Rect = [i64; 4];
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contact {
    pub state: &'static str,
    pub owner: Option<usize>,
    pub group: Option<usize>,
    pub point: [i64; 2],
    pub diameter: i64,
}

// All source coordinates are integral centimil ticks. Checked i128 squares are
// used instead of approximating tangency with f64 or testing only Via centre.
fn disk_intersects_rect(p: [i64; 2], diameter: i64, r: Rect) -> bool {
    let x = i128::from(p[0]);
    let y = i128::from(p[1]);
    let dx = if x < i128::from(r[0]) {
        i128::from(r[0]) - x
    } else if x > i128::from(r[2]) {
        x - i128::from(r[2])
    } else { 0 };
    let dy = if y < i128::from(r[1]) {
        i128::from(r[1]) - y
    } else if y > i128::from(r[3]) {
        y - i128::from(r[3])
    } else { 0 };
    4 * (dx * dx + dy * dy) <= i128::from(diameter) * i128::from(diameter)
}
fn disk_strictly_inside(p: [i64; 2], diameter: i64, r: Rect) -> bool {
    let d = i128::from(diameter);
    let x = 2 * i128::from(p[0]);
    let y = 2 * i128::from(p[1]);
    d > 0 && 2 * i128::from(r[0]) + d < x
        && x < 2 * i128::from(r[2]) - d
        && 2 * i128::from(r[1]) + d < y
        && y < 2 * i128::from(r[3]) - d
}
pub fn classify(
    point: [i64; 2],
    diameter: i64,
    boxes: [Rect; 3],
    holes: [Option<Rect>; 3],
    groups: [usize; 3],
) -> Result<Contact, &'static str> {
    if diameter <= 0 || groups.iter().any(|&g| g >= 3) {
        return Err("D006_INVALID_PAD_OR_GROUP");
    }
    let mut touched = Vec::new();
    let mut retained = Vec::new();
    let mut blocked = false;
    for i in 0..3 {
        let r = boxes[i];
        if r[0] >= r[2] || r[1] >= r[3] {
            return Err("D006_INVALID_RECTANGLE");
        }
        if !disk_intersects_rect(point, diameter, r) {
            continue;
        }
        touched.push(i);
        let clear = holes[i].map_or(true, |hole| !disk_intersects_rect(point, diameter, hole));
        if !clear { blocked = true; }
        if clear && disk_strictly_inside(point, diameter, r) {
            retained.push(i);
        }
    }
    let (state, owner) = if blocked {
        ("SOURCE_COPCUT_RISK", None)
    } else if touched.len() > 1 {
        ("SOURCE_AMBIGUOUS_AREA_OWNER", None)
    } else if retained.len() == 1 {
        ("SOURCE_VIA_PAD_AREA_CANDIDATE", retained.first().copied())
    } else if touched.is_empty() {
        ("SOURCE_SEPARATION_RISK", None)
    } else {
        ("SOURCE_PAD_BOUNDARY_RISK", None)
    };
    Ok(Contact {
        state,
        owner,
        group: owner.map(|i| groups[i]),
        point,
        diameter,
    })
}
fn parse_i64(value: &str) -> Result<i64, &'static str> {
    value.parse().map_err(|_| "D006_MALFORMED_D003")
}
pub fn qualify(
    bytes: &[u8],
    names: [&str; 3],
    net: &str,
    layer: usize,
    via_index: usize,
) -> Result<(String, Contact, route_preview::ViaInstance), &'static str> {
    if via_index != 0 {
        return Err("D006_VIA_SELECTION_UNSUPPORTED");
    }
    let source = route_preview::parse(bytes)?;
    if source.skipped != 0 || source.vias.len() != 1 || !source.arcs.is_empty() {
        return Err("D006_INCOMPLETE_SOURCE_VIA_INVENTORY");
    }
    let selected = source.vias.first().ok_or("D006_VIA_MISSING")?;
    if selected.net != net || layer == 0 {
        return Err("D006_VIA_NET_OR_LAYER");
    }
    let board = route_preview::into_canonical(bytes, &source)?;
    if board.validate().has_errors()
        || board.vias.len() != 1
        || board.traces.len() != source.segments.len()
        || board.sources.len() != 1
    {
        return Err("D006_CANONICAL_SOURCE_MISMATCH");
    }
    let diameter = reference_route_v8::verify_round_pad(&board, selected, via_index, layer)?;
    // This verifies every Source/CANONICAL area rectangle, net and owned cutout.
    let d003 = d003_three_area_copcut::encode(bytes, names, net, layer)?;
    let rows: Vec<_> = d003.lines().collect();
    if rows.len() != 9 {
        return Err("D006_D003_RECEIPT");
    }
    let mut rectangles = [[0; 4]; 3];
    let mut groups = [0usize; 3];
    let mut holes = [None; 3];
    for i in 0..3 {
        let cols: Vec<_> = rows[i+1].split('|').collect();
        if cols.len() != 11 || cols[0] != "AREA" || cols[1] != i.to_string() {
            return Err("D006_D003_AREA");
        }
        for j in 0..4 {
            rectangles[i][j] = parse_i64(cols[j+5])?;
        }
        groups[i] = cols[10].parse().map_err(|_| "D006_D003_GROUP")?;
    }
    let cut: Vec<_> = rows[4].split('|').collect();
    if cut.len() == 6 && cut[0] == "HOLE" {
        let owner: usize = cut[1].parse().map_err(|_| "D006_D003_HOLE")?;
        if owner >= 3 { return Err("D006_D003_HOLE_OWNER"); }
        let mut r = [0; 4];
        for j in 0..4 { r[j] = parse_i64(cut[j+2])?; }
        holes[owner] = Some(r);
    } else if cut != ["HOLE", "NONE"] {
        return Err("D006_D003_HOLE_RECEIPT");
    }
    let contact = classify(selected.point, diameter, rectangles, holes, groups)?;
    Ok((d003, contact, selected.clone()))
}
pub fn encode(
    bytes: &[u8],
    names: [&str; 3],
    net: &str,
    layer: usize,
    via_index: usize,
) -> Result<String, &'static str> {
    let (d003, c, via) = qualify(bytes, names, net, layer, via_index)?;
    let mut sha = String::new();
    let mut net_hex = String::new();
    for b in digest::sha256(bytes) { write!(sha, "{b:02x}").map_err(|_| "D006_SHA")?; }
    for b in net.as_bytes() { write!(net_hex, "{b:02x}").map_err(|_| "D006_NET")?; }
    let mut out = format!(
        "BOARDSI_D006_VIA_THREE_AREA_V1|{sha}|SOURCE_GEOMETRY_ONLY|PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|0|0\n{d003}"
    );
    writeln!(out, "VIA|{via_index}|{layer}|{net_hex}|{}|{}|{}|{}|{}|{}",
        via.point[0], via.point[1], c.diameter, via.drill_ticks,
        via.first_layer, via.last_layer).map_err(|_| "D006_WIRE")?;
    writeln!(out, "CONTACT|{}|{}|{}", c.state,
        c.owner.map_or_else(|| "-".to_string(), |v| v.to_string()),
        c.group.map_or_else(|| "-".to_string(), |v| v.to_string()))
        .map_err(|_| "D006_WIRE")?;
    out.push_str("END|SOURCE_ONLY_NOT_ELECTRICAL\n");
    if out.len() > 16384 { return Err("D006_WIRE_LIMIT"); }
    Ok(out)
}
#[cfg(test)]
mod geometry_tests {
    use super::*;
    const A: Rect = [0, 0, 100, 100];
    const B: Rect = [95, 0, 195, 100];
    const C: Rect = [190, 0, 290, 100];
    fn check(p: [i64; 2], d: i64, h: Option<Rect>) -> Contact {
        classify(p,d,[A,B,C],[h,None,None],[0,0,0]).unwrap()
    }
    #[test]
    fn strict_whole_pad_not_centre_only() {
        assert_eq!(check([50,50], 20, None).state, "SOURCE_VIA_PAD_AREA_CANDIDATE");
        assert_eq!(check([90,50], 20, None).state, "SOURCE_AMBIGUOUS_AREA_OWNER");
        assert_eq!(check([10,50], 20, None).state, "SOURCE_PAD_BOUNDARY_RISK");
        assert_eq!(check([300,50], 20, None).state, "SOURCE_PAD_BOUNDARY_RISK");
        assert_eq!(check([310,50], 20, None).state, "SOURCE_SEPARATION_RISK");
    }
    #[test]
    fn corner_and_cutout_disk_tangency_is_rejected() {
        assert_eq!(check([50,50],20,Some([60,40,80,60])).state, "SOURCE_COPCUT_RISK");
        assert_eq!(check([50,50],20,Some([70,40,80,60])).state,"SOURCE_VIA_PAD_AREA_CANDIDATE");
    }
    #[test]
    fn invalid_geometry_fail_closed() {
        assert!(classify([0,0],0,[A,B,C],[None;3],[0,0,0]).is_err());
        assert!(classify([0,0],20,[A,B,C],[None;3],[3,0,0]).is_err());
    }
}
