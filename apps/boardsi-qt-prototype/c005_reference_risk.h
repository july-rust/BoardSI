// BoardSI C_005: opt-in local Rust source topology risk → Qt.
// This is a geometric warning UI, never electrical Return Path or SI.
#pragma once
#include "imported_artwork.h"
#include <QCryptographicHash>
#include <QFile>
#include <QCheckBox>
#include <QFileInfo>
#include <QFileSystemWatcher>
#include <QHBoxLayout>
#include <QLabel>
#include <QLineEdit>
#include <QProcess>
#include <QPushButton>
#include <QSpinBox>
#include <QTimer>
#include <QVBoxLayout>
#include <QWidget>
#include <QStringList>
#include <vector>
#include <algorithm>
#include <cmath>

namespace c005 {
struct Receipt {
    QString state, reason;
    std::vector<ImportedReferenceRiskMarker> markers;
};
inline QByteArray hashSource(const QString &path) {
    QFile f(path);
    if (!f.open(QIODevice::ReadOnly) || !QFileInfo(path).isFile() ||
        f.size() < 1 || f.size() > 64 * 1024 * 1024) return {};
    QCryptographicHash sha(QCryptographicHash::Sha256);
    while(!f.atEnd()) {
        QByteArray chunk=f.read(65536);
        if(chunk.isEmpty())return {};
        sha.addData(chunk);
    }
    return sha.result().toHex();
}
inline bool boundedHex(const QByteArray &value, QByteArray &decoded) {
    if(value.isEmpty() || value.size()%2 || value.size()>512) return false;
    for(char ch:value)
        if(!((ch>='0' && ch<='9')||(ch>='a' && ch<='f')))return false;
    decoded=QByteArray::fromHex(value);
    return decoded.size()==value.size()/2;
}
inline bool reasonCode(const QByteArray &s) {
    if(s.isEmpty() || s.size()>96)return false;
    for(char ch:s)if(!((ch>='A' && ch<='Z') || (ch>='0' && ch<='9') || ch=='_'))return false;
    return true;
}
inline bool readCoordinate(const QByteArray &s,qint64 &value) {
    bool ok=false; value=s.toLongLong(&ok);
    return ok && value%254==0 && value>=-1422400000LL && value<=1422400000LL;
}
inline bool decode(const QByteArray &wire,const QByteArray &sourceSha,
                   const ImportedArtworkPanel *panel,const std::vector<int> &selected,
                   Receipt &out) {
    if(!panel || wire.isEmpty() || wire.size()>4096 ||
       !wire.endsWith('\n') || (selected.size()!=1 && selected.size()!=2))return false;
    const auto lines=wire.split('\n');
    if(lines.size()<4 || !lines.last().isEmpty())return false;
    const auto head=lines[0].split('|');
    if(head.size()!=9 || head[0]!="BOARDSI_REFERENCE_RISK_V1" ||
       head[1]!=sourceSha || sourceSha.size()!=64 || head[5]!="0" ||
       head[6]!="UNSUPPORTED" || head[7]!="NOT_RUN" || head[8]!="0")return false;
    const QString state=QString::fromLatin1(head[2]);
    if(state!="REFERENCE_SPLIT_RISK" && state!="LAYER_TRANSITION_RISK" &&
       state!="SOURCE_GEOMETRY_ONLY" && state!="UNSUPPORTED")return false;
    if(!reasonCode(head[3]))return false;
    bool ok=false;
    const int count=head[4].toInt(&ok);
    if(!ok || count!=static_cast<int>(selected.size()))return false;
    std::vector<ImportedTrack> tracks;
    QString firstObject;
    for(int i=0;i<count;++i) {
        const auto row=lines[1+i].split('|');
        if(row.size()!=10 || row[0]!="T")return false;
        int idx=row[1].toInt(&ok);
        if(!ok || idx!=selected[static_cast<size_t>(i)])return false;
        const int layer=row[2].toInt(&ok);
        ImportedTrack track;
        if(!ok || !panel->importedTrack(idx,track) || layer!=track.layer)return false;
        QByteArray netBytes,objBytes,partBytes;
        if(!boundedHex(row[3],netBytes) || !boundedHex(row[4],objBytes) ||
           (row[5]!="-" && !boundedHex(row[5],partBytes)) || QString::fromUtf8(netBytes)!=track.net)
            return false;
        qint64 coords[4]={};
        for(int k=0;k<4;++k)if(!readCoordinate(row[6+k],coords[k]))return false;
        if(coords[0]!=track.x0_nm || coords[1]!=track.y0_nm ||
           coords[2]!=track.x1_nm || coords[3]!=track.y1_nm)return false;
        if(i==0)firstObject=QString::fromUtf8(objBytes);
        tracks.push_back(track);
    }
    const bool risk=(state=="REFERENCE_SPLIT_RISK" || state=="LAYER_TRANSITION_RISK");
    const int endIndex=1+count+(risk?1:0);
    if(lines.size()!=endIndex+2)return false;
    const auto end=lines[endIndex].split('|');
    if(end.size()!=4 || end[0]!="END" || end[1]!=QByteArray::number(count) ||
       end[2]!=(risk?"1":"0") || end[3]!="SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")
        return false;
    Receipt candidate;
    candidate.state=state;
    candidate.reason=QString::fromLatin1(head[3]);
    if(risk) {
        const auto marker=lines[1+count].split('|');
        if(marker.size()!=4 || marker[0]!="M")return false;
        qint64 x=0,y=0;
        if(!readCoordinate(marker[1],x) || !readCoordinate(marker[2],y))return false;
        QByteArray via;
        if(marker[3]!="-" && !boundedHex(marker[3],via))return false;
        const auto &t=tracks[0];
        if(state=="REFERENCE_SPLIT_RISK") {
            if(count!=1 || marker[3]!="-" ||
               x!=((t.x0_nm/254+t.x1_nm/254)/2)*254 ||
               y!=((t.y0_nm/254+t.y1_nm/254)/2)*254)return false;
        } else {
            if(count!=2 || via.isEmpty())return false;
            const auto &s=tracks[1];
            bool matching=false;
            for(const auto a:{QPointF(t.x0_nm,t.y0_nm),QPointF(t.x1_nm,t.y1_nm)})
                for(const auto b:{QPointF(s.x0_nm,s.y0_nm),QPointF(s.x1_nm,s.y1_nm)})
                    if(a==b && a==QPointF(x,y))matching=true;
            if(!matching)return false;
        }
        ImportedReferenceRiskMarker m;
        m.state=state;m.reason=candidate.reason;m.net=t.net;m.layer=t.layer;
        m.x_nm=x;m.y_nm=y;m.sourceObject=firstObject;
        m.viaObject=via.isEmpty()?QStringLiteral("-"):QString::fromUtf8(via);
        candidate.markers.push_back(m);
    }
    out=std::move(candidate);
    return true;
}
} // namespace c005

