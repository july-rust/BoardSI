//! A_029 P2-T: bounded candidate for explicitly owned curved COPCLS.
//! Not activated in the production copper importer or Qt transport.
//! This preflight proves only the restricted source cardinal-arc topology and
//! conservative display proxy; it does not prove real copper or physical SI.
use crate::outline_arc::preflight_closed;
use crate::outline_preview::{bounded, fixed, valid_ring};
use crate::outline_topology::{self, Segment};

const MIN_ARC_RADIUS_TICKS: i64 = 1000; // 254 um; source is 254 nm per tick.
const ARC_STEPS: usize = 8; // <= 1% radial conservative preview bound.
const MAX_DISPLAY_VERTICES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurvedCopperCandidate {
    /// Source-exact line and cardinal-quarter-arc segments, not a polygon.
    pub source_segments: Vec<Segment>,
    /// Bounded, conservative visual proxy only, source-grid closed ring.
    pub display_ring: Vec<[i64; 2]>,
}

fn area2(ring: &[[i64; 2]]) -> i128 {
    ring.windows(2)
        .map(|p| {
            i128::from(p[0][0]) * i128::from(p[1][1]) - i128::from(p[1][0]) * i128::from(p[0][1])
        })
        .sum()
}

fn cross(a: [i64; 2], b: [i64; 2], c: [i64; 2]) -> i128 {
    let x = i128::from(b[0]) - i128::from(a[0]);
    let y = i128::from(b[1]) - i128::from(a[1]);
    x * (i128::from(c[1]) - i128::from(b[1])) - y * (i128::from(c[0]) - i128::from(b[0]))
}

fn convex_chords(ring: &[[i64; 2]]) -> bool {
    if ring.len() < 4 || ring.first() != ring.last() {
        return false;
    }
    let winding = area2(ring).signum();
    if winding == 0 {
        return false;
    }
    let n = ring.len() - 1;
    (0..n).all(|i| cross(ring[i], ring[(i + 1) % n], ring[(i + 2) % n]) * winding >= 0)
}

