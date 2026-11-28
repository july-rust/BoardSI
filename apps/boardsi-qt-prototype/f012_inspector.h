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
#include <QTabWidget>
#include <QSpinBox>
#include <QTreeWidget>
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
    QTreeWidget *fields_;
    QSpinBox *parentIndex_, *childIndex_, *startIndex_;
    QPushButton *query_, *next_, *cancel_;
    QLabel *status_;
    int offset_=0, nextOffset_=-1;
    bool valid_=false, suspended_=false;
    void clearDetails() {
        details_->clear();fields_->clear();
    }
    static void appendField(QTreeWidgetItem *parent,const QString &name,const QJsonValue &value) {
        auto *item=new QTreeWidgetItem(parent);
        item->setText(0,name);
        if(value.isObject()) {
            const auto obj=value.toObject();
            if(obj.contains("numerator") && obj.contains("denominator") &&
               obj.value("numerator").isString() && obj.value("denominator").isString())
                item->setText(1,obj.value("numerator").toString()+" / "+obj.value("denominator").toString());
            else if(obj.contains("state"))item->setText(1,obj.value("state").toString());
            for(auto it=obj.constBegin();it!=obj.constEnd();++it)appendField(item,it.key(),it.value());
        } else if(value.isArray()) {
            const auto array=value.toArray();
            item->setText(1,QString("%1 entries").arg(array.size()));
            for(qsizetype i=0;i<array.size();++i)appendField(item,QString::number(i),array.at(i));
        } else {
            const auto text=value.isString()?value.toString():
                value.isNull()?QString("NOT AVAILABLE"):
                value.isBool()?(value.toBool()?QString("true"):QString("false")):
                QString::number(value.toDouble(),'g',17);
            item->setText(1,text);
            item->setToolTip(1,text);
        }
    }
    void publishDetails(const QJsonArray &rows) {
        clearDetails();
        fields_->setUpdatesEnabled(false);
        for(qsizetype i=0;i<rows.size();++i) {
            auto *item=new QTreeWidgetItem(fields_);
            const auto row=rows.at(i).toObject();
            item->setText(0,QString("%1 #%2").arg(category_).arg(offset_+i));
            item->setText(1,row.value("name").toString());
            for(auto it=row.constBegin();it!=row.constEnd();++it)appendField(item,it.key(),it.value());
            if(rows.size()==1)item->setExpanded(true);
        }
        fields_->setUpdatesEnabled(true);
        details_->setPlainText(QString::fromUtf8(QJsonDocument(rows).toJson(QJsonDocument::Indented)));
    }
    void controls() {
        const bool busy=worker_.state()!=QProcess::NotRunning || completing_ || !pendingSelection_.isEmpty();
        const bool available=!path_.isEmpty() && !suspended_ && !busy;
        categoryBox_->setEnabled(available);
        const bool nested=categoryBox_->currentData().toString().endsWith(":");
        parentIndex_->setEnabled(available && nested);
        childIndex_->setEnabled(available && categoryBox_->currentData().toString().endsWith("::"));
        startIndex_->setEnabled(available);
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
        if(category_.endsWith("::")) {
            category_.chop(1);
            category_+=QString::number(parentIndex_->value())+":"+QString::number(childIndex_->value());
        } else if(category_.endsWith(":"))category_+=QString::number(parentIndex_->value());
        request_=QUuid::createUuid().toString(QUuid::Id128);
        valid_=true;completing_=true;
        clearDetails();
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
        publishDetails(rows.toArray());
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
                                      "components","pins","drill-observations","padstacks",
                                      "source-files:","source-conditions:","trace-points:","padstack-entries:",
                                      "trace-history:","via-history:","pad-history:","layer-history:","material-history:",
                                      "outlines","arc-history:","area-history:","net-history:","frame-history:",
                                      "drill-history:","padstack-history:","component-history:","pin-history:","outline-history:",
                                      "area-segments:","outline-segments:","area-cutouts:","outline-cutouts:",
                                      "area-cutout-segments::","outline-cutout-segments::","custom-pad-segments::"};
        for(const auto &category:categories)categoryBox_->addItem(category,category);
        query_=new QPushButton("Inspect / First page",this);
        query_->setObjectName("f012InspectionQuery");
        next_=new QPushButton("Next page",this);
        next_->setObjectName("f012InspectionNext");
        parentIndex_=new QSpinBox(this);
        parentIndex_->setObjectName("f012InspectionParent");
        parentIndex_->setRange(0,1000000);
        childIndex_=new QSpinBox(this);
        childIndex_->setObjectName("f012InspectionChild");
        childIndex_->setRange(0,1000000);
        startIndex_=new QSpinBox(this);
        startIndex_->setObjectName("f012InspectionOffset");
        startIndex_->setRange(0,1000000);
        cancel_=new QPushButton("Cancel inspection",this);
        status_=new QLabel("No accepted Native document",this);
        status_->setObjectName("f012InspectionStatus");
        details_=new QPlainTextEdit(this);
        details_->setObjectName("f012InspectionDetails");
        details_->setReadOnly(true);
        fields_=new QTreeWidget(this);
        fields_->setObjectName("f012InspectionFields");
        fields_->setColumnCount(2);
        fields_->setHeaderLabels({"Field / exact unit","Stored value / evidence"});
        fields_->setUniformRowHeights(true);
        fields_->setColumnWidth(0,250);
        auto *tabs=new QTabWidget(this);
        tabs->addTab(fields_,"Details");
        tabs->addTab(details_,"Exact JSON");
        bar->addWidget(categoryBox_);bar->addWidget(query_);
        bar->addWidget(next_);bar->addWidget(cancel_);
        auto *indices=new QHBoxLayout;
        indices->addWidget(new QLabel("Parent",this));indices->addWidget(parentIndex_);
        indices->addWidget(new QLabel("Child",this));indices->addWidget(childIndex_);
        indices->addWidget(new QLabel("Start index",this));indices->addWidget(startIndex_);
        indices->addStretch();
        layout->addLayout(bar);layout->addLayout(indices);layout->addWidget(status_);layout->addWidget(tabs);
        connect(categoryBox_,QOverload<int>::of(&QComboBox::currentIndexChanged),this,[this](int) {
            startIndex_->setValue(0);
            nextOffset_=-1;clearDetails();
            status_->setText("Select Inspect to query the accepted snapshot");controls();
        });
        const auto resetPage=[this](int) {
            nextOffset_=-1;clearDetails();controls();
        };
        connect(parentIndex_,QOverload<int>::of(&QSpinBox::valueChanged),this,resetPage);
        connect(childIndex_,QOverload<int>::of(&QSpinBox::valueChanged),this,resetPage);
        connect(startIndex_,QOverload<int>::of(&QSpinBox::valueChanged),this,resetPage);
        connect(query_,&QPushButton::clicked,this,[this]{startPage(startIndex_->value());});
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
    qint64 inspectionWorkerPidForTest() const {return worker_.processId();}
    qsizetype inspectionBytesForTest() const {return output_.size();}
    void inspectSelection(const QJsonObject &selection) {
        if(path_.isEmpty() || suspended_)return;
        // Every selection transition invalidates any superseded selection result,
        // including an empty or multi-selection. Keep at most the newest request.
        pendingSelection_=QJsonObject();
        nextOffset_=-1;clearDetails();
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
        nextOffset_=-1;clearDetails();
        invalidate();
        status_->setText("Inspection selection cleared");controls();
    }
    void setDocument(const QString &path,const QString &digest) {
        pendingSelection_=QJsonObject();
        invalidate();path_=path;digest_=digest;nextOffset_=-1;
        clearDetails();
        status_->setText(path.isEmpty()?"No accepted Native document":
                         "Select Inspect to query the accepted snapshot");
        controls();
    }
    void suspend(bool suspended) {
        if(suspended_==suspended)return;
        suspended_=suspended;
        if(suspended) {
            // Even a completed page is invalid as a selection display once
            // a Native operation starts; its old scene may stay interactive.
            pendingSelection_=QJsonObject();
            invalidate();nextOffset_=-1;clearDetails();
            status_->setText("Inspection suspended during Native operation");
        } else {
            status_->setText(path_.isEmpty()?"No accepted Native document":
                             "Select Inspect to query the accepted snapshot");
        }
        controls();
    }
};
