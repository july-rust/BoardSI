//! A_025 P2-P: bounded *LINES* explicit source-owned straight polygon COPCUT holes.
//! A_022 standalone COPCLS/COPCIR geometry remains backward compatible.
//! This records explicit filled COPCLS/COPCIR only; it is not flood,
//! electrical contact, physical stackup, or a verified fabricated conductor.
use crate::outline_preview::{bounded, fixed, orthogonal_copper_holes_valid, valid_ring};
use std::collections::BTreeSet;

const MAX_COPPER: usize = 64;
const MAX_VERTICES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopperGeometry {
    Polygon(Vec<[i64; 2]>), // Exact 0.01-mil ticks; repeated closure vertex.
    Circle {
        centre: [i64; 2],
        radius: i64,
    },
    Curved {
        segments: Vec<crate::outline_topology::Segment>,
        display_ring: Vec<[i64; 2]>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopperArea {
    pub name: String,
    pub net_label: Option<String>, // Source association, NOT connectivity.
    pub layer: u32,
    pub source_line: usize,
    pub geometry: CopperGeometry,
    /// Same named COPPER object's subsequent COPCUT pieces, never standalone pour voids.
    pub cutouts: Vec<CopperGeometry>,
}
fn token(s: &str, max: usize) -> bool {
    !s.is_empty() && s.len() <= max && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
fn point(row: &str, origin: [i64; 2]) -> Result<[i64; 2], &'static str> {
    if row.len() > 16_384 {
        return Err("OVERLONG_COPPER_LINE");
    }
    let items: Vec<_> = row.split_ascii_whitespace().collect();
    if items.len() != 2 {
        return Err("UNSUPPORTED_COPPER_ARC");
    }
    let x = origin[0]
        .checked_add(fixed(items[0])?)
        .ok_or("COPPER_OVERFLOW")?;
    let y = origin[1]
        .checked_add(fixed(items[1])?)
        .ok_or("COPPER_OVERFLOW")?;
    Ok([bounded(x)?, bounded(y)?])
}
// Deliberately restricted initial void support: five-point axis-aligned rectangles.
// Exact closed polygon, not a bounding-box approximation of an arbitrary polygon.
pub fn rectangle_bounds(ring: &[[i64; 2]]) -> Option<[i64; 4]> {
    if ring.len() != 5 || ring[0] != ring[4] || !valid_ring(ring) {
        return None;
    }
    let x0 = ring.iter().map(|v| v[0]).min()?;
    let y0 = ring.iter().map(|v| v[1]).min()?;
    let x1 = ring.iter().map(|v| v[0]).max()?;
    let y1 = ring.iter().map(|v| v[1]).max()?;
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let mut expected = std::collections::BTreeSet::from([[x0, y0], [x0, y1], [x1, y0], [x1, y1]]);
    for v in &ring[..4] {
        if !expected.remove(v) {
            return None;
        }
    }
    if !expected.is_empty() {
        return None;
    }
    Some([x0, y0, x1, y1])
}
/// Fail closed on ambiguous ownership: a COPCUT only subtracts from the COPPER
/// record whose explicit multi-piece header encloses it. Standalone COPCUT is
/// an unassociated pour void and is rejected, not inferred by containment.
pub fn parse(bytes: &[u8], max_layer: u32) -> Result<Vec<CopperArea>, &'static str> {
    // A_027: the V12 G|C plus explicitly owned J|index circle records
    // preserve a hollow COPCIR outer without inventing electrical connectivity.
    // Exact source-tick containment/overlap checks remain in parse_impl.
    // Qt A_027 admits the shape only if its conservative CPU/RHI mesh passes.
    parse_impl(bytes, max_layer, true)
}

fn parse_impl(
    bytes: &[u8],
    max_layer: u32,
    candidate_circles: bool,
) -> Result<Vec<CopperArea>, &'static str> {
    if bytes.len() > crate::route_preview::MAX_FILE_BYTES {
        return Err("INPUT_TOO_LARGE");
    }
    if !(1..=64).contains(&max_layer) {
        return Err("COPPER_LAYER_RANGE");
    }
    let first = bytes.split(|&b| b == b'\n').next().ok_or("EMPTY_FILE")?;
    let header = std::str::from_utf8(first).map_err(|_| "INVALID_HEADER_ENCODING")?;
    if !matches!(
        header.split_ascii_whitespace().next().unwrap_or(""),
        "!PADS-POWERPCB-V9.4-MILS!"
            | "!PADS-POWERPCB-V9.4-MILS-CP1252!"
            | "!PADS-POWERPCB-V9.4-MILS-250L!"
            | "!PADS-POWERPCB-V9.4-MILS-250L-CP1252!"
    ) {
        return Err("UNSUPPORTED_REVISION_OR_UNIT");
    }
    let source = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = source.lines().collect();
    let mut inside = false;
    let mut ended = false;
    let mut found = Vec::new();
    let mut names = BTreeSet::new();
    let mut i = 1;
    while i < lines.len() {
        if lines[i].len() > 16_384 {
            return Err("OVERLONG_COPPER_LINE");
        }
        let fields: Vec<_> = lines[i].split_ascii_whitespace().collect();
        if fields.is_empty() || matches!(fields[0], "*REMARK*" | "REMARK") {
            i += 1;
            continue;
        }
        if fields[0] == "*END*" {
            ended = true;
            break;
        }
        if inside && fields[0] == "*GET*" {
            return Err("UNSUPPORTED_COPPER_REUSE");
        }
        if fields[0].starts_with('*') {
            inside = fields[0] == "*LINES*";
            i += 1;
            continue;
        }
        if !inside {
            i += 1;
            continue;
        }
        // A standalone void can subtract from visible copper: fail closed.
        // No silent render that hides COPCUT/COPCCO under a filled shape.
        if fields.len() >= 2 && fields[1] == "COPCUT" {
            return Err("UNSUPPORTED_COPPER_VOID");
        }
        // A declared multi-piece count that understates a following COPCUT
        // must not silently re-fill the omitted void. Reject orphan piece
        // headers even when the line-item header has already been consumed.
        if fields.len() == 5
            && matches!(
                fields[0],
                "COPCUT" | "COPCCO" | "CIRCUR" | "COPCLS" | "COPCIR" | "COPOPN"
            )
        {
            return Err("UNCONSUMED_COPPER_PIECE");
        }
        if fields.len() < 2 || fields[1] != "COPPER" {
            i += 1;
            continue;
        }
        // name COPPER x y pieces 1 [0 net]. A single explicit owner
        // contains its filled body first, followed by bounded rectangular
        // COPCUT pieces. Never infer ownership from spatial containment.
        if !matches!(fields.len(), 6 | 8)
            || !token(fields[0], 16)
            || fields[5] != "1"
            || (fields.len() == 8 && (fields[6] != "0" || !token(fields[7], 47)))
        {
            return Err("UNSUPPORTED_COPPER_HEADER");
        }
        let piece_count = fields[4]
            .parse::<usize>()
            .map_err(|_| "UNSUPPORTED_COPPER_HEADER")?;
        if !(1..=9).contains(&piece_count) {
            return Err("COPPER_PIECE_LIMIT");
        }
        if found.len() >= MAX_COPPER || !names.insert(fields[0].to_owned()) {
            return Err("COPPER_COUNT_OR_DUPLICATE");
        }
        let source_line = i + 1;
        let origin = [bounded(fixed(fields[2])?)?, bounded(fixed(fields[3])?)?];
        let name = fields[0].to_owned();
        let net_label = if fields.len() == 8 {
            Some(fields[7].to_owned())
        } else {
            None
        };
        i += 1;
        let mut primary = None;
        let mut cutouts = Vec::new();
        let mut parent_layer = 0;
        for part_index in 0..piece_count {
            let piece: Vec<_> = lines
                .get(i)
                .ok_or("TRUNCATED_COPPER")?
                .split_ascii_whitespace()
                .collect();
            if piece.len() != 5 || piece[3] != "0" {
                return Err("UNSUPPORTED_COPPER_PIECE");
            }
            if part_index == 0 {
                if !matches!(piece[0], "COPCLS" | "COPCIR") {
                    return Err("UNSUPPORTED_COPPER_PIECE");
                }
            } else if piece[0] != "COPCUT" && !(candidate_circles && piece[0] == "COPCCO") {
                // Production COPCCO still fails closed until Qt parity.
                return Err("UNSUPPORTED_COPPER_CUTOUT_PIECE");
            }
            let n = piece[1]
                .parse::<usize>()
                .map_err(|_| "COPPER_CORNER_COUNT")?;
            if (matches!(piece[0], "COPCIR" | "COPCCO") && n != 2)
                || (piece[0] == "COPCLS" && !(4..=MAX_VERTICES).contains(&n))
                || (piece[0] == "COPCUT" && !(4..=65).contains(&n))
            {
                return Err("COPPER_CORNER_COUNT");
            }
            let width = fixed(piece[2])?;
            if !(1..=25_000).contains(&width) {
                return Err("COPPER_WIDTH_RANGE");
            }
            let layer = piece[4].parse::<u32>().map_err(|_| "COPPER_LAYER_RANGE")?;
            if !(1..=max_layer).contains(&layer) {
                return Err("COPPER_LAYER_RANGE");
            }
            if part_index == 0 {
                parent_layer = layer;
            } else if layer != parent_layer {
                return Err("COPPER_CUTOUT_LAYER_MISMATCH");
            }
            i += 1;
            let mut vertices = Vec::with_capacity(n);
            let mut source_rows = Vec::with_capacity(n);
            for _ in 0..n {
                let row = lines.get(i).ok_or("TRUNCATED_COPPER")?;
                let fields: Vec<&str> = row.split_ascii_whitespace().collect();
                if fields.len() == 2 {
                    vertices.push(point(row, origin)?);
                } else if fields.len() == 8 && part_index == 0 && piece[0] == "COPCLS" {
                    vertices.push([
                        bounded(
                            origin[0]
                                .checked_add(fixed(fields[0])?)
                                .ok_or("COPPER_OVERFLOW")?,
                        )?,
                        bounded(
                            origin[1]
                                .checked_add(fixed(fields[1])?)
                                .ok_or("COPPER_OVERFLOW")?,
                        )?,
                    ]);
                } else {
                    return Err("UNSUPPORTED_COPPER_ARC");
                }
                source_rows.push(fields);
                i += 1;
            }
            let curved = source_rows.iter().any(|row| row.len() == 8);
            let geometry = if curved {
                if part_index != 0 || piece_count > 2 {
                    return Err("UNSUPPORTED_CURVED_COPPER_CUTOUT");
                }
                let candidate = crate::copper_arc_preflight::validate(&source_rows, origin)?;
                CopperGeometry::Curved {
                    segments: candidate.source_segments,
                    display_ring: candidate.display_ring,
                }
            } else if matches!(piece[0], "COPCIR" | "COPCCO") {
                let a = vertices[0];
                let b = vertices[1];
                if (a[0] == b[0]) == (a[1] == b[1]) {
                    return Err("UNSUPPORTED_COPPER_CIRCLE_DIAMETER");
                }
                let diameter = (b[0] - a[0]).abs().max((b[1] - a[1]).abs());
                if diameter < 2 || diameter % 2 != 0 {
                    return Err("UNSUPPORTED_COPPER_CIRCLE_GRID");
                }
                let centre = [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2];
                let radius = diameter / 2;
                for c in [
                    [centre[0] - radius, centre[1]],
                    [centre[0] + radius, centre[1]],
                    [centre[0], centre[1] - radius],
                    [centre[0], centre[1] + radius],
                ] {
                    bounded(c[0])?;
                    bounded(c[1])?;
                }
                CopperGeometry::Circle { centre, radius }
            } else {
                if !valid_ring(&vertices) {
                    return Err("INVALID_COPPER_RING");
                }
                // Exact closed rings are checked here; orthogonal topology,
                // strictly interior ownership and grid complexity below.
                // Diagonal/intersecting/hollow unsupported cases fail closed.
                CopperGeometry::Polygon(vertices)
            };
            if part_index == 0 {
                primary = Some(geometry);
            } else {
                if let CopperGeometry::Circle { radius, .. } = &geometry {
                    // Qt's conservative 32-edge bounded proxy is qualified
                    // only for source circles >=1000 source ticks.
                    if *radius < 1000 {
                        return Err("UNSUPPORTED_SMALL_CIRCULAR_VOID");
                    }
                }
                cutouts.push(geometry);
            }
        }
        let geometry = primary.ok_or("TRUNCATED_COPPER")?;
        if !cutouts.is_empty() {
            let polygons: Vec<Vec<[i64; 2]>> = cutouts
                .iter()
                .filter_map(|hole| match hole {
                    CopperGeometry::Polygon(vertices) => Some(vertices.clone()),
                    CopperGeometry::Circle { .. } | CopperGeometry::Curved { .. } => None,
                })
                .collect();
            let circles: Vec<([i64; 2], i64)> = cutouts
                .iter()
                .filter_map(|hole| match hole {
                    CopperGeometry::Polygon(_) | CopperGeometry::Curved { .. } => None,
                    CopperGeometry::Circle { centre, radius } => Some((*centre, *radius)),
                })
                .collect();
            if !circles.is_empty() {
                let total = match &geometry {
                    CopperGeometry::Polygon(outer) => outer.len(),
                    CopperGeometry::Circle { .. } => 33,
                    CopperGeometry::Curved { .. } => {
                        return Err("UNSUPPORTED_CURVED_COPPER_CUTOUT")
                    }
                } + polygons.iter().map(Vec::len).sum::<usize>()
                    + circles.len() * 33;
                if total > 128 {
                    return Err("COPPER_CIRCLE_PROXY_COMPLEXITY");
                }
            }
            match &geometry {
                CopperGeometry::Polygon(outer) => {
                    let diagonal = std::iter::once(outer.as_slice())
                        .chain(polygons.iter().map(Vec::as_slice))
                        .any(|ring| {
                            ring.windows(2)
                                .any(|edge| edge[0][0] != edge[1][0] && edge[0][1] != edge[1][1])
                        });
                    if diagonal {
                        crate::polygon_topology::validate_polygon_with_holes(outer, &polygons)
                            .map_err(|_| "UNSUPPORTED_COPPER_CUTOUT_TOPOLOGY")?;
                    } else if !polygons.is_empty()
                        && !orthogonal_copper_holes_valid(outer, &polygons)
                    {
                        return Err("UNSUPPORTED_COPPER_CUTOUT_TOPOLOGY");
                    }
                    if !circles.is_empty() {
                        // An otherwise untested source ring must first pass the
                        // exact general polygon oracle before circle clearance.
                        crate::polygon_topology::validate_polygon_with_holes(outer, &polygons)
                            .map_err(|_| "UNSUPPORTED_COPPER_CUTOUT_TOPOLOGY")?;
                        for &(centre, radius) in &circles {
                            if !crate::circle_cutout_topology::circle_strictly_in_polygon(
                                outer, centre, radius,
                            ) {
                                return Err("UNSUPPORTED_COPPER_CUTOUT_TOPOLOGY");
                            }
                            for hole in &polygons {
                                if !crate::circle_cutout_topology::circle_strictly_disjoint_from_polygon(
                                    hole, centre, radius,
                                ) {
                                    return Err("UNSUPPORTED_COPPER_CUTOUT_TOPOLOGY");
                                }
                            }
                        }
                    }
                }
                CopperGeometry::Curved { display_ring, .. } => {
                    // Only one explicit same-owner straight convex COPCUT.
                    // A_029 has already proved the analytical source arc and
                    // conservative display; strict convex containment certifies
                    // the entire hole remains clear of its display proxy.
                    if cutouts.len() != 1 || polygons.len() != 1 || !circles.is_empty() {
                        return Err("UNSUPPORTED_CURVED_COPPER_CUTOUT");
                    }
                    crate::curved_cutout_topology::validate(display_ring, &polygons[0])?;
                }
                CopperGeometry::Circle { centre, radius } => {
                    // A_028: a closed, simple, same-owner COPCUT polygon is
                    // strictly enclosed when every vertex lies within the
                    // analytic circle. Disjointness is independently proven.
                    if !polygons.is_empty() {
                        crate::polygon_topology::validate_disjoint_polygon_holes(&polygons)
                            .map_err(|_| "UNSUPPORTED_COPPER_CUTOUT_TOPOLOGY")?;
                        for hole in &polygons {
                            if !crate::circle_cutout_topology::polygon_strictly_in_circle(
                                hole, *centre, *radius,
                            ) {
                                return Err("UNSUPPORTED_COPPER_CUTOUT_TOPOLOGY");
                            }
                            for &(c, r) in &circles {
                                if !crate::circle_cutout_topology::circle_strictly_disjoint_from_polygon(
                                    hole, c, r,
                                ) {
                                    return Err("UNSUPPORTED_COPPER_CUTOUT_TOPOLOGY");
                                }
                            }
                        }
                    }
                    for &(c, r) in &circles {
                        if !crate::circle_cutout_topology::circle_strictly_in_circle(
                            *centre, *radius, c, r,
                        ) {
                            return Err("UNSUPPORTED_COPPER_CUTOUT_TOPOLOGY");
                        }
                    }
                }
            }
            for first in 0..circles.len() {
                for second in first + 1..circles.len() {
                    let (a, ar) = circles[first];
                    let (b, br) = circles[second];
                    if !crate::circle_cutout_topology::circles_strictly_disjoint(a, ar, b, br) {
                        return Err("UNSUPPORTED_COPPER_CUTOUT_TOPOLOGY");
                    }
                }
            }
        }
        found.push(CopperArea {
            name,
            net_label,
            layer: parent_layer,
            source_line,
            geometry,
            cutouts,
        });
    }
    if !ended {
        return Err("MISSING_END_MARKER");
    }
    Ok(found)
}

