//! F_005: shared exact E_021 ODB++ Qt subset preflight and wire projection.
//! Native save and Qt importer MUST use the same acceptance gate.
//! Does not authorize manufactured connectivity or any SI analysis.
use boardsi_model::{
    BoardModel, LayerKind, ObjectId, Orientation, PadRole, PadShape, Point, QuarterTurn, Rational,
};
use std::{collections::BTreeMap, fmt::Write as _};

fn hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut result, "{byte:02x}").expect("string write");
    }
    result
}
fn id(x: &ObjectId) -> Result<String, &'static str> {
    if x.source_object.len() + x.part.len() > 512
        || x.source_object.contains('\0')
        || x.part.contains('\0')
    {
        return Err("E021_OBJECT_ID_INVALID");
    }
    Ok(hex(format!("{}\0{}", x.source_object, x.part).as_bytes()))
}
fn rational(x: Rational) -> String {
    format!("{}:{}", x.numerator(), x.denominator())
}
fn qt_dimension_limit(
    value: Rational,
    max_abs_metres: i128,
    require_positive: bool,
) -> Result<(), &'static str> {
    // E_021 Qt decoder's double dimension gate is ±1,000,000 mm for
    // all points and 0 < shape/width <= 1,000 mm. Compare exact rational
    // metres without rounding to the Qt preview's f64 drawing coordinates.
    let max = value
        .denominator()
        .checked_mul(max_abs_metres)
        .ok_or("E021_QT_DIMENSION_LIMIT")?;
    if value.numerator() < -max
        || value.numerator() > max
        || (require_positive && value.numerator() <= 0)
    {
        return Err("E021_QT_DIMENSION_LIMIT");
    }
    Ok(())
}
fn exact_components(board: &BoardModel, p: &Point) -> Result<[Rational; 2], &'static str> {
    let frame = board
        .frames
        .iter()
        .find(|f| f.meta.id == p.frame)
        .ok_or("E021_FRAME_MISSING")?;
    let xy = frame
        .exact_m(p)
        .map_err(|_| "E021_COORDINATE_UNREPRESENTABLE")?;
    qt_dimension_limit(xy[0], 1000, false)?;
    qt_dimension_limit(xy[1], 1000, false)?;
    Ok(xy)
}
fn exact(board: &BoardModel, p: &Point) -> Result<[String; 2], &'static str> {
    let xy = exact_components(board, p)?;
    Ok([rational(xy[0]), rational(xy[1])])
}
fn qt_mm(value: Rational) -> f64 {
    (value.numerator() as f64 / value.denominator() as f64) * 1000.0
}
fn qt_noncollapsed_span(start: f64, extent: f64) -> bool {
    start.is_finite() && extent.is_finite() && extent > 0.0 && (start + extent) > start
}
fn qt_pad_projectable(
    board: &BoardModel,
    p: &Point,
    kind: &str,
    size_a: Rational,
    size_b: Rational,
) -> Result<(), &'static str> {
    // Qt E_021 constructs QRectF(x-a/2,-y-b/2,a,b) or an ellipse with
    // radius a/2. Their final bounding box must have nonzero double width
    // and height. This is a display feasibility gate, not coordinate rounding
    // or a replacement for lossless Canonical geometry.
    let xy = exact_components(board, p)?;
    let cx = qt_mm(xy[0]);
    let cy = -qt_mm(xy[1]);
    let a = qt_mm(size_a);
    let b = if kind == "CIRCLE" { a } else { qt_mm(size_b) };
    let left = cx - a / 2.0;
    let top = cy - b / 2.0;
    let x_visible = qt_noncollapsed_span(left, a);
    let y_visible = qt_noncollapsed_span(top, b);
    if !x_visible || !y_visible {
        return Err("E021_QT_PROJECTION_COLLAPSED");
    }
    Ok(())
}
fn qt_trace_projectable(
    board: &BoardModel,
    start: &Point,
    end: &Point,
    width_m: Rational,
) -> Result<(), &'static str> {
    let a = exact_components(board, start)?;
    let b = exact_components(board, end)?;
    let x1 = qt_mm(a[0]);
    let y1 = -qt_mm(a[1]);
    let x2 = qt_mm(b[0]);
    let y2 = -qt_mm(b[1]);
    let w = qt_mm(width_m);
    if (x1 == x2 && y1 == y2)
        || !qt_noncollapsed_span(x1.min(x2) - w / 2.0, x1.max(x2) - x1.min(x2) + w)
        || !qt_noncollapsed_span(y1.min(y2) - w / 2.0, y1.max(y2) - y1.min(y2) + w)
    {
        return Err("E021_QT_PROJECTION_COLLAPSED");
    }
    Ok(())
}
pub fn render(board: &BoardModel) -> Result<String, &'static str> {
    if board.validate().has_errors() || board.sources.len() != 1 {
        return Err("E021_CANONICAL_INVALID");
    }
    if !board.arcs.is_empty()
        || !board.areas.is_empty()
        || !board.vias.is_empty()
        || !board.drills.is_empty()
        || !board.components.is_empty()
        || !board.pins.is_empty()
    {
        return Err("E021_UNQUALIFIED_GEOMETRY");
    }
    if board.traces.len() + board.pads.len() == 0 || board.traces.len() + board.pads.len() > 256 {
        return Err("E021_OBJECT_LIMIT");
    }
    let files = &board.sources[0].id.files;
    if files.is_empty() || files.len() > 64 {
        return Err("E021_SOURCE_FILE_LIMIT");
    }
    let mut layers: Vec<_> = board
        .stackup
        .layers
        .iter()
        .filter(|x| x.kind == LayerKind::Copper)
        .collect();
    layers.sort_by_key(|x| x.order);
    if !(2..=64).contains(&layers.len()) {
        return Err("E021_LAYER_LIMIT");
    }
    let mut layer_map = BTreeMap::new();
    for (i, l) in layers.iter().enumerate() {
        if layer_map.insert(l.meta.id.clone(), i + 1).is_some() {
            return Err("E021_DUPLICATE_LAYER");
        }
    }
    let number = |id: &ObjectId| -> Result<usize, &'static str> {
        layer_map.get(id).copied().ok_or("E021_LAYER_NOT_MAPPED")
    };
    let mut result = format!(
        "BOARDSI_ODB_QT_V1|{}|{}|{}|GEOMETRY_PARTIAL|CONNECTIVITY_UNKNOWN|SI_NOT_RUN\n",
        files.len(),
        layers.len(),
        board.traces.len() + board.pads.len()
    );
    for file in files {
        if file.logical_path.len() > 1024
            || file.logical_path.is_empty()
            || file.logical_path.starts_with('/')
            || file.logical_path.contains('\\')
            || file
                .logical_path
                .split('/')
                .any(|s| s.is_empty() || s == "." || s == "..")
        {
            return Err("E021_SOURCE_PATH_INVALID");
        }
        writeln!(
            &mut result,
            "F|{}|{}",
            hex(file.logical_path.as_bytes()),
            hex(&file.sha256)
        )
        .unwrap();
    }
    for (i, l) in layers.iter().enumerate() {
        writeln!(
            &mut result,
            "L|{}|{}|{}",
            i + 1,
            hex(l.name.as_bytes()),
            id(&l.meta.id)?
        )
        .unwrap();
    }
    for t in &board.traces {
        if t.centreline.len() != 2 || !t.width_m.positive() {
            return Err("E021_TRACE_UNSUPPORTED");
        }
        qt_dimension_limit(t.width_m, 1, true)?;
        qt_trace_projectable(board, &t.centreline[0], &t.centreline[1], t.width_m)?;
        let a = exact(board, &t.centreline[0])?;
        let b = exact(board, &t.centreline[1])?;
        let net = t
            .net
            .as_ref()
            .map(id)
            .transpose()?
            .unwrap_or_else(|| "-".to_owned());
        writeln!(
            &mut result,
            "T|{}|{}|{}|{}|{}|{}|{}|{}",
            number(&t.layer)?,
            id(&t.meta.id)?,
            a[0],
            a[1],
            b[0],
            b[1],
            rational(t.width_m),
            net
        )
        .unwrap();
    }
    for p in &board.pads {
        if !matches!(
            p.orientation,
            Orientation::Orthogonal {
                turn: QuarterTurn::Zero,
                mirror_x: false
            }
        ) {
            return Err("E021_PAD_ORIENTATION_UNSUPPORTED");
        }
        let stack = board
            .padstacks
            .iter()
            .find(|s| s.meta.id == p.padstack)
            .ok_or("E021_PADSTACK_MISSING")?;
        if stack.entries.len() != 1 || stack.entries[0].role != PadRole::Conductive {
            return Err("E021_PADSTACK_UNSUPPORTED");
        }
        let entry = &stack.entries[0];
        let (kind, a, b) = match &entry.shape {
            PadShape::Circle { diameter_m } if diameter_m.positive() => {
                ("CIRCLE", *diameter_m, Rational::integer(0))
            }
            PadShape::Rectangle { size_m } if size_m[0].positive() && size_m[1].positive() => {
                ("RECT", size_m[0], size_m[1])
            }
            _ => return Err("E021_PAD_SHAPE_UNSUPPORTED"),
        };
        qt_dimension_limit(a, 1, true)?;
        qt_dimension_limit(b, 1, kind == "RECT")?;
        qt_pad_projectable(board, &p.position, kind, a, b)?;
        let xy = exact(board, &p.position)?;
        let net = p
            .net
            .as_ref()
            .map(id)
            .transpose()?
            .unwrap_or_else(|| "-".to_owned());
        writeln!(
            &mut result,
            "P|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            number(&entry.layer)?,
            id(&p.meta.id)?,
            xy[0],
            xy[1],
            kind,
            rational(a),
            rational(b),
            net,
            id(&stack.meta.id)?
        )
        .unwrap();
    }
    writeln!(
        &mut result,
        "END|{}|SOURCE_ONLY",
        board.traces.len() + board.pads.len()
    )
    .unwrap();
    if result.len() > 4 * 1024 * 1024 {
        return Err("E021_IPC_SIZE_LIMIT");
    }
    Ok(result)
}
