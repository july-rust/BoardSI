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
#include <QLineEdit>
#include <QCheckBox>
#include <QPushButton>
#include <QSet>
#include <QSignalBlocker>
#include <QStackedWidget>
#include <QVector>
#include <QVBoxLayout>
#include <limits>
#include <vector>
#include <array>
#include <cstddef>
#include <algorithm>
#include <set>
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

struct ImportedCircle {
    QString name;
    bool cutout = false;
    qint64 cx_nm = 0, cy_nm = 0, radius_nm = 0;
};

struct ImportedOutlineEdge {
    QString name;
    bool cutout = false;
    int sequence = 0;
    qint64 x0_nm = 0, y0_nm = 0, x1_nm = 0, y1_nm = 0;
};

struct ImportedBoundarySegment {
    QString name;
    bool cutout = false;
    bool isArc = false;
    bool clockwise = false;
    int sequence = 0;
    qint64 x0_nm = 0, y0_nm = 0, cx_nm = 0, cy_nm = 0;
    qint64 x1_nm = 0, y1_nm = 0;
};

struct ImportedPourEdge { QString name,net; int layer=0,sequence=0,sourceLine=0; qint64 x0_nm=0,y0_nm=0,x1_nm=0,y1_nm=0; };

// Exact-grid filled COPCLS/COPCIR source shapes, not reconstructed flooded copper.
// Triangle indices are CPU-validated and reused by the RHI GPU batch.
struct ImportedCopperCircleHole {
    qint64 cx_nm=0, cy_nm=0, radius_nm=0; // analytic source circle
};
struct ImportedCopperArea {
    QString name, net;
    int layer=0, sourceLine=0;
    bool circle=false;
    bool curved=false; // A_029: exact original Arc/Line source, display proxy separate.
    std::vector<ImportedBoundarySegment> curvedSegments;
    qint64 cx_nm=0,cy_nm=0,radius_nm=0;
    std::vector<std::pair<qint64,qint64>> ring;
    std::vector<std::array<int,3>> triangles;
    std::vector<std::vector<std::pair<qint64,qint64>>> cutouts;
    std::vector<ImportedCopperCircleHole> roundCutouts;
    // Exact-grid tiling for validated axis-aligned rectangular holes.
    // Consumed unchanged by source-aware RHI rendering and picking.
    std::vector<std::array<std::pair<qint64,qint64>,3>> hollowTriangles;
};
#include "a025_diagonal_polygon.h"
#include "a026_circular_copper.h"
#include "a029_curved_copper.h"
#include "a030_curved_cutout.h"
inline bool validateCopperArea(ImportedCopperArea &a) {
    constexpr qint64 bound=1422400000LL;
    if(a.layer<1 || a.layer>64 || a.sourceLine<=0 ||
       a.name.isEmpty() || a.name.size()>16 || a.net.isEmpty() || a.net.size()>47)
        return false;
    auto coord=[&](qint64 v) {return v%254==0 && v>=-bound && v<=bound;};
    a.triangles.clear();
    a.hollowTriangles.clear();
    if(a.curved)return a.cutouts.empty() ? a029::validate(a) : a030::triangulate(a);
    if(a.circle) {
        // A_027: V12 preserves COPCIR/COPCCO analytic circles while the
        // display-only hollow mesh must conservatively fit source geometry.
        if(!a.ring.empty() || !coord(a.cx_nm) || !coord(a.cy_nm) ||
           !coord(a.radius_nm) || a.radius_nm<=0) return false;
        for(const auto v:{a.cx_nm-a.radius_nm,a.cx_nm+a.radius_nm,
                          a.cy_nm-a.radius_nm,a.cy_nm+a.radius_nm})
            if(!coord(v)) return false;
        // A_028 mixed COPCUT + COPCCO uses the same A_025 verified
        // triangle mesh for selection, CPU fill and RHI draw/picking.
        return (a.cutouts.empty() && a.roundCutouts.empty()) ||
               a026::triangulate(a);
    }
    if(a.ring.size()<4 || a.ring.size()>256 || a.ring.front()!=a.ring.back()) return false;
    const int n=static_cast<int>(a.ring.size())-1;
    for(auto p:a.ring) if(!coord(p.first) || !coord(p.second)) return false;
    if(!a.roundCutouts.empty())return a026::triangulate(a);
    if(!a.cutouts.empty()) {
        // A_025: preserve the exact A_024 orthogonal grid path, and use
        // separate bounded source-vertex triangulation only for diagonal rings.
        auto hasDiagonal=[](const std::vector<std::pair<qint64,qint64>>&ring) {
            for(size_t i=1;i<ring.size();++i)
                if(ring[i-1].first!=ring[i].first &&
                   ring[i-1].second!=ring[i].second)return true;
            return false;
        };
        bool diagonal=hasDiagonal(a.ring);
        for(const auto &hole:a.cutouts)diagonal=diagonal||hasDiagonal(hole);
        if(diagonal)return a025::triangulate(a);
        // A_024: validated *orthogonal* polygon rings. The mesh contains only
        // exact 254-nm-grid coordinates: each grid cell lies wholly in or out
        // of the original source-authored shape. No diagonal interpolation.
        using P=std::pair<qint64,qint64>;
        auto overlap=[](qint64 a,qint64 b,qint64 c,qint64 d) {
            return std::max(std::min(a,b),std::min(c,d))<=
                   std::min(std::max(a,b),std::max(c,d));
        };
        auto touches=[&](P a,P b,P c,P d) {
            const bool av=a.first==b.first,cv=c.first==d.first;
            if(av&&cv)return a.first==c.first &&
                overlap(a.second,b.second,c.second,d.second);
            if(!av&&!cv)return a.second==c.second &&
                overlap(a.first,b.first,c.first,d.first);
            if(av)return overlap(a.second,b.second,c.second,d.second) &&
                overlap(c.first,d.first,a.first,a.first);
            return overlap(c.second,d.second,a.second,a.second) &&
                overlap(a.first,b.first,c.first,c.first);
        };
        auto area2=[](const std::vector<P> &ring)->qint64 {
            qint64 value=0;
            for(size_t i=1;i<ring.size();++i)
                value+=(ring[i-1].first/254)*(ring[i].second/254)-
                       (ring[i].first/254)*(ring[i-1].second/254);
            return value<0?-value:value;
        };
        auto orthogonal=[&](const std::vector<P> &ring)->bool {
            if(ring.size()<5||ring.size()>65||ring.front()!=ring.back())return false;
            for(size_t i=1;i<ring.size();++i) {
                if(!coord(ring[i].first)||!coord(ring[i].second))return false;
                const P a=ring[i-1],b=ring[i];
                if((a.first==b.first)==(a.second==b.second))return false;
            }
            for(size_t i=1;i<ring.size();++i)
                for(size_t j=i+2;j<ring.size();++j) {
                    if(i==1 && j==ring.size()-1)continue;
                    if(touches(ring[i-1],ring[i],ring[j-1],ring[j]))
                        return false;
                }
            return area2(ring)>0;
        };
        auto inside=[](P p,const std::vector<P>&ring) {
            bool found=false;
            for(size_t i=1;i<ring.size();++i) {
                const P x=ring[i-1],y=ring[i];
                if(x.first==y.first &&
                   p.second>=std::min(x.second,y.second) &&
                   p.second<std::max(x.second,y.second) &&
                   p.first<x.first)found=!found;
            }
            return found;
        };
        auto ringTouches=[&](const std::vector<P>&x,const std::vector<P>&y) {
            for(size_t i=1;i<x.size();++i)
                for(size_t j=1;j<y.size();++j)
                    if(touches(x[i-1],x[i],y[j-1],y[j]))return true;
            return false;
        };
        if(a.cutouts.size()>8 || !orthogonal(a.ring))return false;
        size_t total=a.ring.size();
        for(const auto &hole:a.cutouts) {
            total+=hole.size();
            if(total>128 || !orthogonal(hole) || ringTouches(a.ring,hole))
                return false;
            for(size_t i=0;i+1<hole.size();++i)
                if(!inside(hole[i],a.ring))return false;
        }
        for(size_t i=0;i<a.cutouts.size();++i)
            for(size_t j=i+1;j<a.cutouts.size();++j) {
                const auto &x=a.cutouts[i],&y=a.cutouts[j];
                if(ringTouches(x,y)||inside(x.front(),y)||inside(y.front(),x))
                    return false;
            }
        std::vector<qint64> xx,yy;
        auto collect=[&](const std::vector<P>&ring) {
            for(const P &p:ring){xx.push_back(p.first);yy.push_back(p.second);}
        };
        collect(a.ring);
        for(const auto &hole:a.cutouts)collect(hole);
        std::sort(xx.begin(),xx.end());xx.erase(std::unique(xx.begin(),xx.end()),xx.end());
        std::sort(yy.begin(),yy.end());yy.erase(std::unique(yy.begin(),yy.end()),yy.end());
        if(xx.size()<2||yy.size()<2||(xx.size()-1)*(yy.size()-1)>4096)
            return false;
        qint64 filled2=0;
        for(size_t i=1;i<xx.size();++i)
            for(size_t j=1;j<yy.size();++j) {
                const qint64 x0=xx[i-1],x1=xx[i],y0=yy[j-1],y1=yy[j];
                const P mid={x0+(x1-x0)/2,y0+(y1-y0)/2};
                if(!inside(mid,a.ring))continue;
                bool voided=false;
                for(const auto &hole:a.cutouts)
                    if(inside(mid,hole)){voided=true;break;}
                if(voided)continue;
                // The grid is an exact partition for orthogonal boundaries.
                const P p0={x0,y0},p1={x1,y0},p2={x1,y1},p3={x0,y1};
                a.hollowTriangles.push_back({p0,p1,p2});
                a.hollowTriangles.push_back({p0,p2,p3});
                filled2+=2*((x1-x0)/254)*((y1-y0)/254);
            }
        qint64 source2=area2(a.ring);
        for(const auto &hole:a.cutouts)source2-=area2(hole);
        // Exact integer area oracle: reject any hole that the mesh would fill.
        if(source2<=0||filled2!=source2||a.hollowTriangles.empty()) {
            a.hollowTriangles.clear();return false;
        }
        return true;
    }
    // Work in integer source ticks: |cross| < 2.6e14, no MSVC __int128 needed.
    auto cross=[&](int i,int j,int k)->qint64 {
        const auto &a0=a.ring[static_cast<size_t>(i)];
        const auto &b0=a.ring[static_cast<size_t>(j)];
        const auto &c0=a.ring[static_cast<size_t>(k)];
        const qint64 x=(b0.first-a0.first)/254,y=(b0.second-a0.second)/254;
        const qint64 u=(c0.first-a0.first)/254,v=(c0.second-a0.second)/254;
        return x*v-y*u;
    };
    auto onseg=[&](int i,int j,int k) {
        auto p=a.ring[static_cast<size_t>(k)];
        auto x=a.ring[static_cast<size_t>(i)],y=a.ring[static_cast<size_t>(j)];
        return cross(i,j,k)==0 &&
               p.first>=std::min(x.first,y.first) && p.first<=std::max(x.first,y.first) &&
               p.second>=std::min(x.second,y.second) && p.second<=std::max(x.second,y.second);
    };
    qint64 twiceArea=0;
    for(int i=0;i<n;++i) {
        int j=(i+1)%n;
        if(a.ring[static_cast<size_t>(i)]==a.ring[static_cast<size_t>(j)]) return false;
        const auto &x=a.ring[static_cast<size_t>(i)];
        const auto &y=a.ring[static_cast<size_t>(j)];
        twiceArea+=(x.first/254)*(y.second/254)-(y.first/254)*(x.second/254);
        for(int k=i+1;k<n;++k) {
            int l=(k+1)%n;
            if(k==i+1 || (i==0 && k==n-1)) continue;
            const auto x1=cross(i,j,k),x2=cross(i,j,l);
            const auto y1=cross(k,l,i),y2=cross(k,l,j);
            if(onseg(i,j,k)||onseg(i,j,l)||onseg(k,l,i)||onseg(k,l,j)||
               ((x1>0)!=(x2>0) && (y1>0)!=(y2>0))) return false;
        }
    }
    if(twiceArea==0) return false;
    const int winding=twiceArea>0?1:-1;
    std::vector<int> ids;
    for(int i=0;i<n;++i) ids.push_back(i);
    // Ear clipping is bounded (max 255 vertices and 64 objects). Refuse if
    // a degenerate/ambiguous ring cannot be triangulated without inventing fill.
    while(ids.size()>3) {
        bool clipped=false;
        for(size_t i=0;i<ids.size();++i) {
            const int p=ids[(i+ids.size()-1)%ids.size()];
            const int c=ids[i],q=ids[(i+1)%ids.size()];
            if(cross(p,c,q)*winding<=0) continue;
            bool occupied=false;
            for(int vertex:ids) {
                if(vertex==p||vertex==c||vertex==q) continue;
                const qint64 a1=cross(p,c,vertex)*winding;
                const qint64 a2=cross(c,q,vertex)*winding;
                const qint64 a3=cross(q,p,vertex)*winding;
                if(a1>=0 && a2>=0 && a3>=0){occupied=true;break;}
            }
            if(occupied) continue;
            a.triangles.push_back({p,c,q});
            ids.erase(ids.begin()+static_cast<std::ptrdiff_t>(i));
            clipped=true;break;
        }
        if(!clipped) return false;
    }
    if(ids.size()!=3 || cross(ids[0],ids[1],ids[2])*winding<=0) return false;
    a.triangles.push_back({ids[0],ids[1],ids[2]});
    return a.triangles.size()==static_cast<size_t>(n-2);
}
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