// C_006 V2: source-authenticated, bounded parametric gap intervals.
// All rational parameters remain authoritative in Rust; doubles are DISPLAY ONLY.
namespace c006 {
inline bool decode(const QByteArray &wire,const QByteArray &sourceSha,
                   const ImportedArtworkPanel *panel,const std::vector<int> &selected,
                   c005::Receipt &out) {
    if(!panel || selected.size()!=1 || wire.isEmpty() || wire.size()>8192 ||
       !wire.endsWith('\n'))return false;
    const auto lines=wire.split('\n');
    if(lines.size()<4 || !lines.last().isEmpty())return false;
    const auto head=lines[0].split('|');
    if(head.size()!=9 || head[0]!="BOARDSI_REFERENCE_GAP_V2" ||
       head[1]!=sourceSha || sourceSha.size()!=64 ||
       head[4]!="1" || head[5]!="0" || head[6]!="UNSUPPORTED" ||
       head[7]!="NOT_RUN" || head[8]!="0" || !c005::reasonCode(head[3]))return false;
    const QString state=QString::fromLatin1(head[2]);
    if(state!="REFERENCE_SPLIT_RISK" && state!="SOURCE_GEOMETRY_ONLY" &&
       state!="UNSUPPORTED")return false;
    const auto row=lines[1].split('|');
    if(row.size()!=10 || row[0]!="T")return false;
    bool ok=false;
    const int traceIndex=row[1].toInt(&ok);
    if(!ok || traceIndex!=selected[0])return false;
    const int layer=row[2].toInt(&ok);
    ImportedTrack track;
    if(!ok || !panel->importedTrack(traceIndex,track) || layer!=track.layer)return false;
    QByteArray netBytes,objectBytes,partBytes;
    if(!c005::boundedHex(row[3],netBytes) ||
       !c005::boundedHex(row[4],objectBytes) ||
       (row[5]!="-" && !c005::boundedHex(row[5],partBytes)) ||
       QString::fromUtf8(netBytes)!=track.net)return false;
    qint64 coords[4]{};
    for(int k=0;k<4;++k)if(!c005::readCoordinate(row[6+k],coords[k]))return false;
    if(coords[0]!=track.x0_nm || coords[1]!=track.y0_nm ||
       coords[2]!=track.x1_nm || coords[3]!=track.y1_nm)return false;
    const int count=lines.size()-4;
    if(count<0 || count>32 || (state=="REFERENCE_SPLIT_RISK")!=(count>0))return false;
    const auto end=lines[2+count].split('|');
    if(end.size()!=4 || end[0]!="END" || end[1]!="1" ||
       end[2]!=QByteArray::number(count) ||
       end[3]!="SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")return false;
    c005::Receipt candidate;
    candidate.state=state;candidate.reason=QString::fromLatin1(head[3]);
    long double previous=0;
    for(int i=0;i<count;++i) {
        const auto interval=lines[i+2].split('|');
        if(interval.size()!=5 || interval[0]!="I")return false;
        qint64 frac[4]{};
        for(int k=0;k<4;++k) {
            frac[k]=interval[k+1].toLongLong(&ok);
            if(!ok)return false;
        }
        if(frac[1]<=0 || frac[3]<=0 ||
           frac[1]>1000000000000000LL || frac[3]>1000000000000000LL ||
           frac[0]<0 || frac[2]<0 || frac[0]>frac[1] || frac[2]>frac[3])return false;
        const long double lo=static_cast<long double>(frac[0])/frac[1];
        const long double hi=static_cast<long double>(frac[2])/frac[3];
        if(!(lo>=previous && hi>lo && hi<=1))return false;
        previous=hi;
        ImportedReferenceRiskMarker m;
        m.state=state;m.reason=candidate.reason;m.net=track.net;m.layer=track.layer;
        m.sourceObject=QString::fromUtf8(objectBytes);m.viaObject="-";
        m.exactGap=true;
        const auto interpolate=[](qint64 start,qint64 finish,long double t)->qreal {
            return static_cast<qreal>(static_cast<long double>(start) +
                (static_cast<long double>(finish)-start)*t);
        };
        m.startX_nm=interpolate(track.x0_nm,track.x1_nm,lo);
        m.startY_nm=interpolate(track.y0_nm,track.y1_nm,lo);
        m.endX_nm=interpolate(track.x0_nm,track.x1_nm,hi);
        m.endY_nm=interpolate(track.y0_nm,track.y1_nm,hi);
        m.x_nm=static_cast<qint64>((m.startX_nm+m.endX_nm)/2);
        m.y_nm=static_cast<qint64>((m.startY_nm+m.endY_nm)/2);
        m.intervalParameters=QString("%1/%2..%3/%4")
            .arg(frac[0]).arg(frac[1]).arg(frac[2]).arg(frac[3]);
        candidate.markers.push_back(m);
    }
    out=std::move(candidate);
    return true;
}
} // namespace c006


// C_007 V3: explicit multi-trace route envelope around unchanged C_006 V2.
// Each segment is independently verified against imported Qt source geometry.
namespace c007 {
inline bool decode(const QByteArray &wire, const QByteArray &sourceSha,
                   const ImportedArtworkPanel *panel, const std::vector<int> &selected,
                   c005::Receipt &out) {
    if(!panel || selected.size()<2 || selected.size()>16 ||
       wire.isEmpty() || wire.size()>140000 || !wire.endsWith('\n'))return false;
    const auto lines=wire.split('\n');
    if(lines.size()<10 || !lines.last().isEmpty())return false;
    const auto head=lines[0].split('|');
    if(head.size()!=7 || head[0]!="BOARDSI_REFERENCE_ROUTE_V3" ||
       head[1]!=sourceSha || sourceSha.size()!=64 ||
       head[4]!="UNSUPPORTED" || head[5]!="NOT_RUN" || head[6]!="0")
        return false;
    bool ok=false;
    const int count=head[2].toInt(&ok);
    if(!ok || count!=static_cast<int>(selected.size()))return false;
    const QString status=QString::fromLatin1(head[3]);
    if(status!="UNSUPPORTED" && status!="SOURCE_GEOMETRY_ONLY" &&
       status!="REFERENCE_SPLIT_RISK")return false;
    c005::Receipt candidate;
    candidate.state=status;
    candidate.reason="SOURCE_ROUTE_REFERENCE_GEOMETRY_NOT_PHYSICAL";
    int line=1, gaps=0;
    bool sawUnsupported=false;
    ImportedTrack previous;
    for(int i=0;i<count;++i) {
        if(line>=lines.size()-1)return false;
        const auto segment=lines[line++].split('|');
        if(segment.size()!=3 || segment[0]!="SEG")return false;
        const int index=segment[1].toInt(&ok);
        if(!ok || index!=selected[static_cast<size_t>(i)])return false;
        const int rows=segment[2].toInt(&ok);
        if(!ok || rows<3 || rows>35 || line+rows>=lines.size())return false;
        QByteArray original;
        for(int r=0;r<rows;++r) {
            original+=lines[line++];
            original+='\n';
        }
        c005::Receipt single;
        if(!c006::decode(original,sourceSha,panel,{index},single))return false;
        ImportedTrack current;
        if(!panel->importedTrack(index,current))return false;
        if(i>0 && (current.net!=previous.net || current.layer!=previous.layer ||
           current.x0_nm!=previous.x1_nm || current.y0_nm!=previous.y1_nm))
            return false;
        previous=current;
        if(single.state=="UNSUPPORTED")sawUnsupported=true;
        gaps+=static_cast<int>(single.markers.size());
        if(gaps>16*32)return false;
        for(auto &m:single.markers) {
            m.reason="SOURCE_ROUTE_REFERENCE_GAP_NOT_PHYSICAL";
            m.intervalParameters=QString("Route segment %1/%2: %3")
                .arg(i+1).arg(count).arg(m.intervalParameters);
            candidate.markers.push_back(std::move(m));
        }
    }
    if(line!=lines.size()-2)return false;
    const auto end=lines[line].split('|');
    if(end.size()!=4 || end[0]!="END" ||
       end[1]!=QByteArray::number(count) || end[2]!=QByteArray::number(gaps) ||
       end[3]!="SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")return false;
    const QString expected=sawUnsupported?"UNSUPPORTED":
        gaps>0?"REFERENCE_SPLIT_RISK":"SOURCE_GEOMETRY_ONLY";
    if(status!=expected)return false;
    if(sawUnsupported)candidate.markers.clear(); // no partial route PASS
    out=std::move(candidate);
    return true;
}
} // namespace c007

