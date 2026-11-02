// D_014: independently reimport real public synthetic PADS Source on Qt.
#pragma once
#include "d014_two_via_qt.h"
namespace d014 {
inline int selftest(const QString &importer){
    if(!QFileInfo(importer).isExecutable())return 2;
    QTemporaryFile file;
    if(!file.open())return 2;
    const QByteArray source(
        "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n"
        "*PCB*\nMAXLAYER 4\n*VIA*\nSIMPLEVIA 15 2\n-2 35 R\n0 35 R\n"
        "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n"
        "80 250 1 10 0\n150 250 4 10 0 SIMPLEVIA\n"
        "350 250 1 10 0 SIMPLEVIA\n420 250 1 10 0\n*END*\n");
    const auto save=[&](const QByteArray &data){
        return file.resize(0)&&file.seek(0)&&file.write(data)==data.size()&&file.flush();
    };
    if(!save(source))return 2;
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    const auto load=[&](){
        panel.setStatusCallback([&](const QString &s){
            if(s.startsWith("PARTIAL")||s.startsWith("FAILED"))loop.quit();
        });
        if(!panel.openPath(file.fileName()))return false;
        QTimer deadline;deadline.setSingleShot(true);
        QObject::connect(&deadline,&QTimer::timeout,&loop,&QEventLoop::quit);
        deadline.start(8000);loop.exec();deadline.stop();
        return panel.importStatus().startsWith("PARTIAL");
    };
    const auto query=[&](){
        QProcess p;
        p.start(importer,{file.fileName(),"--d014-two-via-source-chain-v1",
            "NET_A","1","4","0","1"});
        if(!p.waitForFinished(10000)||p.exitStatus()!=QProcess::NormalExit
           || p.exitCode()!=0)return QByteArray();
        return p.readAllStandardOutput();
    };
    if(!load())return 2;
    c005::Receipt first,ignored;
    const auto wire=query();
    const bool accepted=!wire.isEmpty() && decode(wire,source,&panel,"NET_A",first)
       && first.state=="TWO_VIA_SOURCE_CANDIDATE_NOT_ELECTRICAL";
    if(!accepted)return 1;
    const bool twoVias=wire.count("VIA|")==2;
    auto forgery=wire;
    forgery.replace("EDGE|2|2|5|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4",
                    "EDGE|2|2|4|SOURCE_ENDPOINT_CONTACT_CANDIDATE|1");
    const bool rejectsForgery=!decode(forgery,source,&panel,"NET_A",ignored);
    auto promotion=wire;
    promotion.replace("BARREL_ELECTRICAL_NOT_VERIFIED","BARREL_ELECTRICAL_VERIFIED");
    const bool rejectsPromotion=!decode(promotion,source,&panel,"NET_A",ignored);
    QByteArray updated=source;
    updated.replace("U1.1 U2.2","U1.1  U2.2");
    if(!save(updated)||!load())return 2;
    const bool stale=panel.acceptedSourceSha256()!=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex()
        && !decode(wire,source,&panel,"NET_A",ignored)
        && decode(query(),updated,&panel,"NET_A",ignored);
    // Forge a SHA-consistent witness for a three-row interior padstack:
    // Rust excludes inner pads, and Qt independently enforces that exclusion.
    auto inner=source;
    inner.replace("SIMPLEVIA 15 2\n-2 35 R\n0 35 R\n",
                  "SIMPLEVIA 15 3\n-2 35 R\n0 35 R\n-1 35 R\n");
    if(!save(inner)||!load())return 2;
    const auto origSha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    const auto innerSha=QCryptographicHash::hash(inner,QCryptographicHash::Sha256).toHex();
    auto forgedPad=wire;
    forgedPad.replace(origSha,innerSha);
    const bool rejectsInnerPad=!decode(forgedPad,inner,&panel,"NET_A",ignored)
        && query().isEmpty();
    // Codex regression: a free arm endpoint at the opposite Via introduces
    // an unrepresented contact and must be rejected, not counted as 3 islands.
    auto extra=source;
    extra.replace("80 250 1 10 0","350 250 1 10 0");
    if(!save(extra)||!load())return 2;
    auto forgedExtra=wire;
    forgedExtra.replace(origSha,QCryptographicHash::hash(extra,QCryptographicHash::Sha256).toHex());
    forgedExtra.replace("TRACE|0|1|8000|25000","TRACE|0|1|35000|25000");
    const bool rejectsExtra=!decode(forgedExtra,extra,&panel,"NET_A",ignored)
        && query().isEmpty();
    // A separate Layer-1 analytic arc is not part of the qualified
    // three-straight-trace Source fixture. It must fail even with a forged SHA.
    auto withArc=source;
    withArc.replace("*END*\n",
        "*SIGNAL* NET_A\nU3.3 U4.4\n"
        "150 250 1 10 0\n250 250 1 10 4096 CCW\n"
        "350 250 1 10 0\n*END*\n");
    if(!save(withArc)||!load())return 2;
    auto forgedArc=wire;
    forgedArc.replace(origSha,QCryptographicHash::hash(withArc,QCryptographicHash::Sha256).toHex());
    const bool rejectsArc=panel.imported()->arcCount()==1
        && !decode(forgedArc,withArc,&panel,"NET_A",ignored)
        && query().isEmpty();
    QByteArray invalid=source;
    invalid.replace("350 250 1 10 0 SIMPLEVIA","350 250 2 10 0 SIMPLEVIA");
    if(!save(invalid)||!load())return 2;
    const bool rejectLayer=query().isEmpty() && panel.imported()->referenceRiskCountForTest()==0;
    const bool ok=accepted&&twoVias&&rejectsForgery&&rejectsPromotion&&stale
        &&rejectsInnerPad&&rejectsExtra&&rejectsArc&&rejectLayer;
    qInfo()<<"D014_NATIVE_QT_TWO_VIA"<<ok<<twoVias<<rejectsForgery
        <<rejectsPromotion<<stale<<rejectsInnerPad<<rejectsExtra<<rejectsArc<<rejectLayer
        <<"SOURCE_ONLY_NO_ELECTRICAL_BARREL";
    return ok?0:1;
}
} // namespace d014
