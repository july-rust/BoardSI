// D_011: source-only layer-local graph; no electrical barrel inference.
#pragma once
#include "d010_two_trace_four_area_qt.h"
namespace d011 {
inline QByteArray expectedGraph(const QByteArray &base,std::array<int,2> layers) {
    const auto r=base.split('\n');
    if(r.size()!=21 || r[18]!="OVERALL|SOURCE_DUAL_LAYER_TWO_TRACE_FOUR_AREA_CANDIDATE")return {};
    const auto h=r[0].split('|');
    if(h.size()!=8 || h[1].size()!=64)return {};
    QByteArray result="BOARDSI_D011_SOURCE_GRAPH_V1|"+h[1]+
        "|SOURCE_GEOMETRY_ONLY|PHYSICAL_NOT_VERIFIED|P009_UNSUPPORTED|SI_NOT_RUN|8|6|2|0\n";
    result+="D010_BEGIN\n"+base+"D010_END\n";
    for(int slot=0;slot<2;++slot) {
        const int layer=layers[slot],o=slot*4;
        const auto local=r[2+slot*3].split('|');
        const auto target=r[2+(slot+2)*3].split('|');
        const auto track=r[14+slot].split('|');
        if(local.size()!=9||target.size()!=9||track.size()!=9
            ||local[4]!=QByteArray::number(layer)||target[4]!=QByteArray::number(layer)
            ||track[2]!=QByteArray::number(layer))return {};
        const auto l=QByteArray::number(layer);
        const QByteArray trace="TRACE:L"+l+":"+track[3]+":"+track[4]+":"
            +track[5]+":"+track[6]+":"+track[7]+":"+track[8];
        const auto node=[&](int i,const QByteArray &kind,const QByteArray &key){
            return "NODE|"+QByteArray::number(i)+"|"+kind+"|"+l+"|"+key+"\n";
        };
        result+=node(o,"VIA_PAD","VIA:0:L"+l);
        result+=node(o+1,"AREA",local[3]);
        result+=node(o+2,"TRACE",trace);
        result+=node(o+3,"AREA",target[3]);
    }
    for(int slot=0;slot<2;++slot) {
        const int o=slot*4; const auto l=QByteArray::number(layers[slot]);
        for(int e=0;e<3;++e) {
            const int a=e==2?o+2:o,b=e==0?o+1:e==1?o+2:o+3;
            result+="EDGE|"+QByteArray::number(slot*3+e)+"|"+
                QByteArray::number(a)+"|"+QByteArray::number(b)+
                "|SOURCE_CONTACT_CANDIDATE|"+l+"\n";
        }
    }
    result+="OVERALL|SOURCE_GRAPH_TWO_LOCAL_COMPONENTS_NOT_ELECTRICAL\n";
    result+="END|SOURCE_ONLY_NO_BARREL_EDGE\n";
    return result.size()<=48000?result:QByteArray();
}
inline bool decode(const QByteArray &wire,const QByteArray &sha,const QByteArray &source,
    const PreviewImportPanel *panel,const QStringList &names,const QString &net,
    std::array<int,2> layers,c005::Receipt &out) {
    if(wire.size()>48000||wire.isEmpty()||!wire.endsWith('\n'))return false;
    const auto rows=wire.split('\n');
    if(rows.size()!=40||rows[1]!="D010_BEGIN"||rows[22]!="D010_END"
        ||rows[38]!="END|SOURCE_ONLY_NO_BARREL_EDGE"||!rows[39].isEmpty())return false;
    QByteArray base;
    for(int i=2;i<=21;++i)base+=rows[i]+"\n";
    c005::Receipt verified;
    if(!d010::decode(base,sha,source,panel,names,net,layers,verified)
        ||verified.state!="SOURCE_DUAL_LAYER_TWO_TRACE_FOUR_AREA_CANDIDATE"
        ||verified.markers.size()!=4)return false;
    // D_010 recomputes every Via/Area/Trace contact using real imported data.
    // Re-materialize each node and edge rather than trusting supplied graph rows.
    if(expectedGraph(base,layers)!=wire)return false;
    verified.state="SOURCE_GRAPH_TWO_LOCAL_COMPONENTS_NOT_ELECTRICAL";
    verified.reason="D_011 8 SOURCE NODES / 6 LOCAL CANDIDATE EDGES; NOT ELECTRICAL";
    out=std::move(verified);
    return true;
}
} // namespace d011
