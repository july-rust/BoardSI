// D_016 real PADS Source X-intersection independent native Qt adversarial suite.
#pragma once
#include "d016_x_junction_qt.h"
#include "d017_oblique_x_junction_smoke.h"
namespace d016 {
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
        "250 350 4 10 0\n250 150 4 10 0\n*END*\n");
    const auto save=[&](const QByteArray &data) {
        return file.resize(0)&&file.seek(0)&&file.write(data)==data.size()&&file.flush();
    };
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    const auto load=[&]() {
        panel.setStatusCallback([&](const QString &s) {
            if (s.startsWith("PARTIAL") || s.startsWith("FAILED")) loop.quit();
        });
        if (!panel.openPath(file.fileName())) return false;
        QTimer deadline; deadline.setSingleShot(true);
        QObject::connect(&deadline,&QTimer::timeout,&loop,&QEventLoop::quit);
        deadline.start(8000);loop.exec();deadline.stop();
        return panel.importStatus().startsWith("PARTIAL");
    };
    const auto query=[&]() {
        QProcess proc;
        proc.start(importer,{file.fileName(),"--d016-x-junction-v1",
            "NET_A","1","4","0","1"});
        if (!proc.waitForFinished(10000) || proc.exitStatus()!=QProcess::NormalExit
            || proc.exitCode()!=0) return QByteArray();
        return proc.readAllStandardOutput();
    };
    if (!save(source) || !load()) return 2;
    c005::Receipt receipt, ignored;
    const auto wire=query();
    const bool accepted=!wire.isEmpty() && decode(wire,source,&panel,"NET_A",receipt)
        && receipt.state=="X_JUNCTION_SOURCE_CANDIDATE_NOT_ELECTRICAL"
        && wire.count("NODE|")==8 && wire.count("EDGE|")==5
        && wire.contains("LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0");
    if (!accepted) return 1;
    auto forged=wire;
    forged.replace("SOURCE_INTERIOR_X_CONTACT_CANDIDATE|4",
                   "SOURCE_INTERIOR_X_CONTACT_CANDIDATE|1");
    const bool rejectsForgery=!decode(forged,source,&panel,"NET_A",ignored);
    auto promoted=wire;
    promoted.replace("BARREL_ELECTRICAL_NOT_VERIFIED","BARREL_ELECTRICAL_VERIFIED");
    const bool rejectsPromotion=!decode(promoted,source,&panel,"NET_A",ignored);
    auto updated=source;
    updated.replace("U3.3 U4.4","U3.3  U4.4");
    if (!save(updated) || !load()) return 2;
    const bool stale=!decode(wire,source,&panel,"NET_A",ignored)
        && decode(query(),updated,&panel,"NET_A",ignored);
    const auto originalSha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    auto nearMiss=source;
    nearMiss.replace("250 150 4 10 0","250 251 4 10 0");
    if (!save(nearMiss) || !load()) return 2;
    auto rehashed=wire;
    rehashed.replace(originalSha,QCryptographicHash::hash(nearMiss,QCryptographicHash::Sha256).toHex());
    const bool rejectsMiss=!decode(rehashed,nearMiss,&panel,"NET_A",ignored)
        && query().isEmpty();
    auto tJunction=source;
    tJunction.replace("250 150 4 10 0","250 250 4 10 0");
    if (!save(tJunction) || !load()) return 2;
    const bool rejectsT=query().isEmpty();
    auto wrongLayer=source;
    wrongLayer.replace("250 350 4 10 0","250 350 1 10 0");
    if (!save(wrongLayer) || !load()) return 2;
    const bool rejectsLayer=query().isEmpty();
    auto copper=source;
    copper.replace("*END*\n","*LINES*\nRA COPPER 0 0 1 1 0 OTHER\n"
        "COPCLS 5 0.01 0 4\n240 240\n350 240\n350 360\n240 360\n240 240\n*END*\n");
    if (!save(copper)) return 2;
    const bool copperLoaded=load();
    auto fakeCopper=wire;
    fakeCopper.replace(originalSha,QCryptographicHash::hash(copper,QCryptographicHash::Sha256).toHex());
    const bool rejectsCopper=query().isEmpty()
        && (!copperLoaded || !decode(fakeCopper,copper,&panel,"NET_A",ignored));
    auto decimal=source;
    decimal.replace("80 250 1 10 0","80.00 250 1 10 0");
    if (!save(decimal) || !load()) return 2;
    const auto decimalWire=query();
    const bool decimalAccepted=!decimalWire.isEmpty()
        && decode(decimalWire,decimal,&panel,"NET_A",ignored);
    const bool ok=accepted&&rejectsForgery&&rejectsPromotion&&stale&&rejectsMiss
        && rejectsT&&rejectsLayer&&rejectsCopper&&decimalAccepted;
    qInfo()<<"D016_NATIVE_QT_X_JUNCTION"<<ok<<accepted<<rejectsForgery
        <<rejectsPromotion<<stale<<rejectsMiss<<rejectsT<<rejectsLayer
        <<rejectsCopper<<decimalAccepted<<"NO_ELECTRICAL_BARREL_PROOF";
    return ok?d017::selftest(importer):1;
}
} // namespace d016
