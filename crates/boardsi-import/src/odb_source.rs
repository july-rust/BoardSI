//! D_056_08: fail-closed ODB++Design *unpacked-file-map* source subset.
//! Siemens ODB++Design Format Specification 8.1 Update 4:
//! matrix/matrix pp.62-66; features pp.164-172; standard symbols pp.187-189.
//! This is NOT a whole-package, step-repeat, drill, full EDA/Net, or Qt importer.
//! Source geometry is not manufactured connectivity, stackup z, or SI proof.
use crate::odb_drill::parse_tools;
use crate::odb_drill_hits::parse_drill_hits;
pub use crate::odb_drill_hits::SourceDrillHit;
pub use crate::odb_drill::{DrillIntent, SourceDrillLayer, SourceDrillTool};
use crate::odb_eda::{self, FeatureLocation};
use crate::odb_surface::SurfaceBuilder;
use boardsi_model::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc as Shared;

const MAX_FILES: usize = 8192;
const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 128 * 1024 * 1024;
const MAX_FEATURES: usize = 1_000_000;

/// All paths are relative to the uncompressed ODB++ product-model root.
/// Archive extraction, symlinks, filesystem traversal and Qt IPC are not handled here.
pub type OdbFileMap = BTreeMap<String, Vec<u8>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OdbError {
    pub path: String,
    pub line: usize,
    pub code: &'static str,
}
fn error(path: &str, line: usize, code: &'static str) -> OdbError {
    OdbError {
        path: path.into(),
        line,
        code,
    }
}
fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 1024
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains('\0')
        && !path.contains(':')
        && path
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != "..")
}
fn ascii_file<'a>(files: &'a OdbFileMap, path: &str) -> Result<&'a str, OdbError> {
    let bytes = files
        .get(path)
        .ok_or_else(|| error(path, 0, "ODB_MISSING_FILE"))?;
    std::str::from_utf8(bytes).map_err(|_| error(path, 0, "ODB_INVALID_TEXT"))
}
fn meta(source: &SourceId, kind: ObjectKind, id: &str, path: &str, line: usize) -> ObjectMeta {
    ObjectMeta {
        id: ObjectId::new(source, kind, id, ""),
        provenance: GeometryProvenance {
            source_object_type: format!("ODB++Design {path}:line:{line}"),
            history: vec![ConversionStep {
                operation: "source-preserved-exact-decimal".into(),
                quality: ConversionQuality::Exact,
                max_error_m: Some(Rational::integer(0)),
                note: "No electrical/manufacturing connection inferred".into(),
            }],
        },
    }
}
#[derive(Clone, Debug)]
struct MatrixLayer {
    row: u32,
    source_line: usize,
    name: String,
    layer_type: String,
    context: String,
    polarity: String,
    start_name: String,
    end_name: String,
    add_type: String,
}
#[derive(Debug)]
struct Matrix {
    steps: Vec<String>,
    layers: Vec<MatrixLayer>,
}
fn parse_matrix(text: &str) -> Result<Matrix, OdbError> {
    let path = "matrix/matrix";
    let mut state: Option<(&str, BTreeMap<String, String>, usize)> = None;
    let mut steps = Vec::new();
    let mut layers = Vec::new();
    for (idx, source_line) in text.lines().enumerate() {
        let line_no = idx + 1;
        let line = source_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "STEP {" || line == "LAYER {" {
            if state.is_some() {
                return Err(error(path, line_no, "ODB_NESTED_BLOCK"));
            }
            state = Some((
                if line.starts_with("STEP") {
                    "STEP"
                } else {
                    "LAYER"
                },
                BTreeMap::new(),
                line_no,
            ));
            continue;
        }
        if line == "}" {
            let (kind, fields, start_line) = state
                .take()
                .ok_or_else(|| error(path, line_no, "ODB_UNMATCHED_BLOCK"))?;
            let get = |key| fields.get(key).map(|s: &String| s.as_str()).unwrap_or("");
            if kind == "STEP" {
                let name = get("NAME");
                let col = get("COL")
                    .parse::<u32>()
                    .map_err(|_| error(path, start_line, "ODB_BAD_STEP_COLUMN"))?;
                if !safe_entity(name) || col == 0 || steps.iter().any(|s| s == name) {
                    return Err(error(path, start_line, "ODB_BAD_STEP"));
                }
                steps.push(name.to_owned());
            } else {
                let name = get("NAME");
                let row = get("ROW")
                    .parse::<u32>()
                    .map_err(|_| error(path, start_line, "ODB_BAD_LAYER_ROW"))?;
                if !safe_entity(name)
                    || row == 0
                    || layers
                        .iter()
                        .any(|l: &MatrixLayer| l.name == name || l.row == row)
                {
                    return Err(error(path, start_line, "ODB_DUPLICATE_OR_BAD_LAYER"));
                }
                layers.push(MatrixLayer {
                    row,
                    source_line: start_line,
                    name: name.into(),
                    layer_type: get("TYPE").into(),
                    context: get("CONTEXT").into(),
                    polarity: get("POLARITY").into(),
                    start_name: get("START_NAME").into(),
                    end_name: get("END_NAME").into(),
                    add_type: get("ADD_TYPE").into(),
                });
            }
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| error(path, line_no, "ODB_BAD_MATRIX_FIELD"))?;
        let key = key.trim();
        if let Some((_, ref mut fields, _)) = state {
            if fields.insert(key.into(), value.trim().into()).is_some() {
                return Err(error(path, line_no, "ODB_DUPLICATE_MATRIX_FIELD"));
            }
        } else {
            return Err(error(path, line_no, "ODB_OUTSIDE_MATRIX_BLOCK"));
        }
    }
    if state.is_some() {
        return Err(error(path, text.lines().count(), "ODB_UNTERMINATED_BLOCK"));
    }
    if steps.is_empty() || layers.is_empty() {
        return Err(error(path, 0, "ODB_EMPTY_MATRIX"));
    }
    layers.sort_by_key(|x| x.row);
    Ok(Matrix { steps, layers })
}
fn safe_entity(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_.+-".contains(&b))
}

/// Represent up to nine decimal places exactly in the source file's unit.
/// No floating-point parsing, truncation, scientific notation or silent snapping.
fn decimal_ticks(value: &str, path: &str, line: usize) -> Result<i64, OdbError> {
    let digits = value.trim();
    let (neg, digits) = if let Some(x) = digits.strip_prefix('-') {
        (true, x)
    } else if let Some(x) = digits.strip_prefix('+') {
        (false, x)
    } else {
        (false, digits)
    };
    let (whole, fract) = digits.split_once('.').unwrap_or((digits, ""));
    if (whole.is_empty() && fract.is_empty())
        || fract.len() > 9
        || !whole.bytes().all(|x| x.is_ascii_digit())
        || !fract.bytes().all(|x| x.is_ascii_digit())
    {
        return Err(error(path, line, "ODB_BAD_EXACT_DECIMAL"));
    }
    let integer = if whole.is_empty() {
        0_i64
    } else {
        whole
            .parse::<i64>()
            .map_err(|_| error(path, line, "ODB_DECIMAL_OVERFLOW"))?
    };
    let fraction = if fract.is_empty() {
        0_i64
    } else {
        fract
            .parse::<i64>()
            .map_err(|_| error(path, line, "ODB_BAD_EXACT_DECIMAL"))?
    };
    let scale = 10_i64.pow((9 - fract.len()) as u32);
    let value = integer
        .checked_mul(1_000_000_000)
        .and_then(|n| fraction.checked_mul(scale).and_then(|f| n.checked_add(f)))
        .ok_or_else(|| error(path, line, "ODB_DECIMAL_OVERFLOW"))?;
    if neg {
        value
            .checked_neg()
            .ok_or_else(|| error(path, line, "ODB_DECIMAL_OVERFLOW"))
    } else {
        Ok(value)
    }
}
fn length_symbol(value: &str, mode: &str, path: &str, line: usize) -> Result<Rational, OdbError> {
    // Standard symbol parameters are decimal reals (ODB++Design 8.1 U4 p.36).
    // A tick is 1e-9 source symbol units, so even fractional-mil sizes remain exact.
    let amount = decimal_ticks(value, path, line)?;
    if amount <= 0 || amount > 10_000_000_000_000_000 {
        return Err(error(path, line, "ODB_SYMBOL_SIZE_RANGE"));
    }
    let (numerator, denominator) = match mode {
        "M" => (i128::from(amount), 1_000_000_000_000_000_i128),
        "I" => (
            i128::from(amount)
                .checked_mul(254)
                .ok_or_else(|| error(path, line, "ODB_SYMBOL_OVERFLOW"))?,
            10_000_000_000_000_000_i128,
        ),
        _ => return Err(error(path, line, "ODB_SYMBOL_UNIT")),
    };
    Rational::new(numerator, denominator).map_err(|_| error(path, line, "ODB_SYMBOL_RATIONAL"))
}

