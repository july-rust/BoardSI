// E_011: actual PreviewWindow + accepted Canonical Rust source + Stackup user-overlay
// lifecycle. Source-bound software audit only; no physical SI or P009 claim.
#pragma once
#include <QEventLoop>
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QTemporaryDir>
#include <QTimer>

static int runE011MainStackupSelftest(const QString &canonicalBinary,
                                     const QString &fixturePath) {
    QFile input(fixturePath);
    if (!input.open(QIODevice::ReadOnly)) return 2;
    const QByteArray sourceA = input.readAll();
    if (!sourceA.contains("!PADS-POWERPCB-V9.4-MILS!") ||
        !sourceA.contains("*SIGNAL* NET_A")) return 2;
    QByteArray sourceB = sourceA;
    sourceB.replace("*SIGNAL* NET_A", "*SIGNAL* NET_B");
    QByteArray unsupported = sourceA;
    unsupported.replace("!PADS-POWERPCB-V9.4-MILS!",
                        "!PADS-POWERPCB-V9.3-MILS!");
    if (sourceA == sourceB || sourceA == unsupported) return 2;
    QTemporaryDir directory;
    if (!directory.isValid()) return 2;
    const auto writeFixture = [&](const QString &name, const QByteArray &bytes) {
        QFile file(directory.filePath(name));
        return file.open(QIODevice::WriteOnly) &&
               file.write(bytes) == bytes.size() && file.flush();
    };
    if (!writeFixture("source-a.asc", sourceA) ||
        !writeFixture("source-b.asc", sourceB) ||
        !writeFixture("unsupported.asc", unsupported)) return 2;

    const QString pathA = directory.filePath("source-a.asc");
    const QString pathB = directory.filePath("source-b.asc");
    const QString badPath = directory.filePath("unsupported.asc");
    const QString overlay = directory.filePath("stackup-a.json");
    const QString unknownOverlay = directory.filePath("unknown.json");
    const QString corrupt = directory.filePath("corrupt.json");

    PreviewWindow window{QString()};
    auto *panel = window.importPanel();
    auto *editor = window.stackupEditor();
    panel->setCanonicalImporterBinaryForTest(canonicalBinary);

    // Drive the actual PreviewWindow controller, not a detached Stackup mock.
    const auto runImport = [&](const QString &path, bool expectedSuccess) {
        if (!window.openCanonicalCandidate(path) ||
            editor->table()->rowCount() != 0 || editor->currentAuditIsFresh() ||
            !editor->auditScenarioRevision().isEmpty()) return false;
        QEventLoop loop;
        QTimer poll, deadline;
        bool timedOut = false;
        poll.setInterval(20);
        deadline.setSingleShot(true);
        QObject::connect(&poll, &QTimer::timeout, &loop, [&] {
            if (!panel->busy() && !panel->importStatus().startsWith("RUNNING"))
                loop.quit();
        });
        QObject::connect(&deadline, &QTimer::timeout, &loop, [&] {
            timedOut = true;
            loop.quit();
        });
        poll.start();
        deadline.start(45000);
        loop.exec();
        return !timedOut && !panel->busy() &&
            (expectedSuccess
                ? panel->importStatus().startsWith("CANONICAL SOURCE GEOMETRY")
                : panel->importStatus().startsWith("FAILED"));
    };

    if (!runImport(pathA, true) || editor->table()->rowCount() != 4 ||
        editor->table()->item(0, 1)->text() != "Unknown" ||
        !editor->table()->item(1, 4)->text().isEmpty() ||
        !editor->table()->item(1, 5)->text().isEmpty() ||
        !editor->summary().contains("MATERIALS UNKNOWN") ||
        !editor->validate().contains("RUN BLOCKED")) {
        qWarning("E011_FAIL:source-a-canonical-unknown");
        return 1;
    }
    qInfo("E011_STAGE:main-canonical-unknown");

    // Incomplete/UNKNOWN overlays must be safe to persist, but never qualified.
    const bool unknownRoundtrip = editor->saveTo(unknownOverlay) &&
        editor->loadFrom(unknownOverlay) &&
        editor->table()->item(0, 1)->text() == "Unknown" &&
        editor->validate().contains("RUN BLOCKED") &&
        !editor->currentAuditIsFresh();
    if (!unknownRoundtrip) {
        qWarning("E011_FAIL:unknown-overlay");
        return 1;
    }

    const auto set = [&](int row, int col, const QString &value) {
        editor->table()->item(row, col)->setText(value);
    };
    for (int row = 0; row < 4; ++row) {
        const bool copper = row == 0 || row == 3;
        set(row, 1, copper ? "Copper" : "Core");
        set(row, 2, copper ? "35" : "100");
        set(row, 3, "Synthetic user assumption");
        set(row, 4, copper ? "" : "4.2");
        set(row, 5, copper ? "" : "0.02");
        set(row, 6, copper ? "58000000" : "");
        set(row, 7, row == 3 ? "Ground" : (copper ? "Signal" : "Unknown"));
        set(row, 9, "SYNTHETIC_UNQUALIFIED");
        set(row, 10, "e011-v1");
        set(row, 11, "user-assumption-test");
        set(row, 12, "1000000000");
        set(row, 13, "25");
        set(row, 14, "1000000");
        set(row, 15, "2000000000");
    }
    set(0, 8, "4");
    editor->setAuditFrequency("1000000000");
    const bool audited = editor->validate().startsWith("INPUT FIELDS VALID") &&
        editor->auditCanonical() && editor->currentAuditIsFresh() &&
        editor->summary().contains("RETURN PATH UNSUPPORTED") &&
        editor->summary().contains("SI NOT_RUN");
    const QString firstRevision = editor->auditScenarioRevision();
    if (!audited || firstRevision.size() != 64) {
        qWarning("E011_FAIL:rust-assumed-audit");
        return 1;
    }

    // Saving is local, atomic and source-bound; reloading can never retain
    // the previous successful Rust audit receipt.
    const bool saved = editor->saveTo(overlay);
    set(1, 5, "0.03");
    const bool editedInvalidated = !editor->currentAuditIsFresh() &&
        editor->auditScenarioRevision().isEmpty();
    const bool loaded = editor->loadFrom(overlay) &&
        editor->table()->item(1, 5)->text() == "0.02" &&
        !editor->currentAuditIsFresh() &&
        editor->auditScenarioRevision().isEmpty();
    const bool reaudited = editor->auditCanonical() &&
        editor->currentAuditIsFresh() &&
        editor->auditScenarioRevision() == firstRevision;
    if (!saved || !editedInvalidated || !loaded || !reaudited) {
        qWarning("E011_FAIL:save-reload-revision");
        return 1;
    }
    qInfo("E011_STAGE:source-a-persisted-revalidated");

    // A malformed JSON must not mutate any input cell or fresh audit state.
    if (!writeFixture("corrupt.json", QByteArray("{broken"))) return 2;
    const bool corruptRejected = !editor->loadFrom(corrupt) &&
        editor->table()->item(1, 5)->text() == "0.02" &&
        editor->currentAuditIsFresh() &&
        editor->auditScenarioRevision() == firstRevision;
    set(1, 5, "-0.1");
    const bool badDfBlocked = editor->validate().contains("INVALID DK/DF") &&
        !editor->auditCanonical() && !editor->currentAuditIsFresh();
    set(1, 5, "0.02");
    const bool recovered = editor->auditCanonical() &&
        editor->auditScenarioRevision() == firstRevision;
    if (!corruptRejected || !badDfBlocked || !recovered) {
        qWarning("E011_FAIL:invalid-input-transaction");
        return 1;
    }

    // Source B has identical layer count but a different SHA-256 and Net.
    // No A overlay, reference choice or successful audit may migrate to B.
    const bool importedB = runImport(pathB, true);
    const bool foreignRejected = importedB &&
        editor->table()->rowCount() == 4 &&
        editor->table()->item(0, 1)->text() == "Unknown" &&
        !editor->loadFrom(overlay) &&
        editor->validate().contains("RUN BLOCKED") &&
        editor->auditScenarioRevision().isEmpty();
    if (!foreignRejected) {
        qWarning("E011_FAIL:foreign-source-reimport");
        return 1;
    }
    qInfo("E011_STAGE:distinct-source-invalidated");

    const bool importedAAgain = runImport(pathA, true);
    const bool restoredA = importedAAgain && editor->loadFrom(overlay) &&
        !editor->currentAuditIsFresh() && editor->auditCanonical() &&
        editor->currentAuditIsFresh() &&
        editor->auditScenarioRevision() == firstRevision;
    if (!restoredA) {
        qWarning("E011_FAIL:source-a-restoration");
        return 1;
    }

    // A previous import receipt must be rejected if the disk bytes mutate,
    // even when the source path and layer count stay the same.
    QFile mutate(pathA);
    const bool mutationWritten = mutate.open(QIODevice::WriteOnly | QIODevice::Append) &&
        mutate.write("\n!E011_SOURCE_MUTATED\n") > 0 && mutate.flush();
    mutate.close();
    const bool mutationRejected = mutationWritten &&
        !editor->currentAuditIsFresh() &&
        editor->auditScenarioRevision().isEmpty() &&
        !editor->saveTo(overlay) && !editor->loadFrom(overlay) &&
        !editor->auditCanonical();
    if (!mutationRejected) {
        qWarning("E011_FAIL:post-import-source-mutation");
        return 1;
    }

    // Even a caller-supplied wrong accepted SHA cannot initialize an overlay.
    const QByteArray shaB = QCryptographicHash::hash(sourceB, QCryptographicHash::Sha256).toHex();
    editor->resetForImport(pathA, 4, shaB);
    const bool pinnedShaRejected = editor->table()->rowCount() == 0 &&
        !editor->currentAuditIsFresh();
    const bool malformedRejected = runImport(badPath, false) &&
        editor->table()->rowCount() == 0 &&
        !editor->currentAuditIsFresh() &&
        editor->auditScenarioRevision().isEmpty() &&
        editor->validate().contains("RUN BLOCKED");
    if (!pinnedShaRejected || !malformedRejected) {
        qWarning("E011_FAIL:pinned-sha-or-malformed-import");
        return 1;
    }
    qInfo("E011_STAGE:failed-import-clears-audit");

    const QJsonDocument receipt(QJsonObject{
        {"schema", "boardsi-e011-main-stackup-test-v1"},
        {"pass", true},
        {"canonical_source_unknown", true},
        {"unknown_overlay_roundtrip_not_solver_ready", unknownRoundtrip},
        {"rust_assumed_audit", audited},
        {"save_load_reaudit", saved && editedInvalidated && loaded && reaudited},
        {"malformed_json_atomic_rejection", corruptRejected},
        {"negative_df_fail_closed", badDfBlocked},
        {"foreign_same_layer_count_rejected", foreignRejected},
        {"source_a_reimport_revalidates", restoredA},
        {"disk_source_mutation_invalidates", mutationRejected},
        {"pinned_import_sha_verified", pinnedShaRejected},
        {"unsupported_header_clears_overlay", malformedRejected},
        {"physical_si", "NOT_RUN"},
        {"reference_continuity", "UNSUPPORTED"},
        {"solver_ready", false}
    });
    QFile out("qt-e011-stackup-results.json");
    if (!out.open(QIODevice::WriteOnly) ||
        out.write(receipt.toJson(QJsonDocument::Indented)) < 1) return 2;
    out.close();
    qInfo("E011_MAIN_STACKUP:PASS");
    return 0;
}
