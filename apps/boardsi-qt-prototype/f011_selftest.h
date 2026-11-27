// Actual Main Qt and Native Worker qualification; no mock GUI or CAD claim.
#pragma once

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
    if(!view || !filter || !summary)return 2;
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
    const auto inspect=[&](int count) {
        if(!view->isVisible() || filter->count()!=3 ||
           view->scene()->items().size()!=count)return false;
        QVector<bool> seen(count,false);
        for(auto *graphics:view->scene()->items()) {
            const auto *line=qgraphicsitem_cast<QGraphicsLineItem *>(graphics);
            if(!line || !(line->flags() & QGraphicsItem::ItemIsSelectable))return false;
            const auto p=line->line();
            const auto x=qRound(p.x1()),y=qRound(p.y1());
            const auto i=y*100+x;
            if(i<0 || i>=count || x<0 || x>=100 || seen.at(i) ||
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
        if(visible!=(count+1)/2)return false;
        filter->setCurrentIndex(0);
        auto *picked=view->scene()->itemAt(QPointF(0.125,0),QTransform());
        if(!picked || !picked->toolTip().contains("f011-synthetic-0#0 /"))return false;
        picked->setSelected(true);
        return view->scene()->selectedItems().size()==1;
    };
    for(const int count:{128,129,1000,10000,10001}) {
        const auto file=dir.filePath(QString("fixture-%1.bsi").arg(count));
        QProcess seed;
        auto env=QProcessEnvironment::systemEnvironment();
        env.remove("BOARDSI_ARTWORK_REQUEST");
        seed.setProcessEnvironment(env);
        seed.start(nativeBinary,{"f011-seed",file,QString::number(count)});
        if(!seed.waitForStarted(15000) || !seed.waitForFinished(120000) ||
           seed.exitStatus()!=QProcess::NormalExit || seed.exitCode()!=0) {
            qWarning().noquote()<<"F011 fixture seed"<<count<<seed.readAllStandardError();
            return 1;
        }
        const auto legacy=QJsonDocument::fromJson(seed.readAllStandardOutput()).object();
        if(legacy.value("traces").toInt()!=count ||
           (count>128 && legacy.value("artwork_status").toString()!="NOT_RENDERED"))
            return 1;
        qint64 elapsed=0,gap=0;
        if(!window.openNativeCandidate(file) || !finish(elapsed,gap) ||
           panel->loadedPath()!=file) {
            qWarning().noquote()<<"F011 open"<<count<<panel->status();return 1;
        }
        if(count<=10000) {
            if(!inspect(count))return 1;
        } else if(view->isVisible() || !view->scene()->items().isEmpty() ||
                  !summary->toPlainText().contains("ARTWORK NOT_RENDERED"))return 1;
        qInfo().noquote()<<QJsonDocument(QJsonObject{
            {"schema","boardsi-f011-large-artwork-v1"},{"count",count},
            {"rendered",count<=10000},{"open_ms",elapsed},{"max_timer_gap_ms",gap},
            {"source_absent",true},{"physical_si","NOT_RUN"}}).toJson(QJsonDocument::Compact);
        if(count==10000) {
            const auto copy=dir.filePath("copy.bsi");
            elapsed=0;gap=0;
            if(!window.copyNativeCandidate(copy) || !finish(elapsed,gap) ||
               !window.openNativeCandidate(copy) || !finish(elapsed,gap) ||
               !inspect(count))return 1;
        }
    }
    qInfo().noquote()<<"F011 actual Main Qt large artwork and stream negative cases PASS";
    return 0;
}
