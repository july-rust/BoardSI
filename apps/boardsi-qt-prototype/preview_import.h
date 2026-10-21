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
        const qint64 remaining = 2 * 1024 * 1024 - data_.size();
        if (remaining > 0) data_ += process_.read(qMin<qint64>(remaining, 65536));
        if (process_.bytesAvailable() > 0) { overflow_ = true; process_.kill(); }
    }
    static bool decode(const QByteArray &bytes, int &layers, int &skipped,
                       std::vector<ImportedTrack> &tracks) {
        if (bytes.size() > 2 * 1024 * 1024 || !bytes.endsWith('\n')) return false;
        const auto lines = QString::fromUtf8(bytes).split('\n', Qt::SkipEmptyParts);
        if (lines.size() < 5 || lines.size() > 10040) return false;
        const auto header = lines.first().split('|');
        bool ok = false;
        if (header.size() != 3 || header[0] != "BOARDSI_P2A_PARTIAL_V1" ||
            header[1] != "V9.4-MILS") return false;
        layers = header[2].toInt(&ok);
        if (!ok || layers < 2 || layers > 32) return false;
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
            } else if (parts.size() == 5 && parts[0] == "END" &&
                       i == lines.size() - 1 && layerRecords == layers) {
                const int count = parts[1].toInt(&ok);
                if (!ok || count != static_cast<int>(tracks.size())) return false;
                skipped = parts[2].toInt(&ok);
                if (!ok || skipped < 0 || parts[3] != "CANONICAL_PARTIAL" ||
                    parts[4] != "SI_NOT_RUN") return false;
                end = true;
            } else return false;
        }
        return end && !tracks.empty();
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
            const qint64 budget = 256 - errors_.size();
            if (budget > 0) errors_.append(process_.read(qMin<qint64>(budget, 256)));
            process_.readAllStandardError();
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
                status("FAILED / local import timeout or excessive output / NOTHING IMPORTED");
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
            if (!decode(data_, layers, skipped, tracks) ||
                !imported_->load(layers, skipped, tracks)) {
                status("FAILED / invalid Canonical partial transport / NOTHING IMPORTED");
                return;
            }
            tabs_->setTabText(1, "Imported Partial");
            tabs_->setCurrentWidget(imported_);
            status(QString("PARTIAL V9.4-MILS / %1 straight segments / %2 omitted / SI NOT_RUN")
                       .arg(static_cast<qlonglong>(tracks.size())).arg(skipped));
        });
        connect(&process_, &QProcess::errorOccurred, this,
                [this](QProcess::ProcessError code) {
            if (code == QProcess::FailedToStart) {
                deadline_.stop();
                status("FAILED / local importer process unavailable");
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
    QString importStatus() const { return status_; }
    void setStatusCallback(std::function<void(const QString &)> cb) { callback_ = std::move(cb); }
    bool openPath(const QString &path) {
        if (process_.state() != QProcess::NotRunning) return false;
        if (!QFileInfo(binary_).isAbsolute() || !QFileInfo(binary_).isExecutable() ||
            !QFileInfo(path).isFile()) {
            status("FAILED / unsupported local file or importer unavailable");
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
