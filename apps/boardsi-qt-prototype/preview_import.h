// BoardSI local PADS import controller. Never uploads source files.
// Only restricted V9.4-MILS straight segments -> partial Canonical -> Qt, SI NOT_RUN.
#pragma once
#include "imported_artwork.h"
#include <QCoreApplication>
#include <QFileInfo>
#include <QProcess>
#include <QTimer>
#include <QTabWidget>
#include <QVBoxLayout>
#include <QDebug>
#include <QPushButton>
#include <QLabel>
#include <QHBoxLayout>
#include <functional>
#include <vector>
#include <utility>

class PreviewImportPanel final : public QWidget {
    QTabWidget *tabs_;
    QPushButton *cancelButton_ = nullptr;
    QLabel *importLabel_ = nullptr;
    bool cancelled_ = false;
    SyntheticArtworkPanel *synthetic_;
    ImportedArtworkPanel *imported_;
    QProcess process_;
    QTimer deadline_;
    QByteArray data_;
    QByteArray errors_; // Small bounded diagnostic (fixed importer error codes only).
    QString binary_;
    QString status_ = "NOT IMPORTED / SI NOT_RUN";
    std::function<void(const QString &)> callback_;
    bool expired_ = false, overflow_ = false;
    void status(const QString &message) {
        status_ = message;
        if (importLabel_) importLabel_->setText(message);
        if (cancelButton_) cancelButton_->setEnabled(message.startsWith("RUNNING"));
        if (callback_) callback_(status_);
    }
    void collect() {
        // stdout may arrive in arbitrarily large batches. The cap is cumulative,
        // not per readyRead signal; drain each available chunk before deciding overflow.
        while (process_.bytesAvailable() > 0) {
            const qint64 remaining = 2 * 1024 * 1024 - data_.size();
            if (remaining <= 0) {
                overflow_ = true;
                process_.kill();
                return;
            }
            const QByteArray chunk = process_.read(qMin<qint64>(remaining, 65536));
            if (chunk.isEmpty()) break;
            data_.append(chunk);
        }
    }
    static bool decode(const QByteArray &bytes, int &layers, int &skipped,
                       std::vector<ImportedTrack> &tracks,
                       std::vector<ImportedVia> &vias,
                       std::vector<ImportedPad> &pads) {
        if (bytes.size() > 2 * 1024 * 1024 || !bytes.endsWith('\n')) return false;
        const auto lines = QString::fromUtf8(bytes).split('\n', Qt::SkipEmptyParts);
        if (lines.size() < 4 || lines.size() > 25100) return false;
        const auto header = lines.first().split('|');
        bool ok = false;
        const bool p2d = header.size() == 3 && header[0] == "BOARDSI_P2D_PARTIAL_V3";
        const bool p2c = header.size() == 3 &&
                         header[0] == "BOARDSI_P2C_PARTIAL_V2";
        if (header.size() != 3 ||
            (!p2c && !p2d && header[0] != "BOARDSI_P2A_PARTIAL_V1") ||
            header[1] != "V9.4-MILS") return false;
        layers = header[2].toInt(&ok);
        if (!ok || layers < 1 || layers > 64) return false;
        int layerRecords = 0;
        bool end = false;
        for (qsizetype i = 1; i < lines.size(); ++i) {
            const auto parts = lines[i].split('|');
            if (parts.size() == 3 && parts[0] == "L" && layerRecords < layers &&
                static_cast<int>(i) == layerRecords + 1) {
                int id = parts[1].toInt(&ok);
                if (!ok || id != ++layerRecords) return false;
            } else if (parts.size() == 8 && parts[0] == "T" && layerRecords == layers) {
                ImportedTrack t;
                t.layer = parts[1].toInt(&ok);
                if (!ok || t.layer <= 0 || t.layer > layers) return false;
                t.net = parts[2];
                qint64 *fields[] = {&t.x0_nm,&t.y0_nm,&t.x1_nm,&t.y1_nm,&t.width_nm};
                for (int j = 0; j < 5; ++j) {
                    *fields[j] = parts[j+3].toLongLong(&ok);
                    if (!ok) return false;
                }
                if (tracks.size() >= 10000) return false;
                tracks.push_back(std::move(t));
            } else if (parts.size() == 9 && parts[0] == "V" && layerRecords == layers) {
                ImportedVia via;
                via.net = parts[1];
                via.name = parts[2];
                qint64 *measurements[] = {&via.x_nm, &via.y_nm,
                                         &via.diameter_nm, &via.drill_nm};
                for (int j = 0; j < 4; ++j) {
                    *measurements[j] = parts[j+3].toLongLong(&ok);
                    if (!ok) return false;
                }
                via.firstLayer = parts[7].toInt(&ok);
                if (!ok) return false;
                via.lastLayer = parts[8].toInt(&ok);
                if (!ok || vias.size() >= 10000) return false;
                vias.push_back(std::move(via));
            } else if ((p2c || p2d) && parts.size() == 10 && parts[0] == "P" &&
                       layerRecords == layers && parts[9] == "NET_UNVERIFIED") {
                ImportedPad pad;
                pad.reference = parts[1]; pad.pin = parts[2];
                pad.layer = parts[3].toInt(&ok);
                if (!ok || pad.layer != 1) return false;
                pad.shape = parts[4];
                if (pad.shape != "R" && pad.shape != "S" && !(p2d && pad.shape == "RF")) return false;
                qint64 *v[] = {&pad.x_nm, &pad.y_nm, &pad.width_nm, &pad.height_nm};
                for (int j = 0; j < 4; ++j) {
                    *v[j] = parts[j+5].toLongLong(&ok);
                    if (!ok) return false;
                }
                if (pads.size() >= 5000) return false;
                pads.push_back(std::move(pad));
            } else if (parts[0] == "END" && i == lines.size()-1 &&
                       layerRecords == layers &&
                       (((p2c || p2d) && parts.size() == 6) || (!p2c && !p2d && parts.size() == 5))) {
                const int count = parts[1].toInt(&ok);
                if (!ok || count != static_cast<int>(tracks.size())) return false;
                skipped = parts[2].toInt(&ok);
                if (!ok || skipped < 0) return false;
                if (p2c || p2d) {
                    const int padCount = parts[3].toInt(&ok);
                    if (!ok || padCount != static_cast<int>(pads.size()) ||
                        parts[4] != "CANONICAL_PARTIAL" ||
                        parts[5] != "SI_NOT_RUN") return false;
                } else if (parts[3] != "CANONICAL_PARTIAL" ||
                           parts[4] != "SI_NOT_RUN" || !pads.empty()) return false;
                end = true;
            } else return false;
        }
        return end && (!tracks.empty() || !vias.empty() || !pads.empty());
    }
public:
    explicit PreviewImportPanel(const QString &importBinary = QString(), QWidget *parent = nullptr)
        : QWidget(parent), tabs_(new QTabWidget(this)),
          synthetic_(new SyntheticArtworkPanel(tabs_)),
          imported_(new ImportedArtworkPanel(tabs_)) {
        setObjectName("previewImportP2A");
        binary_ = importBinary;
        if (binary_.isEmpty()) {
#ifdef Q_OS_WIN
            binary_ = QCoreApplication::applicationDirPath() + "/boardsi-import-artwork.exe";
#else
            binary_ = QCoreApplication::applicationDirPath() + "/boardsi-import-artwork";
#endif
        }
        auto *layout = new QVBoxLayout(this);
        layout->setContentsMargins(0, 0, 0, 0);
        auto *controls = new QHBoxLayout();
        importLabel_ = new QLabel("NOT IMPORTED / SI NOT_RUN", this);
        importLabel_->setWordWrap(true);
        cancelButton_ = new QPushButton("Cancel Import", this);
        cancelButton_->setObjectName("importCancelButton");
        cancelButton_->setEnabled(false);
        controls->addWidget(importLabel_, 1);
        controls->addWidget(cancelButton_);
        layout->addLayout(controls);
        connect(cancelButton_, &QPushButton::clicked, this, [this] {
            if (process_.state() != QProcess::NotRunning) {
                cancelled_ = true;
                deadline_.stop();
                process_.kill();
                status("CANCELLED / no new board imported / SI NOT_RUN");
            }
        });
        tabs_->addTab(synthetic_, "Synthetic Fixture");
        tabs_->addTab(imported_, "Imported Partial (none)");
        layout->addWidget(tabs_);
        process_.setProcessChannelMode(QProcess::SeparateChannels);
        process_.setReadChannel(QProcess::StandardOutput);
        connect(&process_, &QProcess::readyReadStandardOutput, this, [this] { collect(); });
        connect(&process_, &QProcess::readyReadStandardError, this, [this] {
            // read() follows the selected stdout channel; collect stderr explicitly.
            const QByteArray stderrChunk = process_.readAllStandardError();
            const qint64 budget = 256 - errors_.size();
            if (budget > 0) errors_.append(stderrChunk.left(budget));
        });
        connect(&process_, &QProcess::finished, this,
                [this](int code, QProcess::ExitStatus exitStatus) {
            deadline_.stop();
            collect();
            if (cancelled_) {
                status("CANCELLED / no new board imported / SI NOT_RUN");
                return;
            }
            if (expired_ || overflow_) {
                status(expired_ ? "FAILED / IMPORT_TIMEOUT / NOTHING IMPORTED"
                                : "FAILED / IMPORT_OUTPUT_LIMIT / NOTHING IMPORTED");
                return;
            }
            if (code != 0 || exitStatus != QProcess::NormalExit) {
                const QString diagnostic = QString::fromLatin1(errors_).trimmed();
                const QString prefix = "P2A_IMPORT_ERROR:";
                QString errorCode;
                if (diagnostic.startsWith(prefix)) {
                    errorCode = diagnostic.mid(prefix.size());
                    if (errorCode.size() > 48) errorCode.clear();
                    for (const QChar character : errorCode) {
                        const ushort ch = character.unicode();
                        if (!((ch >= 'A' && ch <= 'Z') ||
                              (ch >= '0' && ch <= '9') || ch == '_')) {
                            errorCode.clear();
                            break;
                        }
                    }
                }
                status(errorCode.isEmpty()
                    ? QString("FAILED / invalid or unsupported local import / NOTHING IMPORTED")
                    : QString("FAILED / %1 / NOTHING IMPORTED").arg(errorCode));
                return;
            }
            int layers = 0, skipped = 0;
            std::vector<ImportedTrack> tracks;
            std::vector<ImportedVia> vias;
            std::vector<ImportedPad> pads;
            if (!decode(data_, layers, skipped, tracks, vias, pads)) {
                status("FAILED / INVALID_TRANSPORT / NOTHING IMPORTED");
                return;
            }
            if (!imported_->load(layers, skipped, tracks, vias, pads)) {
                status("FAILED / INVALID_GEOMETRY / NOTHING IMPORTED");
                return;
            }
            tabs_->setTabText(1, "Imported Partial");
            tabs_->setCurrentWidget(imported_);
            // The imported tab can be hidden during load(), before its viewport
            // has a usable size. Refit once it is visible in the event loop.
            QTimer::singleShot(0, imported_, [this] {
                if (tabs_->currentWidget() == imported_) imported_->fitArtwork();
            });
            status(QString("PARTIAL V9.4-MILS / %1 straight segments / %2 explicit vias"
                           " / %3 SMD pads / %4 omitted / SI NOT_RUN")
                       .arg(static_cast<qlonglong>(tracks.size()))
                       .arg(static_cast<qlonglong>(vias.size()))
                       .arg(static_cast<qlonglong>(pads.size())).arg(skipped));
        });
        connect(&process_, &QProcess::errorOccurred, this,
                [this](QProcess::ProcessError code) {
            if (code == QProcess::FailedToStart) {
                deadline_.stop();
                status("FAILED / IMPORTER_UNAVAILABLE / NOTHING IMPORTED");
            }
        });
        deadline_.setSingleShot(true);
        connect(&deadline_, &QTimer::timeout, this, [this] {
            expired_ = true;
            if (process_.state() != QProcess::NotRunning) process_.kill();
        });
    }
    SyntheticArtworkPanel *synthetic() const { return synthetic_; }
    ImportedArtworkPanel *imported() const { return imported_; }
    QTabWidget *artworkTabs() const { return tabs_; }
    bool showingImported() const { return tabs_->currentWidget() == imported_; }
    QString importStatus() const { return status_; }
    void setStatusCallback(std::function<void(const QString &)> cb) { callback_ = std::move(cb); }
    bool openPath(const QString &path) {
        if (process_.state() != QProcess::NotRunning) return false;
        // An import attempt replaces the candidate, even when start/parse fails.
        // Never show previous geometry with the current "NOTHING IMPORTED" result.
        imported_->clearArtwork();
        tabs_->setTabText(1, "Imported Partial (none)");
        if (!QFileInfo(binary_).isAbsolute() || !QFileInfo(binary_).isExecutable()) {
            status("FAILED / IMPORTER_UNAVAILABLE / NOTHING IMPORTED");
            return false;
        }
        if (!QFileInfo(path).isFile()) {
            status("FAILED / FILE_NOT_FOUND / NOTHING IMPORTED");
            return false;
        }
        data_.clear();
        errors_.clear();
        expired_ = false;
        overflow_ = false;
        cancelled_ = false;
        status("RUNNING / local PADS route import, SI NOT_RUN");
        deadline_.start(10000);
        process_.start(binary_, {path});
        return true;
    }
    void stop() {
        if (process_.state() != QProcess::NotRunning) process_.kill();
    }
    bool busy() const { return process_.state() != QProcess::NotRunning; }
};
