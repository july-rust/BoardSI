//! C_016 opt-in full Reference Stitch Via Pad source clearance V12.
//! V11 is embedded verbatim; never a fabricated electrical/physical connection.
pub mod geometry;
use crate::{copper_preview, reference_route_v11, reference_route_v8, route_preview};
use copper_preview::CopperGeometry;
use reference_route_v11::geometry::SourceHole;
use std::fmt::Write;

fn hex(bytes:&[u8])->String {
    let mut out=String::with_capacity(bytes.len()*2);
    for x in bytes {write!(&mut out,"{x:02x}").expect("hex");}
    out
}
pub fn encode(bytes:&[u8],indices:&[usize],split:usize,refs:[&str;2],
    layers:[usize;2],stitch_index:usize)->Result<String,&'static str> {
    // Authenticated original-source V8/V11 ancestry is mandatory.
    let v11=reference_route_v11::encode(bytes,indices,split,refs,layers,stitch_index)?;
    let head:Vec<_>=v11.lines().next().ok_or("V12_V11_EMPTY")?.split('|').collect();
    if head.len()!=14 || head[0]!="BOARDSI_REFERENCE_ROUTE_V11"
        || head[1].len()!=64 || head[8]!=indices.len().to_string()
        || head[9]!=split.to_string() || head[10]!=stitch_index.to_string()
        || head[12]!="V8" {return Err("V12_INVALID_ANCESTRY")}
    let signal_index:usize=head[11].parse().map_err(|_|"V12_SIGNAL_INDEX")?;
    let parsed=route_preview::parse(bytes)?;
    let mut board=route_preview::into_canonical(bytes,&parsed)?;
    let areas=copper_preview::parse(bytes,parsed.max_layer)?;
    copper_preview::augment(&mut board,&areas)?;
    if board.validate().has_errors() || board.areas.len()!=areas.len() {
        return Err("V12_CANONICAL_INVALID");
    }
    let stitch=parsed.vias.get(stitch_index).ok_or("V12_STITCH_MISSING")?;
    if stitch_index==signal_index || refs[0]!=refs[1] || stitch.net!=refs[0] {
        return Err("V12_VIA_IDENTITY");
    }
    let mut rows=String::new();
    let mut overall=geometry::Risk::Full;
    for &layer in &layers {
        let diameter=reference_route_v8::verify_round_pad(
            &board,stitch,stitch_index,layer)?;
        let layer_id=&board.stackup.layers
            .get(layer.checked_sub(1).ok_or("V12_LAYER")?)
            .ok_or("V12_LAYER")?.meta.id;
        let matches:Vec<_>=areas.iter().enumerate().filter(|(i,a)|
            a.layer as usize==layer && a.net_label.as_deref()==Some(refs[0])
            && board.areas.get(*i).is_some_and(|c| c.layer==*layer_id)
        ).collect();
        let [(area_index,area)]=matches.as_slice() else {
            return Err("V12_REFERENCE_AREA_AMBIGUOUS")
        };
        let CopperGeometry::Polygon(polygon)=&area.geometry else {
            return Err("V12_OUTER_UNSUPPORTED")
        };
        let outer=copper_preview::rectangle_bounds(polygon).ok_or("V12_OUTER_NONRECT")?;
        let mut holes=Vec::new();
        for hole in &area.cutouts {
            holes.push(match hole {
                CopperGeometry::Polygon(p)=>SourceHole::Rectangle(
                    copper_preview::rectangle_bounds(p).ok_or("V12_HOLE_NONRECT")?),
                CopperGeometry::Circle{centre,radius}=>SourceHole::Circle{
                    centre:*centre,radius:*radius
                },
                CopperGeometry::Curved{..}=>return Err("V12_CURVED_UNSUPPORTED"),
            });
        }
        let (risk,selected)=geometry::classify(stitch.point,diameter,outer,&holes)?;
        if risk.rank()>overall.rank() {overall=risk;}
        let id=hex(board.areas[*area_index].meta.id.source_object.as_bytes());
        // Source-independent V11 record must bind the very same Copper Area.
        let expected_prefix=format!("LAYER|{layer}|{signal_index}|{id}|");
        if !v11.lines().any(|line|line.starts_with(&expected_prefix)){
            return Err("V12_V11_AREA_ID_MISMATCH")
        }
        writeln!(&mut rows,
            "STITCH_PAD|{layer}|{stitch_index}|{id}|{}|{}|{diameter}|{}|{}",
            stitch.point[0],stitch.point[1],risk.wire(),
            selected.map_or_else(||"-".to_string(),|n|n.to_string())
        ).map_err(|_|"V12_FORMAT")?;
    }
    let mut wire=format!(
        "BOARDSI_REFERENCE_ROUTE_V12|{}|{}|UNSUPPORTED|NOT_RUN|0|REFERENCE_TRANSFER_UNVERIFIED|2|{}|{split}|{stitch_index}|{signal_index}|V11\n{rows}LEGACY|V11|{}\n",
        head[1],overall.wire(),indices.len(),v11.lines().count()
    );
    wire.push_str(&v11);
    wire.push_str("END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n");
    if wire.len()>330_000{return Err("V12_OUTPUT_LIMIT")}
    Ok(wire)
}
