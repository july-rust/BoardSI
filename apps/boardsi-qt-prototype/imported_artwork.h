// BoardSI P2-A partial imported artwork. No physical SI or inferred pads/vias/connectivity.
#pragma once
#include "synthetic_artwork.h"
#include <QComboBox>
#include <QGraphicsLineItem>
#include <QGraphicsEllipseItem>
#include <QGraphicsRectItem>
#include <QGraphicsPathItem>
#include <QPainterPath>
#include <cmath>
#include <QLabel>
#include <QPushButton>
#include <QSet>
#include <QSignalBlocker>
#include <QStackedWidget>
#include <QVector>
#include <QVBoxLayout>
#include <limits>
#include <vector>
#include <functional>
#include <utility>

struct ImportedTrack {
    int layer = 0;
    QString net;
    qint64 x0_nm = 0, y0_nm = 0, x1_nm = 0, y1_nm = 0, width_nm = 0;
};

struct ImportedArc {
    int layer = 0;
    QString net;
    qint64 x0_nm = 0, y0_nm = 0;
    qint64 cx_nm = 0, cy_nm = 0;
    qint64 x1_nm = 0, y1_nm = 0;
    qint64 width_nm = 0;
    bool clockwise = false;
    int sourceLine = 0;
};

struct ImportedOutlineEdge {
    QString name;
    bool cutout = false;
    int sequence = 0;
    qint64 x0_nm = 0, y0_nm = 0, x1_nm = 0, y1_nm = 0;
};

struct ImportedPad {
    QString reference, pin, shape;
    int layer = 0;
    QString side = "T";
    int rotation = -1; // Legacy V2/V3 do not carry rotation; never invent zero.
    qint64 x_nm = 0, y_nm = 0, width_nm = 0, height_nm = 0;
};

struct ImportedVia {
    QString net;
    QString name;
    qint64 x_nm = 0, y_nm = 0, diameter_nm = 0, drill_nm = 0;
    int firstLayer = 0, lastLayer = 0;
};

#ifdef BOARDSI_RHI_ARTWORK
#include "rhi_artwork.h"
#endif

