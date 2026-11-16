//! F_003_1 local native file worker. One job per process; bounded JSON receipt.
//! Never use this binary for physical connectivity, geometry display or SI claims.
use boardsi_native::{load, recover_owned_staging, save};
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
        let n = input.read(&mut block).map_err(|_| fail("BSI_SOURCE_READ"))?;
        if n == 0 { break; }
        hasher.update(&block[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
fn receipt(action: &str, path: &Path, read_only: bool) -> Result<Value, String> {
    let before = file_sha256(path)?;
    let loaded = load(path).map_err(native_error)?;
    let after = file_sha256(path)?;
    if before != after {
        return Err(fail("BSI_SOURCE_CHANGED"));
    }
    let b = loaded.model();
    Ok(json!({
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
    }))
}
fn run() -> Result<Value, String> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.is_empty() {
        return Err(fail("BSI_FILE_USAGE"));
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