#[derive(Clone)]
enum Symbol {
    Round(Rational),
    Rectangle([Rational; 2]),
}
fn parse_symbol(name: &str, mode: &str, path: &str, line: usize) -> Result<Symbol, OdbError> {
    if let Some(a) = name.strip_prefix("rect") {
        if let Some((w, h)) = a.split_once('x') {
            return Ok(Symbol::Rectangle([
                length_symbol(w, mode, path, line)?,
                length_symbol(h, mode, path, line)?,
            ]));
        }
    }
    if let Some(s) = name.strip_prefix('s') {
        let side = length_symbol(s, mode, path, line)?;
        return Ok(Symbol::Rectangle([side, side]));
    }
    if let Some(d) = name.strip_prefix('r') {
        return Ok(Symbol::Round(length_symbol(d, mode, path, line)?));
    }
    Err(error(path, line, "ODB_UNSUPPORTED_SYMBOL"))
}
fn positive_symbol_name(value: &str, path: &str, line: usize) -> Result<u32, OdbError> {
    value
        .parse::<u32>()
        .map_err(|_| error(path, line, "ODB_BAD_SYMBOL_REFERENCE"))
}
fn coordinate(value: &str, path: &str, line: usize) -> Result<i64, OdbError> {
    decimal_ticks(value, path, line)
}
fn point(frame: &ObjectId, x: &str, y: &str, path: &str, line: usize) -> Result<Point, OdbError> {
    Ok(Point {
        frame: frame.clone(),
        ticks: [coordinate(x, path, line)?, coordinate(y, path, line)?],
    })
}
fn orientation(value: &str, path: &str, line: usize) -> Result<Orientation, OdbError> {
    let (turn, mirror) = match value {
        "0" => (QuarterTurn::Zero, false),
        "1" => (QuarterTurn::Three, false),
        "2" => (QuarterTurn::Two, false),
        "3" => (QuarterTurn::One, false),
        "4" => (QuarterTurn::Zero, true),
        "5" => (QuarterTurn::Three, true),
        "6" => (QuarterTurn::Two, true),
        "7" => (QuarterTurn::One, true),
        _ => return Err(error(path, line, "ODB_UNSUPPORTED_PAD_ORIENTATION")),
    };
    Ok(Orientation::Orthogonal {
        turn,
        mirror_x: mirror,
    })
}
fn copper_type(value: &str) -> bool {
    matches!(value, "SIGNAL" | "POWER_GROUND" | "MIXED")
}

fn surface_boundary(frame: &ObjectId, ring: &[[i64; 2]]) -> Boundary {
    Boundary {
        segments: ring
            .windows(2)
            .map(|edge| BoundarySegment::Line {
                start: Point {
                    frame: frame.clone(),
                    ticks: edge[0],
                },
                end: Point {
                    frame: frame.clone(),
                    ticks: edge[1],
                },
            })
            .collect(),
    }
}

