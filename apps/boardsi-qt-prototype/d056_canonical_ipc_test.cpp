// D_056_04 independent native Qt IPC oracle; no PCB electrical/SI certification.
#include "d056_canonical_ipc.h"
#include <QCoreApplication>
#include <QDebug>
#include <QFile>
#include <QProcess>
#include <tuple>

static QByteArray key(const QByteArray &s) {
    QByteArray data = s;
    data.append('\0');
    return data.toHex();
}
static QByteArray makeWire(int layers, int first, int last, const QByteArray &kind) {
    const QByteArray frame = key("xy"), stack = key("stack"), drill = key("drill");
    const QByteArray via = key("via");
    const QByteArray sourceSha(64, 'a');
    QByteArray body;
    body += "F|" + frame + "|" + QByteArray("PADS tick").toHex()
            + "|127:500000000|0:1|0:1|0|false|false\n";
    for (int i = 1; i <= layers; ++i)
        body += "L|" + QByteArray::number(i) + "|" + key("layer-" + QByteArray::number(i))
                + "|" + QByteArray::number(i - 1) + "\n";
    const bool middle = last > first + 1;
    body += "S|" + stack + "|" + QByteArray::number(middle ? 3 : 2) + "\n";
    body += "P|" + stack + "|" + QByteArray::number(first)
            + "|CONDUCTIVE|RECT|127:100000|127:125000\n";
    if (middle)
        body += "P|" + stack + "|" + QByteArray::number(first + 1)
                + "|ANTIPAD|CIRCLE|127:50000|0:1\n";
    body += "P|" + stack + "|" + QByteArray::number(last)
            + "|CONDUCTIVE|ANNULUS|127:85000|127:120000\n";
    // Actual annulus outer > inner.
    body.replace("127:85000|127:120000", "127:85000|127:170000");
    body += "D|" + drill + "|127:500000|"
            + QByteArray::number(first) + "|" + QByteArray::number(last)
            + "|true|PLATING_UNVERIFIED\n";
    body += "V|" + via + "|" + stack + "|" + drill + "|-|" + frame
            + "|100|200|" + QByteArray::number(first) + "|"
            + QByteArray::number(last) + "|" + kind
            + "|CONNECTIVITY_UNKNOWN\n";
    body += "END|1|SOURCE_ONLY|GLOBAL_CONNECTIVITY_NOT_VERIFIED\n";
    const QByteArray sha = QCryptographicHash::hash(body, QCryptographicHash::Sha256).toHex();
    const QByteArray header =
        "BOARDSI_CANONICAL_PADSTACK_V1|" + sourceSha + "|" + QByteArray("PADS_ASC").toHex()
        + "|" + QByteArray("V9.4-MILS").toHex() + "|" + QByteArray("importer").toHex()
        + "|" + QByteArray("board.asc").toHex() + "|1|" + QByteArray::number(layers)
        + "|1|1|1|0|" + QByteArray::number(body.size()) + "|"
        + QByteArray::number(body.count('\n')) + "|" + sha
        + "|GEOMETRY_ONLY|SI_NOT_RUN\n";
    return header + body;
}
static bool run() {
    const QString sha(64, QLatin1Char('a'));
    for (const auto &t : {
         std::tuple<int,int,int,QByteArray>{2,1,2,"THROUGH"},
         {4,1,4,"THROUGH"}, {6,2,3,"BURIED"}, {6,2,4,"BURIED"},
         {6,1,3,"BLIND"}, {8,3,6,"BURIED"}, {64,17,41,"BURIED"} }) {
        const int layers = std::get<0>(t), first = std::get<1>(t), last = std::get<2>(t);
        const QByteArray wire = makeWire(layers, first, last, std::get<3>(t));
        d056ipc::Source result;
        if (!d056ipc::decode(wire, sha, result) || result.layerIds.size() != layers
            || result.vias.size() != 1 || result.vias[0].first != first
            || result.vias[0].last != last || result.stacks[0].pads.size() != (last > first + 1 ? 3 : 2)
            || result.drills[0].sourcePlated != true || result.vias[0].net != "-")
            return false;
        if (last > first + 1 && (result.stacks[0].pads[1].conductive()
            || result.stacks[0].pads[1].role != "ANTIPAD")) return false;
        const auto bad = [&](QByteArray modified, const QString &sourceSha = QString(64, QLatin1Char('a'))) {
            d056ipc::Source rejected;
            rejected.sha = "old-result";
            return !d056ipc::decode(modified, sourceSha, rejected)
                && rejected.sha.isEmpty() && rejected.vias.isEmpty();
        };
        if (!bad(wire, QString(64, QLatin1Char('b')))
            || !bad(QByteArray(wire).replace("BOARDSI_CANONICAL_PADSTACK_V1",
                                               "BOARDSI_CANONICAL_PADSTACK_V2"))
            || !bad(QByteArray(wire).replace("CONNECTIVITY_UNKNOWN", "VERIFIED"))
            || !bad(QByteArray(wire).replace("PLATING_UNVERIFIED", "PLATED_PROVEN"))
            || !bad(QByteArray(wire).replace("127:170000", "127:85000"))
            || !bad(QByteArray(wire).replace("END|1|", "END|2|"))
            || !bad(wire.left(wire.size()-8))
            || !bad(QByteArray(wire).append("EXTRA\n"))) return false;
    }
    return true;
}
static bool sourceRoundtrip(const QString &binary, const QString &path) {
    QFile source(path);
    if (!source.open(QIODevice::ReadOnly)) return false;
    const QByteArray sourceBytes = source.readAll();
    const QString sha = QString::fromLatin1(
        QCryptographicHash::hash(sourceBytes, QCryptographicHash::Sha256).toHex());
    QProcess process;
    process.setProcessChannelMode(QProcess::SeparateChannels);
    process.start(binary, {path});
    if (!process.waitForStarted(10000) || !process.waitForFinished(30000)
        || process.exitStatus() != QProcess::NormalExit || process.exitCode() != 0)
        return false;
    d056ipc::Source parsed;
    const QByteArray wire = process.readAllStandardOutput();
    if (!d056ipc::decode(wire, sha, parsed) || parsed.vias.isEmpty()
        || parsed.format != "PADS_ASC" || parsed.version.isEmpty()
        || parsed.stacks.isEmpty() || parsed.drills.isEmpty()) return false;
    d056ipc::Source stale;
    return !d056ipc::decode(wire, QString(64, QLatin1Char('b')), stale)
        && stale.vias.isEmpty();
}
int main(int argc, char **argv) {
    QCoreApplication app(argc, argv);
    if (app.arguments().size() == 4 && app.arguments().at(1) == "--source") {
        return sourceRoundtrip(app.arguments().at(2), app.arguments().at(3)) ? 0 : 1;
    }
    if (app.arguments().size() != 2 || app.arguments().at(1) != "--selftest")
        return 2;
    if (!run()) {
        qCritical("D_056_04 Canonical Rust-to-Qt contract regression FAILED");
        return 1;
    }
    return 0;
}
