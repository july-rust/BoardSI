//! E_017: one-level ODB++ STEP-REPEAT translation, source-only.
//! Siemens ODB++Design 8.1 Update 4, stephdr pp.139-141.
//! Only ANGLE=0 / FLIP=NO / MIRROR=NO, one rectangular child array.
//! All other transforms fail closed; no physical connectivity is inferred.
use crate::odb_source::{decimal_ticks, OdbError, OdbFileMap};
use boardsi_model::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc as Shared;

const MAX_INSTANCES: usize = 64;
const MAX_EXPANDED_OBJECTS: usize = 1_000_000;
const MAX_PLACEMENT_TICKS: i128 = 1_000_000_000_000;

fn err(path: &str, line: usize, code: &'static str) -> OdbError {
    OdbError {
        path: path.into(),
        line,
        code,
    }
}

#[derive(Debug)]
pub(crate) struct Placement {
    pub child: String,
    pub unit: &'static str,
    pub child_unit: &'static str,
    pub child_datum: [i64; 2],
    pub x: i64,
    pub y: i64,
    pub dx: i64,
    pub dy: i64,
    pub nx: usize,
    pub ny: usize,
    pub header_path: String,
    pub record_line: usize,
}

type HeaderFields = BTreeMap<String, String>;
type ParsedHeader = (HeaderFields, Option<HeaderFields>, usize);

fn fields(text: &str, path: &str, array: bool) -> Result<ParsedHeader, OdbError> {
    let mut general = BTreeMap::new();
    let mut repeat: Option<BTreeMap<String, String>> = None;
    let mut inside = false;
    let mut started = false;
    let mut line_start = 0;
    for (i, raw) in text.lines().enumerate() {
        let n = i + 1;
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "STEP-REPEAT {" {
            if !array || inside || started {
                return Err(err(path, n, "ODB_STEP_NESTED_OR_MULTIPLE_REPEAT"));
            }
            started = true;
            inside = true;
            line_start = n;
            repeat = Some(BTreeMap::new());
            continue;
        }
        if line == "}" {
            if !inside {
                return Err(err(path, n, "ODB_STEP_UNMATCHED_BLOCK"));
            }
            inside = false;
            continue;
        }
        if line.contains("STEP-REPEAT") || line == "{" {
            return Err(err(path, n, "ODB_STEP_BAD_REPEAT_GRAMMAR"));
        }
        let (k, v) = line
            .split_once('=')
            .ok_or_else(|| err(path, n, "ODB_STEP_BAD_FIELD"))?;
        let key = k.trim();
        let val = v.trim();
        if key.is_empty() || key.len() > 64 || val.len() > 128 {
            return Err(err(path, n, "ODB_STEP_BAD_FIELD"));
        }
        let target = if inside {
            repeat.as_mut().expect("inside repeat")
        } else {
            &mut general
        };
        if target.insert(key.into(), val.into()).is_some() {
            return Err(err(path, n, "ODB_STEP_DUPLICATE_FIELD"));
        }
        if inside {
            if !matches!(
                key,
                "NAME" | "X" | "Y" | "DX" | "DY" | "NX" | "NY" | "ANGLE" | "FLIP" | "MIRROR"
            ) {
                return Err(err(path, n, "ODB_STEP_UNSUPPORTED_FIELD"));
            }
        } else if !matches!(
            key,
            "UNITS"
                | "X_DATUM"
                | "Y_DATUM"
                | "X_ORIGIN"
                | "Y_ORIGIN"
                | "ID"
                | "TOP_ACTIVE"
                | "BOTTOM_ACTIVE"
                | "RIGHT_ACTIVE"
                | "LEFT_ACTIVE"
                | "AFFECTING_BOM"
                | "AFFECTING_BOM_CHANGED"
        ) && !key.starts_with("ONLINE_")
        {
            return Err(err(path, n, "ODB_STEP_UNSUPPORTED_FIELD"));
        }
    }
    if inside {
        return Err(err(
            path,
            text.lines().count(),
            "ODB_STEP_UNTERMINATED_REPEAT",
        ));
    }
    Ok((general, repeat, line_start))
}