/// Append exact-grid source-filled geometry to the canonical artwork model.
/// A claimed PADS net label is NOT promoted to an electrical Net ID.
pub fn augment(
    board: &mut boardsi_model::BoardModel,
    areas: &[CopperArea],
) -> Result<(), &'static str> {
    use boardsi_model::{
        Boundary, BoundarySegment, CopperArea as ModelCopperArea, Cutout as ModelCutout,
        GeometryProvenance, ObjectId, ObjectKind, ObjectMeta, Point,
    };
    if areas.is_empty() {
        return Ok(());
    }
    let source = board.sources.first().ok_or("MISSING_SOURCE")?.id.clone();
    let frame = board.frames.first().ok_or("MISSING_FRAME")?.meta.id.clone();
    let point = |ticks: [i64; 2]| Point {
        frame: frame.clone(),
        ticks,
    };
    for a in areas {
        let layer_index = (a.layer as usize)
            .checked_sub(1)
            .ok_or("COPPER_LAYER_RANGE")?;
        let layer = board
            .stackup
            .layers
            .get(layer_index)
            .ok_or("COPPER_LAYER_RANGE")?
            .meta
            .id
            .clone();
        let vertices = match &a.geometry {
            CopperGeometry::Polygon(v) => v.clone(),
            CopperGeometry::Curved { display_ring, .. } => display_ring.clone(),
            CopperGeometry::Circle { centre, radius } => {
                let [x, y] = *centre;
                let r = *radius;
                vec![[x + r, y], [x, y + r], [x - r, y], [x, y - r], [x + r, y]]
            }
        };
        let segments = match &a.geometry {
            CopperGeometry::Circle { centre, .. } => vertices
                .windows(2)
                .map(|pair| BoundarySegment::Arc {
                    centre: point(*centre),
                    start: point(pair[0]),
                    end: point(pair[1]),
                    clockwise: false,
                })
                .collect(),
            CopperGeometry::Curved { segments, .. } => segments
                .iter()
                .map(|segment| match *segment {
                    crate::outline_topology::Segment::Line(start, end) => BoundarySegment::Line {
                        start: point(start),
                        end: point(end),
                    },
                    crate::outline_topology::Segment::Arc(a) => BoundarySegment::Arc {
                        centre: point(a.centre),
                        start: point(a.start),
                        end: point(a.end),
                        clockwise: a.clockwise,
                    },
                })
                .collect(),
            CopperGeometry::Polygon(_) => vertices
                .windows(2)
                .map(|pair| BoundarySegment::Line {
                    start: point(pair[0]),
                    end: point(pair[1]),
                })
                .collect(),
        };
        board.areas.push(ModelCopperArea {
            meta: ObjectMeta {
                id: ObjectId::new(
                    &source,
                    ObjectKind::Area,
                    &format!("source-copper-{}-{}", a.name, a.source_line),
                    "",
                ),
                provenance: GeometryProvenance {
                    source_object_type: if matches!(&a.geometry, CopperGeometry::Curved { .. }) {
                        "PADS V9.4 MILS explicitly owned curved COPCLS exact analytical quarter-arcs; conservative preview"
                    } else {
                        "PADS V9.4 MILS *LINES* explicit COPCLS/COPCIR source-filled (restricted)"
                    }.into(),
                    history: vec![],
                },
            },
            layer,
            net: None, // Explicitly unverified electrical ownership.
            outer: Boundary { segments },
            cutouts: a
                .cutouts
                .iter()
                .enumerate()
                .map(|(index, cut)| {
                    let segments = match cut {
                        CopperGeometry::Curved { .. } => unreachable!("curved void refused"),
                        CopperGeometry::Polygon(vertices) => vertices
                            .windows(2)
                            .map(|pair| BoundarySegment::Line {
                                start: point(pair[0]),
                                end: point(pair[1]),
                            })
                            .collect(),
                        CopperGeometry::Circle { centre, radius } => {
                            let [x, y] = *centre;
                            let r = *radius;
                            let cardinal = [
                                [x + r, y],
                                [x, y + r],
                                [x - r, y],
                                [x, y - r],
                                [x + r, y],
                            ];
                            cardinal
                                .windows(2)
                                .map(|pair| BoundarySegment::Arc {
                                    centre: point(*centre),
                                    start: point(pair[0]),
                                    end: point(pair[1]),
                                    clockwise: false,
                                })
                                .collect()
                        }
                    };
                    ModelCutout {
                        meta: ObjectMeta {
                            id: ObjectId::new(
                                &source,
                                ObjectKind::Cutout,
                                &format!(
                                    "source-copper-{}-{}-cutout-{}",
                                    a.name, a.source_line, index
                                ),
                                "",
                            ),
                            provenance: GeometryProvenance {
                                source_object_type: match cut {
                                    CopperGeometry::Polygon(_) => "PADS V9.4 MILS same-COPPER COPCUT bounded straight polygonal source void",
                                    CopperGeometry::Circle { .. } => "PADS V9.4 MILS same-COPPER COPCCO analytical circular source void",
                                    CopperGeometry::Curved { .. } => "UNSUPPORTED_CURVED_COPPER_CUTOUT",
                                }
                                .into(),
                                history: vec![],
                            },
                        },
                        boundary: Boundary { segments },
                    }
                })
                .collect(),
        });
    }
    board.sources[0].capability.object_coverage.push(
        "PADS source COPCLS/COPCIR plus strictly bounded combined polygonal COPCUT; net/contact NOT_VERIFIED".into(),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE:&str="!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*LINES*\nGND_ZONE COPPER 100 200 1 1 0 GND\nCOPCLS 5 0.01 0 2\n0 0\n10 0\n10 20\n0 20\n0 0\n*END*\n";
    #[test]
    fn polygon_grid_and_source_net() {
        let a = parse(SAMPLE.as_bytes(), 4).unwrap();
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].source_line, 5);
        assert_eq!(a[0].net_label.as_deref(), Some("GND"));
        assert_eq!(
            a[0].geometry,
            CopperGeometry::Polygon(vec![
                [10000, 20000],
                [11000, 20000],
                [11000, 22000],
                [10000, 22000],
                [10000, 20000]
            ])
        );
    }
    #[test]
    fn circle_only_no_net_and_literal_net_are_distinct() {
        let circle = SAMPLE
            .replace(
                "GND_ZONE COPPER 100 200 1 1 0 GND",
                "GND_ZONE COPPER 100 200 1 1",
            )
            .replace(
                "COPCLS 5 0.01 0 2\n0 0\n10 0\n10 20\n0 20\n0 0",
                "COPCIR 2 0.01 0 2\n0 0\n20 0",
            );
        let a = parse(circle.as_bytes(), 4).unwrap();
        assert_eq!(a[0].net_label, None);
        assert_eq!(
            a[0].geometry,
            CopperGeometry::Circle {
                centre: [11000, 20000],
                radius: 1000
            }
        );
        let literal = parse(
            SAMPLE.replace(" 0 GND", " 0 NET_NOT_DECLARED").as_bytes(),
            4,
        )
        .unwrap();
        assert_eq!(literal[0].net_label.as_deref(), Some("NET_NOT_DECLARED"));
    }
    #[test]
    fn refuses_void_multifill_arc_and_invalid_ring() {
        for sample in [
            SAMPLE.replace("COPCLS", "COPCUT"),
            SAMPLE.replace("*END*", "CUTZONE COPCUT 0 0 1 0\n*END*"),
            SAMPLE.replace("1 1 0 GND", "2 1 0 GND"),
            SAMPLE.replace("COPCLS 5", "COPCLS 4"),
            SAMPLE.replace("0 20\n0 0", "0 20\n0 1"),
            SAMPLE.replace("10 0\n10 20\n0 20", "10 20\n10 0\n0 20"),
            SAMPLE.replace("10 0", "10 0 90 0 0 0 0 0"),
        ] {
            assert!(parse(sample.as_bytes(), 4).is_err(), "{sample}");
        }
    }
    #[test]
    fn accepts_explicit_combined_rectangle_cutout_with_canonical_provenance() {
        let source = SAMPLE.replace("1 1 0 GND", "2 1 0 GND").replace(
            "0 20\n0 0\n*END*",
            "0 20\n0 0\nCOPCUT 5 0.01 0 2\n2 2\n4 2\n4 6\n2 6\n2 2\n*END*",
        );
        let a = parse(source.as_bytes(), 4).unwrap();
        // Misstated 1-piece owner must refuse the subsequent source COPCUT.
        // Otherwise copper would be silently filled across a real void.
        assert!(parse(source.replace("2 1 0 GND", "1 1 0 GND").as_bytes(), 4).is_err());
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].cutouts.len(), 1);
        assert_eq!(
            rectangle_bounds(match &a[0].cutouts[0] {
                CopperGeometry::Polygon(r) => r,
                CopperGeometry::Circle { .. } | CopperGeometry::Curved { .. } =>
                    panic!("expected polygon"),
            }),
            Some([10200, 20200, 10400, 20600])
        );
    }
    #[test]
    fn refuses_wrong_owner_layer_and_touching_holes() {
        let source = SAMPLE.replace("1 1 0 GND", "2 1 0 GND").replace(
            "0 20\n0 0\n*END*",
            "0 20\n0 0\nCOPCUT 5 0.01 0 2\n2 2\n4 2\n4 6\n2 6\n2 2\n*END*",
        );
        assert!(parse(
            source
                .replace("COPCUT 5 0.01 0 2", "COPCUT 5 0.01 0 1")
                .as_bytes(),
            4
        )
        .is_err());
        assert!(parse(
            source
                .replace("2 2\n4 2\n4 6\n2 6\n2 2", "0 2\n4 2\n4 6\n0 6\n0 2")
                .as_bytes(),
            4
        )
        .is_err());
        assert!(parse(
            source
                .replace("COPCUT 5 0.01 0 2", "COPCCO 5 0.01 0 2")
                .as_bytes(),
            4
        )
        .is_err());
    }
    #[test]
    fn source_names_matching_piece_keywords_are_legal() {
        for name in ["COPCLS", "COPCIR", "COPCUT", "COPCCO"] {
            let a = SAMPLE.replace("GND_ZONE COPPER", &format!("{name} COPPER"));
            let areas = parse(a.as_bytes(), 4).unwrap();
            assert_eq!(areas[0].name, name);
        }
    }
    #[test]
    fn accepts_orthogonal_nonrectangular_copper_and_holes() {
        let input = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*LINES*\nCOP_ORTHO COPPER 40 40 3 1 0 GND\nCOPCLS 7 0.01 0 1\n0 0\n120 0\n120 50\n90 50\n90 90\n0 90\n0 0\nCOPCUT 7 0.01 0 1\n20 20\n50 20\n50 30\n35 30\n35 45\n20 45\n20 20\nCOPCUT 5 0.01 0 1\n60 60\n80 60\n80 80\n60 80\n60 60\n*END*\n";
        let areas = parse(input.as_bytes(), 4).unwrap();
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].cutouts.len(), 2);
        assert_eq!(areas[0].name, "COP_ORTHO");
        assert_eq!(areas[0].net_label.as_deref(), Some("GND"));
        // Every accepted area carries the exact source ring, not its box.
        assert_eq!(
            areas[0].geometry,
            CopperGeometry::Polygon(vec![
                [4000, 4000],
                [16000, 4000],
                [16000, 9000],
                [13000, 9000],
                [13000, 13000],
                [4000, 13000],
                [4000, 4000]
            ])
        );
        // A_025: a well-formed diagonal remains source-owned and is accepted.
        // Invalid original ring topology continues to fail closed.
        assert!(parse(input.replace("90 50\n90 90", "90 50\n91 90").as_bytes(), 4).is_ok());
        // Declare fewer pieces than authored; never omit a hole silently.
        assert!(parse(input.replace("40 40 3 1", "40 40 2 1").as_bytes(), 4).is_err());
    }
    #[test]
    fn rejects_orthogonal_nested_touching_and_outside_holes() {
        let base = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*LINES*\nA COPPER 0 0 3 1\nCOPCLS 5 0.01 0 1\n0 0\n100 0\n100 100\n0 100\n0 0\nCOPCUT 5 0.01 0 1\n20 20\n60 20\n60 60\n20 60\n20 20\nCOPCUT 5 0.01 0 1\n70 70\n80 70\n80 80\n70 80\n70 70\n*END*\n";
        assert_eq!(parse(base.as_bytes(), 4).unwrap()[0].cutouts.len(), 2);
        let nested = base.replace(
            "70 70\n80 70\n80 80\n70 80\n70 70",
            "30 30\n40 30\n40 40\n30 40\n30 30",
        );
        assert!(parse(nested.as_bytes(), 4).is_err());
        let touching = base.replace(
            "70 70\n80 70\n80 80\n70 80\n70 70",
            "60 30\n80 30\n80 40\n60 40\n60 30",
        );
        assert!(parse(touching.as_bytes(), 4).is_err());
        let outside = base.replace(
            "70 70\n80 70\n80 80\n70 80\n70 70",
            "95 70\n105 70\n105 80\n95 80\n95 70",
        );
        assert!(parse(outside.as_bytes(), 4).is_err());
        let cross = base.replace(
            "20 20\n60 20\n60 60\n20 60\n20 20",
            "20 20\n60 60\n60 20\n20 60\n20 20",
        );
        assert!(parse(cross.as_bytes(), 4).is_err());
    }
    #[test]
    fn a025_diagonal_source_preserves_vertices_and_rejects_touching() {
        let asc = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*LINES*\nDIA COPPER 0 0 2 1 0 GND\nCOPCLS 5 0.01 0 1\n0 0\n120 0\n110 90\n0 90\n0 0\nCOPCUT 4 0.01 0 1\n30 20\n50 20\n40 40\n30 20\n*END*\n";
        let v = parse(asc.as_bytes(), 4).unwrap();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].cutouts.len(), 1);
        let outer = match &v[0].geometry {
            CopperGeometry::Polygon(p) => p,
            _ => panic!("polygon expected"),
        };
        assert_eq!(outer[2], [11000, 9000]);
        assert_eq!(v[0].net_label.as_deref(), Some("GND"));
        assert!(parse(
            asc.replace("30 20\n50 20\n40 40\n30 20", "0 20\n50 20\n40 40\n0 20")
                .as_bytes(),
            4
        )
        .is_err());
        assert!(parse(
            asc.replace("DIA COPPER 0 0 2 1", "DIA COPPER 0 0 1 1")
                .as_bytes(),
            4
        )
        .is_err());
    }
    #[test]
    fn a026_source_owned_circle_accepts_explicit_v12_geometry() {
        let source = include_bytes!("../tests/fixtures/pads_copper_circular_void_v94.asc");
        let areas = parse(source, 4).unwrap();
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].cutouts.len(), 3);
        // A mistaken owner piece count must not paint filled copper over a void.
        let understated = String::from_utf8_lossy(source)
            .replace("COP_CIRC COPPER 50 50 4 1", "COP_CIRC COPPER 50 50 1 1");
        assert!(parse(understated.as_bytes(), 4).is_err());
    }
    #[test]
    fn a026_isolated_candidate_parses_owned_circle_and_mixed_holes() {
        let fixture = include_bytes!("../tests/fixtures/pads_copper_circular_void_v94.asc");
        let areas = parse_impl(fixture, 4, true).unwrap();
        assert_eq!(areas.len(), 1);
        assert_eq!(areas[0].cutouts.len(), 3);
        assert_eq!(areas[0].net_label.as_deref(), Some("GND"));
        assert_eq!(
            areas[0].cutouts[0],
            CopperGeometry::Circle {
                centre: [9000, 8000],
                radius: 1000
            }
        );
        assert_eq!(
            areas[0].cutouts[1],
            CopperGeometry::Circle {
                centre: [13000, 12000],
                radius: 1200
            }
        );
        // Production now has the matching lossless V12 Qt path.
        assert_eq!(parse(fixture, 4).unwrap(), areas);
    }

    #[test]
    fn a026_isolated_candidate_rejects_tangent_overlap_and_bad_circle() {
        // Git for Windows may check out .asc fixtures with CRLF. Normalize
        // only the test string so every adversarial mutation actually fires.
        let source = include_str!("../tests/fixtures/pads_copper_circular_void_v94.asc")
            .replace("\r\n", "\n");
        for (case, malformed) in [
            (
                "outer tangent",
                source.replace("30 30\n50 30", "0 30\n20 30"),
            ),
            (
                "circle overlap",
                source.replace("68 70\n92 70", "42 30\n62 30"),
            ),
            (
                "polygon overlap",
                source.replace("30 30\n50 30", "30 85\n50 85"),
            ),
            (
                "layer mismatch",
                source.replacen("COPCCO 2 0.01 0 1", "COPCCO 2 0.01 0 2", 1),
            ),
            (
                "diagonal diameter",
                source.replace("30 30\n50 30", "30 30\n50 50"),
            ),
            (
                "owner piece count",
                source.replace("COP_CIRC COPPER 50 50 4 1", "COP_CIRC COPPER 50 50 1 1"),
            ),
        ] {
            assert_ne!(malformed, source, "mutation was not applied: {case}");
            let candidate = parse_impl(malformed.as_bytes(), 4, true);
            assert!(
                candidate.is_err(),
                "candidate accepted malformed {case}: {candidate:?}"
            );
        }
    }

    #[test]
    fn a027_source_circle_outer_hole_retains_analytic_geometry() {
        let source = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*LINES*\nZ COPPER 0 0 2 1\nCOPCIR 2 0.01 0 1\n-40 0\n40 0\nCOPCCO 2 0.01 0 1\n-10 0\n10 0\n*END*\n";
        let parsed = parse_impl(source.as_bytes(), 4, true).unwrap();
        assert_eq!(parsed[0].cutouts.len(), 1);
        assert_eq!(
            parsed[0].cutouts[0],
            CopperGeometry::Circle {
                centre: [0, 0],
                radius: 1000
            }
        );
        assert_eq!(parse(source.as_bytes(), 4).unwrap(), parsed);
    }

    #[test]
    fn a027_rejects_tangent_overlapping_and_misowned_hollow_circles() {
        let good = include_str!("../tests/fixtures/pads_copper_hollow_circle_v94.asc")
            .replace("\r\n", "\n");
        let parsed = parse(good.as_bytes(), 4).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].cutouts.len(), 2);
        assert!(matches!(parsed[0].geometry, CopperGeometry::Circle { .. }));
        assert_eq!(parsed[0].net_label.as_deref(), Some("GND"));
        // The source does not certify an electrical NetId merely by naming GND.
        for (case, src) in [
            ("outer tangent", good.replace("25 0\n55 0", "70 0\n100 0")),
            ("hole overlap", good.replace("25 0\n55 0", "-35 0\n-5 0")),
            (
                "hole layer",
                good.replacen("COPCCO 2 0.01 0 1", "COPCCO 2 0.01 0 2", 1),
            ),
            ("zero hole", good.replace("25 0\n55 0", "25 0\n25 0")),
            (
                "undeclared piece",
                good.replace("COP_RING COPPER 200 200 3 1", "COP_RING COPPER 200 200 2 1"),
            ),
            (
                "misowned void",
                good.replace("COP_RING COPPER 200 200 3 1", "COP_RING COPPER 200 200 1 1"),
            ),
        ] {
            assert_ne!(src, good, "mutation did not apply: {case}");
            assert!(parse(src.as_bytes(), 4).is_err(), "accepted {case}");
        }
        // A_028 accepts strictly verified source-owned COPCUT inside COPCIR
        // while preserving fail-closed behavior for contacts and overlaps.
        let mixed = good.replace(
            "COPCCO 2 0.01 0 1\n25 0\n55 0",
            "COPCUT 5 0.01 0 1\n20 20\n30 20\n30 30\n20 30\n20 20",
        );
        assert_ne!(mixed, good);
        assert_eq!(parse(mixed.as_bytes(), 4).unwrap()[0].cutouts.len(), 2);
    }

    #[test]
    fn a028_circle_polygon_and_mixed_fixtures_preserve_analytic_source() {
        for (fixture, circles) in [
            (
                include_bytes!("../tests/fixtures/pads_copper_circular_polygon_v94.asc").as_slice(),
                0,
            ),
            (
                include_bytes!("../tests/fixtures/pads_copper_circular_mixed_v94.asc").as_slice(),
                1,
            ),
        ] {
            let result = parse(fixture, 4).unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].cutouts.len(), 2);
            assert!(matches!(result[0].geometry, CopperGeometry::Circle { .. }));
            assert_eq!(result[0].net_label.as_deref(), Some("GND"));
            assert_eq!(
                result[0]
                    .cutouts
                    .iter()
                    .filter(|v| matches!(v, CopperGeometry::Circle { .. }))
                    .count(),
                circles,
            );
            assert!(matches!(result[0].cutouts[0], CopperGeometry::Polygon(_)));
        }
    }

    #[test]
    fn a028_rejects_poly_contact_overlap_misownership_and_mixed_collision() {
        let poly = include_str!("../tests/fixtures/pads_copper_circular_polygon_v94.asc")
            .replace("\r\n", "\n");
        let mixed = include_str!("../tests/fixtures/pads_copper_circular_mixed_v94.asc")
            .replace("\r\n", "\n");
        let malformed = [
            ("polygon touches circle", poly.replace("-45 -15", "-100 0")),
            (
                "polygon overlaps sibling",
                poly.replace(
                    "30 -20\n50 -20\n40 5\n30 -20",
                    "-40 -10\n-20 -10\n-30 10\n-40 -10",
                ),
            ),
            (
                "self crossing hole",
                poly.replace(
                    "-45 -15\n-25 -15\n-25 5\n-45 5\n-45 -15",
                    "-45 -15\n-25 5\n-25 -15\n-45 5\n-45 -15",
                ),
            ),
            (
                "wrong layer",
                poly.replacen("COPCUT 5 0.01 0 1", "COPCUT 5 0.01 0 2", 1),
            ),
            (
                "undeclared piece",
                poly.replace("COP_POLY COPPER 200 200 3 1", "COP_POLY COPPER 200 200 1 1"),
            ),
            (
                "orphan polygon",
                poly.replace("COP_POLY COPPER 200 200 3 1", "COP_POLY COPPER 200 200 2 1"),
            ),
            (
                "mixed circle contacts polygon",
                mixed.replace("25 10\n55 10", "-40 0\n-10 0"),
            ),
            (
                "mixed circle touches outer",
                mixed.replace("25 10\n55 10", "70 0\n100 0"),
            ),
            (
                "mixed wrong layer",
                mixed.replace("COPCCO 2 0.01 0 1", "COPCCO 2 0.01 0 2"),
            ),
        ];
        for (case, src) in malformed {
            let original = if case.starts_with("mixed") {
                &mixed
            } else {
                &poly
            };
            assert_ne!(&src, original, "mutation missing: {case}");
            assert!(parse(src.as_bytes(), 4).is_err(), "accepted {case}");
        }
    }

    #[test]
    fn rejects_range_truncation_duplicate_and_end() {
        assert!(parse(SAMPLE.as_bytes(), 1).is_err());
        assert!(parse(SAMPLE.replace("10 0", "10.001 0").as_bytes(), 4).is_err());
        assert!(parse(SAMPLE.replace("*END*", "").as_bytes(), 4).is_err());
        let duplicate = SAMPLE.replace(
            "*END*",
            "GND_ZONE COPPER 0 0 1 1\nCOPCIR 2 0.01 0 1\n0 0\n10 0\n*END*",
        );
        assert!(parse(duplicate.as_bytes(), 4).is_err());
    }
    #[test]
    fn a029_accepts_redundant_collinear_source_corner_with_exact_arc_count() {
        let source = include_bytes!("../tests/fixtures/pads_curved_copper_collinear_v94.asc");
        let coppers = parse(source, 4).unwrap();
        assert_eq!(coppers.len(), 1);
        match &coppers[0].geometry {
            CopperGeometry::Curved {
                segments,
                display_ring,
            } => {
                assert_eq!(segments.len(), 9);
                assert_eq!(display_ring.len(), 38);
                assert_eq!(
                    segments
                        .iter()
                        .filter(|s| matches!(s, crate::outline_topology::Segment::Arc(_)))
                        .count(),
                    4
                );
            }
            _ => panic!("expected source arc geometry"),
        }
    }
    #[test]
    fn a029_canonical_keeps_analytic_arcs_and_unverified_net() {
        let input = include_bytes!("../tests/fixtures/pads_curved_copper_candidate_v94.asc");
        let parsed = crate::route_preview::parse(input).unwrap();
        let mut board = crate::route_preview::into_canonical(input, &parsed).unwrap();
        let areas = parse(input, parsed.max_layer).unwrap();
        augment(&mut board, &areas).unwrap();
        assert_eq!(board.areas.len(), 1);
        assert_eq!(board.areas[0].outer.segments.len(), 8);
        assert_eq!(board.areas[0].cutouts.len(), 0);
        assert!(board.areas[0].net.is_none());
        assert_eq!(
            board.areas[0]
                .outer
                .segments
                .iter()
                .filter(|s| matches!(s, boardsi_model::BoundarySegment::Arc { .. }))
                .count(),
            4
        );
        assert!(!board.validate().has_errors());
    }
    #[test]
    fn a029_curved_copper_source_preserves_exact_arcs_and_no_cutout() {
        let asc = include_bytes!("../tests/fixtures/pads_curved_copper_candidate_v94.asc");
        let areas = parse(asc, 4).unwrap();
        assert_eq!(areas.len(), 1);
        let (segments, visual) = match &areas[0].geometry {
            CopperGeometry::Curved {
                segments,
                display_ring,
            } => (segments, display_ring),
            _ => panic!("expected curved analytical source"),
        };
        assert_eq!(
            segments
                .iter()
                .filter(|s| matches!(s, crate::outline_topology::Segment::Arc(_)))
                .count(),
            4
        );
        assert_eq!(visual.len(), 37);
        assert!(areas[0].cutouts.is_empty());
        let src = std::str::from_utf8(asc).unwrap();
        assert!(parse(
            src.replace("COP_ARC COPPER 0 0 1 1", "COP_ARC COPPER 0 0 2 1")
                .as_bytes(),
            4
        )
        .is_err());
        assert!(parse(src.replace("2700 900", "2700 450").as_bytes(), 4).is_err());
    }
}
