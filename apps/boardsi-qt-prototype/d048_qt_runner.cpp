// D_048 Native Qt independent real source reimport / trace-round-pad.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d048_trace_round_pad_width_qt.h"
#include <QApplication>
#include <QFile>
#include <QProcess>
#include <QTemporaryFile>
#include <QEventLoop>
#include <QTimer>
int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if(argc!=3)return 2;
    const QString importer=QString::fromLocal8Bit(argv[1]);
    QFile fixture(QString::fromLocal8Bit(argv[2]));
    if(!fixture.open(QIODevice::ReadOnly))return 2;
    QByteArray source=fixture.readAll();
    source.replace("\r\n","\n");
    fixture.close();
    QTemporaryFile temp;
    if(!temp.open())return 2;
    const auto save=[&](const QByteArray &bytes) {
        return temp.resize(0) && temp.seek(0) &&
               temp.write(bytes)==bytes.size() && temp.flush();
    };
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    const auto load=[&]() {
        panel.setStatusCallback([&loop](const QString &state) {
            if(state.startsWith("PARTIAL") || state.startsWith("FAILED"))loop.quit();
        });
        if(!panel.openPath(temp.fileName()))return false;
        QTimer timeout;
        timeout.setSingleShot(true);
        QObject::connect(&timeout,&QTimer::timeout,&loop,&QEventLoop::quit);
        timeout.start(8000);
        loop.exec();
        return panel.importStatus().startsWith("PARTIAL");
    };
    const auto run=[&](QByteArray &wire) {
        QProcess process;
        process.start(importer,{temp.fileName(),"--d048-trace-round-pad-width-v1"});
        if(!process.waitForFinished(10000) ||
           process.exitStatus()!=QProcess::NormalExit)return -1;
        wire=process.readAllStandardOutput();
        return process.exitCode();
    };
    if(!save(source) || !load())return 2;
    QByteArray original;
    if(run(original)!=0 || !d048::decode(original,source,&panel) ||
       !original.contains("COUNT|NODES|9|EDGES|7|LOCAL_COMPONENTS|2"))return 1;
    auto forged=original;
    forged.replace("E|6|0|8|","E|6|1|8|");
    const bool forgeryRejected=forged!=original && !d048::decode(forged,source,&panel);
    QByteArray reversed=source;
    reversed.replace("222 285 1 8 0\n222 315 1 8 0",
                     "222 315 1 8 0\n222 285 1 8 0");
    if(reversed==source || !save(reversed) || !load())return 2;
    QByteArray wire;
    const bool direction=run(wire)==0 && d048::decode(wire,reversed,&panel);
    QByteArray netB=source;
    netB.replace("NET_A","NET_B");
    if(netB==source || !save(netB) || !load())return 2;
    const bool netVariant=run(wire)==0 && d048::decode(wire,netB,&panel) &&
                          !d048::decode(original,source,&panel);
    QByteArray crlf=source;
    crlf.replace("\n","\r\n");
    if(crlf==source || !save(crlf) || !load())return 2;
    const bool crlfVariant=run(wire)==0 && d048::decode(wire,crlf,&panel);
    QByteArray tab=source;
    tab.replace("*SIGNAL* NET_A","*SIGNAL*\tNET_A");
    if(tab==source || !save(tab) || !load())return 2;
    const bool tabVariant=run(wire)==0 && d048::decode(wire,tab,&panel);
    QByteArray cap=source;
    cap.replace("222 285 1 8 0\n222 315 1 8 0",
                "217 316 1 8 0\n220 319 1 8 0");
    if(cap==source || !save(cap) || !load())return 2;
    const bool capContact=run(wire)==0 && d048::decode(wire,cap,&panel);
    const auto rejected=[&](const QByteArray &mutation) {
        if(mutation==source || !save(mutation))return false;
        const bool imported=load();
        QByteArray rejectedWire;
        if(run(rejectedWire)==0)return false;
        if(!imported)return panel.importStatus().startsWith("FAILED");
        return panel.acceptedSourceSha256()==
                   QCryptographicHash::hash(mutation,QCryptographicHash::Sha256).toHex()
            && d048::reconstruct(mutation,&panel).isEmpty();
    };
    QByteArray nearTangent=source;
    nearTangent.replace("222 285 1 8 0\n222 315 1 8 0",
                        "223.99 285 1 8 0\n223.99 315 1 8 0");
    if(nearTangent==source || !save(nearTangent) || !load())return 2;
    const bool oneTickContact=run(wire)==0 && d048::decode(wire,nearTangent,&panel);
    QByteArray nearGap=source;
    nearGap.replace("222 285 1 8 0\n222 315 1 8 0",
                    "224.01 285 1 8 0\n224.01 315 1 8 0");
    const bool oneTickGapRejected=rejected(nearGap);
    QByteArray tangent=source;
    tangent.replace("222 285 1 8 0\n222 315 1 8 0",
                    "224 285 1 8 0\n224 315 1 8 0");
    const bool boundaryRejected=rejected(tangent);
    QByteArray gap=source;
    gap.replace("222 285 1 8 0\n222 315 1 8 0",
                "225 285 1 8 0\n225 315 1 8 0");
    const bool gapRejected=rejected(gap);
    QByteArray corner=source;
    corner.replace("222 285 1 8 0\n222 315 1 8 0",
                   "222 322 1 8 0\n230 330 1 8 0");
    const bool cornerRejected=rejected(corner);
    QByteArray owner=source;
    owner.replace("U1.1 U2.1","U1.2 U2.2");
    const bool ownerRejected=rejected(owner);
    QByteArray layer=source;
    layer.replace("222 285 1 8 0","222 285 2 8 0");
    const bool layerRejected=rejected(layer);
    QByteArray shape=source;
    shape.replace("PAD 1 3\n-2 40 R 0","PAD 1 3\n-2 40 S 0");
    const bool shapeRejected=rejected(shape);
    QByteArray missing=source;
    missing.replace("U1.1 U2.1\n222 285 1 8 0\n222 315 1 8 0\n","");
    const bool missingRejected=rejected(missing);
    if(!save(source) || !load() || !d048::decode(original,source,&panel))return 2;
    const bool ok=forgeryRejected && direction && netVariant && crlfVariant &&
        tabVariant && capContact && oneTickContact && oneTickGapRejected && boundaryRejected && gapRejected && cornerRejected &&
        ownerRejected && layerRejected && shapeRejected && missingRejected;
    qInfo()<<"D048_NATIVE_QT_TRACE_PAD_WIDTH"<<ok
           <<"GLOBAL_ELECTRICAL_NOT_VERIFIED";
    return ok?0:1;
}
