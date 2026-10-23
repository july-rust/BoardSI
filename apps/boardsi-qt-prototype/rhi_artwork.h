// BoardSI A_014: optional Qt Quick Scene Graph artwork backend.
// Uses only public Qt Quick APIs. The authoritative coordinates remain integer nm
// in the imported Canonical preview data; GPU floats are strictly viewport-local.
// GPU backend or frame events are not proof of a hardware adapter.
#pragma once
#include <QColor>
#include <QMouseEvent>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSGFlatColorMaterial>
#include <QSGGeometry>
#include <QSGGeometryNode>
#include <QSGRendererInterface>
#include <QTimer>
#include <QWheelEvent>
#include <QWidget>
#include <QVBoxLayout>
#include <QGuiApplication>
#include <QPointer>
#include <QPointF>
#include <QRectF>
#include <QtGlobal>
#include <algorithm>
#include <cmath>
#include <functional>
#include <map>
#include <limits>
#include <QShowEvent>
#include <vector>

class BoardSiRhiItem final : public QQuickItem {
    std::vector<ImportedTrack> tracks_;
    std::vector<ImportedVia> vias_;
    std::vector<ImportedPad> pads_;
    QVector<bool> visibility_;
    QString net_ = "All nets";
    qint64 originX_ = 0, originY_ = 0;
    QRectF bounds_;
    QPointF pan_;
    QPointF pressed_;
    bool dragging_ = false;
    qreal zoom_ = 1.0;
    int layers_ = 0;
    std::function<void(int,int)> picked_;

