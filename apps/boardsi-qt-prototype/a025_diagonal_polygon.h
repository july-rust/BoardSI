// A_025 P2-P. Exact source-tick triangulation for validated straight polygons
// with bounded holes. No third-party geometry library or fabricated flood.
// Included AFTER ImportedCopperArea; all intermediate predicates are signed 64-bit
// source ticks (|coord|<=5,600,000, <=128 source vertices).
#pragma once
#include <algorithm>
#include <array>
#include <cstdint>
#include <limits>
#include <utility>
#include <vector>
namespace a025 {
using P=std::pair<qint64,qint64>;
using Ring=std::vector<P>;
using Tri=std::array<P,3>;
inline qint64 cross(P a,P b,P c) {
    return (b.first-a.first)*(c.second-a.second)-
           (b.second-a.second)*(c.first-a.first);
}
inline qint64 area(const Ring &r) {
    qint64 a=0;
    for(size_t i=0;i<r.size();++i) {
        const P x=r[i],y=r[(i+1)%r.size()];
        a+=x.first*y.second-y.first*x.second;
    }
    return a;
}
inline bool on(P a,P b,P p) {
    return cross(a,b,p)==0 &&
           p.first>=std::min(a.first,b.first) && p.first<=std::max(a.first,b.first) &&
           p.second>=std::min(a.second,b.second) && p.second<=std::max(a.second,b.second);
}
inline bool touch(P a,P b,P c,P d) {
    const qint64 x=cross(a,b,c),y=cross(a,b,d),z=cross(c,d,a),w=cross(c,d,b);
    return (x==0&&on(a,b,c)) || (y==0&&on(a,b,d)) ||
           (z==0&&on(c,d,a)) || (w==0&&on(c,d,b)) ||
           ((x>0)!=(y>0) && (z>0)!=(w>0));
}
inline bool properCross(P a,P b,P c,P d) {
    const qint64 x=cross(a,b,c),y=cross(a,b,d),z=cross(c,d,a),w=cross(c,d,b);
    return x!=0 && y!=0 && z!=0 && w!=0 &&
           (x>0)!=(y>0) && (z>0)!=(w>0);
}
// Exact odd-even ray crossing for numerator / denominator rational query.
inline bool inside(P numerator,qint64 den,const Ring &r) {
    bool found=false;
    for(size_t i=0;i<r.size();++i) {
        const P a=r[i],b=r[(i+1)%r.size()];
        const P x={a.first*den,a.second*den},y={b.first*den,b.second*den};
        if(on(x,y,numerator))return false; // boundary is never interior
        if((x.second>numerator.second)!=(y.second>numerator.second)) {
            const qint64 side=cross(x,y,numerator);
            if((side>0)==(y.second>x.second?true:false)) {
                // The comparison above intentionally uses y.second only:
                // replace by normal ray rule directly below.
            }
            if((side>0)==(y.second>x.second?false:true)) {
                // no-op (kept out of the area predicate)
            }
            if((side>0)==(y.second>x.second))found=!found;
        }
    }
    return found;
}
inline bool simple(const Ring &r) {
    if(r.size()<3 || r.size()>64)return false;
    for(size_t i=0;i<r.size();++i) {
        const P prev=r[(i+r.size()-1)%r.size()];
        const P curr=r[i],next=r[(i+1)%r.size()];
        if(curr==next || cross(prev,curr,next)==0)return false;
        for(size_t j=i+2;j<r.size();++j) {
            if(i==0&&j==r.size()-1)continue;
            if(touch(curr,next,r[j],r[(j+1)%r.size()]))return false;
        }
    }
    return area(r)!=0;
}
inline bool contact(const Ring &a,const Ring &b) {
    for(size_t i=0;i<a.size();++i)
        for(size_t j=0;j<b.size();++j)
            if(touch(a[i],a[(i+1)%a.size()],b[j],b[(j+1)%b.size()]))
                return true;
    return false;
}
inline Ring decode(const std::vector<P>&raw) {
    if(raw.size()<4 || raw.size()>65 || raw.front()!=raw.back())return {};
    Ring ring;
    for(size_t i=0;i+1<raw.size();++i) {
        const P p=raw[i];
        if(p.first%254 || p.second%254 || p.first/254 < -5600000 ||
           p.first/254 > 5600000 || p.second/254 < -5600000 ||
           p.second/254 > 5600000)return {};
        ring.push_back({p.first/254,p.second/254});
    }
    return ring;
}
inline bool clearSegment(P a,P b,const Ring &ring) {
    for(size_t k=0;k<ring.size();++k) {
        const P c=ring[k],d=ring[(k+1)%ring.size()];
        if(c==a || d==a || c==b || d==b) {
            // Contact at intentional end vertices only. Reject collinear
            // overlap and any third-vertex contact along the bridge.
            if((c!=a&&c!=b&&on(a,b,c)) ||
               (d!=a&&d!=b&&on(a,b,d)) ||
               (a!=c&&a!=d&&on(c,d,a)) ||
               (b!=c&&b!=d&&on(c,d,b)) ||
               ((a==c||a==d)&&(b==c||b==d)))return false;
            continue;
        }
        if(touch(a,b,c,d))return false;
    }
    return true;
}
inline bool validBridge(P a,P b,const Ring &outer,
                        const std::vector<Ring>&holes,
                        const std::vector<std::pair<P,P>>&bridges) {
    if(a==b || !inside({a.first+b.first,a.second+b.second},2,outer))
        return false;
    for(const Ring &h:holes)
        if(inside({a.first+b.first,a.second+b.second},2,h))return false;
    if(!clearSegment(a,b,outer))return false;
    for(const Ring &h:holes)if(!clearSegment(a,b,h))return false;
    for(const auto &e:bridges) {
        if(a==e.first||a==e.second||b==e.first||b==e.second) {
            if((e.first!=a&&e.first!=b&&on(a,b,e.first)) ||
               (e.second!=a&&e.second!=b&&on(a,b,e.second)))return false;
        } else if(touch(a,b,e.first,e.second))return false;
    }
    return true;
}
inline bool triangulate(ImportedCopperArea &src) {
    // Only invoked when a same-owner hole is present. The legacy orthogonal
    // grid path is preserved, and the no-hole path remains unchanged.
    Ring outer=decode(src.ring);
    if(!simple(outer) || src.cutouts.empty() || src.cutouts.size()>8)return false;
    size_t sourceCount=src.ring.size();
    std::vector<Ring> holes;
    for(const auto &raw:src.cutouts) {
        sourceCount+=raw.size();
        if(sourceCount>128)return false;
        Ring h=decode(raw);
        if(!simple(h) || contact(outer,h) || !inside(h.front(),1,outer))
            return false;
        for(const Ring &old:holes)
            if(contact(old,h)||inside(h.front(),1,old)||inside(old.front(),1,h))
                return false;
        holes.push_back(std::move(h));
    }
    const qint64 expected=std::abs(area(outer))-
        [&]{qint64 sum=0;for(const auto &h:holes)sum+=std::abs(area(h));return sum;}();
    if(expected<=0)return false;
    if(area(outer)<0)std::reverse(outer.begin(),outer.end());
    for(auto &h:holes)if(area(h)>0)std::reverse(h.begin(),h.end());
    Ring merged=outer;
    std::vector<std::pair<P,P>> bridges;
    std::vector<size_t> order(holes.size());
    for(size_t i=0;i<order.size();++i)order[i]=i;
    std::sort(order.begin(),order.end(),[&](size_t i,size_t j){
        const auto rx=[](const Ring&r){return std::max_element(r.begin(),r.end(),
                [](P a,P b){return a.first<b.first;})->first;};
        return rx(holes[i])>rx(holes[j]);
    });
    for(size_t idx:order) {
        const Ring &h=holes[idx];
        size_t right=0;
        for(size_t i=1;i<h.size();++i)
            if(h[i].first>h[right].first ||
               (h[i].first==h[right].first && h[i].second<h[right].second))
                right=i;
        const P hp=h[right];
        size_t dest=merged.size();
        qint64 best=std::numeric_limits<qint64>::max();
        for(size_t i=0;i<merged.size();++i) {
            const P o=merged[i];
            if(!validBridge(o,hp,outer,holes,bridges))continue;
            const qint64 dx=o.first-hp.first,dy=o.second-hp.second;
            const qint64 dist=dx*dx+dy*dy;
            if(dist<best) {best=dist;dest=i;}
        }
        if(dest==merged.size())return false;
        Ring combined;
        combined.reserve(merged.size()+h.size()+2);
        combined.insert(combined.end(),merged.begin(),merged.begin()+static_cast<std::ptrdiff_t>(dest+1));
        for(size_t j=0;j<h.size();++j)combined.push_back(h[(right+j)%h.size()]);
        combined.push_back(hp);
        combined.push_back(merged[dest]);
        combined.insert(combined.end(),merged.begin()+static_cast<std::ptrdiff_t>(dest+1),merged.end());
        bridges.push_back({merged[dest],hp});
        merged.swap(combined);
    }
    std::vector<Tri> triangles;
    triangles.reserve(merged.size());
    size_t budget=merged.size()*merged.size();
    while(merged.size()>3 && budget-->0) {
        bool changed=false;
        for(size_t i=0;i<merged.size();++i) {
            const P p=merged[(i+merged.size()-1)%merged.size()];
            const P c=merged[i],n=merged[(i+1)%merged.size()];
            if(cross(p,c,n)<=0)continue;
            bool occupied=false;
            for(const P v:merged) {
                if(v==p||v==c||v==n)continue; // duplicated bridge endpoint
                if(cross(p,c,v)>=0 && cross(c,n,v)>=0 && cross(n,p,v)>=0) {
                    occupied=true;break;
                }
            }
            if(occupied)continue;
            const P centre={p.first+c.first+n.first,p.second+c.second+n.second};
            if(!inside(centre,3,outer))continue;
            bool voided=false;
            for(const Ring &h:holes)if(inside(centre,3,h)) {voided=true;break;}
            if(voided)continue;
            triangles.push_back({p,c,n});
            merged.erase(merged.begin()+static_cast<std::ptrdiff_t>(i));
            changed=true;break;
        }
        if(!changed)return false;
    }
    if(merged.size()!=3 || cross(merged[0],merged[1],merged[2])<=0)
        return false;
    triangles.push_back({merged[0],merged[1],merged[2]});
    qint64 sum=0;
    for(const Tri &t:triangles) {
        const qint64 a=cross(t[0],t[1],t[2]);
        if(a<=0)return false;
        sum+=a;
        const P cen={t[0].first+t[1].first+t[2].first,
                     t[0].second+t[1].second+t[2].second};
        if(!inside(cen,3,outer))return false;
        for(const Ring &h:holes)if(inside(cen,3,h))return false;
        // No proper crossing of any source-authored contour edge.
        for(size_t k=0;k<3;++k) {
            const P x=t[k],y=t[(k+1)%3];
            for(const Ring *r : std::vector<const Ring*>{&outer}) {
                for(size_t j=0;j<r->size();++j)
                    if(properCross(x,y,(*r)[j],(*r)[(j+1)%r->size()]))return false;
            }
            for(const Ring &r:holes)
                for(size_t j=0;j<r.size();++j)
                    if(properCross(x,y,r[j],r[(j+1)%r.size()]))return false;
        }
    }
    if(sum!=expected || triangles.size()>144)return false;
    // Ear clipping also must not produce overlapping interiors: check triangle
    // proper edge crossings and each triangle centroid against all others.
    for(size_t i=0;i<triangles.size();++i) {
        const auto &x=triangles[i];
        for(size_t j=i+1;j<triangles.size();++j) {
            const auto &y=triangles[j];
            for(size_t e=0;e<3;++e)
                for(size_t f=0;f<3;++f)
                    if(properCross(x[e],x[(e+1)%3],y[f],y[(f+1)%3]))
                        return false;
            auto isInsideTri=[](const P&c,const Tri&t){
                return cross(t[0],t[1],c)>0 &&
                       cross(t[1],t[2],c)>0 &&
                       cross(t[2],t[0],c)>0;
            };
            // Tested in exact 1/3 ticks by scaling each source triangle.
            const P cx={x[0].first+x[1].first+x[2].first,
                        x[0].second+x[1].second+x[2].second};
            const P cy={y[0].first+y[1].first+y[2].first,
                        y[0].second+y[1].second+y[2].second};
            auto scaled=[](const Tri&t) {return Tri{{{3*t[0].first,3*t[0].second},
                {3*t[1].first,3*t[1].second},{3*t[2].first,3*t[2].second}}};};
            if(isInsideTri(cx,scaled(y)) || isInsideTri(cy,scaled(x)))return false;
        }
    }
    src.hollowTriangles.clear();
    for(const Tri &t:triangles)
        src.hollowTriangles.push_back({P{t[0].first*254,t[0].second*254},
             P{t[1].first*254,t[1].second*254},
             P{t[2].first*254,t[2].second*254}});
    return !src.hollowTriangles.empty();
}
} // namespace a025
