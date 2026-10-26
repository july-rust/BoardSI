// A_032_2: strict opt-in endpoint-evidence suffix parser. No whole-board SI claims.
#pragma once
#include "imported_artwork.h"
#include <QCryptographicHash>
#include <QFile>
#include <QSet>
#include <QStringList>
#include <limits>

namespace a032 {
inline bool splitWire(const QByteArray &data, QByteArray &artwork, QByteArray &evidence) {
    constexpr auto marker = "\nBOARDSI_CONTACT_RECORDS_V1|";
    const qsizetype pos = data.lastIndexOf(marker);
    if (pos < 0 || pos > 2*1024*1024 || !data.endsWith('\n')) return false;
    artwork = data.left(pos+1);
    evidence = data.mid(pos+1);
    return !artwork.isEmpty() && evidence.size() <= 1024*1024;
}

inline bool decimal(const QString &s, qint64 &value, bool signedValue) {
    if (s.isEmpty() || s.size()>20) return false;
    int first=0;
    if (signedValue && s[0]=='-') first=1;
    if (first==s.size()) return false;
    for(int i=first;i<s.size();++i) if(s[i]<'0' || s[i]>'9') return false;
    bool ok=false;
    value=s.toLongLong(&ok);
    return ok;
}

inline bool encodeEndpoint(const ImportedContactEnd &p,
                           const std::vector<ImportedTrack> &tracks,
                           const std::vector<ImportedArc> &arcs,
                           const std::vector<ImportedVia> &vias,
                           qint64 x, qint64 y, QString &net) {
    if(p.kind=='T') {
        if(p.index<0 || p.index>=static_cast<int>(tracks.size()) ||
           (p.site!=0 && p.site!=1)) return false;
        const auto &t=tracks[static_cast<size_t>(p.index)];
        if(t.layer!=p.layer) return false;
        const qint64 ax=p.site==0?t.x0_nm:t.x1_nm;
        const qint64 ay=p.site==0?t.y0_nm:t.y1_nm;
        if(ax!=x || ay!=y) return false;
        net=t.net; return true;
    }
    if(p.kind=='A') {
        if(p.index<0 || p.index>=static_cast<int>(arcs.size()) ||
           (p.site!=0 && p.site!=1)) return false;
        const auto &a=arcs[static_cast<size_t>(p.index)];
        if(a.layer!=p.layer) return false;
        const qint64 ax=p.site==0?a.x0_nm:a.x1_nm;
        const qint64 ay=p.site==0?a.y0_nm:a.y1_nm;
        if(ax!=x || ay!=y) return false;
        net=a.net; return true;
    }
    if(p.kind=='V') {
        if(p.index<0 || p.index>=static_cast<int>(vias.size()) || p.site!=0)
            return false;
        const auto &v=vias[static_cast<size_t>(p.index)];
        if(p.layer<v.firstLayer || p.layer>v.lastLayer ||
           x!=v.x_nm || y!=v.y_nm) return false;
        // Explicit conductive-pad qualification resides in the validated
        // Rust Canonical graph. Span alone is NOT reclassified as contact here.
        net=v.net; return true;
    }
    return false;
}

inline bool decode(const QByteArray &wire, const QString &sourcePath,
                   const std::vector<ImportedTrack> &tracks,
                   const std::vector<ImportedArc> &arcs,
                   const std::vector<ImportedVia> &vias,
                   const QByteArray &stderrAudit,
                   std::vector<ImportedContactMarker> &markers) {
    if(wire.isEmpty() || wire.size()>1024*1024 || !wire.endsWith('\n') ||
       wire.contains('\r') || wire.contains('\0')) return false;
    const QStringList lines=QString::fromLatin1(wire).split('\n',Qt::SkipEmptyParts);
    if(lines.size()<2 || lines.size()>100002) return false;
    const QStringList header=lines.front().split('|');
    if(header.size()!=4 || header[0]!="BOARDSI_CONTACT_RECORDS_V1" ||
       header[2]!="LOCAL_ENDPOINT_ONLY" || header[3].size()!=64)
        return false;
    for(const QChar ch:header[3]) if(!((ch>='0'&&ch<='9') || (ch>='a'&&ch<='f'))) return false;
    QFile source(sourcePath);
    if(!source.open(QIODevice::ReadOnly) || source.size()>8*1024*1024) return false;
    QCryptographicHash digest(QCryptographicHash::Sha256);
    if(!digest.addData(&source) ||
       QString::fromLatin1(digest.result().toHex())!=header[3]) return false;
    qint64 count=0;
    if(!decimal(header[1],count,false) || count<0 || count>100000 ||
       lines.size()!=count+2) return false;
    const QStringList trailer=lines.back().split('|');
    qint64 tailCount=0;
    if(trailer.size()!=2 || trailer[0]!="BOARDSI_CONTACT_END_V1" ||
       !decimal(trailer[1],tailCount,false) || count!=tailCount) return false;
    const QStringList audit=QString::fromLatin1(stderrAudit).trimmed().split('|');
    qint64 local=0,barrel=0;
    if(audit.size()!=5 || audit[0]!="BOARDSI_CONTACT_AUDIT_V1" ||
       !decimal(audit[1],local,false) || !decimal(audit[2],barrel,false) ||
       local+barrel!=count) return false;

    std::vector<ImportedContactMarker> parsed;
    parsed.reserve(static_cast<size_t>(count));
    QSet<QString> unique;
    qint64 countedLocal=0,countedBarrel=0;
    for(qint64 i=1;i<=count;++i) {
        const QString &row=lines[static_cast<int>(i)];
        if(unique.contains(row)) return false;
        unique.insert(row);
        const QStringList fields=row.split('|');
        if(fields.size()!=12 || fields[0]!="E" ||
           (fields[1]!="S" && fields[1]!="B") ||
           fields[2].size()!=1 || fields[6].size()!=1) return false;
        ImportedContactMarker m;
        m.barrel=fields[1]=="B";
        m.a.kind=fields[2].at(0).toLatin1();
        m.b.kind=fields[6].at(0).toLatin1();
        qint64 aIndex=0,aSite=0,aLayer=0,bIndex=0,bSite=0,bLayer=0;
        if(!decimal(fields[3],aIndex,false) || !decimal(fields[4],aSite,false) ||
           !decimal(fields[5],aLayer,false) || !decimal(fields[7],bIndex,false) ||
           !decimal(fields[8],bSite,false) || !decimal(fields[9],bLayer,false) ||
           aIndex>10000 || bIndex>10000 || aSite>1 || bSite>1 ||
           aLayer<1 || aLayer>64 || bLayer<1 || bLayer>64 ||
           !decimal(fields[10],m.x_nm,true) || !decimal(fields[11],m.y_nm,true) ||
           m.x_nm%254!=0 || m.y_nm%254!=0 ||
           m.x_nm< -1422400000LL || m.x_nm>1422400000LL ||
           m.y_nm< -1422400000LL || m.y_nm>1422400000LL) return false;
        m.a.index=static_cast<int>(aIndex);m.a.site=static_cast<int>(aSite);
        m.a.layer=static_cast<int>(aLayer);
        m.b.index=static_cast<int>(bIndex);m.b.site=static_cast<int>(bSite);
        m.b.layer=static_cast<int>(bLayer);
        QString aNet,bNet;
        if(!encodeEndpoint(m.a,tracks,arcs,vias,m.x_nm,m.y_nm,aNet) ||
           !encodeEndpoint(m.b,tracks,arcs,vias,m.x_nm,m.y_nm,bNet) ||
           aNet.isEmpty() || aNet!=bNet) return false;
        if(m.barrel) {
            if(m.a.kind!='V' || m.b.kind!='V' ||
               m.a.index!=m.b.index || m.a.layer==m.b.layer) return false;
            ++countedBarrel;
        } else {
            if(m.a.layer!=m.b.layer ||
               (m.a.kind=='V' && m.b.kind=='V') ||
               (m.a.kind==m.b.kind && m.a.index==m.b.index)) return false;
            ++countedLocal;
        }
        m.net=aNet;
        parsed.push_back(std::move(m));
    }
    if(countedLocal!=local || countedBarrel!=barrel) return false;
    markers=std::move(parsed);
    return true;
}
} // namespace a032
