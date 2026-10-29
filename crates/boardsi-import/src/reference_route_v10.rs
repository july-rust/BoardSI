//! C_014 opt-in Source-exact multiple rectangular COPCUT / Signal Via Pad audit.
//! Geometry only: no manufactured copper, physical SI, P009, solver or DRC.
use crate::{copper_preview,reference_route_v8,reference_route_v9,route_preview};
use std::fmt::Write;
#[path="reference_route_v10/geometry.rs"]
mod geometry;
const MAX_WIRE:usize=270_000;
fn hex(s:&str)->String{
    let mut r=String::with_capacity(s.len()*2);
    for b in s.bytes(){write!(&mut r,"{b:02x}").expect("hex");}r
}
pub fn encode(bytes:&[u8],indices:&[usize],split:usize,refs:[&str;2],
              layers:[usize;2],reference_via:usize)->Result<String,&'static str>{
    let v8=reference_route_v8::encode(bytes,indices,split,refs,layers,reference_via)?;
    let h:Vec<_>=v8.lines().next().ok_or("V10_V8_EMPTY")?.split('|').collect();
    if h.len()!=13 || h[0]!="BOARDSI_REFERENCE_ROUTE_V8" || h[1].len()!=64
       || h[10]!=reference_via.to_string(){return Err("V10_V8_HEADER");}
    let signal_index=h[11].parse::<usize>().map_err(|_|"V10_SIGNAL_INDEX")?;
    let parsed=route_preview::parse(bytes)?;
    let mut board=route_preview::into_canonical(bytes,&parsed)?;
    let areas=copper_preview::parse(bytes,parsed.max_layer)?;
    copper_preview::augment(&mut board,&areas)?;
    if board.validate().has_errors() || board.areas.len()!=areas.len(){
        return Err("V10_CANONICAL_INVALID");
    }
    let signal=parsed.vias.get(signal_index).ok_or("V10_SIGNAL_MISSING")?;
    let stitch=parsed.vias.get(reference_via).ok_or("V10_STITCH_MISSING")?;
    if signal.net.is_empty() || signal.net==stitch.net || stitch.net!=refs[0]
       || refs[0]!=refs[1]{return Err("V10_NET_MISMATCH");}
    let mut rows=String::new();
    let mut overall="SOURCE_CUTOUT_GAP_OBSERVED";
    let mut single=true;
    for &layer in &layers{
        let diameter=reference_route_v8::verify_round_pad(&board,signal,signal_index,layer)?;
        reference_route_v8::verify_round_pad(&board,stitch,reference_via,layer)?;
        let layer_id=&board.stackup.layers
           .get(layer.checked_sub(1).ok_or("V10_LAYER")?)
           .ok_or("V10_LAYER")?.meta.id;
        let matching:Vec<_>=areas.iter().enumerate().filter(|(i,a)|{
            a.layer as usize==layer && a.net_label.as_deref()==Some(refs[0])
              && board.areas.get(*i).is_some_and(|b|b.layer==*layer_id)
        }).collect();
        let [(area_index,area)]=matching.as_slice() else{
            return Err("V10_AMBIGUOUS_REFERENCE_AREA");
        };
        let copper_preview::CopperGeometry::Polygon(ring)=&area.geometry else{
            return Err("V10_OUTER_UNSUPPORTED");
        };
        let outer=copper_preview::rectangle_bounds(ring).ok_or("V10_OUTER_NONRECT")?;
        if !geometry::pad_inside(signal.point,diameter,outer){
            return Err("V10_PAD_OUTSIDE_OUTER");
        }
        if area.cutouts.len()>8{return Err("V10_TOO_MANY_CUTOUTS");}
        if area.cutouts.len()>1{single=false;}
        let mut holes:Vec<[i64;4]>=Vec::with_capacity(area.cutouts.len());
        for cutout in &area.cutouts{
            let copper_preview::CopperGeometry::Polygon(p)=cutout else{
                return Err("V10_CUTOUT_UNSUPPORTED");
            };
            let rect=copper_preview::rectangle_bounds(p).ok_or("V10_CUTOUT_NONRECT")?;
            if !geometry::hole_inside(rect,outer)
               || holes.iter().any(|h|!geometry::disjoint(*h,rect)){
                return Err("V10_CUTOUT_TOUCH_OR_OUTSIDE");
            }
            holes.push(rect);
        }
        let (state,chosen)=geometry::classify(signal.point,diameter,&holes);
        if geometry::severity(state)>geometry::severity(overall){overall=state;}
        let source=&board.areas[*area_index].meta.id.source_object;
        writeln!(&mut rows,"LAYER|{layer}|{signal_index}|{}|{}|{}|{diameter}|{}|{state}|{}",
           hex(source),signal.point[0],signal.point[1],holes.len(),
           chosen.map_or_else(||"-".to_string(),|i|i.to_string()))
           .map_err(|_|"V10_FORMAT")?;
        for (i,[x0,y0,x1,y1]) in holes.iter().copied().enumerate(){
            writeln!(&mut rows,"HOLE|{layer}|{i}|{x0}|{y0}|{x1}|{y1}")
                .map_err(|_|"V10_FORMAT")?;
        }
    }
    let (legacy_kind,legacy)=if single{
        ("V9",reference_route_v9::encode(bytes,indices,split,refs,layers,reference_via)?)
    } else{("V8",v8)};
    let mut out=format!(
        "BOARDSI_REFERENCE_ROUTE_V10|{}|{overall}|UNSUPPORTED|NOT_RUN|0|REFERENCE_TRANSFER_UNVERIFIED|2|{}|{split}|{reference_via}|{signal_index}|{legacy_kind}\n{rows}LEGACY|{legacy_kind}|{}\n",
        h[1],indices.len(),legacy.lines().count()
    );
    out.push_str(&legacy);
    out.push_str("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n");
    if out.len()>MAX_WIRE{return Err("V10_OUTPUT_LIMIT");}
    Ok(out)
}
