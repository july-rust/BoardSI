// D_050 actual PreviewImportPanel reImport and independent Qt graph oracle.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d050_mixed_pad_width_bridge_qt.h"
#include <QApplication>
#include <QFile>
#include <QProcess>
#include <QTemporaryFile>
#include <QEventLoop>
#include <QTimer>
#include <QDebug>

int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if(argc!=3)return 2;
    const QString importer=QString::fromLocal8Bit(argv[1]);
    QFile fixture(QString::fromLocal8Bit(argv[2]));
    if(!fixture.open(QIODevice::ReadOnly))return 2;
    QByteArray originalSource=fixture.readAll();
    originalSource.replace("\r\n","\n");
    fixture.close();
    QTemporaryFile temp;
    if(!temp.open())return 2;
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    const auto save=[&](const QByteArray &bytes) {
        return temp.resize(0) && temp.seek(0) &&
               temp.write(bytes)==bytes.size() && temp.flush();
    };
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
        process.start(importer,{temp.fileName(),"--d050-mixed-pad-width-bridge-v1"});
        if(!process.waitForFinished(10000) ||
           process.exitStatus()!=QProcess::NormalExit)return -1;
        wire=process.readAllStandardOutput();
        return process.exitCode();
    };
    QByteArray original;
    const auto positive=[&](const QByteArray &source) {
        if(source==originalSource || !save(source) || !load())return false;
        QByteArray wire;
        return run(wire)==0 && d050::decode(wire,source,&panel) &&
            !d050::decode(original,source,&panel) &&
            wire.contains("COUNT|NODES|11|EDGES|10|LOCAL_COMPONENTS|1");
    };
    const auto rejected=[&](const QByteArray &source) {
        if(source==originalSource || !save(source))return false;
        const bool imported=load();
        QByteArray wire;
        if(run(wire)==0)return false;
        if(!imported)return panel.importStatus().startsWith("FAILED");
        return panel.acceptedSourceSha256()==
            QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex()
            && d050::reconstruct(source,&panel).isEmpty();
    };
    if(!save(originalSource) || !load())return 2;
    if(run(original)!=0 || !d050::decode(original,originalSource,&panel) ||
       !original.contains("COUNT|NODES|11|EDGES|10|LOCAL_COMPONENTS|1"))return 1;
    QByteArray forged=original;
    forged.replace("E|8|8|10|","E|8|0|10|");
    const bool forgedEdgeRejected=forged!=original &&
        !d050::decode(forged,originalSource,&panel);
    QByteArray forgedCount=original;
    forgedCount.replace("LOCAL_COMPONENTS|1","LOCAL_COMPONENTS|2");
    const bool forgedCountRejected=forgedCount!=original &&
        !d050::decode(forgedCount,originalSource,&panel);
    auto mutate=[&](const QByteArray &oldText,const QByteArray &newText) {
        QByteArray bytes=originalSource;
        bytes.replace(oldText,newText);
        return bytes;
    };
    const QByteArray bridge="U1.1 U2.1\n226 300 1 8 0\n378 300 1 8 0";
    const QByteArray bridgeReversed=
        "U1.1 U2.1\n378 300 1 8 0\n226 300 1 8 0";
    QByteArray netB=originalSource;
    netB.replace("NET_A","NET_B");
    QByteArray crlf=originalSource;
    crlf.replace("\n","\r\n");
    const std::array<QByteArray,9> valid={
        mutate(bridge,bridgeReversed),
        netB,crlf,
        mutate("*SIGNAL* NET_A","*SIGNAL*\tNET_A"),
        mutate("226 300 1 8 0","+226 300 1 8.00 128"),
        mutate("378 300 1 8 0","378 300 65 8 0"),
        mutate("226 300 1 8 0","229.99 300 1 8 0"),
        mutate("378 300 1 8 0","376.01 300 1 8 0"),
        mutate("226 300 1 8 0","224.01 300 1 8 0"),
    };
    bool positiveOk=true;
    for(int i=0;i<static_cast<int>(valid.size());++i) {
        if(!positive(valid[static_cast<size_t>(i)])) {
            qWarning()<<"D050 positive mutation failed:"<<i;
            positiveOk=false;
        }
    }
    const std::array<QByteArray,19> hostile={
        mutate("226 300 1 8 0","230 300 1 8 0"),
        mutate("226 300 1 8 0","230.01 300 1 8 0"),
        mutate("226 300 1 8 0","224 300 1 8 0"),
        mutate("378 300 1 8 0","376 300 1 8 0"),
        mutate("378 300 1 8 0","375.99 300 1 8 0"),
        mutate("378 300 1 8 0","380 300 1 8 0"),
        mutate(bridge,"U1.1 U2.1\n226 322 1 8 0\n378 322 1 8 0"),
        mutate("226 300 1 8 0","226 300 2 8 0"),
        mutate("226 300 1 8 0","226 300 1 10 0"),
        mutate("226 300 1 8 0","226 300 1 8.001 0"),
        mutate("226 300 1 8 0","226 300 1 8 16"),
        mutate(bridge,"U1.2 U2.1\n226 300 1 8 0\n378 300 1 8 0"),
        mutate(bridge,QByteArray("*SIGNAL* NET_B\n")+bridge),
        mutate("222 285 1 8 0","227 285 1 8 0"),
        mutate("PAD 1 3\n-2 40 R 0","PAD 1 3\n-2 40 S 0"),
        mutate("U2 TYPEA 400 300","U2 TYPEA 398 300"),
        mutate("NET_A","NET_C"),
        mutate(bridge+"\n",""),
        mutate("*END*",bridge+"\n*END*"),
    };
    bool rejectedOk=true;
    for(int i=0;i<static_cast<int>(hostile.size());++i) {
        if(!rejected(hostile[static_cast<size_t>(i)])) {
            qWarning()<<"D050 hostile mutation not rejected:"<<i;
            rejectedOk=false;
        }
    }
    if(!save(originalSource) || !load() ||
       !d050::decode(original,originalSource,&panel))return 2;
    const bool ok=positiveOk && rejectedOk && forgedEdgeRejected &&
                  forgedCountRejected;
    qInfo()<<"D050_NATIVE_QT_MIXED_PAD_BRIDGE"<<ok
           <<"GLOBAL_ELECTRICAL_NOT_VERIFIED";
    return ok?0:1;
}
