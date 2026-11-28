// F_012 initial read-only inspector; no source file access or electrical claims.
#pragma once
#include <QComboBox>
#include <QHBoxLayout>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLabel>
#include <QPlainTextEdit>
#include <QProcess>
#include <QProcessEnvironment>
#include <QPushButton>
#include <QTimer>
#include <QUuid>
#include <QVBoxLayout>
#include <QWidget>

class F012Inspector final : public QWidget {
    QProcess worker_;
    QTimer timeout_;
    QString binary_, path_, digest_, request_, category_;
    QByteArray output_;
    QJsonObject selection_, pendingSelection_;
    bool completing_=false;
    QComboBox *categoryBox_;
    QPlainTextEdit *details_;
    QPushButton *query_, *next_, *cancel_;
    QLabel *status_;
    int offset_=0, nextOffset_=-1;
    bool valid_=false, suspended_=false;
    void controls() {
        const bool busy=worker_.state()!=QProcess::NotRunning || completing_ || !pendingSelection_.isEmpty();
        const bool available=!path_.isEmpty() && !suspended_ && !busy;
        categoryBox_->setEnabled(available);
        query_->setEnabled(available);
        next_->setEnabled(available && nextOffset_>=0);
        cancel_->setEnabled(busy);
    }
    void invalidate() {
        valid_=false;
        timeout_.stop();
        if(worker_.state()!=QProcess::NotRunning)worker_.kill();
    }
    void collect() {
        const auto chunk=worker_.readAllStandardOutput();
        if(!valid_)return;
        if(chunk.size()>60000-output_.size()) {
            status_->setText("FAILED / inspection response limit");
            invalidate();
        } else output_+=chunk;
    }
    void startPage(int offset,const QJsonObject &selection=QJsonObject()) {
        if(path_.isEmpty() || suspended_ || worker_.state()!=QProcess::NotRunning)return;
        output_.clear();
        offset_=offset;
        selection_=selection;
        nextOffset_=-1;
        category_=categoryBox_->currentData().toString();
        request_=QUuid::createUuid().toString(QUuid::Id128);
        valid_=true;completing_=true;
        details_->clear();
        status_->setText("RUNNING / validating native snapshot");
        auto env=QProcessEnvironment::systemEnvironment();
        env.remove("BOARDSI_ARTWORK_REQUEST");
        worker_.setProcessEnvironment(env);
        timeout_.start(60000);
        QStringList args={"inspect",path_,digest_,request_,category_,
                          QString::number(offset_),selection.isEmpty()?"128":"1"};
        if(!selection.isEmpty())
            args.append(QString::fromUtf8(QJsonDocument(selection).toJson(QJsonDocument::Compact)));
        worker_.start(binary_,args);
        controls();
    }
    void finish(int code,QProcess::ExitStatus exit) {
        completing_=false;
        collect();
        timeout_.stop();
        const bool accepted=valid_;
        valid_=false;
        if(!accepted) {startPendingSelection();controls();return;}
        QJsonParseError error;
        const auto doc=QJsonDocument::fromJson(output_,&error);
        const auto result=doc.object();
        const auto rows=result.value("rows");
        const auto total=result.value("total").toInt(-1);
        const auto next=result.value("next");
        const int size=int(rows.toArray().size());
        const int end=offset_+size;
        if(code!=0 || exit!=QProcess::NormalExit || error.error!=QJsonParseError::NoError ||
           !doc.isObject() || result.value("protocol")!="BSI_NATIVE_INSPECT_V1" ||
           result.value("request")!=request_ || result.value("sha256")!=digest_ ||
           result.value("category")!=category_ || result.value("offset").toInt(-1)!=offset_ ||
           result.value("physical_si")!="NOT_RUN" || result.value("physical_contact")!="NOT_VERIFIED" ||
           (selection_.isEmpty() ? !result.value("selection").isNull() :
              result.value("selection").toObject()!=selection_) ||
           !rows.isArray() || size>(selection_.isEmpty()?128:1) || total<end ||
           (end<total && (size==0 || next.toInt(-1)!=end)) ||
           (end==total && !next.isNull())) {
            status_->setText("FAILED / native inspection receipt rejected");
            controls();return;
        }
        for(const auto &row:rows.toArray()) {
            if(!row.isObject()) {
                status_->setText("FAILED / invalid inspection row");
                controls();return;
            }
        }
        nextOffset_=selection_.isEmpty() && end<total?end:-1;
        details_->setPlainText(QString::fromUtf8(QJsonDocument(rows.toArray()).toJson(QJsonDocument::Indented)));
        status_->setText(QString("Native snapshot / %1–%2 of %3 / physical SI NOT_RUN")
                         .arg(size?offset_+1:offset_).arg(end).arg(total));
        controls();
    }
    void startPendingSelection() {
        if(completing_ || worker_.state()!=QProcess::NotRunning ||
           pendingSelection_.isEmpty() || suspended_ || path_.isEmpty())return;
        const auto ref=pendingSelection_;
        pendingSelection_=QJsonObject();
        const int boxIndex=categoryBox_->findData(ref.value("category").toString());
        const int index=ref.value("index").toInt(-1);
        if(boxIndex<0 || index<0)return;
        categoryBox_->setCurrentIndex(boxIndex);
        startPage(index,ref);
    }
public:
    explicit F012Inspector(const QString &binary,QWidget *parent=nullptr)
        : QWidget(parent),binary_(binary) {
        auto *layout=new QVBoxLayout(this);
        auto *bar=new QHBoxLayout;
        categoryBox_=new QComboBox(this);
        categoryBox_->setObjectName("f012InspectionCategory");
        const QStringList categories={"board","sources","layers","materials","nets","frames",
                                      "traces","arcs","pads","vias","drills","areas",
                                      "components","pins","drill-observations"};
        for(const auto &category:categories)categoryBox_->addItem(category,category);
        query_=new QPushButton("Inspect / First page",this);
        query_->setObjectName("f012InspectionQuery");
        next_=new QPushButton("Next page",this);
        cancel_=new QPushButton("Cancel inspection",this);
        status_=new QLabel("No accepted Native document",this);
        status_->setObjectName("f012InspectionStatus");
        details_=new QPlainTextEdit(this);
        details_->setObjectName("f012InspectionDetails");
        details_->setReadOnly(true);
        bar->addWidget(categoryBox_);bar->addWidget(query_);
        bar->addWidget(next_);bar->addWidget(cancel_);
        layout->addLayout(bar);layout->addWidget(status_);layout->addWidget(details_);
        connect(categoryBox_,QOverload<int>::of(&QComboBox::currentIndexChanged),this,[this](int) {
            nextOffset_=-1;details_->clear();
            status_->setText("Select Inspect to query the accepted snapshot");controls();
        });
        connect(query_,&QPushButton::clicked,this,[this]{startPage(0);});
        connect(next_,&QPushButton::clicked,this,[this]{if(nextOffset_>=0)startPage(nextOffset_);});
        connect(cancel_,&QPushButton::clicked,this,[this]{
            pendingSelection_=QJsonObject();
            status_->setText("CANCELLED / inspection");invalidate();controls();
        });
        connect(&worker_,&QProcess::readyReadStandardOutput,this,[this]{collect();});
        connect(&worker_,&QProcess::readyReadStandardError,this,[this]{
            // Drain but never display untrusted worker stderr as metadata.
            worker_.readAllStandardError();
        });
        connect(&worker_,QOverload<int,QProcess::ExitStatus>::of(&QProcess::finished),
                this,[this](int code,QProcess::ExitStatus why){finish(code,why);});
        connect(&worker_,&QProcess::stateChanged,this,[this](QProcess::ProcessState){controls();});
        connect(&worker_,&QProcess::errorOccurred,this,[this](QProcess::ProcessError why){
            if(why==QProcess::FailedToStart) {
                completing_=false;
                pendingSelection_=QJsonObject();
                invalidate();status_->setText("FAILED / inspection worker unavailable");controls();
            }
        });
        timeout_.setSingleShot(true);
        connect(&timeout_,&QTimer::timeout,this,[this]{
            pendingSelection_=QJsonObject();
            status_->setText("FAILED / inspection timeout");invalidate();controls();
        });
        controls();
    }
    ~F012Inspector() override {
        QObject::disconnect(&worker_,nullptr,this,nullptr);
        invalidate();
    }
    void inspectSelection(const QJsonObject &selection) {
        if(path_.isEmpty() || suspended_)return;
        // Every selection transition invalidates any superseded selection result,
        // including an empty or multi-selection. Keep at most the newest request.
        pendingSelection_=QJsonObject();
        nextOffset_=-1;details_->clear();
        const auto category=selection.value("category").toString();
        const int index=selection.value("index").toInt(-1);
        if(index>=0 && categoryBox_->findData(category)>=0)
            pendingSelection_=selection;
        status_->setText(pendingSelection_.isEmpty()?"No single supported object selected":
                         "RUNNING / latest artwork selection");
        if(completing_ || worker_.state()!=QProcess::NotRunning)invalidate();
        startPendingSelection();
        controls();
    }
    void cancelSelection() {
        pendingSelection_=QJsonObject();
        nextOffset_=-1;details_->clear();
        invalidate();
        status_->setText("Inspection selection cleared");controls();
    }
    void setDocument(const QString &path,const QString &digest) {
        pendingSelection_=QJsonObject();
        invalidate();path_=path;digest_=digest;nextOffset_=-1;
        details_->clear();
        status_->setText(path.isEmpty()?"No accepted Native document":
                         "Select Inspect to query the accepted snapshot");
        controls();
    }
    void suspend(bool suspended) {
        suspended_=suspended;
        if(suspended && (valid_ || !pendingSelection_.isEmpty())) {
            pendingSelection_=QJsonObject();
            invalidate();nextOffset_=-1;
            status_->setText("Inspection cancelled during Native operation");
        }
        controls();
    }
};
