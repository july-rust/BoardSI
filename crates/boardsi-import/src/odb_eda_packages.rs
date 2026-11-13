//! E_018: strict source-only ODB++ EDA PKG/PIN rectangles.
//! Siemens ODB++Design 8.1 Update 4, eda/data pp.112-115.
//! No component instance, pad, plated via, electrical graph, or SI is inferred.
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRectangle {
    /// Exact 1e-9 source-unit ticks; convert using the audit's metres_per_tick.
    pub lower_left: [i64; 2],
    pub size: [i64; 2],
    pub source_line: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePin {
    pub name: String,
    pub ordinal: usize,
    pub center: [i64; 2],
    pub finished_hole_ticks: i64,
    pub pin_type: char,
    pub electrical_type: char,
    pub mount_type: char,
    pub source_uid: Option<u32>,
    pub source_line: usize,
    pub outlines: Vec<SourceRectangle>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePackage {
    pub name: String,
    /// Zero-based PKG order, not component-placement index.
    pub ordinal: usize,
    pub pitch_ticks: i64,
    pub bounds: [i64; 4],
    pub source_uid: Option<u32>,
    pub source_line: usize,
    pub outlines: Vec<SourceRectangle>,
    pub pins: Vec<SourcePin>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageError {
    pub line: usize,
    pub code: &'static str,
}
fn err(line: usize, code: &'static str) -> PackageError {
    PackageError { line, code }
}
const MAX_PACKAGES: usize = 4096;
const MAX_PINS: usize = 65536;
const MAX_OUTLINES: usize = 131072;

fn decimal(raw: &str, line: usize) -> Result<i64, PackageError> {
    // Reuse the importer-wide exact decimal grammar; never use f64.
    super::odb_source::decimal_ticks(raw, "eda/data", line)
        .map_err(|_| err(line, "ODB_EDA_PKG_BAD_EXACT_COORDINATE"))
}
fn printable_name(s: &str) -> bool {
    !s.is_empty() && s.len() <= 256 && s.bytes().all(|c| (33..=126).contains(&c) && c != b';')
}
fn head_and_uid(line: &str, no: usize) -> Result<(Vec<&str>, Option<u32>), PackageError> {
    let mut fields = line.split(';');
    let mut words: Vec<&str> = fields.next().unwrap_or("").split_whitespace().collect();
    let mut uid: Option<u32> = None;
    // The official syntax allows ID=<id> after ';', and PIN often uses a bare ID.
    if words.last().is_some_and(|s| s.starts_with("ID=")) {
        let val = words.pop().expect("present");
        uid = Some(parse_uid(val, no)?);
    }
    for extra in fields {
        let extra = extra.trim();
        if extra.is_empty() {
            continue;
        }
        if !extra.starts_with("ID=") || extra.split_whitespace().count() != 1 || uid.is_some() {
            return Err(err(no, "ODB_EDA_PKG_UNSUPPORTED_ATTRIBUTE"));
        }
        uid = Some(parse_uid(extra, no)?);
    }
    Ok((words, uid))
}
fn parse_uid(word: &str, no: usize) -> Result<u32, PackageError> {
    let value = word
        .strip_prefix("ID=")
        .ok_or_else(|| err(no, "ODB_EDA_PKG_BAD_UID"))?;
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(err(no, "ODB_EDA_PKG_BAD_UID"));
    }
    value
        .parse::<u32>()
        .map_err(|_| err(no, "ODB_EDA_PKG_BAD_UID"))
}

/// Parse solely SOURCE package/pin *definitions*. Placement and Net are separate evidence.
/// This parser sees the full EDA file, so an orphan outline or malformed package cannot be ignored.
pub fn parse_packages(text: &str) -> Result<Vec<SourcePackage>, PackageError> {
    let mut result: Vec<SourcePackage> = Vec::new();
    let mut pkg: Option<usize> = None;
    let mut pin: Option<usize> = None;
    let mut require_immediate_pkg_rc = false;
    let mut units = false;
    let mut all_uids = BTreeSet::<u32>::new();
    let mut pin_names = BTreeSet::<(usize, String)>::new();
    let mut total_pins = 0usize;
    let mut total_rects = 0usize;
    for (i, raw) in text.lines().enumerate() {
        let no = i + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let kind = line.split_whitespace().next().unwrap_or("");
        if require_immediate_pkg_rc && kind != "RC" {
            if matches!(kind, "CR" | "SQ" | "CT" | "OB" | "OS" | "OC" | "OE" | "CE") {
                return Err(err(no, "ODB_EDA_PKG_OUTLINE_UNSUPPORTED"));
            }
            return Err(err(no, "ODB_EDA_PKG_IMMEDIATE_OUTLINE_REQUIRED"));
        }
        if kind != "RC" && kind != "PRP" {
            if let (Some(pi), Some(ni)) = (pkg, pin) {
                if result[pi].pins[ni].outlines.is_empty() {
                    return Err(err(no, "ODB_EDA_PIN_MISSING_OUTLINE"));
                }
            }
        }
        match kind {
            "PKG" => {
                if !units {
                    return Err(err(no, "ODB_EDA_PKG_UNITS_REQUIRED"));
                }
                if result.len() >= MAX_PACKAGES {
                    return Err(err(no, "ODB_EDA_PKG_LIMIT"));
                }
                let (p, uid) = head_and_uid(line, no)?;
                if p.len() != 7 || !printable_name(p[1]) {
                    return Err(err(no, "ODB_EDA_PKG_BAD_RECORD"));
                }
                let pitch = decimal(p[2], no)?;
                let bounds = [
                    decimal(p[3], no)?,
                    decimal(p[4], no)?,
                    decimal(p[5], no)?,
                    decimal(p[6], no)?,
                ];
                if pitch < 0 || bounds[0] >= bounds[2] || bounds[1] >= bounds[3] {
                    return Err(err(no, "ODB_EDA_PKG_BAD_BOUNDS"));
                }
                if let Some(id) = uid {
                    if !all_uids.insert(id) {
                        return Err(err(no, "ODB_EDA_PKG_DUPLICATE_UID"));
                    }
                }
                let index = result.len();
                result.push(SourcePackage {
                    name: p[1].to_owned(),
                    ordinal: index,
                    pitch_ticks: pitch,
                    bounds,
                    source_uid: uid,
                    source_line: no,
                    outlines: Vec::new(),
                    pins: Vec::new(),
                });
                pkg = Some(index);
                pin = None;
                require_immediate_pkg_rc = true;
            }
            "PIN" => {
                let pi = pkg.ok_or_else(|| err(no, "ODB_EDA_PIN_WITHOUT_PACKAGE"))?;
                let (p, uid) = head_and_uid(line, no)?;
                if p.len() != 8 || !printable_name(p[1]) {
                    return Err(err(no, "ODB_EDA_PIN_BAD_RECORD"));
                }
                if p[2] != "S" || p[6] != "E" || p[7] != "S" {
                    return Err(err(no, "ODB_EDA_PIN_UNSUPPORTED_TYPE"));
                }
                let center = [decimal(p[3], no)?, decimal(p[4], no)?];
                let fhs = decimal(p[5], no)?;
                if fhs != 0 {
                    return Err(err(no, "ODB_EDA_PIN_UNSUPPORTED_HOLE"));
                }
                if total_pins >= MAX_PINS {
                    return Err(err(no, "ODB_EDA_PIN_LIMIT"));
                }
                if !pin_names.insert((pi, p[1].to_owned())) {
                    return Err(err(no, "ODB_EDA_PIN_DUPLICATE_NAME"));
                }
                if let Some(id) = uid {
                    if !all_uids.insert(id) {
                        return Err(err(no, "ODB_EDA_PKG_DUPLICATE_UID"));
                    }
                }
                let ordinal = result[pi].pins.len();
                result[pi].pins.push(SourcePin {
                    name: p[1].to_owned(),
                    ordinal,
                    center,
                    finished_hole_ticks: fhs,
                    pin_type: 'S',
                    electrical_type: 'E',
                    mount_type: 'S',
                    source_uid: uid,
                    source_line: no,
                    outlines: Vec::new(),
                });
                total_pins += 1;
                pin = Some(ordinal);
            }
            "RC" => {
                let pi = pkg.ok_or_else(|| err(no, "ODB_EDA_ORPHAN_OUTLINE"))?;
                let p: Vec<&str> = line.split_whitespace().collect();
                if p.len() != 5 {
                    return Err(err(no, "ODB_EDA_RC_BAD_RECORD"));
                }
                let lower = [decimal(p[1], no)?, decimal(p[2], no)?];
                let size = [decimal(p[3], no)?, decimal(p[4], no)?];
                if size.iter().any(|&v| v <= 0)
                    || lower[0].checked_add(size[0]).is_none()
                    || lower[1].checked_add(size[1]).is_none()
                {
                    return Err(err(no, "ODB_EDA_RC_BAD_BOUNDS"));
                }
                if total_rects >= MAX_OUTLINES {
                    return Err(err(no, "ODB_EDA_RC_LIMIT"));
                }
                let rc = SourceRectangle {
                    lower_left: lower,
                    size,
                    source_line: no,
                };
                if let Some(ni) = pin {
                    result[pi].pins[ni].outlines.push(rc);
                } else {
                    result[pi].outlines.push(rc);
                }
                total_rects += 1;
                require_immediate_pkg_rc = false;
            }
            "PRP" if pkg.is_some() => {
                // Polarity-marker and other package semantics are not decoded.
                return Err(err(no, "ODB_EDA_PKG_PROPERTY_UNSUPPORTED"));
            }
            "CR" | "SQ" | "CT" | "OB" | "OS" | "OC" | "OE" | "CE" => {
                return Err(err(no, "ODB_EDA_PKG_OUTLINE_UNSUPPORTED"));
            }
            "UNITS=MM" | "UNITS=INCH" => {
                if !result.is_empty() {
                    return Err(err(no, "ODB_EDA_PKG_LATE_UNITS"));
                }
                units = true;
            }
            "NET" | "SNT" | "FID" | "LYR" | "HDR" | "FGR" | "PRP" => {
                // These records are checked independently by odb_eda::parse.
                // They terminate package scope except PRP outside PKG.
                pkg = None;
                pin = None;
            }
            _ if kind.starts_with('@') || kind.starts_with('&') => {
                pkg = None;
                pin = None;
            }
            _ => {
                // Other EDA records remain rejected by the original strict parser.
                pkg = None;
                pin = None;
            }
        }
    }
    if require_immediate_pkg_rc {
        return Err(err(text.lines().count(), "ODB_EDA_PKG_MISSING_OUTLINE"));
    }
    if let (Some(pi), Some(ni)) = (pkg, pin) {
        if result[pi].pins[ni].outlines.is_empty() {
            return Err(err(text.lines().count(), "ODB_EDA_PIN_MISSING_OUTLINE"));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    const GOOD: &str="HDR source\nUNITS=MM\nLYR top bottom\nNET CLK\nSNT TRC\nFID C 0 0\nPKG QFN 0.5 -2 -2 2 2;ID=12\nRC -2 -2 4 4\nPIN 1 S -1.5 0 0 E S ID=13\nRC -1.75 -0.2 0.5 0.4\nPIN 2 S 1.5 0 0 E S;ID=14\nRC 1.25 -0.2 0.5 0.4\n";
    #[test]
    fn package_pin_index_and_exact_ticks() {
        let p = parse_packages(GOOD).unwrap();
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].ordinal, 0);
        assert_eq!(p[0].source_uid, Some(12));
        assert_eq!(p[0].pins.len(), 2);
        assert_eq!(p[0].pins[0].center[0], -1_500_000_000);
        assert_eq!(p[0].pins[1].ordinal, 1);
        assert_eq!(p[0].pins[1].source_uid, Some(14));
    }
    #[test]
    fn multiple_packages_and_names_in_independent_scopes() {
        let two =
            format!("{GOOD}PKG SECOND 0 0 0 2 2\nRC 0 0 2 2\nPIN 1 S 0.5 0.5 0 E S\nRC 0 0 1 1\n");
        let p = parse_packages(&two).unwrap();
        assert_eq!(p.len(), 2);
        assert_eq!(p[1].ordinal, 1);
        assert_eq!(p[1].pins[0].name, "1");
    }
    #[test]
    fn hostile_grammar_and_unknowns_fail_closed() {
        let cases = [
            (
                "RC -2 -2 4 4",
                "CR 0 0 2",
                "ODB_EDA_PKG_OUTLINE_UNSUPPORTED",
            ),
            (
                "PKG QFN 0.5 -2 -2 2 2;ID=12",
                "PKG QFN 0.5 2 -2 -2 2;ID=12",
                "ODB_EDA_PKG_BAD_BOUNDS",
            ),
            (
                "PKG QFN 0.5 -2 -2 2 2;ID=12\nRC -2 -2 4 4",
                "PKG QFN 0.5 -2 -2 2 2;ID=12\nPIN 1 S 0 0 0 E S",
                "ODB_EDA_PKG_IMMEDIATE_OUTLINE_REQUIRED",
            ),
            (
                "PIN 1 S -1.5 0 0 E S ID=13",
                "PIN 1 T -1.5 0 0 E S ID=13",
                "ODB_EDA_PIN_UNSUPPORTED_TYPE",
            ),
            (
                "PIN 1 S -1.5 0 0 E S ID=13",
                "PIN 1 S -1.5 0 0.1 E S ID=13",
                "ODB_EDA_PIN_UNSUPPORTED_HOLE",
            ),
            (
                "PIN 1 S -1.5 0 0 E S ID=13",
                "PIN 1 S -1.5 0 0 E S ID=12",
                "ODB_EDA_PKG_DUPLICATE_UID",
            ),
            (
                "RC -1.75 -0.2 0.5 0.4",
                "RC -1.75 -0.2 -0.5 0.4",
                "ODB_EDA_RC_BAD_BOUNDS",
            ),
            (
                "PIN 2 S 1.5 0 0 E S;ID=14",
                "PIN 1 S 1.5 0 0 E S;ID=14",
                "ODB_EDA_PIN_DUPLICATE_NAME",
            ),
            (
                "PIN 2 S 1.5 0 0 E S;ID=14\nRC 1.25 -0.2 0.5 0.4",
                "PIN 2 S 1.5 0 0 E S;ID=14",
                "ODB_EDA_PIN_MISSING_OUTLINE",
            ),
            (
                "PIN 1 S -1.5 0 0 E S ID=13",
                "PIN 1 S -1.5 0 0 E S;0=1;ID=13",
                "ODB_EDA_PKG_UNSUPPORTED_ATTRIBUTE",
            ),
        ];
        for (a, b, code) in cases {
            let text = GOOD.replace(a, b);
            assert_eq!(parse_packages(&text).unwrap_err().code, code, "{text}");
        }
        assert_eq!(
            parse_packages("UNITS=MM\nPIN 1 S 0 0 0 E S\n")
                .unwrap_err()
                .code,
            "ODB_EDA_PIN_WITHOUT_PACKAGE"
        );
    }
}