class ImportedArtworkPanel final : public QWidget {
    QGraphicsScene *scene_;
    SyntheticArtworkCanvas *view_;
    QStackedWidget *canvasStack_;
#ifdef BOARDSI_RHI_ARTWORK
    BoardSiRhiView *rhi_ = nullptr;
#endif
    QComboBox *layers_;
    QComboBox *nets_;
    QLabel *summary_;
    QVector<bool> layerVisible_;
    std::vector<std::pair<QGraphicsLineItem *, ImportedTrack>> tracks_;
    std::vector<std::pair<QGraphicsPathItem *, ImportedArc>> arcs_;
    std::vector<std::pair<QGraphicsEllipseItem *, ImportedVia>> vias_;
    std::vector<std::pair<QGraphicsItem *, ImportedPad>> pads_;
    std::vector<std::pair<QGraphicsLineItem *, ImportedOutlineEdge>> outlineEdges_;
    std::function<void(const QString &)> detailsCallback_;
    int skipped_ = 0;
    void refresh() {
#ifdef BOARDSI_RHI_ARTWORK
        if (rhi_) rhi_->filter(layerVisible_, nets_->currentText());
#endif
        // A GPU backend change must never fabricate a successful PCB import.
        if (tracks_.empty() && arcs_.empty() && vias_.empty() && pads_.empty() &&
            outlineEdges_.empty()) {
            summary_->setText("No imported PCB. Synthetic demo is a separate tab.\n"
                              "Artwork renderer: " + rendererStatus());
            if (detailsCallback_) detailsCallback_(summary_->text());
            return;
        }
        const QString net = nets_->currentText();
        int visible = 0;
        int highlighted = 0;
        QSet<int> layersWithTraces;
        QSet<QString> netsWithTraces;
        QSet<int> layersWithArcs;
        QSet<QString> netsWithArcs;
        QSet<QString> netsWithVias;
        for (const auto &entry : tracks_) {
            layersWithTraces.insert(entry.second.layer);
            netsWithTraces.insert(entry.second.net);
            const ImportedTrack &t = entry.second;
            const bool shown = t.layer > 0 && t.layer < layerVisible_.size() && layerVisible_[t.layer];
            entry.first->setVisible(shown);
            const bool active = net == "All nets" || net == t.net;
            entry.first->setOpacity(active ? 1.0 : 0.12);
            if (shown) { ++visible; if (net != "All nets" && active) ++highlighted; }
        }
        int visibleArcs = 0;
        for (const auto &entry : arcs_) {
            const auto &arc = entry.second;
            layersWithArcs.insert(arc.layer);
            netsWithArcs.insert(arc.net);
            const bool shown = arc.layer > 0 && arc.layer < layerVisible_.size() &&
                               layerVisible_[arc.layer];
            entry.first->setVisible(shown);
            entry.first->setOpacity(net == "All nets" || net == arc.net ? 1.0 : 0.12);
            if (shown) ++visibleArcs;
        }
        int visibleVias = 0;
        int highlightedVias = 0;
        for (const auto &entry : vias_) {
            const ImportedVia &via = entry.second;
            netsWithVias.insert(via.net);
            bool shown = false;
            for (int n = via.firstLayer; n <= via.lastLayer; ++n) {
                if (n > 0 && n < layerVisible_.size() && layerVisible_[n]) {
                    shown = true;
                    break;
                }
            }
            entry.first->setVisible(shown);
            const bool active = net == "All nets" || net == via.net;
            entry.first->setOpacity(active ? 1.0 : 0.12);
            if (shown) {
                ++visibleVias;
                if (net != "All nets" && active) ++highlightedVias;
            }
        }
        int visiblePads = 0;
        int bottomPads = 0;
        for (const auto &entry : pads_) {
            if (entry.second.side == "B") ++bottomPads;
            const ImportedPad &pad = entry.second;
            const bool shown = pad.layer > 0 && pad.layer < layerVisible_.size() &&
                               layerVisible_[pad.layer];
            entry.first->setVisible(shown);
            // Pad net ownership has not been verified. Never highlight pads
            // as electrically connected to a selected route net.
            entry.first->setOpacity(net == "All nets" ? 1.0 : 0.12);
            if (shown) ++visiblePads;
        }
        // Declared layers are NOT necessarily copper layers with accepted traces.
        // Counts below describe only rendered straight-route data, not inferred geometry.
        const int layerId = layers_->currentData().toInt();
        const bool selectedVisible = layerId > 0 && layerId < layerVisible_.size() &&
                                     layerVisible_[layerId];
        const QString selectedLayer = layers_->currentText().isEmpty()
                                          ? QString("None") : layers_->currentText();
        QString line =
            QString("IMPORTED PARTIAL / PADS V9.4 MILS / SI NOT_RUN\n"
                    "Declared layers: %1  Layers with straight traces: %2\n"
                    "Nets with straight traces: %3\n")
                .arg(layerVisible_.size() - 1).arg(layersWithTraces.size())
                .arg(netsWithTraces.size()) +
            QString("Nets with explicit vias: %1\n").arg(netsWithVias.size()) +
            QString("Selected layer: %1 (ID %2; %3)\nSelected net: %4\n")
                .arg(selectedLayer).arg(layerId)
                .arg(selectedVisible ? "visible" : "hidden/not loaded").arg(net) +
            QString("Segments: %1  Visible: %2  Highlighted: %3\n"
                    "Explicit round through vias: %4  Visible: %5  Highlighted: %6\n"
                    "Component SMD pads: %8  Visible: %9  Net: NOT_VERIFIED\n"
                    "Unsupported route/part records omitted: %7\n"
                    "Other pads, unsupported vias, pours, electrical connectivity, stackup: NOT IMPORTED")
                .arg(static_cast<qlonglong>(tracks_.size())).arg(visible)
                .arg(highlighted).arg(static_cast<qlonglong>(vias_.size()))
                .arg(visibleVias).arg(highlightedVias).arg(skipped_)
                .arg(static_cast<qlonglong>(pads_.size())).arg(visiblePads);
        line += QString("\nAnalytical route arcs: %1  Visible: %2"
                        "\nLayers with analytical arcs: %3  Nets with analytical arcs: %4")
                    .arg(static_cast<qlonglong>(arcs_.size())).arg(visibleArcs)
                    .arg(layersWithArcs.size()).arg(netsWithArcs.size());
        line += QString("\nBoard outline/cutout straight edges: %1"
                        " (artwork only; NOT electrical geometry)")
                    .arg(static_cast<qlonglong>(outlineEdges_.size()));
        line += QString("\nTop SMD: %1  Bottom SMD: %2")
                    .arg(static_cast<qlonglong>(pads_.size()) - bottomPads).arg(bottomPads);
        const auto selected = scene_->selectedItems();
        if (!selected.isEmpty() && !selected.first()->data(1).toString().isEmpty())
            line += "\nSelected object: " + selected.first()->data(1).toString();
        line += "\nArtwork renderer: " + rendererStatus();
        summary_->setText(line);
        if (detailsCallback_) detailsCallback_(line);
    }
public:
    explicit ImportedArtworkPanel(QWidget *parent = nullptr)
        : QWidget(parent), scene_(new QGraphicsScene(this)),
          view_(new SyntheticArtworkCanvas(scene_, this)),
          canvasStack_(new QStackedWidget(this)),
          layers_(new QComboBox(this)), nets_(new QComboBox(this)),
          summary_(new QLabel("No imported PCB. Synthetic demo is a separate tab.", this)) {
        setObjectName("importedArtworkP2A");
        layers_->setObjectName("importedLayerSelect");
        auto *layout = new QVBoxLayout(this);
        auto *bar = new QHBoxLayout();
        bar->addWidget(new QLabel("Layer:", this));
        bar->addWidget(layers_);
        auto *toggle = new QPushButton("Toggle layer", this);
        toggle->setObjectName("importedLayerToggle");
        bar->addWidget(toggle);
        auto *all = new QPushButton("Show all", this);
        bar->addWidget(all);
        bar->addWidget(new QLabel("Net:", this));
        nets_->setObjectName("importedNetSelect");
        bar->addWidget(nets_);
        auto *fit = new QPushButton("Fit", this);
        fit->setObjectName("importedFit");
        bar->addWidget(fit);
        auto *zoomIn = new QPushButton("+", this);
        zoomIn->setObjectName("importedZoomIn");
        auto *zoomOut = new QPushButton("-", this);
        zoomOut->setObjectName("importedZoomOut");
        bar->addWidget(zoomIn);
        bar->addWidget(zoomOut);
        layout->addLayout(bar);
        canvasStack_->addWidget(view_);
#ifdef BOARDSI_RHI_ARTWORK
        // A native Qt Quick window avoids QQuickWidget's extra offscreen render
        // pass. A missing Qt Quick module or a headless platform keeps the old
        // CPU/OpenGL canvas and all established regression tests unchanged.
        const QString policy = qEnvironmentVariable("BOARDSI_ARTWORK_RENDERER")
                                   .trimmed().toLower();
        // Qt may use software WARP behind a D3D11 graphicsApi(). Reject a
        // request for that backend before creating the Qt Quick scene.
        const bool softwareRhi = BoardSiRhiView::softwareRendererRequested();
        if (softwareRhi) view_->forceCpuFallbackFromRhi();
        if (!softwareRhi &&
            QGuiApplication::platformName() != "offscreen" &&
            QGuiApplication::platformName() != "minimal" &&
            (policy.isEmpty() || policy == "auto" || policy == "gpu")) {
            rhi_ = new BoardSiRhiView(canvasStack_);
            canvasStack_->addWidget(rhi_);
            canvasStack_->setCurrentWidget(rhi_);
            rhi_->onChanged([this] {
                if (rhi_->failed()) {
                    view_->forceCpuFallbackFromRhi();
                    canvasStack_->setCurrentWidget(view_);
                }
                refresh();
            });
            rhi_->onPicked([this](int kind,int index) {
                if (kind == 1) selectVia(index);
                else if (kind == 2) selectPad(index);
                else if (kind == 3) selectTrack(index);
                else if (kind == 4) selectArc(index);
            });
        }
#endif
        layout->addWidget(canvasStack_, 1);
        summary_->setWordWrap(true);
        layout->addWidget(summary_);
        connect(toggle, &QPushButton::clicked, this, [this] {
            const int n = layers_->currentData().toInt();
            if (n > 0 && n < layerVisible_.size()) { layerVisible_[n] = !layerVisible_[n]; refresh(); }
        });
        connect(all, &QPushButton::clicked, this, [this] {
            for (int i = 1; i < layerVisible_.size(); ++i) layerVisible_[i] = true;
            refresh();
        });
        connect(layers_, &QComboBox::currentIndexChanged, this, [this](int) { refresh(); });
        connect(nets_, &QComboBox::currentTextChanged, this, [this] { refresh(); });
        connect(fit, &QPushButton::clicked, this, [this] { fitArtwork(); });
        connect(zoomIn, &QPushButton::clicked, this, [this] {
#ifdef BOARDSI_RHI_ARTWORK
            if (rhi_ && canvasStack_->currentWidget() == rhi_) {
                rhi_->zoom(1.2); return;
            }
#endif
            if (view_->transform().m11() * 1.2 <= 120) view_->scale(1.2, 1.2);
        });
        connect(zoomOut, &QPushButton::clicked, this, [this] {
#ifdef BOARDSI_RHI_ARTWORK
            if (rhi_ && canvasStack_->currentWidget() == rhi_) {
                rhi_->zoom(1.0/1.2); return;
            }
#endif
            if (view_->transform().m11() / 1.2 >= 0.03) view_->scale(1.0/1.2, 1.0/1.2);
        });
        connect(scene_, &QGraphicsScene::selectionChanged, this, [this] {
            for (auto *item : scene_->selectedItems()) {
                const auto name = item->data(0).toString();
                const int pos = nets_->findText(name);
                if (pos >= 0) { nets_->setCurrentIndex(pos); break; }
            }
            refresh();
        });
        view_->setRendererChangedCallback([this] { refresh(); });
        setMinimumHeight(230);
    }
    ~ImportedArtworkPanel() override {
        // Scene item destruction may emit selectionChanged. Disconnect first:
        // other Qt child widgets can already be in their destruction sequence.
        QObject::disconnect(scene_, nullptr, this, nullptr);
        scene_->clearSelection();
    }
    void setDetailsCallback(std::function<void(const QString &)> callback) {
        detailsCallback_ = std::move(callback);
    }
    // Starting a new import invalidates the last artwork. In particular, a failed
    // attempt must not leave old copper visible under a "NOTHING IMPORTED" status.
    void clearArtwork() {
        // Clearing selected graphics items can synchronously emit
        // selectionChanged. Prevent callbacks from iterating freed pointers.
        const QSignalBlocker muteSelection(scene_);
        tracks_.clear();
        arcs_.clear();
        vias_.clear();
        pads_.clear();
        outlineEdges_.clear();
        scene_->clear();
        layerVisible_.clear();
        layers_->clear();
        nets_->clear();
        nets_->addItem("All nets");
        skipped_ = 0;
#ifdef BOARDSI_RHI_ARTWORK
        if (rhi_) rhi_->load(0, {}, {}, {});
#endif
        scene_->setSceneRect(QRectF());
        summary_->setText("No imported PCB. Synthetic demo is a separate tab.\n"
                          "Artwork renderer: " + rendererStatus());
        if (detailsCallback_) detailsCallback_(summary_->text());
    }
    bool load(int layerCount, int skipped, const std::vector<ImportedTrack> &input,
              const std::vector<ImportedVia> &viaInput = {},
              const std::vector<ImportedPad> &padInput = {},
              const std::vector<ImportedArc> &arcInput = {},
              const std::vector<ImportedOutlineEdge> &outlineInput = {}) {
        if (layerCount < 1 || layerCount > 64 ||
            (input.empty() && viaInput.empty() && padInput.empty() &&
             arcInput.empty() && outlineInput.empty()) ||
            input.size() + arcInput.size() > 10000 || viaInput.size() > 10000 ||
            padInput.size() > 5000 || outlineInput.size() > 16384) return false;
        // Keep the old artwork untouched if a candidate fails validation.
        for (const auto &t : input) {
            if (t.layer <= 0 || t.layer > layerCount || t.width_nm <= 0 ||
                t.net.isEmpty() || t.net.size() > 64) return false;
            for (auto coordinate : {t.x0_nm, t.y0_nm, t.x1_nm, t.y1_nm}) {
                if (coordinate < -100000000000LL || coordinate > 100000000000LL)
                    return false;
            }
        }
        for (const auto &a : arcInput) {
            if (a.layer < 1 || a.layer > layerCount || a.width_nm <= 0 ||
                a.net.isEmpty() || a.net.size() > 64 || a.sourceLine <= 0) return false;
            for (auto v : {a.x0_nm,a.y0_nm,a.cx_nm,a.cy_nm,a.x1_nm,a.y1_nm,a.width_nm}) {
                if (v % 254 != 0 || v < -1422400000LL || v > 1422400000LL)
                    return false;
            }
            const auto r2 = [&](qint64 x,qint64 y) {
                const qint64 dx=(x-a.cx_nm)/254,dy=(y-a.cy_nm)/254;
                return dx*dx+dy*dy;
            };
            if (a.x0_nm == a.x1_nm && a.y0_nm == a.y1_nm) return false;
            if (r2(a.x0_nm,a.y0_nm) == 0 ||
                r2(a.x0_nm,a.y0_nm) != r2(a.x1_nm,a.y1_nm)) return false;
        }
        for (const auto &via : viaInput) {
            if (via.net.isEmpty() || via.net.size() > 64 || via.name.isEmpty() ||
                via.name.size() > 64 || via.firstLayer < 1 ||
                via.firstLayer >= via.lastLayer || via.lastLayer > layerCount ||
                via.drill_nm <= 0 || via.diameter_nm <= via.drill_nm ||
                via.diameter_nm > 100000000000LL ||
                via.x_nm < -100000000000LL || via.x_nm > 100000000000LL ||
                via.y_nm < -100000000000LL || via.y_nm > 100000000000LL)
                return false;
        }
        for (const auto &pad : padInput) {
            if ((pad.side != "T" && pad.side != "B") ||
                (pad.side == "T" && pad.layer != 1) ||
                (pad.side == "B" && (layerCount < 2 || pad.layer != layerCount)) ||
                (pad.rotation != -1 && pad.rotation != 0 && pad.rotation != 90 &&
                 pad.rotation != 180 && pad.rotation != 270) ||
                (pad.side == "B" && pad.rotation == -1) ||
                pad.reference.isEmpty() || pad.pin.isEmpty() ||
                pad.width_nm <= 0 || pad.height_nm <= 0 ||
                (pad.shape != "R" && pad.shape != "S" && pad.shape != "RF") ||
                (pad.shape != "RF" && pad.height_nm != pad.width_nm) ||
                pad.x_nm < -100000000000LL || pad.x_nm > 100000000000LL ||
                pad.y_nm < -100000000000LL || pad.y_nm > 100000000000LL)
                return false;
        }
        // Strict transport continuity, distinct rings and exact source-grid bounds.
        QString ringKey;
        qint64 firstX=0,firstY=0,lastX=0,lastY=0;
        int expectedSequence=0,outerRings=0;
        auto closeRing=[&]() {
            return expectedSequence==0 || (expectedSequence>=3 &&
                firstX==lastX && firstY==lastY);
        };
        for (const auto &e : outlineInput) {
            if (e.name.isEmpty() || e.name.size()>16 || e.sequence<0) return false;
            for (auto v : {e.x0_nm,e.y0_nm,e.x1_nm,e.y1_nm}) {
                if (v%254!=0 || v < -1422400000LL || v > 1422400000LL) return false;
            }
            const QString key=(e.cutout ? "CUT|" : "OUTER|")+e.name;
            if (ringKey!=key) {
                if (!closeRing() || e.sequence!=0) return false;
                ringKey=key;expectedSequence=0;
                firstX=e.x0_nm;firstY=e.y0_nm;
                if (!e.cutout) ++outerRings;
            } else if (e.x0_nm!=lastX || e.y0_nm!=lastY) return false;
            if (e.sequence!=expectedSequence ||
                (e.x0_nm==e.x1_nm && e.y0_nm==e.y1_nm)) return false;
            ++expectedSequence;lastX=e.x1_nm;lastY=e.y1_nm;
        }
        if (!closeRing() || (!outlineInput.empty() && outerRings!=1)) return false;
        {
            const QSignalBlocker muteSelection(scene_);
            tracks_.clear();
            arcs_.clear();
            vias_.clear();
            pads_.clear();
            outlineEdges_.clear();
            scene_->clear();
        }
        layerVisible_ = QVector<bool>(layerCount + 1, true);
        layers_->clear();
        nets_->clear();
        nets_->addItem("All nets");
        for (int n = 1; n <= layerCount; ++n) {
            layers_->addItem(n == 1 ? "Top" : (n == layerCount ? "Bottom" : QString("Inner %1").arg(n)), n);
        }
        for (const auto &t : input) {
            if (nets_->findText(t.net) < 0) nets_->addItem(t.net);
            const QColor color = t.layer == 1 ? QColor("#f29d55") :
                                 t.layer == layerCount ? QColor("#55b4e9") : QColor("#a9d48e");
            // PADS Y increases upwards; Qt scene Y increases downwards.
            const QLineF line(t.x0_nm / 1e6, -t.y0_nm / 1e6,
                              t.x1_nm / 1e6, -t.y1_nm / 1e6);
            auto *item = scene_->addLine(line, QPen(color, t.width_nm / 1e6,
                                                   Qt::SolidLine, Qt::RoundCap, Qt::RoundJoin));
            item->setData(0, t.net);
            item->setData(1, QString("Straight trace / layer %1 / width %2 mm")
                                 .arg(t.layer).arg(t.width_nm / 1e6, 0, 'f', 6));
            item->setFlag(QGraphicsItem::ItemIsSelectable);
            tracks_.push_back({item, t});
        }
        for (const auto &a : arcInput) {
            if (nets_->findText(a.net) < 0) nets_->addItem(a.net);
            constexpr qreal pi = 3.14159265358979323846;
            const qreal cx = a.cx_nm/1e6, cy = -a.cy_nm/1e6;
            const qreal dx = (a.x0_nm-a.cx_nm)/1e6;
            const qreal dy = (a.y0_nm-a.cy_nm)/1e6;
            const qreal r = std::hypot(dx,dy);
            const qreal first = std::atan2(dy,dx)*180.0/pi;
            const qreal end = std::atan2(
                static_cast<qreal>(a.y1_nm-a.cy_nm),
                static_cast<qreal>(a.x1_nm-a.cx_nm))*180.0/pi;
            qreal sweep = end - first;
            if (a.clockwise) { while (sweep >= 0) sweep -= 360.0; }
            else { while (sweep <= 0) sweep += 360.0; }
            QPainterPath path;
            path.moveTo(a.x0_nm/1e6,-a.y0_nm/1e6);
            path.arcTo(QRectF(cx-r,cy-r,2*r,2*r),first,sweep);
            const QColor color(a.layer == 1 ? "#f29d55" :
                               a.layer == layerCount ? "#55b4e9" : "#a9d48e");
            auto *item=scene_->addPath(path,QPen(color,a.width_nm/1e6,
                                   Qt::SolidLine,Qt::RoundCap,Qt::RoundJoin));
            item->setData(0,a.net);
            item->setData(1,QString("Arc trace / layer %1 / width %2 mm / centre (%3, %4) mm"
                                   " / %5 / source line %6 / SI NOT_RUN")
                               .arg(a.layer).arg(a.width_nm/1e6,0,'f',6)
                               .arg(a.cx_nm/1e6,0,'f',6).arg(a.cy_nm/1e6,0,'f',6)
                               .arg(a.clockwise ? "CW" : "CCW").arg(a.sourceLine));
            item->setFlag(QGraphicsItem::ItemIsSelectable);
            arcs_.push_back({item,a});
        }
        for (const auto &via : viaInput) {
            if (nets_->findText(via.net) < 0) nets_->addItem(via.net);
            const qreal x = via.x_nm / 1e6;
            const qreal y = -via.y_nm / 1e6;
            const qreal radius = via.diameter_nm / 2e6;
            const qreal drillRadius = via.drill_nm / 2e6;
            auto *outer = scene_->addEllipse(
                QRectF(x-radius, y-radius, 2*radius, 2*radius),
                QPen(QColor("#e7d488"), 0), QBrush(QColor("#b98b45")));
            auto *hole = new QGraphicsEllipseItem(
                QRectF(x-drillRadius, y-drillRadius, 2*drillRadius, 2*drillRadius), outer);
            hole->setPen(QPen(QColor("#554332"), 0));
            hole->setBrush(QBrush(QColor("#232c34")));
            outer->setData(0, via.net);
            outer->setData(1, QString("Via %1 / outer %2 mm / drill %3 mm / span L%4-L%5"
                                      " / electrical contact NOT_VERIFIED")
                                   .arg(via.name)
                                   .arg(via.diameter_nm / 1e6, 0, 'f', 6)
                                   .arg(via.drill_nm / 1e6, 0, 'f', 6)
                                   .arg(via.firstLayer).arg(via.lastLayer));
            outer->setFlag(QGraphicsItem::ItemIsSelectable);
            vias_.push_back({outer, via});
        }
        for (const auto &pad : padInput) {
            const qreal width = pad.width_nm / 1e6;
            const qreal height = pad.height_nm / 1e6;
            const qreal x = pad.x_nm / 1e6;
            const qreal y = -pad.y_nm / 1e6;
            const QRectF box(x-width/2, y-height/2, width, height);
            // Same fixed top-view coordinates; bottom pads were mirrored by the importer.
            const QColor color(pad.side == "B" ? "#55b4e9" : "#d0a55a");
            QGraphicsItem *item = pad.shape == "R"
                ? static_cast<QGraphicsItem *>(scene_->addEllipse(
                      box, QPen(color.darker(140), 0), QBrush(color)))
                : static_cast<QGraphicsItem *>(scene_->addRect(
                      box, QPen(color.darker(140), 0), QBrush(color)));
            item->setData(1, QString("Pad %1.%2 / layer %3 / %4 / %5 mm x %6 mm"
                                     " / centre (%7, %8) mm / net NOT_VERIFIED"
                                     " / side %9 / rotation %10 / source PADS PART")
                                  .arg(pad.reference).arg(pad.pin).arg(pad.layer)
                                  .arg(pad.shape == "R" ? "round" : (pad.shape == "S" ? "square" : "rectangular RF"))
                                  .arg(width, 0, 'f', 6).arg(height, 0, 'f', 6)
                                  .arg(x, 0, 'f', 6).arg(-y, 0, 'f', 6)
                                  .arg(pad.side == "B" ? "Bottom" : "Top")
                                  .arg(pad.rotation < 0 ? QString("NOT_VERIFIED") :
                                       QString("%1 deg").arg(pad.rotation)));
            item->setFlag(QGraphicsItem::ItemIsSelectable);
            pads_.push_back({item, pad});
        }
        for (const auto &e : outlineInput) {
            const QLineF line(e.x0_nm/1e6,-e.y0_nm/1e6,
                              e.x1_nm/1e6,-e.y1_nm/1e6);
            const QColor color(e.cutout ? "#e99f87" : "#e0e8ee");
            auto *item=scene_->addLine(line,QPen(color,0));
            item->setData(1,e.cutout ? "Board Cutout (not copper)" :
                                         "Board Outline (not copper)");
            outlineEdges_.push_back({item,e});
        }
        skipped_ = skipped;
#ifdef BOARDSI_RHI_ARTWORK
        if (rhi_) rhi_->load(layerCount,input,viaInput,padInput,arcInput,outlineInput);
#endif
        scene_->setSceneRect(scene_->itemsBoundingRect().adjusted(-1, -1, 1, 1));
        refresh();
        fitArtwork();
        return true;
    }
    void fitArtwork() {
        if (!tracks_.empty() || !arcs_.empty() || !vias_.empty() ||
            !pads_.empty() || !outlineEdges_.empty()) {
            // Qt's Fit is cumulative with the existing view transform. Reset
            // first so a Fit after Zoom is repeatable and does not drift.
            view_->resetTransform();
            view_->fitInView(scene_->sceneRect(), Qt::KeepAspectRatio);
#ifdef BOARDSI_RHI_ARTWORK
            if (rhi_) rhi_->fit();
#endif
        }
    }
    int segmentCount() const { return static_cast<int>(tracks_.size()); }
    int arcCount() const { return static_cast<int>(arcs_.size()); }
    int outlineCount() const { return static_cast<int>(outlineEdges_.size()); }
    int viaCount() const { return static_cast<int>(vias_.size()); }
    int padCount() const { return static_cast<int>(pads_.size()); }
#ifdef BOARDSI_RHI_ARTWORK
    BoardSiRhiView *rhiViewForTest() const { return rhi_; }
    bool rhiCanvasActiveForTest() const {
        return rhi_ && canvasStack_->currentWidget() == rhi_;
    }
#endif
    bool selectArc(int index) {
        if (index < 0 || index >= static_cast<int>(arcs_.size())) return false;
        scene_->clearSelection();
        arcs_[static_cast<size_t>(index)].first->setSelected(true);
        return true;
    }
    bool selectTrack(int index) {
        if (index < 0 || index >= static_cast<int>(tracks_.size())) return false;
        scene_->clearSelection();
        tracks_[static_cast<size_t>(index)].first->setSelected(true);
        return true;
    }
    bool selectPad(int index) {
        if (index < 0 || index >= static_cast<int>(pads_.size())) return false;
        scene_->clearSelection();
        pads_[static_cast<size_t>(index)].first->setSelected(true);
        return true;
    }
    bool selectVia(int index) {
        if (index < 0 || index >= static_cast<int>(vias_.size())) return false;
        scene_->clearSelection();
        vias_[static_cast<size_t>(index)].first->setSelected(true);
        return true;
    }
    int visibleSegments() const {
        int count = 0;
        for (const auto &t : tracks_) if (t.first->isVisible()) ++count;
        for (const auto &a : arcs_) if (a.first->isVisible()) ++count;
        return count;
    }
    bool selectNet(const QString &net) {
        const int index = nets_->findText(net);
        if (index < 0) return false;
        nets_->setCurrentIndex(index);
        return true;
    }
    qreal scale() const { return view_->transform().m11(); }
    QString rendererStatus() const {
#ifdef BOARDSI_RHI_ARTWORK
        if (rhi_ && canvasStack_->currentWidget() == rhi_) return rhi_->status();
#endif
        return view_->rendererStatus();
    }
    QString rendererPolicy() const { return view_->rendererPolicy(); }
    QString summaryText() const { return summary_->text(); }
};
