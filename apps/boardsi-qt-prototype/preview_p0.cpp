// BoardSI Developer Preview P0. UI only: no PCB geometry or physical SI solver.
// Reuse the independently regression-tested M0 QProcess IPC state machines verbatim.
// This single-translation-unit adapter is deliberately narrow for the first preview.
#define BOARDSI_RUNTIME_TEST
#include "main.cpp"
#define BOARDSI_MODEL_V2_NO_MAIN
#include "model_v2.cpp"

#include <QAction>
#include <QCloseEvent>
#include <QCryptographicHash>
#include <QDir>
#include <QFileDialog>
#include <QGroupBox>
#include <QGuiApplication>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QMainWindow>
#include <QPlainTextEdit>
#include <QSplitter>
#include <QStatusBar>
#include <QSysInfo>
#include <QTabWidget>
#include <QTextEdit>
#include <QToolBar>
#include <QTreeWidget>
#include <QVBoxLayout>

class PreviewWindow : public QMainWindow {
    Window *ipc;
    ModelHandoffWindow *identity = nullptr;
    QAction *openAction = nullptr;
    QAction *saveAction = nullptr;
    QAction *runAction = nullptr;
    QAction *stopAction = nullptr;
    QLabel *artwork = nullptr;
    QLabel *properties = nullptr;
    QLabel *validation = nullptr;
    QLabel *stateSummary = nullptr;
    QTextEdit *resultSummary = nullptr;
    QPlainTextEdit *log = nullptr;
    QProgressBar *overallProgress = nullptr;
    QString previousStatus;
    QString previousError;
    QString previousIdentity;
    QString chosenFile;

    static QWidget *unsupportedTab(const QString &message, QWidget *parent) {
        auto *page = new QWidget(parent);
        auto *layout = new QVBoxLayout(page);
        auto *label = new QLabel(message, page);
        label->setWordWrap(true);
        label->setAlignment(Qt::AlignCenter);
        layout->addWidget(label);
        return page;
    }

