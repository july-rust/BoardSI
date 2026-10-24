//! A_018 P2-I: exact-source-grid, bounded PADS V9.4 arc-corner preflight.
//! This is NOT yet an accepted board outline. It validates a deliberately
//! restricted cardinal quarter-arc before Canonical topology/IPC/viewer work.
//! PADS Layout ASCII 9.4, Lines Definition Format, pp. 30-31.
use crate::outline_preview::{bounded, fixed};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardinalArcCorner {
    pub start: [i64; 2],
    pub end: [i64; 2],
    pub centre: [i64; 2],
    pub clockwise: bool,
}

/// PADS row: start-x start-y begin-angle sweep-angle bbox-x1 bbox-y1 bbox-x2 bbox-y2.
/// Angles are source integers in tenths of a degree; positive sweep is CCW.
/// Only exactly representable 90-degree cardinal arcs are normalized here.
/// Other arcs remain unsupported rather than silently rounded or tessellated.
pub fn parse_cardinal(
    fields: &[&str],
    origin: [i64; 2],
) -> Result<CardinalArcCorner, &'static str> {
    if fields.len() != 8 {
        return Err("INVALID_OUTLINE_ARC_FIELDS");
    }
    let coord = |s: &str, axis: usize| -> Result<i64, &'static str> {
        origin[axis]
            .checked_add(fixed(s)?)
            .ok_or("OUTLINE_COORD_OVERFLOW")
            .and_then(bounded)
    };
    let start = [coord(fields[0], 0)?, coord(fields[1], 1)?];
    let begin = fields[2]
        .parse::<i32>()
        .map_err(|_| "INVALID_OUTLINE_ARC_ANGLE")?;
    let sweep = fields[3]
        .parse::<i32>()
        .map_err(|_| "INVALID_OUTLINE_ARC_ANGLE")?;
    if sweep != 900 && sweep != -900 {
        return Err("UNSUPPORTED_OUTLINE_ARC_SWEEP");
    }
    let quadrant = match begin.rem_euclid(3600) {
        0 => 0usize,
        900 => 1,
        1800 => 2,
        2700 => 3,
        _ => return Err("UNSUPPORTED_OUTLINE_ARC_ANGLE"),
    };
    let low = [coord(fields[4], 0)?, coord(fields[5], 1)?];
    let high = [coord(fields[6], 0)?, coord(fields[7], 1)?];
    if high[0] <= low[0] || high[1] <= low[1] || high[0] - low[0] != high[1] - low[1] {
        return Err("INVALID_OUTLINE_ARC_BOUNDS");
    }
    let diameter = high[0] - low[0];
    if diameter < 2 || diameter % 2 != 0 {
        return Err("UNSUPPORTED_OUTLINE_ARC_PRECISION");
    }
    let radius = diameter / 2;
    let centre = [low[0] + radius, low[1] + radius];
    let compass = [
        [centre[0] + radius, centre[1]],
        [centre[0], centre[1] + radius],
        [centre[0] - radius, centre[1]],
        [centre[0], centre[1] - radius],
    ];
    if start != compass[quadrant] {
        return Err("OUTLINE_ARC_START_MISMATCH");
    }
    let dest = if sweep > 0 {
        (quadrant + 1) % 4
    } else {
        (quadrant + 3) % 4
    };
    Ok(CardinalArcCorner {
        start,
        end: compass[dest],
        centre,
        clockwise: sweep < 0,
    })
}

