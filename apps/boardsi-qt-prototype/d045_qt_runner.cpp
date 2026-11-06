// D_045 native Qt independent reimport and exact finite-width contact regression.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d045_perpendicular_width_qt.h"
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
        process.start(importer,{temp.fileName(),"--d045-perpendicular-width-v1"});
        if(!process.waitForFinished(10000) ||
           process.exitStatus()!=QProcess::NormalExit)return -1;
        wire=process.readAllStandardOutput();
        return process.exitCode();
    };
    if(!save(source) || !load())return 2;
    QByteArray original;
    if(run(original)!=0 || !d045::decode(original,source,&panel) ||
       !original.contains("COUNT|NODES|9|EDGES|10|LOCAL_COMPONENTS|1"))return 1;
    auto forged=original;
    forged.replace("E|9|7|8|","E|9|6|8|");
    const bool forgeryRejected=forged!=original && !d045::decode(forged,source,&panel);
    QByteArray reversed=source;
    reversed.replace("224 350 1 42 0\n236 350 1 42 0",
                     "236 350 1 42 0\n224 350 1 42 0");
    if(reversed==source || !save(reversed) || !load())return 2;
    QByteArray wire;
    const bool direction=run(wire)==0 && d045::decode(wire,reversed,&panel);
    QByteArray netB=source;
    netB.replace("NET_A","NET_B");
    if(netB==source || !save(netB) || !load())return 2;
    const bool netVariant=run(wire)==0 && d045::decode(wire,netB,&panel) &&
                          !d045::decode(original,source,&panel);
    QByteArray crlf=source;
    crlf.replace("\n","\r\n");
    if(crlf==source || !save(crlf) || !load())return 2;
    const bool crlfVariant=run(wire)==0 && d045::decode(wire,crlf,&panel);
    QByteArray tab=source;
    tab.replace("*SIGNAL* NET_A","*SIGNAL*\tNET_A");
    if(tab==source || !save(tab) || !load())return 2;
    const bool tabVariant=run(wire)==0 && d045::decode(wire,tab,&panel);
    // Reimport every hostile SOURCE into Qt; CLI rejection is not Qt parity.
    const auto rejected=[&](const QByteArray &mutation) {
        if(mutation==source || !save(mutation))return false;
        const bool imported=load();
        QByteArray rejectedWire;
        if(run(rejectedWire)==0)return false;
        if(!imported)return panel.importStatus().startsWith("FAILED");
        return panel.acceptedSourceSha256()==
                   QCryptographicHash::hash(mutation,QCryptographicHash::Sha256).toHex()
            && d045::reconstruct(mutation,&panel).isEmpty();
    };
    QByteArray tangent=source;
    tangent.replace("1 42 0","1 38 0");
    const bool boundaryRejected=rejected(tangent);
    QByteArray corner=source;
    corner.replace("224 350 1 42 0\n236 350 1 42 0",
                   "224 361 1 42 0\n236 361 1 42 0");
    const bool cornerRejected=rejected(corner);
    QByteArray gap=source;
    gap.replace("1 42 0","1 36 0");
    const bool gapRejected=rejected(gap);
    QByteArray owner=source;
    owner.replace("U1.1 U2.1","U1.1 U1.2");
    const bool ownerRejected=rejected(owner);
    QByteArray layer=source;
    layer.replace("224 350 1 42 0","224 350 2 42 0");
    const bool layerRejected=rejected(layer);
    QByteArray missing=source;
    missing.replace("U1.1 U2.1\n224 350 1 42 0\n236 350 1 42 0\n","");
    const bool missingRejected=rejected(missing);
    if(!save(source) || !load() || !d045::decode(original,source,&panel))return 2;
    const bool ok=forgeryRejected && direction && netVariant && crlfVariant &&
        tabVariant && boundaryRejected && cornerRejected && gapRejected && ownerRejected &&
        layerRejected && missingRejected;
    qInfo()<<"D045_NATIVE_QT_FINITE_WIDTH"<<ok
           <<"GLOBAL_ELECTRICAL_NOT_VERIFIED";
    return ok?0:1;
}
