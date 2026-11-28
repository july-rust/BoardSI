// F_003_1: real native .bsi file QProcess controller, not an artwork decoder.
// The backend owns the exact Canonical model; no Qt drawing is serialized as .bsi.
#pragma once
#include <QBrush>
#include <QComboBox>
#include <QGraphicsItem>
#include <QGraphicsScene>
#include <QGraphicsView>
#include <QElapsedTimer>
#include <QMouseEvent>
#include <QPushButton>
#include <QScrollBar>
#include <QWheelEvent>
#include <QJsonArray>
#include <QPainter>
#include <QPainterPath>
#include <QPen>
#include <cmath>
#include <QCoreApplication>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLabel>
#include <QPlainTextEdit>
#include <QProcess>
#include <QProcessEnvironment>
#include <QUuid>
#include "f011_artwork_stream.h"
#include "f012_inspector.h"
#include <QRegularExpression>
#include <QTimer>
#include <QVBoxLayout>
#include <QWidget>
#include <QVariant>
#include <functional>

// Keep the existing QGraphicsView selection semantics. Wheel zoom and middle
// drag pan operate only on the display transform, never native coordinates.
class F011NativeArtworkView final : public QGraphicsView {
    bool panning_=false;
    QPoint last_;
protected:
    void wheelEvent(QWheelEvent *event) override {
        const int delta=event->angleDelta().y();
        if(delta==0) {QGraphicsView::wheelEvent(event);return;}
        const double factor=std::pow(1.2,double(delta)/120.0);
        const double next=transform().m11()*factor;
        if(std::isfinite(next) && next>=1e-6 && next<=1e6)scale(factor,factor);
        event->accept();
    }
    void mousePressEvent(QMouseEvent *event) override {
        if(event->button()!=Qt::MiddleButton) {
            QGraphicsView::mousePressEvent(event);return;
        }
        panning_=true;last_=event->position().toPoint();
        viewport()->setCursor(Qt::ClosedHandCursor);
        event->accept();
    }
    void mouseMoveEvent(QMouseEvent *event) override {
        if(!panning_) {QGraphicsView::mouseMoveEvent(event);return;}
        const QPoint now=event->position().toPoint();
        const QPoint delta=now-last_;last_=now;
        horizontalScrollBar()->setValue(horizontalScrollBar()->value()-delta.x());
        verticalScrollBar()->setValue(verticalScrollBar()->value()-delta.y());
        event->accept();
    }
    void mouseReleaseEvent(QMouseEvent *event) override {
        if(panning_ && event->button()==Qt::MiddleButton) {
            panning_=false;viewport()->unsetCursor();event->accept();return;
        }
        QGraphicsView::mouseReleaseEvent(event);
    }
public:
    explicit F011NativeArtworkView(QGraphicsScene *scene,QWidget *parent)
        : QGraphicsView(scene,parent) {
        setTransformationAnchor(QGraphicsView::AnchorUnderMouse);
    }
};

