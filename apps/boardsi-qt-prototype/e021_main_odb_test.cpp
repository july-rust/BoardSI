// E_021: real main PreviewImportPanel -> Rust ODB++ -> source-bound IPC ->
// existing ImportedArtworkPanel; positive/negative/reimport/cancel (not SI).
#include "preview_import.h"
#include <QApplication>
#include <QCryptographicHash>
#include <QEventLoop>
#include <QFile>
#include <QGraphicsScene>
#include <QProcess>
#include <QPushButton>
#include <QTemporaryDir>
#include <QTimer>

static bool writeFile(const QString &path,const QByteArray &bytes) {
    QFile f(path);
    return f.open(QIODevice::WriteOnly|QIODevice::Truncate) &&
           f.write(bytes)==bytes.size() && f.flush();
}
static bool exercise(const QString &binary) {
    QTemporaryDir temp;
    if(!temp.isValid())return false;
    QDir root(temp.path());
    if(!root.mkpath("matrix") || !root.mkpath("steps/pcb/layers/sig1") ||
       !root.mkpath("steps/pcb/layers/sig2")) return false;
    const QByteArray matrix=
        "STEP {\nCOL=1\nNAME=pcb\n}\n"
        "LAYER {\nROW=1\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=sig1\nPOLARITY=POSITIVE\n}\n"
        "LAYER {\nROW=2\nCONTEXT=BOARD\nTYPE=SIGNAL\nNAME=sig2\nPOLARITY=POSITIVE\n}\n";
    const QByteArray a="UNITS=MM\nF 2\n$0 r100\n$1 rect200x100 M\n"
                       "L 0 0 1.5 0 0 P 0\nP 1.5 0 1 P 0 0\n";
    const QByteArray b="UNITS=MM\nF 2\n$0 r100\n$1 rect200x100 M\n"
                       "L 0 0 1.8 0 0 P 0\nP 1.8 0 1 P 0 0\n";
    const QString first=root.filePath("steps/pcb/layers/sig1/features");
    const QString second=root.filePath("steps/pcb/layers/sig2/features");
    if(!writeFile(root.filePath("matrix/matrix"),matrix) ||
       !writeFile(first,a) || !writeFile(second,a))return false;
    PreviewImportPanel panel;
    panel.setOdbImporterBinaryForTest(binary);
    auto *art=panel.imported();
    const auto run=[&](bool success) {
        QEventLoop loop;
        QTimer timer;
        timer.setSingleShot(true);
        QObject::connect(&timer,&QTimer::timeout,&loop,&QEventLoop::quit);
        bool completed=false,passed=false;
        panel.setStatusCallback([&](const QString &state){
            if(state.startsWith("CANONICAL SOURCE GEOMETRY") ||
               state.startsWith("FAILED") || state.startsWith("CANCELLED")) {
                completed=true;
                passed=state.startsWith("CANONICAL SOURCE GEOMETRY");
                loop.quit();
            }
        });
        if(!panel.openOdbPath(temp.path(),"pcb") || art->canonicalMode() ||
           !panel.acceptedSourceSha256().isEmpty()) return false;
        timer.start(45000);loop.exec();
        return completed && passed==success && !panel.busy();
    };
    if(!run(true) || !art->canonicalMode() || !art->odbMode() ||
       art->canonicalGlyphCount()!=4 || !art->canonicalContains(0,{0.75,0}) ||
       panel.importedLayerCount()!=2 ||
       panel.acceptedSourceSha256().size()!=64 ||
       !art->canonicalSourceSha().contains(
           QString::fromLatin1(panel.acceptedSourceSha256())))return false;
    const QString shaA=art->canonicalSourceSha();
    auto *scene=art->findChild<QGraphicsScene*>();
    if(!scene)return false;
    const auto *hit=scene->itemAt(QPointF(0.75,0),QTransform());
    if(!hit || hit->data(3).toString()!="E021OdbCanonical" ||
       !hit->data(2).toString().contains(shaA) ||
       !art->selectCanonicalGlyph(0) ||
       !art->summaryText().contains("E_021"))return false;
    auto *hide=panel.findChild<QPushButton*>("importedHideAllLayers");
    auto *show=panel.findChild<QPushButton*>("importedShowAllLayers");
    if(!hide || !show)return false;
    hide->click();
    if(art->canonicalVisibleGlyphCount()!=0 || art->selectCanonicalGlyph(0))return false;
    show->click();
    if(art->canonicalVisibleGlyphCount()!=4)return false;
    if(!writeFile(first,b) || !run(true) || !art->odbMode() ||
       art->canonicalSourceSha()==shaA ||
       art->summaryText().contains(shaA) || art->canonicalGlyphCount()!=4 ||
       !art->canonicalContains(0,{0.9,0}))return false;
    // A supported file mutation must never retain the preceding selection.
    QByteArray invalid=a;
    invalid.replace("L 0 0", "X 0 0");
    if(!writeFile(first,invalid) || !run(false) ||
       art->canonicalMode() || art->canonicalGlyphCount()!=0 ||
       !panel.acceptedSourceSha256().isEmpty())return false;
    // An unsupported transport version must fail closed.
    if(!writeFile(first,a))return false;
    QProcess producer;
    producer.start(binary,{temp.path(),"pcb"});
    if(!producer.waitForFinished(30000) || producer.exitCode()!=0)return false;
    const QByteArray good=producer.readAllStandardOutput();
    e021ipc::Source decoded;
    if(!e021ipc::decode(good,temp.path(),decoded))return false;
    QByteArray version=good;
    version.replace("BOARDSI_ODB_QT_V1","BOARDSI_ODB_QT_V2");
    if(e021ipc::decode(version,temp.path(),decoded))return false;
    QByteArray tampered=good;
    tampered.replace("T|1|","T|99|");
    if(e021ipc::decode(tampered,temp.path(),decoded))return false;
    // Cancelling a real child process before its completion must discard its result.
    panel.setStatusCallback({});
    if(!panel.openOdbPath(temp.path(),"pcb"))return false;
    auto *cancel=panel.findChild<QPushButton*>("importCancelButton");
    if(!cancel)return false;
    cancel->click();
    QEventLoop drained;
    QTimer stop;
    stop.setSingleShot(true);
    QObject::connect(&stop,&QTimer::timeout,&drained,&QEventLoop::quit);
    QTimer poll;
    QObject::connect(&poll,&QTimer::timeout,&drained,[&]{
        if(!panel.busy())drained.quit();
    });
    stop.start(15000);poll.start(10);drained.exec();
    return !panel.busy() && !art->canonicalMode() &&
           !panel.importStatus().startsWith("CANONICAL SOURCE GEOMETRY");
}
int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if(app.arguments().size()!=2)return 2;
    return exercise(app.arguments()[1])?0:1;
}
