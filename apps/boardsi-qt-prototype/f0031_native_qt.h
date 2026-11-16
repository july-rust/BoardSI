// F_003_1: real native .bsi file QProcess controller, not an artwork decoder.
// The backend owns the exact Canonical model; no Qt drawing is serialized as .bsi.
#pragma once
#include <QCoreApplication>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLabel>
#include <QPlainTextEdit>
#include <QProcess>
#include <QRegularExpression>
#include <QTimer>
#include <QVBoxLayout>
#include <QWidget>
#include <functional>

class F0031NativePanel final : public QWidget {
    QProcess child_;
    QTimer timeout_;
    QLabel *state_;
    QPlainTextEdit *summary_;
    QByteArray output_, errors_;
    QString binary_, operation_, target_, loaded_, acceptedDigest_;
    bool running_=false, cancelled_=false, readOnly_=true;
    std::function<void()> onChanged_;
    void state(const QString &value) {
        state_->setText(value);
        if (onChanged_) onChanged_();
    }
    void stopWithError(const QString &value) {
        if (!running_) return;
        errors_=value.toUtf8().left(256);
        child_.kill();
    }
    void collect(bool error) {
        QByteArray &buffer=error ? errors_ : output_;
        const QByteArray chunk=error ? child_.readAllStandardError()
                                     : child_.readAllStandardOutput();
        const qsizetype limit=error ? 512 : 4096;
        if(chunk.size()>limit-buffer.size()) {
            stopWithError("BSI_WORKER_OUTPUT_LIMIT");
            return;
        }
        buffer+=chunk;
    }
    bool begin(const QString &operation, const QString &target,
               const QStringList &arguments) {
        if(running_ || !QFileInfo(binary_).isExecutable()) {
            state("FAILED / native file worker unavailable or busy");
            return false;
        }
        running_=true; cancelled_=false; output_.clear(); errors_.clear();
        operation_=operation; target_=target;
        timeout_.start(60000);
        state("RUNNING / Native "+operation);
        child_.start(binary_,arguments);
        return true;
    }
    void completed(int exitCode, QProcess::ExitStatus exitStatus) {
        if(!running_) return;
        timeout_.stop();
        collect(false); collect(true);
        running_=false;
        if(cancelled_) {
            state("CANCELLED / outcome unconfirmed; re-open target to verify");
            return;
        }
        if(exitStatus!=QProcess::NormalExit || exitCode!=0) {
            state("FAILED / "+QString::fromUtf8(errors_).trimmed().left(256));
            return;
        }
        QJsonParseError error;
        const auto document=QJsonDocument::fromJson(output_.trimmed(),&error);
        const auto receipt=document.object();
        if(error.error!=QJsonParseError::NoError || !document.isObject() ||
           receipt.value("protocol").toString()!="BSI_NATIVE_FILE_V1" ||
           receipt.value("action").toString()!=operation_ ||
           receipt.value("path").toString()!=target_ ||
           receipt.value("physical_si").toString()!="NOT_RUN" ||
           !receipt.value("geometry_only").toBool(false) ||
           !receipt.value("read_only").isBool() ||
           !QRegularExpression("^[0-9a-f]{64}$")
                .match(receipt.value("sha256").toString()).hasMatch()) {
            state("FAILED / invalid native worker receipt");
            return;
        }
        for(const auto &name:{"traces","arcs","areas","pads","vias","drills","layers","nets","sources"}) {
            if(!receipt.value(QString::fromLatin1(name)).isDouble() ||
               receipt.value(QString::fromLatin1(name)).toDouble()<0) {
                state("FAILED / invalid native object counts");
                return;
            }
        }
        loaded_=target_;
        acceptedDigest_=receipt.value("sha256").toString();
        readOnly_=receipt.value("read_only").toBool();
        QStringList counts;
        for(const auto &name:{"layers","nets","sources","traces","arcs","areas","pads","vias","drills"})
            counts << QString("%1: %2").arg(QString::fromLatin1(name))
                                       .arg(receipt.value(QString::fromLatin1(name)).toInt());
        summary_->setPlainText(
            "NATIVE .bsi / SQLite v1 / STRUCTURALLY LOADED\n"
            "Source: "+loaded_+"\n"+
            counts.join("\n")+"\n\n"+
            (readOnly_ ? "READ ONLY (unpreservable optional extension or evidence)\n" :
                         "Editable native snapshot; Save As requires revalidation\n")+
            "Canonical data is stored exactly in Rust.\n"
            "Native geometry is NOT rendered on this preview tab.\n"
            "Physical connectivity, manufacturing and SI: NOT_VERIFIED / NOT_RUN");
        state("COMPLETED / Native "+operation_);
    }
public:
    explicit F0031NativePanel(const QString &importBinary,
                              const QString &explicitWorker=QString(),
                              QWidget *parent=nullptr) : QWidget(parent) {
        if(!explicitWorker.isEmpty()) binary_=explicitWorker;
        else {
            QString dir=importBinary.isEmpty()?QCoreApplication::applicationDirPath()
                                             :QFileInfo(importBinary).absolutePath();
#ifdef Q_OS_WIN
            binary_=dir+"/boardsi-native-file.exe";
#else
            binary_=dir+"/boardsi-native-file";
#endif
        }
        auto *layout=new QVBoxLayout(this);
        state_=new QLabel("IDLE / Native file not loaded",this);
        state_->setObjectName("f0031NativeState");
        summary_=new QPlainTextEdit(this);
        summary_->setObjectName("f0031NativeSummary");
        summary_->setReadOnly(true);
        summary_->setPlainText("Native .bsi file not loaded. Only structurally verified models are accepted.\n"
                               "Native geometry artwork viewer is NOT IMPLEMENTED in F_003_1.");
        layout->addWidget(state_); layout->addWidget(summary_,1);
        timeout_.setSingleShot(true);
        connect(&timeout_,&QTimer::timeout,this,[this] {
            if(running_) stopWithError("BSI_WORKER_TIMEOUT");
        });
        connect(&child_,&QProcess::readyReadStandardOutput,this,[this] {collect(false);});
        connect(&child_,&QProcess::readyReadStandardError,this,[this] {collect(true);});
        connect(&child_,QOverload<int,QProcess::ExitStatus>::of(&QProcess::finished),
                this,[this](int code,QProcess::ExitStatus why){completed(code,why);});
        connect(&child_,&QProcess::errorOccurred,this,[this](QProcess::ProcessError e) {
            if(running_ && e==QProcess::FailedToStart) {
                timeout_.stop();running_=false;
                state("FAILED / native worker could not start");
            }
        });
    }
    ~F0031NativePanel() override {
        if(child_.state()!=QProcess::NotRunning) child_.kill();
    }
    void setChanged(std::function<void()> cb) {onChanged_=std::move(cb);}
    bool busy() const {return running_;}
    bool hasLoaded() const {return !loaded_.isEmpty();}
    bool isReadOnly() const {return readOnly_;}
    QString loadedPath() const {return loaded_;}
    QString status() const {return state_->text();}
    void forget() {
        if(!running_) {loaded_.clear();acceptedDigest_.clear();readOnly_=true;state("IDLE / source import selected");}
    }
    bool open(const QString &file) {
        if(running_) return false;
        return begin("open",file,{"open",file});
    }
    bool savePads(const QString &source,const QString &target,const QByteArray &sha) {
        if(running_ || sha.size()!=64) return false;
        return begin("save-pads",target,{"save-pads",source,target,QString::fromLatin1(sha)});
    }
    bool saveNative(const QString &target) {
        if(running_ || loaded_.isEmpty() || readOnly_) return false;
        return begin("save-loaded",target,{"save-loaded",loaded_,target,acceptedDigest_});
    }
    void cancel() {
        if(!running_) return;
        cancelled_=true;
        timeout_.stop();
        child_.kill();
    }
};
