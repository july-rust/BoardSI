// D_050: independent SOURCE-exact mixed-pad bridge / 1-component graph.
#pragma once
#include "d040_multi_component_qt.h"
#include "d047_trace_square_pad_width_qt.h"
#include "d048_trace_round_pad_width_qt.h"
#include <array>
#include <algorithm>

namespace d050 {
inline bool sourceCoordTicks(QByteArray value, qint64 &ticks) {
    const bool negative=value.startsWith('-');
    if(negative || value.startsWith('+'))value.remove(0,1);
    const auto parts=value.split('.');
    if(parts.isEmpty() || parts.size()>2 || parts[0].isEmpty() ||
       (parts.size()==2 && (parts[1].isEmpty() || parts[1].size()>2)))
        return false;
    for(const auto &part:parts) {
        for(const char digit:part) {
            if(digit<'0' || digit>'9')return false;
        }
    }
    bool ok=false;
    const qint64 integral=parts[0].toLongLong(&ok);
    if(!ok || integral>56000)return false;
    qint64 fraction=0;
    if(parts.size()==2) {
        fraction=parts[1].toLongLong(&ok);
        if(!ok)return false;
        if(parts[1].size()==1)fraction*=10;
    }
    const qint64 magnitude=integral*100+fraction;
    if(magnitude>5600000)return false;
    ticks=negative?-magnitude:magnitude;
    return true;
}

inline bool sourcePairTrack(const ImportedTrack &track,
                            const std::array<QByteArray,2> &vertices) {
    std::array<std::array<qint64,2>,2> points{};
    for(int i=0;i<2;++i) {
        const auto f=vertices[static_cast<size_t>(i)].simplified().split(' ');
        if(f.size()!=5 ||
           !sourceCoordTicks(f[0],points[static_cast<size_t>(i)][0]) ||
           !sourceCoordTicks(f[1],points[static_cast<size_t>(i)][1]))return false;
        // Match the PADS SOURCE straight-segment grammar, not literal tokens:
        // optional '+' and 0..2 decimal places are legal; terminal layer 65
        // inherits the preceding layer only on the second vertex.
        qint64 width=0;
        bool layerOk=false,flagOk=false;
        const qint64 layer=f[2].toLongLong(&layerOk,10);
        const qint64 flag=f[4].toLongLong(&flagOk,10);
        if(!sourceCoordTicks(f[3],width) || width!=800 ||
           !layerOk || !flagOk ||
           (i==0 ? layer!=1 : (layer!=1 && layer!=65)) ||
           flag<0 || flag>32767 || (flag & ~qint64(0x0680))!=0)return false;
    }
    const qint64 ax=track.x0_nm/254,ay=track.y0_nm/254;
    const qint64 bx=track.x1_nm/254,by=track.y1_nm/254;
    return (ax==points[0][0] && ay==points[0][1] &&
            bx==points[1][0] && by==points[1][1]) ||
           (ax==points[1][0] && ay==points[1][1] &&
            bx==points[0][0] && by==points[0][1]);
}

// The narrow positive geometry witness is intentionally checked with integer
// squared distances, not bounding boxes or floating-point tolerances.
inline bool bridgeVerticesCandidate(const std::array<QByteArray,2> &vertices) {
    qint64 x[2]={},y[2]={};
    for(int i=0;i<2;++i) {
        const auto f=vertices[static_cast<size_t>(i)].simplified().split(' ');
        if(f.size()!=5 || !sourceCoordTicks(f[0],x[i]) ||
           !sourceCoordTicks(f[1],y[i]))return false;
    }
    const qint64 a=std::min(x[0],x[1]),b=std::max(x[0],x[1]);
    return y[0]==30000 && y[1]==30000 && a>22400 && a<23000 &&
           b>37600 && b<38000;
}
inline bool positiveBridgeTrace(const ImportedTrack &track) {
    if(track.layer!=1 || track.width_nm!=800LL*254 ||
       track.x0_nm%254!=0 || track.y0_nm%254!=0 ||
       track.x1_nm%254!=0 || track.y1_nm%254!=0)return false;
    const qint64 x0=track.x0_nm/254,x1=track.x1_nm/254;
    const qint64 y0=track.y0_nm/254,y1=track.y1_nm/254;
    const qint64 a=std::min(x0,x1),b=std::max(x0,x1);
    if(y0!=30000 || y1!=30000 || a<=22400 || a>=23000 ||
       b<=37600 || b>=38000)return false;
    const qint64 dTrace=a-22200,dU1=a-20000,dU2=40000-b;
    return dTrace*dTrace<800LL*800 && dU1*dU1>2400LL*2400 &&
           dU2*dU2>2000LL*2000 && dU2*dU2<2400LL*2400;
}

inline QByteArray reconstruct(const QByteArray &source,const PreviewImportPanel *panel) {
    if(!panel || !panel->imported() || source.isEmpty() ||
       source.size()>8*1024*1024)return {};
    QByteArray baseline,net;
    int signalCount=0,roundCount=0,squareCount=0,bridgeCount=0;
    bool inSignal=false;
    std::array<QByteArray,2> roundVertices{},squareVertices{},bridgeVertices{};
    const auto rows=source.split('\n');
    for(int i=0;i<rows.size();++i) {
        const auto fields=rows[i].simplified().split(' ');
        if(fields[0]=="*SIGNAL*") {
            if(fields.size()!=2 || ++signalCount!=1)return {};
            net=fields[1];inSignal=true;
        } else if(fields[0]=="*END*")inSignal=false;
        const bool round=fields.size()==2 && fields[0]=="U1.1" && fields[1]=="U2.1";
        const bool square=fields.size()==2 && fields[0]=="U1.2" && fields[1]=="U2.2";
        if(round || square) {
            if(!inSignal || i+2>=rows.size() ||
               rows[i+1].simplified().split(' ').size()!=5 ||
               rows[i+2].simplified().split(' ').size()!=5)return {};
            const std::array<QByteArray,2> vertices={rows[i+1],rows[i+2]};
            if(round && bridgeVerticesCandidate(vertices)) {
                if(++bridgeCount!=1)return {};
                bridgeVertices=vertices;
            } else if(round) {
                if(++roundCount!=1)return {};
                roundVertices=vertices;
            } else {
                if(++squareCount!=1)return {};
                squareVertices=vertices;
            }
            i+=2;
        } else baseline+=rows[i]+'\n';
    }
    if(signalCount!=1 || roundCount!=1 || squareCount!=1 || bridgeCount!=1 ||
       (net!="NET_A" && net!="NET_B") || !d040::boundedSections(baseline))return {};
    const auto *art=panel->imported();
    if(art->importedSkippedCount()!=0 || art->importedTraceCount()!=7 ||
       art->padCount()!=4 || art->arcCount()!=0 || art->viaCount()!=0)return {};
    const auto sha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    if(panel->acceptedSourceSha256()!=sha)return {};
    const QString qNet=QString::fromLatin1(net);
    QStringList declared;
    if(!d005::declaredPins(source,sha,qNet,declared) ||
       !declared.contains("U1.1") || !declared.contains("U1.2") ||
       !declared.contains("U2.1") || !declared.contains("U2.2"))return {};
    std::array<ImportedTrack,7> tracks;
    for(int i=0;i<7;++i) {
        auto &t=tracks[static_cast<size_t>(i)];
        if(!art->importedTrack(i,t) || t.layer!=1 || t.net!=qNet ||
           t.width_nm<=0 || t.width_nm%254!=0 ||
           t.x0_nm%254!=0 || t.y0_nm%254!=0 ||
           t.x1_nm%254!=0 || t.y1_nm%254!=0 ||
           (t.x0_nm==t.x1_nm && t.y0_nm==t.y1_nm))return {};
    }
    std::array<d040::Path,4> paths;
    std::array<int,7> used{};
    for(int i=0;i<4;++i) {
        auto &p=paths[static_cast<size_t>(i)];
        if(!art->importedPad(i,p.pad))return {};
        const auto &pad=p.pad;
        if(pad.layer!=1 || pad.side!="T" ||
           (pad.shape!="R" && pad.shape!="S") ||
           pad.width_nm<=0 || pad.width_nm!=pad.height_nm ||
           pad.x_nm%254!=0 || pad.y_nm%254!=0 ||
           !declared.contains(pad.reference+"."+pad.pin))return {};
        for(int j=0;j<7;++j) {
            const auto &t=tracks[static_cast<size_t>(j)];
            const bool atA=t.x0_nm==pad.x_nm && t.y0_nm==pad.y_nm;
            const bool atB=t.x1_nm==pad.x_nm && t.y1_nm==pad.y_nm;
            if(atA==atB)continue;
            if(p.trackIndex>=0 || t.width_nm>pad.width_nm/2)return {};
            p.trackIndex=j;p.trace=t;
            p.jointX=atA?t.x1_nm:t.x0_nm;
            p.jointY=atA?t.y1_nm:t.y0_nm;
            ++used[static_cast<size_t>(j)];
        }
        if(p.trackIndex<0)return {};
    }
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
        // Match pad-shape/size inventory, not only the two new contacts.
        if(p.pad.shape!=(i%2==0?"R":"S") ||
           p.pad.width_nm!=(i%2==0?4000:3000)*254LL)return {};
    }
    int roundIndex=-1,squareIndex=-1,bridgeIndex=-1,extras=0;
    for(int i=0;i<7;++i) {
        if(used[static_cast<size_t>(i)]!=0) {
            if(used[static_cast<size_t>(i)]!=1)return {};
            continue;
        }
        ++extras;
        const auto &trace=tracks[static_cast<size_t>(i)];
        if(sourcePairTrack(trace,roundVertices) &&
           d048::positiveTraceRound(trace,paths[0].pad)) {
            if(roundIndex>=0)return {};
            roundIndex=i;
        } else if(sourcePairTrack(trace,squareVertices) &&
                  d047::positiveTraceSquare(trace,paths[1].pad)) {
            if(squareIndex>=0)return {};
            squareIndex=i;
        } else if(sourcePairTrack(trace,bridgeVertices) &&
                  positiveBridgeTrace(trace)) {
            if(bridgeIndex>=0)return {};
            bridgeIndex=i;
        } else return {};
    }
    if(extras!=3 || roundIndex<0 || squareIndex<0 || bridgeIndex<0 ||
       roundIndex==squareIndex || roundIndex==bridgeIndex ||
       squareIndex==bridgeIndex)return {};
    // D_049 also accepts alternative Pad-contact cap witnesses. D_050's
    // new edge must be bound to the exact existing x=22200 vertical trace;
    // do not silently connect a moved supplementary contact.
    const auto exactEndpoints=[](const ImportedTrack &t,
                                 qint64 ax,qint64 ay,qint64 bx,qint64 by) {
        const bool forward=t.x0_nm==ax*254 && t.y0_nm==ay*254 &&
                           t.x1_nm==bx*254 && t.y1_nm==by*254;
        const bool backward=t.x0_nm==bx*254 && t.y0_nm==by*254 &&
                            t.x1_nm==ax*254 && t.y1_nm==ay*254;
        return forward || backward;
    };
    if(!exactEndpoints(tracks[static_cast<size_t>(roundIndex)],
                       22200,28500,22200,31500) ||
       !exactEndpoints(tracks[static_cast<size_t>(squareIndex)],
                       21700,39000,21700,41000))return {};
    // Bounded round y=27000..33000 and square y=38000..43500 regions
    // cannot touch each other, the baseline vertical rails, or remote U2.
    QByteArray out="BOARDSI_D050_MIXED_PAD_WIDTH_BRIDGE_V1|"+sha+
        "|1|SOURCE_LOCAL_MIXED_PAD_FINITE_WIDTH_BRIDGE_ONLY|P009_UNSUPPORTED|SI_NOT_RUN"
        "|GLOBAL_ELECTRICAL_NOT_VERIFIED\nC|0|U1_U2|"+net+"|1|11|10\n";
    for(int i=0;i<4;++i) {
        const auto &pad=paths[static_cast<size_t>(i)].pad;
        out+="P|"+QByteArray::number(i)+"|"+pad.reference.toUtf8()+"|"+
             pad.pin.toUtf8()+"|1|"+QByteArray::number(pad.x_nm/254)+"|"+
             QByteArray::number(pad.y_nm/254)+"|"+
             QByteArray::number(pad.width_nm/254)+"|"+
             QByteArray::number(pad.height_nm/254)+"|"+pad.shape.toUtf8()+"\n";
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
    for(int idx=4;idx<=5;++idx) {
        const auto &trace=tracks[static_cast<size_t>(idx==4?roundIndex:squareIndex)];
        out+="T|"+QByteArray::number(idx)+"|1|"+net+"|"+
             QByteArray::number(trace.x0_nm/254)+"|"+
             QByteArray::number(trace.y0_nm/254)+"|"+
             QByteArray::number(trace.x1_nm/254)+"|"+
             QByteArray::number(trace.y1_nm/254)+"|"+
             QByteArray::number(trace.width_nm/254)+"\n";
    }
    const auto &bridge=tracks[static_cast<size_t>(bridgeIndex)];
    const qint64 bx0=std::min(bridge.x0_nm,bridge.x1_nm)/254;
    const qint64 bx1=std::max(bridge.x0_nm,bridge.x1_nm)/254;
    out+="T|6|1|"+net+"|"+QByteArray::number(bx0)+"|30000|"+
         QByteArray::number(bx1)+"|30000|800\n";
    for(int c=0;c<2;++c) {
        const int pad=c*2,trace=pad+4,edge=c*3;
        out+="E|"+QByteArray::number(edge)+"|"+QByteArray::number(pad)+"|"+
             QByteArray::number(trace)+"|PAD_TRACE_LOCAL_COPPER\n";
        out+="E|"+QByteArray::number(edge+1)+"|"+QByteArray::number(pad+1)+"|"+
             QByteArray::number(trace+1)+"|PAD_TRACE_LOCAL_COPPER\n";
        out+="E|"+QByteArray::number(edge+2)+"|"+QByteArray::number(trace)+"|"+
             QByteArray::number(trace+1)+"|EXACT_TRACE_JUNCTION_LOCAL_COPPER\n";
    }
    out+="E|6|0|8|POSITIVE_FINITE_WIDTH_TRACE_ROUND_PAD_CONTACT\n"
         "E|7|1|9|POSITIVE_FINITE_WIDTH_TRACE_PAD_CONTACT\n"
         "E|8|8|10|POSITIVE_FINITE_WIDTH_TRACE_TRACE_BRIDGE\n"
         "E|9|2|10|POSITIVE_FINITE_WIDTH_TRACE_ROUND_PAD_BRIDGE\n"
         "COUNT|NODES|11|EDGES|10|LOCAL_COMPONENTS|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n"
         "END|SOURCE_LOCAL_MIXED_PAD_FINITE_WIDTH_BRIDGE_ONLY_NO_GLOBAL_PASS\n";
    return out;
}
inline bool decode(const QByteArray &wire,const QByteArray &source,
                   const PreviewImportPanel *panel) {
    return !wire.isEmpty() && wire.size()<=16384 && wire==reconstruct(source,panel);
}
} // namespace d050
