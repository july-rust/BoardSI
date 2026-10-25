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
#include <QSGRenderNode>
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
#include <unordered_map>
#include <atomic>
#include <memory>
#ifdef Q_OS_MACOS
extern "C" bool boardSiMetalDeviceConfirmed(void *opaqueDevice);
#endif
#ifdef Q_OS_WIN
#include <d3d11.h>
#include <dxgi1_2.h>
#endif
// Win32 headers may publish min/max macros which invalidate std::min/max.
#if defined(min)
#undef min
#endif
#if defined(max)
#undef max
#endif
// Windows SDK also defines near/far compatibility macros; the existing
// native Qt tests use a local lambda named near.
#if defined(near)
#undef near
#endif
#if defined(far)
#undef far
#endif
#include <limits>
#include <QShowEvent>
#include <QHideEvent>
#include <vector>

// Adapter check on Qt's render thread. DeviceResource access on the GUI
// thread is forbidden; inside QSGRenderNode::render it is supported.
class BoardSiRhiAdapterProbe final : public QSGRenderNode {
    QQuickWindow *win_;
    std::shared_ptr<std::atomic<int>> proof_;
public:
    BoardSiRhiAdapterProbe(QQuickWindow *win,std::shared_ptr<std::atomic<int>> proof)
      : win_(win),proof_(std::move(proof)) {}
    RenderingFlags flags() const override { return {}; }
    StateFlags changedStates() const override { return {}; }
    QRectF rect() const override { return QRectF(0,0,1,1); }
    void render(const RenderState *) override {
        if (!win_) { proof_->store(-2,std::memory_order_release); return; }
#ifdef Q_OS_WIN
        if (!win_ || win_->rendererInterface()->graphicsApi()!=
                         QSGRendererInterface::Direct3D11) {
            proof_->store(-2,std::memory_order_release); return;
        }
        auto *device=static_cast<ID3D11Device *>(
            win_->rendererInterface()->getResource(win_,QSGRendererInterface::DeviceResource));
        if (!device) {proof_->store(-2,std::memory_order_release);return;}
        IDXGIDevice *dxgi=nullptr;
        if (FAILED(device->QueryInterface(__uuidof(IDXGIDevice),
                                         reinterpret_cast<void **>(&dxgi))) || !dxgi) {
            proof_->store(-2,std::memory_order_release);return;
        }
        IDXGIAdapter *adapter=nullptr;
        const HRESULT h=dxgi->GetAdapter(&adapter);
        dxgi->Release();
        if (FAILED(h) || !adapter) {proof_->store(-2,std::memory_order_release);return;}
        IDXGIAdapter1 *adapter1=nullptr;
        const HRESULT qi=adapter->QueryInterface(
            __uuidof(IDXGIAdapter1),reinterpret_cast<void **>(&adapter1));
        adapter->Release();
        if (FAILED(qi) || !adapter1) {
            proof_->store(-2,std::memory_order_release);return;
        }
        DXGI_ADAPTER_DESC1 desc{};
        const HRESULT got=adapter1->GetDesc1(&desc);
        adapter1->Release();
        if (FAILED(got)) {proof_->store(-2,std::memory_order_release);return;}
        proof_->store((desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE) ? -1 : 1,
                      std::memory_order_release);
#elif defined(Q_OS_MACOS)
        if (win_->rendererInterface()->graphicsApi()!=QSGRendererInterface::Metal) {
            proof_->store(-2,std::memory_order_release); return;
        }
        // The renderer's actual Metal device, not just a Metal API enum.
        void *resource=win_->rendererInterface()->getResource(
            win_,QSGRendererInterface::DeviceResource);
        proof_->store(boardSiMetalDeviceConfirmed(resource) ? 1 : -2,
                      std::memory_order_release);
#else
        proof_->store(2,std::memory_order_release);
#endif
    }
};

