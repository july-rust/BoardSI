// BoardSI Preview P1 artwork fixture. Entire geometry below is synthetic.
// NOT imported PCB artwork, real Canonical Board Model, or physical SI evidence.
#pragma once

#include <QBrush>
#include <QCheckBox>
#include <QColor>
#include <QComboBox>
#include <QGraphicsItem>
#include <QGraphicsScene>
#include <QGraphicsView>
#include <QHBoxLayout>
#include <QLineF>
#include <QLabel>
#include <QPainter>
#include <QPen>
#include <QPointF>
#include <QRectF>
#include <QPushButton>
#include <QTimer>
#include <QVBoxLayout>
#include <QWheelEvent>
#include <QWidget>
#include <QString>

#include <functional>
#include <vector>
#include <utility>

class SyntheticArtworkCanvas final : public QGraphicsView {
public:
    explicit SyntheticArtworkCanvas(QGraphicsScene *scene, QWidget *parent = nullptr)
        : QGraphicsView(scene, parent) {
        setRenderHint(QPainter::Antialiasing);
        setDragMode(QGraphicsView::ScrollHandDrag);
        setTransformationAnchor(QGraphicsView::AnchorUnderMouse);
        setBackgroundBrush(QColor("#111923"));
        setMinimumHeight(220);
    }

protected:
    void wheelEvent(QWheelEvent *event) override {
        const int delta = event->angleDelta().y();
        if (delta == 0) {
            QGraphicsView::wheelEvent(event);
            return;
        }
        const qreal current = transform().m11();
        const qreal factor = delta > 0 ? 1.2 : 1.0 / 1.2;
        if (current * factor >= 0.03 && current * factor <= 120.0)
            scale(factor, factor);
        event->accept();
    }
};

class SyntheticArtworkPanel final : public QWidget {
    struct Primitive {
        QGraphicsItem *item;
        QString layer;
        QString net;
    };
    QGraphicsScene *scene_;
    SyntheticArtworkCanvas *canvas_;
    QComboBox *nets_;
    QCheckBox *top_;
    QCheckBox *bottom_;
    QLabel *summary_;
    std::vector<Primitive> primitives_;
    std::function<void(const QString &)> detailsCallback_;

    void append(QGraphicsItem *item, const QString &layer, const QString &net) {
        item->setData(0, net);
        item->setData(1, layer);
        item->setFlag(QGraphicsItem::ItemIsSelectable);
        primitives_.push_back({item, layer, net});
    }

    void trace(const QString &layer, const QString &net, const QColor &color,
               const QPointF &a, const QPointF &b) {
        auto *item = scene_->addLine(QLineF(a, b),
                                     QPen(color, 0.8, Qt::SolidLine, Qt::RoundCap));
        append(item, layer, net);
    }

    void pad(const QString &layer, const QString &net, const QColor &color,
             qreal x, qreal y) {
        auto *item = scene_->addRect(QRectF(x - 1.8, y - 1.1, 3.6, 2.2),
                                     QPen(color.lighter(145), 0.25), color);
        append(item, layer, net);
    }

    void via(const QString &net, qreal x, qreal y) {
        auto *item = scene_->addEllipse(QRectF(x - 1.1, y - 1.1, 2.2, 2.2),
                                        QPen(QColor("#e7d488"), 0.4),
                                        QColor("#3b6072"));
        append(item, "Through", net);
    }

    void createFixture() {
        // Scene coordinates are illustrative millimetres, not source CAD coordinates.
        scene_->setSceneRect(0.0, 0.0, 112.0, 76.0);
        scene_->addRect(QRectF(5.0, 8.0, 102.0, 60.0),
                        QPen(QColor("#8199a1"), 0.5),
                        QBrush(QColor("#203b36")));
        auto *banner = scene_->addText("SYNTHETIC PCB / NOT IMPORTED / NOT PCB SI");
        banner->setDefaultTextColor(QColor("#f5dea3"));
        banner->setScale(0.45);
        banner->setPos(7.0, 0.5);

        const QColor topColor("#f1a15b");
        const QColor bottomColor("#6cbbec");
        // Three fictional signal nets, two mock copper layers and through vias.
        trace("Top", "DATA_P0", topColor, {12, 20}, {48, 20});
        trace("Top", "DATA_P0", topColor, {48, 20}, {76, 32});
        pad("Top", "DATA_P0", topColor, 12, 20);
        pad("Top", "DATA_P0", topColor, 76, 32);
        via("DATA_P0", 48, 20);

        trace("Bottom", "DATA_P0", bottomColor, {48, 20}, {92, 20});
        pad("Bottom", "DATA_P0", bottomColor, 92, 20);

        trace("Top", "CLK_N0", topColor, {12, 42}, {50, 42});
        trace("Top", "CLK_N0", topColor, {50, 42}, {84, 48});
        pad("Top", "CLK_N0", topColor, 12, 42);
        pad("Top", "CLK_N0", topColor, 84, 48);

        trace("Bottom", "CTRL0", bottomColor, {24, 56}, {88, 56});
        pad("Bottom", "CTRL0", bottomColor, 24, 56);
        pad("Bottom", "CTRL0", bottomColor, 88, 56);
        via("CTRL0", 60, 56);
    }

