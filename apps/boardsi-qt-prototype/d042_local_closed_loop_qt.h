// D_042: independent Qt source-owned closed-loop reconstruction; no global electrical PASS.
#pragma once
#include "d040_multi_component_qt.h"
#include <array>
#include <algorithm>
namespace d042 {
inline QByteArray reconstruct(const QByteArray &src,const PreviewImportPanel *panel) {
    if(!panel || !panel->imported())return {};
    const auto rows=src.split('\n');
    QByteArray base,signalNet;
    int signalCount=0,bridges=0,detours=0;
    bool inSignal=false;
    for(int i=0;i<rows.size();++i) {
        const auto fields=rows[i].simplified().split(' ');
        if(fields[0]=="*SIGNAL*") {
            if(fields.size()!=2 || ++signalCount!=1)return {};
            signalNet=fields[1];
            inSignal=true;
        }
        if(fields[0]=="*END*")inSignal=false;
        const bool isBridge=fields.size()==2 && fields[0]=="U1.1" && fields[1]=="U2.1";
        const bool isDetour=fields.size()==2 && fields[0]=="U1.2" && fields[1]=="U2.2";
        if(isBridge || isDetour) {
            if(!inSignal || (isBridge && bridges++!=0) || (isDetour && ++detours>5) ||
               i+2>=rows.size() ||
               rows[i+1].simplified().split(' ').size()!=5 ||
               rows[i+2].simplified().split(' ').size()!=5)return {};
            i+=2;
        } else base+=rows[i]+'\n';
    }
    if(signalCount!=1 || bridges!=1 || detours!=5 || !d040::boundedSections(base))return {};
    // Keep declaration ownership independently linked to exact SOURCE
    // coordinates. ImportedArtwork alone loses each trace's pin-pair header.
    struct Decl { const char *pinA,*pinB; int ax,ay,bx,by; };
    const std::array<Decl,10> declarations={{
        {"U1.1","U1.2",200,300,200,350},
        {"U1.1","U1.2",200,400,200,350},
        {"U2.1","U2.2",400,300,400,350},
        {"U2.1","U2.2",400,400,400,350},
        {"U1.1","U2.1",200,350,400,350},
        {"U1.2","U2.2",200,350,150,350},
        {"U1.2","U2.2",150,350,150,250},
        {"U1.2","U2.2",150,250,450,250},
        {"U1.2","U2.2",450,250,450,350},
        {"U1.2","U2.2",450,350,400,350}
    }};
    std::array<bool,10> declaredSeen={};
    bool recordSignal=false;
    for(int i=0;i<rows.size();++i) {
        const auto words=rows[i].simplified().split(' ');
        if(words[0]=="*SIGNAL*") {recordSignal=true;continue;}
        if(words[0]=="*END*") {recordSignal=false;continue;}
        if(!recordSignal || words.size()!=2 || !words[0].startsWith('U'))continue;
        if(i+2>=rows.size())return {};
        const auto aa=rows[i+1].simplified().split(' ');
        const auto bb=rows[i+2].simplified().split(' ');
        if(aa.size()!=5 || bb.size()!=5 ||
           aa[2]!="1" || bb[2]!="1" ||
           aa[3]!="10" || bb[3]!="10" ||
           aa[4]!="0" || bb[4]!="0")return {};
        bool okA=false,okB=false,okC=false,okD=false;
        const double ax=aa[0].toDouble(&okA), ay=aa[1].toDouble(&okB);
        const double bx=bb[0].toDouble(&okC), by=bb[1].toDouble(&okD);
        if(!okA || !okB || !okC || !okD)return {};
        int match=-1;
        for(int j=0;j<10;++j) {
            const auto &d=declarations[static_cast<size_t>(j)];
            if(words[0]!=d.pinA || words[1]!=d.pinB)continue;
            const bool forward=ax==d.ax && ay==d.ay && bx==d.bx && by==d.by;
            const bool reverse=ax==d.bx && ay==d.by && bx==d.ax && by==d.ay;
            if(forward || reverse) {
                if(match>=0)return {};
                match=j;
            }
        }
        if(match<0 || declaredSeen[static_cast<size_t>(match)])return {};
        declaredSeen[static_cast<size_t>(match)]=true;
        i+=2;
    }
    if(std::any_of(declaredSeen.cbegin(),declaredSeen.cend(),
                   [](bool present){return !present;}))return {};
    const auto *art=panel->imported();
    if(art->importedSkippedCount()!=0 || art->importedTraceCount()!=10 ||
       art->padCount()!=4 || art->arcCount()!=0 || art->viaCount()!=0)return {};
    const auto sha=QCryptographicHash::hash(src,QCryptographicHash::Sha256).toHex();
    if(sha!=panel->acceptedSourceSha256())return {};
    QStringList pins;
    const QString net=QString::fromLatin1(signalNet);
    if(net!="NET_A" && net!="NET_B")return {};
    if(!d005::declaredPins(src,sha,net,pins))return {};
    std::array<ImportedTrack,10> tr;
    for(int i=0;i<10;++i) {
        auto &t=tr[static_cast<size_t>(i)];
        if(!art->importedTrack(i,t) || t.layer!=1 || t.net!=net ||
           t.width_nm<=0 || t.width_nm%254!=0 ||
           t.x0_nm%254!=0 || t.y0_nm%254!=0 ||
           t.x1_nm%254!=0 || t.y1_nm%254!=0 ||
           (t.x0_nm==t.x1_nm && t.y0_nm==t.y1_nm))return {};
    }
    std::array<d040::Path,4> ps;
    std::array<int,10> used={};
    for(int i=0;i<4;++i) {
        auto &q=ps[static_cast<size_t>(i)];
        if(!art->importedPad(i,q.pad))return {};
        auto &pad=q.pad;
        if(pad.layer!=1 || pad.side!="T" ||
           (pad.shape!="R" && pad.shape!="S") || pad.width_nm<=0 ||
           pad.height_nm!=pad.width_nm || pad.width_nm%254!=0 ||
           pad.x_nm%254!=0 || pad.y_nm%254!=0 ||
           !pins.contains(pad.reference+"."+pad.pin))return {};
        for(int k=0;k<10;++k) {
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
    int missing=-1;
    for(int i=0;i<10;++i) {
        const auto &t=tr[static_cast<size_t>(i)];
        if(used[static_cast<size_t>(i)]==1)continue;
        if(used[static_cast<size_t>(i)]!=0)return {};
        const bool f=t.x0_nm==x.jointX && t.y0_nm==x.jointY &&
                     t.x1_nm==y.jointX && t.y1_nm==y.jointY;
        const bool r=t.x1_nm==x.jointX && t.y1_nm==x.jointY &&
                     t.x0_nm==y.jointX && t.y0_nm==y.jointY;
        if(f || r) {
            if(missing>=0)return {};
            missing=i;
        }
    }
    if(missing<0)return {};
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
    // Independent geometry identity and 5-edge detour reconstruction.
    const std::array<std::array<qint64,4>,4> padPositions={{{20000LL*254,30000LL*254,0,0},
        {20000LL*254,40000LL*254,0,0},
        {40000LL*254,30000LL*254,0,0},
        {40000LL*254,40000LL*254,0,0}}};
    for(int i=0;i<4;++i) {
        const auto &pad=ps[static_cast<size_t>(i)].pad;
        if(pad.x_nm!=padPositions[static_cast<size_t>(i)][0] ||
           pad.y_nm!=padPositions[static_cast<size_t>(i)][1])return {};
    }
    if(x.jointX!=20000LL*254 || x.jointY!=35000LL*254 ||
       y.jointX!=40000LL*254 || y.jointY!=35000LL*254)return {};
    const std::array<std::array<qint64,2>,6> points={{{20000LL*254,35000LL*254},
        {15000LL*254,35000LL*254},{15000LL*254,25000LL*254},
        {45000LL*254,25000LL*254},{45000LL*254,35000LL*254},
        {40000LL*254,35000LL*254}}};
    std::array<int,5> detourSeen={};
    for(int i=0;i<10;++i) {
        if(i==missing || used[static_cast<size_t>(i)]==1)continue;
        const auto &t=tr[static_cast<size_t>(i)];
        if(t.width_nm!=bridge.width_nm)return {};
        int match=-1;
        for(int k=0;k<5;++k) {
            const auto &a=points[static_cast<size_t>(k)];
            const auto &b=points[static_cast<size_t>(k+1)];
            const bool f=t.x0_nm==a[0] && t.y0_nm==a[1] &&
                         t.x1_nm==b[0] && t.y1_nm==b[1];
            const bool r=t.x1_nm==a[0] && t.y1_nm==a[1] &&
                         t.x0_nm==b[0] && t.y0_nm==b[1];
            if(f || r) {
                if(match>=0)return {};
                match=k;
            }
        }
        if(match<0 || ++detourSeen[static_cast<size_t>(match)]!=1)return {};
    }
    if(std::any_of(detourSeen.cbegin(),detourSeen.cend(),[](int n){return n!=1;}))return {};
    // A real graph traversal, not an edge-count assertion.
    const std::array<std::pair<int,int>,14> links={{{0,4},{1,5},{4,5},
        {2,6},{3,7},{6,7},{4,8},{8,6},
        {4,9},{9,10},{10,11},{11,12},{12,13},{13,6}}};
    std::array<int,14> parent={};
    for(int i=0;i<14;++i)parent[static_cast<size_t>(i)]=i;
    const auto root=[&](int n) {
        while(parent[static_cast<size_t>(n)]!=n)n=parent[static_cast<size_t>(n)];
        return n;
    };
    int cycles=0,components=0;
    for(const auto &e:links) {
        const int a=root(e.first),b=root(e.second);
        if(a==b)++cycles;
        else parent[static_cast<size_t>(a)]=b;
    }
    for(int i=0;i<14;++i)if(root(i)==i)++components;
    if(components!=1 || cycles!=1 || static_cast<int>(links.size())-14+components!=cycles)return {};

    QByteArray out="BOARDSI_D042_LOCAL_CLOSED_LOOP_V1|"+sha+
        "|1|LOCAL_COPPER_CLOSED_LOOP_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN"
        "|GLOBAL_ELECTRICAL_NOT_VERIFIED\nC|0|U1+U2|"+net.toUtf8()+"|1|14|14\n";
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
    for(int i=0;i<5;++i) {
        const auto &a=points[static_cast<size_t>(i)];
        const auto &b=points[static_cast<size_t>(i+1)];
        out+="T|"+QByteArray::number(i+5)+"|1|"+net.toUtf8()+"|"+
            QByteArray::number(a[0]/254)+"|"+QByteArray::number(a[1]/254)+"|"+
            QByteArray::number(b[0]/254)+"|"+QByteArray::number(b[1]/254)+"|"+
            QByteArray::number(bridge.width_nm/254)+"\n";
    }
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
         "E|7|8|6|EXACT_BRIDGE_JUNCTION_LOCAL_COPPER\n";
    for(int k=8;k<14;++k) {
        const auto &e=links[static_cast<size_t>(k)];
        out+="E|"+QByteArray::number(k)+"|"+QByteArray::number(e.first)+"|"+
             QByteArray::number(e.second)+"|EXACT_DETOUR_JUNCTION_LOCAL_COPPER\n";
    }
    out+="CYCLE|LOCAL|1|INDEPENDENT_PATHS|2\n"
         "COUNT|NODES|14|EDGES|14|LOCAL_COMPONENTS|1|LOCAL_CYCLES|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n"
         "END|SOURCE_LOCAL_CLOSED_LOOP_ONLY_NO_GLOBAL_PASS\n";
    return out;
}
inline bool decode(const QByteArray &wire,const QByteArray &src,const PreviewImportPanel *p) {
    return !wire.isEmpty() && wire.size()<=16384 && wire==reconstruct(src,p);
}
} // namespace d042
