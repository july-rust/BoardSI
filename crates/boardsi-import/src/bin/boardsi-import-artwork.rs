//! Offline local PADS ASC (V9.4 MILS only) -> partial Canonical Model -> Qt line protocol.
//! Standard output is a bounded transport, never an SI simulation or full CAD import.
use boardsi_import::route_preview::{into_canonical, parse, MAX_FILE_BYTES};
use std::{env,fs,io::{self, Write}};
const NM: i64 = 25_400;
fn run() -> Result<(), &'static str> {
    let args: Vec<_> = env::args_os().collect();
    if args.len() != 2 { return Err("USAGE"); }
    let path = &args[1];
    let meta = fs::metadata(path).map_err(|_| "FILE_UNAVAILABLE")?;
    if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 { return Err("INVALID_FILE_SIZE"); }
    let bytes = fs::read(path).map_err(|_| "FILE_READ_FAILED")?;
    let result = parse(&bytes)?;
    let board = into_canonical(&bytes, &result)?;
    if board.traces.len() != result.segments.len() || board.validate().has_errors() {
        return Err("CANONICAL_VALIDATION_FAILED");
    }
    let mut out = Vec::new();
    writeln!(out, "BOARDSI_P2A_PARTIAL_V1|V9.4-MILS|{}", result.max_layer).map_err(|_| "FORMAT_ERROR")?;
    for layer in &board.stackup.layers {
        writeln!(out, "L|{}|{}", layer.order + 1, layer.name).map_err(|_| "FORMAT_ERROR")?;
    }
    for segment in &result.segments {
        let a = [segment.a[0].checked_mul(NM).ok_or("COORD_OVERFLOW")?,
                 segment.a[1].checked_mul(NM).ok_or("COORD_OVERFLOW")?];
        let b = [segment.b[0].checked_mul(NM).ok_or("COORD_OVERFLOW")?,
                 segment.b[1].checked_mul(NM).ok_or("COORD_OVERFLOW")?];
        writeln!(out, "T|{}|{}|{}|{}|{}|{}|{}",
                 segment.layer, segment.net, a[0], a[1], b[0], b[1],
                 segment.width_ticks.checked_mul(NM).ok_or("WIDTH_OVERFLOW")?)
            .map_err(|_| "FORMAT_ERROR")?;
    }
    writeln!(out, "END|{}|{}|CANONICAL_PARTIAL|SI_NOT_RUN",
             result.segments.len(), result.skipped).map_err(|_| "FORMAT_ERROR")?;
    if out.len() > 2 * 1024 * 1024 { return Err("OUTPUT_LIMIT"); }
    io::stdout().write_all(&out).map_err(|_| "STDOUT_FAILED")?;
    Ok(())
}
fn main() {
    if let Err(err) = run() {
        eprintln!("P2A_IMPORT_ERROR:{err}");
        std::process::exit(3);
    }
}