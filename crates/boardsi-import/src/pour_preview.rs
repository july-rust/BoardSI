//! A_021: strict, non-electrical PADS V9.4 *POUR* POUROUT boundary preflight.
//! Reference: PADS Layout ASCII Format Specification 9.4, pp. 56-59.
//! POUROUT is a user-created outline, NOT proof of actual flooded copper.
//! Only one straight POLY piece is accepted. All other pour grammar fails closed.
use crate::outline_preview::{bounded, fixed, valid_ring};
use std::collections::BTreeSet;

const MAX_POURS: usize = 64;
const MAX_VERTICES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PourBoundary {
    pub name: String,
    pub net_label: Option<String>, // Claimed source label; never verified connectivity.
    pub layer: u32,
    pub vertices: Vec<[i64; 2]>, // Exact 0.01-mil ticks; closed polygon.
    pub source_line: usize,
}

fn token(s: &str, max: usize) -> bool {
    !s.is_empty() && s.len() <= max && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Parses only straight POUROUT boundary records. Does NOT return copper fill.
/// max_layer is the validated physical copper-layer count from the route importer.
pub fn parse(bytes: &[u8], max_layer: u32) -> Result<Vec<PourBoundary>, &'static str> {
    if bytes.len() > crate::route_preview::MAX_FILE_BYTES {
        return Err("INPUT_TOO_LARGE");
    }
    if !(1..=64).contains(&max_layer) {
        return Err("POUR_LAYER_RANGE");
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
    // Non-UTF8 CP1252 bytes outside the active POUR section are not interpreted.
    let source = String::from_utf8_lossy(bytes);
    let rows: Vec<&str> = source.lines().collect();
    let mut in_pour = false;
    let mut saw_end = false;
    let mut names = BTreeSet::new();
    let mut found = Vec::new();
    let mut i = 1usize;
    while i < rows.len() {
        if rows[i].len() > 16_384 {
            return Err("OVERLONG_LINE");
        }
        let parts: Vec<&str> = rows[i].split_ascii_whitespace().collect();
        if parts.is_empty() || matches!(parts[0], "*REMARK*" | "REMARK") {
            i += 1;
            continue;
        }
        if parts[0] == "*END*" {
            saw_end = true;
            break;
        }
        if parts[0].starts_with('*') {
            in_pour = parts[0] == "*POUR*";
            i += 1;
            continue;
        }
        if !in_pour {
            i += 1;
            continue;
        }
        // Grammar on p. 56: name type xloc yloc pieces flags
        // [ownername signame ...]. Owner/net labels are provenance, not connectivity.
        if (parts.len() != 6 && parts.len() != 8)
            || parts[1] != "POUROUT"
            || !token(parts[0], 16)
            || (parts.len() == 8
                && (!token(parts[6], 16) || !token(parts[7], 47) || parts[6] != parts[0]))
        {
            return Err("UNSUPPORTED_POUR_HEADER");
        }
        let name = parts[0].to_owned();
        let source_line = i + 1;
        if found.len() >= MAX_POURS || !names.insert(name.clone()) {
            return Err("POUR_COUNT_OR_DUPLICATE");
        }
        let origin = [bounded(fixed(parts[2])?)?, bounded(fixed(parts[3])?)?];
        if parts[4] != "1" || parts[5] != "0" {
            // Multi-piece/flagged pours may have unmodelled islands, fills and holes.
            return Err("UNSUPPORTED_POUR_PIECES_OR_FLAGS");
        }
        i += 1;
        let piece: Vec<&str> = rows
            .get(i)
            .ok_or("TRUNCATED_POUR")?
            .split_ascii_whitespace()
            .collect();
        if piece.len() != 5 || piece[0] != "POLY" || piece[2] != "0" {
            return Err("UNSUPPORTED_POUR_PIECE");
        }
        let n = piece[1].parse::<usize>().map_err(|_| "POUR_CORNER_COUNT")?;
        if !(4..=MAX_VERTICES).contains(&n) {
            return Err("POUR_CORNER_COUNT");
        }
        let width = fixed(piece[3])?;
        if !(1..=25_000).contains(&width) {
            return Err("POUR_WIDTH_RANGE");
        }
        let layer = piece[4].parse::<u32>().map_err(|_| "POUR_LAYER_RANGE")?;
        if !(1..=max_layer).contains(&layer) {
            return Err("POUR_LAYER_RANGE");
        }
        i += 1;
        let mut vertices = Vec::with_capacity(n);
        for _ in 0..n {
            let row = rows.get(i).ok_or("TRUNCATED_POUR")?;
            if row.len() > 16_384 {
                return Err("OVERLONG_LINE");
            }
            let xy: Vec<&str> = row.split_ascii_whitespace().collect();
            if xy.len() != 2 {
                return Err("UNSUPPORTED_POUR_ARC_OR_POINT");
            }
            let x = origin[0]
                .checked_add(fixed(xy[0])?)
                .ok_or("POUR_COORD_OVERFLOW")?;
            let y = origin[1]
                .checked_add(fixed(xy[1])?)
                .ok_or("POUR_COORD_OVERFLOW")?;
            vertices.push([bounded(x)?, bounded(y)?]);
            i += 1;
        }
        if !valid_ring(&vertices) {
            return Err("INVALID_POUR_RING");
        }
        found.push(PourBoundary {
            name,
            net_label: (parts.len() == 8).then(|| parts[7].to_owned()),
            layer,
            vertices,
            source_line,
        });
    }
    if !saw_end {
        return Err("MISSING_END_MARKER");
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n*PCB*\nMAXIMUMLAYER 2\n*POUR*\nPLANE1 POUROUT 100 200 1 0 PLANE1 GND\nPOLY 5 0 0.01 2\n0 0\n10 0\n10 20\n0 20\n0 0\n*END*\n";

    #[test]
    fn accepts_exact_ticks_and_only_boundary_semantics() {
        let p = parse(SAMPLE.as_bytes(), 2).unwrap();
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].name, "PLANE1");
        assert_eq!(p[0].net_label.as_deref(), Some("GND"));
        assert_eq!(p[0].layer, 2);
        assert_eq!(
            p[0].vertices,
            vec![
                [10000, 20000],
                [11000, 20000],
                [11000, 22000],
                [10000, 22000],
                [10000, 20000],
            ]
        );
    }

    #[test]
    fn empty_section_is_not_fabricated_copper() {
        assert!(parse(b"!PADS-POWERPCB-V9.4-MILS!\n*ROUTE*\n*END*\n", 2)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn rejects_non_boundary_and_multiflood_semantics() {
        for specimen in [
            SAMPLE.replace("POUROUT", "HATOUT"),
            SAMPLE.replace("POUROUT", "VOIDOUT"),
            SAMPLE.replace("POLY 5 0", "POLY 5 1"),
            SAMPLE.replace("200 1 0", "200 2 0"),
            SAMPLE.replace("200 1 0", "200 1 16"),
        ] {
            assert!(parse(specimen.as_bytes(), 2).is_err());
        }
    }

    #[test]
    fn rejects_unclosed_crossing_and_degenerate_rings() {
        for specimen in [
            SAMPLE.replace("0 20\n0 0", "0 20\n0 1"),
            SAMPLE.replace("10 0\n10 20\n0 20", "10 20\n10 0\n0 20"),
            SAMPLE.replace("10 20\n0 20", "0 0\n0 0"),
        ] {
            assert!(parse(specimen.as_bytes(), 2).is_err());
        }
    }

    #[test]
    fn rejects_truncation_wrong_layer_precision_and_duplicate_name() {
        assert!(parse(SAMPLE.as_bytes(), 1).is_err());
        assert!(parse(SAMPLE.replace("10 0", "10.001 0").as_bytes(), 2).is_err());
        assert!(parse(SAMPLE.trim_end_matches("*END*\n").as_bytes(), 2).is_err());
        let duplicate = SAMPLE.replace(
            "*END*",
            "PLANE1 POUROUT 0 0 1 0\nPOLY 4 0 0.01 1\n0 0\n1 0\n0 1\n0 0\n*END*",
        );
        assert!(parse(duplicate.as_bytes(), 2).is_err());
    }
}
