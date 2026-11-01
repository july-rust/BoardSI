#pragma once
#include "d006_via_three_area_qt.h"
#include "d004_endpoint_attachment_qt.h"
namespace d007 {
// Source geometry only; electrical and SI are not verified.

using Rect = std::array<qint64,4>;
inline bool decode(const QByteArray &wire,const QByteArray &sha,const QByteArray &source,
    const ImportedArtworkPanel *panel,const QStringList &names,const QString &net,
    int layer,c005::Receipt &out) {
    if(!panel || panel->importedSkippedCount()!=0 || panel->viaCount()!=1
       || panel->importedTraceCount()!=1 || panel->arcCount()!=0
       || wire.size()>32768 || !wire.endsWith('\n') || sha.size()!=64
       || QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex()!=sha)return false;
    const auto lines=wire.split('\n');
    if(lines.size()!=18 || !lines.last().isEmpty()
       || lines[16]!="END|SOURCE_ONLY_NOT_ELECTRICAL")return false;
    const auto h=lines[0].split('|');
    if(h.size()!=8 || h[0]!="BOARDSI_D007_VIA_TRACE_THREE_AREA_V1"
       || h[1]!=sha || h[2]!="SOURCE_GEOMETRY_ONLY"
       || h[3]!="PHYSICAL_NOT_VERIFIED" || h[4]!="P009_UNSUPPORTED"
       || h[5]!="SI_NOT_RUN" || h[6]!="0" || h[7]!="0")return false;
    QByteArray embedded;
    for(int i=1;i<=13;++i){embedded+=lines[i];embedded+='\n';}
    c005::Receipt viaReceipt;
    if(!d006::decode(embedded,sha,source,panel,names,net,layer,viaReceipt))return false;
    ImportedTrack track;
    if(!panel->importedTrack(0,track) || track.net!=net || track.layer!=layer
       || track.width_nm<=0)return false;
    const std::array<qint64,5> raw={track.x0_nm,track.y0_nm,
        track.x1_nm,track.y1_nm,track.width_nm};
    for(qint64 v:raw)if(v%254!=0)return false;
    const auto trace=lines[14].split('|');
    if(trace.size()!=9 || trace[0]!="TRACE" || trace[1]!="0"
       || trace[2]!=QByteArray::number(layer))return false;
    QByteArray n;
    if(!c005::boundedHex(trace[3],n) || QString::fromUtf8(n)!=net)return false;
    for(int i=0;i<5;++i)
        if(trace[i+4]!=QByteArray::number(raw[static_cast<size_t>(i)]/254))return false;
    const std::array<std::array<qint64,2>,2> pts={
        std::array<qint64,2>{raw[0]/254,raw[1]/254},
        std::array<qint64,2>{raw[2]/254,raw[3]/254}};
    ImportedVia via;
    if(!panel->importedVia(0,via) || via.net!=net || via.diameter_nm<=via.drill_nm
       || via.x_nm%254 || via.y_nm%254 || via.diameter_nm%254)return false;
    const std::array<qint64,2> centre={via.x_nm/254,via.y_nm/254};
    const qint64 diameter=via.diameter_nm/254;
    const auto contains=[&](const std::array<qint64,2> &p){
        const qint64 dx=p[0]-centre[0],dy=p[1]-centre[1];
        return 4*(dx*dx+dy*dy)<diameter*diameter;};
    const bool a=contains(pts[0]),b=contains(pts[1]);
    //__D007_BODY__

}
