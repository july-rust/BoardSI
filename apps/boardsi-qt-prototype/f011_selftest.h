// Actual Main Qt and Native Worker qualification; no mock GUI or CAD claim.
#pragma once
#include <QVector>
#include <QPair>

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
    auto finish=[&](qint64 &elapsedMs,qint64 &maxGap) {
        QEventLoop loop;
        QTimer timer;
        QElapsedTimer elapsed;elapsed.start();
        qint64 last=0;
        timer.setInterval(10);
        QObject::connect(&timer,&QTimer::timeout,&loop,[&] {
            const auto now=elapsed.elapsed();
            maxGap=qMax(maxGap,now-last);last=now;
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
        for(int n=0;n<6;++n) {
            QWheelEvent wheel(QPointF(local),QPointF(global),QPoint(),QPoint(0,120),
                Qt::NoButton,Qt::NoModifier,Qt::NoScrollPhase,false);
            QCoreApplication::sendEvent(view->viewport(),&wheel);
        }
        const auto zoomed=view->transform().m11();
        if(!(zoomed>before))return false;
        auto *bar=view->horizontalScrollBar();
        if(bar->maximum()<=bar->minimum()+20)return false;
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
        QFile status("/proc/self/status");
        if(status.open(QIODevice::ReadOnly)) {
            const auto match=QRegularExpression("VmHWM:\\s+(\\d+)\\s+kB")
                .match(QString::fromLatin1(status.readAll()));
            if(match.hasMatch())peak=match.captured(1).toLongLong()*1024;
        }
#endif
        qInfo().noquote()<<QJsonDocument(QJsonObject{
            {"schema","boardsi-f011-large-artwork-v1"},{"count",count},
            {"rendered",count<=10000},{"mixed",mixed},
            {"open_ms",elapsed},{"max_timer_gap_ms",gap},
            {"worker_and_transfer_ms",panel->workerMsForTest()},
            {"scene_ms",panel->sceneMsForTest()},
            {"stream_bytes",qint64(panel->streamBytesForTest())},
            {"batch_count",panel->streamBatchesForTest()},
            {"qt_process_peak_rss_bytes",peak},
            {"host_arch",QSysInfo::currentCpuArchitecture()},
            {"source_absent",true},{"physical_si","NOT_RUN"}}).toJson(QJsonDocument::Compact);
        if(count==10000) {
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
                elapsed=0;gap=0;
                if(!window.openNativeCandidate(copy) || !finish(elapsed,gap) ||
                   !inspect(count,mixed))return 1;
            }
        }
    }
    qInfo().noquote()<<"F011 actual Main Qt large artwork and stream negative cases PASS";
    return 0;
}
