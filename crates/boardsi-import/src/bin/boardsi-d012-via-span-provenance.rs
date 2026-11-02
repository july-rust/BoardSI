//! D_012: qualify source Via span, never infer physical plated-barrel electrical continuity.
#[path = "boardsi-d011-source-contact-graph.rs"]
#[allow(dead_code)]
pub mod d011;
use boardsi_import::route_preview;
use std::fmt::Write;

pub fn encode(
    bytes: &[u8],
    names: [&str; 4],
    net: &str,
    layers: [usize; 2],
    index: usize,
) -> Result<String, &'static str> {
    let graph = d011::encode(bytes, names, net, layers, index)?;
    let rows: Vec<_> = graph.lines().collect();
    if rows.len() != 39
        || !graph.ends_with('\n')
        || rows[1] != "D010_BEGIN"
        || rows[22] != "D010_END"
        || rows[38] != "END|SOURCE_ONLY_NO_BARREL_EDGE"
        || !rows[0].starts_with("BOARDSI_D011_SOURCE_GRAPH_V1|")
    {
        return Err("D012_UNQUALIFIED_D011");
    }
    let header: Vec<_> = rows[0].split('|').collect();
    let via: Vec<_> = rows[3].split('|').collect();
    if header.len() != 10 || via.len() != 9 || via[0] != "VIA" || via[1] != "0" {
        return Err("D012_BAD_SOURCE_GRAPH");
    }
    let parsed = route_preview::parse(bytes)?;
    let source = parsed.vias.first().ok_or("D012_SOURCE_VIA_MISSING")?;
    let mut net_hex = String::with_capacity(net.len() * 2);
    for b in net.bytes() {
        write!(&mut net_hex, "{b:02x}").map_err(|_| "D012_NET_HEX")?;
    }
    if parsed.vias.len() != 1
        || via[2] != net_hex
        || via[3] != source.point[0].to_string()
        || via[4] != source.point[1].to_string()
        || via[5] != source.diameter_ticks.to_string()
        || via[6] != source.drill_ticks.to_string()
        || via[7] != source.first_layer.to_string()
        || via[8] != source.last_layer.to_string()
        || layers[0] < source.first_layer as usize
        || layers[1] > source.last_layer as usize
    {
        return Err("D012_VIA_SOURCE_MISMATCH");
    }
    // D_010 independently reconciled this Source Via with its Canonical Via,
    // Drill/Span and both conductive Pad entries. In route_preview::into_canonical,
    // Drill.plated is set to true by the importer, NOT read as source proof.
    // Never promote this assumption to a source-verified electrical edge.
    let mut wire = format!(
        "BOARDSI_D012_VIA_SPAN_PROVENANCE_V1|{}|SOURCE_SPAN_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_NOT_VERIFIED|NO_CROSS_LAYER_ELECTRICAL_EDGE\nD011_BEGIN\n",
        header[1]
    );
    wire.push_str(&graph);
    wire.push_str("D011_END\n");
    wire.push_str(&format!(
        "SPAN|0|{}|{}|{}|{}|{}|{}|{}|{}|{}|PLATING_IMPORTER_ASSUMED\n",
        via[2], via[3], via[4], via[5], via[6], via[7], via[8], layers[0], layers[1]
    ));
    wire.push_str("OVERALL|SOURCE_SPAN_PROVENANCE_ONLY\nEND|NO_ELECTRICAL_BARREL_EDGE\n");
    if wire.len() > 64_000 {
        return Err("D012_WIRE_LIMIT");
    }
    Ok(wire)
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