class F0031NativePanel final : public QWidget {
    QProcess child_;
    QTimer timeout_, renderTimer_;
    F011ArtworkStream stream_;
    QGraphicsScene *stagedScene_=nullptr;
    QJsonObject pendingReceipt_;
    QJsonArray pendingItems_;
    qsizetype renderIndex_=0;
    QStringList pendingLayers_;
    bool transportFailed_=false;
#ifdef BOARDSI_RUNTIME_TEST
    bool holdRenderForTest_=false;
#endif
    QLabel *state_;
    QPlainTextEdit *summary_;
    QGraphicsScene *artworkScene_;
    QGraphicsView *artworkView_;
    QComboBox *artworkLayer_;
    QPushButton *artworkFit_;
    F012Inspector *inspector_=nullptr;
    QPushButton *inspectToggle_=nullptr;
    QElapsedTimer phaseClock_;
    qint64 workerMs_=0, sceneMs_=0;
    qsizetype streamBytes_=0;
    int streamBatches_=0;
    QByteArray errors_;
    QString binary_, operation_, target_, loaded_, acceptedDigest_, expectedSourceManifest_, expectedDrillSha_;
    bool running_=false, cancelled_=false, readOnly_=true;
    int timeoutMs_=60000;
    std::function<void()> onChanged_;
    void state(const QString &value) {
        state_->setText(value);
        if(inspector_)inspector_->suspend(running_);
        if (onChanged_) onChanged_();
    }
    void discardRender() {
        renderTimer_.stop();
        if(stagedScene_) {
            delete stagedScene_;
            stagedScene_=nullptr;
        }
        pendingReceipt_=QJsonObject();pendingItems_=QJsonArray();
        pendingLayers_.clear();renderIndex_=0;
    }
    void stopWithError(const QString &value) {
        if (!running_) return;
        errors_=value.toUtf8().left(256);
        transportFailed_=true;
        if(stagedScene_) {
            discardRender();timeout_.stop();running_=false;
            state("FAILED / "+value);
        } else {
            child_.kill();
        }
    }
    void collect(bool error) {
        const QByteArray chunk=error ? child_.readAllStandardError()
                                     : child_.readAllStandardOutput();
        if(transportFailed_)return;
        if(error) {
            if(chunk.size()>512-errors_.size()) {
                stopWithError("BSI_WORKER_OUTPUT_LIMIT");
                return;
            }
            errors_+=chunk;
        } else if(!stream_.feed(chunk)) {
            stopWithError("BSI_F011_STREAM_INVALID");
        }
    }
    bool begin(const QString &operation, const QString &target,
               const QStringList &arguments) {
        if(running_ || !QFileInfo(binary_).isExecutable()) {
            state("FAILED / native file worker unavailable or busy");
            return false;
        }
        running_=true; cancelled_=false; transportFailed_=false; errors_.clear();
        QString token=QUuid::createUuid().toString(QUuid::Id128);
        stream_.reset(token);
        auto environment=QProcessEnvironment::systemEnvironment();
        environment.insert("BOARDSI_ARTWORK_REQUEST",token);
        child_.setProcessEnvironment(environment);
        operation_=operation; target_=target;
        workerMs_=0;sceneMs_=0;streamBytes_=0;streamBatches_=0;
        phaseClock_.start();
        timeout_.start(timeoutMs_);
        state("RUNNING / Native "+operation);
        child_.start(binary_,arguments);
        return true;
    }
    // Bounded and strictly checked Source-only graphical overview.
    // A view item is never an electrically connected model or Solver claim.
    bool appendArtwork(const QJsonObject &body,QGraphicsScene *scene,QStringList &acceptedLayers) {
        const QString schema=body.value("schema").toString();
        if((schema!="BSI_KICAD_SOURCE_PREVIEW_V1" &&
            schema!="BSI_IPC2581_SOURCE_PREVIEW_V1" &&
            schema!="BSI_GERBER_DRILL_SOURCE_PREVIEW_V1" &&
            schema!="BSI_F009_NATIVE_CAD_PREVIEW_V1") ||
           body.value("units").toString()!="integer-nm" ||
           body.value("solver_ready").toBool(true) ||
           !body.value("items").isArray() || !body.value("layers").isArray())
            return false;
        const QJsonArray items=body.value("items").toArray();
        const QJsonArray layers=body.value("layers").toArray();
        if(items.isEmpty() || items.size()>128 ||
           layers.size()<(schema=="BSI_GERBER_DRILL_SOURCE_PREVIEW_V1" ? 1 : 2) ||
           layers.size()>32)
            return false;
        QStringList layerNames;
        for(const auto &value:layers) {
            const QString name=value.toObject().value("name").toString();
            if(name.isEmpty() || name=="*" || name=="All copper layers" ||
               layerNames.contains(name))return false;
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
            // The Native model stores board-space Y positive upward.
            // The existing PADS/ODB source artwork maps it to Qt screen Y
            // negative upward. Preserve the same handedness on Native Open.
            p=QPointF(x,schema=="BSI_F009_NATIVE_CAD_PREVIEW_V1" ? -y : y);
            return true;
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
                QPen pen(Qt::darkCyan,width);
                pen.setCapStyle(Qt::RoundCap);
                static_cast<QGraphicsLineItem *>(item)->setPen(pen);
            } else if(kind=="via") {
                QPointF p;double diameter=0,drill=0;
                if(layer!="*" || !point(row.value("at"),p) ||
                   !number(row.value("diameter_nm"),diameter) ||
                   !number(row.value("drill_nm"),drill) ||
                   drill<=0 || drill>=diameter ||
                   diameter<=0 || diameter>1000.0 ||
                   !layerNames.contains(row.value("span_first").toString()) ||
                   !layerNames.contains(row.value("span_last").toString())) {
                    cleanup();return false;
                }
                QPainterPath ring;
                ring.setFillRule(Qt::OddEvenFill);
                ring.addEllipse(p,diameter/2,diameter/2);
                ring.addEllipse(p,drill/2,drill/2);
                auto *viaRing=new QGraphicsPathItem(ring);
                viaRing->setPen(QPen(Qt::darkYellow,0));
                viaRing->setBrush(QBrush(Qt::yellow));
                item=viaRing;
            } else if(kind=="rectangle") {
                QPointF p;double width=0,height=0;
                if(layer=="*" || !point(row.value("at"),p) ||
                   !number(row.value("width_nm"),width) ||
                   !number(row.value("height_nm"),height) ||
                   width<=0 || height<=0 || width>1000.0 || height>1000.0) {
                    cleanup();return false;
                }
                auto *rect=new QGraphicsRectItem(
                    p.x()-width/2,p.y()-height/2,width,height);
                rect->setPen(QPen(Qt::darkCyan,0));
                rect->setBrush(QBrush(Qt::cyan));
                item=rect;
            } else if(kind=="flash" || kind=="drill-source") {
                QPointF p;double diameter=0;
                if(!point(row.value("at"),p) ||
                   !number(row.value("diameter_nm"),diameter) ||
                   diameter<=0 || diameter>1000.0 ||
                   (kind=="flash" && layer=="*") ||
                   (kind=="drill-source" &&
                     (layer!="*" ||
                      row.value("plating").toString()!="UNKNOWN" ||
                      row.value("layer_span").toString()!="UNKNOWN" ||
                      row.value("alignment").toString()!="NOT_VERIFIED" ||
                      (!row.value("tool_id").isDouble() ||
                       !std::isfinite(row.value("tool_id").toDouble()) ||
                       std::floor(row.value("tool_id").toDouble())!=
                           row.value("tool_id").toDouble() ||
                       row.value("tool_id").toDouble()<1.0 ||
                       row.value("tool_id").toDouble()>4294967295.0)))) {
                    cleanup();return false;
                }
                auto *circle=new QGraphicsEllipseItem(
                    p.x()-diameter/2,p.y()-diameter/2,diameter,diameter);
                circle->setPen(QPen(kind=="flash" ? Qt::darkCyan : Qt::darkGray,0));
                circle->setBrush(kind=="flash" ? QBrush(Qt::cyan) : QBrush(Qt::NoBrush));
                item=circle;
            } else {
                cleanup();return false;
            }
            item->setData(0,layer);
            if(row.value("inspection_ref").isObject())
                item->setData(2,QVariant(row.value("inspection_ref").toObject()));
            if(kind=="via") {
                // Via annulus exists ONLY on explicit native Padstack copper
                // layers. Source-layer spans or logical net names are not proof
                // of a pad on every inner layer.
                if(!row.value("pad_layers").isArray()) {
                    delete item;cleanup();return false;
                }
                QStringList pads;
                for(const auto &v:row.value("pad_layers").toArray()) {
                    const QString entry=v.toString();
                    if(entry.isEmpty() || !layerNames.contains(entry) ||
                       pads.contains(entry)) {
                        delete item;cleanup();return false;
                    }
                    pads<<entry;
                }
                if(pads.isEmpty()) {
                    delete item;cleanup();return false;
                }
                item->setData(1,pads);
            }
            const QString sourceName=schema=="BSI_IPC2581_SOURCE_PREVIEW_V1"
                ? QStringLiteral("IPC-2581C") :
                schema=="BSI_GERBER_DRILL_SOURCE_PREVIEW_V1"
                ? QStringLiteral("Gerber/Excellon") :
                schema=="BSI_F009_NATIVE_CAD_PREVIEW_V1"
                ? QStringLiteral("PADS/ODB Native") : QStringLiteral("KiCad9");
            item->setToolTip(sourceName+" SOURCE "+kind+" / "+id+
              " / Layer "+layer+" / Net "+row.value("net").toString("UNKNOWN")+
              " / Physical contact NOT_VERIFIED / SI NOT_RUN");
            item->setFlag(QGraphicsItem::ItemIsSelectable,true);
            staged.append(item);
        }
        if(!acceptedLayers.isEmpty() && acceptedLayers!=layerNames) {
            cleanup();return false;
        }
        acceptedLayers=layerNames;
        for(auto *item:staged)scene->addItem(item);
        return true;
    }
    void renderNext() {
        if(!running_ || !stagedScene_)return;
        // One bounded batch of scene work per Qt event-loop turn. The old
        // document is untouched until every geometry item has been accepted.
        auto artwork=pendingReceipt_.value("artwork").toObject();
        QJsonArray batch;
        const auto end=qMin(renderIndex_+qsizetype(128),pendingItems_.size());
        while(renderIndex_<end)batch.append(pendingItems_.at(renderIndex_++));
        artwork.insert("items",batch);
        if(!appendArtwork(artwork,stagedScene_,pendingLayers_)) {
            stopWithError("BSI_F011_ARTWORK_INVALID");return;
        }
        if(renderIndex_<pendingItems_.size()) {
#ifdef BOARDSI_RUNTIME_TEST
            if(holdRenderForTest_)return;
#endif
            renderTimer_.start(0);
            return;
        }
        timeout_.stop();
        auto *old=artworkScene_;
        artworkScene_=stagedScene_;stagedScene_=nullptr;
        artworkView_->setScene(artworkScene_);
        connectInspectorPicking();
        old->deleteLater();
        artworkLayer_->clear();
        artworkLayer_->addItem("All copper layers");
        artworkLayer_->addItems(pendingLayers_);
        artworkLayer_->setCurrentIndex(0);
        artworkView_->setVisible(true);
        artworkLayer_->setVisible(true);
        artworkFit_->setVisible(true);
        fitArtwork();
        sceneMs_=phaseClock_.elapsed();
        const auto receipt=pendingReceipt_;
        discardRender();
        running_=false;
        publishReceipt(receipt,true);
    }
    void completed(int exitCode, QProcess::ExitStatus exitStatus) {
        if(!running_) return;
        timeout_.stop();
        collect(false); collect(true);
        running_=false;
        if(cancelled_) {
            stream_.reset(QString());
            state("CANCELLED / outcome unconfirmed; re-open target to verify");
            return;
        }
        if(transportFailed_ || exitStatus!=QProcess::NormalExit || exitCode!=0) {
            stream_.reset(QString());
            state("FAILED / "+QString::fromUtf8(errors_).trimmed().left(256));
            return;
        }
        QJsonObject receipt;
        const bool streamValid=stream_.finish(receipt);
        workerMs_=phaseClock_.elapsed();
        streamBytes_=stream_.bytesReceived();streamBatches_=stream_.batchesReceived();
        stream_.reset(QString());
        if(!streamValid ||
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
        if(operation_=="save-ipc2581" &&
           (receipt.value("source_format").toString()!="IPC2581C_SOURCE_SUBSET" ||
            receipt.value("source_sha256").toString()!=expectedSourceManifest_)) {
            state("FAILED / IPC-2581C source SHA receipt mismatch");
            return;
        }
        if(operation_=="save-gerber-drill" &&
           (receipt.value("source_format").toString()!="GERBER_X2_EXCELLON_SOURCE_SUBSET" ||
            receipt.value("source_sha256").toString()!=expectedSourceManifest_ ||
            receipt.value("drill_sha256").toString()!=expectedDrillSha_)) {
            state("FAILED / Gerber/Excellon source SHA receipt mismatch");
            return;
        }
        const QString sourceFormat=receipt.value("source_format").toString();
        const bool f009Native=sourceFormat=="PADS_V94_SOURCE_SUBSET" ||
                              sourceFormat=="ODB_SOURCE_SUBSET";
        const QString expectedSchema=f009Native ? "BSI_F009_NATIVE_CAD_PREVIEW_V1" :
            sourceFormat=="KICAD9_SOURCE_SUBSET" ? "BSI_KICAD_SOURCE_PREVIEW_V1" :
            sourceFormat=="IPC2581C_SOURCE_SUBSET" ? "BSI_IPC2581_SOURCE_PREVIEW_V1" :
            sourceFormat=="GERBER_X2_EXCELLON_SOURCE_SUBSET" ? "BSI_GERBER_DRILL_SOURCE_PREVIEW_V1" :
            QString();
        const bool knownArtwork=!expectedSchema.isEmpty();
        const QString artworkStatus=receipt.value("artwork_status").toString();
        const QString artworkReason=receipt.value("artwork_reason").toString();
        if((knownArtwork &&
            ((artworkStatus!="RENDERABLE" && artworkStatus!="NOT_RENDERED") ||
             (artworkStatus=="NOT_RENDERED" &&
              (!artworkReason.startsWith("BSI_") || receipt.contains("artwork"))) ||
             (artworkStatus=="RENDERABLE" &&
              (!receipt.value("artwork").isObject() ||
               receipt.value("artwork").toObject().value("schema").toString()!=expectedSchema ||
               receipt.contains("artwork_reason"))))) ||
           (!knownArtwork && (receipt.contains("artwork") ||
                             receipt.contains("artwork_status") ||
                             receipt.contains("artwork_reason")))) {
            state("FAILED / invalid native CAD artwork receipt");
            return;
        }
        const bool sourcePreview=knownArtwork && artworkStatus=="RENDERABLE";
        if(sourcePreview) {
            auto artwork=receipt.value("artwork").toObject();
            pendingItems_=artwork.take("items").toArray();
            if(pendingItems_.isEmpty() || pendingItems_.size()>10000) {
                pendingItems_=QJsonArray();
                state("FAILED / invalid native artwork size");
                return;
            }
            receipt.insert("artwork",artwork);
            pendingReceipt_=receipt;
            renderIndex_=0;pendingLayers_.clear();
            stagedScene_=new QGraphicsScene(this);
            stream_.reset(QString());
            running_=true;
            phaseClock_.restart();
            timeout_.start(timeoutMs_);
            renderTimer_.start(0);
            return;
        }
        stream_.reset(QString());
        artworkScene_->clear();
        artworkLayer_->setVisible(false);
        artworkView_->setVisible(false);
        artworkFit_->setVisible(false);
        publishReceipt(receipt,false);
    }
    void connectInspectorPicking() {
        auto *scene=artworkScene_;
        connect(scene,&QGraphicsScene::selectionChanged,this,[this,scene] {
            if(scene!=artworkScene_ || running_ || !inspector_ || !inspectToggle_->isChecked())return;
            const auto selected=scene->selectedItems();
            if(selected.size()!=1)return;
            const auto ref=selected.front()->data(2).value<QJsonObject>();
            if(!ref.isEmpty())inspector_->inspectSelection(ref);
        });
    }
    void fitArtwork() {
        if(artworkScene_->items().isEmpty())return;
        artworkView_->fitInView(
            artworkScene_->itemsBoundingRect().adjusted(-0.4,-0.4,0.4,0.4),
            Qt::KeepAspectRatio);
    }
    void publishReceipt(const QJsonObject &receipt,bool sourcePreview) {
        const bool knownArtwork=receipt.contains("artwork_status");
        const auto artworkReason=receipt.value("artwork_reason").toString();
        loaded_=target_;
        acceptedDigest_=receipt.value("sha256").toString();
        inspector_->setDocument(loaded_,acceptedDigest_);
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
            "Canonical data is stored exactly in Rust.\n"+
            (sourcePreview ? "SOURCE artwork: bounded display/picking only; no electrical contact proof.\n" :
               knownArtwork ? "ARTWORK NOT_RENDERED / "+artworkReason+"\n" :
                            "Native geometry is NOT rendered on this preview tab.\n")+
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
                               "Supported Native geometry has bounded Artwork rendering; SI NOT_RUN.");
        layout->addWidget(state_); layout->addWidget(summary_,1);
        artworkLayer_=new QComboBox(this);
        artworkLayer_->setObjectName("f006KiCadLayerFilter");
        artworkScene_=new QGraphicsScene(this);
        artworkView_=new F011NativeArtworkView(artworkScene_,this);
        artworkView_->setObjectName("f006KiCadArtwork");
        artworkView_->setDragMode(QGraphicsView::RubberBandDrag);
        artworkView_->setRenderHint(QPainter::Antialiasing,true);
        artworkLayer_->setVisible(false);
        artworkView_->setVisible(false);
        artworkFit_=new QPushButton("Fit to View",this);
        artworkFit_->setObjectName("f011NativeFit");
        artworkFit_->setVisible(false);
        connect(artworkFit_,&QPushButton::clicked,this,[this]{fitArtwork();});
        layout->addWidget(artworkLayer_);
        layout->addWidget(artworkFit_);
        layout->addWidget(artworkView_,3);
        inspectToggle_=new QPushButton("Native Board Inspector",this);
        inspectToggle_->setCheckable(true);
        inspector_=new F012Inspector(binary_,this);
        inspector_->setVisible(false);
        layout->addWidget(inspectToggle_);
        layout->addWidget(inspector_,2);
        connect(inspectToggle_,&QPushButton::toggled,inspector_,&QWidget::setVisible);
        connect(artworkLayer_,QOverload<int>::of(&QComboBox::currentIndexChanged),
                this,[this](int index) {
            const QString selected=index<=0 ? QString() : artworkLayer_->itemText(index);
            for(auto *item:artworkScene_->items()) {
                const QString layer=item->data(0).toString();
                const QStringList padLayers=item->data(1).toStringList();
                item->setVisible(selected.isEmpty() || layer==selected ||
                    (layer=="*" &&
                     (padLayers.isEmpty() || padLayers.contains(selected))));
            }
        });
        renderTimer_.setSingleShot(true);
        connect(&renderTimer_,&QTimer::timeout,this,[this]{renderNext();});
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
    void holdRenderForTest(bool hold) {if(!running_)holdRenderForTest_=hold;}
    qsizetype stagedCountForTest() const {return stagedScene_?renderIndex_:0;}
    qint64 workerPidForTest() const {return child_.processId();}
    qint64 workerMsForTest() const {return workerMs_;}
    qint64 sceneMsForTest() const {return sceneMs_;}
    qsizetype streamBytesForTest() const {return streamBytes_;}
    int streamBatchesForTest() const {return streamBatches_;}
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
        if(!running_) {
            loaded_.clear();acceptedDigest_.clear();readOnly_=true;
            inspector_->setDocument(QString(),QString());
            artworkScene_->clear();
            artworkLayer_->setVisible(false);
            artworkView_->setVisible(false);
            artworkFit_->setVisible(false);
            summary_->setPlainText("Native .bsi file not loaded.");
            state("IDLE / source import selected");
        }
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
    bool saveIpc2581(const QString &source,const QString &target,
                     const QByteArray &sourceSha) {
        if(running_ || sourceSha.size()!=64) return false;
        const bool started=begin("save-ipc2581",target,
            {"save-ipc2581",source,target,QString::fromLatin1(sourceSha)});
        if(started) expectedSourceManifest_=QString::fromLatin1(sourceSha);
        return started;
    }
    bool saveGerberDrill(const QString &gerber,const QString &drill,
                         const QString &target,const QByteArray &gerberSha,
                         const QByteArray &drillSha) {
        if(running_ || gerberSha.size()!=64 || drillSha.size()!=64) return false;
        const bool started=begin("save-gerber-drill",target,
            {"save-gerber-drill",gerber,drill,target,QString::fromLatin1(gerberSha),
             QString::fromLatin1(drillSha)});
        if(started) {
            expectedSourceManifest_=QString::fromLatin1(gerberSha);
            expectedDrillSha_=QString::fromLatin1(drillSha);
        }
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
        if(stagedScene_) {
            discardRender();running_=false;
            state("CANCELLED / artwork not published; re-open target to verify");
        } else {
            child_.kill();
        }
    }
};
