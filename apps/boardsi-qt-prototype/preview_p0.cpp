// BoardSI Developer Preview P0. UI only: no PCB geometry or physical SI solver.
// Reuse the independently regression-tested M0 QProcess IPC state machines verbatim.
// This single-translation-unit adapter is deliberately narrow for the first preview.
#define BOARDSI_RUNTIME_TEST
#include "main.cpp"
#define BOARDSI_MODEL_V2_NO_MAIN
#include "model_v2.cpp"
#include "synthetic_artwork.h"
#include "preview_import.h"

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
    SyntheticArtworkPanel *artwork = nullptr;
    PreviewImportPanel *importUI = nullptr;
    QString importResultStatus = "NOT IMPORTED";
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
        stateSummary->setText("Worker: " + status + " | PCB: " + importResultStatus + " | SI NOT_RUN");
        validation->setText(
            (chosenFile.isEmpty() ? QString("PCB: NOT IMPORTED") :
                "PCB candidate: " + chosenFile +
                (importResultStatus.startsWith("PARTIAL") ?
                    " (PARTIAL STRAIGHT-ROUTE IMPORT)" : " (NOT IMPORTED)")) +
            "\nSynthetic Artwork UI: DISPLAYED (fixture only)\nImported Artwork: " + importResultStatus + "\nPhysical SI: NOT_RUN\nWorker: " + status +
            (error.isEmpty() ? QString() : "\nWorker error: " + error.left(250)));
        resultSummary->setPlainText(
            "DEVELOPER PREVIEW P2-A - PARTIAL Import / NOT PCB SI\n\n"
            "Worker: independent Rust process\n" + status + "\n" +
            ipc->resultLabel()->text() +
            (error.isEmpty() ? QString() : "\nERROR: " + error) +
            "\n\nGenerated demonstration numbers are NOT physical PCB S-parameters.\n" +
            (importResultStatus.startsWith("PARTIAL") ?
                "Imported PCB: PARTIAL straight-route segments ONLY; no pad/via/stackup/connectivity. Physical SI NOT_RUN." :
                "Synthetic fixture only; no real PCB imported or physical SI analysis performed."));
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
    PreviewWindow(const QString &workerPath, const QString &firstFrame = QString(),
                  const QString &secondFrame = QString(), const QString &importBinary = QString()) {
        setObjectName("boardsiPreviewP0");
        setWindowTitle("BoardSI Developer Preview P2-A - PARTIAL IMPORT / NOT PCB SI");
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
            "BoardSI PREVIEW P2-A | Synthetic Demo / Imported Partial | NOT PCB SI | Developer build",
            root);
        header->setStyleSheet("font-weight: bold; padding: 8px; background: #2b3441; color: #ffffff;");
        vertical->addWidget(header);

        auto *horizontal = new QSplitter(Qt::Horizontal, root);
        auto *tree = new QTreeWidget(horizontal);
        tree->setHeaderLabel("Project / Workflow");
        tree->setMinimumWidth(185);
        auto *project = new QTreeWidgetItem(tree, {"Project"});
        for (const auto &name : {"PCB - select file (P2-A partial)", "Substrate - UNSUPPORTED",
                                 "Synthetic Layers - UI ONLY", "Synthetic Nets - UI ONLY"})
            new QTreeWidgetItem(project, {QString::fromLatin1(name)});
        auto *workflow = new QTreeWidgetItem(tree, {"Workflow"});
        for (const auto &name : {"Select PCB file", "Import - PADS V9.4-MILS partial",
                                 "Run synthetic Worker", "Inspect demo status"})
            new QTreeWidgetItem(workflow, {QString::fromLatin1(name)});
        tree->expandAll();

        auto *center = new QWidget(horizontal);
        auto *centerLayout = new QVBoxLayout(center);
        auto *centerTitle = new QLabel("Artwork Viewer - Synthetic or Imported Partial (never mixed)", center);
        centerTitle->setStyleSheet("font-weight: bold;");
        centerLayout->addWidget(centerTitle);
        importUI = new PreviewImportPanel(importBinary, center);
        artwork = importUI->synthetic();
        centerLayout->addWidget(importUI, 3);

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
            "Mode: Developer Preview P2-A\n"
            "Source: synthetic or imported partial\n"
            "PCB: NOT LOADED\n"
            "Stackup: NOT LOADED\n"
            "Solver: NOT PCB SI", propBox);
        properties->setWordWrap(true);
        properties->setTextFormat(Qt::PlainText); // Local filenames must never become rich text.
        propLayout->addWidget(properties);
        rightLayout->addWidget(propBox);
        auto *validBox = new QGroupBox("Validation", right);
        auto *validLayout = new QVBoxLayout(validBox);
        validation = new QLabel(
            "PCB import: NOT_RUN\n"
            "Synthetic Artwork UI: DISPLAYED\nImported Artwork: NOT_RUN\n"
            "Physical SI: NOT_RUN\n"
            "Worker: awaiting demo", validBox);
        validation->setWordWrap(true);
        validation->setTextFormat(Qt::PlainText);
        validLayout->addWidget(validation);
        rightLayout->addWidget(validBox);
        importUI->setStatusCallback([this](const QString &state) {
            importResultStatus = state;
            log->appendPlainText("PCB Import: " + state);
            if (state.startsWith("PARTIAL")) {
                properties->setText(
                    "Imported straight segments only; Canonical partial model.\n" + state +
                    "\nStackup, connectivity and SI: NOT_RUN");
            } else if (state.startsWith("FAILED") || state.startsWith("CANCELLED") ||
                       state.startsWith("RUNNING")) {
                properties->setText("PCB: NOT IMPORTED\n" + state +
                                    "\nStackup, connectivity and SI: NOT_RUN");
            }
        });
        artwork->setDetailsCallback([this](const QString &details) {
            const QString validity = importResultStatus.startsWith("PARTIAL") ?
                " (PARTIAL STRAIGHT-ROUTE IMPORT)" : " (NOT IMPORTED)";
            properties->setText(details +
                (chosenFile.isEmpty() ? QString() :
                 "\nSelected file: " + chosenFile + validity));
        });

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
                this, "Open local PADS V9.4-MILS file - partial artwork only", QString(),
                "PCB candidate (*.asc *.ASC);;All files (*)");
            if (file.isEmpty()) return;
            chosenFile = QFileInfo(file).fileName();
            if (!importUI->openPath(file))
                log->appendPlainText("Open PCB: unsupported PADS source, importer busy or local executable unavailable.");
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
    SyntheticArtworkPanel *artworkPanel() const { return artwork; }
    PreviewImportPanel *importPanel() const { return importUI; }

