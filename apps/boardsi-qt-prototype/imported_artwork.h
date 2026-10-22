// BoardSI P2-A partial imported artwork. No physical SI or inferred pads/vias/connectivity.
#pragma once
#include "synthetic_artwork.h"
#include <QComboBox>
#include <QGraphicsLineItem>
#include <QGraphicsEllipseItem>
#include <QLabel>
#include <QPushButton>
#include <QSet>
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

struct ImportedVia {
    QString net;
    QString name;
    qint64 x_nm = 0, y_nm = 0, diameter_nm = 0, drill_nm = 0;
    int firstLayer = 0, lastLayer = 0;
};

class ImportedArtworkPanel final : public QWidget {
    QGraphicsScene *scene_;
    SyntheticArtworkCanvas *view_;
    QComboBox *layers_;
    QComboBox *nets_;
    QLabel *summary_;
    QVector<bool> layerVisible_;
    std::vector<std::pair<QGraphicsLineItem *, ImportedTrack>> tracks_;
    std::vector<std::pair<QGraphicsEllipseItem *, ImportedVia>> vias_;
    std::function<void(const QString &)> detailsCallback_;
    int skipped_ = 0;
    void refresh() {
        const QString net = nets_->currentText();
        int visible = 0;
        int highlighted = 0;
        QSet<int> layersWithTraces;
        QSet<QString> netsWithTraces;
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
            QString("Nets with explicit vias: %1\\n").arg(netsWithVias.size()) +
            QString("Selected layer: %1 (ID %2; %3)\nSelected net: %4\n")
                .arg(selectedLayer).arg(layerId)
                .arg(selectedVisible ? "visible" : "hidden/not loaded").arg(net) +
            QString("Segments: %1  Visible: %2  Highlighted: %3\n"
                    "Explicit round through vias: %4  Visible: %5  Highlighted: %6\n"
                    "Unsupported route records omitted: %7\n"
                    "Other pads, unsupported vias, pours, electrical connectivity, stackup: NOT IMPORTED")
                .arg(static_cast<qlonglong>(tracks_.size())).arg(visible)
                .arg(highlighted).arg(static_cast<qlonglong>(vias_.size()))
                .arg(visibleVias).arg(highlightedVias).arg(skipped_);
        const auto selected = scene_->selectedItems();
        if (!selected.isEmpty() && !selected.first()->data(1).toString().isEmpty())
            line += "\nSelected object: " + selected.first()->data(1).toString();
        summary_->setText(line);
        if (detailsCallback_) detailsCallback_(line);
    }
public:
    explicit ImportedArtworkPanel(QWidget *parent = nullptr)
        : QWidget(parent), scene_(new QGraphicsScene(this)),
          view_(new SyntheticArtworkCanvas(scene_, this)),
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
        layout->addWidget(view_, 1);
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
            if (view_->transform().m11() * 1.2 <= 120) view_->scale(1.2, 1.2);
        });
        connect(zoomOut, &QPushButton::clicked, this, [this] {
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
        scene_->clear();
        tracks_.clear();
        vias_.clear();
        layerVisible_.clear();
        layers_->clear();
        nets_->clear();
        nets_->addItem("All nets");
        skipped_ = 0;
        scene_->setSceneRect(QRectF());
        summary_->setText("No imported PCB. Synthetic demo is a separate tab.");
        if (detailsCallback_) detailsCallback_(summary_->text());
    }
    bool load(int layerCount, int skipped, const std::vector<ImportedTrack> &input,
              const std::vector<ImportedVia> &viaInput = {}) {
        if (layerCount < 1 || layerCount > 64 || (input.empty() && viaInput.empty()) ||
            input.size() > 10000 || viaInput.size() > 10000) return false;
        // Keep the old artwork untouched if a candidate fails validation.
        for (const auto &t : input) {
            if (t.layer <= 0 || t.layer > layerCount || t.width_nm <= 0 ||
                t.net.isEmpty() || t.net.size() > 64) return false;
            for (auto coordinate : {t.x0_nm, t.y0_nm, t.x1_nm, t.y1_nm}) {
                if (coordinate < -100000000000LL || coordinate > 100000000000LL)
                    return false;
            }
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
        scene_->clear();
        tracks_.clear();
        vias_.clear();
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
        skipped_ = skipped;
        scene_->setSceneRect(scene_->itemsBoundingRect().adjusted(-1, -1, 1, 1));
        refresh();
        fitArtwork();
        return true;
    }
    void fitArtwork() {
        if (!tracks_.empty() || !vias_.empty()) {
            // Qt's Fit is cumulative with the existing view transform. Reset
            // first so a Fit after Zoom is repeatable and does not drift.
            view_->resetTransform();
            view_->fitInView(scene_->sceneRect(), Qt::KeepAspectRatio);
        }
    }
    int segmentCount() const { return static_cast<int>(tracks_.size()); }
    int viaCount() const { return static_cast<int>(vias_.size()); }
    bool selectVia(int index) {
        if (index < 0 || index >= static_cast<int>(vias_.size())) return false;
        scene_->clearSelection();
        vias_[static_cast<size_t>(index)].first->setSelected(true);
        return true;
    }
    int visibleSegments() const {
        int count = 0;
        for (const auto &t : tracks_) if (t.first->isVisible()) ++count;
        return count;
    }
    bool selectNet(const QString &net) {
        const int index = nets_->findText(net);
        if (index < 0) return false;
        nets_->setCurrentIndex(index);
        return true;
    }
    qreal scale() const { return view_->transform().m11(); }
};
