// C_023 V15: independent Qt readback of exact Source Copper rectangle corridor.
// Never equate Source geometric coverage with electrical/physical return current.
#pragma once
#include "c022_reference_area_risk.h"
#include <algorithm>
#include <array>
#include <vector>

namespace c023 {
inline bool decode(const QByteArray &wire,const QByteArray &sourceSha,
                   const ImportedArtworkPanel *panel,const QStringList &names,
                   const QString &referenceNet,int layer,int traceIndex,
                   c005::Receipt &out) {
    const int n=names.size();
    if(!panel || n<3 || n>6 || wire.isEmpty() || wire.size()>12288 ||
       !wire.endsWith('\n') || sourceSha.size()!=64 || traceIndex<0)return false;
    const auto lines=wire.split('\n');
    if(lines.size()<n+4 || !lines.last().isEmpty())return false;
    const auto h=lines[0].split('|');
    if(h.size()!=12 || h[0]!="BOARDSI_REFERENCE_CORRIDOR_V15" ||
       h[1]!=sourceSha || h[2]!=QByteArray::number(n) ||
       h[3]!=QByteArray::number(traceIndex) || h[8]!="UNSUPPORTED" ||
       h[9]!="NOT_RUN" || h[10]!="0" || h[11]!="0")return false;
    bool ok=false;
    const int count=h[7].toInt(&ok);
    if(!ok || count<0 || count>7 || lines.size()!=n+count+4 ||
       lines[n+count+2]!="END|"+QByteArray::number(n)+"|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")
       return false;
    const int groupCount=h[4].toInt(&ok);
    if(!ok || groupCount<1 || groupCount>n)return false;
    const int boundaryPairs=h[5].toInt(&ok);
    if(!ok || boundaryPairs<0 || boundaryPairs>15)return false;
    const QByteArray v14State=groupCount>1?"DISCONNECTED_ISLAND_RISK":
        boundaryPairs>0?"BOUNDARY_ONLY_RISK":"POSITIVE_AREA_OVERLAP_CANDIDATE";
    QByteArray v14="BOARDSI_REFERENCE_AREA_V14|"+sourceSha+"|"+
        QByteArray::number(n)+"|"+h[4]+"|"+v14State+"|"+h[5]+
        "|UNSUPPORTED|NOT_RUN|0|0\n";
    std::vector<std::array<qint64,4>> rectangles;
    for(int i=0;i<n;++i) {
        v14+=lines[i+1]+'\n';
        const auto f=lines[i+1].split('|');
        if(f.size()!=11 || f[0]!="AREA")return false;
        std::array<qint64,4> r{};
        for(int j=0;j<4;++j) {
            r[static_cast<size_t>(j)]=f[5+j].toLongLong(&ok);
            if(!ok)return false;
        }
        rectangles.push_back(r);
    }
    v14+="END|"+QByteArray::number(n)+"|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL\n";
    c005::Receipt candidate;
    // C_022 independently checks ALL source identities and polygon vertices,
    // Net/Layer, component groups and boundary pairs from imported Artwork.
    if(!c022::decode(v14,sourceSha,panel,names,referenceNet,layer,candidate))
        return false;
    const auto row=lines[n+1].split('|');
    if(row.size()!=8 || row[0]!="TRACE" ||
       row[1]!=QByteArray::number(traceIndex))return false;
    ImportedTrack trace;
    QByteArray traceNet;
    if(!panel->importedTrack(traceIndex,trace) ||
       !c005::boundedHex(row[3],traceNet) ||
       QString::fromUtf8(traceNet)!=trace.net || trace.net==referenceNet ||
       trace.layer==layer || row[2]!=QByteArray::number(trace.layer))return false;
    std::array<qint64,4> t{};
    for(int k=0;k<4;++k) {
        t[static_cast<size_t>(k)]=row[4+k].toLongLong(&ok);
        if(!ok || t[static_cast<size_t>(k)] < -5600000 ||
            t[static_cast<size_t>(k)] > 5600000)return false;
    }
    if(trace.x0_nm!=t[0]*254 || trace.y0_nm!=t[1]*254 ||
       trace.x1_nm!=t[2]*254 || trace.y1_nm!=t[3]*254)return false;
    bool horizontal=t[1]==t[3] && t[0]!=t[2];
    bool vertical=t[0]==t[2] && t[1]!=t[3];
    if(!horizontal && !vertical)return false;
    const qint64 fixed=horizontal?t[1]:t[0];
    const qint64 lo=horizontal?std::min(t[0],t[2]):std::min(t[1],t[3]);
    const qint64 hi=horizontal?std::max(t[0],t[2]):std::max(t[1],t[3]);
    std::vector<std::array<qint64,2>> intervals;
    for(const auto &r:rectangles) {
        if(r[0]>=r[2] || r[1]>=r[3])return false;
        const bool inside=horizontal?(r[1]<fixed && fixed<r[3]):
                                      (r[0]<fixed && fixed<r[2]);
        if(inside) {
            const qint64 start=std::max(lo,horizontal?r[0]:r[1]);
            const qint64 end=std::min(hi,horizontal?r[2]:r[3]);
            if(end>start)intervals.push_back({start,end});
        }
    }
    std::sort(intervals.begin(),intervals.end());
    std::vector<std::array<qint64,2>> gaps;
    qint64 cursor=lo;
    bool touch=false;
    for(const auto &interval:intervals) {
        if(interval[0]>cursor)gaps.push_back({cursor,interval[0]});
        else if(interval[0]==cursor && cursor!=lo)touch=true;
        cursor=std::max(cursor,interval[1]);
    }
    if(cursor<hi)gaps.push_back({cursor,hi});
    if(static_cast<int>(gaps.size())!=count)return false;
    for(int i=0;i<count;++i) {
        const auto g=lines[n+i+2].split('|');
        if(g.size()!=3 || g[0]!="GAP" ||
           g[1]!=QByteArray::number(gaps[static_cast<size_t>(i)][0]) ||
           g[2]!=QByteArray::number(gaps[static_cast<size_t>(i)][1]))
           return false;
        const auto [start,end]=gaps[static_cast<size_t>(i)];
        ImportedReferenceRiskMarker m;
        m.state="CORRIDOR_CANDIDATE_RISK";
        m.reason="C_023 REFERENCE GAP / SOURCE GEOMETRY ONLY / P009 UNSUPPORTED / SI NOT_RUN";
        m.net=trace.net;m.layer=trace.layer;
        m.sourceObject="C_023 SOURCE TRACE "+QString::number(traceIndex);
        m.viaObject="-";m.exactGap=true;
        m.startX_nm=(horizontal?start:fixed)*254;
        m.startY_nm=(horizontal?fixed:start)*254;
        m.endX_nm=(horizontal?end:fixed)*254;
        m.endY_nm=(horizontal?fixed:end)*254;
        m.x_nm=(m.startX_nm+m.endX_nm)/2;
        m.y_nm=(m.startY_nm+m.endY_nm)/2;
        m.intervalParameters=QString("%1..%2 source ticks").arg(start).arg(end);
        candidate.markers.push_back(std::move(m));
    }
    const QByteArray state=count>0?"REFERENCE_GAP_RISK":
        groupCount>1?"DISCONNECTED_COMPONENT_RISK":
        (touch || boundaryPairs>0)?"BOUNDARY_ONLY_RISK":"COVERED_SOURCE_CANDIDATE";
    if(h[6]!=state)return false;
    candidate.state=QString::fromLatin1(state);
    candidate.reason=QString("C_023 %1 GAP(S) / %2 SOURCE COMPONENT(S) / P009 UNSUPPORTED / SI NOT_RUN")
        .arg(count).arg(groupCount);
    out=std::move(candidate);
    return true;
}
} // namespace c023
