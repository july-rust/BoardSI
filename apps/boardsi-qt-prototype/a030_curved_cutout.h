// BoardSI A_030 / P2-U: single straight convex COPCUT inside an already
// validated, inward-conservative cardinal-arc COPCLS display proxy.
// Source analytical arcs remain in curvedSegments. The shared CPU/RHI mesh
// is display ONLY, not actual filled copper or electrical connectivity.
#pragma once
#include <algorithm>
#include <utility>
#include <vector>
namespace a030 {
using P=std::pair<qint64,qint64>;
inline bool holeConvex(const std::vector<P>&hole) {
    if(!a025::simple(a025::decode(hole)))return false;
    const auto p=a025::decode(hole);
    const qint64 winding=a025::area(p);
    if(winding==0)return false;
    for(size_t i=0;i<p.size();++i) {
        const qint64 z=a025::cross(p[i],p[(i+1)%p.size()],p[(i+2)%p.size()]);
        if(z==0 || (z>0)!=(winding>0))return false;
    }
    return true;
}
inline bool triangulate(ImportedCopperArea &a) {
    // The initial un-holed G|R was already accepted by A_029 in Qt.
    if(!a.curved || a.circle || a.cutouts.size()!=1 ||
       !a.roundCutouts.empty() || !holeConvex(a.cutouts.front()))
        return false;
    ImportedCopperArea proxy=a;
    proxy.cutouts.clear();
    if(!a029::validate(proxy))return false;
    // Remove ONLY redundant straight-line collinear vertices in display
    // proxy. A_029 deliberately permits source redundant collinears; A_025
    // exact-grid ear-clipper does not. Never touch analytical source segments.
    std::vector<P> ring(proxy.ring.begin(),proxy.ring.end()-1);
    bool changed=true;
    while(changed && ring.size()>3) {
        changed=false;
        for(size_t i=0;i<ring.size();++i) {
            const P prev=ring[(i+ring.size()-1)%ring.size()];
            const P here=ring[i],next=ring[(i+1)%ring.size()];
            if(a025::cross(prev,here,next)==0 && a025::on(prev,next,here)) {
                ring.erase(ring.begin()+static_cast<std::ptrdiff_t>(i));
                changed=true;break;
            }
        }
    }
    if(ring.size()<3 || ring.size()+1+a.cutouts[0].size()>128)return false;
    ring.push_back(ring.front());
    // Strictly separated A_025 topology and exactly conserved triangle area
    // against the reduced but geometrically IDENTICAL conservative polygon.
    proxy.ring=std::move(ring);
    proxy.cutouts=a.cutouts;
    proxy.curved=false;
    proxy.curvedSegments.clear();
    proxy.hollowTriangles.clear();
    if(!a025::triangulate(proxy) || proxy.hollowTriangles.empty())return false;
    a.ring=std::move(proxy.ring);
    a.hollowTriangles=std::move(proxy.hollowTriangles);
    return true;
}
} // namespace a030
