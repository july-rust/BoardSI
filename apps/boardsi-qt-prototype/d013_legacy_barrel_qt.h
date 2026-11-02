// D_013: SOURCE-only legacy barrel compatibility projection, never electrical proof.
#pragma once
#include "d012_via_span_provenance_qt.h"
namespace d013 {
inline QByteArray expectedWitness(const QByteArray &span, std::array<int,2> layers) {
    if(span.isEmpty() || span.size()>64000 || !span.endsWith('\n')
        || layers[0]<=0 || layers[0]>=layers[1])return {};
    const auto header=span.left(span.indexOf('\n')).split('|');
    if(header.size()!=6 || header[0]!="BOARDSI_D012_VIA_SPAN_PROVENANCE_V1"
        || header[1].size()!=64 || header[2]!="SOURCE_SPAN_ONLY"
        || header[3]!="PLATING_IMPORTER_ASSUMED"
        || header[4]!="BARREL_NOT_VERIFIED"
        || header[5]!="NO_CROSS_LAYER_ELECTRICAL_EDGE"
        || !span.endsWith("END|NO_ELECTRICAL_BARREL_EDGE\n"))return {};
    QByteArray wire="BOARDSI_D013_LEGACY_BARREL_PROVENANCE_V1|"+header[1]+
        "|SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED"
        "|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE\nD012_BEGIN\n";
    wire+=span+"D012_END\n";
    wire+="LEGACY|0|VIA:0:L"+QByteArray::number(layers[0])+
        "|VIA:0:L"+QByteArray::number(layers[1])+
        "|V1_COMPATIBILITY_PROJECTION|PLATING_IMPORTER_ASSUMED"
        "|BARREL_ELECTRICAL_NOT_VERIFIED\n";
    wire+="COUNT|LEGACY_COMPAT_BARRELS|1|SOURCE_QUALIFIED_LOCAL_CONTACTS_NOT_ELECTRICAL\n"
          "OVERALL|LEGACY_PROJECTION_NOT_ELECTRICAL\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n";
    return wire.size()<=72000?wire:QByteArray();
}
inline bool decode(const QByteArray &wire,const QByteArray &sha,const QByteArray &source,
    const PreviewImportPanel *panel,const QStringList &names,const QString &net,
    std::array<int,2> layers,c005::Receipt &out) {
    static const QByteArray delimiter="\nD012_BEGIN\n";
    if(wire.isEmpty() || wire.size()>72000 || !wire.endsWith('\n'))return false;
    const auto start=wire.indexOf(delimiter);
    if(start<0)return false;
    const auto begin=start+delimiter.size();
    const auto end=wire.indexOf("D012_END\n",begin);
    if(end<begin)return false;
    const auto span=wire.mid(begin,end-begin);
    c005::Receipt verified;
    // D_012 independently reconstructs D_011/D_010 from actually imported PADS.
    // A compatibility projection must NOT create an electrical cross-layer edge.
    if(!d012::decode(span,sha,source,panel,names,net,layers,verified)
        || verified.state!="SOURCE_SPAN_PROVENANCE_IMPORTER_ASSUMED"
        || expectedWitness(span,layers)!=wire)return false;
    verified.state="LEGACY_BARREL_COMPATIBILITY_NOT_ELECTRICAL";
    verified.reason="D_013 LEGACY V1 BARREL COMPATIBILITY ONLY; IMPORTER-ASSUMED PLATING; ELECTRICAL BARREL NOT VERIFIED";
    out=std::move(verified);
    return true;
}
} // namespace d013
