#pragma once
#include "d006_via_three_area_qt.h"
#include "d004_endpoint_attachment_qt.h"
namespace d007 {
// Source geometry only; electrical and SI are not verified.

using Rect = std::array<qint64,4>;
inline bool decode(const QByteArray &wire,const QByteArray &sha,const QByteArray &source,
    const ImportedArtworkPanel *panel,const QStringList &names,const QString &net,
    int layer,c005::Receipt &out) {
    if(!panel || panel->importedSkippedCount()!=0 || panel->viaCount()!=1
       || panel->importedTraceCount()!=1 || panel->arcCount()!=0
       || wire.size()>32768 || !wire.endsWith('\n') || sha.size()!=64
       || QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex()!=sha)return false;
    const auto lines=wire.split('\n');
    if(lines.size()!=18 || !lines.last().isEmpty()
       || lines[16]!="END|SOURCE_ONLY_NOT_ELECTRICAL")return false;
    const auto h=lines[0].split('|');
    if(h.size()!=8 || h[0]!="BOARDSI_D007_VIA_TRACE_THREE_AREA_V1"
       || h[1]!=sha || h[2]!="SOURCE_GEOMETRY_ONLY"
       || h[3]!="PHYSICAL_NOT_VERIFIED" || h[4]!="P009_UNSUPPORTED"
       || h[5]!="SI_NOT_RUN" || h[6]!="0" || h[7]!="0")return false;
    QByteArray embedded;
    for(int i=1;i<=13;++i){embedded+=lines[i];embedded+='\n';}
    c005::Receipt viaReceipt;
    if(!d006::decode(embedded,sha,source,panel,names,net,layer,viaReceipt))return false;
    ImportedTrack track;
    if(!panel->importedTrack(0,track) || track.net!=net || track.layer!=layer
       || track.width_nm<=0)return false;
    const std::array<qint64,5> raw={track.x0_nm,track.y0_nm,
        track.x1_nm,track.y1_nm,track.width_nm};
    for(qint64 v:raw)if(v%254!=0)return false;
    const auto trace=lines[14].split('|');
    if(trace.size()!=9 || trace[0]!="TRACE" || trace[1]!="0"
       || trace[2]!=QByteArray::number(layer))return false;
    QByteArray n;
    if(!c005::boundedHex(trace[3],n) || QString::fromUtf8(n)!=net)return false;
    for(int i=0;i<5;++i)
        if(trace[i+4]!=QByteArray::number(raw[static_cast<size_t>(i)]/254))return false;
    const std::array<std::array<qint64,2>,2> pts={
        std::array<qint64,2>{raw[0]/254,raw[1]/254},
        std::array<qint64,2>{raw[2]/254,raw[3]/254}};
    ImportedVia via;
    if(!panel->importedVia(0,via) || via.net!=net || via.diameter_nm<=via.drill_nm
       || via.x_nm%254 || via.y_nm%254 || via.diameter_nm%254)return false;
    const std::array<qint64,2> centre={via.x_nm/254,via.y_nm/254};
    const qint64 diameter=via.diameter_nm/254;
    const auto contains=[&](const std::array<qint64,2> &p){
        const qint64 dx=p[0]-centre[0],dy=p[1]-centre[1];
        return 4*(dx*dx+dy*dy)<diameter*diameter;};
    const bool a=contains(pts[0]),b=contains(pts[1]);
    std::array<Rect,3> boxes{},holes{};
    std::array<bool,3> hasHole={false,false,false};
    std::array<int,3> groups{};
    for(int i=0;i<3;++i) {
        const auto ar=lines[i+3].split('|');
        if(ar.size()!=11 || ar[0]!="AREA" || ar[1]!=QByteArray::number(i))return false;
        bool ok=false;
        groups[static_cast<size_t>(i)]=ar[10].toInt(&ok);
        if(!ok || groups[static_cast<size_t>(i)]<0 || groups[static_cast<size_t>(i)]>2)return false;
        QByteArray id;
        if(!c005::boundedHex(ar[3],id))return false;
        ImportedCopperArea area;
        if(!panel->importedCopperGeometry(QString::fromUtf8(id),layer,net,area)
           || !d002::rectFromRing(area.ring,boxes[static_cast<size_t>(i)])
           || area.cutouts.size()>1 || !area.roundCutouts.empty())return false;
        if(!area.cutouts.empty()) {
            if(!d002::rectFromRing(area.cutouts[0],holes[static_cast<size_t>(i)]))return false;
            hasHole[static_cast<size_t>(i)]=true;
        }
    }
    const auto contact=lines[12].split('|');
    if(contact.size()!=4 || contact[0]!="CONTACT")return false;
    bool valid=false;
    const int viaOwner=contact[1]=="SOURCE_VIA_PAD_AREA_CANDIDATE"
        ?contact[2].toInt(&valid):-1;
    if(viaOwner>=0 && (!valid || viaOwner>2))return false;
    QByteArray state="SOURCE_VIA_PAD_AREA_NOT_VERIFIED";
    int ve=-1,ae=-1,other=-1,component=-1;
    if(viaOwner>=0) {
        if(a==b)state=a?"SOURCE_AMBIGUOUS_OWNER":"SOURCE_VIA_TRACE_DISCONNECTED";
        else {
            const int endpoint=a?0:1;
            const auto at=d004::attach(pts[static_cast<size_t>(1-endpoint)],
                boxes,holes,hasHole,groups);
            if(at.state=="SOURCE_ENDPOINT_ATTACHMENT_CANDIDATE" && at.owner!=viaOwner) {
                state="SOURCE_VIA_TRACE_AREA_CANDIDATE";
                ve=endpoint;ae=1-endpoint;other=at.owner;component=at.group;
            } else if(at.state=="SOURCE_COPCUT_BLOCKED")state="SOURCE_COPCUT_RISK";
            else if(at.state=="SOURCE_BOUNDARY_RISK")state="SOURCE_PAD_BOUNDARY_RISK";
            else if(at.state=="SOURCE_AMBIGUOUS_OWNER")state="SOURCE_AMBIGUOUS_OWNER";
            else if(at.state=="SOURCE_ENDPOINT_ATTACHMENT_CANDIDATE")state="SOURCE_SAME_AREA_NOT_CHAIN";
            else state="SOURCE_TRACE_AREA_DISCONNECTED";
        }
    }
    const auto link=lines[15].split('|');
    if(link.size()!=8 || link[0]!="LINK" || link[1]!=state)return false;
    const auto exact=[&](int idx,int v) {
        return link[idx]==(v<0?QByteArray("-"):QByteArray::number(v));
    };
    if(!exact(2,ve) || !exact(3,ae) || !exact(4,viaOwner)
       || !exact(5,other) || !exact(6,component)
       || link[7]!=(ve>=0?"1":"0"))return false;
    c005::Receipt result;
    result.state=QString::fromLatin1(state);
    result.reason="D_007 SOURCE VIA/TRACE/AREA ONLY / NOT ELECTRICAL / P009 UNSUPPORTED";
    if(ve>=0) {
        ImportedReferenceRiskMarker marker;
        marker.state="CORRIDOR_CANDIDATE_RISK";
        marker.reason="D_007 SOURCE CONTACT CHAIN CANDIDATE / NOT ELECTRICAL";
        marker.net=net;marker.layer=layer;
        marker.sourceObject="D007-via-trace-area";marker.viaObject=via.name;
        marker.x_nm=raw[static_cast<size_t>(ve*2)];
        marker.y_nm=raw[static_cast<size_t>(ve*2+1)];
        marker.diameter_nm=via.diameter_nm;
        result.markers.push_back(marker);
    }
    out=std::move(result);
    return true;
}
inline int selftest(const QString &importer) {
    const QFileInfo binary(importer);
    const QString cli=binary.absolutePath()+"/boardsi-d007-via-trace-three-area"
        +(binary.suffix()=="exe"?".exe":"");
    if(!QFileInfo(cli).isExecutable())return 2;
    QTemporaryFile file;
    if(!file.open())return 2;
    const QByteArray source(
        "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n"
        "*PCB*\nMAXLAYER 4\n*VIA*\nSIMPLEVIA 15 2\n-2 35 R\n0 35 R\n"
        "*LINES*\n"
        "RA COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n"
        "100 200\n200 200\n200 300\n100 300\n100 200\n"
        "RB COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n"
        "190 220\n290 220\n290 320\n190 320\n190 220\n"
        "RC COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n"
        "280 240\n380 240\n380 340\n280 340\n280 240\n"
        "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n"
        "150 250 1 10 0 SIMPLEVIA\n250 250 1 10 0\n*END*\n");
    const auto save=[&](const QByteArray &bytes) {
        return file.resize(0) && file.seek(0) &&
               file.write(bytes)==bytes.size() && file.flush();
    };
    if(!save(source))return 2;
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    const auto load=[&]() {
        panel.setStatusCallback([&](const QString &text){
            if(text.startsWith("PARTIAL") || text.startsWith("FAILED"))loop.quit();
        });
        if(!panel.openPath(file.fileName()))return false;
        QTimer deadline;deadline.setSingleShot(true);
        QObject::connect(&deadline,&QTimer::timeout,&loop,&QEventLoop::quit);
        deadline.start(8000);loop.exec();deadline.stop();
        return panel.importStatus().startsWith("PARTIAL");
    };
    const auto query=[&]() {
        QProcess p;
        p.start(cli,{file.fileName(),"RA","RB","RC","NET_A","1","0"});
        if(!p.waitForFinished(10000) || p.exitStatus()!=QProcess::NormalExit
            || p.exitCode()!=0)return QByteArray();
        return p.readAllStandardOutput();
    };
    if(!load())return 2;
    const auto digest=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    const auto wire=query();
    c005::Receipt first;
    const bool accepted=panel.acceptedSourceSha256()==digest
        && decode(wire,digest,source,panel.imported(),{"RA","RB","RC"},"NET_A",1,first)
        && first.markers.size()==1;
    if(!accepted)return 1;
    panel.imported()->setReferenceRisks(first.markers);
    const bool displayed=panel.imported()->referenceRiskCountForTest()==1;
    auto forged=wire;
    forged.replace("LINK|SOURCE_VIA_TRACE_AREA_CANDIDATE|0|1|0|1|0|1",
                   "LINK|SOURCE_TRACE_AREA_DISCONNECTED|-|-|0|-|-|0");
    c005::Receipt ignored;
    const bool noForgery=!decode(forged,digest,source,panel.imported(),
        {"RA","RB","RC"},"NET_A",1,ignored);
    auto changed=source;
    changed.replace("250 250 1 10 0","160 250 1 10 0");
    if(!save(changed) || !load())return 2;
    const auto newSha=QCryptographicHash::hash(changed,QCryptographicHash::Sha256).toHex();
    c005::Receipt newResult;
    const bool stale=panel.acceptedSourceSha256()==newSha
        && panel.imported()->referenceRiskCountForTest()==0
        && !decode(wire,newSha,changed,panel.imported(),
                   {"RA","RB","RC"},"NET_A",1,ignored);
    const bool sameArea=decode(query(),newSha,changed,panel.imported(),
        {"RA","RB","RC"},"NET_A",1,newResult)
        && newResult.state=="SOURCE_SAME_AREA_NOT_CHAIN" && newResult.markers.empty();
    const bool ok=accepted && displayed && noForgery && stale && sameArea;
    qInfo()<<"D007_NATIVE_QT_VIA_TRACE_THREE_AREA"<<ok<<accepted<<displayed
           <<noForgery<<stale<<sameArea<<"PHYSICAL_NOT_VERIFIED_P009_UNSUPPORTED";
    return ok?0:1;
}


}