/// Validate a closed PADS line/arc piece without promoting it to BoardModel.
/// The last source row must repeat the first point, just as the PADS format
/// requires. Chord topology is only a preflight: exact arc intersections and
/// cutout containment still must be certified before import is enabled.
pub fn preflight_closed(
    rows: &[Vec<&str>],
    origin: [i64; 2],
) -> Result<Vec<Option<CardinalArcCorner>>, &'static str> {
    if !(4..=256).contains(&rows.len()) {
        return Err("OUTLINE_COUNT_RANGE");
    }
    let mut vertices = Vec::with_capacity(rows.len());
    let mut arcs = Vec::with_capacity(rows.len() - 1);
    for (index, row) in rows.iter().enumerate() {
        if row.len() != 2 && row.len() != 8 {
            return Err("INVALID_OUTLINE_ARC_FIELDS");
        }
        if index == rows.len() - 1 && row.len() != 2 {
            return Err("INVALID_OUTLINE_ARC_CLOSURE");
        }
        let point = [
            origin[0]
                .checked_add(fixed(row[0])?)
                .ok_or("OUTLINE_COORD_OVERFLOW")
                .and_then(bounded)?,
            origin[1]
                .checked_add(fixed(row[1])?)
                .ok_or("OUTLINE_COORD_OVERFLOW")
                .and_then(bounded)?,
        ];
        vertices.push(point);
        if index < rows.len() - 1 {
            arcs.push(if row.len() == 8 {
                Some(parse_cardinal(row, origin)?)
            } else {
                None
            });
        }
    }
    if vertices.first() != vertices.last() {
        return Err("INVALID_OUTLINE_RING");
    }
    for (i, arc) in arcs.iter().enumerate() {
        if let Some(arc) = arc {
            if arc.start != vertices[i] || arc.end != vertices[i + 1] {
                return Err("OUTLINE_ARC_ENDPOINT_MISMATCH");
            }
        }
    }
    if !crate::outline_preview::valid_ring(&vertices) {
        return Err("INVALID_OUTLINE_RING");
    }
    Ok(arcs)
}