    void refresh() {
        runAction->setEnabled(ipc->runButton()->isEnabled());
        stopAction->setEnabled(ipc->cancelButton()->isEnabled());
        overallProgress->setValue(ipc->progressBar()->value());
        const QString status = ipc->statusLabel()->text();
        const QString error = ipc->errorLabel()->text();
        stateSummary->setText("SYNTHETIC / NOT PCB SI  |  " + status);
        resultSummary->setPlainText(
            "DEVELOPER PREVIEW P0  -  SYNTHETIC / NOT PCB SI\n\n"
            "Worker: independent Rust process\n" + status + "\n" +
            ipc->resultLabel()->text() +
            "\n\nGenerated demonstration numbers are NOT physical PCB S-parameters.\n"
            "No PCB import, artwork or impedance analysis has been performed.");
        if (status != previousStatus) {
            log->appendPlainText("Worker: " + status);
            previousStatus = status;
        }
        if (!error.isEmpty() && error != previousError)
            log->appendPlainText("ERROR: " + error);
        previousError = error;
        if (identity) {
            const QString current = identity->statusLabel()->text();
            if (current != previousIdentity) {
                log->appendPlainText("Canonical A/B: " + current);
                previousIdentity = current;
            }
        }
    }

public:
    PreviewWindow(const QString &workerPath, const QString &firstFrame,
                  const QString &secondFrame = QString()) {
        setObjectName("boardsiPreviewP0");
        setWindowTitle("BoardSI Developer Preview P0 - SYNTHETIC / NOT PCB SI");
        auto *bar = addToolBar("Preview controls");
        bar->setMovable(false);
        openAction = bar->addAction("Open PCB");
        saveAction = bar->addAction("Save Project");
        saveAction->setEnabled(false);
        saveAction->setToolTip("NOT IMPLEMENTED - project serialization");
        bar->addSeparator();
        runAction = bar->addAction("Run");
        stopAction = bar->addAction("Stop");
        stopAction->setEnabled(false);
        runAction->setToolTip("Run synthetic progress demonstration (NOT PCB SI)");
        stopAction->setToolTip("Request asynchronous Worker cancellation");

        auto *root = new QWidget(this);
        auto *vertical = new QVBoxLayout(root);
        auto *header = new QLabel(
            "BoardSI PREVIEW P0    |    SYNTHETIC / NOT PCB SI    |    Developer build only",
            root);
        header->setStyleSheet("font-weight: bold; padding: 8px; background: #2b3441; color: #ffffff;");
        vertical->addWidget(header);

        auto *horizontal = new QSplitter(Qt::Horizontal, root);
        auto *tree = new QTreeWidget(horizontal);
        tree->setHeaderLabel("Project / Workflow");
        tree->setMinimumWidth(185);
        auto *project = new QTreeWidgetItem(tree, {"Project"});
        for (const auto &name : {"PCB - NOT IMPORTED", "Substrate - UNSUPPORTED",
                                 "Layers - UNSUPPORTED", "Nets - UNSUPPORTED"})
            new QTreeWidgetItem(project, {QString::fromLatin1(name)});
        auto *workflow = new QTreeWidgetItem(tree, {"Workflow"});
        for (const auto &name : {"Select PCB file only", "Import - NOT IMPLEMENTED",
                                 "Run synthetic Worker", "Inspect demo status"})
            new QTreeWidgetItem(workflow, {QString::fromLatin1(name)});
        tree->expandAll();

        auto *center = new QWidget(horizontal);
        auto *centerLayout = new QVBoxLayout(center);
        auto *centerTitle = new QLabel("Artwork Viewer", center);
        centerTitle->setStyleSheet("font-weight: bold;");
        centerLayout->addWidget(centerTitle);
        artwork = new QLabel(
            "ARTWORK VIEWER - NOT IMPLEMENTED\n\n"
            "No board geometry has been imported.\n"
            "Open PCB only selects a local file; it does not parse or analyze it.",
            center);
        artwork->setObjectName("artworkUnsupported");
        artwork->setAlignment(Qt::AlignCenter);
        artwork->setWordWrap(true);
        artwork->setMinimumHeight(190);
        artwork->setStyleSheet("background: #29313b; color: #f2f2f2; padding: 20px;");
        centerLayout->addWidget(artwork, 3);

        auto *tabs = new QTabWidget(center);
        resultSummary = new QTextEdit(tabs);
        resultSummary->setObjectName("previewResults");
        resultSummary->setReadOnly(true);
        tabs->addTab(resultSummary, "Results");
        tabs->addTab(unsupportedTab(
            "S-PARAMETERS - NOT AVAILABLE\nSynthetic Worker data are not physical PCB SI results.", tabs),
            "S-Parameters");
        tabs->addTab(unsupportedTab(
            "IMPEDANCE - NOT AVAILABLE\nNo PCB geometry or validated field solve exists in P0.", tabs),
            "Impedance");
        log = new QPlainTextEdit(tabs);
        log->setObjectName("previewLog");
        log->setReadOnly(true);
        log->setMaximumBlockCount(250);
        tabs->addTab(log, "Log");
        ipc = new Window(workerPath);
        ipc->setObjectName("syntheticWorkerControls");
        tabs->addTab(ipc, "Progress");
        centerLayout->addWidget(tabs, 2);

        auto *right = new QWidget(horizontal);
        right->setMinimumWidth(255);
        auto *rightLayout = new QVBoxLayout(right);
        auto *propBox = new QGroupBox("Properties", right);
        auto *propLayout = new QVBoxLayout(propBox);
        properties = new QLabel(
            "Mode: Developer Preview P0\n"
            "Source: synthetic only\n"
            "PCB: NOT LOADED\n"
            "Stackup: NOT LOADED\n"
            "Solver: NOT PCB SI", propBox);
        properties->setWordWrap(true);
        propLayout->addWidget(properties);
        rightLayout->addWidget(propBox);
        auto *validBox = new QGroupBox("Validation", right);
        auto *validLayout = new QVBoxLayout(validBox);
        validation = new QLabel(
            "PCB import: NOT_RUN\n"
            "Artwork: NOT_RUN\n"
            "Physical SI: NOT_RUN\n"
            "Worker: awaiting demo", validBox);
        validation->setWordWrap(true);
        validLayout->addWidget(validation);
        rightLayout->addWidget(validBox);

        ModelFrame a, b;
        if (readModelFrame(firstFrame, a) && readModelFrame(secondFrame, b) &&
            a.run != b.run && a.model != b.model && a.snapshot != b.snapshot) {
            auto *identityBox = new QGroupBox("Canonical Model A/B - SYNTHETIC", right);
            auto *identityLayout = new QVBoxLayout(identityBox);
            identity = new ModelHandoffWindow(workerPath, a, b, false);
            identity->setObjectName("canonicalSyntheticAB");
            identityLayout->addWidget(identity);
            rightLayout->addWidget(identityBox);
        } else {
            rightLayout->addWidget(unsupportedTab(
                "Canonical A/B proof: NOT_RUN\nGenerate the two synthetic fixture frames to enable it.", right));
        }
        rightLayout->addStretch(1);
        horizontal->addWidget(tree);
        horizontal->addWidget(center);
        horizontal->addWidget(right);
        horizontal->setStretchFactor(0, 1);
        horizontal->setStretchFactor(1, 5);
        horizontal->setStretchFactor(2, 2);
        vertical->addWidget(horizontal);
        setCentralWidget(root);

        stateSummary = new QLabel("SYNTHETIC / NOT PCB SI - idle", this);
        overallProgress = new QProgressBar(this);
        overallProgress->setRange(0, 100);
        overallProgress->setMaximumWidth(155);
        statusBar()->addWidget(stateSummary, 1);
        statusBar()->addPermanentWidget(overallProgress);

        connect(runAction, &QAction::triggered, this, [this] { ipc->runButton()->click(); });
        connect(stopAction, &QAction::triggered, this, [this] { ipc->cancelButton()->click(); });
        connect(openAction, &QAction::triggered, this, [this] {
            const QString file = QFileDialog::getOpenFileName(
                this, "Select local PCB file - NOT IMPORTED", QString(),
                "PCB candidate (*.asc *.ASC);;All files (*)");
            if (file.isEmpty()) return;
            chosenFile = QFileInfo(file).fileName();
            properties->setText(
                "Selected file: " + chosenFile + "\n"
                "Status: NOT IMPORTED\n"
                "Format: NOT VERIFIED\n"
                "Artwork: NOT AVAILABLE\n"
                "Solver: NOT PCB SI");
            validation->setText(
                "Local file selected only\n"
                "No file read, signature probe, parsing or SI performed.\n"
                "Import/geometry: NOT_RUN");
            log->appendPlainText("Open PCB: local file selected; no import performed.");
        });
        auto *poll = new QTimer(this);
        poll->setInterval(100);
        connect(poll, &QTimer::timeout, this, [this] { refresh(); });
        poll->start();
        refresh();
        resize(1240, 780);
    }

