// E_010: actual main PreviewImportPanel -> Rust Canonical IPC -> main
// ImportedArtworkPanel picking and distinct-source reimport (SOURCE only).
#include "preview_import.h"
#include <QApplication>
#include <QCheckBox>
#include <QComboBox>
#include <QCryptographicHash>
#include <QEventLoop>
#include <QFile>
#include <QPushButton>
#include <QTemporaryDir>
#include <QTimer>

static bool run(const QString &binary, const QString &sourcePath) {
    QFile input(sourcePath);
    if (!input.open(QIODevice::ReadOnly)) return false;
    const QByteArray original=input.readAll();
    if (!original.contains("*SIGNAL* NET_A") ||
        !original.contains("!PADS-POWERPCB-V9.4-MILS!")) return false;
    QByteArray second=original;
    second.replace("*SIGNAL* NET_A","*SIGNAL* NET_B");
    QByteArray invalid=original;
    invalid.replace("!PADS-POWERPCB-V9.4-MILS!",
                    "!PADS-POWERPCB-V9.3-MILS!");
    if (second==original || invalid==original) return false;
    QTemporaryDir directory;
    if (!directory.isValid()) return false;
    const auto save=[&](const QString &name,const QByteArray &bytes) {
        QFile out(directory.filePath(name));
        return out.open(QIODevice::WriteOnly) &&
               out.write(bytes)==bytes.size() && out.flush();
    };
    if (!save("source-b.asc",second) || !save("unsupported.asc",invalid)) return false;
    const QString shaA=QString::fromLatin1(
        QCryptographicHash::hash(original,QCryptographicHash::Sha256).toHex());
    const QString shaB=QString::fromLatin1(
        QCryptographicHash::hash(second,QCryptographicHash::Sha256).toHex());
    if (shaA==shaB) return false;
    PreviewImportPanel panel;
    panel.setCanonicalImporterBinaryForTest(binary);
    const auto import=[&](const QString &path,bool expectSuccess) {
        bool completed=false,success=false;
        QEventLoop loop;
        QTimer deadline;
        deadline.setSingleShot(true);
        QObject::connect(&deadline,&QTimer::timeout,&loop,&QEventLoop::quit);
        panel.setStatusCallback([&](const QString &status) {
            if (status.startsWith("CANONICAL SOURCE GEOMETRY")) {
                completed=true; success=true; loop.quit();
            } else if (status.startsWith("FAILED") ||
                       status.startsWith("CANCELLED")) {
                completed=true; loop.quit();
            }
        });
        if (!panel.openCanonicalPath(path) ||
            panel.imported()->canonicalMode() ||
            panel.imported()->canonicalGlyphCount()!=0 ||
            !panel.acceptedSourceSha256().isEmpty()) return false;
        deadline.start(45000);
        loop.exec();
        return completed && success==expectSuccess && !panel.busy();
    };
    auto *art=panel.imported();
    if (!import(sourcePath,true) || !art->canonicalMode() ||
        art->canonicalSourceSha()!=shaA ||
        panel.acceptedSourceSha256()!=shaA.toLatin1() ||
        art->canonicalGlyphCount()<2) return false;
    // PADS V9.4 SOURCE GRID: 1 tick = 0.000254 mm (not 1 mil).
    // Public fixture via at (300, 200) ticks; 35 tick OD / 15 tick drill.
    const QPointF centre(0.0762,-0.0508);
    const QPointF copperPoint=centre+QPointF(0.003,0);
    auto *scene=art->findChild<QGraphicsScene*>();
    if (!scene ||
        art->canonicalContains(0,centre) ||
        !art->canonicalContains(0,copperPoint)) return false;
    // The actual QGraphicsScene hit-test must agree with Canonical copper,
    // not just the model's standalone footprint helper.
    auto *hit=scene->itemAt(copperPoint,QTransform());
    if (!hit || hit->data(3).toString()!="D05605Canonical" ||
        !hit->flags().testFlag(QGraphicsItem::ItemIsSelectable) ||
        !hit->data(2).toString().contains(shaA) ||
        scene->itemAt(centre,QTransform()) ||
        !art->selectCanonicalGlyph(0) ||
        !art->summaryText().contains("Selected: Canonical Via") ||
        !art->summaryText().contains("SOURCE SHA256 "+shaA) ||
        !art->summaryText().contains("NET_A")) return false;
    auto *hide=panel.findChild<QPushButton*>("importedHideAllLayers");
    auto *show=panel.findChild<QPushButton*>("importedShowAllLayers");
    if (!hide || !show) return false;
    hide->click();
    if (art->canonicalVisibleGlyphCount()!=0 ||
        art->selectCanonicalGlyph(0)) return false;
    show->click();
    if (art->canonicalVisibleGlyphCount()!=art->canonicalGlyphCount() ||
        !art->selectCanonicalGlyph(0)) return false;

    auto *nets=panel.findChild<QComboBox*>("importedNetSelect");
    auto *only=panel.findChild<QCheckBox*>("importedNetOnly");
    if (!nets || !only) return false;
    int netA=-1;
    for (int i=0;i<nets->count();++i)
        if (nets->itemText(i).contains("NET_A")) {netA=i;break;}
    if (netA<0) return false;
    nets->setCurrentIndex(netA);
    only->setChecked(true);
    if (art->canonicalVisibleGlyphCount()!=art->canonicalGlyphCount() ||
        !art->selectCanonicalGlyph(0)) return false;

    // Re-import different valid SOURCE using the SAME main Qt controller.
    if (!import(directory.filePath("source-b.asc"),true) ||
        !art->canonicalMode() ||
        art->canonicalSourceSha()!=shaB ||
        panel.acceptedSourceSha256()!=shaB.toLatin1() ||
        art->canonicalGlyphCount()<2 ||
        art->summaryText().contains("Selected: Canonical Via") ||
        art->summaryText().contains(shaA) ||
        art->summaryText().contains("NET_A") ||
        !art->selectCanonicalGlyph(0) ||
        !art->summaryText().contains("SOURCE SHA256 "+shaB) ||
        !art->summaryText().contains("NET_B") ||
        art->summaryText().contains("NET_A")) return false;

    // Unsupported V9.3 header fails in the REAL Rust importer; the main Qt
    // controller must not keep the preceding Source-B selection on screen.
    if (!import(directory.filePath("unsupported.asc"),false) ||
        art->canonicalMode() || art->canonicalGlyphCount()!=0 ||
        !art->canonicalSourceSha().isEmpty() ||
        !panel.acceptedSourceSha256().isEmpty() ||
        !panel.importStatus().startsWith("FAILED") ||
        art->selectCanonicalGlyph(0)) return false;
    panel.stop();
    return !panel.busy();
}
int main(int argc,char **argv) {
    QApplication app(argc,argv);
    const auto args=app.arguments();
    if (args.size()!=3) return 2;
    return run(args[1],args[2]) ? 0 : 1;
}
