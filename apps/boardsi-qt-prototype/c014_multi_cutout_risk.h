// C_014: independent Qt receipt/source-copper cross-check, Source geometry ONLY.
// Included after c012/c013; never promotes a geometric risk to physical DRC.
#pragma once
namespace c014 {
inline bool decode(const QByteArray &wire,const QByteArray &sha,
  const ImportedArtworkPanel *panel,const std::vector<int> &selected,
  int split,int stitchIndex,const QString &referenceNet,
  const std::array<int,2> &layers,c005::Receipt &out) {
    if(!panel || wire.isEmpty() || wire.size()>270000 || !wire.endsWith('\n'))
        return false;
    const auto lines=wire.split('\n');
    if(lines.size()<24 || !lines.last().isEmpty())return false;
    const auto h=lines[0].split('|');
    if(h.size()!=13 || h[0]!="BOARDSI_REFERENCE_ROUTE_V10" ||
       sha.size()!=64 || h[1]!=sha || h[3]!="UNSUPPORTED" ||
       h[4]!="NOT_RUN" || h[5]!="0" ||
       h[6]!="REFERENCE_TRANSFER_UNVERIFIED" || h[7]!="2" ||
       h[8]!=QByteArray::number(static_cast<int>(selected.size())) ||
       h[9]!=QByteArray::number(split) ||
       h[10]!=QByteArray::number(stitchIndex) ||
       (h[12]!="V8" && h[12]!="V9"))return false;
    bool ok=false;
    const int signalIndex=h[11].toInt(&ok);
    if(!ok || signalIndex<0 || signalIndex==stitchIndex)return false;
    ImportedVia signal;
    if(!panel->importedVia(signalIndex,signal) || signal.net.isEmpty() ||
       signal.net==referenceNet || signal.diameter_nm<=signal.drill_nm)return false;
    int pos=1;
    QString overall="SOURCE_CUTOUT_GAP_OBSERVED";
    bool allSingle=true;
    std::vector<ImportedReferenceRiskMarker> markers;
    for(int i=0;i<2;++i) {
        const int layer=layers[static_cast<size_t>(i)];
        if(pos>=lines.size()-2)return false;
        const auto row=lines[pos++].split('|');
        if(row.size()!=10 || row[0]!="LAYER" ||
           row[1]!=QByteArray::number(layer) ||
           row[2]!=QByteArray::number(signalIndex))return false;
        QByteArray sourceId;
        if(!c005::boundedHex(row[3],sourceId))return false;
        const qint64 x=row[4].toLongLong(&ok);
        if(!ok || x<-5600000 || x>5600000 || x*254!=signal.x_nm)return false;
        const qint64 y=row[5].toLongLong(&ok);
        if(!ok || y<-5600000 || y>5600000 || y*254!=signal.y_nm)return false;
        const qint64 diameter=row[6].toLongLong(&ok);
        if(!ok || diameter<=0 || diameter>5600000 ||
           diameter*254!=signal.diameter_nm)return false;
        const int count=row[7].toInt(&ok);
        if(!ok || count<0 || count>8)return false;
        if(count>1)allSingle=false;
        ImportedCopperArea area;
        if(panel->importedReferenceAreaCount(layer,referenceNet)!=1 ||
           !panel->importedCopperGeometry(QString::fromUtf8(sourceId),
               layer,referenceNet,area) ||
           area.circle || area.curved || !area.roundCutouts.empty() ||
           static_cast<int>(area.cutouts.size())!=count)return false;
        if(area.ring.size()!=5)return false;
        qint64 xmin=area.ring[0].first,ymin=area.ring[0].second;
        qint64 xmax=xmin,ymax=ymin;
        for(const auto &p:area.ring){
            xmin=std::min(xmin,p.first); ymin=std::min(ymin,p.second);
            xmax=std::max(xmax,p.first); ymax=std::max(ymax,p.second);
        }
        if(!c013::rectangle(area.ring,xmin,ymin,xmax,ymax) ||
           2*(signal.x_nm-xmin)<=signal.diameter_nm ||
           2*(xmax-signal.x_nm)<=signal.diameter_nm ||
           2*(signal.y_nm-ymin)<=signal.diameter_nm ||
           2*(ymax-signal.y_nm)<=signal.diameter_nm)return false;
        std::vector<std::array<qint64,4>> holes;
        for(int j=0;j<count;++j){
            if(pos>=lines.size()-2)return false;
            const auto r=lines[pos++].split('|');
            if(r.size()!=7 || r[0]!="HOLE" ||
               r[1]!=QByteArray::number(layer) ||
               r[2]!=QByteArray::number(j))return false;
            std::array<qint64,4> b{};
            for(int k=0;k<4;++k){
                b[static_cast<size_t>(k)]=r[3+k].toLongLong(&ok);
                if(!ok || b[static_cast<size_t>(k)] < -5600000 ||
                   b[static_cast<size_t>(k)] > 5600000)return false;
                b[static_cast<size_t>(k)]*=254;
            }
            if(!c013::rectangle(area.cutouts[static_cast<size_t>(j)],
                              b[0],b[1],b[2],b[3]) ||
               b[0]<=xmin || b[1]<=ymin || b[2]>=xmax || b[3]>=ymax)
                return false;
            for(const auto &prior:holes)
                if(!(b[2]<prior[0] || prior[2]<b[0] ||
                     b[3]<prior[1] || prior[3]<b[1]))return false;
            holes.push_back(b);
        }
        QString state=count==0?"SOURCE_CUTOUT_MISSING_RISK":
                             "SOURCE_REFERENCE_COPPER_OVERLAP_RISK";
        int chosen=-1;
        qreal gapMm=0;
        for(int j=0;j<count;++j){
            const auto &b=holes[static_cast<size_t>(j)];
            const qint64 distances[4]={signal.x_nm-b[0],b[2]-signal.x_nm,
                                       signal.y_nm-b[1],b[3]-signal.y_nm};
            const qint64 minMargin=*std::min_element(std::begin(distances),
                                                      std::end(distances));
            if(minMargin>=0){
                chosen=j;
                state=2*minMargin>signal.diameter_nm?"SOURCE_CUTOUT_GAP_OBSERVED":
                    2*minMargin==signal.diameter_nm?"SOURCE_CUTOUT_TANGENCY_RISK":
                    "SOURCE_REFERENCE_COPPER_OVERLAP_RISK";
                gapMm=(static_cast<qreal>(minMargin)-
                    static_cast<qreal>(signal.diameter_nm)/2.)/1000000.;
                break;
            }
        }
        if(row[8]!=state.toLatin1() ||
           row[9]!=(chosen<0?"-":QByteArray::number(chosen)) ||
           c013::severity(state)<0)return false;
        if(c013::severity(state)>c013::severity(overall))overall=state;
        const QString detail=QString("L%1 SignalVia=%2 / %3 / SourceArea=%4 / COPCUT count=%5 / chosen=%6 / Pad=%7 mm / approximate gap=%8 mm / %9")
            .arg(layer).arg(signalIndex).arg(signal.net)
            .arg(QString::fromUtf8(sourceId)).arg(count).arg(chosen)
            .arg(signal.diameter_nm/1e6,0,'f',6).arg(gapMm,0,'f',6).arg(state);
        ImportedReferenceRiskMarker m;
        m.state="SIGNAL_MULTI_CUTOUT_CLEARANCE_RISK";
        m.reason="C_014 "+detail+
                 " / SOURCE GEOMETRY ONLY / P009 UNSUPPORTED / SI NOT_RUN";
        m.net=signal.net;m.viaObject=signal.name;m.sourceObject=detail;
        m.layer=layer;m.x_nm=signal.x_nm;m.y_nm=signal.y_nm;
        m.diameter_nm=signal.diameter_nm;
        markers.push_back(std::move(m));
    }
    if(h[2]!=overall.toLatin1() ||
       (allSingle && h[12]!="V9") || (!allSingle && h[12]!="V8"))
        return false;
    if(pos>=lines.size()-2)return false;
    const auto legacyRow=lines[pos++].split('|');
    if(legacyRow.size()!=3 || legacyRow[0]!="LEGACY" ||
       legacyRow[1]!=h[12])return false;
    const int count=legacyRow[2].toInt(&ok);
    if(!ok || count<17 || count>900 ||
       pos+count!=lines.size()-2 ||
       lines[pos+count]!="END|2|SOURCE_GEOMETRY_ONLY_NOT_PHYSICAL")
        return false;
    QByteArray legacy;
    for(int i=0;i<count;++i){legacy+=lines[pos+i];legacy+='\n';}
    c005::Receipt receipt;
    const bool valid=(h[12]=="V9")?
        c013::decode(legacy,sha,panel,selected,split,stitchIndex,referenceNet,layers,receipt):
        c012::decode(legacy,sha,panel,selected,split,stitchIndex,referenceNet,layers,receipt);
    if(!valid)return false;
    receipt.markers.insert(receipt.markers.end(),markers.begin(),markers.end());
    receipt.state=overall;
    receipt.reason="C_014 MULTI-COPCUT SOURCE ONLY / P009 UNSUPPORTED / SI NOT_RUN";
    out=std::move(receipt);
    return true;
}
} // namespace c014
