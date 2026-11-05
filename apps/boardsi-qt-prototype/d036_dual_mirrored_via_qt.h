// D_036 independent Qt reconstruction of one dual-negative-4:3 Via SOURCE
// candidate graph. NOT a manufactured plated barrel or global electrical proof.
#pragma once
#include "d015_t_junction_qt.h"

namespace d036 {
inline bool decode(const QByteArray &wire, const QByteArray &source,
                   const PreviewImportPanel *panel, const QString &net,
                   c005::Receipt &out) {
    if (!panel || !panel->imported() || wire.isEmpty() || wire.size()>16384
        || source.isEmpty() || source.size()>8*1024*1024) return false;
    const auto sha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    if (panel->acceptedSourceSha256()!=sha
        || panel->imported()->importedSkippedCount()!=0) return false;
    const auto *art=panel->imported();
    ImportedVia a,b,extraVia;
    if (!art->importedVia(0,a) || !art->importedVia(1,b)
        || art->importedVia(2,extraVia) || art->importedTraceCount()!=5
        || art->arcCount()!=0) return false;
    if (a.net!=net || b.net!=net || a.name.isEmpty() || a.name!=b.name
        || a.firstLayer!=1 || a.lastLayer!=4
        || b.firstLayer!=1 || b.lastLayer!=4
        || (a.x_nm==b.x_nm && a.y_nm==b.y_nm)
        || a.drill_nm<=0 || a.diameter_nm<=a.drill_nm
        || a.drill_nm!=b.drill_nm || a.diameter_nm!=b.diameter_nm
        || !d015::boundedSourceInventory(source,a.name,3,8)
        || !d014::twoRowViaDefinition(source,a.name)
        || !d006::declaredConductivePad(source,a,1)
        || !d006::declaredConductivePad(source,a,4)
        || !d006::declaredConductivePad(source,b,1)
        || !d006::declaredConductivePad(source,b,4)) return false;
    const auto exact=[](qint64 value) {
        return value%254==0 && value>=-1422400000LL && value<=1422400000LL;
    };
    for (const auto value:{a.x_nm,a.y_nm,b.x_nm,b.y_nm,a.drill_nm,a.diameter_nm})
        if (!exact(value)) return false;
    ImportedTrack t[5];
    for (int i=0;i<5;++i) {
        if (!art->importedTrack(i,t[i]) || t[i].net!=net
            || t[i].width_nm<=0 || !exact(t[i].width_nm)
            || !exact(t[i].x0_nm) || !exact(t[i].y0_nm)
            || !exact(t[i].x1_nm) || !exact(t[i].y1_nm)
            || (t[i].x0_nm==t[i].x1_nm && t[i].y0_nm==t[i].y1_nm))
            return false;
    }
    if (t[0].layer!=1 || t[1].layer!=4 || t[2].layer!=1
        || t[3].layer!=4 || t[4].layer!=4
        || t[0].x1_nm!=a.x_nm || t[0].y1_nm!=a.y_nm
        || t[1].x0_nm!=a.x_nm || t[1].y0_nm!=a.y_nm
        || t[1].x1_nm!=b.x_nm || t[1].y1_nm!=b.y_nm
        || t[2].x0_nm!=b.x_nm || t[2].y0_nm!=b.y_nm) return false;
    const auto grid=[](qint64 x,qint64 y) {
        return d014::GridPoint{x/254,y/254};
    };
    const auto pa=grid(a.x_nm,a.y_nm),pb=grid(b.x_nm,b.y_nm);
    const auto armA0=grid(t[0].x0_nm,t[0].y0_nm),armA1=grid(t[0].x1_nm,t[0].y1_nm);
    const auto armB0=grid(t[2].x0_nm,t[2].y0_nm),armB1=grid(t[2].x1_nm,t[2].y1_nm);
    if (d014::onSegment(pb,armA0,armA1)
        || d014::onSegment(pa,armB0,armB1)
        || d014::segmentsTouch(armA0,armA1,armB0,armB1)) return false;
    const auto main0=grid(t[1].x0_nm,t[1].y0_nm);
    const auto main1=grid(t[1].x1_nm,t[1].y1_nm);
    const auto mx=main1.x-main0.x,my=main1.y-main0.y;
    const auto len2=mx*mx+my*my;
    const auto same=[](const d014::GridPoint &u,const d014::GridPoint &v) {
        return u.x==v.x && u.y==v.y;
    };
    const auto contact=[&](int trace,const d014::GridPoint &via,bool atA) {
        const auto u=grid(t[trace].x0_nm,t[trace].y0_nm);
        const auto v=grid(t[trace].x1_nm,t[trace].y1_nm);
        if (same(u,v) || same(u,via)==same(v,via)) return false;
        const auto exterior=same(u,via)?v:u;
        const qint64 dx=exterior.x-via.x,dy=exterior.y-via.y;
        const qint64 dot=mx*dx+my*dy,cross=mx*dy-my*dx;
        const qint64 projection=(exterior.x-main0.x)*mx+(exterior.y-main0.y)*my;
        return dot!=0 && cross<0 && 4*qAbs(cross)==3*qAbs(dot)
            && (atA ? (same(via,main0) && projection<0)
                    : (same(via,main1) && projection>len2));
    };
    if (!contact(3,pa,true) || !contact(4,pb,false)) return false;

    QByteArray expected="BOARDSI_D036_DUAL_MIRRORED_VIA_ENDPOINT_CONTACT_V1|"+sha+
        "|SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED"
        "|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE\n";
    const ImportedVia vias[2]={a,b};
    for (int i=0;i<2;++i) {
        const auto &v=vias[i];
        expected+="VIA|"+QByteArray::number(i)+"|"+QByteArray::number(v.x_nm/254)
            +"|"+QByteArray::number(v.y_nm/254)+"|"+QByteArray::number(v.diameter_nm/254)
            +"|"+QByteArray::number(v.drill_nm/254)+"|"+QByteArray::number(v.firstLayer)
            +"|"+QByteArray::number(v.lastLayer)+"|"+v.name.toUtf8()+"\n";
    }
    for (int i=0;i<5;++i) {
        const auto &v=t[i];
        expected+="TRACE|"+QByteArray::number(i)+"|"+QByteArray::number(v.layer)
            +"|"+QByteArray::number(v.x0_nm/254)+"|"+QByteArray::number(v.y0_nm/254)
            +"|"+QByteArray::number(v.x1_nm/254)+"|"+QByteArray::number(v.y1_nm/254)
            +"|"+QByteArray::number(v.width_nm/254)+"\n";
    }
    expected+="CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|3|VIA|A|X|"
        +QByteArray::number(pa.x)+"|Y|"+QByteArray::number(pa.y)+"\n"
        +"CONTACT|MAIN_TRACE|1|ENDPOINT_TRACE|4|VIA|B|X|"
        +QByteArray::number(pb.x)+"|Y|"+QByteArray::number(pb.y)+"\n";
    expected+="NODE|0|VIA_PAD|1|0\nNODE|1|VIA_PAD|4|0\n"
        "NODE|2|VIA_PAD|4|1\nNODE|3|VIA_PAD|1|1\n"
        "NODE|4|TRACE|1|0\nNODE|5|TRACE|4|1\n"
        "NODE|6|TRACE|1|2\nNODE|7|TRACE|4|3\nNODE|8|TRACE|4|4\n"
        "EDGE|0|0|4|SOURCE_ENDPOINT_CONTACT_CANDIDATE|1\n"
        "EDGE|1|1|5|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4\n"
        "EDGE|2|2|5|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4\n"
        "EDGE|3|3|6|SOURCE_ENDPOINT_CONTACT_CANDIDATE|1\n"
        "EDGE|4|5|7|SOURCE_NON45_OBLIQUE_MIRRORED_VIA_A_ENDPOINT_CONTACT_CANDIDATE|4\n"
        "EDGE|5|1|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4\n"
        "EDGE|6|5|8|SOURCE_NON45_OBLIQUE_MIRRORED_VIA_B_ENDPOINT_CONTACT_CANDIDATE|4\n"
        "EDGE|7|2|8|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4\n"
        "COUNT|NODES|9|LAYER_LOCAL_EDGES|8|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0\n"
        "OVERALL|DUAL_MIRRORED_NEGATIVE_4_3_VIA_ENDPOINT_SOURCE_CANDIDATE_ONLY\n"
        "END|FAIL_CLOSED_NO_BARREL_PROOF\n";
    if (wire!=expected) return false;
    out.state="DUAL_MIRRORED_VIA_ENDPOINT_SOURCE_CANDIDATE_NOT_ELECTRICAL";
    out.reason="D_036 dual Via-A/B signed 4:3 SOURCE only; barrel and electrical NOT VERIFIED";
    return true;
}
} // namespace d036
