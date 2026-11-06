//! D_045 bounded finite-width, perpendicular, same-layer trace candidate contacts.
//! Exact SOURCE geometry only. Neither manufactured continuity nor global SI proof.
use crate::{d040_multi_component, digest, route_preview};
use std::fmt::Write;

fn stripped_source(bytes: &[u8]) -> Result<Vec<u8>, &'static str> {
    let text = std::str::from_utf8(bytes).map_err(|_| "D045_UTF8")?;
    let rows: Vec<_> = text.lines().collect();
    let mut base = String::new();
    let mut in_signal = false;
    let mut bridge_count = 0;
    let mut i = 0;
    while i < rows.len() {
        let fields: Vec<_> = rows[i].split_ascii_whitespace().collect();
        if fields.first() == Some(&"*SIGNAL*") {
            in_signal = true;
        } else if fields.first() == Some(&"*END*") {
            in_signal = false;
        }
        if fields == ["U1.1", "U2.1"] {
            if !in_signal
                || bridge_count != 0
                || i + 2 >= rows.len()
                || rows[i + 1].split_ascii_whitespace().count() != 5
                || rows[i + 2].split_ascii_whitespace().count() != 5
            {
                return Err("D045_SOURCE_BRIDGE_INVENTORY");
            }
            bridge_count += 1;
            i += 3;
        } else {
            base.push_str(rows[i]);
            base.push('\n');
            i += 1;
        }
    }
    if bridge_count != 1 {
        return Err("D045_BRIDGE_MISSING");
    }
    Ok(base.into_bytes())
}

#[derive(Clone)]
struct Track {
    net: String,
    a: [i64; 2],
    b: [i64; 2],
    width: i64,
}

fn parse_track(row: &str) -> Result<Track, &'static str> {
    let f: Vec<_> = row.split('|').collect();
    if f.len() != 9 || f[0] != "T" || f[2] != "1" {
        return Err("D045_BASE_TRACE");
    }
    let n = |i: usize| f[i].parse::<i64>().map_err(|_| "D045_BASE_VALUE");
    Ok(Track {
        net: f[3].to_owned(),
        a: [n(4)?, n(5)?],
        b: [n(6)?, n(7)?],
        width: n(8)?,
    })
}

#[derive(Clone, Copy)]
struct Rect {
    xmin: i128,
    ymin: i128,
    xmax: i128,
    ymax: i128,
}

impl Rect {
    fn trace(t: &Track) -> Self {
        let x0 = i128::from(t.a[0]) * 2;
        let x1 = i128::from(t.b[0]) * 2;
        let y0 = i128::from(t.a[1]) * 2;
        let y1 = i128::from(t.b[1]) * 2;
        let w = i128::from(t.width);
        Self {
            xmin: x0.min(x1) - w,
            ymin: y0.min(y1) - w,
            xmax: x0.max(x1) + w,
            ymax: y0.max(y1) + w,
        }
    }

    fn touches(self, other: Self) -> bool {
        self.xmin <= other.xmax
            && other.xmin <= self.xmax
            && self.ymin <= other.ymax
            && other.ymin <= self.ymax
    }
}

// Exact minimum distance between orthogonal, non-intersecting round-capped strokes.
fn perpendicular_positive(a: &Track, b: &Track) -> bool {
    if a.width <= 0 || b.width <= 0 {
        return false;
    }
    let (v, h) = if a.a[0] == a.b[0] && a.a[1] != a.b[1] && b.a[1] == b.b[1] && b.a[0] != b.b[0] {
        (a, b)
    } else if b.a[0] == b.b[0] && b.a[1] != b.b[1] && a.a[1] == a.b[1] && a.a[0] != a.b[0] {
        (b, a)
    } else {
        return false;
    };
    let x = i128::from(v.a[0]);
    let y = i128::from(h.a[1]);
    let hx0 = i128::from(h.a[0].min(h.b[0]));
    let hx1 = i128::from(h.a[0].max(h.b[0]));
    let vy0 = i128::from(v.a[1].min(v.b[1]));
    let vy1 = i128::from(v.a[1].max(v.b[1]));
    let dx = (hx0 - x).max(x - hx1).max(0);
    let dy = (vy0 - y).max(y - vy1).max(0);
    if dx == 0 && dy == 0 {
        return false;
    }
    let twice_radius = i128::from(a.width) + i128::from(b.width);
    4 * (dx * dx + dy * dy) < twice_radius * twice_radius
}

fn same_pair(a: [i64; 2], b: [i64; 2], x: [i64; 2], y: [i64; 2]) -> bool {
    (a == x && b == y) || (a == y && b == x)
}

