// A_026 P2-Q: analytic COPCCO source voids -> conservative exact-grid
// display-only circumscribed proxies. No source geometry is overwritten.
// CPU/RHI share the identical validated triangles. No physical SI claims.
#pragma once
#include <cmath>
#include <cstdint>
#include <limits>
#include <utility>
#include <vector>
namespace a026 {
using P=std::pair<qint64,qint64>;
constexpr int kSegments=32;
constexpr qint64 kGrid=254;
constexpr qint64 kBoundTicks=5600000;
constexpr qint64 kMinRadiusTicks=1000; // 0.254 mm; smaller circles fail closed
// Maximum proxy radial deviation <= r*(sec(pi/32)-1)+4 ticks.
// With r>=1000 ticks, <=0.89% of the source radius.
inline bool validCircle(const ImportedCopperCircleHole &h) {
    auto coord=[](qint64 v) {
        return v%kGrid==0 && v/kGrid>=-kBoundTicks && v/kGrid<=kBoundTicks;
    };
    if(!coord(h.cx_nm)||!coord(h.cy_nm)||!coord(h.radius_nm)||
       h.radius_nm/kGrid<kMinRadiusTicks) return false;
    const qint64 r=h.radius_nm;
    return h.cx_nm>=std::numeric_limits<qint64>::min()+r &&
           h.cx_nm<=std::numeric_limits<qint64>::max()-r &&
           h.cy_nm>=std::numeric_limits<qint64>::min()+r &&
           h.cy_nm<=std::numeric_limits<qint64>::max()-r &&
           coord(h.cx_nm-r)&&coord(h.cx_nm+r) &&
           coord(h.cy_nm-r)&&coord(h.cy_nm+r);
}
inline bool proxy(const ImportedCopperCircleHole &h,std::vector<P> &ring) {
    if(!validCircle(h)) return false;
    const qint64 x=h.cx_nm/kGrid,y=h.cy_nm/kGrid,r=h.radius_nm/kGrid;
    constexpr long double pi=3.141592653589793238462643383279502884L;
    constexpr long double n=static_cast<long double>(kSegments);
    const long double nominal=static_cast<long double>(r);
    const long double target=(nominal+3.L)/std::cos(pi/n);
    ring.clear();ring.reserve(kSegments+1);
    for(int i=0;i<kSegments;++i) {
        const long double angle=2*pi*(static_cast<long double>(i)+0.5L)/n;
        const qint64 dx=static_cast<qint64>(std::llround(target*std::cos(angle)));
        const qint64 dy=static_cast<qint64>(std::llround(target*std::sin(angle)));
        const qint64 px=x+dx,py=y+dy;
        if(px<-kBoundTicks||px>kBoundTicks||
           py<-kBoundTicks||py>kBoundTicks)return false;
        ring.push_back({px*kGrid,py*kGrid});
    }
    ring.push_back(ring.front());
    // Every complete polygon edge must lie strictly OUTSIDE source circle.
    // Integer source coordinates fit double mantissas exactly. The 1-tick
    // clearance guard dominates all floating arithmetic approximation here.
    for(size_t i=1;i<ring.size();++i) {
        const auto a=ring[i-1],b=ring[i];
        const long double ax=static_cast<long double>(a.first/kGrid-x);
        const long double ay=static_cast<long double>(a.second/kGrid-y);
        const long double dx=static_cast<long double>((b.first-a.first)/kGrid);
        const long double dy=static_cast<long double>((b.second-a.second)/kGrid);
        const long double len2=dx*dx+dy*dy;
        if(len2<=0)return false;
        const long double t=std::clamp(-(ax*dx+ay*dy)/len2,0.L,1.L);
        const long double distance=std::hypot(ax+t*dx,ay+t*dy);
        if(!std::isfinite(distance)||distance<nominal+1.L)return false;
        if(std::hypot(ax,ay)>nominal*1.01L||
           std::hypot(ax+dx,ay+dy)>nominal*1.01L)return false;
    }
    return true;
}
// A_027: display-only INSCRIBED polygon for a source COPCIR outer.
// Convexity + every checked integer vertex strictly inside the analytic
// circle prove every mesh triangle stays inside source copper. Independent
// segment-distance test caps radial display underfill to <=1% of radius.
inline bool innerProxy(const ImportedCopperCircleHole &outer,std::vector<P> &ring) {
    if(!validCircle(outer))return false;
    const qint64 cx=outer.cx_nm/kGrid,cy=outer.cy_nm/kGrid;
    const qint64 r=outer.radius_nm/kGrid;
    constexpr long double pi=3.141592653589793238462643383279502884L;
    const long double rad=static_cast<long double>(r)-3.L;
    if(rad<=0)return false;
    ring.clear();ring.reserve(kSegments+1);
    for(int i=0;i<kSegments;++i) {
        const long double angle=2*pi*static_cast<long double>(i)/kSegments;
        const qint64 dx=static_cast<qint64>(std::llround(rad*std::cos(angle)));
        const qint64 dy=static_cast<qint64>(std::llround(rad*std::sin(angle)));
        const qint64 x=cx+dx,y=cy+dy;
        if(x<-kBoundTicks||x>kBoundTicks||y<-kBoundTicks||y>kBoundTicks)
            return false;
        // Here 2*r*r <= 6.3e13, safely within signed 64-bit.
        if(dx*dx+dy*dy>=r*r)return false;
        ring.push_back({x*kGrid,y*kGrid});
    }
    ring.push_back(ring.front());
    for(size_t i=1;i<ring.size();++i) {
        const auto a=ring[i-1],b=ring[i];
        const long double ax=static_cast<long double>(a.first/kGrid-cx);
        const long double ay=static_cast<long double>(a.second/kGrid-cy);
        const long double bx=static_cast<long double>(b.first/kGrid-cx);
        const long double by=static_cast<long double>(b.second/kGrid-cy);
        const long double dx=bx-ax,dy=by-ay;
        const long double len=std::hypot(dx,dy);
        if(len<=0 || !std::isfinite(len))return false;
        const long double distance=std::abs(ax*by-ay*bx)/len;
        if(!std::isfinite(distance) || distance<0.99L*r)return false;
    }
    return true;
}
inline bool triangulate(ImportedCopperArea &a) {
    // A_028: source COPCIR may own exact COPCUT polygon voids, optionally
    // alongside source COPCCO. Source topology is independently checked by
    // Rust. Here display proxies must pass the full A_025 triangle oracle.
    if((a.roundCutouts.empty() && a.cutouts.empty()) ||
       a.cutouts.size()+a.roundCutouts.size()>8)return false;
    ImportedCopperArea proxyArea=a;
    if(a.circle) {
        // Inscribed display proxy never creates copper beyond the analytic
        // source circle. Polygon voids remain on their EXACT source grid.
        if(!innerProxy({a.cx_nm,a.cy_nm,a.radius_nm},proxyArea.ring))
            return false;
        proxyArea.circle=false; // A_025 consumes polygonal display proxies.
    }
    for(const auto &hole:a.roundCutouts) {
        std::vector<P> ring;
        if(!proxy(hole,ring))return false;
        proxyArea.cutouts.push_back(std::move(ring));
    }
    // The A_025 independent integer-grid mesh checks strict separation,
    // triangle overlap, total area and bounded complexity. Both RHI and CPU
    // use ONLY these triangles; no analytic disk may paint over the hole.
    if(!a025::triangulate(proxyArea)||proxyArea.hollowTriangles.empty())
        return false;
    a.hollowTriangles=std::move(proxyArea.hollowTriangles);
    return true;
}
} // namespace a026
