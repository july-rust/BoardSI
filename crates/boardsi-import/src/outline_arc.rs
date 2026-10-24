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