    QPointF world(qint64 x, qint64 y) const {
        // Subtract before converting to floating point: prevents GPU precision
        // loss for large absolute CAD positions.
        return QPointF(static_cast<double>(x - originX_) / 1e6,
                       -static_cast<double>(y - originY_) / 1e6);
    }
    qreal scaleFactor() const {
        return zoom_ * std::min(std::max<qreal>(1.0, width() - 32.0) /
                                   std::max<qreal>(1e-6, bounds_.width()),
                               std::max<qreal>(1.0, height() - 32.0) /
                                   std::max<qreal>(1e-6, bounds_.height()));
    }
    QPointF pixel(const QPointF &p) const {
        const qreal scale = scaleFactor();
        return QPointF((p.x() - bounds_.center().x()) * scale + width() / 2 + pan_.x(),
                       (p.y() - bounds_.center().y()) * scale + height() / 2 + pan_.y());
    }
    using Vertices = std::vector<QPointF>;
    static void triangle(Vertices &v, QPointF a, QPointF b, QPointF c) {
        v.push_back(a); v.push_back(b); v.push_back(c);
    }
    static void rectangle(Vertices &v, qreal x, qreal y, qreal w, qreal h) {
        const QPointF a(x,y), b(x+w,y), c(x+w,y+h), d(x,y+h);
        triangle(v,a,b,c); triangle(v,a,c,d);
    }
    static void disk(Vertices &v, QPointF c, qreal r, int segments = 24) {
        if (r <= 0) return;
        constexpr qreal pi = 3.14159265358979323846;
        for (int i = 0; i < segments; ++i) {
            const qreal a = 2 * pi * i / segments;
            const qreal b = 2 * pi * (i+1) / segments;
            triangle(v, c, c + QPointF(r*std::cos(a),r*std::sin(a)),
                     c + QPointF(r*std::cos(b),r*std::sin(b)));
        }
    }
    static void thickLine(Vertices &v, QPointF a, QPointF b, qreal halfWidth) {
        const QPointF diff = b-a;
        const qreal len = std::hypot(diff.x(),diff.y());
        if (len <= 1e-9) { disk(v,a,halfWidth); return; }
        const QPointF d(-diff.y() * halfWidth / len,
                         diff.x() * halfWidth / len);
        triangle(v,a+d,b+d,b-d); triangle(v,a+d,b-d,a-d);
        disk(v,a,halfWidth,12); disk(v,b,halfWidth,12);
    }
    bool visible(int layer) const {
        return layer > 0 && layer < visibility_.size() && visibility_[layer];
    }
    static bool inView(QPointF p, qreal radius, qreal w, qreal h) {
        return p.x()+radius >= -24 && p.y()+radius >= -24 &&
               p.x()-radius <= w+24 && p.y()-radius <= h+24;
    }
protected:
    QSGNode *updatePaintNode(QSGNode *old, UpdatePaintNodeData *) override {
        delete old;
        auto *root = new QSGNode();
        if (bounds_.isEmpty() || width() < 2 || height() < 2) return root;
        // Batch by color: O(number of color groups) scene graph nodes, not one
        // QPainter/QGraphicsItem draw call per CAD primitive.
        std::map<QRgb,Vertices> groups;
        Vertices holes;
        const qreal px = scaleFactor();
        for (const auto &t : tracks_) {
            if (!visible(t.layer)) continue;
            const QPointF a = pixel(world(t.x0_nm,t.y0_nm));
            const QPointF b = pixel(world(t.x1_nm,t.y1_nm));
            const qreal halfWidth = t.width_nm / 2e6 * px;
            const qreal extent = std::max<qreal>(20.0,halfWidth);
            if (!QRectF(a,b).normalized().adjusted(-extent,-extent,extent,extent)
                    .intersects(QRectF(0,0,width(),height()))) continue;
            const QColor base = t.layer == 1 ? QColor("#f29d55") :
                 t.layer == layers_ ? QColor("#55b4e9") : QColor("#a9d48e");
            QColor color(base);
            if (net_ != "All nets" && net_ != t.net) color.setAlpha(40);
            thickLine(groups[color.rgba()],a,b,
                      std::max<qreal>(0.1,halfWidth));
        }
        for (const auto &pad : pads_) {
            if (!visible(pad.layer)) continue;
            const QPointF c = pixel(world(pad.x_nm,pad.y_nm));
            const qreal w = pad.width_nm / 1e6 * px;
            const qreal h = pad.height_nm / 1e6 * px;
            if (!inView(c,std::max(w,h)/2,width(),height())) continue;
            QColor color(pad.side == "B" ? "#55b4e9" : "#d0a55a");
            // PADS pad nets are NOT_VERIFIED, and must never be promoted
            // to electrical connectivity on selection.
            if (net_ != "All nets") color.setAlpha(40);
            auto &v = groups[color.rgba()];
            if (pad.shape == "R") disk(v,c,w/2);
            else rectangle(v,c.x()-w/2,c.y()-h/2,w,h);
        }
        for (const auto &via : vias_) {
            bool shown = false;
            for (int l=via.firstLayer; l<=via.lastLayer; ++l)
                if (visible(l)) { shown=true; break; }
            if (!shown) continue;
            const QPointF c = pixel(world(via.x_nm,via.y_nm));
            const qreal outer = via.diameter_nm / 2e6 * px;
            if (!inView(c,outer,width(),height())) continue;
            QColor color("#b98b45");
            if (net_ != "All nets" && net_ != via.net) color.setAlpha(40);
            disk(groups[color.rgba()],c,outer);
            disk(holes,c,via.drill_nm / 2e6 * px);
        }
        // Bounded batch nodes. No private QRhi APIs or per-object GPU nodes.
        for (auto &entry : groups) {
            if (entry.second.empty()) continue;
            if (entry.second.size() > static_cast<size_t>(std::numeric_limits<int>::max())) continue;
            auto *node = new QSGGeometryNode();
            auto *geometry = new QSGGeometry(QSGGeometry::defaultAttributes_Point2D(),
                                           static_cast<int>(entry.second.size()));
            geometry->setDrawingMode(QSGGeometry::DrawTriangles);
            auto *dst = geometry->vertexDataAsPoint2D();
            for (size_t i=0; i<entry.second.size(); ++i)
                dst[i].set(static_cast<float>(entry.second[i].x()),
                           static_cast<float>(entry.second[i].y()));
            auto *material = new QSGFlatColorMaterial();
            material->setColor(QColor::fromRgba(entry.first));
            node->setGeometry(geometry);
            node->setMaterial(material);
            node->setFlag(QSGNode::OwnsGeometry);
            node->setFlag(QSGNode::OwnsMaterial);
            root->appendChildNode(node);
        }
        if (!holes.empty() && holes.size() <= static_cast<size_t>(std::numeric_limits<int>::max())) {
            auto *node = new QSGGeometryNode();
            auto *geometry = new QSGGeometry(QSGGeometry::defaultAttributes_Point2D(),
                                           static_cast<int>(holes.size()));
            geometry->setDrawingMode(QSGGeometry::DrawTriangles);
            auto *dst = geometry->vertexDataAsPoint2D();
            for (size_t i=0; i<holes.size(); ++i)
                dst[i].set(static_cast<float>(holes[i].x()),static_cast<float>(holes[i].y()));
            auto *material = new QSGFlatColorMaterial();
            material->setColor(QColor("#232c34"));
            node->setGeometry(geometry);
            node->setMaterial(material);
            node->setFlag(QSGNode::OwnsGeometry);
            node->setFlag(QSGNode::OwnsMaterial);
            root->appendChildNode(node);
        }
        return root;
    }
    void mousePressEvent(QMouseEvent *event) override {
        pressed_ = event->position(); dragging_ = false;
        event->accept();
    }
    void mouseMoveEvent(QMouseEvent *event) override {
        if (!(event->buttons() & Qt::LeftButton)) return;
        QPointF delta = event->position() - pressed_;
        if (std::hypot(delta.x(),delta.y()) >= 1) dragging_ = true;
        pan_ += delta; pressed_ = event->position(); update(); event->accept();
    }
    void mouseReleaseEvent(QMouseEvent *event) override {
        if (!dragging_ && picked_) {
            // Screen-space picking is independent of float GPU coordinates.
            for (size_t i=0;i<pads_.size();++i) {
                const auto &p=pads_[i];
                if (!visible(p.layer)) continue;
                const QPointF c=pixel(world(p.x_nm,p.y_nm));
                const qreal dx=event->position().x()-c.x();
                const qreal dy=event->position().y()-c.y();
                const qreal hw=p.width_nm / 2e6 * scaleFactor()+3;
                const qreal hh=p.height_nm / 2e6 * scaleFactor()+3;
                const bool hit=p.shape=="R" ? std::hypot(dx,dy)<=hw :
                                            std::abs(dx)<=hw && std::abs(dy)<=hh;
                if (hit) {
                    picked_(2,static_cast<int>(i)); event->accept(); return;
                }
            }
            for (size_t i=0;i<vias_.size();++i) {
                const auto &v=vias_[i];
                bool shown=false;
                for(int layer=v.firstLayer;layer<=v.lastLayer;++layer)
                    if(visible(layer)) {shown=true;break;}
                if(!shown) continue;
                const QPointF c=pixel(world(v.x_nm,v.y_nm));
                if (std::hypot(event->position().x()-c.x(),event->position().y()-c.y()) <=
                    v.diameter_nm / 2e6 * scaleFactor()+3) {
                    picked_(1,static_cast<int>(i)); event->accept(); return;
                }
            }
            // Existing QGraphicsScene supports straight copper selection by
            // net and provenance. Mirror that behavior in the RHI viewport.
            for (size_t i=tracks_.size();i>0;--i) {
                const auto &t=tracks_[i-1];
                if (!visible(t.layer)) continue;
                const QPointF a=pixel(world(t.x0_nm,t.y0_nm));
                const QPointF b=pixel(world(t.x1_nm,t.y1_nm));
                const QPointF v=b-a, m=event->position()-a;
                const qreal length2=v.x()*v.x()+v.y()*v.y();
                const qreal u=length2>0 ?
                    std::clamp((m.x()*v.x()+m.y()*v.y())/length2,0.0,1.0) : 0.0;
                const QPointF nearest=a+v*u;
                if(std::hypot(event->position().x()-nearest.x(),
                              event->position().y()-nearest.y()) <=
                              std::max<qreal>(3.0,t.width_nm/2e6*scaleFactor()+2)) {
                    picked_(3,static_cast<int>(i-1));event->accept();return;
                }
            }
        }
        event->accept();
    }
    void wheelEvent(QWheelEvent *event) override {
        const qreal factor=event->angleDelta().y()>0?1.2:1.0/1.2;
        if (zoom_*factor >= 0.03 && zoom_*factor <= 120.0)
            zoom_*=factor;
        update(); event->accept();
    }
public:
    explicit BoardSiRhiItem(QQuickItem *parent=nullptr) : QQuickItem(parent) {
        setFlag(ItemHasContents,true);
        setAcceptedMouseButtons(Qt::LeftButton);
    }
    void onPicked(std::function<void(int,int)> callback) { picked_=std::move(callback); }
    void load(int layers,const std::vector<ImportedTrack> &tracks,
              const std::vector<ImportedVia> &vias,const std::vector<ImportedPad> &pads) {
        tracks_=tracks; vias_=vias; pads_=pads; layers_=layers;
        visibility_=QVector<bool>(layers+1,true);
        qint64 x=0,y=0; bool initialized=false;
        auto ingest=[&](qint64 a,qint64 b) {
            if (!initialized) { x=a;y=b;initialized=true; }
            else { x=std::min(x,a);y=std::min(y,b); }
        };
        for (const auto &t:tracks_) { ingest(t.x0_nm,t.y0_nm);ingest(t.x1_nm,t.y1_nm); }
        for (const auto &v:vias_) ingest(v.x_nm,v.y_nm);
        for (const auto &p:pads_) ingest(p.x_nm,p.y_nm);
        originX_=x;originY_=y;
        QRectF rect;
        bool first=true;
        auto add=[&](QPointF p,qreal r) {
            const QRectF b(p.x()-r,p.y()-r,2*r,2*r);
            if(first){rect=b;first=false;} else rect=rect.united(b);
        };
        for(const auto &t:tracks_) {
            add(world(t.x0_nm,t.y0_nm),t.width_nm/2e6);
            add(world(t.x1_nm,t.y1_nm),t.width_nm/2e6);
        }
        for(const auto &v:vias_) add(world(v.x_nm,v.y_nm),v.diameter_nm/2e6);
        for(const auto &p:pads_) {
            const QPointF c=world(p.x_nm,p.y_nm);
            add(c,std::max(p.width_nm,p.height_nm)/2e6);
        }
        bounds_=rect.adjusted(-1,-1,1,1);
        pan_=QPointF();zoom_=1.0;update();
    }
    void filter(QVector<bool> layers,const QString &net) {
        visibility_=std::move(layers); net_=net; update();
    }
    void fit() { pan_=QPointF();zoom_=1.0;update(); }
    void zoom(qreal factor) {
        if(zoom_*factor >= 0.03 && zoom_*factor <= 120.0) {
            zoom_*=factor; update();
        }
    }
    int primitiveCount() const {
        return static_cast<int>(tracks_.size()+vias_.size()+pads_.size());
    }
    QPointF localPoint(qint64 x,qint64 y) const { return world(x,y); }
    // Read-only test seam: validate the actual GPU scene's imported source
    // coordinates/filters, not the parallel hidden QGraphicsScene mirror.
    bool containsExactPad(const QString &ref,const QString &pin,int layer,
                          int rotation,qint64 x,qint64 y,qint64 w,qint64 h) const {
        for(const auto &p:pads_) {
            if(p.reference==ref && p.pin==pin && p.layer==layer &&
               p.rotation==rotation && p.x_nm==x && p.y_nm==y &&
               p.width_nm==w && p.height_nm==h) return true;
        }
        return false;
    }
    int visiblePadCount() const {
        int n=0;
        for(const auto &p:pads_) if(visible(p.layer)) ++n;
        return n;
    }
    QPointF screenPointForTest(qint64 x,qint64 y) const {
        return pixel(world(x,y));
    }
};

