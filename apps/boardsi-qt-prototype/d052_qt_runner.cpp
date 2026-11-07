// D_052 Native Qt runs actual SOURCE for all four real midspan gap states.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d052_bridge_midspan_break_qt.h"
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
    QFile file(QString::fromLocal8Bit(argv[2]));
    if(!file.open(QIODevice::ReadOnly))return 2;
    QByteArray both=file.readAll();
    both.replace("\r\n","\n");
    file.close();
    QByteArray intact;
    d052::Path a,b;
    if(!d052::prepare(both,intact,a,b) || a.gap!=2000 || b.gap!=2000)
        return 2;
    const QByteArray first="U1.1 U2.1\n226 300 1 8 0\n378 300 1 8 0";
    const QByteArray second="U1.2 U2.2\n222 400 1 8 0\n382 400 1 8 0";
    const QByteArray cutFirst="U1.1 U2.1\n226 300 1 8 0\n290 300 1 8 0\n"
                              "U1.1 U2.1\n310 300 1 8 0\n378 300 1 8 0";
    const QByteArray cutSecond="U1.2 U2.2\n222 400 1 8 0\n290 400 1 8 0\n"
                               "U1.2 U2.2\n310 400 1 8 0\n382 400 1 8 0";
    QTemporaryFile realFile,canonicalFile,baselineFile;
    if(!realFile.open() || !canonicalFile.open() || !baselineFile.open())return 2;
    PreviewImportPanel real(importer),full(importer),baseline(importer);
    const auto save=[](QTemporaryFile &f,const QByteArray &bytes) {
        return f.resize(0) && f.seek(0) && f.write(bytes)==bytes.size() && f.flush();
    };
    const auto load=[&](PreviewImportPanel &panel,QTemporaryFile &f) {
        QEventLoop loop;
        panel.setStatusCallback([&loop](const QString &state) {
            if(state.startsWith("PARTIAL") || state.startsWith("FAILED"))loop.quit();
        });
        const bool started=panel.openPath(f.fileName());
        if(started) {
            QTimer timeout;
            timeout.setSingleShot(true);
            QObject::connect(&timeout,&QTimer::timeout,&loop,&QEventLoop::quit);
            timeout.start(8000);
            loop.exec();
        }
        const bool accepted=started && panel.importStatus().startsWith("PARTIAL");
        panel.setStatusCallback({}); // disconnect stack-local QEventLoop
        if(panel.busy()) {
            panel.stop();
            QElapsedTimer cleanup;
            cleanup.start();
            while(panel.busy() && cleanup.elapsed()<1500)
                QCoreApplication::processEvents(QEventLoop::AllEvents,25);
        }
        return accepted && !panel.busy() && panel.importStatus().startsWith("PARTIAL");
    };
    QByteArray canonical,reduced;
    const auto reimport=[&](const QByteArray &source) {
        d052::Path p,q;
        std::array<QByteArray,2> vertices{};
        if(!d052::prepare(source,canonical,p,q) ||
           !d051::stripSecond(canonical,reduced,vertices) ||
           !save(realFile,source) || !load(real,realFile) ||
           !save(canonicalFile,canonical) || !load(full,canonicalFile) ||
           !save(baselineFile,reduced) || !load(baseline,baselineFile))
            return false;
        return true;
    };
    const auto run=[&](QByteArray &wire) {
        QProcess process;
        process.start(importer,{realFile.fileName(),
                                "--d052-dual-bridge-midspan-break-v1"});
        if(!process.waitForFinished(10000) ||
           process.exitStatus()!=QProcess::NormalExit)return -1;
        wire=process.readAllStandardOutput();
        return process.exitCode();
    };
    const auto accepted=[&](const QByteArray &src,const QByteArray &expected) {
        if(!reimport(src))return false;
        QByteArray receipt;
        if(run(receipt)!=0 || !d052::decode(receipt,src,&real,canonical,&full,reduced,&baseline) ||
           !receipt.contains(expected))return false;
        QByteArray forged=receipt;
        forged.replace("|GLOBAL_ELECTRICAL|NOT_VERIFIED","|GLOBAL_ELECTRICAL|VERIFIED");
        if(forged==receipt ||
           d052::decode(forged,src,&real,canonical,&full,reduced,&baseline))return false;
        QByteArray stale=src;
        stale.replace("NET_A","NET_B");
        if(stale!=src && d052::decode(receipt,stale,&real,canonical,&full,reduced,&baseline))
            return false;
        return true;
    };
    QByteArray cutA=intact;cutA.replace(first,cutFirst);
    QByteArray cutB=intact;cutB.replace(second,cutSecond);
    const std::array<std::pair<QByteArray,QByteArray>,4> states={{
        {intact,"D|INTACT|12|12|1|1|REACHABLE\n"},
        {cutA,"D|BREAK_A|13|12|1|0|REACHABLE\n"},
        {cutB,"D|BREAK_B|13|12|1|0|REACHABLE\n"},
        {both,"D|BREAK_BOTH|14|12|2|0|SEPARATED\n"}
    }};
    bool all=true;
    for(size_t i=0;i<states.size();++i) {
        if(!accepted(states[i].first,states[i].second)) {
            qWarning()<<"D052 state failed"<<i;all=false;
        }
    }
    const auto mutate=[&](const QByteArray &from,const QByteArray &to) {
        QByteArray src=both;
        src.replace(from,to);
        return src;
    };
    QByteArray netB=both;netB.replace("NET_A","NET_B");
    QByteArray crlf=both;crlf.replace("\n","\r\n");
    const std::array<QByteArray,5> positives={{
        netB,crlf,
        mutate("*SIGNAL* NET_A","*SIGNAL*\tNET_A"),
        mutate("290 300 1 8 0","300 300 1 8 0")
          .replace("310 300 1 8 0","308.01 300 1 8 0"),
        mutate("290 400 1 8 0","300 400 1 8 0")
          .replace("310 400 1 8 0","308.01 400 1 8 0")
    }};
    for(size_t i=0;i<positives.size();++i) {
        if(positives[i]==both ||
           !accepted(positives[i],"D|BREAK_BOTH|14|12|2|0|SEPARATED\n")) {
            qWarning()<<"D052 positive failed"<<i;all=false;
        }
    }
    const std::array<QByteArray,13> hostile={{
        mutate("310 300 1 8 0","298 300 1 8 0"), // exact tangent
        mutate("310 300 1 8 0","297.99 300 1 8 0"),
        mutate("310 400 1 8 0","298 400 1 8 0"),
        mutate("310 400 1 8 0","297.99 400 1 8 0"),
        mutate("290 300 1 8 0","270 300 1 8 0"),
        mutate("226 300 1 8 0","230 300 1 8 0"),
        mutate("310 300 1 8 0","310 300 2 8 0"),
        mutate("310 400 1 8 0","310 400 1 10 0"),
        mutate("U1.1 U2.1\n310","U1.2 U2.1\n310"),
        mutate("PAD 2 3\n-2 30 S 0","PAD 2 3\n-2 30 R 0"),
        mutate("NET_A","NET_C"),
        mutate("U1.2 U2.2\n310 400 1 8 0\n382 400 1 8 0\n",""),
        mutate("*END*","FORGED\n*END*")
    }};
    for(size_t i=0;i<hostile.size();++i) {
        QByteArray candidate,readout;
        d052::Path p,q;
        if(hostile[i]==both ||
           (d052::prepare(hostile[i],candidate,p,q) &&
            reimport(hostile[i]) &&
            !d052::reconstruct(hostile[i],&real,canonical,&full,reduced,&baseline).isEmpty())) {
            qWarning()<<"D052 Qt hostile accepted"<<i;all=false;
        }
        if(!save(realFile,hostile[i]) || run(readout)==0) {
            qWarning()<<"D052 Rust hostile accepted"<<i;all=false;
        }
    }
    qInfo()<<"D052_NATIVE_QT_REAL_MIDSPAN_BREAK"<<all
           <<"GLOBAL_ELECTRICAL_NOT_VERIFIED";
    return all?0:1;
}
