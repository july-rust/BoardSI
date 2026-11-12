//! D_056_08 strict SOURCE-only drill tool and MATRIX span audit.
//! Optional --hits enumerates source P hit/tool references, NOT physical barrels or SI.
use std::{env, path::Path, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if !(args.len() == 3 || (args.len() == 4 && args[3] == "--hits")) {
        eprintln!("Usage: boardsi-odb-drill-audit <unpacked-odb-product-root-or-USTAR-or-ZIP0-8-or-GZIP-tar> <step> [--hits]");
        process::exit(2);
    }
    let with_hits = args.len() == 4;
    match boardsi_import::odb_source::audit_path_drill_source(
        Path::new(&args[1]),
        &args[2],
        with_hits,
    ) {
        Ok(report) => {
            let layers = report.layers;
            println!("status=SOURCE_DRILL_TOOL_INTENT_ONLY");
            println!("drill_layers={}", layers.len());
            println!(
                "tools={}",
                layers.iter().map(|l| l.tools.len()).sum::<usize>()
            );
            for (index, layer) in layers.iter().enumerate() {
                println!("layer_{index}.name={}", layer.name);
                println!("layer_{index}.type={}", layer.subtype);
                println!("layer_{index}.units={}", layer.unit);
                println!(
                    "layer_{index}.span_start={}",
                    layer
                        .explicit_start_layer
                        .as_deref()
                        .unwrap_or("NOT_VERIFIED")
                );
                println!(
                    "layer_{index}.span_end={}",
                    layer
                        .explicit_end_layer
                        .as_deref()
                        .unwrap_or("NOT_VERIFIED")
                );
                println!("layer_{index}.tool_count={}", layer.tools.len());
                println!(
                    "layer_{index}.matrix_source_line={}",
                    layer.matrix_source_line
                );
                let mut matrix_hex = String::with_capacity(64);
                for byte in &layer.matrix_sha256 {
                    use std::fmt::Write;
                    write!(&mut matrix_hex, "{byte:02x}").expect("writing to String cannot fail");
                }
                println!("layer_{index}.matrix_sha256={matrix_hex}");
                println!("layer_{index}.source_file={}", layer.source_file);
                let mut hex = String::with_capacity(64);
                for byte in &layer.source_sha256 {
                    use std::fmt::Write;
                    write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
                }
                println!("layer_{index}.source_sha256={hex}");
            }
            if let Some(hits) = report.hits {
                println!("status=SOURCE_DRILL_TOOL_AND_HIT_EVIDENCE");
                println!("source_hit_count={}", hits.len());
                for (index, hit) in hits.iter().take(20).enumerate() {
                    println!(
                        "hit_{index}=layer:{};dcode:{};xy:{},{};units:{};source_line:{}",
                        hit.layer_name, hit.tool_number, hit.x, hit.y, hit.units, hit.source_line
                    );
                }
                println!("hit_preview_truncated={}", hits.len() > 20);
            } else {
                println!("finished_hole_positions=NOT_IMPORTED");
            }
            println!("plated_barrel_physical_continuity=NOT_VERIFIED");
            println!("backdrill_remaining_copper=NOT_VERIFIED");
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
