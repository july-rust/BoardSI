//! D_047: bounded SOURCE-local positive-width trace to SMD square-pad contact.
//! Not whole-board connectivity or manufactured electrical continuity.
use crate::{d040_multi_component, digest, route_preview};
use std::fmt::Write;

fn stripped_source(bytes: &[u8]) -> Result<Vec<u8>, &'static str> {
    let text = std::str::from_utf8(bytes).map_err(|_| "D047_UTF8")?;
    let rows: Vec<_> = text.lines().collect();
    let mut base = String::new();
    let mut in_signal = false;
    let mut witness = 0;
    let mut i = 0;
    while i < rows.len() {
        let fields: Vec<_> = rows[i].split_ascii_whitespace().collect();
        if fields.first() == Some(&"*SIGNAL*") {
            in_signal = true;
        } else if fields.first() == Some(&"*END*") {
            in_signal = false;
        }
        if fields == ["U1.2", "U2.2"] {
            if !in_signal
                || witness != 0
                || i + 2 >= rows.len()
                || rows[i + 1].split_ascii_whitespace().count() != 5
                || rows[i + 2].split_ascii_whitespace().count() != 5
            {
                return Err("D047_SOURCE_INVENTORY");
            }
            witness += 1;
            i += 3;
        } else {
            base.push_str(rows[i]);
            base.push('\n');
            i += 1;
        }
    }
    if witness != 1 {
        return Err("D047_TRACE_MISSING");
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
        return Err("D047_BASE_TRACK");
    }
    let n = |i: usize| f[i].parse::<i64>().map_err(|_| "D047_BASE_VALUE");
    Ok(Track {
        net: f[3].to_owned(),
        a: [n(4)?, n(5)?],
        b: [n(6)?, n(7)?],
        width: n(8)?,
    })
}

fn same_pair(a: [i64; 2], b: [i64; 2], x: [i64; 2], y: [i64; 2]) -> bool {
    (a == x && b == y) || (a == y && b == x)
}

