//! F_003_1 local native file worker. One job per process; bounded JSON receipt.
//! Never use this binary for physical connectivity, geometry display or SI claims.
mod f009_native_artwork;
mod f011_artwork_stream;

use boardsi_native::{
    canonical_semantics_equal, load, migrate_to_current, recover_owned_staging, save, save_guarded,
    save_with_drill_evidence_guarded, DrillObservation, GerberDrillEvidence, NativeError,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::ExitCode,
};

fn fail(code: &str) -> String {
    code.to_string()
}
fn arg_path(args: &[std::ffi::OsString], i: usize) -> Result<PathBuf, String> {
    args.get(i)
        .map(PathBuf::from)
        .ok_or_else(|| fail("BSI_FILE_USAGE"))
}
fn arg_text(args: &[std::ffi::OsString], i: usize) -> Result<String, String> {
    args.get(i)
        .and_then(|a| a.to_str())
        .map(str::to_owned)
        .ok_or_else(|| fail("BSI_FILE_USAGE"))
}
fn parent(path: &Path) -> &Path {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}
fn native_error(err: boardsi_native::NativeError) -> String {
    format!("{}: {err}", err.code())
}
/// Bind an accepted Qt document to exact on-disk bytes, not merely a path.
/// Run both before and after SQLite load to detect a concurrent replacement.
fn file_sha256(path: &Path) -> Result<String, String> {
    let meta = fs::metadata(path).map_err(|_| fail("BSI_SOURCE_MISSING"))?;
    if !meta.is_file() || meta.len() > 512 * 1024 * 1024 {
        return Err(fail("BSI_SOURCE_SIZE"));
    }
    let mut input = fs::File::open(path).map_err(|_| fail("BSI_SOURCE_READ"))?;
    let mut hasher = Sha256::new();
    let mut block = [0u8; 65536];
    loop {
        let n = input
            .read(&mut block)
            .map_err(|_| fail("BSI_SOURCE_READ"))?;
        if n == 0 {
            break;
        }
        hasher.update(&block[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
/// Optional F_006 read-only preview for a strictly qualified KiCad source.
/// No original CAD code, graphics tessellation or electrical connection is
/// published. Positions are exact source nm; Qt alone handles drawing.
fn source_preview(board: &boardsi_model::BoardModel) -> Result<Option<Value>, String> {
    use boardsi_model::{
        LayerKind, Orientation, PadRole, PadShape, QuarterTurn, Rational, SourceFormat,
    };
    if board.sources.len() != 1 {
        return Ok(None);
    }
    let is_kicad = matches!(&board.sources[0].id.format, SourceFormat::KiCad)
        && board.sources[0].id.conditions.iter().any(|(k, v)| {
            k == "scope"
                && v == "f006-kicad9-straight-segment-normal-through-via-nominal-source-only"
        });
    let is_ipc = matches!(&board.sources[0].id.format, SourceFormat::Ipc2581)
        && board.sources[0].id.conditions.iter().any(|(k, v)| {
            k == "scope" && v == "f007-ipc2581c-inline-round-line-polyline-nm-source-only"
        });
    if !is_kicad && !is_ipc {
        return Ok(None);
    }
    let max_items = f011_artwork_stream::item_limit();
    let referenced_drills = board
        .vias
        .iter()
        .map(|via| &via.drill)
        .collect::<std::collections::BTreeSet<_>>();
    if board.traces.len() + board.vias.len() > max_items
        || board.frames.len() != 1
        || !board.arcs.is_empty()
        || !board.areas.is_empty()
        || !board.outlines.is_empty()
        || !board.pads.is_empty()
        || !board.components.is_empty()
        || !board.pins.is_empty()
        || (is_ipc && !board.vias.is_empty())
        || board
            .drills
            .iter()
            .any(|d| !referenced_drills.contains(&d.meta.id))
    {
        return Err(fail("BSI_KICAD_PREVIEW_UNSUPPORTED"));
    }
    let frame = &board.frames[0];
    if frame.metres_per_tick
        != Rational::new(1, 1_000_000_000).map_err(|_| fail("BSI_KICAD_PREVIEW_UNIT"))?
        || frame.origin_m != [Rational::integer(0); 2]
        || !matches!(
            &frame.orientation,
            Orientation::Orthogonal {
                turn: QuarterTurn::Zero,
                mirror_x: false
            }
        )
    {
        return Err(fail("BSI_KICAD_PREVIEW_FRAME"));
    }
    let to_nm = |x: Rational| -> Result<i64, String> {
        let scaled = x
            .numerator()
            .checked_mul(1_000_000_000)
            .ok_or_else(|| fail("BSI_KICAD_PREVIEW_DIMENSION"))?;
        if scaled % x.denominator() != 0 {
            return Err(fail("BSI_KICAD_PREVIEW_DIMENSION"));
        }
        i64::try_from(scaled / x.denominator()).map_err(|_| fail("BSI_KICAD_PREVIEW_DIMENSION"))
    };
    let mut layers = board
        .stackup
        .layers
        .iter()
        .filter(|l| l.kind == LayerKind::Copper)
        .map(|l| (l.meta.id.clone(), l.name.clone(), l.order))
        .collect::<Vec<_>>();
    layers.sort_by_key(|l| l.2);
    let find_name = |id: &boardsi_model::ObjectId| -> Result<&str, String> {
        layers
            .iter()
            .find(|(layer, _, _)| layer == id)
            .map(|(_, name, _)| name.as_str())
            .ok_or_else(|| fail("BSI_KICAD_PREVIEW_LAYER"))
    };
    let mut rows = Vec::new();
    for tr in &board.traces {
        if tr.centreline.len() < 2 || tr.centreline.iter().any(|p| p.frame != frame.meta.id) {
            return Err(fail("BSI_SOURCE_PREVIEW_TRACE"));
        }
        let width = to_nm(tr.width_m)?;
        if width <= 0 || width > 1_000_000_000 {
            return Err(fail("BSI_SOURCE_PREVIEW_WIDTH"));
        }
        for (index, pair) in tr.centreline.windows(2).enumerate() {
            if rows.len() >= max_items {
                return Err(fail("BSI_SOURCE_PREVIEW_ITEM_LIMIT"));
            }
            rows.push(json!({
                "kind":"trace","id":format!("{}#{}",tr.meta.id.source_object,index),
                "layer":find_name(&tr.layer)?,
                "a":pair[0].ticks,"b":pair[1].ticks,
                "width_nm":width,
                "net":tr.net.as_ref().map(|n|n.source_object.as_str())
            }));
        }
    }
    let padstacks = board
        .padstacks
        .iter()
        .map(|p| (&p.meta.id, p))
        .collect::<std::collections::BTreeMap<_, _>>();
    let drills = board
        .drills
        .iter()
        .map(|d| (&d.meta.id, d))
        .collect::<std::collections::BTreeMap<_, _>>();
    for via in &board.vias {
        if via.position.frame != frame.meta.id || !via.connected_layers.is_empty() {
            return Err(fail("BSI_KICAD_PREVIEW_VIA"));
        }
        let stack = padstacks
            .get(&via.padstack)
            .ok_or_else(|| fail("BSI_KICAD_PREVIEW_VIA"))?;
        let first = stack
            .entries
            .first()
            .ok_or_else(|| fail("BSI_KICAD_PREVIEW_VIA"))?;
        let diameter = match &first.shape {
            PadShape::Circle { diameter_m } => to_nm(*diameter_m)?,
            _ => return Err(fail("BSI_KICAD_PREVIEW_SHAPE")),
        };
        if stack.entries.iter().any(|e|e.role!=PadRole::Conductive
            || !matches!(&e.shape,PadShape::Circle{diameter_m} if to_nm(*diameter_m).ok()==Some(diameter))) {
            return Err(fail("BSI_KICAD_PREVIEW_SHAPE"));
        }
        let drill = drills
            .get(&via.drill)
            .ok_or_else(|| fail("BSI_KICAD_PREVIEW_DRILL"))?;
        let drill_nm = to_nm(drill.diameter_m)?;
        if drill_nm <= 0 || drill_nm >= diameter || diameter > 1_000_000_000 {
            return Err(fail("BSI_KICAD_PREVIEW_DRILL"));
        }
        if rows.len() >= max_items {
            return Err(fail("BSI_SOURCE_PREVIEW_ITEM_LIMIT"));
        }
        let pad_layers = stack
            .entries
            .iter()
            .map(|entry| find_name(&entry.layer))
            .collect::<Result<Vec<_>, _>>()?;
        rows.push(json!({
            "kind":"via","id":via.meta.id.source_object,"layer":"*",
            "at":via.position.ticks,"diameter_nm":diameter,"drill_nm":drill_nm,
            "span_first":find_name(&via.span.first_layer)?,
            "span_last":find_name(&via.span.last_layer)?,
            "pad_layers":pad_layers,
            "net":via.net.as_ref().map(|n|n.source_object.as_str()),
            "contact":"NOT_VERIFIED"
        }));
    }
    Ok(Some(json!({
        "schema":if is_ipc {"BSI_IPC2581_SOURCE_PREVIEW_V1"} else {"BSI_KICAD_SOURCE_PREVIEW_V1"},
        "units":"integer-nm","solver_ready":false,
        "layers":layers.iter().map(|(_,name,order)|json!({"name":name,"order":order})).collect::<Vec<_>>(),
        "items":rows
    })))
}

// F_008: SOURCE-only flashes plus independent Excellon observations. The
// preview is derived from the just-read native .bsi, never from unsaved input.
fn gerber_source_preview(
    board: &boardsi_model::BoardModel,
    evidence: &GerberDrillEvidence,
) -> Result<Value, String> {
    use boardsi_model::{PadShape, Rational};
    if board.stackup.layers.len() != 1
        || board.pads.len() + evidence.holes.len() > f011_artwork_stream::item_limit()
    {
        return Err(fail("BSI_F008_PREVIEW_LIMIT"));
    }
    let layer = &board.stackup.layers[0];
    let to_nm = |r: Rational| -> Result<i64, String> {
        let n = r
            .numerator()
            .checked_mul(1_000_000_000)
            .ok_or_else(|| fail("BSI_F008_PREVIEW_DIMENSION"))?;
        if n % r.denominator() != 0 {
            return Err(fail("BSI_F008_PREVIEW_DIMENSION"));
        }
        i64::try_from(n / r.denominator()).map_err(|_| fail("BSI_F008_PREVIEW_DIMENSION"))
    };
    let mut items = Vec::new();
    for pad in &board.pads {
        if pad.net.is_some()
            || !matches!(
                &pad.orientation,
                boardsi_model::Orientation::Orthogonal {
                    turn: boardsi_model::QuarterTurn::Zero,
                    mirror_x: false
                }
            )
        {
            return Err(fail("BSI_F008_PREVIEW_PAD"));
        }
        let stack = board
            .padstacks
            .iter()
            .find(|s| s.meta.id == pad.padstack)
            .ok_or_else(|| fail("BSI_F008_PREVIEW_PADSTACK"))?;
        if stack.entries.len() != 1 || stack.entries[0].layer != layer.meta.id {
            return Err(fail("BSI_F008_PREVIEW_PADSTACK"));
        }
        let diameter = match &stack.entries[0].shape {
            PadShape::Circle { diameter_m } => to_nm(*diameter_m)?,
            _ => return Err(fail("BSI_F008_PREVIEW_SHAPE")),
        };
        items.push(json!({
            "kind":"flash","id":pad.meta.id.source_object,
            "layer":layer.name,"at":pad.position.ticks,"diameter_nm":diameter,
            "net":"UNKNOWN","contact":"NOT_VERIFIED"
        }));
    }
    for (index, h) in evidence.holes.iter().enumerate() {
        items.push(json!({
            "kind":"drill-source","id":format!("drill-{}-{}",h.tool_id,index),
            "layer":"*","at":h.centre_nm,"diameter_nm":h.diameter_nm,
            "tool_id":h.tool_id,"source_offset":h.source_offset,
            "plating":"UNKNOWN","layer_span":"UNKNOWN","alignment":"NOT_VERIFIED"
        }));
    }
    Ok(json!({
        "schema":"BSI_GERBER_DRILL_SOURCE_PREVIEW_V1","units":"integer-nm",
        "solver_ready":false,"layers":[{"name":layer.name,"order":layer.order}],
        "items":items,"alignment_verified":false
    }))
}

// F_010: apply the same bounds as the actual Qt renderer to every derived
// preview. These bounds never restrict the Native Reader or persistence.
fn preview_bounds(preview: Value) -> Result<Value, String> {
    let invalid = || fail("BSI_F010_PREVIEW_UNREPRESENTABLE");
    let items = preview["items"].as_array().ok_or_else(invalid)?;
    let layers = preview["layers"].as_array().ok_or_else(invalid)?;
    let gerber = preview["schema"] == "BSI_GERBER_DRILL_SOURCE_PREVIEW_V1";
    if items.is_empty()
        || items.len() > f011_artwork_stream::item_limit()
        || layers.len() < if gerber { 1 } else { 2 }
        || layers.len() > 32
    {
        return Err(invalid());
    }
    let mut names = std::collections::BTreeSet::new();
    for layer in layers {
        let name = layer["name"].as_str().ok_or_else(invalid)?;
        if name.is_empty() || name == "*" || name == "All copper layers" || !names.insert(name) {
            return Err(invalid());
        }
    }
    let point = |p: &Value| {
        p.as_array().is_some_and(|xy| {
            xy.len() == 2
                && xy.iter().all(|v| {
                    v.as_i64()
                        .is_some_and(|n| (-1_000_000_000_000..=1_000_000_000_000).contains(&n))
                })
        })
    };
    let size = |v: &Value| v.as_i64().is_some_and(|n| (1..=1_000_000_000).contains(&n));
    for row in items {
        let layer = row["layer"].as_str().ok_or_else(invalid)?;
        if row["id"].as_str().map_or(true, str::is_empty)
            || (layer != "*" && !names.contains(layer))
        {
            return Err(invalid());
        }
        let valid = match row["kind"].as_str() {
            Some("trace") => {
                layer != "*"
                    && point(&row["a"])
                    && point(&row["b"])
                    && row["a"] != row["b"]
                    && size(&row["width_nm"])
            }
            Some("rectangle") => {
                layer != "*"
                    && point(&row["at"])
                    && size(&row["width_nm"])
                    && size(&row["height_nm"])
            }
            Some("flash") => layer != "*" && point(&row["at"]) && size(&row["diameter_nm"]),
            Some("via") => {
                let mut pads = std::collections::BTreeSet::new();
                layer == "*"
                    && point(&row["at"])
                    && size(&row["diameter_nm"])
                    && size(&row["drill_nm"])
                    && row["drill_nm"].as_i64() < row["diameter_nm"].as_i64()
                    && ["span_first", "span_last"]
                        .iter()
                        .all(|key| row[*key].as_str().is_some_and(|n| names.contains(n)))
                    && row["pad_layers"].as_array().is_some_and(|ls| {
                        !ls.is_empty()
                            && ls.iter().all(|v| {
                                v.as_str()
                                    .is_some_and(|n| names.contains(n) && pads.insert(n))
                            })
                    })
            }
            Some("drill-source") => {
                gerber
                    && layer == "*"
                    && point(&row["at"])
                    && size(&row["diameter_nm"])
                    && row["plating"] == "UNKNOWN"
                    && row["layer_span"] == "UNKNOWN"
                    && row["alignment"] == "NOT_VERIFIED"
                    && row["tool_id"]
                        .as_u64()
                        .is_some_and(|n| (1..=u64::from(u32::MAX)).contains(&n))
            }
            _ => false,
        };
        if !valid {
            return Err(invalid());
        }
    }
    Ok(preview)
}

fn set_artwork(response: &mut Value, preview: Result<Value, String>) {
    match preview.and_then(preview_bounds) {
        Ok(artwork) => {
            response["artwork_status"] = json!("RENDERABLE");
            response["artwork"] = artwork;
        }
        Err(reason) => {
            response["artwork_status"] = json!("NOT_RENDERED");
            response["artwork_reason"] = json!(reason);
        }
    }
}

fn receipt(action: &str, path: &Path, read_only: bool) -> Result<Value, String> {
    let before = file_sha256(path)?;
    let loaded = load(path).map_err(native_error)?;
    let after = file_sha256(path)?;
    if before != after {
        return Err(fail("BSI_SOURCE_CHANGED"));
    }
    let b = loaded.model();
    let mut response = json!({
        "protocol":"BSI_NATIVE_FILE_V1",
        "action": action,
        "path": path.to_string_lossy(),
        "sha256":before,
        "read_only": read_only || loaded.is_read_only(),
        "traces": b.traces.len(), "arcs": b.arcs.len(), "areas": b.areas.len(),
        "padstacks": b.padstacks.len(), "pads": b.pads.len(),
        "vias": b.vias.len(), "drills": b.drills.len(),
        "layers": b.stackup.layers.len(), "nets": b.nets.len(),
        "sources": b.sources.len(),
        "geometry_only": true, "physical_si": "NOT_RUN"
    });
    if let Some(evidence) = loaded.drill_evidence() {
        response["source_format"] = json!("GERBER_X2_EXCELLON_SOURCE_SUBSET");
        response["drill_observations"] = json!(evidence.holes.len());
        set_artwork(&mut response, gerber_source_preview(b, evidence));
    } else if b.sources.len() == 1 {
        use boardsi_model::SourceFormat;
        let format = match b.sources[0].id.format {
            SourceFormat::PadsAsc => "PADS_V94_SOURCE_SUBSET",
            SourceFormat::Odb => "ODB_SOURCE_SUBSET",
            SourceFormat::KiCad => "KICAD9_SOURCE_SUBSET",
            SourceFormat::Ipc2581 => "IPC2581C_SOURCE_SUBSET",
            _ => return Ok(response),
        };
        response["source_format"] = json!(format);
        let preview = match b.sources[0].id.format {
            SourceFormat::PadsAsc | SourceFormat::Odb => f009_native_artwork::render(b)
                .map_err(str::to_owned)
                .and_then(|(qualified_format, artwork)| {
                    if qualified_format == format {
                        Ok(artwork)
                    } else {
                        Err(fail("BSI_F009_SOURCE_FORMAT_MISMATCH"))
                    }
                }),
            _ => source_preview(b)
                .and_then(|artwork| artwork.ok_or_else(|| fail("BSI_F010_PREVIEW_SCOPE"))),
        };
        set_artwork(&mut response, preview);
    }
    Ok(response)
}
fn run() -> Result<Value, String> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.is_empty() {
        return Err(fail("BSI_FILE_USAGE"));
    }
    #[cfg(feature = "f0031-qualification")]
    if args.len() == 2
        && arg_text(&args, 0)? == "open"
        && env::var("BOARDSI_F0031_QUAL_MODE").ok().as_deref() == Some("pause-before-open")
    {
        let marker = env::var_os("BOARDSI_F0031_QUAL_MARKER")
            .ok_or_else(|| fail("BSI_QUAL_MARKER_MISSING"))?;
        fs::write(marker, b"real-native-file-worker-paused")
            .map_err(|_| fail("BSI_QUAL_MARKER_WRITE"))?;
        loop {
            std::thread::park();
        }
    }
    // CI-only deterministic Native fixture, not a CAD parser expansion or
    // vendor-source claim. The source adapter supplies a small valid template;
    // synthetic copies deliberately exercise the persistence/display path.
    #[cfg(feature = "f0031-qualification")]
    if (args.len() == 3 || args.len() == 4) && arg_text(&args, 0)? == "f011-seed" {
        let path = arg_path(&args, 1)?;
        let count: usize = arg_text(&args, 2)?
            .parse()
            .map_err(|_| fail("BSI_F011_FIXTURE_COUNT"))?;
        if !(1..=100_000).contains(&count) {
            return Err(fail("BSI_F011_FIXTURE_COUNT"));
        }
        let mixed = if args.len() == 4 {
            if arg_text(&args, 3)? != "mixed" {
                return Err(fail("BSI_F011_FIXTURE_KIND"));
            }
            true
        } else {
            false
        };
        let template = br#"(kicad_pcb (version 20241229) (generator "F011 synthetic fixture")
          (layers (0 "F.Cu" signal) (2 "B.Cu" signal))
          (net 0 "") (net 1 "CLK")
          (segment (start 0 0) (end 0.25 0) (width 0.1)
           (layer "F.Cu") (net 1)
           (uuid "223e4567-e89b-12d3-a456-426614174000"))
          (via (at 0 1) (size 0.6) (drill 0.3) (layers "F.Cu" "B.Cu") (net 1)
           (uuid "123e4567-e89b-12d3-a456-426614174000")))"#;
        let mut board = boardsi_import::kicad9_source::into_native_canonical(template)
            .map_err(|e| format!("BSI_F011_FIXTURE_TEMPLATE:{}", e.code))?;
        let trace = board
            .traces
            .pop()
            .ok_or_else(|| fail("BSI_F011_FIXTURE_TEMPLATE"))?;
        let back = board
            .stackup
            .layers
            .iter()
            .find(|l| l.name == "B.Cu")
            .ok_or_else(|| fail("BSI_F011_FIXTURE_LAYER"))?
            .meta
            .id
            .clone();
        let via = board
            .vias
            .pop()
            .ok_or_else(|| fail("BSI_F011_FIXTURE_TEMPLATE"))?;
        let drill = board
            .drills
            .pop()
            .ok_or_else(|| fail("BSI_F011_FIXTURE_TEMPLATE"))?;
        if !mixed {
            board.padstacks.clear();
        }
        for i in 0..count {
            let x = (i % 100) as i64 * 1_000_000;
            let y = (i / 100) as i64 * 1_000_000;
            if mixed && i % 2 == 1 {
                let mut item = via.clone();
                let mut hole = drill.clone();
                item.meta.id.source_object = format!("f011-synthetic-{i}");
                item.meta.provenance.source_object_type = "F011 synthetic test via".into();
                hole.meta.id.source_object = format!("f011-synthetic-drill-{i}");
                item.drill = hole.meta.id.clone();
                item.position.ticks = [x, y];
                board.drills.push(hole);
                board.vias.push(item);
                continue;
            }
            let mut item = trace.clone();
            item.meta.id.source_object = format!("f011-synthetic-{i}");
            item.meta.provenance.source_object_type = "F011 synthetic test trace".into();
            if i % 2 == 1 {
                item.layer = back.clone();
            }
            item.centreline[0].ticks = [x, y];
            item.centreline[1].ticks = [x + 250_000, y];
            board.traces.push(item);
        }
        save(&path, &board).map_err(native_error)?;
        return receipt("f011-seed", &path, false);
    }
    match arg_text(&args, 0)?.as_str() {
        "open" if args.len() == 2 => {
            let path = arg_path(&args, 1)?;
            recover_owned_staging(parent(&path)).map_err(|e| format!("BSI_RECOVERY_IO: {e}"))?;
            receipt("open", &path, false)
        }
        "save-pads" if args.len() == 4 => {
            let source = arg_path(&args, 1)?;
            let target = arg_path(&args, 2)?;
            let expected = arg_text(&args, 3)?;
            if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(fail("BSI_SOURCE_SHA_INVALID"));
            }
            let metadata = fs::metadata(&source).map_err(|_| fail("BSI_SOURCE_MISSING"))?;
            if !metadata.is_file()
                || metadata.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64
            {
                return Err(fail("BSI_SOURCE_SIZE"));
            }
            let bytes = fs::read(&source).map_err(|_| fail("BSI_SOURCE_READ"))?;
            let actual = format!("{:x}", Sha256::digest(&bytes));
            if actual != expected.to_ascii_lowercase() {
                return Err(fail("BSI_SOURCE_CHANGED"));
            }
            // Apply the SAME strict fabrication and source-coverage gate as
            // the Qt Canonical importer; route parsing alone could silently
            // discard unrelated PARTDECAL and manufacturing declarations.
            boardsi_import::canonical_padstack_ipc::encode_pads_source(&bytes)
                .map_err(|_| fail("BSI_SOURCE_NOT_ACCEPTED_QT_CANONICAL"))?;
            let parsed = boardsi_import::route_preview::parse_generalized(&bytes)
                .map_err(|_| fail("BSI_SOURCE_UNSUPPORTED"))?;
            if parsed.skipped != 0 {
                return Err(fail("BSI_SOURCE_INCOMPLETE"));
            }
            let board = boardsi_import::route_preview::into_canonical(&bytes, &parsed)
                .map_err(|_| fail("BSI_SOURCE_CANONICAL"))?;
            save(&target, &board).map_err(native_error)?;
            receipt("save-pads", &target, false)
        }
        "save-kicad" if args.len() == 4 => {
            let source = arg_path(&args, 1)?;
            let target = arg_path(&args, 2)?;
            let expected = arg_text(&args, 3)?;
            if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(fail("BSI_SOURCE_SHA_INVALID"));
            }
            let expected = expected.to_ascii_lowercase();
            let meta = fs::metadata(&source).map_err(|_| fail("BSI_SOURCE_MISSING"))?;
            if !meta.is_file() || meta.len() > 8 * 1024 * 1024 {
                return Err(fail("BSI_SOURCE_SIZE"));
            }
            let bytes = fs::read(&source).map_err(|_| fail("BSI_SOURCE_READ"))?;
            if format!("{:x}", Sha256::digest(&bytes)) != expected {
                return Err(fail("BSI_SOURCE_CHANGED"));
            }
            let board = boardsi_import::kicad9_source::into_native_canonical(&bytes)
                .map_err(|e| format!("BSI_KICAD_SOURCE_UNSUPPORTED:{}", e.code))?;
            // Only a fully revalidated source may replace an existing .bsi.
            // The same strict importer is applied to the guard path; a source
            // rewrite that keeps path, size or mtime is still detected.
            save_guarded(&target, &board, || {
                let fresh_bytes = fs::read(&source).map_err(|_| NativeError::SourceChanged)?;
                if format!("{:x}", Sha256::digest(&fresh_bytes)) != expected {
                    return Err(NativeError::SourceChanged);
                }
                let fresh = boardsi_import::kicad9_source::into_native_canonical(&fresh_bytes)
                    .map_err(|_| NativeError::SourceChanged)?;
                if !canonical_semantics_equal(&board, &fresh) {
                    return Err(NativeError::SourceChanged);
                }
                Ok(())
            })
            .map_err(native_error)?;
            let mut response = receipt("save-kicad", &target, false)?;
            response["source_sha256"] = json!(expected);
            response["source_format"] = json!("KICAD9_SOURCE_SUBSET");
            Ok(response)
        }
        "save-ipc2581" if args.len() == 4 => {
            let source = arg_path(&args, 1)?;
            let target = arg_path(&args, 2)?;
            let expected = arg_text(&args, 3)?;
            if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(fail("BSI_SOURCE_SHA_INVALID"));
            }
            let expected = expected.to_ascii_lowercase();
            let meta = fs::metadata(&source).map_err(|_| fail("BSI_SOURCE_MISSING"))?;
            if !meta.is_file() || meta.len() > 8 * 1024 * 1024 {
                return Err(fail("BSI_SOURCE_SIZE"));
            }
            let bytes = fs::read(&source).map_err(|_| fail("BSI_SOURCE_READ"))?;
            if format!("{:x}", Sha256::digest(&bytes)) != expected {
                return Err(fail("BSI_SOURCE_CHANGED"));
            }
            let board = boardsi_import::ipc2581c_source::into_native_canonical(&bytes)
                .map_err(|e| format!("BSI_IPC2581_SOURCE_UNSUPPORTED:{}", e.code))?;
            // Display limits do not gate a structurally valid Native save.
            save_guarded(&target, &board, || {
                let fresh_bytes = fs::read(&source).map_err(|_| NativeError::SourceChanged)?;
                if format!("{:x}", Sha256::digest(&fresh_bytes)) != expected {
                    return Err(NativeError::SourceChanged);
                }
                let fresh = boardsi_import::ipc2581c_source::into_native_canonical(&fresh_bytes)
                    .map_err(|_| NativeError::SourceChanged)?;
                if !canonical_semantics_equal(&board, &fresh) {
                    return Err(NativeError::SourceChanged);
                }
                Ok(())
            })
            .map_err(native_error)?;
            let mut response = receipt("save-ipc2581", &target, false)?;
            response["source_sha256"] = json!(expected);
            response["source_format"] = json!("IPC2581C_SOURCE_SUBSET");
            Ok(response)
        }
        "save-gerber-drill" if args.len() == 6 => {
            let gerber = arg_path(&args, 1)?;
            let drill = arg_path(&args, 2)?;
            let target = arg_path(&args, 3)?;
            let gerber_sha = arg_text(&args, 4)?;
            let drill_sha = arg_text(&args, 5)?;
            let valid_sha = |s: &str| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit());
            if !valid_sha(&gerber_sha) || !valid_sha(&drill_sha) {
                return Err(fail("BSI_SOURCE_SHA_INVALID"));
            }
            let gerber_sha = gerber_sha.to_ascii_lowercase();
            let drill_sha = drill_sha.to_ascii_lowercase();
            let read_source = |path: &Path| -> Result<Vec<u8>, String> {
                let meta = fs::metadata(path).map_err(|_| fail("BSI_SOURCE_MISSING"))?;
                if !meta.is_file() || meta.len() > 8 * 1024 * 1024 {
                    return Err(fail("BSI_SOURCE_SIZE"));
                }
                fs::read(path).map_err(|_| fail("BSI_SOURCE_READ"))
            };
            let gerber_bytes = read_source(&gerber)?;
            let drill_bytes = read_source(&drill)?;
            let sha = |bytes: &[u8]| format!("{:x}", Sha256::digest(bytes));
            if sha(&gerber_bytes) != gerber_sha || sha(&drill_bytes) != drill_sha {
                return Err(fail("BSI_SOURCE_CHANGED"));
            }
            let pair =
                boardsi_import::gerber_excellon_source::inspect_pair(&gerber_bytes, &drill_bytes)
                    .map_err(|e| format!("BSI_F008_SOURCE_UNSUPPORTED:{}", e.code))?;
            let board =
                boardsi_import::gerber_excellon_source::into_native_canonical_from_pair(&pair)
                    .map_err(|e| format!("BSI_F008_CANONICAL_UNSUPPORTED:{}", e.code))?;
            let evidence = GerberDrillEvidence {
                drill_sha256: pair.drill.identity.files[0].sha256,
                holes: pair
                    .drill
                    .holes
                    .iter()
                    .map(|h| DrillObservation {
                        centre_nm: h.centre_nm,
                        diameter_nm: h.diameter_nm,
                        tool_id: h.tool_id,
                        source_offset: h.source_offset as u64,
                    })
                    .collect(),
            };
            save_with_drill_evidence_guarded(&target, &board, Some(&evidence), || {
                let fresh_g = read_source(&gerber).map_err(|_| NativeError::SourceChanged)?;
                let fresh_d = read_source(&drill).map_err(|_| NativeError::SourceChanged)?;
                if sha(&fresh_g) != gerber_sha || sha(&fresh_d) != drill_sha {
                    return Err(NativeError::SourceChanged);
                }
                let fresh_pair =
                    boardsi_import::gerber_excellon_source::inspect_pair(&fresh_g, &fresh_d)
                        .map_err(|_| NativeError::SourceChanged)?;
                let fresh =
                    boardsi_import::gerber_excellon_source::into_native_canonical_from_pair(
                        &fresh_pair,
                    )
                    .map_err(|_| NativeError::SourceChanged)?;
                let fresh_evidence = GerberDrillEvidence {
                    drill_sha256: fresh_pair.drill.identity.files[0].sha256,
                    holes: fresh_pair
                        .drill
                        .holes
                        .iter()
                        .map(|h| DrillObservation {
                            centre_nm: h.centre_nm,
                            diameter_nm: h.diameter_nm,
                            tool_id: h.tool_id,
                            source_offset: h.source_offset as u64,
                        })
                        .collect(),
                };
                if !canonical_semantics_equal(&board, &fresh) || evidence != fresh_evidence {
                    return Err(NativeError::SourceChanged);
                }
                Ok(())
            })
            .map_err(native_error)?;
            let mut result = receipt("save-gerber-drill", &target, false)?;
            result["source_sha256"] = json!(gerber_sha);
            result["drill_sha256"] = json!(drill_sha);
            Ok(result)
        }
        "save-odb" if args.len() == 5 => {
            let root = arg_path(&args, 1)?;
            let step = arg_text(&args, 2)?;
            let target = arg_path(&args, 3)?;
            let expected = arg_text(&args, 4)?;
            if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(fail("BSI_SOURCE_SHA_INVALID"));
            }
            let expected = expected.to_ascii_lowercase();
            // Never create a second ODB parser or quietly skip unsupported copper.
            // Identical preflight to the real E_021 Qt importer; this also
            // validates all Source files, their logical paths and exact hashes.
            let board = boardsi_import::odb_source::import_directory(&root, &step)
                .map_err(|e| format!("BSI_ODB_SOURCE_UNSUPPORTED:{}", e.code))?;
            boardsi_import::odb_qt_wire::render(&board)
                .map_err(|e| format!("BSI_ODB_QT_SUBSET_UNSUPPORTED:{e}"))?;
            let manifest = |board: &boardsi_model::BoardModel| -> String {
                let mut hash = Sha256::new();
                for source in &board.sources[0].id.files {
                    for b in source.logical_path.as_bytes() {
                        hash.update(format!("{b:02x}").as_bytes());
                    }
                    hash.update(b"|");
                    for b in &source.sha256 {
                        hash.update(format!("{b:02x}").as_bytes());
                    }
                    hash.update(b"\n");
                }
                format!("{:x}", hash.finalize())
            };
            let actual = manifest(&board);
            if actual != expected {
                return Err(fail("BSI_SOURCE_CHANGED"));
            }
            // Re-open the complete SOURCE directory *after* staging has been
            // committed, synced and independently verified. A changed source
            // aborts before the original destination is replaced.
            save_guarded(&target, &board, || {
                let fresh = boardsi_import::odb_source::import_directory(&root, &step)
                    .map_err(|_| NativeError::SourceChanged)?;
                boardsi_import::odb_qt_wire::render(&fresh)
                    .map_err(|_| NativeError::SourceChanged)?;
                if manifest(&fresh) != expected || !canonical_semantics_equal(&board, &fresh) {
                    return Err(NativeError::SourceChanged);
                }
                Ok(())
            })
            .map_err(native_error)?;
            let mut response = receipt("save-odb", &target, false)?;
            response["source_manifest_sha256"] = json!(actual);
            response["source_format"] = json!("ODB_SOURCE_SUBSET");
            Ok(response)
        }
        "save-loaded" if args.len() == 4 => {
            let source = arg_path(&args, 1)?;
            let target = arg_path(&args, 2)?;
            let expected = arg_text(&args, 3)?;
            if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(fail("BSI_SOURCE_SHA_INVALID"));
            }
            let expected = expected.to_ascii_lowercase();
            if file_sha256(&source)? != expected {
                return Err(fail("BSI_SOURCE_CHANGED"));
            }
            let loaded = load(&source).map_err(native_error)?;
            // Never accept a different valid board replacing the opened file.
            if file_sha256(&source)? != expected {
                return Err(fail("BSI_SOURCE_CHANGED"));
            }
            if loaded.is_read_only() {
                return Err(fail("BSI_UNSUPPORTED_REQUIRED"));
            }
            loaded.save_as(&target).map_err(native_error)?;
            receipt("save-loaded", &target, false)
        }
        "migrate-v1" if args.len() == 3 => {
            let source = arg_path(&args, 1)?;
            let target = arg_path(&args, 2)?;
            let result =
                migrate_to_current(source.as_path(), target.as_path()).map_err(native_error)?;
            let mut proof = receipt("migrate-v1", &target, false)?;
            let fields = proof.as_object_mut().expect("JSON object");
            fields.insert("from_schema".into(), json!(result.from_schema));
            fields.insert("to_schema".into(), json!(result.to_schema));
            fields.insert("steps".into(), json!(result.steps));
            fields.insert("copied".into(), json!(result.copied));
            Ok(proof)
        }
        "recover" if args.len() == 2 => {
            let dir = arg_path(&args, 1)?;
            let cleaned =
                recover_owned_staging(&dir).map_err(|e| format!("BSI_RECOVERY_IO: {e}"))?;
            Ok(json!({"protocol":"BSI_NATIVE_FILE_V1","action":"recover",
                "cleaned":cleaned,"physical_si":"NOT_RUN"}))
        }
        _ => Err(fail("BSI_FILE_USAGE")),
    }
}
/// Check the *entire* serialized worker receipt at the final publish boundary,
/// after action-specific fields were added. Qt bounds stdout at 65,536 bytes.
/// Native structural load must still succeed when derived preview is too big.
fn bound_artwork_receipt(mut value: Value) -> Value {
    let is_f009 = matches!(
        value["source_format"].as_str(),
        Some("PADS_V94_SOURCE_SUBSET" | "ODB_SOURCE_SUBSET")
    ) && value["artwork_status"] == "RENDERABLE";
    if value["artwork_status"] == "RENDERABLE" && value.to_string().len() >= 60_000 {
        if let Some(map) = value.as_object_mut() {
            map.remove("artwork");
        }
        value["artwork_status"] = json!("NOT_RENDERED");
        value["artwork_reason"] = json!(if is_f009 {
            "BSI_F009_PREVIEW_OUTPUT_LIMIT"
        } else {
            "BSI_F010_PREVIEW_OUTPUT_LIMIT"
        });
    }
    value
}