protected:
    void closeEvent(QCloseEvent *event) override {
        // Preserve the existing nonblocking Worker close/kill discipline.
        if (importUI && importUI->busy()) {
            importUI->stop(); event->ignore();
            QTimer::singleShot(200, this, [this] { if (isVisible()) close(); });
            return;
        }
        if ((identity && !identity->stopForPreviewClose()) || !ipc->close()) {
            event->ignore();
            QTimer::singleShot(200, this, [this] { if (isVisible()) close(); });
            return;
        }
        QMainWindow::closeEvent(event);
    }
};

static int runPreviewSmoke(QApplication &app, PreviewWindow &window,
                           const QString &worker, const QString &fixture) {
    app.setQuitOnLastWindowClosed(false);
    QElapsedTimer clock;
    clock.start();
    int phase = 0;
    const bool testImport = !fixture.isEmpty();
    QString failure;
    bool importedGeometryVerified = false;
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
                !window.artworkPanel() || window.artworkPanel()->primitiveCount() < 12 ||
                !window.modelIdentity()) {
                stopWith("Preview widgets, unsupported labels or synthetic A/B fixtures missing"); return;
            }
            const QImage rendered = window.grab().toImage();
            if (rendered.isNull() || rendered.width() < 500 || rendered.height() < 350) {
                stopWith("Actual Qt window raster capture failed"); return;
            }
            // Exercise the real P1 Qt scene and controls before the unchanged P0 Worker phases.
            auto *art = window.artworkPanel();
            const int initial = art->visiblePrimitives();
            if (initial != art->primitiveCount() ||
                !art->setLayerVisible("Top", false) ||
                art->visiblePrimitives() >= initial ||
                !art->setLayerVisible("Bottom", false) ||
                art->visiblePrimitives() != 0 ||
                !art->setLayerVisible("Top", true) ||
                !art->selectNet("DATA_P0") ||
                art->highlightedPrimitives() < 3 ||
                art->highlightedPrimitives() >= art->visiblePrimitives()) {
                stopWith("P1 synthetic layer visibility / net highlight failed"); return;
            }
            const qreal beforeZoom = art->canvasScale();
            auto *zoom = window.findChild<QPushButton *>("syntheticZoomIn");
            auto *fit = window.findChild<QPushButton *>("syntheticFit");
            if (!zoom || !fit) { stopWith("P1 zoom controls missing"); return; }
            zoom->click();
            if (art->canvasScale() <= beforeZoom) {
                stopWith("P1 zoom did not change scene transform"); return;
            }
            fit->click();
            const qreal afterFit = art->canvasScale();
            // Fit must restore the original scene transform, not merely exist.
            if (beforeZoom <= 0.0 || qAbs(afterFit - beforeZoom) >
                    qMax(qreal(0.02), beforeZoom * qreal(0.01))) {
                stopWith("P1 Fit did not restore the original scene transform"); return;
            }
            if (!art->selectNet("All nets") ||
                !art->setLayerVisible("Bottom", true) ||
                art->visiblePrimitives() != initial ||
                !window.findChild<QLabel *>("syntheticArtworkStatus")->text().contains("NOT IMPORTED")) {
                stopWith("P1 fixture reset / provenance label failed"); return;
            }
            if (testImport) {
                if (!window.importPanel()->openPath(fixture)) {
                    stopWith("P2-A local importer did not start"); return;
                }
                phase = 6; return;
            }
            window.runControl()->trigger(); phase = 1; return;
        }
        if (phase == 6 && window.importPanel()->importStatus().startsWith("PARTIAL")) {
            auto *panel = window.importPanel()->imported();
            auto *scene = panel->findChild<QGraphicsScene *>();
            if (!scene || scene->items().size() != 3 || panel->segmentCount() != 3 ||
                panel->visibleSegments() != 3) {
                stopWith("P2-A imported scene does not contain the expected three tracks"); return;
            }
            // Independent millimetre reference from the PUBLIC synthetic PADS fixture.
            // Verify the *rendered* Qt geometry, Y inversion, net and real pen width,
            // not just the Rust line-protocol strings.
            struct ExpectedLine { const char *net; qreal x0,y0,x1,y1,width; };
            const ExpectedLine reference[] = {
                {"NET_A",  2.54, -5.08,  7.62, -5.08, 0.254},
                {"NET_A",  7.62, -5.08,  7.62,-10.16, 0.254},
                {"NET_B", 25.40, -7.62, 38.10, -7.62, 0.3048}
            };
            const auto near = [](qreal x, qreal y) { return qAbs(x - y) < 0.000001; };
            for (const auto &want : reference) {
                bool found = false;
                for (QGraphicsItem *item : scene->items()) {
                    const auto *line = qgraphicsitem_cast<QGraphicsLineItem *>(item);
                    if (!line || line->data(0).toString() != QString::fromLatin1(want.net)) continue;
                    const QLineF actual = line->line();
                    if (near(actual.x1(), want.x0) && near(actual.y1(), want.y0) &&
                        near(actual.x2(), want.x1) && near(actual.y2(), want.y1) &&
                        near(line->pen().widthF(), want.width)) {
                        found = true;
                        break;
                    }
                }
                if (!found) {
                    stopWith("P2-A Qt scene geometry/net/width differs from independent reference");
                    return;
                }
            }
            if (!panel->selectNet("NET_A")) {
                stopWith("P2-A net selection unavailable"); return;
            }
            for (QGraphicsItem *item : scene->items()) {
                if (item->data(0).toString() == "NET_B" && !near(item->opacity(), 0.12)) {
                    stopWith("P2-A net highlight opacity mismatch"); return;
                }
            }
            auto *toggle = window.findChild<QPushButton *>("importedLayerToggle");
            auto *fit = window.findChild<QPushButton *>("importedFit");
            auto *zoom = window.findChild<QPushButton *>("importedZoomIn");
            if (!toggle || !fit || !zoom) {
                stopWith("P2-A layer/zoom controls missing"); return;
            }
            toggle->click(); // The initially selected copper layer is Top (1).
            if (panel->visibleSegments() != 1) {
                stopWith("P2-A layer 1 hide does not leave only layer 4 visible"); return;
            }
            toggle->click();
            if (panel->visibleSegments() != 3) {
                stopWith("P2-A layer visibility restore failed"); return;
            }
            fit->click(); // Establish baseline after the tab has become visible.
            const qreal fitScale = panel->scale();
            zoom->click();
            if (panel->scale() <= fitScale) {
                stopWith("P2-A imported zoom did not change the canvas"); return;
            }
            fit->click();
            if (fitScale <= 0 || qAbs(panel->scale() - fitScale) >
                                  qMax(qreal(0.02), fitScale * qreal(0.01))) {
                stopWith("P2-A imported Fit did not restore the canvas"); return;
            }
            importedGeometryVerified = true;
            // Regression: a failed replacement must not retain the previous board
            // under the misleading "NOTHING IMPORTED" status.
            if (window.importPanel()->openPath(fixture + ".missing-board") ||
                panel->segmentCount() != 0 || panel->visibleSegments() != 0 ||
                !window.importPanel()->importStatus().startsWith("FAILED")) {
                stopWith("P2-A stale geometry survived a failed import"); return;
            }
            if (!window.importPanel()->openPath(fixture)) {
                stopWith("P2-A re-import failed to start after a rejected file"); return;
            }
            phase = 7; return;
        }
        if (phase == 6 && window.importPanel()->importStatus().startsWith("FAILED")) {
            stopWith("P2-A initial parser returned failure"); return;
        }
        if (phase == 7 && window.importPanel()->importStatus().startsWith("PARTIAL")) {
            if (window.importPanel()->imported()->segmentCount() != 3) {
                stopWith("P2-A re-import after failure has incorrect geometry"); return;
            }
            window.runControl()->trigger(); phase = 1; return;
        }
        if (phase == 7 && window.importPanel()->importStatus().startsWith("FAILED")) {
            stopWith("P2-A re-import after failure returned FAILED"); return;
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
        {"asserted", "window raster; synthetic fixture/layers/net highlight/zoom; unsupported import; run; progress; cancel; restart; completion; error; Canonical A/B receipt"},
        {"p1_synthetic_artwork", phase >= 1},
        {"p2a_imported_artwork", testImport && rc == 0 && phase == 5},
        {"p2a_geometry_mm_width_y_net_verified", importedGeometryVerified},
        {"p1_scope", "synthetic illustrative geometry only; no PCB file imported"}
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
    const bool testing = (args.size() == 5 || args.size() == 7) && args.at(1) == "--selftest";
    QString worker, first, second, importer, fixture;
    if (testing) {
        worker = args.at(2); first = args.at(3); second = args.at(4);
        if (args.size() == 7) { importer = args.at(5); fixture = args.at(6); }
    } else if (args.size() == 4 || args.size() == 5) {
        worker = args.at(1); first = args.at(2); second = args.at(3);
        if (args.size() == 5) importer = args.at(4);
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
        qCritical("Usage: boardsi-preview-p0 [--selftest] WORKER FRAME_A FRAME_B [IMPORTER [FIXTURE]]");
        return 2;
    }
    PreviewWindow window(worker, first, second, importer);
    window.show();
    return testing ? runPreviewSmoke(app, window, worker, fixture) : app.exec();
}
