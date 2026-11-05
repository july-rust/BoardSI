// D_041 native Qt SOURCE bridge regression; independent of Rust receipt.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d041_local_bridge_qt.h"
#include <QApplication>
int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if(argc!=3)return 2;
    const QString importer=QString::fromLocal8Bit(argv[1]);
    QFile fixture(QString::fromLocal8Bit(argv[2]));
    if(!fixture.open(QIODevice::ReadOnly))return 2;
    QByteArray original=fixture.readAll();
    original.replace("\r\n","\n");
    fixture.close();
    QTemporaryFile temp;
    if(!temp.open())return 2;
    auto save=[&](const QByteArray &data) {
        return temp.resize(0) && temp.seek(0) &&
               temp.write(data)==data.size() && temp.flush();
    };
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    auto load=[&]() {
        panel.setStatusCallback([&loop](const QString &s) {
            if(s.startsWith("PARTIAL") || s.startsWith("FAILED"))loop.quit();
        });
        if(!panel.openPath(temp.fileName()))return false;
        QTimer timeout;
        timeout.setSingleShot(true);
        QObject::connect(&timeout,&QTimer::timeout,&loop,&QEventLoop::quit);
        timeout.start(8000);
        loop.exec();
        return panel.importStatus().startsWith("PARTIAL");
    };
    auto run=[&](QByteArray &wire) {
        QProcess process;
        process.start(importer,{temp.fileName(),"--d041-local-bridge-v1"});
        if(!process.waitForFinished(10000) ||
           process.exitStatus()!=QProcess::NormalExit)return -1;
        wire=process.readAllStandardOutput();
        return process.exitCode();
    };
    if(!save(original) || !load())return 2;
    QByteArray wire;
    if(run(wire)!=0 || !d041::decode(wire,original,&panel) ||
       !wire.contains("COUNT|NODES|9|EDGES|8|LOCAL_COMPONENTS|1"))return 1;
    QByteArray forgery=wire;
    forgery.replace("E|7|8|6|","E|7|8|5|");
    const bool forged=forgery!=wire && !d041::decode(forgery,original,&panel);
    QByteArray reverse=original;
    reverse.replace("200 350 1 10 0\n400 350 1 10 0\n*END*",
                    "400 350 1 10 0\n200 350 1 10 0\n*END*");
    if(reverse==original || !save(reverse) || !load())return 2;
    QByteArray reversed;
    const bool direction=run(reversed)==0 && d041::decode(reversed,reverse,&panel) &&
        reversed.mid(reversed.indexOf('\n'))==wire.mid(wire.indexOf('\n'));
    QByteArray onlyB=original;
    onlyB.replace("*SIGNAL* NET_A","*SIGNAL* NET_B");
    if(onlyB==original || !save(onlyB) || !load())return 2;
    QByteArray bWire;
    const bool netB=run(bWire)==0 && d041::decode(bWire,onlyB,&panel) &&
        bWire.contains("C|0|U1+U2|NET_B|1|9|8\n");
    const bool stale=!d041::decode(wire,onlyB,&panel);
    QByteArray gap=original;
    gap.replace("400 350 1 10 0\n*END*","400 350.01 1 10 0\n*END*");
    if(gap==original || !save(gap) || !load())return 2;
    QByteArray bad;
    const bool gapRejected=run(bad)!=0 && d041::reconstruct(gap,&panel).isEmpty();
    QByteArray crossNet=original;
    crossNet.replace("U2.1 U2.2\n400 300 1 10 0",
                     "*SIGNAL* NET_B\nU2.1 U2.2\n400 300 1 10 0");
    if(crossNet==original || !save(crossNet))return 2;
    const bool wrongNet=run(bad)!=0;
    const bool ok=forged && direction && netB && stale && gapRejected && wrongNet;
    qInfo()<<"D041_NATIVE_QT_LOCAL_BRIDGE"<<ok<<forged<<direction<<netB
           <<stale<<gapRejected<<wrongNet<<"GLOBAL_ELECTRICAL_NOT_VERIFIED";
    return ok?0:1;
}