// C_008 V4: TWO explicit conductor layers and ONE plated transition witness.
// Independent C_006 V2 segment records are rechecked against accepted artwork.
namespace c008 {
inline bool decode(const QByteArray &wire, const QByteArray &sourceSha,
                   const ImportedArtworkPanel *panel, const std::vector<int> &selected,
                   int split, c005::Receipt &out) {
    if(!panel || selected.size()<2 || selected.size()>16 ||
       split<1 || split>=static_cast<int>(selected.size()) ||
       wire.isEmpty() || wire.size()>150000 || !wire.endsWith('\n'))return false;
    const auto lines=wire.split('\n');
    if(lines.size()<12 || !lines.last().isEmpty())return false;
    const auto head=lines[0].split('|');
    if(head.size()!=9 || head[0]!="BOARDSI_REFERENCE_ROUTE_V4" ||
       sourceSha.size()!=64 || head[1]!=sourceSha ||
       head[4]!="UNSUPPORTED" || head[5]!="NOT_RUN" || head[6]!="0" ||
       head[7]!="REFERENCE_TRANSFER_UNVERIFIED")return false;
    bool ok=false;
    const int count=head[2].toInt(&ok);
    if(!ok || count!=static_cast<int>(selected.size()))return false;
    const int receiptSplit=head[3].toInt(&ok);
    if(!ok || receiptSplit!=split)return false;
    const QString state=QString::fromLatin1(head[8]);
    if(state!="SOURCE_GEOMETRY_ONLY" && state!="REFERENCE_SPLIT_RISK" &&
       state!="UNSUPPORTED")return false;
    const auto vr=lines[1].split('|');
    if(vr.size()!=10 || vr[0]!="VIA")return false;
    const int viaIndex=vr[1].toInt(&ok);
    if(!ok)return false;
    ImportedVia via;
    if(!panel->importedVia(viaIndex,via))return false;
    QByteArray viaName,viaObject;
    if(!c005::boundedHex(vr[2],viaName) ||
       !c005::boundedHex(vr[3],viaObject) ||
       via.name!=QString::fromUtf8(viaName) ||
       !QString::fromUtf8(viaObject).startsWith("via-line-"))return false;
    qint64 x=0,y=0;
    if(!c005::readCoordinate(vr[4],x) || !c005::readCoordinate(vr[5],y) ||
       x!=via.x_nm || y!=via.y_nm)return false;
    const int first=vr[6].toInt(&ok);
    if(!ok || first!=via.firstLayer)return false;
    const int last=vr[7].toInt(&ok);
    if(!ok || last!=via.lastLayer)return false;
    const int layerA=vr[8].toInt(&ok);
    if(!ok || layerA<first || layerA>last)return false;
    const int layerB=vr[9].toInt(&ok);
    if(!ok || layerB<first || layerB>last || layerB==layerA)return false;
    c005::Receipt candidate;
    candidate.state=state;
    candidate.reason="REFERENCE_TRANSFER_UNVERIFIED";
    int line=2, gaps=0;
    bool unsupported=false;
    ImportedTrack previous;
    for(int i=0;i<count;++i) {
        if(line>=lines.size()-1)return false;
        const auto seg=lines[line++].split('|');
        if(seg.size()!=3 || seg[0]!="SEG")return false;
        const int index=seg[1].toInt(&ok);
        if(!ok || index!=selected[static_cast<size_t>(i)])return false;
        const int rows=seg[2].toInt(&ok);
        if(!ok || rows<3 || rows>35 || line+rows>=lines.size())return false;
        QByteArray original;
        for(int j=0;j<rows;++j) { original+=lines[line++];original+='\n'; }
        c005::Receipt single;
        if(!c006::decode(original,sourceSha,panel,{index},single))return false;
        ImportedTrack current;
        if(!panel->importedTrack(index,current) ||
           current.net!=via.net ||
           current.layer!=(i<split?layerA:layerB))return false;
        if(i>0 &&
           (current.x0_nm!=previous.x1_nm || current.y0_nm!=previous.y1_nm))
            return false;
        if(i==split &&
           (previous.x1_nm!=x || previous.y1_nm!=y))return false;
        if(single.state=="UNSUPPORTED")unsupported=true;
        gaps+=static_cast<int>(single.markers.size());
        if(gaps>16*32)return false;
        for(auto &m:single.markers) {
            m.reason="C008_LAYER_REFERENCE_GAP_NOT_PHYSICAL";
            m.intervalParameters=QString("C_008 layer %1 segment %2/%3: %4")
                .arg(current.layer).arg(i+1).arg(count).arg(m.intervalParameters);
            candidate.markers.push_back(std::move(m));
        }
        previous=current;
    }
    if(line!=lines.size()-2)return false;
    const auto end=lines[line].split('|');
    if(end.size()!=5 || end[0]!="END" ||
       end[1]!=QByteArray::number(count) ||
       end[2]!=QByteArray::number(gaps) ||
       end[3]!="1" ||
       end[4]!="SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")return false;
    const QString expected=unsupported?"UNSUPPORTED":
        gaps>0?"REFERENCE_SPLIT_RISK":"SOURCE_GEOMETRY_ONLY";
    if(state!=expected)return false;
    if(unsupported)candidate.markers.clear();
    // No electrically certified layer-to-layer Return Path, even if all gaps = 0.
    ImportedReferenceRiskMarker marker;
    marker.state="LAYER_TRANSITION_RISK";
    marker.reason="REFERENCE_TRANSFER_UNVERIFIED";
    marker.sourceObject=QString::fromUtf8(viaObject);
    marker.viaObject=QString::fromUtf8(viaName);
    marker.net=via.net;marker.layer=layerA;marker.x_nm=x;marker.y_nm=y;
    candidate.markers.push_back(marker);
    out=std::move(candidate);
    return true;
}
} // namespace c008

// C_009: V5 wraps an intact C_008 V4 receipt and two source-bound Copper
// contacts. Qt rechecks source identity and Via geometry; this is not P009.
namespace c009 {
inline bool decode(const QByteArray &wire, const QByteArray &sourceSha,
                   const ImportedArtworkPanel *panel, const std::vector<int> &selected,
                   int split, int referenceViaIndex, const QString &referenceNet,
                   const std::array<int, 2> &referenceLayers, c005::Receipt &out) {
    if (!panel || wire.isEmpty() || wire.size()>170000 || !wire.endsWith('\n'))
        return false;
    const auto lines=wire.split('\n');
    if (lines.size()<17 || !lines.last().isEmpty()) return false;
    const auto head=lines[0].split('|');
    if (head.size()!=14 || head[0]!="BOARDSI_REFERENCE_ROUTE_V5" ||
        head[1]!=sourceSha || sourceSha.size()!=64 ||
        head[2]!="VERIFIED_SOURCE_STITCH_CANDIDATE" ||
        head[3]!="UNSUPPORTED" || head[4]!="NOT_RUN" || head[5]!="0" ||
        head[6]!="REFERENCE_TRANSFER_UNVERIFIED") return false;
    QByteArray decodedNet, sourceObject;
    if (!c005::boundedHex(head[7],decodedNet) ||
        !c005::boundedHex(head[9],sourceObject) ||
        QString::fromUtf8(decodedNet)!=referenceNet ||
        !QString::fromUtf8(sourceObject).startsWith("via-line-")) return false;
    bool ok=false;
    const int viaIndex=head[8].toInt(&ok);
    if(!ok || viaIndex!=referenceViaIndex) return false;
    ImportedVia via;
    if(!panel->importedVia(viaIndex,via) || via.net!=referenceNet ||
       via.firstLayer>referenceLayers[0] || via.lastLayer<referenceLayers[0] ||
       via.firstLayer>referenceLayers[1] || via.lastLayer<referenceLayers[1])
        return false;
    qint64 x=0,y=0;
    if(!c005::readCoordinate(head[10],x) || !c005::readCoordinate(head[11],y) ||
       x!=via.x_nm || y!=via.y_nm) return false;
    for(int i=0;i<2;++i) {
        const auto fields=lines[1+i].split('|');
        if(fields.size()!=3 || fields[0]!="CONTACT" ||
           fields[1]!=QByteArray::number(referenceLayers[static_cast<size_t>(i)]))
            return false;
        QByteArray areaId;
        if(!c005::boundedHex(fields[2],areaId) ||
           !panel->importedCopperSource(QString::fromUtf8(areaId),
               referenceLayers[static_cast<size_t>(i)],referenceNet))
            return false;
    }
    const auto record=lines[3].split('|');
    if(record.size()!=2 || record[0]!="V4") return false;
    const int n=record[1].toInt(&ok);
    if(!ok || n<5 || n>600 || 4+n!=lines.size()-2) return false;
    QByteArray legacy;
    for(int i=0;i<n;++i) {legacy+=lines[4+i];legacy+='\n';}
    if(lines[4+n]!="END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL") return false;
    c005::Receipt verified;
    if(!c008::decode(legacy,sourceSha,panel,selected,split,verified))
        return false;
    const auto signal=lines[5].split('|');
    if(signal.size()!=10 || signal[0]!="VIA") return false;
    qint64 sx=0,sy=0;
    if(!c005::readCoordinate(signal[4],sx) ||
       !c005::readCoordinate(signal[5],sy)) return false;
    const qint64 dx=(sx-x)/254,dy=(sy-y)/254;
    // Both coordinates are source-bounded to ±5.6M ticks; the square fits qint64.
    const qint64 squared=dx*dx+dy*dy;
    if(squared<=0 || head[12]!=QByteArray::number(squared)) return false;
    const double declared=head[13].toDouble(&ok);
    if(!ok || !std::isfinite(declared) ||
       std::abs(declared-std::sqrt(static_cast<double>(squared))*0.000254)>0.000001)
        return false;
    ImportedReferenceRiskMarker marker;
    marker.state="STITCH_CANDIDATE_RISK";
    marker.reason=QString("VERIFIED_SOURCE_STITCH_CANDIDATE / "
        "REFERENCE_TRANSFER_UNVERIFIED / %1 mm (display only)").arg(declared,0,'f',6);
    marker.net=referenceNet;
    marker.layer=referenceLayers[0];
    marker.x_nm=x;marker.y_nm=y;
    marker.sourceObject=QString::fromUtf8(sourceObject);
    marker.viaObject=via.name;
    verified.markers.push_back(marker);
    verified.state="VERIFIED_SOURCE_STITCH_CANDIDATE";
    verified.reason="REFERENCE_TRANSFER_UNVERIFIED / P009 UNSUPPORTED / SI NOT_RUN";
    out=std::move(verified);
    return true;
}
} // namespace c009

