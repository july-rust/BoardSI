//! D_056_08: source-evidence-qualified EDA NET/SNT/FID ownership subset.
//! Official ODB++Design 8.1 Update 4, eda/data pp. 105–116.
//!
//! FID C l f refers to 0-based LYR[l] and the 0-based feature record
//! in that layer's features file. It is a *design Net assignment only*.
//! Never infer via plating, physical copper continuity or a valid SI graph.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FeatureLocation {
    Trace(usize),
    Pad(usize),
    Arc(usize),
    Area(usize),
    Areas(Vec<usize>), // one Source Feature / FID, several disconnected canonical areas
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EdaError {
    pub line: usize,
    pub code: &'static str,
}
fn err(line: usize, code: &'static str) -> EdaError {
    EdaError { line, code }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EdaNet {
    pub name: String,
    pub source_line: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EdaAssignment {
    pub layer: String,
    pub feature_index: usize,
    pub net_name: String,
    pub source_line: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EdaOwnership {
    pub nets: Vec<EdaNet>,
    pub assignments: Vec<EdaAssignment>,
    pub packages: Vec<crate::odb_eda_packages::SourcePackage>,
}

/// An intentionally restricted EDA grammar. All FID records are checked
/// against the exact source feature-registry; we do not silently skip unsupported
/// PKG/PIN/other FIDs and then claim complete per-object Net mapping.
pub(super) fn parse(
    text: &str,
    features: &BTreeMap<(String, usize), FeatureLocation>,
) -> Result<EdaOwnership, EdaError> {
    // Validate the complete PKG/PIN/RC grammar before accepting any NET/FID.
    // A source definition never creates a placed component or electrical edge.
    let packages =
        crate::odb_eda_packages::parse_packages(text).map_err(|e| err(e.line, e.code))?;
    let mut layer_names: Option<Vec<String>> = None;
    let mut nets = Vec::new();
    let mut assignments = Vec::new();
    let mut net_name: Option<String> = None;
    let mut subnet: Option<&str> = None;
    let mut names = BTreeSet::new();
    let mut seen_features = BTreeMap::<(String, usize), String>::new();
    let mut saw_hdr = false;
    let mut units: Option<&str> = None;
    for (i, raw) in text.lines().enumerate() {
        let no = i + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        let kind = parts[0];
        match kind {
            "HDR" => {
                if saw_hdr || net_name.is_some() || parts.len() < 2 {
                    return Err(err(no, "ODB_EDA_BAD_HDR"));
                }
                saw_hdr = true;
            }
            _ if kind.starts_with("UNITS=") => {
                if !saw_hdr || units.is_some() || layer_names.is_some() || net_name.is_some() {
                    return Err(err(no, "ODB_EDA_BAD_UNITS_ORDER"));
                }
                let value = line.strip_prefix("UNITS=").expect("prefix checked");
                if !matches!(value, "MM" | "INCH") {
                    return Err(err(no, "ODB_EDA_UNSUPPORTED_UNITS"));
                }
                units = Some(value);
            }
            "LYR" => {
                if units.is_none() {
                    return Err(err(no, "ODB_EDA_MISSING_UNITS"));
                }
                if layer_names.is_some() || net_name.is_some() || parts.len() < 2 {
                    return Err(err(no, "ODB_EDA_BAD_LYR"));
                }
                let mut uniq = BTreeSet::new();
                for layer in &parts[1..] {
                    if !uniq.insert(*layer) {
                        return Err(err(no, "ODB_EDA_DUPLICATE_LYR"));
                    }
                }
                layer_names = Some(parts[1..].iter().map(|s| (*s).to_owned()).collect());
            }
            "NET" => {
                if layer_names.is_none() {
                    return Err(err(no, "ODB_EDA_NET_BEFORE_LYR"));
                }
                let name = line
                    .strip_prefix("NET ")
                    .and_then(|s| {
                        s.trim_start()
                            .split(|c: char| c == ';' || c.is_whitespace())
                            .next()
                    })
                    .ok_or_else(|| err(no, "ODB_EDA_BAD_NET"))?;
                if name.is_empty()
                    || name.len() > 256
                    || !name.bytes().all(|b| (33..=126).contains(&b) && b != b';')
                {
                    return Err(err(no, "ODB_EDA_BAD_NET"));
                }
                // ODB permits multiple $NONE$ records (distinct groups of
                // unassigned features). Named nets remain unique; duplicate
                // FID assignments are rejected independently for every subnet.
                if name != "$NONE$" && !names.insert(name.to_owned()) {
                    return Err(err(no, "ODB_EDA_AMBIGUOUS_NET_NAME"));
                }
                nets.push(EdaNet {
                    name: name.into(),
                    source_line: no,
                });
                net_name = Some(name.into());
                subnet = None;
            }
            "SNT" => {
                if net_name.is_none() {
                    return Err(err(no, "ODB_EDA_SNT_WITHOUT_NET"));
                }
                subnet = match parts.as_slice() {
                    ["SNT", "TRC"] => Some("TRC"),
                    ["SNT", "VIA"] => Some("VIA"),
                    ["SNT", "PLN", ..]
                        if parts.len() == 5
                            && matches!(parts[2], "S" | "O")
                            && matches!(parts[3], "C" | "R" | "O" | "E")
                            && parts[4]
                                .parse::<f64>()
                                .ok()
                                .map(|x| x.is_finite() && x >= 0.0)
                                .unwrap_or(false) =>
                    {
                        Some("PLN")
                    }
                    _ => return Err(err(no, "ODB_EDA_UNSUPPORTED_SUBNET")),
                };
            }
            "FID" => {
                let name = net_name
                    .as_ref()
                    .ok_or_else(|| err(no, "ODB_EDA_FID_WITHOUT_NET"))?;
                let sn = subnet.ok_or_else(|| err(no, "ODB_EDA_FID_WITHOUT_SUBNET"))?;
                if parts.len() != 4 || parts[1] != "C" {
                    return Err(err(no, "ODB_EDA_UNSUPPORTED_FID"));
                }
                let layer_num = parts[2]
                    .parse::<usize>()
                    .map_err(|_| err(no, "ODB_EDA_BAD_LAYER_INDEX"))?;
                let feature_num = parts[3]
                    .parse::<usize>()
                    .map_err(|_| err(no, "ODB_EDA_BAD_FEATURE_INDEX"))?;
                let layer = layer_names
                    .as_ref()
                    .and_then(|layers| layers.get(layer_num))
                    .ok_or_else(|| err(no, "ODB_EDA_BAD_LAYER_INDEX"))?
                    .clone();
                let loc = features
                    .get(&(layer.clone(), feature_num))
                    .ok_or_else(|| err(no, "ODB_EDA_UNRESOLVED_FID"))?;
                // VIA is a design subnet *not* proof that a drill was plated.
                // In this geometry-only subset a VIA FID can only attach to
                // a Pad. TOP/PKG/PIN physical attachment is not imported.
                let valid_kind = matches!(
                    (sn, loc),
                    ("TRC", FeatureLocation::Trace(_) | FeatureLocation::Arc(_))
                        | ("VIA", FeatureLocation::Pad(_))
                        | ("PLN", FeatureLocation::Area(_) | FeatureLocation::Areas(_))
                );
                if !valid_kind {
                    return Err(err(no, "ODB_EDA_FID_SUBNET_KIND_CONFLICT"));
                }
                if seen_features
                    .insert((layer.clone(), feature_num), name.clone())
                    .is_some()
                {
                    return Err(err(no, "ODB_EDA_DUPLICATE_FID"));
                }
                if name != "$NONE$" {
                    assignments.push(EdaAssignment {
                        layer,
                        feature_index: feature_num,
                        net_name: name.clone(),
                        source_line: no,
                    });
                }
            }
            _ if line.starts_with('@') || line.starts_with('&') => {
                if net_name.is_some() {
                    return Err(err(no, "ODB_EDA_LATE_LOOKUP_TABLE"));
                }
                // Preserve lookup definitions in source SHA, but do not infer
                // physical impedance, net role or attribute semantics.
            }
            "PKG" => {
                // Package definitions terminate the previous NET/SNT section.
                // Never allow a subsequent FID to inherit stale design ownership.
                net_name = None;
                subnet = None;
            }
            "PIN" | "RC" => {
                // Already fully validated by the source-only package parser.
            }
            "PRP" => {
                // PRP is a design property, not an FID, and does not grant
                // authoritative manufacturing or connectivity evidence.
            }
            _ => return Err(err(no, "ODB_EDA_UNSUPPORTED_RECORD")),
        }
    }
    if !saw_hdr || units.is_none() || layer_names.is_none() || nets.is_empty() {
        return Err(err(0, "ODB_EDA_INCOMPLETE"));
    }
    Ok(EdaOwnership {
        nets,
        assignments,
        packages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn features() -> BTreeMap<(String, usize), FeatureLocation> {
        BTreeMap::from([
            (("top".into(), 0), FeatureLocation::Trace(0)),
            (("top".into(), 1), FeatureLocation::Pad(0)),
            (("top".into(), 2), FeatureLocation::Area(0)),
            (("bottom".into(), 0), FeatureLocation::Arc(0)),
        ])
    }
    #[test]
    fn official_zero_based_layer_feature_and_subnet_semantics() {
        let data = "HDR Mentor PADS\nUNITS=MM\nLYR top bottom\nNET CLK;ID=1030\nSNT TRC\nFID C 0 0\nFID C 1 0\nNET GND;ID=5\nSNT VIA\nFID C 0 1\nSNT PLN S E 0\nFID C 0 2\n";
        let d = parse(data, &features()).unwrap();
        assert_eq!(d.nets.len(), 2);
        assert_eq!(d.assignments.len(), 4);
        assert_eq!(d.assignments[0].layer, "top");
        assert_eq!(d.assignments[1].feature_index, 0);
        assert_eq!(d.assignments[2].net_name, "GND");
    }
    #[test]
    fn missing_or_conflicting_evidence_fails_closed() {
        let valid = "HDR x\nUNITS=MM\nLYR top bottom\nNET CLK\nSNT TRC\nFID C 0 0\n";
        for (from, to, code) in [
            ("FID C 0 0", "FID C 2 0", "ODB_EDA_BAD_LAYER_INDEX"),
            ("FID C 0 0", "FID C 0 12", "ODB_EDA_UNRESOLVED_FID"),
            ("FID C 0 0", "FID H 0 0", "ODB_EDA_UNSUPPORTED_FID"),
            ("FID C 0 0", "FID C 0 1", "ODB_EDA_FID_SUBNET_KIND_CONFLICT"),
            ("SNT TRC", "SNT TOP T 0 0", "ODB_EDA_UNSUPPORTED_SUBNET"),
            ("NET CLK", "NET CLK\nNET CLK", "ODB_EDA_AMBIGUOUS_NET_NAME"),
            ("FID C 0 0", "FID C 0 0\nFID C 0 0", "ODB_EDA_DUPLICATE_FID"),
            ("SNT TRC", "SNT TRC\nPKG U1", "ODB_EDA_PKG_BAD_RECORD"),
        ] {
            let input = valid.replace(from, to);
            assert_eq!(parse(&input, &features()).unwrap_err().code, code);
        }
    }
    #[test]
    fn repeated_none_net_groups_are_valid_but_have_no_design_ownership() {
        let d = parse(
            "HDR x\nUNITS=MM\nLYR top bottom\nNET $NONE$\nSNT TRC\nFID C 0 0\nNET $NONE$\nSNT TRC\nFID C 1 0\n",
            &features(),
        )
        .unwrap();
        assert_eq!(d.nets.len(), 2);
        assert!(d.assignments.is_empty());
        let duplicate_fid = "HDR x\nUNITS=MM\nLYR top bottom\nNET $NONE$\nSNT TRC\nFID C 0 0\nNET $NONE$\nSNT TRC\nFID C 0 0\n";
        assert_eq!(
            parse(duplicate_fid, &features()).unwrap_err().code,
            "ODB_EDA_DUPLICATE_FID"
        );
    }

    #[test]
    fn e018_package_never_leaks_prior_net_subnet_ownership_to_later_fid() {
        let template = "HDR x\nUNITS=MM\nLYR top bottom\nNET CLK\nSNT TRC\nPKG QFN 1 -2 -2 2 2\nRC -2 -2 4 4\nPIN 1 S 0 0 0 E S\nRC 0 0 1 1\nFID C 0 0\n";
        assert_eq!(
            parse(template, &features()).unwrap_err().code,
            "ODB_EDA_FID_WITHOUT_NET"
        );
        let explicit = template.replace("FID C 0 0", "NET NEW\nSNT TRC\nFID C 0 0");
        assert_eq!(
            parse(&explicit, &features()).unwrap().assignments[0].net_name,
            "NEW"
        );
    }

    #[test]
    fn required_eda_units_order_and_duplicates_fail_closed() {
        let good = "HDR Mentor\nUNITS=MM\nLYR top bottom\nNET CLK\nSNT TRC\nFID C 0 0\n";
        assert_eq!(parse(good, &features()).unwrap().assignments.len(), 1);
        assert!(parse(&good.replace("UNITS=MM", "UNITS=INCH"), &features()).is_ok());
        for (mutation, code) in [
            (good.replace("UNITS=MM\n", ""), "ODB_EDA_MISSING_UNITS"),
            (
                good.replace("UNITS=MM", "UNITS=MILS"),
                "ODB_EDA_UNSUPPORTED_UNITS",
            ),
            (
                good.replace("UNITS=MM", "UNITS=MM\nUNITS=MM"),
                "ODB_EDA_BAD_UNITS_ORDER",
            ),
            (
                good.replace("HDR Mentor\nUNITS=MM", "UNITS=MM\nHDR Mentor"),
                "ODB_EDA_BAD_UNITS_ORDER",
            ),
        ] {
            assert_eq!(parse(&mutation, &features()).unwrap_err().code, code);
        }
    }
    #[test]
    fn none_is_unassigned_and_not_an_electrical_short() {
        let d = parse(
            "HDR x\nUNITS=MM\nLYR top\nNET $NONE$\nSNT TRC\nFID C 0 0\n",
            &features(),
        )
        .unwrap();
        assert_eq!(d.nets[0].name, "$NONE$");
        assert!(d.assignments.is_empty());
    }
}
