// D_004: one selected source Trace -> three-area COPCUT geometry-only groups.
// NO electrical, manufactured copper, P009, solver or cache claims.
#pragma once
#include "d003_three_area_qt.h"
#include <QCryptographicHash>
#include <QProcess>
#include <QTemporaryFile>
#include <QEventLoop>
#include <QTimer>
#include <array>
namespace d004 {
using Rect=d003::Rect;
inline bool inside(const std::array<qint64,2> &p,const Rect &r) {
    return r[0]<p[0] && p[0]<r[2] && r[1]<p[1] && p[1]<r[3];
}
inline bool closed(const std::array<qint64,2> &p,const Rect &r) {
    return r[0]<=p[0] && p[0]<=r[2] && r[1]<=p[1] && p[1]<=r[3];
}
struct Result {
    QByteArray state;
    int owner=-1;
    int group=-1;
};
inline Result attach(const std::array<qint64,2> &p,
    const std::array<Rect,3> &boxes,const std::array<Rect,3> &holes,
    const std::array<bool,3> &hasHole,const std::array<int,3> &groups) {
    int found=-1;
    bool ambiguous=false,blocked=false,boundary=false;
    for(int i=0;i<3;++i) {
        const auto &r=boxes[static_cast<size_t>(i)];
        const bool strictly=inside(p,r);
        if(closed(p,r) && !strictly)boundary=true;
        if(!strictly)continue;
        if(hasHole[static_cast<size_t>(i)] && closed(p,holes[static_cast<size_t>(i)])) {
            if(inside(p,holes[static_cast<size_t>(i)]))blocked=true;
            else boundary=true;
            continue;
        }
        if(found>=0)ambiguous=true;
        else found=i;
    }
    if(ambiguous)return {"SOURCE_AMBIGUOUS_OWNER",-1,-1};
    if(blocked)return {"SOURCE_COPCUT_BLOCKED",-1,-1};
    if(boundary)return {"SOURCE_BOUNDARY_RISK",-1,-1};
    if(found>=0)return {"SOURCE_ENDPOINT_ATTACHMENT_CANDIDATE",found,
                         groups[static_cast<size_t>(found)]};
    return {"SOURCE_SEPARATION_RISK",-1,-1};
}
inline bool decode(const QByteArray &wire,const QByteArray &sourceSha,
    const ImportedArtworkPanel *artwork,const QStringList &names,
    const QString &net,int layer,c005::Receipt &out) {
    if(!artwork || sourceSha.size()!=64 || wire.size()>12288 ||
       !wire.endsWith('\n') || artwork->importedSkippedCount()!=0 ||
       artwork->importedTraceCount()!=1 ||
       artwork->arcCount()!=0 || artwork->viaCount()!=0)return false;
    const auto lines=wire.split('\n');
    if(lines.size()!=15 || !lines.last().isEmpty() ||
       lines[13]!="END|2|SOURCE_ONLY_NOT_PHYSICAL")return false;
    const auto header=lines[0].split('|');
    if(header.size()!=8 || header[0]!="BOARDSI_D004_ENDPOINT_V1" ||
       header[1]!=sourceSha || header[2]!="SOURCE_ATTACHMENT_ONLY" ||
       header[3]!="PHYSICAL_NOT_VERIFIED" || header[4]!="P009_UNSUPPORTED" ||
       header[5]!="SI_NOT_RUN" || header[6]!="0" || header[7]!="0")return false;
    QByteArray d003Wire;
    for(int i=1;i<=9;++i) { d003Wire+=lines[i];d003Wire+='\n'; }
    c005::Receipt basis;
    if(!d003::decode(d003Wire,sourceSha,artwork,names,net,layer,basis))return false;
    ImportedTrack track;
    if(!artwork->importedTrack(0,track) || track.layer!=layer || track.net!=net ||
       track.width_nm<=0 || track.width_nm%254!=0)return false;
    const std::array<qint64,5> positions={track.x0_nm,track.y0_nm,
        track.x1_nm,track.y1_nm,track.width_nm};
    for(qint64 p:positions)if(p%254!=0)return false;
    const auto t=lines[10].split('|');
    if(t.size()!=9 || t[0]!="TRACE" || t[1]!="0" ||
       t[2]!=QByteArray::number(layer))return false;
    QByteArray netBytes;
    if(!c005::boundedHex(t[3],netBytes) || QString::fromUtf8(netBytes)!=net)return false;
    for(int k=0;k<5;++k)if(t[4+k]!=QByteArray::number(positions[static_cast<size_t>(k)]/254))return false;
    std::array<Rect,3> boxes{},holes{};
    std::array<bool,3> hasHole={false,false,false};
    std::array<int,3> groups{};
    for(int i=0;i<3;++i) {
        const auto row=lines[i+2].split('|');
        if(row.size()!=11 || row[0]!="AREA")return false;
        bool ok=false;
        groups[static_cast<size_t>(i)]=row[10].toInt(&ok);
        if(!ok || groups[static_cast<size_t>(i)]<0 || groups[static_cast<size_t>(i)]>2)return false;
        const auto baseRow=lines[i+2].split('|');
        QByteArray idBytes;
        if(!c005::boundedHex(baseRow[3],idBytes))return false;
        ImportedCopperArea area;
        if(!artwork->importedCopperGeometry(QString::fromUtf8(idBytes),layer,net,area)
           || !d002::rectFromRing(area.ring,boxes[static_cast<size_t>(i)])
           || area.cutouts.size()>1 || !area.roundCutouts.empty())return false;
        if(!area.cutouts.empty()) {
            if(!d002::rectFromRing(area.cutouts[0],holes[static_cast<size_t>(i)]))return false;
            hasHole[static_cast<size_t>(i)]=true;
        }
    }
    const std::array<std::array<qint64,2>,2> points={
        std::array<qint64,2>{positions[0]/254,positions[1]/254},
        std::array<qint64,2>{positions[2]/254,positions[3]/254}
    };
    c005::Receipt result;
    result.state="SOURCE_ATTACHMENT_ONLY";
    result.reason="D_004 ENDPOINT SOURCE GEOMETRY ONLY / P009 UNSUPPORTED / SI_NOT_RUN";
    for(int i=0;i<2;++i) {
        const auto checked=attach(points[static_cast<size_t>(i)],boxes,holes,hasHole,groups);
        const auto row=lines[i+11].split('|');
        if(row.size()!=5 || row[0]!="POINT" || row[1]!=QByteArray::number(i) ||
           row[2]!=checked.state ||
           row[3]!=(checked.owner<0?QByteArray("-"):QByteArray::number(checked.owner)) ||
           row[4]!=(checked.group<0?QByteArray("-"):QByteArray::number(checked.group)))return false;
        ImportedReferenceRiskMarker marker;
        marker.state="CORRIDOR_CANDIDATE_RISK";
        marker.reason=QString("D_004 ENDPOINT %1 %2 / SOURCE ONLY / PHYSICAL NOT_VERIFIED")
                      .arg(i).arg(QString::fromLatin1(checked.state));
        marker.net=net;marker.layer=layer;marker.sourceObject=QString("D004-trace-0-endpoint-%1").arg(i);
        marker.viaObject="-";
        marker.x_nm=positions[static_cast<size_t>(i*2)];
        marker.y_nm=positions[static_cast<size_t>(i*2+1)];
        result.markers.push_back(marker);
    }
    out=std::move(result);
    return true;
}
inline int selftest(const QString &importer) {
    QTemporaryFile source;
    if(!source.open())return 2;
    const QByteArray original(
        "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n"
        "*PCB*\nMAXLAYER 4\n*LINES*\n"
        "RA COPPER 0 0 2 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n"
        "100 100\n240 100\n240 240\n100 240\n100 100\n"
        "COPCUT 5 0.01 0 2\n130 130\n170 130\n170 170\n130 170\n130 130\n"
        "RB COPPER 0 0 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n"
        "200 200\n320 200\n320 320\n200 320\n200 200\n"
        "RC COPPER 0 0 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n"
        "300 300\n390 300\n390 390\n300 390\n300 300\n"
        "*ROUTE*\n*SIGNAL* NET_REF\nR1.1 R2.2\n"
        "120 180 2 10 0\n150 150 2 10 0\n*END*\n");
    if(source.write(original)!=original.size() || !source.flush())return 2;
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    auto load=[&]() {
        panel.setStatusCallback([&loop](const QString &s) {
            if(s.startsWith("PARTIAL") || s.startsWith("FAILED"))loop.quit();
        });
        if(!panel.openPath(source.fileName()))return false;
        QTimer limit;limit.setSingleShot(true);
        QObject::connect(&limit,&QTimer::timeout,&loop,&QEventLoop::quit);
        limit.start(8000);loop.exec();limit.stop();
        return panel.importStatus().startsWith("PARTIAL");
    };
    auto query=[&]() {
        QProcess p;
        p.start(importer,QStringList{source.fileName(),"--d004-endpoint-attachment-v1",
                "RA","RB","RC","NET_REF","2","0"});
        if(!p.waitForFinished(10000) || p.exitStatus()!=QProcess::NormalExit ||
           p.exitCode()!=0)return QByteArray();
        return p.readAllStandardOutput();
    };
    if(!load())return 2;
    const auto originalSha=QCryptographicHash::hash(original,QCryptographicHash::Sha256).toHex();
    const QByteArray firstWire=query();
    c005::Receipt first;
    const bool verified=panel.acceptedSourceSha256()==originalSha &&
        decode(firstWire,originalSha,panel.imported(),{"RA","RB","RC"},"NET_REF",2,first) &&
        first.markers.size()==2 &&
        first.markers[0].reason.contains("SOURCE_ENDPOINT_ATTACHMENT_CANDIDATE") &&
        first.markers[1].reason.contains("SOURCE_COPCUT_BLOCKED");
    if(!verified)return 1;
    QByteArray tampered=firstWire;
    tampered.replace("POINT|0|SOURCE_ENDPOINT_ATTACHMENT_CANDIDATE|0|0",
                     "POINT|0|SOURCE_SEPARATION_RISK|-|-");
    c005::Receipt rejected;
    const bool cannotSpoof=!decode(tampered,originalSha,panel.imported(),
                                  {"RA","RB","RC"},"NET_REF",2,rejected);
    panel.imported()->setReferenceRisks(first.markers);
    const bool displayed=panel.imported()->referenceRiskCountForTest()==2;
    // Codex P2: prove that an independently imported, partially skipped
    // Source can never accept a forged D_004 receipt with its current SHA.
    QByteArray omitted=original;
    omitted.replace("150 150 2 10 0\\n*END*",
                    "150 150 2 10 0\\n160 160 2 10 24576\\n*END*");
    if(!source.resize(0) || !source.seek(0) ||
       source.write(omitted)!=omitted.size() || !source.flush())return 2;
    if(!load())return 2;
    const QByteArray omittedSha=QCryptographicHash::hash(
        omitted,QCryptographicHash::Sha256).toHex();
    QByteArray forged=firstWire;
    forged.replace(originalSha,omittedSha);
    const bool rejectedOmitted=panel.imported()->importedSkippedCount()>0 &&
        panel.imported()->importedTraceCount()==1 &&
        !decode(forged,omittedSha,panel.imported(),
                {"RA","RB","RC"},"NET_REF",2,rejected);
    QByteArray modified=original;
    modified.replace("150 150 2 10 0","350 350 2 10 0");
    if(!source.resize(0) || !source.seek(0) ||
       source.write(modified)!=modified.size() || !source.flush())return 2;
    if(!load())return 2;
    const auto fresh=QCryptographicHash::hash(modified,QCryptographicHash::Sha256).toHex();
    c005::Receipt second;
    const bool stale=panel.imported()->referenceRiskCountForTest()==0 &&
        fresh!=originalSha && panel.acceptedSourceSha256()==fresh &&
        !decode(firstWire,fresh,panel.imported(),{"RA","RB","RC"},"NET_REF",2,rejected);
    const bool reloaded=decode(query(),fresh,panel.imported(),
                              {"RA","RB","RC"},"NET_REF",2,second) &&
        second.markers.size()==2 &&
        second.markers[1].reason.contains("SOURCE_ENDPOINT_ATTACHMENT_CANDIDATE");
    const bool ok=verified && cannotSpoof && displayed && rejectedOmitted && stale && reloaded;
    qInfo()<<"D004_NATIVE_QT_ENDPOINT_ATTACHMENT"<<ok<<verified<<cannotSpoof<<displayed
           <<rejectedOmitted<<stale<<reloaded<<"PHYSICAL_NOT_VERIFIED_P009_UNSUPPORTED";
    return ok?0:1;
}
} // namespace d004
