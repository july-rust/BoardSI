// D_023 independent native Qt endpoint Source contact and Via-pad triangle proof.
#pragma once
#include "d015_t_junction_qt.h"
namespace d023 {
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
        "150 250 4 10 0\n50 250 4 10 0\n*END*\n");
    const auto save=[&](const QByteArray &b) {
        return file.resize(0) && file.seek(0) && file.write(b)==b.size() && file.flush();
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
    const auto query=[&](const QString &mode=QStringLiteral("--d023-collinear-opposite-endpoint-contact-v1")) {
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
        && d015::decode_opposite_endpoint_contact(wire,source,&panel,"NET_A",receipt)
        && receipt.state=="COLLINEAR_OPPOSITE_ENDPOINT_CONTACT_SOURCE_CANDIDATE_NOT_ELECTRICAL"
        && wire.count("NODE|")==8 && wire.count("EDGE|")==6
        && wire.contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|X|15000|Y|25000")
        && wire.contains("EDGE|5|1|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4")
        && wire.contains("COUNT|NODES|8|LAYER_LOCAL_EDGES|6|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0");
    if (!accepted) return 1;
    const bool oldModes=query("--d022-collinear-endpoint-contact-v1").isEmpty()
        && query("--d015-t-junction-v1").isEmpty()
        && query("--d019-oblique-t-junction-v1").isEmpty()
        && query("--d020-contained-collinear-overlap-v1").isEmpty()
        && query("--d021-partial-collinear-overlap-v1").isEmpty();
    auto forged=wire;
    forged.replace("EDGE|5|1|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4",
                   "EDGE|5|2|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|1");
    const bool rejectsForgery=!d015::decode_opposite_endpoint_contact(forged,source,&panel,"NET_A",ignored);
    auto forgedContact=wire;
    forgedContact.replace("X|15000|Y|25000","X|15001|Y|25000");
    const bool rejectsCoordinate=!d015::decode_opposite_endpoint_contact(forgedContact,source,&panel,"NET_A",ignored);
    auto promoted=wire;
    promoted.replace("BARREL_ELECTRICAL_NOT_VERIFIED","BARREL_ELECTRICAL_VERIFIED");
    const bool rejectsPromotion=!d015::decode_opposite_endpoint_contact(promoted,source,&panel,"NET_A",ignored);
    auto changed=source;
    changed.replace("U3.3 U4.4","U3.3  U4.4");
    if (!save(changed) || !load()) return 2;
    const bool rejectsStale=!d015::decode_opposite_endpoint_contact(wire,source,&panel,"NET_A",ignored)
        && d015::decode_opposite_endpoint_contact(query(),changed,&panel,"NET_A",ignored);
    bool rejectsHostiles=true;
    for (auto bad : {
        QByteArray(source).replace("150 250 4 10 0","150.01 250 4 10 0"),
        QByteArray(source).replace("150 250 4 10 0","150 250.01 4 10 0"),
        QByteArray(source).replace("50 250 4 10 0","50 250.01 4 10 0"),
        QByteArray(source).replace("150 250 4 10 0","100 250 4 10 0"),
        QByteArray(source).replace("50 250 4 10 0","150 250 4 10 0"),
        QByteArray(source).replace("150 250 4 10 0","350 250 4 10 0"),
        QByteArray(source).replace("150 250 4 10 0","150 250 1 10 0"),
        QByteArray(source).replace("*SIGNAL* NET_A\nU3.3","*SIGNAL* OTHER\nU3.3"),
        QByteArray(source).replace("*END*","*LINES*\nCOPCLS 5 0.01 0 4\n*END*"),
    }) {
        if (!save(bad) || !query().isEmpty()) rejectsHostiles=false;
    }
    auto reverse=source;
    reverse.replace("150 250 4 10 0\n50 250 4 10 0",
                    "50 250 4 10 0\n150 250 4 10 0");
    if (!save(reverse) || !load()) return 2;
    const bool reversed=d015::decode_opposite_endpoint_contact(query(),reverse,&panel,"NET_A",ignored);
    auto decimal=source;
    decimal.replace("80 250 1 10 0","80.00 250 1 10 0");
    if (!save(decimal) || !load()) return 2;
    const bool lexical=d015::decode_opposite_endpoint_contact(query(),decimal,&panel,"NET_A",ignored);
    const bool ok=accepted && oldModes && rejectsForgery && rejectsCoordinate
        && rejectsPromotion && rejectsStale && rejectsHostiles && reversed && lexical;
    qInfo()<<"D023_NATIVE_QT_ENDPOINT_SOURCE"<<ok<<accepted<<oldModes
        <<rejectsForgery<<rejectsCoordinate<<rejectsPromotion<<rejectsStale
        <<rejectsHostiles<<reversed<<lexical<<"NO_ELECTRICAL_PROOF";
    return ok?0:1;
}
} // namespace d023
