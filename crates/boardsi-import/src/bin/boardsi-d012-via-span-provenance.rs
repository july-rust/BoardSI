//! D_012: bounded Via Source Span and importer-assumed plating witness, no electrical claim.
#[path = "boardsi-d011-source-contact-graph.rs"]
#[allow(dead_code)]
pub mod d011;
#[path = "boardsi-d012-witness-core.rs"]
#[allow(dead_code)]
pub mod d012_core;
use boardsi_import::route_preview;

pub fn encode(
    bytes: &[u8],
    names: [&str; 4],
    net: &str,
    layers: [usize; 2],
    index: usize,
) -> Result<String, &'static str> {
    let graph = d011::encode(bytes, names, net, layers, index)?;
    d012_core::encode_from_qualified_graph(bytes, &graph, net, layers)
}

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 10 {
        eprintln!(
            "D012_USAGE: board.asc AREA_A AREA_B AREA_C AREA_D NET LAYER_A LAYER_B VIA_INDEX"
        );
        std::process::exit(2);
    }
    let txt = |i: usize| args[i].to_str().ok_or("D012_UTF8");
    let result = (|| -> Result<String, &'static str> {
        let path = std::path::Path::new(&args[1]);
        let metadata = std::fs::metadata(path).map_err(|_| "D012_FILE")?;
        if !metadata.is_file() || metadata.len() > route_preview::MAX_FILE_BYTES as u64 {
            return Err("D012_FILE_LIMIT");
        }
        let bytes = std::fs::read(path).map_err(|_| "D012_READ")?;
        encode(
            &bytes,
            [txt(2)?, txt(3)?, txt(4)?, txt(5)?],
            txt(6)?,
            [
                txt(7)?.parse().map_err(|_| "D012_LAYER")?,
                txt(8)?.parse().map_err(|_| "D012_LAYER")?,
            ],
            txt(9)?.parse().map_err(|_| "D012_INDEX")?,
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