class BoardSiRhiItem final : public QQuickItem {
    std::vector<ImportedTrack> tracks_;
    std::vector<ImportedVia> vias_;
    std::vector<ImportedPad> pads_;
    std::vector<ImportedArc> arcs_;
    std::vector<ImportedOutlineEdge> outlineEdges_;
    std::vector<ImportedPourEdge> pours_;
    std::vector<ImportedCopperArea> copper_;
    std::vector<ImportedCircle> circles_;
    std::vector<ImportedBoundarySegment> boundaries_;
    QVector<bool> visibility_;
    QString net_ = "All nets";
    bool netOnly_ = false; // Declared labels, not verified connectivity.
    qint64 originX_ = 0, originY_ = 0;
    QRectF bounds_;
    QPointF pan_;
    QPointF pressed_;
    bool dragging_ = false;
    qreal zoom_ = 1.0;
    int layers_ = 0;
    std::shared_ptr<std::atomic<int>> adapterProof_ =
        std::make_shared<std::atomic<int>>(0);
    std::atomic<quint64> modelGeneration_{0};
    std::atomic<quint64> lastSyncedGeneration_{0};
    std::atomic<quint64> sceneRevision_{0};
    std::atomic<quint64> lastSyncedSceneRevision_{0};
    void requestSceneUpdate() {
        sceneRevision_.fetch_add(1,std::memory_order_release);
        update();
    }
    std::function<void(int,int)> picked_;
    // Coarse source-nm bins, not rasterized polygons; long routes are indexed
    // separately so that an unusually long trace cannot explode the index.
    std::unordered_map<quint64,std::vector<int>> trackBins_;
    std::vector<int> spanningTracks_;
    qint64 trackBinNm_ = 1;
    static qint64 floorBin(qint64 value,qint64 step) {
        return value>=0 ? value/step : -((-value+step-1)/step);
    }
    static quint64 binKey(qint64 x,qint64 y) {
        return (static_cast<quint64>(x)<<32) | static_cast<quint32>(y);
    }
    void rebuildTrackBins() {
        trackBins_.clear(); spanningTracks_.clear();
        trackBinNm_=std::max<qint64>(1,static_cast<qint64>(
            std::ceil(std::max(bounds_.width(),bounds_.height())*1e6/128.0)));
        for(size_t i=0;i<tracks_.size();++i) {
            const auto &t=tracks_[i];
            const qint64 r=(t.width_nm+1)/2;
            const qint64 x0=floorBin(std::min(t.x0_nm,t.x1_nm)-r-originX_,trackBinNm_);
            const qint64 x1=floorBin(std::max(t.x0_nm,t.x1_nm)+r-originX_,trackBinNm_);
            const qint64 y0=floorBin(std::min(t.y0_nm,t.y1_nm)-r-originY_,trackBinNm_);
            const qint64 y1=floorBin(std::max(t.y0_nm,t.y1_nm)+r-originY_,trackBinNm_);
            if(x1-x0>15 || y1-y0>15 || (x1-x0+1)*(y1-y0+1)>256) {
                spanningTracks_.push_back(static_cast<int>(i)); continue;
            }
            for(qint64 x=x0;x<=x1;++x)
                for(qint64 y=y0;y<=y1;++y)
                    trackBins_[binKey(x,y)].push_back(static_cast<int>(i));
        }
    }
    int trackIndexAt(const QPointF &pos) const {
        if(tracks_.empty()) return -1;
        const qreal scale=scaleFactor();
        if(!(scale>0) || !std::isfinite(scale)) return -1;
        const qreal mmX=(pos.x()-width()/2-pan_.x())/scale+bounds_.center().x();
        const qreal mmY=(pos.y()-height()/2-pan_.y())/scale+bounds_.center().y();
        const qint64 nx=originX_+static_cast<qint64>(std::llround(mmX*1e6));
        const qint64 ny=originY_-static_cast<qint64>(std::llround(mmY*1e6));
        const qint64 tolerance=static_cast<qint64>(std::ceil(3e6/scale))+1;
        const qint64 x0=floorBin(nx-tolerance-originX_,trackBinNm_);
        const qint64 x1=floorBin(nx+tolerance-originX_,trackBinNm_);
        const qint64 y0=floorBin(ny-tolerance-originY_,trackBinNm_);
        const qint64 y1=floorBin(ny+tolerance-originY_,trackBinNm_);
        std::vector<int> candidates(spanningTracks_);
        if(x1-x0>256 || y1-y0>256 || (x1-x0+1)*(y1-y0+1)>512) {
            candidates.clear(); candidates.reserve(tracks_.size());
            for(size_t i=0;i<tracks_.size();++i)
                candidates.push_back(static_cast<int>(i));
        } else {
            for(qint64 x=x0;x<=x1;++x)
                for(qint64 y=y0;y<=y1;++y) {
                    const auto it=trackBins_.find(binKey(x,y));
                    if(it!=trackBins_.end())
                        candidates.insert(candidates.end(),it->second.begin(),it->second.end());
                }
        }
        std::sort(candidates.begin(),candidates.end(),std::greater<int>());
        candidates.erase(std::unique(candidates.begin(),candidates.end()),candidates.end());
        for(const int index:candidates) {
            const auto &t=tracks_[static_cast<size_t>(index)];
            if(!visible(t.layer) || !includedNet(t.net)) continue;
            const QPointF a=pixel(world(t.x0_nm,t.y0_nm));
            const QPointF b=pixel(world(t.x1_nm,t.y1_nm));
            const QPointF v=b-a,delta=pos-a;
            const qreal len2=v.x()*v.x()+v.y()*v.y();
            const qreal u=len2>0 ? std::clamp(
                (delta.x()*v.x()+delta.y()*v.y())/len2,0.0,1.0) : 0.0;
            const QPointF nearest=a+v*u;
            if(std::hypot(pos.x()-nearest.x(),pos.y()-nearest.y()) <=
               std::max<qreal>(3.0,t.width_nm/2e6*scale+2))
                return index;
        }
        return -1;
    }

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
    struct ArcAngles {
        qreal start, sweep;
    };
    static ArcAngles arcAngles(const ImportedArc &a) {
        constexpr qreal pi=3.14159265358979323846;
        const qreal first=std::atan2(static_cast<qreal>(a.y0_nm-a.cy_nm),
                                     static_cast<qreal>(a.x0_nm-a.cx_nm));
        const qreal last=std::atan2(static_cast<qreal>(a.y1_nm-a.cy_nm),
                                    static_cast<qreal>(a.x1_nm-a.cx_nm));
        qreal sweep=last-first;
        if (a.clockwise) { while (sweep >= 0) sweep -= 2*pi; }
        else { while (sweep <= 0) sweep += 2*pi; }
        return {first,sweep};
    }
    static bool angleOnSweep(const ArcAngles &angles, qreal phi) {
        constexpr qreal twoPi=6.28318530717958647692;
        qreal ccw=std::fmod(phi-angles.start,twoPi);
        if(ccw<0) ccw+=twoPi;
        if(angles.sweep>0) return ccw<=angles.sweep+1e-10;
        const qreal cw=ccw<=1e-10 ? 0.0 : ccw-twoPi;
        return cw>=angles.sweep-1e-10;
    }
    // Total arc geometry is bounded across the entire visible scene, not per
    // primitive. A small fixture retains high LOD, 10k arcs cannot explode.
    static int arcSegmentBudgetPerVisibleArc(size_t visibleArcs) {
        const size_t divisor=std::max<size_t>(1,visibleArcs);
        return std::clamp(static_cast<int>(220000/divisor),4,512);
    }
    int arcIndexAt(const QPointF &pos) const {
        // No invented straight chord: hit testing follows actual circle sweep.
        constexpr qreal twoPi=6.28318530717958647692;
        for(size_t i=arcs_.size();i>0;--i) {
            const auto &a=arcs_[i-1];
            if(!visible(a.layer) || !includedNet(a.net)) continue;
            const QPointF centre=pixel(world(a.cx_nm,a.cy_nm));
            const QPointF delta=pos-centre;
            const qreal radius=std::hypot(
                static_cast<qreal>(a.x0_nm-a.cx_nm),
                static_cast<qreal>(a.y0_nm-a.cy_nm))/1e6*scaleFactor();
            if(std::abs(std::hypot(delta.x(),delta.y())-radius)>
               std::max<qreal>(3.0,a.width_nm/2e6*scaleFactor()+2)) continue;
            const qreal phi=std::atan2(-delta.y(),delta.x());
            const auto angles=arcAngles(a);
            qreal travel=phi-angles.start;
            if(a.clockwise) {while(travel>0) travel-=twoPi;
                             while(travel<=-twoPi) travel+=twoPi;}
            else {while(travel<0) travel+=twoPi;
                  while(travel>=twoPi) travel-=twoPi;}
            if((a.clockwise && travel>=angles.sweep) ||
               (!a.clockwise && travel<=angles.sweep))
                return static_cast<int>(i-1);
        }
        return -1;
    }
    // Mechanical straight outlines are independent of copper/net visibility.
    // Use source-nm geometry projected to screen space, identical to CPU artwork.
    int outlineIndexAt(const QPointF &pos) const {
        for(size_t i=outlineEdges_.size();i>0;--i) {
            const auto &e=outlineEdges_[i-1];
            const QPointF a=pixel(world(e.x0_nm,e.y0_nm));
            const QPointF b=pixel(world(e.x1_nm,e.y1_nm));
            const QPointF delta=b-a,offset=pos-a;
            const qreal length2=delta.x()*delta.x()+delta.y()*delta.y();
            const qreal u=length2>0 ? std::clamp(
                (delta.x()*offset.x()+delta.y()*offset.y())/length2,0.0,1.0):0.0;
            const QPointF nearest=a+delta*u;
            if(std::hypot(pos.x()-nearest.x(),pos.y()-nearest.y())<=3.5)
                return static_cast<int>(i-1);
        }
        return -1;
    }
    int copperIndexAt(const QPointF &pos) const {
        // Use the same validated triangles as the Scene Graph, in screen
        // coordinates. Source nm remains integer and never roundtrips via GPU.
        auto side=[](QPointF a,QPointF b,QPointF p) {
            return (b.x()-a.x())*(p.y()-a.y())-
                   (b.y()-a.y())*(p.x()-a.x());
        };
        for(size_t i=copper_.size();i>0;--i) {
            const auto &c=copper_[i-1];
            if(!visible(c.layer) || !includedNet(c.net) ||
               (netOnly_ && net_!="All nets" && c.net=="-")) continue;
            if(c.circle) {
                const QPointF center=pixel(world(c.cx_nm,c.cy_nm));
                if(std::hypot(pos.x()-center.x(),pos.y()-center.y())<=
                   c.radius_nm/1e6*scaleFactor()) return static_cast<int>(i-1);
                continue;
            }
            for(const auto &tri:c.hollowTriangles) {
                const auto point=[&](std::pair<qint64,qint64> p) {
                    return pixel(world(p.first,p.second));
                };
                const QPointF a=point(tri[0]),b=point(tri[1]),p2=point(tri[2]);
                const auto x=side(a,b,pos),y=side(b,p2,pos),z=side(p2,a,pos);
                if((x>=0 && y>=0 && z>=0)||(x<=0 && y<=0 && z<=0))
                    return static_cast<int>(i-1);
            }
            if(!c.hollowTriangles.empty())continue;
            for(const auto &indices:c.triangles) {
                const auto point=[&](int n) {
                    auto p=c.ring[static_cast<size_t>(n)];
                    return pixel(world(p.first,p.second));
                };
                const QPointF a=point(indices[0]),b=point(indices[1]),p2=point(indices[2]);
                const auto x=side(a,b,pos),y=side(b,p2,pos),z=side(p2,a,pos);
                if((x>=0 && y>=0 && z>=0)||(x<=0 && y<=0 && z<=0))
                    return static_cast<int>(i-1);
            }
        }
        return -1;
    }
    int pourIndexAt(const QPointF &pos) const {
        for(size_t i=pours_.size();i>0;--i) {
            const auto &e=pours_[i-1];
            if(!visible(e.layer) || !includedNet(e.net) ||
               (netOnly_ && net_!="All nets" && e.net=="-"))
                continue;
            const QPointF a=pixel(world(e.x0_nm,e.y0_nm));
            const QPointF b=pixel(world(e.x1_nm,e.y1_nm));
            const QPointF delta=b-a, offset=pos-a;
            const qreal length2=delta.x()*delta.x()+delta.y()*delta.y();
            const qreal u=length2>0 ? std::clamp(
                (delta.x()*offset.x()+delta.y()*offset.y())/length2,0.0,1.0):0.0;
            const QPointF nearest=a+delta*u;
            if(std::hypot(pos.x()-nearest.x(),pos.y()-nearest.y())<=3.5)
                return static_cast<int>(i-1);
        }
        return -1;
    }
    int boundaryIndexAt(const QPointF &pos) const {
        for(size_t i=boundaries_.size();i>0;--i) {
            const auto &b=boundaries_[i-1];
            const QPointF start=pixel(world(b.x0_nm,b.y0_nm));
            const QPointF end=pixel(world(b.x1_nm,b.y1_nm));
            if(!b.isArc) {
                const QPointF delta=end-start,offset=pos-start;
                const qreal length2=delta.x()*delta.x()+delta.y()*delta.y();
                const qreal u=length2>0 ? std::clamp(
                    (delta.x()*offset.x()+delta.y()*offset.y())/length2,0.0,1.0):0.0;
                const QPointF nearest=start+delta*u;
                if(std::hypot(pos.x()-nearest.x(),pos.y()-nearest.y())<=3.5)
                    return static_cast<int>(i-1);
                continue;
            }
            ImportedArc arc;
            arc.x0_nm=b.x0_nm;arc.y0_nm=b.y0_nm;
            arc.cx_nm=b.cx_nm;arc.cy_nm=b.cy_nm;
            arc.x1_nm=b.x1_nm;arc.y1_nm=b.y1_nm;
            arc.clockwise=b.clockwise;
            const QPointF c=pixel(world(b.cx_nm,b.cy_nm));
            const qreal radius=std::hypot(static_cast<qreal>(b.x0_nm-b.cx_nm),
                                           static_cast<qreal>(b.y0_nm-b.cy_nm))
                                /1e6*scaleFactor();
            const QPointF delta=pos-c;
            if(std::abs(std::hypot(delta.x(),delta.y())-radius)>3.5)
                continue;
            const qreal phi=std::atan2(-delta.y(),delta.x());
            if(angleOnSweep(arcAngles(arc),phi)) return static_cast<int>(i-1);
        }
        return -1;
    }
    int circleIndexAt(const QPointF &pos) const {
        for(size_t i=circles_.size();i>0;--i) {
            const auto &a=circles_[i-1];
            const QPointF c=pixel(world(a.cx_nm,a.cy_nm));
            const qreal r=a.radius_nm/1e6*scaleFactor();
            if (std::abs(std::hypot(pos.x()-c.x(),pos.y()-c.y())-r)<=3.5)
                return static_cast<int>(i-1);
        }
        return -1;
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
    // A_019 P2-J: only omit caps below the PHYSICAL half-pixel threshold.
    // Item dimensions and scaleFactor() use logical pixels; high-DPI displays
    // need QWindow devicePixelRatio() to preserve visibly rounded endpoints.
    // Source centreline, width and picking remain exact and unchanged.
    static constexpr qreal subpixelCapRadius = 0.5;
    static int nonDegenerateLineVerticesForTest(qreal halfWidthLogicalPx,
                                                qreal devicePixelRatio) {
        const qreal dpr=std::max<qreal>(1.0,devicePixelRatio);
        return halfWidthLogicalPx*dpr <= subpixelCapRadius ? 6 : 78;
    }
    static void thickLine(Vertices &v, QPointF a, QPointF b, qreal halfWidth,
                          qreal devicePixelRatio) {
        const QPointF diff = b-a;
        const qreal len = std::hypot(diff.x(),diff.y());
        if (len <= 1e-9) { disk(v,a,halfWidth); return; }
        const QPointF d(-diff.y() * halfWidth / len,
                         diff.x() * halfWidth / len);
        triangle(v,a+d,b+d,b-d); triangle(v,a+d,b-d,a-d);
        if (halfWidth*std::max<qreal>(1.0,devicePixelRatio) > subpixelCapRadius) {
            disk(v,a,halfWidth,12); disk(v,b,halfWidth,12);
        }
    }
    bool visible(int layer) const {
        return layer > 0 && layer < visibility_.size() && visibility_[layer];
    }
    bool includedNet(const QString &name) const {
        return !netOnly_ || net_ == "All nets" || name == net_;
    }
    static bool inView(QPointF p, qreal radius, qreal w, qreal h) {
        return p.x()+radius >= -24 && p.y()+radius >= -24 &&
               p.x()-radius <= w+24 && p.y()-radius <= h+24;
    }
protected:
    QSGNode *updatePaintNode(QSGNode *old, UpdatePaintNodeData *) override {
        delete old;
        auto *root = new QSGNode();
        // Synchronization phase executes on the render thread. Capture the
        // exact artwork generation installed in this scene graph update.
        lastSyncedGeneration_.store(
            modelGeneration_.load(std::memory_order_acquire),
            std::memory_order_release);
        lastSyncedSceneRevision_.store(
            sceneRevision_.load(std::memory_order_acquire),
            std::memory_order_release);
        if (window())
            root->appendChildNode(new BoardSiRhiAdapterProbe(window(),adapterProof_));
        if (bounds_.isEmpty() || width() < 2 || height() < 2) return root;
        // Batch by color: O(number of color groups) scene graph nodes, not one
        // QPainter/QGraphicsItem draw call per CAD primitive.
        std::map<QRgb,Vertices> groups;
        // Preserve the source stacking order of overlapping copper fills.
        // Merge adjacent same-color areas only; never reorder by RGB key.
        // Bounded by at most 64 explicit source copper areas.
        std::vector<std::pair<QRgb,Vertices>> copperGroups;
        Vertices holes;
        const qreal px = scaleFactor();
        // Scene Graph sync is on the render thread. The window DPR is used
        // only to decide the raster LOD, never to alter Canonical geometry.
        const qreal deviceRatio = window() ?
            std::max<qreal>(1.0,window()->devicePixelRatio()) : 1.0;
        for (const auto &t : tracks_) {
            if (!visible(t.layer) || !includedNet(t.net)) continue;
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
                      std::max<qreal>(0.1,halfWidth),deviceRatio);
        }
        // Explicit source-filled copper. The user-authored polygon is
        // triangulated once on import, NOT reconstructed from POUROUT.
        // Keep fills in separate batched nodes behind tracks/vias/outline.
        for(const auto &c:copper_) {
            if(!visible(c.layer) || !includedNet(c.net) ||
               (netOnly_ && net_!="All nets" && c.net=="-")) continue;
            const QColor base(c.layer==1 ? "#e6aa55" :
                              c.layer==layers_ ? "#509fca" : "#8fb66f");
            QColor color(base);
            if(net_!="All nets" && net_!=c.net) color.setAlpha(40);
            if(copperGroups.empty() || copperGroups.back().first!=color.rgba())
                copperGroups.emplace_back(color.rgba(),Vertices{});
            auto &v=copperGroups.back().second;
            if(c.circle) {
                const auto center=pixel(world(c.cx_nm,c.cy_nm));
                const qreal radius=c.radius_nm/1e6*px;
                if(!inView(center,radius,width(),height())) continue;
                const int steps=std::clamp(static_cast<int>(
                    std::ceil(3.14159265358979323846 *
                              std::sqrt(std::max<qreal>(1,radius)))),24,256);
                disk(v,center,radius,steps);
            } else if(!c.hollowTriangles.empty()) {
                for(const auto &tri:c.hollowTriangles) {
                    auto point=[&](std::pair<qint64,qint64> p) {
                        return pixel(world(p.first,p.second));
                    };
                    const auto a=point(tri[0]),b=point(tri[1]),p2=point(tri[2]);
                    if(QRectF(a,b).normalized().united(QRectF(b,p2).normalized())
                        .united(QRectF(p2,a).normalized())
                        .intersects(QRectF(0,0,width(),height())))
                        triangle(v,a,b,p2);
                }
            } else {
                for(const auto &tri:c.triangles) {
                    auto point=[&](int n){
                        const auto p=c.ring[static_cast<size_t>(n)];
                        return pixel(world(p.first,p.second));
                    };
                    const auto a=point(tri[0]),b=point(tri[1]),p2=point(tri[2]);
                    if(QRectF(a,b).normalized().united(QRectF(b,p2).normalized())
                        .united(QRectF(p2,a).normalized())
                        .intersects(QRectF(0,0,width(),height())))
                        triangle(v,a,b,p2);
                }
            }
        }
        // POUROUT is a declared unfilled boundary. Batch on RHI; never
        // tessellate the polygon interior into invented copper triangles.
        for(const auto &e:pours_) {
            if(!visible(e.layer) || !includedNet(e.net) ||
               (netOnly_ && net_!="All nets" && e.net=="-"))
                continue;
            const QPointF a=pixel(world(e.x0_nm,e.y0_nm));
            const QPointF b=pixel(world(e.x1_nm,e.y1_nm));
            if(!QRectF(a,b).normalized().adjusted(-4,-4,4,4)
                   .intersects(QRectF(0,0,width(),height()))) continue;
            QColor color("#8be1c6");
            if(net_!="All nets" && net_!=e.net) color.setAlpha(40);
            thickLine(groups[color.rgba()],a,b,0.85,deviceRatio);
        }
        // Mechanical boundaries are net- and layer-independent, never copper.
        // One batched color group per outline kind; no per-edge scene nodes.
        for (const auto &e : outlineEdges_) {
            const QPointF a=pixel(world(e.x0_nm,e.y0_nm));
            const QPointF b=pixel(world(e.x1_nm,e.y1_nm));
            if (!QRectF(a,b).normalized().adjusted(-4,-4,4,4)
                    .intersects(QRectF(0,0,width(),height()))) continue;
            const QColor color(e.cutout ? "#e99f87" : "#e0e8ee");
            thickLine(groups[color.rgba()],a,b,0.85,deviceRatio);
        }
        // Analytical mixed mechanical outline arcs. GPU chords exist only
        // as screen-space triangles; source/Canonical/IPC remain exact.
        // Bound total mixed mechanical arc tessellation across the scene.
        // Source geometry remains exact; only GPU display polygons are bounded.
        size_t mixedArcCount=0;
        for(const auto &b : boundaries_) if(b.isArc) ++mixedArcCount;
        const int maxMixedSegments=arcSegmentBudgetPerVisibleArc(mixedArcCount);
        for(const auto &b : boundaries_) {
            const QColor color(b.cutout ? "#e99f87" : "#e0e8ee");
            auto &dst=groups[color.rgba()];
            const QPointF start=pixel(world(b.x0_nm,b.y0_nm));
            const QPointF end=pixel(world(b.x1_nm,b.y1_nm));
            if(!b.isArc) {
                if(QRectF(start,end).normalized().adjusted(-4,-4,4,4)
                    .intersects(QRectF(0,0,width(),height())))
                    thickLine(dst,start,end,0.85,deviceRatio);
                continue;
            }
            const QPointF centre=pixel(world(b.cx_nm,b.cy_nm));
            const qreal rad=std::hypot(start.x()-centre.x(),start.y()-centre.y());
            if(!inView(centre,rad+2,width(),height())) continue;
            const qreal a0=std::atan2(static_cast<qreal>(b.y0_nm-b.cy_nm),
                                      static_cast<qreal>(b.x0_nm-b.cx_nm));
            const qreal sweep=(b.clockwise?-1.0:1.0)*1.57079632679489661923;
            const int steps=std::min(maxMixedSegments,std::clamp(static_cast<int>(
                std::ceil(std::abs(sweep)*std::sqrt(std::max<qreal>(1,rad)/4.0))),4,512));
            const qreal inner=std::max<qreal>(0,rad-0.85),outer=rad+0.85;
            auto at=[&](qreal angle,qreal radius) {
                return QPointF(centre.x()+radius*std::cos(angle),
                               centre.y()-radius*std::sin(angle));
            };
            for(int j=0;j<steps;++j) {
                const qreal p=a0+sweep*j/steps;
                const qreal q=a0+sweep*(j+1)/steps;
                const QPointF o0=at(p,outer),o1=at(q,outer);
                const QPointF i0=at(p,inner),i1=at(q,inner);
                triangle(dst,o0,o1,i0);triangle(dst,i0,o1,i1);
            }
        }
        // Preserve analytical circles in IPC/Canonical. The following is
        // display-only chord tessellation, bounded by the 64 shape parser cap.
        constexpr qreal pi=3.14159265358979323846;
        for (const auto &circle : circles_) {
            const QPointF c=pixel(world(circle.cx_nm,circle.cy_nm));
            const qreal r=circle.radius_nm/1e6*px;
            if (!inView(c,r+2,width(),height())) continue;
            const int steps=std::clamp(
                static_cast<int>(std::ceil(pi*std::sqrt(std::max<qreal>(1,r)))),
                24,512);
            QColor color(circle.cutout ? "#e99f87" : "#e0e8ee");
            auto &v=groups[color.rgba()];
            for(int i=0;i<steps;++i) {
                const qreal a=2*pi*i/steps, b=2*pi*(i+1)/steps;
                const QPointF p0(c.x()+r*std::cos(a),c.y()-r*std::sin(a));
                const QPointF p1(c.x()+r*std::cos(b),c.y()-r*std::sin(b));
                const QPointF chord=p1-p0;
                const qreal len=std::hypot(chord.x(),chord.y());
                if(len<=1e-9) continue;
                const QPointF perp(-chord.y()*0.85/len,chord.x()*0.85/len);
                triangle(v,p0+perp,p1+perp,p1-perp);
                triangle(v,p0+perp,p1-perp,p0-perp);
            }
        }
        size_t visibleArcCount=0;
        for (const auto &arc : arcs_) {
            if(!visible(arc.layer) || !includedNet(arc.net)) continue;
            const QPointF c=pixel(world(arc.cx_nm,arc.cy_nm));
            const qreal r=std::hypot(static_cast<qreal>(arc.x0_nm-arc.cx_nm),
                                     static_cast<qreal>(arc.y0_nm-arc.cy_nm))/1e6*px;
            if(inView(c,r+arc.width_nm/2e6*px,width(),height())) ++visibleArcCount;
        }
        const int arcSegmentsPerObject=arcSegmentBudgetPerVisibleArc(visibleArcCount);
        for (const auto &a : arcs_) {
            if (!visible(a.layer) || !includedNet(a.net)) continue;
            const QPointF c=pixel(world(a.cx_nm,a.cy_nm));
            const qreal rad=std::hypot(static_cast<qreal>(a.x0_nm-a.cx_nm),
                                       static_cast<qreal>(a.y0_nm-a.cy_nm))/1e6*px;
            const qreal halfWidth=a.width_nm/2e6*px;
            if (!inView(c,rad+halfWidth,width(),height())) continue;
            const QColor base=a.layer==1 ? QColor("#f29d55") :
                   a.layer==layers_ ? QColor("#55b4e9") : QColor("#a9d48e");
            QColor color(base);
            if(net_!="All nets" && net_!=a.net) color.setAlpha(40);
            auto &v=groups[color.rgba()];
            const auto angles=arcAngles(a);
            // Display-only bounded tessellation; Canonical Arc remains exact.
            const int steps=std::min(arcSegmentsPerObject,std::clamp(
                static_cast<int>(std::ceil(std::abs(angles.sweep)*
                                          std::sqrt(std::max<qreal>(1,rad)/4.0))),
                4,512));
            // Compact annular sector: six vertices per step instead of
            // thickLine's 78 (repeated end disks). Round only the actual ends.
            const qreal inner=std::max<qreal>(0,rad-halfWidth);
            const qreal outer=rad+std::max<qreal>(0.1,halfWidth);
            auto ring=[&](qreal angle,qreal radius) {
                return QPointF(c.x()+radius*std::cos(angle),
                               c.y()-radius*std::sin(angle));
            };
            for(int i=0;i<steps;++i) {
                const qreal a0=angles.start+angles.sweep*i/steps;
                const qreal a1=angles.start+angles.sweep*(i+1)/steps;
                const QPointF o0=ring(a0,outer),o1=ring(a1,outer);
                const QPointF n0=ring(a0,inner),n1=ring(a1,inner);
                triangle(v,o0,o1,n0); triangle(v,n0,o1,n1);
            }
            disk(v,pixel(world(a.x0_nm,a.y0_nm)),std::max<qreal>(0.1,halfWidth),12);
            disk(v,pixel(world(a.x1_nm,a.y1_nm)),std::max<qreal>(0.1,halfWidth),12);
        }
        for (const auto &pad : pads_) {
            if (!visible(pad.layer) || (netOnly_ && net_ != "All nets")) continue;
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
            if (!shown || !includedNet(via.net)) continue;
            const QPointF c = pixel(world(via.x_nm,via.y_nm));
            const qreal outer = via.diameter_nm / 2e6 * px;
            if (!inView(c,outer,width(),height())) continue;
            QColor color("#b98b45");
            if (net_ != "All nets" && net_ != via.net) color.setAlpha(40);
            disk(groups[color.rgba()],c,outer);
            disk(holes,c,via.drill_nm / 2e6 * px);
        }
        // Source-order-preserving filled copper batches precede line/route layers in the scene
        // graph's child order, matching CPU artwork Z=-1.
        for(auto &entry:copperGroups) {
            if(entry.second.empty() ||
               entry.second.size()>static_cast<size_t>(std::numeric_limits<int>::max())) continue;
            auto *node=new QSGGeometryNode();
            auto *geometry=new QSGGeometry(QSGGeometry::defaultAttributes_Point2D(),
                                          static_cast<int>(entry.second.size()));
            geometry->setDrawingMode(QSGGeometry::DrawTriangles);
            auto *dst=geometry->vertexDataAsPoint2D();
            for(size_t i=0;i<entry.second.size();++i)
                dst[i].set(static_cast<float>(entry.second[i].x()),
                           static_cast<float>(entry.second[i].y()));
            auto *material=new QSGFlatColorMaterial();
            material->setColor(QColor::fromRgba(entry.first));
            node->setGeometry(geometry);node->setMaterial(material);
            node->setFlag(QSGNode::OwnsGeometry);
            node->setFlag(QSGNode::OwnsMaterial);
            root->appendChildNode(node);
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
        pan_ += delta; pressed_ = event->position(); requestSceneUpdate(); event->accept();
    }
    void mouseReleaseEvent(QMouseEvent *event) override {
        if (!dragging_ && picked_) {
            // Screen-space picking is independent of float GPU coordinates.
            for (size_t i=0;i<pads_.size();++i) {
                const auto &p=pads_[i];
                if (!visible(p.layer) || (netOnly_ && net_ != "All nets")) continue;
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
                if(!shown || !includedNet(v.net)) continue;
                const QPointF c=pixel(world(v.x_nm,v.y_nm));
                if (std::hypot(event->position().x()-c.x(),event->position().y()-c.y()) <=
                    v.diameter_nm / 2e6 * scaleFactor()+3) {
                    picked_(1,static_cast<int>(i)); event->accept(); return;
                }
            }
            // Mouse input and the native smoke use the same RHI arc hit-test.
            const int arcIndex=arcIndexAt(event->position());
            if(arcIndex>=0) {
                picked_(4,arcIndex); event->accept(); return;
            }
            const int circleIndex=circleIndexAt(event->position());
            if(circleIndex>=0) {
                picked_(5,circleIndex); event->accept(); return;
            }
            const int boundaryIndex=boundaryIndexAt(event->position());
            if(boundaryIndex>=0) {
                picked_(6,boundaryIndex);event->accept();return;
            }
            const int pourIndex=pourIndexAt(event->position());
            if(pourIndex>=0) {
                picked_(8,pourIndex);event->accept();return;
            }
            const int outlineIndex=outlineIndexAt(event->position());
            if(outlineIndex>=0) {
                picked_(7,outlineIndex); event->accept(); return;
            }
            const int pickedTrack=trackIndexAt(event->position());
            if(pickedTrack>=0) {
                picked_(3,pickedTrack);event->accept();return;
            }
            const int copperIndex=copperIndexAt(event->position());
            if(copperIndex>=0) {
                picked_(9,copperIndex);event->accept();return;
            }
        }
        event->accept();
    }
    void wheelEvent(QWheelEvent *event) override {
        const qreal factor=event->angleDelta().y()>0?1.2:1.0/1.2;
        if (zoom_*factor >= 0.03 && zoom_*factor <= 120.0)
            zoom_*=factor;
        requestSceneUpdate(); event->accept();
    }
public:
    explicit BoardSiRhiItem(QQuickItem *parent=nullptr) : QQuickItem(parent) {
        setFlag(ItemHasContents,true);
        setAcceptedMouseButtons(Qt::LeftButton);
    }
    void onPicked(std::function<void(int,int)> callback) { picked_=std::move(callback); }
    void load(int layers,const std::vector<ImportedTrack> &tracks,
              const std::vector<ImportedVia> &vias,const std::vector<ImportedPad> &pads,
              const std::vector<ImportedArc> &arcs = {},
              const std::vector<ImportedOutlineEdge> &outlines = {},
              const std::vector<ImportedCircle> &circles = {},
               const std::vector<ImportedBoundarySegment> &boundary = {},
               const std::vector<ImportedPourEdge> &pours = {},
               const std::vector<ImportedCopperArea> &copper = {}) {
        tracks_=tracks; vias_=vias; pads_=pads; arcs_=arcs;
        outlineEdges_=outlines; circles_=circles; boundaries_=boundary; pours_=pours; copper_=copper; layers_=layers;
        adapterProof_->store(0,std::memory_order_release);
        visibility_=QVector<bool>(layers+1,true);
        qint64 x=0,y=0; bool initialized=false;
        auto ingest=[&](qint64 a,qint64 b) {
            if (!initialized) { x=a;y=b;initialized=true; }
            else { x=std::min(x,a);y=std::min(y,b); }
        };
        for (const auto &t:tracks_) { ingest(t.x0_nm,t.y0_nm);ingest(t.x1_nm,t.y1_nm); }
        for (const auto &a:arcs_) {
            ingest(a.x0_nm,a.y0_nm); ingest(a.x1_nm,a.y1_nm); ingest(a.cx_nm,a.cy_nm);
        }
        for (const auto &v:vias_) ingest(v.x_nm,v.y_nm);
        for (const auto &p:pads_) ingest(p.x_nm,p.y_nm);
        for (const auto &e:outlineEdges_) {
            ingest(e.x0_nm,e.y0_nm);ingest(e.x1_nm,e.y1_nm);
        }
        for(const auto &e:pours_) {
            ingest(e.x0_nm,e.y0_nm);ingest(e.x1_nm,e.y1_nm);
        }
        for(const auto &c:copper_) {
            if(c.circle) ingest(c.cx_nm-c.radius_nm,c.cy_nm-c.radius_nm);
            else for(const auto &p:c.ring) ingest(p.first,p.second);
        }
        for (const auto &c:circles_) ingest(c.cx_nm-c.radius_nm,c.cy_nm-c.radius_nm);
        for(const auto &b:boundaries_) {ingest(b.x0_nm,b.y0_nm);ingest(b.x1_nm,b.y1_nm);}
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
        for(const auto &a:arcs_) {
            constexpr qreal halfPi=1.57079632679489661923;
            const qreal rad=std::hypot(static_cast<qreal>(a.x0_nm-a.cx_nm),
                                       static_cast<qreal>(a.y0_nm-a.cy_nm))/1e6;
            const qreal halfWidth=a.width_nm/2e6;
            const QPointF centre=world(a.cx_nm,a.cy_nm);
            add(world(a.x0_nm,a.y0_nm),halfWidth);
            add(world(a.x1_nm,a.y1_nm),halfWidth);
            const auto angles=arcAngles(a);
            for(int k=0;k<4;++k) {
                const qreal phi=k*halfPi;
                if(angleOnSweep(angles,phi))
                    add(QPointF(centre.x()+rad*std::cos(phi),
                                centre.y()-rad*std::sin(phi)),halfWidth);
            }
        }
        for(const auto &v:vias_) add(world(v.x_nm,v.y_nm),v.diameter_nm/2e6);
        for(const auto &p:pads_) {
            const QPointF c=world(p.x_nm,p.y_nm);
            add(c,std::max(p.width_nm,p.height_nm)/2e6);
        }
        for (const auto &e:outlineEdges_) {
            // QRectF::united ignores a zero-area rectangle. Use a tiny
            // display-only non-null extent so outline-only boards and
            // boundaries wider than the copper contribute to Fit.
            constexpr qreal outlineExtentMm=0.000001;
            add(world(e.x0_nm,e.y0_nm),outlineExtentMm);
            add(world(e.x1_nm,e.y1_nm),outlineExtentMm);
        }
        for(const auto &e:pours_) {
            add(world(e.x0_nm,e.y0_nm),0.000001);
            add(world(e.x1_nm,e.y1_nm),0.000001);
        }
        for(const auto &c:copper_) {
            if(c.circle) add(world(c.cx_nm,c.cy_nm),c.radius_nm/1e6);
            else for(const auto &p:c.ring) add(world(p.first,p.second),0.000001);
        }
        for (const auto &c:circles_)
            add(world(c.cx_nm,c.cy_nm),c.radius_nm/1e6);
        for(const auto &b:boundaries_) {
            add(world(b.x0_nm,b.y0_nm),0.000001);
            add(world(b.x1_nm,b.y1_nm),0.000001);
        }
        bounds_=rect.adjusted(-1,-1,1,1);
        rebuildTrackBins();
        pan_=QPointF();zoom_=1.0;
        modelGeneration_.fetch_add(1,std::memory_order_release);
        requestSceneUpdate();
    }
    void filter(QVector<bool> layers,const QString &net,bool onlySelectedNet=false) {
        visibility_=std::move(layers); net_=net; netOnly_=onlySelectedNet;
        requestSceneUpdate();
    }
    void fit() { pan_=QPointF();zoom_=1.0;requestSceneUpdate(); }
    void zoom(qreal factor) {
        if(zoom_*factor >= 0.03 && zoom_*factor <= 120.0) {
            zoom_*=factor; requestSceneUpdate();
        }
    }
    quint64 requestedSceneRevision() const {
        return sceneRevision_.load(std::memory_order_acquire);
    }
    quint64 syncedSceneRevision() const {
        return lastSyncedSceneRevision_.load(std::memory_order_acquire);
    }
    quint64 requestFreshFrame() {
        requestSceneUpdate();
        return requestedSceneRevision();
    }
    int primitiveCount() const {
        return static_cast<int>(tracks_.size()+vias_.size()+pads_.size()+arcs_.size()+
                                outlineEdges_.size()+circles_.size()+boundaries_.size()+pours_.size()+copper_.size());
    }
    int copperCountForTest() const {return static_cast<int>(copper_.size());}
    bool dispatchCopperPickForTest(int index) const {
        if(index<0 || index>=static_cast<int>(copper_.size())) return false;
        const auto &c=copper_[static_cast<size_t>(index)];
        if(c.circle) return copperIndexAt(pixel(world(c.cx_nm,c.cy_nm)))==index;
        // Test centroid of an actually filled triangle, excluding hollow voids.
        if(!c.hollowTriangles.empty()) {
            const auto &t=c.hollowTriangles.front();
            QPointF centroid;
            for(const auto &p:t)centroid+=pixel(world(p.first,p.second))/3.0;
            return copperIndexAt(centroid)==index;
        }
        if(c.triangles.empty()) return false;
        const auto &t=c.triangles.front();
        QPointF centroid;
        for(const int j:t) {
            const auto p=c.ring[static_cast<size_t>(j)];
            centroid+=pixel(world(p.first,p.second))/3.0;
        }
        return copperIndexAt(centroid)==index;
    }
    // A_024: search an exact source-grid cell within an authored void.
    // The old hole-bounding-box centre is not necessarily inside an L-hole.
    bool rejectCopperHolePickForTest(int index) const {
        if(index<0 || index>=static_cast<int>(copper_.size()))return false;
        const auto &c=copper_[static_cast<size_t>(index)];
        if(c.cutouts.empty())return false;
        const auto &h=c.cutouts.front();
        std::vector<qint64> xx,yy;
        for(const auto &p:h){xx.push_back(p.first);yy.push_back(p.second);}
        std::sort(xx.begin(),xx.end());xx.erase(std::unique(xx.begin(),xx.end()),xx.end());
        std::sort(yy.begin(),yy.end());yy.erase(std::unique(yy.begin(),yy.end()),yy.end());
        for(size_t i=1;i<xx.size();++i)for(size_t j=1;j<yy.size();++j) {
            const qint64 x=xx[i-1]+(xx[i]-xx[i-1])/2;
            const qint64 y=yy[j-1]+(yy[j]-yy[j-1])/2;
            bool inside=false;
            for(size_t k=1;k<h.size();++k) {
                const auto &p=h[k-1],&q=h[k];
                if(p.first==q.first &&
                    y>=std::min(p.second,q.second) &&
                    y<std::max(p.second,q.second) && x<p.first)inside=!inside;
            }
            if(inside)return copperIndexAt(pixel(world(x,y)))==-1;
        }
        return false;
    }
    bool rejectCopperSourcePointForTest(qint64 x_nm,qint64 y_nm) const {
        return copperIndexAt(pixel(world(x_nm,y_nm)))==-1;
    }
    int outlineCountForTest() const { return static_cast<int>(outlineEdges_.size()); }
    int circleCountForTest() const { return static_cast<int>(circles_.size()); }
    int boundaryCountForTest() const { return static_cast<int>(boundaries_.size()); }
    bool boundaryFitBoundsForTest() const {
        for(const auto &b:boundaries_) {
            if(!bounds_.contains(world(b.x0_nm,b.y0_nm)) ||
               !bounds_.contains(world(b.x1_nm,b.y1_nm))) return false;
        }
        return !boundaries_.empty();
    }
    bool dispatchBoundaryPickForTest(int index) {
        if(!picked_ || width()<2 || height()<2 ||
           index<0 || index>=static_cast<int>(boundaries_.size())) return false;
        const auto &b=boundaries_[static_cast<size_t>(index)];
        QPointF p=pixel(world(b.x0_nm,b.y0_nm));
        if(b.isArc) {
            const QPointF c=pixel(world(b.cx_nm,b.cy_nm));
            const QPointF arcDelta=p-c;
            const qreal angle=std::atan2(-arcDelta.y(),arcDelta.x())+
                (b.clockwise?-1.0:1.0)*3.14159265358979323846/4.0;
            const qreal radius=std::hypot(arcDelta.x(),arcDelta.y());
            p=QPointF(c.x()+radius*std::cos(angle),
                      c.y()-radius*std::sin(angle));
        } else {
            p=(p+pixel(world(b.x1_nm,b.y1_nm)))/2;
        }
        const bool ok=boundaryIndexAt(p)==index;
        if(ok) picked_(6,index);
        return ok;
    }

    bool circleFitBoundsForTest() const {
        for (const auto &c:circles_) {
            const QPointF centre=world(c.cx_nm,c.cy_nm);
            const qreal r=c.radius_nm/1e6;
            for (QPointF p : {QPointF(centre.x()-r,centre.y()),
                              QPointF(centre.x()+r,centre.y()),
                              QPointF(centre.x(),centre.y()-r),
                              QPointF(centre.x(),centre.y()+r)})
                if(!bounds_.contains(p)) return false;
        }
        return true;
    }
    bool dispatchCirclePickForTest(int index) {
        if(!picked_ || width()<2 || height()<2 ||
           index<0 || index>=static_cast<int>(circles_.size())) return false;
        const auto &c=circles_[static_cast<size_t>(index)];
        const QPointF center=pixel(world(c.cx_nm,c.cy_nm));
        const QPointF edge(center.x()+c.radius_nm/1e6*scaleFactor(),center.y());
        if(circleIndexAt(edge)!=index) return false;
        picked_(5,index);
        return true;
    }
    bool outlineFitBoundsForTest() const {
        for (const auto &e : outlineEdges_) {
            if (!bounds_.contains(world(e.x0_nm,e.y0_nm)) ||
                !bounds_.contains(world(e.x1_nm,e.y1_nm))) return false;
        }
        return true;
    }
    bool containsExactOutlineForTest(bool cutout, qint64 x0, qint64 y0,
                                     qint64 x1, qint64 y1) const {
        for (const auto &e : outlineEdges_) {
            if (e.cutout==cutout && e.x0_nm==x0 && e.y0_nm==y0 &&
                e.x1_nm==x1 && e.y1_nm==y1) return true;
        }
        return false;
    }
    // Native test seams exercise the same sweep used by GPU tessellation and
    // the same arcIndexAt path used by real mouseReleaseEvent.
    bool arcSweepForTest(int index, bool clockwise) const {
        if(index<0 || index>=static_cast<int>(arcs_.size())) return false;
        const auto &a=arcs_[static_cast<size_t>(index)];
        const auto sweep=arcAngles(a).sweep;
        constexpr qreal twoPi=6.28318530717958647692;
        return a.clockwise==clockwise && std::isfinite(sweep) &&
               (clockwise ? sweep<0 : sweep>0) &&
               std::abs(sweep)>0.001 && std::abs(sweep)<twoPi;
    }
    bool dispatchArcPickForTest(int index) {
        if(!picked_ || width()<2 || height()<2 ||
           index<0 || index>=static_cast<int>(arcs_.size())) return false;
        const auto &a=arcs_[static_cast<size_t>(index)];
        if(!visible(a.layer) || !includedNet(a.net)) return false;
        const auto angles=arcAngles(a);
        const QPointF centre=pixel(world(a.cx_nm,a.cy_nm));
        const qreal radius=std::hypot(
            static_cast<qreal>(a.x0_nm-a.cx_nm),
            static_cast<qreal>(a.y0_nm-a.cy_nm))/1e6*scaleFactor();
        const qreal middle=angles.start+angles.sweep/2;
        const QPointF point(centre.x()+radius*std::cos(middle),
                            centre.y()-radius*std::sin(middle));
        if(arcIndexAt(point)!=index) return false;
        picked_(4,index);
        return true;
    }

    // The native GPU path and this test seam share the same hit test.
    bool dispatchOutlinePickForTest(int index) {
        if(!picked_ || width()<2 || height()<2 ||
           index<0 || index>=static_cast<int>(outlineEdges_.size())) return false;
        const auto &e=outlineEdges_[static_cast<size_t>(index)];
        const QPointF a=pixel(world(e.x0_nm,e.y0_nm));
        const QPointF b=pixel(world(e.x1_nm,e.y1_nm));
        if(outlineIndexAt((a+b)/2)!=index) return false;
        picked_(7,index);
        return true;
    }
    bool arcBudgetBoundedForTest() const {
        // 10k near-full arcs cannot produce an unbounded GPU vertex upload.
        const int perArc=arcSegmentBudgetPerVisibleArc(10000);
        const size_t arcVertexLimit=static_cast<size_t>(10000) *
                     (static_cast<size_t>(perArc)*6+72);
        return perArc>=4 && perArc<=22 && arcVertexLimit<=2100000;
    }
    bool arcFitBoundsForTest(int index) const {
        if(index<0 || index>=static_cast<int>(arcs_.size()))return false;
        const auto &a=arcs_[static_cast<size_t>(index)];
        const QPointF start=world(a.x0_nm,a.y0_nm);
        const QPointF end=world(a.x1_nm,a.y1_nm);
        return bounds_.contains(start) && bounds_.contains(end);
    }
    // 1: D3D11 non-software adapter; -1: WARP; -2: unavailable;
    // 0: pending; 2: platform other than Windows.
    int d3dAdapterProof() const {
        return adapterProof_->load(std::memory_order_acquire);
    }
    quint64 sourceGeneration() const {
        return modelGeneration_.load(std::memory_order_acquire);
    }
    quint64 syncedGeneration() const {
        return lastSyncedGeneration_.load(std::memory_order_acquire);
    }
    QPointF localPoint(qint64 x,qint64 y) const { return world(x,y); }
    qreal projectedHalfWidthForTest(qint64 sourceWidthNm) const {
        return sourceWidthNm / 2e6 * scaleFactor();
    }
    int lineVertexCountForTest(qreal halfWidthLogicalPx,qreal devicePixelRatio=1.0) const {
        return nonDegenerateLineVerticesForTest(halfWidthLogicalPx,devicePixelRatio);
    }
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
        for(const auto &p:pads_) if(visible(p.layer) && (!netOnly_ || net_=="All nets")) ++n;
        return n;
    }
    QPointF screenPointForTest(qint64 x,qint64 y) const {
        return pixel(world(x,y));
    }
    int trackAtForTest(QPointF screen) const {return trackIndexAt(screen);}
    size_t indexBucketsForTest() const {return trackBins_.size();}
    size_t spanningTracksForTest() const {return spanningTracks_.size();}
    void panByForTest(QPointF delta) {pan_+=delta;requestSceneUpdate();}
};

