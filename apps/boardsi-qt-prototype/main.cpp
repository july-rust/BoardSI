// M0 source prototype: independent Worker, asynchronous signals only.
#include <QApplication>
#include <QCloseEvent>
#include <QDebug>
#include <QElapsedTimer>
#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLabel>
#include <QProcess>
#include <QProgressBar>
#include <QPushButton>
#include <QSet>
#include <utility>
#include <QTimer>
#include <QUuid>
#include <QVBoxLayout>
#include <QWidget>
#include <QtEndian>
#include <cmath>

class Window : public QWidget {
    friend class RuntimeValidation;
    QProcess process;
    QTimer monitor;
    QElapsedTimer clock;
    QPushButton *start, *cancel, *force;
    QLabel *status, *worker, *error, *results;
    QProgressBar *progress;
    QString path, run, snapshot;
    QStringList arguments;
    QString diagnostics;
    QByteArray buffer;
    qint64 lastRx = 0, lastHeartbeat = 0, cancelAt = -1, terminalAt = -1;
    quint64 expected = 1, request = 1;
    int points = 100, resultCount = 0;
    int incompleteReads = 0;
    enum State { Idle, Awaiting, Running, CancelRequested, Acknowledged, Completed, Cancelled, Failed } state = Idle;
    bool closing = false, pumping = false, exitSeen = false, scopedFailure = false;
    int exitCode = 0;
    QProcess::ExitStatus exitKind = QProcess::NormalExit;
    quint64 generation = 0;
    // Instrumentation timestamps are host-side receipt/update times, not GPU paint.
    QJsonArray observations;
    void finishExit() {
        const bool expectedFailureExit = scopedFailure && state == Failed && (exitCode == 1 || exitCode == 2);
        if (exitKind != QProcess::NormalExit || (scopedFailure ? !expectedFailureExit : exitCode != 0) || !terminal() || !buffer.isEmpty()) fail("Worker abnormal exit or incomplete IPC");
        worker->setText(QString("Worker exited: code=%1 status=%2").arg(exitCode).arg(static_cast<int>(exitKind)));
        start->setEnabled(true); cancel->setEnabled(false); force->setEnabled(false); if (closing) close();
    }
    static bool keys(const QJsonObject &object, QSet<QString> allowed) {
        if (object.size() != allowed.size()) return false;
        for (auto it = object.begin(); it != object.end(); ++it) if (!allowed.contains(it.key())) return false;
        return true;
    }
    bool terminal() const { return state == Completed || state == Cancelled || state == Failed; }
    void fail(const QString &code) {
        state = Failed; resultCount = 0; progress->setValue(0);
        results->setText("Synthetic results: 0");
        error->setText(scopedFailure ? error->text() + " | " + code : code);
        scopedFailure = false; terminalAt = clock.elapsed();
        status->setText("Failed; provisional results discarded"); cancel->setEnabled(false);
        process.kill(); // Nonblocking; finished reports actual exit separately.
    }
    bool send(QJsonObject body) {
        QJsonObject envelope{{"version", 1}, {"run_id", run}, {"snapshot_id", snapshot},
                             {"seq", static_cast<double>(request++)}, {"kind", "request"}, {"body", body}};
        const QByteArray json = QJsonDocument(envelope).toJson(QJsonDocument::Compact);
        QByteArray frame(4, '\0'); qToBigEndian<quint32>(static_cast<quint32>(json.size()), frame.data()); frame += json;
        if (process.write(frame) != frame.size()) {
            if (body.value("type").toString() == "Cancel" && cancelAt >= 0 &&
                (state == CancelRequested || state == Completed || (state == Failed && scopedFailure))) {
                if (!exitSeen && process.state() != QProcess::NotRunning && state == CancelRequested)
                    worker->setText("Cancel not delivered; awaiting Worker outcome");
                return false;
            }
            fail("IPC WriteError"); return false;
        }
        return true;
    }
    static bool integer(const QJsonValue &v, double expectedValue) {
        return v.isDouble() && v.toDouble() == expectedValue;
    }
    bool accept(const QJsonObject &m) {
        const auto receivedAt = clock.elapsed();
        if (!keys(m, {"version", "run_id", "snapshot_id", "seq", "kind", "body"}) || terminal() || !integer(m.value("version"), 1) || m.value("run_id").toString() != run ||
            m.value("snapshot_id").toString() != snapshot || !integer(m.value("seq"), static_cast<double>(expected)) ||
            !m.value("body").isObject()) return false;
        const auto b = m.value("body").toObject(); const auto type = b.value("type").toString();
        QSet<QString> fields{"type"};
        if (type == "Progress") fields.unite({"done", "total"});
        else if (type == "Result") fields.unite({"point", "frequency_hz", "s11", "s21", "synthetic"});
        else if (type == "Failed") fields.unite({"code", "context"});
        if (!keys(b, fields)) return false;
        const auto kind = (type == "Started" || type == "CancelAcknowledged") ? "response" : "event";
        if (m.value("kind").toString() != kind) return false;
        if (type == "Started" && state == Awaiting) { state = Running; cancel->setEnabled(true); status->setText("Running synthetic RLGC (not PCB SI)"); }
        else if (type == "Heartbeat" && state != Awaiting) { lastHeartbeat = clock.elapsed(); }
        else if (type == "CancelAcknowledged" && state == CancelRequested) { state = Acknowledged; status->setText("Cancel acknowledged; stopping calculation"); }
        else if (type == "Cancelled" && state == Acknowledged) { state = Cancelled; terminalAt = clock.elapsed(); resultCount = 0; progress->setValue(0); results->setText("Synthetic results: 0"); status->setText("Cancelled; awaiting process exit"); }
        else if (type == "Completed" && (state == Running || state == CancelRequested) && resultCount == points) { state = Completed; terminalAt = clock.elapsed(); cancel->setEnabled(false); status->setText("Synthetic calculation completed; awaiting process exit"); }
        else if (type == "Failed" && b.value("code").isString() && b.value("context").isString()) {
            state = Failed; scopedFailure = true; terminalAt = clock.elapsed(); resultCount = 0; progress->setValue(0); cancel->setEnabled(false);
            results->setText("Synthetic results: 0");
            error->setText(b.value("code").toString() + ": " + b.value("context").toString());
            status->setText("Worker reported failure; awaiting process exit");
        }
        else if (type == "Progress" && state != Awaiting && integer(b.value("done"), resultCount) && integer(b.value("total"), points)) { progress->setValue(resultCount); }
        else if (type == "Result" && state != Awaiting && resultCount < points && integer(b.value("point"), resultCount) &&
                 integer(b.value("frequency_hz"), resultCount * 1e6) && b.value("synthetic").isBool() && b.value("synthetic").toBool()) {
            for (const auto &key : {"s11", "s21"}) {
                if (!b.value(key).isArray()) return false;
                const auto values = b.value(key).toArray(); if (values.size() != 2) return false;
                for (const auto &v : values) if (!v.isDouble() || !std::isfinite(v.toDouble())) return false;
            }
            ++resultCount; // Only a count is retained; there is no Validated flag.
            results->setText(QString("Synthetic results: %1").arg(resultCount));
        } else return false;
        if (observations.size() < 4096) observations.append(QJsonObject{{"type", type}, {"received_ms", receivedAt}, {"updated_ms", clock.elapsed()}});
        ++expected; lastRx = clock.elapsed(); return true;
    }
    void pump() {
        if (pumping) return;
        if (state == Failed && !scopedFailure) { process.readAllStandardOutput(); buffer.clear(); if (exitSeen) finishExit(); return; }
        pumping = true;
        // Bound work per callback so queued UI events can run between bursts.
        // Apply backpressure instead of growing the application buffer during bursts.
        buffer += process.read(qMin<qint64>(65540, 131080 - buffer.size()));
        if (!buffer.isEmpty() && buffer.size() < 4) ++incompleteReads;
        for (int batch = 0; batch < 8 && buffer.size() >= 4; ++batch) {
            const auto size = qFromBigEndian<quint32>(buffer.constData());
            if (size == 0 || size > 65536) { fail("IPC FrameSize"); break; }
            if (buffer.size() < static_cast<qsizetype>(size) + 4) { ++incompleteReads; break; }
            const QByteArray json = buffer.mid(4, size); buffer.remove(0, static_cast<qsizetype>(size) + 4);
            QJsonParseError parse; const auto doc = QJsonDocument::fromJson(json, &parse);
            if (parse.error != QJsonParseError::NoError || !doc.isObject() || !accept(doc.object())) { fail("IPC invalid frame, scope, sequence or state"); break; }
        }
        pumping = false;
        if (buffer.size() > 131080) { fail("IPC buffer limit"); return; }
        if ((state != Failed || scopedFailure) && (process.bytesAvailable() > 0 || (buffer.size() >= 4 && buffer.size() >= static_cast<qsizetype>(qFromBigEndian<quint32>(buffer.constData())) + 4)))
        {
            const auto token = generation;
            QTimer::singleShot(0, this, [this, token] { if (token == generation) pump(); });
        }
        else if (exitSeen) finishExit();
    }
protected:
    void closeEvent(QCloseEvent *event) override {
        if (process.state() != QProcess::NotRunning) { closing = true; process.kill(); event->ignore(); }
        else event->accept();
    }
public:
    explicit Window(QString executable, QStringList args = {}) : path(std::move(executable)), arguments(std::move(args)) {
        auto *layout = new QVBoxLayout(this);
        start = new QPushButton("Start", this); cancel = new QPushButton("Cancel", this); force = new QPushButton("Force stop", this);
        status = new QLabel("M0 synthetic Worker prototype", this); worker = new QLabel("Worker not started", this); error = new QLabel(this); progress = new QProgressBar(this);
        results = new QLabel("Synthetic results: 0", this);
        progress->setRange(0, points); cancel->setEnabled(false); force->setEnabled(false);
        layout->addWidget(start); layout->addWidget(cancel); layout->addWidget(force); layout->addWidget(status); layout->addWidget(progress); layout->addWidget(worker); layout->addWidget(error);
        layout->addWidget(results);
        process.setProcessChannelMode(QProcess::SeparateChannels);
        process.setReadChannel(QProcess::StandardOutput);
        connect(start, &QPushButton::clicked, this, [this] {
            if (process.state() != QProcess::NotRunning) return;
            if (!QFileInfo(path).isAbsolute() || !QFileInfo(path).isExecutable()) { error->setText("Absolute executable Worker path required"); return; }
            run = QUuid::createUuid().toString(QUuid::WithoutBraces);
            ++generation; observations = QJsonArray{}; diagnostics.clear(); incompleteReads = 0; results->setText("Synthetic results: 0");
            snapshot = "synthetic-rlgc-v1:points=100:delay_ms=50";
            exitSeen = false; scopedFailure = false; terminalAt = -1; state = Awaiting; expected = request = 1; resultCount = 0; buffer.clear(); cancelAt = -1;
            clock.restart(); lastRx = lastHeartbeat = 0; error->clear(); progress->setValue(0); start->setEnabled(false); force->setEnabled(false);
            status->setText("Starting Worker"); process.start(path, arguments);
        });
        connect(&process, &QProcess::started, this, [this] {
            worker->setText("Worker running");
            send({{"type", "Start"}, {"snapshot", QJsonObject{{"points", points}, {"delay_ms", 50}}}});
        });
        connect(cancel, &QPushButton::clicked, this, [this] {
            if (state != Running) return;
            state = CancelRequested; cancelAt = clock.elapsed(); cancel->setEnabled(false); status->setText("Cancel requested; awaiting acknowledgement"); send({{"type", "Cancel"}});
        });
        connect(force, &QPushButton::clicked, this, [this] { fail("User forced termination"); });
        connect(&process, &QProcess::readyReadStandardOutput, this, [this] { pump(); });
        connect(&process, &QProcess::readyReadStandardError, this, [this] {
            // Diagnostics are drained and displayed separately from protocol output.
            diagnostics = (diagnostics + QString::fromUtf8(process.readAllStandardError())).right(2048);
            worker->setText(diagnostics.trimmed());
        });
        connect(&process, &QProcess::errorOccurred, this, [this](QProcess::ProcessError code) {
            if (code == QProcess::WriteError && cancelAt >= 0 &&
                (state == CancelRequested || state == Completed || (state == Failed && scopedFailure))) {
                // A queued Cancel write may fail after Completed or scoped Failed.
                // Do not overwrite the final OS exit status or actionable failure.
                if (!exitSeen && process.state() != QProcess::NotRunning && state == CancelRequested)
                    worker->setText("Cancel not delivered; awaiting Worker outcome");
                return;
            }
            fail("Worker process error " + QString::number(static_cast<int>(code)));
            // Crash still has unread stdout and a forthcoming finished signal.
            // Only FailedToStart has no finished signal to await.
            if (code == QProcess::FailedToStart) { start->setEnabled(true); cancel->setEnabled(false); force->setEnabled(false); if (closing) close(); }
        });
        connect(&process, &QProcess::finished, this, [this](int code, QProcess::ExitStatus exitStatus) {
            exitSeen = true; exitCode = code; exitKind = exitStatus; pump();
        });
        monitor.setInterval(100);
        connect(&monitor, &QTimer::timeout, this, [this] {
            if (state == Idle || process.state() == QProcess::NotRunning) return;
            const qint64 now = clock.elapsed();
            if (now - lastRx >= 5000 || now - lastHeartbeat >= 5000) worker->setText("Warning: no traffic or heartbeat for 5 s; process still running");
            if (now - lastRx < 5000 && now - lastHeartbeat < 5000 && worker->text().startsWith("Warning:")) worker->setText("Worker running");
            const qint64 stopAt = cancelAt >= 0 ? cancelAt : terminalAt;
            force->setEnabled(stopAt >= 0 && now - stopAt >= 10000);
        }); monitor.start(); resize(440, 300);
    }
};
#ifndef BOARDSI_RUNTIME_TEST
int main(int argc, char **argv) {
    QApplication app(argc, argv);
    if (argc != 2) { qCritical("usage: boardsi-qt-prototype ABSOLUTE_WORKER_PATH"); return 2; }
    Window window(QString::fromLocal8Bit(argv[1])); window.show(); return app.exec();
}
#endif