    void refreshArtwork() {
        const QString net = selectedNet();
        int visible = 0;
        int highlighted = 0;
        for (const Primitive &primitive : primitives_) {
            const bool show = primitive.layer == "Top" ? top_->isChecked() :
                              primitive.layer == "Bottom" ? bottom_->isChecked() :
                              (top_->isChecked() || bottom_->isChecked());
            primitive.item->setVisible(show);
            const bool emphasize = net == "All nets" || primitive.net == net;
            primitive.item->setOpacity(emphasize ? 1.0 : 0.14);
            if (show) {
                ++visible;
                if (net != "All nets" && primitive.net == net) ++highlighted;
            }
        }
        const QString state = QString(
            "SYNTHETIC FIXTURE ONLY / NOT IMPORTED\n"
            "Net: %1\nTop: %2  Bottom: %3\n"
            "Visible primitives: %4  Highlighted: %5\n"
            "Coordinates: illustrative mm\nPhysical SI: NOT_RUN")
            .arg(net)
            .arg(top_->isChecked() ? "shown" : "hidden")
            .arg(bottom_->isChecked() ? "shown" : "hidden")
            .arg(visible).arg(highlighted);
        summary_->setText(state);
        if (detailsCallback_) detailsCallback_(state);
    }

public:
    explicit SyntheticArtworkPanel(QWidget *parent = nullptr)
        : QWidget(parent), scene_(new QGraphicsScene(this)),
          canvas_(new SyntheticArtworkCanvas(scene_, this)),
          nets_(new QComboBox(this)), top_(new QCheckBox("Top", this)),
          bottom_(new QCheckBox("Bottom", this)), summary_(new QLabel(this)) {
        setObjectName("syntheticArtworkP1");
        auto *layout = new QVBoxLayout(this);
        auto *row = new QHBoxLayout();
        auto *fixture = new QLabel("Synthetic board (2 layers)", this);
        fixture->setStyleSheet("font-weight: bold;");
        row->addWidget(fixture);
        top_->setObjectName("syntheticLayerTop");
        bottom_->setObjectName("syntheticLayerBottom");
        top_->setChecked(true);
        bottom_->setChecked(true);
        row->addWidget(top_);
        row->addWidget(bottom_);
        nets_->setObjectName("syntheticNetSelect");
        nets_->addItems({"All nets", "DATA_P0", "CLK_N0", "CTRL0"});
        row->addWidget(nets_);
        auto *zoomIn = new QPushButton("+", this);
        zoomIn->setObjectName("syntheticZoomIn");
        auto *zoomOut = new QPushButton("-", this);
        zoomOut->setObjectName("syntheticZoomOut");
        auto *fit = new QPushButton("Fit", this);
        fit->setObjectName("syntheticFit");
        row->addWidget(zoomIn);
        row->addWidget(zoomOut);
        row->addWidget(fit);
        layout->addLayout(row);
        canvas_->setObjectName("syntheticCanvas");
        layout->addWidget(canvas_, 1);
        summary_->setObjectName("syntheticArtworkStatus");
        summary_->setWordWrap(true);
        layout->addWidget(summary_);
        createFixture();

        connect(top_, &QCheckBox::toggled, this, [this] { refreshArtwork(); });
        connect(bottom_, &QCheckBox::toggled, this, [this] { refreshArtwork(); });
        connect(nets_, &QComboBox::currentTextChanged, this,
                [this] { refreshArtwork(); });
        connect(zoomIn, &QPushButton::clicked, this, [this] {
            const qreal scale = canvas_->transform().m11();
            if (scale * 1.2 <= 120.0) canvas_->scale(1.2, 1.2);
        });
        connect(zoomOut, &QPushButton::clicked, this, [this] {
            const qreal scale = canvas_->transform().m11();
            if (scale / 1.2 >= 0.03) canvas_->scale(1.0 / 1.2, 1.0 / 1.2);
        });
        connect(fit, &QPushButton::clicked, this, [this] { fitArtwork(); });
        connect(scene_, &QGraphicsScene::selectionChanged, this, [this] {
            for (QGraphicsItem *item : scene_->selectedItems()) {
                const QString net = item->data(0).toString();
                if (!net.isEmpty()) {
                    selectNet(net);
                    return;
                }
            }
        });
        refreshArtwork();
        QTimer::singleShot(0, this, [this] { fitArtwork(); });
    }

    void fitArtwork() {
        canvas_->fitInView(scene_->sceneRect(), Qt::KeepAspectRatio);
    }
    void setDetailsCallback(std::function<void(const QString &)> callback) {
        detailsCallback_ = std::move(callback);
        refreshArtwork();
    }
    QString selectedNet() const { return nets_->currentText(); }
    bool selectNet(const QString &net) {
        const int index = nets_->findText(net);
        if (index < 0) return false;
        nets_->setCurrentIndex(index);
        return true;
    }
    bool setLayerVisible(const QString &layer, bool visible) {
        if (layer == "Top") top_->setChecked(visible);
        else if (layer == "Bottom") bottom_->setChecked(visible);
        else return false;
        return true;
    }
    int primitiveCount() const { return static_cast<int>(primitives_.size()); }
    int visiblePrimitives() const {
        int count = 0;
        for (const Primitive &primitive : primitives_)
            if (primitive.item->isVisible()) ++count;
        return count;
    }
    int highlightedPrimitives() const {
        if (selectedNet() == "All nets") return 0;
        int count = 0;
        for (const Primitive &primitive : primitives_)
            if (primitive.item->isVisible() && primitive.net == selectedNet() &&
                primitive.item->opacity() > 0.9) ++count;
        return count;
    }
    qreal canvasScale() const { return canvas_->transform().m11(); }
};
