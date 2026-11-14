//! E_027: fail-closed, opt-in Gerber X2 circular-flash and Excellon decimal-drill
//! SOURCE inventories. Exact source geometry is not a BoardModel, via, net or SI proof.
use boardsi_model::{
    CapabilityState, ConversionQuality, ConversionStep, GeometryProvenance, ImportCapability,
    ObjectId, ObjectKind, ObjectMeta, Rational, SourceFile, SourceFormat, SourceId, SourceIdentity,
};
use std::{collections::BTreeMap, sync::Arc};

const MAX_FILE: usize = 8 * 1024 * 1024;
const MAX_RECORDS: usize = 100_000;
const MAX_TOOLS: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportError {
    pub code: &'static str,
    pub byte_offset: usize,
}
fn fail(code: &'static str, byte_offset: usize) -> ImportError {
    ImportError { code, byte_offset }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CopperLayer {
    pub meta: ObjectMeta,
    pub source_level: u32,
    pub source_position: String,
    pub attribute_offset: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CopperFlash {
    pub meta: ObjectMeta,
    pub centre_nm: [i64; 2],
    pub diameter_nm: i64,
    pub aperture_id: u32,
    pub source_offset: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DrillHole {
    pub meta: ObjectMeta,
    pub centre_nm: [i64; 2],
    pub diameter_nm: i64,
    pub tool_id: u32,
    pub source_offset: usize,
    /// Excellon geometry alone is neither plated nor non-plated evidence.
    pub plating: Option<bool>,
    /// Drill hits alone cannot prove physical layer span.
    pub layer_span: Option<(ObjectId, ObjectId)>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GerberInventory {
    pub identity: SourceId,
    pub layer: CopperLayer,
    pub flashes: Vec<CopperFlash>,
    pub capability: ImportCapability,
    pub solver_ready: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct DrillInventory {
    pub identity: SourceId,
    pub holes: Vec<DrillHole>,
    pub capability: ImportCapability,
    pub solver_ready: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PairInventory {
    pub artwork: GerberInventory,
    pub drill: DrillInventory,
    /// There is no machine-verifiable alignment/revision witness in this slice.
    pub alignment_verified: bool,
    pub solver_ready: bool,
}

fn check_input(bytes: &[u8]) -> Result<&str, ImportError> {
    if bytes.len() > MAX_FILE {
        return Err(fail("INPUT_TOO_LARGE", MAX_FILE));
    }
    if bytes
        .iter()
        .any(|b| !b.is_ascii() || (b.is_ascii_control() && !matches!(*b, b'\r' | b'\n' | b'\t')))
    {
        return Err(fail("NON_ASCII_OR_CONTROL", 0));
    }
    std::str::from_utf8(bytes).map_err(|_| fail("INVALID_UTF8", 0))
}
fn identity(format: SourceFormat, revision: &str, path: &str, bytes: &[u8]) -> SourceId {
    Arc::new(SourceIdentity {
        format,
        format_version: revision.into(),
        files: vec![SourceFile {
            logical_path: path.into(),
            sha256: crate::digest::sha256(bytes),
        }],
        importer_version: env!("CARGO_PKG_VERSION").into(),
        conditions: vec![("scope".into(), "e027-source-only".into())],
    })
}
fn meta(source: &SourceId, kind: ObjectKind, key: &str, ty: &str) -> ObjectMeta {
    ObjectMeta {
        id: ObjectId::new(source, kind, key, ""),
        provenance: GeometryProvenance {
            source_object_type: ty.into(),
            history: vec![ConversionStep {
                operation: "preserve-source-exact-nm".into(),
                quality: ConversionQuality::Exact,
                max_error_m: Some(Rational::integer(0)),
                note: "Partial source geometry; not a manufactured electrical or SI proof".into(),
            }],
        },
    }
}
fn partial_capability(coverage: &str) -> ImportCapability {
    ImportCapability {
        geometry: CapabilityState::Partial,
        net: CapabilityState::Missing,
        pin_pad: CapabilityState::Missing,
        stackup: CapabilityState::Missing,
        reference: CapabilityState::Missing,
        model: CapabilityState::Missing,
        object_coverage: vec![coverage.into()],
    }
}
fn mm_decimal_nm(text: &str, offset: usize) -> Result<i64, ImportError> {
    if text.is_empty() || text.len() > 32 {
        return Err(fail("INVALID_DECIMAL", offset));
    }
    let (negative, s) = if let Some(rest) = text.strip_prefix('-') {
        (true, rest)
    } else {
        (false, text)
    };
    let (int, frac) = s
        .split_once('.')
        .ok_or_else(|| fail("DECIMAL_POINT_REQUIRED", offset))?;
    if int.is_empty()
        || frac.is_empty()
        || frac.len() > 6
        || !int.bytes().all(|b| b.is_ascii_digit())
        || !frac.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(fail("INVALID_DECIMAL", offset));
    }
    let i = int
        .parse::<i64>()
        .map_err(|_| fail("DIMENSION_OVERFLOW", offset))?;
    let f = frac
        .parse::<i64>()
        .map_err(|_| fail("INVALID_DECIMAL", offset))?;
    let scale = 10_i64.pow((6 - frac.len()) as u32);
    let n = i
        .checked_mul(1_000_000)
        .and_then(|v| f.checked_mul(scale).and_then(|f| v.checked_add(f)))
        .ok_or_else(|| fail("DIMENSION_OVERFLOW", offset))?;
    if negative {
        n.checked_neg()
            .ok_or_else(|| fail("DIMENSION_OVERFLOW", offset))
    } else {
        Ok(n)
    }
}
fn positive_decimal(text: &str, offset: usize) -> Result<i64, ImportError> {
    let n = mm_decimal_nm(text, offset)?;
    if n <= 0 {
        Err(fail("INVALID_DIAMETER", offset))
    } else {
        Ok(n)
    }
}
fn gerber_commands(src: &str) -> Result<Vec<(bool, &str, usize)>, ImportError> {
    let b = src.as_bytes();
    let mut i = 0;
    let mut commands = Vec::new();
    while i < b.len() {
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if i == b.len() {
            break;
        }
        if commands.len() >= MAX_RECORDS {
            return Err(fail("RECORD_LIMIT", i));
        }
        let start = i;
        if b[i] == b'%' {
            i += 1;
            let s = i;
            while i < b.len() && b[i] != b'*' && b[i] != b'%' {
                i += 1;
            }
            if b.get(i) != Some(&b'*') || b.get(i + 1) != Some(&b'%') || s == i {
                return Err(fail("BAD_EXTENDED_COMMAND", start));
            }
            let cmd = &src[s..i];
            i += 2;
            commands.push((true, cmd, start));
        } else {
            while i < b.len() && b[i] != b'*' && b[i] != b'%' {
                i += 1;
            }
            if b.get(i) != Some(&b'*') || start == i {
                return Err(fail("BAD_GERBER_COMMAND", start));
            }
            commands.push((false, &src[start..i], start));
            i += 1;
        }
    }
    Ok(commands)
}
fn gerber_format(s: &str, pos: usize) -> Result<(usize, usize), ImportError> {
    let s = s
        .strip_prefix("FSLAX")
        .ok_or_else(|| fail("UNSUPPORTED_COORDINATE_FORMAT", pos))?;
    let (x, y) = s
        .split_once('Y')
        .ok_or_else(|| fail("UNSUPPORTED_COORDINATE_FORMAT", pos))?;
    if x != y || x.len() != 2 || !x.bytes().all(|b| b.is_ascii_digit()) {
        return Err(fail("UNSUPPORTED_COORDINATE_FORMAT", pos));
    }
    let ints = (x.as_bytes()[0] - b'0') as usize;
    let frac = (x.as_bytes()[1] - b'0') as usize;
    if !(1..=6).contains(&ints) || !(1..=6).contains(&frac) {
        return Err(fail("UNSUPPORTED_COORDINATE_FORMAT", pos));
    }
    Ok((ints, frac))
}
fn fixed_nm(s: &str, ints: usize, frac: usize, pos: usize) -> Result<i64, ImportError> {
    let (negative, digits) = if let Some(rest) = s.strip_prefix('-') {
        (true, rest)
    } else {
        (false, s)
    };
    if digits.is_empty()
        || digits.len() > ints + frac
        || !digits.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(fail("BAD_FIXED_COORDINATE", pos));
    }
    let value = digits
        .parse::<i64>()
        .map_err(|_| fail("COORDINATE_OVERFLOW", pos))?
        .checked_mul(10_i64.pow((6 - frac) as u32))
        .ok_or_else(|| fail("COORDINATE_OVERFLOW", pos))?;
    if negative {
        value
            .checked_neg()
            .ok_or_else(|| fail("COORDINATE_OVERFLOW", pos))
    } else {
        Ok(value)
    }
}
fn copper_function(s: &str, offset: usize) -> Result<(u32, String), ImportError> {
    let args = s
        .strip_prefix("TF.FileFunction,Copper,")
        .ok_or_else(|| fail("UNSUPPORTED_FILE_FUNCTION", offset))?;
    let (layer, side) = args
        .split_once(',')
        .ok_or_else(|| fail("BAD_LAYER_ATTRIBUTE", offset))?;
    let raw = layer
        .strip_prefix('L')
        .ok_or_else(|| fail("BAD_LAYER_ATTRIBUTE", offset))?;
    if raw.is_empty() || raw.starts_with('0') || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return Err(fail("BAD_LAYER_ATTRIBUTE", offset));
    }
    let number = raw
        .parse::<u32>()
        .map_err(|_| fail("BAD_LAYER_ATTRIBUTE", offset))?;
    if number == 0
        || number > 64
        || !matches!(side, "Top" | "Inr" | "Bot")
        || (number == 1 && side != "Top")
        || (number > 1 && side == "Top")
    {
        return Err(fail("BAD_LAYER_ATTRIBUTE", offset));
    }
    Ok((number, side.into()))
}

/// Strict one-file Gerber X2 LZ absolute circular flash subset.
/// Unsupported D01/region/arcs/polarity/attribute modifiers are fatal, never skipped.
pub fn inspect_gerber(bytes: &[u8]) -> Result<GerberInventory, ImportError> {
    let source = check_input(bytes)?;
    let commands = gerber_commands(source)?;
    let mut fmt = None;
    let mut units = false;
    let mut func = None;
    let mut apertures = BTreeMap::<u32, i64>::new();
    let mut selected = None;
    let mut positive = false;
    let mut rows = Vec::<([i64; 2], i64, u32, usize)>::new();
    let mut ended = false;
    for (extended, cmd, pos) in commands {
        if ended {
            return Err(fail("TRAILING_COMMAND", pos));
        }
        if extended {
            if cmd.starts_with("FS") {
                if fmt.replace(gerber_format(cmd, pos)?).is_some() {
                    return Err(fail("DUPLICATE_FORMAT", pos));
                }
            } else if cmd == "MOMM" {
                if units {
                    return Err(fail("DUPLICATE_UNITS", pos));
                }
                units = true;
            } else if cmd.starts_with("TF.") {
                let (number, side) = copper_function(cmd, pos)?;
                if func.replace((number, side, pos)).is_some() {
                    return Err(fail("DUPLICATE_FILE_FUNCTION", pos));
                }
            } else if let Some(def) = cmd.strip_prefix("ADD") {
                let (n, spec) = def
                    .split_once('C')
                    .ok_or_else(|| fail("UNSUPPORTED_APERTURE", pos))?;
                let aperture_id = n.parse::<u32>().map_err(|_| fail("BAD_APERTURE_ID", pos))?;
                let diameter = positive_decimal(
                    spec.strip_prefix(',')
                        .ok_or_else(|| fail("BAD_APERTURE", pos))?,
                    pos,
                )?;
                if aperture_id < 10
                    || apertures.len() >= MAX_TOOLS
                    || apertures.insert(aperture_id, diameter).is_some()
                {
                    return Err(fail("INVALID_OR_DUPLICATE_APERTURE", pos));
                }
            } else if cmd == "LPD" {
                if positive {
                    return Err(fail("DUPLICATE_POLARITY", pos));
                }
                positive = true;
            } else {
                return Err(fail("UNSUPPORTED_GERBER_EXTENSION", pos));
            }
        } else if cmd == "M02" {
            ended = true;
        } else if cmd.starts_with("G04") {
            // Gerber comment: no geometry or source attribute may be inferred.
        } else if let Some(n) = cmd.strip_prefix('D') {
            let id = n
                .parse::<u32>()
                .map_err(|_| fail("BAD_APERTURE_SELECTION", pos))?;
            if !apertures.contains_key(&id) {
                return Err(fail("UNKNOWN_APERTURE", pos));
            }
            selected = Some(id);
        } else if let Some(coords) = cmd.strip_prefix('X') {
            let (x, tail) = coords
                .split_once('Y')
                .ok_or_else(|| fail("INCOMPLETE_FLASH", pos))?;
            let y = tail
                .strip_suffix("D03")
                .ok_or_else(|| fail("UNSUPPORTED_DRAW", pos))?;
            let (ints, frac) = fmt.ok_or_else(|| fail("MISSING_FORMAT", pos))?;
            if !units || func.is_none() || !positive {
                return Err(fail("MISSING_GERBER_DECLARATION", pos));
            }
            let id = selected.ok_or_else(|| fail("APERTURE_NOT_SELECTED", pos))?;
            if rows.len() >= MAX_RECORDS {
                return Err(fail("FLASH_LIMIT", pos));
            }
            rows.push((
                [fixed_nm(x, ints, frac, pos)?, fixed_nm(y, ints, frac, pos)?],
                apertures[&id],
                id,
                pos,
            ));
        } else {
            return Err(fail("UNSUPPORTED_GERBER_COMMAND", pos));
        }
    }
    if !ended || !units || !positive || fmt.is_none() || func.is_none() || rows.is_empty() {
        return Err(fail("INCOMPLETE_GERBER", bytes.len()));
    }
    let (number, side, attribute_offset) = func.expect("checked above");
    let id = identity(
        SourceFormat::Gerber,
        "X2-circular-flash-subset",
        "artwork.gbr",
        bytes,
    );
    let layer = CopperLayer {
        meta: meta(
            &id,
            ObjectKind::Layer,
            &format!("X2:Copper:L{number}:{side}"),
            "Gerber X2 FileFunction",
        ),
        source_level: number,
        source_position: side,
        attribute_offset,
    };
    let flashes = rows
        .into_iter()
        .map(
            |(centre_nm, diameter_nm, aperture_id, source_offset)| CopperFlash {
                meta: meta(
                    &id,
                    ObjectKind::Pad,
                    &format!("flash@byte:{source_offset}"),
                    "Gerber circular flash copper",
                ),
                centre_nm,
                diameter_nm,
                aperture_id,
                source_offset,
            },
        )
        .collect();
    Ok(GerberInventory {
        identity: id,
        layer,
        flashes,
        capability: partial_capability("Gerber X2 one copper layer circular D03 flashes"),
        solver_ready: false,
    })
}

/// Explicit decimal-point METRIC Excellon round drill-hit subset only.
/// Missing fabrication declarations never imply plated PTH, NPTH or via spans.
pub fn inspect_excellon(bytes: &[u8]) -> Result<DrillInventory, ImportError> {
    let source = check_input(bytes)?;
    let mut header = 0u8; // 0=before M48, 1=header, 2=body, 3=ended
    let mut units = false;
    let mut tools = BTreeMap::<u32, i64>::new();
    let mut selected = None;
    let mut rows = Vec::<([i64; 2], i64, u32, usize)>::new();
    let mut offset = 0usize;
    let mut commands = 0usize;
    for line in source.split_inclusive('\n') {
        let pos = offset;
        offset += line.len();
        let cmd = line.trim();
        if cmd.is_empty() {
            continue;
        }
        commands += 1;
        if commands > MAX_RECORDS {
            return Err(fail("RECORD_LIMIT", pos));
        }
        if header == 3 {
            return Err(fail("TRAILING_DRILL_COMMAND", pos));
        }
        if header == 0 {
            if cmd != "M48" {
                return Err(fail("MISSING_M48", pos));
            }
            header = 1;
        } else if header == 1 {
            if cmd == "METRIC" {
                if units {
                    return Err(fail("DUPLICATE_UNITS", pos));
                }
                units = true;
            } else if cmd == "%" {
                if !units || tools.is_empty() {
                    return Err(fail("INCOMPLETE_DRILL_HEADER", pos));
                }
                header = 2;
            } else if let Some(tool) = cmd.strip_prefix('T') {
                let (number, diameter) = tool
                    .split_once('C')
                    .ok_or_else(|| fail("BAD_TOOL_DECLARATION", pos))?;
                if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(fail("BAD_TOOL_ID", pos));
                }
                let id = number
                    .parse::<u32>()
                    .map_err(|_| fail("BAD_TOOL_ID", pos))?;
                let d = positive_decimal(diameter, pos)?;
                if id == 0 || tools.len() >= MAX_TOOLS || tools.insert(id, d).is_some() {
                    return Err(fail("DUPLICATE_OR_INVALID_TOOL", pos));
                }
            } else {
                return Err(fail("UNSUPPORTED_DRILL_HEADER", pos));
            }
        } else if cmd == "M30" {
            header = 3;
        } else if let Some(n) = cmd.strip_prefix('T') {
            if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
                return Err(fail("BAD_TOOL_SELECTION", pos));
            }
            let id = n
                .parse::<u32>()
                .map_err(|_| fail("BAD_TOOL_SELECTION", pos))?;
            if !tools.contains_key(&id) {
                return Err(fail("UNKNOWN_DRILL_TOOL", pos));
            }
            selected = Some(id);
        } else if let Some(coords) = cmd.strip_prefix('X') {
            let (x, y) = coords
                .split_once('Y')
                .ok_or_else(|| fail("INCOMPLETE_DRILL_HIT", pos))?;
            let id = selected.ok_or_else(|| fail("DRILL_TOOL_NOT_SELECTED", pos))?;
            if rows.len() >= MAX_RECORDS {
                return Err(fail("DRILL_LIMIT", pos));
            }
            rows.push((
                [mm_decimal_nm(x, pos)?, mm_decimal_nm(y, pos)?],
                tools[&id],
                id,
                pos,
            ));
        } else {
            return Err(fail("UNSUPPORTED_DRILL_COMMAND", pos));
        }
    }
    if header != 3 || rows.is_empty() {
        return Err(fail("INCOMPLETE_DRILL_FILE", bytes.len()));
    }
    let id = identity(
        SourceFormat::Other("Excellon".into()),
        "metric-decimal-round-hit-subset",
        "drill.drl",
        bytes,
    );
    let holes = rows
        .into_iter()
        .map(
            |(centre_nm, diameter_nm, tool_id, source_offset)| DrillHole {
                meta: meta(
                    &id,
                    ObjectKind::Drill,
                    &format!("hit@byte:{source_offset}"),
                    "Excellon round drill hit",
                ),
                centre_nm,
                diameter_nm,
                tool_id,
                source_offset,
                plating: None,
                layer_span: None,
            },
        )
        .collect();
    Ok(DrillInventory {
        identity: id,
        holes,
        capability: partial_capability("Excellon METRIC decimal round drill hits"),
        solver_ready: false,
    })
}
pub fn inspect_pair(gerber: &[u8], drill: &[u8]) -> Result<PairInventory, ImportError> {
    Ok(PairInventory {
        artwork: inspect_gerber(gerber)?,
        drill: inspect_excellon(drill)?,
        alignment_verified: false,
        solver_ready: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    const G: &str = "%FSLAX36Y36*%\n%MOMM*%\n%TF.FileFunction,Copper,L1,Top*%\n%ADD10C,0.600*%\n%LPD*%\nD10*\nX012345678Y002000000D03*\nM02*\n";
    const D: &str = "M48\nMETRIC\nT01C0.300\n%\nT01\nX12.345678Y2.000000\nM30\n";
    #[test]
    fn exact_source_geometry_and_unknown_physics() {
        let pair = inspect_pair(G.as_bytes(), D.as_bytes()).unwrap();
        assert_eq!(pair.artwork.layer.source_level, 1);
        assert_eq!(pair.artwork.layer.source_position, "Top");
        assert_eq!(pair.artwork.flashes[0].centre_nm, [12_345_678, 2_000_000]);
        assert_eq!(pair.artwork.flashes[0].diameter_nm, 600_000);
        assert_eq!(pair.drill.holes[0].centre_nm, [12_345_678, 2_000_000]);
        assert_eq!(pair.drill.holes[0].diameter_nm, 300_000);
        assert_eq!(pair.drill.holes[0].plating, None);
        assert_eq!(pair.drill.holes[0].layer_span, None);
        assert!(!pair.alignment_verified && !pair.solver_ready);
        assert_eq!(pair.artwork.capability.net, CapabilityState::Missing);
        assert_eq!(pair.drill.capability.stackup, CapabilityState::Missing);
        assert_ne!(pair.artwork.identity, pair.drill.identity);
    }
    #[test]
    fn declared_inner_and_bottom_layers_remain_source_only() {
        for (layer, n, side) in [("L2,Inr", 2, "Inr"), ("L6,Bot", 6, "Bot")] {
            let source = G.replace("L1,Top", layer);
            let inv = inspect_gerber(source.as_bytes()).unwrap();
            assert_eq!(
                (inv.layer.source_level, inv.layer.source_position.as_str()),
                (n, side)
            );
            assert!(!inv.solver_ready);
        }
    }
    #[test]
    fn gerber_mutations_fail_closed() {
        let invalid = [
            G.replace("FSLAX36Y36", "FSTAX36Y36"),
            G.replace("MOMM", "MOIN"),
            G.replace("Copper,L1,Top", "Copper,L1,Bot"),
            G.replace("ADD10C,0.600", "ADD10R,0.600X0.600"),
            G.replace("ADD10C,0.600", "ADD10C,0.000"),
            G.replace("D10*", "D11*"),
            G.replace("D03*", "D01*"),
            G.replace("LPD", "LPC"),
            G.replace("M02*", "M00*"),
            format!("{G}D10*"),
            G.replace("X012345678", "X123456789012345"),
            G.replace("Y002000000", "Y002000000D02"),
            G.replace("%ADD10C,0.600*%", "%ADD10C,0.600*%%ADD10C,0.700*%"),
        ];
        for bad in invalid {
            assert!(inspect_gerber(bad.as_bytes()).is_err(), "{bad}");
        }
    }
    #[test]
    fn drill_mutations_fail_closed() {
        let invalid = [
            D.replace("METRIC", "INCH"),
            D.replace("METRIC", "METRIC,LZ"),
            D.replace("X12.345678", "X12345678"),
            D.replace("T01\nX", "T02\nX"),
            D.replace("T01C0.300", "T01C0.000"),
            D.replace("M30", "M00"),
            format!("{D}X1.0Y1.0"),
            D.replace("X12.345678Y2.000000", "X12.345678"),
            D.replace("T01C0.300", "T01C0.300\nT01C0.600"),
            D.replace("M48", "M71"),
        ];
        for bad in invalid {
            assert!(inspect_excellon(bad.as_bytes()).is_err(), "{bad}");
        }
    }
    #[test]
    fn distinct_sha_and_truncation_invalidate() {
        let a = inspect_pair(G.as_bytes(), D.as_bytes()).unwrap();
        let changed = G.replace("012345678", "012345679");
        let b = inspect_pair(changed.as_bytes(), D.as_bytes()).unwrap();
        assert_ne!(a.artwork.identity, b.artwork.identity);
        assert_eq!(a.drill.identity, b.drill.identity);
        assert!(inspect_gerber(&G.as_bytes()[..G.len() - 2]).is_err());
        assert!(inspect_excellon(&D.as_bytes()[..D.len() - 4]).is_err());
    }
    #[test]
    fn byte_limits_reject_before_allocation_or_state_promotion() {
        let huge = vec![b'X'; MAX_FILE + 1];
        assert_eq!(inspect_gerber(&huge).unwrap_err().code, "INPUT_TOO_LARGE");
        assert_eq!(inspect_excellon(&huge).unwrap_err().code, "INPUT_TOO_LARGE");
        assert!(inspect_gerber(b"\0").is_err());
    }
}
