// D_051 actual two independent Native Qt reImports: complete SOURCE and
// reduced D_050 baseline. Rebuilds complete graph and virtual cut-set.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d051_dual_bridge_loop_qt.h"
#include <QApplication>
#include <QFile>
#include <QProcess>
#include <QTemporaryFile>
#include <QEventLoop>
#include <QTimer>
#include <QElapsedTimer>
#include <QDebug>
#include <array>

int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if(argc!=3)return 2;
    const QString importer=QString::fromLocal8Bit(argv[1]);
    QFile fixture(QString::fromLocal8Bit(argv[2]));
    if(!fixture.open(QIODevice::ReadOnly))return 2;
    QByteArray original=fixture.readAll();
    original.replace("\r\n","\n");
    fixture.close();
    QTemporaryFile fullFile,baseFile;
    if(!fullFile.open() || !baseFile.open())return 2;
    PreviewImportPanel full(importer),baseline(importer);
    const auto save=[](QTemporaryFile &file,const QByteArray &bytes) {
        return file.resize(0) && file.seek(0) &&
            file.write(bytes)==bytes.size() && file.flush();
    };
    const auto load=[&](PreviewImportPanel &panel,QTemporaryFile &file) {
        QEventLoop loop;
        panel.setStatusCallback([&loop](const QString &state) {
            if(state.startsWith("PARTIAL") || state.startsWith("FAILED"))loop.quit();
        });
        const bool started=panel.openPath(file.fileName());
        if(started) {
            QTimer timeout;
            timeout.setSingleShot(true);
            QObject::connect(&timeout,&QTimer::timeout,&loop,&QEventLoop::quit);
            timeout.start(8000);
            loop.exec();
        }
        const bool accepted=started && panel.importStatus().startsWith("PARTIAL");
        // Never retain a callback to a stack-local QEventLoop. If our 8 s
        // watchdog wins, kill and drain the worker before the next import.
        panel.setStatusCallback({});
        if(panel.busy()) {
            panel.stop();
            QElapsedTimer cleanup;
            cleanup.start();
            while(panel.busy() && cleanup.elapsed()<1500)
                QCoreApplication::processEvents(QEventLoop::AllEvents,25);
        }
        return accepted && !panel.busy() &&
            panel.importStatus().startsWith("PARTIAL");
    };
    QByteArray reduced;
    const auto reimport=[&](const QByteArray &source) {
        if(!save(fullFile,source) || !load(full,fullFile))return false;
        std::array<QByteArray,2> vertices{};
        if(!d051::stripSecond(source,reduced,vertices) ||
           !save(baseFile,reduced))return false;
        return load(baseline,baseFile);
    };
    const auto run=[&](QByteArray &wire) {
        QProcess process;
        process.start(importer,{fullFile.fileName(),
                                "--d051-dual-finite-width-loop-v1"});
        if(!process.waitForFinished(10000) ||
           process.exitStatus()!=QProcess::NormalExit)return -1;
        wire=process.readAllStandardOutput();
        return process.exitCode();
    };
    if(!reimport(original))return 2;
    QByteArray receipt;
    if(run(receipt)!=0 || !d051::decode(receipt,original,&full,reduced,&baseline) ||
       !receipt.contains("D|FIRST|11|10|1|0|REACHABLE\n") ||
       !receipt.contains("D|SECOND|11|10|1|0|REACHABLE\n") ||
       !receipt.contains("D|BOTH|10|8|2|0|SEPARATED\n"))return 1;
    const auto positive=[&](const QByteArray &source) {
        if(source==original || !reimport(source))return false;
        QByteArray wire;
        return run(wire)==0 && d051::decode(wire,source,&full,reduced,&baseline) &&
           !d051::decode(receipt,source,&full,reduced,&baseline);
    };
    const auto rejected=[&](const QByteArray &source) {
        if(source==original || !save(fullFile,source))return false;
        const bool loaded=load(full,fullFile);
        bool baseLoaded=false;
        std::array<QByteArray,2> vertices{};
        if(loaded && d051::stripSecond(source,reduced,vertices) &&
           save(baseFile,reduced))baseLoaded=load(baseline,baseFile);
        QByteArray wire;
        if(run(wire)==0)return false;
        if(!loaded)return full.importStatus().startsWith("FAILED");
        return full.acceptedSourceSha256()==
            QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex()
            && (!baseLoaded ||
                d051::reconstruct(source,&full,reduced,&baseline).isEmpty());
    };
    QByteArray forgedEdge=receipt;
    forgedEdge.replace("E|10|9|11|","E|10|0|11|");
    const bool forgedEdgeRejected=forgedEdge!=receipt &&
       !d051::decode(forgedEdge,original,&full,reduced,&baseline);
    QByteArray forgedCycle=receipt;
    forgedCycle.replace("CYCLE_RANK|1","CYCLE_RANK|2");
    const bool forgedCycleRejected=forgedCycle!=receipt &&
       !d051::decode(forgedCycle,original,&full,reduced,&baseline);
    const auto change=[&](const QByteArray &before,const QByteArray &after) {
        QByteArray src=original;
        src.replace(before,after);
        return src;
    };
    const QByteArray bridge="U1.2 U2.2\n222 400 1 8 0\n382 400 1 8 0";
    QByteArray netB=original;netB.replace("NET_A","NET_B");
    QByteArray crlf=original;crlf.replace("\n","\r\n");
    const std::array<QByteArray,9> valid={{
        change(bridge,"U1.2 U2.2\n382 400 1 8 0\n222 400 1 8 0"),
        netB,crlf,
        change("*SIGNAL* NET_A","*SIGNAL*\tNET_A"),
        change("222 400 1 8 0","+222 400 1 8.00 128"),
        change("382 400 1 8 0","382 400 65 8 0"),
        change("222 400 1 8 0","224.99 400 1 8 0"),
        change("382 400 1 8 0","381.01 400 1 8 0"),
        change("222 400 1 8 0","219.01 400 1 8 0")
    }};
    bool accepted=true;
    for(int i=0;i<int(valid.size());++i) {
        if(!positive(valid[static_cast<size_t>(i)])) {
            qWarning()<<"D051 positive mutation failed"<<i;
            accepted=false;
        }
    }
    const std::array<QByteArray,18> hostile={{
        change("222 400 1 8 0","225 400 1 8 0"),
        change("222 400 1 8 0","225.01 400 1 8 0"),
        change("222 400 1 8 0","219 400 1 8 0"),
        change("382 400 1 8 0","381 400 1 8 0"),
        change("382 400 1 8 0","380.99 400 1 8 0"),
        change("382 400 1 8 0","385 400 1 8 0"),
        change("222 400 1 8 0","222 401 1 8 0"),
        change("222 400 1 8 0","222 400 2 8 0"),
        change("222 400 1 8 0","222 400 1 10 0"),
        change("222 400 1 8 0","222 400 1 8.001 0"),
        change(bridge,"U1.1 U2.2\n222 400 1 8 0\n382 400 1 8 0"),
        change(bridge,QByteArray("*SIGNAL* NET_B\n")+bridge),
        change("217 390 1 8 0","217 370 1 8 0"),
        change("PAD 2 3\n-2 30 S 0","PAD 2 3\n-2 30 R 0"),
        change("U2 TYPEA 400 300","U2 TYPEA 398 300"),
        change("NET_A","NET_C"),
        change(bridge+"\n",""),
        change("*END*",bridge+"\n*END*")
    }};
    bool rejectedAll=true;
    for(int i=0;i<int(hostile.size());++i) {
        if(!rejected(hostile[static_cast<size_t>(i)])) {
            qWarning()<<"D051 hostile mutation not rejected"<<i;
            rejectedAll=false;
        }
    }
    if(!reimport(original) || !d051::decode(receipt,original,&full,reduced,&baseline))
        return 2;
    const bool ok=accepted && rejectedAll && forgedEdgeRejected &&
        forgedCycleRejected;
    qInfo()<<"D051_NATIVE_QT_DUAL_BRIDGE_LOOP"<<ok
           <<"GLOBAL_ELECTRICAL_NOT_VERIFIED";
    return ok?0:1;
}
