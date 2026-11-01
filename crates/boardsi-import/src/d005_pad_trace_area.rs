//! D_005: source-owned Pad -> one straight Trace -> COPCLS component hypothesis.
//! Geometry candidate only; NEVER a manufactured/electrical contact or solver result.
use crate::{d001_pad_endpoint, d004_endpoint_attachment, digest};
use std::fmt::Write;

#[derive(Debug, PartialEq, Eq)]
pub struct Candidate {
    pub pad_endpoint: Option<usize>,
    pub pad_index: Option<usize>,
    pub area_endpoint: Option<usize>,
    pub area_owner: Option<usize>,
    pub component: Option<usize>,
}
/// D_004 validates source/COPCLS/COPCUT, one straight Trace and immutable SHA.
/// D_001 independently validates source/Canonical Pad inventory and declared Pin/Net.
/// Both are evaluated on the SAME source bytes and neither may create physical edges.
pub fn qualify(
    bytes: &[u8],
    names: [&str; 3],
    net: &str,
    layer: usize,
    trace_index: usize,
) -> Result<(String, Candidate), &'static str> {
    let d004 = d004_endpoint_attachment::encode(bytes, names, net, layer, trace_index)?;
    let pads = d001_pad_endpoint::audit(bytes)?;
    if pads.ambiguous_owners != 0 {
        return Err("D005_AMBIGUOUS_PAD_OWNER");
    }
    let point_rows: Vec<&str> = d004.lines().filter(|r| r.starts_with("POINT|")).collect();
    if point_rows.len() != 2 {
        return Err("D005_BAD_D004_POINTS");
    }
    let mut points = [None; 2];
    for (i, row) in point_rows.iter().enumerate() {
        let cols: Vec<&str> = row.split('|').collect();
        if cols.len() != 5 || cols[0] != "POINT" || cols[1] != i.to_string() {
            return Err("D005_BAD_D004_POINT");
        }
        if cols[2] == "SOURCE_ENDPOINT_ATTACHMENT_CANDIDATE" {
            let owner = cols[3]
                .parse::<usize>()
                .map_err(|_| "D005_BAD_AREA_OWNER")?;
            let group = cols[4]
                .parse::<usize>()
                .map_err(|_| "D005_BAD_AREA_COMPONENT")?;
            if owner >= 3 || group >= 3 {
                return Err("D005_BAD_AREA_GROUP");
            }
            points[i] = Some((owner, group));
        }
    }
    let mut found = Vec::new();
    for witness in &pads.witnesses {
        if witness.trace_index != trace_index
            || witness.layer as usize != layer
            || witness.endpoint >= 2
        {
            continue;
        }
        let opposite = 1 - witness.endpoint;
        if let Some((owner, group)) = points[opposite] {
            found.push((witness.endpoint, witness.pad_index, opposite, owner, group));
        }
    }
    if found.len() > 1 {
        return Err("D005_MULTIPLE_PATH_CANDIDATES");
    }
    let result = if let Some(&(pe, pi, ae, owner, group)) = found.first() {
        Candidate {
            pad_endpoint: Some(pe),
            pad_index: Some(pi),
            area_endpoint: Some(ae),
            area_owner: Some(owner),
            component: Some(group),
        }
    } else {
        Candidate {
            pad_endpoint: None,
            pad_index: None,
            area_endpoint: None,
            area_owner: None,
            component: None,
        }
    };
    Ok((d004, result))
}
pub fn encode(
    bytes: &[u8],
    names: [&str; 3],
    net: &str,
    layer: usize,
    trace_index: usize,
) -> Result<String, &'static str> {
    let (d004, candidate) = qualify(bytes, names, net, layer, trace_index)?;
    let mut sha = String::new();
    for byte in digest::sha256(bytes) {
        write!(&mut sha, "{byte:02x}").map_err(|_| "D005_HEX")?;
    }
    let mut out = format!(
        "BOARDSI_D005_PAD_TRACE_AREA_V1|{sha}|SOURCE_CANDIDATE_ONLY|PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|0|0\n{d004}"
    );
    let value = |v: Option<usize>| v.map_or_else(|| "-".to_owned(), |i| i.to_string());
    writeln!(
        &mut out,
        "LINK|{}|{}|{}|{}|{}|{}",
        if candidate.pad_endpoint.is_some() {
            "SOURCE_PAD_TRACE_AREA_CANDIDATE"
        } else {
            "SOURCE_CONTACT_NOT_VERIFIED"
        },
        value(candidate.pad_endpoint),
        value(candidate.pad_index),
        value(candidate.area_endpoint),
        value(candidate.area_owner),
        value(candidate.component)
    )
    .map_err(|_| "D005_WIRE_WRITE")?;
    out.push_str("END|SOURCE_ONLY_NOT_PHYSICAL\n");
    if out.len() > 16384 {
        return Err("D005_WIRE_LIMIT");
    }
    Ok(out)
}
