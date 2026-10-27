// B_001: explicit local stackup/reference overlay for the restricted PADS Preview.
// Never infer copper, material or electrical continuity from a PADS layer number.
#pragma once
#include <QCryptographicHash>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QHeaderView>
#include <QFileDialog>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonParseError>
#include <QLineEdit>
#include <QProcess>
#include <QCoreApplication>
#include <QLabel>
#include <QPushButton>
#include <QSaveFile>
#include <QTableWidget>
#include <QVBoxLayout>
#include <QWidget>
#include <QtGlobal>
#include <cmath>

class B001StackupEditor final : public QWidget {
    QTableWidget *rows_ = nullptr;
    QLabel *message_ = nullptr;
    QLineEdit *auditFrequency_ = nullptr;
    QLineEdit *signalNet_ = nullptr;
    QLineEdit *referenceNet_ = nullptr;
    QJsonArray acceptedCopperInventory_;
    QJsonArray acceptedNetLabels_;
    bool acceptedNetInventoryComplete_ = false;
    QString sourcePath_;
    QByteArray importDigest_; // Pinned at successful import; never trust a later path-only match.
    static constexpr int Columns = 16;
    // Layer ID / kind / thickness in micrometres / material / Dk / Df /
    // conductivity S/m / net-role hint / reference layer / source document / revision /
    // method / reference frequency / temperature / material band min/max.
    QString field(int r, int c) const {
        const auto *item = rows_->item(r, c);
        return item ? item->text().trimmed() : QString();
    }
    static bool positive(const QString &s) {
        bool ok = false;
        const double v = s.toDouble(&ok);
        return ok && std::isfinite(v) && v > 0;
    }
    static bool nonnegative(const QString &s) {
        bool ok = false;
        const double v = s.toDouble(&ok);
        return ok && std::isfinite(v) && v >= 0;
    }
    static QByteArray sourceDigest(const QString &path) {
        QFile f(path);
        if (!f.open(QIODevice::ReadOnly) || f.size() > 64 * 1024 * 1024) return {};
        QCryptographicHash hash(QCryptographicHash::Sha256);
        while (!f.atEnd()) {
            const QByteArray chunk = f.read(65536);
            if (chunk.isEmpty()) return {};
            hash.addData(chunk);
        }
        return hash.result().toHex();
    }
    void put(int r, int c, const QString &value) {
        auto *item = new QTableWidgetItem(value);
        if (c == 0) item->setFlags(item->flags() & ~Qt::ItemIsEditable);
        rows_->setItem(r, c, item);
    }
    void status(const QString &s) { message_->setText(s); }
public:
    explicit B001StackupEditor(QWidget *parent = nullptr) : QWidget(parent) {
        setObjectName("b001StackupEditor");
        auto *layout = new QVBoxLayout(this);
        auto *title = new QLabel(
            "STACKUP / REFERENCE OVERLAY — USER INPUT ONLY\n"
            "No vendor material imported; physical connectivity and SI remain NOT_VERIFIED.", this);
        title->setWordWrap(true);
        layout->addWidget(title);
        rows_ = new QTableWidget(0, Columns, this);
        rows_->setObjectName("b001StackupTable");
        rows_->setHorizontalHeaderLabels({"Layer ID", "Kind", "Thickness (integer um)",
            "Material", "Dk", "Df", "Conductivity (S/m)", "Role hint",
            "Ref layer ID", "Source document", "Source revision",
            "Measurement method", "Ref frequency (Hz)", "Temp (C)",
            "Valid from (Hz)", "Valid to (Hz)"});
        rows_->horizontalHeader()->setSectionResizeMode(QHeaderView::ResizeToContents);
        rows_->setMinimumHeight(160);
        layout->addWidget(rows_);
        auditFrequency_ = new QLineEdit(this);
        auditFrequency_->setObjectName("b001AuditFrequencyHz");
        auditFrequency_->setPlaceholderText("Analysis frequency in Hz (explicit, required for Rust audit)");
        layout->addWidget(auditFrequency_);
        signalNet_ = new QLineEdit(this);
        signalNet_->setObjectName("b001SignalNet");
        signalNet_->setPlaceholderText("Signal Net (exact imported name; optional)");
        layout->addWidget(signalNet_);
        referenceNet_ = new QLineEdit(this);
        referenceNet_->setObjectName("b001ReferenceNet");
        referenceNet_->setPlaceholderText("Reference Net (exact imported name; not a layer role hint)");
        layout->addWidget(referenceNet_);
        message_ = new QLabel("NO SOURCE / PHYSICAL SI NOT_RUN", this);
        message_->setObjectName("b001StackupStatus");
        message_->setWordWrap(true);
        layout->addWidget(message_);
        // Any user edit or changed audit frequency invalidates the previous
        // Rust receipt immediately: no stale 'ASSUMED' result survives a scenario edit.
        connect(rows_, &QTableWidget::itemChanged, this, [this](QTableWidgetItem *) {
            status("STACKUP INPUT MODIFIED / RE-AUDIT REQUIRED / SI NOT_RUN");
        });
        connect(auditFrequency_, &QLineEdit::textChanged, this, [this](const QString &) {
            status("AUDIT FREQUENCY MODIFIED / RE-AUDIT REQUIRED / SI NOT_RUN");
        });
        for (auto *entry : {signalNet_, referenceNet_}) {
            connect(entry, &QLineEdit::textChanged, this, [this](const QString &) {
                status("REFERENCE NET SELECTION MODIFIED / RE-AUDIT REQUIRED / SI NOT_RUN");
            });
        }
        auto *commands = new QHBoxLayout;
        auto *check = new QPushButton("Validate input", this);
        auto *canonical = new QPushButton("Audit in Rust Canonical Model", this);
        canonical->setObjectName("b001CanonicalAudit");
        auto *save = new QPushButton("Save overlay...", this);
        auto *load = new QPushButton("Load overlay...", this);
        commands->addWidget(check);
        commands->addWidget(canonical);
        commands->addWidget(save);
        commands->addWidget(load);
        layout->addLayout(commands);
        connect(check, &QPushButton::clicked, this, [this] { status(validate()); });
        connect(canonical, &QPushButton::clicked, this, [this] {
            if (!auditCanonical()) status("CANONICAL AUDIT BLOCKED / SI NOT_RUN");
        });
        connect(save, &QPushButton::clicked, this, [this] {
            const QString path = QFileDialog::getSaveFileName(
                this, "Save local stackup overlay", QString(), "BoardSI overlay (*.json)");
            if (!path.isEmpty()) status(saveTo(path) ? "OVERLAY SAVED / SI NOT_RUN" : "SAVE REJECTED");
        });
        connect(load, &QPushButton::clicked, this, [this] {
            const QString path = QFileDialog::getOpenFileName(
                this, "Load local stackup overlay", QString(), "BoardSI overlay (*.json)");
            if (!path.isEmpty()) status(loadFrom(path) ? validate() : "LOAD REJECTED / SOURCE MISMATCH");
        });
    }
    QTableWidget *table() const { return rows_; }
    void setAuditFrequency(const QString &hz) { auditFrequency_->setText(hz); }
    void selectReferenceNets(const QString &signal, const QString &reference) {
        signalNet_->setText(signal);
        referenceNet_->setText(reference);
    }
    void setSourceCopperInventory(const QJsonArray &copper, const QJsonArray &nets, bool complete) {
        // Only the import controller may call this after accepted decode.
        acceptedCopperInventory_ = copper;
        acceptedNetLabels_ = nets;
        acceptedNetInventoryComplete_ = complete;
        status("SOURCE COPPER INVENTORY UPDATED / REFERENCE NOT_PROVED / SI NOT_RUN");
    }
    QString summary() const { return message_->text(); }
    void resetForImport(const QString &path, int layerCount) {
        // Begin reimport by calling with 0; fail-closed regardless of import outcome.
        sourcePath_ = layerCount > 0 ? path : QString();
        importDigest_.clear();
        acceptedCopperInventory_ = QJsonArray();
        acceptedNetLabels_ = QJsonArray();
        acceptedNetInventoryComplete_ = false;
        signalNet_->clear();
        referenceNet_->clear();
        rows_->setRowCount(0);
        if (sourcePath_.isEmpty() || layerCount < 1 || layerCount > 64) {
            status("NO QUALIFIED STACKUP / SI NOT_RUN");
            return;
        }
        importDigest_ = sourceDigest(sourcePath_);
        if (importDigest_.size() != 64) {
            sourcePath_.clear();
            status("SOURCE HASH UNAVAILABLE OR TOO LARGE / RUN BLOCKED");
            return;
        }
        rows_->setRowCount(layerCount);
        for (int r = 0; r < layerCount; ++r) {
            put(r, 0, QString::number(r + 1));
            put(r, 1, "Unknown");
            for (int c = 2; c < Columns; ++c)
                put(r, c, c == 7 ? "Unknown" : QString());
        }
        status("IMPORTED LAYER NUMBERS ONLY / MATERIALS UNKNOWN / SI NOT_RUN");
    }
    QString validate() const {
        if (sourcePath_.isEmpty() || rows_->rowCount() == 0)
            return "MISSING SOURCE / RUN BLOCKED";
        for (int r = 0; r < rows_->rowCount(); ++r) {
            const QString kind = field(r, 1);
            if (kind != "Copper" && kind != "Core" &&
                kind != "Prepreg" && kind != "Mask")
                return QString("L%1: UNKNOWN LAYER KIND / RUN BLOCKED").arg(r + 1);
            if (!positive(field(r, 2)) || !std::isfinite(field(r, 2).toDouble()) ||
                std::floor(field(r, 2).toDouble()) != field(r, 2).toDouble())
                return QString("L%1: INVALID OR MISSING THICKNESS / RUN BLOCKED").arg(r + 1);
            if (field(r, 3).isEmpty() || field(r, 9).isEmpty())
                return QString("L%1: MISSING MATERIAL/PROVENANCE / RUN BLOCKED").arg(r + 1);
            if (kind == "Copper") {
                if (!positive(field(r, 6)))
                    return QString("L%1: INVALID CONDUCTIVITY / RUN BLOCKED").arg(r + 1);
            } else if (!positive(field(r, 4)) || !nonnegative(field(r, 5))) {
                return QString("L%1: INVALID DK/DF / RUN BLOCKED").arg(r + 1);
            }
            const QString role = field(r, 7);
            if (role != "Signal" && role != "Ground" && role != "Power" &&
                role != "Floating" && role != "Unknown")
                return QString("L%1: INVALID ROLE HINT / RUN BLOCKED").arg(r + 1);
            if (kind != "Copper" && role != "Unknown")
                return QString("L%1: NON-COPPER ROLE UNSUPPORTED / RUN BLOCKED").arg(r + 1);
            const QString reference = field(r, 8);
            if (!reference.isEmpty()) {
                bool ok = false;
                const int ref = reference.toInt(&ok);
                if (!ok || ref < 1 || ref > rows_->rowCount() ||
                    ref == r + 1 || field(ref - 1, 1) != "Copper" ||
                    kind != "Copper")
                    return QString("L%1: INVALID REFERENCE LAYER / RUN BLOCKED").arg(r + 1);
            }
        }
        return "INPUT FIELDS VALID / REFERENCE CONTACT NOT_VERIFIED / PHYSICAL SI RUN BLOCKED";
    }
    bool saveTo(const QString &path) const {
        if (sourcePath_.isEmpty() || rows_->rowCount() < 1 ||
            rows_->rowCount() > 64) return false;
        const QByteArray digest = sourceDigest(sourcePath_);
        if (digest.size() != 64 || digest != importDigest_) return false;
        // Reject destructive save destinations, including source symlinks or aliases
        // through symlinked parent directories. Never replace the imported board.
        const QString sourceCanonical = QFileInfo(sourcePath_).canonicalFilePath();
        const QFileInfo destination(path);
        const QString targetCanonical = destination.canonicalFilePath();
        const QString parentCanonical = QDir(destination.absolutePath()).canonicalPath();
        if (sourceCanonical.isEmpty() || parentCanonical.isEmpty() || destination.isDir())
            return false;
#ifdef Q_OS_WIN
        constexpr auto pathCase = Qt::CaseInsensitive;
#else
        constexpr auto pathCase = Qt::CaseSensitive;
#endif
        const QString proposed = QDir::cleanPath(QDir(parentCanonical).filePath(destination.fileName()));
        if ((!targetCanonical.isEmpty() &&
             QString::compare(sourceCanonical, targetCanonical, pathCase) == 0) ||
            QString::compare(sourceCanonical, proposed, pathCase) == 0)
            return false;
        QJsonArray layers;
        for (int r = 0; r < rows_->rowCount(); ++r) {
            QJsonArray row;
            for (int c = 0; c < Columns; ++c) {
                const QString value = field(r, c);
                if (value.size() > 512) return false;
                row.append(value);
            }
            layers.append(row);
        }
        const QJsonObject root{{"schema", "boardsi-b001-stackup-overlay-v2"},
            {"source_sha256", QString::fromLatin1(digest)}, {"layers", layers},
            {"audit_frequency_hz", auditFrequency_->text().trimmed()},
            {"selected_signal_net", signalNet_->text().trimmed()},
            {"selected_reference_net", referenceNet_->text().trimmed()},
            {"scope", "USER_OVERLAY_NOT_PHYSICAL_SI"}};
        const QByteArray bytes = QJsonDocument(root).toJson(QJsonDocument::Indented);
        if (bytes.size() > 65536) return false; // symmetric with loadFrom()
        QSaveFile output(path);
        if (!output.open(QIODevice::WriteOnly)) return false;
        if (output.write(bytes) != bytes.size()) {
            output.cancelWriting();
            return false;
        }
        return output.commit();
    }
    bool loadFrom(const QString &path) {
        if (sourcePath_.isEmpty() || rows_->rowCount() < 1) return false;
        QFile input(path);
        if (!input.open(QIODevice::ReadOnly) || input.size() > 65536) return false;
        QJsonParseError error;
        const QJsonDocument doc = QJsonDocument::fromJson(input.readAll(), &error);
        if (error.error != QJsonParseError::NoError || !doc.isObject()) return false;
        const QJsonObject root = doc.object();
        const QByteArray digest = sourceDigest(sourcePath_);
        if (digest.size() != 64 || digest != importDigest_ ||
            (root.value("schema").toString() != "boardsi-b001-stackup-overlay-v2" &&
             root.value("schema").toString() != "boardsi-b001-stackup-overlay-v1") ||
            root.value("source_sha256").toString().toLatin1() != digest ||
            root.value("scope").toString() != "USER_OVERLAY_NOT_PHYSICAL_SI") return false;
        const QJsonArray layers = root.value("layers").toArray();
        if (layers.size() != rows_->rowCount()) return false;
        const bool legacy = root.value("schema").toString() == "boardsi-b001-stackup-overlay-v1";
        // Validate everything before mutating visible state.
        for (int r = 0; r < layers.size(); ++r) {
            if (!layers[r].isArray()) return false;
            const QJsonArray values = layers[r].toArray();
            if (values.size() != (legacy ? 10 : Columns) ||
                values[0].toString() != QString::number(r + 1)) return false;
            for (const auto &v : values)
                if (!v.isString() || v.toString().size() > 512) return false;
        }
        for (int r = 0; r < layers.size(); ++r) {
            const QJsonArray values = layers[r].toArray();
            for (int c = 1; c < Columns; ++c)
                put(r, c, c < values.size() ? values[c].toString() : QString());
        }
        auditFrequency_->setText(legacy ? QString() :
            root.value("audit_frequency_hz").toString());
        const QString storedSignal = root.value("selected_signal_net").toString();
        const QString storedReference = root.value("selected_reference_net").toString();
        if (storedSignal.size() > 64 || storedReference.size() > 64) return false;
        // Explicit choices are user input, not source-certified Net roles.
        signalNet_->setText(legacy ? QString() : storedSignal);
        referenceNet_->setText(legacy ? QString() : storedReference);
        status("OVERLAY LOADED / CANONICAL MUST BE RE-AUDITED / SI NOT_RUN");
        return true;
    }
    bool auditCanonical() {
        if (!validate().startsWith("INPUT FIELDS VALID")) return false;
        bool valid = false;
        const double freq = auditFrequency_->text().trimmed().toDouble(&valid);
        if (!valid || !std::isfinite(freq) || freq < 0.0) return false;
        const QByteArray digest = sourceDigest(sourcePath_);
        if (digest != importDigest_ || digest.size() != 64) return false;
        QJsonArray layers;
        for (int r = 0; r < rows_->rowCount(); ++r) {
            QJsonArray row;
            for (int c = 0; c < Columns; ++c) {
                const QString value = field(r, c);
                if (value.size() > 512) return false;
                row.append(value);
            }
            layers.append(row);
        }
        const QJsonDocument request(QJsonObject{
            {"schema", "boardsi-b001-stackup-overlay-v2"},
            {"source_sha256", QString::fromLatin1(digest)},
            {"scope", "USER_OVERLAY_NOT_PHYSICAL_SI"},
            {"audit_frequency_hz", auditFrequency_->text().trimmed()},
            {"selected_signal_net", signalNet_->text().trimmed()},
            {"selected_reference_net", referenceNet_->text().trimmed()},
            {"reference_copper_inventory", acceptedCopperInventory_},
            {"source_net_labels", acceptedNetLabels_},
            {"net_inventory_complete", acceptedNetInventoryComplete_},
            {"layers", layers}});
        const QByteArray bytes = request.toJson(QJsonDocument::Compact);
        if (bytes.size() > 65536) return false;
        QString executable = qEnvironmentVariable("BOARDSI_STACKUP_VALIDATOR");
        if (executable.isEmpty()) {
#ifdef Q_OS_WIN
            executable = QCoreApplication::applicationDirPath() + "/boardsi-stackup-config.exe";
#else
            executable = QCoreApplication::applicationDirPath() + "/boardsi-stackup-config";
#endif
        }
        if (!QFileInfo(executable).isExecutable()) return false;
        QProcess process;
        process.setProgram(executable);
        process.setArguments({"--audit-stdin-v1"});
        process.start();
        if (!process.waitForStarted(2500)) return false;
        if (process.write(bytes) != bytes.size()) { process.kill(); process.waitForFinished(1000); return false; }
        process.closeWriteChannel();
        if (!process.waitForFinished(5000)) { process.kill(); process.waitForFinished(1000); return false; }
        const QByteArray resultBytes = process.readAllStandardOutput();
        if (process.exitStatus() != QProcess::NormalExit || resultBytes.size() > 4096)
            return false;
        QJsonParseError error;
        const QJsonDocument result = QJsonDocument::fromJson(resultBytes, &error);
        if (error.error != QJsonParseError::NoError || !result.isObject()) return false;
        const QJsonObject auditFields = result.object();
        if (process.exitCode() != 0) {
            // Display only the native fixed reason code, never raw stdout or
            // proprietary Net/source names. This is not a solver waiver.
            const QString reason = auditFields.value("reason").toString();
            if (!reason.isEmpty() && reason.size() <= 64) {
                bool codeOnly = true;
                for (const QChar c : reason)
                    if ((c < 'A' || c > 'Z') && (c < '0' || c > '9') && c != '_')
                        codeOnly = false;
                if (codeOnly)
                    status("RUST REJECTED / " + reason + " / SI NOT_RUN");
            }
            return false;
        }
        if (auditFields.value("schema").toString() != "boardsi-b001-canonical-audit-v1" ||
            auditFields.value("canonical_stackup").toString() != "ASSUMED" ||
            auditFields.value("reference_continuity").toString() != "UNSUPPORTED_NO_COPPER_NET_EVIDENCE" ||
            auditFields.value("solver_ready").toBool(true) ||
            auditFields.value("source_sha256").toString().toLatin1() != digest ||
            auditFields.value("layer_count").toInt() != rows_->rowCount()) return false;
        const QString candidate = auditFields.value("reference_candidate").toString();
        if (candidate != "NOT_SELECTED" &&
            candidate != "NO_SOURCE_REFERENCE_EVIDENCE" &&
            candidate != "UNKNOWN_OR_MISSING_SOURCE_NET" &&
            candidate != "OBSERVED_LABELS_NO_TOPOLOGY_PROOF" &&
            candidate != "CUTOUT_OR_SPLIT_UNSUPPORTED") return false;
        status("RUST CANONICAL STACKUP ASSUMED / SOURCE REFERENCE " + candidate +
               " / RETURN PATH UNSUPPORTED / SI NOT_RUN");
        return true;
    }
};
