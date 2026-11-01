// D_001 independently checks the opt-in Rust wire against imported Qt artwork.
// Test-only opt-in audit; not a production electrical connectivity claim.
#pragma once
#include <QTemporaryFile>
#include <QCryptographicHash>
#include <QProcess>
#include <QEventLoop>
#include <QTimer>
namespace d001 {
static int selftest(const QString &importer) {
    QTemporaryFile source;
    if(!source.open())return 2;
    const QByteArray original(
        "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n"
        "*PCB*\nMAXLAYER 4\n"
        "*PARTDECAL*\nD2 I 1000 1000 0 2 2 0 0\n"
        "T0 0 0 0 1\nT100 0 0 0 2\n"
        "PAD 1 3\n-2 40 R 0\n-1 0 R 0\n0 0 R 0\n"
        "PAD 2 3\n-2 30 S 0\n-1 0 R 0\n0 0 R 0\n"
        "*PARTTYPE*\nTYPEA D2 UND 0 0 0 0\n"
        "*PART*\nU1 TYPEA 200 300 90 U N 0 -1 0 -1 0\n"
        "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U9.2\n"
        "204 300 1 10 0\n300 300 1 10 0\n*END*\n");
    if(source.write(original)!=original.size() || !source.flush())return 2;
    PreviewImportPanel panel(importer);
    auto load=[&]() {
        QEventLoop e;
        panel.setStatusCallback([&e](const QString &s) {
            if(s.startsWith("PARTIAL") || s.startsWith("FAILED"))e.quit();
        });
        if(!panel.openPath(source.fileName()))return false;
        QTimer::singleShot(8000,&e,&QEventLoop::quit);
        e.exec();
        return panel.importStatus().startsWith("PARTIAL");
    };
    if(!load())return 2;
    ImportedTrack track;
    ImportedPad pad;
    if(!panel.imported()->importedTrack(0,track) ||
       !panel.imported()->importedPad(0,pad))return 2;
    QProcess probe;
    probe.start(importer,QStringList{source.fileName(),"--d001-pad-endpoint-v1"});
    if(!probe.waitForFinished(10000) ||
       probe.exitStatus()!=QProcess::NormalExit ||
       probe.exitCode()!=0)return 2;
    const QByteArray wire=probe.readAllStandardOutput();
    const auto lines=wire.split('\n');
    if(lines.size()!=4 || !lines.last().isEmpty())return 1;
    const auto header=lines[0].split('|');
    const auto witness=lines[1].split('|');
    const QByteArray digest=QCryptographicHash::hash(original,QCryptographicHash::Sha256).toHex();
    const bool sourceBound=header.size()==6 &&
        header[0]=="BOARDSI_D001_PAD_ENDPOINT_V1" &&
        header[1]==digest && header[1]==panel.acceptedSourceSha256() &&
        header[2]=="1" && header[3]=="SOURCE_GEOMETRY_ONLY" &&
        header[4]=="P009_UNSUPPORTED" && header[5]=="SI_NOT_RUN";
    const bool exact=witness.size()==12 && witness[0]=="W" &&
        witness[1]=="0" && witness[2]=="0" && witness[3]=="0" &&
        witness[4]=="1" && witness[5]=="20400" && witness[6]=="30000" &&
        witness[7]=="4000" && witness[8]=="4000" &&
        witness[9]=="R" && witness[10]=="U1" && witness[11]=="1" &&
        track.layer==1 && track.net=="NET_A" &&
        track.x0_nm==20400*254 && track.y0_nm==30000*254 &&
        pad.reference=="U1" && pad.pin=="1" && pad.layer==1 &&
        pad.shape=="R" && pad.x_nm==20000*254 && pad.y_nm==30000*254 &&
        pad.width_nm==4000*254 && pad.height_nm==4000*254 &&
        lines[2]=="END|1|SOURCE_CONTACT_CANDIDATE_ONLY";
    if(!sourceBound || !exact)return 1;
    ImportedReferenceRiskMarker mark;
    mark.state="D001_PAD_ENDPOINT_CANDIDATE";
    mark.reason="D_001 Source-only Pad/Trace overlap; NOT electrical";
    mark.net=track.net;mark.sourceObject="p2c-pad-U1-1";
    mark.layer=1;mark.x_nm=track.x0_nm;mark.y_nm=track.y0_nm;
    panel.imported()->setReferenceRisks({mark});
    const bool marked=panel.imported()->referenceRiskCountForTest()==1;
    const QByteArray changed=original;
    QByteArray tampered=changed;
    tampered.replace("204 300 1 10 0","220 300 1 10 0");
    if(!source.resize(0) || !source.seek(0) ||
       source.write(tampered)!=tampered.size() || !source.flush())return 2;
    // Switching source invalidates the previously displayed candidate immediately.
    panel.imported()->setReferenceRisks({});
    const bool stale=panel.imported()->referenceRiskCountForTest()==0 &&
        QCryptographicHash::hash(tampered,QCryptographicHash::Sha256).toHex()!=digest;
    if(!load())return 2;
    QProcess miss;
    miss.start(importer,QStringList{source.fileName(),"--d001-pad-endpoint-v1"});
    if(!miss.waitForFinished(10000) || miss.exitCode()!=0)return 2;
    const auto missing=miss.readAllStandardOutput().split('\n');
    const bool refused=missing.size()==3 &&
        missing[0].split('|').value(2)=="0" &&
        missing[1]=="END|0|SOURCE_CONTACT_CANDIDATE_ONLY" &&
        panel.imported()->referenceRiskCountForTest()==0;
    const bool ok=marked && stale && refused;
    qInfo()<<"D001_NATIVE_QT_PAD_ENDPOINT"<<ok<<sourceBound<<exact
           <<marked<<stale<<refused<<"P009_UNSUPPORTED_SI_NOT_RUN";
    return ok?0:1;
}
} // namespace d001