class BoardSiRhiView final : public QWidget {
    QQuickWindow *window_=nullptr;
    BoardSiRhiItem *artwork_=nullptr;
    QString status_="RHI_PENDING_FRAME_HW_NOT_VERIFIED";
    std::function<void()> changed_;
    bool seenFrame_=false;
    quint64 loadGeneration_=0;
    quint64 requiredSceneRevision_=0;
    quint64 deadlineGeneration_=std::numeric_limits<quint64>::max();
    quint64 deadlineEpoch_=0;
    std::atomic<bool> visibleDuringFrame_{false};

    void enterCpuFallback(const QString &reason) {
        if(failed()) return;
        ++deadlineEpoch_;
        status_="CPU_FALLBACK_RHI_"+reason;
        if(changed_) changed_();
    }
    void armFrameDeadline() {
        if (!isVisible() || seenFrame_ || failed() ||
            deadlineGeneration_ == loadGeneration_) return;
        const quint64 watched = loadGeneration_;
        const quint64 epoch = ++deadlineEpoch_;
        deadlineGeneration_=watched;
        QTimer::singleShot(3000,this,[this,watched,epoch] {
            if (epoch != deadlineEpoch_ || watched != loadGeneration_ ||
                seenFrame_ || failed()) return;
            if (!isVisible()) {
                // Re-arm after hidden tab is shown, not indefinitely pending.
                deadlineGeneration_=std::numeric_limits<quint64>::max();
                return;
            }
            enterCpuFallback("NO_FRAME");
        });
    }
protected:
    void showEvent(QShowEvent *event) override {
        QWidget::showEvent(event);
        visibleDuringFrame_.store(true,std::memory_order_release);
        // A previously presented frame cannot certify this tab activation.
        // Hidden-tab device-loss races must therefore produce a new frame or
        // hit the normal bounded CPU fallback deadline on re-exposure.
        if(!failed()) {
            seenFrame_=false;
            status_="RHI_PENDING_FRAME_HW_NOT_VERIFIED";
            requiredSceneRevision_=artwork_->requestFreshFrame();
            deadlineGeneration_=std::numeric_limits<quint64>::max();
            ++deadlineEpoch_;
            armFrameDeadline();
        }
    }
    void hideEvent(QHideEvent *event) override {
        visibleDuringFrame_.store(false,std::memory_order_release);
        QWidget::hideEvent(event);
        // Old timeouts were for a previously visible window. They cannot
        // consume time while hidden or block the next showEvent deadline.
        ++deadlineEpoch_;
        deadlineGeneration_=std::numeric_limits<quint64>::max();
        if(!failed()) {
            seenFrame_=false;
            status_="RHI_PENDING_FRAME_HW_NOT_VERIFIED";
        }
    }
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
            // Direct signal captures render-thread generation BEFORE queuing
            // delivery; an older frame cannot verify newly loaded artwork.
            const quint64 presented=artwork_->syncedGeneration();
            const quint64 revision=artwork_->syncedSceneRevision();
            QMetaObject::invokeMethod(this,[this,presented,revision] {
            if(presented != loadGeneration_ || revision < requiredSceneRevision_ ||
               seenFrame_ || failed() || !isVisible()) return;
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
            } else if (backend=="DIRECT3D11") {
                const int proof=artwork_->d3dAdapterProof();
                if (proof==1)
                    status_="RHI_FRAME_DIRECT3D11_HARDWARE_CONFIRMED";
                else if (proof==-1)
                    status_="CPU_FALLBACK_RHI_SOFTWARE_D3D11_ADAPTER";
                else
                    status_="CPU_FALLBACK_RHI_D3D11_ADAPTER_NOT_VERIFIED";
            } else if (backend=="METAL") {
#ifdef Q_OS_MACOS
                const int proof=artwork_->d3dAdapterProof();
                status_=proof==1 ? "RHI_FRAME_METAL_HARDWARE_CONFIRMED" :
                                   "RHI_FRAME_METAL_HARDWARE_NOT_VERIFIED";
#else
                status_="RHI_FRAME_METAL_HARDWARE_NOT_VERIFIED";
#endif
            } else {
                status_="RHI_FRAME_"+backend+"_HARDWARE_NOT_VERIFIED";
            }
            if(changed_) changed_();
            },Qt::QueuedConnection);
        },Qt::DirectConnection);
        QObject::connect(window_,&QQuickWindow::sceneGraphError,this,
                         [this](QQuickWindow::SceneGraphError,const QString &) {
            enterCpuFallback("SCENEGRAPH_ERROR");
        },Qt::QueuedConnection);
        // Device/scene graph invalidation arrives from the render thread.
        // Hidden or not-yet-presented tabs are NOT automatically device loss.
        // Always marshal the final UI fallback to the GUI event loop.
        QObject::connect(window_,&QQuickWindow::sceneGraphInvalidated,window_,
                         [this] {
            const bool visibleAtSignal=visibleDuringFrame_.load(std::memory_order_acquire);
            QMetaObject::invokeMethod(this,[this,visibleAtSignal] {
                if(!visibleAtSignal || failed()) return;
                if(!isVisible()) {
                    // Preserve invalidation across a concurrent hide; on show
                    // an explicitly new revision and deadline are mandatory.
                    seenFrame_=false;
                    status_="RHI_PENDING_FRAME_HW_NOT_VERIFIED";
                    ++deadlineEpoch_;
                    deadlineGeneration_=std::numeric_limits<quint64>::max();
                } else if(seenFrame_) {
                    enterCpuFallback("SCENEGRAPH_INVALIDATED");
                }
            },Qt::QueuedConnection);
        },Qt::DirectConnection);
        auto *container=QWidget::createWindowContainer(window_,this);
        container->setMinimumHeight(220);
        layout->addWidget(container);
    }
    void onChanged(std::function<void()> callback) {changed_=std::move(callback);}
    void onPicked(std::function<void(int,int)> callback) {artwork_->onPicked(std::move(callback));}
    void load(int layers,const std::vector<ImportedTrack> &tracks,
              const std::vector<ImportedVia> &vias,const std::vector<ImportedPad> &pads,
              const std::vector<ImportedArc> &arcs = {},
              const std::vector<ImportedOutlineEdge> &outlines = {},
              const std::vector<ImportedCircle> &circles = {},
               const std::vector<ImportedBoundarySegment> &boundary = {},
               const std::vector<ImportedPourEdge> &pours = {},
               const std::vector<ImportedCopperArea> &copper = {}) {
        seenFrame_=false;
        status_="RHI_PENDING_FRAME_HW_NOT_VERIFIED";
        artwork_->load(layers,tracks,vias,pads,arcs,outlines,circles,boundary,pours,copper);
        loadGeneration_=artwork_->sourceGeneration();
        requiredSceneRevision_=artwork_->requestedSceneRevision();
        deadlineGeneration_=std::numeric_limits<quint64>::max();
        ++deadlineEpoch_;
        armFrameDeadline();
        if (changed_) changed_();
    }
    void filter(QVector<bool> layers,const QString &net,bool onlySelectedNet=false) {artwork_->filter(std::move(layers),net,onlySelectedNet);}
    void fit() {artwork_->fit();}
    void zoom(qreal factor) {artwork_->zoom(factor);}
    QQuickWindow *quickWindowForTest() const {return window_;}
    // Fault injection drives precisely the same fallback code, but does not
    // claim to emulate a hardware driver reset on hosted CI.
    void injectSceneGraphFailureForTest() {enterCpuFallback("SCENEGRAPH_ERROR");}
    QString status() const {return status_;}
    bool failed() const {return status_.startsWith("CPU_FALLBACK");}
    int primitiveCount() const {return artwork_->primitiveCount();}
    BoardSiRhiItem *itemForTest() const {return artwork_;}
};
