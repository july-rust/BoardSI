//! D_056_08 source-scope diagnostic only, NOT a complete PCB/Qt/SI importer.
use std::{env, path::Path, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    let options = if args.len() >= 3 { &args[3..] } else { &[] };
    if args.len() < 3
        || options.len() > 2
        || options
            .iter()
            .any(|o| !matches!(o.as_str(), "--eda-nets" | "--source-holes"))
        || options.iter().filter(|s| *s == "--eda-nets").count() > 1
        || options.iter().filter(|s| *s == "--source-holes").count() > 1
    {
        eprintln!("Usage: boardsi-odb-import <unpacked-product-root-or-USTAR-or-ZIP0-8-or-GZIP-tar> <step> [--eda-nets] [--source-holes]");
        process::exit(2);
    }
    let with_eda = options.iter().any(|s| s == "--eda-nets");
    let with_holes = options.iter().any(|s| s == "--source-holes");
    let path = Path::new(&args[1]);
    // One strictly bounded source snapshot; the optional hole inventory is
    // fabrication-intent only and NEVER manufactures a conductive via.
    let parsed = if with_holes {
        boardsi_import::odb_fabrication::import_path_with_holes(path, &args[2], with_eda)
            .map(|report| (report.board, Some(report.source_holes.len())))
    } else if path.is_file() {
        boardsi_import::odb_source::import_archive_file(path, &args[2], with_eda)
            .map(|board| (board, None))
    } else if with_eda {
        boardsi_import::odb_source::import_directory_with_eda(path, &args[2])
            .map(|board| (board, None))
    } else {
        boardsi_import::odb_source::import_directory(path, &args[2]).map(|board| (board, None))
    };
    match parsed {
        Ok((board, source_holes)) => {
            let source = &board.sources[0];
            println!(
                "status={}",
                match (with_eda, with_holes) {
                    (true, true) => "SOURCE_GEOMETRY_NET_AND_DRILL_EVIDENCE_PARTIAL",
                    (false, true) => "SOURCE_GEOMETRY_AND_DRILL_EVIDENCE_PARTIAL",
                    (true, false) => "SOURCE_GEOMETRY_AND_DESIGN_NET_PARTIAL",
                    (false, false) => "SOURCE_GEOMETRY_PARTIAL",
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
            if let Some(hole_count) = source_holes {
                println!("fabrication_source_holes={hole_count}");
                println!("fabrication_hole_conductive_edges=0");
                println!("fab_process_qualification=NOT_VERIFIED");
            }
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