// C_010: intact C_009 V5 with two strict source-Copper corridor area identities.
// All markers remain risk/candidate only; no P009 or electrical PASS.
namespace c010 {
inline bool decode(const QByteArray &wire, const QByteArray &sourceSha,
                   const ImportedArtworkPanel *panel, const std::vector<int> &selected,
                   int split, int referenceViaIndex, const QString &referenceNet,
                   const std::array<int,2> &referenceLayers, c005::Receipt &out) {
    if(!panel || wire.size()>185000 || !wire.endsWith('\n') || selected.size()<2)
        return false;
    const auto lines=wire.split('\n');
    if(lines.size()<20 || !lines.last().isEmpty())return false;
    const auto h=lines[0].split('|');
    if(h.size()!=11 || h[0]!="BOARDSI_REFERENCE_ROUTE_V6" ||
       h[1]!=sourceSha || sourceSha.size()!=64 ||
       h[3]!="UNSUPPORTED" || h[4]!="NOT_RUN" || h[5]!="0" ||
       h[6]!="REFERENCE_TRANSFER_UNVERIFIED" || h[7]!="2" ||
       h[8]!=QByteArray::number(static_cast<int>(selected.size())) ||
       h[9]!=QByteArray::number(split) ||
       h[10]!=QByteArray::number(referenceViaIndex))return false;
    const auto wrapper=lines[3].split('|');
    if(wrapper.size()!=2 || wrapper[0]!="V5")return false;
    bool ok=false;
    const int n=wrapper[1].toInt(&ok);
    if(!ok || n<8 || n>600 || 4+n!=lines.size()-2 ||
       lines[4+n]!="END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")return false;
    QByteArray v5;
    for(int j=0;j<n;++j) {v5+=lines[4+j];v5+='\n';}
    c005::Receipt receipt;
    if(!c009::decode(v5,sourceSha,panel,selected,split,referenceViaIndex,
                      referenceNet,referenceLayers,receipt))return false;
    QString firstBad="SAME_SOURCE_AREA_CORRIDOR_CANDIDATE";
    QStringList details;
    for(int i=0;i<2;++i) {
        const auto row=lines[1+i].split('|');
        const auto contact=lines[5+i].split('|');
        if(row.size()!=5 || row[0]!="CORRIDOR" ||
           row[1]!=QByteArray::number(referenceLayers[static_cast<size_t>(i)]) ||
           contact.size()!=3 || contact[0]!="CONTACT" ||
           contact[1]!=row[1] || contact[2]!=row[3])return false;
        QByteArray copper,viaArea;
        if(!c005::boundedHex(row[3],viaArea) ||
           !panel->importedCopperSource(QString::fromUtf8(viaArea),
                    referenceLayers[static_cast<size_t>(i)],referenceNet))return false;
        const QString state=QString::fromLatin1(row[4]);
        const bool absent=row[2]=="-";
        if(!absent && (!c005::boundedHex(row[2],copper) ||
           !panel->importedCopperSource(QString::fromUtf8(copper),
                    referenceLayers[static_cast<size_t>(i)],referenceNet)))return false;
        if((state=="SAME_SOURCE_AREA" && (absent || row[2]!=row[3])) ||
           (state=="REFERENCE_AREA_MISMATCH" && (absent || row[2]==row[3])) ||
           (state=="REFERENCE_CORRIDOR_GAP" && !absent) ||
           (state=="SOURCE_GEOMETRY_UNSUPPORTED" && !absent) ||
           (state!="SAME_SOURCE_AREA" && state!="REFERENCE_AREA_MISMATCH" &&
            state!="REFERENCE_CORRIDOR_GAP" &&
            state!="SOURCE_GEOMETRY_UNSUPPORTED"))return false;
        if(state!="SAME_SOURCE_AREA" && firstBad=="SAME_SOURCE_AREA_CORRIDOR_CANDIDATE")
            firstBad=state;
        details << QString("L%1 corridor=%2 via-contact=%3 %4")
            .arg(referenceLayers[static_cast<size_t>(i)])
            .arg(absent?QString("UNSUPPORTED"):QString::fromUtf8(copper))
            .arg(QString::fromUtf8(viaArea)).arg(state);
    }
    if(h[2]!=firstBad.toLatin1())return false;
    ImportedTrack first;
    if(!panel->importedTrack(selected.front(),first))return false;
    ImportedReferenceRiskMarker m;
    m.state="CORRIDOR_CANDIDATE_RISK";
    m.reason=QString("C_010 %1 / %2 / SOURCE GEOMETRY ONLY / P009 UNSUPPORTED / SI NOT_RUN")
        .arg(firstBad,details.join(" / "));
    m.net=first.net;m.layer=referenceLayers[0];
    m.x_nm=first.x0_nm;m.y_nm=first.y0_nm;
    m.sourceObject=details.join(" / ");m.viaObject=QString::number(referenceViaIndex);
    receipt.markers.push_back(std::move(m));
    receipt.state=firstBad;
    receipt.reason="REFERENCE_TRANSFER_UNVERIFIED / P009 UNSUPPORTED / SI NOT_RUN";
    out=std::move(receipt);
    return true;
}
} // namespace c010

// C_011: exact Source Circular Pad Footprint risk wraps unchanged V6 receipt.
// The GUI independently checks its imported Via geometry and V6 Area identity.
namespace c011 {
inline bool decode(const QByteArray &wire,const QByteArray &sourceSha,
                   const ImportedArtworkPanel *panel,const std::vector<int> &selected,
                   int split,int viaIndex,const QString &referenceNet,
                   const std::array<int,2> &referenceLayers,c005::Receipt &out) {
    if(!panel || wire.isEmpty() || wire.size()>200000 || !wire.endsWith('\n'))return false;
    const auto lines=wire.split('\n');
    if(lines.size()<20 || !lines.last().isEmpty())return false;
    const auto h=lines[0].split('|');
    if(h.size()!=11 || h[0]!="BOARDSI_REFERENCE_ROUTE_V7" ||
       h[1]!=sourceSha || sourceSha.size()!=64 || h[3]!="UNSUPPORTED" ||
       h[4]!="NOT_RUN" || h[5]!="0" ||
       h[6]!="REFERENCE_TRANSFER_UNVERIFIED" || h[7]!="2" ||
       h[8]!=QByteArray::number(static_cast<int>(selected.size())) ||
       h[9]!=QByteArray::number(split) || h[10]!=QByteArray::number(viaIndex))
        return false;
    const auto wrap=lines[3].split('|');
    bool ok=false;
    if(wrap.size()!=2 || wrap[0]!="V6")return false;
    const int count=wrap[1].toInt(&ok);
    if(!ok || count<10 || count>750 || 4+count!=lines.size()-2 ||
       lines[4+count]!="END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")return false;
    QByteArray v6;
    for(int i=0;i<count;++i) {v6+=lines[4+i];v6+='\n';}
    c005::Receipt receipt;
    if(!c010::decode(v6,sourceSha,panel,selected,split,viaIndex,
                     referenceNet,referenceLayers,receipt))return false;
    ImportedVia via;
    if(!panel->importedVia(viaIndex,via) || via.net!=referenceNet ||
       via.diameter_nm<=via.drill_nm || via.drill_nm<=0)return false;
    QString firstBad="PAD_FOOTPRINT_CONTAINED_CANDIDATE";
    QStringList details;
    for(int i=0;i<2;++i) {
        const auto row=lines[1+i].split('|');
        const auto corridor=lines[5+i].split('|');
        if(row.size()!=6 || row[0]!="PAD" ||
           row[1]!=QByteArray::number(referenceLayers[static_cast<size_t>(i)]) ||
           corridor.size()!=5 || corridor[0]!="CORRIDOR" ||
           corridor[1]!=row[1] || corridor[3]!=row[2])return false;
        QByteArray area;
        if(!c005::boundedHex(row[2],area) ||
           !panel->importedCopperSource(QString::fromUtf8(area),
                referenceLayers[static_cast<size_t>(i)],referenceNet))return false;
        const qint64 diameterTicks=row[3].toLongLong(&ok);
        if(!ok || diameterTicks<=0 || diameterTicks>5600000 ||
           diameterTicks*254!=via.diameter_nm)return false;
        const qint64 drillTicks=row[4].toLongLong(&ok);
        if(!ok || drillTicks<=0 || drillTicks>=diameterTicks ||
           drillTicks*254!=via.drill_nm)return false;
        const QString state=QString::fromLatin1(row[5]);
        if(state!="PAD_FOOTPRINT_CONTAINED_CANDIDATE" &&
           state!="PAD_BOUNDARY_RISK" && state!="PAD_CUTOUT_RISK" &&
           state!="V6_CORRIDOR_RISK")return false;
        if((corridor[4]!="SAME_SOURCE_AREA")!=(state=="V6_CORRIDOR_RISK"))
            return false;
        if(state!="PAD_FOOTPRINT_CONTAINED_CANDIDATE" &&
           firstBad=="PAD_FOOTPRINT_CONTAINED_CANDIDATE")firstBad=state;
        details << QString("L%1 area=%2 diameter=%3 mm drill=%4 mm %5")
            .arg(referenceLayers[static_cast<size_t>(i)])
            .arg(QString::fromUtf8(area))
            .arg(via.diameter_nm/1e6,0,'f',6)
            .arg(via.drill_nm/1e6,0,'f',6).arg(state);
    }
    if(h[2]!=firstBad.toLatin1())return false;
    ImportedReferenceRiskMarker m;
    m.state="STITCH_PAD_FOOTPRINT_RISK";
    m.reason=QString("C_011 %1 / %2 / SOURCE GEOMETRY ONLY / P009 UNSUPPORTED / SI NOT_RUN")
        .arg(firstBad,details.join(" / "));
    m.net=referenceNet;
    m.x_nm=via.x_nm;m.y_nm=via.y_nm;
    m.diameter_nm=via.diameter_nm; // Source-checked, viewer-only Pad extent
    m.sourceObject=details.join(" / ");m.viaObject=via.name;
    for(int layer:referenceLayers) {
        m.layer=layer;
        receipt.markers.push_back(m); // One source-sized pad rim per Reference Layer.
    }
    receipt.state=firstBad;
    receipt.reason="REFERENCE_TRANSFER_UNVERIFIED / P009 UNSUPPORTED / SI NOT_RUN";
    out=std::move(receipt);
    return true;
}
} // namespace c011

