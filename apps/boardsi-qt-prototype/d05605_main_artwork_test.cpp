// D_056_05: native MAIN ImportedArtworkPanel acceptance (not a side viewer).
#include "preview_import.h"
#include <QApplication>
#include <QCryptographicHash>
#include <QEventLoop>
#include <QFileInfo>
#include <QThread>
#include <QFile>
#include <QPushButton>
#include <QTimer>
#include <QDebug>

static QString key(const QByteArray &source) {
    QByteArray text=source; text.append('\0');
    return QString::fromLatin1(text.toHex());
}
static d056ipc::Source synthetic(int layerCount,int first,int last) {
    d056ipc::Source s;
    s.sha=QString(64,QLatin1Char('a'));
    s.format="PADS_ASC";s.version="V9.4-MILS";
    const QString frame=key("frame"),stack=key("stack"),drill=key("drill");
    s.frames.push_back(d056ipc::Frame{frame,"PADS tick","127:500000000",
        "0:1","0:1",0,false,false});
    for(int layer=1;layer<=layerCount;++layer) {
        s.layerIds.push_back(key("layer-"+QByteArray::number(layer)));
        s.layerOrders.push_back(layer-1);
    }
    d056ipc::Stack st;st.id=stack;
    st.pads.push_back(d056ipc::Pad{first,"CONDUCTIVE","RECT",
        "127:100000","127:125000"});
    if (last>first+1)
        st.pads.push_back(d056ipc::Pad{first+1,"ANTIPAD","CIRCLE",
            "127:50000","0:1"});
    st.pads.push_back(d056ipc::Pad{last,"CONDUCTIVE","ANNULUS",
        "127:85000","127:170000"});
    s.stacks.push_back(st);
    s.drills.push_back(d056ipc::Drill{drill,"127:500000",first,last,true});
    s.vias.push_back(d056ipc::Via{key("via"),stack,drill,"-",frame,
        last==layerCount && first==1 ? "THROUGH" :
        first==1 || last==layerCount ? "BLIND" : "BURIED",
        1000,2000,first,last});
    return s;
}
static bool verifySynthetic() {
    ImportedArtworkPanel panel;
    const QString sha(64,QLatin1Char('a'));
    for(const auto &test: {std::array<int,3>{2,1,2},{4,1,4},
             {6,2,3},{6,2,4},{8,3,6},{64,17,41}}) {
        const auto source=synthetic(test[0],test[1],test[2]);
        if(!panel.loadCanonical(source,sha) || !panel.canonicalMode() ||
           panel.canonicalSourceSha()!=sha ||
           panel.canonicalGlyphCount()!=(test[2]>test[1]+1 ? 3 : 2) ||
           panel.canonicalLayerGlyphCount(test[1])!=1 ||
           panel.canonicalLayerGlyphCount(test[2])!=1 ||
           panel.canonicalLayerGlyphCount(1)!=(test[1]==1 ? 1 : 0))
            return false;
        if(test[2]>test[1]+1 &&
           panel.canonicalLayerGlyphCount(test[1]+1)!=1)return false;
        const QPointF centre(0.254,-0.508);
        const int lastGlyph=panel.canonicalGlyphCount()-1;
        if(panel.canonicalContains(lastGlyph,centre) ||
           !panel.canonicalContains(lastGlyph,centre+QPointF(0.5,0)) ||
           panel.canonicalContains(0,centre) ||
           !panel.canonicalContains(0,centre+QPointF(0.3,0)))
            return false;
        if(test[2]>test[1]+1 && panel.canonicalContains(1,centre))
            return false; // Antipad is never conductive/pickable.
        auto *hide=panel.findChild<QPushButton*>("importedHideAllLayers");
        auto *show=panel.findChild<QPushButton*>("importedShowAllLayers");
        if(!hide || !show)return false;
        hide->click();
        if(panel.canonicalVisibleGlyphCount()!=0)return false;
        show->click();
        if(panel.canonicalVisibleGlyphCount()!=panel.canonicalGlyphCount())
            return false;
        const auto rejected=synthetic(test[0],test[1],test[2]);
        if(panel.loadCanonical(rejected,QString(64,QLatin1Char('b'))) ||
           !panel.canonicalMode() || panel.canonicalSourceSha()!=sha)
            return false; // Transactional rejection preserves accepted model.
    }
    // Existing V1-V14 importer routes must still load using unchanged models.
    const ImportedTrack legacy{1,"legacy",0,0,254000,0,25400};
    if(!panel.load(2,0,{legacy}) || panel.canonicalMode() ||
       panel.canonicalGlyphCount()!=0 || panel.segmentCount()!=1)
        return false;
    panel.clearArtwork();
    return panel.canonicalSourceSha().isEmpty() &&
           panel.canonicalGlyphCount()==0;
}
static bool verifyActualRust(const QString &binary,const QString &path) {
    QFile file(path);
    if(!file.open(QIODevice::ReadOnly))return false;
    const QByteArray sha=QCryptographicHash::hash(
        file.readAll(),QCryptographicHash::Sha256).toHex();
    PreviewImportPanel panel;
    bool complete=false,successful=false;
    // A real child process with a guaranteed slow response allows the
    // existing main GUI Cancel button to be exercised without sleep races.
    panel.setCanonicalImporterBinaryForTest(QCoreApplication::applicationFilePath());
    if(!panel.openCanonicalPath(path))return false;
    auto *cancel=panel.findChild<QPushButton*>("importCancelButton");
    if(!cancel || !cancel->isEnabled())return false;
    cancel->click();
    QEventLoop cancelLoop;
    QTimer cancelPoll,cancelDeadline;
    bool cancelTimedOut=false;
    cancelDeadline.setSingleShot(true);
    QObject::connect(&cancelDeadline,&QTimer::timeout,&cancelLoop,[&] {
        cancelTimedOut=true;cancelLoop.quit();
    });
    QObject::connect(&cancelPoll,&QTimer::timeout,&cancelLoop,[&] {
        if(!panel.busy())cancelLoop.quit();
    });
    cancelPoll.start(10);cancelDeadline.start(10000);cancelLoop.exec();
    if(cancelTimedOut || panel.busy() || panel.imported()->canonicalMode() ||
       !panel.acceptedSourceSha256().isEmpty() ||
       !panel.importStatus().startsWith("CANCELLED"))return false;
    // Same controller must safely recover after cancellation.
    panel.setCanonicalImporterBinaryForTest(binary);
    QEventLoop loop;
    QTimer deadline;
    deadline.setSingleShot(true);
    QObject::connect(&deadline,&QTimer::timeout,&loop,&QEventLoop::quit);
    panel.setStatusCallback([&](const QString &message) {
        if(message.startsWith("CANONICAL SOURCE GEOMETRY")) {
            complete=true;successful=true;loop.quit();
        } else if(message.startsWith("FAILED") || message.startsWith("CANCELLED")) {
            complete=true;loop.quit();
        }
    });
    if(!panel.openCanonicalPath(path))return false;
    deadline.start(45000);
    loop.exec();
    if(!complete || !successful || !panel.imported()->canonicalMode() ||
       panel.imported()->canonicalSourceSha()!=QString::fromLatin1(sha) ||
       panel.importedLayerCount()<2 || panel.imported()->canonicalGlyphCount()<1)
        return false;
    panel.stop();
    return !panel.busy();
}
int main(int argc,char **argv) {
    QApplication app(argc,argv);
    const auto args=app.arguments();
    if(args.size()==2 && QFileInfo(args[1]).isFile()) {
        QThread::msleep(3000); // Child process intentionally killed by UI Cancel.
        return 1;
    }
    if(args.size()==2 && args[1]=="--selftest")
        return verifySynthetic() ? 0 : 1;
    if(args.size()==4 && args[1]=="--source")
        return verifyActualRust(args[2],args[3]) ? 0 : 1;
    qCritical("Usage: boardsi-d05605-main-artwork-test --selftest | --source BIN FILE");
    return 2;
}