fn parse_features_content(
    board: &mut BoardModel,
    source: &SourceId,
    step: &str,
    layer: &MatrixLayer,
    text: &str,
    registry: &mut BTreeMap<(String, usize), FeatureLocation>,
) -> Result<(), OdbError> {
    let path = format!("steps/{step}/layers/{}/features", layer.name);
    let layer_id = ObjectId::new(
        source,
        ObjectKind::Layer,
        &format!("matrix:{}", layer.name),
        "",
    );
    let mut symbols: BTreeMap<u32, Symbol> = BTreeMap::new();
    let mut feature_count: usize = 0;
    let mut advertised_count: Option<usize> = None;
    let mut unit: Option<&'static str> = None;
    let mut ids: BTreeSet<String> = BTreeSet::new();
    let mut surface: Option<(String, usize, SurfaceBuilder)> = None;
    let frame_id = ObjectId::new(source, ObjectKind::Frame, &path, "");
    for (index, source_line) in text.lines().enumerate() {
        let line_no = index + 1;
        let line = source_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if surface.is_some() {
            // Surface records are multi-line; all geometry must be consumed or
            // the whole layer is unsupported. ODB features count S once, not
            // once per OS/OB/OE/SE. No implicit closing or arc tessellation.
            let fields: Vec<&str> = line.split_whitespace().collect();
            let kind = fields[0];
            let close_surface = kind == "SE";
            if !close_surface {
                let builder = &mut surface.as_mut().expect("surface checked").2;
                match kind {
                    "OB" if fields.len() == 4 => {
                        let start = [
                            coordinate(fields[1], &path, line_no)?,
                            coordinate(fields[2], &path, line_no)?,
                        ];
                        let hole = match fields[3] {
                            "I" => false,
                            "H" => true,
                            _ => return Err(error(&path, line_no, "ODB_SURFACE_BAD_POLYGON_ROLE")),
                        };
                        builder
                            .begin(start, hole)
                            .map_err(|code| error(&path, line_no, code))?;
                    }
                    "OS" if fields.len() == 3 => {
                        let p = [
                            coordinate(fields[1], &path, line_no)?,
                            coordinate(fields[2], &path, line_no)?,
                        ];
                        builder
                            .line(p)
                            .map_err(|code| error(&path, line_no, code))?;
                    }
                    "OE" if fields.len() == 1 => builder
                        .close()
                        .map_err(|code| error(&path, line_no, code))?,
                    "OC" => return Err(error(&path, line_no, "ODB_SURFACE_ARC_UNSUPPORTED")),
                    _ => return Err(error(&path, line_no, "ODB_SURFACE_BAD_RECORD")),
                }
            } else {
                if fields.len() != 1 {
                    return Err(error(&path, line_no, "ODB_SURFACE_BAD_SE"));
                }
                let (id, start_line, builder) = surface.take().expect("surface checked");
                let (outer, holes) = builder
                    .finish()
                    .map_err(|code| error(&path, line_no, code))?;
                let cutouts = holes
                    .iter()
                    .enumerate()
                    .map(|(n, hole)| Cutout {
                        meta: meta(
                            source,
                            ObjectKind::Cutout,
                            &format!("{id}/hole-{n}"),
                            &path,
                            start_line,
                        ),
                        boundary: surface_boundary(&frame_id, hole),
                    })
                    .collect();
                board.areas.push(CopperArea {
                    meta: meta(source, ObjectKind::Area, &id, &path, start_line),
                    layer: layer_id.clone(),
                    net: None,
                    outer: surface_boundary(&frame_id, &outer),
                    cutouts,
                });
            }
            continue;
        }
        if let Some(u) = line.strip_prefix("UNITS=") {
            if unit.is_some() {
                return Err(error(&path, line_no, "ODB_DUPLICATE_UNITS"));
            }
            unit = match u.trim() {
                "MM" => Some("MM"),
                "INCH" => Some("INCH"),
                _ => return Err(error(&path, line_no, "ODB_UNSUPPORTED_UNITS")),
            };
            continue;
        }
        if let Some(f) = line.strip_prefix("F ") {
            if advertised_count.is_some() {
                return Err(error(&path, line_no, "ODB_DUPLICATE_FEATURE_COUNT"));
            }
            advertised_count = Some(
                f.trim()
                    .parse::<usize>()
                    .map_err(|_| error(&path, line_no, "ODB_BAD_FEATURE_COUNT"))?,
            );
            continue;
        }
        if let Some(decl) = line.strip_prefix('$') {
            let p: Vec<&str> = decl.split_whitespace().collect();
            if p.len() < 2 || p.len() > 3 {
                return Err(error(&path, line_no, "ODB_BAD_SYMBOL_DECL"));
            }
            let serial = positive_symbol_name(p[0], &path, line_no)?;
            // The default for standard symbols is microns; per-symbol I/M overrides.
            let mode = if let Some(explicit) = p.get(2).copied() {
                explicit
            } else {
                match unit {
                    Some("MM") => "M",
                    Some("INCH") => "I",
                    _ => return Err(error(&path, line_no, "ODB_SYMBOL_BEFORE_UNITS")),
                }
            };
            let parsed = parse_symbol(p[1], mode, &path, line_no)?;
            if symbols.insert(serial, parsed).is_some() {
                return Err(error(&path, line_no, "ODB_DUPLICATE_SYMBOL"));
            }
            continue;
        }
        if line.starts_with('@') || line.starts_with('&') || line.starts_with("ID=") {
            // Attribute lookup definitions are retained in the source bytes but cannot
            // be used as a fabricated Net or connection. No electrical edge is added.
            continue;
        }
        let (record, extras) = line.split_once(';').unwrap_or((line, ""));
        let fields: Vec<&str> = record.split_whitespace().collect();
        if fields.is_empty() {
            continue;
        }
        let kind = fields[0];
        if !matches!(kind, "L" | "P" | "A" | "S") {
            // Custom symbols, text, drill and other copper forms
            // cannot be silently skipped as if the layer's copper were complete.
            return Err(error(&path, line_no, "ODB_UNSUPPORTED_CONDUCTIVE_FEATURE"));
        }
        if unit.is_none() {
            return Err(error(&path, line_no, "ODB_MISSING_UNITS"));
        }
        feature_count += 1;
        if feature_count > MAX_FEATURES {
            return Err(error(&path, line_no, "ODB_TOO_MANY_FEATURES"));
        }
        for field in extras.split(';') {
            for part in field.split(',') {
                if let Some(id) = part.trim().strip_prefix("ID=") {
                    if id.is_empty() || !ids.insert(id.to_owned()) {
                        return Err(error(&path, line_no, "ODB_DUPLICATE_OR_BAD_FEATURE_ID"));
                    }
                }
            }
        }
        let feature_id = format!("{path}:{line_no}");
        let loc = match kind {
            "L" => FeatureLocation::Trace(board.traces.len()),
            "P" => FeatureLocation::Pad(board.pads.len()),
            "A" => FeatureLocation::Arc(board.arcs.len()),
            "S" => FeatureLocation::Area(board.areas.len()),
            _ => return Err(error(&path, line_no, "ODB_UNSUPPORTED_CONDUCTIVE_FEATURE")),
        };
        // FID feature numbers are record ordinals (zero-based), not file
        // source-line numbers, and S/OB/OS/OE/SE counts as *one* feature.
        if registry
            .insert((layer.name.clone(), feature_count - 1), loc)
            .is_some()
        {
            return Err(error(&path, line_no, "ODB_EDA_DUPLICATE_FEATURE_ORDINAL"));
        }
        if kind == "S" {
            if fields.len() != 3 {
                return Err(error(&path, line_no, "ODB_SURFACE_BAD_ARITY"));
            }
            if fields[1] != "P" {
                return Err(error(&path, line_no, "ODB_SURFACE_NEGATIVE_UNSUPPORTED"));
            }
            let _dcode = fields[2]
                .parse::<u32>()
                .map_err(|_| error(&path, line_no, "ODB_BAD_DCODE"))?;
            surface = Some((feature_id, line_no, SurfaceBuilder::default()));
        } else if kind == "L" {
            if fields.len() != 8 {
                return Err(error(&path, line_no, "ODB_BAD_LINE_ARITY"));
            }
            if fields[6] != "P" {
                return Err(error(&path, line_no, "ODB_NEGATIVE_OR_UNKNOWN_COPPER"));
            }
            let sym = positive_symbol_name(fields[5], &path, line_no)?;
            let width_m = match symbols.get(&sym) {
                Some(Symbol::Round(x)) => *x,
                _ => return Err(error(&path, line_no, "ODB_UNSUPPORTED_LINE_SYMBOL")),
            };
            let _dcode = fields[7]
                .parse::<u32>()
                .map_err(|_| error(&path, line_no, "ODB_BAD_DCODE"))?;
            let first = point(&frame_id, fields[1], fields[2], &path, line_no)?;
            let last = point(&frame_id, fields[3], fields[4], &path, line_no)?;
            // ODB++ permits zero-length copper lines, but Canonical Trace
            // forbids coincident consecutive points. Do not output an
            // unvalidatable model; point-copper conversion is not accepted yet.
            if first == last {
                return Err(error(&path, line_no, "ODB_ZERO_LENGTH_LINE_UNSUPPORTED"));
            }
            board.traces.push(Trace {
                meta: meta(source, ObjectKind::Trace, &feature_id, &path, line_no),
                layer: layer_id.clone(),
                net: None,
                centreline: vec![first, last],
                width_m,
            });
        } else if kind == "P" {
            if fields.len() != 7 {
                return Err(error(&path, line_no, "ODB_BAD_PAD_ARITY"));
            }
            if fields[4] != "P" {
                return Err(error(&path, line_no, "ODB_NEGATIVE_OR_UNKNOWN_COPPER"));
            }
            let sym = positive_symbol_name(fields[3], &path, line_no)?;
            let shape = match symbols.get(&sym) {
                Some(Symbol::Round(x)) => PadShape::Circle { diameter_m: *x },
                Some(Symbol::Rectangle(x)) => PadShape::Rectangle { size_m: *x },
                None => return Err(error(&path, line_no, "ODB_UNKNOWN_PAD_SYMBOL")),
            };
            let _dcode = fields[5]
                .parse::<u32>()
                .map_err(|_| error(&path, line_no, "ODB_BAD_DCODE"))?;
            let orientation = orientation(fields[6], &path, line_no)?;
            let stack_id = ObjectId::new(source, ObjectKind::Padstack, &feature_id, "");
            board.padstacks.push(Padstack {
                meta: meta(source, ObjectKind::Padstack, &feature_id, &path, line_no),
                entries: vec![PadstackLayer {
                    layer: layer_id.clone(),
                    role: PadRole::Conductive,
                    shape,
                }],
            });
            board.pads.push(Pad {
                meta: meta(source, ObjectKind::Pad, &feature_id, &path, line_no),
                position: point(&frame_id, fields[1], fields[2], &path, line_no)?,
                orientation,
                padstack: stack_id,
                net: None,
            });
        } else {
            if fields.len() != 11 {
                return Err(error(&path, line_no, "ODB_BAD_ARC_ARITY"));
            }
            if fields[8] != "P" {
                return Err(error(&path, line_no, "ODB_NEGATIVE_OR_UNKNOWN_COPPER"));
            }
            let sym = positive_symbol_name(fields[7], &path, line_no)?;
            let width_m = match symbols.get(&sym) {
                Some(Symbol::Round(x)) => *x,
                _ => return Err(error(&path, line_no, "ODB_UNSUPPORTED_ARC_SYMBOL")),
            };
            let _dcode = fields[9]
                .parse::<u32>()
                .map_err(|_| error(&path, line_no, "ODB_BAD_DCODE"))?;
            let clockwise = match fields[10] {
                "Y" => true,
                "N" => false,
                _ => return Err(error(&path, line_no, "ODB_BAD_ARC_DIRECTION")),
            };
            let start = point(&frame_id, fields[1], fields[2], &path, line_no)?;
            let end = point(&frame_id, fields[3], fields[4], &path, line_no)?;
            let centre = point(&frame_id, fields[5], fields[6], &path, line_no)?;
            // ODB permits 360-degree arcs with matching start/end. The Canonical
            // Arc contract currently rejects start==end, so never emit an invalid
            // BoardModel or degrade the circle into a zero-sweep arc.
            if start == end {
                return Err(error(&path, line_no, "ODB_FULL_CIRCLE_UNSUPPORTED"));
            }
            // Check the source-grid circle radius exactly, without floating point.
            let r2 = |p: &Point| -> Option<i128> {
                let dx = i128::from(p.ticks[0]) - i128::from(centre.ticks[0]);
                let dy = i128::from(p.ticks[1]) - i128::from(centre.ticks[1]);
                dx.checked_mul(dx)?.checked_add(dy.checked_mul(dy)?)
            };
            let a = r2(&start).ok_or_else(|| error(&path, line_no, "ODB_ARC_OVERFLOW"))?;
            let b = r2(&end).ok_or_else(|| error(&path, line_no, "ODB_ARC_OVERFLOW"))?;
            if a == 0 || a != b {
                return Err(error(&path, line_no, "ODB_ARC_RADIUS_CONFLICT"));
            }
            board.arcs.push(Arc {
                meta: meta(source, ObjectKind::Arc, &feature_id, &path, line_no),
                layer: layer_id.clone(),
                net: None,
                start,
                end,
                centre,
                clockwise,
                width_m,
            });
        }
    }
    if surface.is_some() {
        return Err(error(&path, text.lines().count(), "ODB_SURFACE_MISSING_SE"));
    }
    let unit = unit.ok_or_else(|| error(&path, 0, "ODB_MISSING_UNITS"))?;
    if let Some(expected) = advertised_count {
        if expected != feature_count {
            return Err(error(&path, 0, "ODB_FEATURE_COUNT_CONFLICT"));
        }
    }
    board.frames.push(CoordinateFrame {
        meta: meta(source, ObjectKind::Frame, &path, &path, 0),
        unit_label: format!("ODB++Design {unit}; exact 1e-9 source unit ticks"),
        metres_per_tick: if unit == "MM" {
            Rational::new(1, 1_000_000_000_000)
                .map_err(|_| error(&path, 0, "ODB_FRAME_RATIONAL"))?
        } else {
            Rational::new(254, 10_000_000_000_000)
                .map_err(|_| error(&path, 0, "ODB_FRAME_RATIONAL"))?
        },
        origin_m: [Rational::integer(0); 2],
        orientation: Orientation::Orthogonal {
            turn: QuarterTurn::Zero,
            mirror_x: false,
        },
        source_layer_order_reversed: false,
    });
    Ok(())
}