/// Build analytical Canonical segments for *preflight tests only*.
/// This does not establish exact curved-ring intersection or containment;
/// the production importer must not accept this boundary until those gates
/// and versioned IPC/CPU/RHI rendering have passed independent validation.
pub fn build_preflight_boundary(
    board: &boardsi_model::BoardModel,
    rows: &[Vec<&str>],
    origin: [i64; 2],
) -> Result<boardsi_model::Boundary, &'static str> {
    use boardsi_model::{Boundary, BoundarySegment, Point};
    let arcs = preflight_closed(rows, origin)?;
    let frame = board
        .frames
        .first()
        .ok_or("MISSING_FRAME")?
        .meta
        .id
        .clone();
    let mut points = Vec::with_capacity(rows.len());
    for row in rows {
        points.push([
            origin[0]
                .checked_add(fixed(row[0])?)
                .ok_or("OUTLINE_COORD_OVERFLOW")
                .and_then(bounded)?,
            origin[1]
                .checked_add(fixed(row[1])?)
                .ok_or("OUTLINE_COORD_OVERFLOW")
                .and_then(bounded)?,
        ]);
    }
    let to_point = |ticks: [i64; 2]| Point {
        frame: frame.clone(),
        ticks,
    };
    let mut segments = Vec::with_capacity(arcs.len());
    for (index, arc) in arcs.iter().enumerate() {
        segments.push(if let Some(a) = arc {
            BoundarySegment::Arc {
                centre: to_point(a.centre),
                start: to_point(a.start),
                end: to_point(a.end),
                clockwise: a.clockwise,
            }
        } else {
            BoundarySegment::Line {
                start: to_point(points[index]),
                end: to_point(points[index + 1]),
            }
        });
    }
    Ok(Boundary { segments })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_cardinal_quarter_arcs_positive_and_negative() {
        let ccw = parse_cardinal(
            &["200", "100", "0", "900", "0", "0", "200", "200"],
            [10_000, 20_000],
        )
        .unwrap();
        assert_eq!(ccw.start, [30_000, 30_000]);
        assert_eq!(ccw.centre, [20_000, 30_000]);
        assert_eq!(ccw.end, [20_000, 40_000]);
        assert!(!ccw.clockwise);

        let cw = parse_cardinal(
            &["100", "200", "900", "-900", "0", "0", "200", "200"],
            [10_000, 20_000],
        )
        .unwrap();
        assert_eq!(cw.end, [30_000, 30_000]);
        assert!(cw.clockwise);
    }
    #[test]
    fn rejects_non_exact_or_inconsistent_arc_records() {
        let cases: [([&str; 8], &str); 6] = [
            (
                ["200", "100", "100", "900", "0", "0", "200", "200"],
                "UNSUPPORTED_OUTLINE_ARC_ANGLE",
            ),
            (
                ["200", "100", "0", "450", "0", "0", "200", "200"],
                "UNSUPPORTED_OUTLINE_ARC_SWEEP",
            ),
            (
                ["200", "100", "0", "900", "0", "0", "200", "210"],
                "INVALID_OUTLINE_ARC_BOUNDS",
            ),
            (
                ["200", "100", "0", "900", "0", "0", "200.01", "200.01"],
                "UNSUPPORTED_OUTLINE_ARC_PRECISION",
            ),
            (
                ["190", "100", "0", "900", "0", "0", "200", "200"],
                "OUTLINE_ARC_START_MISMATCH",
            ),
            (
                ["200", "100", "not-a-number", "900", "0", "0", "200", "200"],
                "INVALID_OUTLINE_ARC_ANGLE",
            ),
        ];
        for (fields, expected) in cases {
            assert_eq!(parse_cardinal(&fields, [0, 0]).unwrap_err(), expected);
        }
    }
    #[test]
    fn preflight_mixed_rounded_rectangle_preserves_four_exact_arcs() {
        let source = "100 0\n300 0 2700 900 200 0 400 200\n400 100\n400 200 0 900 200 100 400 300\n300 300\n100 300 900 900 0 100 200 300\n0 200\n0 100 1800 900 0 0 200 200\n100 0";
        let rows: Vec<Vec<&str>> = source
            .lines()
            .map(|line| line.split_ascii_whitespace().collect())
            .collect();
        let arcs = preflight_closed(&rows, [0, 0]).unwrap();
        assert_eq!(arcs.len(), 8);
        assert_eq!(arcs.iter().filter(|a| a.is_some()).count(), 4);
        assert_eq!(arcs[1].unwrap().end, [40_000, 10_000]);
        assert_eq!(arcs[7].unwrap().end, [10_000, 0]);

        let malformed = source.replace("400 100", "400 101");
        let bad_rows: Vec<Vec<&str>> = malformed
            .lines()
            .map(|line| line.split_ascii_whitespace().collect())
            .collect();
        assert_eq!(
            preflight_closed(&bad_rows, [0, 0]).unwrap_err(),
            "OUTLINE_ARC_ENDPOINT_MISMATCH"
        );
    }

    #[test]
    fn canonical_preflight_keeps_analytical_arcs_not_display_chords() {
        let source = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\\n*PCB*\\nMAXLAYER 4\\n*END*\\n";
        let parsed = crate::route_preview::parse(source.as_bytes()).unwrap();
        let board = crate::route_preview::into_canonical(source.as_bytes(), &parsed).unwrap();
        let text = "100 0\\n300 0 2700 900 200 0 400 200\\n400 100\\n400 200 0 900 200 100 400 300\\n300 300\\n100 300 900 900 0 100 200 300\\n0 200\\n0 100 1800 900 0 0 200 200\\n100 0";
        let rows: Vec<Vec<&str>> = text
            .lines()
            .map(|line| line.split_ascii_whitespace().collect())
            .collect();
        let boundary = build_preflight_boundary(&board, &rows, [0, 0]).unwrap();
        assert_eq!(boundary.segments.len(), 8);
        assert_eq!(
            boundary
                .segments
                .iter()
                .filter(|segment| matches!(segment, boardsi_model::BoundarySegment::Arc { .. }))
                .count(),
            4
        );
    }

    #[test]
    fn rejects_non_square_and_missing_fields_and_out_of_range() {
        assert_eq!(
            parse_cardinal(&["1", "2"], [0, 0]).unwrap_err(),
            "INVALID_OUTLINE_ARC_FIELDS"
        );
        assert_eq!(
            parse_cardinal(&["200", "100", "0", "900", "0", "0", "200", "100"], [0, 0])
                .unwrap_err(),
            "INVALID_OUTLINE_ARC_BOUNDS"
        );
        assert_eq!(
            parse_cardinal(
                &["200", "100", "0", "900", "0", "0", "200", "200"],
                [5_600_000, 0]
            )
            .unwrap_err(),
            "OUTLINE_COORD_RANGE"
        );
    }
}
