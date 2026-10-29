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

class C005ReferenceRiskController final: public QWidget {
    ImportedArtworkPanel *artwork_ = nullptr;
    QString importer_,source_;
    QSpinBox *first_=nullptr,*second_=nullptr,*refA_=nullptr,*refB_=nullptr;
    QLineEdit *net_=nullptr,*route_=nullptr,*netB_=nullptr;
    QCheckBox *viaMode_=nullptr;
    QSpinBox *split_=nullptr;
    bool requestedV4_=false;
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
        return QString("%1\n%2\n%3\n%4\n%5\n%6\n%7\n%8\n%9")
            .arg(first_->value()).arg(second_->value()).arg(refA_->value())
            .arg(refB_->value()).arg(net_->text()).arg(route_->text())
            .arg(netB_->text()).arg(split_->value()).arg(viaMode_->isChecked());
    }
    void invalidate() {
        ++generation_;
        deadline_.stop();
        if(process_.state()!=QProcess::NotRunning) {
            process_.kill();process_.waitForFinished(100);
        }
        requestedSha_.clear();requestedOptions_.clear();requestedIndices_.clear();
        requestedV4_=false;
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
        const bool parsed=requestedV4_
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
        layout->addWidget(new QLabel("C_008 — opt-in two-layer / plated Via source Reference risk (V1-V3 preserved); NEVER electrical Return Path.",this));
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
                args={source_,"--reference-route-v4",canonical.join(','),
                      QString::number(split_->value()),net_->text(),
                      QString::number(refA_->value()),netB_->text(),
                      QString::number(refB_->value())};
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
        requestedSha_=sha;requestedOptions_=options();activeGeneration_=generation_;
        process_.setProgram(importer_);process_.setArguments(args);process_.start();
        deadline_.start();
        status_->setText("SOURCE AUDIT RUNNING / Solver BLOCKED");
    }
    QString currentStatusForTest() const {return status_->text();}
    bool auditRunningForTest() const {return process_.state()!=QProcess::NotRunning;}
};
