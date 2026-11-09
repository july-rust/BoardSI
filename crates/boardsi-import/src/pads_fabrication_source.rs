//! D_056_07: evidence-only PADS V9.4 MILS PARTDECAL drill/slot scanner.
//!
//! This is deliberately NOT a route-via parser and NOT a manufactured copper
//! continuity oracle. A decal padstack `N` is not evidence for NPTH syntax in
//! the separate *VIA* or *ROUTE* sections.
//!
//! Grammar: Mentor Graphics PADS Layout ASCII Format Specification 9.4,
//! Part Decal Definition Format, pp. 36-38 (PDF pages 35-37).
//! See docs/architecture/d056-07-fabrication-source.md for boundaries.
//! Conservative supported subset: I-unit decal, no extra graphics/attributes,
//! R source pad, top-side explicitly declared nonzero drill and P/N, optional
//! complete slotted tuple. Unmodeled optional fields fail closed.
use crate::digest::sha256;
use std::collections::BTreeSet;

const MAX_INPUT_BYTES: usize = 8 * 1024 * 1024;
const MAX_SOURCE_LINES: usize = 200_000;
const MAX_LINE_BYTES: usize = 4_096;
const MAX_RECORDS: usize = 100_000;
const MAX_TICKS: i64 = 100_000; // 1000 mil in 0.01-mil exact source units

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatingDeclaration {
    Plated,
    NonPlated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrillShape {
    Round {
        diameter_ticks: i64,
    },
    /// Geometry parameters only; the native slot "length" field has not
    /// independently proved how a CAM system constructs the endcap envelope.
    Slot {
        width_ticks: i64,
        source_length_ticks: i64,
        orientation_millidegrees: u32,
        offset_ticks: i64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartDecalDrillEvidence {
    pub decal: String,
    pub pin_selector: String,
    pub source_line: usize,
    pub source_sha256: [u8; 32],
    pub plating: PlatingDeclaration,
    pub shape: DrillShape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScanError {
    UnsupportedSource,
    UnsupportedDecal,
    UnsupportedPadstack,
    UnsupportedGeometry,
    Malformed,
    Truncated,
    TooLarge,
}

/// Parse unsigned/signed decimal MILS to a checked 0.01 MILS source grid.
/// Never round extra source precision to an apparent exact geometry.
fn ticks(raw: &str) -> Option<i64> {
    let (negative, digits) = match (raw.strip_prefix('-'), raw.strip_prefix('+')) {
        (Some(n), _) => (true, n),
        (None, Some(n)) => (false, n),
        _ => (false, raw),
    };
    let (whole, fractional) = digits.split_once('.').unwrap_or((digits, ""));
    if whole.is_empty()
        || !whole.bytes().all(|c| c.is_ascii_digit())
        || fractional.len() > 2
        || !fractional.bytes().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    let whole = whole.parse::<i64>().ok()?;
    let fraction = match fractional.len() {
        0 => 0,
        1 => fractional.parse::<i64>().ok()?.checked_mul(10)?,
        _ => fractional.parse::<i64>().ok()?,
    };
    let abs = whole.checked_mul(100)?.checked_add(fraction)?;
    if abs > MAX_TICKS * 2 {
        return None;
    }
    if negative {
        abs.checked_neg()
    } else {
        Some(abs)
    }
}

fn milli_degrees(raw: &str) -> Option<u32> {
    let (whole, frac) = raw.split_once('.').unwrap_or((raw, ""));
    if whole.is_empty()
        || !whole.bytes().all(|c| c.is_ascii_digit())
        || frac.len() > 3
        || !frac.bytes().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    let w = whole.parse::<u32>().ok()?;
    let f = match frac.len() {
        0 => 0,
        1 => frac.parse::<u32>().ok()?.checked_mul(100)?,
        2 => frac.parse::<u32>().ok()?.checked_mul(10)?,
        _ => frac.parse::<u32>().ok()?,
    };
    let value = w.checked_mul(1_000)?.checked_add(f)?;
    (value < 180_000).then_some(value)
}

fn simple_id(raw: &str, limit: usize) -> bool {
    !raw.is_empty()
        && raw.len() <= limit
        && raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}

fn drill_row(
    row: &str,
    decal: &str,
    selector: &str,
    source_line: usize,
    source_sha256: [u8; 32],
) -> Result<Option<PartDecalDrillEvidence>, ScanError> {
    let p: Vec<_> = row.split_ascii_whitespace().collect();
    if p.len() < 3 || p[0] != "-2" || p[2] != "R" {
        return Err(ScanError::UnsupportedGeometry);
    }
    let pad_width = ticks(p[1]).ok_or(ScanError::Malformed)?;
    if !(1..=MAX_TICKS).contains(&pad_width) {
        return Err(ScanError::Malformed);
    }
    // Existing PADS exports use an optional zero field for round pads without
    // a hole. Treat that spelling as an explicit *absence*, never as NPTH.
    if p.len() == 3 || (p.len() == 4 && p[3] == "0") {
        return Ok(None);
    }
    if !matches!(p.len(), 5 | 8) || !matches!(p[4], "P" | "N") {
        return Err(ScanError::UnsupportedPadstack);
    }
    let diameter = ticks(p[3]).ok_or(ScanError::Malformed)?;
    if !(1..=MAX_TICKS).contains(&diameter) {
        return Err(ScanError::Malformed);
    }
    let plating = match p[4] {
        "P" => PlatingDeclaration::Plated,
        "N" => PlatingDeclaration::NonPlated,
        _ => return Err(ScanError::UnsupportedPadstack),
    };
    let shape = if p.len() == 5 {
        DrillShape::Round {
            diameter_ticks: diameter,
        }
    } else {
        let angle = milli_degrees(p[5]).ok_or(ScanError::Malformed)?;
        let length = ticks(p[6]).ok_or(ScanError::Malformed)?;
        let offset = ticks(p[7]).ok_or(ScanError::Malformed)?;
        if !(100..=MAX_TICKS).contains(&length)
            || length < diameter
            || offset.unsigned_abs() > 50_000
            || offset.unsigned_abs() * 2 > length as u64
        {
            return Err(ScanError::Malformed);
        }
        DrillShape::Slot {
            width_ticks: diameter,
            source_length_ticks: length,
            orientation_millidegrees: angle,
            offset_ticks: offset,
        }
    };
    Ok(Some(PartDecalDrillEvidence {
        decal: decal.to_owned(),
        pin_selector: selector.to_owned(),
        source_line,
        source_sha256,
        plating,
        shape,
    }))
}

/// Strictly inspect source-declared part-decal drill records. Any unsupported
/// geometry/grammar in a scanned decal aborts the entire result transaction;
/// callers must not infer connection, manufacture, or electrical continuity.
pub fn scan_pads_v94_part_decal_drills(
    source: &[u8],
) -> Result<Vec<PartDecalDrillEvidence>, ScanError> {
    if source.len() > MAX_INPUT_BYTES {
        return Err(ScanError::TooLarge);
    }
    let text = std::str::from_utf8(source).map_err(|_| ScanError::UnsupportedSource)?;
    let mut iter = text.lines().enumerate();
    let (_, header) = iter.next().ok_or(ScanError::Truncated)?;
    if !header.starts_with("!PADS-POWERPCB-V9.4-MILS!") {
        return Err(ScanError::UnsupportedSource);
    }
    let lines: Vec<_> = std::iter::once((0, header))
        .chain(iter)
        .take(MAX_SOURCE_LINES + 1)
        .collect();
    if lines.len() > MAX_SOURCE_LINES || lines.iter().any(|(_, s)| s.len() > MAX_LINE_BYTES) {
        return Err(ScanError::TooLarge);
    }
    let digest = sha256(source);
    let mut output = Vec::new();
    let mut i = 1;
    let mut saw_decal = false;
    let mut saw_end = false;
    let mut seen_decal_names = BTreeSet::new();
    while i < lines.len() {
        let row = lines[i].1.trim();
        let token = row.split_ascii_whitespace().next().unwrap_or("");
        if token == "*END*" {
            saw_end = true;
            break;
        }
        if token != "*PARTDECAL*" {
            i += 1;
            continue;
        }
        saw_decal = true;
        i += 1;
        while i < lines.len() {
            let row = lines[i].1.trim();
            let token = row.split_ascii_whitespace().next().unwrap_or("");
            if token == "*REMARK*" || row.is_empty() || token == "REMARK" {
                i += 1;
                continue;
            }
            if token.starts_with('*') {
                // Never allow malformed/unknown section markers to terminate
                // this decal quietly and hide a later drilled-pad record.
                const NEXT_SECTIONS: &[&str] = &[
                    "*PARTTYPE*",
                    "*PART*",
                    "*VIA*",
                    "*ROUTE*",
                    "*LINES*",
                    "*POUR*",
                    "*COPPER*",
                    "*COPCUT*",
                    "*TEXT*",
                    "*PCB*",
                    "*NET*",
                    "*SIGNAL*",
                    "*PARTDECAL*",
                    "*END*",
                ];
                if row != token || !NEXT_SECTIONS.contains(&token) {
                    return Err(ScanError::UnsupportedDecal);
                }
                break;
            }
            let header: Vec<_> = row.split_ascii_whitespace().collect();
            if !(7..=9).contains(&header.len())
                || header[1] != "I"
                || header[4] != "0"
                || ticks(header[2]).is_none()
                || ticks(header[3]).is_none()
                || ticks(header[4]).is_none()
                || !simple_id(header[0], 40)
                || header.get(7).is_some_and(|x| *x != "0")
                || header.get(8).is_some_and(|x| *x != "0")
            {
                return Err(ScanError::UnsupportedDecal);
            }
            let terminal_count = header[5]
                .parse::<usize>()
                .map_err(|_| ScanError::Malformed)?;
            let stack_count = header[6]
                .parse::<usize>()
                .map_err(|_| ScanError::Malformed)?;
            if !(1..=256).contains(&terminal_count) || !(1..=256).contains(&stack_count) {
                return Err(ScanError::UnsupportedDecal);
            }
            let decal = header[0];
            if !seen_decal_names.insert(decal.to_owned()) {
                return Err(ScanError::Malformed);
            }
            i += 1;
            if i + terminal_count > lines.len() {
                return Err(ScanError::Truncated);
            }
            let mut seen_terminals = BTreeSet::new();
            for _ in 0..terminal_count {
                let t: Vec<_> = lines[i].1.split_ascii_whitespace().collect();
                if t.len() != 5
                    || !t[0].starts_with('T')
                    || ticks(&t[0][1..]).is_none()
                    || ticks(t[1]).is_none()
                    || ticks(t[2]).is_none()
                    || ticks(t[3]).is_none()
                    || !simple_id(t[4], 7)
                    || !seen_terminals.insert(t[4])
                {
                    return Err(ScanError::UnsupportedDecal);
                }
                i += 1;
            }
            let mut seen_stack_selectors = BTreeSet::new();
            for _ in 0..stack_count {
                if i >= lines.len() {
                    return Err(ScanError::Truncated);
                }
                let p: Vec<_> = lines[i].1.split_ascii_whitespace().collect();
                if p.len() != 3
                    || p[0] != "PAD"
                    || !simple_id(p[1], 7)
                    || !seen_stack_selectors.insert(p[1])
                {
                    return Err(ScanError::UnsupportedPadstack);
                }
                let count = p[2].parse::<usize>().map_err(|_| ScanError::Malformed)?;
                if count != 3 || i + count >= lines.len() {
                    return Err(ScanError::Truncated);
                }
                i += 1;
                let mut seen_layers = BTreeSet::new();
                for _ in 0..count {
                    let raw = lines[i].1.trim();
                    let fields: Vec<_> = raw.split_ascii_whitespace().collect();
                    if fields.is_empty() || !seen_layers.insert(fields[0]) {
                        return Err(ScanError::Malformed);
                    }
                    if fields[0] == "-2" {
                        if let Some(drill) = drill_row(raw, decal, p[1], lines[i].0 + 1, digest)? {
                            output.push(drill);
                            if output.len() > MAX_RECORDS {
                                return Err(ScanError::TooLarge);
                            }
                        }
                    } else {
                        // Refuse unrecognized, potentially conductive/drilled
                        // syntax; never silently discard another drilled row.
                        if fields.len() != 3 && fields.len() != 4 {
                            return Err(ScanError::UnsupportedPadstack);
                        }
                        if !matches!(fields[0], "-1" | "0")
                            || fields[2] != "R"
                            || (fields.len() == 4 && fields[3] != "0")
                            || ticks(fields[1]) != Some(0)
                        {
                            return Err(ScanError::UnsupportedPadstack);
                        }
                    }
                    i += 1;
                }
                if !seen_layers.contains("-2")
                    || !seen_layers.contains("-1")
                    || !seen_layers.contains("0")
                {
                    return Err(ScanError::UnsupportedPadstack);
                }
            }
            // PADS PARTDECAL: PAD 0 is the default; otherwise each declared
            // terminal must have its own explicit matching selector.
            if !seen_stack_selectors.contains("0")
                && seen_terminals
                    .iter()
                    .any(|terminal| !seen_stack_selectors.contains(*terminal))
            {
                return Err(ScanError::UnsupportedPadstack);
            }
        }
    }
    if !saw_decal || !saw_end {
        return Err(ScanError::Truncated);
    }
    Ok(output)
}

impl PartDecalDrillEvidence {
    /// Convert SOURCE units to metres exactly. The caller must provide an
    /// independently validated physical drill span; a component-decal
    /// footprint alone is insufficient to infer its manufacturing depth.
    pub fn to_canonical_hole(
        &self,
        span: boardsi_model::PhysicalLayerSpan,
        copper_layers: u32,
    ) -> Result<boardsi_model::FabricationHole, boardsi_model::ModelError> {
        use boardsi_model::{
            FabricationHole, FabricationHoleShape, FabricationOperation, FabricationPlating,
            FabricationSourceEvidence, Rational,
        };
        fn metres(ticks: i64) -> Result<Rational, boardsi_model::ModelError> {
            Rational::new(i128::from(ticks) * 127, 500_000_000)
        }
        let shape = match self.shape {
            DrillShape::Round { diameter_ticks } => FabricationHoleShape::Round {
                diameter_m: metres(diameter_ticks)?,
            },
            DrillShape::Slot {
                width_ticks,
                source_length_ticks,
                orientation_millidegrees,
                offset_ticks,
            } => FabricationHoleShape::Slot {
                width_m: metres(width_ticks)?,
                declared_length_m: metres(source_length_ticks)?,
                orientation_millidegrees,
                offset_along_axis_m: metres(offset_ticks)?,
            },
        };
        let hole = FabricationHole {
            shape,
            plating: match self.plating {
                PlatingDeclaration::Plated => FabricationPlating::SourceDeclaredPlated,
                PlatingDeclaration::NonPlated => FabricationPlating::NonPlated,
            },
            span,
            operation: FabricationOperation::Drilled,
            evidence: FabricationSourceEvidence {
                format: "PADS-POWERPCB-V9.4-MILS-PARTDECAL".into(),
                source_sha256: self.source_sha256,
                source_line: self.source_line,
                qualified_native_grammar: true,
            },
        };
        hole.validate(copper_layers)?;
        Ok(hole)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(top_row: &str) -> Vec<u8> {
        format!(
            "!PADS-POWERPCB-V9.4-MILS!\n*PARTDECAL*\nD7 I 1000 1000 0 1 1 0 0\nT0 0 0 0 1\nPAD 0 3\n{top_row}\n-1 0 R 0\n0 0 R 0\n*END*\n"
        )
        .into_bytes()
    }

    #[test]
    fn source_confirmed_round_npth_and_plated() {
        let npth = fixture("-2 40 R 20 N");
        let rows = scan_pads_v94_part_decal_drills(&npth).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].source_line, 6);
        assert_eq!(rows[0].source_sha256, sha256(&npth));
        assert_eq!(rows[0].plating, PlatingDeclaration::NonPlated);
        assert_eq!(
            rows[0].shape,
            DrillShape::Round {
                diameter_ticks: 2000
            }
        );
        let pth = fixture("-2 40 R 20 P");
        assert_eq!(
            scan_pads_v94_part_decal_drills(&pth).unwrap()[0].plating,
            PlatingDeclaration::Plated
        );
        assert!(scan_pads_v94_part_decal_drills(&fixture("-2 40 R 0"))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn slot_source_dimensions_not_fabricated_copper() {
        let rows =
            scan_pads_v94_part_decal_drills(&fixture("-2 40 R 20 N 179.999 65.25 -5")).unwrap();
        assert_eq!(
            rows[0].shape,
            DrillShape::Slot {
                width_ticks: 2000,
                source_length_ticks: 6525,
                orientation_millidegrees: 179_999,
                offset_ticks: -500,
            }
        );
    }

    #[test]
    fn remarks_do_not_hide_source_declared_drills() {
        let baseline = String::from_utf8(fixture("-2 40 R 20 N")).unwrap();
        let with_remark = baseline.replace(
            "*PARTDECAL*\n",
            "*PARTDECAL*\n*REMARK* This is a legal PADS remark\n",
        );
        let rows = scan_pads_v94_part_decal_drills(with_remark.as_bytes()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].source_line, 7);
        assert_eq!(rows[0].plating, PlatingDeclaration::NonPlated);
    }

    #[test]
    fn malformed_markers_cannot_hide_subsequent_holes() {
        let baseline = String::from_utf8(fixture("-2 40 R 20 N")).unwrap();
        for line in ["*REMARK*oops", "*BOGUS*", "*PARTTYPE*trailing"] {
            let candidate = baseline.replace("*PARTDECAL*\n", &format!("*PARTDECAL*\n{line}\n"));
            assert!(
                scan_pads_v94_part_decal_drills(candidate.as_bytes()).is_err(),
                "{line}"
            );
        }
    }

    #[test]
    fn source_padstack_selectors_cover_all_declared_terminals() {
        let baseline = String::from_utf8(fixture("-2 40 R 20 N")).unwrap();
        let unmatched = baseline.replace("PAD 0 3", "PAD 2 3");
        assert_eq!(
            scan_pads_v94_part_decal_drills(unmatched.as_bytes()),
            Err(ScanError::UnsupportedPadstack)
        );
        let matched = baseline.replace("PAD 0 3", "PAD 1 3");
        assert_eq!(
            scan_pads_v94_part_decal_drills(matched.as_bytes())
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn slot_offset_must_fit_within_declared_drill_extent() {
        for bad in [
            "-2 40 R 20 N 0 20 500",
            "-2 40 R 20 N 0 65.25 32.63",
            "-2 40 R 20 N 90 40 -20.01",
        ] {
            assert!(
                scan_pads_v94_part_decal_drills(&fixture(bad)).is_err(),
                "{bad}"
            );
        }
        assert!(
            scan_pads_v94_part_decal_drills(&fixture("-2 40 R 20 N 0 65.25 32.62")).is_ok()
        );
    }

    #[test]
    fn incomplete_or_duplicated_padstack_rows_are_rejected() {
        let baseline = String::from_utf8(fixture("-2 40 R 20 N")).unwrap();
        for broken in [
            baseline.replace("0 0 R 0", "-1 0 R 0"),
            baseline.replace("-1 0 R 0", "0 0 R 0"),
            baseline.replace("-1 0 R 0", "-1 20 R 0"),
            baseline.replace("PAD 0 3", "PAD 0 4"),
        ] {
            assert!(scan_pads_v94_part_decal_drills(broken.as_bytes()).is_err());
        }
    }

    #[test]
    fn terminal_coordinates_must_be_source_numeric() {
        let baseline = String::from_utf8(fixture("-2 40 R 20 N")).unwrap();
        for terminal in [
            "TotallyInvalid bad bad bad 1",
            "T0 bad 0 0 1",
            "T0 0 bad 0 1",
            "T0 0 0 bad 1",
            "T1.234 0 0 0 1",
        ] {
            let candidate = baseline.replace("T0 0 0 0 1", terminal);
            assert!(
                scan_pads_v94_part_decal_drills(candidate.as_bytes()).is_err(),
                "{terminal}"
            );
        }
        let invalid_header = baseline.replace("D7 I 1000 1000 0", "D7 I bad 1000 0");
        assert!(scan_pads_v94_part_decal_drills(invalid_header.as_bytes()).is_err());
    }

    #[test]
    fn fail_closed_on_missing_malformed_or_unsupported_rows() {
        for bad in [
            "-2 40 R 20 X",
            "-2 40 R 20 N 180 65 0",
            "-2 40 R 20 N 45.0001 65 0",
            "-2 40 R 20 N 45 19 0",
            "-2 40 R 20 N 45 65 600",
            "-2 40 R 20 N 45 65",
            "-2 40 R 20",
            "-2 40 S 20 N",
            "-2 40 R 0 N",
            "-2 40 R 20 N 45 65 0 extra",
        ] {
            assert!(
                scan_pads_v94_part_decal_drills(&fixture(bad)).is_err(),
                "{bad}"
            );
        }
        let mut truncated = fixture("-2 40 R 20 N");
        truncated.truncate(truncated.len() - "*END*\n".len());
        assert_eq!(
            scan_pads_v94_part_decal_drills(&truncated),
            Err(ScanError::Truncated)
        );
        let mut changed = fixture("-2 40 R 20 N");
        changed[0] = b'?';
        assert_eq!(
            scan_pads_v94_part_decal_drills(&changed),
            Err(ScanError::UnsupportedSource)
        );
    }

    #[test]
    fn exact_canonical_hole_preserves_npth_and_slot_across_stackups() {
        let record = scan_pads_v94_part_decal_drills(&fixture("-2 40 R 20 N 45.125 65.25 0"))
            .unwrap()
            .remove(0);
        for layers in [2, 4, 6, 8, 64] {
            let span = boardsi_model::PhysicalLayerSpan {
                first: 1,
                last: layers,
            };
            let hole = record.to_canonical_hole(span, layers).unwrap();
            assert_eq!(hole.plating, boardsi_model::FabricationPlating::NonPlated);
            assert!(!hole.has_verified_manufactured_barrel());
            if let boardsi_model::FabricationHoleShape::Slot {
                width_m,
                declared_length_m,
                orientation_millidegrees,
                ..
            } = hole.shape
            {
                assert_eq!(
                    width_m,
                    boardsi_model::Rational::new(127 * 2000, 500_000_000).unwrap()
                );
                assert_eq!(
                    declared_length_m,
                    boardsi_model::Rational::new(127 * 6525, 500_000_000).unwrap()
                );
                assert_eq!(orientation_millidegrees, 45_125);
            } else {
                panic!("slot was not preserved");
            }
        }
    }

    #[test]
    fn source_mutation_changes_digest_no_connectivity_claim() {
        let original = scan_pads_v94_part_decal_drills(&fixture("-2 40 R 20 N")).unwrap();
        let changed = scan_pads_v94_part_decal_drills(&fixture("-2 41 R 20 N")).unwrap();
        assert_ne!(original[0].source_sha256, changed[0].source_sha256);
        assert_eq!(original[0].plating, changed[0].plating);
    }
}
