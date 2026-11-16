//! E_021/F_005: Qt transport uses the same strict subset validation as native ODB saving.
use std::{env, path::Path, process};
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 || !Path::new(&args[1]).is_dir() {
        eprintln!("E021_ERROR:USAGE_DIRECTORY_AND_STEP");
        process::exit(2);
    }
    match boardsi_import::odb_source::import_directory(Path::new(&args[1]), &args[2]) {
        Ok(board) => match boardsi_import::odb_qt_wire::render(&board) {
            Ok(wire) => print!("{wire}"),
            Err(code) => {
                eprintln!("E021_ERROR:{code}");
                process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("E021_ERROR:{}", e.code);
            process::exit(1);
        }
    }
}
