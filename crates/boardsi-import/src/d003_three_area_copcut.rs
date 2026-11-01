//! D_003: three source-owned COPCLS rectangles, one optional owned rectangular COPCUT.
//! Graph edges are source-geometry positive-area candidates, NEVER electrical edges.
use crate::{copper_preview, d002_copcut_contact, digest, pour_preview, route_preview};
use copper_preview::CopperGeometry;
use std::fmt::Write;

fn hex(bytes: &[u8]) -> String {
    let mut s=String::with_capacity(bytes.len()*2);
    for b in bytes { write!(&mut s, "{b:02x}").expect("hex"); }
    s
}

/// Edges are AB, AC, BC. Component IDs are smallest selected index.
/// An internally contained rectangular hole does not disconnect its own
/// rectangular annulus, but can eliminate pairwise intersection completely.
pub fn components(
    boxes: [[i64;4];3], holes: [Option<[i64;4]>;3]
) -> Result<([usize;3],[bool;3]), &'static str> {
    if holes.iter().filter(|h|h.is_some()).count()>1 {
        return Err("D003_MORE_THAN_ONE_CUTOUT");
    }
    let mut groups=[0usize,1,2];
    let mut edges=[false;3];
    for (k,(i,j)) in [(0,1),(0,2),(1,2)].iter().copied().enumerate() {
        let result=d002_copcut_contact::classify(
            [boxes[i],boxes[j]],[holes[i],holes[j]])?;
        if result=="SOURCE_CONTACT_CANDIDATE" {
            edges[k]=true;
            let lo=groups[i].min(groups[j]);
            let hi=groups[i].max(groups[j]);
            for group in &mut groups { if *group==hi { *group=lo; } }
        }
    }
    Ok((groups,edges))
}
fn matching_ring(segs:&[boardsi_model::BoundarySegment], vertices:&[[i64;2]]) -> bool {
    if vertices.len()!=5 || segs.len()!=4 { return false; }
    segs.iter().zip(vertices.windows(2)).all(|(s,pair)| match s {
        boardsi_model::BoundarySegment::Line{start,end} =>
            start.ticks==pair[0] && end.ticks==pair[1] && start.frame==end.frame,
        _ => false,
    })
}

/// Restricted opt-in full Source and Canonical consistency. Does not touch
/// default artwork, physical net graph, solver, cache or P009.
pub fn encode(bytes:&[u8], names:[&str;3], net:&str, layer:usize)
    -> Result<String,&'static str> {
    if names.iter().any(|n|n.is_empty() || n.len()>16)
        || names[0]==names[1] || names[0]==names[2] || names[1]==names[2]
        || net.is_empty() || net.len()>47 || layer==0 {
        return Err("D003_INVALID_SELECTION");
    }
    let parsed=route_preview::parse(bytes)?;
    if !pour_preview::parse(bytes,parsed.max_layer)?.is_empty() {
        return Err("D003_UNVERIFIED_POUR");
    }
    let mut board=route_preview::into_canonical(bytes,&parsed)?;
    let areas=copper_preview::parse(bytes,parsed.max_layer)?;
    copper_preview::augment(&mut board,&areas)?;
    if board.validate().has_errors() || areas.len()!=3 || board.areas.len()!=3 {
        return Err("D003_INCOMPLETE_INVENTORY");
    }
    let canonical_layer=&board.stackup.layers.get(layer-1)
        .ok_or("D003_LAYER_INVALID")?.meta.id;
    let mut boxes=[[0i64;4];3];
    let mut holes=[None;3];
    let mut ids=[String::new(),String::new(),String::new()];
    for (slot,name) in names.iter().enumerate() {
        let idx=areas.iter().position(|a|a.name==*name)
            .ok_or("D003_AREA_UNKNOWN")?;
        let src=&areas[idx];
        let canon=&board.areas[idx];
        if src.layer as usize!=layer || canon.layer!=*canonical_layer
            || src.net_label.as_deref()!=Some(net) || canon.net.is_some()
            || src.cutouts.len()!=canon.cutouts.len() || src.cutouts.len()>1 {
            return Err("D003_UNSUPPORTED_OWNERSHIP");
        }
        let CopperGeometry::Polygon(outer)=&src.geometry else {
            return Err("D003_UNSUPPORTED_OUTER");
        };
        boxes[slot]=copper_preview::rectangle_bounds(outer)
            .ok_or("D003_NOT_RECTANGLE")?;
        if !matching_ring(&canon.outer.segments,outer) {
            return Err("D003_CANONICAL_OUTER_MISMATCH");
        }
        ids[slot]=hex(canon.meta.id.source_object.as_bytes());
        if let Some(cut)=src.cutouts.first() {
            let CopperGeometry::Polygon(vertices)=cut else {
                return Err("D003_UNSUPPORTED_CUTOUT");
            };
            let bounds=copper_preview::rectangle_bounds(vertices)
                .ok_or("D003_CUTOUT_NOT_RECTANGLE")?;
            if !matching_ring(&canon.cutouts[0].boundary.segments,vertices) {
                return Err("D003_CANONICAL_CUTOUT_MISMATCH");
            }
            holes[slot]=Some(bounds);
        }
    }
    let (groups,edges)=components(boxes,holes)?;
    let group_count=groups.iter().copied()
        .collect::<std::collections::BTreeSet<_>>().len();
    let state=if group_count==1 {
        "SOURCE_GEOMETRIC_COMPONENT_CANDIDATE"
    } else { "SOURCE_SEPARATION_RISK" };
    let sha=hex(&digest::sha256(bytes));
    let mut wire=format!(
        "BOARDSI_D003_THREE_AREA_V1|{sha}|{state}|{group_count}|PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|0|0\n"
    );
    for (i,name) in names.iter().enumerate() {
        let r=boxes[i];
        writeln!(&mut wire,"AREA|{i}|{}|{}|{layer}|{}|{}|{}|{}|{}|{}",
            hex(name.as_bytes()),ids[i],r[0],r[1],r[2],r[3],
            hex(net.as_bytes()),groups[i]).map_err(|_|"D003_WIRE_WRITE")?;
    }
    if let Some((owner,h))=holes.iter().enumerate()
        .find_map(|(i,h)|h.map(|r|(i,r))) {
        writeln!(&mut wire,"HOLE|{owner}|{}|{}|{}|{}",
            h[0],h[1],h[2],h[3]).map_err(|_|"D003_WIRE_WRITE")?;
    } else { wire.push_str("HOLE|NONE\n"); }
    for (i,(a,b)) in [(0,1),(0,2),(1,2)].iter().copied().enumerate() {
        writeln!(&mut wire,"EDGE|{a}|{b}|{}",
            if edges[i]{"SOURCE_CONTACT_CANDIDATE"}
            else {"SOURCE_SEPARATION_RISK"}).map_err(|_|"D003_WIRE_WRITE")?;
    }
    wire.push_str("END|3|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n");
    if wire.len()>8192 { return Err("D003_WIRE_LIMIT"); }
    Ok(wire)
}