fn sub(a: [i128; 2], b: [i128; 2]) -> [i128; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn dot(a: [i128; 2], b: [i128; 2]) -> i128 {
    a[0] * b[0] + a[1] * b[1]
}
fn cross(a: [i128; 2], b: [i128; 2]) -> i128 {
    a[0] * b[1] - a[1] * b[0]
}
fn strict_point_segment(p: [i128; 2], a: [i128; 2], b: [i128; 2], width_sq: i128) -> bool {
    let d = sub(b, a);
    let w = sub(p, a);
    let length_sq = dot(d, d);
    if length_sq == 0 {
        return false;
    }
    let projection = dot(w, d);
    if projection <= 0 {
        return 4 * dot(w, w) < width_sq;
    }
    if projection >= length_sq {
        let tail = sub(p, b);
        return 4 * dot(tail, tail) < width_sq;
    }
    let area = cross(d, w);
    4 * area * area < width_sq * length_sq
}

/// The entire supported trace centerline is strictly to the right of the
/// bounded square pad. Hence nearest distance to the *square* equals nearest
/// distance to its right vertical edge. Both endpoint projections and both
/// square corners are compared exactly, including diagonal round-cap cases.
fn positive_square_contact(trace: &Track) -> bool {
    let side = 3_000_i128;
    let right = 20_000_i128 + side / 2;
    let edge0 = [right, 40_000 - side / 2];
    let edge1 = [right, 40_000 + side / 2];
    if trace.width != 800
        || trace.a == trace.b
        || [trace.a, trace.b].iter().any(|p| {
            !(21_600..=23_000).contains(&p[0])
                || !(38_000..=43_500).contains(&p[1])
                || i128::from(p[0]) <= right
        })
    {
        return false;
    }
    // Worst-case SOURCE tick products are <2^63; i128 is independently safe.
    let p = [i128::from(trace.a[0]), i128::from(trace.a[1])];
    let q = [i128::from(trace.b[0]), i128::from(trace.b[1])];
    let width_sq = i128::from(trace.width) * i128::from(trace.width);
    strict_point_segment(p, edge0, edge1, width_sq)
        || strict_point_segment(q, edge0, edge1, width_sq)
        || strict_point_segment(edge0, p, q, width_sq)
        || strict_point_segment(edge1, p, q, width_sq)
}

/// Qualifies exactly one SOURCE-owned 8-mil track contacting U1.2 square pad.
pub fn encode(bytes: &[u8]) -> Result<String, &'static str> {
    if bytes.is_empty() || bytes.len() > route_preview::MAX_FILE_BYTES {
        return Err("D047_SOURCE_SIZE");
    }
    let baseline = d040_multi_component::encode(&stripped_source(bytes)?)?;
    let old: Vec<Track> = baseline
        .lines()
        .filter(|line| line.starts_with("T|"))
        .map(parse_track)
        .collect::<Result<_, _>>()?;
    if old.len() != 4
        || !baseline
            .lines()
            .any(|line| line == "P|1|U1|2|1|20000|40000|3000|3000|S")
        || old.iter().any(|t| t.net != old[0].net || t.width != 1000)
    {
        return Err("D047_UNSUPPORTED_BASE");
    }
    let route = route_preview::parse(bytes)?;
    if route.skipped != 0
        || route.segments.len() != 5
        || !route.arcs.is_empty()
        || !route.vias.is_empty()
    {
        return Err("D047_ROUTE_INVENTORY");
    }
    let mut used = [false; 4];
    let mut extra: Option<Track> = None;
    for segment in &route.segments {
        if segment.layer != 1 || segment.net != old[0].net || segment.width_ticks <= 0 {
            return Err("D047_NET_LAYER_WIDTH");
        }
        if let Some((index, _)) = old.iter().enumerate().find(|(i, base)| {
            !used[*i]
                && base.net == segment.net
                && base.width == segment.width_ticks
                && same_pair(base.a, base.b, segment.a, segment.b)
        }) {
            used[index] = true;
        } else {
            if extra.is_some() {
                return Err("D047_DUPLICATE_OR_UNSUPPORTED");
            }
            extra = Some(Track {
                net: segment.net.clone(),
                a: segment.a,
                b: segment.b,
                width: segment.width_ticks,
            });
        }
    }
    if used.iter().any(|used| !used) {
        return Err("D047_BASE_TRACE_MISSING");
    }
    let trace = extra.ok_or("D047_TRACK_MISSING")?;
    if !positive_square_contact(&trace) {
        return Err("D047_GAP_BOUNDARY_OR_UNSUPPORTED");
    }
    let mut sha = String::new();
    for byte in digest::sha256(bytes) {
        write!(&mut sha, "{byte:02x}").map_err(|_| "D047_FORMAT")?;
    }
    let mut out = format!(
        "BOARDSI_D047_TRACE_SQUARE_PAD_WIDTH_V1|{sha}|1|SOURCE_LOCAL_TRACE_PAD_FINITE_WIDTH_ONLY|P009_UNSUPPORTED|SI_NOT_RUN|GLOBAL_ELECTRICAL_NOT_VERIFIED\n"
    );
    writeln!(&mut out, "C|0|U1|{}|1|5|4", old[0].net).map_err(|_| "D047_FORMAT")?;
    writeln!(&mut out, "C|1|U2|{}|1|4|3", old[0].net).map_err(|_| "D047_FORMAT")?;
    for row in baseline
        .lines()
        .filter(|line| line.starts_with("P|") || line.starts_with("T|"))
    {
        writeln!(&mut out, "{row}").map_err(|_| "D047_FORMAT")?;
    }
    writeln!(
        &mut out,
        "T|4|1|{}|{}|{}|{}|{}|{}",
        trace.net, trace.a[0], trace.a[1], trace.b[0], trace.b[1], trace.width
    )
    .map_err(|_| "D047_FORMAT")?;
    for row in baseline.lines().filter(|line| line.starts_with("E|")) {
        writeln!(&mut out, "{row}").map_err(|_| "D047_FORMAT")?;
    }
    out.push_str(
        "E|6|1|8|POSITIVE_FINITE_WIDTH_TRACE_PAD_CONTACT\n\
         COUNT|NODES|9|EDGES|7|LOCAL_COMPONENTS|2|GLOBAL_ELECTRICAL|NOT_VERIFIED\n\
         END|SOURCE_LOCAL_TRACE_PAD_FINITE_WIDTH_ONLY_NO_GLOBAL_PASS\n",
    );
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_side_and_corner_are_positive_but_tangent_is_not() {
        let sample = Track {
            net: "NET_A".to_owned(),
            a: [21_700, 39_000],
            b: [21_700, 41_000],
            width: 800,
        };
        assert!(positive_square_contact(&sample));
        assert!(positive_square_contact(&Track {
            a: sample.b,
            b: sample.a,
            ..sample.clone()
        }));
        assert!(positive_square_contact(&Track {
            a: [21_700, 41_700],
            b: [22_500, 42_500],
            ..sample.clone()
        }));
        for (a, b) in [
            ([21_900, 39_000], [21_900, 41_000]), // exact tangent
            ([22_000, 39_000], [22_000, 41_000]), // gap
            ([21_600, 41_900], [22_400, 42_700]), // AABB only; corner sqrt(1^2+4^2)>4 mil
            ([21_500, 39_000], [21_500, 41_000]), // centerline on pad boundary
        ] {
            assert!(!positive_square_contact(&Track {
                a,
                b,
                ..sample.clone()
            }));
        }
    }
}
