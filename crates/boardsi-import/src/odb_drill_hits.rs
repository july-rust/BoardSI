//! D_056_08: ODB++ drill-layer Source P hit ↔ source-declared Excellon
//! tool-number audit. The resulting hits are observations, not Canonical vias,
//! plated barrels, physical connectivity edges, or SI.
use crate::odb_drill::{DrillIntent, SourceDrillLayer};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceDrillHit {
    pub layer_name: String,
    pub source_file: String,
    pub source_sha256: [u8; 32],
    /// Independently hashed declaration of the Excellon tool and layer span.
    pub tool_source_file: String,
    pub tool_source_sha256: [u8; 32],
    pub matrix_source_sha256: [u8; 32],
    pub matrix_source_line: usize,
    pub source_line: usize,
    /// 0-based actual P feature ordinal, distinct from source line.
    pub feature_index: usize,
    pub symbol_name: String,
    /// Exact source decimal with at most nine fractional digits.
    pub x: String,
    pub y: String,
    pub units: String,
    /// Explicit P dcode; 0 cannot identify a drill tool and fails closed.
    pub tool_number: u32,
    pub tool_source_line: usize,
    pub tool_intent: DrillIntent,
    pub tool_subtype: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HitError {
    pub line: usize,
    pub code: &'static str,
}
fn err(line: usize, code: &'static str) -> HitError {
    HitError { line, code }
}
const MAX_HITS: usize = 1_000_000;
const MAX_SYMBOLS: usize = 16384;

/// Validate a fixed-point source coordinate exactly, not via f64.
/// Coordinates are merely recorded; no source-space integer conversion is
/// necessary to preserve the signed textual source value and its unit.
fn decimal(raw: &str, allow_negative: bool) -> bool {
    let val = if allow_negative {
        raw.strip_prefix('-').unwrap_or(raw)
    } else {
        raw
    };
    if val.is_empty() || val.starts_with('+') {
        return false;
    }
    let (whole, fract) = val.split_once('.').unwrap_or((val, ""));
    if (whole.is_empty() && fract.is_empty()) || fract.len() > 9 {
        return false;
    }
    if !whole.bytes().all(|x| x.is_ascii_digit()) || !fract.bytes().all(|x| x.is_ascii_digit()) {
        return false;
    }
    // Bound physical coordinates to 1e9 source units and prevent huge strings.
    if whole.len() > 15 {
        return false;
    }
    let integer = if whole.is_empty() {
        0
    } else if let Ok(v) = whole.parse::<i64>() {
        v
    } else {
        return false;
    };
    integer <= 1_000_000_000
}
pub(super) fn parse_drill_hits(
    text: &str,
    layer: &SourceDrillLayer,
    feature_path: &str,
    source_sha256: [u8; 32],
) -> Result<Vec<SourceDrillHit>, HitError> {
    let tools: BTreeMap<u32, _> = layer.tools.iter().map(|t| (t.number, t)).collect();
    let mut symbols = BTreeMap::<u32, String>::new();
    let mut units: Option<&str> = None;
    let mut advertised_count: Option<usize> = None;
    let mut count = 0usize;
    let mut hits = Vec::new();
    let mut ids = BTreeSet::new();
    for (i, raw) in text.lines().enumerate() {
        let no = i + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(value) = line.strip_prefix("UNITS=") {
            if units.is_some() || count != 0 || !matches!(value, "MM" | "INCH") {
                return Err(err(no, "ODB_DRILL_HIT_UNITS"));
            }
            if value != layer.unit {
                return Err(err(no, "ODB_DRILL_HIT_TOOL_UNIT_CONFLICT"));
            }
            units = Some(value);
            continue;
        }
        if let Some(value) = line.strip_prefix("F ") {
            if advertised_count.is_some() || count != 0 {
                return Err(err(no, "ODB_DRILL_HIT_FEATURE_COUNT"));
            }
            advertised_count = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| err(no, "ODB_DRILL_HIT_BAD_COUNT"))?,
            );
            continue;
        }
        if let Some(decl) = line.strip_prefix('$') {
            if units.is_none() || count != 0 || symbols.len() >= MAX_SYMBOLS {
                return Err(err(no, "ODB_DRILL_HIT_SYMBOL_ORDER"));
            }
            let parts: Vec<&str> = decl.split_whitespace().collect();
            if !(2..=3).contains(&parts.len()) {
                return Err(err(no, "ODB_DRILL_HIT_BAD_SYMBOL"));
            }
            let number = parts[0]
                .parse::<u32>()
                .map_err(|_| err(no, "ODB_DRILL_HIT_BAD_SYMBOL"))?;
            if parts.len() == 3 && !matches!(parts[2], "I" | "M") {
                return Err(err(no, "ODB_DRILL_HIT_SYMBOL_UNIT"));
            }
            let diameter = parts[1]
                .strip_prefix('r')
                .filter(|rest| {
                    rest.as_bytes()
                        .first()
                        .is_some_and(|b| b.is_ascii_digit() || *b == b'.')
                })
                .ok_or_else(|| err(no, "ODB_DRILL_HIT_NONCIRCULAR_SYMBOL"))?;
            if !decimal(diameter, false) || diameter.bytes().all(|x| x == b'0' || x == b'.') {
                return Err(err(no, "ODB_DRILL_HIT_BAD_DIAMETER"));
            }
            if symbols.insert(number, parts[1].into()).is_some() {
                return Err(err(no, "ODB_DRILL_HIT_DUPLICATE_SYMBOL"));
            }
            continue;
        }
        if line.starts_with('@') || line.starts_with('&') || line.starts_with("ID=") {
            // Attribute lookup and file-ID cannot establish plated continuity.
            if count != 0 {
                return Err(err(no, "ODB_DRILL_HIT_LATE_LOOKUP"));
            }
            continue;
        }
        if units.is_none() {
            return Err(err(no, "ODB_DRILL_HIT_MISSING_UNITS"));
        }
        let (record, extras) = line.split_once(';').unwrap_or((line, ""));
        let tokens: Vec<&str> = record.split_whitespace().collect();
        if tokens.first() != Some(&"P") {
            // Drill layer containing L/A/S, negative features or undocumented
            // records cannot be reported as a fully enumerated hole inventory.
            return Err(err(no, "ODB_DRILL_HIT_UNSUPPORTED_RECORD"));
        }
        if tokens.len() != 7 {
            return Err(err(no, "ODB_DRILL_HIT_PAD_ARITY"));
        }
        if tokens[4] != "P" {
            return Err(err(no, "ODB_DRILL_HIT_NEGATIVE_PAD"));
        }
        if !decimal(tokens[1], true) || !decimal(tokens[2], true) {
            return Err(err(no, "ODB_DRILL_HIT_BAD_COORDINATE"));
        }
        let symbol_id = tokens[3]
            .parse::<u32>()
            .map_err(|_| err(no, "ODB_DRILL_HIT_BAD_SYMBOL_REF"))?;
        let symbol_name = symbols
            .get(&symbol_id)
            .ok_or_else(|| err(no, "ODB_DRILL_HIT_MISSING_SYMBOL"))?
            .clone();
        let dcode = tokens[5]
            .parse::<u32>()
            .map_err(|_| err(no, "ODB_DRILL_HIT_BAD_TOOL_NUMBER"))?;
        if dcode == 0 {
            return Err(err(no, "ODB_DRILL_HIT_UNASSIGNED_TOOL"));
        }
        let tool = tools
            .get(&dcode)
            .ok_or_else(|| err(no, "ODB_DRILL_HIT_UNKNOWN_TOOL"))?;
        if !matches!(tokens[6], "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7") {
            return Err(err(no, "ODB_DRILL_HIT_NONORTHOGONAL_ORIENTATION"));
        }
        for part in extras.split(';') {
            for atom in part.split(',') {
                if let Some(id) = atom.trim().strip_prefix("ID=") {
                    if id.is_empty() || !ids.insert(id.to_owned()) {
                        return Err(err(no, "ODB_DRILL_HIT_DUPLICATE_ID"));
                    }
                }
            }
        }
        if count >= MAX_HITS {
            return Err(err(no, "ODB_DRILL_HIT_LIMIT"));
        }
        hits.push(SourceDrillHit {
            layer_name: layer.name.clone(),
            source_file: feature_path.into(),
            source_sha256,
            tool_source_file: layer.source_file.clone(),
            tool_source_sha256: layer.source_sha256,
            matrix_source_sha256: layer.matrix_sha256,
            matrix_source_line: layer.matrix_source_line,
            source_line: no,
            feature_index: count,
            symbol_name,
            x: tokens[1].into(),
            y: tokens[2].into(),
            units: units.expect("verified").into(),
            tool_number: dcode,
            tool_source_line: tool.source_line,
            tool_intent: tool.intent,
            tool_subtype: tool.subtype.clone(),
        });
        count += 1;
    }
    if let Some(expected) = advertised_count {
        if expected != count {
            return Err(err(text.lines().count(), "ODB_DRILL_HIT_COUNT_MISMATCH"));
        }
    }
    if units.is_none() {
        return Err(err(0, "ODB_DRILL_HIT_MISSING_UNITS"));
    }
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::odb_drill::{DrillIntent, SourceDrillTool};
    fn layer() -> SourceDrillLayer {
        SourceDrillLayer {
            name: "drill_top_bottom".into(),
            matrix_row: 7,
            matrix_source_line: 11,
            matrix_sha256: [2; 32],
            unit: "MM".into(),
            subtype: "".into(),
            explicit_start_layer: Some("top".into()),
            explicit_end_layer: Some("bottom".into()),
            source_file: "steps/pcb/layers/drill_top_bottom/tools".into(),
            source_sha256: [3; 32],
            tools: vec![SourceDrillTool {
                number: 9,
                intent: DrillIntent::Via,
                subtype: "LASER".into(),
                finish_size: "100".into(),
                drill_size: Some("120".into()),
                min_tolerance: None,
                max_tolerance: None,
                bit: None,
                source_line: 4,
            }],
            geometry_hits: None,
        }
    }
    const VALID: &str =
        "UNITS=MM\nF 2\n$0 r100\nP 1.25 -0.25 0 P 9 0;ID=hole1\nP 2 0.125 0 P 9 0;ID=hole2\n";
    #[test]
    fn exact_source_pad_hits_and_tool_crossreference() {
        let v = parse_drill_hits(VALID, &layer(), "features", [7; 32]).unwrap();
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].x, "1.25");
        assert_eq!(v[0].y, "-0.25");
        assert_eq!(v[0].tool_source_line, 4);
        assert_eq!(v[0].feature_index, 0);
        assert_eq!(v[1].feature_index, 1);
        assert_eq!(v[0].source_sha256, [7; 32]);
        assert_eq!(v[0].tool_source_sha256, [3; 32]);
        assert_eq!(v[0].matrix_source_sha256, [2; 32]);
        assert_eq!(v[0].matrix_source_line, 11);
    }
    #[test]
    fn rejects_unknown_tool_units_bad_shapes_and_unsupported_drill_geometry() {
        for (input, expected) in [
            (
                VALID.replace("P 9 0", "P 0 0"),
                "ODB_DRILL_HIT_UNASSIGNED_TOOL",
            ),
            (
                VALID.replace("P 9 0", "P 3 0"),
                "ODB_DRILL_HIT_UNKNOWN_TOOL",
            ),
            (
                VALID.replace("UNITS=MM", "UNITS=INCH"),
                "ODB_DRILL_HIT_TOOL_UNIT_CONFLICT",
            ),
            (
                VALID.replace("$0 r100", "$0 rect100x100"),
                "ODB_DRILL_HIT_NONCIRCULAR_SYMBOL",
            ),
            (VALID.replace("F 2", "F 3"), "ODB_DRILL_HIT_COUNT_MISMATCH"),
            (
                VALID.replace("P 1.25", "L 1.25"),
                "ODB_DRILL_HIT_UNSUPPORTED_RECORD",
            ),
            (
                VALID.replace("ID=hole2", "ID=hole1"),
                "ODB_DRILL_HIT_DUPLICATE_ID",
            ),
            (VALID.replace("1.25", "1e2"), "ODB_DRILL_HIT_BAD_COORDINATE"),
        ] {
            let code = parse_drill_hits(&input, &layer(), "features", [8; 32])
                .unwrap_err()
                .code;
            assert_eq!(code, expected, "input={input}");
        }
    }
}
