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
inline bool triangulate(ImportedCopperArea &a) {
    if(a.circle||a.roundCutouts.empty()||
       a.cutouts.size()+a.roundCutouts.size()>8)return false;
    ImportedCopperArea proxyArea=a;
    for(const auto &hole:a.roundCutouts) {
        std::vector<P> ring;
        if(!proxy(hole,ring))return false;
        proxyArea.cutouts.push_back(std::move(ring));
    }
    // Bounded exact-source-tick polygon mesh with independent area,
    // touching and overlap checks from A_025. No approximated void can
    // introduce copper over an actual circular source void.
    if(!a025::triangulate(proxyArea)||proxyArea.hollowTriangles.empty())
        return false;
    a.hollowTriangles=std::move(proxyArea.hollowTriangles);
    return true;
}
} // namespace a026
