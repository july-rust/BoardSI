//! A_016 P2-G: deliberately restricted PADS V9.4 MILS *LINES* BOARD parser.
//! Basis: PADS Layout ASCII Format Specification 9.4, pp. 27-31.
//! Straight CLOSED and BRDCLS polygons only; arcs/circles/reuse fail closed.
//! Geometry is outline artwork ONLY: no copper, topology, drilling or SI proof.
use boardsi_model::*;

const MAX_CORNER_COUNT: usize = 256;
const MAX_SHAPES: usize = 64;
const MAX_COORD: i64 = 5_600_000; // 0.01 mil ticks, 254 nm/tick.
const NM_PER_TICK: i64 = 254;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    pub name: String,
    pub cutout: bool,
    pub vertices: Vec<[i64; 2]>, // First point repeated last; source orientation retained.
    pub source_line: usize,
}
fn fixed(s: &str) -> Result<i64, &'static str> {
    let (negative, digits) = match s.strip_prefix('-') {
        Some(v) => (true, v),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let (whole, fraction) = digits.split_once('.').unwrap_or((digits, ""));
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || fraction.len() > 2
        || !fraction.bytes().all(|b| b.is_ascii_digit())
    {
        return Err("OUTLINE_NUMBER");
    }
    let a = whole.parse::<i64>().map_err(|_| "OUTLINE_NUMBER")?;
    let b = match fraction.len() {
        0 => 0,
        1 => fraction.parse::<i64>().map_err(|_| "OUTLINE_NUMBER")? * 10,
        _ => fraction.parse::<i64>().map_err(|_| "OUTLINE_NUMBER")?,
    };
    let v = a
        .checked_mul(100)
        .and_then(|v| v.checked_add(b))
        .ok_or("OUTLINE_COORD_OVERFLOW")?;
    if negative {
        v.checked_neg().ok_or("OUTLINE_COORD_OVERFLOW")
    } else {
        Ok(v)
    }
}
fn bounded(v: i64) -> Result<i64, &'static str> {
    if (-MAX_COORD..=MAX_COORD).contains(&v) {
        Ok(v)
    } else {
        Err("OUTLINE_COORD_RANGE")
    }
}
fn orient(a: [i64; 2], b: [i64; 2], c: [i64; 2]) -> i128 {
    let x = i128::from(b[0] - a[0]);
    let y = i128::from(b[1] - a[1]);
    x * i128::from(c[1] - a[1]) - y * i128::from(c[0] - a[0])
}
fn on_segment(a: [i64; 2], b: [i64; 2], p: [i64; 2]) -> bool {
    orient(a, b, p) == 0
        && (a[0].min(b[0])..=a[0].max(b[0])).contains(&p[0])
        && (a[1].min(b[1])..=a[1].max(b[1])).contains(&p[1])
}
fn intersect(a: [i64; 2], b: [i64; 2], c: [i64; 2], d: [i64; 2]) -> bool {
    let (u, v, w, x) = (
        orient(a, b, c),
        orient(a, b, d),
        orient(c, d, a),
        orient(c, d, b),
    );
    if (u == 0 && on_segment(a, b, c))
        || (v == 0 && on_segment(a, b, d))
        || (w == 0 && on_segment(c, d, a))
        || (x == 0 && on_segment(c, d, b))
    {
        return true;
    }
    (u > 0) != (v > 0) && (w > 0) != (x > 0)
}
fn valid_ring(p: &[[i64; 2]]) -> bool {
    if p.len() < 4 || p.first() != p.last() {
        return false;
    }
    let n = p.len() - 1;
    let mut twice_area = 0i128;
    for i in 0..n {
        if p[i] == p[i + 1] {
            return false;
        }
        twice_area += i128::from(p[i][0]) * i128::from(p[i + 1][1])
            - i128::from(p[i + 1][0]) * i128::from(p[i][1]);
        for j in i + 1..n {
            // Adjacent edges share an endpoint by definition.
            if j == i + 1 || (i == 0 && j == n - 1) {
                continue;
            }
            if intersect(p[i], p[i + 1], p[j], p[j + 1]) {
                return false;
            }
        }
    }
    twice_area != 0
}
fn inside(p: [i64; 2], ring: &[[i64; 2]]) -> bool {
    let mut winding = false;
    for w in ring.windows(2) {
        let (a, b) = (w[0], w[1]);
        if on_segment(a, b, p) {
            return false;
        }
        if (a[1] > p[1]) != (b[1] > p[1]) {
            let cross = orient(a, b, p);
            if (cross > 0) == (b[1] > a[1]) {
                winding = !winding;
            }
        }
    }
    winding
}
fn rings_touch(a: &[[i64; 2]], b: &[[i64; 2]]) -> bool {
    a.windows(2)
        .any(|aa| b.windows(2).any(|bb| intersect(aa[0], aa[1], bb[0], bb[1])))
}

