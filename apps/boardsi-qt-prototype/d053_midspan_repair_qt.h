// D_053: independent Native Qt source-import graph for genuine copper repairs.
// Bounded Layer-1 geometry only; never physical/global continuity.
#pragma once
#include "d052_bridge_midspan_break_qt.h"
#include <QCryptographicHash>
#include <array>
#include <algorithm>

namespace d053 {
struct Repair { int row=-1; qint64 lo=0, hi=0, y=0; };
using Repairs=std::array<Repair,2>;
inline bool prepare(const QByteArray &source,QByteArray &bare,Repairs &repair) {
    bare.clear();repair=Repairs{};
    if(source.isEmpty() || source.size()>8*1024*1024)return false;
    const auto rows=source.split('\n');
    bool signal=false;
    for(int i=0;i<rows.size();++i) {
        const auto f=rows[i].simplified().split(' ');
        if(f[0]=="*SIGNAL*")signal=true;
        else if(f[0]=="*END*")signal=false;
        if(!signal)continue;
        for(int side=0;side<2;++side) {
            const auto o1=side==0?"U1.1":"U1.2";
            const auto o2=side==0?"U2.1":"U2.2";
            const qint64 y=side==0?30000:40000;
            d052::Piece p;
            if(!d052::vertexPiece(rows,i,o1,o2,y,28000,32000,p))continue;
            if(p.lo>30000 || p.hi<30000 || repair[size_t(side)].row>=0)return false;
            repair[size_t(side)]={i,p.lo,p.hi,y};
            i+=2;
            break;
        }
    }
    for(int i=0;i<rows.size();++i) {
        if(i==rows.size()-1 && rows[i].isEmpty())continue;
        bool skip=false;
        for(const auto &r:repair)
            if(r.row>=0 && i>=r.row && i<r.row+3)skip=true;
        if(!skip)bare+=rows[i]+'\n';
    }
    return true;
}
struct Edge { int u=0,v=0; QByteArray label; };
inline bool graph(int n,const QList<Edge> &edges,int &comp,int &cycle,bool &reachable) {
    if(n<14 || n>16 || edges.size()<12 || edges.size()>16)return false;
    std::array<int,16> par{};
    for(int i=0;i<n;++i)par[size_t(i)]=i;
    const auto root=[&](int x) {
        while(par[size_t(x)]!=x)x=par[size_t(x)];
        return x;
    };
    for(const auto &e:edges) {
        if(e.u<0 || e.v<0 || e.u>=n || e.v>=n || e.u==e.v)return false;
        const int a=root(e.u),b=root(e.v);
        if(a!=b)par[size_t(a)]=b;
    }
    std::array<bool,16> seen{};
    comp=0;
    for(int i=0;i<n;++i) {
        const int r=root(i);
        if(!seen[size_t(r)]) {seen[size_t(r)]=true;++comp;}
    }
    cycle=edges.size()+comp-n;
    reachable=root(0)==root(2);
    return cycle>=0;
}
inline QByteArray track(int index,const QByteArray &net,const Repair &r) {
    return "T|"+QByteArray::number(index)+"|1|"+net+"|"+
      QByteArray::number(r.lo)+"|"+QByteArray::number(r.y)+"|"+
      QByteArray::number(r.hi)+"|"+QByteArray::number(r.y)+"|800\n";
}
inline QByteArray reconstruct(const QByteArray &source,
                              const PreviewImportPanel *actual,
                              const QByteArray &bare,
                              const PreviewImportPanel *barePanel,
                              const QByteArray &canonical,
                              const PreviewImportPanel *full,
                              const QByteArray &reduced,
                              const PreviewImportPanel *baseline) {
    if(!actual || !barePanel || !full || !baseline || !actual->imported()
       || !barePanel->imported() || !full->imported() || !baseline->imported())return {};
    QByteArray fresh;
    Repairs repairs;
    if(!prepare(source,fresh,repairs) || fresh!=bare)return {};
    const QByteArray d052wire=d052::reconstruct(bare,barePanel,canonical,full,reduced,baseline);
    if(d052wire.isEmpty())return {};
    const QByteArray sha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    if(actual->acceptedSourceSha256()!=sha)return {};
    QList<QList<QByteArray>> pads,tracks,targets;
    QList<Edge> edges;
    QByteArray net;
    bool splitA=false,splitB=false;
    for(const auto &line:d052wire.split('\n')) {
        const auto f=line.split('|');
        if(line.startsWith("P|"))pads.append(f);
        if(line.startsWith("T|")) {
            if(f.size()!=9 || f[1].toInt()!=tracks.size())return {};
            tracks.append(f);
            if(net.isEmpty())net=f[3];
        }
        if(line.startsWith("E|")) {
            if(f.size()!=5 || f[1].toInt()!=edges.size())return {};
            edges.append({f[2].toInt(),f[3].toInt(),f[4]});
        }
        if(line.startsWith("S|A|BREAK|"))splitA=true;
        if(line.startsWith("S|B|BREAK|"))splitB=true;
    }
    const bool ra=repairs[0].row>=0,rb=repairs[1].row>=0;
    QByteArray state;
    int wantedN=0,wantedE=0,wantedC=0,wantedCycle=0;
    if(splitA && splitB && !ra && !rb) {
        state="BREAK_BOTH";wantedN=14;wantedE=12;wantedC=2;
    } else if(splitA && splitB && ra && !rb) {
        state="REPAIR_A_ONLY";wantedN=15;wantedE=14;wantedC=1;
    } else if(splitA && splitB && !ra && rb) {
        state="REPAIR_B_ONLY";wantedN=15;wantedE=14;wantedC=1;
    } else if(splitA && splitB && ra && rb) {
        state="REPAIR_BOTH";wantedN=16;wantedE=16;wantedC=1;wantedCycle=1;
    } else if(splitA && !splitB && ra && !rb) {
        state="BREAK_A_REPAIR_A";wantedN=14;wantedE=14;wantedC=1;wantedCycle=1;
    } else if(!splitA && splitB && !ra && rb) {
        state="BREAK_B_REPAIR_B";wantedN=14;wantedE=14;wantedC=1;wantedCycle=1;
    } else return {};
    const int bareN=12+int(splitA)+int(splitB);
    if(tracks.size()!=bareN-4 || pads.size()!=4 || edges.size()!=12
       || (net!="NET_A" && net!="NET_B"))return {};
    for(const auto &line:tracks)targets.append(line);
    std::array<qint64,2> overlapL{},overlapR{};
    int next=bareN;
    for(int side=0;side<2;++side) {
        const auto &r=repairs[size_t(side)];
        if(r.row<0)continue;
        if(!(side==0?splitA:splitB))return {};
        const int leftTrack=6+side;
        const int leftNode=10+side;
        const int rightTrack=8+(side==1?int(splitA):0);
        if(rightTrack>=tracks.size())return {};
        const auto &l=tracks[leftTrack],&rr=tracks[rightTrack];
        const qint64 leftStart=l[4].toLongLong(),leftEnd=l[6].toLongLong();
        const qint64 rightStart=rr[4].toLongLong(),rightEnd=rr[6].toLongLong();
        if(l[5].toLongLong()!=r.y || rr[5].toLongLong()!=r.y ||
           leftEnd>=rightStart || r.lo<=leftStart || r.hi>=rightEnd)return {};
        const qint64 dl=std::max<qint64>(0,r.lo-leftEnd);
        const qint64 dr=std::max<qint64>(0,rightStart-r.hi);
        if(dl>=800 || dr>=800)return {};
        overlapL[size_t(side)]=800-dl;
        overlapR[size_t(side)]=800-dr;
        targets.append(track(tracks.size()+int(ra && rb && side==1),net,r).trimmed().split('|'));
        const int rightNode=12+(side==1?int(splitA):0);
        const QByteArray prefix="POSITIVE_FINITE_WIDTH_REPAIR_"+
            QByteArray(side==0?"A":"B");
        edges.append({leftNode,next,prefix+"_LEFT"});
        edges.append({rightNode,next,prefix+"_RIGHT"});
        ++next;
    }
    const auto *art=actual->imported();
    if(art->importedSkippedCount()!=0 || art->viaCount()!=0 || art->arcCount()!=0 ||
       art->padCount()!=4 || art->importedTraceCount()!=targets.size())return {};
    std::array<bool,4> padUsed{};
    for(int i=0;i<4;++i) {
        ImportedPad pad;
        if(!art->importedPad(i,pad))return {};
        int hits=0;
        for(int j=0;j<4;++j) {
            if(!padUsed[size_t(j)] && d051::oldPadMatches(pad,pads[j])) {
                padUsed[size_t(j)]=true;++hits;
            }
        }
        if(hits!=1)return {};
    }
    QList<bool> matched;
    for(int i=0;i<targets.size();++i)matched.append(false);
    for(int i=0;i<art->importedTraceCount();++i) {
        ImportedTrack item;
        if(!art->importedTrack(i,item))return {};
        int hits=0;
        for(int j=0;j<targets.size();++j) {
            if(!matched[j] && d051::oldTrackMatches(item,targets[j])) {
                matched[j]=true;++hits;
            }
        }
        if(hits!=1)return {};
    }
    for(const auto ok:matched)if(!ok)return {};
    int components=0,cycles=0;bool reachable=false;
    if(next!=wantedN || edges.size()!=wantedE ||
       !graph(next,edges,components,cycles,reachable) ||
       components!=wantedC || cycles!=wantedCycle ||
       reachable!=(wantedC==1))return {};
    QByteArray out="BOARDSI_D053_SOURCE_MIDSPAN_REPAIR_V1|"+sha+
      "|1|SOURCE_LOCAL_DUAL_BRIDGE_REPAIR_ONLY|P009_UNSUPPORTED|SI_NOT_RUN"
      "|GLOBAL_ELECTRICAL_NOT_VERIFIED\nC|0|U1_U2|"+net+"|1|"+
      QByteArray::number(next)+"|"+QByteArray::number(edges.size())+'\n';
    for(const auto &p:pads)out+=p.join('|')+'\n';
    for(const auto &t:targets)out+=t.join('|')+'\n';
    for(int i=0;i<edges.size();++i) {
        const auto &e=edges[i];
        out+="E|"+QByteArray::number(i)+"|"+QByteArray::number(e.u)+
             "|"+QByteArray::number(e.v)+"|"+e.label+'\n';
    }
    for(const auto &row:d052wire.split('\n'))
        if(row.startsWith("S|"))out+=row+'\n';
    for(int i=0;i<2;++i) {
        if(repairs[size_t(i)].row<0)continue;
        out+="R|"+QByteArray(i==0?"A":"B")+
            "|LEFT_COPPER_OVERLAP_TICKS|"+QByteArray::number(overlapL[size_t(i)])+
            "|RIGHT_COPPER_OVERLAP_TICKS|"+QByteArray::number(overlapR[size_t(i)])+'\n';
    }
    out+="D|"+state+"|"+QByteArray::number(next)+"|"+QByteArray::number(edges.size())+
         "|"+QByteArray::number(components)+"|"+QByteArray::number(cycles)+
         (reachable?"|REACHABLE\n":"|SEPARATED\n");
    out+="COUNT|NODES|"+QByteArray::number(next)+"|EDGES|"+
         QByteArray::number(edges.size())+"|LOCAL_COMPONENTS|"+QByteArray::number(components)+
         "|CYCLE_RANK|"+QByteArray::number(cycles)+
         "|GLOBAL_ELECTRICAL|NOT_VERIFIED\n"
         "END|SOURCE_LOCAL_DUAL_BRIDGE_REPAIR_ONLY_NO_GLOBAL_PASS\n";
    return out;
}
inline bool decode(const QByteArray &receipt,const QByteArray &source,
                   const PreviewImportPanel *actual,const QByteArray &bare,
                   const PreviewImportPanel *barePanel,const QByteArray &canonical,
                   const PreviewImportPanel *full,const QByteArray &reduced,
                   const PreviewImportPanel *baseline) {
    return !receipt.isEmpty() && receipt.size()<24000 &&
      receipt==reconstruct(source,actual,bare,barePanel,canonical,full,reduced,baseline);
}
} // namespace d053
