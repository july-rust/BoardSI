// D_056_05: source-evidenced Canonical padstack geometry for the MAIN Artwork panel.
// Visual-only: declared plating is not a manufactured barrel/electrical edge.
#pragma once
#include "d056_canonical_ipc.h"
#include <QColor>
#include <QHash>
#include <QPainterPath>
#include <QRectF>
#include <QString>
#include <QVector>
#include <algorithm>
#include <cmath>

namespace d05605 {
struct Glyph {
    int layer = 0;
    QString viaId, padstackId, drillId, netId, sourceSha, role, shape, kind;
    int first = 0, last = 0;
    bool conductive = false;
    double majorMm = 0.0, minorMm = 0.0, openingMm = 0.0;
    QPointF centre;
    QPainterPath footprint; // Filled area excludes exact annulus/drill void.
    QPainterPath outline;   // Nonconductive roles are outlines ONLY.
    QString details;
};
struct Model {
    QString sourceSha;
    int layerCount = 0;
    QVector<Glyph> glyphs;
    QRectF bounds;
};

// Convert display geometry ONLY; retain original Rational numbers in the
// immutable d056ipc::Source. No floating result is ever used as source proof.
inline bool millimetres(const QString &value, double &out, bool strictlyPositive=false) {
    if (!d056ipc::rational(value, strictlyPositive)) return false;
    out = d056ipc::dimension(value) * 1000.0;
    return std::isfinite(out) && (!strictlyPositive || out > 0.0) &&
           std::abs(out) <= 1000000.0;
}
inline QString readableId(const QString &hex) {
    const QByteArray bytes = QByteArray::fromHex(hex.toLatin1());
    const int split = bytes.indexOf('\0');
    if (split < 0) return QStringLiteral("UNKNOWN");
    return QString::fromUtf8(bytes.left(split)) +
        QStringLiteral(" / ") + QString::fromUtf8(bytes.mid(split + 1));
}
inline bool make(const d056ipc::Source &source, const QString &expectedSha,
                 Model &out) {
    out = {};
    if (source.sha != expectedSha || !d056ipc::hash64(expectedSha) ||
        source.layerIds.size() < 2 || source.layerIds.size() > 64 ||
        source.frames.isEmpty() || source.vias.size() > 10000 ||
        source.stacks.size() > 10000 || source.drills.size() > 10000)
        return false;
    Model built;
    built.sourceSha = source.sha;
    built.layerCount = source.layerIds.size();
    bool hasBounds = false;
    QHash<QString,const d056ipc::Stack *> stackById;
    QHash<QString,const d056ipc::Drill *> drillById;
    QHash<QString,const d056ipc::Frame *> frameById;
    for (const auto &entry:source.stacks) {
        if(stackById.contains(entry.id))return false;
        stackById.insert(entry.id,&entry);
    }
    for (const auto &entry:source.drills) {
        if(drillById.contains(entry.id))return false;
        drillById.insert(entry.id,&entry);
    }
    for (const auto &entry:source.frames) {
        if(frameById.contains(entry.id))return false;
        frameById.insert(entry.id,&entry);
    }
    for (const auto &via : source.vias) {
        const auto *stack=stackById.value(via.stack,nullptr);
        const auto *drill=drillById.value(via.drill,nullptr);
        const auto *frame=frameById.value(via.frame,nullptr);
        if (!stack || !drill || !frame || via.first < 1 ||
            via.last > built.layerCount || via.first >= via.last ||
            drill->first != via.first || drill->last != via.last ||
            frame->turn < 0 || frame->turn > 3)
            return false;
        double step=0,ox=0,oy=0,drillMm=0;
        if (!millimetres(frame->tick,step,true) ||
            !millimetres(frame->originX,ox) ||
            !millimetres(frame->originY,oy) ||
            !millimetres(drill->diameter,drillMm,true))
            return false;
        // Mirror, quarter turn, translation: source frame owns these values.
        double x = static_cast<double>(via.x) * step;
        double y = static_cast<double>(via.y) * step;
        if (frame->mirror) x = -x;
        const double oldX = x, oldY = y;
        switch (frame->turn) {
        case 1: x = -oldY; y = oldX; break;
        case 2: x = -oldX; y = -oldY; break;
        case 3: x = oldY; y = -oldX; break;
        default: break;
        }
        x += ox; y += oy;
        if (!std::isfinite(x) || !std::isfinite(y) ||
            std::abs(x) > 1000000.0 || std::abs(y) > 1000000.0)
            return false;
        const QPointF centre(x,-y);
        for (const auto &pad : stack->pads) {
            double major=0, minor=0;
            if (pad.layer < via.first || pad.layer > via.last ||
                !millimetres(pad.first,major,true) ||
                (pad.shape != "CIRCLE" && !millimetres(pad.second,minor,true)))
                return false;
            if (major <= 0 || major > 10000 || minor > 10000)
                return false;
            Glyph g;
            g.layer=pad.layer; g.viaId=via.id; g.padstackId=via.stack;
            g.drillId=via.drill; g.netId=via.net; g.sourceSha=source.sha;
            g.role=pad.role; g.shape=pad.shape; g.kind=via.kind;
            g.first=via.first; g.last=via.last; g.conductive=pad.conductive();
            g.centre=centre;
            g.majorMm=major;g.minorMm=minor;
            QPainterPath shape;
            const bool thermal=pad.shape=="THERMAL_RT" || pad.shape=="THERMAL_ST";
            if (thermal) {
                double spokeWidth=0.0;
                if (!pad.conductive() || !millimetres(pad.spokeWidth,spokeWidth,true)
                    || !(minor>major) || spokeWidth>major
                    || pad.spokeCount<1 || pad.spokeCount>64
                    || pad.spokeAngle<0 || pad.spokeAngle>=360) return false;
                QPainterPath outer, centreCopper;
                if (pad.shape=="THERMAL_RT") {
                    outer.addEllipse(centre,minor/2,minor/2);
                    centreCopper.addEllipse(centre,major/2,major/2);
                } else {
                    outer.addRect(QRectF(x-minor/2,-y-minor/2,minor,minor));
                    centreCopper.addRect(QRectF(x-major/2,-y-major/2,major,major));
                }
                QPainterPath spokes;
                constexpr double pi=3.14159265358979323846;
                for(int k=0;k<pad.spokeCount;++k) {
                    const double theta=(pad.spokeAngle+
                        360.0*k/pad.spokeCount)*pi/180.0;
                    double dx=std::cos(theta),dy=std::sin(theta);
                    if (frame->mirror) dx=-dx;
                    const double oldDx=dx,oldDy=dy;
                    switch(frame->turn) {
                    case 1: dx=-oldDy; dy=oldDx;break;
                    case 2: dx=-oldDx; dy=-oldDy;break;
                    case 3: dx=oldDy;dy=-oldDx;break;
                    default: break;
                    }
                    dy=-dy; // ImportedArtwork scene flips source Y only once.
                    const QPointF along(dx,dy),across(-dy,dx);
                    const QPointF start=centre+along*(major/2);
                    const QPointF end=centre+along*(minor*2);
                    QPolygonF quad;
                    quad << start+across*(spokeWidth/2)
                         << end+across*(spokeWidth/2)
                         << end-across*(spokeWidth/2)
                         << start-across*(spokeWidth/2);
                    spokes.addPolygon(quad);
                }
                shape=centreCopper.united(spokes.intersected(outer));
                g.outline=outer; // Clearance envelope is never filled Copper.
            } else {
                if (pad.shape == "RECT")
                    shape.addRect(QRectF(x-major/2, -y-minor/2, major, minor));
                else if (pad.shape == "CIRCLE" || pad.shape == "ANNULUS")
                    shape.addEllipse(centre, major/2, major/2);
                else return false;
                g.outline=shape;
            }
            if (g.conductive) {
                const double opening=(pad.shape=="ANNULUS")
                    ? std::max(minor,drillMm) : drillMm;
                // A source-declared pad fully removed by its drill is not
                // substituted with a solid copper circle.
                if (opening >= major || (pad.shape=="RECT" && opening>=minor))
                    return false;
                g.openingMm=opening;
                QPainterPath hole;
                hole.addEllipse(centre,opening/2,opening/2);
                if (thermal) g.footprint=shape.subtracted(hole);
                else {
                    shape.setFillRule(Qt::OddEvenFill);
                    shape.addPath(hole);
                    g.footprint=shape;
                }
            }
            g.details=QString("Canonical Via %1 / Padstack %2 / Drill %3 / physical L%4 / %5 %6"
                " / Via %7 L%8-L%9 / pad %10 x %11 mm / drilled hole %12 mm"
                " / SOURCE SHA256 %13 / declared net ID %14 / plating UNVERIFIED"
                " / connectivity UNKNOWN / GLOBAL Open/Short NOT_VERIFIED / SI NOT_RUN")
                .arg(readableId(via.id)).arg(readableId(via.stack))
                .arg(readableId(via.drill)).arg(pad.layer).arg(pad.role).arg(pad.shape)
                .arg(via.kind).arg(via.first).arg(via.last)
                .arg(major,0,'g',12).arg(minor,0,'g',12)
                .arg(drillMm,0,'g',12).arg(source.sha)
                .arg(via.net=="-" ? QString("UNKNOWN") : readableId(via.net));
            if (thermal) g.details+=QString(" / RT/ST spokes %1, origin %2 deg / width %3 mm / relief outer %4 mm / PLANE FILL UNKNOWN")
                .arg(pad.spokeCount).arg(pad.spokeAngle)
                .arg(pad.spokeWidth).arg(minor,0,'g',12);
            const QRectF area=g.outline.boundingRect();
            if (!hasBounds) { built.bounds=area; hasBounds=true; }
            else built.bounds=built.bounds.united(area);
            built.glyphs.append(std::move(g));
            if (built.glyphs.size() > 640000) return false;
        }
    }
    if (!hasBounds || !built.bounds.isValid() || built.bounds.isEmpty())
        return false;
    out=std::move(built);
    return true;
}
} // namespace d05605
