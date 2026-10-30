// C_022: independent Qt geometry readback for bounded source-owned V14 rectangles.
// A geometric component candidate is NOT electrical continuity.
#pragma once
#include "imported_artwork.h"
#include <QByteArray>
#include <QStringList>
#include <algorithm>
#include <array>
#include <vector>

namespace c022 {
inline bool decode(const QByteArray &wire, const QByteArray &sourceSha,
                   const ImportedArtworkPanel *panel, const QStringList &selected,
                   const QString &referenceNet, int layer, c005::Receipt &out) {
    const int n=selected.size();
    if(!panel || n<3 || n>6 || wire.isEmpty() || wire.size()>8192 ||
       !wire.endsWith('\n') || sourceSha.size()!=64 || referenceNet.isEmpty() ||
       layer<1 || panel->importedReferenceAreaCount(layer,referenceNet)!=n)return false;
    for(int i=0;i<n;++i)
        if(selected[i].isEmpty() || selected[i].size()>64 ||
           selected.indexOf(selected[i])!=i)return false;
    const auto lines=wire.split('\n');
    if(lines.size()!=n+3 || !lines.last().isEmpty() ||
       lines[n+1]!="END|"+QByteArray::number(n)+"|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")
        return false;
    const auto head=lines[0].split('|');
    if(head.size()!=10 || head[0]!="BOARDSI_REFERENCE_AREA_V14" ||
       head[1]!=sourceSha || head[2]!=QByteArray::number(n) ||
       head[6]!="UNSUPPORTED" || head[7]!="NOT_RUN" ||
       head[8]!="0" || head[9]!="0")return false;
    std::vector<std::array<qint64,4>> boxes;
    std::vector<int> recordedGroups;
    QStringList identities;
    for(int i=0;i<n;++i) {
        const auto row=lines[i+1].split('|');
        if(row.size()!=11 || row[0]!="AREA" ||
           row[1]!=QByteArray::number(i) || row[4]!=QByteArray::number(layer))
            return false;
        QByteArray nameBytes,idBytes,netBytes;
        if(!c005::boundedHex(row[2],nameBytes) ||
           !c005::boundedHex(row[3],idBytes) ||
           !c005::boundedHex(row[9],netBytes))return false;
        const QString name=QString::fromUtf8(nameBytes);
        const QString id=QString::fromUtf8(idBytes);
        if(name!=selected[i] || QString::fromUtf8(netBytes)!=referenceNet ||
           !id.startsWith("source-copper-"+name+"-"))return false;
        ImportedCopperArea imported;
        if(!panel->importedCopperGeometry(id,layer,referenceNet,imported) ||
           imported.name!=name || imported.circle || imported.curved ||
           !imported.cutouts.empty() || !imported.roundCutouts.empty() ||
           imported.ring.size()!=5 || imported.ring.front()!=imported.ring.back())
            return false;
        std::array<qint64,4> b{};
        for(int k=0;k<4;++k) {
            bool ok=false;
            b[static_cast<size_t>(k)]=row[5+k].toLongLong(&ok);
            if(!ok || b[static_cast<size_t>(k)] < -5600000 ||
               b[static_cast<size_t>(k)] > 5600000)return false;
        }
        if(b[0]>=b[2] || b[1]>=b[3])return false;
        bool seen[4]={false,false,false,false};
        for(int k=0;k<4;++k) {
            const auto pt=imported.ring[static_cast<size_t>(k)];
            if(pt.first%254 || pt.second%254)return false;
            int corner=-1;
            if(pt.first==b[0]*254 && pt.second==b[1]*254)corner=0;
            if(pt.first==b[2]*254 && pt.second==b[1]*254)corner=1;
            if(pt.first==b[2]*254 && pt.second==b[3]*254)corner=2;
            if(pt.first==b[0]*254 && pt.second==b[3]*254)corner=3;
            if(corner<0 || seen[corner])return false;
            seen[corner]=true;
        }
        bool ok=false;
        const int group=row[10].toInt(&ok);
        if(!ok || group<0 || group>i)return false;
        boxes.push_back(b);
        recordedGroups.push_back(group);
        identities<<id;
    }
    std::vector<int> expected;
    for(int i=0;i<n;++i)expected.push_back(i);
    int boundary=0;
    for(int i=0;i<n;++i)for(int j=i+1;j<n;++j) {
        const auto &a=boxes[static_cast<size_t>(i)];
        const auto &b=boxes[static_cast<size_t>(j)];
        const qint64 dx=std::min(a[2],b[2])-std::max(a[0],b[0]);
        const qint64 dy=std::min(a[3],b[3])-std::max(a[1],b[1]);
        if(dx>0 && dy>0) {
            const int lo=std::min(expected[static_cast<size_t>(i)],expected[static_cast<size_t>(j)]);
            const int hi=std::max(expected[static_cast<size_t>(i)],expected[static_cast<size_t>(j)]);
            for(auto &g:expected)if(g==hi)g=lo;
        } else if(dx>=0 && dy>=0)++boundary;
    }
    if(expected!=recordedGroups)return false;
    std::vector<int> distinct=expected;
    std::sort(distinct.begin(),distinct.end());
    distinct.erase(std::unique(distinct.begin(),distinct.end()),distinct.end());
    const int groupCount=static_cast<int>(distinct.size());
    const QByteArray state=groupCount>1?"DISCONNECTED_ISLAND_RISK":
                           boundary>0?"BOUNDARY_ONLY_RISK":
                           "POSITIVE_AREA_OVERLAP_CANDIDATE";
    if(head[3]!=QByteArray::number(groupCount) || head[4]!=state ||
       head[5]!=QByteArray::number(boundary))return false;
    c005::Receipt receipt;
    receipt.state=QString::fromLatin1(state);
    receipt.reason=QString("C_022 %1 SOURCE GEOMETRY GROUP(S) / P009 UNSUPPORTED / SI NOT_RUN")
        .arg(groupCount);
    for(int i=0;i<n;++i) {
        const auto &b=boxes[static_cast<size_t>(i)];
        ImportedReferenceRiskMarker marker;
        marker.state="CORRIDOR_CANDIDATE_RISK";
        marker.reason=QString("C_022 %1 / COMPONENT %2 / %3 / SOURCE GEOMETRY ONLY / P009 UNSUPPORTED / SI NOT_RUN")
            .arg(QString::fromLatin1(state))
            .arg(expected[static_cast<size_t>(i)])
            .arg(identities[i]);
        marker.net=referenceNet;marker.layer=layer;
        marker.x_nm=((b[0]+b[2])/2)*254;
        marker.y_nm=((b[1]+b[3])/2)*254;
        marker.sourceObject=identities[i];
        marker.viaObject="-";
        receipt.markers.push_back(marker);
    }
    out=std::move(receipt);
    return true;
}
} // namespace c022
