// C_021: independent Qt read-back of two SOURCE-only Reference rectangles.
// Never infer electrical continuity / physical Return Current from this receipt.
#pragma once
#include "imported_artwork.h"
#include <QByteArray>
#include <QString>
#include <array>
#include <algorithm>

namespace c021 {
inline bool decode(const QByteArray &wire, const QByteArray &sourceSha,
                   const ImportedArtworkPanel *panel, const QString &areaA,
                   const QString &areaB, const QString &referenceNet, int layer,
                   c005::Receipt &out) {
    if(!panel || wire.size()>4096 || !wire.endsWith('\n') || sourceSha.size()!=64 ||
       areaA.isEmpty() || areaA==areaB || referenceNet.isEmpty() || layer<1 ||
       panel->importedReferenceAreaCount(layer,referenceNet)!=2)return false;
    const auto lines=wire.split('\n');
    if(lines.size()!=5 || !lines.last().isEmpty() ||
       lines[3]!="END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")return false;
    const auto head=lines[0].split('|');
    if(head.size()!=8 || head[0]!="BOARDSI_REFERENCE_AREA_V13" ||
       head[1]!=sourceSha || head[3]!="NOT_REQUESTED" ||
       head[4]!="UNSUPPORTED" || head[5]!="NOT_RUN" ||
       head[6]!="0" || head[7]!="0")return false;
    std::array<std::array<qint64,4>,2> boxes{};
    QStringList identities;
    for(int i=0;i<2;++i) {
        const auto row=lines[1+i].split('|');
        if(row.size()!=10 || row[0]!="AREA" || row[1]!=QByteArray::number(i) ||
           row[4]!=QByteArray::number(layer))return false;
        QByteArray nameBytes,idBytes,netBytes;
        if(!c005::boundedHex(row[2],nameBytes) ||
           !c005::boundedHex(row[3],idBytes) ||
           !c005::boundedHex(row[9],netBytes))return false;
        const QString name=QString::fromUtf8(nameBytes);
        const QString selected=i==0?areaA:areaB;
        const QString sourceId=QString::fromUtf8(idBytes);
        if(name!=selected || QString::fromUtf8(netBytes)!=referenceNet ||
           !sourceId.startsWith("source-copper-"+name+"-"))return false;
        ImportedCopperArea imported;
        if(!panel->importedCopperGeometry(sourceId,layer,referenceNet,imported) ||
           imported.name!=name || imported.circle || imported.curved ||
           !imported.cutouts.empty() || !imported.roundCutouts.empty() ||
           imported.ring.size()!=5 ||
           imported.ring.front()!=imported.ring.back())return false;
        for(int k=0;k<4;++k) {
            bool ok=false;
            boxes[static_cast<size_t>(i)][static_cast<size_t>(k)]=row[5+k].toLongLong(&ok);
            if(!ok || boxes[static_cast<size_t>(i)][static_cast<size_t>(k)] < -5600000 ||
               boxes[static_cast<size_t>(i)][static_cast<size_t>(k)] > 5600000)return false;
        }
        const auto &r=boxes[static_cast<size_t>(i)];
        if(r[0]>=r[2] || r[1]>=r[3])return false;
        bool seen[4]={false,false,false,false};
        for(int k=0;k<4;++k) {
            const auto pt=imported.ring[static_cast<size_t>(k)];
            if(pt.first%254 || pt.second%254)return false;
            int corner=-1;
            if(pt.first==r[0]*254 && pt.second==r[1]*254)corner=0;
            if(pt.first==r[2]*254 && pt.second==r[1]*254)corner=1;
            if(pt.first==r[2]*254 && pt.second==r[3]*254)corner=2;
            if(pt.first==r[0]*254 && pt.second==r[3]*254)corner=3;
            if(corner<0 || seen[corner])return false;
            seen[corner]=true;
        }
        identities << sourceId;
    }
    const auto &a=boxes[0],&b=boxes[1];
    const qint64 dx=std::min(a[2],b[2])-std::max(a[0],b[0]);
    const qint64 dy=std::min(a[3],b[3])-std::max(a[1],b[1]);
    const QByteArray state=(dx>0 && dy>0)
        ? "POSITIVE_AREA_OVERLAP_CANDIDATE"
        : (dx>=0 && dy>=0)
          ? "BOUNDARY_ONLY_RISK" : "DISCONNECTED_ISLAND_RISK";
    if(head[2]!=state)return false;
    ImportedReferenceRiskMarker marker;
    marker.state="CORRIDOR_CANDIDATE_RISK"; // existing viewer warning-only style
    marker.reason=QString("C_021 %1 / %2 / SOURCE GEOMETRY ONLY / P009 UNSUPPORTED / SI NOT_RUN")
        .arg(QString::fromLatin1(state),identities.join(" / "));
    marker.net=referenceNet;marker.layer=layer;
    marker.x_nm=((a[0]+a[2])/2)*254;
    marker.y_nm=((a[1]+a[3])/2)*254;
    marker.sourceObject=identities.join(" / ");
    marker.viaObject="-";
    c005::Receipt r;
    r.state=QString::fromLatin1(state);
    r.reason="SOURCE_RECTANGLE_RELATION_ONLY / P009 UNSUPPORTED / SI NOT_RUN";
    r.markers.push_back(marker);
    out=std::move(r);
    return true;
}
} // namespace c021