/// Strict source-owned COPCLS candidate (no holes).
///
/// PADS point rows are either x y or 8-field cardinal arc records. Reject
/// unsupported arc sweeps, concavity, overlap, invalid source closure and
/// excessive geometry. All source arcs must bulge outward from a convex ring.
/// This is a **preflight only**; wire/CPU/RHI parity is a separate gate.
pub fn validate(
    rows: &[Vec<&str>],
    origin: [i64; 2],
) -> Result<CurvedCopperCandidate, &'static str> {
    if rows.len() < 4 || rows.len() > 65 {
        return Err("COPPER_ARC_ROW_LIMIT");
    }
    if !rows.iter().any(|row| row.len() == 8) {
        return Err("COPPER_ARC_MISSING");
    }
    let arcs = preflight_closed(rows, origin)?;
    let mut vertices = Vec::with_capacity(rows.len());
    for row in rows {
        if row.len() != 2 && row.len() != 8 {
            return Err("COPPER_ARC_ROW");
        }
        let x = origin[0]
            .checked_add(fixed(row[0])?)
            .ok_or("COPPER_ARC_OVERFLOW")
            .and_then(bounded)?;
        let y = origin[1]
            .checked_add(fixed(row[1])?)
            .ok_or("COPPER_ARC_OVERFLOW")
            .and_then(bounded)?;
        vertices.push([x, y]);
    }
    if !convex_chords(&vertices) {
        return Err("COPPER_ARC_NONCONVEX");
    }
    let ccw = area2(&vertices) > 0;
    let mut segments = Vec::with_capacity(arcs.len());
    let mut display = vec![vertices[0]];
    for (i, arc) in arcs.iter().enumerate() {
        match arc {
            None => {
                segments.push(Segment::Line(vertices[i], vertices[i + 1]));
                display.push(vertices[i + 1]);
            }
            Some(a) => {
                // Outward convex arc: source winding and quarter sweep agree.
                if a.clockwise == ccw {
                    return Err("COPPER_ARC_NOT_OUTWARD");
                }
                let r = (a.start[0] - a.centre[0])
                    .abs()
                    .max((a.start[1] - a.centre[1]).abs());
                if r < MIN_ARC_RADIUS_TICKS {
                    return Err("COPPER_ARC_TOO_SMALL");
                }
                segments.push(Segment::Arc(*a));
                let theta =
                    ((a.start[1] - a.centre[1]) as f64).atan2((a.start[0] - a.centre[0]) as f64);
                let signed_quarter = if a.clockwise {
                    -std::f64::consts::FRAC_PI_2
                } else {
                    std::f64::consts::FRAC_PI_2
                };
                for step in 1..=ARC_STEPS {
                    let p = if step == ARC_STEPS {
                        a.end
                    } else {
                        let angle = theta + signed_quarter * step as f64 / ARC_STEPS as f64;
                        let inner_r = (r - 2) as f64;
                        let x = a.centre[0] + (inner_r * angle.cos()).round() as i64;
                        let y = a.centre[1] + (inner_r * angle.sin()).round() as i64;
                        let dx = i128::from(x) - i128::from(a.centre[0]);
                        let dy = i128::from(y) - i128::from(a.centre[1]);
                        if dx * dx + dy * dy >= i128::from(r) * i128::from(r) {
                            return Err("COPPER_ARC_PROXY_OUTSIDE");
                        }
                        [bounded(x)?, bounded(y)?]
                    };
                    if display.last() == Some(&p) {
                        return Err("COPPER_ARC_PROXY_DEGENERATE");
                    }
                    display.push(p);
                }
            }
        }
        if display.len() > MAX_DISPLAY_VERTICES {
            return Err("COPPER_ARC_PROXY_LIMIT");
        }
    }
    if !outline_topology::ring_valid(&segments) || !valid_ring(&display) || !convex_chords(&display)
    {
        return Err("COPPER_ARC_INVALID_TOPOLOGY");
    }
    // Conservative proxy radius: eight 11.25-degree chords per quarter arc.
    // Worst underfill <= 1-cos(pi/32) + 3/r < 0.8% at r>=1000 ticks.
    // This bound is geometric only; not an exporter, electrical or GPU proof.
    Ok(CurvedCopperCandidate {
        source_segments: segments,
        display_ring: display,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurvedSourceCopper {
    pub name: String,
    pub net_label: Option<String>, // Source label, NOT an electrical NetId.
    pub source_line: usize,
    pub layer: u32,
    pub candidate: CurvedCopperCandidate,
}

/// Audit one bounded explicit source-owned PADS V9.4 COPCLS object.
///
/// Unlike containment heuristics, ownership is determined solely by a
/// complete one-piece COPPER header and the immediately following COPCLS.
/// Deliberately isolated from the production copper parser until the Qt
/// transport and GPU are proven to consume the identical conservative mesh.
pub fn inspect_owned_source(
    bytes: &[u8],
    max_layer: u32,
) -> Result<CurvedSourceCopper, &'static str> {
    if bytes.len() > crate::route_preview::MAX_FILE_BYTES {
        return Err("INPUT_TOO_LARGE");
    }
    if !(1..=64).contains(&max_layer) {
        return Err("COPPER_LAYER_RANGE");
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "INVALID_COPPER_ENCODING")?;
    let mut lines = text.lines();
    let first = lines.next().ok_or("EMPTY_FILE")?;
    if !matches!(
        first.split_ascii_whitespace().next().unwrap_or(""),
        "!PADS-POWERPCB-V9.4-MILS!"
            | "!PADS-POWERPCB-V9.4-MILS-CP1252!"
            | "!PADS-POWERPCB-V9.4-MILS-250L!"
            | "!PADS-POWERPCB-V9.4-MILS-250L-CP1252!"
    ) {
        return Err("UNSUPPORTED_REVISION_OR_UNIT");
    }
    let mut in_lines = false;
    let mut end = false;
    let mut found = None;
    // Enumerate from 2 since the first physical line is the PADS header.
    let rows: Vec<&str> = lines.collect();
    let mut i = 0;
    while i < rows.len() {
        let raw = rows[i];
        if raw.len() > 16_384 {
            return Err("OVERLONG_COPPER_LINE");
        }
        let p: Vec<&str> = raw.split_ascii_whitespace().collect();
        if p.is_empty() || matches!(p[0], "REMARK" | "*REMARK*") {
            i += 1;
            continue;
        }
        if p[0] == "*END*" {
            end = true;
            break;
        }
        if p[0].starts_with('*') {
            if p[0] == "*GET*" && in_lines {
                return Err("UNSUPPORTED_COPPER_REUSE");
            }
            in_lines = p[0] == "*LINES*";
            i += 1;
            continue;
        }
        if !in_lines {
            i += 1;
            continue;
        }
        if p[0] == "COPCUT" || p[0] == "COPCCO" || (p.len() >= 2 && p[1] == "COPCUT") {
            return Err("ORPHAN_COPPER_CUTOUT");
        }
        if p.len() < 2 || p[1] != "COPPER" {
            i += 1;
            continue;
        }
        if found.is_some()
            || !matches!(p.len(), 6 | 8)
            || p[0].is_empty()
            || p[0].len() > 16
            || !p[0].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            || p[4] != "1"
            || p[5] != "1"
            || (p.len() == 8
                && (p[6] != "0"
                    || p[7].is_empty()
                    || p[7].len() > 47
                    || !p[7].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')))
        {
            return Err("UNSUPPORTED_CURVED_COPPER_OWNER");
        }
        let origin = [bounded(fixed(p[2])?)?, bounded(fixed(p[3])?)?];
        let source_line = i + 2;
        i += 1;
        let piece: Vec<&str> = rows
            .get(i)
            .ok_or("TRUNCATED_CURVED_COPPER")?
            .split_ascii_whitespace()
            .collect();
        if piece.len() != 5
            || piece[0] != "COPCLS"
            || piece[3] != "0"
            || !(1..=25_000).contains(&fixed(piece[2])?)
        {
            return Err("UNSUPPORTED_CURVED_COPPER_PIECE");
        }
        let count = piece[1]
            .parse::<usize>()
            .map_err(|_| "COPPER_CORNER_COUNT")?;
        if !(4..=65).contains(&count) {
            return Err("COPPER_ARC_ROW_LIMIT");
        }
        let layer = piece[4].parse::<u32>().map_err(|_| "COPPER_LAYER_RANGE")?;
        if layer == 0 || layer > max_layer {
            return Err("COPPER_LAYER_RANGE");
        }
        i += 1;
        if rows.len() - i < count {
            return Err("TRUNCATED_CURVED_COPPER");
        }
        let corners: Vec<Vec<&str>> = rows[i..i + count]
            .iter()
            .map(|row| row.split_ascii_whitespace().collect())
            .collect();
        if !corners.iter().any(|row| row.len() == 8) {
            return Err("COPPER_ARC_MISSING");
        }
        let candidate = validate(&corners, origin)?;
        found = Some(CurvedSourceCopper {
            name: p[0].to_owned(),
            net_label: p.get(7).map(|s| (*s).to_owned()),
            source_line,
            layer,
            candidate,
        });
        i += count;
    }
    if !end {
        return Err("MISSING_END_MARKER");
    }
    found.ok_or("CURVED_COPPER_NOT_FOUND")
}

impl CurvedCopperCandidate {
    /// Exact analytical preflight model. Never derives boundary geometry
    /// from the proxy; may only enter production after full viewer gating.
    pub fn analytical_boundary(&self, frame: &boardsi_model::ObjectId) -> boardsi_model::Boundary {
        use boardsi_model::{Boundary, BoundarySegment, Point};
        let pt = |ticks| Point {
            frame: frame.clone(),
            ticks,
        };
        let segments = self
            .source_segments
            .iter()
            .map(|segment| match *segment {
                Segment::Line(start, end) => BoundarySegment::Line {
                    start: pt(start),
                    end: pt(end),
                },
                Segment::Arc(a) => BoundarySegment::Arc {
                    centre: pt(a.centre),
                    start: pt(a.start),
                    end: pt(a.end),
                    clockwise: a.clockwise,
                },
            })
            .collect();
        Boundary { segments }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROUNDED: &str =
        "100 0\n300 0 2700 900 200 0 400 200\n400 100\n400 200 0 900 200 100 400 300\n300 300\n100 300 900 900 0 100 200 300\n0 200\n0 100 1800 900 0 0 200 200\n100 0";

    fn check(source: &str) -> Result<CurvedCopperCandidate, &'static str> {
        let rows: Vec<Vec<&str>> = source
            .lines()
            .map(|line| line.split_ascii_whitespace().collect())
            .collect();
        validate(&rows, [0, 0])
    }

    #[test]
    fn rounded_rectangle_retains_four_exact_arcs_and_conservative_proxy() {
        let c = check(ROUNDED).unwrap();
        assert_eq!(c.source_segments.len(), 8);
        assert_eq!(
            c.source_segments
                .iter()
                .filter(|s| matches!(s, Segment::Arc(_)))
                .count(),
            4
        );
        assert_eq!(c.display_ring.len(), 37);
        assert_eq!(c.display_ring.first(), c.display_ring.last());
        assert!(valid_ring(&c.display_ring));
        // Source arcs are never replaced by the display chord proxies.
        assert!(matches!(c.source_segments[1], Segment::Arc(_)));
    }

    #[test]
    fn rejects_unsupported_angle_and_broken_endpoint() {
        assert!(check(&ROUNDED.replace("2700 900", "2700 450")).is_err());
        assert!(check(&ROUNDED.replace("400 100\n", "401 100\n")).is_err());
        assert!(check(&ROUNDED.replace("400 200 0 900", "400 200 90 900")).is_err());
        assert!(check(&ROUNDED.replace("100 0\n300", "100 1\n300")).is_err());
    }

    #[test]
    fn refuses_bow_tie_wrong_sweep_and_unbounded_source() {
        assert!(check(&ROUNDED.replace("300 300\n100 300", "300 300\n100 -300")).is_err());
        assert!(check(&ROUNDED.replace("2700 900", "2700 -900")).is_err());
        assert!(validate(
            &ROUNDED
                .lines()
                .map(|row| row.split_ascii_whitespace().collect())
                .collect::<Vec<Vec<&str>>>(),
            [5_590_000, 0]
        )
        .is_err());
    }

    #[test]
    fn no_arc_is_not_misrepresented_as_a_curved_source() {
        assert_eq!(
            check("0 0\n100 0\n100 100\n0 100\n0 0").unwrap_err(),
            "COPPER_ARC_MISSING"
        );
    }

    #[test]
    fn explicit_pads_owner_is_required_for_curved_candidate() {
        let file = format!(
            "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*LINES*\nCURVED COPPER 0 0 1 1 0 GND\nCOPCLS 9 0.01 0 2\n{}\n*END*\n",
            ROUNDED
        );
        let obj = inspect_owned_source(file.as_bytes(), 4).unwrap();
        assert_eq!(obj.name, "CURVED");
        assert_eq!(obj.net_label.as_deref(), Some("GND"));
        assert_eq!(obj.layer, 2);
        assert_eq!(obj.source_line, 5);
        assert_eq!(obj.candidate.source_segments.len(), 8);
        assert!(
            inspect_owned_source(file.replace("0 0 1 1 0 GND", "0 0 2 1 0 GND").as_bytes(), 4)
                .is_err()
        );
        assert!(inspect_owned_source(file.replace("COPCLS 9", "COPCUT 9").as_bytes(), 4).is_err());
        assert!(inspect_owned_source(
            file.replace("*END*", "COPCUT 5 0.01 0 2\n*END*").as_bytes(),
            4
        )
        .is_err());
    }

    #[test]
    fn analytical_boundary_does_not_promote_visual_proxy_to_source_geometry() {
        let source = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n100 200 1 10 0\n300 200 1 10 0\n*END*\n";
        let parsed = crate::route_preview::parse(source.as_bytes()).unwrap();
        let board = crate::route_preview::into_canonical(source.as_bytes(), &parsed).unwrap();
        let candidate = check(ROUNDED).unwrap();
        let boundary = candidate.analytical_boundary(&board.frames[0].meta.id);
        assert_eq!(boundary.segments.len(), 8);
        assert_eq!(
            boundary
                .segments
                .iter()
                .filter(|s| matches!(s, boardsi_model::BoundarySegment::Arc { .. }))
                .count(),
            4
        );
        assert_ne!(boundary.segments.len(), candidate.display_ring.len() - 1);
    }

    #[test]
    fn public_pads_fixture_retains_analytic_source_and_net_uncertainty() {
        let asc = include_bytes!("../tests/fixtures/pads_curved_copper_candidate_v94.asc");
        let shape = inspect_owned_source(asc, 4).unwrap();
        assert_eq!(shape.name, "COP_ARC");
        assert_eq!(shape.net_label.as_deref(), Some("NET_UNVERIFIED"));
        assert_eq!(shape.candidate.display_ring.len(), 37);
        // Production import preserves exact analytical source arcs separately
        // from the conservative visualization-only ring.
        let imported = crate::copper_preview::parse(asc, 4).unwrap();
        assert_eq!(imported.len(), 1);
        assert!(matches!(
            imported[0].geometry,
            crate::copper_preview::CopperGeometry::Curved { .. }
        ));
    }
}
