// D_008: separate source-bound pads on two Copper layers. Geometry only.
#pragma once
#include "d006_via_three_area_qt.h"
namespace d008 {
using Rect = std::array<qint64,4>;
inline bool decode(const QByteArray &wire,const QByteArray &sha,const QByteArray &source,
    const PreviewImportPanel *panel,const QStringList &names,const QString &net,
    std::array<int,2> layers,c005::Receipt &out) {
    const auto *artwork=panel?panel->imported():nullptr;
    if(!artwork || panel->acceptedSourceSha256()!=sha || names.size()!=2 || layers[0]<1 || layers[0]>=layers[1]
       || names[0]==names[1] || artwork->importedSkippedCount()!=0
       || artwork->viaCount()!=1 || artwork->copperCount()!=2
       || artwork->arcCount()!=0 || artwork->pourEdgeCount()!=0
       || wire.isEmpty() || wire.size()>16384 || !wire.endsWith('\n')
       || sha.size()!=64 || QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex()!=sha
       || artwork->importedReferenceAreaCount(layers[0],net)!=1
       || artwork->importedReferenceAreaCount(layers[1],net)!=1) return false;
    const auto rows=wire.split('\n');
    if(rows.size()!=11 || !rows.last().isEmpty()
       || rows[9]!="END|SOURCE_ONLY_NOT_ELECTRICAL")return false;
    const auto h=rows[0].split('|');
    if(h.size()!=8 || h[0]!="BOARDSI_D008_DUAL_LAYER_VIA_AREA_V1"
       || h[1]!=sha || h[2]!="SOURCE_GEOMETRY_ONLY"
       || h[3]!="PHYSICAL_NOT_VERIFIED" || h[4]!="P009_UNSUPPORTED"
       || h[5]!="SI_NOT_RUN" || h[6]!="0" || h[7]!="0")return false;
    const auto v=rows[1].split('|');
    if(v.size()!=9 || v[0]!="VIA" || v[1]!="0")return false;
    QByteArray declaredNet;
    if(!c005::boundedHex(v[2],declaredNet) || QString::fromUtf8(declaredNet)!=net)return false;
    std::array<qint64,6> expected{};
    for(int i=0;i<6;++i){
        bool valid=false; expected[static_cast<size_t>(i)]=v[i+3].toLongLong(&valid);
        if(!valid)return false;
    }
    // Source tick domain: +/-5,600,000; round pad max 5,600,000;
    // drill max 100,000. Reject before any signed multiplication.
    if(expected[0]<-5'600'000 || expected[0]>5'600'000
       || expected[1]<-5'600'000 || expected[1]>5'600'000
       || expected[2]<=0 || expected[2]>5'600'000
       || expected[3]<=0 || expected[3]>100'000
       || expected[4]<1 || expected[4]>64
       || expected[5]<1 || expected[5]>64)return false;
    ImportedVia imported;
    if(!artwork->importedVia(0,imported) || imported.net!=net
       || imported.x_nm!=expected[0]*254 || imported.y_nm!=expected[1]*254
       || imported.diameter_nm!=expected[2]*254 || imported.drill_nm!=expected[3]*254
       || imported.firstLayer!=expected[4] || imported.lastLayer!=expected[5]
       || expected[2]<=expected[3] || expected[3]<=0
       || layers[0]<imported.firstLayer || layers[1]>imported.lastLayer
       || !d006::declaredConductivePad(source,imported,layers[0])
       || !d006::declaredConductivePad(source,imported,layers[1]))return false;
    const std::array<qint64,2> centre={expected[0],expected[1]};
    bool both=true;
    for(int i=0;i<2;++i) {
        const int layer=layers[static_cast<size_t>(i)];
        const auto a=rows[2+i*3].split('|');
        if(a.size()!=9 || a[0]!="AREA" || a[1]!=QByteArray::number(i)
           || a[4]!=QByteArray::number(layer))return false;
        QByteArray name,id;
        if(!c005::boundedHex(a[2],name) || !c005::boundedHex(a[3],id)
           || QString::fromUtf8(name)!=names[i]
           || !QString::fromUtf8(id).startsWith("source-copper-"+names[i]+"-"))return false;
        ImportedCopperArea area;
        if(!artwork->importedCopperGeometry(QString::fromUtf8(id),layer,net,area)
           || area.name!=names[i] || area.circle || area.curved
           || !area.roundCutouts.empty() || area.cutouts.size()>1)return false;
        Rect rect{};
        if(!d002::rectFromRing(area.ring,rect))return false;
        for(int k=0;k<4;++k){
            bool valid=false;const qint64 value=a[5+k].toLongLong(&valid);
            if(!valid || value!=rect[static_cast<size_t>(k)])return false;
        }
        const auto holeRow=rows[3+i*3].split('|');
        if(holeRow.size()<3 || holeRow[0]!="HOLE"
           || holeRow[1]!=QByteArray::number(i))return false;
        bool hasHole=!area.cutouts.empty();
        Rect hole{};
        if(hasHole){
            if(holeRow.size()!=6 || !d002::rectFromRing(area.cutouts[0],hole))return false;
            for(int k=0;k<4;++k){
                bool valid=false;const qint64 n=holeRow[k+2].toLongLong(&valid);
                if(!valid || n!=hole[static_cast<size_t>(k)])return false;
            }
        } else if(holeRow.size()!=3 || holeRow[2]!="NONE")return false;
        const QByteArray state=hasHole && d006::intersects(centre,expected[2],hole)
           ?"SOURCE_COPCUT_RISK":
           d006::inside(centre,expected[2],rect)?"SOURCE_VIA_PAD_AREA_CANDIDATE":
           d006::intersects(centre,expected[2],rect)?"SOURCE_PAD_BOUNDARY_RISK":
           "SOURCE_SEPARATION_RISK";
        const auto contact=rows[4+i*3].split('|');
        if(contact.size()!=3 || contact[0]!="CONTACT"
           || contact[1]!=QByteArray::number(i) || contact[2]!=state)return false;
        both &= state=="SOURCE_VIA_PAD_AREA_CANDIDATE";
    }
    const auto overall=rows[8].split('|');
    const QByteArray state=both?"SOURCE_DUAL_LAYER_VIA_PAD_AREA_CANDIDATE":
        "SOURCE_DUAL_LAYER_NOT_VERIFIED";
    if(overall.size()!=2 || overall[0]!="OVERALL" || overall[1]!=state)return false;
    c005::Receipt receipt;
    receipt.state=QString::fromLatin1(state);
    receipt.reason="D_008 DUAL-LAYER SOURCE GEOMETRY ONLY / NOT ELECTRICAL / P009 UNSUPPORTED";
    if(both){
        for(int layer:layers){
            ImportedReferenceRiskMarker marker;
            marker.state="CORRIDOR_CANDIDATE_RISK";
            marker.reason="D_008 VIA PAD / COPCLS SOURCE CANDIDATE / NOT ELECTRICAL";
            marker.net=net; marker.layer=layer;
            marker.sourceObject="D008-dual-layer-source"; marker.viaObject=imported.name;
            marker.x_nm=imported.x_nm;marker.y_nm=imported.y_nm;
            marker.diameter_nm=imported.diameter_nm;
            receipt.markers.push_back(marker);
        }
    }
    out=std::move(receipt);
    return true;
}
inline int selftest(const QString &importer){
    if(!QFileInfo(importer).isExecutable())return 2;
    QTemporaryFile file;
    if(!file.open())return 2;
    const QByteArray source(
        "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n"
        "*PCB*\nMAXLAYER 4\n*VIA*\nSIMPLEVIA 15 2\n-2 35 R\n0 35 R\n"
        "*LINES*\n"
        "RA COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n"
        "100 200\n200 200\n200 300\n100 300\n100 200\n"
        "RB COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 4\n"
        "100 200\n200 200\n200 300\n100 300\n100 200\n"
        "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n"
        "150 250 1 10 0 SIMPLEVIA\n250 250 1 10 0\n*END*\n");
    const auto save=[&](const QByteArray &b){
        return file.resize(0) && file.seek(0) && file.write(b)==b.size() && file.flush();
    };
    if(!save(source))return 2;
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    const auto load=[&](){
        panel.setStatusCallback([&](const QString &text){
            if(text.startsWith("PARTIAL") || text.startsWith("FAILED"))loop.quit();
        });
        if(!panel.openPath(file.fileName()))return false;
        QTimer deadline;deadline.setSingleShot(true);
        QObject::connect(&deadline,&QTimer::timeout,&loop,&QEventLoop::quit);
        deadline.start(8000);loop.exec();deadline.stop();
        return panel.importStatus().startsWith("PARTIAL");
    };
    const auto query=[&](){
        QProcess p;
        p.start(importer,{file.fileName(),"--d008-dual-layer-via-area-v1",
                          "RA","RB","NET_A","1","4","0"});
        if(!p.waitForFinished(10000)||p.exitStatus()!=QProcess::NormalExit
           || p.exitCode()!=0)return QByteArray();
        return p.readAllStandardOutput();
    };
    if(!load())return 2;
    const auto sha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    c005::Receipt first,ignored;
    const auto wire=query();
    const bool accepted=panel.acceptedSourceSha256()==sha
        && decode(wire,sha,source,&panel,{"RA","RB"},"NET_A",{1,4},first)
        && first.markers.size()==2;
    if(!accepted)return 1;
    panel.imported()->setReferenceRisks(first.markers);
    const bool displayed=panel.imported()->referenceRiskCountForTest()==2;
    auto forged=wire;
    forged.replace("OVERALL|SOURCE_DUAL_LAYER_VIA_PAD_AREA_CANDIDATE",
                   "OVERALL|SOURCE_DUAL_LAYER_NOT_VERIFIED");
    const bool noForgery=!decode(forged,sha,source,&panel,
        {"RA","RB"},"NET_A",{1,4},ignored);
    auto forgedCoord=wire;
    forgedCoord.replace("VIA|0|4e45545f41|15000|25000|3500",
                        "VIA|0|4e45545f41|-9223372036854760808|25000|3500");
    const bool noOverflow=!decode(forgedCoord,sha,source,&panel,
        {"RA","RB"},"NET_A",{1,4},ignored);
    auto sameArtwork=source;
    sameArtwork.replace("U1.1 U2.2", "U1.1  U2.2");
    if(!save(sameArtwork)||!load())return 2;
    const auto sameArtSha=QCryptographicHash::hash(sameArtwork,QCryptographicHash::Sha256).toHex();
    c005::Receipt refreshed;
    const bool noCachedReplay=sameArtSha!=sha
        && panel.acceptedSourceSha256()==sameArtSha
        && !decode(wire,sha,source,&panel,{"RA","RB"},"NET_A",{1,4},ignored)
        && decode(query(),sameArtSha,sameArtwork,&panel,{"RA","RB"},"NET_A",{1,4},refreshed)
        && refreshed.markers.size()==2;
    auto changed=source;
    changed.replace("100 200\n200 200\n200 300\n100 300\n100 200\n"
                    "*ROUTE*","135 200\n235 200\n235 300\n135 300\n135 200\n"
                    "*ROUTE*");
    if(!save(changed)||!load())return 2;
    const auto nextSha=QCryptographicHash::hash(changed,QCryptographicHash::Sha256).toHex();
    c005::Receipt next;
    const bool stale=nextSha!=sha && panel.acceptedSourceSha256()==nextSha
        && panel.imported()->referenceRiskCountForTest()==0
        && !decode(wire,nextSha,changed,&panel,{"RA","RB"},"NET_A",{1,4},ignored);
    const bool boundary=decode(query(),nextSha,changed,&panel,
        {"RA","RB"},"NET_A",{1,4},next)
        && next.state=="SOURCE_DUAL_LAYER_NOT_VERIFIED" && next.markers.empty();
    const bool ok=accepted && displayed && noForgery && noOverflow && noCachedReplay
        && stale && boundary;
    qInfo()<<"D008_NATIVE_QT_DUAL_LAYER_SOURCE"<<ok<<accepted<<displayed
           <<noForgery<<noOverflow<<noCachedReplay<<stale<<boundary
           <<"PHYSICAL_NOT_VERIFIED_P009_UNSUPPORTED";
    return ok?0:1;
}
} // namespace d008
