//! D_056_08 source-scope diagnostic only, NOT a complete PCB/Qt/SI importer.
use std::{env, path::Path, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: boardsi-odb-import <unpacked-odb-product-root> <step>");
        process::exit(2);
    }
    match boardsi_import::odb_source::import_directory(Path::new(&args[1]), &args[2]) {
        Ok(board) => {
            let source = &board.sources[0];
            println!("status=SOURCE_GEOMETRY_PARTIAL");
            println!("format=ODB++Design");
            println!("step={}", args[2]);
            println!("source_files={}", source.id.files.len());
            println!("copper_layers={}", board.stackup.layers.len());
            println!("traces={}", board.traces.len());
            println!("arcs={}", board.arcs.len());
            println!("pads={}", board.pads.len());
            println!("vias=0  # UNSUPPORTED");
            println!("nets=0  # NOT_IMPORTED");
            println!("manufactured_connectivity=NOT_VERIFIED");
            println!("global_open_short=NOT_VERIFIED");
            println!("si_result=NOT_VERIFIED");
        }
        Err(err) => {
            eprintln!(
                "status=UNSUPPORTED code={} relative_source={} line={}",
                err.code, err.path, err.line
            );
            process::exit(1);
        }
    }
}
