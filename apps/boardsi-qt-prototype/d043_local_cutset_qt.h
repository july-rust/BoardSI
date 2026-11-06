// D_043: source-local counterfactual trace cuts; independent native Qt reconstruction.
#pragma once
#include "d042_local_closed_loop_qt.h"
#include <QVector>
#include <array>
#include <utility>

namespace d043 {
inline bool validCuts(const QVector<int> &cuts) {
    if(cuts.isEmpty())return true;
    if(cuts.size()==1)return cuts[0]>=4 && cuts[0]<=9;
    return cuts.size()==2 && cuts[0]==4 && cuts[1]>=5 && cuts[1]<=9;
}
inline QByteArray reconstruct(const QByteArray &src,const PreviewImportPanel *panel,
                               const QVector<int> &cuts) {
    if(!validCuts(cuts))return {};
    // D_042 Qt independently revalidates CURRENT ImportedArtwork, SOURCE
    // Pin-Pair/geometry, freshness and the full 14-node closed-loop receipt.
    const auto full=d042::reconstruct(src,panel);
    if(full.isEmpty())return {};
    const auto header=full.split('\n').first().split('|');
    if(header.size()<2 || header[0]!="BOARDSI_D042_LOCAL_CLOSED_LOOP_V1" ||
       header[1].size()!=64)return {};
    std::array<std::pair<int,int>,14> links={};
    int count=0;
    for(const auto &line:full.split('\n')) {
        if(!line.startsWith("E|"))continue;
        const auto fields=line.split('|');
        if(fields.size()!=5 || count>=14)return {};
        bool ok0=false,ok1=false,ok2=false;
        const int index=fields[1].toInt(&ok0);
        const int a=fields[2].toInt(&ok1);
        const int b=fields[3].toInt(&ok2);
        if(!ok0 || !ok1 || !ok2 || index!=count || a<0 || a>=14 ||
           b<0 || b>=14 || a==b)return {};
        links[static_cast<size_t>(count++)]={a,b};
    }
    if(count!=14)return {};
    std::array<bool,14> enabled={};
    enabled.fill(true);
    for(const int cut:cuts)enabled[static_cast<size_t>(4+cut)]=false;
    std::array<int,14> parents={};
    for(int i=0;i<14;++i)parents[static_cast<size_t>(i)]=i;
    const auto root=[&](int n) {
        while(parents[static_cast<size_t>(n)]!=n)n=parents[static_cast<size_t>(n)];
        return n;
    };
    int nodes=0,edges=0,cycles=0,components=0;
    for(const bool on:enabled)if(on)++nodes;
    for(const auto &edge:links) {
        if(!enabled[static_cast<size_t>(edge.first)] ||
           !enabled[static_cast<size_t>(edge.second)])continue;
        ++edges;
        const int a=root(edge.first),b=root(edge.second);
        if(a==b)++cycles;
        else parents[static_cast<size_t>(a)]=b;
    }
    std::array<bool,14> seen={};
    for(int i=0;i<14;++i) {
        if(!enabled[static_cast<size_t>(i)])continue;
        const int r=root(i);
        if(!seen[static_cast<size_t>(r)]) {
            seen[static_cast<size_t>(r)]=true;
            ++components;
        }
    }
    if(edges-nodes+components!=cycles ||
       root(0)!=root(1) || root(2)!=root(3))return {};
    const bool reachable=root(0)==root(2);
    QByteArray cutName="NONE";
    if(!cuts.isEmpty()) {
        cutName.clear();
        for(const int cut:cuts) {
            if(!cutName.isEmpty())cutName+=',';
            cutName+="T"+QByteArray::number(cut);
        }
    }
    QByteArray wire="BOARDSI_D043_LOCAL_CUTSET_V1|"+header[1]+
        "|SOURCE_LOCAL_COUNTERFACTUAL_ONLY|P009_UNSUPPORTED|SI_NOT_RUN"
        "|GLOBAL_ELECTRICAL_NOT_VERIFIED\nCUT|"+cutName+"\n";
    wire+="COUNT|NODES|"+QByteArray::number(nodes)+
          "|EDGES|"+QByteArray::number(edges)+
          "|LOCAL_COMPONENTS|"+QByteArray::number(components)+
          "|LOCAL_CYCLES|"+QByteArray::number(cycles)+
          "|U1_U2_REACHABLE|"+(reachable?"YES":"NO")+"\n";
    wire+="END|SOURCE_LOCAL_DIAGNOSTIC_NO_PHYSICAL_OPEN_SHORT\n";
    return wire;
}
inline bool decode(const QByteArray &wire,const QByteArray &src,
                   const PreviewImportPanel *panel,const QVector<int> &cuts) {
    const auto independent=reconstruct(src,panel,cuts);
    return !independent.isEmpty() && independent==wire;
}
} // namespace d043
