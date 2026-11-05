//! D_040: source-owned two isolated same-layer local copper components.
//! This bounded diagnostic does NOT certify PCB-wide electrical Open/Short.
use crate::{d001_pad_endpoint, digest, part_preview, route_preview};
use std::fmt::Write;

fn bounded_source(bytes: &[u8]) -> Result<(), &'static str> {
    let text = std::str::from_utf8(bytes).map_err(|_| "D040_UTF8")?;
    let mut sections = [0usize; 7];
    let mut active: Option<usize> = None;
    let mut ended = false;
    let mut decal = Vec::new();
    let mut types = Vec::new();
    let mut parts = Vec::new();
    let mut pairs = [0usize; 2];
    let mut vertices = 0usize;
    let mut nets = Vec::new();
    for row in text.lines().skip(1) {
        let line = row.trim();
        if ended {
            if !line.is_empty() { return Err("D040_AFTER_END"); }
            continue;
        }
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        if fields.is_empty() || matches!(fields[0], "REMARK" | "*REMARK*") {
            continue;
        }
        if fields[0].starts_with('*') {
            let index = match fields[0] {
                "*PCB*" => 0,
                "*PARTDECAL*" => 1,
                "*PARTTYPE*" => 2,
                "*PART*" => 3,
                "*ROUTE*" => 4,
                "*SIGNAL*" => 5,
                "*END*" => 6,
                _ => return Err("D040_UNSUPPORTED_SECTION"),
            };
            sections[index] += 1;
            if sections[index] > if index == 5 { 2 } else { 1 } {
                return Err("D040_DUPLICATE_SECTION");
            }
            if index == 5 {
                if active != Some(4) && active != Some(5) { return Err("D040_SIGNAL_CONTEXT"); }
                if fields.len() != 2 || !matches!(fields[1], "NET_A" | "NET_B")
                    || nets.contains(&fields[1]) { return Err("D040_NET_SECTION"); }
                nets.push(fields[1]);
            } else if fields.len() != 1 {
                return Err("D040_SECTION_FIELDS");
            }
            active = Some(index);
            if index == 6 { ended = true; }
            continue;
        }
        match active {
            Some(0) if fields.as_slice() == ["MAXLAYER", "4"] => {},
            Some(1) => decal.push(fields.join(" ")),
            Some(2) => types.push(fields.join(" ")),
            Some(3) => parts.push(fields),
            Some(5) => {
                if fields.len() == 2 {
                    match fields.as_slice() {
                        ["U1.1", "U1.2"] => pairs[0] += 1,
                        ["U2.1", "U2.2"] => pairs[1] += 1,
                        _ => return Err("D040_UNSUPPORTED_PIN_PAIR"),
                    }
                } else if fields.len() == 5
                    && fields.iter().all(|s| s.parse::<f64>().is_ok()) {
                    vertices += 1;
                } else {
                    return Err("D040_UNSUPPORTED_ROUTE_ROW");
                }
            }
            _ => return Err("D040_UNSUPPORTED_SOURCE_ROW"),
        }
    }
    if sections[0..5] != [1, 1, 1, 1, 1]
        || !(sections[5] == 1 || sections[5] == 2)
        || sections[6] != 1 || pairs != [2, 2] || vertices != 8 {
        return Err("D040_SOURCE_INVENTORY");
    }
    let expected_decal = [
        "D2 I 1000 1000 0 2 2 0 0", "T0 0 0 0 1",
        "T100 0 0 0 2", "PAD 1 3", "-2 40 R 0",
        "-1 0 R 0", "0 0 R 0", "PAD 2 3",
        "-2 30 S 0", "-1 0 R 0", "0 0 R 0",
    ];
    if decal.iter().map(String::as_str).collect::<Vec<_>>() != expected_decal
        || types != ["TYPEA D2 UND 0 0 0 0"]
        || parts.len() != 2 { return Err("D040_PART_DEFINITION"); }
    for (index, fields) in parts.iter().enumerate() {
        let reference = if index == 0 { "U1" } else { "U2" };
        if fields.len() != 12 || fields[0] != reference || fields[1] != "TYPEA"
            || fields[4] != "90" || fields[5..] != ["U", "N", "0", "-1", "0", "-1", "0"]
            || fields[2].parse::<f64>().is_err() || fields[3].parse::<f64>().is_err() {
            return Err("D040_PART_OWNERSHIP");
        }
    }
    Ok(())
}

