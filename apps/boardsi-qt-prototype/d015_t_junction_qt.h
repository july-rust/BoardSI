// D_015 independent Qt reconstruction of bounded T-junction source witness.
// Never treat imported plating assumptions as electrical barrel proof.
#pragma once
#include "d014_two_via_qt.h"

namespace d015 {
// Independent strict source-section inventory: an unseen *LINES*/COPCLS
// or duplicate VIA must never slip through a route-only Source witness.
inline bool boundedSourceInventory(const QByteArray &source,const QString &viaName) {
    QByteArray section;
    bool header=false,ended=false,bottom=false,top=false;
    int maxLayer=0,viaDefs=0,signalCount=0,pairs=0,points=0;
    for (const auto &raw:source.split('\n')) {
        const auto line=raw.trimmed();
        if (line.isEmpty()) continue;
        if (ended) return false;
        if (!header) {
            if (!line.startsWith("!PADS-POWERPCB-V9.4-MILS!")) return false;
            header=true;
            continue;
        }
        const auto words=line.simplified().split(' ');
        if (words.isEmpty()) continue;
        const auto &first=words[0];
        if (first=="*REMARK*" || first=="REMARK") continue;
        if (first=="*PCB*" && section.isEmpty()) {section="pcb";continue;}
        if (first=="*VIA*" && section=="pcb" && maxLayer==1) {
            section="via";continue;
        }
        if (first=="*ROUTE*" && section=="via" && viaDefs==1 && bottom && top) {
            section="route";continue;
        }
        if (first=="*END*" && section=="route" && words.size()==1) {
            ended=true;continue;
        }
        if (first=="*SIGNAL*" && section=="route" && words.size()==2) {
            ++signalCount;continue;
        }
        if (first.startsWith('*')) return false; // includes unmodeled COPCLS
        if (section=="pcb" && words.size()==2 && first=="MAXLAYER" && words[1]=="4") {
            if (++maxLayer!=1) return false;
        } else if (section=="via" && first==viaName.toUtf8() && words.size()==3 && words[2]=="2") {
            if (++viaDefs!=1) return false;
        } else if (section=="via" && words.size()==3 && (first=="-2" || first=="0")) {
            if (first=="-2") {if (bottom) return false;bottom=true;}
            else {if (top) return false;top=true;}
        } else if (section=="route" && signalCount>0 && words.size()==2
                   && words[0].contains('.') && words[1].contains('.')) {
            ++pairs;
        } else if (section=="route" && signalCount>0
                   && (words.size()==5 || words.size()==6)) {
            for (int i=0;i<5;++i) {
                bool ok=false; words[i].toLongLong(&ok);if (!ok) return false;
            }
            if (words.size()==6 && words[5]!=viaName.toUtf8()) return false;
            ++points;
        } else {
            return false;
        }
    }
    return header && ended && maxLayer==1 && viaDefs==1 && bottom && top
        && signalCount==2 && pairs==2 && points==6;
}
inline bool decode(const QByteArray &wire, const QByteArray &source,
                   const PreviewImportPanel *panel, const QString &net, c005::Receipt &out) {
    if (!panel || !panel->imported() || wire.isEmpty() || wire.size() > 16384
        || source.isEmpty() || source.size() > 8*1024*1024) return false;
    const auto sha=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    if (panel->acceptedSourceSha256()!=sha
        || panel->imported()->importedSkippedCount()!=0) return false;
    const auto *art=panel->imported();
    ImportedVia via[2], extra;
    if (!art->importedVia(0,via[0]) || !art->importedVia(1,via[1])
        || art->importedVia(2,extra) || art->importedTraceCount()!=4
        || art->arcCount()!=0) return false;
    const auto &a=via[0], &b=via[1];
    if (a.net!=net || b.net!=net || a.name.isEmpty() || a.name!=b.name
        || (a.x_nm==b.x_nm && a.y_nm==b.y_nm)
        || a.firstLayer!=1 || a.lastLayer!=4 || b.firstLayer!=1 || b.lastLayer!=4
        || a.drill_nm<=0 || a.diameter_nm<=a.drill_nm
        || a.diameter_nm!=b.diameter_nm || a.drill_nm!=b.drill_nm
        || !boundedSourceInventory(source,a.name)
        || !d014::twoRowViaDefinition(source,a.name)
        || !d006::declaredConductivePad(source,a,1)
        || !d006::declaredConductivePad(source,a,4)
        || !d006::declaredConductivePad(source,b,1)
        || !d006::declaredConductivePad(source,b,4)) return false;
    const auto exact=[](qint64 value) {
        return value%254==0 && value>=-1422400000LL && value<=1422400000LL;
    };
    for (const auto value : {a.x_nm,a.y_nm,b.x_nm,b.y_nm,a.diameter_nm,a.drill_nm})
        if (!exact(value)) return false;
    ImportedTrack t[4];
    for (int i=0;i<4;++i) {
        if (!art->importedTrack(i,t[i]) || t[i].net!=net
            || t[i].width_nm<=0 || !exact(t[i].width_nm)
            || !exact(t[i].x0_nm) || !exact(t[i].x1_nm)
            || !exact(t[i].y0_nm) || !exact(t[i].y1_nm)) return false;
    }
    if (t[0].layer!=1 || t[1].layer!=4 || t[2].layer!=1 || t[3].layer!=4
        || t[0].x1_nm!=a.x_nm || t[0].y1_nm!=a.y_nm
        || t[1].x0_nm!=a.x_nm || t[1].y0_nm!=a.y_nm
        || t[1].x1_nm!=b.x_nm || t[1].y1_nm!=b.y_nm
        || t[2].x0_nm!=b.x_nm || t[2].y0_nm!=b.y_nm
        || (t[0].x0_nm==b.x_nm && t[0].y0_nm==b.y_nm)
        || (t[2].x1_nm==a.x_nm && t[2].y1_nm==a.y_nm)
        || (t[0].x0_nm==t[2].x1_nm && t[0].y0_nm==t[2].y1_nm))
        return false;
    const auto grid=[](qint64 x,qint64 y){return d014::GridPoint{x/254,y/254};};
    const auto pa=grid(a.x_nm,a.y_nm),pb=grid(b.x_nm,b.y_nm);
    const auto armA0=grid(t[0].x0_nm,t[0].y0_nm),armA1=grid(t[0].x1_nm,t[0].y1_nm);
    const auto armB0=grid(t[2].x0_nm,t[2].y0_nm),armB1=grid(t[2].x1_nm,t[2].y1_nm);
    if (d014::onSegment(pb,armA0,armA1)
        || d014::onSegment(pa,armB0,armB1)
        || d014::segmentsTouch(armA0,armA1,armB0,armB1)) return false;
    const auto main0=grid(t[1].x0_nm,t[1].y0_nm);
    const auto main1=grid(t[1].x1_nm,t[1].y1_nm);
    const auto free=grid(t[3].x0_nm,t[3].y0_nm);
    const auto join=grid(t[3].x1_nm,t[3].y1_nm);
    if ((join.x==main0.x && join.y==main0.y)
        || (join.x==main1.x && join.y==main1.y)
        || (free.x==join.x && free.y==join.y)
        || !d014::onSegment(join,main0,main1)
        || d014::onSegment(free,main0,main1)
        || (main1.x-main0.x)*(join.x-free.x)
            +(main1.y-main0.y)*(join.y-free.y)!=0) return false;
    QByteArray expected="BOARDSI_D015_T_JUNCTION_V1|"+sha+
        "|SOURCE_GEOMETRY_ONLY|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED"
        "|CROSS_LAYER_ELECTRICAL_NOT_PROVEN|SOLVER_READY_FALSE\n";
    for (int i=0;i<2;++i) {
        const auto &v=via[i];
        expected+="VIA|"+QByteArray::number(i)+"|"+QByteArray::number(v.x_nm/254)
            +"|"+QByteArray::number(v.y_nm/254)+"|"+QByteArray::number(v.diameter_nm/254)
            +"|"+QByteArray::number(v.drill_nm/254)+"|"+QByteArray::number(v.firstLayer)
            +"|"+QByteArray::number(v.lastLayer)+"|"+v.name.toUtf8()+"\n";
    }
    for (int i=0;i<4;++i) {
        const auto &v=t[i];
        expected+="TRACE|"+QByteArray::number(i)+"|"+QByteArray::number(v.layer)
            +"|"+QByteArray::number(v.x0_nm/254)+"|"+QByteArray::number(v.y0_nm/254)
            +"|"+QByteArray::number(v.x1_nm/254)+"|"+QByteArray::number(v.y1_nm/254)
            +"|"+QByteArray::number(v.width_nm/254)+"\n";
    }
    expected+="NODE|0|VIA_PAD|1|0\nNODE|1|VIA_PAD|4|0\nNODE|2|VIA_PAD|4|1\n"
        "NODE|3|VIA_PAD|1|1\nNODE|4|TRACE|1|0\nNODE|5|TRACE|4|1\n"
        "NODE|6|TRACE|1|2\nNODE|7|TRACE|4|3\n"
        "EDGE|0|0|4|SOURCE_ENDPOINT_CONTACT_CANDIDATE|1\n"
        "EDGE|1|1|5|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4\n"
        "EDGE|2|2|5|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4\n"
        "EDGE|3|3|6|SOURCE_ENDPOINT_CONTACT_CANDIDATE|1\n"
        "EDGE|4|5|7|SOURCE_INTERIOR_T_CONTACT_CANDIDATE|4\n"
        "COUNT|NODES|8|LAYER_LOCAL_EDGES|5|LOCAL_COMPONENTS|3|ELECTRICAL_CROSS_LAYER_EDGES|0\n"
        "OVERALL|T_JUNCTION_SOURCE_CANDIDATE_ONLY\nEND|FAIL_CLOSED_NO_BARREL_PROOF\n";
    if (expected!=wire) return false;
    out.state="T_JUNCTION_SOURCE_CANDIDATE_NOT_ELECTRICAL";
    out.reason="D_015 SOURCE CONTACT ONLY; BARREL NOT VERIFIED";
    return true;
}
} // namespace d015
