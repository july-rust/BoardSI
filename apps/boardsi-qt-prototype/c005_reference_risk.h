// BoardSI C_005: opt-in local Rust source topology risk → Qt.
// This is a geometric warning UI, never electrical Return Path or SI.
#pragma once
#include "imported_artwork.h"
#include <QCryptographicHash>
#include <QFile>
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

class C005ReferenceRiskController final: public QWidget {
    ImportedArtworkPanel *artwork_ = nullptr;
    QString importer_,source_;
    QSpinBox *first_=nullptr,*second_=nullptr,*refA_=nullptr,*refB_=nullptr;
    QLineEdit *net_=nullptr;
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
        return QString("%1\n%2\n%3\n%4\n%5")
            .arg(first_->value()).arg(second_->value()).arg(refA_->value())
            .arg(refB_->value()).arg(net_->text());
    }
    void invalidate() {
        ++generation_;
        deadline_.stop();
        if(process_.state()!=QProcess::NotRunning) {
            process_.kill();process_.waitForFinished(100);
        }
        requestedSha_.clear();requestedOptions_.clear();requestedIndices_.clear();
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
        if(exitStatus!=QProcess::NormalExit || exitCode!=0 ||
           !c005::decode(bytes,requestedSha_,artwork_,requestedIndices_,receipt)) {
            status_->setText("UNSUPPORTED / INVALID OR UNAVAILABLE RUST AUDIT / SI NOT_RUN");
            return;
        }
        artwork_->setReferenceRisks(receipt.markers);
        status_->setText(receipt.state+" / "+receipt.reason+
            " / SOURCE GEOMETRY ONLY / P009 UNSUPPORTED / Solver BLOCKED");
    }
public:
    C005ReferenceRiskController(ImportedArtworkPanel *artwork,
                                const QString &importer,QWidget *parent=nullptr)
        :QWidget(parent),artwork_(artwork),importer_(importer),process_(this),deadline_(this) {
        setObjectName("c005ReferenceRiskPanel");
        auto *layout=new QVBoxLayout(this);
        layout->addWidget(new QLabel("C_005 — Source reference geometry risk ONLY; no electrical Return Path.",this));
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
           second_->value()==first_->value()) {
            status_->setText("MISSING/INVALID AUDIT INPUT / SI NOT_RUN");return;
        }
        const QByteArray sha=c005::hashSource(source_);
        if(sha.size()!=64 || acceptedSha_.size()!=64 || sha!=acceptedSha_) {
            status_->setText("SOURCE CHANGED / REIMPORT REQUIRED / SI NOT_RUN");return;
        }
        requestedIndices_={first_->value()};
        QStringList args{source_,"--reference-risk-v1",
            QString::number(first_->value()),net_->text(),QString::number(refA_->value())};
        if(second_->value()>=0) {
            requestedIndices_.push_back(second_->value());
            args << QString::number(second_->value()) << QString::number(refB_->value());
        }
        ImportedTrack witness;
        for(int idx:requestedIndices_)if(!artwork_->importedTrack(idx,witness)) {
            status_->setText("TRACE INDEX NOT IMPORTED / SI NOT_RUN");return;
        }
        requestedSha_=sha;requestedOptions_=options();activeGeneration_=generation_;
        process_.setProgram(importer_);process_.setArguments(args);process_.start();
        deadline_.start();
        status_->setText("SOURCE AUDIT RUNNING / Solver BLOCKED");
    }
    QString currentStatusForTest() const {return status_->text();}
    bool auditRunningForTest() const {return process_.state()!=QProcess::NotRunning;}
};
