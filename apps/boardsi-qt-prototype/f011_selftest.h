// Actual Main Qt and Native Worker qualification; no mock GUI or CAD claim.
#pragma once
#include <QVector>
#include <QPair>
#include <QSaveFile>
#if defined(Q_OS_MACOS)
#include <libproc.h>
#include <sys/proc_info.h>
#endif

// Samples are lower bounds on the live worker peak, not a claim that no
// higher transient allocation occurred. Windows/Linux expose OS high-water
// values when the worker still exists; macOS samples current resident bytes.
static qint64 f011ResidentSample(qint64 pid,bool highWater=true) {
    if(pid<=0)return -1;
#ifdef Q_OS_WIN
    HANDLE process=OpenProcess(PROCESS_QUERY_INFORMATION|PROCESS_VM_READ,FALSE,DWORD(pid));
    if(!process)return -1;
    PROCESS_MEMORY_COUNTERS counters{};
    counters.cb=sizeof(counters);
    const bool ok=GetProcessMemoryInfo(process,&counters,sizeof(counters));
    CloseHandle(process);
    return ok?qint64(highWater?counters.PeakWorkingSetSize:counters.WorkingSetSize):-1;
#elif defined(Q_OS_MACOS)
    Q_UNUSED(highWater);
    struct proc_taskinfo info{};
    const int result=proc_pidinfo(int(pid),PROC_PIDTASKINFO,0,&info,sizeof(info));
    return result==sizeof(info)?qint64(info.pti_resident_size):-1;
#elif defined(Q_OS_LINUX)
    QFile status(QString("/proc/%1/status").arg(pid));
    if(status.open(QIODevice::ReadOnly)) {
        const auto match=QRegularExpression(
            highWater?"VmHWM:\\s+(\\d+)\\s+kB":"VmRSS:\\s+(\\d+)\\s+kB")
            .match(QString::fromLatin1(status.readAll()));
        if(match.hasMatch())return match.captured(1).toLongLong()*1024;
    }
    return -1;
#else
    Q_UNUSED(highWater);
    return -1;
#endif
}

static bool f011StreamNegativeSelftest() {
    const QString token="0123456789abcdef0123456789abcdef";
    auto frame=[&](QJsonObject obj) {
        obj.insert("protocol","BSI_ARTWORK_STREAM_V1");
        obj.insert("request",token);
        return QJsonDocument(obj).toJson(QJsonDocument::Compact)+"\n";
    };
    const auto begin=frame(QJsonObject{{"kind","begin"},{"item_count",1},
        {"receipt",QJsonObject{{"artwork_status","RENDERABLE"},
                               {"artwork",QJsonObject{{"units","integer-nm"}}}}}});
    const auto batch=frame(QJsonObject{{"kind","batch"},{"sequence",0},
        {"items",QJsonArray{QJsonObject{{"id","trace/1"}}}}});
    const auto digest=QString::fromLatin1(QCryptographicHash::hash(
        begin+batch,QCryptographicHash::Sha256).toHex());
    const auto end=frame(QJsonObject{{"kind","end"},{"batch_count",1},
        {"item_count",1},{"sha256",digest}});
    const auto good=begin+batch+end;
    F011ArtworkStream decoder;
    QJsonObject receipt;
    decoder.reset(token);
    for(qsizetype n=0;n<good.size();n+=7)
        if(!decoder.feed(good.mid(n,7)))return false;
    if(!decoder.finish(receipt) ||
       receipt.value("artwork").toObject().value("items").toArray().size()!=1)return false;
    QByteArray wrongRequest=good;
    wrongRequest.replace(token.toLatin1(),QByteArray(32,'a'));
    QByteArray wrongSequence=batch;
    wrongSequence.replace("\"sequence\":0","\"sequence\":1");
    QByteArray wrongHash=end;
    wrongHash.replace(digest.toLatin1(),QByteArray(64,'0'));
    for(const auto &bad:{begin+end,begin+batch+batch+end,begin+wrongSequence+end,
                         wrongRequest,begin+batch+wrongHash,good+"x",begin+batch,
                         QByteArray(60000,'x')}) {
        decoder.reset(token);
        if(decoder.feed(bad) && decoder.finish(receipt))return false;
    }
    return true;
}

