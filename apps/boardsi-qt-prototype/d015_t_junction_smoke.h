// D_015 real imported-source Qt adversarial test (no fabricated electrical proof).
#pragma once
#include "d015_t_junction_qt.h"
namespace d015 {
inline int selftest(const QString &importer) {
    if (!QFileInfo(importer).isExecutable()) return 2;
    QTemporaryFile file;
    if (!file.open()) return 2;
    const QByteArray source(
        "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n"
        "*PCB*\nMAXLAYER 4\n*VIA*\nSIMPLEVIA 15 2\n-2 35 R\n0 35 R\n"
        "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n"
        "80 250 1 10 0\n150 250 4 10 0 SIMPLEVIA\n"
        "350 250 1 10 0 SIMPLEVIA\n420 250 1 10 0\n"
        "*SIGNAL* NET_A\nU3.3 U4.4\n"
        "250 350 4 10 0\n250 250 4 10 0\n*END*\n");
    const auto save=[&](const QByteArray &bytes) {
        return file.resize(0) && file.seek(0) && file.write(bytes)==bytes.size()
            && file.flush();
    };
    if (!save(source)) return 2;
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    const auto load=[&]() {
        panel.setStatusCallback([&](const QString &status) {
            if (status.startsWith("PARTIAL") || status.startsWith("FAILED")) loop.quit();
        });
        if (!panel.openPath(file.fileName())) return false;
        QTimer deadline; deadline.setSingleShot(true);
        QObject::connect(&deadline,&QTimer::timeout,&loop,&QEventLoop::quit);
        deadline.start(8000);loop.exec();deadline.stop();
        return panel.importStatus().startsWith("PARTIAL");
    };
    const auto query=[&]() {
        QProcess proc;
        proc.start(importer,{file.fileName(),"--d015-t-junction-v1",
            "NET_A","1","4","0","1"});
        if (!proc.waitForFinished(10000)
            || proc.exitStatus()!=QProcess::NormalExit || proc.exitCode()!=0)
            return QByteArray();
        return proc.readAllStandardOutput();
    };
    if (!load()) return 2;
    c005::Receipt receipt,ignored;
    const auto wire=query();
    const bool accepted=!wire.isEmpty() && decode(wire,source,&panel,"NET_A",receipt)
        && receipt.state=="T_JUNCTION_SOURCE_CANDIDATE_NOT_ELECTRICAL"
        && wire.count("NODE|")==8 && wire.count("EDGE|")==5
        && wire.contains("LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0");
    if (!accepted) return 1;
    auto forged=wire;
    forged.replace("EDGE|4|5|7|SOURCE_INTERIOR_T_CONTACT_CANDIDATE|4",
                   "EDGE|4|3|7|SOURCE_INTERIOR_T_CONTACT_CANDIDATE|1");
    const bool rejectsForgery=!decode(forged,source,&panel,"NET_A",ignored);
    auto promoted=wire;
    promoted.replace("BARREL_ELECTRICAL_NOT_VERIFIED","BARREL_ELECTRICAL_VERIFIED");
    const bool rejectsPromotion=!decode(promoted,source,&panel,"NET_A",ignored);
    auto updated=source;
    updated.replace("U3.3 U4.4","U3.3  U4.4");
    if (!save(updated) || !load()) return 2;
    const bool stale=!decode(wire,source,&panel,"NET_A",ignored)
        && decode(query(),updated,&panel,"NET_A",ignored);
    auto oneTick=source;
    oneTick.replace("250 250 4 10 0","250 251 4 10 0");
    if (!save(oneTick) || !load()) return 2;
    const auto originalSha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    auto fakeTick=wire;
    fakeTick.replace(originalSha,QCryptographicHash::hash(oneTick,QCryptographicHash::Sha256).toHex());
    const bool rejectsMiss=!decode(fakeTick,oneTick,&panel,"NET_A",ignored)
        && query().isEmpty();
    auto overrun=source;
    overrun.replace("250 250 4 10 0","250 150 4 10 0");
    if (!save(overrun) || !load()) return 2;
    const bool rejectsOverrun=query().isEmpty();
    auto wrongLayer=source;
    wrongLayer.replace("250 350 4 10 0","250 350 1 10 0");
    if (!save(wrongLayer) || !load()) return 2;
    const bool rejectsLayer=query().isEmpty();
    auto extra=source;
    extra.replace("250 250 4 10 0","250 250 4 10 0\n270 270 4 10 0");
    if (!save(extra) || !load()) return 2;
    const bool rejectsExtra=query().isEmpty();
    const bool ok=accepted && rejectsForgery && rejectsPromotion && stale
        && rejectsMiss && rejectsOverrun && rejectsLayer && rejectsExtra;
    qInfo()<<"D015_NATIVE_QT_T_JUNCTION"<<ok<<accepted<<rejectsForgery
        <<rejectsPromotion<<stale<<rejectsMiss<<rejectsOverrun
        <<rejectsLayer<<rejectsExtra<<"NO_ELECTRICAL_BARREL_PROOF";
    return ok?0:1;
}
} // namespace d015
