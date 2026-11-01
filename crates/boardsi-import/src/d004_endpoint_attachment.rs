//! D_004: attach one Source straight Trace's endpoints to D_003 geometric components.
//! An attachment is a *source geometry candidate*, never an electrical contact.
use crate::{d003_three_area_copcut, route_preview};
use std::fmt::Write;

pub type Rect = [i64; 4];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attachment {
    pub state: &'static str,
    pub owner: Option<usize>,
    pub component: Option<usize>,
}
fn strict(p: [i64; 2], r: Rect) -> bool {
    r[0] < p[0] && p[0] < r[2] && r[1] < p[1] && p[1] < r[3]
}
fn closed(p: [i64; 2], r: Rect) -> bool {
    r[0] <= p[0] && p[0] <= r[2] && r[1] <= p[1] && p[1] <= r[3]
}
/// No nearest-area guessing or label-only contact. Multiple retained owners
/// are ambiguous even when their geometry candidate component IDs agree.
pub fn attach(
    p: [i64; 2],
    boxes: [Rect; 3],
    holes: [Option<Rect>; 3],
    groups: [usize; 3],
) -> Attachment {
    let mut owners = Vec::new();
    let mut blocked = false;
    let mut boundary = false;
    for i in 0..3 {
        let inside = strict(p, boxes[i]);
        let on_or_in = closed(p, boxes[i]);
        if on_or_in && !inside {
            boundary = true;
        }
        if !inside {
            continue;
        }
        if let Some(h) = holes[i] {
            if closed(p, h) {
                if strict(p, h) {
                    blocked = true;
                } else {
                    boundary = true;
                }
                continue;
            }
        }
        owners.push(i);
    }
    if owners.len() > 1 {
        Attachment {
            state: "SOURCE_AMBIGUOUS_OWNER",
            owner: None,
            component: None,
        }
    } else if blocked || boundary {
        // A cutout/boundary is never silently promoted by an adjacent Area.
        Attachment {
            state: if blocked {
                "SOURCE_COPCUT_BLOCKED"
            } else {
                "SOURCE_BOUNDARY_RISK"
            },
            owner: None,
            component: None,
        }
    } else if let Some(&i) = owners.first() {
        Attachment {
            state: "SOURCE_ENDPOINT_ATTACHMENT_CANDIDATE",
            owner: Some(i),
            component: Some(groups[i]),
        }
    } else {
        Attachment {
            state: "SOURCE_SEPARATION_RISK",
            owner: None,
            component: None,
        }
    }
}
fn number(s: &str) -> Result<i64, &'static str> {
    s.parse::<i64>().map_err(|_| "D004_BAD_D003_RECEIPT")
}
fn hex(bytes: &[u8]) -> String {
    let mut s = String::new();
    for b in bytes {
        write!(&mut s, "{b:02x}").expect("hex");
    }
    s
}
/// Opt-in source/Canonical/SHA parity is delegated to D_003. The appended
/// Trace and points are validated from the SAME immutable source byte buffer.
pub fn encode(
    bytes: &[u8],
    names: [&str; 3],
    net: &str,
    layer: usize,
    trace_index: usize,
) -> Result<String, &'static str> {
    if trace_index != 0 {
        return Err("D004_TRACE_SELECTION_UNSUPPORTED");
    }
    let d003 = d003_three_area_copcut::encode(bytes, names, net, layer)?;
    let parsed = route_preview::parse(bytes)?;
    if parsed.skipped != 0
        || parsed.segments.len() != 1
        || !parsed.arcs.is_empty()
        || !parsed.vias.is_empty()
    {
        return Err("D004_INCOMPLETE_OR_UNSUPPORTED_TRACE_INVENTORY");
    }
    let seg = &parsed.segments[0];
    if seg.layer as usize != layer || seg.net != net || seg.width_ticks <= 0 {
        return Err("D004_TRACE_NET_LAYER_OR_WIDTH");
    }
    let rows: Vec<&str> = d003.lines().collect();
    if rows.len() != 9 {
        return Err("D004_BAD_D003_RECEIPT");
    }
    let mut boxes = [[0; 4]; 3];
    let mut groups = [0usize; 3];
    let mut holes = [None; 3];
    for i in 0..3 {
        let a: Vec<&str> = rows[i + 1].split('|').collect();
        if a.len() != 11 || a[0] != "AREA" {
            return Err("D004_BAD_D003_RECEIPT");
        }
        for j in 0..4 {
            boxes[i][j] = number(a[j + 5])?;
        }
        groups[i] = a[10]
            .parse::<usize>()
            .map_err(|_| "D004_BAD_D003_RECEIPT")?;
        if groups[i] > 2 {
            return Err("D004_BAD_D003_RECEIPT");
        }
    }
    let cut: Vec<&str> = rows[4].split('|').collect();
    if cut.len() == 6 && cut[0] == "HOLE" {
        let owner = cut[1]
            .parse::<usize>()
            .map_err(|_| "D004_BAD_D003_RECEIPT")?;
        if owner > 2 {
            return Err("D004_BAD_D003_RECEIPT");
        }
        let mut r = [0; 4];
        for j in 0..4 {
            r[j] = number(cut[j + 2])?;
        }
        holes[owner] = Some(r);
    } else if cut != ["HOLE", "NONE"] {
        return Err("D004_BAD_D003_RECEIPT");
    }
    let a = attach(seg.a, boxes, holes, groups);
    let b = attach(seg.b, boxes, holes, groups);
    let sha = hex(&crate::digest::sha256(bytes));
    let mut out = format!(
        "BOARDSI_D004_ENDPOINT_V1|{sha}|SOURCE_ATTACHMENT_ONLY|PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|0|0\n{d003}"
    );
    writeln!(
        out,
        "TRACE|0|{layer}|{}|{}|{}|{}|{}|{}",
        hex(net.as_bytes()),
        seg.a[0],
        seg.a[1],
        seg.b[0],
        seg.b[1],
        seg.width_ticks
    )
    .map_err(|_| "D004_WIRE_WRITE")?;
    for (i, result) in [a, b].iter().enumerate() {
        let owner = result.owner.map(|v| v.to_string()).unwrap_or("-".into());
        let group = result
            .component
            .map(|v| v.to_string())
            .unwrap_or("-".into());
        writeln!(out, "POINT|{i}|{}|{owner}|{group}", result.state)
            .map_err(|_| "D004_WIRE_WRITE")?;
    }
    out.push_str("END|2|SOURCE_ONLY_NOT_PHYSICAL\n");
    if out.len() > 12288 {
        return Err("D004_WIRE_LIMIT");
    }
    Ok(out)
}