class BoardSiRhiView final : public QWidget {
    bool timeoutStarted_=false;
protected:
    void showEvent(QShowEvent *event) override {
        QWidget::showEvent(event);
        if (timeoutStarted_) return;
        timeoutStarted_=true;
        // A hidden import tab is not a failed GPU. Start the frame deadline
        // only after the native window container is actually displayed.
        QTimer::singleShot(3000,this,[this] {
            if (seenFrame_ || failed()) return;
            // Inactive tabs must not time out while hidden.
            if (!isVisible()) { timeoutStarted_=false; return; }
            status_="CPU_FALLBACK_RHI_NO_FRAME";
            if (changed_) changed_();
        });
    }
private:
    QQuickWindow *window_=nullptr;
    BoardSiRhiItem *artwork_=nullptr;
    QString status_="RHI_PENDING_FRAME_HW_NOT_VERIFIED";
    std::function<void()> changed_;
    bool seenFrame_=false;
public:
    static bool softwareRendererRequested() {
        const auto value = qEnvironmentVariable("QSG_RHI_PREFER_SOFTWARE_RENDERER")
                               .trimmed().toLower();
        // Covers Qt's explicit WARP/software-adapter request behind D3D11
        // or Vulkan. A D3D11 backend name alone is never hardware proof.
        return !value.isEmpty() && value != "0" && value != "false" &&
               value != "off" && value != "no";
    }
    explicit BoardSiRhiView(QWidget *parent=nullptr) : QWidget(parent) {
        auto *layout=new QVBoxLayout(this);
        layout->setContentsMargins(0,0,0,0);
#if defined(Q_OS_WIN)
        QQuickWindow::setGraphicsApi(QSGRendererInterface::Direct3D11);
#elif defined(Q_OS_MACOS)
        QQuickWindow::setGraphicsApi(QSGRendererInterface::Metal);
#endif
        window_=new QQuickWindow();
        window_->setColor(QColor("#111923"));
        artwork_=new BoardSiRhiItem(window_->contentItem());
        artwork_->setParentItem(window_->contentItem());
        artwork_->setWidth(window_->width());
        artwork_->setHeight(window_->height());
        QObject::connect(window_,&QQuickWindow::widthChanged,artwork_,
                         [this](int w){artwork_->setWidth(w);artwork_->update();});
        QObject::connect(window_,&QQuickWindow::heightChanged,artwork_,
                         [this](int h){artwork_->setHeight(h);artwork_->update();});
        QObject::connect(window_,&QQuickWindow::frameSwapped,this,[this] {
            if(seenFrame_) return;
            seenFrame_=true;
            const auto api=window_->rendererInterface()->graphicsApi();
            QString backend="UNEXPECTED";
            if(api==QSGRendererInterface::Direct3D11) backend="DIRECT3D11";
            else if(api==QSGRendererInterface::Metal) backend="METAL";
            else if(api==QSGRendererInterface::OpenGL) backend="OPENGL";
            else if(api==QSGRendererInterface::Vulkan) backend="VULKAN";
            else if(api==QSGRendererInterface::Software) backend="SOFTWARE";
            // Qt's software Scene Graph must not count as GPU-accelerated.
            // A recognized API without adapter identity is still NOT_VERIFIED.
            if (backend=="SOFTWARE" || backend=="UNEXPECTED") {
                status_="CPU_FALLBACK_RHI_SOFTWARE_OR_UNKNOWN";
            } else {
                status_="RHI_FRAME_"+backend+"_HARDWARE_NOT_VERIFIED";
            }
            if(changed_) changed_();
        },Qt::QueuedConnection);
        QObject::connect(window_,&QQuickWindow::sceneGraphError,this,
                         [this](QQuickWindow::SceneGraphError,const QString &) {
            status_="CPU_FALLBACK_RHI_SCENEGRAPH_ERROR";
            if(changed_) changed_();
        },Qt::QueuedConnection);
        auto *container=QWidget::createWindowContainer(window_,this);
        container->setMinimumHeight(220);
        layout->addWidget(container);
    }
    void onChanged(std::function<void()> callback) {changed_=std::move(callback);}
    void onPicked(std::function<void(int,int)> callback) {artwork_->onPicked(std::move(callback));}
    void load(int layers,const std::vector<ImportedTrack> &tracks,
              const std::vector<ImportedVia> &vias,const std::vector<ImportedPad> &pads) {
        artwork_->load(layers,tracks,vias,pads);
    }
    void filter(QVector<bool> layers,const QString &net) {artwork_->filter(std::move(layers),net);}
    void fit() {artwork_->fit();}
    void zoom(qreal factor) {artwork_->zoom(factor);}
    QString status() const {return status_;}
    bool failed() const {return status_.startsWith("CPU_FALLBACK");}
    int primitiveCount() const {return artwork_->primitiveCount();}
    BoardSiRhiItem *itemForTest() const {return artwork_;}
};
