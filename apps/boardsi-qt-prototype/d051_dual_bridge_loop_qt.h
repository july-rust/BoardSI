// D_051: independent Native Qt SOURCE/ImportedArtwork dual-bridge loop oracle.
// Candidate same-layer geometry, never global electrical proof.
#pragma once
#include "d050_mixed_pad_width_bridge_qt.h"
#include <array>
#include <algorithm>
#include <utility>

namespace d051 {
inline bool candidateVertices(const std::array<QByteArray,2> &vertices) {
    qint64 x[2]={}, y[2]={};
    for(int i=0;i<2;++i) {
        const auto parts=vertices[static_cast<size_t>(i)].simplified().split(' ');
        if(parts.size()!=5 || !d050::sourceCoordTicks(parts[0],x[i]) ||
           !d050::sourceCoordTicks(parts[1],y[i]))return false;
    }
    const auto a=std::min(x[0],x[1]),b=std::max(x[0],x[1]);
    return y[0]==40000 && y[1]==40000 &&
        a>=21900 && a<=22500 && b>=38100 && b<=38500;
}
inline bool stripSecond(const QByteArray &source,QByteArray &reduced,
                        std::array<QByteArray,2> &vertices) {
    reduced.clear();
    int candidates=0;
    bool inSignal=false;
    const auto rows=source.split('\n');
    for(int i=0;i<rows.size();++i) {
        if(i==rows.size()-1 && rows[i].isEmpty())continue;
        const auto f=rows[i].simplified().split(' ');
        if(f[0]=="*SIGNAL*")inSignal=true;
        else if(f[0]=="*END*")inSignal=false;
        if(inSignal && f.size()==2 && f[0]=="U1.2" && f[1]=="U2.2" &&
           i+2<rows.size()) {
            const std::array<QByteArray,2> route={rows[i+1],rows[i+2]};
            if(candidateVertices(route)) {
                if(++candidates!=1)return false;
                vertices=route;i+=2;
                continue;
            }
        }
        reduced+=rows[i]+'\n';
    }
    return candidates==1 && !reduced.isEmpty();
}
inline bool positiveSecond(const ImportedTrack &t) {
    if(t.layer!=1 || t.width_nm!=800LL*254 ||
       t.x0_nm%254 || t.y0_nm%254 || t.x1_nm%254 || t.y1_nm%254)return false;
    const qint64 a=std::min(t.x0_nm,t.x1_nm)/254;
    const qint64 b=std::max(t.x0_nm,t.x1_nm)/254;
    if(t.y0_nm!=40000LL*254 || t.y1_nm!=40000LL*254 ||
       a<=21900 || a>=22500 || b<=38100 || b>=38500)return false;
    const qint64 d=a-21700, square=38500-b;
    return d*d<800LL*800 && square*square<400LL*400;
}
inline bool oldTrackMatches(const ImportedTrack &actual,
                            const QList<QByteArray> &fields) {
    if(fields.size()!=9 || fields[0]!="T")return false;
    bool ok=true,worked=false;
    const auto read=[&](int index) {
        bool parsed=false;
        const qint64 result=fields[index].toLongLong(&parsed);
        worked=parsed;
        return result;
    };
    const qint64 layer=read(2);ok=ok&&worked;
    const qint64 x0=read(4);ok=ok&&worked;
    const qint64 y0=read(5);ok=ok&&worked;
    const qint64 x1=read(6);ok=ok&&worked;
    const qint64 y1=read(7);ok=ok&&worked;
    const qint64 width=read(8);ok=ok&&worked;
    if(!ok || actual.layer!=layer ||
       actual.net!=QString::fromLatin1(fields[3]) ||
       actual.width_nm!=width*254)return false;
    const bool forward=actual.x0_nm==x0*254 && actual.y0_nm==y0*254 &&
        actual.x1_nm==x1*254 && actual.y1_nm==y1*254;
    const bool backward=actual.x0_nm==x1*254 && actual.y0_nm==y1*254 &&
        actual.x1_nm==x0*254 && actual.y1_nm==y0*254;
    return forward || backward;
}
inline bool oldPadMatches(const ImportedPad &actual,
                          const QList<QByteArray> &f) {
    if(f.size()!=10 || f[0]!="P")return false;
    bool ok[5]={};
    const qint64 layer=f[4].toLongLong(&ok[0]);
    const qint64 x=f[5].toLongLong(&ok[1]);
    const qint64 y=f[6].toLongLong(&ok[2]);
    const qint64 w=f[7].toLongLong(&ok[3]);
    const qint64 h=f[8].toLongLong(&ok[4]);
    for(const bool value:ok)if(!value)return false;
    return actual.reference==QString::fromLatin1(f[2]) &&
        actual.pin==QString::fromLatin1(f[3]) &&
        actual.layer==layer && actual.side=="T" &&
        actual.x_nm==x*254 && actual.y_nm==y*254 &&
        actual.width_nm==w*254 && actual.height_nm==h*254 &&
        actual.shape==QString::fromLatin1(f[9]);
}
inline bool diagnostic(const std::array<std::pair<int,int>,12> &edges,
                       bool dropFirst,bool dropSecond,
                       int &nodes,int &links,int &components,int &cycles,
                       bool &reachable) {
    std::array<int,12> parent{};
    for(int i=0;i<12;++i)parent[static_cast<size_t>(i)]=i;
    const auto isRemoved=[=](int n) {
        return (dropFirst && n==10) || (dropSecond && n==11);
    };
    const auto root=[&](int n) {
        while(parent[static_cast<size_t>(n)]!=n)n=parent[static_cast<size_t>(n)];
        return n;
    };
    links=0;
    for(const auto &e:edges) {
        if(e.first<0 || e.first>=12 || e.second<0 ||
           e.second>=12 || e.first==e.second)return false;
        if(isRemoved(e.first) || isRemoved(e.second))continue;
        const int a=root(e.first),b=root(e.second);
        if(a!=b)parent[static_cast<size_t>(a)]=b;
        ++links;
    }
    nodes=12-int(dropFirst)-int(dropSecond);
    std::array<int,12> visited{};
    components=0;
    for(int i=0;i<12;++i) {
        if(isRemoved(i))continue;
        const int r=root(i);
        if(!visited[static_cast<size_t>(r)]) {
            visited[static_cast<size_t>(r)]=1;
            ++components;
        }
    }
    cycles=links-nodes+components;
    reachable=root(0)==root(2);
    return cycles>=0;
}
inline QByteArray reconstruct(const QByteArray &source,
                              const PreviewImportPanel *full,
                              const QByteArray &reduced,
                              const PreviewImportPanel *baseline) {
    if(!full || !baseline || !full->imported() || !baseline->imported() ||
       source.isEmpty() || source.size()>8*1024*1024)return {};
    QByteArray sourceReduced;
    std::array<QByteArray,2> vertices{};
    if(!stripSecond(source,sourceReduced,vertices) || sourceReduced!=reduced)return {};
    const QByteArray oldWire=d050::reconstruct(reduced,baseline);
    if(oldWire.isEmpty())return {};
    const QByteArray sha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    if(full->acceptedSourceSha256()!=sha)return {};
    const auto *art=full->imported();
    if(art->importedSkippedCount()!=0 || art->importedTraceCount()!=8 ||
       art->padCount()!=4 || art->arcCount()!=0 || art->viaCount()!=0)return {};
    QList<QList<QByteArray>> originalPads,originalTracks,originalEdges;
    QByteArray net;
    for(const auto &line:oldWire.split('\n')) {
        const auto f=line.split('|');
        if(line.startsWith("P|"))originalPads.append(f);
        if(line.startsWith("T|")) {
            originalTracks.append(f);
            if(net.isEmpty())net=f.value(3);
        }
        if(line.startsWith("E|"))originalEdges.append(f);
    }
    if(originalPads.size()!=4 || originalTracks.size()!=7 ||
       originalEdges.size()!=10 || (net!="NET_A" && net!="NET_B"))return {};
    std::array<bool,4> padUsed{};
    for(int i=0;i<4;++i) {
        ImportedPad p;
        if(!art->importedPad(i,p))return {};
        int hits=0;
        for(int j=0;j<4;++j) {
            if(!padUsed[static_cast<size_t>(j)] &&
               oldPadMatches(p,originalPads[j])) {
                padUsed[static_cast<size_t>(j)]=true;++hits;
            }
        }
        if(hits!=1)return {};
    }
    std::array<bool,7> used{};
    ImportedTrack second;
    int secondCount=0;
    for(int i=0;i<8;++i) {
        ImportedTrack t;
        if(!art->importedTrack(i,t) || t.layer!=1 ||
           t.net!=QString::fromLatin1(net) || t.width_nm<=0 ||
           t.width_nm%254 || t.x0_nm%254 || t.y0_nm%254 ||
           t.x1_nm%254 || t.y1_nm%254 ||
           (t.x0_nm==t.x1_nm && t.y0_nm==t.y1_nm))return {};
        int matching=-1;
        for(int j=0;j<7;++j) {
            if(oldTrackMatches(t,originalTracks[j])) {
                if(matching>=0 || used[static_cast<size_t>(j)])return {};
                matching=j;
            }
        }
        if(matching>=0)used[static_cast<size_t>(matching)]=true;
        else {
            if(++secondCount!=1 || !d050::sourcePairTrack(t,vertices) ||
               !positiveSecond(t))return {};
            second=t;
        }
    }
    if(secondCount!=1)return {};
    for(const bool matched:used)if(!matched)return {};
    std::array<std::pair<int,int>,12> edges{};
    for(int i=0;i<10;++i) {
        const auto &e=originalEdges[i];
        if(e.size()!=5 || e[0]!="E")return {};
        bool idOk=false,aOk=false,bOk=false;
        const int index=e[1].toInt(&idOk);
        const int a=e[2].toInt(&aOk),b=e[3].toInt(&bOk);
        if(!idOk || !aOk || !bOk || index!=i)return {};
        edges[static_cast<size_t>(i)]={a,b};
    }
    edges[10]={9,11};edges[11]={3,11};
    const std::array<std::array<int,6>,4> expected={{
        {{12,12,1,1,1,0}}, {{11,10,1,0,1,0}},
        {{11,10,1,0,1,0}}, {{10,8,2,0,0,0}}
    }};
    const std::array<const char*,4> names={{"NONE","FIRST","SECOND","BOTH"}};
    QByteArray diagnoses;
    for(int i=0;i<4;++i) {
        int n=0,e=0,c=0,cycles=0;bool reach=false;
        if(!diagnostic(edges,i==1 || i==3,i==2 || i==3,n,e,c,cycles,reach) ||
           n!=expected[static_cast<size_t>(i)][0] ||
           e!=expected[static_cast<size_t>(i)][1] ||
           c!=expected[static_cast<size_t>(i)][2] ||
           cycles!=expected[static_cast<size_t>(i)][3] ||
           int(reach)!=expected[static_cast<size_t>(i)][4])return {};
        diagnoses+="D|"+QByteArray(names[static_cast<size_t>(i)])+"|"+
            QByteArray::number(n)+"|"+QByteArray::number(e)+"|"+
            QByteArray::number(c)+"|"+QByteArray::number(cycles)+"|"+
            (reach?"REACHABLE\n":"SEPARATED\n");
    }
    QByteArray out="BOARDSI_D051_DUAL_FINITE_WIDTH_LOOP_V1|"+sha+
        "|1|SOURCE_LOCAL_DUAL_FINITE_WIDTH_REDUNDANCY_ONLY|P009_UNSUPPORTED|SI_NOT_RUN"
        "|GLOBAL_ELECTRICAL_NOT_VERIFIED\nC|0|U1_U2|"+net+"|1|12|12\n";
    for(const auto &line:oldWire.split('\n'))
        if(line.startsWith("P|") || line.startsWith("T|"))out+=line+'\n';
    const qint64 a=std::min(second.x0_nm,second.x1_nm)/254;
    const qint64 b=std::max(second.x0_nm,second.x1_nm)/254;
    out+="T|7|1|"+net+"|"+QByteArray::number(a)+"|40000|"+
        QByteArray::number(b)+"|40000|800\n";
    for(const auto &line:oldWire.split('\n'))
        if(line.startsWith("E|"))out+=line+'\n';
    out+="E|10|9|11|POSITIVE_FINITE_WIDTH_TRACE_TRACE_SECOND_BRIDGE\n"
         "E|11|3|11|POSITIVE_FINITE_WIDTH_TRACE_SQUARE_PAD_SECOND_BRIDGE\n";
    out+=diagnoses;
    out+="COUNT|NODES|12|EDGES|12|LOCAL_COMPONENTS|1|CYCLE_RANK|1|GLOBAL_ELECTRICAL|NOT_VERIFIED\n"
         "END|SOURCE_LOCAL_DUAL_FINITE_WIDTH_REDUNDANCY_ONLY_NO_GLOBAL_PASS\n";
    return out;
}
inline bool decode(const QByteArray &receipt,const QByteArray &source,
                   const PreviewImportPanel *full,const QByteArray &reduced,
                   const PreviewImportPanel *baseline) {
    return !receipt.isEmpty() && receipt.size()<16384 &&
        receipt==reconstruct(source,full,reduced,baseline);
}
} // namespace d051
