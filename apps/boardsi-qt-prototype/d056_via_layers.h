// D_056 opt-in layer-aware PADS via diagnostic viewer.
// This is NOT the legacy Qt Artwork V1-V14 protocol and does not certify
// plating, complete PCB connectivity, Open/Short or an SI simulation.
#pragma once
#include <QComboBox>
#include <QCryptographicHash>
#include <QFile>
#include <QFileInfo>
#include <QGraphicsPathItem>
#include <QGraphicsScene>
#include <QGraphicsView>
#include <QLabel>
#include <QPainterPath>
#include <QProcess>
#include <QSet>
#include <QTimer>
#include <QVector>
#include <QBrush>
#include <QPen>
#include <utility>
#include <QVBoxLayout>
#include <QWidget>
#include <algorithm>
#include <cmath>
#include <limits>

namespace d056 {
struct Pad {
    int layer = 0;
    QString shape;
    qint64 outer = 0, inner = 0;
    bool conductive() const { return shape == "R" || shape == "S" || shape == "A"; }
};
struct Via {
    int sourceLine = 0, first = 0, last = 0;
    QString name, net, kind;
    qint64 x = 0, y = 0, drill = 0;
    bool plated = false;
    QVector<Pad> pads;
};
struct Source {
    int layers = 0;
    QVector<Via> vias;
};

// All coordinates and diameters in the D_056 source wire are integer
// 0.01-mil ticks. No float approximation is used to accept/reject data.
// Millimetres below are for drawing ONLY (1 tick = 0.000254 mm).
inline constexpr double mmPerTick = 0.000254;
inline bool integer(const QString &text, qint64 &value) {
    if (text.isEmpty() || text.size() > 20) return false;
    bool ok = false;
    value = text.toLongLong(&ok, 10);
    return ok;
}
inline bool bounded(const QString &s, qint64 low, qint64 high, qint64 &n) {
    return integer(s, n) && n >= low && n <= high;
}
inline bool decode(const QByteArray &bytes, Source &out) {
    // No partially accepted inventory and no lines silently skipped.
    out = {};
    if (bytes.isEmpty() || bytes.size() > 1'000'000 || !bytes.endsWith('\n') ||
        QString::fromUtf8(bytes).toUtf8() != bytes) return false;
    const QStringList lines = QString::fromUtf8(bytes).split('\n', Qt::KeepEmptyParts);
    if (lines.size() < 4 || lines.size() > 30000 || !lines.last().isEmpty()) return false;
    const auto h = lines.first().split('|', Qt::KeepEmptyParts);
    if (h.size() != 6 ||
        (h[0] != "BOARDSI_SOURCE_VIA_TOPOLOGY_V1" &&
         h[0] != "BOARDSI_SOURCE_VIA_TOPOLOGY_V2") ||
        h[1] != "V9.4-MILS" || h[4] != "GEOMETRY_ONLY" ||
        h[5] != "SI_NOT_RUN") return false;
    const bool v2 = h[0].endsWith("_V2");
    qint64 layers = 0, count = 0;
    if (!bounded(h[2], 2, 64, layers) ||
        !bounded(h[3], 0, 10000, count)) return false;
    Source parsed;
    parsed.layers = int(layers);
    int pendingPads = 0;
    bool contacts = false, ended = false;
    for (int i = 1; i < lines.size() - 1; ++i) {
        const auto f = lines[i].split('|', Qt::KeepEmptyParts);
        if (f.isEmpty()) return false;
        if (f[0] == "V" && f.size() == 13 && !contacts && !ended) {
            if (pendingPads != 0 || parsed.vias.size() >= count) return false;
            qint64 index, sourceLine, x, y, first, last, drill, pads;
            if (!bounded(f[1], parsed.vias.size(), parsed.vias.size(), index) ||
                f[2].isEmpty() || f[2].size() > 128 ||
                f[3].size() > 128 ||
                !bounded(f[4], 1, 10000000, sourceLine) ||
                !bounded(f[5], -5600000, 5600000, x) ||
                !bounded(f[6], -5600000, 5600000, y) ||
                !bounded(f[7], 1, layers - 1, first) ||
                !bounded(f[8], first + 1, layers, last) ||
                !bounded(f[10], 1, 100000, drill) ||
                !bounded(f[12], 1, 64, pads) ||
                (f[11] != "true" && f[11] != "false")) return false;
            const bool top = first == 1, bottom = last == layers;
            if (!((f[9] == "Through" && top && bottom) ||
                  (f[9] == "Blind" && top != bottom) ||
                  (f[9] == "Buried" && !top && !bottom) ||
                  (f[9] == "Microvia" && last == first + 1))) return false;
            Via via;
            via.name = f[2]; via.net = f[3]; via.sourceLine = int(sourceLine);
            via.x = x; via.y = y; via.first = int(first); via.last = int(last);
            via.drill = drill; via.kind = f[9]; via.plated = f[11] == "true";
            parsed.vias.push_back(std::move(via));
            pendingPads = int(pads);
        } else if (f[0] == "P" && !contacts && !ended && pendingPads > 0) {
            if ((f.size() != 5 && f.size() != 6) || parsed.vias.isEmpty()) return false;
            qint64 index, layer, outer, inner = 0;
            const int owner = parsed.vias.size() - 1;
            Via &via = parsed.vias.last();
            if (!bounded(f[1], owner, owner, index) ||
                !bounded(f[2], via.first, via.last, layer) ||
                !bounded(f[4], via.drill + 1, 5600000, outer)) return false;
            if (f[3] == "A") {
                if (!v2 || f.size() != 6 ||
                    !bounded(f[5], via.drill, outer - 1, inner)) return false;
            } else if ((f[3] != "R" && f[3] != "S" &&
                        f[3] != "RA" && f[3] != "SA") ||
                       f.size() != 5 || (!v2 && f[3] != "R")) return false;
            for (const auto &p : via.pads) if (p.layer == layer) return false;
            via.pads.push_back(Pad{int(layer), f[3], outer, inner});
            --pendingPads;
        } else if (f[0] == "C" && f.size() == 6 &&
                   pendingPads == 0 && parsed.vias.size() == count && !ended) {
            qint64 trace, sourceLine, owner;
            if (!bounded(f[1], 0, 100000, trace) ||
                !bounded(f[2], 1, 10000000, sourceLine) ||
                !bounded(f[3], 0, count - 1, owner) ||
                f[5] != "SOURCE_LOCAL_CANDIDATE_ONLY" ||
                (f[4] != "Same" && f[4] != "Different" &&
                 f[4] != "Unknown")) return false;
            contacts = true;
        } else if (f[0] == "END" && f.size() == 4 && !ended &&
                   pendingPads == 0 && parsed.vias.size() == count) {
            qint64 expected = 0;
            if (!bounded(f[1], count, count, expected) ||
                f[2] != "SOURCE_PROVENANCE_ONLY" ||
                f[3] != "GLOBAL_CONNECTIVITY_NOT_VERIFIED" ||
                i != lines.size() - 2) return false;
            ended = true;
        } else {
            return false;
        }
    }
    if (!ended) return false;
    for (const auto &via : parsed.vias) {
        bool copper = false;
        for (const auto &pad : via.pads) copper |= pad.conductive();
        if (!copper) return false;
    }
    out = std::move(parsed);
    return true;
}

inline QPainterPath footprint(const Pad &pad) {
    const double d = double(pad.outer) * mmPerTick, r = d / 2.0;
    QPainterPath p;
    p.setFillRule(Qt::OddEvenFill);
    if (pad.shape == "S" || pad.shape == "SA") p.addRect(-r, -r, d, d);
    else p.addEllipse(QPointF(0, 0), r, r);
    if (pad.shape == "A") {
        const double inner = double(pad.inner) * mmPerTick / 2.0;
        p.addEllipse(QPointF(0, 0), inner, inner);
    }
    return p;
}

class Panel final : public QWidget {
    QComboBox *layer_ = nullptr;
    QLabel *status_ = nullptr;
    QGraphicsScene *scene_ = nullptr;
    QGraphicsView *view_ = nullptr;
    QProcess worker_;
    QTimer deadline_;
    QByteArray output_;
    QString file_;
    QByteArray originalHash_;
    Source source_;

