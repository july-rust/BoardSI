//! D_056_08 source-evidence-only drill tools contract.
//! Official ODB++Design 8.1 Update 4, layer tools pp. 184–186.
//! No drill-hit positions, manufactured barrels or electrical edges are inferred.
use std::collections::{BTreeMap, BTreeSet};

const MAX_TOOLS: usize = 2048;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrillIntent {
    /// SOURCE declared plated; not independent as-manufactured proof.
    Plated,
    NonPlated,
    /// VIA category alone does not prove production plating or connected copper.
    Via,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceDrillTool {
    pub number: u32,
    pub intent: DrillIntent,
    pub subtype: String,
    /// FINISH_SIZE is in micrometres (UNITS=MM) or mils (UNITS=INCH).
    /// -1 records a source-unspecified finished hole size.
    pub finish_size: String,
    pub drill_size: Option<String>,
    pub min_tolerance: Option<String>,
    pub max_tolerance: Option<String>,
    pub bit: Option<String>,
    pub source_line: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceDrillLayer {
    pub name: String,
    pub matrix_row: u32,
    pub unit: String,
    pub subtype: String,
    /// Only explicit, validated copper-layer start/end names are qualified.
    /// Missing names remain None, never guessed to mean copper span.
    pub explicit_start_layer: Option<String>,
    pub explicit_end_layer: Option<String>,
    pub source_file: String,
    pub source_sha256: [u8; 32],
    pub tools: Vec<SourceDrillTool>,
    pub geometry_hits: Option<usize>, // NONE; drill features not independently mapped.
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ToolError {
    pub line: usize,
    pub code: &'static str,
}
fn err(line: usize, code: &'static str) -> ToolError {
    ToolError { line, code }
}
fn decimal(raw: &str, allow_minus_one: bool) -> bool {
    if allow_minus_one && raw == "-1" {
        return true;
    }
    if raw.is_empty()
        || raw.starts_with('-')
        || raw.starts_with('+')
        || raw.contains('e')
        || raw.contains('E')
    {
        return false;
    }
    let (whole, frac) = raw.split_once('.').unwrap_or((raw, ""));
    if whole.is_empty()
        || frac.len() > 9
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !frac.bytes().all(|b| b.is_ascii_digit())
    {
        return false;
    }
    let value = match whole.parse::<i64>() {
        Ok(x) if x <= 1_000_000_000 => x,
        _ => return false,
    };
    value > 0 || frac.bytes().any(|x| x != b'0')
}
fn complete(
    fields: &BTreeMap<String, String>,
    line: usize,
    seen: &mut BTreeSet<u32>,
) -> Result<SourceDrillTool, ToolError> {
    let get = |key| fields.get(key).map(String::as_str);
    let number = get("NUM")
        .ok_or_else(|| err(line, "ODB_TOOL_MISSING_NUM"))?
        .parse::<u32>()
        .map_err(|_| err(line, "ODB_TOOL_BAD_NUM"))?;
    if number == 0 || !seen.insert(number) {
        return Err(err(line, "ODB_TOOL_DUPLICATE_OR_ZERO_NUM"));
    }
    let intent = match get("TYPE") {
        Some("PLATED") => DrillIntent::Plated,
        Some("NON_PLATED") => DrillIntent::NonPlated,
        Some("VIA") => DrillIntent::Via,
        _ => return Err(err(line, "ODB_TOOL_UNSUPPORTED_TYPE")),
    };
    let subtype = get("TYPE2").unwrap_or("STANDARD");
    let good_subtype = match intent {
        DrillIntent::Plated => matches!(subtype, "STANDARD" | "PRESS_FIT"),
        DrillIntent::NonPlated => subtype == "STANDARD",
        DrillIntent::Via => matches!(subtype, "STANDARD" | "PHOTO" | "LASER"),
    };
    if !good_subtype {
        return Err(err(line, "ODB_TOOL_INVALID_TYPE2"));
    }
    let finished = get("FINISH_SIZE").ok_or_else(|| err(line, "ODB_TOOL_MISSING_FINISH_SIZE"))?;
    if finished != "-1" && !decimal(finished, false) {
        return Err(err(line, "ODB_TOOL_BAD_FINISH_SIZE"));
    }
    if let Some(size) = get("DRILL_SIZE") {
        if !decimal(size, false) {
            return Err(err(line, "ODB_TOOL_BAD_DRILL_SIZE"));
        }
    }
    for k in ["MIN_TOL", "MAX_TOL"] {
        if let Some(v) = get(k) {
            if !decimal(v, true) && v != "0" {
                return Err(err(line, "ODB_TOOL_BAD_TOLERANCE"));
            }
        }
    }
    Ok(SourceDrillTool {
        number,
        intent,
        subtype: subtype.into(),
        finish_size: finished.into(),
        drill_size: get("DRILL_SIZE").map(str::to_owned),
        min_tolerance: get("MIN_TOL").map(str::to_owned),
        max_tolerance: get("MAX_TOL").map(str::to_owned),
        bit: get("BIT").map(str::to_owned),
        source_line: line,
    })
}
/// Fail-closed parser of the bounded structured layer/tools file.
pub(super) fn parse_tools(text: &str) -> Result<(String, Vec<SourceDrillTool>), ToolError> {
    let mut units: Option<String> = None;
    let mut pending_open = false;
    let mut active: Option<(BTreeMap<String, String>, usize)> = None;
    let mut tools = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, raw) in text.lines().enumerate() {
        let no = index + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if pending_open {
            if line != "{" {
                return Err(err(no, "ODB_TOOL_EXPECTED_OPEN"));
            }
            pending_open = false;
            active = Some((BTreeMap::new(), no));
            continue;
        }
        if line == "TOOLS {" || line == "TOOLS" {
            if active.is_some() || units.is_none() {
                return Err(err(no, "ODB_TOOL_BAD_BLOCK_ORDER"));
            }
            if line == "TOOLS" {
                pending_open = true;
            } else {
                active = Some((BTreeMap::new(), no));
            }
            continue;
        }
        if line == "}" {
            let (fields, start_line) = active
                .take()
                .ok_or_else(|| err(no, "ODB_TOOL_UNMATCHED_CLOSE"))?;
            if tools.len() >= MAX_TOOLS {
                return Err(err(no, "ODB_TOOL_LIMIT"));
            }
            tools.push(complete(&fields, start_line, &mut seen)?);
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| err(no, "ODB_TOOL_BAD_RECORD"))?;
        let key = key.trim();
        let value = value.trim();
        if let Some((ref mut fields, _)) = active {
            if !matches!(
                key,
                "NUM"
                    | "TYPE"
                    | "TYPE2"
                    | "MIN_TOL"
                    | "MAX_TOL"
                    | "BIT"
                    | "FINISH_SIZE"
                    | "DRILL_SIZE"
            ) {
                return Err(err(no, "ODB_TOOL_UNKNOWN_FIELD"));
            }
            if fields.insert(key.into(), value.into()).is_some() {
                return Err(err(no, "ODB_TOOL_DUPLICATE_FIELD"));
            }
        } else {
            match key {
                "UNITS" if units.is_none() && tools.is_empty() => {
                    if !matches!(value, "MM" | "INCH") {
                        return Err(err(no, "ODB_TOOL_UNSUPPORTED_UNITS"));
                    }
                    units = Some(value.into());
                }
                "THICKNESS" if units.is_some() => {
                    // OBSOLETE: do not turn this into a valid BoardStackup or z estimate.
                    if !decimal(value, false) {
                        return Err(err(no, "ODB_TOOL_BAD_OBSOLETE_THICKNESS"));
                    }
                }
                "USER_PARAMS" if units.is_some() => {
                    if value.len() > 2048 {
                        return Err(err(no, "ODB_TOOL_USER_PARAMS_LIMIT"));
                    }
                }
                _ => return Err(err(no, "ODB_TOOL_UNSUPPORTED_HEADER")),
            }
        }
    }
    if pending_open || active.is_some() {
        return Err(err(text.lines().count(), "ODB_TOOL_UNCLOSED_BLOCK"));
    }
    Ok((
        units.ok_or_else(|| err(0, "ODB_TOOL_MISSING_UNITS"))?,
        tools,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    const GOOD: &str = "UNITS=MM\nTOOLS {\nNUM=1\nTYPE=VIA\nTYPE2=LASER\nMIN_TOL=0\nMAX_TOL=0\nBIT=\nFINISH_SIZE=100\nDRILL_SIZE=125\n}\nTOOLS\n{\nNUM=2\nTYPE=NON_PLATED\nTYPE2=STANDARD\nFINISH_SIZE=-1\n}\n";
    #[test]
    fn two_tools_exact_intent_and_unknown_finished_size() {
        let (u, t) = parse_tools(GOOD).unwrap();
        assert_eq!(u, "MM");
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].intent, DrillIntent::Via);
        assert_eq!(t[0].subtype, "LASER");
        assert_eq!(t[0].drill_size.as_deref(), Some("125"));
        assert_eq!(t[1].intent, DrillIntent::NonPlated);
        assert_eq!(t[1].finish_size, "-1");
        assert_eq!(t[0].source_line, 2);
        assert_eq!(t[1].source_line, 13);
    }
    #[test]
    fn invalid_tool_units_types_or_sizes_fail_closed() {
        for (input, code) in [
            (GOOD.replace("UNITS=MM\n", ""), "ODB_TOOL_BAD_BLOCK_ORDER"),
            (
                GOOD.replace("UNITS=MM", "UNITS=CM"),
                "ODB_TOOL_UNSUPPORTED_UNITS",
            ),
            (
                GOOD.replace("TYPE2=LASER", "TYPE2=PRESS_FIT"),
                "ODB_TOOL_INVALID_TYPE2",
            ),
            (
                GOOD.replace("NUM=2", "NUM=1"),
                "ODB_TOOL_DUPLICATE_OR_ZERO_NUM",
            ),
            (
                GOOD.replace("FINISH_SIZE=100", "FINISH_SIZE=-2"),
                "ODB_TOOL_BAD_FINISH_SIZE",
            ),
            (
                GOOD.replace("DRILL_SIZE=125", "DRILL_SIZE=1e3"),
                "ODB_TOOL_BAD_DRILL_SIZE",
            ),
            (
                GOOD.replace("FINISH_SIZE=100", "FINISH_SIZE=100\nBAD=1"),
                "ODB_TOOL_UNKNOWN_FIELD",
            ),
            (
                GOOD.replace("TOOLS {\nNUM=1", "TOOLS {\nNUM=1\nNUM=1"),
                "ODB_TOOL_DUPLICATE_FIELD",
            ),
            (
                GOOD.replace("FINISH_SIZE=-1\n}", "FINISH_SIZE=-1"),
                "ODB_TOOL_UNCLOSED_BLOCK",
            ),
        ] {
            let err = parse_tools(&input).unwrap_err();
            assert_eq!(err.code, code, "Unexpected error for input={input}");
        }
    }
    #[test]
    fn declared_plated_intent_is_not_fabricated_connectivity() {
        let (_, t) = parse_tools(
            "UNITS=INCH\nTOOLS {\nNUM=9\nTYPE=PLATED\nTYPE2=PRESS_FIT\nFINISH_SIZE=11.5\n}\n",
        )
        .unwrap();
        assert_eq!(t[0].intent, DrillIntent::Plated);
        assert_eq!(t[0].finish_size, "11.5"); // mils, not 11.5 inches!
    }
}