// C_012: independently validate bounded V8 Source ticks and literal V7 receipt.
namespace c012 {
inline bool decode(const QByteArray &wire,const QByteArray &sourceSha,
                   const ImportedArtworkPanel *panel,const std::vector<int> &selected,
                   int split,int referenceViaIndex,const QString &referenceNet,
                   const std::array<int,2> &layers,c005::Receipt &out) {
    if(!panel || wire.isEmpty() || wire.size()>220000 || !wire.endsWith('\n'))return false;
    const auto lines=wire.split('\n');
    if(lines.size()<25 || !lines.last().isEmpty())return false;
    const auto h=lines[0].split('|');
    if(h.size()!=13 || h[0]!="BOARDSI_REFERENCE_ROUTE_V8" ||
       sourceSha.size()!=64 || h[1]!=sourceSha || h[3]!="UNSUPPORTED" ||
       h[4]!="NOT_RUN" || h[5]!="0" || h[6]!="REFERENCE_TRANSFER_UNVERIFIED" ||
       h[7]!="2" || h[8]!=QByteArray::number(static_cast<int>(selected.size())) ||
       h[9]!=QByteArray::number(split) || h[10]!=QByteArray::number(referenceViaIndex))
        return false;
    bool ok=false;
    const int signalIndex=h[11].toInt(&ok);
    if(!ok || signalIndex<0 || signalIndex==referenceViaIndex)return false;
    const auto wrap=lines[3].split('|');
    if(wrap.size()!=2 || wrap[0]!="V7")return false;
    const int count=wrap[1].toInt(&ok);
    if(!ok || count<15 || count>850 || 4+count!=lines.size()-2 ||
       lines[4+count]!="END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")return false;
    QByteArray v7;
    for(int i=0;i<count;++i){v7+=lines[4+i];v7+='\n';}
    c005::Receipt receipt;
    if(!c011::decode(v7,sourceSha,panel,selected,split,referenceViaIndex,
                     referenceNet,layers,receipt) ||
       h[12]!=receipt.state.toLatin1())return false;
    ImportedVia signal,reference;
    ImportedTrack before,after;
    if(split<=0 || split>=static_cast<int>(selected.size()) ||
       !panel->importedVia(signalIndex,signal) ||
       !panel->importedVia(referenceViaIndex,reference) ||
       !panel->importedTrack(selected[static_cast<size_t>(split-1)],before) ||
       !panel->importedTrack(selected[static_cast<size_t>(split)],after) ||
       before.net!=after.net ||
       signal.x_nm!=before.x1_nm || signal.y_nm!=before.y1_nm ||
       signal.x_nm!=after.x0_nm || signal.y_nm!=after.y0_nm ||
       signal.firstLayer>before.layer || signal.lastLayer<before.layer ||
       signal.firstLayer>after.layer || signal.lastLayer<after.layer ||
       signal.net!=before.net || reference.net!=referenceNet ||
       signal.net.isEmpty() || reference.net.isEmpty() ||
       signal.net==reference.net || signal.diameter_nm<=signal.drill_nm ||
       reference.diameter_nm<=reference.drill_nm)return false;
    QString overall=receipt.state=="PAD_FOOTPRINT_CONTAINED_CANDIDATE"
        ?"SOURCE_PAD_SEPARATE_OBSERVED":"V7_REFERENCE_FOOTPRINT_RISK";
    QStringList descriptions;
    std::vector<ImportedReferenceRiskMarker> additions;
    for(int i=0;i<2;++i){
        const auto row=lines[1+i].split('|');
        const int layer=layers[static_cast<size_t>(i)];
        if(row.size()!=13 || row[0]!="CLEARANCE" ||
           row[1]!=QByteArray::number(layer) ||
           row[2]!=QByteArray::number(signalIndex) ||
           row[3]!=QByteArray::number(referenceViaIndex) ||
           layer<1 || signal.firstLayer>layer || signal.lastLayer<layer ||
           reference.firstLayer>layer || reference.lastLayer<layer)return false;
        qint64 t[8]={};
        for(int j=0;j<8;++j) {
            t[j]=row[4+j].toLongLong(&ok);
            if(!ok)return false;
            if(j<4 && (t[j]<-5600000 || t[j]>5600000))return false;
            if(j>=4 && (t[j]<=0 || t[j]>5600000))return false;
        }
        if(t[0]*254!=signal.x_nm || t[1]*254!=signal.y_nm ||
           t[2]*254!=reference.x_nm || t[3]*254!=reference.y_nm ||
           t[4]*254!=signal.diameter_nm || t[5]*254!=reference.diameter_nm ||
           t[6]*254!=signal.drill_nm || t[7]*254!=reference.drill_nm ||
           t[4]<=t[6] || t[5]<=t[7])return false;
        const qint64 dx=t[0]-t[2],dy=t[1]-t[3];
        // All source coordinates are bounded to +/-5,600,000 ticks;
        // the following signed 64-bit products are bounded below 1.1e15.
        const qint64 fourDist=4*(dx*dx+dy*dy);
        const qint64 sum=t[4]+t[5];
        const qint64 sumSquare=sum*sum;
        const QByteArray state=fourDist<sumSquare?"SOURCE_PAD_OVERLAP_RISK"
            :fourDist==sumSquare?"SOURCE_PAD_TANGENT_RISK"
            :"SOURCE_PAD_SEPARATE_OBSERVED";
        if(row[12]!=state)return false;
        if(overall=="SOURCE_PAD_SEPARATE_OBSERVED" &&
           state!="SOURCE_PAD_SEPARATE_OBSERVED")overall=QString::fromLatin1(state);
        const qreal approximateMm=
            (std::hypot(static_cast<qreal>(dx),static_cast<qreal>(dy))
              -static_cast<qreal>(sum)/2.0)*0.000254;
        const QString details=QString("L%1 SignalVia=%2 %3 %4 mm / StitchVia=%5 %6 %7 mm / approximate signed edge gap=%8 mm / %9")
            .arg(layer).arg(signalIndex).arg(signal.net)
            .arg(signal.diameter_nm/1e6,0,'f',6)
            .arg(referenceViaIndex).arg(reference.net)
            .arg(reference.diameter_nm/1e6,0,'f',6)
            .arg(approximateMm,0,'f',6).arg(QString::fromLatin1(state));
        descriptions << details;
        for(const auto &via:{signal,reference}){
            ImportedReferenceRiskMarker m;
            m.state="VIA_PAD_CLEARANCE_RISK";
            m.reason=QString("C_012 %1 / %2 / SOURCE GEOMETRY ONLY / P009 UNSUPPORTED / SI NOT_RUN")
                .arg(QString::fromLatin1(state),details);
            m.net=via.net;m.viaObject=via.name;m.sourceObject=details;
            m.layer=layer;m.x_nm=via.x_nm;m.y_nm=via.y_nm;
            m.diameter_nm=via.diameter_nm;
            additions.push_back(std::move(m));
        }
    }
    if(h[2]!=overall.toLatin1())return false;
    receipt.markers.insert(receipt.markers.end(),additions.begin(),additions.end());
    receipt.state=overall;
    receipt.reason="C_012 SOURCE VIA PAD GEOMETRY ONLY / P009 UNSUPPORTED / SI NOT_RUN";
    out=std::move(receipt);
    return true;
}
} // namespace c012

