// D_012 native Qt smoke: actually accepted PADS source and independently audited graph.
#pragma once
#include "d012_via_span_provenance_qt.h"
namespace d012 {
inline int selftest(const QString &importer) {
    if(!QFileInfo(importer).isExecutable())return 2;
    QTemporaryFile f;
    if(!f.open())return 2;
    const QByteArray source(
        "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n"
        "*PCB*\nMAXLAYER 4\n*VIA*\nSIMPLEVIA 15 2\n-2 35 R\n0 35 R\n"
        "*LINES*\n"
        "RA COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n"
        "100 200\n200 200\n200 300\n100 300\n100 200\n"
        "RB COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 4\n"
        "100 200\n200 200\n200 300\n100 300\n100 200\n"
        "RC COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n"
        "280 240\n380 240\n380 340\n280 340\n280 240\n"
        "RD COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 4\n"
        "280 240\n380 240\n380 340\n280 340\n280 240\n"
        "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n"
        "330 280 1 10 0\n150 250 4 10 0 SIMPLEVIA\n330 280 4 10 0\n*END*\n");
    const auto save=[&](const QByteArray &s) {
        return f.resize(0)&&f.seek(0)&&f.write(s)==s.size()&&f.flush();
    };
    if(!save(source))return 2;
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    const auto load=[&]() {
        panel.setStatusCallback([&](const QString &s) {
            if(s.startsWith("PARTIAL")||s.startsWith("FAILED"))loop.quit();
        });
        if(!panel.openPath(f.fileName()))return false;
        QTimer deadline;deadline.setSingleShot(true);
        QObject::connect(&deadline,&QTimer::timeout,&loop,&QEventLoop::quit);
        deadline.start(8000);loop.exec();deadline.stop();
        return panel.importStatus().startsWith("PARTIAL");
    };
    const auto query=[&]() {
        QProcess p;
        p.start(importer,{f.fileName(),"--d012-via-span-provenance-v1",
            "RA","RB","RC","RD","NET_A","1","4","0"});
        if(!p.waitForFinished(10000)||p.exitStatus()!=QProcess::NormalExit
            || p.exitCode()!=0)return QByteArray();
        return p.readAllStandardOutput();
    };
    if(!load())return 2;
    const auto sha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    c005::Receipt first,ignored;
    const auto wire=query();
    const bool accepted=!wire.isEmpty()&&decode(wire,sha,source,&panel,
        {"RA","RB","RC","RD"},"NET_A",{1,4},first)&&first.markers.size()==4;
    if(!accepted)return 1;
    panel.imported()->setReferenceRisks(first.markers);
    const bool displayed=panel.imported()->referenceRiskCountForTest()==4;
    auto forgery=wire;
    forgery.replace("EDGE|0|0|1|SOURCE_CONTACT_CANDIDATE|1",
                    "EDGE|0|0|4|SOURCE_CONTACT_CANDIDATE|1");
    const bool noForgery=!decode(forgery,sha,source,&panel,
        {"RA","RB","RC","RD"},"NET_A",{1,4},ignored);
    auto nodeForgery=wire;
    nodeForgery.replace("NODE|1|AREA|1|","NODE|1|AREA|4|");
    const bool noNodeForgery=!decode(nodeForgery,sha,source,&panel,
        {"RA","RB","RC","RD"},"NET_A",{1,4},ignored);
    auto platingForgery=wire;
    platingForgery.replace("PLATING_IMPORTER_ASSUMED","PLATING_SOURCE_VERIFIED");
    const bool noPlatingPromotion=!decode(platingForgery,sha,source,&panel,
        {"RA","RB","RC","RD"},"NET_A",{1,4},ignored);
    auto spanForgery=wire;
    spanForgery.replace("SPAN|0|4e45545f41|","SPAN|0|4f54484552|");
    const bool noSpanForgery=!decode(spanForgery,sha,source,&panel,
        {"RA","RB","RC","RD"},"NET_A",{1,4},ignored);
    auto newBytes=source;
    newBytes.replace("U1.1 U2.2","U1.1  U2.2");
    if(!save(newBytes)||!load())return 2;
    const auto newSha=QCryptographicHash::hash(newBytes,QCryptographicHash::Sha256).toHex();
    c005::Receipt fresh;
    const bool stale=newSha!=sha&&panel.acceptedSourceSha256()==newSha
        &&panel.imported()->referenceRiskCountForTest()==0
        &&!decode(wire,sha,source,&panel,{"RA","RB","RC","RD"},"NET_A",{1,4},ignored)
        &&decode(query(),newSha,newBytes,&panel,{"RA","RB","RC","RD"},"NET_A",{1,4},fresh);
    auto invalid=source;
    invalid.replace("330 280 4 10 0","280 280 4 10 0");
    if(!save(invalid)||!load())return 2;
    const bool boundary=panel.imported()->referenceRiskCountForTest()==0&&query().isEmpty();
    const bool ok=accepted&&displayed&&noForgery&&noNodeForgery&&noPlatingPromotion&&noSpanForgery&&stale&&boundary;
    qInfo()<<"D012_NATIVE_QT_SOURCE_GRAPH"<<ok<<accepted<<displayed
        <<noForgery<<noNodeForgery<<noPlatingPromotion<<noSpanForgery<<stale<<boundary<<"PLATING_IMPORTER_ASSUMED_NO_ELECTRICAL_BARREL_EDGE";
    return ok?0:1;
}
} // namespace d012
