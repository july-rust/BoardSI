// D_012: source span and importer-assumed plating provenance; NOT an electrical barrel.
#pragma once
#include "d011_source_contact_graph_qt.h"
namespace d012 {
inline QByteArray expectedWitness(const QByteArray &graph, std::array<int,2> layers) {
    if(graph.isEmpty() || graph.size()>48000 || !graph.endsWith('\n'))return {};
    const auto rows=graph.split('\n');
    if(rows.size()!=40 || rows[1]!="D010_BEGIN"
        || rows[22]!="D010_END" || rows[38]!="END|SOURCE_ONLY_NO_BARREL_EDGE")return {};
    const auto h=rows[0].split('|'), v=rows[3].split('|');
    if(h.size()!=10 || h[0]!="BOARDSI_D011_SOURCE_GRAPH_V1"
        || h[1].size()!=64 || v.size()!=9 || v[0]!="VIA" || v[1]!="0")return {};
    QByteArray witness="BOARDSI_D012_VIA_SPAN_PROVENANCE_V1|"+h[1]+
        "|SOURCE_SPAN_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_NOT_VERIFIED|NO_CROSS_LAYER_ELECTRICAL_EDGE\nD011_BEGIN\n";
    witness+=graph+"D011_END\n";
    witness+="SPAN|0|"+v[2]+"|"+v[3]+"|"+v[4]+"|"+v[5]+"|"+v[6]+"|"+
        v[7]+"|"+v[8]+"|"+QByteArray::number(layers[0])+"|"+
        QByteArray::number(layers[1])+"|PLATING_IMPORTER_ASSUMED\n";
    witness+="OVERALL|SOURCE_SPAN_PROVENANCE_ONLY\nEND|NO_ELECTRICAL_BARREL_EDGE\n";
    return witness.size()<=64000?witness:QByteArray();
}
inline bool decode(const QByteArray &wire,const QByteArray &sha,const QByteArray &source,
    const PreviewImportPanel *panel,const QStringList &names,const QString &net,
    std::array<int,2> layers,c005::Receipt &out) {
    static const QByteArray prefix="BOARDSI_D012_VIA_SPAN_PROVENANCE_V1|";
    const QByteArray start="\nD011_BEGIN\n";
    if(wire.isEmpty() || wire.size()>64000 || !wire.endsWith('\n'))return false;
    const auto startPos=wire.indexOf(start);
    if(startPos<0 || !wire.startsWith(prefix+sha+
       "|SOURCE_SPAN_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_NOT_VERIFIED|NO_CROSS_LAYER_ELECTRICAL_EDGE"+start))return false;
    const qsizetype begin=startPos+start.size();
    const qsizetype end=wire.indexOf("D011_END\n",begin);
    if(end<begin)return false;
    const auto graph=wire.mid(begin,end-begin);
    c005::Receipt verified;
    // Independently recompute from actual accepted imported PADS artwork:
    // D_011 -> D_010 confirms Via Source/Canonical drill/span/pads, traces and areas.
    if(!d011::decode(graph,sha,source,panel,names,net,layers,verified)
        || verified.state!="SOURCE_GRAPH_TWO_LOCAL_COMPONENTS_NOT_ELECTRICAL"
        || verified.markers.size()!=4
        || expectedWitness(graph,layers)!=wire)return false;
    verified.state="SOURCE_SPAN_PROVENANCE_IMPORTER_ASSUMED";
    verified.reason="D_012 SOURCE SPAN; IMPORTER-ASSUMED PLATING; BARREL NOT VERIFIED; NO ELECTRICAL EDGE";
    out=std::move(verified);
    return true;
}
} // namespace d012