/// Does not use the file extension as evidence. Skips non-BOARD *LINES* items
/// structurally; never interprets COPPER/KEEPOUT as a physical board boundary.
pub fn parse(bytes: &[u8]) -> Result<Vec<Shape>, &'static str> {
    if bytes.len() > crate::route_preview::MAX_FILE_BYTES {
        return Err("INPUT_TOO_LARGE");
    }
    // Public parser: validate the complete source envelope independently of
    // route_preview::parse. An arbitrary file with an embedded *LINES* section
    // must never be promoted to a physical board outline.
    let first = bytes.split(|&b| b == b'\n').next().ok_or("EMPTY_FILE")?;
    let header = std::str::from_utf8(first).map_err(|_| "INVALID_HEADER_ENCODING")?;
    let tag = header.split_ascii_whitespace().next().unwrap_or("");
    if !matches!(
        tag,
        "!PADS-POWERPCB-V9.4-MILS!"
            | "!PADS-POWERPCB-V9.4-MILS-CP1252!"
            | "!PADS-POWERPCB-V9.4-MILS-250L!"
            | "!PADS-POWERPCB-V9.4-MILS-250L-CP1252!"
    ) {
        return Err("UNSUPPORTED_REVISION_OR_UNIT");
    }
    let source = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = source.lines().collect();
    let mut shapes = Vec::new();
    let mut in_lines = false;
    let mut found_end = false;
    let mut i = 1usize; // This public parser independently checked the header.
    while i < lines.len() {
        if lines[i].len() > 16_384 {
            return Err("OVERLONG_LINE");
        }
        let raw = lines[i].trim();
        let f: Vec<&str> = raw.split_ascii_whitespace().collect();
        if f.is_empty() || f[0] == "*REMARK*" || f[0] == "REMARK" {
            i += 1;
            continue;
        }
        if f[0] == "*END*" {
            found_end = true;
            break;
        }
        // *GET* is a library-reference statement INSIDE *LINES*, not a
        // new section. Reuse geometry is unsupported: reject explicitly
        // instead of dropping a later BOARD record without warning.
        if in_lines && f[0] == "*GET*" {
            return Err("UNSUPPORTED_OUTLINE_GET");
        }
        if f[0].starts_with('*') {
            in_lines = f[0] == "*LINES*";
            i += 1;
            continue;
        }
        if !in_lines {
            i += 1;
            continue;
        }
        if f.len() >= 2 && f[1] == "BOARD" && f.len() < 6 {
            return Err("UNSUPPORTED_OUTLINE_HEADER");
        }
        if f.len() < 6 || !matches!(f[1], "BOARD" | "LINES" | "COPPER" | "COPCUT" | "KEEPOUT") {
            // Unsupported text/reuse is not granted board-outline semantics.
            i += 1;
            continue;
        }
        if f[1] != "BOARD" {
            i += 1;
            continue;
        }
        if f.len() > 7
            || (f.len() == 7 && f[6] != "0")
            || f[0].len() > 16
            || !f[0].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err("UNSUPPORTED_OUTLINE_HEADER");
        }
        let origin = [bounded(fixed(f[2])?)?, bounded(fixed(f[3])?)?];
        if f[4] != "1" || f[5] != "0" {
            return Err("UNSUPPORTED_OUTLINE_PIECES");
        }
        let source_line = i + 1;
        i += 1;
        let piece_line = lines.get(i).ok_or("TRUNCATED_OUTLINE")?;
        if piece_line.len() > 16_384 {
            return Err("OVERLONG_LINE");
        }
        let p: Vec<&str> = piece_line.split_ascii_whitespace().collect();
        if p.len() != 5 || !matches!(p[0], "CLOSED" | "BRDCLS") || p[3] != "0" || p[4] != "0" {
            return Err("UNSUPPORTED_OUTLINE_PIECE");
        }
        let count = p[1].parse::<usize>().map_err(|_| "INVALID_OUTLINE_COUNT")?;
        if !(4..=MAX_CORNER_COUNT).contains(&count) {
            return Err("OUTLINE_COUNT_RANGE");
        }
        let width = fixed(p[2])?;
        if !(1..=25_000).contains(&width) {
            return Err("OUTLINE_WIDTH_RANGE");
        }
        let cutout = p[0] == "BRDCLS";
        i += 1;
        let mut vertices = Vec::with_capacity(count);
        for _ in 0..count {
            let row = lines.get(i).ok_or("TRUNCATED_OUTLINE")?;
            if row.len() > 16_384 {
                return Err("OVERLONG_LINE");
            }
            let pts: Vec<_> = row.split_ascii_whitespace().collect();
            if pts.len() != 2 {
                return Err("UNSUPPORTED_OUTLINE_ARC_OR_POINT");
            }
            let x = origin[0]
                .checked_add(fixed(pts[0])?)
                .ok_or("OUTLINE_COORD_OVERFLOW")?;
            let y = origin[1]
                .checked_add(fixed(pts[1])?)
                .ok_or("OUTLINE_COORD_OVERFLOW")?;
            vertices.push([bounded(x)?, bounded(y)?]);
            i += 1;
        }
        if !valid_ring(&vertices) {
            return Err("INVALID_OUTLINE_RING");
        }
        if shapes.len() >= MAX_SHAPES {
            return Err("OUTLINE_SHAPE_LIMIT");
        }
        shapes.push(Shape {
            name: f[0].into(),
            cutout,
            vertices,
            source_line,
        });
    }
    if !found_end {
        return Err("MISSING_END_MARKER");
    }
    if shapes.is_empty() {
        return Ok(shapes);
    }
    let outers: Vec<_> = shapes.iter().filter(|s| !s.cutout).collect();
    if outers.len() != 1 {
        return Err("OUTLINE_OUTER_COUNT");
    }
    let outer = &outers[0].vertices;
    let cuts: Vec<_> = shapes.iter().filter(|s| s.cutout).collect();
    for cut in &cuts {
        if rings_touch(outer, &cut.vertices)
            || !cut.vertices[..cut.vertices.len() - 1]
                .iter()
                .all(|&p| inside(p, outer))
        {
            return Err("CUTOUT_NOT_CONTAINED");
        }
    }
    for (i, a) in cuts.iter().enumerate() {
        for b in &cuts[i + 1..] {
            if rings_touch(&a.vertices, &b.vertices)
                || inside(a.vertices[0], &b.vertices)
                || inside(b.vertices[0], &a.vertices)
            {
                return Err("CUTOUT_OVERLAP");
            }
        }
    }
    Ok(shapes)
}
pub fn augment(board: &mut BoardModel, shapes: &[Shape]) -> Result<(), &'static str> {
    if shapes.is_empty() {
        return Ok(());
    }
    let source: SourceId = board.sources.first().ok_or("MISSING_SOURCE")?.id.clone();
    let frame = board.frames.first().ok_or("MISSING_FRAME")?.meta.id.clone();
    let meta = |kind: ObjectKind, key: &str| ObjectMeta {
        id: ObjectId::new(&source, kind, key, ""),
        provenance: GeometryProvenance {
            source_object_type: "PADS V9.4 MILS *LINES* BOARD polygon (strict subset)".into(),
            history: vec![],
        },
    };
    let boundary = |s: &Shape| Boundary {
        segments: s
            .vertices
            .windows(2)
            .map(|pair| BoundarySegment::Line {
                start: Point {
                    frame: frame.clone(),
                    ticks: pair[0],
                },
                end: Point {
                    frame: frame.clone(),
                    ticks: pair[1],
                },
            })
            .collect(),
    };
    let outer = shapes.iter().find(|s| !s.cutout).ok_or("MISSING_OUTER")?;
    let cuts = shapes
        .iter()
        .filter(|s| s.cutout)
        .map(|s| Cutout {
            meta: meta(
                ObjectKind::Cutout,
                &format!("outline-cut-{}-{}", s.name, s.source_line),
            ),
            boundary: boundary(s),
        })
        .collect();
    board.outlines.push(Outline {
        meta: meta(
            ObjectKind::Outline,
            &format!("outline-{}-{}", outer.name, outer.source_line),
        ),
        outer: boundary(outer),
        cutouts: cuts,
    });
    let coverage = &mut board.sources[0].capability.object_coverage;
    coverage.push(
        "LINES BOARD CLOSED/BRDCLS exact straight boundaries; no arcs or circle support".into(),
    );
    if board.validate().has_errors() {
        return Err("CANONICAL_OUTLINE_INVALID");
    }
    Ok(())
}
pub fn wire_nm(ticks: i64) -> Result<i64, &'static str> {
    ticks.checked_mul(NM_PER_TICK).ok_or("OUTLINE_NM_OVERFLOW")
}
#[cfg(test)]
mod tests {
    use super::*;
    const INPUT:&str="!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXLAYER 4\n*LINES*\nBOARD1 BOARD 100 200 1 0\nCLOSED 5 1 0 0\n0 0\n400 0\n400 300\n0 300\n0 0\nHOLE1 BOARD 100 200 1 0\nBRDCLS 5 1 0 0\n100 100\n200 100\n200 200\n100 200\n100 100\n*END*\n";
    #[test]
    fn exact_closed_outline_and_cutout_with_origin() {
        let shapes = parse(INPUT.as_bytes()).unwrap();
        assert_eq!(shapes.len(), 2);
        assert_eq!(shapes[0].vertices[0], [10_000, 20_000]);
        assert_eq!(shapes[0].vertices[1], [50_000, 20_000]);
        assert_eq!(wire_nm(shapes[0].vertices[0][0]).unwrap(), 2_540_000);
        let route = crate::route_preview::parse(INPUT.as_bytes()).unwrap();
        let mut model = crate::route_preview::into_canonical(INPUT.as_bytes(), &route).unwrap();
        augment(&mut model, &shapes).unwrap();
        assert_eq!(model.outlines.len(), 1);
        assert_eq!(model.outlines[0].outer.segments.len(), 4);
        assert_eq!(model.outlines[0].cutouts.len(), 1);
    }
    #[test]
    fn rejects_get_inside_lines_instead_of_silently_dropping_later_board() {
        let with_get = INPUT.replace(
            "BOARD1 BOARD 100 200 1 0",
            "*GET* LIBRARY_SHAPE\nBOARD1 BOARD 100 200 1 0",
        );
        assert_eq!(
            parse(with_get.as_bytes()).unwrap_err(),
            "UNSUPPORTED_OUTLINE_GET"
        );
        // A GET in an unrelated section is not a physical board reference.
        let outside_lines = INPUT.replace("*PCB*", "*PCB*\n*GET* UNRELATED");
        assert_eq!(parse(outside_lines.as_bytes()).unwrap().len(), 2);
    }
    #[test]
    fn validates_envelope_even_when_called_without_route_parser() {
        for bad in [
            INPUT.replace("!PADS-POWERPCB-V9.4-MILS!", "NON_PADS_FILE"),
            INPUT.replace("V9.4-MILS", "V9.5-MM"),
            INPUT.replace(
                "!PADS-POWERPCB-V9.4-MILS!",
                "!PADS-POWERPCB-V9.4-MILS-CP932!",
            ),
        ] {
            assert_eq!(
                parse(bad.as_bytes()).unwrap_err(),
                "UNSUPPORTED_REVISION_OR_UNIT"
            );
        }
        assert_eq!(
            parse(INPUT.replace("*END*", "").as_bytes()).unwrap_err(),
            "MISSING_END_MARKER"
        );
        // Requiring *END* also applies when a file has no accepted outlines.
        assert_eq!(
            parse(b"!PADS-POWERPCB-V9.4-MILS!\n*PCB*\nMAXLAYER 4\n").unwrap_err(),
            "MISSING_END_MARKER"
        );
        for suffix in ["-CP1252", "-250L", "-250L-CP1252"] {
            let with_suffix = INPUT.replace(
                "!PADS-POWERPCB-V9.4-MILS!",
                &format!("!PADS-POWERPCB-V9.4-MILS{}!", suffix),
            );
            assert_eq!(parse(with_suffix.as_bytes()).unwrap().len(), 2);
        }
    }
    #[test]
    fn rejects_oversized_piece_headers_and_vertex_rows() {
        let piece = INPUT.replace(
            "CLOSED 5 1 0 0",
            &format!("CLOSED 5 1 0 0{}", " ".repeat(16_385)),
        );
        let vertex = INPUT.replace("400 300", &format!("400 300{}", " ".repeat(16_385)));
        assert_eq!(parse(piece.as_bytes()).unwrap_err(), "OVERLONG_LINE");
        assert_eq!(parse(vertex.as_bytes()).unwrap_err(), "OVERLONG_LINE");
    }
    #[test]
    fn rejects_broken_shapes_and_unsupported_arcs() {
        for bad in [
            INPUT.replace("100 100\n*END*", "100 101\n*END*"),
            INPUT.replace("CLOSED 5 1 0 0", "CIRCLE 5 1 0 0"),
            INPUT.replace("BRDCLS 5 1 0 0", "BRDCIR 5 1 0 0"),
            INPUT.replace("100 100\n200 100", "-100 -100\n200 100"),
            INPUT.replace("0 300\n0 0", "400 0\n0 0"),
        ] {
            assert!(parse(bad.as_bytes()).is_err());
        }
    }
}
