// M0 F4 restricted synthetic Canonical Model -> immutable snapshot -> OS Worker -> Qt.
// NOT a board importer, RLGC field solver, or validated physical PCB result.
#include <QApplication>
#include <QDebug>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLabel>
#include <QProcess>
#include <QPushButton>
#include <QTimer>
#include <QVBoxLayout>
#include <QWidget>
#include <QtEndian>

struct ModelFrame {
    QByteArray bytes;
    QString run;
    QString model;
    QString snapshot;
};
static bool readModelFrame(const QString &path, ModelFrame &out) {
    QFile f(path);
    if (!f.open(QIODevice::ReadOnly) || f.size() < 5 || f.size() > 65540) return false;
    const QByteArray raw = f.readAll();
    const quint32 n = qFromBigEndian<quint32>(raw.constData());
    if (n == 0 || n > 65536 || raw.size() != static_cast<qsizetype>(n) + 4) return false;
    QJsonParseError error;
    const auto doc = QJsonDocument::fromJson(raw.mid(4), &error);
    if (error.error != QJsonParseError::NoError || !doc.isObject()) return false;
    const auto obj = doc.object();
    if (obj.size() != 8 || obj.value("schema").toInt(-1) != 2 ||
        obj.value("kind").toString() != "SyntheticSnapshotV2" ||
        !obj.value("model_json").isString() || !obj.value("points").isDouble() ||
        !obj.value("delay_ms").isDouble()) return false;
    out.run = obj.value("run_id").toString();
    out.model = obj.value("model_sha256").toString();
    out.snapshot = obj.value("snapshot_id").toString();
    if (out.run.isEmpty() || out.model.size() != 64 || out.snapshot.size() != 64) return false;
    out.bytes = raw; // Immutable capture before QProcess launch; file changes do not mutate run.
    return true;
}
class ModelHandoffWindow : public QWidget {
    QProcess child;
    QLabel *status;
    QPushButton *launch;
    QTimer timeout;
    QString executable;
    ModelFrame a, b, active;
    QByteArray pending, received;
    bool selftest = false, parsed = false, accepted = false, stale = false, protocolError = false;
    bool timedOut = false;
    int stage = 0;
    void endTest(bool ok, const QString &reason) {
        status->setText(reason);
        qInfo().noquote() << "M0_SYNTHETIC_QT_MODEL_HANDOFF" << (ok ? "PASS" : "FAIL") << reason;
        if (selftest) QTimer::singleShot(0, qApp, [ok] { qApp->exit(ok ? 0 : 1); });
    }
    void consume() {
        received += child.readAllStandardOutput();
        if (received.size() > 65540) { protocolError = true; received.clear(); child.kill(); return; }
        if (received.size() < 4) return;
        const quint32 n = qFromBigEndian<quint32>(received.constData());
        if (n == 0 || n > 65536) { protocolError = true; received.clear(); child.kill(); return; }
        if (received.size() < static_cast<qsizetype>(n) + 4) return;
        if (received.size() != static_cast<qsizetype>(n) + 4 || parsed) {
            protocolError = true; received.clear(); child.kill(); return;
        }
        QJsonParseError err;
        auto doc = QJsonDocument::fromJson(received.mid(4, n), &err);
        received.clear(); parsed = true;
        if (err.error != QJsonParseError::NoError || !doc.isObject()) {
            protocolError = true; return;
        }
        const QJsonObject obj = doc.object();
        if (obj.size() != 6 || obj.value("schema").toInt(-1) != 2 ||
            obj.value("type").toString() != "SyntheticModelReceipt" ||
            obj.value("synthetic") != true ||
            !obj.value("run_id").isString() || !obj.value("model_sha256").isString() ||
            !obj.value("snapshot_id").isString()) {
            protocolError = true; return;
        }
        const bool identity = obj.value("run_id").toString() == active.run &&
                              obj.value("model_sha256").toString() == active.model &&
                              obj.value("snapshot_id").toString() == active.snapshot;
        // Late result from previous model/run is discarded, never shown as accepted.
        if (!identity) { stale = true; status->setText("Stale model result discarded"); }
        else { accepted = true; status->setText("Restricted synthetic model receipt accepted"); }
    }
    static QByteArray tamperDigest(QByteArray frame) {
        const QByteArray key = "\"model_sha256\":\"";
        const qsizetype p = frame.indexOf(key);
        if (p < 0 || p + key.size() >= frame.size()) return {};
        const qsizetype target = p + key.size();
        frame[target] = frame.at(target) == 'a' ? 'b' : 'a';
        return frame;
    }
    void startStage(int target) {
        if (child.state() != QProcess::NotRunning) { endTest(false, "Overlap"); return; }
        stage = target; parsed = accepted = stale = protocolError = timedOut = false;
        pending = target == 1 ? b.bytes : a.bytes;
        if (target == 2) pending = tamperDigest(pending);
        if (pending.isEmpty()) { endTest(false, "Fixture corruption"); return; }
        active = target == 1 ? b : a;
        received.clear(); launch->setEnabled(false);
        status->setText("Starting separate OS Worker");
        // Arm before process launch: FailedToStart can be delivered immediately.
        timeout.start(10000);
        if (selftest && target == 3) {
            // A nonexistent executable deterministically exercises the QProcess
            // failed-start error path after the successful identity tests.
            const QString missing = executable + ".boardsi-m0-intentionally-absent";
            if (QFileInfo::exists(missing)) { timeout.stop(); endTest(false, "Missing Worker fixture exists"); return; }
            child.start(missing, {"--model-snapshot-v2"});
        } else {
            child.start(executable, {"--model-snapshot-v2"});
        }
    }
public:
    ModelHandoffWindow(QString worker, ModelFrame first, ModelFrame second, bool testing)
        : executable(std::move(worker)), a(std::move(first)), b(std::move(second)), selftest(testing) {
        auto *layout = new QVBoxLayout(this);
        launch = new QPushButton("Run immutable synthetic model", this);
        status = new QLabel("M0 synthetic-only Qt/Worker model identity proof", this);
        layout->addWidget(launch); layout->addWidget(status);
        if (!QFileInfo(executable).isAbsolute() || !QFileInfo(executable).isExecutable()) {
            QTimer::singleShot(0, this, [this] { endTest(false, "Worker must be an absolute executable"); });
            return;
        }
        timeout.setSingleShot(true);
        connect(&timeout, &QTimer::timeout, this, [this] {
            timedOut = true;
            timeout.stop();
            child.kill();
            // Selftest phase 4 deliberately injects the timeout without
            // waiting 10 s, then verifies the OS-exit handler preserves it.
            if (selftest && stage == 4) status->setText("Worker timeout");
            else endTest(false, "Worker timeout");
        });
        connect(launch, &QPushButton::clicked, this, [this] { startStage(1); });
        connect(&child, &QProcess::started, this, [this] {
            if (selftest && stage == 4) { timeout.timeout(); return; }
            if (child.write(pending) != pending.size()) {
                protocolError = true; child.kill(); return;
            }
            if (selftest && stage == 0) {
                // Deterministic modification after immutable frame dispatch.
                // A's response cannot be accepted as the currently selected B model/run.
                active = b;
            }
        });
        connect(&child, &QProcess::readyReadStandardOutput, this, [this] { consume(); });
        connect(&child, &QProcess::finished, this, [this](int code, QProcess::ExitStatus exitKind) {
            timeout.stop(); consume(); launch->setEnabled(true);
            if (timedOut) {
                // A kill-induced finished() is expected, not a new diagnosis.
                // Preserve the original timeout and restore launch for retry.
                if (selftest && stage == 4)
                    endTest(status->text() == "Worker timeout" && launch->isEnabled(),
                            "timeout diagnosis preserved after OS exit; UI recovered");
                return;
            }
            if (exitKind != QProcess::NormalExit || protocolError || !received.isEmpty()) {
                endTest(false, "Framing or process error"); return;
            }
            if (!selftest) {
                endTest(code == 0 && accepted, code == 0 && accepted ?
                        "Synthetic-only receipt accepted" : "Worker did not validate model");
                return;
            }
            if (stage == 0) {
                if (code != 0 || !parsed || accepted || !stale) {
                    endTest(false, "Stale result not rejected"); return;
                }
                startStage(1);
            } else if (stage == 1) {
                if (code != 0 || !parsed || !accepted || stale) {
                    endTest(false, "Fresh model receipt mismatch"); return;
                }
                startStage(2);
            } else if (stage == 2) {
                if (code == 0 || parsed || accepted) {
                    endTest(false, "Tampered model snapshot accepted"); return;
                }
                startStage(3);
            } else if (stage == 3) {
                endTest(false, "Missing Worker unexpectedly finished");
            }
        });
        connect(&child, &QProcess::errorOccurred, this, [this](QProcess::ProcessError e) {
            if (e != QProcess::FailedToStart) return;
            // FailedToStart does not guarantee a subsequent finished() signal.
            // Restore the UI now, preserving the actionable error and allowing retry.
            timeout.stop();
            launch->setEnabled(true);
            pending.clear();
            received.clear();
            if (selftest && stage == 3) {
                if (child.state() != QProcess::NotRunning || timeout.isActive() || !launch->isEnabled())
                    endTest(false, "Failed-start UI recovery incomplete");
                else
                    QTimer::singleShot(0, this, [this] { startStage(4); });
            } else {
                endTest(false, "OS Worker failed to start");
            }
        });
        resize(460, 120);
        if (selftest) QTimer::singleShot(0, this, [this] { startStage(0); });
    }
};
int main(int argc, char **argv) {
    QApplication app(argc, argv);
    QString executable, firstPath, secondPath;
    bool testing = false;
    if (argc == 5 && QString::fromLocal8Bit(argv[1]) == "--selftest") {
        testing = true; executable = QString::fromLocal8Bit(argv[2]);
        firstPath = QString::fromLocal8Bit(argv[3]); secondPath = QString::fromLocal8Bit(argv[4]);
    } else if (argc == 4) {
        executable = QString::fromLocal8Bit(argv[1]); firstPath = QString::fromLocal8Bit(argv[2]);
        secondPath = QString::fromLocal8Bit(argv[3]);
    } else {
        qCritical("Usage: qt-model-v2 [--selftest] WORKER_ABSOLUTE SNAP_A SNAP_B");
        return 2;
    }
    ModelFrame first, second;
    if (!readModelFrame(firstPath, first) || !readModelFrame(secondPath, second) ||
        first.run == second.run || first.model == second.model || first.snapshot == second.snapshot) {
        qCritical("Synthetic Canonical Model snapshots missing, invalid, or indistinguishable");
        return 2;
    }
    ModelHandoffWindow window(executable, first, second, testing);
    window.show();
    return app.exec();
}
