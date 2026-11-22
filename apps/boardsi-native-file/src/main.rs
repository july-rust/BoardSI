//! F_003_1 local native file worker. One job per process; bounded JSON receipt.
//! Never use this binary for physical connectivity, geometry display or SI claims.
mod f009_native_artwork;

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
    if board.traces.len() + board.vias.len() > 128 || board.frames.len() != 1 {
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
    let layers = board
        .stackup
        .layers
        .iter()
        .filter(|l| l.kind == LayerKind::Copper)
        .map(|l| (l.meta.id.clone(), l.name.clone(), l.order))
        .collect::<Vec<_>>();
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
            if rows.len() >= 128 {
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
    for via in &board.vias {
        if via.position.frame != frame.meta.id || !via.connected_layers.is_empty() {
            return Err(fail("BSI_KICAD_PREVIEW_VIA"));
        }
        let stack = board
            .padstacks
            .iter()
            .find(|p| p.meta.id == via.padstack)
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
        let drill = board
            .drills
            .iter()
            .find(|d| d.meta.id == via.drill)
            .ok_or_else(|| fail("BSI_KICAD_PREVIEW_DRILL"))?;
        let drill_nm = to_nm(drill.diameter_m)?;
        if drill_nm <= 0 || drill_nm >= diameter || diameter > 1_000_000_000 {
            return Err(fail("BSI_KICAD_PREVIEW_DRILL"));
        }
        rows.push(json!({
            "kind":"via","id":via.meta.id.source_object,"layer":"*",
            "at":via.position.ticks,"diameter_nm":diameter,"drill_nm":drill_nm,
            "span_first":find_name(&via.span.first_layer)?,
            "span_last":find_name(&via.span.last_layer)?,
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
    if board.stackup.layers.len() != 1 || board.pads.len() + evidence.holes.len() > 128 {
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
        let preview = gerber_source_preview(b, evidence)?;
        response["source_format"] = json!("GERBER_X2_EXCELLON_SOURCE_SUBSET");
        response["artwork"] = preview;
        response["drill_observations"] = json!(evidence.holes.len());
    } else if b.sources.len() == 1
        && matches!(
            b.sources[0].id.format,
            boardsi_model::SourceFormat::PadsAsc | boardsi_model::SourceFormat::Odb
        )
    {
        let source_format = match b.sources[0].id.format {
            boardsi_model::SourceFormat::PadsAsc => "PADS_V94_SOURCE_SUBSET",
            _ => "ODB_SOURCE_SUBSET",
        };
        response["source_format"] = json!(source_format);
        match f009_native_artwork::render(b) {
            Ok((qualified_format, preview)) if qualified_format == source_format => {
                response["artwork"] = preview;
                response["artwork_status"] = json!("RENDERABLE");
            }
            Ok(_) => {
                response["artwork_status"] = json!("NOT_RENDERED");
                response["artwork_reason"] = json!("BSI_F009_SOURCE_FORMAT_MISMATCH");
            }
            Err(reason) => {
                // A valid .bsi remains structurally loadable even when the
                // bounded Qt representation cannot render every CAD primitive.
                response["artwork_status"] = json!("NOT_RENDERED");
                response["artwork_reason"] = json!(reason);
            }
        }
    } else if let Some(preview) = source_preview(b)? {
        let source_format = if preview["schema"] == "BSI_IPC2581_SOURCE_PREVIEW_V1" {
            "IPC2581C_SOURCE_SUBSET"
        } else {
            "KICAD9_SOURCE_SUBSET"
        };
        response["source_format"] = json!(source_format);
        response["artwork"] = preview;
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
            // Qualify the exact Qt preview *before* publishing a new .bsi.
            // A bounded-source file that cannot be represented in the 64 KiB
            // QProcess receipt must never produce a successful file write
            // followed by a misleading worker failure.
            let preview =
                source_preview(&board)?.ok_or_else(|| fail("BSI_IPC2581_PREVIEW_UNAVAILABLE"))?;
            let preview_bytes =
                serde_json::to_vec(&preview).map_err(|_| fail("BSI_IPC2581_PREVIEW_ENCODING"))?;
            if preview_bytes.len() > 48 * 1024 {
                return Err(fail("BSI_IPC2581_PREVIEW_SIZE"));
            }
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
            let preview = gerber_source_preview(&board, &evidence)?;
            if serde_json::to_vec(&preview)
                .map_err(|_| fail("BSI_F008_PREVIEW_ENCODING"))?
                .len()
                > 48 * 1024
            {
                return Err(fail("BSI_F008_PREVIEW_SIZE"));
            }
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
fn main() -> ExitCode {
    match run() {
        Ok(value) => {
            println!("{}", value);
            ExitCode::SUCCESS
        }
        Err(code) => {
            eprintln!("{code}");
            ExitCode::FAILURE
        }
    }
}
