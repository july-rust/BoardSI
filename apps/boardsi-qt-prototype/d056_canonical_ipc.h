// D_056_04: strict, transactional, source-bound Canonical Padstack IPC V1.
// Geometry and source declarations only. NEVER a manufactured electrical/SI verdict.
#pragma once
#include <QByteArray>
#include <QCryptographicHash>
#include <QMap>
#include <QRegularExpression>
#include <QSet>
#include <QString>
#include <QStringList>
#include <QVector>
#include <algorithm>
#include <cmath>
#include <utility>

namespace d056ipc {
struct Pad {
    int layer = 0;
    QString role, shape, first, second;
    bool conductive() const { return role == "CONDUCTIVE"; }
};
struct Stack { QString id; QVector<Pad> pads; };
struct Drill { QString id, diameter; int first = 0, last = 0; bool sourcePlated = false; };
struct Via {
    QString id, stack, drill, net, frame, kind;
    qint64 x = 0, y = 0;
    int first = 0, last = 0;
};
struct Frame {
    QString id, unit, tick, originX, originY;
    int turn = 0;
    bool mirror = false, layerReversed = false;
};
struct Source {
    QString sha, format, version, importer, logicalPath;
    QVector<QPair<QString, QString>> conditions;
    QVector<Frame> frames;
    QVector<QString> layerIds;
    QVector<int> layerOrders;
    QVector<Stack> stacks;
    QVector<Drill> drills;
    QVector<Via> vias;
};

inline bool decimal(const QString &v, bool signedValue, qint64 *out = nullptr) {
    static const QRegularExpression positive(QStringLiteral("^(0|[1-9][0-9]*)$"));
    static const QRegularExpression signedRe(QStringLiteral("^(0|-?[1-9][0-9]*)$"));
    if (v.size() > 20 || !(signedValue ? signedRe : positive).match(v).hasMatch())
        return false;
    bool ok = false;
    const qint64 value = v.toLongLong(&ok, 10);
    if (!ok || (!signedValue && value < 0)) return false;
    if (out) *out = value;
    return true;
}
inline bool inRange(const QString &v, qint64 lo, qint64 hi, int *dst = nullptr) {
    qint64 n = 0;
    if (!decimal(v, false, &n) || n < lo || n > hi) return false;
    if (dst) *dst = int(n);
    return true;
}
inline bool hash64(const QString &s) {
    static const QRegularExpression re(QStringLiteral("^[0-9a-f]{64}$"));
    return re.match(s).hasMatch();
}
inline bool hexToken(const QString &s, bool objectId) {
    if (s.size() > 2048 || s.size() % 2 != 0) return false;
    static const QRegularExpression re(QStringLiteral("^[0-9a-f]*$"));
    if (!re.match(s).hasMatch()) return false;
    const QByteArray bytes = QByteArray::fromHex(s.toLatin1());
    if (bytes.size() * 2 != s.size() || (bytes.isEmpty() && objectId)) return false;
    if (QString::fromUtf8(bytes).toUtf8() != bytes) return false;
    if (objectId && bytes.count('\0') != 1) return false;
    return true;
}
inline bool rational(const QString &s, bool positive) {
    const auto parts = s.split(':', Qt::KeepEmptyParts);
    if (parts.size() != 2) return false;
    static const QRegularExpression n(QStringLiteral("^(0|-?[1-9][0-9]*)$"));
    static const QRegularExpression d(QStringLiteral("^[1-9][0-9]*$"));
    if (parts[0].size() > 40 || parts[1].size() > 39
        || !n.match(parts[0]).hasMatch() || !d.match(parts[1]).hasMatch()) return false;
    if (positive && (parts[0] == "0" || parts[0].startsWith('-'))) return false;
    return true;
}
inline double dimension(const QString &s) {
    const auto p = s.split(':');
    bool okA = false, okB = false;
    const double a = p.value(0).toDouble(&okA);
    const double b = p.value(1).toDouble(&okB);
    return okA && okB && b > 0 ? a / b : -1.0;
}

inline bool decode(const QByteArray &wire, const QString &expectedSourceSha, Source &out) {
    out = {}; // never expose partially parsed or stale source data
    if (wire.size() < 150 || wire.size() > 4 * 1024 * 1024
        || !wire.endsWith('\n') || wire.contains('\r') || !hash64(expectedSourceSha))
        return false;
    const int split = wire.indexOf('\n');
    if (split < 0 || split > 4096) return false;
    const QByteArray rawHead = wire.left(split);
    if (QString::fromUtf8(rawHead).toUtf8() != rawHead) return false;
    const auto h = QString::fromUtf8(rawHead).split('|', Qt::KeepEmptyParts);
    if (h.size() != 17 || h[0] != "BOARDSI_CANONICAL_PADSTACK_V1"
        || !hash64(h[1]) || h[1] != expectedSourceSha
        || h[2].isEmpty() || h[3].isEmpty() || h[4].isEmpty() || h[5].isEmpty()
        || !hexToken(h[2], false) || !hexToken(h[3], false)
        || !hexToken(h[4], false) || !hexToken(h[5], false)
        || h[15] != "GEOMETRY_ONLY" || h[16] != "SI_NOT_RUN") return false;
    int frames = 0, layers = 0, stacks = 0, drills = 0, vias = 0, conditions = 0;
    int bytes = 0, records = 0;
    if (!inRange(h[6], 1, 1024, &frames) || !inRange(h[7], 2, 64, &layers)
        || !inRange(h[8], 0, 10000, &stacks) || !inRange(h[9], 0, 10000, &drills)
        || !inRange(h[10], 0, 10000, &vias) || !inRange(h[11], 0, 1024, &conditions)
        || !inRange(h[12], 1, 4 * 1024 * 1024, &bytes)
        || !inRange(h[13], 1, 700000, &records) || !hash64(h[14]))
        return false;
    const QByteArray rawBody = wire.mid(split + 1);
    if (rawBody.size() != bytes || QString::fromUtf8(rawBody).toUtf8() != rawBody
        || QString::fromLatin1(QCryptographicHash::hash(rawBody, QCryptographicHash::Sha256).toHex()) != h[14])
        return false;
    const auto lines = QString::fromUtf8(rawBody).split('\n', Qt::KeepEmptyParts);
    if (lines.size() != records + 1 || !lines.last().isEmpty()) return false;
    Source parsed;
    parsed.sha = h[1];
    parsed.format = QString::fromUtf8(QByteArray::fromHex(h[2].toLatin1()));
    parsed.version = QString::fromUtf8(QByteArray::fromHex(h[3].toLatin1()));
    parsed.importer = QString::fromUtf8(QByteArray::fromHex(h[4].toLatin1()));
    parsed.logicalPath = QString::fromUtf8(QByteArray::fromHex(h[5].toLatin1()));
    parsed.layerIds.reserve(layers);
    parsed.layerOrders.reserve(layers);
    parsed.frames.reserve(frames);
    parsed.stacks.reserve(stacks);
    parsed.drills.reserve(drills);
    parsed.vias.reserve(vias);
    QSet<QString> frameIds, stackIds, drillIds, viaIds, layerIds;
    QMap<QString, int> stackPositions, drillPositions;
    QMap<QString, QSet<int>> padLayers;
    int stage = 0, pending = 0;
    bool ended = false;
    for (int i = 0; i < records; ++i) {
        const auto r = lines[i].split('|', Qt::KeepEmptyParts);
        if (r.isEmpty() || ended) return false;
        const QString &tag = r[0];
        int nextStage = 0;
        if (tag == "I") nextStage = 0;
        else if (tag == "F") nextStage = 1;
        else if (tag == "L") nextStage = 2;
        else if (tag == "S" || tag == "P") nextStage = 3;
        else if (tag == "D") nextStage = 4;
        else if (tag == "V") nextStage = 5;
        else if (tag == "END") nextStage = 6;
        else return false;
        if (nextStage < stage || (pending > 0 && tag != "P")) return false;
        stage = nextStage;
        if (tag == "I") {
            if (r.size() != 3 || parsed.conditions.size() >= conditions
                || !hexToken(r[1], false) || !hexToken(r[2], false)) return false;
            parsed.conditions.push_back({
                QString::fromUtf8(QByteArray::fromHex(r[1].toLatin1())),
                QString::fromUtf8(QByteArray::fromHex(r[2].toLatin1()))});
        } else if (tag == "F") {
            int turn = 0;
            if (r.size() != 9 || parsed.frames.size() >= frames
                || !hexToken(r[1], true) || frameIds.contains(r[1])
                || r[2].isEmpty() || !hexToken(r[2], false) || !rational(r[3], true)
                || !rational(r[4], false) || !rational(r[5], false)
                || !inRange(r[6], 0, 3, &turn)
                || (r[7] != "true" && r[7] != "false")
                || (r[8] != "true" && r[8] != "false")) return false;
            frameIds.insert(r[1]);
            parsed.frames.push_back(Frame{r[1], QString::fromUtf8(QByteArray::fromHex(r[2].toLatin1())), r[3], r[4], r[5],
                turn, r[7] == "true", r[8] == "true"});
        } else if (tag == "L") {
            int ordinal = 0, order = 0;
            if (r.size() != 4 || parsed.layerIds.size() >= layers
                || !inRange(r[1], parsed.layerIds.size() + 1,
                            parsed.layerIds.size() + 1, &ordinal)
                || !hexToken(r[2], true) || layerIds.contains(r[2])
                || !inRange(r[3], 0, 100000, &order)
                || (!parsed.layerOrders.isEmpty() && order <= parsed.layerOrders.last()))
                return false;
            layerIds.insert(r[2]);
            parsed.layerIds.push_back(r[2]);
            parsed.layerOrders.push_back(order);
        } else if (tag == "S") {
            if (r.size() != 3 || parsed.stacks.size() >= stacks || pending
                || !hexToken(r[1], true) || stackIds.contains(r[1])
                || !inRange(r[2], 0, 64, &pending)) return false;
            stackIds.insert(r[1]);
            stackPositions.insert(r[1], parsed.stacks.size());
            parsed.stacks.push_back(Stack{r[1], {}});
        } else if (tag == "P") {
            int layer = 0;
            if (r.size() != 7 || pending <= 0 || parsed.stacks.isEmpty()
                || r[1] != parsed.stacks.last().id
                || !inRange(r[2], 1, layers, &layer)
                || padLayers[r[1]].contains(layer)
                || (r[3] != "CONDUCTIVE" && r[3] != "ANTIPAD" && r[3] != "CLEARANCE")
                || (r[4] != "CIRCLE" && r[4] != "RECT" && r[4] != "ANNULUS")
                || !rational(r[5], true)
                || !rational(r[6], r[4] != "CIRCLE")) return false;
            if (r[4] == "CIRCLE" && r[6] != "0:1") return false;
            if (r[4] == "ANNULUS") {
                const double outer = dimension(r[5]), inner = dimension(r[6]);
                if (!std::isfinite(outer) || !std::isfinite(inner) || inner <= 0
                    || outer <= inner || (outer - inner) <= outer * 1e-14)
                    return false; // precision-ambiguous boundaries fail closed
            }
            padLayers[r[1]].insert(layer);
            parsed.stacks.last().pads.push_back(Pad{layer, r[3], r[4], r[5], r[6]});
            --pending;
        } else if (tag == "D") {
            int first = 0, last = 0;
            if (r.size() != 7 || parsed.drills.size() >= drills
                || !hexToken(r[1], true) || drillIds.contains(r[1])
                || !rational(r[2], true)
                || !inRange(r[3], 1, layers - 1, &first)
                || !inRange(r[4], first + 1, layers, &last)
                || (r[5] != "true" && r[5] != "false")
                || r[6] != "PLATING_UNVERIFIED") return false;
            drillIds.insert(r[1]);
            drillPositions.insert(r[1], parsed.drills.size());
            parsed.drills.push_back(Drill{r[1], r[2], first, last, r[5] == "true"});
        } else if (tag == "V") {
            int first = 0, last = 0;
            qint64 x = 0, y = 0;
            if (r.size() != 12 || parsed.vias.size() >= vias
                || !hexToken(r[1], true) || viaIds.contains(r[1])
                || !stackIds.contains(r[2]) || !drillIds.contains(r[3])
                || (r[4] != "-" && !hexToken(r[4], true))
                || !frameIds.contains(r[5])
                || !decimal(r[6], true, &x) || !decimal(r[7], true, &y)
                || !inRange(r[8], 1, layers - 1, &first)
                || !inRange(r[9], first + 1, layers, &last)
                || (r[10] != "THROUGH" && r[10] != "BLIND"
                    && r[10] != "BURIED" && r[10] != "MICROVIA_DECLARED")
                || r[11] != "CONNECTIVITY_UNKNOWN") return false;
            const bool top = first == 1, bottom = last == layers;
            if ((r[10] == "THROUGH" && !(top && bottom))
                || (r[10] == "BLIND" && top == bottom)
                || (r[10] == "BURIED" && (top || bottom))
                || (r[10] == "MICROVIA_DECLARED" && last != first + 1))
                return false;
            const auto drillIt = drillPositions.constFind(r[3]);
            const auto stackIt = stackPositions.constFind(r[2]);
            if (drillIt == drillPositions.cend() || stackIt == stackPositions.cend())
                return false;
            const Drill &d = parsed.drills.at(drillIt.value());
            const Stack &stack = parsed.stacks.at(stackIt.value());
            if (d.first != first || d.last != last) return false;
            if (!std::any_of(stack.pads.cbegin(), stack.pads.cend(),
                    [](const Pad &p) { return p.conductive(); })) return false;
            for (const auto &p : stack.pads)
                if (p.layer < first || p.layer > last) return false;
            viaIds.insert(r[1]);
            parsed.vias.push_back(Via{r[1], r[2], r[3], r[4], r[5], r[10],
                x, y, first, last});
        } else if (tag == "END") {
            if (r.size() != 4 || i != records - 1
                || !inRange(r[1], vias, vias)
                || r[2] != "SOURCE_ONLY"
                || r[3] != "GLOBAL_CONNECTIVITY_NOT_VERIFIED") return false;
            ended = true;
        }
    }
    if (!ended || pending || parsed.conditions.size() != conditions
        || parsed.frames.size() != frames || parsed.layerIds.size() != layers
        || parsed.stacks.size() != stacks || parsed.drills.size() != drills
        || parsed.vias.size() != vias) return false;
    out = std::move(parsed);
    return true;
}
} // namespace d056ipc
