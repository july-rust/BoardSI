//! D_013: legacy compatibility barrel provenance, not Source-verified conduction.
#[path = "boardsi-d012-via-span-provenance.rs"]
#[allow(dead_code)]
pub mod d012;
#[path = "../d013_barrel_witness.rs"]
#[allow(dead_code)]
pub mod d013_core;
use boardsi_import::route_preview;

pub fn encode(
    bytes: &[u8],
    names: [&str; 4],
    net: &str,
    layers: [usize; 2],
    index: usize,
) -> Result<String, &'static str> {
    let span = d012::encode(bytes, names, net, layers, index)?;
    d013_core::encode_from_qualified_span(bytes, &span, layers)
}

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 10 {
        eprintln!(
            "D013_USAGE: board.asc AREA_A AREA_B AREA_C AREA_D NET LAYER_A LAYER_B VIA_INDEX"
        );
        std::process::exit(2);
    }
    let text = |i: usize| args[i].to_str().ok_or("D013_UTF8");
    let result = (|| -> Result<String, &'static str> {
        let path = std::path::Path::new(&args[1]);
        let meta = std::fs::metadata(path).map_err(|_| "D013_FILE")?;
        if !meta.is_file() || meta.len() > route_preview::MAX_FILE_BYTES as u64 {
            return Err("D013_FILE_LIMIT");
        }
        let bytes = std::fs::read(path).map_err(|_| "D013_READ")?;
        encode(
            &bytes,
            [text(2)?, text(3)?, text(4)?, text(5)?],
            text(6)?,
            [
                text(7)?.parse().map_err(|_| "D013_LAYER")?,
                text(8)?.parse().map_err(|_| "D013_LAYER")?,
            ],
            text(9)?.parse().map_err(|_| "D013_VIA")?,
        )
    })();
    match result {
        Ok(wire) => print!("{wire}"),
        Err(reason) => {
            eprintln!("{reason}");
            std::process::exit(3);
        }
    }
}
