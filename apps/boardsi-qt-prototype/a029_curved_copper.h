// BoardSI A_029. Source-exact cardinal COPCLS arcs, conservative display mesh.
// Source and mesh MUST NOT be conflated with fabricated copper or electrical SI.
#pragma once
#include <cmath>
#include <set>
#include <utility>
#include <vector>
#include <array>
#include <algorithm>
#include <cstdint>
namespace a029 {
using P=std::pair<qint64,qint64>; // source 254-nm ticks
inline qint64 cross(P a,P b,P c) {
    return (b.first-a.first)*(c.second-b.second)-
           (b.second-a.second)*(c.first-b.first);
}
inline qint64 area2(const std::vector<P>&ring) {
    qint64 sum=0;
    for(size_t i=1;i<ring.size();++i)
        sum+=ring[i-1].first*ring[i].second-ring[i].first*ring[i-1].second;
    return sum;
}
inline bool convex(const std::vector<P>&ring) {
    if(ring.size()<4||ring.size()>256||ring.front()!=ring.back())return false;
    const qint64 area=area2(ring);
    if(area==0)return false;
    const size_t n=ring.size()-1;
    std::set<P> unique(ring.begin(),ring.end()-1);
    if(unique.size()!=n)return false;
    for(size_t i=0;i<n;++i) {
        const P a=ring[i],b=ring[(i+1)%n],c=ring[(i+2)%n];
        if(a==b || cross(a,b,c)*(area>0?1:-1)<0)return false;
    }
    return true;
}
inline bool point(qint64 x,qint64 y) {
    return x%254==0&&y%254==0 &&
           x>=-1422400000LL&&x<=1422400000LL &&
           y>=-1422400000LL&&y<=1422400000LL;
}
inline bool validate(ImportedCopperArea &c) {
    if(!c.curved||c.circle||!c.cutouts.empty()||!c.roundCutouts.empty()||
       !c.ring.empty()||c.curvedSegments.size()<3||c.curvedSegments.size()>64)
        return false;
    std::vector<P> chord, display;
    chord.reserve(c.curvedSegments.size()+1);
    display.reserve(256);
    for(size_t i=0;i<c.curvedSegments.size();++i) {
        const auto &e=c.curvedSegments[i];
        if(!point(e.x0_nm,e.y0_nm)||!point(e.x1_nm,e.y1_nm)||
           !point(e.cx_nm,e.cy_nm)||(e.x0_nm==e.x1_nm&&e.y0_nm==e.y1_nm))
            return false;
        P start{e.x0_nm/254,e.y0_nm/254},end{e.x1_nm/254,e.y1_nm/254};
        if(i && chord.back()!=start)return false;
        if(i==0){chord.push_back(start);display.push_back(start);}
        chord.push_back(end);
        if(e.isArc) {
            const qint64 cx=e.cx_nm/254,cy=e.cy_nm/254;
            const qint64 dx0=start.first-cx,dy0=start.second-cy;
            const qint64 dx1=end.first-cx,dy1=end.second-cy;
            const qint64 r=std::max(std::abs(dx0),std::abs(dy0));
            if(r<1000 || (dx0==0)==(dy0==0) ||
               (dx1==0)==(dy1==0) || dx0*dx0+dy0*dy0!=r*r ||
               dx1*dx1+dy1*dy1!=r*r || dx0*dx1+dy0*dy1!=0 ||
               ((dx0*dy1-dy0*dx1)<0)!=e.clockwise)return false;
            const double theta=std::atan2(static_cast<double>(dy0),static_cast<double>(dx0));
            const double step=(e.clockwise?-1.0:1.0)*
                1.57079632679489661923/8.0;
            for(int j=1;j<=8;++j) {
                P p=end;
                if(j<8) {
                    const double a=theta+step*j;
                    p={cx+static_cast<qint64>(std::llround((r-2)*std::cos(a))),
                       cy+static_cast<qint64>(std::llround((r-2)*std::sin(a)))};
                    const qint64 xx=p.first-cx,yy=p.second-cy;
                    if(xx*xx+yy*yy>=r*r)return false;
                }
                if(p==display.back()||p.first < -5600000 || p.first > 5600000 ||
                   p.second < -5600000 || p.second > 5600000)return false;
                display.push_back(p);
            }
        } else {
            if(e.cx_nm!=0||e.cy_nm!=0||e.clockwise)return false;
            display.push_back(end);
        }
        if(display.size()>256)return false;
    }
    if(!convex(chord)||!convex(display))return false;
    const bool ccw=area2(chord)>0;
    int arcs=0;
    for(const auto &e:c.curvedSegments) {
        if(e.isArc) {
            ++arcs;
            if(e.clockwise==ccw)return false; // Only convex OUTWARD curves.
        }
    }
    if(arcs==0)return false;
    // A convex polygon inscribed in the convex source curved ring cannot
    // over-paint it. The proxy is display only; original arcs are separate.
    // An integer-grid fan covers the exact proxy once, shared by CPU and RHI.
    qint64 sum=0;
    for(size_t i=1;i+1<display.size()-1;++i) {
        const qint64 a=cross(display.front(),display[i],display[i+1]);
        if(a==0||(a>0)!=ccw)return false;
        sum+=a;
        c.hollowTriangles.push_back({{
            {display.front().first*254,display.front().second*254},
            {display[i].first*254,display[i].second*254},
            {display[i+1].first*254,display[i+1].second*254}
        }});
    }
    // Fan excludes the closing repeated vertex by construction.
    if(sum!=area2(display)||c.hollowTriangles.size()<1) {
        c.hollowTriangles.clear();
        return false;
    }
    c.ring.clear();
    for(P p:display)c.ring.push_back({p.first*254,p.second*254});
    return true;
}
} // namespace a029
