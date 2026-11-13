//! E_021: a bounded, SOURCE-only ODB++ Canonical -> Qt artwork transport.
//! This is NOT the D_056_04 single-file via protocol or a solver input.
//! Directory + one selected STEP, positive straight Trace and round/rect Pad only.
use boardsi_model::{BoardModel, LayerKind, ObjectId, Orientation, PadRole, PadShape, Point, QuarterTurn, Rational};
use std::{collections::BTreeMap, env, fmt::Write as _, path::Path, process};

fn hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes { write!(&mut result, "{byte:02x}").expect("string write"); }
    result
}
fn id(x: &ObjectId) -> Result<String, &'static str> {
    if x.source_object.len() + x.part.len() > 512
        || x.source_object.contains('\0') || x.part.contains('\0') {
        return Err("E021_OBJECT_ID_INVALID");
    }
    Ok(hex(format!("{}\0{}", x.source_object, x.part).as_bytes()))
}
fn rational(x: Rational) -> String { format!("{}:{}", x.numerator(), x.denominator()) }
fn exact(board: &BoardModel, p: &Point) -> Result<[String; 2], &'static str> {
    let frame = board.frames.iter().find(|f| f.meta.id == p.frame).ok_or("E021_FRAME_MISSING")?;
    let xy = frame.exact_m(p).map_err(|_| "E021_COORDINATE_UNREPRESENTABLE")?;
    Ok([rational(xy[0]), rational(xy[1])])
}
fn render(board: &BoardModel) -> Result<String, &'static str> {
    if board.validate().has_errors() || board.sources.len() != 1 {
        return Err("E021_CANONICAL_INVALID");
    }
    if !board.arcs.is_empty() || !board.areas.is_empty() || !board.vias.is_empty()
        || !board.drills.is_empty() || !board.components.is_empty() || !board.pins.is_empty() {
        return Err("E021_UNQUALIFIED_GEOMETRY");
    }
    if board.traces.len() + board.pads.len() == 0 ||
       board.traces.len() + board.pads.len() > 256 {
        return Err("E021_OBJECT_LIMIT");
    }
    let files = &board.sources[0].id.files;
    if files.is_empty() || files.len() > 64 { return Err("E021_SOURCE_FILE_LIMIT"); }
    let mut layers: Vec<_> = board.stackup.layers.iter().filter(|x| x.kind == LayerKind::Copper).collect();
    layers.sort_by_key(|x| x.order);
    if !(2..=64).contains(&layers.len()) { return Err("E021_LAYER_LIMIT"); }
    let mut layer_map = BTreeMap::new();
    for (i, l) in layers.iter().enumerate() {
        if layer_map.insert(l.meta.id.clone(), i+1).is_some() { return Err("E021_DUPLICATE_LAYER"); }
    }
    let number = |id: &ObjectId| -> Result<usize, &'static str> {
        layer_map.get(id).copied().ok_or("E021_LAYER_NOT_MAPPED")
    };
    let mut result = format!("BOARDSI_ODB_QT_V1|{}|{}|{}|GEOMETRY_PARTIAL|CONNECTIVITY_UNKNOWN|SI_NOT_RUN\n",
        files.len(), layers.len(), board.traces.len()+board.pads.len());
    for file in files {
        if file.logical_path.len() > 1024 || file.logical_path.is_empty()
            || file.logical_path.starts_with('/') || file.logical_path.contains('\\')
            || file.logical_path.split('/').any(|s| s.is_empty() || s=="." || s=="..") {
            return Err("E021_SOURCE_PATH_INVALID");
        }
        writeln!(&mut result, "F|{}|{}", hex(file.logical_path.as_bytes()), hex(&file.sha256)).unwrap();
    }
    for (i,l) in layers.iter().enumerate() {
        writeln!(&mut result, "L|{}|{}|{}", i+1, hex(l.name.as_bytes()), id(&l.meta.id)?).unwrap();
    }
    for t in &board.traces {
        if t.centreline.len() != 2 || !t.width_m.positive() { return Err("E021_TRACE_UNSUPPORTED"); }
        let a = exact(board, &t.centreline[0])?;
        let b = exact(board, &t.centreline[1])?;
        let net = t.net.as_ref().map(id).transpose()?.unwrap_or_else(|| "-".to_owned());
        writeln!(&mut result, "T|{}|{}|{}|{}|{}|{}|{}|{}",
            number(&t.layer)?, id(&t.meta.id)?, a[0],a[1],b[0],b[1],
            rational(t.width_m),net).unwrap();
    }
    for p in &board.pads {
        if !matches!(p.orientation, Orientation::Orthogonal{turn:QuarterTurn::Zero,mirror_x:false}) {
            return Err("E021_PAD_ORIENTATION_UNSUPPORTED");
        }
        let stack = board.padstacks.iter().find(|s| s.meta.id == p.padstack)
            .ok_or("E021_PADSTACK_MISSING")?;
        if stack.entries.len() != 1 || stack.entries[0].role != PadRole::Conductive {
            return Err("E021_PADSTACK_UNSUPPORTED");
        }
        let entry = &stack.entries[0];
        let (kind,a,b) = match &entry.shape {
            PadShape::Circle{diameter_m} if diameter_m.positive() => 
                ("CIRCLE", *diameter_m, Rational::integer(0)),
            PadShape::Rectangle{size_m} if size_m[0].positive() && size_m[1].positive() =>
                ("RECT",size_m[0],size_m[1]),
            _ => return Err("E021_PAD_SHAPE_UNSUPPORTED"),
        };
        let xy = exact(board, &p.position)?;
        let net = p.net.as_ref().map(id).transpose()?.unwrap_or_else(|| "-".to_owned());
        writeln!(&mut result, "P|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            number(&entry.layer)?,id(&p.meta.id)?,xy[0],xy[1],kind,
            rational(a),rational(b),net,id(&stack.meta.id)?).unwrap();
    }
    writeln!(&mut result, "END|{}|SOURCE_ONLY",board.traces.len()+board.pads.len()).unwrap();
    if result.len() > 4*1024*1024 { return Err("E021_IPC_SIZE_LIMIT"); }
    Ok(result)
}
fn main() {
    let args:Vec<String> = env::args().collect();
    if args.len()!=3 || !Path::new(&args[1]).is_dir() {
        eprintln!("E021_ERROR:USAGE_DIRECTORY_AND_STEP"); process::exit(2);
    }
    match boardsi_import::odb_source::import_directory(Path::new(&args[1]), &args[2]) {
        Ok(board) => match render(&board) {
            Ok(wire) => print!("{wire}"),
            Err(code) => { eprintln!("E021_ERROR:{code}"); process::exit(1); }
        },
        Err(e) => { eprintln!("E021_ERROR:{}",e.code); process::exit(1); }
    }
}