    Window *workerControls() const { return ipc; }
    ModelHandoffWindow *modelIdentity() const { return identity; }
    QAction *runControl() const { return runAction; }
    QAction *stopControl() const { return stopAction; }
    QAction *openControl() const { return openAction; }
    QAction *saveControl() const { return saveAction; }

protected:
    void closeEvent(QCloseEvent *event) override {
        // Preserve the existing nonblocking Worker close/kill discipline.
        if (!ipc->close()) {
            event->ignore();
            QTimer::singleShot(200, this, [this] { if (isVisible()) close(); });
            return;
        }
        QMainWindow::closeEvent(event);
    }
};

static int runPreviewSmoke(QApplication &app, PreviewWindow &window,
                           const QString &worker) {
    app.setQuitOnLastWindowClosed(false);
    QElapsedTimer clock;
    clock.start();
    int phase = 0;
    QString failure;
    auto *tick = new QTimer(&app);
    tick->setInterval(50);
    QObject::connect(tick, &QTimer::timeout, &app, [&] {
        auto *engine = window.workerControls();
        const auto status = engine->statusLabel()->text();
        const auto stopWith = [&](const QString &why) { failure = why; app.exit(1); };
        if (clock.elapsed() > 25000) { stopWith("P0 smoke timed out at phase " + QString::number(phase)); return; }
        if (phase == 0) {
            if (!window.isVisible() || window.saveControl()->isEnabled() ||
                !window.openControl()->isEnabled() || !window.runControl()->isEnabled() ||
                window.findChild<QLabel *>("artworkUnsupported")->text().contains("NOT IMPLEMENTED") ||
                !window.modelIdentity()) {
                stopWith("Preview widgets, unsupported labels or synthetic A/B fixtures missing"); return;
            }
            const QImage rendered = window.grab().toImage();
            if (rendered.isNull() || rendered.width() < 500 || rendered.height() < 350) {
                stopWith("Actual Qt window raster capture failed"); return;
            }
            window.runControl()->trigger(); phase = 1; return;
        }
        if (phase == 1 && engine->progressBar()->value() > 0 && window.stopControl()->isEnabled()) {
            window.stopControl()->trigger(); phase = 2; return;
        }
        if (phase == 2 && status.startsWith("Cancelled") && engine->runButton()->isEnabled() &&
            window.runControl()->isEnabled()) {
            window.runControl()->trigger(); phase = 3; return;
        }
        if (phase == 3 && status.startsWith("Synthetic calculation completed") &&
            engine->runButton()->isEnabled() && engine->progressBar()->value() == 100 &&
            engine->resultLabel()->text() == "Synthetic results: 100") {
            PreviewWindow failedWorker(QDir::tempPath() + "/boardsi-p0-missing-worker-" +
                                       QString::number(QCoreApplication::applicationPid()));
            failedWorker.workerControls()->runButton()->click();
            if (!failedWorker.workerControls()->errorLabel()->text().contains(
                    "Absolute executable Worker path required")) {
                stopWith("Unavailable Worker error path not displayed"); return;
            }
            window.modelIdentity()->runButton()->click();
            phase = 4; return;
        }
        if (phase == 4 && window.modelIdentity()->statusLabel()->text() ==
                              "Synthetic-only receipt accepted") {
            phase = 5; app.exit(0);
        }
    });
    tick->start();
    const int rc = app.exec();
    tick->stop();
    QJsonObject record{
        {"schema", "boardsi-preview-p0-smoke-v1"},
        {"pass", rc == 0 && phase == 5},
        {"stage", phase},
        {"failure", failure},
        {"mode", "SYNTHETIC / NOT PCB SI"},
        {"native_window_shown", true},
        {"native_backend", QGuiApplication::platformName()},
        {"host_os", QSysInfo::prettyProductName()},
        {"host_arch", QSysInfo::currentCpuArchitecture()},
        {"qt_version", qVersion()},
        {"worker_path", QFileInfo(worker).fileName()},
        {"source_sha", qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"elapsed_ms", clock.elapsed()},
        {"asserted", "window raster; unsupported features; run; progress; cancel; restart; completion; error; Canonical A/B receipt"}
    };
    QFile evidence("qt-preview-p0-results.json");
    if (!evidence.open(QIODevice::WriteOnly) ||
        evidence.write(QJsonDocument(record).toJson(QJsonDocument::Indented)) < 1) return 2;
    qInfo().noquote() << QJsonDocument(record).toJson(QJsonDocument::Compact);
    return rc == 0 && phase == 5 ? 0 : 1;
}

int main(int argc, char **argv) {
    QApplication app(argc, argv);
    const QStringList args = app.arguments();
    const bool testing = args.size() == 5 && args.at(1) == "--selftest";
    QString worker, first, second;
    if (testing) {
        worker = args.at(2); first = args.at(3); second = args.at(4);
    } else if (args.size() == 4) {
        worker = args.at(1); first = args.at(2); second = args.at(3);
    } else if (args.size() == 1) {
        const QString dir = QCoreApplication::applicationDirPath();
#ifdef Q_OS_WIN
        worker = dir + "/boardsi-worker.exe";
#else
        worker = dir + "/boardsi-worker";
#endif
        first = dir + "/qt-model-a.frame";
        second = dir + "/qt-model-b.frame";
    } else {
        qCritical("Usage: boardsi-preview-p0 [--selftest] [WORKER_ABSOLUTE FRAME_A FRAME_B]");
        return 2;
    }
    PreviewWindow window(worker, first, second);
    window.show();
    return testing ? runPreviewSmoke(app, window, worker) : app.exec();
}
