// BoardSI local PADS import controller. Never uploads source files.
// Only restricted V9.4-MILS straight segments -> partial Canonical -> Qt, SI NOT_RUN.
#pragma once
#include "imported_artwork.h"
#include <QCoreApplication>
#include <QFileInfo>
#include <QProcess>
#include <QTimer>
#include <QTabWidget>
#include <QVBoxLayout>
#include <QDebug>
#include <QPushButton>
#include <QLabel>
#include <QHBoxLayout>
#include <functional>
#include <vector>
#include <utility>

class PreviewImportPanel final : public QWidget {
    QTabWidget *tabs_;
    QPushButton *cancelButton_ = nullptr;
    QLabel *importLabel_ = nullptr;
    bool cancelled_ = false;
    SyntheticArtworkPanel *synthetic_;
    ImportedArtworkPanel *imported_;
    QProcess process_;
    QTimer deadline_;
    QByteArray data_;
    QByteArray errors_; // Small bounded diagnostic (fixed importer error codes only).
    QString binary_;
    QString status_ = "NOT IMPORTED / SI NOT_RUN";
    std::function<void(const QString &)> callback_;
    bool expired_ = false, overflow_ = false, stderr_overflow_ = false;
    void status(const QString &message) {
        status_ = message;
        if (importLabel_) importLabel_->setText(message);
        if (cancelButton_) cancelButton_->setEnabled(message.startsWith("RUNNING"));
        if (callback_) callback_(status_);
    }
    void collect() {
        // stdout may arrive in arbitrarily large batches. The cap is cumulative,
        // not per readyRead signal; drain each available chunk before deciding overflow.
        while (process_.bytesAvailable() > 0) {
            const qint64 remaining = 2 * 1024 * 1024 - data_.size();
            if (remaining <= 0) {
                overflow_ = true;
                process_.kill();
                return;
            }
            const QByteArray chunk = process_.read(qMin<qint64>(remaining, 65536));
            if (chunk.isEmpty()) break;
            data_.append(chunk);
        }
    }
    static void appendBoundedStderr(QByteArray &captured, bool &overflow,
                                    const QByteArray &chunk) {
        const qint64 budget = 256 - captured.size();
        if (chunk.size() > budget) overflow = true;
        if (budget > 0) captured.append(chunk.left(budget));
    }
    void collectStderr() {
        // Reuse the same bounded collector as native multi-chunk regressions.
        appendBoundedStderr(errors_, stderr_overflow_, process_.readAllStandardError());
    }
    static bool decodeContactAudit(const QByteArray &bytes, QString &label) {
        // Separate versioned success diagnostic, never part of V1-V14
        // artwork stdout. Old local import workers simply have no audit.
        if (bytes.isEmpty()) {
            label = "Local endpoint contact evidence NOT_EXPOSED / full-board connectivity NOT_VERIFIED";
            return true;
        }
        if (bytes.size() > 256) return false;
        const auto fields = QString::fromLatin1(bytes).trimmed().split('|');
        if (fields.size() != 5 ||
            fields[0] != "BOARDSI_CONTACT_AUDIT_V1" ||
            fields[3] != "CONFLICT_DETAIL_NOT_EXPOSED" ||
            fields[4] != "LOCAL_ENDPOINT_ONLY")
            return false;
        bool ok = false;
        const qulonglong local = fields[1].toULongLong(&ok);
        if (!ok || local > 100000) return false;
        const qulonglong barrel = fields[2].toULongLong(&ok);
        if (!ok || barrel > 100000) return false;
        label = QString("A_032 source-backed exact local endpoint edges: %1; "
                        "qualified via-barrel edges: %2; "
                        "cross-Net conflict details NOT_EXPOSED; "
                        "area/Pad/Pin contact NOT_EVALUATED; "
                        "full-board connectivity NOT_VERIFIED")
                    .arg(local).arg(barrel);
        return true;
    }
    static bool decode(const QByteArray &bytes, int &layers, int &skipped,
                       std::vector<ImportedTrack> &tracks,
                       std::vector<ImportedVia> &vias,
                       std::vector<ImportedPad> &pads,
                       std::vector<ImportedArc> &arcs,
                       std::vector<ImportedOutlineEdge> &outlines,
                       std::vector<ImportedCircle> &circles,
                       std::vector<ImportedBoundarySegment> &boundary,
                       std::vector<ImportedPourEdge> &pours,
                       std::vector<ImportedCopperArea> &coppers) {
        if (bytes.size() > 2 * 1024 * 1024 || !bytes.endsWith('\n')) return false;
        const auto lines = QString::fromUtf8(bytes).split('\n', Qt::SkipEmptyParts);
        // V6 can carry 16,320 BOARD perimeter edges in addition to existing
        // 10,000 route segments/arcs, 10,000 vias and 5,000 pads. Byte cap
        // remains 2 MiB; preserve the narrower legacy V1-V5 record ceiling.
        if (lines.size() < 4 || lines.size() > 45000) return false;
        const auto header = lines.first().split('|');
        bool ok = false;
        const bool p2g = header.size() == 3 && header[0] == "BOARDSI_P2G_PARTIAL_V6";
        const bool p2h = header.size() == 3 && header[0] == "BOARDSI_P2H_PARTIAL_V7";
        const bool p2q = header.size() == 3 && header[0] == "BOARDSI_P2Q_PARTIAL_V12";
        const bool p2u = header.size() == 3 && header[0] == "BOARDSI_P2U_PARTIAL_V14";
        const bool p2t = header.size() == 3 &&
            (header[0] == "BOARDSI_P2T_PARTIAL_V13" || p2u);
        const bool p2n = header.size() == 3 &&
            (header[0] == "BOARDSI_P2N_PARTIAL_V11" || p2q || p2u);
        const bool p2m = header.size() == 3 &&
            (header[0] == "BOARDSI_P2M_PARTIAL_V10" || p2n || p2t);
        const bool p2l = header.size() == 3 &&
            (header[0] == "BOARDSI_P2L_PARTIAL_V9" || p2m);
        const bool p2i = header.size() == 3 &&
            (header[0] == "BOARDSI_P2I_PARTIAL_V8" || p2l);
        if (!p2g && !p2h && !p2i && lines.size() > 25100) return false;
        const bool p2f = header.size() == 3 && header[0] == "BOARDSI_P2F_PARTIAL_V5";
        const bool p2e = header.size() == 3 && header[0] == "BOARDSI_P2E_PARTIAL_V4";
        const bool p2d = header.size() == 3 && header[0] == "BOARDSI_P2D_PARTIAL_V3";
        const bool p2c = header.size() == 3 &&
                         header[0] == "BOARDSI_P2C_PARTIAL_V2";
        if (header.size() != 3 ||
            (!p2c && !p2d && !p2e && !p2f && !p2g && !p2h && !p2i &&
             header[0] != "BOARDSI_P2A_PARTIAL_V1") ||
            header[1] != "V9.4-MILS") return false;
        layers = header[2].toInt(&ok);
        if (!ok || layers < 1 || layers > 64) return false;
        int layerRecords = 0;
        bool end = false;
        for (qsizetype i = 1; i < lines.size(); ++i) {
            const auto parts = lines[i].split('|');
            if (parts.size() == 3 && parts[0] == "L" && layerRecords < layers &&
                static_cast<int>(i) == layerRecords + 1) {
                int id = parts[1].toInt(&ok);
                if (!ok || id != ++layerRecords) return false;
            } else if (parts.size() == 8 && parts[0] == "T" && layerRecords == layers) {
                ImportedTrack t;
                t.layer = parts[1].toInt(&ok);
                if (!ok || t.layer <= 0 || t.layer > layers) return false;
                t.net = parts[2];
                qint64 *fields[] = {&t.x0_nm,&t.y0_nm,&t.x1_nm,&t.y1_nm,&t.width_nm};
                for (int j = 0; j < 5; ++j) {
                    *fields[j] = parts[j+3].toLongLong(&ok);
                    if (!ok) return false;
                }
                if (tracks.size() >= 10000) return false;
                tracks.push_back(std::move(t));
            } else if ((p2f || p2g || p2h || p2i) && parts.size() == 12 && parts[0] == "A" &&
                       layerRecords == layers) {
                ImportedArc a;
                a.layer=parts[1].toInt(&ok);
                if(!ok || a.layer<1 || a.layer>layers) return false;
                a.net=parts[2];
                if(a.net.isEmpty() || a.net.size()>64) return false;
                qint64 *coordinates[]={&a.x0_nm,&a.y0_nm,&a.cx_nm,&a.cy_nm,
                                       &a.x1_nm,&a.y1_nm};
                for(int j=0;j<6;++j) {
                    *coordinates[j]=parts[j+3].toLongLong(&ok);
                    if(!ok) return false;
                }
                if(parts[9]!="CW" && parts[9]!="CCW") return false;
                a.clockwise=parts[9]=="CW";
                a.width_nm=parts[10].toLongLong(&ok);
                if(!ok) return false;
                a.sourceLine=parts[11].toInt(&ok);
                if(!ok || a.sourceLine<=0 || arcs.size()>=10000 ||
                    tracks.size()+arcs.size()>=10000) return false;
                arcs.push_back(std::move(a));
            } else if ((p2g || p2h || p2i) && parts.size() == 8 && parts[0] == "O" &&
                       layerRecords == layers) {
                ImportedOutlineEdge e;
                if (parts[1] != "OUTER" && parts[1] != "CUT") return false;
                e.cutout=parts[1]=="CUT";e.name=parts[2];
                e.sequence=parts[3].toInt(&ok);
                if (!ok || e.sequence<0 || e.name.isEmpty() ||
                    e.name.size()>16 || outlines.size()>=16384) return false;
                qint64 *coords[]={&e.x0_nm,&e.y0_nm,&e.x1_nm,&e.y1_nm};
                for (int j=0;j<4;++j) {
                    *coords[j]=parts[j+4].toLongLong(&ok);
                    if(!ok) return false;
                }
                outlines.push_back(std::move(e));
            } else if ((p2h || p2i) && parts.size() == 6 && parts[0] == "C" &&
                       layerRecords == layers) {
                ImportedCircle c;
                if (parts[1] != "OUTER" && parts[1] != "CUT") return false;
                c.cutout = parts[1] == "CUT"; c.name = parts[2];
                if(c.name.isEmpty() || c.name.size()>16 || circles.size()>=64)
                    return false;
                qint64 *fields[] = {&c.cx_nm,&c.cy_nm,&c.radius_nm};
                for(int j=0;j<3;++j) {
                    *fields[j]=parts[j+3].toLongLong(&ok);
                    if(!ok) return false;
                }
                circles.push_back(std::move(c));
            } else if (p2i && parts[0] == "B" && layerRecords == layers &&
                       (parts.size() == 9 || parts.size() == 12)) {
                ImportedBoundarySegment b;
                if(parts[1] != "OUTER" && parts[1] != "CUT") return false;
                b.cutout = parts[1] == "CUT";
                b.name = parts[2];
                b.sequence = parts[3].toInt(&ok);
                if(!ok || b.sequence < 0 || b.name.isEmpty() ||
                   b.name.size() > 16 || boundary.size() >= 16384) return false;
                b.isArc = parts[4] == "A";
                if(!b.isArc && parts[4] != "L") return false;
                if(parts.size() != (b.isArc ? 12 : 9)) return false;
                qint64 *coords[] = {&b.x0_nm,&b.y0_nm,&b.cx_nm,&b.cy_nm,
                                   &b.x1_nm,&b.y1_nm};
                const int count = b.isArc ? 6 : 4;
                for(int j=0;j<count;++j) {
                    qint64 *target = b.isArc ? coords[j] :
                        (j<2 ? coords[j] : coords[j+2]);
                    *target = parts[j+5].toLongLong(&ok);
                    if(!ok || *target % 254 != 0 ||
                       *target < -1422400000LL || *target > 1422400000LL)
                        return false;
                }
                if(b.isArc) {
                    if(parts[11] != "CW" && parts[11] != "CCW") return false;
                    b.clockwise = parts[11] == "CW";
                    const qint64 dx0=(b.x0_nm-b.cx_nm)/254;
                    const qint64 dy0=(b.y0_nm-b.cy_nm)/254;
                    const qint64 dx1=(b.x1_nm-b.cx_nm)/254;
                    const qint64 dy1=(b.y1_nm-b.cy_nm)/254;
                    if(dx0*dx0+dy0*dy0 == 0 ||
                       dx0*dx0+dy0*dy0 != dx1*dx1+dy1*dy1 ||
                       (dx0==0)==(dy0==0) || (dx1==0)==(dy1==0) ||
                       (dx0*dx1 + dy0*dy1)!=0) return false;
                    const qint64 cross = dx0*dy1-dy0*dx1;
                    if((cross<0) != b.clockwise) return false;
                } else if(b.x0_nm==b.x1_nm && b.y0_nm==b.y1_nm) return false;
                // Decode must itself reject broken mixed paths before load().
                if(!boundary.empty()) {
                    const auto &prev=boundary.back();
                    const QString old=(prev.cutout ? "CUT|" : "OUTER|")+prev.name;
                    const QString next=(b.cutout ? "CUT|" : "OUTER|")+b.name;
                    if(old==next) {
                        if(b.sequence!=prev.sequence+1 ||
                           b.x0_nm!=prev.x1_nm || b.y0_nm!=prev.y1_nm)
                            return false;
                    } else if(b.sequence!=0) return false;
                } else if(b.sequence!=0) return false;
                boundary.push_back(std::move(b));
            } else if (p2l && parts.size()==10 && parts[0]=="Q" &&
                       layerRecords==layers) {
                ImportedPourEdge e;
                e.layer=parts[1].toInt(&ok);
                if(!ok || e.layer<1 || e.layer>layers) return false;
                e.name=parts[2];e.net=parts[3];
                e.sequence=parts[4].toInt(&ok);
                if(!ok || e.sequence<0 || e.name.isEmpty() || e.name.size()>16 ||
                   e.net.isEmpty() || e.net.size()>47 || pours.size()>=16384)
                    return false;
                qint64 *coords[]={&e.x0_nm,&e.y0_nm,&e.x1_nm,&e.y1_nm};
                for(int j=0;j<4;++j) {
                    *coords[j]=parts[j+5].toLongLong(&ok);
                    if(!ok || *coords[j]%254!=0 ||
                       *coords[j]<-1422400000LL || *coords[j]>1422400000LL)
                        return false;
                }
                e.sourceLine=parts[9].toInt(&ok);
                if(!ok || e.sourceLine<=0 || (e.x0_nm==e.x1_nm && e.y0_nm==e.y1_nm))
                    return false;
                pours.push_back(std::move(e));
            } else if (p2m && parts.size() >= 9 && parts[0]=="G" &&
                       layerRecords==layers) {
                ImportedCopperArea c;
                c.circle=(parts[1]=="C");
                c.curved=(parts[1]=="R");
                if(c.curved && !p2t)return false;
                if(!c.circle && !c.curved && parts[1]!="P") return false;
                c.layer=parts[2].toInt(&ok);
                if(!ok || c.layer<1 || c.layer>layers) return false;
                c.name=parts[3]; c.net=parts[4];
                c.sourceLine=parts[5].toInt(&ok);
                if(!ok || c.sourceLine<1 || coppers.size()>=64) return false;
                if(c.curved) {
                    const int n=parts[6].toInt(&ok);
                    if(!ok || n<3 || n>64 || parts.size()!=7+8*n)return false;
                    for(int j=0;j<n;++j) {
                        const int k=7+8*j;
                        ImportedBoundarySegment e;
                        e.isArc=parts[k]=="A";
                        if(!e.isArc && parts[k]!="L")return false;
                        qint64 *coordinates[]={&e.x0_nm,&e.y0_nm,&e.cx_nm,&e.cy_nm,
                                               &e.x1_nm,&e.y1_nm};
                        for(int x=0;x<6;++x) {
                            *coordinates[x]=parts[k+x+1].toLongLong(&ok);
                            if(!ok)return false;
                        }
                        if(parts[k+7]=="CW")e.clockwise=true;
                        else if(parts[k+7]=="CCW")e.clockwise=false;
                        else if(parts[k+7]=="-" && !e.isArc)e.clockwise=false;
                        else return false;
                        if(e.isArc && parts[k+7]=="-")return false;
                        c.curvedSegments.push_back(e);
                    }
                } else if(c.circle) {
                    if(parts.size()!=9) return false;
                    c.cx_nm=parts[6].toLongLong(&ok);if(!ok)return false;
                    c.cy_nm=parts[7].toLongLong(&ok);if(!ok)return false;
                    c.radius_nm=parts[8].toLongLong(&ok);if(!ok)return false;
                } else {
                    const int n=parts[6].toInt(&ok);
                    if(!ok || n<4 || n>256 || parts.size()!=7+2*n) return false;
                    for(int j=0;j<n;++j) {
                        const qint64 x=parts[7+2*j].toLongLong(&ok);if(!ok)return false;
                        const qint64 y=parts[8+2*j].toLongLong(&ok);if(!ok)return false;
                        c.ring.push_back({x,y});
                    }
                }
                if(!validateCopperArea(c)) return false;
                for(const auto &old:coppers) if(old.name==c.name) return false;
                coppers.push_back(std::move(c));
            } else if (p2n && parts.size()>=11 && parts[0]=="H" &&
                       layerRecords==layers) {
                // V11 H is an explicitly source-owned polygon void, indexed to G.
                // Never spatially infer ownership of standalone COPCUT.
                const int id=parts[1].toInt(&ok);
                if(!ok || id<0 || id>=static_cast<int>(coppers.size())) return false;
                const int n=parts[2].toInt(&ok);
                if(!ok || n<4 || n>65 || parts.size()!=3+2*n ||
                   coppers[static_cast<size_t>(id)].cutouts.size()>=8) return false;
                std::vector<std::pair<qint64,qint64>> hole;
                for(int j=0;j<n;++j) {
                    const qint64 x=parts[3+2*j].toLongLong(&ok);if(!ok)return false;
                    const qint64 y=parts[4+2*j].toLongLong(&ok);if(!ok)return false;
                    hole.push_back({x,y});
                }
                coppers[static_cast<size_t>(id)].cutouts.push_back(std::move(hole));
            } else if (p2q && parts.size()==5 && parts[0]=="J" &&
                       layerRecords==layers) {
                // Only V12 carries analytic circular COPCCO holes. It never
                // infers ownership by spatial proximity: owner is a G index.
                const int id=parts[1].toInt(&ok);
                if(!ok || id<0 || id>=static_cast<int>(coppers.size()))
                    return false;
                auto &owner=coppers[static_cast<size_t>(id)];
                if(owner.cutouts.size()+owner.roundCutouts.size()>=8) return false;
                ImportedCopperCircleHole hole;
                hole.cx_nm=parts[2].toLongLong(&ok);if(!ok)return false;
                hole.cy_nm=parts[3].toLongLong(&ok);if(!ok)return false;
                hole.radius_nm=parts[4].toLongLong(&ok);if(!ok)return false;
                if(!a026::validCircle(hole))return false;
                owner.roundCutouts.push_back(hole);
            } else if (parts.size() == 9 && parts[0] == "V" && layerRecords == layers) {
                ImportedVia via;
                via.net = parts[1];
                via.name = parts[2];
                qint64 *measurements[] = {&via.x_nm, &via.y_nm,
                                         &via.diameter_nm, &via.drill_nm};
                for (int j = 0; j < 4; ++j) {
                    *measurements[j] = parts[j+3].toLongLong(&ok);
                    if (!ok) return false;
                }
                via.firstLayer = parts[7].toInt(&ok);
                if (!ok) return false;
                via.lastLayer = parts[8].toInt(&ok);
                if (!ok || vias.size() >= 10000) return false;
                vias.push_back(std::move(via));
            } else if ((p2c || p2d || p2e || p2f || p2g || p2h || p2i) &&
                       parts.size() == ((p2e || p2f || p2g || p2h || p2i) ? 12 : 10) &&
                       parts[0] == "P" && layerRecords == layers &&
                       parts[9] == "NET_UNVERIFIED") {
                ImportedPad pad;
                pad.reference = parts[1]; pad.pin = parts[2];
                pad.layer = parts[3].toInt(&ok);
                if (!ok) return false;
                pad.side = (p2e || p2f || p2g || p2h || p2i) ? parts[10] : "T";
                pad.rotation = (p2e || p2f || p2g || p2h || p2i) ? parts[11].toInt(&ok) : -1;
                if (!ok || ((p2e || p2f || p2g || p2h || p2i) && pad.rotation != 0 && pad.rotation != 90 &&
                            pad.rotation != 180 && pad.rotation != 270) ||
                    !((pad.side == "T" && pad.layer == 1) ||
                      ((p2e || p2f || p2g || p2h || p2i) && pad.side == "B" && layers >= 2 && pad.layer == layers)))
                    return false;
                pad.shape = parts[4];
                if (pad.shape != "R" && pad.shape != "S" &&
                    !((p2d || p2e || p2f || p2g || p2h || p2i) && pad.shape == "RF")) return false;
                qint64 *v[] = {&pad.x_nm, &pad.y_nm, &pad.width_nm, &pad.height_nm};
                for (int j = 0; j < 4; ++j) {
                    *v[j] = parts[j+5].toLongLong(&ok);
                    if (!ok) return false;
                }
                if (pads.size() >= 5000) return false;
                pads.push_back(std::move(pad));
            } else if (parts[0] == "END" && i == lines.size()-1 &&
                       layerRecords == layers &&
                       (((p2c || p2d || p2e || p2f || p2g || p2h || p2i) &&
                         parts.size() == (p2u ? 15 : p2t ? 14 : p2n ? 14 :
                             p2m ? 13 : p2l ? 12 : p2i ? 11 : p2h ? 10 :
                             p2g ? 9 : p2f ? 7 : 6)) ||
                        (!p2c && !p2d && !p2e && !p2f && !p2g && !p2h && !p2i &&
                         parts.size() == 5))) {
                const int count = parts[1].toInt(&ok);
                if (!ok || count != static_cast<int>(tracks.size())) return false;
                skipped = parts[2].toInt(&ok);
                if (!ok || skipped < 0) return false;
                if (p2c || p2d || p2e || p2f || p2g || p2h || p2i) {
                    const int padCount = parts[3].toInt(&ok);
                    if (!ok || padCount != static_cast<int>(pads.size())) return false;
                    if (p2g || p2h || p2i) {
                        const int arcCount=parts[4].toInt(&ok);
                        if(!ok || arcCount!=static_cast<int>(arcs.size())) return false;
                        int outerCount=0,cutCount=0;
                        QString lastKey;
                        for(const auto &e:outlines) {
                            const QString key=(e.cutout ? "CUT|" : "OUTER|")+e.name;
                            if(key!=lastKey) {
                                if(e.cutout)++cutCount;else ++outerCount;
                                lastKey=key;
                            }
                        }
                        for(const auto &c:circles) {
                            if(c.cutout) ++cutCount;
                            else ++outerCount;
                        }
                        if(p2i) {
                            QString last;
                            for(const auto &b:boundary) {
                                const QString key=(b.cutout ? "CUT|" : "OUTER|")+b.name;
                                if(key!=last) {
                                    if(b.cutout) ++cutCount;
                                    else ++outerCount;
                                    last=key;
                                }
                            }
                        }
                        if((p2l ? outerCount>1 : outerCount!=1) || parts[5].toInt(&ok)!=outerCount ||
                           !ok || parts[6].toInt(&ok)!=cutCount || !ok)
                            return false;
                        if (p2i) {
                            const int circleCount=parts[7].toInt(&ok);
                            if(!ok || circleCount != static_cast<int>(circles.size()))
                                return false;
                            const int arcCountB=parts[8].toInt(&ok);
                            if(!ok || (p2l ? arcCountB<0 : arcCountB<1) ||
                               parts[p2u ? 13 : (p2t ? 12 : (p2n ? 12 : (p2m ? 11 : (p2l ? 10 : 9))))]!="CANONICAL_PARTIAL" ||
                               parts[p2u ? 14 : (p2t ? 13 : (p2n ? 13 : (p2m ? 12 : (p2l ? 11 : 10))))]!="SI_NOT_RUN") return false;
                            if(p2l) {
                                int unique=0;
                                QString name;
                                for(const auto &e:pours) if(e.name!=name) {
                                    ++unique;name=e.name;
                                }
                                if(unique<(p2m ? 0 : 1) || parts[9].toInt(&ok)!=unique || !ok)
                                    return false;
                            }
                            if(p2m && (coppers.empty() ||
                               parts[10].toInt(&ok)!=static_cast<int>(coppers.size()) || !ok))
                                return false;
                            if(p2t) {
                                int curved=0;
                                for(const auto &c:coppers)
                                    if(c.curved)++curved;
                                if(curved<1 || parts[11].toInt(&ok)!=curved || !ok)
                                    return false;
                            }
                            if(p2n) {
                                int holes=0;
                                for(auto &c:coppers) {
                                    if(!validateCopperArea(c))return false;
                                    holes+=static_cast<int>(c.cutouts.size()+c.roundCutouts.size());
                                }
                                if(holes<1 || parts[p2u ? 12 : 11].toInt(&ok)!=holes || !ok) return false;
                                if(p2q) {
                                    int circularHoleCount=0;
                                    for(const auto &c:coppers)
                                        circularHoleCount+=static_cast<int>(c.roundCutouts.size());
                                    if(circularHoleCount<1)return false;
                                }
                            }
                            int countB=0;
                            for(const auto &b : boundary) if(b.isArc) ++countB;
                            if(countB!=arcCountB) return false;
                            // Every B ring must close; transport cannot count an
                            // unclosed ring as a valid physical outline.
                            for(size_t idx=0;idx<boundary.size();) {
                                const auto &first=boundary[idx];
                                const QString key=(first.cutout ? "CUT|" : "OUTER|")+first.name;
                                size_t ringEnd=idx+1;
                                while(ringEnd<boundary.size() &&
                                      ((boundary[ringEnd].cutout ? "CUT|" : "OUTER|")+boundary[ringEnd].name)==key)
                                    ++ringEnd;
                                if(ringEnd-idx<3 ||
                                   boundary[ringEnd-1].x1_nm!=first.x0_nm ||
                                   boundary[ringEnd-1].y1_nm!=first.y0_nm) return false;
                                idx=ringEnd;
                            }
                            if(boundary.empty() && !p2l) return false;
                        } else if (p2h) {
                            if(parts[7].toInt(&ok)!=static_cast<int>(circles.size()) ||
                               !ok || parts[8]!="CANONICAL_PARTIAL" ||
                               parts[9]!="SI_NOT_RUN" || circles.empty()) return false;
                        } else if (parts[7]!="CANONICAL_PARTIAL" ||
                                   parts[8]!="SI_NOT_RUN" || !circles.empty()) return false;
                    } else if (p2f) {
                        const int arcCount=parts[4].toInt(&ok);
                        if(!ok || arcCount!=static_cast<int>(arcs.size()) ||
                           arcCount<1 || parts[5]!="CANONICAL_PARTIAL" ||
                           parts[6]!="SI_NOT_RUN") return false;
                    } else if(parts[4] != "CANONICAL_PARTIAL" ||
                              parts[5] != "SI_NOT_RUN") return false;
                } else if (parts[3] != "CANONICAL_PARTIAL" ||
                           parts[4] != "SI_NOT_RUN" || !pads.empty()) return false;
                end = true;
            } else return false;
        }
        return end && (!tracks.empty() || !arcs.empty() || !vias.empty() ||
                       !pads.empty() || !outlines.empty() || !circles.empty() ||
                       !boundary.empty() || !pours.empty() || !coppers.empty());
    }
public:
    // Exercise the production decoder from native tests, especially malformed
    // V7 END trailers. No bypass of the real parser/validation logic.
    static bool validContactAuditForTest(const QByteArray &bytes) {
        QString label;
        return decodeContactAudit(bytes, label);
    }
    static bool validContactAuditChunksForTest(const std::vector<QByteArray> &chunks) {
        QByteArray captured;
        bool overflow = false;
        for (const auto &chunk : chunks) appendBoundedStderr(captured, overflow, chunk);
        QString label;
        return !overflow && decodeContactAudit(captured, label);
    }
    static bool validTransportForTest(const QByteArray &bytes) {
        int layers=0, skipped=0;
        std::vector<ImportedTrack> tracks;
        std::vector<ImportedVia> vias;
        std::vector<ImportedPad> pads;
        std::vector<ImportedArc> arcs;
        std::vector<ImportedOutlineEdge> outlines;
        std::vector<ImportedCircle> circles;
        std::vector<ImportedBoundarySegment> boundary;
        std::vector<ImportedPourEdge> pours;
        std::vector<ImportedCopperArea> coppers;
        return decode(bytes,layers,skipped,tracks,vias,pads,arcs,outlines,circles,boundary,pours,coppers);
    }
    explicit PreviewImportPanel(const QString &importBinary = QString(), QWidget *parent = nullptr)
        : QWidget(parent), tabs_(new QTabWidget(this)),
          synthetic_(new SyntheticArtworkPanel(tabs_)),
          imported_(new ImportedArtworkPanel(tabs_)) {
        setObjectName("previewImportP2A");
        binary_ = importBinary;
        if (binary_.isEmpty()) {
#ifdef Q_OS_WIN
            binary_ = QCoreApplication::applicationDirPath() + "/boardsi-import-artwork.exe";
#else
            binary_ = QCoreApplication::applicationDirPath() + "/boardsi-import-artwork";
#endif
        }
        auto *layout = new QVBoxLayout(this);
        layout->setContentsMargins(0, 0, 0, 0);
        auto *controls = new QHBoxLayout();
        importLabel_ = new QLabel("NOT IMPORTED / SI NOT_RUN", this);
        importLabel_->setWordWrap(true);
        cancelButton_ = new QPushButton("Cancel Import", this);
        cancelButton_->setObjectName("importCancelButton");
        cancelButton_->setEnabled(false);
        controls->addWidget(importLabel_, 1);
        controls->addWidget(cancelButton_);
        layout->addLayout(controls);
        connect(cancelButton_, &QPushButton::clicked, this, [this] {
            if (process_.state() != QProcess::NotRunning) {
                cancelled_ = true;
                deadline_.stop();
                process_.kill();
                status("CANCELLED / no new board imported / SI NOT_RUN");
            }
        });
        tabs_->addTab(synthetic_, "Synthetic Fixture");
        tabs_->addTab(imported_, "Imported Partial (none)");
        layout->addWidget(tabs_);
        process_.setProcessChannelMode(QProcess::SeparateChannels);
        process_.setReadChannel(QProcess::StandardOutput);
        connect(&process_, &QProcess::readyReadStandardOutput, this, [this] { collect(); });
        connect(&process_, &QProcess::readyReadStandardError, this,
                [this] { collectStderr(); });
        connect(&process_, &QProcess::finished, this,
                [this](int code, QProcess::ExitStatus exitStatus) {
            deadline_.stop();
            collect();
            collectStderr();
            if (cancelled_) {
                status("CANCELLED / no new board imported / SI NOT_RUN");
                return;
            }
            if (expired_ || overflow_ || stderr_overflow_) {
                status(expired_ ? "FAILED / IMPORT_TIMEOUT / NOTHING IMPORTED"
                       : stderr_overflow_ ? "FAILED / INVALID_CONTACT_AUDIT / NOTHING IMPORTED"
                       : "FAILED / IMPORT_OUTPUT_LIMIT / NOTHING IMPORTED");
                return;
            }
            if (code != 0 || exitStatus != QProcess::NormalExit) {
                const QString diagnostic = QString::fromLatin1(errors_).trimmed();
                const QString prefix = "P2A_IMPORT_ERROR:";
                QString errorCode;
                if (diagnostic.startsWith(prefix)) {
                    errorCode = diagnostic.mid(prefix.size());
                    if (errorCode.size() > 48) errorCode.clear();
                    for (const QChar character : errorCode) {
                        const ushort ch = character.unicode();
                        if (!((ch >= 'A' && ch <= 'Z') ||
                              (ch >= '0' && ch <= '9') || ch == '_')) {
                            errorCode.clear();
                            break;
                        }
                    }
                }
                status(errorCode.isEmpty()
                    ? QString("FAILED / invalid or unsupported local import / NOTHING IMPORTED")
                    : QString("FAILED / %1 / NOTHING IMPORTED").arg(errorCode));
                return;
            }
            int layers = 0, skipped = 0;
            std::vector<ImportedTrack> tracks;
            std::vector<ImportedVia> vias;
            std::vector<ImportedPad> pads;
            std::vector<ImportedArc> arcs;
            std::vector<ImportedOutlineEdge> outlines;
            std::vector<ImportedCircle> circles;
            std::vector<ImportedBoundarySegment> boundary;
            std::vector<ImportedPourEdge> pours;
            std::vector<ImportedCopperArea> coppers;
            if (!decode(data_, layers, skipped, tracks, vias, pads, arcs, outlines, circles, boundary,pours,coppers)) {
                status("FAILED / INVALID_TRANSPORT / NOTHING IMPORTED");
                return;
            }
            QString contactAudit;
            if (!decodeContactAudit(errors_, contactAudit)) {
                status("FAILED / INVALID_CONTACT_AUDIT / NOTHING IMPORTED");
                return;
            }
            if (!imported_->load(layers, skipped, tracks, vias, pads, arcs, outlines, circles, boundary,pours,coppers)) {
                status("FAILED / INVALID_GEOMETRY / NOTHING IMPORTED");
                return;
            }
            imported_->setLocalContactAudit(contactAudit);
            tabs_->setTabText(1, "Imported Partial");
            tabs_->setCurrentWidget(imported_);
            // The imported tab can be hidden during load(), before its viewport
            // has a usable size. Refit once it is visible in the event loop.
            QTimer::singleShot(0, imported_, [this] {
                if (tabs_->currentWidget() == imported_) imported_->fitArtwork();
            });
            status(QString("PARTIAL V9.4-MILS / %1 straight segments / %2 explicit vias"
                           " / %3 SMD pads / %4 route arcs / %5 omitted / %6 outline edges / %7 circles / %8 mixed boundary parts / %9 POUROUT edges / %10 filled source copper areas / SI NOT_RUN")
                       .arg(static_cast<qlonglong>(tracks.size()))
                       .arg(static_cast<qlonglong>(vias.size()))
                       .arg(static_cast<qlonglong>(pads.size()))
                       .arg(static_cast<qlonglong>(arcs.size())).arg(skipped)
                       .arg(static_cast<qlonglong>(outlines.size()))
                       .arg(static_cast<qlonglong>(circles.size()))
                       .arg(static_cast<qlonglong>(boundary.size()))
                       .arg(static_cast<qlonglong>(pours.size()))
                       .arg(static_cast<qlonglong>(coppers.size())));
        });
        connect(&process_, &QProcess::errorOccurred, this,
                [this](QProcess::ProcessError code) {
            if (code == QProcess::FailedToStart) {
                deadline_.stop();
                status("FAILED / IMPORTER_UNAVAILABLE / NOTHING IMPORTED");
            }
        });
        deadline_.setSingleShot(true);
        connect(&deadline_, &QTimer::timeout, this, [this] {
            expired_ = true;
            if (process_.state() != QProcess::NotRunning) process_.kill();
        });
    }
    SyntheticArtworkPanel *synthetic() const { return synthetic_; }
    ImportedArtworkPanel *imported() const { return imported_; }
    QTabWidget *artworkTabs() const { return tabs_; }
    bool showingImported() const { return tabs_->currentWidget() == imported_; }
    QString importStatus() const { return status_; }
    void setStatusCallback(std::function<void(const QString &)> cb) { callback_ = std::move(cb); }
    bool openPath(const QString &path) {
        if (process_.state() != QProcess::NotRunning) return false;
        // An import attempt replaces the candidate, even when start/parse fails.
        // Never show previous geometry with the current "NOTHING IMPORTED" result.
        imported_->clearArtwork();
        tabs_->setTabText(1, "Imported Partial (none)");
        if (!QFileInfo(binary_).isAbsolute() || !QFileInfo(binary_).isExecutable()) {
            status("FAILED / IMPORTER_UNAVAILABLE / NOTHING IMPORTED");
            return false;
        }
        if (!QFileInfo(path).isFile()) {
            status("FAILED / FILE_NOT_FOUND / NOTHING IMPORTED");
            return false;
        }
        data_.clear();
        errors_.clear();
        expired_ = false;
        overflow_ = false;
        stderr_overflow_ = false;
        cancelled_ = false;
        status("RUNNING / local PADS route import, SI NOT_RUN");
        deadline_.start(10000);
        process_.start(binary_, {path});
        return true;
    }
    void stop() {
        if (process_.state() != QProcess::NotRunning) process_.kill();
    }
    bool busy() const { return process_.state() != QProcess::NotRunning; }
};
