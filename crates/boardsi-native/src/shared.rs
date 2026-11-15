use boardsi_model::*;
use std::collections::BTreeMap;
use crate::{invalid,Result};

pub(crate) fn kind_name(k:ObjectKind)->&'static str {
    match k {ObjectKind::Frame=>"Frame",ObjectKind::Layer=>"Layer",ObjectKind::Material=>"Material",
        ObjectKind::Net=>"Net",ObjectKind::Component=>"Component",ObjectKind::Pin=>"Pin",
        ObjectKind::Trace=>"Trace",ObjectKind::Arc=>"Arc",ObjectKind::Area=>"Area",
        ObjectKind::Cutout=>"Cutout",ObjectKind::Padstack=>"Padstack",ObjectKind::Pad=>"Pad",
        ObjectKind::Via=>"Via",ObjectKind::Drill=>"Drill",ObjectKind::Outline=>"Outline",
        ObjectKind::Node=>"Node",ObjectKind::Edge=>"Edge"}
}
pub(crate) fn from_kind(k:&str)->Result<ObjectKind>{
    match k {"Frame"=>Ok(ObjectKind::Frame),"Layer"=>Ok(ObjectKind::Layer),
        "Material"=>Ok(ObjectKind::Material),"Net"=>Ok(ObjectKind::Net),
        "Component"=>Ok(ObjectKind::Component),"Pin"=>Ok(ObjectKind::Pin),
        "Trace"=>Ok(ObjectKind::Trace),"Arc"=>Ok(ObjectKind::Arc),
        "Area"=>Ok(ObjectKind::Area),"Cutout"=>Ok(ObjectKind::Cutout),
        "Padstack"=>Ok(ObjectKind::Padstack),"Pad"=>Ok(ObjectKind::Pad),
        "Via"=>Ok(ObjectKind::Via),"Drill"=>Ok(ObjectKind::Drill),
        "Outline"=>Ok(ObjectKind::Outline),"Node"=>Ok(ObjectKind::Node),
        "Edge"=>Ok(ObjectKind::Edge),_=>Err(invalid("unknown object kind"))}
}
pub(crate) fn cap_name(s:CapabilityState)->&'static str{
    match s{CapabilityState::Complete=>"Complete",CapabilityState::Partial=>"Partial",
        CapabilityState::Missing=>"Missing",CapabilityState::Conflicting=>"Conflicting"}
}
pub(crate) fn from_cap(s:&str)->Result<CapabilityState>{
    match s{"Complete"=>Ok(CapabilityState::Complete),"Partial"=>Ok(CapabilityState::Partial),
        "Missing"=>Ok(CapabilityState::Missing),"Conflicting"=>Ok(CapabilityState::Conflicting),
        _=>Err(invalid("capability"))}
}
pub(crate) fn quality_name(q:ConversionQuality)->&'static str{
    match q{ConversionQuality::Exact=>"Exact",ConversionQuality::Approximate=>"Approximate",
        ConversionQuality::Lossy=>"Lossy",ConversionQuality::Pending=>"Pending"}
}
pub(crate) fn from_quality(s:&str)->Result<ConversionQuality>{
    match s{"Exact"=>Ok(ConversionQuality::Exact),"Approximate"=>Ok(ConversionQuality::Approximate),
        "Lossy"=>Ok(ConversionQuality::Lossy),"Pending"=>Ok(ConversionQuality::Pending),
        _=>Err(invalid("conversion quality"))}
}
pub(crate) fn layer_name(k:LayerKind)->&'static str{
    match k{LayerKind::Copper=>"Copper",LayerKind::Dielectric(DielectricKind::Core)=>"DielectricCore",
        LayerKind::Dielectric(DielectricKind::Prepreg)=>"DielectricPrepreg",
        LayerKind::Dielectric(DielectricKind::Other)=>"DielectricOther",
        LayerKind::Mask=>"Mask",LayerKind::Other=>"Other"}
}
pub(crate) fn from_layer(s:&str)->Result<LayerKind>{
    match s{"Copper"=>Ok(LayerKind::Copper),
        "DielectricCore"=>Ok(LayerKind::Dielectric(DielectricKind::Core)),
        "DielectricPrepreg"=>Ok(LayerKind::Dielectric(DielectricKind::Prepreg)),
        "DielectricOther"=>Ok(LayerKind::Dielectric(DielectricKind::Other)),
        "Mask"=>Ok(LayerKind::Mask),"Other"=>Ok(LayerKind::Other),
        _=>Err(invalid("layer kind"))}
}
pub(crate) fn net_name(k:NetRole)->&'static str{
    match k{NetRole::Signal=>"Signal",NetRole::Ground=>"Ground",NetRole::Power=>"Power",
        NetRole::Floating=>"Floating",NetRole::Unknown=>"Unknown"}
}
pub(crate) fn from_net(s:&str)->Result<NetRole>{
    match s{"Signal"=>Ok(NetRole::Signal),"Ground"=>Ok(NetRole::Ground),
        "Power"=>Ok(NetRole::Power),"Floating"=>Ok(NetRole::Floating),
        "Unknown"=>Ok(NetRole::Unknown),_=>Err(invalid("net role"))}
}
pub(crate) fn source_name(f:&SourceFormat)->(&'static str,Option<&str>){
    match f{SourceFormat::PadsAsc=>("PadsAsc",None),SourceFormat::Odb=>("Odb",None),
        SourceFormat::Ipc2581=>("Ipc2581",None),SourceFormat::KiCad=>("KiCad",None),
        SourceFormat::Gerber=>("Gerber",None),SourceFormat::Synthetic=>("Synthetic",None),
        SourceFormat::Other(x)=>("Other",Some(x.as_str()))}
}
pub(crate) fn from_source(s:&str,other:Option<String>)->Result<SourceFormat>{
    Ok(match s{"PadsAsc"=>SourceFormat::PadsAsc,"Odb"=>SourceFormat::Odb,
        "Ipc2581"=>SourceFormat::Ipc2581,"KiCad"=>SourceFormat::KiCad,
        "Gerber"=>SourceFormat::Gerber,"Synthetic"=>SourceFormat::Synthetic,
        "Other" if other.as_ref().is_some_and(|x|!x.is_empty())=>SourceFormat::Other(other.expect("checked")),
        _=>return Err(invalid("source format"))})
}
pub(crate) fn bool_from(n:i64)->Result<bool>{
    match n{0=>Ok(false),1=>Ok(true),_=>Err(invalid("non boolean SQL value"))}
}
pub(crate) fn opt_id(ids:&BTreeMap<ObjectId,i64>,id:&Option<ObjectId>)->Result<Option<i64>>{
    id.as_ref().map(|x|id_of(ids,x)).transpose()
}
pub(crate) fn id_of(ids:&BTreeMap<ObjectId,i64>,id:&ObjectId)->Result<i64>{
    ids.get(id).copied().ok_or_else(||invalid("object ref missing"))
}
pub(crate) fn lookup(ids:&BTreeMap<i64,ObjectMeta>,id:i64)->Result<ObjectId>{
    ids.get(&id).map(|m|m.id.clone()).ok_or_else(||invalid("broken object FK"))
}
pub(crate) fn meta(ids:&BTreeMap<i64,ObjectMeta>,id:i64,kind:ObjectKind)->Result<ObjectMeta>{
    let m=ids.get(&id).ok_or_else(||invalid("unknown object"))?;
    if m.id.kind!=kind{return Err(invalid("object kind mismatch"));}
    Ok(m.clone())
}