/// Read an already-unpacked product-model directory without following links.
fn read_directory(root: &std::path::Path) -> Result<OdbFileMap, OdbError> {
    fn walk(
        root: &std::path::Path,
        dir: &std::path::Path,
        depth: usize,
        total: &mut usize,
        files: &mut OdbFileMap,
    ) -> Result<(), OdbError> {
        if depth > 16 {
            return Err(error("directory", 0, "ODB_DIRECTORY_DEPTH"));
        }
        let entries =
            std::fs::read_dir(dir).map_err(|_| error("directory", 0, "ODB_DIRECTORY_IO"))?;
        for entry in entries {
            let entry = entry.map_err(|_| error("directory", 0, "ODB_DIRECTORY_IO"))?;
            let full = entry.path();
            let relative = full
                .strip_prefix(root)
                .map_err(|_| error("directory", 0, "ODB_UNSAFE_ROOT"))?;
            let relative = relative
                .to_str()
                .ok_or_else(|| error("directory", 0, "ODB_UNSUPPORTED_PATH_ENCODING"))?
                .replace(std::path::MAIN_SEPARATOR, "/");
            if !safe_path(&relative) {
                return Err(error(&relative, 0, "ODB_UNSAFE_LOGICAL_PATH"));
            }
            let info = std::fs::symlink_metadata(&full)
                .map_err(|_| error(&relative, 0, "ODB_ENTRY_IO"))?;
            if info.file_type().is_symlink() {
                return Err(error(&relative, 0, "ODB_SYMLINK_UNSUPPORTED"));
            }
            if info.is_dir() {
                walk(root, &full, depth + 1, total, files)?;
            } else if info.is_file() {
                let len: usize = info
                    .len()
                    .try_into()
                    .map_err(|_| error(&relative, 0, "ODB_FILE_TOO_LARGE"))?;
                if len > MAX_FILE_BYTES {
                    return Err(error(&relative, 0, "ODB_FILE_TOO_LARGE"));
                }
                *total = total
                    .checked_add(len)
                    .ok_or_else(|| error(&relative, 0, "ODB_SIZE_OVERFLOW"))?;
                if *total > MAX_TOTAL_BYTES || files.len() >= MAX_FILES {
                    return Err(error(&relative, 0, "ODB_IMPORT_LIMIT"));
                }
                let bytes =
                    std::fs::read(&full).map_err(|_| error(&relative, 0, "ODB_ENTRY_IO"))?;
                if bytes.len() != len || bytes.len() > MAX_FILE_BYTES {
                    return Err(error(&relative, 0, "ODB_FILE_RACE_OR_TOO_LARGE"));
                }
                if files.insert(relative.clone(), bytes).is_some() {
                    return Err(error(&relative, 0, "ODB_DUPLICATE_PATH"));
                }
            } else {
                return Err(error(&relative, 0, "ODB_NON_REGULAR_FILE"));
            }
        }
        Ok(())
    }
    let root_info =
        std::fs::symlink_metadata(root).map_err(|_| error("directory", 0, "ODB_DIRECTORY_IO"))?;
    if !root_info.is_dir() || root_info.file_type().is_symlink() {
        return Err(error("directory", 0, "ODB_UNSAFE_ROOT"));
    }
    let mut files = OdbFileMap::new();
    let mut total = 0;
    walk(root, root, 0, &mut total, &mut files)?;
    Ok(files)
}

/// Geometry-only import from an already-unpacked ODB++ product root.
pub fn import_directory(root: &std::path::Path, step: &str) -> Result<BoardModel, OdbError> {
    import_unpacked(&read_directory(root)?, step)
}

/// Opt-in EDA NET/SNT/FID Source assignment import (no electrical graph).
pub fn import_directory_with_eda(
    root: &std::path::Path,
    step: &str,
) -> Result<BoardModel, OdbError> {
    import_unpacked_with_eda(&read_directory(root)?, step)
}

fn verified_source_files(files: &OdbFileMap) -> Result<Vec<SourceFile>, OdbError> {
    if files.len() > MAX_FILES {
        return Err(error("", 0, "ODB_TOO_MANY_FILES"));
    }
    let mut total = 0usize;
    let mut source_files = Vec::with_capacity(files.len());
    for (path, bytes) in files {
        if !safe_path(path) {
            return Err(error(path, 0, "ODB_UNSAFE_LOGICAL_PATH"));
        }
        if bytes.len() > MAX_FILE_BYTES {
            return Err(error(path, 0, "ODB_FILE_TOO_LARGE"));
        }
        total = total
            .checked_add(bytes.len())
            .ok_or_else(|| error(path, 0, "ODB_SIZE_OVERFLOW"))?;
        if total > MAX_TOTAL_BYTES {
            return Err(error(path, 0, "ODB_TOTAL_TOO_LARGE"));
        }
        source_files.push(SourceFile {
            logical_path: path.clone(),
            sha256: crate::digest::sha256(bytes),
        });
    }
    Ok(source_files)
}

/// Independent, source-only Drill Tool / Matrix span audit. No hole positions,
/// Canonical Via/Drill objects or manufactured/electrical edges are synthesized.
pub fn audit_unpacked_drill_tools(
    files: &OdbFileMap,
    step: &str,
) -> Result<Vec<SourceDrillLayer>, OdbError> {
    if !safe_entity(step) {
        return Err(error("steps", 0, "ODB_BAD_SELECTED_STEP"));
    }
    // Independently validate bounded source file paths, sizes, and SHA
    // without requiring the incomplete Copper geometry parser to accept a
    // valid but not-yet-supported ODB++ artwork symbol or contour.
    let _source_files = verified_source_files(files)?;
    let matrix = parse_matrix(ascii_file(files, "matrix/matrix")?)?;
    if !matrix.steps.iter().any(|s| s == step) {
        return Err(error("matrix/matrix", 0, "ODB_UNDECLARED_STEP"));
    }
    let copper: Vec<String> = matrix
        .layers
        .iter()
        .filter(|l| l.context == "BOARD" && copper_type(&l.layer_type))
        .map(|l| l.name.clone())
        .collect();
    let matrix_sha256 = crate::digest::sha256(
        files
            .get("matrix/matrix")
            .ok_or_else(|| error("matrix/matrix", 0, "ODB_MISSING_FILE"))?,
    );
    if !(2..=64).contains(&copper.len()) {
        return Err(error(
            "matrix/matrix",
            0,
            "ODB_UNSUPPORTED_COPPER_LAYER_COUNT",
        ));
    }
    let mut records = Vec::new();
    for layer in &matrix.layers {
        if layer.context != "BOARD" || layer.layer_type != "DRILL" {
            continue;
        }
        if layer.polarity != "POSITIVE" || records.len() >= 256 {
            return Err(error("matrix/matrix", 0, "ODB_DRILL_LAYER_UNSUPPORTED"));
        }
        if !matches!(layer.add_type.as_str(), "" | "BACKDRILL" | "DUAL_DIAMETER") {
            return Err(error("matrix/matrix", 0, "ODB_DRILL_SUBTYPE_UNSUPPORTED"));
        }
        let (start, end) = match (layer.start_name.as_str(), layer.end_name.as_str()) {
            ("", "") => (None, None), // Source defaults are NOT copper-span evidence.
            ("", _) | (_, "") => {
                return Err(error("matrix/matrix", 0, "ODB_DRILL_INCOMPLETE_SPAN"))
            }
            (a, z) => {
                let first = copper
                    .iter()
                    .position(|x| x == a)
                    .ok_or_else(|| error("matrix/matrix", 0, "ODB_DRILL_SPAN_NOT_COPPER"))?;
                let last = copper
                    .iter()
                    .position(|x| x == z)
                    .ok_or_else(|| error("matrix/matrix", 0, "ODB_DRILL_SPAN_NOT_COPPER"))?;
                if first >= last {
                    return Err(error("matrix/matrix", 0, "ODB_DRILL_REVERSED_OR_ZERO_SPAN"));
                }
                (Some(a.to_owned()), Some(z.to_owned()))
            }
        };
        let path = format!("steps/{step}/layers/{}/tools", layer.name);
        let content = ascii_file(files, &path)?;
        let (unit, tools) = parse_tools(content).map_err(|e| error(&path, e.line, e.code))?;
        records.push(SourceDrillLayer {
            name: layer.name.clone(),
            matrix_row: layer.row,
            matrix_source_line: layer.source_line,
            matrix_sha256,
            unit,
            subtype: layer.add_type.clone(),
            explicit_start_layer: start,
            explicit_end_layer: end,
            source_file: path.clone(),
            source_sha256: crate::digest::sha256(
                files
                    .get(&path)
                    .ok_or_else(|| error(&path, 0, "ODB_MISSING_FILE"))?,
            ),
            tools,
            geometry_hits: None,
        });
    }
    Ok(records)
}

