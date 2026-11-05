// D_039: independent Qt readback of bounded SMD Pad--Trace--Trace--Pad
// local copper geometry. Never a global electrical or SI qualification.
#pragma once
#include "d005_pad_trace_area_qt.h"
#include <QFile>
#include <QTemporaryFile>
#include <QProcess>
#include <QCryptographicHash>
#include <QEventLoop>
#include <QTimer>
#include <array>
#include <algorithm>

namespace d039 {
struct Path {
    ImportedPad pad;
    ImportedTrack trace;
    qint64 jointX = 0;
    qint64 jointY = 0;
    int trackIndex = -1;
};

inline bool boundedSections(const QByteArray &source) {
    if(source.isEmpty() || source.size()>8*1024*1024)return false;
    const int lineBreak=source.indexOf('\n');
    if(lineBreak<=0)return false;
    const QByteArray header=source.left(lineBreak).trimmed().split(' ').first();
    if(header!="!PADS-POWERPCB-V9.4-MILS!" &&
       header!="!PADS-POWERPCB-V9.4-MILS-CP1252!" &&
       header!="!PADS-POWERPCB-V9.4-MILS-250L!" &&
       header!="!PADS-POWERPCB-V9.4-MILS-250L-CP1252!")
        return false;
    const auto lines=source.split('\n');
    const std::array<QByteArray,7> names={
        "*PCB*","*PARTDECAL*","*PARTTYPE*","*PART*",
        "*ROUTE*","*SIGNAL*","*END*"
    };
    std::array<int,7> counts={};
    std::array<int,3> definitions={};
    int active=-1;
    bool ended=false;
    for(int i=1;i<lines.size();++i) {
        const auto row=lines[i].trimmed();
        if(ended) {
            if(!row.isEmpty())return false;
            continue;
        }
        if(row.isEmpty())continue; // match Rust and PADS parser blank-row handling
        const auto fields=row.simplified().split(' ');
        if(fields.isEmpty() || fields[0]=="*REMARK*" || fields[0]=="REMARK")
            continue;
        const QByteArray tag=fields[0];
        if(!tag.startsWith('*')) {
            if(active==1) {
                if(tag=="D2" && fields.size()==9 && fields[1]=="I")
                    ++definitions[0];
                else if(tag!="PAD" && tag!="-2" && tag!="-1" &&
                        tag!="0" && !tag.startsWith('T'))
                    return false;
            } else if(active==2) {
                if(tag!="TYPEA")return false;
                ++definitions[1];
            } else if(active==3) {
                if(tag!="U1")return false;
                ++definitions[2];
            }
            if(std::any_of(definitions.cbegin(),definitions.cend(),
                           [](int count){return count>1;}))
                return false;
            continue;
        }
        const auto pos=std::find(names.cbegin(),names.cend(),tag);
        if(pos==names.cend())return false;
        active=static_cast<int>(pos-names.cbegin());
        if(++counts[static_cast<size_t>(active)]!=1)return false;
        if(tag=="*END*")ended=true;
    }
    return std::all_of(counts.cbegin(),counts.cend(),
                       [](int count){return count==1;}) &&
           std::all_of(definitions.cbegin(),definitions.cend(),
                       [](int count){return count==1;});
}

inline bool strictlyBetween(qint64 a,qint64 m,qint64 b) {
    return (a<m && m<b) || (b<m && m<a);
}

// Reconstruct the complete expected wire from Qt's independently checked
// imported geometry and the original SOURCE pin declarations. A forged Rust
// receipt, stale SOURCE digest or unsupported geometry never passes.
inline QByteArray reconstruct(const QByteArray &source,
                              const PreviewImportPanel *panel) {
    if(!panel || !panel->imported() || !boundedSections(source))
        return {};
    const auto *art=panel->imported();
    if(art->importedSkippedCount()!=0 || art->importedTraceCount()!=2 ||
       art->padCount()!=2 || art->viaCount()!=0 || art->arcCount()!=0)
        return {};
    const QByteArray sha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    if(sha!=panel->acceptedSourceSha256())return {};
    const QString net="NET_A";
    QStringList declared;
    if(!d005::declaredPins(source,sha,net,declared))return {};
    std::array<ImportedTrack,2> traces;
    for(int t=0;t<2;++t) {
        auto &tr=traces[static_cast<size_t>(t)];
        if(!art->importedTrack(t,tr) || tr.net!=net || tr.layer!=1 ||
           tr.width_nm<=0 || tr.width_nm%254!=0 ||
           tr.x0_nm%254!=0 || tr.y0_nm%254!=0 ||
           tr.x1_nm%254!=0 || tr.y1_nm%254!=0 ||
           (tr.x0_nm==tr.x1_nm && tr.y0_nm==tr.y1_nm))
            return {};
    }
    std::array<Path,2> paths;
    for(int p=0;p<2;++p) {
        auto &item=paths[static_cast<size_t>(p)];
        if(!art->importedPad(p,item.pad))return {};
        const auto &pad=item.pad;
        if(pad.layer!=1 || pad.side!="T" ||
           (pad.shape!="R" && pad.shape!="S") ||
           pad.width_nm<=0 || pad.height_nm!=pad.width_nm ||
           pad.x_nm%254!=0 || pad.y_nm%254!=0 ||
           pad.width_nm%254!=0 || pad.width_nm>5600000LL*254 ||
           pad.reference.isEmpty() || pad.pin.isEmpty() ||
           !declared.contains(pad.reference+"."+pad.pin))
            return {};
        for(int t=0;t<2;++t) {
            const auto &tr=traces[static_cast<size_t>(t)];
            const bool a=(tr.x0_nm==pad.x_nm && tr.y0_nm==pad.y_nm);
            const bool b=(tr.x1_nm==pad.x_nm && tr.y1_nm==pad.y_nm);
            if(a==b)continue;
            if(item.trackIndex>=0)return {};
            if(tr.width_nm>pad.width_nm/2)return {};
            item.trace=tr;
            item.trackIndex=t;
            item.jointX=a?tr.x1_nm:tr.x0_nm;
            item.jointY=a?tr.y1_nm:tr.y0_nm;
        }
        if(item.trackIndex<0)return {};
    }
    if(paths[0].trackIndex==paths[1].trackIndex ||
       (paths[0].pad.reference==paths[1].pad.reference &&
        paths[0].pad.pin==paths[1].pad.pin) ||
       paths[0].trace.width_nm!=paths[1].trace.width_nm ||
       paths[0].jointX!=paths[1].jointX ||
       paths[0].jointY!=paths[1].jointY)
        return {};
    const auto &a=paths[0].pad,&b=paths[1].pad;
    const qint64 x=paths[0].jointX,y=paths[0].jointY;
    if(!((a.x_nm==b.x_nm && x==a.x_nm &&
          strictlyBetween(a.y_nm,y,b.y_nm)) ||
         (a.y_nm==b.y_nm && y==a.y_nm &&
          strictlyBetween(a.x_nm,x,b.x_nm))))
        return {};
    const auto distance=[&](const ImportedPad &p) {
        return qAbs(x-p.x_nm)+qAbs(y-p.y_nm);
    };
    if(2*distance(a)<=a.width_nm || 2*distance(b)<=b.width_nm)
        return {};
    std::sort(paths.begin(),paths.end(),[](const Path &left,const Path &right){
        return std::make_pair(left.pad.reference,left.pad.pin) <
               std::make_pair(right.pad.reference,right.pad.pin);
    });
    QByteArray expected="BOARDSI_D039_LOCAL_COPPER_V1|"+sha+
        "|NET_A|1|LOCAL_COPPER_GEOMETRY_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN"
        "|GLOBAL_ELECTRICAL_NOT_VERIFIED\n";
    for(int i=0;i<2;++i) {
        const auto &p=paths[static_cast<size_t>(i)].pad;
        expected+="P|"+QByteArray::number(i)+"|"+p.reference.toUtf8()+"|"+
            p.pin.toUtf8()+"|1|"+QByteArray::number(p.x_nm/254)+"|"+
            QByteArray::number(p.y_nm/254)+"|"+QByteArray::number(p.width_nm/254)+
            "|"+QByteArray::number(p.height_nm/254)+"|"+p.shape.toUtf8()+"\n";
    }
    for(int i=0;i<2;++i) {
        const auto &p=paths[static_cast<size_t>(i)];
        expected+="T|"+QByteArray::number(i)+"|1|"+
            QByteArray::number(p.pad.x_nm/254)+"|"+
            QByteArray::number(p.pad.y_nm/254)+"|"+
            QByteArray::number(p.jointX/254)+"|"+
            QByteArray::number(p.jointY/254)+"|"+
            QByteArray::number(p.trace.width_nm/254)+"\n";
    }
    expected+="E|0|0|2|PAD_TRACE_LOCAL_COPPER\n"
              "E|1|1|3|PAD_TRACE_LOCAL_COPPER\n"
              "E|2|2|3|EXACT_TRACE_JUNCTION_LOCAL_COPPER\n"
              "COUNT|NODES|4|EDGES|3|LOCAL_COMPONENTS|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n"
              "END|LOCAL_COPPER_GEOMETRY_ONLY_NO_GLOBAL_PASS\n";
    return expected;
}

inline bool decode(const QByteArray &wire,const QByteArray &source,
                   const PreviewImportPanel *panel) {
    return !wire.isEmpty() && wire.size()<=16384 &&
           wire==reconstruct(source,panel);
}

inline int selftest(const QString &importer,const QString &fixturePath) {
    QFile fixture(fixturePath);
    if(!fixture.open(QIODevice::ReadOnly))return 2;
    QByteArray original=fixture.readAll();
    original.replace("\r\n","\n");
    fixture.close();
    QTemporaryFile input;
    if(!input.open())return 2;
    auto save=[&](const QByteArray &data) {
        return input.resize(0) && input.seek(0) &&
               input.write(data)==data.size() && input.flush();
    };
    if(!save(original))return 2;
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    auto load=[&]() {
        panel.setStatusCallback([&loop](const QString &s){
            if(s.startsWith("PARTIAL") || s.startsWith("FAILED"))loop.quit();
        });
        if(!panel.openPath(input.fileName()))return false;
        QTimer deadline;
        deadline.setSingleShot(true);
        QObject::connect(&deadline,&QTimer::timeout,&loop,&QEventLoop::quit);
        deadline.start(8000);
        loop.exec();
        deadline.stop();
        return panel.importStatus().startsWith("PARTIAL");
    };
    auto run=[&](QByteArray &wire) {
        QProcess proc;
        proc.start(importer,QStringList{input.fileName(),"--d039-local-copper-v1"});
        if(!proc.waitForFinished(10000) || proc.exitStatus()!=QProcess::NormalExit)
            return -1;
        wire=proc.readAllStandardOutput();
        return proc.exitCode();
    };
    if(!load())return 2;
    QByteArray wire;
    const bool accepted=run(wire)==0 && decode(wire,original,&panel);
    if(!accepted)return 1;
    QByteArray forgery=wire;
    forgery.replace("E|2|2|3|EXACT_TRACE_JUNCTION_LOCAL_COPPER",
                    "E|2|1|3|EXACT_TRACE_JUNCTION_LOCAL_COPPER");
    const bool forgedRejected=forgery!=wire && !decode(forgery,original,&panel);
    ImportedReferenceRiskMarker marker;
    marker.state="D001_PAD_ENDPOINT_CANDIDATE"; // existing source-only viewer overlay
    marker.reason="Same-layer bounded pad/trace footprint only; global NOT VERIFIED";
    marker.net="NET_A";
    marker.sourceObject="D039:U1.1-U1.2";
    marker.layer=1;
    marker.x_nm=20000*254;
    marker.y_nm=35000*254;
    panel.imported()->setReferenceRisks({marker});
    const bool shown=panel.imported()->referenceRiskCountForTest()==1;
    QByteArray gap=original;
    gap.replace("200 400 1 10 0\n200 350 1 10 0",
                "200 400 1 10 0\n200 350.01 1 10 0");
    if(gap==original || !save(gap) || !load())return 2;
    const bool stale=panel.imported()->referenceRiskCountForTest()==0 &&
        panel.acceptedSourceSha256()!=
            QCryptographicHash::hash(original,QCryptographicHash::Sha256).toHex() &&
        !decode(wire,gap,&panel);
    QByteArray rejectedWire;
    const bool gapRejected=run(rejectedWire)!=0;
    QByteArray wrongNet=original;
    wrongNet.replace("*SIGNAL* NET_A","*SIGNAL* NET_B");
    if(wrongNet==original || !save(wrongNet) || !load())return 2;
    const bool netRejected=run(rejectedWire)!=0 &&
        !decode(wire,wrongNet,&panel);
    const bool ok=accepted && forgedRejected && shown && stale &&
                  gapRejected && netRejected;
    qInfo()<<"D039_NATIVE_QT_LOCAL_COPPER"<<ok<<accepted<<forgedRejected
           <<shown<<stale<<gapRejected<<netRejected
           <<"GLOBAL_ELECTRICAL_NOT_VERIFIED";
    return ok?0:1;
}
} // namespace d039