// Verified endpoint records are never equivalent to complete Net connectivity.
struct ImportedContactEnd { char kind='T'; int index=0,site=0,layer=0; };
struct ImportedContactMarker {
    bool barrel=false;
    ImportedContactEnd a,b;
    qint64 x_nm=0,y_nm=0;
    QString net;
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
    QLabel *explorerStatus_ = nullptr;
    QLineEdit *netSearch_ = nullptr;
    QCheckBox *netOnly_ = nullptr;
    QVector<bool> layerVisible_;
    QVector<bool> savedLayerVisibility_;
    bool hasSavedVisibility_ = false;
    std::vector<std::pair<QGraphicsLineItem *, ImportedTrack>> tracks_;
    std::vector<std::pair<QGraphicsPathItem *, ImportedArc>> arcs_;
    std::vector<std::pair<QGraphicsEllipseItem *, ImportedVia>> vias_;
    std::vector<std::pair<QGraphicsItem *, ImportedPad>> pads_;
    std::vector<std::pair<QGraphicsLineItem *, ImportedOutlineEdge>> outlineEdges_;
    std::vector<std::pair<QGraphicsLineItem *, ImportedPourEdge>> pours_;
    std::vector<std::pair<QGraphicsItem *, ImportedCopperArea>> copper_;
    std::vector<std::pair<QGraphicsEllipseItem *, ImportedCircle>> circles_;
    std::vector<std::pair<QGraphicsPathItem *, ImportedBoundarySegment>> boundaries_;
    std::function<void(const QString &)> detailsCallback_;
    int skipped_ = 0;
    std::vector<std::pair<QGraphicsEllipseItem*,ImportedContactMarker>> contacts_;
    QString localContactAudit_ = "Local endpoint contact evidence: NOT_EXPOSED; full-board connectivity NOT_VERIFIED";
    void refresh() {
#ifdef BOARDSI_RHI_ARTWORK
        if (rhi_) rhi_->filter(layerVisible_, nets_->currentText(), netOnly_ && netOnly_->isChecked());
#endif
        // A GPU backend change must never fabricate a successful PCB import.
        if (tracks_.empty() && arcs_.empty() && vias_.empty() && pads_.empty() &&
            outlineEdges_.empty() && circles_.empty() && boundaries_.empty() && pours_.empty() && copper_.empty()) {
            summary_->setText("No imported PCB. Synthetic demo is a separate tab.\n"
                              "Artwork renderer: " + rendererStatus());
            if (detailsCallback_) detailsCallback_(summary_->text());
            return;
        }
        const QString net = nets_->currentText();
        const bool netOnly = netOnly_ && netOnly_->isChecked() && net != "All nets";
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
            const bool shown = t.layer > 0 && t.layer < layerVisible_.size() && layerVisible_[t.layer] && (!netOnly || net == t.net);
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
                               layerVisible_[arc.layer] && (!netOnly || net == arc.net);
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
            const bool visibleVia = shown && (!netOnly || net == via.net);
            entry.first->setVisible(visibleVia);
            const bool active = net == "All nets" || net == via.net;
            entry.first->setOpacity(active ? 1.0 : 0.12);
            if (visibleVia) {
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
            entry.first->setVisible(shown && !netOnly);
            // Pad net ownership has not been verified. Never highlight pads
            // as electrically connected to a selected route net.
            entry.first->setOpacity(net == "All nets" ? 1.0 : 0.12);
            if (shown && !netOnly) ++visiblePads;
        }
        int visiblePours=0;
        for(const auto &entry:pours_) {
            const auto &e=entry.second;
            const bool shown=e.layer>0 && e.layer<layerVisible_.size() &&
                layerVisible_[e.layer] &&
                (!netOnly || (e.net!="-" && e.net==net));
            entry.first->setVisible(shown);
            entry.first->setOpacity(net=="All nets" || net==e.net ? 1.0 : 0.12);
            if(shown) ++visiblePours;
        }
        int visibleCopper=0;
        for(const auto &entry:copper_) {
            const auto &e=entry.second;
            const bool shown=e.layer>0 && e.layer<layerVisible_.size() &&
                layerVisible_[e.layer] && (!netOnly || (e.net!="-" && e.net==net));
            entry.first->setVisible(shown);
            entry.first->setOpacity(net=="All nets" || net==e.net ? 1.0 : 0.12);
            if(shown) ++visibleCopper;
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
                        " / analytical circles: %2 / mixed segments: %3 (artwork only; NOT electrical geometry)")
                    .arg(static_cast<qlonglong>(outlineEdges_.size()))
                    .arg(static_cast<qlonglong>(circles_.size()))
                    .arg(static_cast<qlonglong>(boundaries_.size()));
        line += QString("\nPOUROUT unfilled boundary edges: %1  Visible: %2"
                        " / actual flood and connectivity NOT_VERIFIED")
                    .arg(static_cast<qlonglong>(pours_.size())).arg(visiblePours);
        line += QString("\nExplicit source filled COPCLS/COPCIR: %1  Visible: %2"
                        " / fabricated copper and connectivity NOT_VERIFIED")
                    .arg(static_cast<qlonglong>(copper_.size())).arg(visibleCopper);
        line += QString("\nTop SMD: %1  Bottom SMD: %2")
                    .arg(static_cast<qlonglong>(pads_.size()) - bottomPads).arg(bottomPads);
        const auto selected = scene_->selectedItems();
        if (!selected.isEmpty()) {
            const auto *item = selected.first();
            const QString text = item->data(2).toString().isEmpty() ?
                                 item->data(1).toString() : item->data(2).toString();
            if (!text.isEmpty()) line += "\nSelected object: " + text;
        }
        int selectedLayerCount=0, selectedNetCount=0;
        for (const auto &e: tracks_) { if(e.second.layer==layerId) ++selectedLayerCount; if(e.second.net==net) ++selectedNetCount; }
        for (const auto &e: arcs_) { if(e.second.layer==layerId) ++selectedLayerCount; if(e.second.net==net) ++selectedNetCount; }
        for (const auto &e: vias_) { if(e.second.firstLayer<=layerId && layerId<=e.second.lastLayer) ++selectedLayerCount; if(e.second.net==net) ++selectedNetCount; }
        for (const auto &e: pads_) if(e.second.layer==layerId) ++selectedLayerCount;
        for (const auto &e: pours_) if(e.second.layer==layerId) ++selectedLayerCount;
        for (const auto &e: copper_) if(e.second.layer==layerId) ++selectedLayerCount;
        line += QString("\nLayer selected imported copper objects: %1"
                        "\nSelected net declared route/via objects: %2"
                        "\nNet connectivity: NOT_VERIFIED"
                        "\nShow selected net only: %3")
                    .arg(selectedLayerCount)
                    .arg(net=="All nets" ? QString("ALL") : QString::number(selectedNetCount))
                    .arg(netOnly ? "YES" : "NO");
        if(explorerStatus_) explorerStatus_->setText(
            QString("Layer %1: %2 objects | Net %3: %4 route/via objects | Electrical connectivity NOT_VERIFIED")
                .arg(layerId).arg(selectedLayerCount).arg(net)
                .arg(net=="All nets" ? QString("ALL") : QString::number(selectedNetCount)));
        for(const auto &entry:contacts_) {
            const auto &m=entry.second;
            const bool layerShown=(m.a.layer<layerVisible_.size() && layerVisible_[m.a.layer]) ||
                                  (m.barrel && m.b.layer<layerVisible_.size() && layerVisible_[m.b.layer]);
            entry.first->setVisible(layerShown && (!netOnly || net==m.net));
        }
        // The exact local contact audit comes from Rust and is independent
        // of CPU/RHI rendering, selection filtering and GPU picking.
        line += "\n" + localContactAudit_;
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
        all->setObjectName("importedShowAllLayers");
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

        // A_020 P2-K: explorer operations use already-imported source objects.
        auto *explorer = new QHBoxLayout();
        auto *hideAll = new QPushButton("Hide all layers", this);
        hideAll->setObjectName("importedHideAllLayers");
        auto *solo = new QPushButton("Solo layer", this);
        solo->setObjectName("importedSoloLayer");
        auto *restore = new QPushButton("Restore layers", this);
        restore->setObjectName("importedRestoreLayers");
        explorer->addWidget(hideAll);
        explorer->addWidget(solo);
        explorer->addWidget(restore);
        netSearch_ = new QLineEdit(this);
        netSearch_->setObjectName("importedNetSearch");
        netSearch_->setPlaceholderText("Find imported net (exact/unique)");
        netSearch_->setMaxLength(128);
        auto *findNet = new QPushButton("Find net", this);
        findNet->setObjectName("importedNetFind");
        explorer->addWidget(netSearch_, 2);
        explorer->addWidget(findNet);
        netOnly_ = new QCheckBox("Show selected net only", this);
        netOnly_->setObjectName("importedNetOnly");
        explorer->addWidget(netOnly_);
        layout->addLayout(explorer);
        explorerStatus_ = new QLabel("No imported copper objects / SI NOT_RUN", this);
        explorerStatus_->setObjectName("importedExplorerStatus");
        explorerStatus_->setWordWrap(true);
        layout->addWidget(explorerStatus_);
        const auto findNetName = [this] {
            const QString term = netSearch_->text().trimmed();
            if (term.isEmpty()) {
                explorerStatus_->setText("Enter an imported net name");
                return;
            }
            // PADS net labels are case-sensitive. Never silently change the
            // selected electrical label when two names differ only by case.
            int found = -1;
            for (int i=1; i<nets_->count(); ++i)
                if (nets_->itemText(i) == term) { found=i; break; }
            if (found < 0) {
                for (int i=1; i<nets_->count(); ++i) {
                    if (nets_->itemText(i).compare(term,Qt::CaseInsensitive)==0) {
                        if (found >= 0) {
                            explorerStatus_->setText("Ambiguous case-folded net; use exact case");
                            return;
                        }
                        found=i;
                    }
                }
            }
            if (found < 0) {
                for (int i=1; i<nets_->count(); ++i) {
                    if (nets_->itemText(i).contains(term,Qt::CaseInsensitive)) {
                        if (found >= 0) {
                            explorerStatus_->setText("Ambiguous partial net; use exact name");
                            return;
                        }
                        found=i;
                    }
                }
            }
            if (found < 0) {
                explorerStatus_->setText("Net not found in accepted imported objects");
                return;
            }
            nets_->setCurrentIndex(found);
            refresh();
        };
        connect(findNet,&QPushButton::clicked,this,findNetName);
        connect(netSearch_,&QLineEdit::returnPressed,this,findNetName);
        connect(hideAll,&QPushButton::clicked,this,[this] {
            for (int i=1;i<layerVisible_.size();++i) layerVisible_[i]=false;
            refresh();
        });
        connect(solo,&QPushButton::clicked,this,[this] {
            const int selected=layers_->currentData().toInt();
            if(selected<1 || selected>=layerVisible_.size()) return;
            if(!hasSavedVisibility_) {
                savedLayerVisibility_=layerVisible_;
                hasSavedVisibility_=true;
            }
            for(int i=1;i<layerVisible_.size();++i) layerVisible_[i]=(i==selected);
            refresh();
        });
        connect(restore,&QPushButton::clicked,this,[this] {
            if(!hasSavedVisibility_ ||
               savedLayerVisibility_.size()!=layerVisible_.size()) return;
            layerVisible_=savedLayerVisibility_;
            savedLayerVisibility_.clear();
            hasSavedVisibility_=false;
            refresh();
        });
        connect(netOnly_,&QCheckBox::toggled,this,[this](bool) { refresh(); });
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
                else if (kind == 5) selectCircle(index);
                else if (kind == 6) selectBoundary(index);
                else if (kind == 7) selectOutline(index);
                else if (kind == 8) selectPour(index);
                else if (kind == 9) selectCopper(index);
                else if (kind == 10 && index>=0 && index<static_cast<int>(contacts_.size())) {
                    if(detailsCallback_) detailsCallback_(contacts_[static_cast<size_t>(index)].first->data(2).toString());
                }
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
    void setLocalContacts(const std::vector<ImportedContactMarker> &markers) {
        const QSignalBlocker muteSelection(scene_);
        for(const auto &p:contacts_) { scene_->removeItem(p.first); delete p.first; }
        contacts_.clear();
        contacts_.reserve(markers.size());
        for(const auto &m:markers) {
            // ItemIgnoresTransformations gives fixed-size screen-space markers
            // without moving their exact integer-nm source anchor.
            auto *item=scene_->addEllipse(QRectF(-4,-4,8,8),
                QPen(QColor(m.barrel?"#7ae7c6":"#9be3fd"),1),
                QBrush(QColor(m.barrel?"#286952":"#266c89")));
            item->setPos(m.x_nm/1e6,-m.y_nm/1e6);
            item->setFlag(QGraphicsItem::ItemIgnoresTransformations);
            item->setFlag(QGraphicsItem::ItemIsSelectable);
            item->setZValue(10);
            item->setData(0,m.net);
            item->setData(2,QString("A_032 verified LOCAL endpoint / %1 / Net %2 / %3,%4 nm"
                " / %5[%6] site %7 L%8 <> %9[%10] site %11 L%12"
                " / complete connectivity NOT_VERIFIED")
                .arg(m.barrel?"ViaBarrel":"SameLayerContact").arg(m.net)
                .arg(m.x_nm).arg(m.y_nm)
                .arg(QChar(m.a.kind)).arg(m.a.index).arg(m.a.site).arg(m.a.layer)
                .arg(QChar(m.b.kind)).arg(m.b.index).arg(m.b.site).arg(m.b.layer));
            contacts_.push_back({item,m});
        }
#ifdef BOARDSI_RHI_ARTWORK
        if(rhi_)rhi_->setLocalContacts(markers);
#endif
        refresh();
    }
    void setLocalContactAudit(const QString &auditedText) {
        localContactAudit_ = auditedText;
        refresh();
    }
    // Starting a new import invalidates the last artwork. In particular, a failed
    // attempt must not leave old copper visible under a "NOTHING IMPORTED" status.
    void clearArtwork() {
        // Clearing selected graphics items can synchronously emit
        // selectionChanged. Prevent callbacks from iterating freed pointers.
        const QSignalBlocker muteSelection(scene_);
        contacts_.clear();
        tracks_.clear();
        arcs_.clear();
        vias_.clear();
        pads_.clear();
        outlineEdges_.clear();
        circles_.clear();
        boundaries_.clear();
        pours_.clear();
        copper_.clear();
        scene_->clear();
        layerVisible_.clear();
        layers_->clear();
        nets_->clear();
        nets_->addItem("All nets");
        if(netSearch_) netSearch_->clear();
        if(netOnly_) netOnly_->setChecked(false);
        savedLayerVisibility_.clear(); hasSavedVisibility_=false;
        if(explorerStatus_) explorerStatus_->setText("No imported copper objects / SI NOT_RUN");
        skipped_ = 0;
        localContactAudit_ = "Local endpoint contact evidence: NOT_EXPOSED; full-board connectivity NOT_VERIFIED";
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
              const std::vector<ImportedOutlineEdge> &outlineInput = {},
              const std::vector<ImportedCircle> &circleInput = {},
              const std::vector<ImportedBoundarySegment> &boundaryInput = {},
              const std::vector<ImportedPourEdge> &pourInput = {},
              const std::vector<ImportedCopperArea> &copperInput = {}) {
        if (layerCount < 1 || layerCount > 64 ||
            (input.empty() && viaInput.empty() && padInput.empty() &&
             arcInput.empty() && outlineInput.empty() && circleInput.empty() && boundaryInput.empty() && pourInput.empty() && copperInput.empty()) ||
            input.size() + arcInput.size() > 10000 || viaInput.size() > 10000 ||
            padInput.size() > 5000 || outlineInput.size() > 16384 ||
            circleInput.size() > 64 || boundaryInput.size() > 16384 ||
            pourInput.size() > 16384 || copperInput.size() > 64 ||
            outlineInput.size() + circleInput.size() + boundaryInput.size() > 16384)
            return false;
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
        if (!closeRing()) return false;
        int outerCircles=0;
        QSet<QString> circleKeys;
        for (const auto &c : circleInput) {
            constexpr qint64 limit_nm = 1422400000LL;
            // Validate individual operands before any signed additions:
            // an untrusted V7 circle must never overflow qint64 extrema.
            if (c.name.isEmpty() || c.name.size()>16 ||
                c.radius_nm<=0 || c.radius_nm>limit_nm ||
                c.cx_nm < -limit_nm || c.cx_nm > limit_nm ||
                c.cy_nm < -limit_nm || c.cy_nm > limit_nm ||
                c.cx_nm%254!=0 || c.cy_nm%254!=0 || c.radius_nm%254!=0)
                return false;
            for (const auto v : {c.cx_nm-c.radius_nm,c.cx_nm+c.radius_nm,
                                 c.cy_nm-c.radius_nm,c.cy_nm+c.radius_nm}) {
                if (v < -1422400000LL || v > 1422400000LL) return false;
            }
            const QString key=(c.cutout ? "CUT|" : "OUTER|")+c.name;
            if(circleKeys.contains(key) || key==ringKey) return false;
            circleKeys.insert(key);
            if(!c.cutout) ++outerCircles;
        }
        int boundaryOuter=0;
        QString lastBoundaryKey;
        qint64 firstBX=0,firstBY=0,lastBX=0,lastBY=0;
        int seq=0;
        QSet<QString> boundaryKeys;
        for(const auto &b : boundaryInput) {
            if(b.name.isEmpty() || b.name.size()>16 || b.sequence<0) return false;
            for(auto coord : {b.x0_nm,b.y0_nm,b.x1_nm,b.y1_nm,b.cx_nm,b.cy_nm}) {
                if(coord%254 != 0 || coord < -1422400000LL || coord > 1422400000LL)
                    return false;
            }
            const QString key=(b.cutout ? "CUT|" : "OUTER|")+b.name;
            if(key!=lastBoundaryKey) {
                if(!lastBoundaryKey.isEmpty() &&
                   (lastBX!=firstBX || lastBY!=firstBY || seq<3)) return false;
                if(boundaryKeys.contains(key) || circleKeys.contains(key) ||
                   (key==ringKey)) return false;
                boundaryKeys.insert(key);lastBoundaryKey=key;
                firstBX=b.x0_nm;firstBY=b.y0_nm;seq=0;
                if(!b.cutout) ++boundaryOuter;
            } else if(b.x0_nm!=lastBX || b.y0_nm!=lastBY) return false;
            if(b.sequence!=seq || (b.x0_nm==b.x1_nm && b.y0_nm==b.y1_nm))
                return false;
            if(b.isArc) {
                const qint64 dx0=(b.x0_nm-b.cx_nm)/254;
                const qint64 dy0=(b.y0_nm-b.cy_nm)/254;
                const qint64 dx1=(b.x1_nm-b.cx_nm)/254;
                const qint64 dy1=(b.y1_nm-b.cy_nm)/254;
                if(dx0*dx0+dy0*dy0==0 ||
                   dx0*dx0+dy0*dy0!=dx1*dx1+dy1*dy1 ||
                   ((dx0==0)==(dy0==0)) || ((dx1==0)==(dy1==0)) ||
                   dx0*dx1+dy0*dy1!=0 ||
                   ((dx0*dy1-dy0*dx1<0)!=b.clockwise)) return false;
            }
            ++seq;lastBX=b.x1_nm;lastBY=b.y1_nm;
        }
        if(!boundaryInput.empty() &&
           (lastBX!=firstBX || lastBY!=firstBY || seq<3)) return false;
        QString pourKey;
        QSet<QString> pourNames;
        qint64 firstPX=0,firstPY=0,lastPX=0,lastPY=0;
        int pourSeq=0;
        for(const auto &e:pourInput) {
            if(e.layer<1 || e.layer>layerCount || e.name.isEmpty() ||
               e.name.size()>16 || e.net.isEmpty() || e.net.size()>47 ||
               e.sourceLine<=0 || e.sequence<0) return false;
            for(auto v:{e.x0_nm,e.y0_nm,e.x1_nm,e.y1_nm})
                if(v%254!=0 || v<-1422400000LL || v>1422400000LL) return false;
            if(e.name!=pourKey) {
                if(!pourKey.isEmpty() &&
                   (pourSeq<3 || lastPX!=firstPX || lastPY!=firstPY)) return false;
                if(pourNames.contains(e.name) || e.sequence!=0) return false;
                pourNames.insert(e.name);pourKey=e.name;pourSeq=0;
                firstPX=e.x0_nm;firstPY=e.y0_nm;
            } else if(e.x0_nm!=lastPX || e.y0_nm!=lastPY) return false;
            if(e.sequence!=pourSeq || (e.x0_nm==e.x1_nm && e.y0_nm==e.y1_nm))
                return false;
            ++pourSeq;lastPX=e.x1_nm;lastPY=e.y1_nm;
        }
        if(!pourInput.empty() &&
           (pourSeq<3 || lastPX!=firstPX || lastPY!=firstPY)) return false;
        auto copperChecked=copperInput;
        QSet<QString> copperNames;
        for(auto &c:copperChecked) {
            if(c.layer>layerCount || !validateCopperArea(c) || copperNames.contains(c.name))
                return false;
            copperNames.insert(c.name);
        }
        if ((!outlineInput.empty() || !circleInput.empty() || !boundaryInput.empty()) &&
            outerRings+outerCircles+boundaryOuter!=1) return false;
        {
            const QSignalBlocker muteSelection(scene_);
            contacts_.clear();
            tracks_.clear();
            arcs_.clear();
            vias_.clear();
            pads_.clear();
            outlineEdges_.clear();
            circles_.clear();
            boundaries_.clear();
            pours_.clear();
            copper_.clear();
            scene_->clear();
        }
        layerVisible_ = QVector<bool>(layerCount + 1, true);
        savedLayerVisibility_.clear(); hasSavedVisibility_=false;
        if(netSearch_) netSearch_->clear();
        if(netOnly_) netOnly_->setChecked(false);
        const QSignalBlocker muteLayers(layers_);
        const QSignalBlocker muteNets(nets_);
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
            item->setData(1, QString("Straight trace / layer %1 / width %2 mm"
                                     " / start (%3, %4) mm / end (%5, %6) mm"
                                     " / length %7 mm / net %8 / connectivity NOT_VERIFIED")
                                 .arg(t.layer).arg(t.width_nm / 1e6, 0, 'f', 6)
                                 .arg(t.x0_nm/1e6,0,'f',6).arg(t.y0_nm/1e6,0,'f',6)
                                 .arg(t.x1_nm/1e6,0,'f',6).arg(t.y1_nm/1e6,0,'f',6)
                                 .arg(std::hypot(static_cast<double>(t.x1_nm-t.x0_nm),
                                                 static_cast<double>(t.y1_nm-t.y0_nm))/1e6,0,'f',6)
                                 .arg(t.net));
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
                               .arg(a.clockwise ? "CW" : "CCW").arg(a.sourceLine)
                               + QString(" / start (%1, %2) mm / end (%3, %4) mm"
                                         " / radius %5 mm / net %6 / connectivity NOT_VERIFIED")
                                   .arg(a.x0_nm/1e6,0,'f',6).arg(a.y0_nm/1e6,0,'f',6)
                                   .arg(a.x1_nm/1e6,0,'f',6).arg(a.y1_nm/1e6,0,'f',6)
                                   .arg(std::hypot(static_cast<double>(a.x0_nm-a.cx_nm),
                                                   static_cast<double>(a.y0_nm-a.cy_nm))/1e6,0,'f',6)
                                   .arg(a.net));
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
                                   .arg(via.firstLayer).arg(via.lastLayer)
                                   + QString(" / centre (%1, %2) mm / declared net %3"
                                             " / connectivity NOT_VERIFIED")
                                       .arg(via.x_nm/1e6,0,'f',6).arg(via.y_nm/1e6,0,'f',6)
                                       .arg(via.net));
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
        for(const auto &c:copperChecked) {
            if(c.net!="-" && nets_->findText(c.net)<0) nets_->addItem(c.net);
            const QColor color(c.layer==1 ? "#e6aa55" :
                               c.layer==layerCount ? "#509fca" : "#8fb66f");
            QGraphicsItem *item=nullptr;
            if(c.circle && c.hollowTriangles.empty()) {
                const qreal r=c.radius_nm/1e6;
                item=scene_->addEllipse(QRectF((c.cx_nm-c.radius_nm)/1e6,
                    (-c.cy_nm-c.radius_nm)/1e6,2*r,2*r),
                    QPen(color.darker(135),0),QBrush(color));
            }else{
                QPainterPath path;
                // A hollow COPCIR has analytic circle source and NO
                // polygonal ring; its verified triangles are below.
                if(!c.circle) {
                    path.moveTo(c.ring[0].first/1e6,-c.ring[0].second/1e6);
                    for(size_t i=1;i<c.ring.size();++i)
                        path.lineTo(c.ring[i].first/1e6,-c.ring[i].second/1e6);
                    path.closeSubpath();
                }
                // CPU and RHI must share the SAME verified triangles for
                // approximate circular voids; do not render exact Qt circles
                // while RHI uses an intentionally conservative polygon.
                path.setFillRule(Qt::OddEvenFill);
                if(!c.hollowTriangles.empty()) {
                    path=QPainterPath();
                    for(const auto &tri:c.hollowTriangles) {
                        path.moveTo(tri[0].first/1e6,-tri[0].second/1e6);
                        path.lineTo(tri[1].first/1e6,-tri[1].second/1e6);
                        path.lineTo(tri[2].first/1e6,-tri[2].second/1e6);
                        path.closeSubpath();
                    }
                    path.setFillRule(Qt::WindingFill);
                } else for(const auto &cut:c.cutouts) {
                    path.moveTo(cut[0].first/1e6,-cut[0].second/1e6);
                    for(size_t i=1;i<cut.size();++i)
                        path.lineTo(cut[i].first/1e6,-cut[i].second/1e6);
                    path.closeSubpath();
                }
                item=scene_->addPath(path,
                    c.hollowTriangles.empty() ? QPen(color.darker(135),0) : QPen(Qt::NoPen),
                    QBrush(color));
            }
            item->setZValue(-1); // Beneath trace/pad/POUROUT outlines.
            item->setData(0,c.net=="-" ? QString() : c.net);
            item->setData(2,QString("COPPER %1 / %2 filled SOURCE geometry / layer %3"
                " / source line %4 / declared net %5 / vertices %6 / cutouts %7"
                " / connectivity NOT_VERIFIED / fabricated copper NOT_VERIFIED / SI NOT_RUN")
                .arg(c.name).arg(c.circle ? "COPCIR" : (c.curved ? "COPCLS ARC (conservative shared mesh)" : "COPCLS"))
                .arg(c.layer).arg(c.sourceLine).arg(c.net)
                .arg(c.circle ? 0 : static_cast<int>(c.ring.size()))
                .arg(static_cast<int>(c.cutouts.size()+c.roundCutouts.size())));
            item->setFlag(QGraphicsItem::ItemIsSelectable);
            copper_.push_back({item,c});
        }
        for(const auto &e:pourInput) {
            if(e.net!="-" && nets_->findText(e.net)<0) nets_->addItem(e.net);
            const QLineF line(e.x0_nm/1e6,-e.y0_nm/1e6,e.x1_nm/1e6,-e.y1_nm/1e6);
            auto *item=scene_->addLine(line,QPen(QColor("#8be1c6"),0,Qt::DashLine));
            item->setData(0,e.net=="-" ? QString() : e.net);
            item->setData(2,QString("POUROUT %1 / unfilled boundary / layer %2"
                " / source line %3 / segment %4 / start (%5, %6) mm"
                " / end (%7, %8) mm / declared net %9 / flood NOT_VERIFIED / SI NOT_RUN")
                .arg(e.name).arg(e.layer).arg(e.sourceLine).arg(e.sequence)
                .arg(e.x0_nm/1e6,0,'f',6).arg(e.y0_nm/1e6,0,'f',6)
                .arg(e.x1_nm/1e6,0,'f',6).arg(e.y1_nm/1e6,0,'f',6).arg(e.net));
            item->setFlag(QGraphicsItem::ItemIsSelectable);
            pours_.push_back({item,e});
        }
        for (const auto &e : outlineInput) {
            const QLineF line(e.x0_nm/1e6,-e.y0_nm/1e6,
                              e.x1_nm/1e6,-e.y1_nm/1e6);
            const QColor color(e.cutout ? "#e99f87" : "#e0e8ee");
            auto *item=scene_->addLine(line,QPen(color,0));
            item->setData(1,e.cutout ? "Board Cutout (not copper)" :
                                         "Board Outline (not copper)");
            item->setData(2,QString("%1 %2 / segment %3 / start (%4, %5) mm"
                                    " / end (%6, %7) mm / not copper")
                             .arg(e.cutout ? "Board Cutout" : "Board Outline")
                             .arg(e.name).arg(e.sequence)
                             .arg(e.x0_nm/1e6,0,'f',6).arg(e.y0_nm/1e6,0,'f',6)
                             .arg(e.x1_nm/1e6,0,'f',6).arg(e.y1_nm/1e6,0,'f',6));
            item->setFlag(QGraphicsItem::ItemIsSelectable);
            outlineEdges_.push_back({item,e});
        }
        for (const auto &c : circleInput) {
            const QColor color(c.cutout ? "#e99f87" : "#e0e8ee");
            const qreal radius=c.radius_nm/1e6;
            auto *item=scene_->addEllipse(
                QRectF((c.cx_nm-c.radius_nm)/1e6,(-c.cy_nm-c.radius_nm)/1e6,
                       2*radius,2*radius),QPen(color,0),Qt::NoBrush);
            item->setData(1,QString("%1 %2 / centre (%3, %4) mm / radius %5 mm / not copper")
                          .arg(c.cutout ? "Board Cutout" : "Board Outline")
                          .arg(c.name).arg(c.cx_nm/1e6,0,'f',6)
                          .arg(c.cy_nm/1e6,0,'f',6).arg(radius,0,'f',6));
            item->setFlag(QGraphicsItem::ItemIsSelectable);
            circles_.push_back({item,c});
        }
        for(const auto &b : boundaryInput) {
            QPainterPath path;
            const QPointF start(b.x0_nm/1e6, -b.y0_nm/1e6);
            path.moveTo(start);
            if(b.isArc) {
                const qreal radius=std::hypot(
                    static_cast<qreal>(b.x0_nm-b.cx_nm),
                    static_cast<qreal>(b.y0_nm-b.cy_nm))/1e6;
                const qreal startAngle=std::atan2(
                    static_cast<qreal>(b.y0_nm-b.cy_nm),
                    static_cast<qreal>(b.x0_nm-b.cx_nm))*180.0/3.14159265358979323846;
                path.arcTo(QRectF(b.cx_nm/1e6-radius,-b.cy_nm/1e6-radius,
                                  2*radius,2*radius),
                           startAngle,b.clockwise?-90.0:90.0);
            } else {
                path.lineTo(QPointF(b.x1_nm/1e6,-b.y1_nm/1e6));
            }
            auto *item=scene_->addPath(path,
                QPen(QColor(b.cutout ? "#e99f87" : "#e0e8ee"),0),Qt::NoBrush);
            item->setData(1,QString("Board %1 %2 segment %3 / %4 / not copper")
                .arg(b.cutout ? "Cutout" : "Outline").arg(b.name).arg(b.sequence)
                .arg(b.isArc ? "Arc" : "Line"));
            item->setFlag(QGraphicsItem::ItemIsSelectable);
            boundaries_.push_back({item,b});
        }
        skipped_ = skipped;
#ifdef BOARDSI_RHI_ARTWORK
        if (rhi_) rhi_->load(layerCount,input,viaInput,padInput,arcInput,outlineInput,circleInput,boundaryInput,pourInput,copperChecked);
#endif
        scene_->setSceneRect(scene_->itemsBoundingRect().adjusted(-1, -1, 1, 1));
        refresh();
        fitArtwork();
        return true;
    }
    void fitArtwork() {
        if (!tracks_.empty() || !arcs_.empty() || !vias_.empty() ||
            !pads_.empty() || !outlineEdges_.empty() || !circles_.empty() || !boundaries_.empty() || !pours_.empty() || !copper_.empty()) {
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
    int circleCount() const { return static_cast<int>(circles_.size()); }
    int boundaryCount() const { return static_cast<int>(boundaries_.size()); }
    int pourEdgeCount() const { return static_cast<int>(pours_.size()); }
    int copperCount() const {return static_cast<int>(copper_.size());}
    bool selectCopper(int index) {
        if(index<0 || index>=static_cast<int>(copper_.size())) return false;
        scene_->clearSelection();
        copper_[static_cast<size_t>(index)].first->setSelected(true);
        return true;
    }
    bool selectPour(int index) {
        if(index<0 || index>=static_cast<int>(pours_.size())) return false;
        scene_->clearSelection();
        pours_[static_cast<size_t>(index)].first->setSelected(true);
        return true;
    }
    bool selectOutline(int index) {
        if(index<0 || index>=static_cast<int>(outlineEdges_.size())) return false;
        scene_->clearSelection();
        outlineEdges_[static_cast<size_t>(index)].first->setSelected(true);
        return true;
    }
    void selectBoundary(int index) {
        if (index < 0 || index >= static_cast<int>(boundaries_.size())) return;
        scene_->clearSelection();
        boundaries_[static_cast<size_t>(index)].first->setSelected(true);
    }

    int viaCount() const { return static_cast<int>(vias_.size()); }
    int padCount() const { return static_cast<int>(pads_.size()); }
#ifdef BOARDSI_RHI_ARTWORK
    BoardSiRhiView *rhiViewForTest() const { return rhi_; }
    bool rhiCanvasActiveForTest() const {
        return rhi_ && canvasStack_->currentWidget() == rhi_;
    }
#endif
    bool selectCircle(int index) {
        if (index < 0 || index >= static_cast<int>(circles_.size())) return false;
        scene_->clearSelection();
        circles_[static_cast<size_t>(index)].first->setSelected(true);
        return true;
    }
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
