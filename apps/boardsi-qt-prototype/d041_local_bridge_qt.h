// D_041 independent Qt source/local bridge graph; not global electrical PASS.
#pragma once
#include "d040_multi_component_qt.h"
#include <array>
#include <algorithm>
namespace d041 {
inline QByteArray reconstruct(const QByteArray &src,const PreviewImportPanel *panel) {
    if(!panel || !panel->imported() || src.count("*SIGNAL*")!=1)return {};
    const QByteArray marker="U1.1 U2.1\n";
    const int at=src.indexOf(marker);
    if(at<0 || src.indexOf(marker,at+1)>=0)return {};
    const int p=at+marker.size(),a=src.indexOf('\n',p);
    const int b=a<0?-1:src.indexOf('\n',a+1);
    if(a<0 || b<0)return {};
    QByteArray base=src;base.remove(at,b+1-at);
    if(!d040::boundedSections(base))return {};
    const auto *art=panel->imported();
    if(art->importedSkippedCount()!=0 || art->importedTraceCount()!=5 ||
       art->padCount()!=4 || art->arcCount()!=0 || art->viaCount()!=0)return {};
    const auto sha=QCryptographicHash::hash(src,QCryptographicHash::Sha256).toHex();
    if(sha!=panel->acceptedSourceSha256())return {};
    QStringList pins;
    const QString net=src.contains("*SIGNAL* NET_A")?"NET_A":"NET_B";
    if(net=="NET_B" && !src.contains("*SIGNAL* NET_B"))return {};
    if(!d005::declaredPins(src,sha,net,pins))return {};
    std::array<ImportedTrack,5> tr;
    for(int i=0;i<5;++i) {
        auto &t=tr[static_cast<size_t>(i)];
        if(!art->importedTrack(i,t) || t.layer!=1 || t.net!=net ||
           t.width_nm<=0 || t.width_nm%254!=0 ||
           t.x0_nm%254!=0 || t.y0_nm%254!=0 ||
           t.x1_nm%254!=0 || t.y1_nm%254!=0 ||
           (t.x0_nm==t.x1_nm && t.y0_nm==t.y1_nm))return {};
    }
    std::array<d040::Path,4> ps;
    std::array<int,5> used={};
    for(int i=0;i<4;++i) {
        auto &q=ps[static_cast<size_t>(i)];
        if(!art->importedPad(i,q.pad))return {};
        auto &pad=q.pad;
        if(pad.layer!=1 || pad.side!="T" ||
           (pad.shape!="R" && pad.shape!="S") || pad.width_nm<=0 ||
           pad.height_nm!=pad.width_nm || pad.width_nm%254!=0 ||
           pad.x_nm%254!=0 || pad.y_nm%254!=0 ||
           !pins.contains(pad.reference+"."+pad.pin))return {};
        for(int k=0;k<5;++k) {
            const auto &t=tr[static_cast<size_t>(k)];
            const bool start=t.x0_nm==pad.x_nm && t.y0_nm==pad.y_nm;
            const bool end=t.x1_nm==pad.x_nm && t.y1_nm==pad.y_nm;
            if(start==end)continue;
            if(q.trackIndex>=0 || t.width_nm>pad.width_nm/2)return {};
            q.trackIndex=k;q.trace=t;
            q.jointX=start?t.x1_nm:t.x0_nm;
            q.jointY=start?t.y1_nm:t.y0_nm;
            ++used[static_cast<size_t>(k)];
        }
        if(q.trackIndex<0)return {};
    }
    int missing=-1;
    for(int i=0;i<5;++i) {
        if(used[static_cast<size_t>(i)]==0) {
            if(missing>=0)return {};
            missing=i;
        } else if(used[static_cast<size_t>(i)]!=1)return {};
    }
    if(missing<0)return {};
    std::sort(ps.begin(),ps.end(),[](const auto &x,const auto &y){
        return std::make_pair(x.pad.reference,x.pad.pin) <
               std::make_pair(y.pad.reference,y.pad.pin);
    });
    for(int i=0;i<4;++i) {
        const auto &q=ps[static_cast<size_t>(i)];
        if(q.pad.reference!=(i<2?"U1":"U2") ||
           q.pad.pin!=(i%2==0?"1":"2"))return {};
    }
    for(int i=0;i<4;i+=2) {
        const auto &x=ps[static_cast<size_t>(i)];
        const auto &y=ps[static_cast<size_t>(i+1)];
        if(x.jointX!=y.jointX || x.jointY!=y.jointY ||
           x.trace.width_nm!=y.trace.width_nm)return {};
        if(!((x.pad.x_nm==y.pad.x_nm && x.jointX==x.pad.x_nm &&
              d039::strictlyBetween(x.pad.y_nm,x.jointY,y.pad.y_nm)) ||
             (x.pad.y_nm==y.pad.y_nm && x.jointY==x.pad.y_nm &&
              d039::strictlyBetween(x.pad.x_nm,x.jointX,y.pad.x_nm))))return {};
        if(2*(qAbs(x.pad.x_nm-x.jointX)+qAbs(x.pad.y_nm-x.jointY))<=x.pad.width_nm ||
           2*(qAbs(y.pad.x_nm-y.jointX)+qAbs(y.pad.y_nm-y.jointY))<=y.pad.width_nm)return {};
    }
    const auto &x=ps[0],&y=ps[2];
    const auto &bridge=tr[static_cast<size_t>(missing)];
    const bool fwd=bridge.x0_nm==x.jointX && bridge.y0_nm==x.jointY &&
                   bridge.x1_nm==y.jointX && bridge.y1_nm==y.jointY;
    const bool rev=bridge.x1_nm==x.jointX && bridge.y1_nm==x.jointY &&
                   bridge.x0_nm==y.jointX && bridge.y0_nm==y.jointY;
    if((!fwd && !rev) || bridge.width_nm!=x.trace.width_nm ||
       bridge.width_nm!=y.trace.width_nm)return {};
    const bool h=x.jointY==y.jointY && x.jointX!=y.jointX &&
        std::all_of(ps.cbegin(),ps.cend(),[](const auto &q){return q.pad.x_nm==q.jointX;});
    const bool v=x.jointX==y.jointX && x.jointY!=y.jointY &&
        std::all_of(ps.cbegin(),ps.cend(),[](const auto &q){return q.pad.y_nm==q.jointY;});
    if(!h && !v)return {};
    const d040::Box2 bbr={2*std::min(x.jointX,y.jointX)-bridge.width_nm,
        2*std::min(x.jointY,y.jointY)-bridge.width_nm,
        2*std::max(x.jointX,y.jointX)+bridge.width_nm,
        2*std::max(x.jointY,y.jointY)+bridge.width_nm};
    for(const auto &q:ps)if(bbr.intersects(d040::padBox(q.pad)))return {};
    for(int i=0;i<2;++i)for(int j=2;j<4;++j) {
        const auto &p0=ps[static_cast<size_t>(i)];
        const auto &p1=ps[static_cast<size_t>(j)];
        if(d040::padBox(p0.pad).intersects(d040::padBox(p1.pad)) ||
           d040::padBox(p0.pad).intersects(d040::traceBox(p1)) ||
           d040::traceBox(p0).intersects(d040::padBox(p1.pad)) ||
           d040::traceBox(p0).intersects(d040::traceBox(p1)))return {};
    }
    QByteArray out="BOARDSI_D041_LOCAL_BRIDGE_V1|"+sha+
        "|1|LOCAL_COPPER_BRIDGE_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN"
        "|GLOBAL_ELECTRICAL_NOT_VERIFIED\nC|0|U1+U2|"+net.toUtf8()+"|1|9|8\n";
    for(int i=0;i<4;++i) {
        const auto &q=ps[static_cast<size_t>(i)];
        const auto &p0=q.pad;
        out+="P|"+QByteArray::number(i)+"|"+p0.reference.toUtf8()+"|"+
             p0.pin.toUtf8()+"|1|"+QByteArray::number(p0.x_nm/254)+"|"+
             QByteArray::number(p0.y_nm/254)+"|"+QByteArray::number(p0.width_nm/254)+
             "|"+QByteArray::number(p0.height_nm/254)+"|"+p0.shape.toUtf8()+"\n";
    }
    for(int i=0;i<4;++i) {
        const auto &q=ps[static_cast<size_t>(i)];
        out+="T|"+QByteArray::number(i)+"|1|"+net.toUtf8()+"|"+
             QByteArray::number(q.pad.x_nm/254)+"|"+QByteArray::number(q.pad.y_nm/254)+"|"+
             QByteArray::number(q.jointX/254)+"|"+QByteArray::number(q.jointY/254)+"|"+
             QByteArray::number(q.trace.width_nm/254)+"\n";
    }
    out+="T|4|1|"+net.toUtf8()+"|"+QByteArray::number(x.jointX/254)+"|"+
         QByteArray::number(x.jointY/254)+"|"+QByteArray::number(y.jointX/254)+"|"+
         QByteArray::number(y.jointY/254)+"|"+QByteArray::number(bridge.width_nm/254)+"\n";
    for(int c=0;c<2;++c) {
        const int p0=2*c,t=p0+4,e=3*c;
        out+="E|"+QByteArray::number(e)+"|"+QByteArray::number(p0)+"|"+
             QByteArray::number(t)+"|PAD_TRACE_LOCAL_COPPER\n";
        out+="E|"+QByteArray::number(e+1)+"|"+QByteArray::number(p0+1)+"|"+
             QByteArray::number(t+1)+"|PAD_TRACE_LOCAL_COPPER\n";
        out+="E|"+QByteArray::number(e+2)+"|"+QByteArray::number(t)+"|"+
             QByteArray::number(t+1)+"|EXACT_TRACE_JUNCTION_LOCAL_COPPER\n";
    }
    out+="E|6|4|8|EXACT_BRIDGE_JUNCTION_LOCAL_COPPER\n"
         "E|7|8|6|EXACT_BRIDGE_JUNCTION_LOCAL_COPPER\n"
         "COUNT|NODES|9|EDGES|8|LOCAL_COMPONENTS|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n"
         "END|SOURCE_LOCAL_BRIDGE_ONLY_NO_GLOBAL_PASS\n";
    return out;
}
inline bool decode(const QByteArray &wire,const QByteArray &src,const PreviewImportPanel *p) {
    return !wire.isEmpty() && wire.size()<=16384 && wire==reconstruct(src,p);
}
} // namespace d041
