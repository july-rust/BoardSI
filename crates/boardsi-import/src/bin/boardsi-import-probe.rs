//! Offline, read-only M1 detection smoke. Does not accept a PCB for analysis.
use boardsi_import::{probe_pads_reader, Signature};
use std::{env, fs::File, process::ExitCode};

fn main() -> ExitCode {
    let mut args = env::args_os();
    let _program = args.next();
    let Some(path) = args.next() else {
        eprintln!("usage: boardsi-import-probe FILE");
        return ExitCode::from(2);
    };
    if args.next().is_some() {
        eprintln!("expected exactly one source file");
        return ExitCode::from(2);
    }
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