// C_013: validate byte-identical V8, then cross-check Source COPCUT rectangle
// against the actual imported Qt Copper Area. Source geometry, never DRC.
namespace c013 {
inline bool rectangle(const std::vector<std::pair<qint64,qint64>> &ring,
                      qint64 xmin,qint64 ymin,qint64 xmax,qint64 ymax) {
    if(ring.size()!=5 || ring.front()!=ring.back() ||
       xmin>=xmax || ymin>=ymax)return false;
    const std::array<std::pair<qint64,qint64>,4> corners{{
        {xmin,ymin},{xmax,ymin},{xmax,ymax},{xmin,ymax}}};
    for(const auto &corner:corners) {
        int count=0;
        for(size_t k=0;k<4;++k)if(ring[k]==corner)++count;
        if(count!=1)return false;
    }
    return true;
}
inline int severity(const QString &s) {
    if(s=="SOURCE_REFERENCE_COPPER_OVERLAP_RISK")return 3;
    if(s=="SOURCE_CUTOUT_TANGENCY_RISK")return 2;
    if(s=="SOURCE_CUTOUT_MISSING_RISK")return 1;
    return s=="SOURCE_CUTOUT_GAP_OBSERVED"?0:-1;
}
inline bool decode(const QByteArray &wire,const QByteArray &sha,
                   const ImportedArtworkPanel *panel,const std::vector<int> &selected,
                   int split,int referenceViaIndex,const QString &referenceNet,
                   const std::array<int,2> &layers,c005::Receipt &out) {
    if(!panel || wire.isEmpty() || wire.size()>250000 || !wire.endsWith('\n'))return false;
    const auto lines=wire.split('\n');
    if(lines.size()<27 || !lines.last().isEmpty())return false;
    const auto h=lines[0].split('|');
    if(h.size()!=12 || h[0]!="BOARDSI_REFERENCE_ROUTE_V9" ||
       sha.size()!=64 || h[1]!=sha || h[3]!="UNSUPPORTED" || h[4]!="NOT_RUN" ||
       h[5]!="0" || h[6]!="REFERENCE_TRANSFER_UNVERIFIED" || h[7]!="2" ||
       h[8]!=QByteArray::number(static_cast<int>(selected.size())) ||
       h[9]!=QByteArray::number(split) ||
       h[10]!=QByteArray::number(referenceViaIndex))return false;
    bool ok=false;
    const int signalIndex=h[11].toInt(&ok);
    if(!ok || signalIndex<0 || signalIndex==referenceViaIndex)return false;
    const auto wrapped=lines[3].split('|');
    if(wrapped.size()!=2 || wrapped[0]!="V8")return false;
    const int count=wrapped[1].toInt(&ok);
    if(!ok || count<17 || count>900 || 4+count!=lines.size()-2 ||
       lines[4+count]!="END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")return false;
    QByteArray v8;
    for(int i=0;i<count;++i){v8+=lines[4+i];v8+='\n';}
    c005::Receipt receipt;
    if(!c012::decode(v8,sha,panel,selected,split,referenceViaIndex,
                     referenceNet,layers,receipt))return false;
    ImportedVia signal;
    if(!panel->importedVia(signalIndex,signal) || signal.net.isEmpty() ||
       signal.net==referenceNet || signal.diameter_nm<=signal.drill_nm)return false;
    QString overall="SOURCE_CUTOUT_GAP_OBSERVED";
    QStringList descriptions;
    std::vector<ImportedReferenceRiskMarker> markers;
    for(int i=0;i<2;++i) {
        const int layer=layers[static_cast<size_t>(i)];
        const auto row=lines[1+i].split('|');
        if(row.size()!=12 || row[0]!="CUTOUT" ||
           row[1]!=QByteArray::number(layer) ||
           row[2]!=QByteArray::number(signalIndex))return false;
        QByteArray sourceId;
        if(!c005::boundedHex(row[3],sourceId))return false;
        ImportedCopperArea area;
        if(!panel->importedCopperGeometry(QString::fromUtf8(sourceId),
                                           layer,referenceNet,area) ||
           area.circle || area.curved || !area.roundCutouts.empty() ||
           area.cutouts.size()>1)return false;
        qint64 x=row[4].toLongLong(&ok);
        if(!ok || x*254!=signal.x_nm)return false;
        qint64 y=row[5].toLongLong(&ok);
        if(!ok || y*254!=signal.y_nm)return false;
        qint64 diameter=row[6].toLongLong(&ok);
        if(!ok || diameter<=0 || diameter*254!=signal.diameter_nm)return false;
        // The rectangular Source outer is independently bounded as well.
        if(area.ring.size()!=5)return false;
        qint64 ominx=area.ring[0].first,ominy=area.ring[0].second;
        qint64 omaxx=ominx,omaxy=ominy;
        for(const auto &p:area.ring) {
            ominx=std::min(ominx,p.first);ominy=std::min(ominy,p.second);
            omaxx=std::max(omaxx,p.first);omaxy=std::max(omaxy,p.second);
        }
        if(!rectangle(area.ring,ominx,ominy,omaxx,omaxy) ||
           2*(signal.x_nm-ominx)<=signal.diameter_nm ||
           2*(omaxx-signal.x_nm)<=signal.diameter_nm ||
           2*(signal.y_nm-ominy)<=signal.diameter_nm ||
           2*(omaxy-signal.y_nm)<=signal.diameter_nm)return false;
        QString state;
        qreal approximateGapMm=0;
        if(row[7]=="-" && row[8]=="-" && row[9]=="-" && row[10]=="-") {
            if(!area.cutouts.empty())return false;
            state="SOURCE_CUTOUT_MISSING_RISK";
        } else {
            qint64 b[4]{};
            for(int j=0;j<4;++j) {
                b[j]=row[7+j].toLongLong(&ok);
                if(!ok || b[j]<-5600000 || b[j]>5600000)return false;
            }
            if(area.cutouts.size()!=1 ||
               !rectangle(area.cutouts[0],b[0]*254,b[1]*254,b[2]*254,b[3]*254))
                return false;
            const qint64 distances[4]{x-b[0],b[2]-x,y-b[1],b[3]-y};
            const qint64 minMargin=*std::min_element(std::begin(distances),
                                                     std::end(distances));
            // Coordinates and diameter are Source-bounded; all products
            // remain within 64-bit. Only exact integer comparisons classify.
            state=minMargin<=0 ||
                2*minMargin<diameter?"SOURCE_REFERENCE_COPPER_OVERLAP_RISK":
                2*minMargin==diameter?"SOURCE_CUTOUT_TANGENCY_RISK":
                "SOURCE_CUTOUT_GAP_OBSERVED";
            approximateGapMm=(static_cast<qreal>(minMargin)-
                               static_cast<qreal>(diameter)/2.0)*0.000254;
        }
        if(row[11]!=state.toLatin1())return false;
        if(severity(state)<0)return false;
        if(severity(state)>severity(overall))overall=state;
        const QString details=QString("L%1 SignalVia=%2 %3 / SourceArea=%4 / Pad=%5 mm / approximate Cutout edge gap=%6 mm / %7")
            .arg(layer).arg(signalIndex).arg(signal.net)
            .arg(QString::fromUtf8(sourceId))
            .arg(signal.diameter_nm/1e6,0,'f',6)
            .arg(approximateGapMm,0,'f',6).arg(state);
        descriptions<<details;
        ImportedReferenceRiskMarker m;
        m.state="SIGNAL_CUTOUT_CLEARANCE_RISK";
        m.reason="C_013 "+details+
            " / SOURCE GEOMETRY ONLY / P009 UNSUPPORTED / SI NOT_RUN";
        m.net=signal.net;m.viaObject=signal.name;m.sourceObject=details;
        m.layer=layer;m.x_nm=signal.x_nm;m.y_nm=signal.y_nm;
        m.diameter_nm=signal.diameter_nm;
        markers.push_back(std::move(m));
    }
    if(h[2]!=overall.toLatin1())return false;
    receipt.markers.insert(receipt.markers.end(),markers.begin(),markers.end());
    receipt.state=overall;
    receipt.reason="C_013 SOURCE COPCUT GEOMETRY ONLY / P009 UNSUPPORTED / SI NOT_RUN";
    out=std::move(receipt);
    return true;
}
} // namespace c013

