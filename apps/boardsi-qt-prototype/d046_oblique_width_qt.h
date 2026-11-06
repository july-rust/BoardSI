// D_046 independent finite-width oblique SOURCE-only geometry receipt.
#pragma once
#include "d040_multi_component_qt.h"
#include <array>
#include <algorithm>
namespace d046 {
struct Rect {
    qint64 x0,y0,x1,y1;
    bool touches(const Rect &o)const {
        return x0<=o.x1 && o.x0<=x1 && y0<=o.y1 && o.y0<=y1;
    }
};
// Independent exact SOURCE-tick segment projection, intentionally not Rust's edge list.
// Before products all coordinates are in 0.01 mil ticks, bounded to [19000..27000]
// x and [29000..41000] y (max abs delta 12000 ticks); qint64 products stay < 2^63.
inline bool obliquePositive(const ImportedTrack &a,const ImportedTrack &b) {
    using P=std::array<qint64,2>;
    const auto valid=[](const ImportedTrack &t) {
        if(t.width_nm<=0 || t.width_nm>10000*254LL || t.width_nm%254!=0)return false;
        for(qint64 x:{t.x0_nm,t.x1_nm})
            if(x%254!=0 || x<19000*254LL || x>27000*254LL)return false;
        for(qint64 y:{t.y0_nm,t.y1_nm})
            if(y%254!=0 || y<29000*254LL || y>41000*254LL)return false;
        return true;
    };
    if(!valid(a)||!valid(b))return false;
    const P p0={a.x0_nm/254,a.y0_nm/254}, p1={a.x1_nm/254,a.y1_nm/254};
    const P q0={b.x0_nm/254,b.y0_nm/254}, q1={b.x1_nm/254,b.y1_nm/254};
    const auto sub=[](P u,P v)->P {return {u[0]-v[0],u[1]-v[1]};};
    const auto dot=[](P u,P v)->qint64 {return u[0]*v[0]+u[1]*v[1];};
    const auto cross=[](P u,P v)->qint64 {return u[0]*v[1]-u[1]*v[0];};
    const auto onSegment=[](P u,P v,P p) {
        return p[0]>=std::min(u[0],v[0]) && p[0]<=std::max(u[0],v[0]) &&
               p[1]>=std::min(u[1],v[1]) && p[1]<=std::max(u[1],v[1]);
    };
    const P u=sub(p1,p0),v=sub(q1,q0);
    if(dot(u,u)==0 || dot(v,v)==0 || cross(u,v)==0 || dot(u,v)==0)return false;
    const qint64 o1=cross(u,sub(q0,p0)), o2=cross(u,sub(q1,p0));
    const qint64 o3=cross(v,sub(p0,q0)), o4=cross(v,sub(p1,q0));
    if((o1==0 && onSegment(p0,p1,q0)) ||
       (o2==0 && onSegment(p0,p1,q1)) ||
       (o3==0 && onSegment(q0,q1,p0)) ||
       (o4==0 && onSegment(q0,q1,p1)) ||
       (((o1>0 && o2<0)||(o1<0 && o2>0)) &&
        ((o3>0 && o4<0)||(o3<0 && o4>0))))return false;
    const qint64 sum=(a.width_nm+b.width_nm)/254;
    const qint64 limit=sum*sum;
    const auto pointNear=[&](P p,P s,P t) {
        const P d=sub(t,s),w=sub(p,s);
        const qint64 len=dot(d,d),pr=dot(w,d);
        if(len==0)return false;
        if(pr<=0)return 4*dot(w,w)<limit;
        if(pr>=len){const P tail=sub(p,t);return 4*dot(tail,tail)<limit;}
        const qint64 area=cross(d,w);
        return 4*area*area<limit*len;
    };
    return pointNear(p0,q0,q1) || pointNear(p1,q0,q1) ||
           pointNear(q0,p0,p1) || pointNear(q1,p0,p1);
}
inline Rect traceBox(const ImportedTrack &t) {
    return {2*std::min(t.x0_nm,t.x1_nm)-t.width_nm,
            2*std::min(t.y0_nm,t.y1_nm)-t.width_nm,
            2*std::max(t.x0_nm,t.x1_nm)+t.width_nm,
            2*std::max(t.y0_nm,t.y1_nm)+t.width_nm};
}
inline Rect padBox(const ImportedPad &p) {
    return {2*p.x_nm-p.width_nm,2*p.y_nm-p.height_nm,
            2*p.x_nm+p.width_nm,2*p.y_nm+p.height_nm};
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
    const std::array<qint64,4> xs={20000,20000,26000,26000};
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
    if(extraTrack.y0_nm==extraTrack.y1_nm ||
       extraTrack.x0_nm==extraTrack.x1_nm ||
       extraTrack.x0_nm<22000*254LL || extraTrack.x0_nm>24000*254LL ||
       extraTrack.x1_nm<22000*254LL || extraTrack.x1_nm>24000*254LL ||
       extraTrack.y0_nm<33000*254LL || extraTrack.y0_nm>37000*254LL ||
       extraTrack.y1_nm<33000*254LL || extraTrack.y1_nm>37000*254LL ||
       extraTrack.width_nm>10000*254LL)return {};
    const auto envelope=traceBox(extraTrack);
    for(const auto &p:paths) {
        if(!obliquePositive(extraTrack,p.trace) ||
           envelope.touches(padBox(p.pad)))return {};
    }
    QByteArray out="BOARDSI_D046_OBLIQUE_WIDTH_CONTACT_V1|"+sha+
        "|1|SOURCE_LOCAL_FINITE_WIDTH_ONLY|P009_UNSUPPORTED|SI_NOT_RUN"
        "|GLOBAL_ELECTRICAL_NOT_VERIFIED\nC|0|U1+U2|"+net+"|1|9|10\n";
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
    for(int i=0;i<4;++i)
        out+="E|"+QByteArray::number(6+i)+"|"+QByteArray::number(4+i)+
             "|8|POSITIVE_FINITE_WIDTH_OBLIQUE_CONTACT\n";
    out+="COUNT|NODES|9|EDGES|10|LOCAL_COMPONENTS|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n"
         "END|SOURCE_LOCAL_FINITE_WIDTH_ONLY_NO_GLOBAL_PASS\n";
    return out;
}
inline bool decode(const QByteArray &wire,const QByteArray &source,
                   const PreviewImportPanel *panel) {
    return !wire.isEmpty() && wire.size()<=16384 && wire==reconstruct(source,panel);
}
} // namespace d046
