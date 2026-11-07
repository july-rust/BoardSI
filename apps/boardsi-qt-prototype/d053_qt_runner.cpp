// D_053 Native Qt actual reImport of PADS SOURCE repair traces.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d053_midspan_repair_qt.h"
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
    QByteArray both=fixture.readAll();
    both.replace("\r\n","\n");
    const QByteArray originalA="U1.1 U2.1\n226 300 1 8 0\n378 300 1 8 0";
    const QByteArray originalB="U1.2 U2.2\n222 400 1 8 0\n382 400 1 8 0";
    const QByteArray cutA="U1.1 U2.1\n226 300 1 8 0\n290 300 1 8 0\n"
                          "U1.1 U2.1\n310 300 1 8 0\n378 300 1 8 0";
    const QByteArray cutB="U1.2 U2.2\n222 400 1 8 0\n290 400 1 8 0\n"
                          "U1.2 U2.2\n310 400 1 8 0\n382 400 1 8 0";
    const QByteArray repairA="U1.1 U2.1\n288 300 1 8 0\n312 300 1 8 0\n";
    const QByteArray repairB="U1.2 U2.2\n288 400 1 8 0\n312 400 1 8 0\n";
    QByteArray intact;
    d052::Path a,b;
    if(!d052::prepare(both,intact,a,b) || a.gap!=2000 || b.gap!=2000)return 2;
    const auto insert=[](QByteArray s,const QByteArray &repair) {
        s.replace("*END*",repair+"*END*");
        return s;
    };
    QByteArray brokenA=intact,brokenB=intact;
    brokenA.replace(originalA,cutA);
    brokenB.replace(originalB,cutB);
    const std::array<std::pair<QByteArray,QByteArray>,6> states={{
        {both,"D|BREAK_BOTH|14|12|2|0|SEPARATED\n"},
        {insert(both,repairA),"D|REPAIR_A_ONLY|15|14|1|0|REACHABLE\n"},
        {insert(both,repairB),"D|REPAIR_B_ONLY|15|14|1|0|REACHABLE\n"},
        {insert(both,repairA+repairB),"D|REPAIR_BOTH|16|16|1|1|REACHABLE\n"},
        {insert(brokenA,repairA),"D|BREAK_A_REPAIR_A|14|14|1|1|REACHABLE\n"},
        {insert(brokenB,repairB),"D|BREAK_B_REPAIR_B|14|14|1|1|REACHABLE\n"},
    }};
    QTemporaryFile realFile,bareFile,canonicalFile,baselineFile;
    if(!realFile.open() || !bareFile.open() || !canonicalFile.open() || !baselineFile.open())return 2;
    PreviewImportPanel real(importer),barePanel(importer),full(importer),baseline(importer);
    const auto save=[](QTemporaryFile &f,const QByteArray &bytes) {
        return f.resize(0) && f.seek(0) && f.write(bytes)==bytes.size() && f.flush();
    };
    const auto load=[&](PreviewImportPanel &panel,QTemporaryFile &f) {
        QEventLoop loop;
        panel.setStatusCallback([&loop](const QString &status) {
            if(status.startsWith("PARTIAL") || status.startsWith("FAILED"))loop.quit();
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
        panel.setStatusCallback({});
        if(panel.busy()) {
            panel.stop();
            QElapsedTimer cleanup;cleanup.start();
            while(panel.busy() && cleanup.elapsed()<1500)
                QCoreApplication::processEvents(QEventLoop::AllEvents,25);
        }
        return accepted && !panel.busy() && panel.importStatus().startsWith("PARTIAL");
    };
    QByteArray stripped,canonical,reduced;
    const auto reimport=[&](const QByteArray &source) {
        d053::Repairs repairs;
        d052::Path p,q;
        std::array<QByteArray,2> v{};
        if(!d053::prepare(source,stripped,repairs) ||
           !d052::prepare(stripped,canonical,p,q) ||
           !d051::stripSecond(canonical,reduced,v))return false;
        return save(realFile,source) && load(real,realFile) &&
            save(bareFile,stripped) && load(barePanel,bareFile) &&
            save(canonicalFile,canonical) && load(full,canonicalFile) &&
            save(baselineFile,reduced) && load(baseline,baselineFile);
    };
    const auto run=[&](const QByteArray &source,QByteArray &readout) {
        if(!save(realFile,source))return -1;
        QProcess process;
        process.start(importer,{realFile.fileName(),"--d053-source-midspan-repair-v1"});
        if(!process.waitForFinished(10000) ||
           process.exitStatus()!=QProcess::NormalExit)return -1;
        readout=process.readAllStandardOutput();
        return process.exitCode();
    };
    const auto accepted=[&](const QByteArray &source,const QByteArray &expected) {
        if(!reimport(source))return false;
        QByteArray receipt;
        if(run(source,receipt)!=0 ||
           !d053::decode(receipt,source,&real,stripped,&barePanel,canonical,&full,reduced,&baseline) ||
           !receipt.contains(expected))return false;
        QByteArray forged=receipt;
        forged.replace("|GLOBAL_ELECTRICAL|NOT_VERIFIED","|GLOBAL_ELECTRICAL|VERIFIED");
        if(forged==receipt ||
           d053::decode(forged,source,&real,stripped,&barePanel,canonical,&full,reduced,&baseline))return false;
        QByteArray stale=source;stale.replace("NET_A","NET_B");
        if(stale!=source &&
           d053::decode(receipt,stale,&real,stripped,&barePanel,canonical,&full,reduced,&baseline))
           return false;
        return true;
    };
    bool all=true;
    for(size_t i=0;i<states.size();++i) {
        if(!accepted(states[i].first,states[i].second)) {
            qWarning()<<"D053 state fail"<<i;all=false;
        }
    }
    const QByteArray repaired=states[3].first;
    const auto mutate=[&](const QByteArray &from,const QByteArray &to) {
        QByteArray src=repaired;src.replace(from,to);return src;
    };
    QByteArray netB=repaired;netB.replace("NET_A","NET_B");
    QByteArray crlf=repaired;crlf.replace("\n","\r\n");
    QByteArray oneTick=mutate("288 300 1 8 0","297.99 300 1 8 0");
    oneTick.replace("312 300 1 8 0","302.01 300 1 8 0");
    oneTick.replace("288 400 1 8 0","297.99 400 1 8 0");
    oneTick.replace("312 400 1 8 0","302.01 400 1 8 0");
    for(const auto &positive:std::array<QByteArray,3>{{netB,crlf,oneTick}}) {
        if(!accepted(positive,"D|REPAIR_BOTH|16|16|1|1|REACHABLE\n")) {
            qWarning()<<"D053 positive variation failed";all=false;
        }
    }
    const std::array<QByteArray,12> hostile={{
        mutate("288 300 1 8 0","298 300 1 8 0"),
        mutate("312 300 1 8 0","302 300 1 8 0"),
        mutate("288 400 1 8 0","298.01 400 1 8 0"),
        mutate("312 400 1 8 0","301.99 400 1 8 0"),
        mutate("312 300 1 8 0","312 300 2 8 0"),
        mutate("312 300 1 8 0","312 300 1 9 0"),
        mutate("U1.1 U2.1\n288","U1.2 U2.1\n288"),
        mutate("PAD 2 3\n-2 30 S 0","PAD 2 3\n-2 30 R 0"),
        mutate("NET_A","NET_C"),
        mutate("*END*",repairA+"*END*"),
        mutate("*END*","FORGED\n*END*"),
        mutate("290 300 1 8 0","270 300 1 8 0")
    }};
    for(size_t i=0;i<hostile.size();++i) {
        QByteArray output;
        if(run(hostile[i],output)==0) {
            qWarning()<<"D053 Rust hostile accepted"<<i;all=false;
        }
        if(reimport(hostile[i]) &&
           !d053::reconstruct(hostile[i],&real,stripped,&barePanel,
                              canonical,&full,reduced,&baseline).isEmpty()) {
            qWarning()<<"D053 Qt hostile accepted"<<i;all=false;
        }
    }
    qInfo()<<"D053_NATIVE_QT_SOURCE_REPAIR"<<all<<"GLOBAL_ELECTRICAL_NOT_VERIFIED";
    return all?0:1;
}
