// D_005: Qt independent imported SMD Pad/Trace/COPCLS geometry readback.
// This opt-in selftest never proves electrical contact, P009, or SI solver readiness.
#pragma once
#include "d004_endpoint_attachment_qt.h"
#include <QTemporaryFile>
#include <QProcess>
#include <QCryptographicHash>
#include <QEventLoop>
#include <QTimer>
#include <QRegularExpression>
#include <QHash>
#include <array>
namespace d005 {
// Independently inspect the exact source bytes for the one supported
// PADS ROUTE SIGNAL pin-pair declaration; Pad geometry alone is insufficient.
inline bool declaredPins(const QByteArray &source,const QByteArray &sha,const QString &net,
    QStringList &pins) {
    if(source.isEmpty() || source.size()>16*1024*1024 ||
       QCryptographicHash::hash(source,QCryptographicHash::Sha256).toHex()!=sha)return false;
    QString normalized=QString::fromUtf8(source);
    normalized.replace("\r\n","\n");
    const auto lines=normalized.split('\n');
    const QRegularExpression whitespace("\\s+");
    bool inRoute=false;
    QString currentNet;
    QHash<QString,QString> memberships;
    // Parity with Rust part_preview::declared_route_pin_nets:
    // *REMARK*/REMARK never end a signal; *SIGNAL* Net is the second field;
    // multiple pairs and multiple SIGNAL blocks can declare Pin membership.
    for(const auto &raw:lines) {
        const QString line=raw.trimmed();
        const QStringList fields=line.split(whitespace,Qt::SkipEmptyParts);
        if(fields.isEmpty() || fields[0]=="*REMARK*" || fields[0]=="REMARK")continue;
        const QString first=fields[0];
        if(first=="*END*")break;
        if(first.startsWith('*')) {
            if(first=="*ROUTE*") {
                inRoute=true;
                currentNet.clear();
            } else if(first=="*SIGNAL*" && inRoute) {
                currentNet=fields.value(1);
            } else {
                inRoute=false;
                currentNet.clear();
            }
            continue;
        }
        if(!inRoute || currentNet.isEmpty() || fields.size()!=2)continue;
        const int a=fields[0].lastIndexOf('.'),b=fields[1].lastIndexOf('.');
        if(a<=0 || a==fields[0].size()-1 || b<=0 || b==fields[1].size()-1)continue;
        for(const auto &token:fields) {
            auto existing=memberships.constFind(token);
            if(existing!=memberships.cend() && existing.value()!=currentNet)return false;
            memberships.insert(token,currentNet);
        }
    }
    for(auto it=memberships.cbegin();it!=memberships.cend();++it) {
        if(it.value()==net)pins.push_back(it.key());
        if(pins.size()>128)return false; // D_005 bounded diagnostic
    }
    return !pins.isEmpty();
}
inline bool padContains(const std::array<qint64,2> &p,const ImportedPad &pad) {
    if(pad.width_nm<=0 || pad.height_nm<=0 ||
       pad.x_nm%254 || pad.y_nm%254 || pad.width_nm%254 || pad.height_nm%254) return false;
    const qint64 dx=p[0]-pad.x_nm/254,dy=p[1]-pad.y_nm/254;
    const qint64 w=pad.width_nm/254,h=pad.height_nm/254;
    // Explicit bound prevents integer overflow on malformed external geometry.
    constexpr qint64 limit=100000000;
    if(dx<=-limit || dx>=limit || dy<=-limit || dy>=limit ||
       w>=limit || h>=limit)return false;
    if(pad.shape=="S" || pad.shape=="RF")return 2*(dx<0?-dx:dx)<w && 2*(dy<0?-dy:dy)<h;
    if(pad.shape=="R" && w==h)return 4*(dx*dx+dy*dy)<w*w;
    return false;
}
inline bool decode(const QByteArray &wire,const QByteArray &sha,const QByteArray &source,
    const ImportedArtworkPanel *artwork,const QStringList &names,
    const QString &net,int layer,c005::Receipt &out) {
    if(!artwork || sha.size()!=64 || wire.size()>16384 || !wire.endsWith('\n') ||
       artwork->importedSkippedCount()!=0 || artwork->importedTraceCount()!=1 ||
       artwork->arcCount()!=0 || artwork->viaCount()!=0 || artwork->padCount()>5000)return false;
    QStringList declared;
    if(!declaredPins(source,sha,net,declared))return false;
    const auto lines=wire.split('\n');
    if(lines.size()!=18 || !lines.last().isEmpty() ||
       lines[16]!="END|SOURCE_ONLY_NOT_PHYSICAL")return false;
    const auto header=lines[0].split('|');
    if(header.size()!=8 || header[0]!="BOARDSI_D005_PAD_TRACE_AREA_V1" ||
       header[1]!=sha || header[2]!="SOURCE_CANDIDATE_ONLY" ||
       header[3]!="PHYSICAL_NOT_VERIFIED" || header[4]!="P009_UNSUPPORTED" ||
       header[5]!="SI_NOT_RUN" || header[6]!="0" || header[7]!="0")return false;
    QByteArray previous;
    for(int i=1;i<=14;++i) {previous+=lines[i];previous+='\n';}
    c005::Receipt sourceValidated;
    if(!d004::decode(previous,sha,artwork,names,net,layer,sourceValidated))return false;
    ImportedTrack t;
    if(!artwork->importedTrack(0,t) || t.net!=net || t.layer!=layer ||
       t.x0_nm%254 || t.y0_nm%254 || t.x1_nm%254 || t.y1_nm%254)return false;
    const std::array<std::array<qint64,2>,2> points={
        std::array<qint64,2>{t.x0_nm/254,t.y0_nm/254},
        std::array<qint64,2>{t.x1_nm/254,t.y1_nm/254}
    };
    std::array<int,2> owners={-1,-1};
    for(int e=0;e<2;++e) {
        int count=0;
        for(int p=0;p<artwork->padCount();++p) {
            ImportedPad pad;
            if(!artwork->importedPad(p,pad))return false;
            if(pad.layer==layer && padContains(points[static_cast<size_t>(e)],pad)) {
                if(++count>1)return false; // ambiguous geometric owner: fail closed
                const QString sourcePin=pad.reference+"."+pad.pin;
                // PADS SIGNAL declared pins express net membership, not an
                // ordered association with the source Trace coordinate endpoints.
                if(declared.contains(sourcePin))
                    owners[static_cast<size_t>(e)]=p;
            }
        }
    }
    std::array<int,2> areaOwners={-1,-1};
    std::array<int,2> areaGroups={-1,-1};
    for(int e=0;e<2;++e) {
        const auto fields=lines[12+e].split('|');
        if(fields.size()!=5 || fields[0]!="POINT" ||
           fields[1]!=QByteArray::number(e))return false;
        if(fields[2]=="SOURCE_ENDPOINT_ATTACHMENT_CANDIDATE") {
            bool a=false,b=false;
            const int o=fields[3].toInt(&a),g=fields[4].toInt(&b);
            if(!a || !b || o<0 || o>=3 || g<0 || g>=3)return false;
            areaOwners[static_cast<size_t>(e)]=o;
            areaGroups[static_cast<size_t>(e)]=g;
        }
    }
    int found=-1;
    for(int e=0;e<2;++e) {
        if(owners[static_cast<size_t>(e)]>=0 &&
           areaOwners[static_cast<size_t>(1-e)]>=0) {
            if(found!=-1)return false;
            found=e;
        }
    }
    const auto link=lines[15].split('|');
    if(link.size()!=7 || link[0]!="LINK")return false;
    const bool positive=found>=0;
    if(link[1]!=(positive?"SOURCE_PAD_TRACE_AREA_CANDIDATE":"SOURCE_CONTACT_NOT_VERIFIED"))return false;
    const auto exact=[&](int i,int n)->bool {
        return link[i]==(n<0?QByteArray("-"):QByteArray::number(n));
    };
    if(!exact(2,found) ||
       !exact(3,positive?owners[static_cast<size_t>(found)]:-1) ||
       !exact(4,positive?1-found:-1) ||
       !exact(5,positive?areaOwners[static_cast<size_t>(1-found)]:-1) ||
       !exact(6,positive?areaGroups[static_cast<size_t>(1-found)]:-1))return false;
    c005::Receipt result;
    result.state="SOURCE_ATTACHMENT_ONLY";
    result.reason="D_005 PAD-TRACE-AREA SOURCE ONLY / PHYSICAL NOT_VERIFIED / P009 UNSUPPORTED";
    if(positive) {
        ImportedReferenceRiskMarker marker;
        marker.state="CORRIDOR_CANDIDATE_RISK";
        marker.reason="D_005 PAD-TRACE-AREA SOURCE CANDIDATE / NOT ELECTRICAL";
        marker.net=net;marker.layer=layer;marker.sourceObject="D005-pad-trace-area";
        marker.viaObject="-"; marker.x_nm=points[static_cast<size_t>(found)][0]*254;
        marker.y_nm=points[static_cast<size_t>(found)][1]*254;
        result.markers.push_back(marker);
    }
    out=std::move(result);
    return true;
}
inline int selftest(const QString &importer) {
    QTemporaryFile source;
    if(!source.open())return 2;
    const QByteArray original(
      "!PADS-POWERPCB-V9.4-MILS! DESIGN DATABASE ASCII FILE 1.0\n"
      "*PCB*\nMAXLAYER 4\n"
      "*PARTDECAL*\nD2 I 1000 1000 0 2 2 0 0\nT0 0 0 0 1\nT100 0 0 0 2\n"
      "PAD 1 3\n-2 40 R 0\n-1 0 R 0\n0 0 R 0\n"
      "PAD 2 3\n-2 30 S 0\n-1 0 R 0\n0 0 R 0\n"
      "*PARTTYPE*\nTYPEA D2 UND 0 0 0 0\n"
      "*PART*\nU1 TYPEA 200 300 90 U N 0 -1 0 -1 0\n"
      "*LINES*\n"
      "RA COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n"
      "240 240\n350 240\n350 360\n240 360\n240 240\n"
      "RB COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n"
      "340 320\n450 320\n450 420\n340 420\n340 320\n"
      "RC COPPER 0 0 1 1 0 NET_A\nCOPCLS 5 0.01 0 1\n"
      "440 390\n520 390\n520 480\n440 480\n440 390\n"
      "*ROUTE*\n*SIGNAL* NET_A\nU1.1 U9.2\n"
      "204 300 1 10 0\n260 310 1 10 0\n*END*\n");
    auto save=[&](const QByteArray &bytes) {
        return source.resize(0) && source.seek(0) &&
               source.write(bytes)==bytes.size() && source.flush();
    };
    if(!save(original))return 2;
    PreviewImportPanel panel(importer);
    QEventLoop loop;
    auto load=[&]() {
        panel.setStatusCallback([&](const QString &s) {
            if(s.startsWith("PARTIAL") || s.startsWith("FAILED"))loop.quit();
        });
        if(!panel.openPath(source.fileName()))return false;
        QTimer deadline;deadline.setSingleShot(true);
        QObject::connect(&deadline,&QTimer::timeout,&loop,&QEventLoop::quit);
        deadline.start(8000);loop.exec();deadline.stop();
        return panel.importStatus().startsWith("PARTIAL");
    };
    auto query=[&]() {
        QProcess p;
        p.start(importer,QStringList{source.fileName(),"--d005-pad-trace-area-v1",
                "RA","RB","RC","NET_A","1","0"});
        if(!p.waitForFinished(10000) || p.exitStatus()!=QProcess::NormalExit ||
           p.exitCode()!=0)return QByteArray();
        return p.readAllStandardOutput();
    };
    if(!load())return 2;
    const auto digest=QCryptographicHash::hash(original,QCryptographicHash::Sha256).toHex();
    const QByteArray wire=query();
    c005::Receipt first;
    const bool verified=panel.acceptedSourceSha256()==digest &&
        decode(wire,digest,original,panel.imported(),{"RA","RB","RC"},"NET_A",1,first) &&
        first.markers.size()==1;
    if(!verified)return 1;
    QByteArray forged=wire;
    forged.replace("LINK|SOURCE_PAD_TRACE_AREA_CANDIDATE|0|0|1|0|0",
                   "LINK|SOURCE_CONTACT_NOT_VERIFIED|-|-|-|-|-");
    c005::Receipt rejected;
    const bool cannotSpoof=!decode(forged,digest,original,panel.imported(),
                                    {"RA","RB","RC"},"NET_A",1,rejected);
    panel.imported()->setReferenceRisks(first.markers);
    const bool displayed=panel.imported()->referenceRiskCountForTest()==1;
    QByteArray changed=original;
    changed.replace("204 300 1 10 0","220 300 1 10 0");
    if(!save(changed) || !load())return 2;
    const auto fresh=QCryptographicHash::hash(changed,QCryptographicHash::Sha256).toHex();
    c005::Receipt second;
    const bool stale=fresh!=digest && panel.acceptedSourceSha256()==fresh &&
        panel.imported()->referenceRiskCountForTest()==0 &&
        !decode(wire,fresh,changed,panel.imported(),{"RA","RB","RC"},"NET_A",1,rejected);
    const bool negative=decode(query(),fresh,changed,panel.imported(),{"RA","RB","RC"},"NET_A",1,second) &&
        second.markers.empty();
    // Same geometry with a non-owning Source Pin must stay a negative witness.
    // A forged positive LINK against the same source SHA is rejected.
    QByteArray wrongPin=original;
    wrongPin.replace("U1.1 U9.2","U1.2 U9.2");
    if(!save(wrongPin) || !load())return 2;
    const auto wrongSha=QCryptographicHash::hash(wrongPin,QCryptographicHash::Sha256).toHex();
    const QByteArray negativeWire=query();
    c005::Receipt wrongReceipt;
    const bool wrongPinNegative=decode(negativeWire,wrongSha,wrongPin,panel.imported(),
         {"RA","RB","RC"},"NET_A",1,wrongReceipt) && wrongReceipt.markers.empty();
    QByteArray forgedPositive=negativeWire;
    forgedPositive.replace("LINK|SOURCE_CONTACT_NOT_VERIFIED|-|-|-|-|-",
                           "LINK|SOURCE_PAD_TRACE_AREA_CANDIDATE|0|0|1|0|0");
    const bool wrongPinCannotSpoof=!decode(forgedPositive,wrongSha,wrongPin,panel.imported(),
         {"RA","RB","RC"},"NET_A",1,rejected);
    // PADS pin declarations denote Net membership, not geometric endpoint order.
    QByteArray reversed=original;
    reversed.replace("204 300 1 10 0\n260 310 1 10 0",
                     "260 310 1 10 0\n204 300 1 10 0");
    if(!save(reversed) || !load())return 2;
    const auto reverseSha=QCryptographicHash::hash(reversed,QCryptographicHash::Sha256).toHex();
    const QByteArray reverseWire=query();
    c005::Receipt reverseReceipt;
    const bool reversedPositive=reverseWire.contains(
           "LINK|SOURCE_PAD_TRACE_AREA_CANDIDATE|1|0|0|0|0") &&
        decode(reverseWire,reverseSha,reversed,panel.imported(),
               {"RA","RB","RC"},"NET_A",1,reverseReceipt) &&
        reverseReceipt.markers.size()==1;
    QByteArray reverseForged=reverseWire;
    reverseForged.replace("LINK|SOURCE_PAD_TRACE_AREA_CANDIDATE|1|0|0|0|0",
                          "LINK|SOURCE_CONTACT_NOT_VERIFIED|-|-|-|-|-");
    const bool reverseForgeryRejected=!decode(reverseForged,reverseSha,reversed,panel.imported(),
           {"RA","RB","RC"},"NET_A",1,rejected);
    // A single SIGNAL can contain more than one source pin-pair record;
    // the later owning Pin still qualifies when D_001 and D_004 agree.
    QByteArray extraPair=original;
    extraPair.replace("U1.1 U9.2\n", "U1.2 U9.2\n");
    extraPair.replace("*END*\n","U1.1 U8.2\n*END*\n");
    if(!save(extraPair) || !load())return 2;
    const auto extraSha=QCryptographicHash::hash(extraPair,QCryptographicHash::Sha256).toHex();
    const QByteArray extraWire=query();
    c005::Receipt extraReceipt;
    const bool extraPairPositive=extraWire.contains(
           "LINK|SOURCE_PAD_TRACE_AREA_CANDIDATE|0|0|1|0|0") &&
        decode(extraWire,extraSha,extraPair,panel.imported(),
               {"RA","RB","RC"},"NET_A",1,extraReceipt) &&
        extraReceipt.markers.size()==1;
    QByteArray extraForged=extraWire;
    extraForged.replace("LINK|SOURCE_PAD_TRACE_AREA_CANDIDATE|0|0|1|0|0",
                        "LINK|SOURCE_CONTACT_NOT_VERIFIED|-|-|-|-|-");
    const bool extraForgedRejected=!decode(extraForged,extraSha,extraPair,panel.imported(),
           {"RA","RB","RC"},"NET_A",1,rejected);
    QByteArray remarked=extraPair;
    remarked.replace("U1.1 U8.2\n","*REMARK* late owning pin\nU1.1 U8.2\n");
    if(!save(remarked) || !load())return 2;
    const auto remarkSha=QCryptographicHash::hash(remarked,QCryptographicHash::Sha256).toHex();
    const QByteArray remarkWire=query();
    c005::Receipt remarkReceipt;
    const bool remarkPositive=remarkWire.contains(
           "LINK|SOURCE_PAD_TRACE_AREA_CANDIDATE|0|0|1|0|0") &&
        decode(remarkWire,remarkSha,remarked,panel.imported(),
               {"RA","RB","RC"},"NET_A",1,remarkReceipt) &&
        remarkReceipt.markers.size()==1;
    QByteArray annotated=original;
    annotated.replace("*SIGNAL* NET_A\n","*SIGNAL* NET_A 1024 0\n");
    if(!save(annotated) || !load())return 2;
    const auto metaSha=QCryptographicHash::hash(annotated,QCryptographicHash::Sha256).toHex();
    const QByteArray metaWire=query();
    c005::Receipt metaReceipt;
    const bool metadataPositive=metaWire.contains(
           "LINK|SOURCE_PAD_TRACE_AREA_CANDIDATE|0|0|1|0|0") &&
        decode(metaWire,metaSha,annotated,panel.imported(),
               {"RA","RB","RC"},"NET_A",1,metaReceipt) &&
        metaReceipt.markers.size()==1;
    const bool ok=verified && cannotSpoof && displayed && stale && negative &&
                  wrongPinNegative && wrongPinCannotSpoof &&
                  reversedPositive && reverseForgeryRejected &&
                  extraPairPositive && extraForgedRejected &&
                  remarkPositive && metadataPositive;
    qInfo()<<"D005_NATIVE_QT_PAD_TRACE_AREA"<<ok<<verified<<cannotSpoof
           <<displayed<<stale<<negative<<wrongPinNegative<<wrongPinCannotSpoof
           <<reversedPositive<<reverseForgeryRejected<<extraPairPositive<<extraForgedRejected
           <<remarkPositive<<metadataPositive<<"SOURCE_ONLY_PHYSICAL_NOT_VERIFIED";
    return ok?0:1;
}
} // namespace d005
