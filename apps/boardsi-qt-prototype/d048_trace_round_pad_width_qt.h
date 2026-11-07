// D_048 independent finite-width trace-to-round-pad SOURCE-only geometry receipt.
#pragma once
#include "d040_multi_component_qt.h"
#include <array>
#include <algorithm>
namespace d048 {
// The bounded 8-mil track centerline is strictly right of round U1.1.
// Minimum distance to the rectangle equals distance to its right edge.
// Reconstruct independently; do not trust Rust geometry edge list.
inline bool positiveTraceRound(const ImportedTrack &t,const ImportedPad &pad) {
    using P=std::array<qint64,2>;
    if(pad.shape!="R" || pad.side!="T" || pad.layer!=1 ||
       pad.width_nm!=4000*254LL || pad.height_nm!=4000*254LL ||
       pad.x_nm!=20000*254LL || pad.y_nm!=30000*254LL ||
       t.width_nm!=800*254LL || t.layer!=1)return false;
    for(qint64 x:{t.x0_nm,t.x1_nm})
        if(x%254!=0 || x<21400*254LL || x>23000*254LL)return false;
    for(qint64 y:{t.y0_nm,t.y1_nm})
        if(y%254!=0 || y<27000*254LL || y>33000*254LL)return false;
    const P a={t.x0_nm/254,t.y0_nm/254},b={t.x1_nm/254,t.y1_nm/254};
    if(a==b)return false;
    const P center={pad.x_nm/254,pad.y_nm/254};
    const auto sub=[](P u,P v)->P{return {u[0]-v[0],u[1]-v[1]};};
    const auto dot=[](P u,P v)->qint64{return u[0]*v[0]+u[1]*v[1];};
    const auto cross=[](P u,P v)->qint64{return u[0]*v[1]-u[1]*v[0];};
    const P delta=sub(b,a),w=sub(center,a);
    const qint64 length=dot(delta,delta);
    if(!length)return false;
    const qint64 projection=dot(w,delta);
    qint64 numerator=0,denominator=1;
    if(projection<=0)numerator=dot(w,w);
    else if(projection>=length)numerator=dot(sub(center,b),sub(center,b));
    else {
        const qint64 area=cross(delta,w);
        numerator=area*area;
        denominator=length;
    }
    // Entire bounded SOURCE domain keeps these products within signed qint64.
    // A round-capped track must overlap the circle positively, while the
    // centerline itself stays outside the round SMD copper disc.
    return numerator>4000000LL*denominator &&
           numerator<5760000LL*denominator;
}
inline QByteArray reconstruct(const QByteArray &source,const PreviewImportPanel *panel) {
    if(!panel || !panel->imported())return {};
    QByteArray baseline,net;
    int signalCount=0,candidates=0;
    bool inSignal=false;
    const auto rows=source.split('\n');
    for(int i=0;i<rows.size();++i) {
        const auto f=rows[i].simplified().split(' ');
        if(f[0]=="*SIGNAL*") {
            if(f.size()!=2 || ++signalCount!=1)return {};
            net=f[1];inSignal=true;
        } else if(f[0]=="*END*")inSignal=false;
        if(f.size()==2 && f[0]=="U1.1" && f[1]=="U2.1") {
            if(!inSignal || candidates++ || i+2>=rows.size() ||
               rows[i+1].simplified().split(' ').size()!=5 ||
               rows[i+2].simplified().split(' ').size()!=5)return {};
            i+=2;
        } else baseline+=rows[i]+'\n';
    }
    if(signalCount!=1 || candidates!=1 || !d040::boundedSections(baseline) ||
       (net!="NET_A" && net!="NET_B"))return {};
    const auto *art=panel->imported();
    if(art->importedSkippedCount()!=0 || art->importedTraceCount()!=5 ||
       art->padCount()!=4 || art->arcCount()!=0 || art->viaCount()!=0)return {};
    const auto sha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    if(sha!=panel->acceptedSourceSha256())return {};
    QStringList declared;
    const QString qNet=QString::fromLatin1(net);
    if(!d005::declaredPins(source,sha,qNet,declared) ||
       !declared.contains("U1.1") || !declared.contains("U2.1"))return {};
    std::array<ImportedTrack,5> tracks;
    for(int i=0;i<5;++i) {
        auto &t=tracks[static_cast<size_t>(i)];
        if(!art->importedTrack(i,t) || t.layer!=1 || t.net!=qNet ||
           t.width_nm<=0 || t.width_nm%254!=0 ||
           t.x0_nm%254!=0 || t.y0_nm%254!=0 ||
           t.x1_nm%254!=0 || t.y1_nm%254!=0 ||
           (t.x0_nm==t.x1_nm && t.y0_nm==t.y1_nm))return {};
    }
    std::array<d040::Path,4> paths;
    std::array<int,5> used={};
    for(int i=0;i<4;++i) {
        auto &p=paths[static_cast<size_t>(i)];
        if(!art->importedPad(i,p.pad))return {};
        const auto &pad=p.pad;
        if(pad.layer!=1 || pad.side!="T" ||
           (pad.shape!="R" && pad.shape!="S") ||
           pad.width_nm<=0 || pad.height_nm!=pad.width_nm ||
           pad.width_nm%254!=0 || pad.x_nm%254!=0 || pad.y_nm%254!=0 ||
           !declared.contains(pad.reference+"."+pad.pin))return {};
        for(int j=0;j<5;++j) {
            const auto &t=tracks[static_cast<size_t>(j)];
            const bool a=t.x0_nm==pad.x_nm && t.y0_nm==pad.y_nm;
            const bool b=t.x1_nm==pad.x_nm && t.y1_nm==pad.y_nm;
            if(a==b)continue;
            if(p.trackIndex>=0 || t.width_nm>pad.width_nm/2)return {};
            p.trackIndex=j;p.trace=t;
            p.jointX=a?t.x1_nm:t.x0_nm;
            p.jointY=a?t.y1_nm:t.y0_nm;
            ++used[static_cast<size_t>(j)];
        }
        if(p.trackIndex<0)return {};
    }
    int extra=-1;
    for(int i=0;i<5;++i) {
        if(used[static_cast<size_t>(i)]==0) {
            if(extra>=0)return {};
            extra=i;
        } else if(used[static_cast<size_t>(i)]!=1)return {};
    }
    if(extra<0)return {};
    std::sort(paths.begin(),paths.end(),[](const auto &a,const auto &b) {
        return std::make_pair(a.pad.reference,a.pad.pin) <
               std::make_pair(b.pad.reference,b.pad.pin);
    });
    const std::array<qint64,4> xs={20000,20000,40000,40000};
    const std::array<qint64,4> ys={30000,40000,30000,40000};
    for(int i=0;i<4;++i) {
        const auto &p=paths[static_cast<size_t>(i)];
        if(p.pad.reference!=(i<2?"U1":"U2") ||
           p.pad.pin!=(i%2==0?"1":"2") ||
           p.pad.x_nm!=xs[static_cast<size_t>(i)]*254 ||
           p.pad.y_nm!=ys[static_cast<size_t>(i)]*254 ||
           p.jointX!=xs[static_cast<size_t>(i)]*254 ||
           p.jointY!=35000*254 || p.trace.width_nm!=1000*254 ||
           p.trace.x0_nm!=p.trace.x1_nm ||
           p.trace.y0_nm==p.trace.y1_nm)return {};
    }
    const auto &extraTrack=tracks[static_cast<size_t>(extra)];
    if(extraTrack.width_nm!=800*254LL ||
       !positiveTraceRound(extraTrack,paths[0].pad))return {};
    // Bounded x>=21400, width 800: no collision with the x=20000
    // vertical rails; U2 is remote in this synthetic fixture.
    QByteArray out="BOARDSI_D048_TRACE_ROUND_PAD_WIDTH_V1|"+sha+
        "|1|SOURCE_LOCAL_TRACE_ROUND_PAD_FINITE_WIDTH_ONLY|P009_UNSUPPORTED|SI_NOT_RUN"
        "|GLOBAL_ELECTRICAL_NOT_VERIFIED\nC|0|U1|"+net+"|1|5|4\nC|1|U2|"+net+"|1|4|3\n";
    for(int i=0;i<4;++i) {
        const auto &p=paths[static_cast<size_t>(i)].pad;
        out+="P|"+QByteArray::number(i)+"|"+p.reference.toUtf8()+"|"+
             p.pin.toUtf8()+"|1|"+QByteArray::number(p.x_nm/254)+"|"+
             QByteArray::number(p.y_nm/254)+"|"+QByteArray::number(p.width_nm/254)+
             "|"+QByteArray::number(p.height_nm/254)+"|"+p.shape.toUtf8()+"\n";
    }
    for(int i=0;i<4;++i) {
        const auto &p=paths[static_cast<size_t>(i)];
        out+="T|"+QByteArray::number(i)+"|1|"+net+"|"+
             QByteArray::number(p.pad.x_nm/254)+"|"+
             QByteArray::number(p.pad.y_nm/254)+"|"+
             QByteArray::number(p.jointX/254)+"|"+
             QByteArray::number(p.jointY/254)+"|"+
             QByteArray::number(p.trace.width_nm/254)+"\n";
    }
    out+="T|4|1|"+net+"|"+
         QByteArray::number(extraTrack.x0_nm/254)+"|"+
         QByteArray::number(extraTrack.y0_nm/254)+"|"+
         QByteArray::number(extraTrack.x1_nm/254)+"|"+
         QByteArray::number(extraTrack.y1_nm/254)+"|"+
         QByteArray::number(extraTrack.width_nm/254)+"\n";
    for(int c=0;c<2;++c) {
        const int p=c*2,t=p+4,e=c*3;
        out+="E|"+QByteArray::number(e)+"|"+QByteArray::number(p)+"|"+
             QByteArray::number(t)+"|PAD_TRACE_LOCAL_COPPER\n";
        out+="E|"+QByteArray::number(e+1)+"|"+QByteArray::number(p+1)+"|"+
             QByteArray::number(t+1)+"|PAD_TRACE_LOCAL_COPPER\n";
        out+="E|"+QByteArray::number(e+2)+"|"+QByteArray::number(t)+"|"+
             QByteArray::number(t+1)+"|EXACT_TRACE_JUNCTION_LOCAL_COPPER\n";
    }
    out+="E|6|0|8|POSITIVE_FINITE_WIDTH_TRACE_ROUND_PAD_CONTACT\n";
    out+="COUNT|NODES|9|EDGES|7|LOCAL_COMPONENTS|2|GLOBAL_ELECTRICAL|NOT_VERIFIED\n"
         "END|SOURCE_LOCAL_TRACE_ROUND_PAD_FINITE_WIDTH_ONLY_NO_GLOBAL_PASS\n";
    return out;
}
inline bool decode(const QByteArray &wire,const QByteArray &source,
                   const PreviewImportPanel *panel) {
    return !wire.isEmpty() && wire.size()<=16384 && wire==reconstruct(source,panel);
}
} // namespace d048
