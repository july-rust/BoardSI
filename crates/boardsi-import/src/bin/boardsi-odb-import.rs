//! D_056_08 source-scope diagnostic only, NOT a complete PCB/Qt/SI importer.
use std::{env, path::Path, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if !matches!(args.len(), 3 | 4) || (args.len() == 4 && args[3] != "--eda-nets") {
        eprintln!("Usage: boardsi-odb-import <unpacked-odb-product-root-or-uncompressed.tar> <step> [--eda-nets]");
        process::exit(2);
    }
    let with_eda = args.len() == 4;
    let path = Path::new(&args[1]);
    // A regular archive is decoded in-process. No executable shell, tar, gzip
    // or filesystem extraction is invoked; unsupported archives fail closed.
    let parsed = if path.is_file() {
        boardsi_import::odb_source::import_tar_file(path, &args[2], with_eda)
    } else if with_eda {
        boardsi_import::odb_source::import_directory_with_eda(path, &args[2])
    } else {
        boardsi_import::odb_source::import_directory(path, &args[2])
    };
    match parsed {
        Ok(board) => {
            let source = &board.sources[0];
            println!(
                "status={}",
                if with_eda {
                    "SOURCE_GEOMETRY_AND_DESIGN_NET_PARTIAL"
                } else {
                    "SOURCE_GEOMETRY_PARTIAL"
                }
            );
            println!("format=ODB++Design");
            println!("step={}", args[2]);
            println!("source_files={}", source.id.files.len());
            println!("copper_layers={}", board.stackup.layers.len());
            println!("traces={}", board.traces.len());
            println!("arcs={}", board.arcs.len());
            println!("pads={}", board.pads.len());
            println!("copper_areas={}", board.areas.len());
            println!(
                "copper_cutouts={}",
                board.areas.iter().map(|a| a.cutouts.len()).sum::<usize>()
            );
            println!("vias=0  # UNSUPPORTED");
            println!("nets={}", board.nets.len());
            println!(
                "net_source_status={}",
                if with_eda {
                    "DESIGN_NET_OWNERSHIP_PARTIAL"
                } else {
                    "NOT_IMPORTED"
                }
            );
            println!(
                "owned_traces={}",
                board.traces.iter().filter(|t| t.net.is_some()).count()
            );
            println!(
                "owned_arcs={}",
                board.arcs.iter().filter(|t| t.net.is_some()).count()
            );
            println!(
                "owned_pads={}",
                board.pads.iter().filter(|t| t.net.is_some()).count()
            );
            println!(
                "owned_areas={}",
                board.areas.iter().filter(|t| t.net.is_some()).count()
            );
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
