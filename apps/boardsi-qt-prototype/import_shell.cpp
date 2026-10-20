// M1 first usable GUI entry: offline bounded Rust PADS signature probe.
// No board geometry, artwork, Canonical import or SI calculation is performed.
#include <QApplication>
#include <QFileDialog>
#include <QFileInfo>
#include <QLabel>
#include <QProcess>
#include <QPushButton>
#include <QTimer>
#include <QVBoxLayout>
#include <QWidget>
#include <QDebug>
#include <utility>

class ImportShell final : public QWidget {
    QProcess probe;
    QTimer deadline;
    QPushButton *open;
    QLabel *status;
    QString probeBinary;
    QString expected;
    QByteArray output;
    bool testing = false;
    bool overflow = false;
    bool timedOut = false;
    bool terminalReported = false;

    void collectOutput() {
        if (overflow) return;
        const qint64 remaining = 1024 - output.size();
        if (remaining > 0) output += probe.read(remaining);
        if (probe.bytesAvailable() > 0) {
            overflow = true;
            probe.kill();
        }
    }
    void finishTest(bool ok, const QString &message) {
        if (terminalReported) return;
        terminalReported = true;
        status->setText(message);
        if (testing) {
            qInfo().noquote() << "M1_QT_PADS_SIGNATURE_SHELL" << (ok ? "PASS" : "FAIL");
            QTimer::singleShot(0, qApp, [ok] { qApp->exit(ok ? 0 : 1); });
        }
    }
    void runProbe(const QString &source) {
        if (probe.state() != QProcess::NotRunning) return;
        terminalReported = false;
        timedOut = false;
        if (!QFileInfo(probeBinary).isAbsolute() || !QFileInfo(probeBinary).isExecutable()) {
            finishTest(false, "Local Rust signature probe is unavailable");
            return;
        }
        if (!QFileInfo(source).isFile()) {
            finishTest(false, "Selected source is not a regular file");
            return;
        }
        output.clear();
        overflow = false;
        status->setText("Checking local PCB file signature (no geometry import)");
        open->setEnabled(false);
        deadline.start(5000);
        probe.start(probeBinary, {source});
    }
public:
    ImportShell(QString executable, bool selfTest, QString expectedResult)
        : probeBinary(std::move(executable)), expected(std::move(expectedResult)),
          testing(selfTest) {
        auto *layout = new QVBoxLayout(this);
        open = new QPushButton("Open PCB", this);
        status = new QLabel("No PCB loaded. PADS header probe only; SI is disabled.", this);
        status->setWordWrap(true);
        layout->addWidget(open);
        layout->addWidget(status);
        connect(open, &QPushButton::clicked, this, [this] {
            const QString file = QFileDialog::getOpenFileName(
                this, "Open PCB source", QString(), "PADS ASCII PCB (*.asc *.ASC);;All files (*)");
            if (!file.isEmpty()) runProbe(file);
        });
        probe.setProcessChannelMode(QProcess::SeparateChannels);
        probe.setReadChannel(QProcess::StandardOutput);
        connect(&probe, &QProcess::readyReadStandardOutput, this, [this] {
            collectOutput();
        });
        connect(&probe, &QProcess::readyReadStandardError, this, [this] {
            // Do not log source filenames or potentially confidential diagnostics.
            probe.readAllStandardError();
        });
        connect(&probe, &QProcess::finished, this, [this](int code, QProcess::ExitStatus exitStatus) {
            deadline.stop();
            open->setEnabled(true);
            if (terminalReported) return;
            if (timedOut) {
                finishTest(false, "Signature probe timeout; nothing imported");
                return;
            }
            collectOutput();
            QByteArray line = output;
            bool hasNewline = false;
            if (line.endsWith("\r\n")) {
                line.chop(2);
                hasNewline = true;
            } else if (line.endsWith('\n')) {
                line.chop(1);
                hasNewline = true;
            }
            const bool validExit = exitStatus == QProcess::NormalExit && !overflow && hasNewline;
            const bool pcb = validExit && code == 0 &&
                line == "PADS_PCB_ASCII_SIGNATURE_ONLY; geometry=NOT_PARSED; si=NOT_RUN";
            const bool unknown = validExit && code == 3 &&
                line == "UNKNOWN_OR_UNSUPPORTED; pcb_import=UNSUPPORTED";
            const bool schematic = validExit && code == 3 &&
                line == "PADS_SCHEMATIC_ASCII; pcb_import=UNSUPPORTED";
            const bool matched = (expected == "PCB" && pcb) ||
                (expected == "UNKNOWN" && unknown) ||
                (expected == "SCHEMATIC" && schematic);
            if (pcb) finishTest(!testing || matched,
                "PADS PCB signature detected. Geometry NOT_PARSED; SI NOT_RUN.");
            else if (unknown || schematic) finishTest(!testing || matched,
                "Unsupported PCB input. Nothing imported; SI NOT_RUN.");
            else finishTest(false, "Signature probe failed; nothing imported.");
        });
        connect(&probe, &QProcess::errorOccurred, this, [this](QProcess::ProcessError error) {
            if (error == QProcess::FailedToStart) {
                deadline.stop();
                open->setEnabled(true);
                finishTest(false, "Unable to start the local signature probe");
            }
        });
        deadline.setSingleShot(true);
        connect(&deadline, &QTimer::timeout, this, [this] {
            timedOut = true;
            if (probe.state() != QProcess::NotRunning) probe.kill();
            else finishTest(false, "Signature probe timeout; nothing imported");
        });
        resize(470, 120);
        setWindowTitle("BoardSI - PADS file preflight (M1 prototype)");
    }
    void selfTestFile(const QString &source) { runProbe(source); }
};

int main(int argc, char **argv) {
    QApplication app(argc, argv);
    const QStringList args = app.arguments(); // Unicode paths on all native OSes.
    const bool selfTest = args.size() == 5 && args.at(2) == "--selftest";
    if (args.size() != 2 && !selfTest) {
        qCritical("usage: boardsi-qt-import-shell ABSOLUTE_PROBE_PATH [--selftest FILE PCB|UNKNOWN|SCHEMATIC]");
        return 2;
    }
    ImportShell window(args.at(1), selfTest, selfTest ? args.at(4) : QString());
    window.show();
    if (selfTest) {
        const QString source = args.at(3);
        QTimer::singleShot(0, &window, [&window, source] { window.selfTestFile(source); });
    }
    return app.exec();
}
