// D_052: independent Native Qt real SOURCE midspan-gap and graph oracle.
// Exact same-layer local candidate geometry only; NO global electrical proof.
#pragma once
#include "d051_dual_bridge_loop_qt.h"
#include <QCryptographicHash>
#include <QList>
#include <array>
#include <algorithm>

namespace d052 {
struct Piece { int row=-1; qint64 lo=0,hi=0; };
struct Path { QList<Piece> pieces; qint64 gap=0; };
struct Edge { int u=0,v=0; QByteArray label; };

inline bool vertexPiece(const QList<QByteArray> &rows,int i,
                        const QByteArray &owner1,const QByteArray &owner2,
                        qint64 y,qint64 left,qint64 right,Piece &result) {
    if(i+2>=rows.size())return false;
    const auto head=rows[i].simplified().split(' ');
    if(head.size()!=2 || head[0]!=owner1 || head[1]!=owner2)return false;
    const auto v0=rows[i+1].simplified().split(' ');
    const auto v1=rows[i+2].simplified().split(' ');
    if(v0.size()!=5 || v1.size()!=5)return false;
    qint64 x0=0,x1=0,y0=0,y1=0,w0=0,w1=0;
    if(!d050::sourceCoordTicks(v0[0],x0) ||
       !d050::sourceCoordTicks(v1[0],x1) ||
       !d050::sourceCoordTicks(v0[1],y0) ||
       !d050::sourceCoordTicks(v1[1],y1) ||
       !d050::sourceCoordTicks(v0[3],w0) ||
       !d050::sourceCoordTicks(v1[3],w1) ||
       y0!=y || y1!=y || w0!=800 || w1!=800 ||
       !(v0[2]=="1" || v0[2]=="65") ||
       !(v1[2]=="1" || v1[2]=="65") ||
       !(v0[4]=="0" || v0[4]=="128") ||
       !(v1[4]=="0" || v1[4]=="128"))return false;
    const qint64 lo=std::min(x0,x1),hi=std::max(x0,x1);
    if(lo<left || hi>right || lo>=hi)return false;
    result={i,lo,hi};
    return true;
}
inline bool qualify(Path &path,qint64 left,qint64 right) {
    std::sort(path.pieces.begin(),path.pieces.end(),
              [](const Piece &a,const Piece &b){return a.lo<b.lo;});
    if(path.pieces.size()==1) {
        const auto p=path.pieces[0];
        return p.lo==left && p.hi==right;
    }
    if(path.pieces.size()!=2)return false;
    const auto a=path.pieces[0],b=path.pieces[1];
    if(a.lo!=left || b.hi!=right || a.hi<28000 || a.hi>30500 ||
       b.lo<29500 || b.lo>32000 || b.lo-a.hi<=800)return false;
    path.gap=b.lo-a.hi;
    return true;
}
inline bool prepare(const QByteArray &source,QByteArray &canonical,
                    Path &a,Path &b) {
    canonical.clear();a=Path{};b=Path{};
    if(source.isEmpty() || source.size()>8*1024*1024)return false;
    const auto rows=source.split('\n');
    bool signal=false;
    for(int i=0;i<rows.size();++i) {
        const auto f=rows[i].simplified().split(' ');
        if(f[0]=="*SIGNAL*")signal=true;
        else if(f[0]=="*END*")signal=false;
        if(!signal)continue;
        Piece p;
        if(vertexPiece(rows,i,"U1.1","U2.1",30000,22600,37800,p)) {
            a.pieces.append(p);i+=2;
        } else if(vertexPiece(rows,i,"U1.2","U2.2",40000,22200,38200,p)) {
            b.pieces.append(p);i+=2;
        }
    }
    if(!qualify(a,22600,37800) || !qualify(b,22200,38200))return false;
    for(int i=0;i<rows.size();++i) {
        if(i==rows.size()-1 && rows[i].isEmpty())continue;
        if(i==a.pieces[0].row)
            canonical+="U1.1 U2.1\n226 300 1 8 0\n378 300 1 8 0\n";
        if(i==b.pieces[0].row)
            canonical+="U1.2 U2.2\n222 400 1 8 0\n382 400 1 8 0\n";
        bool skip=false;
        for(const auto &p:a.pieces)
            if(i>=p.row && i<p.row+3)skip=true;
        for(const auto &p:b.pieces)
            if(i>=p.row && i<p.row+3)skip=true;
        if(!skip)canonical+=rows[i]+'\n';
    }
    return true;
}
inline QByteArray trackLine(int index,const QByteArray &net,
                           qint64 lo,qint64 hi,qint64 y) {
    return "T|"+QByteArray::number(index)+"|1|"+net+"|"+
        QByteArray::number(lo)+"|"+QByteArray::number(y)+"|"+
        QByteArray::number(hi)+"|"+QByteArray::number(y)+"|800\n";
}
inline bool graph(const std::array<Edge,12> &edges,int n,
                  int &components,int &cycles,bool &reachable) {
    if(n<12 || n>14)return false;
    std::array<int,14> roots{};
    for(int i=0;i<n;++i)roots[static_cast<size_t>(i)]=i;
    const auto root=[&](int id) {
        while(roots[static_cast<size_t>(id)]!=id)
            id=roots[static_cast<size_t>(id)];
        return id;
    };
    for(const auto &edge:edges) {
        if(edge.u<0 || edge.v<0 || edge.u>=n ||
           edge.v>=n || edge.u==edge.v)return false;
        const int u=root(edge.u),v=root(edge.v);
        if(u!=v)roots[static_cast<size_t>(u)]=v;
    }
    std::array<bool,14> seen{};
    components=0;
    for(int i=0;i<n;++i) {
        const auto r=root(i);
        if(!seen[static_cast<size_t>(r)]) {
            seen[static_cast<size_t>(r)]=true;++components;
        }
    }
    cycles=12-n+components;
    reachable=root(0)==root(2);
    return cycles>=0;
}
inline QByteArray reconstruct(const QByteArray &source,
                              const PreviewImportPanel *actual,
                              const QByteArray &canonical,
                              const PreviewImportPanel *full,
                              const QByteArray &reduced,
                              const PreviewImportPanel *baseline) {
    if(!actual || !full || !baseline || !actual->imported()
       || !full->imported() || !baseline->imported())return {};
    QByteArray fresh;
    Path a,b;
    if(!prepare(source,fresh,a,b) || fresh!=canonical)return {};
    const QByteArray old=d051::reconstruct(canonical,full,reduced,baseline);
    if(old.isEmpty())return {};
    const QByteArray sha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    if(actual->acceptedSourceSha256()!=sha)return {};
    const int n=12+int(a.gap>0)+int(b.gap>0);
    const auto *art=actual->imported();
    if(art->importedSkippedCount()!=0 || art->importedTraceCount()!=n-4 ||
       art->padCount()!=4 || art->arcCount()!=0 || art->viaCount()!=0)return {};
    QList<QList<QByteArray>> expectedPads,expectedTracks;
    std::array<Edge,12> edges{};
    int edgeCount=0;
    QByteArray net;
    for(const auto &line:old.split('\n')) {
        const auto f=line.split('|');
        if(line.startsWith("P|"))expectedPads.append(f);
        if(line.startsWith("T|")) {
            expectedTracks.append(f);
            if(net.isEmpty())net=f.value(3);
        }
        if(line.startsWith("E|")) {
            if(f.size()!=5 || edgeCount>=12)return {};
            bool idOk=false,uOk=false,vOk=false;
            const int id=f[1].toInt(&idOk);
            const int u=f[2].toInt(&uOk),v=f[3].toInt(&vOk);
            if(!idOk || !uOk || !vOk || id!=edgeCount)return {};
            edges[static_cast<size_t>(edgeCount++)]={u,v,f[4]};
        }
    }
    if(edgeCount!=12 || expectedPads.size()!=4 || expectedTracks.size()!=8
       || (net!="NET_A" && net!="NET_B") ||
       edges[9].u!=2 || edges[9].v!=10 ||
       edges[11].u!=3 || edges[11].v!=11)return {};
    std::array<bool,4> padUsed{};
    for(int i=0;i<4;++i) {
        ImportedPad p;
        if(!art->importedPad(i,p))return {};
        int hits=0;
        for(int j=0;j<4;++j) {
            if(!padUsed[static_cast<size_t>(j)] &&
               d051::oldPadMatches(p,expectedPads[j])) {
                padUsed[static_cast<size_t>(j)]=true;++hits;
            }
        }
        if(hits!=1)return {};
    }
    QList<QList<QByteArray>> targets;
    for(int i=0;i<6;++i)targets.append(expectedTracks[i]);
    for(const auto &p:a.pieces)targets.append(trackLine(6,net,p.lo,p.hi,30000).trimmed().split('|'));
    for(const auto &p:b.pieces)targets.append(trackLine(7,net,p.lo,p.hi,40000).trimmed().split('|'));
    if(targets.size()!=n-4)return {};
    QList<bool> matched;
    for(int i=0;i<targets.size();++i)matched.append(false);
    for(int i=0;i<art->importedTraceCount();++i) {
        ImportedTrack t;
        if(!art->importedTrack(i,t))return {};
        int hits=0;
        for(int j=0;j<targets.size();++j) {
            if(!matched[j] && d051::oldTrackMatches(t,targets[j])) {
                matched[j]=true;++hits;
            }
        }
        if(hits!=1)return {};
    }
    for(const auto ok:matched)if(!ok)return {};
    int next=12;
    if(a.gap>0)edges[9].v=next++;
    if(b.gap>0)edges[11].v=next++;
    int components=0,cycles=0;bool reachable=false;
    if(next!=n || !graph(edges,n,components,cycles,reachable))return {};
    const int expectedComponents=a.gap>0 && b.gap>0?2:1;
    const int expectedCycles=a.gap==0 && b.gap==0?1:0;
    if(components!=expectedComponents || cycles!=expectedCycles ||
       reachable!=(expectedComponents==1))return {};
    const char *state=a.gap>0?(b.gap>0?"BREAK_BOTH":"BREAK_A"):
                           (b.gap>0?"BREAK_B":"INTACT");
    QByteArray out="BOARDSI_D052_DUAL_BRIDGE_MIDSPAN_BREAK_V1|"+sha+
        "|1|SOURCE_LOCAL_DUAL_BRIDGE_MIDSPAN_BREAK_ONLY|P009_UNSUPPORTED|SI_NOT_RUN"
        "|GLOBAL_ELECTRICAL_NOT_VERIFIED\nC|0|U1_U2|"+net+"|1|"+
        QByteArray::number(n)+"|12\n";
    for(const auto &line:old.split('\n'))
        if(line.startsWith("P|"))out+=line+'\n';
    for(int i=0;i<6;++i)out+=expectedTracks[i].join('|')+'\n';
    out+=trackLine(6,net,a.pieces[0].lo,a.pieces[0].hi,30000);
    out+=trackLine(7,net,b.pieces[0].lo,b.pieces[0].hi,40000);
    int trackIndex=8;
    if(a.gap>0)out+=trackLine(trackIndex++,net,a.pieces[1].lo,a.pieces[1].hi,30000);
    if(b.gap>0)out+=trackLine(trackIndex++,net,b.pieces[1].lo,b.pieces[1].hi,40000);
    for(int i=0;i<12;++i) {
        const auto &edge=edges[static_cast<size_t>(i)];
        out+="E|"+QByteArray::number(i)+"|"+QByteArray::number(edge.u)+
             "|"+QByteArray::number(edge.v)+"|"+edge.label+'\n';
    }
    for(const auto &info:std::array<std::pair<const char*,qint64>,2>{{
            {"A",a.gap},{"B",b.gap}}}) {
        if(info.second==0)out+="S|"+QByteArray(info.first)+"|INTACT\n";
        else out+="S|"+QByteArray(info.first)+"|BREAK|CENTER_GAP_TICKS|"+
            QByteArray::number(info.second)+"|COPPER_GAP_TICKS|"+
            QByteArray::number(info.second-800)+'\n';
    }
    out+="D|"+QByteArray(state)+"|"+QByteArray::number(n)+
         "|12|"+QByteArray::number(components)+"|"+
         QByteArray::number(cycles)+"|"+(reachable?"REACHABLE\n":"SEPARATED\n");
    out+="COUNT|NODES|"+QByteArray::number(n)+"|EDGES|12|LOCAL_COMPONENTS|"+
         QByteArray::number(components)+"|CYCLE_RANK|"+QByteArray::number(cycles)+
         "|GLOBAL_ELECTRICAL|NOT_VERIFIED\n"
         "END|SOURCE_LOCAL_DUAL_BRIDGE_MIDSPAN_BREAK_ONLY_NO_GLOBAL_PASS\n";
    return out;
}
inline bool decode(const QByteArray &receipt,const QByteArray &source,
                   const PreviewImportPanel *actual,const QByteArray &canonical,
                   const PreviewImportPanel *full,const QByteArray &reduced,
                   const PreviewImportPanel *baseline) {
    return !receipt.isEmpty() && receipt.size()<20000 &&
        receipt==reconstruct(source,actual,canonical,full,reduced,baseline);
}
} // namespace d052
