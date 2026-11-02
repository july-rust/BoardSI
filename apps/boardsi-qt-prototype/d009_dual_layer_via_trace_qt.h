// D_009: independently verify actual imported dual-layer pad/area and B-trace to third area.
// Geometric candidate only; never electrical, P009 or Solver acceptance.
#pragma once
#include "d008_dual_layer_via_area_qt.h"
namespace d009 {
using Rect = std::array<qint64,4>;
inline bool decode(const QByteArray &wire,const QByteArray &sha,const QByteArray &source,
    const PreviewImportPanel *panel,const QStringList &names,const QString &net,
    std::array<int,2> layers,c005::Receipt &out) {
    const auto *art=panel?panel->imported():nullptr;
    if(!art || panel->acceptedSourceSha256()!=sha || names.size()!=3
        || layers[0]<1 || layers[0]>=layers[1]
        || names[0]==names[1] || names[0]==names[2] || names[1]==names[2]
        || art->importedSkippedCount()!=0 || art->viaCount()!=1
        || art->importedTraceCount()!=1 || art->copperCount()!=3
        || art->arcCount()!=0 || art->pourEdgeCount()!=0
        || wire.isEmpty() || wire.size()>32768 || !wire.endsWith('\n')
        || sha.size()!=64 || QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex()!=sha
        || art->importedReferenceAreaCount(layers[0],net)!=1
        || art->importedReferenceAreaCount(layers[1],net)!=2) return false;
    const auto rows=wire.split('\n');
    if(rows.size()!=16 || !rows.last().isEmpty()
        || rows[14]!="END|SOURCE_ONLY_NOT_ELECTRICAL")return false;
    const auto h=rows[0].split('|');
    if(h.size()!=8 || h[0]!="BOARDSI_D009_DUAL_LAYER_VIA_TRACE_THIRD_AREA_V1"
        || h[1]!=sha || h[2]!="SOURCE_GEOMETRY_ONLY"
        || h[3]!="PHYSICAL_NOT_VERIFIED" || h[4]!="P009_UNSUPPORTED"
        || h[5]!="SI_NOT_RUN" || h[6]!="0" || h[7]!="0")return false;
    const auto v=rows[1].split('|');
    if(v.size()!=9 || v[0]!="VIA" || v[1]!="0")return false;
    QByteArray declaredNet;
    if(!c005::boundedHex(v[2],declaredNet) || QString::fromUtf8(declaredNet)!=net)return false;
    std::array<qint64,6> expected{};
    for(int i=0;i<6;++i) {
        bool ok=false;
        expected[static_cast<size_t>(i)]=v[i+3].toLongLong(&ok);
        if(!ok)return false;
    }
    // Check source-grid domain before multiplication and squared distance.
    if(expected[0]<-5'600'000 || expected[0]>5'600'000
        || expected[1]<-5'600'000 || expected[1]>5'600'000
        || expected[2]<=0 || expected[2]>5'600'000
        || expected[3]<=0 || expected[3]>100'000
        || expected[4]<1 || expected[4]>64
        || expected[5]<1 || expected[5]>64)return false;
    ImportedVia via;
    if(!art->importedVia(0,via) || via.net!=net
        || via.x_nm!=expected[0]*254 || via.y_nm!=expected[1]*254
        || via.diameter_nm!=expected[2]*254 || via.drill_nm!=expected[3]*254
        || via.firstLayer!=expected[4] || via.lastLayer!=expected[5]
        || expected[2]<=expected[3]
        || layers[0]<via.firstLayer || layers[1]>via.lastLayer
        || !d006::declaredConductivePad(source,via,layers[0])
        || !d006::declaredConductivePad(source,via,layers[1]))return false;
    const std::array<qint64,2> centre={expected[0],expected[1]};
    std::array<Rect,3> boxes{};
    Rect thirdHole{};
    bool thirdHasHole=false;
    bool both=true;
    for(int i=0;i<3;++i) {
        const int layer=i==0?layers[0]:layers[1];
        const auto a=rows[2+i*3].split('|');
        if(a.size()!=9 || a[0]!="AREA" || a[1]!=QByteArray::number(i)
            || a[4]!=QByteArray::number(layer))return false;
        QByteArray name,id;
        if(!c005::boundedHex(a[2],name) || !c005::boundedHex(a[3],id)
            || QString::fromUtf8(name)!=names[i]
            || !QString::fromUtf8(id).startsWith("source-copper-"+names[i]+"-"))return false;
        ImportedCopperArea area;
        if(!art->importedCopperGeometry(QString::fromUtf8(id),layer,net,area)
            || area.name!=names[i] || area.circle || area.curved
            || !area.roundCutouts.empty() || area.cutouts.size()>1
            || !d002::rectFromRing(area.ring,boxes[static_cast<size_t>(i)]))return false;
        for(int k=0;k<4;++k) {
            bool valid=false;
            const qint64 value=a[5+k].toLongLong(&valid);
            if(!valid || value!=boxes[static_cast<size_t>(i)][static_cast<size_t>(k)])return false;
        }
        const auto holeRow=rows[3+i*3].split('|');
        if(holeRow.size()<3 || holeRow[0]!="HOLE"
            || holeRow[1]!=QByteArray::number(i))return false;
        const bool hasHole=!area.cutouts.empty();
        Rect hole{};
        if(hasHole) {
            if(holeRow.size()!=6 || !d002::rectFromRing(area.cutouts[0],hole))return false;
            for(int k=0;k<4;++k) {
                bool valid=false;
                const qint64 value=holeRow[k+2].toLongLong(&valid);
                if(!valid || value!=hole[static_cast<size_t>(k)])return false;
            }
        } else if(holeRow.size()!=3 || holeRow[2]!="NONE")return false;
        if(i==2) { thirdHasHole=hasHole; thirdHole=hole; continue; }
        const QByteArray state=hasHole && d006::intersects(centre,expected[2],hole)
            ?"SOURCE_COPCUT_RISK":
            d006::inside(centre,expected[2],boxes[static_cast<size_t>(i)])
            ?"SOURCE_VIA_PAD_AREA_CANDIDATE":
            d006::intersects(centre,expected[2],boxes[static_cast<size_t>(i)])
            ?"SOURCE_PAD_BOUNDARY_RISK":"SOURCE_SEPARATION_RISK";
        const auto contact=rows[4+i*3].split('|');
        if(contact.size()!=3 || contact[0]!="CONTACT"
            || contact[1]!=QByteArray::number(i) || contact[2]!=state)return false;
        both &= state=="SOURCE_VIA_PAD_AREA_CANDIDATE";
    }
    ImportedTrack track;
    if(!art->importedTrack(0,track) || track.net!=net
        || track.layer!=layers[1] || track.width_nm<=0)return false;
    const std::array<qint64,5> raw={
        track.x0_nm,track.y0_nm,track.x1_nm,track.y1_nm,track.width_nm};
    for(qint64 n:raw) {
        if(n%254!=0 || n/254 < -5'600'000 || n/254 > 5'600'000)return false;
    }
    const auto trace=rows[11].split('|');
    if(trace.size()!=9 || trace[0]!="TRACE" || trace[1]!="0"
        || trace[2]!=QByteArray::number(layers[1]))return false;
    QByteArray traceNet;
    if(!c005::boundedHex(trace[3],traceNet) || QString::fromUtf8(traceNet)!=net)return false;
    for(int i=0;i<5;++i) {
        if(trace[i+4]!=QByteArray::number(raw[static_cast<size_t>(i)]/254))return false;
    }
    const std::array<qint64,2> p0={raw[0]/254,raw[1]/254};
    const std::array<qint64,2> p1={raw[2]/254,raw[3]/254};
    if(p0==p1)return false;
    const auto padContains=[&](const std::array<qint64,2>&p){
        const qint64 dx=p[0]-centre[0],dy=p[1]-centre[1];
        return 4*(dx*dx+dy*dy)<expected[2]*expected[2];
    };
    const auto strict=[](const std::array<qint64,2>&p,const Rect&r){
        return p[0]>r[0] && p[0]<r[2] && p[1]>r[1] && p[1]<r[3];
    };
    const auto touched=[](const std::array<qint64,2>&p,const Rect&r){
        return p[0]>=r[0] && p[0]<=r[2] && p[1]>=r[1] && p[1]<=r[3];
    };
    const bool a=padContains(p0),b=padContains(p1);
    QByteArray state="SOURCE_TRACE_AREA_NOT_VERIFIED";
    int ve=-1,ae=-1;
    if(both && a!=b) {
        const int endpoint=a?0:1;
        const auto other=a?p1:p0;
        if(strict(other,boxes[2]) && !(thirdHasHole && touched(other,thirdHole))
            && !touched(other,boxes[1])
            && !d006::intersects(centre,expected[2],boxes[2])) {
            state="SOURCE_DUAL_LAYER_VIA_TRACE_THIRD_AREA_CANDIDATE";
            ve=endpoint;ae=1-endpoint;
        } else if(thirdHasHole && touched(other,thirdHole))state="SOURCE_COPCUT_RISK";
        else if(touched(other,boxes[1]))state="SOURCE_AMBIGUOUS_AREA_OWNER";
        else if(touched(other,boxes[2]))state="SOURCE_BOUNDARY_RISK";
    } else if(a && b)state="SOURCE_AMBIGUOUS_ENDPOINT";
    else if(!a && !b)state="SOURCE_VIA_TRACE_DISCONNECTED";
    const auto third=rows[10].split('|');
    if(third.size()!=3 || third[0]!="CONTACT" || third[1]!="2"
        || third[2]!=(ve>=0?"SOURCE_TRACE_AREA_ENDPOINT_CANDIDATE":
                           "SOURCE_TRACE_AREA_NOT_VERIFIED"))return false;
    const auto link=rows[12].split('|');
    if(link.size()!=7 || link[0]!="LINK" || link[1]!=state
        || link[2]!=(ve>=0?QByteArray::number(ve):QByteArray("-"))
        || link[3]!=(ae>=0?QByteArray::number(ae):QByteArray("-"))
        || link[4]!="1" || link[5]!="2" || link[6]!=(ve>=0?"1":"0"))return false;
    const QByteArray overall=ve>=0?
        "SOURCE_DUAL_LAYER_VIA_TRACE_THIRD_AREA_CANDIDATE":
        "SOURCE_DUAL_LAYER_TRACE_NOT_VERIFIED";
    const auto end=rows[13].split('|');
    if(end.size()!=2 || end[0]!="OVERALL" || end[1]!=overall)return false;
    c005::Receipt receipt;
    receipt.state=QString::fromLatin1(overall);
    receipt.reason="D_009 SOURCE DUAL-LAYER VIA / TRACE / THIRD AREA; NOT ELECTRICAL; P009 UNSUPPORTED";
    if(ve>=0) {
        for(int layer:layers) {
            ImportedReferenceRiskMarker marker;
            marker.state="CORRIDOR_CANDIDATE_RISK";
            marker.reason="D_009 VIA PAD SOURCE CONTACT / NOT ELECTRICAL";
            marker.net=net;marker.layer=layer;
            marker.sourceObject="D009-via-pad";marker.viaObject=via.name;
            marker.x_nm=via.x_nm;marker.y_nm=via.y_nm;
            marker.diameter_nm=via.diameter_nm;
            receipt.markers.push_back(marker);
        }
        ImportedReferenceRiskMarker last;
        last.state="CORRIDOR_CANDIDATE_RISK";
        last.reason="D_009 TRACE THIRD-AREA SOURCE ENDPOINT / NOT ELECTRICAL";
        last.net=net;last.layer=layers[1];
        last.sourceObject="D009-trace-third-area";last.viaObject=via.name;
        last.x_nm=raw[static_cast<size_t>(ae*2)];
        last.y_nm=raw[static_cast<size_t>(ae*2+1)];
        last.diameter_nm=track.width_nm;
        receipt.markers.push_back(last);
    }
    out=std::move(receipt);
    return true;
}
inline int selftest(const QString &importer) {
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
        "RC COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 4\n"
        "280 240\n380 240\n380 340\n280 340\n280 240\n"
        "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n"
        "150 250 4 10 0 SIMPLEVIA\n330 280 4 10 0\n*END*\n");
    const auto save=[&](const QByteArray &bytes) {
        return file.resize(0) && file.seek(0)
            && file.write(bytes)==bytes.size() && file.flush();
    };
    if(!save(source))return 2;
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    const auto load=[&]() {
        panel.setStatusCallback([&](const QString &text) {
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
        p.start(importer,{file.fileName(),"--d009-dual-layer-via-trace-v1",
                          "RA","RB","RC","NET_A","1","4","0"});
        if(!p.waitForFinished(10000) || p.exitStatus()!=QProcess::NormalExit
            || p.exitCode()!=0)return QByteArray();
        return p.readAllStandardOutput();
    };
    if(!load())return 2;
    const auto sha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    c005::Receipt first,ignored;
    const auto wire=query();
    const bool accepted=panel.acceptedSourceSha256()==sha
        && decode(wire,sha,source,&panel,{"RA","RB","RC"},"NET_A",{1,4},first)
        && first.markers.size()==3;
    if(!accepted)return 1;
    panel.imported()->setReferenceRisks(first.markers);
    const bool displayed=panel.imported()->referenceRiskCountForTest()==3;
    auto forged=wire;
    forged.replace("LINK|SOURCE_DUAL_LAYER_VIA_TRACE_THIRD_AREA_CANDIDATE|0|1|1|2|1",
                   "LINK|SOURCE_TRACE_AREA_NOT_VERIFIED|-|-|1|2|0");
    const bool noForgery=!decode(forged,sha,source,&panel,{"RA","RB","RC"},"NET_A",
                                 {1,4},ignored);
    auto forgedCoord=wire;
    forgedCoord.replace("VIA|0|4e45545f41|15000|25000|3500",
                        "VIA|0|4e45545f41|-9223372036854760808|25000|3500");
    const bool noOverflow=!decode(forgedCoord,sha,source,&panel,{"RA","RB","RC"},"NET_A",
                                  {1,4},ignored);
    auto same=source;
    same.replace("U1.1 U2.2","U1.1  U2.2");
    if(!save(same)||!load())return 2;
    const auto newSha=QCryptographicHash::hash(same,QCryptographicHash::Sha256).toHex();
    c005::Receipt fresh;
    const bool freshSha=newSha!=sha && panel.acceptedSourceSha256()==newSha
        && !decode(wire,sha,source,&panel,{"RA","RB","RC"},"NET_A",{1,4},ignored)
        && decode(query(),newSha,same,&panel,{"RA","RB","RC"},"NET_A",{1,4},fresh)
        && fresh.markers.size()==3;
    auto changed=source;
    changed.replace("330 280 4 10 0","280 280 4 10 0");
    if(!save(changed)||!load())return 2;
    const auto thirdSha=QCryptographicHash::hash(changed,QCryptographicHash::Sha256).toHex();
    c005::Receipt invalid;
    const bool stale=panel.acceptedSourceSha256()==thirdSha
        && panel.imported()->referenceRiskCountForTest()==0
        && !decode(wire,thirdSha,changed,&panel,{"RA","RB","RC"},"NET_A",{1,4},ignored);
    const bool boundary=decode(query(),thirdSha,changed,&panel,{"RA","RB","RC"},"NET_A",
                               {1,4},invalid)
        && invalid.state=="SOURCE_DUAL_LAYER_TRACE_NOT_VERIFIED"
        && invalid.markers.empty();
    const bool ok=accepted && displayed && noForgery && noOverflow && freshSha
        && stale && boundary;
    qInfo()<<"D009_NATIVE_QT_DUAL_LAYER_VIA_TRACE"<<ok<<accepted<<displayed
           <<noForgery<<noOverflow<<freshSha<<stale<<boundary
           <<"PHYSICAL_NOT_VERIFIED_P009_UNSUPPORTED";
    return ok?0:1;
}
} // namespace d009
