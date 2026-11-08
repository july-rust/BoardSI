// Standalone opt-in UI for SOURCE-declared D_056 per-layer via geometry.
// Does NOT modify the legacy main artwork protocol or certify electrical SI.
#include "d056_via_layers.h"
#include <QApplication>
#include <QDebug>
#include <QTimer>

static bool selftest() {
    const QByteArray input =
        "BOARDSI_SOURCE_VIA_TOPOLOGY_V2|V9.4-MILS|6|1|GEOMETRY_ONLY|SI_NOT_RUN\n"
        "V|0|VIA_A|NET_A|10|100|200|2|4|Buried|1500|true|3\n"
        "P|0|2|S|3200\n"
        "P|0|3|RA|5000\n"
        "P|0|4|A|4000|2500\n"
        "END|1|SOURCE_PROVENANCE_ONLY|GLOBAL_CONNECTIVITY_NOT_VERIFIED\n";
    d056::Panel panel;
    panel.show();
    if (!panel.loadWire(input) || panel.source().layers != 6 ||
        panel.source().vias.size() != 1 ||
        panel.source().vias.first().pads.size() != 3 ||
        !panel.selectLayer(2) || panel.displayedItemCount() != 1)
        return false;
    if (!panel.selectLayer(1) || panel.displayedItemCount() != 0)
        return false; // Hole-only/nonexistent layers do not invent copper.
    if (!panel.selectLayer(3) || panel.displayedItemCount() != 1)
        return false;
    auto *clearance = dynamic_cast<QGraphicsPathItem *>(panel.scene()->items().first());
    if (!clearance || clearance->brush().style() != Qt::NoBrush ||
        clearance->pen().style() != Qt::DashLine) return false;
    if (!panel.selectLayer(4) || panel.displayedItemCount() != 1)
        return false;
    auto *ring = dynamic_cast<QGraphicsPathItem *>(panel.scene()->items().first());
    if (!ring || ring->path().contains(QPointF(0, 0)) ||
        !ring->path().contains(QPointF(0.4, 0))) return false;
    const auto bad = [&](QByteArray changed) {
        d056::Source parsed;
        return !d056::decode(changed, parsed) && parsed.vias.isEmpty();
    };
    if (!bad(QByteArray(input).replace("P|0|3|RA|5000", "P|0|2|RA|5000")) ||
        !bad(QByteArray(input).replace("A|4000|2500", "A|4000|900")) ||
        !bad(QByteArray(input).replace("|Buried|", "|Through|")) ||
        !bad(QByteArray(input).replace("GLOBAL_CONNECTIVITY_NOT_VERIFIED",
                                       "VERIFIED")) ||
        !bad(QByteArray(input).replace("END|1|", "END|2|")) ||
        !bad(QByteArray(input).replace("VIA_A", "VIA|A"))) return false;
    // An empty but source-valid VIA inventory is not a fabricated via.
    const QByteArray empty =
        "BOARDSI_SOURCE_VIA_TOPOLOGY_V1|V9.4-MILS|4|0|GEOMETRY_ONLY|SI_NOT_RUN\n"
        "END|0|SOURCE_PROVENANCE_ONLY|GLOBAL_CONNECTIVITY_NOT_VERIFIED\n";
    d056::Source noVia;
    if (!d056::decode(empty, noVia) || noVia.vias.size() != 0 ||
        noVia.layers != 4 || !panel.loadWire(empty) ||
        panel.displayedItemCount() != 0) return false;
    const QByteArray v1 =
        "BOARDSI_SOURCE_VIA_TOPOLOGY_V1|V9.4-MILS|2|1|GEOMETRY_ONLY|SI_NOT_RUN\n"
        "V|0|V0|N0|4|0|0|1|2|Through|1000|true|2\n"
        "P|0|1|R|3000\nP|0|2|R|4000\n"
        "END|1|SOURCE_PROVENANCE_ONLY|GLOBAL_CONNECTIVITY_NOT_VERIFIED\n";
    d056::Source legacy;
    if (!d056::decode(v1, legacy) || legacy.vias[0].pads.size() != 2 ||
        !bad(QByteArray(v1).replace("P|0|2|R|4000", "P|0|2|SA|4000")))
        return false;
    return true;
}

int main(int argc, char **argv) {
    QApplication app(argc, argv);
    const QStringList args = app.arguments();
    if (args.size() == 2 && args[1] == "--selftest") {
        if (selftest()) return 0;
        qCritical("D_056 layer-aware Qt native geometry regression FAILED");
        return 1;
    }
    if (args.size() != 3) {
        qCritical("usage: boardsi-d056-via-viewer /absolute/boardsi-import-via-topology /absolute/board.asc");
        return 2;
    }
    d056::Panel panel;
    panel.setWindowTitle("BoardSI D_056 source-layer via geometry (SI NOT_RUN)");
    panel.show();
    QTimer::singleShot(0, &panel, [&] {
        if (!panel.importSource(args[1], args[2])) QCoreApplication::exit(2);
    });
    return app.exec();
}
