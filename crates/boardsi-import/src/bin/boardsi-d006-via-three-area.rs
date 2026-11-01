//! D_006 independent opt-in CLI. No default artwork, connectivity, or solver mutation.
use boardsi_import::{d003_three_area_copcut, digest, route_preview};
use boardsi_model::{PadRole, PadShape, Rational, ViaKind};
use std::fmt::Write;
type Rect = [i64;4];

fn rect_distance_squared(p:[i64;2],r:Rect)->i128 {
    let x=i128::from(p[0]); let y=i128::from(p[1]);
    let dx=if x<i128::from(r[0]) { i128::from(r[0])-x }
        else if x>i128::from(r[2]) { x-i128::from(r[2]) } else {0};
    let dy=if y<i128::from(r[1]) { i128::from(r[1])-y }
        else if y>i128::from(r[3]) { y-i128::from(r[3]) } else {0};
    dx*dx+dy*dy
}
fn intersects(p:[i64;2],d:i64,r:Rect)->bool {
    4*rect_distance_squared(p,r)<=i128::from(d)*i128::from(d)
}
fn inside(p:[i64;2],d:i64,r:Rect)->bool {
    let x=2*i128::from(p[0]);let y=2*i128::from(p[1]);let d=i128::from(d);
    d>0 && x>2*i128::from(r[0])+d && x<2*i128::from(r[2])-d &&
        y>2*i128::from(r[1])+d && y<2*i128::from(r[3])-d
}
fn classify(p:[i64;2],d:i64,rects:[Rect;3],holes:[Option<Rect>;3],groups:[usize;3])
 -> Result<(&'static str, Option<usize>,Option<usize>), &'static str> {
    if d<=0 || groups.iter().any(|&i|i>=3) {return Err("D006_INVALID_PAD");}
    let mut touched=Vec::new();let mut inside_owners=Vec::new();let mut blocked=false;
    for i in 0..3 {
        let r=rects[i];
        if r[0]>=r[2] || r[1]>=r[3] {return Err("D006_BAD_RECT");}
        if intersects(p,d,r) {
            touched.push(i);
            let clear=holes[i].map_or(true,|h| !intersects(p,d,h));
            if !clear {blocked=true;}
            if clear && inside(p,d,r) {inside_owners.push(i);}
        }
    }
    if blocked {return Ok(("SOURCE_COPCUT_RISK",None,None));}
    if touched.len()>1 {return Ok(("SOURCE_AMBIGUOUS_AREA_OWNER",None,None));}
    if inside_owners.len()==1 {
        let i=inside_owners[0];return Ok(("SOURCE_VIA_PAD_AREA_CANDIDATE",Some(i),Some(groups[i])));
    }
    Ok((if touched.is_empty() {"SOURCE_SEPARATION_RISK"} else {"SOURCE_PAD_BOUNDARY_RISK"},None,None))
}
fn parse_num(s:&str)->Result<i64,&'static str> {
    s.parse().map_err(|_|"D006_BAD_D003_NUMBER")
}
pub fn encode(bytes:&[u8],names:[&str;3],net:&str,layer:usize,index:usize)
 -> Result<String,&'static str> {
    if index!=0 || layer==0 {return Err("D006_UNSUPPORTED_SELECTION");}
    let parsed=route_preview::parse(bytes)?;
    if parsed.skipped!=0 || parsed.vias.len()!=1 || !parsed.arcs.is_empty() {
        return Err("D006_INCOMPLETE_SOURCE_INVENTORY");
    }
    let src=&parsed.vias[0];
    if src.net!=net || src.first_layer as usize>layer || (src.last_layer as usize)<layer {
        return Err("D006_VIA_NET_LAYER");
    }
    let board=route_preview::into_canonical(bytes,&parsed)?;
    if board.validate().has_errors() || board.sources.len()!=1 ||
       board.vias.len()!=1 || board.traces.len()!=parsed.segments.len() {
        return Err("D006_CANONICAL_INVENTORY");
    }
    let via=&board.vias[0];
    let net_id=board.nets.iter().find(|n|n.name==net).ok_or("D006_NET")?.meta.id.clone();
    if via.kind!=ViaKind::Through || via.net.as_ref()!=Some(&net_id) ||
       via.position.ticks!=src.point || src.drill_ticks<=0 ||
       src.diameter_ticks<=src.drill_ticks {return Err("D006_VIA_SOURCE");}
    let drill=board.drills.iter().find(|d|d.meta.id==via.drill).ok_or("D006_DRILL")?;
    let expected_drill=Rational::new(i128::from(src.drill_ticks)*127,500_000_000)
        .map_err(|_|"D006_DRILL_RATIO")?;
    if !drill.plated || drill.span!=via.span || drill.diameter_m!=expected_drill {
        return Err("D006_DRILL_MISMATCH");
    }
    let stack=board.padstacks.iter().find(|s|s.meta.id==via.padstack).ok_or("D006_PADSTACK")?;
    let selected_layer=board.stackup.layers.get(layer-1).ok_or("D006_LAYER")?;
    let conductor:Vec<_>=stack.entries.iter().filter(|e|e.layer==selected_layer.meta.id).collect();
    let expected=Rational::new(i128::from(src.diameter_ticks)*127,500_000_000)
        .map_err(|_|"D006_PAD_RATIO")?;
    if stack.entries.iter().any(|e|e.role!=PadRole::Conductive) ||
       conductor.len()!=1 ||
       !matches!(&conductor[0].shape,PadShape::Circle{diameter_m} if *diameter_m==expected) {
        return Err("D006_PAD_SOURCE_MISMATCH");
    }
    let d003=d003_three_area_copcut::encode(bytes,names,net,layer)?;
    let rows:Vec<_>=d003.lines().collect();
    if rows.len()!=9 {return Err("D006_D003_ROWS");}
    let mut rects=[[0i64;4];3];let mut groups=[0usize;3];let mut holes=[None;3];
    for i in 0..3 {
        let f:Vec<_>=rows[i+1].split('|').collect();
        if f.len()!=11 || f[0]!="AREA" || f[1]!=i.to_string() {return Err("D006_D003_AREA");}
        for j in 0..4 {rects[i][j]=parse_num(f[j+5])?;}
        groups[i]=f[10].parse().map_err(|_|"D006_D003_GROUP")?;
    }
    let hole:Vec<_>=rows[4].split('|').collect();
    if hole.len()==6 && hole[0]=="HOLE" {
        let owner:usize=hole[1].parse().map_err(|_|"D006_HOLE_OWNER")?;
        if owner>=3 {return Err("D006_HOLE_OWNER");}
        let mut r=[0;4];for j in 0..4 {r[j]=parse_num(hole[j+2])?;}
        holes[owner]=Some(r);
    } else if hole!=["HOLE","NONE"] {return Err("D006_HOLE_ROW");}
    let (state,owner,group)=classify(src.point,src.diameter_ticks,rects,holes,groups)?;
    let mut sha=String::new();let mut hex_net=String::new();
    for b in digest::sha256(bytes) {write!(sha,"{b:02x}").map_err(|_|"D006_SHA")?;}
    for b in net.as_bytes() {write!(hex_net,"{b:02x}").map_err(|_|"D006_NETHEX")?;}
    let mut wire=format!(
        "BOARDSI_D006_VIA_THREE_AREA_V1|{sha}|SOURCE_GEOMETRY_ONLY|PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|0|0\n{d003}");
    writeln!(wire,"VIA|0|{layer}|{hex_net}|{}|{}|{}|{}|{}|{}",
        src.point[0],src.point[1],src.diameter_ticks,src.drill_ticks,
        src.first_layer,src.last_layer).map_err(|_|"D006_WIRE")?;
    writeln!(wire,"CONTACT|{state}|{}|{}",
       owner.map_or_else(||"-".to_owned(),|v|v.to_string()),
       group.map_or_else(||"-".to_owned(),|v|v.to_string())).map_err(|_|"D006_WIRE")?;
    wire.push_str("END|SOURCE_ONLY_NOT_ELECTRICAL\n");
    if wire.len()>16384 {return Err("D006_WIRE_LIMIT");}
    Ok(wire)
}
fn main() {
    let args:Vec<_>=std::env::args_os().collect();
    if args.len()!=8 {
        eprintln!("D006_USAGE: <board.asc> <AREA_A> <AREA_B> <AREA_C> <NET> <LAYER> <VIA_INDEX>");
        std::process::exit(2);
    }
    let txt=|i:usize|args[i].to_str().ok_or("D006_UTF8");
    let outcome=(||->Result<String,&'static str>{
        let metadata=std::fs::metadata(&args[1]).map_err(|_|"D006_FILE")?;
        if !metadata.is_file() || metadata.len()>route_preview::MAX_FILE_BYTES as u64 {
            return Err("D006_FILE_SIZE");
        }
        let bytes=std::fs::read(&args[1]).map_err(|_|"D006_READ")?;
        encode(&bytes,[txt(2)?,txt(3)?,txt(4)?],txt(5)?,
           txt(6)?.parse().map_err(|_|"D006_LAYER")?,
           txt(7)?.parse().map_err(|_|"D006_INDEX")?)
    })();
    match outcome {
        Ok(result)=>print!("{result}"),
        Err(error)=>{eprintln!("{error}");std::process::exit(3);}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const A:Rect=[0,0,100,100];
    const B:Rect=[95,0,195,100];
    const C:Rect=[190,0,290,100];
    #[test]
    fn whole_footprint_and_area_ambiguity() {
        assert_eq!(classify([50,50],20,[A,B,C],[None;3],[0,0,0]).unwrap().0,
            "SOURCE_VIA_PAD_AREA_CANDIDATE");
        assert_eq!(classify([90,50],20,[A,B,C],[None;3],[0,0,0]).unwrap().0,
            "SOURCE_AMBIGUOUS_AREA_OWNER");
        assert_eq!(classify([10,50],20,[A,B,C],[None;3],[0,0,0]).unwrap().0,
            "SOURCE_PAD_BOUNDARY_RISK");
    }
    #[test]
    fn cutout_tangent_rejected() {
        assert_eq!(classify([50,50],20,[A,B,C],[Some([60,40,80,60]),None,None],[0,0,0]).unwrap().0,
            "SOURCE_COPCUT_RISK");
    }
}
