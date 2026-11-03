// D_017 Source-only strictly oblique X crossing; independent real-Qt reimport.
#pragma once
#include "d016_x_junction_qt.h"
namespace d017 {
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
        "220 350 4 10 0\n300 150 4 10 0\n*END*\n");
    const auto save=[&](const QByteArray &bytes) {
        return file.resize(0) && file.seek(0)
            && file.write(bytes)==bytes.size() && file.flush();
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
        proc.start(importer,{file.fileName(),"--d017-oblique-x-junction-v1",
            "NET_A","1","4","0","1"});
        if (!proc.waitForFinished(10000) || proc.exitStatus()!=QProcess::NormalExit
            || proc.exitCode()!=0) return QByteArray();
        return proc.readAllStandardOutput();
    };
    if (!save(source) || !load()) return 2;
    c005::Receipt receipt,ignored;
    const auto wire=query();
    const bool accepted=!wire.isEmpty()
        && d016::decode_oblique(wire,source,&panel,"NET_A",receipt)
        && receipt.state=="OBLIQUE_X_JUNCTION_SOURCE_CANDIDATE_NOT_ELECTRICAL"
        && wire.count("NODE|")==8 && wire.count("EDGE|")==5
        && wire.contains("LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0");
    if (!accepted) return 1;
    auto forged=wire;
    forged.replace("SOURCE_INTERIOR_OBLIQUE_X_CONTACT_CANDIDATE|4",
                   "SOURCE_INTERIOR_OBLIQUE_X_CONTACT_CANDIDATE|1");
    const bool rejectsForgery=!d016::decode_oblique(forged,source,&panel,"NET_A",ignored);
    auto promoted=wire;
    promoted.replace("BARREL_ELECTRICAL_NOT_VERIFIED","BARREL_ELECTRICAL_VERIFIED");
    const bool rejectsPromotion=!d016::decode_oblique(promoted,source,&panel,"NET_A",ignored);
    const auto sha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    auto modified=source;
    modified.replace("U3.3 U4.4","U3.3  U4.4");
    if (!save(modified) || !load()) return 2;
    const bool stale=!d016::decode_oblique(wire,source,&panel,"NET_A",ignored)
        && d016::decode_oblique(query(),modified,&panel,"NET_A",ignored);
    auto miss=source;
    miss.replace("300 150 4 10 0","300 251 4 10 0");
    if (!save(miss) || !load()) return 2;
    auto forgedSha=wire;
    forgedSha.replace(sha,QCryptographicHash::hash(miss,QCryptographicHash::Sha256).toHex());
    const bool rejectsMiss=query().isEmpty()
        && !d016::decode_oblique(forgedSha,miss,&panel,"NET_A",ignored);
    auto t=source;
    t.replace("300 150 4 10 0","250 250 4 10 0");
    if (!save(t)) return 2;
    const bool rejectsT=query().isEmpty();
    auto rightAngle=source;
    rightAngle.replace("220 350 4 10 0","260 350 4 10 0");
    rightAngle.replace("300 150 4 10 0","260 150 4 10 0");
    if (!save(rightAngle)) return 2;
    const bool rejectsPerpendicular=query().isEmpty();
    auto wrongLayer=source;
    wrongLayer.replace("220 350 4 10 0","220 350 1 10 0");
    if (!save(wrongLayer)) return 2;
    const bool rejectsLayer=query().isEmpty();
    auto wrongNet=source;
    wrongNet.replace("*SIGNAL* NET_A\nU3.3","*SIGNAL* OTHER\nU3.3");
    if (!save(wrongNet)) return 2;
    const bool rejectsNet=query().isEmpty();
    auto collinear=source;
    collinear.replace("220 350 4 10 0","200 250 4 10 0");
    collinear.replace("300 150 4 10 0","300 250 4 10 0");
    if (!save(collinear)) return 2;
    const bool rejectsCollinear=query().isEmpty();
    auto extra=source;
    extra.replace("*ROUTE*","SIMPLEVIA 15 2\n-2 35 R\n0 35 R\n*ROUTE*");
    if (!save(extra)) return 2;
    const bool rejectsExtraVia=query().isEmpty();
    auto copper=source;
    copper.replace("*END*\n","*LINES*\nRA COPPER 0 0 1 1 0 OTHER\n"
        "COPCLS 5 0.01 0 4\n240 240\n350 240\n350 360\n240 360\n240 240\n*END*\n");
    if (!save(copper)) return 2;
    const bool rejectsCopper=query().isEmpty();
    auto decimal=source;
    decimal.replace("80 250 1 10 0","80.00 250 1 10 0");
    if (!save(decimal) || !load()) return 2;
    const bool acceptsDecimal=d016::decode_oblique(query(),decimal,&panel,"NET_A",ignored);
    const bool ok=accepted && rejectsForgery && rejectsPromotion && stale
        && rejectsMiss && rejectsT && rejectsPerpendicular && rejectsLayer
        && rejectsNet && rejectsCollinear && rejectsExtraVia && rejectsCopper
        && acceptsDecimal;
    qInfo()<<"D017_NATIVE_QT_OBLIQUE_X"<<ok<<accepted<<rejectsForgery
        <<rejectsPromotion<<stale<<rejectsMiss<<rejectsT<<rejectsPerpendicular
        <<rejectsLayer<<rejectsNet<<rejectsCollinear<<rejectsExtraVia
        <<rejectsCopper<<acceptsDecimal<<"NO_ELECTRICAL_BARREL_PROOF";
    return ok?0:1;
}
} // namespace d017
