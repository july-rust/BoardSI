// D_006 opt-in native Qt independent Via Pad/COPCLS readback.
// Source geometry warning only: no physical, return-path or solver acceptance.
#pragma once
#include "d003_three_area_qt.h"
#include <QCryptographicHash>
#include <QEventLoop>
#include <QFileInfo>
#include <QProcess>
#include <QTemporaryFile>
#include <QTimer>
#include <array>

namespace d006 {
using Rect = std::array<qint64, 4>;
inline bool intersects(const std::array<qint64, 2> &p, qint64 d, const Rect &r) {
    if(d<=0) return false;
    const qint64 dx = p[0]<r[0] ? r[0]-p[0] : (p[0]>r[2] ? p[0]-r[2] : 0);
    const qint64 dy = p[1]<r[1] ? r[1]-p[1] : (p[1]>r[3] ? p[1]-r[3] : 0);
    return 4*(dx*dx+dy*dy)<=d*d;
}
inline bool inside(const std::array<qint64, 2> &p, qint64 d, const Rect &r) {
    return d>0 && 2*p[0]>2*r[0]+d && 2*p[0]<2*r[2]-d
               && 2*p[1]>2*r[1]+d && 2*p[1]<2*r[3]-d;
}
// Independently audit PADS *VIA* padstack source. A Via span alone never
// proves a conductive Pad on an interior layer. These are the explicit
// round two/three-row PADS V9.4 definitions admitted by the Rust importer.
inline bool declaredConductivePad(const QByteArray &source, const ImportedVia &via, int layer) {
    if(source.isEmpty() || source.size()>8*1024*1024 ||
       via.name.isEmpty() || layer<via.firstLayer || layer>via.lastLayer)return false;
    auto decimal=[](const QString &s,qint64 &ticks) {
        static const QRegularExpression pattern("^[0-9]{1,7}(\\.[0-9]{1,2})?$");
        if(!pattern.match(s).hasMatch())return false;
        const auto parts=s.split('.');
        bool ok=false;const qint64 whole=parts[0].toLongLong(&ok);
        if(!ok)return false;
        qint64 fraction=0;
        if(parts.size()==2) {
            const QString part=parts[1].leftJustified(2,'0');
            fraction=part.toLongLong(&ok);
            if(!ok)return false;
        }
        ticks=whole*100+fraction;
        return ticks>0 && ticks<=5600000;
    };
    QString normalized=QString::fromUtf8(source);
    normalized.replace("\r\n","\n");
    const QStringList lines=normalized.split('\n');
    const QRegularExpression whitespace("\\s+");
    bool inVia=false,found=false,selected=false;
    for(int i=0;i<lines.size();++i) {
        const auto fields=lines[i].trimmed().split(whitespace,Qt::SkipEmptyParts);
        if(fields.isEmpty() || fields[0]=="*REMARK*" || fields[0]=="REMARK")continue;
        if(fields[0]=="*END*")break;
        if(fields[0].startsWith('*')) {
            inVia=fields[0]=="*VIA*";
            continue;
        }
        if(!inVia || fields[0]!=via.name)continue;
        if(found || fields.size()!=3)return false;
        found=true;
        bool valid=false;
        const int count=fields[2].toInt(&valid);
        if(!valid || count<2 || count>3 || i+count>=lines.size())return false;
        qint64 drill=0;
        if(!decimal(fields[1],drill) || drill*254!=via.drill_nm)return false;
        bool top=false,bottom=false,inner=false;
        qint64 common=0;
        for(int n=1;n<=count;++n) {
            const auto row=lines[i+n].trimmed().split(whitespace,Qt::SkipEmptyParts);
            if(row.size()!=3 || row[2]!="R")return false;
            qint64 width=0;
            if(!decimal(row[1],width) || width<=drill || width*254!=via.diameter_nm)
                return false;
            if(common!=0 && common!=width)return false;
            common=width;
            if(row[0]=="-2" && !top) {
                top=true;
                if(layer==via.firstLayer)selected=true;
            } else if(row[0]=="0" && !bottom) {
                bottom=true;
                if(layer==via.lastLayer)selected=true;
            } else if(row[0]=="-1" && !inner) {
                inner=true;
                if(layer>via.firstLayer && layer<via.lastLayer)selected=true;
            } else return false;
        }
        if(!top || !bottom || (count==3 && !inner))return false;
        i+=count;
    }
    return found && selected;
}
inline bool decode(const QByteArray &wire,const QByteArray &sha,const QByteArray &source,
                   const ImportedArtworkPanel *panel,const QStringList &names,
                   const QString &net,int layer,c005::Receipt &out) {
    if(!panel || names.size()!=3 || sha.size()!=64 || source.isEmpty()
       || source.size()>8*1024*1024 || wire.size()>16384 || !wire.endsWith('\n')
       || QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex()!=sha
       || panel->importedSkippedCount()!=0 || panel->viaCount()!=1
       || panel->arcCount()!=0 || panel->copperCount()!=3 || panel->pourEdgeCount()!=0
       || panel->importedReferenceAreaCount(layer,net)!=3)return false;
    const auto lines=wire.split('\n');
    if(lines.size()!=14 || !lines.last().isEmpty()
       || lines[12]!="END|SOURCE_ONLY_NOT_ELECTRICAL")return false;
    const auto h=lines[0].split('|');
    if(h.size()!=8 || h[0]!="BOARDSI_D006_VIA_THREE_AREA_V1"
       || h[1]!=sha || h[2]!="SOURCE_GEOMETRY_ONLY"
       || h[3]!="PHYSICAL_NOT_VERIFIED" || h[4]!="P009_UNSUPPORTED"
       || h[5]!="SI_NOT_RUN" || h[6]!="0" || h[7]!="0")return false;
    QByteArray base;
    for(int i=1;i<=9;++i) {base+=lines[i];base+='\n';}
    c005::Receipt independent;
    if(!d003::decode(base,sha,panel,names,net,layer,independent))return false;
    const auto v=lines[10].split('|');
    if(v.size()!=10 || v[0]!="VIA" || v[1]!="0"
       || v[2]!=QByteArray::number(layer))return false;
    QByteArray decodedNet;
    if(!c005::boundedHex(v[3],decodedNet) || QString::fromUtf8(decodedNet)!=net)return false;
    std::array<qint64,6> values{};
    for(int i=0;i<6;++i) {
        bool ok=false;
        values[static_cast<size_t>(i)]=v[i+4].toLongLong(&ok);
        if(!ok)return false;
    }
    const std::array<qint64,2> point={values[0],values[1]};
    const qint64 diameter=values[2];
    ImportedVia imported;
    if(!panel->importedVia(0,imported) || !declaredConductivePad(source,imported,layer) || imported.net!=net
       || imported.x_nm!=point[0]*254 || imported.y_nm!=point[1]*254
       || imported.diameter_nm!=diameter*254 || imported.drill_nm!=values[3]*254
       || imported.firstLayer!=values[4] || imported.lastLayer!=values[5]
       || diameter<=values[3] || values[3]<=0
       || layer<values[4] || layer>values[5])return false;
    std::array<Rect,3> boxes{};
    std::array<Rect,3> holes{};
    std::array<bool,3> hasHole={false,false,false};
    std::array<int,3> groups{};
    for(int i=0;i<3;++i) {
        const auto row=lines[i+2].split('|');
        if(row.size()!=11 || row[0]!="AREA")return false;
        for(int j=0;j<4;++j) {
            bool ok=false;
            boxes[static_cast<size_t>(i)][static_cast<size_t>(j)]
                 =row[j+5].toLongLong(&ok);
            if(!ok)return false;
        }
        bool ok=false;groups[static_cast<size_t>(i)]=row[10].toInt(&ok);
        if(!ok || groups[static_cast<size_t>(i)]<0
               || groups[static_cast<size_t>(i)]>=3)return false;
    }
    const auto hole=lines[5].split('|');
    if(hole.size()==6 && hole[0]=="HOLE") {
        bool ok=false;const int owner=hole[1].toInt(&ok);
        if(!ok || owner<0 || owner>=3)return false;
        hasHole[static_cast<size_t>(owner)]=true;
        for(int i=0;i<4;++i) {
            bool valid=false;
            holes[static_cast<size_t>(owner)][static_cast<size_t>(i)]
                =hole[i+2].toLongLong(&valid);
            if(!valid)return false;
        }
    } else if(hole!=QList<QByteArray>{"HOLE","NONE"})return false;
    int touched=0,retained=-1;bool blocked=false;
    for(int i=0;i<3;++i) {
        const auto &box=boxes[static_cast<size_t>(i)];
        if(!intersects(point,diameter,box))continue;
        ++touched;
        if(hasHole[static_cast<size_t>(i)]
           && intersects(point,diameter,holes[static_cast<size_t>(i)]))blocked=true;
        else if(inside(point,diameter,box))retained=i;
    }
    const QByteArray state=blocked?"SOURCE_COPCUT_RISK":
        touched>1?"SOURCE_AMBIGUOUS_AREA_OWNER":
        touched==1 && retained>=0?"SOURCE_VIA_PAD_AREA_CANDIDATE":
        touched==0?"SOURCE_SEPARATION_RISK":"SOURCE_PAD_BOUNDARY_RISK";
    const auto contact=lines[11].split('|');
    if(contact.size()!=4 || contact[0]!="CONTACT" || contact[1]!=state)return false;
    const bool positive=state=="SOURCE_VIA_PAD_AREA_CANDIDATE";
    if(contact[2]!=(positive?QByteArray::number(retained):QByteArray("-"))
       || contact[3]!=(positive?QByteArray::number(groups[static_cast<size_t>(retained)])
                              :QByteArray("-")))return false;
    c005::Receipt result;
    result.state=QString::fromLatin1(state);
    result.reason="D_006 VIA PAD/COPCLS SOURCE GEOMETRY / NOT ELECTRICAL / P009 UNSUPPORTED";
    if(positive) {
        ImportedReferenceRiskMarker marker;
        marker.state="CORRIDOR_CANDIDATE_RISK";
        marker.reason="D_006 SOURCE VIA PAD AREA CANDIDATE / NOT ELECTRICAL";
        marker.net=net;marker.layer=layer;
        marker.sourceObject="D006-source-via-pad";marker.viaObject=imported.name;
        marker.x_nm=imported.x_nm;marker.y_nm=imported.y_nm;
        marker.diameter_nm=imported.diameter_nm;
        result.markers.push_back(marker);
    }
    out=std::move(result);
    return true;
}
inline int selftest(const QString &importer) {
    const QFileInfo importedBinary(importer);
    const QString cli=importedBinary.absolutePath()+"/boardsi-d006-via-three-area"
                       +(importedBinary.suffix()=="exe"?".exe":"");
    if(!QFileInfo(cli).isExecutable())return 2;
    QTemporaryFile file;
    if(!file.open())return 2;
    const QByteArray source(
        "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n"
        "*PCB*\nMAXLAYER 4\n*VIA*\nSIMPLEVIA 15 2\n-2 35 R\n0 35 R\n"
        "*LINES*\n"
        "RA COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n"
        "100 200\n200 200\n200 300\n100 300\n100 200\n"
        "RB COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n"
        "190 220\n290 220\n290 320\n190 320\n190 220\n"
        "RC COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n"
        "280 240\n380 240\n380 340\n280 340\n280 240\n"
        "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U2.2\n"
        "100 250 1 10 0\n150 250 1 10 0 SIMPLEVIA\n200 250 1 10 0\n*END*\n");
    const auto save=[&](const QByteArray &bytes) {
        return file.resize(0) && file.seek(0) &&
               file.write(bytes)==bytes.size() && file.flush();
    };
    if(!save(source))return 2;
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    const auto load=[&]() {
        panel.setStatusCallback([&](const QString &result) {
            if(result.startsWith("PARTIAL") || result.startsWith("FAILED"))loop.quit();
        });
        if(!panel.openPath(file.fileName()))return false;
        QTimer deadline; deadline.setSingleShot(true);
        QObject::connect(&deadline,&QTimer::timeout,&loop,&QEventLoop::quit);
        deadline.start(8000);loop.exec();deadline.stop();
        return panel.importStatus().startsWith("PARTIAL");
    };
    const auto query=[&]() {
        QProcess p;
        p.start(cli,{file.fileName(),"RA","RB","RC","NET_A","1","0"});
        if(!p.waitForFinished(10000) || p.exitStatus()!=QProcess::NormalExit
           || p.exitCode()!=0)return QByteArray();
        return p.readAllStandardOutput();
    };
    if(!load())return 2;
    const auto digest=QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex();
    c005::Receipt first;
    const auto wire=query();
    const bool accepted=panel.acceptedSourceSha256()==digest &&
        decode(wire,digest,source,panel.imported(),{"RA","RB","RC"},"NET_A",1,first) &&
        first.markers.size()==1;
    if(!accepted)return 1;
    ImportedVia observed;
    const bool padLayerGuard=panel.imported()->importedVia(0,observed) &&
        declaredConductivePad(source,observed,1) &&
        !declaredConductivePad(source,observed,2);
    panel.imported()->setReferenceRisks(first.markers);
    const bool displayed=panel.imported()->referenceRiskCountForTest()==1;
    auto forged=wire;
    forged.replace("CONTACT|SOURCE_VIA_PAD_AREA_CANDIDATE|0|0",
                   "CONTACT|SOURCE_SEPARATION_RISK|-|-");
    c005::Receipt out;
    const bool noForgery=!decode(forged,digest,source,panel.imported(),
                                   {"RA","RB","RC"},"NET_A",1,out);
    auto changed=source;
    changed.replace("150 250 1 10 0 SIMPLEVIA","110 250 1 10 0 SIMPLEVIA");
    if(!save(changed) || !load())return 2;
    const auto nextSha=QCryptographicHash::hash(changed,QCryptographicHash::Sha256).toHex();
    c005::Receipt next;
    const bool stale=nextSha!=digest && panel.acceptedSourceSha256()==nextSha &&
        panel.imported()->referenceRiskCountForTest()==0 &&
        !decode(wire,nextSha,changed,panel.imported(),{"RA","RB","RC"},"NET_A",1,out);
    const bool boundary=decode(query(),nextSha,changed,panel.imported(),
                          {"RA","RB","RC"},"NET_A",1,next) && next.markers.empty()
                          && next.state=="SOURCE_PAD_BOUNDARY_RISK";
    const bool ok=accepted && padLayerGuard && displayed && noForgery && stale && boundary;
    qInfo()<<"D006_NATIVE_QT_VIA_THREE_AREA"<<ok<<accepted<<displayed
           <<noForgery<<stale<<boundary<<padLayerGuard<<"PHYSICAL_NOT_VERIFIED_P009_UNSUPPORTED";
    return ok?0:1;
}
} // namespace d006
