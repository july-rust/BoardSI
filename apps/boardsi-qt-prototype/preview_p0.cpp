// BoardSI Developer Preview P0. UI only: no PCB geometry or physical SI solver.
// Reuse the independently regression-tested M0 QProcess IPC state machines verbatim.
// This single-translation-unit adapter is deliberately narrow for the first preview.
#define BOARDSI_RUNTIME_TEST
#include "main.cpp"
#define BOARDSI_MODEL_V2_NO_MAIN
#include "model_v2.cpp"
#include "synthetic_artwork.h"
#include "preview_import.h"
#include "a019_gpu_benchmark.h"
#include "a019_interaction_validation.h"
#include "a020_explorer_validation.h"

#include <QAction>
#include <QCloseEvent>
#include <QCryptographicHash>
#include <QDir>
#include <QEventLoop>
#include <QFileDialog>
#include <QGroupBox>
#include <QGuiApplication>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QJsonArray>
#include <QElapsedTimer>
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
    QString syntheticDetails;
    QString importedDetails;

    static QWidget *unsupportedTab(const QString &message, QWidget *parent) {
        auto *page = new QWidget(parent);
        auto *layout = new QVBoxLayout(page);
        auto *label = new QLabel(message, page);
        label->setWordWrap(true);
        label->setAlignment(Qt::AlignCenter);
        layout->addWidget(label);
        return page;
    }

    // Properties always describe the visible artwork tab, not the other source.
    void refreshProperties() {
        if (!importUI || !properties) return;
        if (!importUI->showingImported()) {
            properties->setText(
                "Source: SYNTHETIC FIXTURE ONLY / NOT IMPORTED\n" +
                (syntheticDetails.isEmpty() ? QString("Illustrative PCB artwork only") : syntheticDetails) +
                "\nPhysical SI: NOT_RUN");
            return;
        }
        const QString candidate = chosenFile.isEmpty() ? QString("No PCB candidate selected") :
                                  QString("PCB candidate: ") + chosenFile;
        const bool loaded = importResultStatus.startsWith("PARTIAL") &&
                            ((importUI->imported()->segmentCount() + importUI->imported()->viaCount() +
                              importUI->imported()->padCount() + importUI->imported()->arcCount() +
                              importUI->imported()->outlineCount() +
                              importUI->imported()->circleCount() +
                              importUI->imported()->boundaryCount() +
                              importUI->imported()->pourEdgeCount()+
                              importUI->imported()->copperCount()) > 0);
        if (!loaded) {
            properties->setText("Source: IMPORTED VIEW / NOTHING IMPORTED\n" +
                                candidate + "\nImport status: " + importResultStatus +
                                "\nStackup, connectivity and SI: NOT_RUN");
            return;
        }
        properties->setText("Source: IMPORTED PARTIAL / PADS V9.4 MILS\n" +
                            candidate + "\n" + importedDetails +
                            "\nStackup, connectivity and SI: NOT_RUN");
    }

    void refresh() {
        openAction->setEnabled(!importUI->busy());
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
                    " (PARTIAL ARTWORK IMPORT)" : " (NOT IMPORTED)")) +
            "\nSynthetic Artwork UI: DISPLAYED (fixture only)\nImported Artwork: " + importResultStatus + "\nPhysical SI: NOT_RUN\nWorker: " + status +
            (error.isEmpty() ? QString() : "\nWorker error: " + error.left(250)));
        resultSummary->setPlainText(
            "DEVELOPER PREVIEW P2-L - PARTIAL Import / NOT PCB SI\n\n"
            "Worker: independent Rust process\n" + status + "\n" +
            ipc->resultLabel()->text() +
            (error.isEmpty() ? QString() : "\nERROR: " + error) +
            "\n\nGenerated demonstration numbers are NOT physical PCB S-parameters.\n" +
            (importResultStatus.startsWith("PARTIAL") ?
                "Imported PCB: PARTIAL accepted straight/arc routes, through vias, restricted SMD pads and restricted straight/arc/circular board artwork; other geometry/stackup/connectivity NOT IMPORTED. Physical SI NOT_RUN." :
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
        setWindowTitle("BoardSI Developer Preview P2-L - PARTIAL IMPORT / NOT PCB SI");
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
            "BoardSI PREVIEW P2-K | Layer/Net Explorer / Imported Partial | NOT PCB SI | Developer build",
            root);
        header->setStyleSheet("font-weight: bold; padding: 8px; background: #2b3441; color: #ffffff;");
        vertical->addWidget(header);

        auto *horizontal = new QSplitter(Qt::Horizontal, root);
        auto *tree = new QTreeWidget(horizontal);
        tree->setHeaderLabel("Project / Workflow");
        tree->setMinimumWidth(185);
        auto *project = new QTreeWidgetItem(tree, {"Project"});
        for (const auto &name : {"PCB - PADS V9.4 partial", "Substrate - UNSUPPORTED",
                                 "Imported Layers - PARTIAL", "Imported Nets - DECLARED ONLY"})
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
            "Mode: Developer Preview P2-K\n"
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
            refreshProperties();
        });
        artwork->setDetailsCallback([this](const QString &details) {
            syntheticDetails = details;
            if (!importUI->showingImported()) refreshProperties();
        });
        importUI->imported()->setDetailsCallback([this](const QString &details) {
            importedDetails = details;
            if (importUI->showingImported()) refreshProperties();
        });
        connect(importUI->artworkTabs(), &QTabWidget::currentChanged, this,
                [this](int) { refreshProperties(); });
        refreshProperties();

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
            openCandidate(file);
        });
        auto *poll = new QTimer(this);
        poll->setInterval(100);
        connect(poll, &QTimer::timeout, this, [this] { refresh(); });
        poll->start();
        refresh();
        resize(1240, 780);
    }

    // Production Open PCB and smoke checks share the same provenance path.
    bool openCandidate(const QString &file) {
        if (importUI->busy()) {
            log->appendPlainText("Open PCB: import already running; active filename retained.");
            return false;
        }
        chosenFile = QFileInfo(file).fileName();
        const bool started = importUI->openPath(file);
        if (!started)
            log->appendPlainText("Open PCB: unsupported PADS source or local executable unavailable.");
        refreshProperties();
        return started;
    }
    QString displayedProperties() const { return properties->text(); }
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
    bool importSummaryVerified = false;
    bool importFailureRecoveryVerified = false;
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
                if (!window.openCandidate(fixture)) {
                    stopWith("P2-A local importer did not start"); return;
                }
                // A second Open during the active QProcess must not alter the
                // accepted filename or the importer state.
                if (window.openCandidate(fixture + ".busy-rejected")) {
                    stopWith("P2-A accepted overlapping import requests"); return;
                }
                phase = 6; return;
            }
            window.runControl()->trigger(); phase = 1; return;
        }
        if (phase == 6 && window.importPanel()->importStatus().startsWith("PARTIAL")) {
            auto *panel = window.importPanel()->imported();
            if (!window.importPanel()->showingImported() ||
                !window.displayedProperties().contains("Source: IMPORTED PARTIAL") ||
                !window.displayedProperties().contains(QFileInfo(fixture).fileName())) {
                stopWith("P2-A imported tab Properties/provenance not synchronized"); return;
            }
            const QString details = window.displayedProperties();
            if (!details.contains("Declared layers: 4") ||
                !details.contains("Layers with straight traces: 2") ||
                !details.contains("Nets with straight traces: 2") ||
                !details.contains("Segments: 3")) {
                stopWith("P2-A imported properties counts do not reflect actual accepted traces"); return;
            }
            auto *layerSelector = window.findChild<QComboBox *>("importedLayerSelect");
            if (!layerSelector || !details.contains("Selected layer: Top")) {
                stopWith("P2-A selected layer missing from imported Properties"); return;
            }
            layerSelector->setCurrentIndex(1); // Declared inner layer has no accepted copper.
            if (!window.displayedProperties().contains("Selected layer: Inner 2")) {
                stopWith("P2-A imported layer selector did not refresh Properties"); return;
            }
            layerSelector->setCurrentIndex(0);
            if (!window.displayedProperties().contains("Selected layer: Top")) {
                stopWith("P2-A imported selected layer not restored"); return;
            }
            importSummaryVerified = true;
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
            if (!panel->selectNet("NET_A") ||
                !window.displayedProperties().contains("Selected net: NET_A")) {
                stopWith("P2-A net selection/Properties callback unavailable"); return;
            }
            window.importPanel()->artworkTabs()->setCurrentIndex(0);
            if (!window.artworkPanel()->selectNet("DATA_P0") ||
                !window.displayedProperties().contains("Source: SYNTHETIC") ||
                window.displayedProperties().contains("PCB candidate:") ||
                window.displayedProperties().contains("Source: IMPORTED PARTIAL")) {
                stopWith("P2-A synthetic Properties leaked imported provenance"); return;
            }
            window.importPanel()->artworkTabs()->setCurrentIndex(1);
            if (!window.displayedProperties().contains("Source: IMPORTED PARTIAL") ||
                !window.displayedProperties().contains("Selected net: NET_A")) {
                stopWith("P2-A imported Properties not restored after tab switch"); return;
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
            if (!window.displayedProperties().contains("Visible: 1")) {
                stopWith("P2-A imported layer change did not refresh Properties"); return;
            }
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
            if (window.openCandidate(fixture + ".missing-board") ||
                panel->segmentCount() != 0 || panel->visibleSegments() != 0 ||
                !window.importPanel()->importStatus().contains("FILE_NOT_FOUND") ||
                !window.displayedProperties().contains("NOTHING IMPORTED") ||
                window.displayedProperties().contains("Source: IMPORTED PARTIAL")) {
                stopWith("P2-A stale geometry or incorrect FILE_NOT_FOUND diagnostic"); return;
            }
            // A real but unrecognized source must fail closed too; it may not
            // resurrect the previous geometry or claim vendor PADS fidelity.
            const QString invalid = QFileInfo(fixture).absoluteDir().filePath("not_a_pcb.asc");
            if (!window.openCandidate(invalid)) {
                stopWith("P2-A invalid-header regression fixture unavailable"); return;
            }
            phase = 8; return;
        }
        if (phase == 6 && window.importPanel()->importStatus().startsWith("FAILED")) {
            stopWith("P2-A initial parser returned failure"); return;
        }
        if (phase == 8 && window.importPanel()->importStatus().startsWith("PARTIAL")) {
            stopWith("P2-A accepted invalid-header fixture as imported geometry"); return;
        }
        if (phase == 8 && window.importPanel()->importStatus().startsWith("FAILED")) {
            if (window.importPanel()->imported()->segmentCount() != 0 ||
                !window.displayedProperties().contains("NOTHING IMPORTED")) {
                stopWith("P2-A invalid header retained stale geometry"); return;
            }
            if (!window.openCandidate(fixture)) {
                stopWith("P2-A re-import after invalid header did not start"); return;
            }
            importFailureRecoveryVerified = true;
            phase = 7; return;
        }
        if (phase == 7 && window.importPanel()->importStatus().startsWith("PARTIAL")) {
            if (window.importPanel()->imported()->segmentCount() != 3 ||
                !window.displayedProperties().contains("Source: IMPORTED PARTIAL") ||
                !window.displayedProperties().contains(QFileInfo(fixture).fileName())) {
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
        {"p2a_import_summary_counts_verified", importSummaryVerified},
        {"p2a_import_failure_recovery_verified", importFailureRecoveryVerified},
        {"p2a_properties_source_isolation_verified", testImport && importedGeometryVerified && rc == 0 && phase == 5},
        {"p1_scope", "synthetic illustrative geometry only; no PCB file imported"}
    };
    QFile evidence("qt-preview-p0-results.json");
    if (!evidence.open(QIODevice::WriteOnly) ||
        evidence.write(QJsonDocument(record).toJson(QJsonDocument::Indented)) < 1) return 2;
    qInfo().noquote() << QJsonDocument(record).toJson(QJsonDocument::Compact);
    return rc == 0 && phase == 5 ? 0 : 1;
}




static int runRendererSmoke(PreviewWindow &window) {
    QElapsedTimer clock;
    clock.start();
    QEventLoop loop;
    auto *canvas = window.artworkPanel()->canvas();
    const bool validCanvas = canvas != nullptr;
    QTimer monitor;
    QObject::connect(&monitor, &QTimer::timeout, &loop, [&] {
        if (!validCanvas) { loop.quit(); return; }
        const QString status = canvas->rendererStatus();
        if (status.startsWith("CPU_") || status == "GPU_OPENGL_ACTIVE")
            loop.quit();
    });
    // The GPU path is only verified after a real frameSwapped signal.
    // Allow the renderer's 3000 ms no-frame fallback before time-out.
    QTimer::singleShot(3900, &loop, &QEventLoop::quit);
    monitor.start(20);
    loop.exec();
    monitor.stop();
    const QString status = validCanvas ? canvas->rendererStatus() : "MISSING_CANVAS";
    const QString requested = qEnvironmentVariable("BOARDSI_ARTWORK_RENDERER")
                                 .trimmed().toLower();
    const bool forcedCpu = requested == "cpu";
    bool actualGlViewport = false;
#ifdef BOARDSI_OPENGL_VIEWPORT
    if (validCanvas)
        actualGlViewport = qobject_cast<QOpenGLWidget *>(canvas->viewport()) != nullptr;
#endif
    const bool policy = validCanvas && canvas->rendererPolicy() == "GPU_PREFERRED" &&
                        window.importPanel()->imported()->rendererPolicy() == "GPU_PREFERRED";
    const bool softwareClassifier =
        SyntheticArtworkCanvas::isSoftwareRendererLabel("Apple Software Renderer") &&
        SyntheticArtworkCanvas::isSoftwareRendererLabel("llvmpipe (LLVM 15)") &&
        SyntheticArtworkCanvas::isSoftwareRendererLabel("SwiftShader Device") &&
        SyntheticArtworkCanvas::isSoftwareRendererLabel("Microsoft Basic Render Driver") &&
        !SyntheticArtworkCanvas::isSoftwareRendererLabel("Apple M3") &&
        !SyntheticArtworkCanvas::isSoftwareRendererLabel("Mesa Intel(R) Graphics") &&
        !SyntheticArtworkCanvas::isSoftwareRendererLabel("NVIDIA GeForce RTX 4070") &&
        SyntheticArtworkCanvas::isSoftwareRendererLabel("OpenSWR") &&
        !SyntheticArtworkCanvas::isKnownHardwareRendererLabel("OpenSWR") &&
        !SyntheticArtworkCanvas::isKnownHardwareRendererLabel("Unknown adapter") &&
        !SyntheticArtworkCanvas::isKnownHardwareRendererLabel("Apple Software Renderer") &&
        SyntheticArtworkCanvas::isKnownHardwareRendererLabel("Apple M3") &&
        SyntheticArtworkCanvas::isKnownHardwareRendererLabel("NVIDIA GeForce RTX 4070") &&
        SyntheticArtworkCanvas::isKnownHardwareRendererLabel("Mesa Intel(R) Graphics");
    const bool mode = forcedCpu ? (status == "CPU_FORCED" && !actualGlViewport) :
        ((status.startsWith("CPU_FALLBACK_") && !actualGlViewport) ||
         (status == "GPU_OPENGL_ACTIVE" && actualGlViewport));
    // Force first display of the empty Imported tab; the renderer update
    // must not turn an empty view into a false "IMPORTED PARTIAL" state.
    auto *imported = window.importPanel()->imported();
    auto *tabs = window.importPanel()->artworkTabs();
    tabs->setCurrentWidget(imported);
    QEventLoop emptyLoop;
    QTimer::singleShot(120, &emptyLoop, &QEventLoop::quit);
    emptyLoop.exec();
    const bool emptyImportStatus = imported->summaryText().contains("No imported PCB") &&
        !imported->summaryText().contains("IMPORTED PARTIAL") &&
        imported->padCount() == 0 && imported->segmentCount() == 0 &&
        imported->viaCount() == 0;
    tabs->setCurrentIndex(0);
    // Pending contexts are NOT GPU results, regardless of GL availability.
    const bool completed = status == "GPU_OPENGL_ACTIVE" ||
                           status.startsWith("CPU_");
    const bool passed = policy && mode && completed && softwareClassifier && emptyImportStatus;
    QJsonObject record{
        {"schema", "boardsi-preview-renderer-policy-v1"},
        {"pass", passed},
        {"default_renderer_policy", "GPU_PREFERRED"},
        {"renderer_setting", requested.isEmpty() ? QString("auto") : requested},
        {"synthetic_renderer_status", status},
        {"imported_renderer_status", window.importPanel()->imported()->rendererStatus()},
        {"actual_gl_viewport", actualGlViewport},
        {"gpu_frame_confirmed", status == "GPU_OPENGL_ACTIVE"},
        {"software_driver_classification_verified", softwareClassifier},
        {"empty_imported_view_preserved", emptyImportStatus},
        {"gpu_adapter", validCanvas ? canvas->rendererName() : QString()},
        {"source_sha", qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"native_backend", QGuiApplication::platformName()},
        {"host_arch", QSysInfo::currentCpuArchitecture()},
        {"elapsed_ms", clock.elapsed()},
        {"scope", "GPU-first Qt OpenGL viewport policy only, no GPU throughput or SI claims"}
    };
    QFile evidence("qt-preview-renderer-results.json");
    if (!evidence.open(QIODevice::WriteOnly) ||
        evidence.write(QJsonDocument(record).toJson(QJsonDocument::Indented)) < 1) return 2;
    qInfo().noquote() << QJsonDocument(record).toJson(QJsonDocument::Compact);
    return passed ? 0 : 1;
}


// A_016 native GUI end-to-end: real Rust V6 transport -> CPU + optional RHI.
// The outline/cutout is mechanical artwork, NEVER electrical connectivity.
static int runOutlineSmoke(PreviewWindow &window, const QString &fixture) {
    QElapsedTimer clock;
    clock.start();
    bool passed = false;
    QString failure;
    bool rhiChecked = false, rhiExact = false;
    QEventLoop loop;
    if (!window.openCandidate(fixture)) {
        failure = "A_016 fixture/importer unavailable";
    } else {
        QTimer tick;
        QObject::connect(&tick, &QTimer::timeout, &loop, [&] {
            const QString status = window.importPanel()->importStatus();
            if (status.startsWith("FAILED") || status.startsWith("CANCELLED")) {
                failure = "A_016 import failed: " + status;
                loop.quit();
                return;
            }
            if (!status.startsWith("PARTIAL")) return;
            auto *panel = window.importPanel()->imported();
            auto *scene = panel->findChild<QGraphicsScene *>();
            int boardEdges = 0, cutoutEdges = 0;
            bool outerExact = false, cutExact = false;
            const auto near = [](qreal a, qreal b) { return qAbs(a - b) < 0.000001; };
            if (scene) for (QGraphicsItem *item : scene->items()) {
                auto *line = qgraphicsitem_cast<QGraphicsLineItem *>(item);
                if (!line) continue;
                const QString kind = item->data(1).toString();
                const QLineF p = line->line();
                if (kind == "Board Outline (not copper)") {
                    ++boardEdges;
                    if (near(p.x1(), 2.54) && near(p.y1(), -5.08) &&
                        near(p.x2(), 12.70) && near(p.y2(), -5.08))
                        outerExact = true;
                }
                if (kind == "Board Cutout (not copper)") {
                    ++cutoutEdges;
                    if (near(p.x1(), 5.08) && near(p.y1(), -7.62) &&
                        near(p.x2(), 7.62) && near(p.y2(), -7.62))
                        cutExact = true;
                }
            }
            const bool counts = panel->outlineCount() == 8 &&
                panel->segmentCount() == 1 && boardEdges == 4 && cutoutEdges == 4 &&
                status.contains("8 outline edges") &&
                window.displayedProperties().contains("Board outline/cutout straight edges: 8");
            // Net/layer filtering must not turn mechanical boundaries into copper.
            const bool netSelected = panel->selectNet("NET_A");
            auto *toggle = window.findChild<QPushButton *>("importedLayerToggle");
            bool independent = netSelected && toggle;
            if (independent) {
                toggle->click();
                for (QGraphicsItem *item : scene->items()) {
                    if (item->data(1).toString().startsWith("Board "))
                        independent &= item->isVisible();
                }
                toggle->click();
            }
#ifdef BOARDSI_RHI_ARTWORK
            if (auto *rhi = panel->rhiViewForTest();
                rhi && panel->rhiCanvasActiveForTest()) {
                rhiChecked = true;
                auto *item = rhi->itemForTest();
                rhiExact = item->outlineCountForTest() == 8 &&
                    item->dispatchOutlinePickForTest(0) &&
                    window.displayedProperties().contains("Board Outline") &&
                    item->containsExactOutlineForTest(false,
                        2540000, 5080000, 12700000, 5080000) &&
                    item->containsExactOutlineForTest(true,
                        5080000, 7620000, 7620000, 7620000) &&
                    item->outlineFitBoundsForTest();
            }
#endif
            passed = counts && outerExact && cutExact && independent &&
                (!rhiChecked || rhiExact);
            if (!passed) failure = "A_016 outline geometry/count/RHI or filter mismatch";
            loop.quit();
        });
        QTimer::singleShot(15000, &loop, &QEventLoop::quit);
        tick.start(20);
        loop.exec();
        tick.stop();
        if (!passed && failure.isEmpty()) failure = "A_016 outline smoke timeout";
    }
    const QJsonObject result{
        {"schema", "boardsi-preview-p2g-outline-smoke-v1"},
        {"pass", passed}, {"failure", failure}, {"elapsed_ms", clock.elapsed()},
        {"source_sha", qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"native_backend", QGuiApplication::platformName()},
        {"host_arch", QSysInfo::currentCpuArchitecture()},
        {"rhi_source_checked", rhiChecked},
        {"rhi_exact_boundary_points", rhiExact},
        {"scope", "public synthetic closed straight BOARD/CUTOUT; source and SI NOT_VERIFIED"}
    };
    QFile receipt("qt-preview-p2g-outline-results.json");
    if (!receipt.open(QIODevice::WriteOnly) ||
        receipt.write(QJsonDocument(result).toJson(QJsonDocument::Indented)) < 1) return 2;
    qInfo().noquote() << QJsonDocument(result).toJson(QJsonDocument::Compact);
    return passed ? 0 : 1;
}


// P2-H: independent native import -> V7 -> Qt CPU/RHI circle acceptance.
static int runCircleSmoke(PreviewWindow &window, const QString &fixture) {
    QElapsedTimer clock; clock.start();
    bool passed=false, rhiChecked=false, rhiExact=false;
    QString failure;
    // Real strict decoder must refuse truncated V7 trailers before indexing.
    const QByteArray transportPrefix=
        "BOARDSI_P2H_PARTIAL_V7|V9.4-MILS|1\n"
        "L|1|TOP\n"
        "C|OUTER|BOARD|0|0|2540\n";
    const bool malformedTransportRejected=
        PreviewImportPanel::validTransportForTest(
            transportPrefix+"END|0|0|0|0|1|0|1|CANONICAL_PARTIAL|SI_NOT_RUN\n") &&
        !PreviewImportPanel::validTransportForTest(
            transportPrefix+"END|0|0|0|0\n");
    bool overflowCircleRejected=false;
    QEventLoop loop;
    if (!window.openCandidate(fixture)) {
        failure="A_017 fixture/importer unavailable";
    } else {
        QTimer tick;
        QObject::connect(&tick,&QTimer::timeout,&loop,[&] {
            const QString status=window.importPanel()->importStatus();
            if(status.startsWith("FAILED") || status.startsWith("CANCELLED")) {
                failure="A_017 import failed: "+status;loop.quit();return;
            }
            if(!status.startsWith("PARTIAL")) return;
            auto *panel=window.importPanel()->imported();
            auto *scene=panel->findChild<QGraphicsScene *>();
            int outer=0,cut=0; bool outerExact=false,cutExact=false;
            if(scene) for(QGraphicsItem *obj:scene->items()) {
                auto *ellipse=qgraphicsitem_cast<QGraphicsEllipseItem *>(obj);
                if(!ellipse) continue;
                const auto kind=obj->data(1).toString();
                const QRectF box=ellipse->rect();
                const auto nearPoint=[](qreal x,qreal y) {
                    return qAbs(x-y)<0.000001;
                };
                if(kind.contains("Board Outline MAIN_CIRCLE")) {
                    ++outer;
                    outerExact=nearPoint(box.x(),2.54) &&
                               nearPoint(box.y(),-10.16) &&
                               nearPoint(box.width(),10.16) &&
                               nearPoint(box.height(),10.16);
                }
                if(kind.contains("Board Cutout INNER_CIRCLE")) {
                    ++cut;
                    cutExact=nearPoint(box.x(),6.35) &&
                             nearPoint(box.y(),-6.35) &&
                             nearPoint(box.width(),2.54) &&
                             nearPoint(box.height(),2.54);
                }
            }
            ImportedCircle overflowCircle;
            overflowCircle.name="BAD_OVERFLOW"; overflowCircle.radius_nm=9223372036854775680LL;
            overflowCircle.cx_nm=9223372036854775680LL;
            overflowCircle.cy_nm=9223372036854775680LL;
            overflowCircleRejected=!panel->load(1,0,{}, {}, {}, {}, {},
                                                {overflowCircle}) &&
                                   panel->circleCount()==2;
            const bool counts=panel->circleCount()==2 && panel->outlineCount()==0 &&
                panel->segmentCount()==0 && outer==1 && cut==1 &&
                status.contains("2 circles") &&
                window.displayedProperties().contains("analytical circles: 2");
            // Mechanical artwork cannot disappear with no nets or hidden copper.
            auto *toggle=window.findChild<QPushButton *>("importedLayerToggle");
            bool independent=(toggle!=nullptr);
            if(independent) {
                toggle->click();
                for(auto *obj:scene->items())
                    if(obj->data(1).toString().startsWith("Board "))
                        independent &= obj->isVisible();
                toggle->click();
            }
#ifdef BOARDSI_RHI_ARTWORK
            if(auto *rhi=panel->rhiViewForTest();
               rhi && panel->rhiCanvasActiveForTest()) {
                rhiChecked=true;
                auto *item=rhi->itemForTest();
                rhiExact=item->circleCountForTest()==2 &&
                         item->circleFitBoundsForTest() &&
                         item->dispatchCirclePickForTest(0) &&
                         window.displayedProperties().contains("MAIN_CIRCLE");
            }
#endif
            passed=counts && outerExact && cutExact && independent &&
                   malformedTransportRejected && overflowCircleRejected &&
                   (!rhiChecked || rhiExact);
            if(!passed) failure="A_017 circle geometry/count/RHI/pick mismatch";
            loop.quit();
        });
        QTimer::singleShot(15000,&loop,&QEventLoop::quit);
        tick.start(20);loop.exec();tick.stop();
        if(!passed && failure.isEmpty()) failure="A_017 circle smoke timeout";
    }
    const QJsonObject result{
        {"schema","boardsi-preview-p2h-circle-smoke-v1"},
        {"pass",passed},{"failure",failure},{"elapsed_ms",clock.elapsed()},
        {"source_sha",qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"native_backend",QGuiApplication::platformName()},
        {"host_arch",QSysInfo::currentCpuArchitecture()},
        {"rhi_source_checked",rhiChecked},{"rhi_circle_fit_and_pick",rhiExact},
        {"v7_truncated_end_rejected",malformedTransportRejected},
        {"v7_overflow_circle_rejected",overflowCircleRejected},
        {"scope","public synthetic V9.4 even-grid circular BOARD and BRDCIR; SI NOT_RUN"}
    };
    QFile receipt("qt-preview-p2h-circle-results.json");
    if(!receipt.open(QIODevice::WriteOnly) ||
       receipt.write(QJsonDocument(result).toJson(QJsonDocument::Indented))<1) return 2;
    qInfo().noquote()<<QJsonDocument(result).toJson(QJsonDocument::Compact);
    return passed?0:1;
}

// A_018 P2-I: actual V8 Import -> CPU/RHI analytical mechanical quarter-arcs.
static int runMixedOutlineSmoke(PreviewWindow &window, const QString &fixture) {
    QElapsedTimer clock; clock.start();
    const QByteArray malformed=
        "BOARDSI_P2I_PARTIAL_V8|V9.4-MILS|1\n"
        "L|1|TOP\n"
        "B|OUTER|TEST|0|L|0|0|2540|0\n"
        "B|OUTER|TEST|1|L|2540|0|2540|2540\n"
        "B|OUTER|TEST|2|L|2540|2540|0|0\n"
        "END|0|0|0|0\n";
    const bool truncatedV8Rejected=
        !PreviewImportPanel::validTransportForTest(malformed);
    bool passed=false,rhiChecked=false,rhiOk=false;
    bool pathOk=true, sourceCountOk=false, statusOk=false, propertiesOk=false,
         visibilityOk=false, scenePresent=false;
    int observedArcs=0,observedLines=0;
    QString failure;
    QEventLoop loop;
    if(!window.openCandidate(fixture)) {
        failure="A_018 public mixed-outline fixture unavailable";
    } else {
        QTimer tick;
        QObject::connect(&tick,&QTimer::timeout,&loop,[&] {
            const QString status=window.importPanel()->importStatus();
            if(status.startsWith("FAILED") || status.startsWith("CANCELLED")) {
                failure="A_018 import failed: "+status;loop.quit();return;
            }
            if(!status.startsWith("PARTIAL")) return;
            auto *panel=window.importPanel()->imported();
            auto *scene=panel->findChild<QGraphicsScene *>();
            int arcs=0,lines=0;
            bool coordinatesOk=true;
            if(scene) {
                for(QGraphicsItem *obj:scene->items()) {
                    auto *path=qgraphicsitem_cast<QGraphicsPathItem *>(obj);
                    if(!path || !(obj->data(1).toString().contains("MAIN_MIX") ||
                                obj->data(1).toString().contains("HOLE_MIX"))) continue;
                    const QString kind=obj->data(1).toString();
                    if(kind.contains("Arc")) {
                        ++arcs;
                        coordinatesOk &= path->path().elementCount()>2;
                    } else if(kind.contains("Line")) {
                        ++lines;
                        coordinatesOk &= path->path().elementCount()==2;
                    }
                }
            }
            observedArcs=arcs;observedLines=lines;pathOk=coordinatesOk;
            scenePresent=scene!=nullptr;
            sourceCountOk=panel->boundaryCount()==16;
            statusOk=status.contains("16 mixed boundary parts");
            propertiesOk=window.displayedProperties().contains("mixed segments: 16");
            const bool counts=scenePresent && sourceCountOk && arcs==8 &&
                lines==8 && pathOk && statusOk && propertiesOk;
            auto *toggle=window.findChild<QPushButton *>("importedLayerToggle");
            bool mechanicalVisible=toggle!=nullptr;
            if(toggle && scene) {
                toggle->click();
                for(auto *obj:scene->items()) {
                    if(obj->data(1).toString().contains("MAIN_MIX") ||
                       obj->data(1).toString().contains("HOLE_MIX"))
                        mechanicalVisible &= obj->isVisible();
                }
                toggle->click();
            }
            visibilityOk=mechanicalVisible;
#ifdef BOARDSI_RHI_ARTWORK
            if(auto *rhi=panel->rhiViewForTest();
               rhi && panel->rhiCanvasActiveForTest()) {
                rhiChecked=true;
                auto *item=rhi->itemForTest();
                rhiOk=item->boundaryCountForTest()==16 &&
                    item->boundaryFitBoundsForTest() &&
                    item->dispatchBoundaryPickForTest(1) &&
                    window.displayedProperties().contains("MAIN_MIX");
            }
#endif
            passed=counts && mechanicalVisible && truncatedV8Rejected &&
                   (!rhiChecked || rhiOk);
            if(!passed) failure="A_018 V8/CPU/RHI geometry, Fit or picking mismatch";
            loop.quit();
        });
        QTimer::singleShot(15000,&loop,&QEventLoop::quit);
        tick.start(20);loop.exec();tick.stop();
        if(!passed && failure.isEmpty()) failure="A_018 mixed outline smoke timeout";
    }
    const QJsonObject result{
        {"schema","boardsi-preview-p2i-mixed-outline-smoke-v1"},
        {"pass",passed},{"failure",failure},{"elapsed_ms",clock.elapsed()},
        {"source_sha",qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"host_arch",QSysInfo::currentCpuArchitecture()},
        {"native_backend",QGuiApplication::platformName()},
        {"rhi_checked",rhiChecked},{"rhi_fit_pick",rhiOk},
        {"source_count_ok",sourceCountOk},{"status_ok",statusOk},
        {"properties_ok",propertiesOk},{"scene_present",scenePresent},
        {"cpu_path_ok",pathOk},{"mechanical_visible",visibilityOk},
        {"cpu_arc_count",observedArcs},{"cpu_line_count",observedLines},
        {"truncated_v8_rejected",truncatedV8Rejected},
        {"scope","PADS V9.4 mixed quarter-arc BOARD and BRDCLS cutout; SI NOT_RUN"}
    };
    QFile receipt("qt-preview-p2i-mixed-outline-results.json");
    if(!receipt.open(QIODevice::WriteOnly) ||
       receipt.write(QJsonDocument(result).toJson(QJsonDocument::Indented))<1) return 2;
    qInfo().noquote()<<QJsonDocument(result).toJson(QJsonDocument::Compact);
    return passed?0:1;
}

// A_015 end-to-end imported PADS V9.4 arcs: Worker -> V5 -> CPU scene.
static int runArcSmoke(PreviewWindow &window, const QString &fixture) {
    QElapsedTimer clock; clock.start();
    bool passed=false; QString failure;
    bool rhiChecked=false, rhiSweepOk=false, rhiPickOk=false;
    QEventLoop loop;
    if(!window.openCandidate(fixture)) failure="A_015 public arc fixture unavailable";
    else {
        QTimer tick;
        QObject::connect(&tick,&QTimer::timeout,&loop,[&] {
            const QString status=window.importPanel()->importStatus();
            if(status.startsWith("FAILED") || status.startsWith("CANCELLED")) {
                failure="A_015 import failed: "+status; loop.quit(); return;
            }
            if(!status.startsWith("PARTIAL"))return;
            const bool receiptHasArcs=status.contains("2 route arcs");
            auto *panel=window.importPanel()->imported();
            bool ccw=false,cw=false;
            auto *scene=panel->findChild<QGraphicsScene *>();
            if(scene) for(QGraphicsItem *item:scene->items()){
                auto *path=qgraphicsitem_cast<QGraphicsPathItem *>(item);
                if(!path)continue;
                const QString p=path->data(1).toString();
                if(p.contains("Arc trace") && p.contains("CCW") &&
                   p.contains("centre (2.540000, 2.540000)"))ccw=true;
                if(p.contains("Arc trace") && p.contains("CW") &&
                   !p.contains("CCW"))cw=true;
            }
            const bool selected=panel->selectArc(0);
            const bool props=window.displayedProperties().contains("Arc trace") &&
                             window.displayedProperties().contains("SI NOT_RUN");
#ifdef BOARDSI_RHI_ARTWORK
            if(auto *rhi=panel->rhiViewForTest();
               rhi && panel->rhiCanvasActiveForTest()) {
                rhiChecked=true;
                auto *item=rhi->itemForTest();
                rhiSweepOk=item->arcSweepForTest(0,false) &&
                            item->arcSweepForTest(1,true);
                // This dispatch exercises the RHI item's real event hit-test
                // and onPicked callback rather than selecting the hidden scene.
                rhiPickOk=item->dispatchArcPickForTest(0) &&
                          window.displayedProperties().contains("Arc trace");
            }
#endif
            passed=panel->arcCount()==2 && panel->segmentCount()==1 &&
                   receiptHasArcs && ccw && cw && selected && props &&
                   (!rhiChecked || (rhiSweepOk && rhiPickOk));
            if(!passed)failure="A_015 geometry/count/provenance/selection mismatch";
            loop.quit();
        });
        QTimer::singleShot(15000,&loop,&QEventLoop::quit);
        tick.start(20);loop.exec();tick.stop();
        if(!passed && failure.isEmpty())failure="A_015 arc smoke timed out";
    }
    QJsonObject report{{"schema","boardsi-preview-p2f-arc-smoke-v1"},
       {"pass",passed},{"failure",failure},{"elapsed_ms",clock.elapsed()},
       {"source_sha",qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
       {"native_backend",QGuiApplication::platformName()},
       {"host_arch",QSysInfo::currentCpuArchitecture()},
       {"rhi_arc_gate_checked",rhiChecked},
       {"rhi_arc_sweep_pass",rhiSweepOk},
       {"rhi_arc_pick_dispatch_pass",rhiPickOk},
       {"scope","public synthetic exact route arcs; connectivity NOT_VERIFIED; SI NOT_RUN"}};
    QFile receipt("qt-preview-p2f-arc-results.json");
    if(!receipt.open(QIODevice::WriteOnly) ||
       receipt.write(QJsonDocument(report).toJson(QJsonDocument::Indented))<1)return 2;
    qInfo().noquote()<<QJsonDocument(report).toJson(QJsonDocument::Compact);
    return passed?0:1;
}

static int runPadSmoke(PreviewWindow &window, const QString &fixture) {
    QElapsedTimer clock;
    clock.start();
    QString failure;
    bool passed = false;
    QEventLoop loop;
    if (!window.openCandidate(fixture)) {
        failure = "P2-C synthetic component pad fixture unavailable";
    } else {
        QTimer tick;
        QObject::connect(&tick, &QTimer::timeout, &loop, [&] {
            const QString status = window.importPanel()->importStatus();
            if (status.startsWith("FAILED") || status.startsWith("CANCELLED")) {
                failure = "P2-C import failed: " + status;
                loop.quit();
            } else if (status.startsWith("PARTIAL")) {
                auto *panel = window.importPanel()->imported();
                auto *scene = panel->findChild<QGraphicsScene *>();
                const auto near = [](qreal a, qreal b) {
                    return qAbs(a-b) < 0.000001;
                };
                bool roundOK = false, squareOK = false;
                if (scene) for (QGraphicsItem *item : scene->items()) {
                    const QString details = item->data(1).toString();
                    if (details.contains("Pad U1.1")) {
                        auto *circle = qgraphicsitem_cast<QGraphicsEllipseItem *>(item);
                        if (circle) {
                            const QRectF actual = circle->rect();
                            roundOK = near(actual.center().x(), 5.08) &&
                                      near(actual.center().y(), -7.62) &&
                                      near(actual.width(), 1.016) && near(actual.height(), 1.016);
                        }
                    } else if (details.contains("Pad U1.2")) {
                        auto *rect = qgraphicsitem_cast<QGraphicsRectItem *>(item);
                        if (rect) {
                            const QRectF actual = rect->rect();
                            squareOK = near(actual.center().x(), 5.08) &&
                                       near(actual.center().y(), -10.16) &&
                                       near(actual.width(), 0.762) && near(actual.height(), 0.762);
                        }
                    }
                }
                const bool selected = panel->selectPad(1);
                const bool props = window.displayedProperties().contains("Pad U1.2") &&
                                   window.displayedProperties().contains("net NOT_VERIFIED") &&
                                   window.displayedProperties().contains("Component SMD pads: 2");
                const bool count = panel->padCount() == 2 && panel->segmentCount() == 1;
                passed = count && roundOK && squareOK && selected && props;
                if (!passed) failure = "P2-C scene geometry/selection/provenance mismatch";
                loop.quit();
            }
        });
        QTimer::singleShot(15000, &loop, &QEventLoop::quit);
        tick.start(20);
        loop.exec();
        tick.stop();
        if (!passed && failure.isEmpty()) failure = "P2-C smoke timed out";
    }
    QJsonObject report{
        {"schema", "boardsi-preview-p2c-pad-smoke-v1"},
        {"pass", passed}, {"failure", failure}, {"elapsed_ms", clock.elapsed()},
        {"source_sha", qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"native_backend", QGuiApplication::platformName()},
        {"host_arch", QSysInfo::currentCpuArchitecture()},
        {"scope", "public synthetic top-only SMD round/square; net NOT_VERIFIED; SI NOT_RUN"}
    };
    QFile receipt("qt-preview-p2c-pad-results.json");
    if (!receipt.open(QIODevice::WriteOnly) ||
        receipt.write(QJsonDocument(report).toJson(QJsonDocument::Indented)) < 1) return 2;
    qInfo().noquote() << QJsonDocument(report).toJson(QJsonDocument::Compact);
    return passed ? 0 : 1;
}


static int runRectSmoke(PreviewWindow &window, const QString &fixture) {
    QElapsedTimer clock;
    clock.start();
    bool passed = false;
    QString failure;
    QEventLoop loop;
    if (!window.openCandidate(fixture)) {
        failure = "P2-D RF synthetic fixture unavailable";
    } else {
        QTimer tick;
        QObject::connect(&tick, &QTimer::timeout, &loop, [&] {
            const QString status = window.importPanel()->importStatus();
            if (status.startsWith("FAILED") || status.startsWith("CANCELLED")) {
                failure = "RF import failed: " + status;
                loop.quit();
            } else if (status.startsWith("PARTIAL")) {
                auto *panel = window.importPanel()->imported();
                auto *scene = panel->findChild<QGraphicsScene *>();
                const auto near = [](qreal a, qreal b) { return qAbs(a - b) < 0.000001; };
                struct ExpectedPad { const char *label; qreal x, y, w, h; };
                // Independent axis-aligned expectations in mm (not from the importer).
                const ExpectedPad expected[] = {
                    {"Pad U0.1",  5.08, -7.62, 2.032, 0.762},
                    {"Pad U0.2",  7.62, -7.62, 0.762, 2.032},
                    {"Pad U1.1", 10.16, -7.62, 0.762, 2.032},
                    {"Pad U1.2", 10.16,-10.16, 2.032, 0.762},
                    {"Pad U2.1", 15.24, -7.62, 2.032, 0.762},
                    {"Pad U2.2", 12.70, -7.62, 0.762, 2.032},
                    {"Pad U3.1", 20.32, -7.62, 0.762, 2.032},
                    {"Pad U3.2", 20.32, -5.08, 2.032, 0.762}
                };
                bool geometry = scene && scene->items().size() == 8;
                if (geometry) for (const auto &want : expected) {
                    bool found = false;
                    for (QGraphicsItem *item : scene->items()) {
                        auto *rect = qgraphicsitem_cast<QGraphicsRectItem *>(item);
                        if (!rect || !item->data(1).toString().contains(want.label)) continue;
                        const QRectF actual = rect->rect();
                        if (near(actual.center().x(), want.x) &&
                            near(actual.center().y(), want.y) &&
                            near(actual.width(), want.w) &&
                            near(actual.height(), want.h) &&
                            item->data(1).toString().contains("net NOT_VERIFIED")) found = true;
                    }
                    if (!found) { geometry = false; break; }
                }
                const bool counts = panel->padCount() == 8 &&
                                    panel->segmentCount() == 0 && panel->viaCount() == 0;
                const bool selected = panel->selectPad(0) &&
                    window.displayedProperties().contains("Pad U0.1") &&
                    window.displayedProperties().contains("Component SMD pads: 8") &&
                    window.displayedProperties().contains("Source: IMPORTED PARTIAL") &&
                    !window.displayedProperties().contains("NOTHING IMPORTED");
                auto *toggle = window.findChild<QPushButton *>("importedLayerToggle");
                bool visibility = false;
                if (toggle && scene) {
                    toggle->click();
                    bool allHidden = true;
                    for (QGraphicsItem *item : scene->items()) allHidden &= !item->isVisible();
                    toggle->click();
                    bool allShown = true;
                    for (QGraphicsItem *item : scene->items()) allShown &= item->isVisible();
                    visibility = allHidden && allShown;
                }
                const bool legacyRotationUnknown = panel->selectPad(2) &&
                    window.displayedProperties().contains("rotation NOT_VERIFIED");
                passed = geometry && counts && selected && visibility && legacyRotationUnknown;
                if (!passed) failure = "RF geometry / pad-only Properties / layer visibility mismatch";
                loop.quit();
            }
        });
        QTimer::singleShot(15000, &loop, &QEventLoop::quit);
        tick.start(20);
        loop.exec();
        tick.stop();
        if (!passed && failure.isEmpty()) failure = "RF GUI smoke timed out";
    }
    QJsonObject report{
        {"schema", "boardsi-preview-p2d-rect-smoke-v1"},
        {"pass", passed}, {"failure", failure}, {"elapsed_ms", clock.elapsed()},
        {"source_sha", qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"native_backend", QGuiApplication::platformName()},
        {"host_arch", QSysInfo::currentCpuArchitecture()},
        {"scope", "public RF pad-only orthogonal 0/90 and part 0/90/180/270; SI NOT_RUN"}
    };
    QFile receipt("qt-preview-p2d-rect-results.json");
    if (!receipt.open(QIODevice::WriteOnly) ||
        receipt.write(QJsonDocument(report).toJson(QJsonDocument::Indented)) < 1) return 2;
    qInfo().noquote() << QJsonDocument(report).toJson(QJsonDocument::Compact);
    return passed ? 0 : 1;
}


static int runBottomSmoke(PreviewWindow &window, const QString &fixture) {
    QElapsedTimer clock;
    clock.start();
    bool passed = false;
    QString failure;
    bool rhiViewChecked = false, rhiFrame = false, softwareCpuFallback = false;
    bool rhiCpuFallback = false;
    QEventLoop loop;
    if (!window.openCandidate(fixture)) {
        failure = "P2-E public bottom fixture unavailable";
    } else {
        QTimer tick;
        QObject::connect(&tick, &QTimer::timeout, &loop, [&] {
            const QString status = window.importPanel()->importStatus();
            if (status.startsWith("FAILED") || status.startsWith("CANCELLED")) {
                failure = "P2-E import failed: " + status;
                loop.quit();
            } else if (status.startsWith("PARTIAL")) {
                auto *panel = window.importPanel()->imported();
#ifdef BOARDSI_RHI_ARTWORK
                const bool softwareRequest = BoardSiRhiView::softwareRendererRequested();
                const bool cpuRequest =
                    qEnvironmentVariable("BOARDSI_ARTWORK_RENDERER").trimmed().toLower()=="cpu";
                const bool nativeRhiTest = !softwareRequest && !cpuRequest &&
                    QGuiApplication::platformName() != "offscreen" &&
                    QGuiApplication::platformName() != "minimal";
                auto *rhi = panel->rhiViewForTest();
                if (nativeRhiTest) {
                    if (!rhi) {
                        failure = "Qt Quick-enabled imported artwork lacks RHI view";
                        loop.quit(); return;
                    }
                    const QString renderState = rhi->status();
                    if (renderState.startsWith("RHI_PENDING_FRAME")) return;
                    if (renderState == "CPU_FALLBACK_RHI_SOFTWARE_D3D11_ADAPTER") {
                        // This is the ONLY acceptable native GPU test fallback:
                        // render-thread DXGI proved the actual D3D11 adapter
                        // was software (e.g. WARP). No-frame, device error and
                        // missing adapter proof are real integration failures.
                        rhiCpuFallback = !panel->rhiCanvasActiveForTest() &&
                            panel->rendererStatus().startsWith("CPU_");
                        if (!rhiCpuFallback) {
                            failure = "Confirmed WARP did not activate CPU artwork";
                            loop.quit(); return;
                        }
                    } else if (renderState.startsWith("CPU_FALLBACK_RHI_")) {
                        failure = "Uncertified RHI failure may not PASS: " + renderState;
                        loop.quit(); return;
                    } else if (renderState.startsWith("RHI_FRAME_") &&
                               panel->rhiCanvasActiveForTest()) {
                        rhiFrame = true;
                    } else {
                        failure = "Imported RHI has no valid frame: " + renderState;
                        loop.quit(); return;
                    }
                } else if (softwareRequest) {
                    softwareCpuFallback = !rhi && panel->rendererStatus().startsWith("CPU_");
                    if (!softwareCpuFallback) {
                        failure = "Qt software RHI request did not force CPU";
                        loop.quit(); return;
                    }
                }
#endif
                auto *scene = panel->findChild<QGraphicsScene *>();
                const auto near = [](qreal a, qreal b) { return qAbs(a - b) < 0.000001; };
                struct Expected { const char *label; qreal x,y,w,h; int layer,rotation; };
                // Independently expected PADS top-view geometry, in mm.
                const Expected expected[] = {
                    {"Pad U0.1", 5.08,-7.62,2.032,0.762,4,0},
                    {"Pad U0.2", 2.54,-7.62,0.762,2.032,4,0},
                    {"Pad U1.1",10.16,-7.62,0.762,2.032,4,90},
                    {"Pad U1.2",10.16,-10.16,2.032,0.762,4,90},
                    {"Pad U2.1",15.24,-7.62,2.032,0.762,4,180},
                    {"Pad U2.2",17.78,-7.62,0.762,2.032,4,180},
                    {"Pad U3.1",20.32,-7.62,0.762,2.032,4,270},
                    {"Pad U3.2",20.32,-5.08,2.032,0.762,4,270},
                    {"Pad U4.1",25.40,-7.62,2.032,0.762,1,0},
                    {"Pad U4.2",27.94,-7.62,0.762,2.032,1,0}
                };
                bool geometry = scene && scene->items().size() == 10;
                if (geometry) for (const auto &want : expected) {
                    bool found = false;
                    for (QGraphicsItem *item : scene->items()) {
                        auto *rect = qgraphicsitem_cast<QGraphicsRectItem *>(item);
                        if (!rect || !item->data(1).toString().contains(want.label)) continue;
                        const QRectF r = rect->rect();
                        const QString d = item->data(1).toString();
                        if (near(r.center().x(),want.x) && near(r.center().y(),want.y) &&
                            near(r.width(),want.w) && near(r.height(),want.h) &&
                            d.contains(QString("layer %1").arg(want.layer)) &&
                            d.contains(QString("rotation %1 deg").arg(want.rotation)) &&
                            d.contains(want.layer == 4 ? "side Bottom" : "side Top") &&
                            d.contains("net NOT_VERIFIED"))
                            found = true;
                    }
                    if (!found) { geometry = false; break; }
                }
                bool rhiGeometry = true, rhiPicking = true;
#ifdef BOARDSI_RHI_ARTWORK
                if (rhiFrame) {
                    auto *item = panel->rhiViewForTest()->itemForTest();
                    rhiViewChecked = item && item->primitiveCount()==10 &&
                                     item->visiblePadCount()==10;
                    rhiGeometry = rhiViewChecked;
                    if (rhiGeometry) for (const auto &want : expected) {
                        const QStringList id=QString::fromLatin1(want.label).mid(4).split('.');
                        if (id.size()!=2 || !item->containsExactPad(id.at(0),id.at(1),
                                want.layer,want.rotation,qRound64(want.x*1e6),
                                qRound64(-want.y*1e6),qRound64(want.w*1e6),
                                qRound64(want.h*1e6))) {
                            rhiGeometry=false; break;
                        }
                    }
                    if (rhiGeometry) {
                        const QPointF at=item->screenPointForTest(2540000,7620000);
                        // Seven-argument Qt 6 constructor: explicit local,
                        // scene and global positions, no deprecated overload.
                        QMouseEvent press(QEvent::MouseButtonPress,at,at,at,
                                         Qt::LeftButton,Qt::LeftButton,Qt::NoModifier);
                        QMouseEvent release(QEvent::MouseButtonRelease,at,at,at,
                                           Qt::LeftButton,Qt::NoButton,Qt::NoModifier);
                        QCoreApplication::sendEvent(item,&press);
                        QCoreApplication::sendEvent(item,&release);
                        rhiPicking=window.displayedProperties().contains("Pad U0.2") &&
                                   window.displayedProperties().contains("side Bottom");
                    }
                }
#endif
                const bool selected = panel->selectPad(1) &&
                    window.displayedProperties().contains("Pad U0.2") &&
                    window.displayedProperties().contains("side Bottom") &&
                    window.displayedProperties().contains("Top SMD: 2  Bottom SMD: 8");
                auto *layers = window.findChild<QComboBox *>("importedLayerSelect");
                auto *toggle = window.findChild<QPushButton *>("importedLayerToggle");
                bool visibility = false;
                if (layers && toggle && scene) {
                    const int index = layers->findData(4);
                    if (index >= 0) {
                        layers->setCurrentIndex(index);
                        toggle->click();
                        int count = 0;
                        for (auto *item : scene->items()) if (item->isVisible()) ++count;
                        const bool onlyTop = count == 2;
#ifdef BOARDSI_RHI_ARTWORK
                        if (rhiFrame) rhiGeometry = rhiGeometry &&
                            panel->rhiViewForTest()->itemForTest()->visiblePadCount()==2;
#endif
                        toggle->click();
                        count = 0;
                        for (auto *item : scene->items()) if (item->isVisible()) ++count;
                        visibility = onlyTop && count == 10;
#ifdef BOARDSI_RHI_ARTWORK
                        if (rhiFrame) rhiGeometry = rhiGeometry &&
                            panel->rhiViewForTest()->itemForTest()->visiblePadCount()==10;
#endif
                    }
                }
                passed = geometry && selected && visibility && rhiGeometry && rhiPicking &&
                         panel->padCount() == 10 && panel->segmentCount() == 0 &&
                         panel->viaCount() == 0;
                if (!passed) failure = "P2-E geometry/side/selection/visibility mismatch";
                loop.quit();
            }
        });
        QTimer::singleShot(15000, &loop, &QEventLoop::quit);
        tick.start(20);
        loop.exec();
        tick.stop();
        if (!passed && failure.isEmpty()) failure = "P2-E smoke timed out";
    }
    QJsonObject report{
        {"schema", "boardsi-preview-p2e-bottom-smoke-v1"},
        {"pass", passed}, {"failure", failure}, {"elapsed_ms", clock.elapsed()},
        {"rhi_view_checked", rhiViewChecked},
        {"rhi_first_frame_confirmed", rhiFrame},
        {"software_rhi_cpu_fallback", softwareCpuFallback},
        {"rhi_cpu_fallback", rhiCpuFallback},
        {"source_sha", qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"native_backend", QGuiApplication::platformName()},
        {"host_arch", QSysInfo::currentCpuArchitecture()},
        {"scope", "PADS V9.4 synthetic top/bottom orthogonal SMD, SI NOT_RUN"}
    };
    QFile receipt("qt-preview-p2e-bottom-results.json");
    if (!receipt.open(QIODevice::WriteOnly) ||
        receipt.write(QJsonDocument(report).toJson(QJsonDocument::Indented)) < 1) return 2;
    qInfo().noquote() << QJsonDocument(report).toJson(QJsonDocument::Compact);
    return passed ? 0 : 1;
}

static int runViaSmoke(PreviewWindow &window, const QString &fixture) {
    QElapsedTimer clock;
    clock.start();
    QString failure;
    bool passed = false;
    QEventLoop loop;
    if (!window.openCandidate(fixture)) {
        failure = "P2-B synthetic via fixture could not be opened";
    } else {
        QTimer tick;
        QObject::connect(&tick, &QTimer::timeout, &loop, [&] {
            const QString status = window.importPanel()->importStatus();
            if (status.startsWith("FAILED") || status.startsWith("CANCELLED")) {
                failure = "P2-B import failed: " + status;
                loop.quit();
            } else if (status.startsWith("PARTIAL")) {
                auto *panel = window.importPanel()->imported();
                const bool count = panel->viaCount() == 1 && panel->segmentCount() == 2;
                const bool net = panel->selectNet("NET_A");
                const bool selected = panel->selectVia(0);
                const bool details = window.displayedProperties().contains(
                    "Explicit round through vias: 1") &&
                    window.displayedProperties().contains("Via SIMPLEVIA") &&
                    window.displayedProperties().contains("outer 0.889000 mm") &&
                    window.displayedProperties().contains("drill 0.381000 mm");
                passed = count && net && selected && details;
                if (!passed) failure = "Via/net/Properties scene mismatch";
                loop.quit();
            }
        });
        QTimer::singleShot(15000, &loop, &QEventLoop::quit);
        tick.start(20);
        loop.exec();
        tick.stop();
        if (!passed && failure.isEmpty()) failure = "Via smoke timed out";
    }
    QJsonObject report{
        {"schema", "boardsi-preview-p2b-via-smoke-v1"},
        {"pass", passed}, {"failure", failure},
        {"elapsed_ms", clock.elapsed()},
        {"source_sha", qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"native_backend", QGuiApplication::platformName()},
        {"host_arch", QSysInfo::currentCpuArchitecture()},
        {"scope", "public synthetic explicit round through-via only; SI NOT_RUN"}
    };
    QFile receipt("qt-preview-p2b-via-results.json");
    if (!receipt.open(QIODevice::WriteOnly) ||
        receipt.write(QJsonDocument(report).toJson(QJsonDocument::Indented)) < 1)
        return 2;
    qInfo().noquote() << QJsonDocument(report).toJson(QJsonDocument::Compact);
    return passed ? 0 : 1;
}

// A_014 stand-alone native Qt Quick exercise. This runs no Rust Worker or
// physical SI. Hardware ownership is explicitly NOT certified by a RHI frame.
// A_019 P2-J bounded source-model scale probe. This intentionally does NOT
// measure GPU FPS, VRAM, actual frame latency, or shipped-board throughput.
// Heavy CI runners can validate 10k/100k/1M synthetic source primitives without
// needing a physical GPU, while the native RHI smoke remains a separate gate.
static int runRhiScaleSmoke() {
    QJsonArray samples;
    bool pass = true;
    QString failure;
#ifdef BOARDSI_RHI_ARTWORK
    constexpr qint64 base = 90000000000LL;
    BoardSiRhiItem item;
    item.setWidth(800);
    item.setHeight(500);
    for (const int count : {10000, 100000, 1000000}) {
        std::vector<ImportedTrack> tracks;
        tracks.reserve(static_cast<size_t>(count));
        for (int i=0;i<count;++i) {
            ImportedTrack t;
            t.layer=1;
            t.net="SYNTHETIC";
            t.x0_nm=base+static_cast<qint64>(i%1000)*100000;
            t.y0_nm=base+static_cast<qint64>(i/1000)*100000;
            t.x1_nm=t.x0_nm+25000;
            t.y1_nm=t.y0_nm;
            t.width_nm=10000;
            tracks.push_back(t);
        }
        QElapsedTimer clock;
        clock.start();
        item.load(4,tracks,{},{});
        const qint64 loadMs=clock.elapsed();
        const QPointF a=item.localPoint(base,base);
        const QPointF b=item.localPoint(base+25000,base+50000);
        const bool exact = std::abs((b.x()-a.x())-0.025)<1e-9 &&
                           std::abs((b.y()-a.y())+0.050)<1e-9;
        const qreal projected = item.projectedHalfWidthForTest(10000);
        const int vertices = item.lineVertexCountForTest(projected);
        const bool mesh = item.lineVertexCountForTest(0.25)==6 &&
                          item.lineVertexCountForTest(1.0)==78 &&
                          (projected<=0.5 ? vertices==6 : vertices==78);
        // Regression: physical display resolution, not logical Qt pixels,
        // determines whether an otherwise subpixel cap may be omitted.
        const bool highDpi = item.lineVertexCountForTest(0.25,2.0)==6 &&
                             item.lineVertexCountForTest(0.26,2.0)==78 &&
                             item.lineVertexCountForTest(0.5,2.0)==78;
        const bool casePass=item.primitiveCount()==count && exact && mesh && highDpi &&
                            std::isfinite(projected) && projected>0;
        if(!casePass) {
            pass=false;
            failure=QString("A_019 scale source/mesh check failed at %1 objects").arg(count);
        }
        samples.append(QJsonObject{
            {"objects",count}, {"source_model_load_ms",loadMs},
            {"exact_local_coordinate",exact},
            {"screen_half_width_px",projected},
            {"nondegenerate_line_vertices",vertices},
            {"baseline_line_vertices",78},
            {"high_dpi_physical_cap_gate",highDpi},
            {"pass",casePass}
        });
    }
#else
    pass=false;
    failure="Qt Quick module not available; scale probe NOT_RUN";
#endif
    QJsonObject result{
        {"schema","boardsi-a019-rhi-model-scale-v1"},
        {"pass",pass},{"samples",samples},
        {"scope","Offline synthetic RHI model ingestion and screen-space vertex policy ONLY"},
        {"rendered_frames","NOT_RUN"}, {"gpu_fps","NOT_MEASURED"},
        {"gpu_adapter","NOT_VERIFIED"}, {"peak_rss","NOT_MEASURED"},
        {"gpu_vram","NOT_MEASURED"}, {"pick_latency_p95","NOT_MEASURED"},
        {"failure",failure}, {"source_sha",qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"host_arch",QSysInfo::currentCpuArchitecture()}
    };
    QFile output("qt-preview-a019-rhi-scale-results.json");
    if(!output.open(QIODevice::WriteOnly) ||
       output.write(QJsonDocument(result).toJson(QJsonDocument::Indented))<1) return 2;
    qInfo().noquote() << QJsonDocument(result).toJson(QJsonDocument::Compact);
    return pass?0:1;
}

static int runRhiGeometrySmoke() {
    QElapsedTimer clock;
    clock.start();
    bool sourceGeometry = false;
    int count = 0;
    QString backend = "NOT_BUILT";
    bool frame = false;
    bool postLoadFrameVerified = false;
    bool fallbackConfirmed = false;
    int adapterProof = 0;
    QString failure;
#ifdef BOARDSI_RHI_ARTWORK
    constexpr qint64 base = 90000000000LL;
    std::vector<ImportedTrack> tracks;
    tracks.reserve(10000);
    for (int i=0;i<10000;++i) {
        ImportedTrack t;
        t.layer=1; t.net="SYNTHETIC";
        t.x0_nm=base+static_cast<qint64>(i)*5000;
        t.x1_nm=t.x0_nm+25000;
        t.y0_nm=base+10000;
        t.y1_nm=base+10000;
        t.width_nm=10000;
        tracks.push_back(t);
    }
    std::vector<ImportedVia> vias;
    ImportedVia via;
    via.name="TEST";via.net="SYNTHETIC";via.firstLayer=1;via.lastLayer=4;
    via.x_nm=base+25000;via.y_nm=base+10000;
    via.diameter_nm=700000;via.drill_nm=300000;
    vias.push_back(via);
    std::vector<ImportedPad> pads;
    ImportedPad p;
    p.reference="U1";p.pin="1";p.shape="RF";p.side="B";p.layer=4;
    p.rotation=90;p.x_nm=base+50000;p.y_nm=base+40000;
    p.width_nm=160000;p.height_nm=80000;
    pads.push_back(p);
    auto *item = new BoardSiRhiItem();
    item->setWidth(800); item->setHeight(500);
    // A_015 exact quarter-circle in the same high-origin source domain.
    ImportedArc arc;
    arc.layer=1; arc.net="SYNTHETIC";
    arc.x0_nm=base+100000; arc.y0_nm=base+100000;
    arc.cx_nm=base+50000; arc.cy_nm=base+100000;
    arc.x1_nm=base+50000; arc.y1_nm=base+150000;
    arc.width_nm=10000; arc.clockwise=false; arc.sourceLine=1;
    const std::vector<ImportedArc> arcs{arc};
    item->load(4,tracks,vias,pads,arcs);
    const QPointF pointA=item->localPoint(base+25000,base+10000);
    const QPointF pointB=item->localPoint(base+50000,base+40000);
    sourceGeometry=item->primitiveCount()==10003 &&
                   std::abs((pointB.x()-pointA.x())-0.025)<1e-9 &&
                   std::abs((pointB.y()-pointA.y())+0.030)<1e-9;
    delete item;
    count=10003;
    // A visible native Qt Quick window is required to prove RHI frame delivery.
    // This does not establish a discrete or hardware GPU adapter.
    if (QGuiApplication::platformName() != "offscreen" &&
        QGuiApplication::platformName() != "minimal") {
        BoardSiRhiView view;
        view.load(4,tracks,vias,pads,arcs);
        view.resize(800,500);
        view.show();
        QEventLoop loop;
        QTimer poll;
        QObject::connect(&poll,&QTimer::timeout,&loop,[&] {
            if (view.status().startsWith("RHI_FRAME_") || view.failed()) loop.quit();
        });
        QTimer::singleShot(4800,&loop,&QEventLoop::quit);
        poll.start(20);loop.exec();poll.stop();
        backend=view.status();
        frame=backend.startsWith("RHI_FRAME_");
        // ONLY render-thread DXGI-proven WARP is an acceptable native
        // fallback. Frame timeout, scene graph error and missing adapter proof
        // must fail the 10,003-object geometry integration test as well.
        fallbackConfirmed=(backend=="CPU_FALLBACK_RHI_SOFTWARE_D3D11_ADAPTER");
        adapterProof=view.itemForTest()->d3dAdapterProof();
        if (!frame && !fallbackConfirmed)
            failure="No RHI frame or verified CPU fallback: "+backend;
        if (frame) {
            // A prior empty/success frame must NOT satisfy the next load.
            // Require a pending transition and a distinct post-load swap.
            view.load(4,tracks,vias,pads,arcs);
            const bool becamePending =
                view.status()=="RHI_PENDING_FRAME_HW_NOT_VERIFIED";
            QEventLoop reloaded;
            QTimer recheck;
            QObject::connect(&recheck,&QTimer::timeout,&reloaded,[&] {
                if (view.status().startsWith("RHI_FRAME_") || view.failed())
                    reloaded.quit();
            });
            QTimer::singleShot(4800,&reloaded,&QEventLoop::quit);
            recheck.start(20);reloaded.exec();recheck.stop();
            postLoadFrameVerified=becamePending && view.status().startsWith("RHI_FRAME_");
            if (!postLoadFrameVerified)
                failure="New artwork load did not produce a fresh verified frame: "+
                        view.status();
        }
    } else {
        backend="HEADLESS_NOT_RUN";
    }
#else
    failure="Qt Quick module not installed in SDK";
#endif
    // Geometry-only headless execution is NOT a native RHI frame PASS.
    const bool pass=sourceGeometry && (fallbackConfirmed ||
                                         (frame && postLoadFrameVerified));
    QJsonObject result{
        {"schema","boardsi-a014-rhi-smoke-v1"},
        {"pass",pass},{"source_geometry_check",sourceGeometry},
        {"source_objects",count},{"backend_status",backend},
        {"rhi_frame_observed",frame},
        {"post_load_frame_verified",postLoadFrameVerified},
        {"cpu_fallback_confirmed",fallbackConfirmed},
        {"d3d11_adapter_proof",(backend.contains("DIRECT3D11") || backend.contains("D3D11")) ? adapterProof : 0},
        {"metal_device_proof",backend.contains("METAL") ? adapterProof : 0},
        {"hardware_adapter",adapterProof==1 ?
            (backend.contains("METAL") ? "METAL_MTLDEVICE_CONFIRMED" :
             backend.contains("DIRECT3D11") ? "DXGI_NON_SOFTWARE_CONFIRMED" :
             "NOT_VERIFIED") : "NOT_VERIFIED"},
        {"render_fps","NOT_MEASURED"},{"failure",failure},
        {"native_backend",QGuiApplication::platformName()},
        {"host_arch",QSysInfo::currentCpuArchitecture()},
        {"elapsed_ms",clock.elapsed()},
        {"source_sha",qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"scope","Synthetic 10003 objects including arc, integer nm local geometry and RHI frame; NOT PCB SI"}
    };
    QFile output("qt-preview-a014-rhi-results.json");
    if (!output.open(QIODevice::WriteOnly) ||
        output.write(QJsonDocument(result).toJson(QJsonDocument::Indented)) < 1) return 2;
    qInfo().noquote() << QJsonDocument(result).toJson(QJsonDocument::Compact);
    return pass?0:1;
}


// A_023 V11: real Rust combined COPPER/COPCUT -> Qt decoder ->
// CPU even-odd hollow path, RHI exact hollow pick, layers/cleanup.
static int runCutoutSmoke(PreviewWindow &window,const QString &fixture,bool orthogonal=false,bool diagonal=false,bool circular=false) {
    bool pass=false,loaded=false,hollow=false,properties=false,filtered=false;
    bool malformed=false,reset=false,rhiPick=false,rhiFrame=false;
    QString failure;
    if(!window.openCandidate(fixture))failure="A023 fixture unavailable";
    else {
        QEventLoop loop;QTimer tick;
        QObject::connect(&tick,&QTimer::timeout,&loop,[&]{
            const auto state=window.importPanel()->importStatus();
            if(state.startsWith("FAILED")||state.startsWith("CANCELLED")) {
                failure="A023 import failed: "+state;loop.quit();return;
            }
            if(!state.startsWith("PARTIAL"))return;
            auto *panel=window.importPanel()->imported();
#ifdef BOARDSI_RHI_ARTWORK
            // Qt Quick source frame must have been presented for THIS import;
            // a stale RHI instance can survive a CPU fallback.
            auto *nativeRhi=panel->rhiViewForTest();
            if(!nativeRhi || !panel->rhiCanvasActiveForTest() || nativeRhi->failed()) {
                failure="A023 RHI unavailable / CPU fallback / NOT_RUN";
                loop.quit();return;
            }
            if(!nativeRhi->status().startsWith("RHI_FRAME_"))return;
            rhiFrame=true;
#else
            failure="A023 CPU-only build / imported RHI frame NOT_RUN";
            loop.quit();return;
#endif
            loaded=panel->copperCount()==1 && state.contains("1 filled source copper areas");
            properties=panel->selectCopper(0) &&
                window.displayedProperties().contains(circular ? "COPPER COP_CIRC" : "COPPER COP_HOLE") &&
                window.displayedProperties().contains(circular ? "cutouts 3" : "cutouts 2") &&
                window.displayedProperties().contains("connectivity NOT_VERIFIED") &&
                (!orthogonal || window.displayedProperties().contains("vertices 7")) &&
                (!diagonal || window.displayedProperties().contains("vertices 5"));
            auto *scene=panel->findChild<QGraphicsScene *>();
            QGraphicsPathItem *copper=nullptr;
            if(scene)for(QGraphicsItem *it:scene->items()) {
                if(it->data(2).toString().startsWith(circular ? "COPPER COP_CIRC" : "COPPER COP_HOLE")) {
                    copper=qgraphicsitem_cast<QGraphicsPathItem *>(it);break;
                }
            }
            constexpr qreal mil=0.0254;
            if(copper) {
                const auto path=copper->path();
                hollow=circular ?
                    (path.contains(QPointF(60*mil,-60*mil)) &&
                     path.contains(QPointF(160*mil,-160*mil)) &&
                     !path.contains(QPointF(90*mil,-80*mil)) &&
                     !path.contains(QPointF(130*mil,-120*mil)) &&
                     !path.contains(QPointF(75*mil,-140*mil)) &&
                     !path.contains(QPointF(180*mil,-160*mil))) :
                    diagonal ?
                    (path.contains(QPointF(50*mil,-50*mil)) &&
                     !path.contains(QPointF(80*mil,-70*mil)) &&
                     !path.contains(QPointF(120*mil,-100*mil)) &&
                     !path.contains(QPointF(155*mil,-125*mil)) &&
                     path.contains(QPointF(130*mil,-65*mil))) :
                    (path.contains(QPointF(50*mil,-50*mil)) &&
                     !path.contains(QPointF(70*mil,-70*mil)) &&
                     (orthogonal ?
                      (!path.contains(QPointF(110*mil,-110*mil)) &&
                       !path.contains(QPointF(145*mil,-110*mil)) &&
                       path.contains(QPointF(80*mil,-80*mil))) :
                      !path.contains(QPointF(110*mil,-90*mil))));
            }
            auto *hide=panel->findChild<QPushButton *>("importedHideAllLayers");
            auto *show=panel->findChild<QPushButton *>("importedShowAllLayers");
            if(hide&&show&&copper) {
                hide->click();const bool hidden=!copper->isVisible();
                show->click();const bool restored=copper->isVisible();
                filtered=hidden&&restored;
            }
            ImportedCopperArea bad;
            bad.name="BAD";bad.net="-";bad.layer=1;bad.sourceLine=1;
            bad.ring={{0,0},{25400,0},{25400,25400},{0,25400},{0,0}};
            bad.cutouts.push_back({{0,2540},{5080,2540},{5080,5080},{0,5080},{0,2540}});
            // A_025 exact Codex P2 reproducer: two independent triangular
            // voids can bridge to the same outer vertex. Each duplicate
            // vertex has a distinct incident sector; never choose by distance
            // alone or reject a valid source-authored board on a no-ear path.
            bool sharedSector=true;
            if(diagonal) {
                constexpr qint64 t=254;
                ImportedCopperArea sector;
                sector.name="SHARED_SECTOR"; sector.net="-";
                sector.layer=1; sector.sourceLine=1;
                sector.ring={{0,0},{1000*t,0},{1000*t,1000*t},
                             {0,1000*t},{0,0}};
                sector.cutouts={
                    {{141*t,24*t},{161*t,24*t},{151*t,44*t},{141*t,24*t}},
                    {{305*t,127*t},{325*t,127*t},{315*t,147*t},{305*t,127*t}}
                };
                sharedSector=validateCopperArea(sector) &&
                    !sector.hollowTriangles.empty();
            }
            ImportedCopperArea roundBad=bad;
            roundBad.cutouts.clear();
            roundBad.roundCutouts.push_back({12700,12700,254}); // invalid radius
            malformed=!validateCopperArea(bad) &&
                (!circular || !validateCopperArea(roundBad)) &&
                !PreviewImportPanel::validTransportForTest(
                    "BOARDSI_P2N_PARTIAL_V11|V9.4-MILS|4\nL|1|Top\n") &&
                (!circular || !PreviewImportPanel::validTransportForTest(
                    "BOARDSI_P2Q_PARTIAL_V12|V9.4-MILS|4\nL|1|Top\n"));
#ifdef BOARDSI_RHI_ARTWORK
            if(auto *rhi=panel->rhiViewForTest()) {
                auto *item=rhi->itemForTest();
                rhiPick=item && item->copperCountForTest()==1 &&
                    item->dispatchCopperPickForTest(0) &&
                    (circular ? item->rejectCopperHolePickForTest(0) &&
                        item->rejectCopperCircleHolePickForTest(0) &&
                        item->rejectCopperCircleHolePickForTest(0,1) :
                        item->rejectCopperHolePickForTest(0) &&
                        item->rejectCopperHolePickForTest(0,1)) &&
                    (!orthogonal || item->rejectCopperSourcePointForTest(
                        145*25400LL,110*25400LL)) &&
                    (!diagonal || item->rejectCopperSourcePointForTest(
                        155*25400LL,125*25400LL));
            }else rhiPick=false; // No RHI object: never certify an unrun RHI test.
#else
            rhiPick=false; // CPU-only build cannot claim Qt Quick/RHI validation.
#endif
            panel->clearArtwork();
            reset=panel->copperCount()==0;
            pass=loaded&&hollow&&properties&&filtered&&malformed&&reset&&rhiFrame&&rhiPick&&sharedSector;
            if(!pass)failure="A023 hollow copper source, CPU/RHI, filter or decoder mismatch";
            loop.quit();
        });
        QTimer::singleShot(15000,&loop,&QEventLoop::quit);
        tick.start(20);loop.exec();tick.stop();
        if(!pass&&failure.isEmpty())failure="A023 native cutout smoke timed out";
    }
    const QJsonObject report{
        {"schema",circular ? "boardsi-preview-a026-circle-v1" :
                  diagonal ? "boardsi-preview-a025-diagonal-v1" :
                  orthogonal ? "boardsi-preview-a024-orthogonal-v1" :
                               "boardsi-preview-a023-cutout-v1"},
        {"pass",pass},{"fixture_import",loaded},{"cpu_hollow_fill",hollow},
        {"source_properties",properties},{"layer_filter",filtered},
        {"malformed_rejected",malformed},{"clear_resets",reset},
        {"orthogonal_source",orthogonal},{"diagonal_source",diagonal},
        {"circular_source",circular},
        {"shared_bridge_sector",diagonal ? pass : true},
        {"rhi_hole_pick",rhiPick},{"rhi_circle_hole_pick",circular && rhiPick},
        {"rhi_imported_frame_confirmed",rhiFrame},
        {"physical_copper","NOT_VERIFIED"},{"physical_si","NOT_RUN"},{"failure",failure}
    };
    QFile output(circular ? "qt-preview-a026-circle-results.json" :
                 diagonal ? "qt-preview-a025-diagonal-results.json" :
                orthogonal ? "qt-preview-a024-orthogonal-results.json" :
                             "qt-preview-a023-cutout-results.json");
    if(!output.open(QIODevice::WriteOnly)||
       output.write(QJsonDocument(report).toJson(QJsonDocument::Indented))<1)return 2;
    qInfo().noquote()<<QJsonDocument(report).toJson(QJsonDocument::Compact);
    return pass?0:1;
}

// A_022 V10: real Rust synthetic COPCLS/COPCIR -> strict Qt decoder ->
// CPU/RHI filled geometry, selection/filter parity and state clearing.
static int runCopperSmoke(PreviewWindow &window,const QString &fixture) {
    bool pass=false,imported=false,properties=false,filtered=false;
    bool malformed=false,reset=false,rhiSource=false;
    QString failure;
    if(!window.openCandidate(fixture)) failure="A022 fixture unavailable";
    else {
        QEventLoop loop;
        QTimer tick;
        QObject::connect(&tick,&QTimer::timeout,&loop,[&] {
            const auto state=window.importPanel()->importStatus();
            if(state.startsWith("FAILED") || state.startsWith("CANCELLED")) {
                failure="A022 import failed: "+state;loop.quit();return;
            }
            if(!state.startsWith("PARTIAL")) return;
            auto *panel=window.importPanel()->imported();
            imported=panel->copperCount()==2 && panel->outlineCount()==4 &&
                     panel->segmentCount()==1 && state.contains("2 filled source copper areas");
            properties=panel->selectCopper(0) &&
                window.displayedProperties().contains("COPPER COP_A") &&
                window.displayedProperties().contains("COPCLS") &&
                window.displayedProperties().contains("source line") &&
                window.displayedProperties().contains("connectivity NOT_VERIFIED");
            auto *hide=panel->findChild<QPushButton *>("importedHideAllLayers");
            auto *show=panel->findChild<QPushButton *>("importedShowAllLayers");
            auto *netOnly=panel->findChild<QCheckBox *>("importedNetOnly");
            auto *scene=panel->findChild<QGraphicsScene *>();
            auto countVisible=[&] {
                int count=0;
                if(scene) for(auto *item:scene->items())
                    if(item->isVisible() && item->data(2).toString().startsWith("COPPER "))
                        ++count;
                return count;
            };
            if(hide && show && netOnly) {
                const bool initial=countVisible()==2;
                hide->click(); const bool hidden=countVisible()==0;
                show->click(); const bool restored=countVisible()==2;
                const bool picked=panel->selectNet("GND");
                netOnly->setChecked(true);
                const bool one=countVisible()==1;
                netOnly->setChecked(false);
                filtered=initial && hidden && restored && picked && one &&
                         countVisible()==2;
            }
            const QByteArray bad="BOARDSI_P2M_PARTIAL_V10|V9.4-MILS|4\nL|1|Top\n";
            malformed=!PreviewImportPanel::validTransportForTest(bad);
            // Test the same production validation independent of the importer:
            // self-crossing polygon may never become Qt/RHI filled copper.
            ImportedCopperArea invalid;
            invalid.name="SELF_CROSS";invalid.net="-";invalid.layer=1;invalid.sourceLine=1;
            invalid.ring={{0,0},{25400,25400},{25400,0},{0,25400},{0,0}};
            malformed=malformed && !validateCopperArea(invalid);
#ifdef BOARDSI_RHI_ARTWORK
            if(auto *rhi=panel->rhiViewForTest()) {
                auto *item=rhi->itemForTest();
                rhiSource=item && item->copperCountForTest()==2 &&
                          item->dispatchCopperPickForTest(0);
            }else rhiSource=true;
#else
            rhiSource=true;
#endif
            panel->clearArtwork();
            reset=panel->copperCount()==0 && panel->segmentCount()==0;
            pass=imported && properties && filtered && malformed && reset && rhiSource;
            if(!pass) failure="A022 source-filled import, properties, filters, decoder or RHI mismatch";
            loop.quit();
        });
        QTimer::singleShot(15000,&loop,&QEventLoop::quit);
        tick.start(20);loop.exec();tick.stop();
        if(!pass && failure.isEmpty()) failure="A022 native import timed out";
    }
    const QJsonObject report{
        {"schema","boardsi-preview-a022-copper-v1"},
        {"pass",pass},{"fixture_import",imported},
        {"source_properties",properties},{"filter_parity",filtered},
        {"malformed_rejected",malformed},{"clear_resets",reset},
        {"rhi_source_checked",rhiSource},
        {"fabricated_copper","NOT_VERIFIED"},
        {"electrical_connectivity","NOT_VERIFIED"},
        {"physical_si","NOT_RUN"},
        {"failure",failure}
    };
    QFile output("qt-preview-a022-copper-results.json");
    if(!output.open(QIODevice::WriteOnly) ||
       output.write(QJsonDocument(report).toJson(QJsonDocument::Indented))<1) return 2;
    qInfo().noquote()<<QJsonDocument(report).toJson(QJsonDocument::Compact);
    return pass?0:1;
}

// A_021: full public PADS fixture -> Rust importer -> Qt V9 decoder ->
// CPU model / RHI source geometry and exact Layer/Net/Properties regression.
static int runPourSmoke(PreviewWindow &window, const QString &fixture) {
    QElapsedTimer clock; clock.start();
    bool pass=false, imported=false, properties=false, filtered=false;
    bool malformedRejected=false, reset=false, rhiSource=false;
    QString failure;
    QEventLoop loop;
    if (!window.openCandidate(fixture)) {
        failure="A021 synthetic PADS fixture unavailable";
    } else {
        QTimer tick;
        QObject::connect(&tick,&QTimer::timeout,&loop,[&] {
            const auto state=window.importPanel()->importStatus();
            if(state.startsWith("FAILED") || state.startsWith("CANCELLED")) {
                failure="A021 source import failed: "+state;
                loop.quit(); return;
            }
            if(!state.startsWith("PARTIAL")) return;
            auto *panel=window.importPanel()->imported();
            imported=panel->pourEdgeCount()==4 && panel->segmentCount()==1 &&
                     panel->outlineCount()==4 &&
                     state.contains("4 POUROUT edges");
            properties=panel->selectPour(0) &&
                window.displayedProperties().contains("POUROUT GND_ZONE") &&
                window.displayedProperties().contains("source line") &&
                window.displayedProperties().contains("flood NOT_VERIFIED") &&
                window.displayedProperties().contains("SI NOT_RUN");
            auto *layers=panel->findChild<QComboBox *>("importedLayerSelect");
            auto *hide=panel->findChild<QPushButton *>("importedHideAllLayers");
            auto *show=panel->findChild<QPushButton *>("importedShowAllLayers");
            auto *netOnly=panel->findChild<QCheckBox *>("importedNetOnly");
            auto *scene=panel->findChild<QGraphicsScene *>();
            auto countVisible=[&]() {
                int n=0;
                if(scene) for(auto *item:scene->items())
                    if(item->isVisible() &&
                       item->data(2).toString().startsWith("POUROUT "))
                        ++n;
                return n;
            };
            if(layers && hide && show && netOnly) {
                const bool initial=countVisible()==4;
                hide->click();
                const bool hidden=countVisible()==0;
                show->click();
                const bool restored=countVisible()==4;
                const bool net=panel->selectNet("NET_A");
                netOnly->setChecked(true);
                const bool visibleExact=countVisible()==4;
                netOnly->setChecked(false);
                filtered=initial && hidden && restored && net && visibleExact;
            }
            // Production decoder: reject incomplete or malformed transport.
            QByteArray invalid="BOARDSI_P2L_PARTIAL_V9|V9.4-MILS|4\nL|1|Top\n";
            malformedRejected=!PreviewImportPanel::validTransportForTest(invalid);
#ifdef BOARDSI_RHI_ARTWORK
            if(auto *rhi=panel->rhiViewForTest()) {
                auto *item=rhi->itemForTest();
                rhiSource=item && item->primitiveCount()==9;
            } else {
                // Some headless software-only builds intentionally omit RHI.
                rhiSource=true;
            }
#else
            rhiSource=true;
#endif
            panel->clearArtwork();
            reset=panel->pourEdgeCount()==0 && panel->segmentCount()==0;
            pass=imported && properties && filtered && malformedRejected &&
                 reset && rhiSource;
            if(!pass) failure="A021 imported geometry, properties, filters, decoder, reset or RHI source mismatch";
            loop.quit();
        });
        QTimer::singleShot(15000,&loop,&QEventLoop::quit);
        tick.start(20);loop.exec();tick.stop();
        if(!pass && failure.isEmpty()) failure="A021 import timed out";
    }
    const QJsonObject report{
        {"schema","boardsi-preview-a021-pour-v1"},
        {"pass",pass},{"fixture_import",imported},
        {"source_properties",properties},
        {"layer_net_filter_restore",filtered},
        {"malformed_v9_rejected",malformedRejected},
        {"clear_resets_pour",reset},
        {"rhi_source_geometry_count",rhiSource},
        {"actual_flood","NOT_VERIFIED"},
        {"electrical_connectivity","NOT_VERIFIED"},
        {"physical_si","NOT_RUN"},
        {"gpu_fps","NOT_MEASURED"},
        {"source_sha",qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"native_backend",QGuiApplication::platformName()},
        {"host_arch",QSysInfo::currentCpuArchitecture()},
        {"elapsed_ms",clock.elapsed()},{"failure",failure}
    };
    QFile receipt("qt-preview-a021-pour-results.json");
    if(!receipt.open(QIODevice::WriteOnly) ||
       receipt.write(QJsonDocument(report).toJson(QJsonDocument::Indented))<1)
        return 2;
    qInfo().noquote()<<QJsonDocument(report).toJson(QJsonDocument::Compact);
    return pass?0:1;
}

int main(int argc, char **argv) {
    QApplication app(argc, argv);
    const QStringList args = app.arguments();
    if (args.size() == 2 && args.at(1) == "--explorer-selftest")
        return runA020ExplorerSmoke();
    if (args.size() == 2 && args.at(1) == "--rhi-geometry-selftest")
        return runRhiGeometrySmoke();
    if (args.size() == 2 && args.at(1) == "--rhi-scale-selftest")
        return runRhiScaleSmoke();
    if (args.size() == 2 && args.at(1) == "--rhi-native-benchmark")
        return runA019NativeRhiBenchmark();
    if (args.size() == 2 && args.at(1) == "--rhi-interaction-selftest")
        return runA019NativeInteractionValidation();
    const bool rendererTesting = args.size() == 5 && args.at(1) == "--renderer-selftest";
    const bool viaTesting = args.size() == 7 && args.at(1) == "--via-selftest";
    const bool padTesting = args.size() == 7 && args.at(1) == "--pad-selftest";
    const bool rectTesting = args.size() == 7 && args.at(1) == "--rect-selftest";
    const bool bottomTesting = args.size() == 7 && args.at(1) == "--bottom-selftest";
    const bool arcTesting = args.size() == 7 && args.at(1) == "--arc-selftest";
    const bool outlineTesting = args.size() == 7 && args.at(1) == "--outline-selftest";
    const bool circleTesting = args.size() == 7 && args.at(1) == "--circle-selftest";
    const bool mixedTesting = args.size() == 7 && args.at(1) == "--mixed-outline-selftest";
    const bool pourTesting = args.size() == 7 && args.at(1) == "--pour-selftest";
    const bool copperTesting = args.size() == 7 && args.at(1) == "--copper-selftest";
    const bool cutoutTesting = args.size() == 7 && args.at(1) == "--cutout-selftest";
    const bool orthogonalTesting = args.size() == 7 && args.at(1) == "--orthogonal-selftest";
    const bool diagonalTesting = args.size() == 7 && args.at(1) == "--diagonal-selftest";
    const bool circularCopperTesting = args.size()==7 && args.at(1)=="--circular-copper-selftest";
    const bool testing = ((args.size() == 5 || args.size() == 7) && args.at(1) == "--selftest") ||
                         rendererTesting || viaTesting || padTesting || rectTesting || bottomTesting ||
                         arcTesting || outlineTesting || circleTesting || mixedTesting || pourTesting || copperTesting || cutoutTesting || orthogonalTesting || diagonalTesting || circularCopperTesting;
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
    return circularCopperTesting ? runCutoutSmoke(window,fixture,false,false,true) :
           diagonalTesting ? runCutoutSmoke(window,fixture,false,true) :
           orthogonalTesting ? runCutoutSmoke(window,fixture,true) :
           cutoutTesting ? runCutoutSmoke(window, fixture) :
           copperTesting ? runCopperSmoke(window, fixture) :
           pourTesting ? runPourSmoke(window, fixture) :
           mixedTesting ? runMixedOutlineSmoke(window, fixture) :
           circleTesting ? runCircleSmoke(window, fixture) :
           outlineTesting ? runOutlineSmoke(window, fixture) :
           arcTesting ? runArcSmoke(window, fixture) :
           rendererTesting ? runRendererSmoke(window) :
           bottomTesting ? runBottomSmoke(window, fixture) :
           rectTesting ? runRectSmoke(window, fixture) :
           padTesting ? runPadSmoke(window, fixture) :
           viaTesting ? runViaSmoke(window, fixture) :
           testing ? runPreviewSmoke(app, window, worker, fixture) : app.exec();
}