static int runF011LargeArtworkSelftest(const QString &nativeBinary) {
    if(!f011StreamNegativeSelftest())return 1;
    QTemporaryDir dir;
    if(!dir.isValid())return 2;
    PreviewWindow window(QString(),QString(),QString(),QString(),nativeBinary);
    window.show();
    auto *panel=window.nativePanel();
    auto *view=panel->findChild<QGraphicsView *>("f006KiCadArtwork");
    auto *filter=panel->findChild<QComboBox *>("f006KiCadLayerFilter");
    auto *summary=panel->findChild<QPlainTextEdit *>("f0031NativeSummary");
    auto *fit=panel->findChild<QPushButton *>("f011NativeFit");
    if(!view || !filter || !summary || !fit)return 2;
    auto *toggle=panel->findChild<QPushButton *>("f012InspectorToggle");
    auto *category=panel->findChild<QComboBox *>("f012InspectionCategory");
    auto *query=panel->findChild<QPushButton *>("f012InspectionQuery");
    auto *offset=panel->findChild<QSpinBox *>("f012InspectionOffset");
    auto *status=panel->findChild<QLabel *>("f012InspectionStatus");
    auto *raw=panel->findChild<QPlainTextEdit *>("f012InspectionDetails");
    auto *inspector=static_cast<F012Inspector *>(category?category->parentWidget():nullptr);
    if(!toggle || !category || !query || !offset || !status || !raw || !inspector)return 2;
    QJsonArray measurements,transportAB,closeSamples,inspectionMeasurements;
    qint64 workerRss=-1;
    auto finish=[&](qint64 &elapsedMs,qint64 &maxGap) {
        workerRss=-1;
        QEventLoop loop;
        QTimer timer;
        QElapsedTimer elapsed;elapsed.start();
        qint64 last=0;
        timer.setInterval(10);
        QObject::connect(&timer,&QTimer::timeout,&loop,[&] {
            const auto now=elapsed.elapsed();
            maxGap=qMax(maxGap,now-last);last=now;
            workerRss=qMax(workerRss,f011ResidentSample(panel->workerPidForTest()));
            if(!panel->busy() || now>120000)loop.quit();
        });
        if(panel->busy()) {timer.start();loop.exec();timer.stop();}
        elapsedMs=elapsed.elapsed();
        return !panel->busy() && panel->status().startsWith("COMPLETED");
    };
    const auto inspect=[&](int count,bool mixed) {
        if(!view->isVisible() || filter->count()!=3 ||
           view->scene()->items().size()!=count)return false;
        QVector<bool> seen(count,false);
        for(auto *graphics:view->scene()->items()) {
            if(!(graphics->flags() & QGraphicsItem::ItemIsSelectable))return false;
            if(const auto *ring=qgraphicsitem_cast<QGraphicsPathItem *>(graphics)) {
                if(!mixed)return false;
                const auto box=ring->path().boundingRect();
                const auto x=qRound(box.center().x()),y=qRound(box.center().y());
                const auto i=y*100+x;
                if(i<0 || i>=count || i%2!=1 || x<0 || x>=100 || seen.at(i) ||
                   qAbs(box.center().x()-x)>1e-9 || qAbs(box.center().y()-y)>1e-9 ||
                   qAbs(box.width()-0.6)>1e-9 || qAbs(box.height()-0.6)>1e-9 ||
                   ring->path().contains(QPointF(x,y)) ||
                   !ring->path().contains(QPointF(x+0.225,y)) ||
                   ring->data(0).toString()!="*" ||
                   !ring->data(1).toStringList().contains("F.Cu") ||
                   !ring->data(1).toStringList().contains("B.Cu") ||
                   !ring->toolTip().contains(QString("f011-synthetic-%1 /").arg(i)))
                    return false;
                seen[i]=true;
                continue;
            }
            const auto *line=qgraphicsitem_cast<QGraphicsLineItem *>(graphics);
            if(!line)return false;
            const auto p=line->line();
            const auto x=qRound(p.x1()),y=qRound(p.y1());
            const auto i=y*100+x;
            if(i<0 || i>=count || x<0 || x>=100 || seen.at(i) || (mixed && i%2!=0) ||
               qAbs(p.x1()-x)>1e-9 || qAbs(p.y1()-y)>1e-9 ||
               qAbs(p.x2()-x-0.25)>1e-9 || qAbs(p.y2()-y)>1e-9 ||
               qAbs(line->pen().widthF()-0.1)>1e-9 ||
               line->data(0).toString()!=(i%2 ? "B.Cu":"F.Cu") ||
               !line->toolTip().contains(QString("f011-synthetic-%1#0 /").arg(i)))
                return false;
            seen[i]=true;
        }
        filter->setCurrentIndex(filter->findText("F.Cu"));
        int visible=0;
        for(auto *item:view->scene()->items())if(item->isVisible())++visible;
        if(visible!=(mixed?count:(count+1)/2))return false;
        filter->setCurrentIndex(filter->findText("B.Cu"));
        visible=0;
        for(auto *item:view->scene()->items())if(item->isVisible())++visible;
        if(visible!=count/2)return false;
        filter->setCurrentIndex(0);
        auto *picked=view->scene()->itemAt(QPointF(0.125,0),QTransform());
        if(!picked || !picked->toolTip().contains("f011-synthetic-0#0 /"))return false;
        picked->setSelected(true);
        if(view->scene()->selectedItems().size()!=1 || !fit->isVisible())return false;
        const auto before=view->transform().m11();
        const QPoint local(80,80);
        const QPoint global=view->viewport()->mapToGlobal(local);
        auto *bar=view->horizontalScrollBar();
        // The viewport aspect ratio is platform-dependent. Establish the
        // actual scrollable range instead of assuming a fixed zoom factor.
        for(int n=0;n<40 && (n==0 || bar->maximum()<=bar->minimum()+20);++n) {
            QWheelEvent wheel(QPointF(local),QPointF(global),QPoint(),QPoint(0,120),
                Qt::NoButton,Qt::NoModifier,Qt::NoScrollPhase,false);
            QCoreApplication::sendEvent(view->viewport(),&wheel);
        }
        const auto zoomed=view->transform().m11();
        if(!(zoomed>before) || bar->maximum()<=bar->minimum()+20)return false;
        bar->setValue(bar->minimum()+(bar->maximum()-bar->minimum())/2);
        const int offset=bar->value();
        QMouseEvent press(QEvent::MouseButtonPress,QPointF(local),QPointF(global),
            Qt::MiddleButton,Qt::MiddleButton,Qt::NoModifier);
        QMouseEvent move(QEvent::MouseMove,QPointF(local+QPoint(10,0)),
            QPointF(global+QPoint(10,0)),Qt::NoButton,Qt::MiddleButton,Qt::NoModifier);
        QMouseEvent release(QEvent::MouseButtonRelease,QPointF(local+QPoint(10,0)),
            QPointF(global+QPoint(10,0)),Qt::MiddleButton,Qt::NoButton,Qt::NoModifier);
        QCoreApplication::sendEvent(view->viewport(),&press);
        QCoreApplication::sendEvent(view->viewport(),&move);
        QCoreApplication::sendEvent(view->viewport(),&release);
        if(bar->value()>=offset)return false;
        fit->click();
        return view->transform().m11()<zoomed;
    };
    const QList<QPair<int,bool>> cases={{128,false},{129,false},{1000,false},
        {10000,false},{10000,true},{10001,false},{100000,false}};
    for(const auto &entry:cases) {
        const int count=entry.first;
        const bool mixed=entry.second;
        const auto file=dir.filePath(QString("fixture-%1-%2.bsi").arg(count).arg(mixed?1:0));
        QProcess seed;
        auto env=QProcessEnvironment::systemEnvironment();
        env.remove("BOARDSI_ARTWORK_REQUEST");
        seed.setProcessEnvironment(env);
        QStringList arguments={"f011-seed",file,QString::number(count)};
        if(mixed)arguments<<"mixed";
        seed.start(nativeBinary,arguments);
        if(!seed.waitForStarted(15000) || !seed.waitForFinished(120000) ||
           seed.exitStatus()!=QProcess::NormalExit || seed.exitCode()!=0) {
            qWarning().noquote()<<"F011 fixture seed"<<count<<seed.readAllStandardError();
            return 1;
        }
        const auto legacy=QJsonDocument::fromJson(seed.readAllStandardOutput()).object();
        if(legacy.value("traces").toInt()!=(mixed?(count+1)/2:count) ||
           legacy.value("vias").toInt()!=(mixed?count/2:0) ||
           (count>128 && legacy.value("artwork_status").toString()!="NOT_RENDERED"))
            return 1;
        if(count==128) {
            // Controlled same-binary, same-file transport A/B: alternate order,
            // three pairs. Legacy and reconstructed framed receipts must agree
            // exactly. This does not claim an old-vs-new GUI speedup.
            const QString token="0123456789abcdef0123456789abcdef";
            for(int repeat=0;repeat<3;++repeat) {
                QJsonObject receipts[2];
                qint64 times[2]={0,0},sizes[2]={0,0};
                for(int order=0;order<2;++order) {
                    const int mode=(order+repeat)%2;
                    QProcess worker;
                    auto e=QProcessEnvironment::systemEnvironment();
                    e.remove("BOARDSI_ARTWORK_REQUEST");
                    if(mode==1)e.insert("BOARDSI_ARTWORK_REQUEST",token);
                    worker.setProcessEnvironment(e);
                    QElapsedTimer clock;clock.start();
                    worker.start(nativeBinary,{"open",file});
                    if(!worker.waitForStarted(15000) || !worker.waitForFinished(60000) ||
                       worker.exitStatus()!=QProcess::NormalExit || worker.exitCode()!=0)return 1;
                    const auto bytes=worker.readAllStandardOutput();
                    times[mode]=clock.elapsed();sizes[mode]=bytes.size();
                    if(mode==0) {
                        QJsonParseError parse;
                        const auto doc=QJsonDocument::fromJson(bytes,&parse);
                        if(parse.error!=QJsonParseError::NoError || !doc.isObject())return 1;
                        receipts[mode]=doc.object();
                    } else {
                        F011ArtworkStream decoder;decoder.reset(token);
                        if(!decoder.feed(bytes) || !decoder.finish(receipts[mode]))return 1;
                    }
                }
                if(receipts[0]!=receipts[1] ||
                   receipts[0].value("artwork_status").toString()!="RENDERABLE")return 1;
                transportAB.append(QJsonObject{{"repeat",repeat+1},
                    {"legacy_ms",times[0]},{"framed_ms",times[1]},
                    {"legacy_bytes",sizes[0]},{"framed_bytes",sizes[1]},
                    {"exact_receipt_equal",true}});
            }
        }
        qint64 elapsed=0,gap=0;
        if(!window.openNativeCandidate(file) || !finish(elapsed,gap) ||
           panel->loadedPath()!=file) {
            qWarning().noquote()<<"F011 open"<<count<<panel->status();return 1;
        }
        if(count<=10000) {
            if(!inspect(count,mixed))return 1;
        } else if(view->isVisible() || !view->scene()->items().isEmpty() ||
                  !summary->toPlainText().contains("ARTWORK NOT_RENDERED"))return 1;
        qint64 peak=boardSiPeakResidentBytes();
#ifdef Q_OS_LINUX
        peak=f011ResidentSample(QCoreApplication::applicationPid());
#endif
        const QJsonObject measurement{
            {"schema","boardsi-f011-large-artwork-v1"},{"count",count},
            {"rendered",count<=10000},{"mixed",mixed},
            {"open_ms",elapsed},{"max_timer_gap_ms",gap},
            {"worker_and_transfer_ms",panel->workerMsForTest()},
            {"scene_ms",panel->sceneMsForTest()},
            {"stream_bytes",qint64(panel->streamBytesForTest())},
            {"batch_count",panel->streamBatchesForTest()},
            {"qt_process_peak_rss_bytes",peak},
            {"worker_sampled_peak_rss_bytes",workerRss},
            {"worker_rss_scope","sampled while process alive; -1 means unavailable"},
            {"host_arch",QSysInfo::currentCpuArchitecture()},
            {"source_absent",true},{"physical_si","NOT_RUN"}};
        measurements.append(measurement);
        qInfo().noquote()<<QJsonDocument(measurement).toJson(QJsonDocument::Compact);
        if(!mixed) {
            // F012 alternative to a retained model: fresh read-only Worker per
            // query. Fixed pre-run bounds derived from F011 runner timings:
            // <=10k: 20 s; larger structural board: 60 s; event gap <=500 ms,
            // reply <60k bytes, sampled Worker RSS <=1 GiB, Qt peak <=1 GiB.
            // Compare initial / near-tail / repeat query on identical bytes.
            view->scene()->clearSelection();
            toggle->click();
            category->setCurrentIndex(category->findData("traces"));
            QFile snapshot(file);
            if(!snapshot.open(QIODevice::ReadOnly))return 1;
            const auto digest=QCryptographicHash::hash(snapshot.readAll(),QCryptographicHash::Sha256);
            snapshot.close();
            for(int start:{0,qMax(0,count-1),0}) {
                offset->setValue(start);
                if(!query->isEnabled())return 1;
                QElapsedTimer clock;clock.start();
                qint64 last=0,inspectionGap=0,sampledRss=-1;
                QEventLoop loop;
                QTimer poll;poll.setInterval(10);
                QObject::connect(&poll,&QTimer::timeout,&loop,[&] {
                    const auto now=clock.elapsed();
                    inspectionGap=qMax(inspectionGap,now-last);last=now;
                    sampledRss=qMax(sampledRss,f011ResidentSample(inspector->inspectionWorkerPidForTest()));
                    if(query->isEnabled() || now>65000)loop.quit();
                });
                query->click();
                if(!query->isEnabled()) {poll.start();loop.exec();poll.stop();}
                const auto page=QJsonDocument::fromJson(raw->toPlainText().toUtf8()).array();
                const auto elapsedMs=clock.elapsed();
                const auto qtRss=f011ResidentSample(QCoreApplication::applicationPid());
                const auto bytes=inspector->inspectionBytesForTest();
                if(!query->isEnabled() || !status->text().startsWith("Native snapshot /") ||
                   page.isEmpty() || page.size()>128 || (start==count-1 && page.size()!=1) ||
                   page.first().toObject().value("meta").toObject().value("id").toObject()
                      .value("source_object").toString()!=QString("f011-synthetic-%1").arg(start) ||
                   bytes<=0 || bytes>=60000 || elapsedMs>(count<=10000?20000:60000) ||
                   inspectionGap>500 || sampledRss>1073741824 || qtRss<=0 || qtRss>1073741824 ||
                   (count>=10000 && sampledRss<=0)) {
                    qWarning().noquote()<<"F012 bounded query failure"<<count<<start<<status->text()
                       <<elapsedMs<<inspectionGap<<bytes<<sampledRss<<qtRss;return 1;
                }
                const QJsonObject record{{"count",count},{"offset",start},
                    {"page_ms",elapsedMs},{"max_timer_gap_ms",inspectionGap},
                    {"response_bytes",qint64(bytes)},{"rows",page.size()},
                    {"worker_sampled_peak_rss_bytes",sampledRss},{"qt_peak_or_sampled_rss_bytes",qtRss},
                    {"mode","fresh-read-only-worker"},{"physical_si","NOT_RUN"}};
                inspectionMeasurements.append(record);
                qInfo().noquote()<<QJsonDocument(record).toJson(QJsonDocument::Compact);
            }
            if(!snapshot.open(QIODevice::ReadOnly) ||
               QCryptographicHash::hash(snapshot.readAll(),QCryptographicHash::Sha256)!=digest)return 1;
            snapshot.close();
            toggle->click();
        }
        if(count==10000) {
            // Hold the *actual* renderer after its first batch while keeping
            // the prior scene visible, then cancel via the production slot.
            const QString oldPath=panel->loadedPath();
            auto *oldScene=view->scene();
            panel->holdRenderForTest(true);
            if(!window.openNativeCandidate(file))return 1;
            QEventLoop cancelLoop;
            QTimer poll;
            QElapsedTimer cancelClock;cancelClock.start();
            bool cancelled=false;
            poll.setInterval(5);
            QObject::connect(&poll,&QTimer::timeout,&cancelLoop,[&]{
                if(panel->stagedCountForTest()>0) {
                    if(view->scene()==oldScene && panel->loadedPath()==oldPath) {
                        panel->cancel();cancelled=panel->status().startsWith("CANCELLED");
                    }
                    cancelLoop.quit();
                } else if(!panel->busy() || cancelClock.elapsed()>60000)cancelLoop.quit();
            });
            poll.start();cancelLoop.exec();poll.stop();
            if(panel->busy())panel->cancel();
            panel->holdRenderForTest(false);
            if(!cancelled || panel->busy() || panel->loadedPath()!=oldPath ||
               view->scene()!=oldScene || view->scene()->items().size()!=count)return 1;
            const auto copy=dir.filePath(mixed?"mixed-copy.bsi":"copy.bsi");
            elapsed=0;gap=0;
            if(!window.copyNativeCandidate(copy) || !finish(elapsed,gap) ||
               !window.openNativeCandidate(copy) || !finish(elapsed,gap) ||
               !inspect(count,mixed))return 1;
            // Close and reopen the same native snapshot; assert no stale
            // scene objects or identity survive the explicit close.
            for(int repeat=0;repeat<2;++repeat) {
                panel->forget();
                if(panel->hasLoaded() || !view->scene()->items().isEmpty() ||
                   view->isVisible())return 1;
                QCoreApplication::sendPostedEvents(nullptr,QEvent::DeferredDelete);
                const auto closedRss=f011ResidentSample(QCoreApplication::applicationPid(),false);
                elapsed=0;gap=0;
                if(!window.openNativeCandidate(copy) || !finish(elapsed,gap) ||
                   !inspect(count,mixed))return 1;
                closeSamples.append(QJsonObject{{"mixed",mixed},{"repeat",repeat+1},
                    {"closed_scene_items",0},{"closed_resident_bytes",closedRss},
                    {"reopen_resident_bytes",f011ResidentSample(QCoreApplication::applicationPid(),false)},
                    {"reopen_ms",elapsed},{"max_timer_gap_ms",gap}});
            }
        }
    }
    const QJsonObject evidence{{"schema","boardsi-f011-qualification-v1"},{"pass",true},
        {"host_arch",QSysInfo::currentCpuArchitecture()},
        {"host_os",QSysInfo::productType()},
        {"source_sha",qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"cases",measurements},{"transport_ab",transportAB},{"close_reopen",closeSamples},
        {"f012_inspection",inspectionMeasurements},
        {"physical_si","NOT_RUN"}};
    QSaveFile proof("qt-f011-qualification.json");
    const auto encoded=QJsonDocument(evidence).toJson(QJsonDocument::Indented);
    if(!proof.open(QIODevice::WriteOnly) || proof.write(encoded)!=encoded.size() ||
       !proof.commit())return 1;
    qInfo().noquote()<<"F011 actual Main Qt large artwork and stream negative cases PASS";
    return 0;
}
