//! D_056_04 opt-in Canonical Padstack geometry-only IPC producer.
use std::io::Write;

fn run() -> Result<Vec<u8>, &'static str> {
    let mut args = std::env::args_os();
    let _exe = args.next();
    let path = args.next().ok_or("USAGE: boardsi-import-canonical-padstack FILE.asc")?;
    if args.next().is_some() { return Err("USAGE: boardsi-import-canonical-padstack FILE.asc"); }
    let meta = std::fs::metadata(&path).map_err(|_| "D05604_FILE_UNAVAILABLE")?;
    if !meta.is_file() || meta.len() > boardsi_import::route_preview::MAX_FILE_BYTES as u64 {
        return Err("D05604_FILE_LIMIT");
    }
    let bytes = std::fs::read(&path).map_err(|_| "D05604_FILE_READ")?;
    boardsi_import::canonical_padstack_ipc::encode_pads_source(&bytes)
}
fn main() {
    match run() {
        Ok(data) => {
            if std::io::stdout().write_all(&data).is_err() {
                eprintln!("D05604_STDOUT_FAILED");
                std::process::exit(1);
            }
        }
        Err(code) => { eprintln!("{code}"); std::process::exit(1); }
    }
}
