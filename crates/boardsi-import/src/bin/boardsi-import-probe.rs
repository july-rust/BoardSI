//! Offline, read-only M1 detection smoke. Does not accept a PCB for SI analysis.
//! The legacy one-argument probe output and exit codes remain unchanged.
use boardsi_import::{probe_pads_reader, Signature};
use boardsi_import::import_readiness::inspect_pads_readiness;
use boardsi_import::route_preview::MAX_FILE_BYTES;
use std::{env, fs::File, io::Read, process::ExitCode};

fn main() -> ExitCode {
    let mut args = env::args_os();
    let _program = args.next();
    let Some(path) = args.next() else {
        eprintln!("usage: boardsi-import-probe FILE [--readiness]");
        return ExitCode::from(2);
    };
    let readiness = match args.next() {
        None => false,
        Some(flag) if flag == "--readiness" && args.next().is_none() => true,
        _ => {
            eprintln!("expected FILE or FILE --readiness");
            return ExitCode::from(2);
        }
    };

    if readiness {
        let mut file = match File::open(&path) {
            Ok(f) => f,
            Err(_) => {
                eprintln!("cannot read source for readiness");
                return ExitCode::from(2);
            }
        };
        let meta = match file.metadata() {
            Ok(m) => m,
            Err(_) => {
                eprintln!("cannot stat source for readiness");
                return ExitCode::from(2);
            }
        };
        if !meta.is_file() || meta.len() > MAX_FILE_BYTES as u64 {
            println!("format=UNKNOWN;parser_status=REJECTED;reason=INVALID_FILE_SIZE;preview_subset_verified=false;solver_ready=false;physical_si=NOT_RUN");
            return ExitCode::from(3);
        }
        // Bound actual bytes even if the file changes size after metadata().
        let mut source = Vec::new();
        if file.take((MAX_FILE_BYTES + 1) as u64).read_to_end(&mut source).is_err() {
            eprintln!("cannot read bounded source for readiness");
            return ExitCode::from(2);
        }
        let result = inspect_pads_readiness(&source);
        println!("{}", result.diagnostic());
        return if result.preview_subset_verified {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(3)
        };
    }

    // Original signature-only output deliberately stays byte-for-byte stable.
    let detected = File::open(path).and_then(probe_pads_reader);
    match detected {
        Ok(Signature::PadsPcbAscii) => {
            println!("PADS_PCB_ASCII_SIGNATURE_ONLY; geometry=NOT_PARSED; si=NOT_RUN");
            ExitCode::SUCCESS
        }
        Ok(Signature::PadsSchematicAscii) => {
            println!("PADS_SCHEMATIC_ASCII; pcb_import=UNSUPPORTED");
            ExitCode::from(3)
        }
        Ok(Signature::Unknown) => {
            println!("UNKNOWN_OR_UNSUPPORTED; pcb_import=UNSUPPORTED");
            ExitCode::from(3)
        }
        Err(err) => {
            eprintln!("cannot read source header: {err}");
            ExitCode::from(2)
        }
    }
}