fn strict_between(a: i64, m: i64, b: i64) -> bool {
    (a < m && m < b) || (b < m && m < a)
}

#[derive(Clone)]
struct Path<'a> {
    pad: &'a part_preview::ComponentPad,
    joint: [i64; 2],
    width: i64,
    net: &'a str,
}

#[derive(Clone, Copy)]
struct Box2 {
    xmin: i128, ymin: i128, xmax: i128, ymax: i128,
}

impl Box2 {
    fn from_pad(p: &part_preview::ComponentPad) -> Self {
        let x = i128::from(p.point[0]) * 2;
        let y = i128::from(p.point[1]) * 2;
        let w = i128::from(p.width_ticks);
        let h = i128::from(p.height_ticks);
        Self { xmin: x-w, ymin: y-h, xmax: x+w, ymax: y+h }
    }
    fn from_trace(p: &Path<'_>) -> Self {
        let x1 = i128::from(p.pad.point[0]) * 2;
        let y1 = i128::from(p.pad.point[1]) * 2;
        let x2 = i128::from(p.joint[0]) * 2;
        let y2 = i128::from(p.joint[1]) * 2;
        let w = i128::from(p.width);
        Self { xmin: x1.min(x2)-w, ymin: y1.min(y2)-w,
               xmax: x1.max(x2)+w, ymax: y1.max(y2)+w }
    }
    fn intersects(self, b: Self) -> bool {
        self.xmin <= b.xmax && b.xmin <= self.xmax
            && self.ymin <= b.ymax && b.ymin <= self.ymax
    }
}

