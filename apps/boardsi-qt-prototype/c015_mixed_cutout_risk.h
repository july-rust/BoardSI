// C_015: independent Source-geometry-only V11 receipt and imported Qt validation.
// V4 circle fallback explicitly DOES NOT claim V8 Source contact evidence.
#pragma once
namespace c015 {
struct Hole {
    bool round=false;
    std::array<qint64,4> rect{};
    qint64 x=0,y=0,r=0;
};
inline bool coord(qint64 t) {return t>=-5600000 && t<=5600000;}
inline qint64 square(qint64 d) {return d*d;}
inline qint64 d2(qint64 ax,qint64 ay,qint64 bx,qint64 by) {
    return square(ax-bx)+square(ay-by);
}
inline bool within(const Hole &h,const std::array<qint64,4> &o) {
    if(!h.round)return h.rect[0]>o[0] && h.rect[1]>o[1] &&
        h.rect[2]<o[2] && h.rect[3]<o[3] &&
        h.rect[0]<h.rect[2] && h.rect[1]<h.rect[3];
    return h.r>0 && h.x-h.r>o[0] && h.y-h.r>o[1] &&
        h.x+h.r<o[2] && h.y+h.r<o[3];
}
inline bool disjoint(const Hole &a,const Hole &b) {
    if(!a.round && !b.round)
        return a.rect[2]<b.rect[0] || b.rect[2]<a.rect[0] ||
            a.rect[3]<b.rect[1] || b.rect[3]<a.rect[1];
    if(a.round && b.round)
        return d2(a.x,a.y,b.x,b.y)>square(a.r+b.r);
    const Hole &c=a.round?a:b,&p=a.round?b:a;
    const qint64 x=std::clamp(c.x,p.rect[0],p.rect[2]);
    const qint64 y=std::clamp(c.y,p.rect[1],p.rect[3]);
    return d2(x,y,c.x,c.y)>square(c.r);
}
inline QString classify(qint64 x,qint64 y,qint64 diameter,
        const std::vector<Hole> &holes,int &selected) {
    selected=-1;
    if(holes.empty())return "SOURCE_CUTOUT_MISSING_RISK";
    for(size_t i=0;i<holes.size();++i) {
        const Hole &h=holes[i];
        if(h.round) {
            const qint64 r2=square(h.r),distance=d2(x,y,h.x,h.y);
            if(distance>r2)continue;
            selected=static_cast<int>(i);
            const qint64 clear=2*h.r-diameter;
            if(clear<0 || 4*distance>square(clear))
                return "SOURCE_REFERENCE_COPPER_OVERLAP_RISK";
            if(4*distance==square(clear))
                return "SOURCE_CUTOUT_TANGENCY_RISK";
            return "SOURCE_CUTOUT_GAP_OBSERVED";
        }
        const auto &r=h.rect;
        if(x<r[0] || x>r[2] || y<r[1] || y>r[3])continue;
        selected=static_cast<int>(i);
        const qint64 margin=std::min({x-r[0],r[2]-x,y-r[1],r[3]-y});
        if(2*margin>diameter)return "SOURCE_CUTOUT_GAP_OBSERVED";
        if(2*margin==diameter)return "SOURCE_CUTOUT_TANGENCY_RISK";
        return "SOURCE_REFERENCE_COPPER_OVERLAP_RISK";
    }
    return "SOURCE_REFERENCE_COPPER_OVERLAP_RISK";
}
inline bool number(const QByteArray &s,qint64 &n) {
    bool ok=false;n=s.toLongLong(&ok);return ok && coord(n);
}
inline bool decode(const QByteArray &wire,const QByteArray &sha,
        const ImportedArtworkPanel *panel,const std::vector<int> &selected,
        int split,int stitchIndex,const QString &referenceNet,
        const std::array<int,2> &layers,c005::Receipt &out) {
    if(!panel || wire.isEmpty() || wire.size()>290000 || !wire.endsWith('\n') ||
       sha.size()!=64 || selected.size()<2 || selected.size()>16)return false;
    const auto lines=wire.split('\n');
    if(lines.size()<13 || !lines.last().isEmpty())return false;
    const auto h=lines[0].split('|');
    if(h.size()!=14 || h[0]!="BOARDSI_REFERENCE_ROUTE_V11" || h[1]!=sha ||
       h[3]!="UNSUPPORTED" || h[4]!="NOT_RUN" || h[5]!="0" ||
       h[6]!="REFERENCE_TRANSFER_UNVERIFIED" || h[7]!="2" ||
       h[8]!=QByteArray::number(static_cast<int>(selected.size())) ||
       h[9]!=QByteArray::number(split) || h[10]!=QByteArray::number(stitchIndex) ||
       h[12]!="V8" ||
       h[13]!="V8_SOURCE_CONTACT_CANDIDATE")
        return false;
    bool ok=false;
    const int signalIndex=h[11].toInt(&ok);
    if(!ok || signalIndex<0 || signalIndex==stitchIndex)return false;
    ImportedVia signal,stitch;
    if(!panel->importedVia(signalIndex,signal) ||
       !panel->importedVia(stitchIndex,stitch) ||
       signal.net.isEmpty() || signal.net==referenceNet ||
       stitch.net!=referenceNet || signal.diameter_nm<=signal.drill_nm ||
       stitch.diameter_nm<=stitch.drill_nm)return false;
    int pos=1;
    QString overall="SOURCE_CUTOUT_GAP_OBSERVED";
    bool circles=false;
    std::vector<ImportedReferenceRiskMarker> markers;
    for(const int layer:layers) {
        if(layer<signal.firstLayer || layer>signal.lastLayer ||
           layer<stitch.firstLayer || layer>stitch.lastLayer ||
           pos>=lines.size()-2)return false;
        const auto row=lines[pos++].split('|');
        if(row.size()!=10 || row[0]!="LAYER" ||
           row[1]!=QByteArray::number(layer) ||
           row[2]!=QByteArray::number(signalIndex))return false;
        QByteArray sourceId;
        if(!c005::boundedHex(row[3],sourceId))return false;
        qint64 x=0,y=0,diameter=0;
        if(!number(row[4],x) || !number(row[5],y) ||
           !number(row[6],diameter) || diameter<=0 ||
           x*254!=signal.x_nm || y*254!=signal.y_nm ||
           diameter*254!=signal.diameter_nm)return false;
        const int count=row[7].toInt(&ok);
        if(!ok || count<0 || count>8)return false;
        ImportedCopperArea area;
        if(panel->importedReferenceAreaCount(layer,referenceNet)!=1 ||
           !panel->importedCopperGeometry(QString::fromUtf8(sourceId),
                layer,referenceNet,area) ||
           area.circle || area.curved ||
           static_cast<int>(area.cutouts.size()+area.roundCutouts.size())!=count ||
           area.ring.size()!=5)return false;
        qint64 xmin=area.ring[0].first,ymin=area.ring[0].second;
        qint64 xmax=xmin,ymax=ymin;
        for(const auto &p:area.ring) {
            xmin=std::min(xmin,p.first); ymin=std::min(ymin,p.second);
            xmax=std::max(xmax,p.first); ymax=std::max(ymax,p.second);
        }
        if(!c013::rectangle(area.ring,xmin,ymin,xmax,ymax))return false;
        std::array<qint64,4> outer{xmin/254,ymin/254,xmax/254,ymax/254};
        if(2*(x-outer[0])<=diameter || 2*(outer[2]-x)<=diameter ||
           2*(y-outer[1])<=diameter || 2*(outer[3]-y)<=diameter)return false;
        std::vector<Hole> holes;
        std::vector<bool> usedRect(area.cutouts.size(),false);
        std::vector<bool> usedCircle(area.roundCutouts.size(),false);
        for(int j=0;j<count;++j) {
            if(pos>=lines.size()-2)return false;
            const auto r=lines[pos++].split('|');
            if(r.size()<7 || r[0]!="HOLE" ||
               r[1]!=QByteArray::number(layer) ||
               r[2]!=QByteArray::number(j))return false;
            Hole h0;
            bool matched=false;
            if(r[3]=="R" && r.size()==8) {
                for(int k=0;k<4;++k)if(!number(r[4+k],h0.rect[k]))return false;
                if(h0.rect[0]>=h0.rect[2] || h0.rect[1]>=h0.rect[3])return false;
                for(size_t k=0;k<area.cutouts.size();++k) {
                    const auto &c=area.cutouts[k];
                    if(!usedRect[k] && c013::rectangle(c,
                        h0.rect[0]*254,h0.rect[1]*254,
                        h0.rect[2]*254,h0.rect[3]*254)) {
                        matched=true;usedRect[k]=true;break;
                    }
                }
            } else if(r[3]=="C" && r.size()==7) {
                h0.round=true;
                if(!number(r[4],h0.x) || !number(r[5],h0.y) ||
                   !number(r[6],h0.r) || h0.r<=0)return false;
                for(size_t k=0;k<area.roundCutouts.size();++k) {
                    const auto &c=area.roundCutouts[k];
                    if(!usedCircle[k] && c.cx_nm==h0.x*254 &&
                       c.cy_nm==h0.y*254 && c.radius_nm==h0.r*254) {
                        matched=true;usedCircle[k]=true;break;
                    }
                }
                circles=true;
            } else return false;
            if(!matched || !within(h0,outer))return false;
            for(const auto &prior:holes)if(!disjoint(prior,h0))return false;
            holes.push_back(h0);
        }
        if(std::any_of(usedRect.begin(),usedRect.end(),[](bool v){return !v;}) ||
           std::any_of(usedCircle.begin(),usedCircle.end(),[](bool v){return !v;}))
            return false;
        int chosen=-1;
        const QString state=classify(x,y,diameter,holes,chosen);
        if(row[8]!=state.toLatin1() ||
           row[9]!=(chosen<0?QByteArray("-"):QByteArray::number(chosen)))
            return false;
        if(c013::severity(state)>c013::severity(overall))overall=state;
        ImportedReferenceRiskMarker m;
        m.state="SIGNAL_MULTI_CUTOUT_CLEARANCE_RISK";
        m.reason=QString("C_015 / L%1 / Source Area %2 / Mixed Cutout count %3 / "
                          "Source state %4 / Original V8 contact %5 / "
                          "SOURCE GEOMETRY ONLY / P009 UNSUPPORTED / SI NOT_RUN")
            .arg(layer).arg(QString::fromUtf8(sourceId)).arg(count)
            .arg(state).arg("source-local candidate / P009 UNSUPPORTED");
        m.net=signal.net;m.viaObject=signal.name;m.sourceObject=QString::fromUtf8(sourceId);
        m.layer=layer;m.x_nm=signal.x_nm;m.y_nm=signal.y_nm;
        m.diameter_nm=signal.diameter_nm;
        markers.push_back(std::move(m));
    }
    if(h[2]!=overall.toLatin1() ||
       pos>=lines.size()-2)return false;
    const auto row=lines[pos++].split('|');
    if(row.size()!=3 || row[0]!="LEGACY" || row[1]!=h[12])return false;
    const int n=row[2].toInt(&ok);
    if(!ok || n<8 || n>1000 || pos+n!=lines.size()-2 ||
       lines[pos+n]!="END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")
        return false;
    QByteArray legacy;
    for(int j=0;j<n;++j) {legacy+=lines[pos+j];legacy+='\n';}
    c005::Receipt receipt;
    const bool valid=c012::decode(legacy,sha,panel,selected,split,stitchIndex,referenceNet,layers,receipt);
    if(!valid)return false;
    receipt.markers.insert(receipt.markers.end(),markers.begin(),markers.end());
    receipt.state=overall;
    receipt.reason=circles?
        "C_015 COPCCO SOURCE ONLY / ORIGINAL V8 SOURCE LOCAL CONTACT CANDIDATE / P009 UNSUPPORTED / SI NOT_RUN":
        "C_015 RECT SOURCE ONLY / ORIGINAL V8 SOURCE LOCAL CONTACT CANDIDATE / P009 UNSUPPORTED / SI NOT_RUN";
    out=std::move(receipt);
    return true;
}
} // namespace c015
