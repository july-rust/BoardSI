// D_031 source-tick exact 3:4 non-45-degree Via-A endpoint native Qt qualification.
// SOURCE GEOMETRY ONLY; never copper, barrel or whole-board electrical proof.
#pragma once
#include "d015_t_junction_qt.h"
namespace d031 {
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
        "150 250 4 10 0\n60 370 4 10 0\n*END*\n");
    const auto save=[&](const QByteArray &bytes) {
        return file.resize(0) && file.seek(0) && file.write(bytes)==bytes.size() && file.flush();
    };
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    const auto load=[&]() {
        panel.setStatusCallback([&](const QString &status) {
            if (status.startsWith("PARTIAL") || status.startsWith("FAILED")) loop.quit();
        });
        if (!panel.openPath(file.fileName())) return false;
        QTimer deadline;
        deadline.setSingleShot(true);
        QObject::connect(&deadline,&QTimer::timeout,&loop,&QEventLoop::quit);
        deadline.start(8000);
        loop.exec();
        deadline.stop();
        return panel.importStatus().startsWith("PARTIAL");
    };
    const auto query=[&](const QString &mode=QStringLiteral("--d031-non45-steep-via-a-endpoint-contact-v1")) {
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
        && d015::decode_non45_steep_via_a_endpoint_contact(wire,source,&panel,"NET_A",receipt)
        && receipt.state=="NON45_STEEP_VIA_A_ENDPOINT_CONTACT_SOURCE_CANDIDATE_NOT_ELECTRICAL"
        && wire.count("NODE|")==8 && wire.count("EDGE|")==6
        && wire.contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|X|15000|Y|25000")
        && wire.contains("EDGE|4|5|7|SOURCE_NON45_STEEP_VIA_A_ENDPOINT_CONTACT_CANDIDATE|4")
        && wire.contains("EDGE|5|1|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4")
        && wire.contains("COUNT|NODES|8|LAYER_LOCAL_EDGES|6|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0");
    if (!accepted) return 1;
    const bool rejectsOldQtMode=!d015::decode_non45_steep_via_b_endpoint_contact(wire,source,&panel,"NET_A",ignored)
        && !d015::decode_oblique_via_b_endpoint_contact(wire,source,&panel,"NET_A",ignored)
        && !d015::decode_non45_oblique_via_b_endpoint_contact(wire,source,&panel,"NET_A",ignored)
        && !d015::decode_non45_oblique_via_a_endpoint_contact(wire,source,&panel,"NET_A",ignored);
    const bool legacyModes=query("--d030-non45-steep-via-b-endpoint-contact-v1").isEmpty()
        && query("--d028-non45-oblique-via-b-endpoint-contact-v1").isEmpty()
        && query("--d029-non45-oblique-via-a-endpoint-contact-v1").isEmpty()
        && query("--d026-oblique-via-b-endpoint-contact-v1").isEmpty()
        && query("--d027-oblique-via-a-endpoint-contact-v1").isEmpty()
        && query("--d025-orthogonal-via-a-endpoint-contact-v1").isEmpty()
        && query("--d024-orthogonal-endpoint-contact-v1").isEmpty()
        && query("--d023-collinear-opposite-endpoint-contact-v1").isEmpty()
        && query("--d022-collinear-endpoint-contact-v1").isEmpty()
        && query("--d015-t-junction-v1").isEmpty()
        && query("--d019-oblique-t-junction-v1").isEmpty()
        && query("--d020-contained-collinear-overlap-v1").isEmpty()
        && query("--d021-partial-collinear-overlap-v1").isEmpty();
    auto forged=wire;
    forged.replace("EDGE|5|1|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4",
                   "EDGE|5|2|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4");
    // Regression for the former D_027 no-op forgery test: replacement must change bytes.
    const bool rejectsForgery=forged!=wire
        && !d015::decode_non45_steep_via_a_endpoint_contact(forged,source,&panel,"NET_A",ignored);
    auto wrongCoordinate=wire;
    wrongCoordinate.replace("X|15000|Y|25000","X|15001|Y|25000");
    const bool rejectsCoordinate=!d015::decode_non45_steep_via_a_endpoint_contact(wrongCoordinate,source,&panel,"NET_A",ignored);
    auto promoted=wire;
    promoted.replace("BARREL_ELECTRICAL_NOT_VERIFIED","BARREL_ELECTRICAL_VERIFIED");
    const bool rejectsPromotion=!d015::decode_non45_steep_via_a_endpoint_contact(promoted,source,&panel,"NET_A",ignored);
    auto altered=source;
    altered.replace("U3.3 U4.4","U3.3  U4.4");
    if (!save(altered) || !load()) return 2;
    const bool rejectsStale=!d015::decode_non45_steep_via_a_endpoint_contact(wire,source,&panel,"NET_A",ignored)
        && d015::decode_non45_steep_via_a_endpoint_contact(query(),altered,&panel,"NET_A",ignored);
    bool rejectsHostiles=true;
    for (const auto &bad : {
        QByteArray(source).replace("150 250 4 10 0\n60 370 4 10 0","150.01 250 4 10 0\n60 370 4 10 0"),
        QByteArray(source).replace("150 250 4 10 0\n60 370 4 10 0","150 250.01 4 10 0\n60 370 4 10 0"),
        QByteArray(source).replace("60 370 4 10 0","60.01 370 4 10 0"),
        QByteArray(source).replace("60 370 4 10 0","60 370.01 4 10 0"),
        QByteArray(source).replace("60 370 4 10 0","30 250 4 10 0"),
        QByteArray(source).replace("60 370 4 10 0","60 340 4 10 0"),
        QByteArray(source).replace("60 370 4 10 0","30 340 4 10 0"),
        QByteArray(source).replace("60 370 4 10 0","150 340 4 10 0"),
        QByteArray(source).replace("60 370 4 10 0","350 250 4 10 0"),
        QByteArray(source).replace("60 370 4 10 0","230 340 4 10 0"),
        QByteArray(source).replace("150 250 4 10 0\n60 370 4 10 0","150 300 4 10 0\n60 370 4 10 0"),
        QByteArray(source).replace("150 250 4 10 0\n60 370 4 10 0","150 250 1 10 0\n60 370 1 10 0"),
        QByteArray(source).replace("*SIGNAL* NET_A\nU3.3","*SIGNAL* OTHER\nU3.3"),
        QByteArray(source).replace("*END*","*LINES*\nCOPCLS 5 0.01 0 4\n*END*")
    }) {
        if (bad==source || !save(bad) || !query().isEmpty()) rejectsHostiles=false;
    }
    // Independent Qt-hostile verification: import each hostile Source, bind the
    // counterfeit wire to its REAL SHA and track coordinates, then ensure the
    // Qt decoder itself rejects the geometry (not merely a stale receipt).
    const auto originalSha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    const QByteArray originalTrack="TRACE|3|4|15000|25000|6000|37000|10\\n";
    const auto rejectsQtReimport=[&](const QByteArray &bad, const QByteArray &hostileTrack) {
        if (bad==source || !save(bad) || !load()) return false;
        const auto hostileSha=QCryptographicHash::hash(bad,QCryptographicHash::Sha256).toHex();
        if (hostileSha==originalSha || panel.acceptedSourceSha256()!=hostileSha
            || wire.count(originalSha)!=1 || !wire.contains(originalTrack)
            || !hostileTrack.startsWith("TRACE|3|4|")) return false;
        auto counterfeit=wire;
        counterfeit.replace(originalSha,hostileSha);
        counterfeit.replace(originalTrack,hostileTrack);
        // If Qt were to drop its D031 predicate, this exact constructed wire
        // would otherwise match its rebuilt Source witness and get accepted.
        return counterfeit!=wire && query().isEmpty()
            && !d015::decode_non45_steep_via_a_endpoint_contact(
                counterfeit,bad,&panel,"NET_A",ignored);
    };
    const bool rejectsQtGeometryHostiles=
        rejectsQtReimport(QByteArray(source).replace(
            "150 250 4 10 0\\n60 370 4 10 0",
            "150.01 250 4 10 0\\n60 370 4 10 0"),
            "TRACE|3|4|15001|25000|6000|37000|10\\n")
        && rejectsQtReimport(QByteArray(source).replace(
            "60 370 4 10 0","60.01 370 4 10 0"),
            "TRACE|3|4|15000|25000|6001|37000|10\\n")
        && rejectsQtReimport(QByteArray(source).replace(
            "60 370 4 10 0","30 340 4 10 0"),
            "TRACE|3|4|15000|25000|3000|34000|10\\n")
        && rejectsQtReimport(QByteArray(source).replace(
            "60 370 4 10 0","240 370 4 10 0"),
            "TRACE|3|4|15000|25000|24000|37000|10\\n")
        && rejectsQtReimport(QByteArray(source).replace(
            "*SIGNAL* NET_A\\nU3.3","*SIGNAL* OTHER\\nU3.3"),originalTrack);
    auto reversedSource=source;
    reversedSource.replace("150 250 4 10 0\n60 370 4 10 0",
                           "60 370 4 10 0\n150 250 4 10 0");
    if (!save(reversedSource) || !load()) return 2;
    const bool reversed=d015::decode_non45_steep_via_a_endpoint_contact(query(),reversedSource,&panel,"NET_A",ignored);
    auto decimal=source;
    decimal.replace("80 250 1 10 0","80.00 250 1 10 0");
    if (!save(decimal) || !load()) return 2;
    const bool lexical=d015::decode_non45_steep_via_a_endpoint_contact(query(),decimal,&panel,"NET_A",ignored);
    const bool ok=accepted && legacyModes && rejectsOldQtMode && rejectsForgery && rejectsCoordinate
        && rejectsPromotion && rejectsStale && rejectsHostiles && rejectsQtGeometryHostiles && reversed && lexical;
    qInfo()<<"D031_NATIVE_QT_NON45_STEEP_VIA_A_SOURCE"<<ok<<accepted<<legacyModes
        <<rejectsOldQtMode<<rejectsForgery<<rejectsCoordinate<<rejectsPromotion<<rejectsStale
        <<rejectsHostiles<<rejectsQtGeometryHostiles<<reversed<<lexical<<"NO_ELECTRICAL_PROOF";
    return ok?0:1;
}
} // namespace d031
