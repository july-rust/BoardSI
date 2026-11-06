// D_043 native Qt independent SOURCE-validated cut-set regression.
#include "preview_import.h"
#include "c005_reference_risk.h"
#include "d043_local_cutset_qt.h"
#include <QApplication>
#include <QProcess>
#include <QTemporaryFile>
#include <QEventLoop>
#include <QTimer>
#include <QFile>

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
    auto run=[&](const QVector<int> &cuts,QByteArray &wire) {
        QStringList args{temp.fileName(),"--d043-local-trace-cutset-v1"};
        for(const int cut:cuts)args.append("T"+QString::number(cut));
        QProcess process;
        process.start(importer,args);
        if(!process.waitForFinished(10000) ||
           process.exitStatus()!=QProcess::NormalExit)return -1;
        wire=process.readAllStandardOutput();
        return process.exitCode();
    };
    if(!save(original) || !load())return 2;
    struct Case { QVector<int> cuts; int nodes,edges,comps,cycles; bool reachable; };
    QVector<Case> cases;
    cases.append({{},14,14,1,1,true});
    for(int i=4;i<=9;++i)cases.append({{i},13,12,1,0,true});
    for(int i=5;i<=9;++i)cases.append({{4,i},12,10,2,0,false});
    if(cases.size()!=12)return 2;
    QByteArray baseline;
    bool passes=true;
    for(const auto &test:cases) {
        QByteArray wire;
        if(run(test.cuts,wire)!=0 ||
           !d043::decode(wire,original,&panel,test.cuts)) {
            passes=false;
            break;
        }
        const auto metric="COUNT|NODES|"+QByteArray::number(test.nodes)+
            "|EDGES|"+QByteArray::number(test.edges)+
            "|LOCAL_COMPONENTS|"+QByteArray::number(test.comps)+
            "|LOCAL_CYCLES|"+QByteArray::number(test.cycles)+
            "|U1_U2_REACHABLE|"+(test.reachable?"YES":"NO")+"\n";
        if(!wire.contains(metric))passes=false;
        if(test.cuts.isEmpty())baseline=wire;
    }
    if(!passes || baseline.isEmpty())return 1;
    const auto forged=QByteArray(baseline).replace("COUNT|NODES|14|","COUNT|NODES|13|");
    passes=forged!=baseline && !d043::decode(forged,original,&panel,{});
    QByteArray bad;
    passes=passes && run({2},bad)!=0 && run({4,4},bad)!=0 &&
           run({6,4},bad)!=0 && run({5,6},bad)!=0;
    // Freshly imported alternate SOURCE invalidates old receipt.
    QByteArray netB=original;
    netB.replace("NET_A","NET_B");
    if(netB==original || !save(netB) || !load())return 2;
    QByteArray bWire;
    passes=passes && run({4,7},bWire)==0 &&
           d043::decode(bWire,netB,&panel,{4,7}) &&
           !d043::decode(baseline,original,&panel,{});
    // Unsupported source is not equivalent to a supported counterfactual cut.
    QByteArray missing=original;
    missing.replace("U1.2 U2.2\n150 250 1 10 0\n450 250 1 10 0\n","");
    if(missing==original || !save(missing) || !load())return 2;
    passes=passes && run({4},bad)!=0 &&
           d043::reconstruct(missing,&panel,{4}).isEmpty();
    // A modified Pin-Pair must fail SOURCE ownership validation even if
    // the imported artwork still describes the same visible trace segment.
    QByteArray owner=original;
    owner.replace("U1.1 U1.2\n200 300 1 10 0","TEMP_OWNER\n200 300 1 10 0");
    owner.replace("U1.2 U2.2\n200 350 1 10 0","U1.1 U1.2\n200 350 1 10 0");
    owner.replace("TEMP_OWNER\n200 300 1 10 0","U1.2 U2.2\n200 300 1 10 0");
    if(owner==original || !save(owner) || !load())return 2;
    passes=passes && run({},bad)!=0 && d043::reconstruct(owner,&panel,{}).isEmpty();
    qInfo()<<"D043_NATIVE_QT_TRACE_CUTSET"<<passes
           <<"SOURCE_LOCAL_ONLY"<<"GLOBAL_ELECTRICAL_NOT_VERIFIED";
    return passes?0:1;
}
