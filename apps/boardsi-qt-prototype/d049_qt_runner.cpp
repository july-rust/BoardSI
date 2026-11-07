// D_049 Native Qt independent real source reimport / trace-round-pad.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d049_mixed_pad_dual_width_qt.h"
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
        process.start(importer,{temp.fileName(),"--d049-mixed-pad-dual-width-v1"});
        if(!process.waitForFinished(10000) ||
           process.exitStatus()!=QProcess::NormalExit)return -1;
        wire=process.readAllStandardOutput();
        return process.exitCode();
    };
    if(!save(source) || !load())return 2;
    QByteArray original;
    if(run(original)!=0 || !d049::decode(original,source,&panel) ||
       !original.contains("COUNT|NODES|10|EDGES|8|LOCAL_COMPONENTS|2"))return 1;
    auto forged=original;
    forged.replace("E|6|0|8|","E|6|1|8|");
    const bool forgeryRejected=forged!=original && !d049::decode(forged,source,&panel);
    auto forgedSquare=original;
    forgedSquare.replace("E|7|1|9|","E|7|0|9|");
    const bool forgerySquareRejected=forgedSquare!=original &&
        !d049::decode(forgedSquare,source,&panel);
    QByteArray reversed=source;
    reversed.replace("222 285 1 8 0\n222 315 1 8 0",
                     "222 315 1 8 0\n222 285 1 8 0");
    if(reversed==source || !save(reversed) || !load())return 2;
    QByteArray wire;
    const bool direction=run(wire)==0 && d049::decode(wire,reversed,&panel);
    QByteArray netB=source;
    netB.replace("NET_A","NET_B");
    if(netB==source || !save(netB) || !load())return 2;
    const bool netVariant=run(wire)==0 && d049::decode(wire,netB,&panel) &&
                          !d049::decode(original,source,&panel);
    QByteArray crlf=source;
    crlf.replace("\n","\r\n");
    if(crlf==source || !save(crlf) || !load())return 2;
    const bool crlfVariant=run(wire)==0 && d049::decode(wire,crlf,&panel);
    QByteArray tab=source;
    tab.replace("*SIGNAL* NET_A","*SIGNAL*\tNET_A");
    if(tab==source || !save(tab) || !load())return 2;
    const bool tabVariant=run(wire)==0 && d049::decode(wire,tab,&panel);
    QByteArray cap=source;
    cap.replace("222 285 1 8 0\n222 315 1 8 0",
                "217 316 1 8 0\n220 319 1 8 0");
    if(cap==source || !save(cap) || !load())return 2;
    const bool capContact=run(wire)==0 && d049::decode(wire,cap,&panel);
    QByteArray squareCap=source;
    squareCap.replace("217 390 1 8 0\n217 410 1 8 0",
                      "217 417 1 8 0\n225 425 1 8 0");
    if(squareCap==source || !save(squareCap) || !load())return 2;
    const bool squareCapContact=run(wire)==0 && d049::decode(wire,squareCap,&panel);
    QByteArray squareReversed=source;
    squareReversed.replace("217 390 1 8 0\n217 410 1 8 0",
                           "217 410 1 8 0\n217 390 1 8 0");
    if(squareReversed==source || !save(squareReversed) || !load())return 2;
    const bool squareDirection=run(wire)==0 && d049::decode(wire,squareReversed,&panel);
    QByteArray roundNumeric=source;
    roundNumeric.replace("222 285 1 8 0","+222 285 1 8.00 128");
    if(roundNumeric==source || !save(roundNumeric) || !load())return 2;
    const bool roundNumericParity=run(wire)==0 && d049::decode(wire,roundNumeric,&panel);
    QByteArray squareNumeric=source;
    squareNumeric.replace("217 390 1 8 0","+217 390 1 8.00 1024");
    if(squareNumeric==source || !save(squareNumeric) || !load())return 2;
    const bool squareNumericParity=run(wire)==0 && d049::decode(wire,squareNumeric,&panel);
    QByteArray roundTerminal=source;
    roundTerminal.replace("222 315 1 8 0","222 315 65 8 0");
    if(roundTerminal==source || !save(roundTerminal) || !load())return 2;
    const bool roundTerminalParity=run(wire)==0 && d049::decode(wire,roundTerminal,&panel);
    QByteArray squareTerminal=source;
    squareTerminal.replace("217 410 1 8 0","217 410 65 8 0");
    if(squareTerminal==source || !save(squareTerminal) || !load())return 2;
    const bool squareTerminalParity=run(wire)==0 && d049::decode(wire,squareTerminal,&panel);
    const auto rejected=[&](const QByteArray &mutation) {
        if(mutation==source || !save(mutation))return false;
        const bool imported=load();
        QByteArray rejectedWire;
        if(run(rejectedWire)==0)return false;
        if(!imported)return panel.importStatus().startsWith("FAILED");
        return panel.acceptedSourceSha256()==
                   QCryptographicHash::hash(mutation,QCryptographicHash::Sha256).toHex()
            && d049::reconstruct(mutation,&panel).isEmpty();
    };
    QByteArray nearTangent=source;
    nearTangent.replace("222 285 1 8 0\n222 315 1 8 0",
                        "223.99 285 1 8 0\n223.99 315 1 8 0");
    if(nearTangent==source || !save(nearTangent) || !load())return 2;
    const bool oneTickContact=run(wire)==0 && d049::decode(wire,nearTangent,&panel);
    QByteArray nearGap=source;
    nearGap.replace("222 285 1 8 0\n222 315 1 8 0",
                    "224.01 285 1 8 0\n224.01 315 1 8 0");
    const bool oneTickGapRejected=rejected(nearGap);
    QByteArray tangent=source;
    tangent.replace("222 285 1 8 0\n222 315 1 8 0",
                    "224 285 1 8 0\n224 315 1 8 0");
    const bool boundaryRejected=rejected(tangent);
    QByteArray squareTangent=source;
    squareTangent.replace("217 390 1 8 0\n217 410 1 8 0",
                          "219 390 1 8 0\n219 410 1 8 0");
    const bool squareBoundaryRejected=rejected(squareTangent);
    QByteArray squareOneTickGap=source;
    squareOneTickGap.replace("217 390 1 8 0\n217 410 1 8 0",
                             "219.01 390 1 8 0\n219.01 410 1 8 0");
    const bool squareOneTickGapRejected=rejected(squareOneTickGap);
    QByteArray gap=source;
    gap.replace("222 285 1 8 0\n222 315 1 8 0",
                "225 285 1 8 0\n225 315 1 8 0");
    const bool gapRejected=rejected(gap);
    QByteArray corner=source;
    corner.replace("222 285 1 8 0\n222 315 1 8 0",
                   "222 322 1 8 0\n230 330 1 8 0");
    const bool cornerRejected=rejected(corner);
    QByteArray squareCorner=source;
    squareCorner.replace("217 390 1 8 0\n217 410 1 8 0",
                         "216 419 1 8 0\n224 427 1 8 0");
    const bool squareCornerRejected=rejected(squareCorner);
    QByteArray owner=source;
    owner.replace("U1.1 U2.1","U1.2 U2.2");
    const bool ownerRejected=rejected(owner);
    QByteArray layer=source;
    layer.replace("222 285 1 8 0","222 285 2 8 0");
    const bool layerRejected=rejected(layer);
    QByteArray squareLayer=source;
    squareLayer.replace("217 390 1 8 0","217 390 2 8 0");
    const bool squareLayerRejected=rejected(squareLayer);
    QByteArray invalidPrecision=source;
    invalidPrecision.replace("222 285 1 8 0","222 285 1 8.001 0");
    const bool invalidPrecisionRejected=rejected(invalidPrecision);
    QByteArray invalidFlag=source;
    invalidFlag.replace("217 390 1 8 0","217 390 1 8 16");
    const bool invalidFlagRejected=rejected(invalidFlag);
    QByteArray squareOwner=source;
    squareOwner.replace("U1.2 U2.2\n217 390","U1.1 U2.2\n217 390");
    const bool squareOwnerRejected=rejected(squareOwner);
    QByteArray shape=source;
    shape.replace("PAD 1 3\n-2 40 R 0","PAD 1 3\n-2 40 S 0");
    const bool shapeRejected=rejected(shape);
    QByteArray squareShape=source;
    squareShape.replace("PAD 2 3\n-2 30 S 0","PAD 2 3\n-2 30 R 0");
    const bool squareShapeRejected=rejected(squareShape);
    QByteArray missing=source;
    missing.replace("U1.1 U2.1\n222 285 1 8 0\n222 315 1 8 0\n","");
    const bool missingRejected=rejected(missing);
    QByteArray squareMissing=source;
    squareMissing.replace("U1.2 U2.2\n217 390 1 8 0\n217 410 1 8 0\n","");
    const bool squareMissingRejected=rejected(squareMissing);
    if(!save(source) || !load() || !d049::decode(original,source,&panel))return 2;
    const bool ok=forgeryRejected && forgerySquareRejected && direction && squareDirection && netVariant && crlfVariant &&
        tabVariant && capContact && squareCapContact && roundNumericParity && squareNumericParity &&
        roundTerminalParity && squareTerminalParity && oneTickContact && oneTickGapRejected && boundaryRejected && squareBoundaryRejected && squareOneTickGapRejected && gapRejected && cornerRejected && squareCornerRejected &&
        ownerRejected && squareOwnerRejected && layerRejected && squareLayerRejected &&
        invalidPrecisionRejected && invalidFlagRejected && shapeRejected && squareShapeRejected && missingRejected && squareMissingRejected;
    qInfo()<<"D049_NATIVE_QT_TRACE_PAD_WIDTH"<<ok
           <<"GLOBAL_ELECTRICAL_NOT_VERIFIED";
    return ok?0:1;
}
