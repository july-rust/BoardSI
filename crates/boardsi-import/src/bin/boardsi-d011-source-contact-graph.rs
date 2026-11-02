//! D_011: bounded, SHA-bound SOURCE contact candidate graph; NOT electrical connectivity.
//! Requalify source geometry through D_010 before materializing any graph edge.
#[path = "boardsi-d010-two-trace-four-area.rs"]
#[allow(dead_code)]
mod d010;
use std::fmt::Write;

pub fn encode(
    bytes: &[u8],
    names: [&str; 4],
    net: &str,
    layers: [usize; 2],
    index: usize,
) -> Result<String, &'static str> {
    let source_receipt = d010::encode(bytes, names, net, layers, index)?;
    graph_from_qualified_receipt(&source_receipt, layers)
}

/// Only call with an independently re-created D_010 receipt for the SAME source.
/// This function is deliberately not a general decoder of arbitrary untrusted receipts.
fn graph_from_qualified_receipt(receipt: &str, layers: [usize; 2]) -> Result<String, &'static str> {
    let rows: Vec<_> = receipt.lines().collect();
    if rows.len() != 20
        || !receipt.ends_with('\n')
        || rows[19] != "END|SOURCE_ONLY_NOT_ELECTRICAL"
        || rows[18] != "OVERALL|SOURCE_DUAL_LAYER_TWO_TRACE_FOUR_AREA_CANDIDATE"
        || layers[0] == 0
        || layers[0] >= layers[1]
    {
        return Err("D011_UNQUALIFIED_SOURCE");
    }
    let header: Vec<_> = rows[0].split('|').collect();
    if header.len() != 8
        || header[0] != "BOARDSI_D010_DUAL_LAYER_TWO_TRACE_FOUR_AREA_V1"
        || header[2..]
            != [
                "SOURCE_GEOMETRY_ONLY",
                "PHYSICAL_NOT_VERIFIED",
                "P009_UNSUPPORTED",
                "SI_NOT_RUN",
                "0",
                "0",
            ]
        || header[1].len() != 64
        || !header[1].bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("D011_BAD_SOURCE_HEADER");
    }
    for slot in 0..4 {
        let area: Vec<_> = rows[2 + slot * 3].split('|').collect();
        let contact: Vec<_> = rows[4 + slot * 3].split('|').collect();
        let expected = if slot < 2 {
            "SOURCE_VIA_PAD_AREA_CANDIDATE"
        } else {
            "SOURCE_TRACE_AREA_ENDPOINT_CANDIDATE"
        };
        if area.len() != 9
            || area[0] != "AREA"
            || area[1] != slot.to_string()
            || area[4] != layers[slot % 2].to_string()
            || area[3].is_empty()
            || area[3].len() > 512
            || !area[3].bytes().all(|b| b.is_ascii_hexdigit())
            || contact != ["CONTACT", &slot.to_string(), expected]
        {
            return Err("D011_AREA_NOT_QUALIFIED");
        }
    }
    for slot in 0..2 {
        let trace: Vec<_> = rows[14 + slot].split('|').collect();
        let link: Vec<_> = rows[16 + slot].split('|').collect();
        if trace.len() != 9
            || trace[0] != "TRACE"
            || trace[1] != slot.to_string()
            || trace[2] != layers[slot].to_string()
            || link.len() != 8
            || link[0] != "LINK"
            || link[1] != slot.to_string()
            || link[2] != "SOURCE_VIA_TRACE_TARGET_AREA_CANDIDATE"
            || link[5] != "0"
            || link[6] != (slot + 2).to_string()
            || link[7] != "1"
            || !matches!(link[3], "0" | "1")
            || !matches!(link[4], "0" | "1")
            || link[3] == link[4]
        {
            return Err("D011_TRACE_NOT_QUALIFIED");
        }
    }

    let mut result = String::new();
    writeln!(&mut result,
        "BOARDSI_D011_SOURCE_GRAPH_V1|{}|SOURCE_GEOMETRY_ONLY|PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|8|6|2|0",
        header[1]).map_err(|_| "D011_FORMAT")?;
    result.push_str("D010_BEGIN\n");
    result.push_str(receipt);
    result.push_str("D010_END\n");

    // Fixed ordering gives stable identities even if PADS source route row order differs.
    // Area IDs come from canonical source ownership; trace key is layer+source geometry.
    for slot in 0..2 {
        let layer = layers[slot];
        let offset = slot * 4;
        let via_key = format!("VIA:0:L{layer}");
        let local: Vec<_> = rows[2 + slot * 3].split('|').collect();
        let target: Vec<_> = rows[2 + (slot + 2) * 3].split('|').collect();
        let trace: Vec<_> = rows[14 + slot].split('|').collect();
        let trace_key = format!(
            "TRACE:L{}:{}:{}:{}:{}:{}:{}",
            layer, trace[3], trace[4], trace[5], trace[6], trace[7], trace[8]
        );
        writeln!(&mut result, "NODE|{}|VIA_PAD|{}|{}", offset, layer, via_key)
            .map_err(|_| "D011_FORMAT")?;
        writeln!(
            &mut result,
            "NODE|{}|AREA|{}|{}",
            offset + 1,
            layer,
            local[3]
        )
        .map_err(|_| "D011_FORMAT")?;
        writeln!(
            &mut result,
            "NODE|{}|TRACE|{}|{}",
            offset + 2,
            layer,
            trace_key
        )
        .map_err(|_| "D011_FORMAT")?;
        writeln!(
            &mut result,
            "NODE|{}|AREA|{}|{}",
            offset + 3,
            layer,
            target[3]
        )
        .map_err(|_| "D011_FORMAT")?;
    }
    for slot in 0..2 {
        let layer = layers[slot];
        let o = slot * 4;
        for (edge, a, b) in [(0, o, o + 1), (1, o, o + 2), (2, o + 2, o + 3)] {
            writeln!(
                &mut result,
                "EDGE|{}|{}|{}|SOURCE_CONTACT_CANDIDATE|{}",
                slot * 3 + edge,
                a,
                b,
                layer
            )
            .map_err(|_| "D011_FORMAT")?;
        }
    }
    result.push_str("OVERALL|SOURCE_GRAPH_TWO_LOCAL_COMPONENTS_NOT_ELECTRICAL\n");
    result.push_str("END|SOURCE_ONLY_NO_BARREL_EDGE\n");
    if result.len() > 48_000 {
        return Err("D011_WIRE_LIMIT");
    }
    Ok(result)
}
