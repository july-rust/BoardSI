// F_003_1: real native .bsi file QProcess controller, not an artwork decoder.
// The backend owns the exact Canonical model; no Qt drawing is serialized as .bsi.
#pragma once
#include <QComboBox>
#include <QGraphicsItem>
#include <QGraphicsScene>
#include <QGraphicsView>
#include <QJsonArray>
#include <QPainter>
#include <QPen>
#include <cmath>
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
    QGraphicsScene *artworkScene_;
    QGraphicsView *artworkView_;
    QComboBox *artworkLayer_;
    QByteArray output_, errors_;
    QString binary_, operation_, target_, loaded_, acceptedDigest_, expectedSourceManifest_;
    bool running_=false, cancelled_=false, readOnly_=true;
    int timeoutMs_=60000;
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
        const qsizetype limit=error ? 512 : 65536;
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
        timeout_.start(timeoutMs_);
        state("RUNNING / Native "+operation);
        child_.start(binary_,arguments);
        return true;
    }
    // Bounded and strictly checked Source-only graphical overview.
    // A view item is never an electrically connected model or Solver claim.
    bool showKiCadArtwork(const QJsonObject &body) {
        if(body.value("schema").toString()!="BSI_KICAD_SOURCE_PREVIEW_V1" ||
           body.value("units").toString()!="integer-nm" ||
           body.value("solver_ready").toBool(true) ||
           !body.value("items").isArray() || !body.value("layers").isArray())
            return false;
        const QJsonArray items=body.value("items").toArray();
        const QJsonArray layers=body.value("layers").toArray();
        if(items.isEmpty() || items.size()>128 || layers.size()<2 || layers.size()>32)
            return false;
        QStringList layerNames;
        for(const auto &value:layers) {
            const QString name=value.toObject().value("name").toString();
            if(name.isEmpty() || layerNames.contains(name))return false;
            layerNames<<name;
        }
        auto number=[](const QJsonValue &value,double &mm)->bool {
            if(!value.isDouble())return false;
            const double nm=value.toDouble();
            if(!std::isfinite(nm) || std::floor(nm)!=nm || std::abs(nm)>1.0e12)
                return false;
            mm=nm/1.0e6;
            return true;
        };
        auto point=[&](const QJsonValue &value,QPointF &p)->bool {
            if(!value.isArray())return false;
            const auto coords=value.toArray();
            if(coords.size()!=2)return false;
            double x=0,y=0;
            if(!number(coords.at(0),x) || !number(coords.at(1),y))
                return false;
            p=QPointF(x,y);return true;
        };
        // Build off-scene first, so a malformed item cannot publish a partial
        // artwork. The QGraphicsScene takes item ownership only on success.
        QList<QGraphicsItem *> staged;
        auto cleanup=[&] {qDeleteAll(staged);staged.clear();};
        for(const auto &value:items) {
            const auto row=value.toObject();
            const QString kind=row.value("kind").toString();
            const QString layer=row.value("layer").toString();
            const QString id=row.value("id").toString();
            if(id.isEmpty() || (layer!="*" && !layerNames.contains(layer))) {
                cleanup();return false;
            }
            QGraphicsItem *item=nullptr;
            if(kind=="trace") {
                QPointF a,b;double width=0;
                if(layer=="*" || !point(row.value("a"),a) ||
                   !point(row.value("b"),b) || a==b ||
                   !number(row.value("width_nm"),width) ||
                   width<=0 || width>1000.0) {
                    cleanup();return false;
                }
                item=new QGraphicsLineItem(QLineF(a,b));
                static_cast<QGraphicsLineItem *>(item)->setPen(QPen(Qt::darkCyan,width));
            } else if(kind=="via") {
                QPointF p;double diameter=0;
                if(layer!="*" || !point(row.value("at"),p) ||
                   !number(row.value("diameter_nm"),diameter) ||
                   diameter<=0 || diameter>1000.0 ||
                   !layerNames.contains(row.value("span_first").toString()) ||
                   !layerNames.contains(row.value("span_last").toString())) {
                    cleanup();return false;
                }
                auto *ellipse=new QGraphicsEllipseItem(
                    p.x()-diameter/2,p.y()-diameter/2,diameter,diameter);
                ellipse->setPen(QPen(Qt::darkYellow,0));
                ellipse->setBrush(QBrush(Qt::yellow));
                item=ellipse;
            } else {
                cleanup();return false;
            }
            item->setData(0,layer);
            item->setToolTip("KiCad9 SOURCE "+kind+" / "+id+
              " / Layer "+layer+" / Net "+row.value("net").toString("UNKNOWN")+
              " / Physical contact NOT_VERIFIED / SI NOT_RUN");
            item->setFlag(QGraphicsItem::ItemIsSelectable,true);
            staged.append(item);
        }
        artworkScene_->clear();
        for(auto *item:staged)artworkScene_->addItem(item);
        artworkLayer_->clear();
        artworkLayer_->addItem("All copper layers");
        artworkLayer_->addItems(layerNames);
        artworkLayer_->setCurrentIndex(0);
        artworkView_->setVisible(true);
        artworkLayer_->setVisible(true);
        artworkView_->fitInView(
            artworkScene_->itemsBoundingRect().adjusted(-0.4,-0.4,0.4,0.4),
            Qt::KeepAspectRatio);
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
        if(operation_=="save-odb" &&
           (receipt.value("source_format").toString()!="ODB_SOURCE_SUBSET" ||
            receipt.value("source_manifest_sha256").toString()!=expectedSourceManifest_)) {
            state("FAILED / ODB++ source manifest receipt mismatch");
            return;
        }
        if(operation_=="save-kicad" &&
           (receipt.value("source_format").toString()!="KICAD9_SOURCE_SUBSET" ||
            receipt.value("source_sha256").toString()!=expectedSourceManifest_)) {
            state("FAILED / KiCad9 source SHA receipt mismatch");
            return;
        }
        const bool kiCadPreview=receipt.value("source_format").toString()=="KICAD9_SOURCE_SUBSET";
        if(kiCadPreview && !showKiCadArtwork(receipt.value("artwork").toObject())) {
            state("FAILED / KiCad native preview contract invalid");
            return;
        }
        if(!kiCadPreview) {
            artworkScene_->clear();
            artworkLayer_->setVisible(false);
            artworkView_->setVisible(false);
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
            (kiCadPreview ? "KiCad 9 SOURCE artwork: bounded display/picking only; no electrical contact proof.\n" :
                            "Native geometry is NOT rendered on this preview tab.\n")
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
        artworkLayer_=new QComboBox(this);
        artworkLayer_->setObjectName("f006KiCadLayerFilter");
        artworkScene_=new QGraphicsScene(this);
        artworkView_=new QGraphicsView(artworkScene_,this);
        artworkView_->setObjectName("f006KiCadArtwork");
        artworkView_->setDragMode(QGraphicsView::ScrollHandDrag);
        artworkView_->setRenderHint(QPainter::Antialiasing,true);
        artworkLayer_->setVisible(false);
        artworkView_->setVisible(false);
        layout->addWidget(artworkLayer_);
        layout->addWidget(artworkView_,3);
        connect(artworkLayer_,QOverload<int>::of(&QComboBox::currentIndexChanged),
                this,[this](int index) {
            const QString selected=index<=0 ? QString() : artworkLayer_->itemText(index);
            for(auto *item:artworkScene_->items()) {
                const QString layer=item->data(0).toString();
                item->setVisible(selected.isEmpty() || layer=="*" || layer==selected);
            }
        });
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
#ifdef BOARDSI_RUNTIME_TEST
    // CI only: drive the *real* QProcess timeout state machine faster.
    void setTimeoutForTest(int milliseconds) {
        if(!running_ && milliseconds>=20 && milliseconds<=60000)
            timeoutMs_=milliseconds;
    }
#endif
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
    bool saveOdb(const QString &root,const QString &step,const QString &target,
                 const QByteArray &manifestSha) {
        if(running_ || step.isEmpty() || manifestSha.size()!=64) return false;
        const bool started=begin("save-odb",target,
            {"save-odb",root,step,target,QString::fromLatin1(manifestSha)});
        if(started) expectedSourceManifest_=QString::fromLatin1(manifestSha);
        return started;
    }
    bool saveKiCad(const QString &source,const QString &target,
                   const QByteArray &sourceSha) {
        if(running_ || sourceSha.size()!=64) return false;
        const bool started=begin("save-kicad",target,
            {"save-kicad",source,target,QString::fromLatin1(sourceSha)});
        if(started) expectedSourceManifest_=QString::fromLatin1(sourceSha);
        return started;
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
