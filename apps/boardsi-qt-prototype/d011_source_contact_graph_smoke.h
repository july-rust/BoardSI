// D_011 native Qt smoke: actually accepted PADS source and independently audited graph.
#pragma once
#include "d011_source_contact_graph_qt.h"
namespace d011 {
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
        p.start(importer,{f.fileName(),"--d011-source-contact-graph-v1",
            "RA","RB","RC","RD","NET_A","1","4","0"});
        if(!p.waitForFinished(10000)||p.exitStatus()!=QProcess::NormalExit
            || p.exitCode()!=0)return QByteArray();
        return p.readAllStandardOutput();
    };
