// D_014 restricted two-Via SOURCE-only geometry, never manufactured continuity.
#pragma once
#include "d013_legacy_barrel_qt.h"
namespace d014 {
// Restrict the named PADS padstack to exactly two exterior conductive rows;
// D_006 checks their pad diameter/drill and Source geometry independently.
inline bool twoRowViaDefinition(const QByteArray &source,const QString &name){
    bool inVia=false,found=false;
    QString text=QString::fromUtf8(source);
    text.replace("\r\n","\n");
    for(const auto &line:text.split('\n')){
        const auto fields=line.simplified().split(' ',Qt::SkipEmptyParts);
        if(fields.isEmpty())continue;
        if(fields[0]=="*END*")break;
        if(fields[0].startsWith('*')){
            inVia=fields[0]=="*VIA*";
            continue;
        }
        if(inVia && fields[0]==name){
            if(found || fields.size()!=3 || fields[2]!="2")return false;
            found=true;
        }
    }
    return found;
}
inline bool decode(const QByteArray &wire,const QByteArray &source,
                   const PreviewImportPanel *panel,const QString &net,c005::Receipt &out){
    if(!panel || !panel->imported() || wire.isEmpty() || wire.size()>16384
       || source.isEmpty() || source.size()>8*1024*1024)return false;
    const QByteArray sha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    if(panel->acceptedSourceSha256()!=sha || panel->imported()->importedSkippedCount()!=0)return false;
    const auto *art=panel->imported();
    ImportedVia a,b,extra;
    if(!art->importedVia(0,a) || !art->importedVia(1,b) || art->importedVia(2,extra)
       || art->importedTraceCount()!=3 || art->arcCount()!=0 || a.net!=net || b.net!=net
       || a.name.isEmpty() || a.name!=b.name || (a.x_nm==b.x_nm && a.y_nm==b.y_nm)
       || a.firstLayer!=1 || a.lastLayer!=4 || b.firstLayer!=1 || b.lastLayer!=4
       || a.drill_nm<=0 || a.diameter_nm<=a.drill_nm
       || a.diameter_nm!=b.diameter_nm || a.drill_nm!=b.drill_nm
       || !twoRowViaDefinition(source,a.name)
       || !d006::declaredConductivePad(source,a,1)
       || !d006::declaredConductivePad(source,a,4)
       || !d006::declaredConductivePad(source,b,1)
       || !d006::declaredConductivePad(source,b,4))return false;
    const auto exact=[](qint64 value){return value%254==0;};
    for(const auto v:{a.x_nm,a.y_nm,b.x_nm,b.y_nm,a.diameter_nm,a.drill_nm})
        if(!exact(v))return false;
    ImportedTrack t[3];
    for(int i=0;i<3;++i)if(!art->importedTrack(i,t[i]) || t[i].net!=net
        || t[i].width_nm<=0 || !exact(t[i].width_nm)
        || !exact(t[i].x0_nm) || !exact(t[i].x1_nm)
        || !exact(t[i].y0_nm) || !exact(t[i].y1_nm))return false;
    if(t[0].layer!=1 || t[1].layer!=4 || t[2].layer!=1
       || t[0].x1_nm!=a.x_nm || t[0].y1_nm!=a.y_nm
       || t[1].x0_nm!=a.x_nm || t[1].y0_nm!=a.y_nm
       || t[1].x1_nm!=b.x_nm || t[1].y1_nm!=b.y_nm
       || t[2].x0_nm!=b.x_nm || t[2].y0_nm!=b.y_nm
       // Refuse additional layer-1 contacts not represented by fixed edges.
       || (t[0].x0_nm==b.x_nm && t[0].y0_nm==b.y_nm)
       || (t[2].x1_nm==a.x_nm && t[2].y1_nm==a.y_nm)
       || (t[0].x0_nm==t[2].x1_nm && t[0].y0_nm==t[2].y1_nm))return false;
    QByteArray expected="BOARDSI_D014_TWO_VIA_CHAIN_V1|"+sha+
      "|SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED"
      "|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE\n";
    const ImportedVia vias[2]={a,b};
    for(int i=0;i<2;++i){
        const auto &v=vias[i];
        expected+="VIA|"+QByteArray::number(i)+"|"+QByteArray::number(v.x_nm/254)+
            "|"+QByteArray::number(v.y_nm/254)+"|"+QByteArray::number(v.diameter_nm/254)+
            "|"+QByteArray::number(v.drill_nm/254)+"|"+QByteArray::number(v.firstLayer)+
            "|"+QByteArray::number(v.lastLayer)+"|"+v.name.toUtf8()+"\n";
    }
    for(int i=0;i<3;++i){
        const auto &v=t[i];
        expected+="TRACE|"+QByteArray::number(i)+"|"+QByteArray::number(v.layer)+
            "|"+QByteArray::number(v.x0_nm/254)+"|"+QByteArray::number(v.y0_nm/254)+
            "|"+QByteArray::number(v.x1_nm/254)+"|"+QByteArray::number(v.y1_nm/254)+
            "|"+QByteArray::number(v.width_nm/254)+"\n";
    }
    expected+="NODE|0|VIA_PAD|1|0\nNODE|1|VIA_PAD|4|0\nNODE|2|VIA_PAD|4|1\n"
      "NODE|3|VIA_PAD|1|1\nNODE|4|TRACE|1|0\nNODE|5|TRACE|4|1\nNODE|6|TRACE|1|2\n"
      "EDGE|0|0|4|SOURCE_ENDPOINT_CONTACT_CANDIDATE|1\n"
      "EDGE|1|1|5|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4\n"
      "EDGE|2|2|5|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4\n"
      "EDGE|3|3|6|SOURCE_ENDPOINT_CONTACT_CANDIDATE|1\n"
      "COUNT|NODES|7|LAYER_LOCAL_EDGES|4|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0\n"
      "OVERALL|TWO_VIA_SOURCE_CHAIN_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n";
    if(expected!=wire)return false;
    out.state="TWO_VIA_SOURCE_CANDIDATE_NOT_ELECTRICAL";
    out.reason="D_014 SOURCE CONTACT ONLY; PLATING IMPORTER-ASSUMED; BARREL NOT VERIFIED";
    return true;
}
} // namespace d014
