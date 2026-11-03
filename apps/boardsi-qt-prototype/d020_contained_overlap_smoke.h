// D_020 independent real-Qt strict Source-only contained collinear overlap.
#pragma once
#include "d015_t_junction_qt.h"
namespace d020 {
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
        "200 250 4 10 0\n250 250 4 10 0\n*END*\n");
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
        QTimer deadline;deadline.setSingleShot(true);
        QObject::connect(&deadline,&QTimer::timeout,&loop,&QEventLoop::quit);
        deadline.start(8000);loop.exec();deadline.stop();
        return panel.importStatus().startsWith("PARTIAL");
    };
    const auto query=[&](const QString &mode=QStringLiteral("--d020-contained-collinear-overlap-v1")) {
        QProcess proc;
        proc.start(importer,{file.fileName(),mode,"NET_A","1","4","0","1"});
        if (!proc.waitForFinished(10000) || proc.exitStatus()!=QProcess::NormalExit
            || proc.exitCode()!=0) return QByteArray();
        return proc.readAllStandardOutput();
    };
    if (!save(source) || !load()) return 2;
    c005::Receipt receipt,ignored;
    const auto wire=query();
    const bool accepted=!wire.isEmpty()
        && d015::decode_contained_overlap(wire,source,&panel,"NET_A",receipt)
        && receipt.state=="COLLINEAR_CONTAINED_OVERLAP_SOURCE_CANDIDATE_NOT_ELECTRICAL"
        && wire.count("NODE|")==8 && wire.count("EDGE|")==5
        && wire.contains("OVERLAP|MAIN_TRACE|1|CONTAINED_TRACE|3|START|20000|25000|END|25000|25000")
        && wire.contains("LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0");
    if (!accepted) return 1;
    const bool rejectsD015=query("--d015-t-junction-v1").isEmpty()
        && query("--d019-oblique-t-junction-v1").isEmpty();
    auto forged=wire;
    forged.replace("SOURCE_COLLINEAR_CONTAINED_OVERLAP_CANDIDATE|4",
                   "SOURCE_COLLINEAR_CONTAINED_OVERLAP_CANDIDATE|1");
    const bool rejectsForgery=!d015::decode_contained_overlap(forged,source,&panel,"NET_A",ignored);
    auto forgedInterval=wire;
    forgedInterval.replace("START|20000|25000|END|25000|25000","START|20000|25000|END|25100|25000");
    const bool rejectsInterval=!d015::decode_contained_overlap(forgedInterval,source,&panel,"NET_A",ignored);
    auto promoted=wire;
    promoted.replace("BARREL_ELECTRICAL_NOT_VERIFIED","BARREL_ELECTRICAL_VERIFIED");
    const bool rejectsPromotion=!d015::decode_contained_overlap(promoted,source,&panel,"NET_A",ignored);
    auto changed=source;
    changed.replace("U3.3 U4.4","U3.3  U4.4");
    if (!save(changed) || !load()) return 2;
    const bool stale=!d015::decode_contained_overlap(wire,source,&panel,"NET_A",ignored)
        && d015::decode_contained_overlap(query(),changed,&panel,"NET_A",ignored);
    auto miss=source;
    miss.replace("250 250 4 10 0","250 251 4 10 0");
    if (!save(miss) || !load()) return 2;
    auto forgedSha=wire;
    forgedSha.replace(QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex(),
        QCryptographicHash::hash(miss,QCryptographicHash::Sha256).toHex());
    const bool rejectsMiss=query().isEmpty()
        && !d015::decode_contained_overlap(forgedSha,miss,&panel,"NET_A",ignored);
    auto endpoint=source;
    endpoint.replace("200 250 4 10 0","150 250 4 10 0");
    if (!save(endpoint)) return 2;
    const bool rejectsEndpoint=query().isEmpty();
    auto partial=source;
    partial.replace("200 250 4 10 0","100 250 4 10 0");
    if (!save(partial)) return 2;
    const bool rejectsPartial=query().isEmpty();
    auto identical=source;
    identical.replace("200 250 4 10 0","150 250 4 10 0");
    identical.replace("250 250 4 10 0","350 250 4 10 0");
    if (!save(identical)) return 2;
    const bool rejectsIdentical=query().isEmpty();
    auto wrongLayer=source;
    wrongLayer.replace("200 250 4 10 0","200 250 1 10 0");
    if (!save(wrongLayer)) return 2;
    const bool rejectsLayer=query().isEmpty();
    auto wrongNet=source;
    wrongNet.replace("*SIGNAL* NET_A\nU3.3","*SIGNAL* OTHER\nU3.3");
    if (!save(wrongNet)) return 2;
    const bool rejectsNet=query().isEmpty();
    auto copper=source;
    copper.replace("*END*\n","*LINES*\nRA COPPER 0 0 1 1 0 OTHER\n"
        "COPCLS 5 0.01 0 4\n240 240\n350 240\n350 360\n240 360\n240 240\n*END*\n");
    if (!save(copper)) return 2;
    const bool rejectsCopper=query().isEmpty();
    auto reverse=source;
    reverse.replace("200 250 4 10 0\n250 250 4 10 0",
                    "250 250 4 10 0\n200 250 4 10 0");
    if (!save(reverse) || !load()) return 2;
    const auto reversedWire=query();
    const bool acceptsReverse=d015::decode_contained_overlap(reversedWire,reverse,&panel,"NET_A",ignored)
        && reversedWire.contains("START|20000|25000|END|25000|25000");
    auto decimal=source;
    decimal.replace("80 250 1 10 0","80.00 250 1 10 0");
    if (!save(decimal) || !load()) return 2;
    const bool acceptsDecimal=d015::decode_contained_overlap(query(),decimal,&panel,"NET_A",ignored);
    const bool ok=accepted && rejectsD015 && rejectsForgery && rejectsInterval
        && rejectsPromotion && stale && rejectsMiss && rejectsEndpoint
        && rejectsPartial && rejectsIdentical && rejectsLayer && rejectsNet
        && rejectsCopper && acceptsReverse && acceptsDecimal;
    qInfo()<<"D020_NATIVE_QT_CONTAINED_OVERLAP"<<ok<<accepted<<rejectsD015
        <<rejectsForgery<<rejectsInterval<<rejectsPromotion<<stale
        <<rejectsMiss<<rejectsEndpoint<<rejectsPartial<<rejectsIdentical
        <<rejectsLayer<<rejectsNet<<rejectsCopper<<acceptsReverse
        <<acceptsDecimal<<"NO_ELECTRICAL_PROOF";
    return ok?0:1;
}
} // namespace d020
