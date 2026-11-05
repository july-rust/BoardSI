// D_040 independent source-reimported Qt local component isolation.
// Conservative geometry envelopes are NOT global electrical short/open proof.
#pragma once
#include "d039_local_copper_qt.h"
#include <array>
#include <algorithm>

namespace d040 {
struct Path {
    ImportedPad pad;
    ImportedTrack trace;
    qint64 jointX = 0;
    qint64 jointY = 0;
    int trackIndex = -1;
};
struct Box2 {
    qint64 xmin, ymin, xmax, ymax;
    bool intersects(const Box2 &b) const {
        return xmin<=b.xmax && b.xmin<=xmax &&
               ymin<=b.ymax && b.ymin<=ymax;
    }
};
inline Box2 padBox(const ImportedPad &p) {
    return {2*p.x_nm-p.width_nm,2*p.y_nm-p.height_nm,
            2*p.x_nm+p.width_nm,2*p.y_nm+p.height_nm};
}
inline Box2 traceBox(const Path &p) {
    const qint64 x=2*p.pad.x_nm,y=2*p.pad.y_nm;
    const qint64 jx=2*p.jointX,jy=2*p.jointY,w=p.trace.width_nm;
    return {std::min(x,jx)-w,std::min(y,jy)-w,
            std::max(x,jx)+w,std::max(y,jy)+w};
}

inline bool boundedSections(const QByteArray &src) {
    if(src.isEmpty() || src.size()>8*1024*1024)return false;
    const int breakAt=src.indexOf('\n');
    if(breakAt<=0)return false;
    const QByteArray tag=src.left(breakAt).simplified().split(' ').first();
    if(tag!="!PADS-POWERPCB-V9.4-MILS!" &&
       tag!="!PADS-POWERPCB-V9.4-MILS-CP1252!" &&
       tag!="!PADS-POWERPCB-V9.4-MILS-250L!" &&
       tag!="!PADS-POWERPCB-V9.4-MILS-250L-CP1252!")return false;
    const std::array<QByteArray,7> sections={
        "*PCB*","*PARTDECAL*","*PARTTYPE*","*PART*",
        "*ROUTE*","*SIGNAL*","*END*"
    };
    const std::array<QByteArray,11> decal={
        "D2 I 1000 1000 0 2 2 0 0","T0 0 0 0 1",
        "T100 0 0 0 2","PAD 1 3","-2 40 R 0",
        "-1 0 R 0","0 0 R 0","PAD 2 3",
        "-2 30 S 0","-1 0 R 0","0 0 R 0"
    };
    std::array<int,7> counts={};
    std::array<int,2> pairs={};
    int active=-1,partIndex=0,decIndex=0,vertices=0,types=0;
    bool ended=false;
    int signalPairs=0;
    QList<QByteArray> netNames;
    bool firstLine=true;
    for(const auto &row:src.split('\n')) {
        // Only the validated first SOURCE line is a header. All later rows,
        // including a header-shaped row after *END*, are source inventory.
        if(firstLine) {
            firstLine=false;
            continue;
        }
        const auto fields=row.simplified().split(' ');
        if(fields.size()==1 && fields[0].isEmpty())continue;
        if(ended)return false;
        const QByteArray token=fields[0];
        if(token=="REMARK" || token=="*REMARK*")continue;
        if(token.startsWith('*')) {
            const auto it=std::find(sections.cbegin(),sections.cend(),token);
            if(it==sections.cend())return false;
            const int index=static_cast<int>(it-sections.cbegin());
            if(++counts[static_cast<size_t>(index)]>(index==5?2:1))return false;
            if((index==5 || index==6) && active==5 && signalPairs==0)
                return false;
            if(index==5) {
                if((active!=4 && active!=5) || fields.size()!=2 ||
                   (fields[1]!="NET_A" && fields[1]!="NET_B") ||
                   netNames.contains(fields[1]))return false;
                netNames.append(fields[1]);
                signalPairs=0;
            } else if(fields.size()!=1)return false;
            active=index;
            if(index==6)ended=true;
            continue;
        }
        if(active==0) {
            if(fields!=QList<QByteArray>({"MAXLAYER","4"}))return false;
        } else if(active==1) {
            if(decIndex>=static_cast<int>(decal.size()) ||
               row.simplified()!=decal[static_cast<size_t>(decIndex++)])return false;
        } else if(active==2) {
            if(row.simplified()!="TYPEA D2 UND 0 0 0 0" || ++types!=1)return false;
        } else if(active==3) {
            const QByteArray ref=partIndex==0?"U1":"U2";
            if(partIndex>=2 || fields.size()!=12 || fields[0]!=ref ||
               fields[1]!="TYPEA" || fields[4]!="90" ||
               fields[5]!="U" || fields[6]!="N" || fields[7]!="0" ||
               fields[8]!="-1" || fields[9]!="0" ||
               fields[10]!="-1" || fields[11]!="0")return false;
            bool okx=false,oky=false;
            fields[2].toDouble(&okx);fields[3].toDouble(&oky);
            if(!okx || !oky)return false;
            ++partIndex;
        } else if(active==5) {
            if(fields.size()==2) {
                ++signalPairs;
                if(fields[0]=="U1.1" && fields[1]=="U1.2")++pairs[0];
                else if(fields[0]=="U2.1" && fields[1]=="U2.2")++pairs[1];
                else return false;
            } else if(fields.size()==5) {
                for(const auto &v:fields) {
                    bool ok=false;v.toDouble(&ok);
                    if(!ok)return false;
                }
                ++vertices;
            } else return false;
        } else return false;
    }
    return counts[0]==1 && counts[1]==1 && counts[2]==1 &&
           counts[3]==1 && counts[4]==1 &&
           (counts[5]==1 || counts[5]==2) && counts[6]==1 &&
           decIndex==11 && types==1 && partIndex==2 &&
           pairs[0]==2 && pairs[1]==2 && vertices==8;
}

inline QByteArray reconstruct(const QByteArray &src,
                              const PreviewImportPanel *panel) {
    if(!panel || !panel->imported() || !boundedSections(src))return {};
    const auto *art=panel->imported();
    if(art->importedSkippedCount()!=0 || art->importedTraceCount()!=4 ||
       art->padCount()!=4 || art->arcCount()!=0 || art->viaCount()!=0)return {};
    const auto sha=QCryptographicHash::hash(src,QCryptographicHash::Sha256).toHex();
    if(sha!=panel->acceptedSourceSha256())return {};
    QStringList declaredA,declaredB;
    const bool hasA=d005::declaredPins(src,sha,"NET_A",declaredA);
    const bool hasB=d005::declaredPins(src,sha,"NET_B",declaredB);
    if(!hasA && !hasB)return {};
    std::array<ImportedTrack,4> traces;
    for(int t=0;t<4;++t) {
        auto &tr=traces[static_cast<size_t>(t)];
        if(!art->importedTrack(t,tr) || tr.layer!=1 ||
           (tr.net!="NET_A" && tr.net!="NET_B") ||
           (tr.net=="NET_A" && !hasA) ||
           (tr.net=="NET_B" && !hasB) ||
           tr.width_nm<=0 || tr.width_nm%254!=0 ||
           tr.x0_nm%254!=0 || tr.y0_nm%254!=0 ||
           tr.x1_nm%254!=0 || tr.y1_nm%254!=0 ||
           (tr.x0_nm==tr.x1_nm && tr.y0_nm==tr.y1_nm))return {};
    }
    std::array<Path,4> paths;
    std::array<int,4> used={};
    for(int i=0;i<4;++i) {
        auto &p=paths[static_cast<size_t>(i)];
        if(!art->importedPad(i,p.pad))return {};
        const auto &pad=p.pad;
        if(pad.layer!=1 || pad.side!="T" ||
           (pad.shape!="R" && pad.shape!="S") ||
           pad.width_nm<=0 || pad.height_nm!=pad.width_nm ||
           pad.width_nm%254!=0 || pad.x_nm%254!=0 || pad.y_nm%254!=0 ||
           pad.reference.isEmpty() || pad.pin.isEmpty())return {};
        for(int t=0;t<4;++t) {
            const auto &tr=traces[static_cast<size_t>(t)];
            const bool start=(tr.x0_nm==pad.x_nm && tr.y0_nm==pad.y_nm);
            const bool end=(tr.x1_nm==pad.x_nm && tr.y1_nm==pad.y_nm);
            if(start==end)continue;
            const QString owner=pad.reference+"."+pad.pin;
            const auto &declared=tr.net=="NET_A"?declaredA:declaredB;
            if(!declared.contains(owner) || p.trackIndex>=0 ||
               tr.width_nm>pad.width_nm/2)return {};
            p.trace=tr;p.trackIndex=t;
            p.jointX=start?tr.x1_nm:tr.x0_nm;
            p.jointY=start?tr.y1_nm:tr.y0_nm;
            ++used[static_cast<size_t>(t)];
        }
        if(p.trackIndex<0)return {};
    }
    if(std::any_of(used.cbegin(),used.cend(),[](int n){return n!=1;}))return {};
    std::sort(paths.begin(),paths.end(),[](const Path &a,const Path &b) {
        return std::make_pair(a.pad.reference,a.pad.pin) <
               std::make_pair(b.pad.reference,b.pad.pin);
    });
    for(int i=0;i<4;++i) {
        const auto &p=paths[static_cast<size_t>(i)];
        const QString ref=i<2?"U1":"U2";
        const QString pin=i%2==0?"1":"2";
        if(p.pad.reference!=ref || p.pad.pin!=pin)return {};
    }
    for(int c=0;c<2;++c) {
        const auto &a=paths[static_cast<size_t>(c*2)];
        const auto &b=paths[static_cast<size_t>(c*2+1)];
        if(a.trace.net!=b.trace.net ||
           a.trace.width_nm!=b.trace.width_nm ||
           a.jointX!=b.jointX || a.jointY!=b.jointY)return {};
        const qint64 x=a.jointX,y=a.jointY;
        if(!((a.pad.x_nm==b.pad.x_nm && x==a.pad.x_nm &&
              d039::strictlyBetween(a.pad.y_nm,y,b.pad.y_nm)) ||
             (a.pad.y_nm==b.pad.y_nm && y==a.pad.y_nm &&
              d039::strictlyBetween(a.pad.x_nm,x,b.pad.x_nm))))return {};
        const auto distance=[&](const ImportedPad &p) {
            return qAbs(x-p.x_nm)+qAbs(y-p.y_nm);
        };
        if(2*distance(a.pad)<=a.pad.width_nm ||
           2*distance(b.pad)<=b.pad.width_nm)return {};
    }
    const std::array<Box2,4> groupA={
        padBox(paths[0].pad),padBox(paths[1].pad),
        traceBox(paths[0]),traceBox(paths[1])
    };
    const std::array<Box2,4> groupB={
        padBox(paths[2].pad),padBox(paths[3].pad),
        traceBox(paths[2]),traceBox(paths[3])
    };
    for(const auto &a:groupA)for(const auto &b:groupB)
        if(a.intersects(b))return {};

    QByteArray out="BOARDSI_D040_MULTI_COMPONENT_V1|"+sha+
        "|1|LOCAL_COPPER_COMPONENT_ISOLATION_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN"
        "|GLOBAL_ELECTRICAL_NOT_VERIFIED\n";
    for(int c=0;c<2;++c) {
        const auto &p=paths[static_cast<size_t>(2*c)];
        out+="C|"+QByteArray::number(c)+"|"+p.pad.reference.toUtf8()+"|"+
            p.trace.net.toUtf8()+"|1|4|3\n";
    }
    for(int i=0;i<4;++i) {
        const auto &p=paths[static_cast<size_t>(i)].pad;
        out+="P|"+QByteArray::number(i)+"|"+p.reference.toUtf8()+"|"+
             p.pin.toUtf8()+"|1|"+QByteArray::number(p.x_nm/254)+"|"+
             QByteArray::number(p.y_nm/254)+"|"+QByteArray::number(p.width_nm/254)+
             "|"+QByteArray::number(p.height_nm/254)+"|"+p.shape.toUtf8()+"\n";
    }
    for(int i=0;i<4;++i) {
        const auto &p=paths[static_cast<size_t>(i)];
        out+="T|"+QByteArray::number(i)+"|1|"+p.trace.net.toUtf8()+"|"+
             QByteArray::number(p.pad.x_nm/254)+"|"+
             QByteArray::number(p.pad.y_nm/254)+"|"+
             QByteArray::number(p.jointX/254)+"|"+
             QByteArray::number(p.jointY/254)+"|"+
             QByteArray::number(p.trace.width_nm/254)+"\n";
    }
    for(int c=0;c<2;++c) {
        const int p=c*2,t=p+4,e=c*3;
        out+="E|"+QByteArray::number(e)+"|"+QByteArray::number(p)+"|"+
             QByteArray::number(t)+"|PAD_TRACE_LOCAL_COPPER\n";
        out+="E|"+QByteArray::number(e+1)+"|"+QByteArray::number(p+1)+"|"+
             QByteArray::number(t+1)+"|PAD_TRACE_LOCAL_COPPER\n";
        out+="E|"+QByteArray::number(e+2)+"|"+QByteArray::number(t)+"|"+
             QByteArray::number(t+1)+"|EXACT_TRACE_JUNCTION_LOCAL_COPPER\n";
    }
    out+="COUNT|NODES|8|EDGES|6|LOCAL_COMPONENTS|2|GLOBAL_ELECTRICAL|NOT_VERIFIED\n"
         "END|LOCAL_COPPER_COMPONENTS_ONLY_NO_GLOBAL_PASS\n";
    return out;
}

inline bool decode(const QByteArray &wire,const QByteArray &source,
                   const PreviewImportPanel *panel) {
    return !wire.isEmpty() && wire.size()<=16384 &&
           wire==reconstruct(source,panel);
}

inline int selftest(const QString &importer,const QString &fixturePath) {
    QFile fixture(fixturePath);
    if(!fixture.open(QIODevice::ReadOnly))return 2;
    QByteArray original=fixture.readAll();
    original.replace("\r\n","\n");
    fixture.close();
    QTemporaryFile temp;
    if(!temp.open())return 2;
    auto save=[&](const QByteArray &src) {
        return temp.resize(0) && temp.seek(0) &&
               temp.write(src)==src.size() && temp.flush();
    };
    if(!save(original))return 2;
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    auto load=[&]() {
        panel.setStatusCallback([&loop](const QString &state) {
            if(state.startsWith("PARTIAL") || state.startsWith("FAILED"))
                loop.quit();
        });
        if(!panel.openPath(temp.fileName()))return false;
        QTimer timeout;
        timeout.setSingleShot(true);
        QObject::connect(&timeout,&QTimer::timeout,&loop,&QEventLoop::quit);
        timeout.start(8000);
        loop.exec();
        return panel.importStatus().startsWith("PARTIAL");
    };
    auto run=[&](QByteArray &wire) {
        QProcess proc;
        proc.start(importer,{temp.fileName(),"--d040-multi-component-v1"});
        if(!proc.waitForFinished(10000) || proc.exitStatus()!=QProcess::NormalExit)
            return -1;
        wire=proc.readAllStandardOutput();
        return proc.exitCode();
    };
    if(!load())return 2;
    QByteArray wire;
    const bool sameNet=run(wire)==0 && decode(wire,original,&panel) &&
        wire.contains("C|1|U2|NET_A|1|4|3\n");
    if(!sameNet)return 1;
    QByteArray mixed=original;
    const QByteArray marker="U2.1 U2.2\n400 300 1 10 0";
    mixed.replace(marker,"*SIGNAL* NET_B\nU2.1 U2.2\n400 300 1 10 0");
    if(mixed==original || !save(mixed) || !load())return 2;
    QByteArray mixedWire;
    const bool twoNets=run(mixedWire)==0 && decode(mixedWire,mixed,&panel) &&
        mixedWire.contains("C|1|U2|NET_B|1|4|3\n");
    if(!twoNets)return 1;
    QByteArray forged=mixedWire;
    forged.replace("E|5|6|7|EXACT_TRACE_JUNCTION_LOCAL_COPPER",
                   "E|5|4|7|EXACT_TRACE_JUNCTION_LOCAL_COPPER");
    const bool forgedRejected=forged!=mixedWire && !decode(forged,mixed,&panel);
    QByteArray onlyB=original;
    onlyB.replace("*SIGNAL* NET_A","*SIGNAL* NET_B");
    if(onlyB==original || !save(onlyB) || !load())return 2;
    QByteArray onlyBWire;
    const bool netBOnly=run(onlyBWire)==0 && decode(onlyBWire,onlyB,&panel) &&
        onlyBWire.contains("C|0|U1|NET_B|1|4|3\n") &&
        onlyBWire.contains("C|1|U2|NET_B|1|4|3\n");
    QByteArray collision=mixed;
    collision.replace("U2 TYPEA 400 300","U2 TYPEA 200 300");
    if(collision==mixed || !save(collision))return 2;
    QByteArray negative;
    const bool collisionRejected=run(negative)!=0;
    if(!save(original) || !load())return 2;
    const bool stale=!decode(mixedWire,original,&panel) &&
        decode(wire,original,&panel);
    QByteArray gap=original;
    gap.replace("400 400 1 10 0\n400 350 1 10 0",
                "400 400 1 10 0\n400 350.01 1 10 0");
    if(gap==original || !save(gap))return 2;
    const bool gapRejected=run(negative)!=0;
    QByteArray trailingHeader=original;
    trailingHeader+="!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n";
    const bool trailerQtRejected=!boundedSections(trailingHeader);
    if(!save(trailingHeader))return 2;
    const bool trailerRustRejected=run(negative)!=0;
    const bool ok=sameNet && twoNets && netBOnly && forgedRejected &&
                  collisionRejected && stale && gapRejected &&
                  trailerQtRejected && trailerRustRejected;
    qInfo()<<"D040_NATIVE_QT_COMPONENT_ISOLATION"<<ok<<sameNet<<twoNets
           <<netBOnly<<forgedRejected<<collisionRejected<<stale<<gapRejected
           <<"GLOBAL_ELECTRICAL_NOT_VERIFIED";
    return ok?0:1;
}
} // namespace d040
