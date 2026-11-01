// D_003: bounded 3-area COPCUT source candidate graph, independent native Qt readback.
// NEVER a manufactured electrical/P009/solver continuity assertion.
#pragma once
#include "d002_copcut_qt.h"
#include <QCryptographicHash>
#include <QEventLoop>
#include <QProcess>
#include <QTemporaryFile>
#include <QTimer>
#include <array>
#include <set>
#include <vector>
#include <algorithm>

namespace d003 {
using Rect=std::array<qint64,4>;
inline bool pairContact(const Rect &a,const Rect &b,
                        const Rect *holeA,const Rect *holeB) {
    std::vector<qint64> xs={a[0],a[2],b[0],b[2]};
    std::vector<qint64> ys={a[1],a[3],b[1],b[3]};
    if(holeA) { xs.push_back((*holeA)[0]);xs.push_back((*holeA)[2]);
                ys.push_back((*holeA)[1]);ys.push_back((*holeA)[3]); }
    if(holeB) { xs.push_back((*holeB)[0]);xs.push_back((*holeB)[2]);
                ys.push_back((*holeB)[1]);ys.push_back((*holeB)[3]); }
    std::sort(xs.begin(),xs.end());xs.erase(std::unique(xs.begin(),xs.end()),xs.end());
    std::sort(ys.begin(),ys.end());ys.erase(std::unique(ys.begin(),ys.end()),ys.end());
    for(size_t i=1;i<xs.size();++i)for(size_t j=1;j<ys.size();++j) {
        const qint64 mx=xs[i-1]+xs[i],my=ys[j-1]+ys[j];
        auto inside=[&](const Rect &r) {
            return 2*r[0]<mx && mx<2*r[2] && 2*r[1]<my && my<2*r[3];
        };
        if(inside(a) && inside(b) && (!holeA || !inside(*holeA))
           && (!holeB || !inside(*holeB)))return true;
    }
    return false;
}
inline bool decode(const QByteArray &wire,const QByteArray &sourceSha,
                   const ImportedArtworkPanel *artwork,const QStringList &names,
                   const QString &net,int layer,c005::Receipt &output) {
    if(!artwork || wire.isEmpty() || wire.size()>8192 || !wire.endsWith('\n') ||
       sourceSha.size()!=64 || names.size()!=3 || names[0].isEmpty() ||
       names[1].isEmpty() || names[2].isEmpty() ||
       names[0]==names[1] || names[0]==names[2] || names[1]==names[2] ||
       layer<=0 || net.isEmpty() ||
       artwork->importedReferenceAreaCount(layer,net)!=3)return false;
    const auto lines=wire.split('\n');
    if(lines.size()!=10 || !lines.last().isEmpty() ||
       lines[8]!="END|3|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")return false;
    const auto h=lines[0].split('|');
    if(h.size()!=9 || h[0]!="BOARDSI_D003_THREE_AREA_V1" ||
       h[1]!=sourceSha || h[4]!="PHYSICAL_NOT_VERIFIED" ||
       h[5]!="P009_UNSUPPORTED" || h[6]!="SI_NOT_RUN" ||
       h[7]!="0" || h[8]!="0")return false;
    std::array<Rect,3> boxes{};
    std::array<std::vector<Rect>,3> sourceHoles{};
    std::array<QString,3> ids{};
    for(int i=0;i<3;++i) {
        const auto row=lines[i+1].split('|');
        if(row.size()!=11 || row[0]!="AREA" ||
           row[1]!=QByteArray::number(i) ||
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
        Rect r{};
        if(!d002::rectFromRing(item.ring,r))return false;
        for(int k=0;k<4;++k) {
            bool ok=false;
            const qint64 value=row[5+k].toLongLong(&ok);
            if(!ok || value!=r[static_cast<size_t>(k)])return false;
        }
        boxes[static_cast<size_t>(i)]=r;
        ids[static_cast<size_t>(i)]=id;
        for(const auto &ring:item.cutouts) {
            Rect cut{};
            if(!d002::rectFromRing(ring,cut))return false;
            sourceHoles[static_cast<size_t>(i)].push_back(cut);
        }
    }
    std::array<bool,3> hasHole={false,false,false};
    std::array<Rect,3> holeBoxes{};
    const auto hole=lines[4].split('|');
    if(hole.size()==2 && hole[0]=="HOLE" && hole[1]=="NONE") {
        for(const auto &v:sourceHoles)if(!v.empty())return false;
    } else {
        if(hole.size()!=6 || hole[0]!="HOLE")return false;
        bool ok=false;const int owner=hole[1].toInt(&ok);
        if(!ok || owner<0 || owner>2)return false;
        for(int i=0;i<3;++i)if(sourceHoles[static_cast<size_t>(i)].size()!=
            static_cast<size_t>(i==owner))return false;
        auto &v=holeBoxes[static_cast<size_t>(owner)];
        for(int k=0;k<4;++k) {
            bool good=false;const qint64 value=hole[k+2].toLongLong(&good);
            if(!good || value!=sourceHoles[static_cast<size_t>(owner)][0]
                                     [static_cast<size_t>(k)])return false;
            v[static_cast<size_t>(k)]=value;
        }
        const auto &r=boxes[static_cast<size_t>(owner)];
        if(v[0]<=r[0] || v[1]<=r[1] || v[2]>=r[2] || v[3]>=r[3] ||
           v[0]>=v[2] || v[1]>=v[3])return false;
        hasHole[static_cast<size_t>(owner)]=true;
    }
    std::array<int,3> groups={0,1,2};
    for(int k=0;k<3;++k) {
        const int i=(k==2)?1:0;
        const int j=(k==0)?1:2;
        const auto &a=boxes[static_cast<size_t>(i)],&b=boxes[static_cast<size_t>(j)];
        const Rect *ca=hasHole[static_cast<size_t>(i)]?&holeBoxes[static_cast<size_t>(i)]:nullptr;
        const Rect *cb=hasHole[static_cast<size_t>(j)]?&holeBoxes[static_cast<size_t>(j)]:nullptr;
        const bool contact=pairContact(a,b,ca,cb);
        const auto edge=lines[k+5].split('|');
        if(edge.size()!=4 || edge[0]!="EDGE" || edge[1]!=QByteArray::number(i) ||
           edge[2]!=QByteArray::number(j) ||
           edge[3]!=(contact?"SOURCE_CONTACT_CANDIDATE":"SOURCE_SEPARATION_RISK"))return false;
        if(contact) {
            const int lo=std::min(groups[static_cast<size_t>(i)],groups[static_cast<size_t>(j)]);
            const int hi=std::max(groups[static_cast<size_t>(i)],groups[static_cast<size_t>(j)]);
            for(int &group:groups)if(group==hi)group=lo;
        }
    }
    for(int i=0;i<3;++i) {
        const auto row=lines[i+1].split('|');
        if(row[10]!=QByteArray::number(groups[static_cast<size_t>(i)]))return false;
    }
    const int count=static_cast<int>(std::set<int>(groups.begin(),groups.end()).size());
    if(h[3]!=QByteArray::number(count))return false;
    const QByteArray state=count==1?"SOURCE_GEOMETRIC_COMPONENT_CANDIDATE":
                                    "SOURCE_SEPARATION_RISK";
    if(h[2]!=state)return false;
    c005::Receipt receipt;
    receipt.state=QString::fromLatin1(state);
    receipt.reason="D_003 SOURCE THREE AREA GRAPH / PHYSICAL NOT_VERIFIED / P009 UNSUPPORTED";
    for(int i=0;i<3;++i) {
        const auto &r=boxes[static_cast<size_t>(i)];
        ImportedReferenceRiskMarker marker;
        marker.state="CORRIDOR_CANDIDATE_RISK";
        marker.reason=QString("D_003 GROUP %1 / SOURCE ONLY / PHYSICAL NOT_VERIFIED / SI NOT_RUN")
                      .arg(groups[static_cast<size_t>(i)]);
        marker.net=net;marker.layer=layer;marker.sourceObject=ids[static_cast<size_t>(i)];
        marker.viaObject="-";marker.x_nm=(r[0]+r[2])*127;
        marker.y_nm=(r[1]+r[3])*127;
        receipt.markers.push_back(marker);
    }
    output=std::move(receipt);
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
        "RC COPPER 0 0 1 1 0 NET_REF\nCOPCLS 5 0.01 0 2\n"
        "360 360\n390 360\n390 390\n360 390\n360 360\n"
        "*ROUTE*\n*SIGNAL* NET_REF\nR1.1 R2.2\n"
        "120 180 2 10 0\n180 180 2 10 0\n*END*\n");
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
            "--d003-three-area-copcut-v1","RA","RB","RC","NET_REF","2"});
        if(!p.waitForFinished(10000) || p.exitStatus()!=QProcess::NormalExit ||
           p.exitCode()!=0)return QByteArray();
        return p.readAllStandardOutput();
    };
    const QByteArray digest=QCryptographicHash::hash(original,QCryptographicHash::Sha256).toHex();
    c005::Receipt first;
    const bool verified=panel.acceptedSourceSha256()==digest &&
        decode(query(),digest,panel.imported(),{"RA","RB","RC"},"NET_REF",2,first) &&
        first.state=="SOURCE_SEPARATION_RISK" && first.markers.size()==3;
    if(!verified)return 1;
    panel.imported()->setReferenceRisks(first.markers);
    const bool shown=panel.imported()->referenceRiskCountForTest()==3;
    QByteArray modified=original;
    modified.replace("200 200\n300 200\n300 300\n200 300\n200 200\n",
                     "340 340\n370 340\n370 370\n340 370\n340 340\n");
    if(!source.resize(0) || !source.seek(0) ||
       source.write(modified)!=modified.size() || !source.flush())return 2;
    if(!load())return 2;
    const QByteArray fresh=QCryptographicHash::hash(modified,QCryptographicHash::Sha256).toHex();
    const bool stale=panel.imported()->referenceRiskCountForTest()==0 &&
        fresh!=digest && panel.acceptedSourceSha256()==fresh;
    c005::Receipt second;
    const bool changed=decode(query(),fresh,panel.imported(),{"RA","RB","RC"},
            "NET_REF",2,second) &&
        second.state=="SOURCE_GEOMETRIC_COMPONENT_CANDIDATE" &&
        !decode(query(),digest,panel.imported(),{"RA","RB","RC"},"NET_REF",2,first);
    const bool ok=verified && shown && stale && changed;
    qInfo()<<"D003_NATIVE_QT_THREE_AREA"<<ok<<verified<<shown<<stale<<changed
           <<"PHYSICAL_NOT_VERIFIED_P009_UNSUPPORTED";
    return ok?0:1;
}
} // namespace d003