class C005ReferenceRiskController final: public QWidget {
    ImportedArtworkPanel *artwork_ = nullptr;
    QString importer_,source_;
    QSpinBox *first_=nullptr,*second_=nullptr,*refA_=nullptr,*refB_=nullptr;
    QLineEdit *net_=nullptr,*route_=nullptr,*netB_=nullptr;
    QCheckBox *viaMode_=nullptr,*stitchMode_=nullptr,*corridorMode_=nullptr,*padMode_=nullptr,*clearanceMode_=nullptr,*cutoutMode_=nullptr;
    QSpinBox *stitchVia_=nullptr;
    QSpinBox *split_=nullptr;
    bool requestedV4_=false;
    bool requestedV5_=false;
    bool requestedV6_=false;
    bool requestedV7_=false;
    bool requestedV8_=false;
    bool requestedV9_=false;
    QLabel *status_=nullptr;
    QPushButton *run_=nullptr;
    QProcess process_;
    QTimer deadline_;
    QFileSystemWatcher sourceWatcher_;
    QByteArray requestedSha_, acceptedSha_;
    QString requestedOptions_;
    std::vector<int> requestedIndices_;
    quint64 generation_=0;
    quint64 activeGeneration_=0;
    QString options() const {
        return QStringList{
            QString::number(first_->value()),QString::number(second_->value()),
            QString::number(refA_->value()),QString::number(refB_->value()),
            net_->text(),route_->text(),netB_->text(),
            QString::number(split_->value()),
            QString::number(viaMode_->isChecked()),
            QString::number(stitchMode_->isChecked()),
            QString::number(stitchVia_->value()),
             QString::number(corridorMode_->isChecked()),
             QString::number(padMode_->isChecked()),
             QString::number(clearanceMode_->isChecked()),
             QString::number(cutoutMode_->isChecked())
        }.join('\n');
    }
    void invalidate() {
        ++generation_;
        deadline_.stop();
        if(process_.state()!=QProcess::NotRunning) {
            process_.kill();process_.waitForFinished(100);
        }
        requestedSha_.clear();requestedOptions_.clear();requestedIndices_.clear();
        requestedV4_=false;
        requestedV5_=false;
        requestedV6_=false;
        requestedV7_=false;
        requestedV8_=false;
        requestedV9_=false;
        artwork_->setReferenceRisks({});
        status_->setText("NOT_AUDITED / SI NOT_RUN / Solver BLOCKED");
    }
    void finished(int exitCode,QProcess::ExitStatus exitStatus) {
        deadline_.stop();
        const QByteArray bytes=process_.readAllStandardOutput();
        if(activeGeneration_!=generation_ || requestedSha_.isEmpty() ||
           options()!=requestedOptions_ || c005::hashSource(source_)!=requestedSha_ ||
           acceptedSha_!=requestedSha_) {
            status_->setText("STALE INPUT / RE-AUDIT REQUIRED / SI NOT_RUN");
            return;
        }
        c005::Receipt receipt;
        const bool parsed=requestedV9_
            ? c013::decode(bytes,requestedSha_,artwork_,requestedIndices_,
                 split_->value(),stitchVia_->value(),net_->text(),
                 {refA_->value(),refB_->value()},receipt)
            : requestedV8_
            ? c012::decode(bytes,requestedSha_,artwork_,requestedIndices_,
                 split_->value(),stitchVia_->value(),net_->text(),
                 {refA_->value(),refB_->value()},receipt)
            : requestedV7_
            ? c011::decode(bytes,requestedSha_,artwork_,requestedIndices_,
                 split_->value(),stitchVia_->value(),net_->text(),
                 {refA_->value(),refB_->value()},receipt)
            : requestedV6_
            ? c010::decode(bytes,requestedSha_,artwork_,requestedIndices_,
                 split_->value(),stitchVia_->value(),net_->text(),
                 {refA_->value(),refB_->value()},receipt)
            : requestedV5_
            ? c009::decode(bytes,requestedSha_,artwork_,requestedIndices_,
                split_->value(),stitchVia_->value(),net_->text(),
                {refA_->value(),refB_->value()},receipt)
            : requestedV4_
            ? c008::decode(bytes,requestedSha_,artwork_,requestedIndices_,split_->value(),receipt)
            : !route_->text().trimmed().isEmpty()
            ? c007::decode(bytes,requestedSha_,artwork_,requestedIndices_,receipt)
            : requestedIndices_.size()==1
                ? c006::decode(bytes,requestedSha_,artwork_,requestedIndices_,receipt)
                : c005::decode(bytes,requestedSha_,artwork_,requestedIndices_,receipt);
        if(exitStatus!=QProcess::NormalExit || exitCode!=0 || !parsed) {
            status_->setText("UNSUPPORTED / INVALID OR UNAVAILABLE RUST AUDIT / SI NOT_RUN");
            return;
        }
        artwork_->setReferenceRisks(receipt.markers);
        const QString routeStats=route_->text().trimmed().isEmpty()?QString():
            QString(" / %1 route segments / %2 risk markers (source/display only)")
                .arg(requestedIndices_.size()).arg(receipt.markers.size());
        status_->setText(receipt.state+" / "+receipt.reason+routeStats+
            " / SOURCE GEOMETRY ONLY / P009 UNSUPPORTED / Solver BLOCKED");
    }
public:
    C005ReferenceRiskController(ImportedArtworkPanel *artwork,
                                const QString &importer,QWidget *parent=nullptr)
        :QWidget(parent),artwork_(artwork),importer_(importer),process_(this),deadline_(this) {
        setObjectName("c005ReferenceRiskPanel");
        auto *layout=new QVBoxLayout(this);
        layout->addWidget(new QLabel("C_010 — opt-in Reference Corridor to Stitch Area (V1-V5 preserved); NEVER electrical Return Path.",this));
        auto *controls=new QHBoxLayout;
        first_=new QSpinBox(this);first_->setRange(0,9999);first_->setObjectName("c005TraceIndex");
        second_=new QSpinBox(this);second_->setRange(-1,9999);second_->setValue(-1);
        second_->setSpecialValueText("None");second_->setObjectName("c005SecondTraceIndex");
        refA_=new QSpinBox(this);refA_->setRange(1,64);refA_->setValue(2);
        refA_->setObjectName("c005RefLayer");
        refB_=new QSpinBox(this);refB_->setRange(1,64);refB_->setValue(2);
        net_=new QLineEdit(this);net_->setPlaceholderText("Exact Reference Net");
        net_->setObjectName("c005ReferenceNet");
        for(QWidget *w:{static_cast<QWidget*>(first_),static_cast<QWidget*>(second_),
                       static_cast<QWidget*>(refA_),static_cast<QWidget*>(refB_),
                       static_cast<QWidget*>(net_)})
            controls->addWidget(w);
        layout->addLayout(controls);
        layout->addWidget(new QLabel("Trace # (0-based), optional 2nd Trace, Ref Layer 1/2, Reference Net",this));
        route_=new QLineEdit(this);
        route_->setObjectName("c007RouteIndices");
        route_->setPlaceholderText("C_007 route (optional): 0,1,2 — 2..16 source-order traces");
        layout->addWidget(route_);
        viaMode_=new QCheckBox("C_008 two-layer via transition V4",this);
        viaMode_->setObjectName("c008ViaMode");
        split_=new QSpinBox(this);split_->setRange(1,15);split_->setValue(1);
        split_->setObjectName("c008RouteSplit");
        netB_=new QLineEdit(this);netB_->setObjectName("c008ReferenceNetB");
        netB_->setPlaceholderText("Exact Reference Net, Layer B");
        auto *viaControls=new QHBoxLayout;
        viaControls->addWidget(viaMode_);
        viaControls->addWidget(new QLabel("First-layer trace count",this));
        viaControls->addWidget(split_);
        viaControls->addWidget(netB_);
        layout->addLayout(viaControls);
        stitchMode_=new QCheckBox("C_009 Reference Stitch Via V5",this);
        stitchMode_->setObjectName("c009StitchMode");
        stitchVia_=new QSpinBox(this);
        stitchVia_->setRange(-1,9999);
        stitchVia_->setValue(-1);
        stitchVia_->setSpecialValueText("No candidate");
        stitchVia_->setObjectName("c009StitchViaIndex");
        auto *stitchControls=new QHBoxLayout;
        stitchControls->addWidget(stitchMode_);
        stitchControls->addWidget(new QLabel("Source Reference Via index",this));
        stitchControls->addWidget(stitchVia_);
        layout->addLayout(stitchControls);
        corridorMode_=new QCheckBox("C_010 Source Copper Corridor V6 (not electrical Return Path)",this);
        corridorMode_->setObjectName("c010CorridorMode");
        layout->addWidget(corridorMode_);
        padMode_=new QCheckBox("C_011 Source Stitch Pad Footprint V7 (geometric risk only)",this);
        padMode_->setObjectName("c011PadMode");
        layout->addWidget(padMode_);
        clearanceMode_=new QCheckBox("C_012 Signal-Stitch Via Pad-to-Pad V8 (geometry only)",this);
        clearanceMode_->setObjectName("c012ClearanceMode");
        layout->addWidget(clearanceMode_);
        cutoutMode_=new QCheckBox("C_013 Signal Via Pad / Reference COPCUT V9 (Source geometry only)",this);
        cutoutMode_->setObjectName("c013CutoutMode");
        layout->addWidget(cutoutMode_);

        run_=new QPushButton("Audit selected source trace(s)",this);
        run_->setObjectName("c005RunAudit");
        layout->addWidget(run_);
        status_=new QLabel("NOT_AUDITED / SI NOT_RUN / Solver BLOCKED",this);
        status_->setObjectName("c005RiskStatus");status_->setWordWrap(true);
        layout->addWidget(status_);
        connect(run_,&QPushButton::clicked,this,[this] {audit();});
        for(auto *s:{first_,second_,refA_,refB_})
            connect(s,QOverload<int>::of(&QSpinBox::valueChanged),this,[this](int){invalidate();});
        connect(net_,&QLineEdit::textChanged,this,[this](const QString &){invalidate();});
        connect(route_,&QLineEdit::textChanged,this,[this](const QString &){invalidate();});
        connect(netB_,&QLineEdit::textChanged,this,[this](const QString &){invalidate();});
        connect(split_,QOverload<int>::of(&QSpinBox::valueChanged),this,[this](int){invalidate();});
        connect(viaMode_,&QCheckBox::toggled,this,[this](bool){invalidate();});
        connect(stitchMode_,&QCheckBox::toggled,this,[this](bool){invalidate();});
        connect(corridorMode_,&QCheckBox::toggled,this,[this](bool){invalidate();});
        connect(padMode_,&QCheckBox::toggled,this,[this](bool){invalidate();});
        connect(clearanceMode_,&QCheckBox::toggled,this,[this](bool){invalidate();});
        connect(cutoutMode_,&QCheckBox::toggled,this,[this](bool){invalidate();});
        connect(stitchVia_,QOverload<int>::of(&QSpinBox::valueChanged),
                this,[this](int){invalidate();});
        connect(&process_,&QProcess::finished,this,
            [this](int code,QProcess::ExitStatus state){finished(code,state);});
        // A previously accepted audit cannot remain visible when source copper
        // bytes are externally edited after the worker has completed.
        // Watcher is event-driven; do not hash large PCBs on the GUI timer.
        connect(&sourceWatcher_,&QFileSystemWatcher::fileChanged,this,
                [this](const QString &path) {
            if(path!=source_ || acceptedSha_.isEmpty())return;
            invalidate();
            acceptedSha_.clear();
            status_->setText("SOURCE FILE CHANGED / REIMPORT REQUIRED / SI NOT_RUN");
        });
        deadline_.setSingleShot(true);deadline_.setInterval(5000);
        connect(&deadline_,&QTimer::timeout,this,[this] {
            invalidate();status_->setText("AUDIT TIMEOUT / SI NOT_RUN");
        });
    }
    void resetSource(const QString &path,const QByteArray &acceptedSha) {
        invalidate();
        if(!sourceWatcher_.files().isEmpty())
            sourceWatcher_.removePaths(sourceWatcher_.files());
        source_=acceptedSha.size()==64?path:QString();
        acceptedSha_=acceptedSha;
        if(!source_.isEmpty())sourceWatcher_.addPath(source_);
    }
    void audit() {
        invalidate();
        if(source_.isEmpty() || importer_.isEmpty() ||
           !QFileInfo(importer_).isExecutable() ||
           net_->text().isEmpty() || net_->text().size()>64 ||
           (route_->text().trimmed().isEmpty() && second_->value()==first_->value())) {
            status_->setText("MISSING/INVALID AUDIT INPUT / SI NOT_RUN");return;
        }
        const QByteArray sha=c005::hashSource(source_);
        if(sha.size()!=64 || acceptedSha_.size()!=64 || sha!=acceptedSha_) {
            status_->setText("SOURCE CHANGED / REIMPORT REQUIRED / SI NOT_RUN");return;
        }
        requestedIndices_={first_->value()};
        QStringList args{source_,"--reference-risk-v1",
            QString::number(first_->value()),net_->text(),QString::number(refA_->value())};
        const QString routeText=route_->text().trimmed();
        if(cutoutMode_->isChecked() && !clearanceMode_->isChecked()) {
            status_->setText("C_013 REQUIRES C_012 V8 / SI NOT_RUN");return;
        }
        if(clearanceMode_->isChecked() && !padMode_->isChecked()) {
            status_->setText("C_012 REQUIRES C_011 V7 / SI NOT_RUN");return;
        }
        if(padMode_->isChecked() && !corridorMode_->isChecked()) {
            status_->setText("C_011 REQUIRES C_010 V6 / SI NOT_RUN");return;
        }
        if(corridorMode_->isChecked() && !stitchMode_->isChecked()) {
            status_->setText("C_010 REQUIRES C_009 V5 / SI NOT_RUN");return;
        }
        if(stitchMode_->isChecked() &&
            (!viaMode_->isChecked() || stitchVia_->value()<0 ||
             net_->text()!=netB_->text() || refA_->value()==refB_->value())) {
            status_->setText("INVALID C_009 REFERENCE VIA CANDIDATE / SI NOT_RUN");
            return;
        }
        if(viaMode_->isChecked() &&
           (routeText.isEmpty() || netB_->text().isEmpty() ||
            netB_->text().size()>64)) {
            status_->setText("INVALID C_008 VIA REFERENCE INPUT / SI NOT_RUN");return;
        }
        if(!routeText.isEmpty()) {
            const auto parts=routeText.split(',');
            if(parts.size()<2 || parts.size()>16 || routeText.size()>95) {
                status_->setText("INVALID C_007 ROUTE SELECTION / SI NOT_RUN");return;
            }
            requestedIndices_.clear();
            for(const QString &part:parts) {
                bool ok=false;
                const int idx=part.toInt(&ok);
                if(!ok || idx<0 || idx>9999 ||
                   std::find(requestedIndices_.begin(),requestedIndices_.end(),idx)!=requestedIndices_.end()) {
                    status_->setText("INVALID OR DUPLICATE C_007 TRACE / SI NOT_RUN");return;
                }
                requestedIndices_.push_back(idx);
            }
            QStringList canonical;
            for(int idx:requestedIndices_)canonical << QString::number(idx);
            if(viaMode_->isChecked()) {
                if(split_->value()>=static_cast<int>(requestedIndices_.size())) {
                    status_->setText("INVALID C_008 VIA SPLIT / SI NOT_RUN");return;
                }
                args={source_,cutoutMode_->isChecked()?"--reference-route-v9":clearanceMode_->isChecked()?"--reference-route-v8":padMode_->isChecked()?"--reference-route-v7":corridorMode_->isChecked()?"--reference-route-v6":stitchMode_->isChecked()?"--reference-route-v5":"--reference-route-v4",canonical.join(','),
                      QString::number(split_->value()),net_->text(),
                      QString::number(refA_->value()),netB_->text(),
                      QString::number(refB_->value())};
                if(stitchMode_->isChecked()) args << QString::number(stitchVia_->value());
            } else {
                args={source_,"--reference-route-v3",canonical.join(','),net_->text(),
                      QString::number(refA_->value())};
            }
        } else {
            if(second_->value()<0)args[1]="--reference-gap-v2";
            if(second_->value()>=0) {
                requestedIndices_.push_back(second_->value());
                args << QString::number(second_->value()) << QString::number(refB_->value());
            }
        }
        ImportedTrack witness;
        for(int idx:requestedIndices_)if(!artwork_->importedTrack(idx,witness)) {
            status_->setText("TRACE INDEX NOT IMPORTED / SI NOT_RUN");return;
        }
        requestedV4_=viaMode_->isChecked();
        requestedV5_=stitchMode_->isChecked();
        requestedV6_=corridorMode_->isChecked();
        requestedV7_=padMode_->isChecked();
        requestedV8_=clearanceMode_->isChecked();
        requestedV9_=cutoutMode_->isChecked();
        requestedSha_=sha;requestedOptions_=options();activeGeneration_=generation_;
        process_.setProgram(importer_);process_.setArguments(args);process_.start();
        deadline_.start();
        status_->setText("SOURCE AUDIT RUNNING / Solver BLOCKED");
    }
    QString currentStatusForTest() const {return status_->text();}
    bool auditRunningForTest() const {return process_.state()!=QProcess::NotRunning;}
};
