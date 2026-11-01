// D_002: independent source/Qt readback of COPCLS/COPCUT geometry-only contact.
// No electrical graph edge, Return Current, physical PASS, Solver or cache.
#pragma once
#include "c022_reference_area_risk.h"
#include <QCryptographicHash>
#include <QEventLoop>
#include <QProcess>
#include <QTemporaryFile>
#include <QTimer>
#include <algorithm>
#include <array>
#include <vector>

namespace d002 {
using Rect = std::array<qint64,4>;

inline bool rectFromRing(
    const std::vector<std::pair<qint64,qint64>> &ring, Rect &out) {
    if(ring.size()!=5 || ring.front()!=ring.back())return false;
    qint64 x0=ring.front().first, x1=x0;
    qint64 y0=ring.front().second, y1=y0;
    for(const auto &p:ring) {
        if(p.first%254 || p.second%254 ||
           p.first < -1422400000LL || p.first > 1422400000LL ||
           p.second < -1422400000LL || p.second > 1422400000LL)return false;
        x0=std::min(x0,p.first);x1=std::max(x1,p.first);
        y0=std::min(y0,p.second);y1=std::max(y1,p.second);
    }
    if(x0>=x1 || y0>=y1)return false;
    bool used[4]={false,false,false,false};
    for(size_t i=0;i<4;++i) {
        const auto p=ring[i];
        int k=-1;
        if(p.first==x0 && p.second==y0)k=0;
        else if(p.first==x1 && p.second==y0)k=1;
        else if(p.first==x1 && p.second==y1)k=2;
        else if(p.first==x0 && p.second==y1)k=3;
        if(k<0 || used[k])return false;
        used[k]=true;
    }
    out={x0/254,y0/254,x1/254,y1/254};
    return true;
}
inline bool decode(const QByteArray &wire,const QByteArray &sha,
                   const ImportedArtworkPanel *artwork,
                   const QStringList &names,const QString &net,int layer,
                   c005::Receipt &out) {
    if(!artwork || wire.isEmpty() || wire.size()>4096 || !wire.endsWith('\n') ||
       sha.size()!=64 || names.size()!=2 || names[0].isEmpty() ||
       names[0]==names[1] || net.isEmpty() || layer<1 ||
       artwork->importedReferenceAreaCount(layer,net)!=2)return false;
    const auto lines=wire.split('\n');
    if(lines.size()!=6 || !lines.last().isEmpty() ||
       lines[4]!="END|2|SOURCE_CONTACT_HYPOTHESIS_ONLY")return false;
    const auto h=lines[0].split('|');
    if(h.size()!=8 || h[0]!="BOARDSI_D002_COPCUT_CONTACT_V1" || h[1]!=sha ||
       h[3]!="PHYSICAL_NOT_VERIFIED" || h[4]!="P009_UNSUPPORTED" ||
       h[5]!="SI_NOT_RUN" || h[6]!="0" || h[7]!="0")return false;
    std::array<Rect,2> boxes{};
    std::array<std::vector<Rect>,2> actualHoles{};
    std::array<QString,2> identities{};
    for(int i=0;i<2;++i) {
        const auto row=lines[i+1].split('|');
        if(row.size()!=10 || row[0]!="AREA" || row[1]!=QByteArray::number(i) ||
           row[4]!=QByteArray::number(layer))return false;
        QByteArray nameBytes,idBytes,netBytes;
        if(!c005::boundedHex(row[2],nameBytes) ||
           !c005::boundedHex(row[3],idBytes) ||
           !c005::boundedHex(row[9],netBytes))return false;
        const QString name=QString::fromUtf8(nameBytes);
        const QString id=QString::fromUtf8(idBytes);
        if(name!=names[i] || QString::fromUtf8(netBytes)!=net ||
           !id.startsWith("source-copper-"+name+"-"))return false;
        ImportedCopperArea item;
        if(!artwork->importedCopperGeometry(id,layer,net,item) ||
           item.name!=name || item.circle || item.curved ||
           !item.roundCutouts.empty() || item.cutouts.size()>1)return false;
        Rect source{};
        if(!rectFromRing(item.ring,source))return false;
        for(int k=0;k<4;++k) {
            bool ok=false;
            const qint64 v=row[5+k].toLongLong(&ok);
            if(!ok || v!=source[static_cast<size_t>(k)])return false;
        }
        boxes[static_cast<size_t>(i)]=source;
        identities[static_cast<size_t>(i)]=id;
        for(const auto &hole:item.cutouts) {
            Rect cut{};
            if(!rectFromRing(hole,cut))return false;
            actualHoles[static_cast<size_t>(i)].push_back(cut);
        }
    }
    std::array<bool,2> hasHole={false,false};
    std::array<Rect,2> holes{};
    const auto cut=lines[3].split('|');
    if(cut.size()==2 && cut[0]=="HOLE" && cut[1]=="NONE") {
        if(!actualHoles[0].empty() || !actualHoles[1].empty())return false;
    } else {
        if(cut.size()!=6 || cut[0]!="HOLE" ||
           (cut[1]!="0" && cut[1]!="1"))return false;
        const int owner=cut[1]=="0"?0:1;
        if(actualHoles[static_cast<size_t>(owner)].size()!=1 ||
           !actualHoles[static_cast<size_t>(1-owner)].empty())return false;
        for(int k=0;k<4;++k) {
            bool ok=false;const qint64 v=cut[k+2].toLongLong(&ok);
            if(!ok || v!=actualHoles[static_cast<size_t>(owner)][0][static_cast<size_t>(k)])
                return false;
            holes[static_cast<size_t>(owner)][static_cast<size_t>(k)]=v;
        }
        const auto &r=boxes[static_cast<size_t>(owner)];
        const auto &v=holes[static_cast<size_t>(owner)];
        if(v[0]<=r[0] || v[1]<=r[1] || v[2]>=r[2] || v[3]>=r[3] ||
           v[0]>=v[2] || v[1]>=v[3])return false;
        hasHole[static_cast<size_t>(owner)]=true;
    }
    std::vector<qint64> xs,ys;
    for(int i=0;i<2;++i) {
        const auto &r=boxes[static_cast<size_t>(i)];
        xs.push_back(r[0]);xs.push_back(r[2]);
        ys.push_back(r[1]);ys.push_back(r[3]);
        if(hasHole[static_cast<size_t>(i)]) {
            const auto &v=holes[static_cast<size_t>(i)];
            xs.push_back(v[0]);xs.push_back(v[2]);
            ys.push_back(v[1]);ys.push_back(v[3]);
        }
    }
    std::sort(xs.begin(),xs.end());xs.erase(std::unique(xs.begin(),xs.end()),xs.end());
    std::sort(ys.begin(),ys.end());ys.erase(std::unique(ys.begin(),ys.end()),ys.end());
    bool contact=false;
    for(size_t x=1;x<xs.size();++x)for(size_t y=1;y<ys.size();++y) {
        const qint64 mx=xs[x-1]+xs[x],my=ys[y-1]+ys[y];
        auto inside=[&](const Rect &r) {
            return 2*r[0]<mx && mx<2*r[2] && 2*r[1]<my && my<2*r[3];
        };
        auto kept=[&](int i) {
            return inside(boxes[static_cast<size_t>(i)]) &&
                (!hasHole[static_cast<size_t>(i)] ||
                 !inside(holes[static_cast<size_t>(i)]));
        };
        if(kept(0) && kept(1))contact=true;
    }
    const QByteArray expected=contact?"SOURCE_CONTACT_CANDIDATE":"SOURCE_SEPARATION_RISK";
    if(h[2]!=expected)return false;
    c005::Receipt receipt;
    receipt.state=QString::fromLatin1(expected);
    receipt.reason="D_002 SOURCE COPCUT HYPOTHESIS / PHYSICAL NOT_VERIFIED / P009 UNSUPPORTED";
    const auto &b=boxes[1];
    ImportedReferenceRiskMarker marker;
    marker.state="CORRIDOR_CANDIDATE_RISK";
    marker.reason=QString("D_002 %1 / SOURCE ONLY / PHYSICAL NOT_VERIFIED / SI NOT_RUN")
        .arg(QString::fromLatin1(expected));
    marker.net=net;marker.layer=layer;marker.sourceObject=identities[1];
    marker.viaObject="-";marker.x_nm=(b[0]+b[2])*127;
    marker.y_nm=(b[1]+b[3])*127;
    receipt.markers.push_back(marker);
    out=std::move(receipt);
    return true;
}
inline int selftest(const QString &importer) {
    QTemporaryFile source;
    if(!source.open())return 2;
    const QByteArray original(
        "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n"
        "*PCB*\nMAXLAYER 4\n*LINES*\n"
        "RA COPPER 0 0 2 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n"
        "100 100\n400 100\n400 400\n100 400\n100 100\n"
        "COPCUT 5 0.01 0 2\n150 150\n350 150\n350 350\n150 350\n150 150\n"
        "RB COPPER 0 0 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n"
        "200 200\n300 200\n300 300\n200 300\n200 200\n"
        "*ROUTE*\n*SIGNAL* NET_REF\nR1.1 R2.2\n120 180 2 10 0\n"
        "180 180 2 10 0\n*END*\n");
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
    if(!load())return 2;
    auto query=[&]() {
        QProcess p;
        p.start(importer,QStringList{source.fileName(),
            "--d002-copcut-contact-v1","RA","RB","NET_REF","2"});
        if(!p.waitForFinished(10000) || p.exitStatus()!=QProcess::NormalExit ||
           p.exitCode()!=0)return QByteArray();
        return p.readAllStandardOutput();
    };
    const auto digest=QCryptographicHash::hash(original,QCryptographicHash::Sha256).toHex();
    c005::Receipt receipt;
    const bool first=panel.acceptedSourceSha256()==digest &&
        decode(query(),digest,panel.imported(),{"RA","RB"},"NET_REF",2,receipt) &&
        receipt.state=="SOURCE_SEPARATION_RISK";
    if(!first)return 1;
    panel.imported()->setReferenceRisks(receipt.markers);
    const bool marked=panel.imported()->referenceRiskCountForTest()==1;
    QByteArray modified=original;
    modified.replace("200 200\n300 200\n300 300\n200 300\n200 200\n",
        "300 300\n380 300\n380 380\n300 380\n300 300\n");
    if(!source.resize(0) || !source.seek(0) ||
       source.write(modified)!=modified.size() || !source.flush())return 2;
    if(!load())return 2;
    const auto changedDigest=QCryptographicHash::hash(modified,QCryptographicHash::Sha256).toHex();
    const bool stale=panel.imported()->referenceRiskCountForTest()==0 &&
        changedDigest!=digest && panel.acceptedSourceSha256()==changedDigest;
    c005::Receipt next;
    const bool second=decode(query(),changedDigest,panel.imported(),
        {"RA","RB"},"NET_REF",2,next) &&
        next.state=="SOURCE_CONTACT_CANDIDATE" &&
        !decode(query(),digest,panel.imported(),{"RA","RB"},"NET_REF",2,receipt);
    const bool ok=first && marked && stale && second;
    qInfo()<<"D002_NATIVE_QT_COPCUT"<<ok<<first<<marked<<stale<<second
           <<"PHYSICAL_NOT_VERIFIED_P009_UNSUPPORTED";
    return ok?0:1;
}
} // namespace d002
