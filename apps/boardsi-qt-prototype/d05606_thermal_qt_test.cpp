// D_056_06: actual main Canonical source RT/ST copper and relief holes.
// Native Qt geometry-only oracle, not electrical continuity or GPU pixel proof.
#include "d05605_canonical_artwork.h"
#include <QApplication>
#include <QCryptographicHash>
#include <QDebug>
#include <QPolygonF>
#include <cmath>

static QString id(const QByteArray &text) {
    QByteArray source=text;source.append('\0');
    return QString::fromLatin1(source.toHex());
}
static QString ticks(int v) {
    return QString::number(static_cast<qint64>(v)*127)+":500000000";
}
static d056ipc::Source source(int layers,int first,int last) {
    d056ipc::Source s;
    s.sha=QString(64,QLatin1Char('a'));
    s.format="PADS_ASC";s.version="V9.4-MILS";
    const auto frame=id("xy"),stack=id("thermal"),drill=id("drill");
    s.frames.push_back(d056ipc::Frame{frame,"PADS 9.4 MILS",
        "127:500000000","0:1","0:1",0,false,false});
    for(int l=1;l<=layers;++l) {
        s.layerIds.push_back(id("layer-"+QByteArray::number(l)));
        s.layerOrders.push_back(l-1);
    }
    d056ipc::Stack pads;pads.id=stack;
    d056ipc::Pad rt{first,"CONDUCTIVE","THERMAL_RT",
        ticks(6000),ticks(8400)};
    rt.spokeWidth=ticks(1500);rt.spokeAngle=45;rt.spokeCount=4;
    pads.pads.push_back(rt);
    if (last>first+1) {
        d056ipc::Pad st{first+1,"CONDUCTIVE","THERMAL_ST",
            ticks(5000),ticks(7500)};
        st.spokeWidth=ticks(1000);st.spokeAngle=0;st.spokeCount=4;
        pads.pads.push_back(st);
    }
    pads.pads.push_back(d056ipc::Pad{last,"CONDUCTIVE","CIRCLE",
        ticks(5000),"0:1"});
    s.stacks.push_back(pads);
    s.drills.push_back(d056ipc::Drill{drill,ticks(1500),first,last,true});
    s.vias.push_back(d056ipc::Via{id("via"),stack,drill,"-",frame,
        first==1&&last==layers?"THROUGH":
        first>1&&last<layers?"BURIED":"BLIND",
        0,0,first,last});
    return s;
}
static bool selftest() {
    for(const auto &caseData:{
        std::array<int,3>{2,1,2}, {4,1,4}, {6,2,4},
        {8,2,3}, {64,17,41}
    }) {
        const auto data=source(caseData[0],caseData[1],caseData[2]);
        d05605::Model model;
        if(!d05605::make(data,data.sha,model)) return false;
        const int expected=caseData[2]>caseData[1]+1?3:2;
        if(model.glyphs.size()!=expected || model.layerCount!=caseData[0])
            return false;
        const auto &rt=model.glyphs.at(0);
        if(!rt.conductive || rt.shape!="THERMAL_RT" ||
            rt.layer!=caseData[1] || rt.footprint.contains(QPointF(0,0)) ||
            !rt.footprint.contains(QPointF(0.65,-0.65)) ||
            rt.footprint.contains(QPointF(0.91,0)) ||
            rt.footprint.contains(QPointF(1.2,0)) ||
            !rt.details.contains("PLANE FILL UNKNOWN")) return false;
        if(expected==3) {
            const auto &st=model.glyphs.at(1);
            if(!st.conductive || st.shape!="THERMAL_ST" ||
                st.layer!=caseData[1]+1 || st.footprint.contains(QPointF(0,0)) ||
                !st.footprint.contains(QPointF(0.8,0)) ||
                st.footprint.contains(QPointF(0.8,0.8))) return false;
        }
        d05605::Model rejected;
        if(d05605::make(data,QString(64,QLatin1Char('b')),rejected)
            || !rejected.glyphs.empty())return false;
        auto broken=data;
        broken.stacks[0].pads[0].spokeCount=0;
        if(d05605::make(broken,broken.sha,rejected))return false;
    }
    return true;
}
int main(int argc,char **argv) {
    QApplication app(argc,argv);
    if(argc!=2 || QString::fromLatin1(argv[1])!="--selftest")return 2;
    if(!selftest()) {
        qCritical("D_056_06 main Qt RT/ST positive/void geometry FAILED");
        return 1;
    }
    return 0;
}