/// Qualifies exactly one synthetic, perpendicular, *positive-area* 5th trace witness.
pub fn encode(bytes: &[u8]) -> Result<String, &'static str> {
    if bytes.is_empty() || bytes.len() > route_preview::MAX_FILE_BYTES {
        return Err("D045_SOURCE_SIZE");
    }
    let baseline = d040_multi_component::encode(&stripped_source(bytes)?)?;
    let old: Vec<Track> = baseline
        .lines()
        .filter(|line| line.starts_with("T|"))
        .map(parse_track)
        .collect::<Result<_, _>>()?;
    if old.len() != 4
        || old.iter().any(|t| t.net != old[0].net || t.width != 1000)
        || old[0].a != [20_000, 30_000]
        || old[1].a != [20_000, 40_000]
        || old[2].a != [26_000, 30_000]
        || old[3].a != [26_000, 40_000]
        || old[0].b != [20_000, 35_000]
        || old[1].b != [20_000, 35_000]
        || old[2].b != [26_000, 35_000]
        || old[3].b != [26_000, 35_000]
    {
        return Err("D045_UNSUPPORTED_BASE_GEOMETRY");
    }
    let route = route_preview::parse(bytes)?;
    if route.skipped != 0
        || route.segments.len() != 5
        || !route.arcs.is_empty()
        || !route.vias.is_empty()
    {
        return Err("D045_ROUTE_INVENTORY");
    }
    let mut used = [false; 4];
    let mut candidate: Option<Track> = None;
    for seg in &route.segments {
        if seg.layer != 1 || seg.net != old[0].net || seg.width_ticks <= 0 {
            return Err("D045_LAYER_NET_WIDTH");
        }
        if let Some((i, _)) = old.iter().enumerate().find(|(i, base)| {
            !used[*i]
                && base.net == seg.net
                && base.width == seg.width_ticks
                && same_pair(base.a, base.b, seg.a, seg.b)
        }) {
            used[i] = true;
        } else {
            if candidate.is_some() {
                return Err("D045_DUPLICATE_OR_UNSUPPORTED");
            }
            candidate = Some(Track {
                net: seg.net.clone(),
                a: seg.a,
                b: seg.b,
                width: seg.width_ticks,
            });
        }
    }
    if used.iter().any(|used| !used) {
        return Err("D045_BASE_TRACE_MISSING");
    }
    let bridge = candidate.ok_or("D045_BRIDGE_NOT_IMPORTED")?;
    // Only one bounded horizontal witness between established vertical rails.
    // Centerline crossings, arbitrary polygons and rotations are unsupported.
    if bridge.a[1] != bridge.b[1]
        || bridge.a[0] == bridge.b[0]
        || !(22_000..=24_000).contains(&bridge.a[0])
        || !(22_000..=24_000).contains(&bridge.b[0])
        || !(33_000..=37_000).contains(&bridge.a[1])
        || bridge.width > 10_000
    {
        return Err("D045_PERPENDICULAR_GEOMETRY");
    }
    let footprint = Rect::trace(&bridge);
    if !old
        .iter()
        .all(|track| perpendicular_positive(track, &bridge))
    {
        return Err("D045_GAP_OR_BOUNDARY_ONLY");
    }
    for row in baseline.lines().filter(|row| row.starts_with("P|")) {
        let f: Vec<_> = row.split('|').collect();
        if f.len() != 10 {
            return Err("D045_BASE_PAD");
        }
        let n = |i: usize| f[i].parse::<i128>().map_err(|_| "D045_PAD_COORD");
        let (x, y, w, h) = (n(5)? * 2, n(6)? * 2, n(7)?, n(8)?);
        if footprint.touches(Rect {
            xmin: x - w,
            ymin: y - h,
            xmax: x + w,
            ymax: y + h,
        }) {
            return Err("D045_PAD_CONTACT_UNSUPPORTED");
        }
    }
    let mut sha = String::new();
    for byte in digest::sha256(bytes) {
        write!(&mut sha, "{byte:02x}").map_err(|_| "D045_FORMAT")?;
    }
    let mut out = format!(
        "BOARDSI_D045_PERPENDICULAR_WIDTH_CONTACT_V1|{sha}|1|SOURCE_LOCAL_FINITE_WIDTH_ONLY|P009_UNSUPPORTED|SI_NOT_RUN|GLOBAL_ELECTRICAL_NOT_VERIFIED\n"
    );
    writeln!(&mut out, "C|0|U1+U2|{}|1|9|10", old[0].net).map_err(|_| "D045_FORMAT")?;
    for row in baseline
        .lines()
        .filter(|line| line.starts_with("P|") || line.starts_with("T|") || line.starts_with("E|"))
    {
        if row.starts_with("E|") {
            continue;
        }
        writeln!(&mut out, "{row}").map_err(|_| "D045_FORMAT")?;
    }
    writeln!(
        &mut out,
        "T|4|1|{}|{}|{}|{}|{}|{}",
        bridge.net, bridge.a[0], bridge.a[1], bridge.b[0], bridge.b[1], bridge.width
    )
    .map_err(|_| "D045_FORMAT")?;
    for row in baseline.lines().filter(|line| line.starts_with("E|")) {
        writeln!(&mut out, "{row}").map_err(|_| "D045_FORMAT")?;
    }
    for i in 0..4 {
        writeln!(
            &mut out,
            "E|{}|{}|8|POSITIVE_FINITE_WIDTH_PERPENDICULAR_CONTACT",
            6 + i,
            4 + i
        )
        .map_err(|_| "D045_FORMAT")?;
    }
    out.push_str(
        "COUNT|NODES|9|EDGES|10|LOCAL_COMPONENTS|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n\
         END|SOURCE_LOCAL_FINITE_WIDTH_ONLY_NO_GLOBAL_PASS\n",
    );
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_perpendicular_overlap_is_symmetric_strict_and_non_crossing() {
        let a = Track {
            net: "NET_A".into(),
            a: [0, 0],
            b: [0, 100],
            width: 10,
        };
        let b = Track {
            net: "NET_A".into(),
            a: [24, 50],
            b: [36, 50],
            width: 42,
        };
        assert!(perpendicular_positive(&a, &b));
        assert!(perpendicular_positive(&b, &a));
        let tangent = Track {
            width: 38,
            ..b.clone()
        };
        assert!(!perpendicular_positive(&a, &tangent));
        let gap = Track {
            width: 36,
            ..b.clone()
        };
        assert!(!perpendicular_positive(&a, &gap));
        let corner = Track {
            a: [24, 111],
            b: [36, 111],
            ..b.clone()
        };
        assert!(Rect::trace(&a).touches(Rect::trace(&corner)));
        assert!(!perpendicular_positive(&a, &corner));
        let crossing = Track {
            a: [-5, 50],
            b: [5, 50],
            ..b.clone()
        };
        assert!(!perpendicular_positive(&a, &crossing));
    }
}