fn main() -> ExitCode {
    let request = match f011_artwork_stream::request() {
        Ok(request) => request,
        Err(code) => {
            eprintln!("{code}");
            return ExitCode::FAILURE;
        }
    };
    match run() {
        Ok(value) => {
            if let Some(token) = request {
                if let Err(code) =
                    f011_artwork_stream::write_receipt(value, &token, std::io::stdout().lock())
                {
                    eprintln!("{code}");
                    return ExitCode::FAILURE;
                }
            } else {
                println!("{}", bound_artwork_receipt(value));
            }
            ExitCode::SUCCESS
        }
        Err(code) => {
            eprintln!("{code}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod f009_receipt_tests {
    use super::*;
    #[test]
    fn bounded_worker_output_preserves_valid_structural_open() {
        let oversized = json!({
            "protocol": "BSI_NATIVE_FILE_V1", "action": "open",
            "source_format": "ODB_SOURCE_SUBSET", "artwork_status": "RENDERABLE",
            "geometry_only": true, "physical_si": "NOT_RUN",
            "artwork": {"items": "x".repeat(70_000)}
        });
        let bounded = bound_artwork_receipt(oversized);
        assert_eq!(bounded["action"], "open");
        assert_eq!(bounded["geometry_only"], true);
        assert_eq!(bounded["artwork_status"], "NOT_RENDERED");
        assert_eq!(bounded["artwork_reason"], "BSI_F009_PREVIEW_OUTPUT_LIMIT");
        assert!(bounded.get("artwork").is_none());
        assert!(bounded.to_string().len() < 60_000);
    }

    #[test]
    fn all_f010_formats_share_full_receipt_limit() {
        for format in [
            "PADS_V94_SOURCE_SUBSET",
            "ODB_SOURCE_SUBSET",
            "KICAD9_SOURCE_SUBSET",
            "IPC2581C_SOURCE_SUBSET",
            "GERBER_X2_EXCELLON_SOURCE_SUBSET",
        ] {
            let receipt = json!({
                "action":"save-loaded","source_format":format,
                "artwork_status":"RENDERABLE","artwork":{"items":"x".repeat(70_000)},
                "sha256":"digest","geometry_only":true
            });
            let bounded = bound_artwork_receipt(receipt);
            assert_eq!(bounded["artwork_status"], "NOT_RENDERED");
            assert_eq!(bounded["sha256"], "digest");
            assert!(bounded.get("artwork").is_none());
        }
    }

    #[test]
    fn preview_limits_do_not_publish_partial_artwork() {
        let preview = json!({
            "schema":"BSI_KICAD_SOURCE_PREVIEW_V1",
            "layers":[{"name":"F.Cu"},{"name":"B.Cu"}],
            "items":[{"kind":"trace","id":"a","layer":"F.Cu",
                "a":[0,0],"b":[1,1],"width_nm":100}]
        });
        assert!(preview_bounds(preview.clone()).is_ok());
        for b in [json!([0, 0]), json!([1_000_000_000_001_i64, 0])] {
            let mut invalid = preview.clone();
            invalid["items"][0]["b"] = b;
            let mut receipt = json!({"geometry_only":true});
            set_artwork(&mut receipt, Ok(invalid));
            assert_eq!(receipt["artwork_status"], "NOT_RENDERED");
            assert!(receipt.get("artwork").is_none());
        }
    }
}
