// D_047 Native Qt independent real source reimport / trace-square-pad.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d047_trace_square_pad_width_qt.h"
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
        process.start(importer,{temp.fileName(),"--d047-trace-square-pad-width-v1"});
        if(!process.waitForFinished(10000) ||
           process.exitStatus()!=QProcess::NormalExit)return -1;
        wire=process.readAllStandardOutput();
        return process.exitCode();
    };
    if(!save(source) || !load())return 2;
    QByteArray original;
    if(run(original)!=0 || !d047::decode(original,source,&panel) ||
       !original.contains("COUNT|NODES|9|EDGES|7|LOCAL_COMPONENTS|2"))return 1;
    auto forged=original;
    forged.replace("E|6|1|8|","E|6|0|8|");
    const bool forgeryRejected=forged!=original && !d047::decode(forged,source,&panel);
    QByteArray reversed=source;
    reversed.replace("217 390 1 8 0\n217 410 1 8 0",
                     "217 410 1 8 0\n217 390 1 8 0");
    if(reversed==source || !save(reversed) || !load())return 2;
    QByteArray wire;
    const bool direction=run(wire)==0 && d047::decode(wire,reversed,&panel);
    QByteArray netB=source;
    netB.replace("NET_A","NET_B");
    if(netB==source || !save(netB) || !load())return 2;
    const bool netVariant=run(wire)==0 && d047::decode(wire,netB,&panel) &&
                          !d047::decode(original,source,&panel);
    QByteArray crlf=source;
    crlf.replace("\n","\r\n");
    if(crlf==source || !save(crlf) || !load())return 2;
    const bool crlfVariant=run(wire)==0 && d047::decode(wire,crlf,&panel);
    QByteArray tab=source;
    tab.replace("*SIGNAL* NET_A","*SIGNAL*\tNET_A");
    if(tab==source || !save(tab) || !load())return 2;
    const bool tabVariant=run(wire)==0 && d047::decode(wire,tab,&panel);
    QByteArray cap=source;
    cap.replace("217 390 1 8 0\n217 410 1 8 0",
                "217 417 1 8 0\n225 425 1 8 0");
    if(cap==source || !save(cap) || !load())return 2;
    const bool capContact=run(wire)==0 && d047::decode(wire,cap,&panel);
    const auto rejected=[&](const QByteArray &mutation) {
        if(mutation==source || !save(mutation))return false;
        const bool imported=load();
        QByteArray rejectedWire;
        if(run(rejectedWire)==0)return false;
        if(!imported)return panel.importStatus().startsWith("FAILED");
        return panel.acceptedSourceSha256()==
                   QCryptographicHash::hash(mutation,QCryptographicHash::Sha256).toHex()
            && d047::reconstruct(mutation,&panel).isEmpty();
    };
    QByteArray tangent=source;
    tangent.replace("217 390 1 8 0\n217 410 1 8 0",
                    "219 390 1 8 0\n219 410 1 8 0");
    const bool boundaryRejected=rejected(tangent);
    QByteArray gap=source;
    gap.replace("217 390 1 8 0\n217 410 1 8 0",
                "220 390 1 8 0\n220 410 1 8 0");
    const bool gapRejected=rejected(gap);
    QByteArray corner=source;
    corner.replace("217 390 1 8 0\n217 410 1 8 0",
                   "216 419 1 8 0\n224 427 1 8 0");
    const bool cornerRejected=rejected(corner);
    QByteArray owner=source;
    owner.replace("U1.2 U2.2","U1.1 U2.2");
    const bool ownerRejected=rejected(owner);
    QByteArray layer=source;
    layer.replace("217 390 1 8 0","217 390 2 8 0");
    const bool layerRejected=rejected(layer);
    QByteArray shape=source;
    shape.replace("PAD 2 3\n-2 30 S 0","PAD 2 3\n-2 30 R 0");
    const bool shapeRejected=rejected(shape);
    QByteArray missing=source;
    missing.replace("U1.2 U2.2\n217 390 1 8 0\n217 410 1 8 0\n","");
    const bool missingRejected=rejected(missing);
    if(!save(source) || !load() || !d047::decode(original,source,&panel))return 2;
    const bool ok=forgeryRejected && direction && netVariant && crlfVariant &&
        tabVariant && capContact && boundaryRejected && gapRejected && cornerRejected &&
        ownerRejected && layerRejected && shapeRejected && missingRejected;
    qInfo()<<"D047_NATIVE_QT_TRACE_PAD_WIDTH"<<ok
           <<"GLOBAL_ELECTRICAL_NOT_VERIFIED";
    return ok?0:1;
}