    static QByteArray digest(const QString &path) {
        QFile f(path);
        if (!QFileInfo(path).isFile() || !f.open(QIODevice::ReadOnly) ||
            f.size() < 1 || f.size() > 64 * 1024 * 1024) return {};
        QCryptographicHash h(QCryptographicHash::Sha256);
        while (!f.atEnd()) {
            QByteArray b = f.read(65536);
            if (b.isEmpty()) return {};
            h.addData(b);
        }
        return h.result();
    }
    void showLayer() {
        scene_->clear();
        const int wanted = layer_->currentData().toInt();
        for (const Via &via : source_.vias) for (const Pad &pad : via.pads) {
            if (wanted != 0 && pad.layer != wanted) continue;
            auto *item = scene_->addPath(footprint(pad));
            item->setPos(double(via.x) * mmPerTick, -double(via.y) * mmPerTick);
            QPen pen(pad.conductive() ? QColor(36, 106, 175) : QColor(130, 130, 130));
            pen.setCosmetic(true);
            if (!pad.conductive()) pen.setStyle(Qt::DashLine);
            item->setPen(pen);
            item->setBrush(pad.conductive() ? QBrush(QColor(110, 190, 225, 180))
                                           : QBrush(Qt::NoBrush));
            item->setToolTip(QString("%1 | Net %2 | Layer %3 | %4 | %5 | "
                                     "source line %6 | GEOMETRY ONLY")
                                 .arg(via.name.toHtmlEscaped(), via.net.toHtmlEscaped()).arg(pad.layer)
                                 .arg(pad.shape, via.kind).arg(via.sourceLine));
        }
        if (!scene_->items().isEmpty()) {
            const QRectF rect = scene_->itemsBoundingRect();
            scene_->setSceneRect(rect.adjusted(-0.1, -0.1, 0.1, 0.1));
            view_->fitInView(scene_->sceneRect(), Qt::KeepAspectRatio);
        }
    }
    void appendOutput() {
        QByteArray chunk = worker_.readAllStandardOutput();
        if (chunk.size() > 1'000'000 - output_.size()) {
            output_.clear();
            worker_.kill();
            status_->setText("FAILED / SOURCE_OUTPUT_TOO_LARGE / SI NOT_RUN");
        } else output_.append(chunk);
    }
public:
    explicit Panel(QWidget *parent = nullptr) : QWidget(parent) {
        auto *layout = new QVBoxLayout(this);
        status_ = new QLabel("D_056 LAYERED VIA / NOT IMPORTED / SI NOT_RUN", this);
        layer_ = new QComboBox(this);
        scene_ = new QGraphicsScene(this);
        view_ = new QGraphicsView(scene_, this);
        layout->addWidget(status_);
        layout->addWidget(layer_);
        layout->addWidget(view_);
        layer_->addItem("All source pad layers", 0);
        connect(layer_, qOverload<int>(&QComboBox::currentIndexChanged),
                this, [this](int) { showLayer(); });
        deadline_.setSingleShot(true);
        connect(&deadline_, &QTimer::timeout, this, [this] {
            worker_.kill();
            status_->setText("FAILED / SOURCE_IMPORT_TIMEOUT / SI NOT_RUN");
        });
        worker_.setProcessChannelMode(QProcess::SeparateChannels);
        connect(&worker_, &QProcess::errorOccurred, this,
                [this](QProcess::ProcessError) {
            if (worker_.state() == QProcess::NotRunning) {
                deadline_.stop();
                clear();
                status_->setText("FAILED / SOURCE_IMPORT_PROCESS_ERROR / SI NOT_RUN");
            }
        });
        connect(&worker_, &QProcess::readyReadStandardOutput, this,
                [this] { appendOutput(); });
        connect(&worker_, &QProcess::finished, this,
                [this](int code, QProcess::ExitStatus exitStatus) {
            deadline_.stop();
            appendOutput();
            if (code != 0 || exitStatus != QProcess::NormalExit ||
                originalHash_.isEmpty() || originalHash_ != digest(file_) ||
                !loadWire(output_)) {
                clear();
                status_->setText("FAILED / SOURCE_UNSUPPORTED_OR_CHANGED / SI NOT_RUN");
            }
        });
        resize(800, 600);
    }
    void clear() {
        source_ = {}; output_.clear();
        scene_->clear();
        layer_->clear();
        layer_->addItem("All source pad layers", 0);
        status_->setText("D_056 / NOTHING IMPORTED / SI NOT_RUN");
    }
    bool loadWire(const QByteArray &bytes) {
        Source fresh;
        if (!decode(bytes, fresh)) { clear(); return false; }
        clear();
        source_ = std::move(fresh);
        for (int layer = 1; layer <= source_.layers; ++layer)
            layer_->addItem(QString("Copper Layer %1").arg(layer), layer);
        showLayer();
        status_->setText(QString("D_056 / %1 source vias / %2 layers / "
                                  "GEOMETRY ONLY / GLOBAL CONNECTIVITY NOT VERIFIED / SI NOT_RUN")
                             .arg(source_.vias.size()).arg(source_.layers));
        return true;
    }
    bool importSource(const QString &importer, const QString &file) {
        if (worker_.state() != QProcess::NotRunning ||
            !QFileInfo(importer).isAbsolute() || !QFileInfo(importer).isExecutable() ||
            !QFileInfo(file).isAbsolute()) return false;
        clear();
        file_ = file;
        originalHash_ = digest(file);
        if (originalHash_.isEmpty()) return false;
        status_->setText("RUNNING / SOURCE-LOCAL VIA GEOMETRY / SI NOT_RUN");
        worker_.start(importer, {file});
        deadline_.start(15000);
        return true;
    }
    int displayedItemCount() const { return scene_->items().size(); }
    bool selectLayer(int layer) {
        const int ix = layer_->findData(layer);
        if (ix < 0) return false;
        layer_->setCurrentIndex(ix);
        return true;
    }
    Source source() const { return source_; }
    QGraphicsScene *scene() const { return scene_; }
};
} // namespace d056
