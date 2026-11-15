//! Exact translation between Canonical geometry and F_002 typed records.
use boardsi_model::*;
use std::collections::BTreeMap;
use crate::{invalid, Field, GeometryRecord, Result};

pub(crate) fn i128_bytes(n: i128) -> Vec<u8> { n.to_le_bytes().to_vec() }
pub(crate) fn from_i128(b: &[u8]) -> Result<i128> {
    if b.len()!=16 { return Err(invalid("i128 length")); }
    Ok(i128::from_le_bytes(b.try_into().expect("length")))
}
pub(crate) fn rational_bytes(r: Rational) -> Vec<u8> {
    let mut b = i128_bytes(r.numerator());
    b.extend_from_slice(&i128_bytes(r.denominator()));
    b
}
pub(crate) fn from_rational(b: &[u8]) -> Result<Rational> {
    if b.len()!=32 { return Err(invalid("rational length")); }
    let n=from_i128(&b[..16])?;
    let d=from_i128(&b[16..])?;
    let r=Rational::new(n,d).map_err(|_|invalid("rational sign"))?;
    if r.numerator()!=n || r.denominator()!=d { return Err(invalid("unreduced rational")); }
    Ok(r)
}
pub(crate) fn sql_orientation(o: &Orientation) -> Result<(&'static str,Option<i64>,Option<f64>,i64)> {
    Ok(match o {
        Orientation::Orthogonal{turn,mirror_x}=>("Orthogonal",Some(turn_code(*turn) as i64),None,*mirror_x as i64),
        Orientation::PendingRotation{radians,mirror_x} if radians.is_finite()=>
            ("PendingRotation",None,Some(*radians),*mirror_x as i64),
        _=>return Err(invalid("nonfinite pending orientation")),
    })
}
pub(crate) fn from_sql_orientation(mode:&str,quarter:Option<i64>,radians:Option<f64>,mirror:i64)->Result<Orientation>{
    if mirror!=0 && mirror!=1 {return Err(invalid("orientation mirror"));}
    match (mode,quarter,radians) {
        ("Orthogonal",Some(q),None)=>Ok(Orientation::Orthogonal{turn:turn_from(q)?,mirror_x:mirror==1}),
        ("PendingRotation",None,Some(r)) if r.is_finite()=>
            Ok(Orientation::PendingRotation{radians:r,mirror_x:mirror==1}),
        _=>Err(invalid("SQL orientation")),
    }
}
fn turn_code(t:QuarterTurn)->u8 {
    match t { QuarterTurn::Zero=>0,QuarterTurn::One=>1,QuarterTurn::Two=>2,QuarterTurn::Three=>3 }
}
fn turn_from(t:i64)->Result<QuarterTurn>{
    match t {0=>Ok(QuarterTurn::Zero),1=>Ok(QuarterTurn::One),
        2=>Ok(QuarterTurn::Two),3=>Ok(QuarterTurn::Three),_=>Err(invalid("quarter turn"))}
}
pub(crate) fn orientation_bytes(o:&Orientation)->Result<Vec<u8>> {
    let (mode,q,r,mirror)=sql_orientation(o)?;
    let mut b=vec![if mode=="Orthogonal"{0}else{1},mirror as u8,q.map_or(255,|x|x as u8),0];
    if let Some(r)=r {b.extend_from_slice(&r.to_le_bytes());}
    Ok(b)
}
pub(crate) fn from_orientation_bytes(b:&[u8])->Result<Orientation> {
    if b.len()<4 || b[1]>1 || b[3]!=0 {return Err(invalid("orientation bytes"));}
    match b[0]{
        0 if b.len()==4=>Ok(Orientation::Orthogonal{turn:turn_from(b[2] as i64)?,mirror_x:b[1]==1}),
        1 if b.len()==12 && b[2]==255=>{
            let r=f64::from_le_bytes(b[4..12].try_into().expect("length"));
            if !r.is_finite(){return Err(invalid("nonfinite orientation"));}
            Ok(Orientation::PendingRotation{radians:r,mirror_x:b[1]==1})
        }
        _=>Err(invalid("unknown orientation")),
    }
}
fn f(id:u16,ty:u16,bytes:Vec<u8>)->Field {Field{id,ty,bytes}}
pub(crate) fn b(v:bool)->Vec<u8>{vec![u8::from(v)]}
pub(crate) fn u32bytes(v:u32)->Vec<u8>{v.to_le_bytes().to_vec()}
fn get_u32(bytes:&[u8])->Result<u32> {
    if bytes.len()!=4 {return Err(invalid("u32 length"));}
    Ok(u32::from_le_bytes(bytes.try_into().expect("length")))
}
fn get_u16(bytes:&[u8])->Result<u16>{
    if bytes.len()!=2{return Err(invalid("u16 length"));}
    Ok(u16::from_le_bytes(bytes.try_into().expect("length")))
}
#[derive(Debug, Clone)]
pub(crate) struct RefDict {
    pub entries:Vec<(ObjectId,&'static str)>,
    indexes:BTreeMap<ObjectId,u32>,
}
impl RefDict {
    fn new(candidates:Vec<(ObjectId,&'static str)>)->Result<Self>{
        let mut indexed:BTreeMap<ObjectId,&'static str>=BTreeMap::new();
        for (id,role) in candidates {
            if let Some(old)=indexed.insert(id.clone(),role) {
                if old!=role {return Err(invalid("conflicting reference roles"));}
            }
        }
        if indexed.len()>65_535 {return Err(invalid("too many geometry refs"));}
        let mut entries=Vec::with_capacity(indexed.len());let mut indexes=BTreeMap::new();
        for (i,(id,role)) in indexed.into_iter().enumerate(){
            indexes.insert(id.clone(),i as u32);entries.push((id,role));
        }
        Ok(Self{entries,indexes})
    }
    fn index(&self,id:&ObjectId)->Result<u32>{
        self.indexes.get(id).copied().ok_or_else(||invalid("unmapped geometry ref"))
    }
    pub fn get(&self,index:u32)->Result<&ObjectId>{
        self.entries.get(index as usize).map(|x|&x.0).ok_or_else(||invalid("geometry ref index"))
    }
}
fn point_refs(p:&Point,r:&mut Vec<(ObjectId,&'static str)>){r.push((p.frame.clone(),"Frame"));}
fn boundary_refs(b:&Boundary,r:&mut Vec<(ObjectId,&'static str)>){
    for seg in &b.segments {
        match seg {
            BoundarySegment::Line{start,end}=>{point_refs(start,r);point_refs(end,r);}
            BoundarySegment::Arc{centre,start,end,..}=>{
                point_refs(centre,r);point_refs(start,r);point_refs(end,r);
            }
        }
    }
}
fn point_bytes(p:&Point,r:&RefDict)->Result<Vec<u8>> {
    let mut b=u32bytes(r.index(&p.frame)?);
    b.extend_from_slice(&p.ticks[0].to_le_bytes());
    b.extend_from_slice(&p.ticks[1].to_le_bytes());
    Ok(b)
}
fn from_point(b:&[u8],r:&RefDict)->Result<Point>{
    if b.len()!=20{return Err(invalid("point size"));}
    let idx=get_u32(&b[..4])?;
    let frame=r.get(idx)?;
    if frame.kind!=ObjectKind::Frame{return Err(invalid("point frame ref kind"));}
    Ok(Point {frame:frame.clone(),ticks:[
        i64::from_le_bytes(b[4..12].try_into().expect("length")),
        i64::from_le_bytes(b[12..20].try_into().expect("length"))]})
}
fn boundary_bytes(b:&Boundary,r:&RefDict)->Result<Vec<u8>> {
    let count=u32::try_from(b.segments.len()).map_err(|_|invalid("boundary size"))?;
    if count==0 || count>1_000_000{return Err(invalid("boundary count"));}
    let mut out=u32bytes(count);
    for seg in &b.segments {
        match seg {
            BoundarySegment::Line{start,end}=>{
                out.push(1);out.extend_from_slice(&point_bytes(start,r)?);
                out.extend_from_slice(&point_bytes(end,r)?);
            }
            BoundarySegment::Arc{centre,start,end,clockwise}=>{
                out.push(2);out.extend_from_slice(&point_bytes(centre,r)?);
                out.extend_from_slice(&point_bytes(start,r)?);
                out.extend_from_slice(&point_bytes(end,r)?);
                out.push(u8::from(*clockwise));
            }
        }
    }
    Ok(out)
}
fn from_boundary(b:&[u8],r:&RefDict)->Result<Boundary>{
    if b.len()<4{return Err(invalid("boundary short"));}
    let count=get_u32(&b[..4])? as usize;
    if count==0||count>1_000_000{return Err(invalid("boundary count"));}
    let mut offset=4usize;let mut segments=Vec::new();
    for _ in 0..count {
        let tag=*b.get(offset).ok_or_else(||invalid("boundary tag"))?;
        let n=match tag{1=>41usize,2=>62usize,_=>return Err(invalid("boundary tag"))};
        let seg=b.get(offset..offset.checked_add(n).ok_or_else(||invalid("overflow"))?)
            .ok_or_else(||invalid("boundary truncated"))?;
        segments.push(match tag{
            1=>BoundarySegment::Line{start:from_point(&seg[1..21],r)?,
                end:from_point(&seg[21..41],r)?},
            _=>{
                if seg[61]>1 {return Err(invalid("boundary clockwise"));}
                BoundarySegment::Arc{centre:from_point(&seg[1..21],r)?,
                    start:from_point(&seg[21..41],r)?,end:from_point(&seg[41..61],r)?,
                    clockwise:seg[61]==1}
            }
        });offset+=n;
    }
    if offset!=b.len(){return Err(invalid("boundary trailing"));}
    Ok(Boundary{segments})
}
fn ref_list(ids:&[ObjectId],r:&RefDict)->Result<Vec<u8>> {
    let mut b=u32bytes(u32::try_from(ids.len()).map_err(|_|invalid("ref count"))?);
    for id in ids {b.extend_from_slice(&u32bytes(r.index(id)?));}Ok(b)
}
fn from_ref_list(b:&[u8],r:&RefDict,expected:ObjectKind)->Result<Vec<ObjectId>>{
    if b.len()<4{return Err(invalid("ref list short"));}
    let count=get_u32(&b[..4])? as usize;
    if count>65_535 || 4usize.checked_add(count.checked_mul(4).ok_or_else(||invalid("overflow"))?)
        !=Some(b.len()){return Err(invalid("ref list count"));}
    let mut v=Vec::with_capacity(count);
    for idx in b[4..].chunks_exact(4) {
        let id=r.get(get_u32(idx)?)?;
        if id.kind!=expected{return Err(invalid("ref target kind"));}
        v.push(id.clone());
    }
    Ok(v)
}
fn span_bytes(span:&ViaSpan,r:&RefDict)->Result<Vec<u8>>{
    let mut b=u32bytes(r.index(&span.first_layer)?);
    b.extend_from_slice(&u32bytes(r.index(&span.last_layer)?));Ok(b)
}
fn from_span(b:&[u8],r:&RefDict)->Result<ViaSpan>{
    if b.len()!=8{return Err(invalid("span size"));}
    let first=r.get(get_u32(&b[..4])?)?.clone();
    let last=r.get(get_u32(&b[4..])?)?.clone();
    if first==last || first.kind!=ObjectKind::Layer || last.kind!=ObjectKind::Layer {
        return Err(invalid("span layers"));
    }
    Ok(ViaSpan{first_layer:first,last_layer:last})
}
fn padstack_bytes(entries:&[PadstackLayer],r:&RefDict)->Result<Vec<u8>> {
    if entries.is_empty() || entries.len()>65_535{return Err(invalid("padstack entries"));}
    let mut out=u32bytes(entries.len() as u32);
    for entry in entries {
        out.extend_from_slice(&u32bytes(r.index(&entry.layer)?));
        out.push(match entry.role{PadRole::Conductive=>0,PadRole::AntiPad=>1,PadRole::Clearance=>2});
        let (shape,payload)=match &entry.shape {
            PadShape::Circle{diameter_m}=>(0,rational_bytes(*diameter_m)),
            PadShape::Rectangle{size_m}=>(1,[rational_bytes(size_m[0]),rational_bytes(size_m[1])].concat()),
            PadShape::Annulus{outer_diameter_m,inner_diameter_m}=>
                (2,[rational_bytes(*outer_diameter_m),rational_bytes(*inner_diameter_m)].concat()),
            PadShape::Thermal{round,centre_size_m,outer_size_m,spoke_width_m,first_spoke_degrees,spoke_count}=>{
                let mut b=b(*round);
                b.extend_from_slice(&rational_bytes(*centre_size_m));
                b.extend_from_slice(&rational_bytes(*outer_size_m));
                b.extend_from_slice(&rational_bytes(*spoke_width_m));
                b.extend_from_slice(&first_spoke_degrees.to_le_bytes());
                b.extend_from_slice(&spoke_count.to_le_bytes());
                (3,b)
            }
            PadShape::Custom(boundary)=>(4,boundary_bytes(boundary,r)?),
        };
        out.push(shape);out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&u32bytes(u32::try_from(payload.len()).map_err(|_|invalid("padstack payload length"))?));
        out.extend_from_slice(&payload);
    }
    Ok(out)
}
fn from_padstack(b:&[u8],r:&RefDict)->Result<Vec<PadstackLayer>>{
    if b.len()<4{return Err(invalid("padstack short"));}
    let count=get_u32(&b[..4])? as usize;
    if count==0||count>65_535{return Err(invalid("padstack count"));}
    let mut pos=4;let mut out=Vec::with_capacity(count);
    for _ in 0..count {
        let hdr=b.get(pos..pos+12).ok_or_else(||invalid("padstack header"))?;
        let layer=r.get(get_u32(&hdr[..4])?)?.clone();
        if layer.kind!=ObjectKind::Layer{return Err(invalid("padstack layer kind"));}
        let role=match hdr[4]{0=>PadRole::Conductive,1=>PadRole::AntiPad,
            2=>PadRole::Clearance,_=>return Err(invalid("padstack role"))};
        let len=get_u32(&hdr[8..12])? as usize;
        if get_u16(&hdr[6..8])?!=0{return Err(invalid("padstack reserved"));}
        pos+=12;
        let body=b.get(pos..pos.checked_add(len).ok_or_else(||invalid("overflow"))?)
            .ok_or_else(||invalid("padstack body"))?;
        let shape=match hdr[5] {
            0 if len==32=>PadShape::Circle{diameter_m:from_rational(body)?},
            1 if len==64=>PadShape::Rectangle{size_m:[from_rational(&body[..32])?,
                from_rational(&body[32..])?]},
            2 if len==64=>PadShape::Annulus{outer_diameter_m:from_rational(&body[..32])?,
                inner_diameter_m:from_rational(&body[32..])?},
            3 if len==103=>{
                if body[0]>1{return Err(invalid("thermal bool"));}
                let angle=get_u16(&body[97..99])?;
                let spokes=get_u32(&body[99..103])?;
                if angle>=360||spokes==0{return Err(invalid("thermal angle/spokes"));}
                PadShape::Thermal{round:body[0]==1,centre_size_m:from_rational(&body[1..33])?,
                    outer_size_m:from_rational(&body[33..65])?,spoke_width_m:from_rational(&body[65..97])?,
                    first_spoke_degrees:angle,spoke_count:spokes}
            }
            4=>PadShape::Custom(from_boundary(body,r)?),
            _=>return Err(invalid("padstack shape")),
        };
        pos+=len;
        out.push(PadstackLayer{layer,role,shape});
    }
    if pos!=b.len(){return Err(invalid("padstack trailing bytes"));}
    Ok(out)
}
pub(crate) enum Geom<'a> {
    Trace(&'a Trace),Arc(&'a Arc),Area(&'a CopperArea),Cutout(&'a Cutout),
    Padstack(&'a Padstack),Pad(&'a Pad),Via(&'a Via),Drill(&'a Drill),Outline(&'a Outline),
}
impl<'a> Geom<'a> {
    pub fn meta(&self)->&ObjectMeta{match self {
        Self::Trace(x)=>&x.meta,Self::Arc(x)=>&x.meta,Self::Area(x)=>&x.meta,
        Self::Cutout(x)=>&x.meta,Self::Padstack(x)=>&x.meta,Self::Pad(x)=>&x.meta,
        Self::Via(x)=>&x.meta,Self::Drill(x)=>&x.meta,Self::Outline(x)=>&x.meta,
    }}
    pub fn encoded(&self)->Result<(GeometryRecord,RefDict)>{
        let mut refs=Vec::new();
        match self {
            Self::Trace(x)=>for p in &x.centreline{point_refs(p,&mut refs);},
            Self::Arc(x)=>for p in [&x.centre,&x.start,&x.end]{point_refs(p,&mut refs);},
            Self::Area(x)=>{boundary_refs(&x.outer,&mut refs);
                for c in &x.cutouts {refs.push((c.meta.id.clone(),"Hole"));}},
            Self::Cutout(x)=>boundary_refs(&x.boundary,&mut refs),
            Self::Padstack(x)=>for e in &x.entries{
                refs.push((e.layer.clone(),"Other"));
                if let PadShape::Custom(b)=&e.shape{boundary_refs(b,&mut refs);}
            },
            Self::Pad(x)=>point_refs(&x.position,&mut refs),
            Self::Via(x)=>{point_refs(&x.position,&mut refs);
                refs.push((x.span.first_layer.clone(),"Other"));
                refs.push((x.span.last_layer.clone(),"Other"));
                for l in &x.connected_layers{refs.push((l.clone(),"Other"));}
                refs.push((x.drill.clone(),"Drill"));},
            Self::Drill(x)=>{refs.push((x.span.first_layer.clone(),"Other"));
                refs.push((x.span.last_layer.clone(),"Other"));},
            Self::Outline(x)=>{boundary_refs(&x.outer,&mut refs);
                for c in &x.cutouts{refs.push((c.meta.id.clone(),"Hole"));}},
        }
        let r=RefDict::new(refs)?;
        let record=match self {
            Self::Trace(x)=>{
                let mut points=u32bytes(u32::try_from(x.centreline.len()).map_err(|_|invalid("trace points"))?);
                for p in &x.centreline{points.extend_from_slice(&point_bytes(p,&r)?);}
                GeometryRecord{kind:1,fields:vec![f(1,7,points),f(2,5,rational_bytes(x.width_m))]}
            }
            Self::Arc(x)=>GeometryRecord{kind:2,fields:vec![
                f(1,6,point_bytes(&x.centre,&r)?),f(2,6,point_bytes(&x.start,&r)?),
                f(3,6,point_bytes(&x.end,&r)?),f(4,1,b(x.clockwise)),f(5,5,rational_bytes(x.width_m))]},
            Self::Area(x)=>GeometryRecord{kind:3,fields:vec![
                f(1,8,boundary_bytes(&x.outer,&r)?),f(2,9,ref_list(
                    &x.cutouts.iter().map(|c|c.meta.id.clone()).collect::<Vec<_>>(),&r)?)]},
            Self::Cutout(x)=>GeometryRecord{kind:4,fields:vec![
                f(1,8,boundary_bytes(&x.boundary,&r)?)]},
            Self::Padstack(x)=>GeometryRecord{kind:5,fields:vec![f(1,11,padstack_bytes(&x.entries,&r)?)]},
            Self::Pad(x)=>GeometryRecord{kind:6,fields:vec![
                f(1,6,point_bytes(&x.position,&r)?),f(2,10,orientation_bytes(&x.orientation)?)]},
            Self::Via(x)=>GeometryRecord{kind:7,fields:vec![
                f(1,6,point_bytes(&x.position,&r)?),
                f(2,2,vec![match x.kind{ViaKind::Through=>0,ViaKind::Blind=>1,
                    ViaKind::Buried=>2,ViaKind::Microvia=>3}]),
                f(3,12,span_bytes(&x.span,&r)?),
                f(4,9,ref_list(&x.connected_layers,&r)?)]},
            Self::Drill(x)=>GeometryRecord{kind:8,fields:vec![
                f(1,5,rational_bytes(x.diameter_m)),f(2,1,b(x.plated)),f(3,12,span_bytes(&x.span,&r)?)]},
            Self::Outline(x)=>GeometryRecord{kind:9,fields:vec![
                f(1,8,boundary_bytes(&x.outer,&r)?),f(2,9,ref_list(
                    &x.cutouts.iter().map(|c|c.meta.id.clone()).collect::<Vec<_>>(),&r)?)]},
        };
        record.validate()?;
        Ok((record,r))
    }
}
pub(crate) enum Decoded {
    Trace(Vec<Point>,Rational),Arc(Point,Point,Point,bool,Rational),
    Area(Boundary,Vec<ObjectId>),Cutout(Boundary),Padstack(Vec<PadstackLayer>),
    Pad(Point,Orientation),Via(Point,ViaKind,ViaSpan,Vec<ObjectId>),
    Drill(Rational,bool,ViaSpan),Outline(Boundary,Vec<ObjectId>),
}
pub(crate) fn decode(record:&GeometryRecord,r:&RefDict)->Result<Decoded>{
    record.validate()?;
    let v:Vec<&[u8]>=record.fields.iter().map(|f|f.bytes.as_slice()).collect();
    Ok(match record.kind {
        1=>{
            let mut pts=Vec::new();
            for p in v[0][4..].chunks_exact(20){pts.push(from_point(p,r)?);}
            Decoded::Trace(pts,from_rational(v[1])?)
        }
        2=>Decoded::Arc(from_point(v[0],r)?,from_point(v[1],r)?,
            from_point(v[2],r)?,v[3][0]==1,from_rational(v[4])?),
        3=>Decoded::Area(from_boundary(v[0],r)?,from_ref_list(v[1],r,ObjectKind::Cutout)?),
        4=>Decoded::Cutout(from_boundary(v[0],r)?),
        5=>Decoded::Padstack(from_padstack(v[0],r)?),
        6=>Decoded::Pad(from_point(v[0],r)?,from_orientation_bytes(v[1])?),
        7=>Decoded::Via(from_point(v[0],r)?,match v[1][0] {
            0=>ViaKind::Through,1=>ViaKind::Blind,2=>ViaKind::Buried,3=>ViaKind::Microvia,
            _=>return Err(invalid("via kind"))},from_span(v[2],r)?,
                from_ref_list(v[3],r,ObjectKind::Layer)?),
        8=>Decoded::Drill(from_rational(v[0])?,v[1][0]==1,from_span(v[2],r)?),
        9=>Decoded::Outline(from_boundary(v[0],r)?,from_ref_list(v[1],r,ObjectKind::Cutout)?),
        _=>return Err(invalid("geometry kind")),
    })
}
pub(crate) fn reference_dict(items:Vec<(ObjectId,&'static str)>)->Result<RefDict>{
    let r=RefDict::new(items)?;
    for (idx,(id,_)) in r.entries.iter().enumerate() {
        if r.index(id)?!=idx as u32{return Err(invalid("reference order"));}
    }
    Ok(r)
}
