// C_016: independent imported Source round stitch-pad vs rectangle/COPCUT/COPCCO.
// Original V11 (including its genuine V8 chain) must pass independently.
#pragma once
namespace c016 {
inline QString classify(qint64 x,qint64 y,qint64 diameter,
    const std::array<qint64,4> &o,const std::vector<c015::Hole> &holes,int &chosen) {
    chosen=-1;
    const qint64 m=std::min({x-o[0],o[2]-x,y-o[1],o[3]-y});
    QString state=2*m<diameter?"STITCH_PAD_OUTER_CROSSING_RISK":
        2*m==diameter?"STITCH_PAD_OUTER_TANGENCY_RISK":
        "STITCH_PAD_FULL_COPPER_FOOTPRINT_CANDIDATE";
    auto rank=[](const QString &s)->int {
        if(s=="STITCH_PAD_FULL_COPPER_FOOTPRINT_CANDIDATE")return 0;
        if(s=="STITCH_PAD_OUTER_TANGENCY_RISK" || s=="STITCH_PAD_CUTOUT_TANGENCY_RISK")return 1;
        return 2;
    };
    for(size_t i=0;i<holes.size();++i) {
        const auto &h=holes[i];
        qint64 a=0,limit=0;
        if(h.round) {
            a=4*c015::d2(x,y,h.x,h.y);
            limit=c015::square(2*h.r+diameter);
        } else {
            const qint64 cx=std::clamp(x,h.rect[0],h.rect[2]);
            const qint64 cy=std::clamp(y,h.rect[1],h.rect[3]);
            a=4*c015::d2(x,y,cx,cy);
            limit=c015::square(diameter);
        }
        if(a==0)return QString();
        const QString next=a<limit?"STITCH_PAD_CUTOUT_INTERSECTION_RISK":
            a==limit?"STITCH_PAD_CUTOUT_TANGENCY_RISK":QString();
        if(!next.isEmpty() && rank(next)>rank(state)) {
            state=next;chosen=static_cast<int>(i);
        }
    }
    return state;
}
inline bool decode(const QByteArray &wire,const QByteArray &sha,
    const ImportedArtworkPanel *panel,const std::vector<int> &selected,
    int split,int stitchIndex,const QString &referenceNet,
    const std::array<int,2> &layers,c005::Receipt &out) {
    const auto reject=[](int line)->bool {qWarning()<<"C016_QT_REJECT"<<line;return false;};
    if(!panel || wire.size()>340000 || wire.size()<50 ||
       !wire.endsWith('\n') || sha.size()!=64)return reject(__LINE__);
    const auto lines=wire.split('\n');
    if(lines.size()<18 || !lines.last().isEmpty())return reject(__LINE__);
    const auto h=lines[0].split('|');
    if(h.size()!=13 || h[0]!="BOARDSI_REFERENCE_ROUTE_V12" ||
       h[1]!=sha || h[3]!="UNSUPPORTED" || h[4]!="NOT_RUN" ||
       h[5]!="0" || h[6]!="REFERENCE_TRANSFER_UNVERIFIED" ||
       h[7]!="2" || h[8]!=QByteArray::number(static_cast<int>(selected.size())) ||
       h[9]!=QByteArray::number(split) ||
       h[10]!=QByteArray::number(stitchIndex) ||
       h[12]!="V11" ||
       lines[lines.size()-2]!="END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")
        return reject(__LINE__);
    bool ok=false;
    const int signalIndex=h[11].toInt(&ok);
    if(!ok || signalIndex<0 || signalIndex==stitchIndex)return reject(__LINE__);
    ImportedVia stitch;
    if(!panel->importedVia(stitchIndex,stitch) || stitch.net!=referenceNet ||
       stitch.diameter_nm<=stitch.drill_nm ||
       stitch.x_nm%254 || stitch.y_nm%254 || stitch.diameter_nm%254)
       return reject(__LINE__);
    const auto legacyInfo=lines[3].split('|');
    if(legacyInfo.size()!=3 || legacyInfo[0]!="LEGACY" || legacyInfo[1]!="V11")
        return reject(__LINE__);
    const int n=legacyInfo[2].toInt(&ok);
    if(!ok || n<15 || n>1000 || 4+n!=lines.size()-2)return reject(__LINE__);
    QByteArray legacy;
    for(int i=0;i<n;++i){legacy+=lines[i+4];legacy+='\n';}
    c005::Receipt original;
    if(!c015::decode(legacy,sha,panel,selected,split,stitchIndex,
                      referenceNet,layers,original))return reject(__LINE__);
    const auto l11=legacy.split('\n');
    const auto h11=l11[0].split('|');
    if(h11.size()!=14 || h11[0]!="BOARDSI_REFERENCE_ROUTE_V11" ||
       h11[1]!=sha || h11[10]!=h[10] || h11[11]!=h[11])
        return reject(__LINE__);
    std::vector<QByteArray> sourceIds;
    std::vector<std::vector<c015::Hole>> sourceHoles;
    for(int pos=1;pos<l11.size()-2;) {
        const auto row=l11[pos].split('|');
        if(row[0]=="LEGACY")break;
        if(row.size()!=10 || row[0]!="LAYER")return reject(__LINE__);
        const int layer=row[1].toInt(&ok);
        if(!ok || sourceIds.size()>=2 ||
           layer!=layers[sourceIds.size()])return reject(__LINE__);
        sourceIds.push_back(row[3]);
        const int count=row[7].toInt(&ok);
        if(!ok || count<0 || count>8)return reject(__LINE__);
        ++pos;
        std::vector<c015::Hole> holes;
        for(int i=0;i<count;++i) {
            if(pos>=l11.size()-2)return reject(__LINE__);
            const auto rr=l11[pos++].split('|');
            if(rr.size()<7 || rr[0]!="HOLE" ||
               rr[1]!=QByteArray::number(layer) ||
               rr[2]!=QByteArray::number(i))return reject(__LINE__);
            c015::Hole hole;
            if(rr[3]=="R" && rr.size()==8) {
                for(int k=0;k<4;++k)
                    if(!c015::number(rr[4+k],hole.rect[k]))return reject(__LINE__);
            } else if(rr[3]=="C" && rr.size()==7) {
                hole.round=true;
                if(!c015::number(rr[4],hole.x) ||
                   !c015::number(rr[5],hole.y) ||
                   !c015::number(rr[6],hole.r))return reject(__LINE__);
            } else return reject(__LINE__);
            holes.push_back(hole);
        }
        sourceHoles.push_back(std::move(holes));
    }
    if(sourceIds.size()!=2 || sourceHoles.size()!=2)return reject(__LINE__);
    QString overall="STITCH_PAD_FULL_COPPER_FOOTPRINT_CANDIDATE";
    auto rank=[](const QString &s)->int {
        if(s=="STITCH_PAD_FULL_COPPER_FOOTPRINT_CANDIDATE")return 0;
        if(s=="STITCH_PAD_OUTER_TANGENCY_RISK" || s=="STITCH_PAD_CUTOUT_TANGENCY_RISK")return 1;
        return 2;
    };
    std::vector<ImportedReferenceRiskMarker> markers;
    for(int i=0;i<2;++i) {
        const int layer=layers[static_cast<size_t>(i)];
        const auto row=lines[i+1].split('|');
        if(row.size()!=9 || row[0]!="STITCH_PAD" ||
           row[1]!=QByteArray::number(layer) ||
           row[2]!=QByteArray::number(stitchIndex) ||
           row[3]!=sourceIds[static_cast<size_t>(i)] ||
           layer<stitch.firstLayer || layer>stitch.lastLayer)
            return reject(__LINE__);
        qint64 x=0,y=0,diameter=0;
        if(!c015::number(row[4],x) || !c015::number(row[5],y) ||
           !c015::number(row[6],diameter) || diameter<=0 ||
           x*254!=stitch.x_nm || y*254!=stitch.y_nm ||
           diameter*254!=stitch.diameter_nm)return reject(__LINE__);
        ImportedCopperArea area;
        if(panel->importedReferenceAreaCount(layer,referenceNet)!=1 ||
           !panel->importedCopperGeometry(QString::fromUtf8(sourceIds[static_cast<size_t>(i)]),
             layer,referenceNet,area) || area.circle || area.curved ||
           area.ring.size()!=5)return reject(__LINE__);
        qint64 xmin=area.ring[0].first,ymin=area.ring[0].second;
        qint64 xmax=xmin,ymax=ymin;
        for(const auto &p:area.ring) {
            xmin=std::min(xmin,p.first);xmax=std::max(xmax,p.first);
            ymin=std::min(ymin,p.second);ymax=std::max(ymax,p.second);
        }
        if(!c013::rectangle(area.ring,xmin,ymin,xmax,ymax) ||
           xmin%254 || ymin%254 || xmax%254 || ymax%254)return reject(__LINE__);
        const std::array<qint64,4> outer{xmin/254,ymin/254,xmax/254,ymax/254};
        const auto &holes=sourceHoles[static_cast<size_t>(i)];
        if(!c015::stitchCentreInCopper(x,y,outer,holes))return reject(__LINE__);
        int chosen=-1;
        const QString state=classify(x,y,diameter,outer,holes,chosen);
        if(state.isEmpty() || row[7]!=state.toLatin1() ||
           row[8]!=(chosen<0?QByteArray("-"):QByteArray::number(chosen)))return reject(__LINE__);
        if(rank(state)>rank(overall))overall=state;
        ImportedReferenceRiskMarker m;
        m.state="STITCH_PAD_FOOTPRINT_RISK";
        m.reason=QString("C_016 L%1 / %2 / Original Source Area %3 / "
           "STITCH PAD GEOMETRY ONLY / P009 UNSUPPORTED / SI NOT_RUN")
           .arg(layer).arg(state).arg(QString::fromUtf8(sourceIds[static_cast<size_t>(i)]));
        m.net=referenceNet;m.viaObject=stitch.name;
        m.sourceObject=QString::fromUtf8(sourceIds[static_cast<size_t>(i)]);
        m.layer=layer;m.x_nm=stitch.x_nm;m.y_nm=stitch.y_nm;
        m.diameter_nm=stitch.diameter_nm;
        markers.push_back(std::move(m));
    }
    if(h[2]!=overall.toLatin1())return reject(__LINE__);
    original.markers.insert(original.markers.end(),markers.begin(),markers.end());
    original.state=overall;
    original.reason="C_016 STITCH PAD SOURCE ONLY / V11/V8 ORIGINAL ANCESTRY / P009 UNSUPPORTED / SI NOT_RUN";
    out=std::move(original);
    return true;
}
} // namespace c016