/// The stable wire is an isolated geometry receipt, not a global Canonical edge.
pub fn encode(bytes: &[u8]) -> Result<String, &'static str> {
    if bytes.is_empty() || bytes.len() > route_preview::MAX_FILE_BYTES {
        return Err("D040_SOURCE_SIZE");
    }
    bounded_source(bytes)?;
    let route = route_preview::parse(bytes)?;
    if route.skipped != 0 || route.segments.len() != 4
        || !route.vias.is_empty() || !route.arcs.is_empty() {
        return Err("D040_ROUTE_INCOMPLETE");
    }
    let pads = part_preview::parse(bytes, route.max_layer);
    if pads.skipped != 0 || pads.pads.len() != 4 { return Err("D040_PAD_INVENTORY"); }
    let audit = d001_pad_endpoint::audit(bytes)?;
    if audit.ambiguous_owners != 0 || audit.cross_net != 0
        || audit.unproven_net != 0 || audit.witnesses.len() != 4 {
        return Err("D040_SOURCE_CONTACTS");
    }
    let mut paths = Vec::new();
    for (index, trace) in route.segments.iter().enumerate() {
        if trace.layer != 1 || !matches!(trace.net.as_str(), "NET_A" | "NET_B")
            || trace.width_ticks <= 0 { return Err("D040_TRACE_PROPERTIES"); }
        let witnesses: Vec<_> = audit.witnesses.iter()
            .filter(|w| w.trace_index == index).collect();
        if witnesses.len() != 1 { return Err("D040_TRACE_PAD_AMBIGUOUS"); }
        let witness = witnesses[0];
        let pad = &pads.pads[witness.pad_index];
        if pad.layer != 1 || pad.bottom || pad.rectangular || pad.width_ticks <= 0
            || pad.width_ticks != pad.height_ticks
            || trace.width_ticks > pad.width_ticks / 2 {
            return Err("D040_PAD_SHAPE");
        }
        let point = if witness.endpoint == 0 { trace.a } else { trace.b };
        let joint = if witness.endpoint == 0 { trace.b } else { trace.a };
        if point != pad.point || point != witness.point || witness.layer != 1 {
            return Err("D040_PAD_CENTRE");
        }
        paths.push(Path { pad, joint, width: trace.width_ticks, net: &trace.net });
    }
    paths.sort_by(|a,b| (&a.pad.reference, &a.pad.pin).cmp(&(&b.pad.reference, &b.pad.pin)));
    for (index,path) in paths.iter().enumerate() {
        let owner = if index < 2 { "U1" } else { "U2" };
        let pin = if index % 2 == 0 { "1" } else { "2" };
        if path.pad.reference != owner || path.pad.pin != pin {
            return Err("D040_PAD_OWNER");
        }
    }
    for pair in paths.chunks_exact(2) {
        let (a,b) = (&pair[0],&pair[1]);
        if a.net != b.net || a.joint != b.joint || a.width != b.width {
            return Err("D040_LOCAL_JUNCTION_OR_NET");
        }
        let pa = a.pad.point; let pb = b.pad.point; let j = a.joint;
        let collinear = (pa[0] == pb[0] && j[0] == pa[0]
            && strict_between(pa[1],j[1],pb[1]))
            || (pa[1] == pb[1] && j[1] == pa[1]
            && strict_between(pa[0],j[0],pb[0]));
        if !collinear { return Err("D040_PATH_NOT_COLLINEAR"); }
        let da = i128::from((j[0]-pa[0]).abs()) + i128::from((j[1]-pa[1]).abs());
        let db = i128::from((j[0]-pb[0]).abs()) + i128::from((j[1]-pb[1]).abs());
        if 2*da <= i128::from(a.pad.width_ticks)
            || 2*db <= i128::from(b.pad.width_ticks) {
            return Err("D040_JUNCTION_INSIDE_PAD");
        }
    }
    // Conservative rejection of every possible cross-component conductor
    // contact. An overlapping AABB is NOT a proven electrical short.
    let boxes = paths.chunks_exact(2).map(|pair| {
        [Box2::from_pad(pair[0].pad), Box2::from_pad(pair[1].pad),
         Box2::from_trace(&pair[0]), Box2::from_trace(&pair[1])]
    }).collect::<Vec<_>>();
    if boxes[0].iter().any(|a| boxes[1].iter().any(|b| a.intersects(*b))) {
        return Err("D040_CROSS_COMPONENT_COPPER_COLLISION");
    }
    let mut sha = String::with_capacity(64);
    for b in digest::sha256(bytes) {
        write!(&mut sha, "{b:02x}").map_err(|_| "D040_FORMAT")?;
    }
    let mut out = format!(
        "BOARDSI_D040_MULTI_COMPONENT_V1|{sha}|1|LOCAL_COPPER_COMPONENT_ISOLATION_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|GLOBAL_ELECTRICAL_NOT_VERIFIED\n"
    );
    for (c,pair) in paths.chunks_exact(2).enumerate() {
        writeln!(&mut out, "C|{c}|{}|{}|1|4|3",
            pair[0].pad.reference, pair[0].net).map_err(|_| "D040_FORMAT")?;
    }
    for (i,p) in paths.iter().enumerate() {
        let shape = if p.pad.square { "S" } else { "R" };
        writeln!(&mut out,"P|{i}|{}|{}|1|{}|{}|{}|{}|{shape}",
            p.pad.reference,p.pad.pin,p.pad.point[0],p.pad.point[1],
            p.pad.width_ticks,p.pad.height_ticks).map_err(|_| "D040_FORMAT")?;
    }
    for (i,p) in paths.iter().enumerate() {
        writeln!(&mut out,"T|{i}|1|{}|{}|{}|{}|{}|{}",
            p.net,p.pad.point[0],p.pad.point[1],p.joint[0],p.joint[1],p.width)
            .map_err(|_| "D040_FORMAT")?;
    }
    for c in 0..2 {
        let p = c*2; let t = p+4; let e = c*3;
        writeln!(&mut out,"E|{e}|{p}|{t}|PAD_TRACE_LOCAL_COPPER")
            .map_err(|_| "D040_FORMAT")?;
        writeln!(&mut out,"E|{}|{}|{}|PAD_TRACE_LOCAL_COPPER",e+1,p+1,t+1)
            .map_err(|_| "D040_FORMAT")?;
        writeln!(&mut out,"E|{}|{t}|{}|EXACT_TRACE_JUNCTION_LOCAL_COPPER",e+2,t+1)
            .map_err(|_| "D040_FORMAT")?;
    }
    out.push_str("COUNT|NODES|8|EDGES|6|LOCAL_COMPONENTS|2|GLOBAL_ELECTRICAL|NOT_VERIFIED\nEND|LOCAL_COPPER_COMPONENTS_ONLY_NO_GLOBAL_PASS\n");
    Ok(out)
}
