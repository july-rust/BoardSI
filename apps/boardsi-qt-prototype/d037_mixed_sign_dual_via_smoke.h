// D_037 Native Qt SOURCE-only dual-branch regression and fail-closed verification.
#pragma once
#include "d037_mixed_sign_dual_via_qt.h"

namespace d037 {
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
        "150 250 4 10 0\n30 160 4 10 0\n"
        "*SIGNAL* NET_A\nU5.5 U6.6\n"
        "350 250 4 10 0\n470 340 4 10 0\n*END*\n");
    const auto save=[&](const QByteArray &bytes) {
        return file.resize(0) && file.seek(0)
            && file.write(bytes)==bytes.size() && file.flush();
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
    const auto query=[&](const QString &mode=QStringLiteral(
            "--d037-non45-mixed-sign-dual-via-endpoint-contact-v1")) {
        QProcess proc;
        proc.start(importer,{file.fileName(),mode,"NET_A","1","4","0","1"});
        if (!proc.waitForFinished(10000) || proc.exitStatus()!=QProcess::NormalExit
            || proc.exitCode()!=0) return QByteArray();
        return proc.readAllStandardOutput();
    };
    if (!save(source) || !load()) return 2;
    c005::Receipt acceptedReceipt,ignored;
    const auto wire=query();
    const bool accepted=!wire.isEmpty()
        && decode(wire,source,&panel,"NET_A",acceptedReceipt)
        && acceptedReceipt.state=="DUAL_MIXED_SIGN_VIA_ENDPOINT_SOURCE_CANDIDATE_NOT_ELECTRICAL"
        && wire.count("NODE|")==9 && wire.count("EDGE|")==8
        && wire.contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|VIA|A|X|15000|Y|25000")
        && wire.contains("CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|4|VIA|B|X|35000|Y|25000")
        && wire.contains("COUNT|NODES|9|LAYER_LOCAL_EDGES|8|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0");
    if (!accepted) return 1;
    bool forgedOk=true;
    const QList<QPair<QByteArray,QByteArray>> tamper = {
        {"EDGE|7|2|8|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4",
         "EDGE|7|1|8|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4"},
        {"EDGE|5|1|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4",
         "EDGE|5|2|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4"},
        {"LOCAL_COMPONENTS|3","LOCAL_COMPONENTS|2"},
        {"BARREL_ELECTRICAL_NOT_VERIFIED","BARREL_ELECTRICAL_VERIFIED"},
        {"SOLVER_READY_FALSE","SOLVER_READY_TRUE"},
        {"CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|VIA|A|X|15000",
         "CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|VIA|A|X|15001"}
    };
    for (const auto &pair:tamper) {
        auto counterfeit=wire;
        counterfeit.replace(pair.first,pair.second);
        forgedOk=forgedOk && counterfeit!=wire
            && !decode(counterfeit,source,&panel,"NET_A",ignored);
    }
    const bool rejectsOldModes=
        query("--d036-non45-dual-mirrored-via-endpoint-contact-v1").isEmpty()
        &&         query("--d034-non45-oblique-mirrored-via-b-endpoint-contact-v1").isEmpty()
        && query("--d035-non45-oblique-mirrored-via-a-endpoint-contact-v1").isEmpty()
        && query("--d033-non45-steep-mirrored-via-a-endpoint-contact-v1").isEmpty()
        && query("--d029-non45-oblique-via-a-endpoint-contact-v1").isEmpty()
        && query("--d015-t-junction-v1").isEmpty();
    auto changed=source;
    changed.replace("U3.3 U4.4","U3.3  U4.4");
    if (!save(changed) || !load()) return 2;
    const bool stale=!decode(wire,source,&panel,"NET_A",ignored)
        && decode(query(),changed,&panel,"NET_A",ignored);

    bool hostileRust=true;
    for (const auto &bad : {
        QByteArray(source).replace("30 160 4 10 0","30 160.01 4 10 0"),
        QByteArray(source).replace("470 340 4 10 0","470 340.01 4 10 0"),
        QByteArray(source).replace("30 160 4 10 0","30 340 4 10 0"),
        QByteArray(source).replace("470 340 4 10 0","470 160 4 10 0"),
        QByteArray(source).replace("150 250 4 10 0\n30 160","150.01 250 4 10 0\n30 160"),
        QByteArray(source).replace("350 250 4 10 0\n470 340","350.01 250 4 10 0\n470 340"),
        QByteArray(source).replace("*SIGNAL* NET_A\nU5.5","*SIGNAL* OTHER\nU5.5"),
        QByteArray(source).replace("*END*","*LINES*\nCOPCLS 5 0.01 0 4\n*END*")
    }) {
        if (bad==source || !save(bad) || !query().isEmpty()) hostileRust=false;
    }
    // Fresh Qt reimport + counterfeit SHA and exact changed Track:
    // verifies Qt geometry itself rejects the hostile (not merely a stale SHA).
    const auto originalSha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    const QByteArray traceA="TRACE|3|4|15000|25000|3000|16000|1000\n";
    const QByteArray traceB="TRACE|4|4|35000|25000|47000|16000|1000\n";
    const auto qtHostile=[&](QByteArray bad,QByteArray oldTrack,QByteArray newTrack) {
        if (bad==source || !save(bad) || !load()) return false;
        const auto hostileSha=QCryptographicHash::hash(bad,QCryptographicHash::Sha256).toHex();
        if (hostileSha==originalSha || panel.acceptedSourceSha256()!=hostileSha
            || !wire.contains(oldTrack) || oldTrack==newTrack) return false;
        auto counterfeit=wire;
        counterfeit.replace(originalSha,hostileSha);
        counterfeit.replace(oldTrack,newTrack);
        return counterfeit!=wire && query().isEmpty()
            && !decode(counterfeit,bad,&panel,"NET_A",ignored);
    };
    const bool qtGeometry=qtHostile(
            QByteArray(source).replace("30 160 4 10 0","30 160.01 4 10 0"),
            traceA,"TRACE|3|4|15000|25000|3000|16001|1000\n")
        && qtHostile(
            QByteArray(source).replace("470 340 4 10 0","470 340.01 4 10 0"),
            traceB,"TRACE|4|4|35000|25000|47000|16001|1000\n")
        && qtHostile(
            QByteArray(source).replace("470 340 4 10 0","470 160 4 10 0"),
            traceB,"TRACE|4|4|35000|25000|47000|34000|1000\n");
    auto reversed=source;
    reversed.replace("150 250 4 10 0\n30 160 4 10 0",
                     "30 160 4 10 0\n150 250 4 10 0");
    reversed.replace("350 250 4 10 0\n470 340 4 10 0",
                     "470 340 4 10 0\n350 250 4 10 0");
    if (!save(reversed) || !load()) return 2;
    const bool bothReversed=decode(query(),reversed,&panel,"NET_A",ignored);
    auto lexical=source;
    lexical.replace("80 250 1 10 0","80.00 250 1 10 0");
    if (!save(lexical) || !load()) return 2;
    const bool sourceLexical=decode(query(),lexical,&panel,"NET_A",ignored);
    const bool ok=accepted && forgedOk && rejectsOldModes && stale && hostileRust
        && qtGeometry && bothReversed && sourceLexical;
    qInfo()<<"D037_NATIVE_QT_DUAL_MIXED_SIGN_VIA_SOURCE"<<ok<<accepted
        <<forgedOk<<rejectsOldModes<<stale<<hostileRust<<qtGeometry
        <<bothReversed<<sourceLexical<<"NO_ELECTRICAL_PROOF";
    return ok?0:1;
}
} // namespace d037