fn unit(g: &BTreeMap<String, String>, path: &str) -> Result<&'static str, OdbError> {
    match g.get("UNITS").map(String::as_str) {
        Some("MM") => Ok("MM"),
        Some("INCH") => Ok("INCH"),
        _ => Err(err(path, 0, "ODB_STEP_UNSUPPORTED_UNITS")),
    }
}
fn number(
    g: &BTreeMap<String, String>,
    key: &str,
    path: &str,
    line: usize,
) -> Result<i64, OdbError> {
    decimal_ticks(
        g.get(key)
            .ok_or_else(|| err(path, line, "ODB_STEP_MISSING_FIELD"))?,
        path,
        line,
    )
}
fn datum(g: &BTreeMap<String, String>, path: &str) -> Result<[i64; 2], OdbError> {
    let x = number(g, "X_DATUM", path, 0)?;
    let y = number(g, "Y_DATUM", path, 0)?;
    for axis in ["X_ORIGIN", "Y_ORIGIN"] {
        if let Some(value) = g.get(axis) {
            if decimal_ticks(value, path, 0)? != 0 {
                return Err(err(path, 0, "ODB_STEP_NONZERO_ORIGIN_UNSUPPORTED"));
            }
        }
    }
    Ok([x, y])
}
fn count(
    g: &BTreeMap<String, String>,
    key: &str,
    path: &str,
    line: usize,
) -> Result<usize, OdbError> {
    let v = g
        .get(key)
        .ok_or_else(|| err(path, line, "ODB_STEP_MISSING_FIELD"))?;
    let n = v
        .parse::<usize>()
        .map_err(|_| err(path, line, "ODB_STEP_BAD_COUNT"))?;
    if n == 0 || n > MAX_INSTANCES {
        return Err(err(path, line, "ODB_STEP_BAD_COUNT"));
    }
    Ok(n)
}
fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_.+-".contains(&b))
}
pub(crate) fn parse(
    files: &OdbFileMap,
    parent: &str,
    steps: &[String],
    copper: &[String],
) -> Result<Option<Placement>, OdbError> {
    let path = format!("steps/{parent}/stephdr");
    let Some(bytes) = files.get(&path) else {
        return Ok(None);
    };
    let text = std::str::from_utf8(bytes).map_err(|_| err(&path, 0, "ODB_INVALID_TEXT"))?;
    // Non-panel stephdr must preserve the pre-E017 importer behavior.
    if !text.contains("STEP-REPEAT") {
        return Ok(None);
    }
    let (general, rep, start) = fields(text, &path, true)?;
    let rep = rep.ok_or_else(|| err(&path, 0, "ODB_STEP_BAD_REPEAT_GRAMMAR"))?;
    let parent_unit = unit(&general, &path)?;
    let parent_datum = datum(&general, &path)?;
    if parent_datum != [0, 0] {
        return Err(err(&path, 0, "ODB_STEP_PARENT_DATUM_UNSUPPORTED"));
    }
    let child = rep
        .get("NAME")
        .ok_or_else(|| err(&path, start, "ODB_STEP_MISSING_FIELD"))?;
    if !safe_name(child) || child == parent || !steps.iter().any(|s| s == child) {
        return Err(err(&path, start, "ODB_STEP_BAD_CHILD"));
    }
    if rep.get("ANGLE").map(String::as_str) != Some("0")
        || rep.get("FLIP").map(String::as_str) != Some("NO")
        || rep.get("MIRROR").map(String::as_str) != Some("NO")
    {
        return Err(err(&path, start, "ODB_STEP_TRANSFORM_UNSUPPORTED"));
    }
    let nx = count(&rep, "NX", &path, start)?;
    let ny = count(&rep, "NY", &path, start)?;
    let instances = nx
        .checked_mul(ny)
        .ok_or_else(|| err(&path, start, "ODB_STEP_REPEAT_LIMIT"))?;
    if instances > MAX_INSTANCES {
        return Err(err(&path, start, "ODB_STEP_REPEAT_LIMIT"));
    }
    let (x, y, dx, dy) = (
        number(&rep, "X", &path, start)?,
        number(&rep, "Y", &path, start)?,
        number(&rep, "DX", &path, start)?,
        number(&rep, "DY", &path, start)?,
    );
    if (nx > 1 && dx == 0) || (ny > 1 && dy == 0) {
        return Err(err(&path, start, "ODB_STEP_DUPLICATE_PLACEMENT"));
    }
    // Do not silently discard top-level positive artwork or design nets.
    for layer in copper {
        let base = format!("steps/{parent}/layers/{layer}/features");
        if files.contains_key(&base) || files.contains_key(&format!("{base}.Z")) {
            return Err(err(&base, 0, "ODB_STEP_PARENT_ARTWORK_UNSUPPORTED"));
        }
    }
    if files.contains_key(&format!("steps/{parent}/eda/data")) {
        return Err(err(&path, 0, "ODB_STEP_PARENT_EDA_UNSUPPORTED"));
    }
    let child_path = format!("steps/{child}/stephdr");
    let child_bytes = files
        .get(&child_path)
        .ok_or_else(|| err(&child_path, 0, "ODB_STEP_CHILD_HEADER_REQUIRED"))?;
    let child_text =
        std::str::from_utf8(child_bytes).map_err(|_| err(&child_path, 0, "ODB_INVALID_TEXT"))?;
    if child_text.contains("STEP-REPEAT") {
        return Err(err(&child_path, 0, "ODB_STEP_NESTED_REPEAT_UNSUPPORTED"));
    }
    let (child_general, _, _) = fields(child_text, &child_path, false)?;
    let child_unit = unit(&child_general, &child_path)?;
    let child_datum = datum(&child_general, &child_path)?;
    let result = Placement {
        child: child.clone(),
        unit: parent_unit,
        child_unit,
        child_datum,
        x,
        y,
        dx,
        dy,
        nx,
        ny,
        header_path: path,
        record_line: start,
    };
    let mut sites = BTreeSet::new();
    for iy in 0..ny {
        for ix in 0..nx {
            let pos = result.position(ix, iy)?;
            if !sites.insert(pos) {
                return Err(err(
                    &result.header_path,
                    start,
                    "ODB_STEP_DUPLICATE_PLACEMENT",
                ));
            }
        }
    }
    Ok(Some(result))
}
impl Placement {
    fn position(&self, ix: usize, iy: usize) -> Result<[i64; 2], OdbError> {
        let calc = |start: i64, delta: i64, index: usize| -> Result<i64, OdbError> {
            let n = i128::from(start)
                .checked_add(i128::from(delta) * index as i128)
                .ok_or_else(|| {
                    err(
                        &self.header_path,
                        self.record_line,
                        "ODB_STEP_COORD_OVERFLOW",
                    )
                })?;
            if n.abs() > MAX_PLACEMENT_TICKS {
                return Err(err(
                    &self.header_path,
                    self.record_line,
                    "ODB_STEP_COORD_LIMIT",
                ));
            }
            i64::try_from(n).map_err(|_| {
                err(
                    &self.header_path,
                    self.record_line,
                    "ODB_STEP_COORD_OVERFLOW",
                )
            })
        };
        Ok([calc(self.x, self.dx, ix)?, calc(self.y, self.dy, iy)?])
    }
    fn metres(&self, ticks: i64, unit: &str) -> Result<Rational, OdbError> {
        let tick = match unit {
            "MM" => Rational::new(1, 1_000_000_000_000),
            "INCH" => Rational::new(254, 10_000_000_000_000),
            _ => return Err(err(&self.header_path, 0, "ODB_STEP_UNSUPPORTED_UNITS")),
        }
        .map_err(|_| err(&self.header_path, 0, "ODB_STEP_RATIONAL"))?;
        Rational::integer(ticks).checked_mul(tick).map_err(|_| {
            err(
                &self.header_path,
                self.record_line,
                "ODB_STEP_RATIONAL_OVERFLOW",
            )
        })
    }
    fn offsets(&self, ix: usize, iy: usize) -> Result<[Rational; 2], OdbError> {
        let pos = self.position(ix, iy)?;
        let mut out = [Rational::integer(0); 2];
        for i in 0..2 {
            let placed = self.metres(pos[i], self.unit)?;
            let child_datum = self.metres(self.child_datum[i], self.child_unit)?;
            out[i] = placed
                .checked_add(child_datum.checked_neg().map_err(|_| {
                    err(
                        &self.header_path,
                        self.record_line,
                        "ODB_STEP_RATIONAL_OVERFLOW",
                    )
                })?)
                .map_err(|_| {
                    err(
                        &self.header_path,
                        self.record_line,
                        "ODB_STEP_RATIONAL_OVERFLOW",
                    )
                })?;
        }
        Ok(out)
    }
}
fn new_id(
    source: &SourceId,
    old: &ObjectId,
    place: Option<(&Placement, usize, usize)>,
) -> ObjectId {
    let name = match place {
        Some((p, ix, iy)) => format!("step-repeat/{}/{iy}/{ix}/{}", p.child, old.source_object),
        None => old.source_object.clone(),
    };
    ObjectId::new(source, old.kind, &name, &old.part)
}
fn witness(meta: &mut ObjectMeta, p: &Placement, ix: usize, iy: usize) {
    meta.provenance.history.push(ConversionStep {
        operation: "odb-step-repeat-exact-frame-translation".into(),
        quality: ConversionQuality::Exact,
        max_error_m: Some(Rational::integer(0)),
        note: format!(
            "{}:line:{}; child={}; ix={ix}; iy={iy}; no rotation, flip, or mirror; source-only",
            p.header_path, p.record_line, p.child
        ),
    });
}
fn translate_point(point: &mut Point, source: &SourceId, p: &Placement, ix: usize, iy: usize) {
    point.frame = new_id(source, &point.frame, Some((p, ix, iy)));
}
fn translate_boundary(
    boundary: &mut Boundary,
    source: &SourceId,
    p: &Placement,
    ix: usize,
    iy: usize,
) {
    for seg in &mut boundary.segments {
        match seg {
            BoundarySegment::Line { start, end } => {
                translate_point(start, source, p, ix, iy);
                translate_point(end, source, p, ix, iy);
            }
            BoundarySegment::Arc {
                centre, start, end, ..
            } => {
                translate_point(centre, source, p, ix, iy);
                translate_point(start, source, p, ix, iy);
                translate_point(end, source, p, ix, iy);
            }
        }
    }
}
pub(crate) fn instantiate(child: BoardModel, p: &Placement) -> Result<BoardModel, OdbError> {
    // This E-stage slice never promotes a child's EDA association or generated
    // manufacturing/electrical evidence into a repeated parent board.
    if !child.nets.is_empty()
        || !child.components.is_empty()
        || !child.pins.is_empty()
        || !child.vias.is_empty()
        || !child.drills.is_empty()
        || !child.outlines.is_empty()
        || !child.connectivity.nodes.is_empty()
        || !child.connectivity.edges.is_empty()
        || child.traces.iter().any(|v| v.net.is_some())
        || child.arcs.iter().any(|v| v.net.is_some())
        || child.pads.iter().any(|v| v.net.is_some())
        || child.areas.iter().any(|v| v.net.is_some())
    {
        return Err(err(
            &p.header_path,
            0,
            "ODB_STEP_ELECTRICAL_EVIDENCE_UNSUPPORTED",
        ));
    }
    let count = p.nx * p.ny;
    let objects = child.frames.len()
        + child.traces.len()
        + child.arcs.len()
        + child.pads.len()
        + child.padstacks.len()
        + child.areas.len();
    let expanded = objects
        .checked_mul(count)
        .ok_or_else(|| err(&p.header_path, 0, "ODB_STEP_EXPANSION_LIMIT"))?;
    if expanded > MAX_EXPANDED_OBJECTS {
        return Err(err(&p.header_path, 0, "ODB_STEP_EXPANSION_LIMIT"));
    }
    let old_source = child
        .sources
        .first()
        .ok_or_else(|| err(&p.header_path, 0, "ODB_STEP_MISSING_SOURCE"))?;
    let mut identity = old_source.id.as_ref().clone();
    identity.importer_version = "e017-odb-step-repeat-translation-v1".into();
    identity.conditions.retain(|(k, _)| k != "selected_step");
    identity.conditions.push((
        "selected_step".into(),
        p.header_path
            .strip_prefix("steps/")
            .unwrap_or(&p.header_path)
            .trim_end_matches("/stephdr")
            .into(),
    ));
    identity
        .conditions
        .push(("step_repeat_child".into(), p.child.clone()));
    identity.conditions.push((
        "step_repeat_mode".into(),
        "single-level-translation-only".into(),
    ));
    identity.conditions.sort();
    let source: SourceId = Shared::new(identity);
    let mut board = BoardModel::empty(child.kind);
    let mut import = old_source.clone();
    import.id = source.clone();
    import.capability.object_coverage.push(
        "E_017 single child step-repeat translation; 0-degree, no flip/mirror; source-only".into(),
    );
    board.sources.push(import);
    board.stackup = child.stackup.clone();
    for layer in &mut board.stackup.layers {
        layer.meta.id = new_id(&source, &layer.meta.id, None);
    }
    for iy in 0..p.ny {
        for ix in 0..p.nx {
            let offsets = p.offsets(ix, iy)?;
            for original in &child.frames {
                let mut item = original.clone();
                item.meta.id = new_id(&source, &item.meta.id, Some((p, ix, iy)));
                witness(&mut item.meta, p, ix, iy);
                for (axis, offset) in offsets.iter().enumerate() {
                    item.origin_m[axis] =
                        item.origin_m[axis].checked_add(*offset).map_err(|_| {
                            err(&p.header_path, p.record_line, "ODB_STEP_RATIONAL_OVERFLOW")
                        })?;
                }
                board.frames.push(item);
            }
            for original in &child.traces {
                let mut item = original.clone();
                item.meta.id = new_id(&source, &item.meta.id, Some((p, ix, iy)));
                item.layer = new_id(&source, &item.layer, None);
                for point in &mut item.centreline {
                    translate_point(point, &source, p, ix, iy);
                }
                witness(&mut item.meta, p, ix, iy);
                board.traces.push(item);
            }
            for original in &child.arcs {
                let mut item = original.clone();
                item.meta.id = new_id(&source, &item.meta.id, Some((p, ix, iy)));
                item.layer = new_id(&source, &item.layer, None);
                for point in [&mut item.start, &mut item.end, &mut item.centre] {
                    translate_point(point, &source, p, ix, iy);
                }
                witness(&mut item.meta, p, ix, iy);
                board.arcs.push(item);
            }
            for original in &child.padstacks {
                let mut item = original.clone();
                item.meta.id = new_id(&source, &item.meta.id, Some((p, ix, iy)));
                for entry in &mut item.entries {
                    entry.layer = new_id(&source, &entry.layer, None);
                }
                witness(&mut item.meta, p, ix, iy);
                board.padstacks.push(item);
            }
            for original in &child.pads {
                let mut item = original.clone();
                item.meta.id = new_id(&source, &item.meta.id, Some((p, ix, iy)));
                item.padstack = new_id(&source, &item.padstack, Some((p, ix, iy)));
                translate_point(&mut item.position, &source, p, ix, iy);
                witness(&mut item.meta, p, ix, iy);
                board.pads.push(item);
            }
            for original in &child.areas {
                let mut item = original.clone();
                item.meta.id = new_id(&source, &item.meta.id, Some((p, ix, iy)));
                item.layer = new_id(&source, &item.layer, None);
                translate_boundary(&mut item.outer, &source, p, ix, iy);
                for hole in &mut item.cutouts {
                    hole.meta.id = new_id(&source, &hole.meta.id, Some((p, ix, iy)));
                    translate_boundary(&mut hole.boundary, &source, p, ix, iy);
                    witness(&mut hole.meta, p, ix, iy);
                }
                witness(&mut item.meta, p, ix, iy);
                board.areas.push(item);
            }
        }
    }
    if board.validate().has_errors() {
        return Err(err(
            &p.header_path,
            p.record_line,
            "ODB_STEP_CANONICAL_INVALID",
        ));
    }
    Ok(board)
}