/// Audit drill tooling from an already-unpacked folder; safe path/symlink and
/// resource checks are inherited from read_directory.
pub fn audit_directory_drill_tools(
    root: &std::path::Path,
    step: &str,
) -> Result<Vec<SourceDrillLayer>, OdbError> {
    audit_unpacked_drill_tools(&read_directory(root)?, step)
}

/// Fully enumerate bounded drill-layer features P hits and join explicit
/// nonzero Excellon dcode to layer/tools NUM. Independent of Copper artwork.
/// A tool audit cannot prove plated barrels, connected layers or SI.
pub fn audit_unpacked_drill_hits(
    files: &OdbFileMap,
    step: &str,
) -> Result<Vec<SourceDrillHit>, OdbError> {
    let drill_layers = audit_unpacked_drill_tools(files, step)?;
    let mut out = Vec::new();
    for layer in drill_layers {
        let path = format!("steps/{step}/layers/{}/features", layer.name);
        let bytes = files
            .get(&path)
            .ok_or_else(|| error(&path, 0, "ODB_MISSING_FILE"))?;
        let text = std::str::from_utf8(bytes)
            .map_err(|_| error(&path, 0, "ODB_INVALID_TEXT"))?;
        let hits = parse_drill_hits(text, &layer, &path, crate::digest::sha256(bytes))
            .map_err(|e| error(&path, e.line, e.code))?;
        if out.len().checked_add(hits.len()).map_or(true, |n| n > MAX_FEATURES) {
            return Err(error(&path, 0, "ODB_DRILL_HIT_TOTAL_LIMIT"));
        }
        out.extend(hits);
    }
    Ok(out)
}

/// Read a native unpacked directory with identical symlink/size/path limits.
pub fn audit_directory_drill_hits(
    root: &std::path::Path,
    step: &str,
) -> Result<Vec<SourceDrillHit>, OdbError> {
    audit_unpacked_drill_hits(&read_directory(root)?, step)
}

/// Source-limited canonical import. All conductive records in the selected step's
/// copper layers must use the accepted syntax or the entire import fails.
pub fn import_unpacked(files: &OdbFileMap, step: &str) -> Result<BoardModel, OdbError> {
    import_inner(files, step, false)
}

/// Strict opt-in import of design Net identities and EDA FID ownership.
/// Unsupported EDA syntax returns an error for the *entire* import.
pub fn import_unpacked_with_eda(files: &OdbFileMap, step: &str) -> Result<BoardModel, OdbError> {
    import_inner(files, step, true)
}

