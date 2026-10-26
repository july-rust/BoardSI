// B_001: explicit local stackup/reference overlay for the restricted PADS Preview.
// Never infer copper, material or electrical continuity from a PADS layer number.
#pragma once
#include <QCryptographicHash>
#include <QFile>
#include <QFileInfo>
#include <QHeaderView>
#include <QFileDialog>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
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
    QString sourcePath_;
    static constexpr int Columns = 10;
    // Layer ID / kind / thickness in micrometres / material / Dk / Df /
    // conductivity S/m / net-role hint / reference layer / source document.
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
        if (!f.open(QIODevice::ReadOnly)) return {};
        QCryptographicHash hash(QCryptographicHash::Sha256);
        while (!f.atEnd()) {
            const QByteArray chunk = f.read(65536);
            if (chunk.isEmpty() || !hash.addData(chunk)) return {};
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
        rows_->setHorizontalHeaderLabels({"Layer ID", "Kind", "Thickness (um)",
            "Material", "Dk", "Df", "Conductivity (S/m)", "Role hint",
            "Ref layer ID", "Property source"});
        rows_->horizontalHeader()->setSectionResizeMode(QHeaderView::ResizeToContents);
        rows_->setMinimumHeight(160);
        layout->addWidget(rows_);
        message_ = new QLabel("NO SOURCE / PHYSICAL SI NOT_RUN", this);
        message_->setObjectName("b001StackupStatus");
        message_->setWordWrap(true);
        layout->addWidget(message_);
        auto *commands = new QHBoxLayout;
        auto *check = new QPushButton("Validate input", this);
        auto *save = new QPushButton("Save overlay...", this);
        auto *load = new QPushButton("Load overlay...", this);
        commands->addWidget(check);
        commands->addWidget(save);
        commands->addWidget(load);
        layout->addLayout(commands);
        connect(check, &QPushButton::clicked, this, [this] { status(validate()); });
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
    QString summary() const { return message_->text(); }
    void resetForImport(const QString &path, int layerCount) {
        // Begin reimport by calling with 0; fail-closed regardless of import outcome.
        sourcePath_ = layerCount > 0 ? path : QString();
        rows_->setRowCount(0);
        if (sourcePath_.isEmpty() || layerCount < 1 || layerCount > 64) {
            status("NO QUALIFIED STACKUP / SI NOT_RUN");
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
            if (!positive(field(r, 2)))
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
        if (digest.size() != 64) return false;
        QJsonArray layers;
        for (int r = 0; r < rows_->rowCount(); ++r) {
            QJsonArray row;
            for (int c = 0; c < Columns; ++c) row.append(field(r, c));
            layers.append(row);
        }
        const QJsonObject root{{"schema", "boardsi-b001-stackup-overlay-v1"},
            {"source_sha256", QString::fromLatin1(digest)}, {"layers", layers},
            {"scope", "USER_OVERLAY_NOT_PHYSICAL_SI"}};
        QSaveFile output(path);
        if (!output.open(QIODevice::WriteOnly)) return false;
        const QByteArray bytes = QJsonDocument(root).toJson(QJsonDocument::Indented);
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
        if (digest.size() != 64 ||
            root.value("schema").toString() != "boardsi-b001-stackup-overlay-v1" ||
            root.value("source_sha256").toString().toLatin1() != digest ||
            root.value("scope").toString() != "USER_OVERLAY_NOT_PHYSICAL_SI") return false;
        const QJsonArray layers = root.value("layers").toArray();
        if (layers.size() != rows_->rowCount()) return false;
        // Validate everything before mutating visible state.
        for (int r = 0; r < layers.size(); ++r) {
            if (!layers[r].isArray()) return false;
            const QJsonArray values = layers[r].toArray();
            if (values.size() != Columns ||
                values[0].toString() != QString::number(r + 1)) return false;
            for (const auto &v : values)
                if (!v.isString() || v.toString().size() > 512) return false;
        }
        for (int r = 0; r < layers.size(); ++r) {
            const QJsonArray values = layers[r].toArray();
            for (int c = 1; c < Columns; ++c) put(r, c, values[c].toString());
        }
        return true;
    }
};