fn import_inner(files: &OdbFileMap, step: &str, with_eda: bool) -> Result<BoardModel, OdbError> {
    if !safe_entity(step) {
        return Err(error("steps", 0, "ODB_BAD_SELECTED_STEP"));
    }
    let source_files = verified_source_files(files)?;
    let matrix = parse_matrix(ascii_file(files, "matrix/matrix")?)?;
    if !matrix.steps.iter().any(|s| s == step) {
        return Err(error("matrix/matrix", 0, "ODB_UNDECLARED_STEP"));
    }
    // Nested step-repeat transformations are not source-qualified in this slice.
    let stephdr = format!("steps/{step}/stephdr");
    if let Some(bytes) = files.get(&stephdr) {
        let header =
            std::str::from_utf8(bytes).map_err(|_| error(&stephdr, 0, "ODB_INVALID_TEXT"))?;
        if header.contains("STEP-REPEAT") {
            return Err(error(&stephdr, 0, "ODB_UNSUPPORTED_STEP_REPEAT"));
        }
    }
    let copper: Vec<&MatrixLayer> = matrix
        .layers
        .iter()
        .filter(|l| l.context == "BOARD" && copper_type(&l.layer_type))
        .collect();
    if !(2..=64).contains(&copper.len()) {
        return Err(error(
            "matrix/matrix",
            0,
            "ODB_UNSUPPORTED_COPPER_LAYER_COUNT",
        ));
    }
    if matrix
        .layers
        .iter()
        .any(|l| l.context != "BOARD" && l.context != "MISC")
    {
        return Err(error("matrix/matrix", 0, "ODB_UNKNOWN_LAYER_CONTEXT"));
    }
    if copper.iter().any(|l| l.polarity != "POSITIVE") {
        return Err(error("matrix/matrix", 0, "ODB_UNSUPPORTED_LAYER_POLARITY"));
    }
    let source: SourceId = Shared::new(SourceIdentity {
        format: SourceFormat::Odb,
        format_version: "8.1-U4-grammar-subset;export-revision-unverified".into(),
        files: source_files,
        importer_version: if with_eda {
            "d056-08-odb-source-subset-v2-eda-fid"
        } else {
            "d056-08-odb-source-subset-v1"
        }
        .into(),
        conditions: if with_eda {
            vec![
                ("selected_step".into(), step.into()),
                (
                    "supported_geometry".into(),
                    "positive-LPA-round-line-and-round-rect-pad".into(),
                ),
                ("with_eda_fid_ownership".into(), "true".into()),
            ]
        } else {
            // Bit-for-bit preserve the geometry-only SourceIdentity contract.
            vec![
                ("selected_step".into(), step.into()),
                (
                    "supported_geometry".into(),
                    "positive-LPA-round-line-and-round-rect-pad".into(),
                ),
            ]
        },
    });
    let mut board = BoardModel::empty(BoardKind::Pcb);
    board.sources.push(ImportSource {
        id: source.clone(), producer: "ODB++Design source adapter".into(),
        producer_version: "source revision unverified".into(),
        capability: ImportCapability {
            geometry: CapabilityState::Partial, net: CapabilityState::Missing,
            pin_pad: CapabilityState::Missing, stackup: CapabilityState::Partial,
            reference: CapabilityState::Missing, model: CapabilityState::Missing,
            object_coverage: vec![
                "matrix BOARD copper ordering, selected STEP, source-valid positive L/P/A and bounded simple S island/hole".into(),
                "No production EDA packages, manufacture drills, plated barrel, complete surfaces, step repeat, or SI".into(),
            ],
        },
    });
    for (index, layer) in copper.iter().enumerate() {
        board.stackup.layers.push(Layer {
            meta: meta(
                &source,
                ObjectKind::Layer,
                &format!("matrix:{}", layer.name),
                "matrix/matrix",
                0,
            ),
            name: layer.name.clone(),
            order: index as u32,
            kind: LayerKind::Copper,
            z_m: Property::Missing,
            thickness_m: Property::Missing,
            material: None,
        });
    }
    let mut registry: BTreeMap<(String, usize), FeatureLocation> = BTreeMap::new();
    for layer in copper {
        let path = format!("steps/{step}/layers/{}/features", layer.name);
        parse_features_content(
            &mut board,
            &source,
            step,
            layer,
            ascii_file(files, &path)?,
            &mut registry,
        )?;
    }
    if with_eda {
        let path = format!("steps/{step}/eda/data");
        let mapping = odb_eda::parse(ascii_file(files, &path)?, &registry)
            .map_err(|e| error(&path, e.line, e.code))?;
        // Complete parsing and duplicate/reference checks precede all model
        // mutations; an invalid input is never returned as a partial success.
        let mut net_ids = BTreeMap::new();
        for entry in mapping.nets {
            if entry.name == "$NONE$" {
                continue; // ODB explicitly designates this as unassigned.
            }
            let id = ObjectId::new(
                &source,
                ObjectKind::Net,
                &format!("eda/data:{}:{}", entry.source_line, entry.name),
                "",
            );
            board.nets.push(Net {
                meta: meta(
                    &source,
                    ObjectKind::Net,
                    &format!("eda/data:{}:{}", entry.source_line, entry.name),
                    &path,
                    entry.source_line,
                ),
                name: entry.name.clone(),
                role: NetRole::Unknown,
            });
            net_ids.insert(entry.name, id);
        }
        for assignment in mapping.assignments {
            let id = net_ids
                .get(&assignment.net_name)
                .ok_or_else(|| error(&path, assignment.source_line, "ODB_EDA_UNKNOWN_NET"))?
                .clone();
            // Keep the exact FID source-line witness on each owned object.
            // Geometry truth and design-Net ownership are distinct evidence;
            // neither constitutes a plated contact or an electrical edge.
            let witness = ConversionStep {
                operation: "odb-eda-fid-design-net-ownership".into(),
                quality: ConversionQuality::Exact,
                max_error_m: Some(Rational::integer(0)),
                note: format!(
                    "{path}:line:{}; net={}; design ownership only; NOT manufactured continuity",
                    assignment.source_line, assignment.net_name
                ),
            };
            match registry
                .get(&(assignment.layer, assignment.feature_index))
                .ok_or_else(|| error(&path, assignment.source_line, "ODB_EDA_UNRESOLVED_FID"))?
            {
                FeatureLocation::Trace(n) => {
                    board.traces[*n].net = Some(id);
                    board.traces[*n].meta.provenance.history.push(witness);
                }
                FeatureLocation::Pad(n) => {
                    board.pads[*n].net = Some(id);
                    board.pads[*n].meta.provenance.history.push(witness);
                }
                FeatureLocation::Arc(n) => {
                    board.arcs[*n].net = Some(id);
                    board.arcs[*n].meta.provenance.history.push(witness);
                }
                FeatureLocation::Area(n) => {
                    board.areas[*n].net = Some(id);
                    board.areas[*n].meta.provenance.history.push(witness);
                }
            }
        }
        board.sources[0].capability.net = CapabilityState::Partial;
        board.sources[0].capability.object_coverage.push(
            "opt-in EDA NET/SNT/FID design Net ownership only; no plating or interconnect proof"
                .into(),
        );
    }
    // No global connectivity Graph or manufactured SI result.
    debug_assert!(board.connectivity.nodes.is_empty() && board.connectivity.edges.is_empty());
    Ok(board)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source(layers: usize) -> OdbFileMap {
        let mut f = OdbFileMap::new();
        let mut matrix = "STEP {\nCOL=1\nNAME=pcb\n}\n".to_string();
        for layer in 1..=layers {
            matrix.push_str(&format!("LAYER {{\nROW={layer}\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=sig{layer}\nPOLARITY=POSITIVE\n}}\n"));
            f.insert(format!("steps/pcb/layers/sig{layer}/features"),
                b"UNITS=MM\nF 3\n$0 r100\n$1 rect200x100 M\nL 0 0 1.5 0 0 P 0\nP 1.5 0 1 P 0 1\nA 1 0 0 1 0 0 0 P 0 Y\n".to_vec());
        }
        f.insert("matrix/matrix".into(), matrix.into_bytes());
        f
    }
    #[test]
    fn flexible_layers_and_canonical_shapes() {
        for n in [2, 4, 6, 8, 64] {
            let board = import_unpacked(&source(n), "pcb").unwrap();
            let report = board.validate();
            assert!(
                !report.has_errors(),
                "source-supported ODB should produce structurally valid Canonical model: {:?}",
                report.issues
            );
            assert_eq!(board.stackup.layers.len(), n);
            assert_eq!(board.traces.len(), n);
            assert_eq!(board.pads.len(), n);
            assert_eq!(board.arcs.len(), n);
            assert_eq!(board.frames.len(), n);
            assert_eq!(
                board.sources[0].capability.geometry,
                CapabilityState::Partial
            );
            assert!(board.connectivity.edges.is_empty());
            assert!(board.vias.is_empty());
            assert!(board.traces[0].net.is_none());
            assert_eq!(board.traces[0].centreline[1].ticks[0], 1_500_000_000);
        }
    }
    #[test]
    fn source_units_and_fractional_standard_symbols_are_exact() {
        let path = "steps/pcb/layers/sig1/features";
        let mut metric = source(2);
        let source_text = String::from_utf8(metric[path].clone()).unwrap();
        let fractional = source_text
            .replace("$0 r100", "$0 r3.25 I")
            .replace("$1 rect200x100 M", "$1 rect200.5x100.25 M");
        metric.insert(path.into(), fractional.into_bytes());
        let board = import_unpacked(&metric, "pcb").unwrap();
        assert_eq!(
            board.traces[0].width_m,
            Rational::new(8255, 100_000_000).unwrap()
        );
        assert_eq!(
            board.padstacks[0].entries[0].shape,
            PadShape::Rectangle {
                size_m: [
                    Rational::new(2005, 10_000_000).unwrap(),
                    Rational::new(10025, 100_000_000).unwrap(),
                ]
            }
        );

        let mut imperial = source(2);
        for layer in ["sig1", "sig2"] {
            let path = format!("steps/pcb/layers/{layer}/features");
            let contents = String::from_utf8(imperial[&path].clone())
                .unwrap()
                .replace("UNITS=MM", "UNITS=INCH")
                .replace("$1 rect200x100 M", "$1 rect200x100 I");
            imperial.insert(path, contents.into_bytes());
        }
        let inch_board = import_unpacked(&imperial, "pcb").unwrap();
        // Unqualified $0 r100 inherits INCH and means 100 mil, not 100 micrometres.
        assert_eq!(
            inch_board.traces[0].width_m,
            Rational::new(254, 100_000).unwrap()
        );
        assert_eq!(
            inch_board.frames[0].metres_per_tick,
            Rational::new(254, 10_000_000_000_000).unwrap()
        );
    }
    #[test]
    fn single_odb_surface_with_source_hole_becomes_canonical_area() {
        let mut f = source(2);
        let path = "steps/pcb/layers/sig1/features";
        let data = String::from_utf8(f[path].clone())
            .unwrap()
            .replace("F 3", "F 4")
            + "S P 0;ID=8401\nOB 0 0 I\nOS 4 0\nOS 4 4\nOS 0 4\nOS 0 0\nOE\nOB 1 1 H\nOS 1 2\nOS 2 2\nOS 2 1\nOS 1 1\nOE\nSE\n";
        f.insert(path.into(), data.into_bytes());
        let board = import_unpacked(&f, "pcb").unwrap();
        assert_eq!(board.areas.len(), 1);
        assert_eq!(board.areas[0].outer.segments.len(), 4);
        assert_eq!(board.areas[0].cutouts.len(), 1);
        assert_eq!(board.areas[0].cutouts[0].boundary.segments.len(), 4);
        assert_eq!(board.areas[0].layer, board.traces[0].layer);
        assert!(!board.validate().has_errors());
        assert!(board.connectivity.edges.is_empty());
    }
    #[test]
    fn malformed_and_unknown_odb_surfaces_fail_closed() {
        let base = "S P 0\nOB 0 0 I\nOS 4 0\nOS 4 4\nOS 0 4\nOS 0 0\nOE\nSE\n";
        for (replacement, expected) in [
            ("S N 0", "ODB_SURFACE_NEGATIVE_UNSUPPORTED"),
            ("S P 0\nOB 0 0 I\nOS 4 0\nOS 4 4\nOS 0 4\nOS 0 0\nOE\n", "ODB_SURFACE_MISSING_SE"),
            ("S P 0\nOB 0 0 I\nOC 4 0 2 0 Y\nOE\nSE\n", "ODB_SURFACE_ARC_UNSUPPORTED"),
            ("S P 0\nOB 0 0 I\nOS 4 4\nOS 0 4\nOS 4 0\nOS 0 0\nOE\nSE\n", "ODB_SURFACE_SELF_INTERSECTION"),
            ("S P 0\nOB 0 0 H\nOS 4 0\nOS 4 4\nOS 0 4\nOS 0 0\nOE\nSE\n", "ODB_SURFACE_HOLE_BEFORE_ISLAND"),
            ("S P 0\nOB 0 0 I\nOS 4 0\nOS 4 4\nOS 0 4\nOS 0 0\nOE\nOB 10 10 H\nOS 11 10\nOS 11 11\nOS 10 11\nOS 10 10\nOE\nSE\n", "ODB_SURFACE_HOLE_OUTSIDE_OR_TOUCHING"),
        ] {
            let mut f = source(2);
            let path = "steps/pcb/layers/sig1/features";
            let record = if replacement == "S N 0" {
                base.replace("S P 0", replacement)
            } else {
                replacement.to_owned()
            };
            let data = String::from_utf8(f[path].clone())
                .unwrap()
                .replace("F 3", "F 4")
                + &record;
            f.insert(path.into(), data.into_bytes());
            assert_eq!(import_unpacked(&f, "pcb").unwrap_err().code, expected);
        }
    }
    #[test]
    fn opt_in_eda_net_fid_assigns_exact_design_ownership_only() {
        let mut f = source(4);
        f.insert(
            "steps/pcb/eda/data".into(),
            b"HDR Mentor PADS\nUNITS=MM\nLYR sig1 sig2 sig3 sig4\nNET DATA_P;ID=31\nSNT TRC\nFID C 0 0\nFID C 0 2\nNET DATA_N;ID=32\nSNT VIA\nFID C 0 1\nNET $NONE$\nSNT TRC\nFID C 1 0\n".to_vec(),
        );
        let untouched = import_unpacked(&f, "pcb").unwrap();
        assert!(untouched.nets.is_empty());
        assert!(untouched.traces.iter().all(|t| t.net.is_none()));
        let board = import_unpacked_with_eda(&f, "pcb").unwrap();
        assert_eq!(board.nets.len(), 2);
        assert_eq!(board.nets[0].name, "DATA_P");
        assert_eq!(board.nets[1].name, "DATA_N");
        assert_eq!(board.traces[0].net, Some(board.nets[0].meta.id.clone()));
        assert!(board.traces[0]
            .meta
            .provenance
            .history
            .iter()
            .any(|v| v.operation == "odb-eda-fid-design-net-ownership"
                && v.note.contains("eda/data:line:6")
                && v.note.contains("design ownership only")));
        assert_eq!(board.pads[0].net, Some(board.nets[1].meta.id.clone()));
        assert_eq!(board.arcs[0].net, Some(board.nets[0].meta.id.clone()));
        assert!(board.traces[1].net.is_none());
        assert_eq!(board.sources[0].capability.net, CapabilityState::Partial);
        assert!(board.connectivity.nodes.is_empty());
        assert!(board.connectivity.edges.is_empty());
        assert!(board.vias.is_empty());
        assert!(!board.validate().has_errors());
        assert_ne!(board.sources[0].id, untouched.sources[0].id);
    }

    #[test]
    fn eda_fid_source_mutations_and_missing_eda_are_fail_closed() {
        let mut f = source(2);
        assert_eq!(
            import_unpacked_with_eda(&f, "pcb").unwrap_err().code,
            "ODB_MISSING_FILE"
        );
        let path = "steps/pcb/eda/data";
        let valid = "HDR test\nUNITS=MM\nLYR sig1 sig2\nNET DATA\nSNT TRC\nFID C 0 0\n";
        f.insert(path.into(), valid.as_bytes().to_vec());
        assert!(!import_unpacked_with_eda(&f, "pcb")
            .unwrap()
            .validate()
            .has_errors());
        for (from, to, expected) in [
            ("FID C 0 0", "FID C 2 0", "ODB_EDA_BAD_LAYER_INDEX"),
            ("FID C 0 0", "FID C 0 99", "ODB_EDA_UNRESOLVED_FID"),
            ("FID C 0 0", "FID H 0 0", "ODB_EDA_UNSUPPORTED_FID"),
            ("SNT TRC", "SNT VIA", "ODB_EDA_FID_SUBNET_KIND_CONFLICT"),
            ("FID C 0 0", "FID C 0 0\nFID C 0 0", "ODB_EDA_DUPLICATE_FID"),
            ("SNT TRC", "SNT TOP T 0 0", "ODB_EDA_UNSUPPORTED_SUBNET"),
        ] {
            let mut altered = f.clone();
            altered.insert(path.into(), valid.replace(from, to).into_bytes());
            assert_eq!(
                import_unpacked_with_eda(&altered, "pcb").unwrap_err().code,
                expected
            );
        }
    }

    #[test]
    fn eda_source_hash_changes_net_object_identity_and_surface_fid_index() {
        let mut f = source(2);
        let path = "steps/pcb/layers/sig1/features";
        let copper = String::from_utf8(f[path].clone())
            .unwrap()
            .replace("F 3", "F 4")
            + "S P 0\nOB 0 0 I\nOS 3 0\nOS 3 3\nOS 0 3\nOS 0 0\nOE\nSE\n";
        f.insert(path.into(), copper.into_bytes());
        f.insert(
            "steps/pcb/eda/data".into(),
            b"HDR x\nUNITS=MM\nLYR sig1 sig2\nNET POWER\nSNT PLN S E 0\nFID C 0 3\n".to_vec(),
        );
        let a = import_unpacked_with_eda(&f, "pcb").unwrap();
        assert_eq!(a.areas.len(), 1);
        assert_eq!(a.areas[0].net, Some(a.nets[0].meta.id.clone()));
        assert!(!a.validate().has_errors());
        let mut changed = f.clone();
        changed
            .get_mut("steps/pcb/eda/data")
            .unwrap()
            .extend_from_slice(b"# source update\n");
        let b = import_unpacked_with_eda(&changed, "pcb").unwrap();
        assert_ne!(a.nets[0].meta.id, b.nets[0].meta.id);
        assert_ne!(a.areas[0].meta.id, b.areas[0].meta.id);
        assert!(b.connectivity.edges.is_empty());
    }

    #[test]
    fn declared_drill_span_and_tools_are_audit_only_not_generated_vias() {
        let mut f = source(6);
        let matrix = String::from_utf8(f["matrix/matrix"].clone()).unwrap()
            + "LAYER {\nROW=7\nCONTEXT=BOARD\nTYPE=DRILL\nNAME=through_drill\nPOLARITY=POSITIVE\nSTART_NAME=sig1\nEND_NAME=sig6\n}\n"
            + "LAYER {\nROW=8\nCONTEXT=BOARD\nTYPE=DRILL\nNAME=bd6\nPOLARITY=POSITIVE\nADD_TYPE=BACKDRILL\nSTART_NAME=sig4\nEND_NAME=sig6\n}\n";
        f.insert("matrix/matrix".into(), matrix.into_bytes());
        let path = "steps/pcb/layers/through_drill/tools";
        f.insert(path.into(),b"UNITS=MM\nTOOLS {\nNUM=1\nTYPE=PLATED\nTYPE2=STANDARD\nFINISH_SIZE=200\nDRILL_SIZE=220\n}\n".to_vec());
        f.insert(
            "steps/pcb/layers/bd6/tools".into(),
            b"UNITS=MM\nTOOLS {\nNUM=2\nTYPE=VIA\nTYPE2=STANDARD\nFINISH_SIZE=300\n}\n".to_vec(),
        );
        let layers = audit_unpacked_drill_tools(&f, "pcb").unwrap();
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0].name, "through_drill");
        assert_eq!(layers[0].explicit_start_layer.as_deref(), Some("sig1"));
        assert_eq!(layers[0].explicit_end_layer.as_deref(), Some("sig6"));
        assert_eq!(layers[0].tools[0].intent, DrillIntent::Plated);
        assert_ne!(layers[0].matrix_source_line, 0);
        assert_eq!(
            layers[0].matrix_sha256,
            crate::digest::sha256(&f["matrix/matrix"])
        );
        assert_eq!(layers[1].subtype, "BACKDRILL");
        assert_eq!(layers[1].tools[0].intent, DrillIntent::Via);
        assert_eq!(layers[0].source_sha256, crate::digest::sha256(&f[path]));
        let board = import_unpacked(&f, "pcb").unwrap();
        assert!(board.drills.is_empty());
        assert!(board.vias.is_empty());
        assert!(board.connectivity.edges.is_empty());
        assert!(!board.validate().has_errors());
    }

    #[test]
    fn drill_tools_audit_accepts_valid_but_geometry_unsupported_copper() {
        let mut f = source(2);
        let layer = "LAYER {\nROW=3\nCONTEXT=BOARD\nTYPE=DRILL\nNAME=drill\nPOLARITY=POSITIVE\nSTART_NAME=sig1\nEND_NAME=sig2\n}\n";
        let matrix = String::from_utf8(f["matrix/matrix"].clone()).unwrap() + layer;
        f.insert("matrix/matrix".into(), matrix.into_bytes());
        f.insert(
            "steps/pcb/layers/drill/tools".into(),
            b"UNITS=MM\nTOOLS {\nNUM=1\nTYPE=NON_PLATED\nFINISH_SIZE=100\n}\n".to_vec(),
        );
        // A real valid exporter may use Copper syntax outside this adapter's
        // restricted L/P/A/S model; tooling proof is independent of artwork.
        f.get_mut("steps/pcb/layers/sig1/features")
            .unwrap()
            .extend_from_slice(b"T unsupported-user-symbol\n");
        assert_eq!(
            import_unpacked(&f, "pcb").unwrap_err().code,
            "ODB_UNSUPPORTED_CONDUCTIVE_FEATURE"
        );
        let report = audit_unpacked_drill_tools(&f, "pcb").unwrap();
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].tools[0].intent, DrillIntent::NonPlated);
        assert_eq!(report[0].explicit_start_layer.as_deref(), Some("sig1"));
    }

    #[test]
    fn drill_source_span_unknown_and_unsafe_mutations_fail_closed() {
        let mut f = source(4);
        let layer = "LAYER {\nROW=5\nCONTEXT=BOARD\nTYPE=DRILL\nNAME=bd\nPOLARITY=POSITIVE\nADD_TYPE=BACKDRILL\n}\n";
        let matrix = String::from_utf8(f["matrix/matrix"].clone()).unwrap() + layer;
        f.insert("matrix/matrix".into(), matrix.into_bytes());
        let path = "steps/pcb/layers/bd/tools";
        f.insert(
            path.into(),
            b"UNITS=INCH\nTOOLS {\nNUM=1\nTYPE=VIA\nTYPE2=LASER\nFINISH_SIZE=11.5\n}\n".to_vec(),
        );
        let accepted = audit_unpacked_drill_tools(&f, "pcb").unwrap();
        assert_eq!(accepted[0].explicit_start_layer, None);
        assert_eq!(accepted[0].explicit_end_layer, None);
        for (mutation, expected) in [
            (
                layer.replace("ADD_TYPE=BACKDRILL", "ADD_TYPE=UNKNOWN"),
                "ODB_DRILL_SUBTYPE_UNSUPPORTED",
            ),
            (
                layer.replace("ADD_TYPE=BACKDRILL", "ADD_TYPE=BACKDRILL\nSTART_NAME=sig1"),
                "ODB_DRILL_INCOMPLETE_SPAN",
            ),
            (
                layer.replace(
                    "ADD_TYPE=BACKDRILL",
                    "ADD_TYPE=BACKDRILL\nSTART_NAME=sig2\nEND_NAME=sig1",
                ),
                "ODB_DRILL_REVERSED_OR_ZERO_SPAN",
            ),
            (
                layer.replace(
                    "ADD_TYPE=BACKDRILL",
                    "ADD_TYPE=BACKDRILL\nSTART_NAME=bogus\nEND_NAME=sig4",
                ),
                "ODB_DRILL_SPAN_NOT_COPPER",
            ),
        ] {
            let mut x = f.clone();
            x.insert(
                "matrix/matrix".into(),
                (String::from_utf8(source(4)["matrix/matrix"].clone()).unwrap() + &mutation)
                    .into_bytes(),
            );
            assert_eq!(
                audit_unpacked_drill_tools(&x, "pcb").unwrap_err().code,
                expected
            );
        }
        let mut missing = f.clone();
        missing.remove(path);
        assert_eq!(
            audit_unpacked_drill_tools(&missing, "pcb")
                .unwrap_err()
                .code,
            "ODB_MISSING_FILE"
        );
        let mut bad = f.clone();
        bad.insert(
            path.into(),
            b"UNITS=CM\nTOOLS {\nNUM=1\nTYPE=VIA\nFINISH_SIZE=11\n}\n".to_vec(),
        );
        assert_eq!(
            audit_unpacked_drill_tools(&bad, "pcb").unwrap_err().code,
            "ODB_TOOL_UNSUPPORTED_UNITS"
        );
    }

    #[test]
    fn source_declared_drill_hits_join_tools_without_inventing_vias() {
        let mut f = source(6);
        let matrix = String::from_utf8(f["matrix/matrix"].clone()).unwrap()
            + "LAYER {\nROW=7\nCONTEXT=BOARD\nTYPE=DRILL\nNAME=drill_a\nPOLARITY=POSITIVE\nSTART_NAME=sig1\nEND_NAME=sig6\n}\n";
        f.insert("matrix/matrix".into(), matrix.into_bytes());
        let tool = "steps/pcb/layers/drill_a/tools";
        let feat = "steps/pcb/layers/drill_a/features";
        f.insert(tool.into(),
            b"UNITS=MM\nTOOLS {\nNUM=7\nTYPE=VIA\nTYPE2=STANDARD\nFINISH_SIZE=110\n}\n".to_vec());
        f.insert(feat.into(),
            b"UNITS=MM\nF 2\n$0 r110\nP 1.25 -0.25 0 P 7 0;ID=a\nP 0 2.5 0 P 7 0;ID=b\n".to_vec());
        let hits = audit_unpacked_drill_hits(&f,"pcb").unwrap();
        assert_eq!(hits.len(),2);
        assert_eq!(hits[0].layer_name,"drill_a");
        assert_eq!(hits[0].tool_number,7);
        assert_eq!(hits[0].tool_intent,DrillIntent::Via);
        assert_eq!(hits[0].x,"1.25");
        assert_eq!(hits[0].y,"-0.25");
        assert_eq!(hits[0].source_line,4);
        assert_eq!(hits[0].source_sha256, crate::digest::sha256(&f[feat]));
        assert_eq!(hits[1].feature_index,1);
        // Unsupported Copper artwork is not a blocker for a tool/hit audit.
        f.get_mut("steps/pcb/layers/sig1/features")
            .unwrap().extend_from_slice(b"T unsupported\n");
        assert_eq!(audit_unpacked_drill_hits(&f,"pcb").unwrap().len(),2);
        assert!(import_unpacked(&f,"pcb").is_err());
        for (from,to,code) in [
            ("P 1.25 -0.25 0 P 7 0","P 1.25 -0.25 0 P 4 0","ODB_DRILL_HIT_UNKNOWN_TOOL"),
            ("F 2","F 3","ODB_DRILL_HIT_COUNT_MISMATCH"),
            ("$0 r110","$0 rect110x110","ODB_DRILL_HIT_NONCIRCULAR_SYMBOL"),
        ] {
            let mut mutated = f.clone();
            mutated.insert(feat.into(),
                String::from_utf8(f[feat].clone()).unwrap().replace(from,to).into_bytes());
            assert_eq!(audit_unpacked_drill_hits(&mutated,"pcb").unwrap_err().code,code);
        }
        f.remove(feat);
        assert_eq!(audit_unpacked_drill_hits(&f,"pcb").unwrap_err().code,"ODB_MISSING_FILE");
    }

    #[test]
    fn source_sha_mutation_changes_identity() {
        let f = source(2);
        let a = import_unpacked(&f, "pcb").unwrap();
        let mut modified = f.clone();
        modified
            .get_mut("steps/pcb/layers/sig1/features")
            .unwrap()
            .extend_from_slice(b"# altered\n");
        let b = import_unpacked(&modified, "pcb").unwrap();
        assert_ne!(a.sources[0].id, b.sources[0].id);
        assert_ne!(a.traces[0].meta.id, b.traces[0].meta.id);
    }
    #[test]
    fn reject_missing_mutated_and_unsupported_copper() {
        let p = "steps/pcb/layers/sig1/features";
        for (before, after, expected) in [
            (
                "L 0 0 1.5 0 0 P 0",
                "L 0 0 1.5 0 0 N 0",
                "ODB_NEGATIVE_OR_UNKNOWN_COPPER",
            ),
            ("P 1.5 0 1 P 0 1", "P 1.5 0 1 P 0 8 30", "ODB_BAD_PAD_ARITY"),
            ("F 3", "F 4", "ODB_FEATURE_COUNT_CONFLICT"),
            (
                "A 1 0 0 1 0 0 0 P 0 Y",
                "A 1 0 0 2 0 0 0 P 0 Y",
                "ODB_ARC_RADIUS_CONFLICT",
            ),
            (
                "A 1 0 0 1 0 0 0 P 0 Y",
                "A 1 0 1 0 0 0 0 P 0 Y",
                "ODB_FULL_CIRCLE_UNSUPPORTED",
            ),
            (
                "L 0 0 1.5 0 0 P 0",
                "L 0 0 0 0 0 P 0",
                "ODB_ZERO_LENGTH_LINE_UNSUPPORTED",
            ),
            (
                "L 0 0 1.5 0 0 P 0",
                "T unsupported",
                "ODB_UNSUPPORTED_CONDUCTIVE_FEATURE",
            ),
        ] {
            let mut f = source(2);
            let data = String::from_utf8(f[p].clone())
                .unwrap()
                .replace(before, after);
            f.insert(p.into(), data.into_bytes());
            assert_eq!(import_unpacked(&f, "pcb").unwrap_err().code, expected);
        }
        let mut f = source(2);
        f.remove(p);
        assert_eq!(
            import_unpacked(&f, "pcb").unwrap_err().code,
            "ODB_MISSING_FILE"
        );
    }
    #[test]
    fn real_unpacked_directory_roundtrip_and_link_rejection() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "boardsi-odb-d05608-{}-{unique}",
            std::process::id()
        ));
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(root.clone());
        for (name, bytes) in source(2) {
            let full = root.join(name);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, bytes).unwrap();
        }
        let board = import_directory(&root, "pcb").unwrap();
        assert_eq!(board.traces.len(), 2);
        assert_eq!(board.pads.len(), 2);
        assert_eq!(board.sources[0].id.files.len(), 3);
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.join("matrix/matrix"), root.join("evil")).unwrap();
            assert_eq!(
                import_directory(&root, "pcb").unwrap_err().code,
                "ODB_SYMLINK_UNSUPPORTED"
            );
        }
    }
    #[test]
    fn unit_overflow_path_and_step_fail_closed() {
        assert_eq!(
            decimal_ticks("1.0000000001", "x", 1).unwrap_err().code,
            "ODB_BAD_EXACT_DECIMAL"
        );
        assert_eq!(
            decimal_ticks("99999999999999999999", "x", 1)
                .unwrap_err()
                .code,
            "ODB_DECIMAL_OVERFLOW"
        );
        assert_eq!(decimal_ticks("-0.25", "x", 1).unwrap(), -250_000_000);
        let mut f = source(2);
        f.insert("../evil".into(), vec![]);
        assert_eq!(
            import_unpacked(&f, "pcb").unwrap_err().code,
            "ODB_UNSAFE_LOGICAL_PATH"
        );
        let f = source(2);
        assert_eq!(
            import_unpacked(&f, "panel").unwrap_err().code,
            "ODB_UNDECLARED_STEP"
        );
    }
}
