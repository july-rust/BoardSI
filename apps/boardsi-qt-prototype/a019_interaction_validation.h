// A_019 native interaction/fallback regression: exact source-nm spatial
// picking and active Qt Quick Scene Graph operations. NOT a driver-reset test.
#pragma once
#include <QElapsedTimer>
#include <QEventLoop>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QFile>
#include <QTimer>
#include <algorithm>
#include <cmath>
#include <vector>

#ifdef BOARDSI_RHI_ARTWORK
static qint64 a019Percentile(std::vector<qint64> values,int numerator) {
    if(values.empty()) return -1;
    std::sort(values.begin(),values.end());
    return values[static_cast<size_t>(
        std::ceil(values.size()*numerator/100.0))-1];
}

static bool a019WaitForFrame(BoardSiRhiView &view,int deadlineMs) {
    if(view.failed()) return false;
    QEventLoop loop;
    QTimer poll;
    QObject::connect(&poll,&QTimer::timeout,&loop,[&] {
        if(view.status().startsWith("RHI_FRAME_") || view.failed()) loop.quit();
    });
    QTimer::singleShot(deadlineMs,&loop,&QEventLoop::quit);
    poll.start(10);
    if(!view.status().startsWith("RHI_FRAME_")) loop.exec();
    poll.stop();
    return view.status().endsWith("_HARDWARE_CONFIRMED");
}

static qint64 a019RenderActionToSwap(BoardSiRhiView &view,
                                     const std::function<void()> &action) {
    QEventLoop loop;
    QElapsedTimer clock;
    qint64 latencyNs=-1;
    clock.start();
    const quint64 requiredRevision=
        view.itemForTest()->requestedSceneRevision()+1;
    const auto connection=QObject::connect(
        view.quickWindowForTest(),&QQuickWindow::frameSwapped,&loop,[&] {
            // Reject swaps from a frame already in flight before this action.
            if(view.itemForTest()->syncedSceneRevision()<requiredRevision)
                return;
            const qint64 measured=clock.nsecsElapsed();
            QMetaObject::invokeMethod(&loop,[&loop,&latencyNs,measured] {
                if(latencyNs<0) {latencyNs=measured;loop.quit();}
            },Qt::QueuedConnection);
        },Qt::DirectConnection);
    action();
    QTimer::singleShot(4000,&loop,&QEventLoop::quit);
    loop.exec();
    QObject::disconnect(connection);
    return latencyNs<0 ? -1 : (latencyNs+500000)/1000000;
}
#endif // BOARDSI_RHI_ARTWORK

static int runA019NativeInteractionValidation() {
    QJsonArray records;
    bool pass=true;
    QString error;
#ifdef BOARDSI_RHI_ARTWORK
    if(QGuiApplication::platformName()=="offscreen" ||
       QGuiApplication::platformName()=="minimal") {
        pass=false;error="HEADLESS_INTERACTION_NOT_RUN";
    } else {
        constexpr qint64 base=90000000000LL;
        for(const int count : {10000,100000,1000000}) {
            std::vector<ImportedTrack> tracks;
            tracks.reserve(static_cast<size_t>(count));
            for(int i=0;i<count-1;++i) {
                ImportedTrack t;
                t.net="SYNTHETIC";t.layer=1;
                t.x0_nm=base+static_cast<qint64>(i%1000)*200000;
                t.y0_nm=base+static_cast<qint64>(i/1000)*200000;
                t.x1_nm=t.x0_nm+120000;t.y1_nm=t.y0_nm;
                t.width_nm=10000;
                tracks.push_back(t);
            }
            // Isolated last trace: no competing selectable geometry within
            // the screen-pixel hit tolerance even at 1M object Fit.
            ImportedTrack sentinel;
            sentinel.net="SYNTHETIC";sentinel.layer=1;
            sentinel.x0_nm=base+300000000;
            sentinel.y0_nm=base+100000000;
            sentinel.x1_nm=sentinel.x0_nm+120000;
            sentinel.y1_nm=sentinel.y0_nm;
            sentinel.width_nm=20000;
            tracks.push_back(sentinel);

            BoardSiRhiView view;
            view.resize(820,520);
            view.show();
            QElapsedTimer loadClock;loadClock.start();
            view.load(4,tracks,{},{});
            const qint64 loadMs=loadClock.elapsed();
            const bool firstFrame=a019WaitForFrame(view,10000);
            auto *item=view.itemForTest();
            const bool countExact=item->primitiveCount()==count;
            const bool indexed=item->indexBucketsForTest()>0 &&
                              item->spanningTracksForTest()<=static_cast<size_t>(count);

            auto query=[&] {
                return item->trackAtForTest(item->screenPointForTest(
                    sentinel.x0_nm+60000,sentinel.y0_nm));
            };
            std::vector<qint64> pickNs;
            bool picked=firstFrame && indexed && countExact;
            for(int i=0;i<40;++i) {
                QElapsedTimer t;t.start();
                const int got=query();
                pickNs.push_back(t.nsecsElapsed());
                if(got!=count-1) picked=false;
            }
            QVector<bool> mask(5,true);
            mask[1]=false;
            view.filter(mask,"All nets");
            const bool filtered=query()==-1;
            mask[1]=true;
            view.filter(mask,"All nets");
            const bool restored=query()==count-1;
            const qint64 panToSwap=a019RenderActionToSwap(view,[&] {
                item->panByForTest(QPointF(16,-11));
            });
            const bool panPick=query()==count-1;
            const qint64 zoomToSwap=a019RenderActionToSwap(view,[&] {
                view.zoom(1.2);
            });
            const bool zoomPick=query()==count-1;
            view.fit();
            const bool fitPick=query()==count-1;

            // A hidden tab must never be misclassified as GPU loss.
            view.hide();
            QCoreApplication::processEvents();
            view.show();
            const bool postShowPending=
                view.status()=="RHI_PENDING_FRAME_HW_NOT_VERIFIED";
            const quint64 postShowRevision=item->requestedSceneRevision();
            const bool postShowFrame=a019WaitForFrame(view,6000) &&
                item->syncedSceneRevision()>=postShowRevision;
            QCoreApplication::processEvents();
            const bool tabSurvived=postShowPending && postShowFrame &&
                !view.failed() && query()==count-1;

            int fallbackEvents=0;
            view.onChanged([&] {++fallbackEvents;});
            view.injectSceneGraphFailureForTest();
            const bool failClosed=view.failed() &&
                view.status()=="CPU_FALLBACK_RHI_SCENEGRAPH_ERROR" &&
                fallbackEvents==1 && item->primitiveCount()==count &&
                query()==count-1;
            view.injectSceneGraphFailureForTest();
            const bool noRepeatedCallback=fallbackEvents==1;

            const bool casePass=firstFrame && countExact && indexed && picked &&
                 filtered && restored && panPick && zoomPick && fitPick &&
                 panToSwap>=0 && zoomToSwap>=0 && tabSurvived &&
                 failClosed && noRepeatedCallback;
            if(!casePass) {
                pass=false;
                error+=QString("%1 objects firstFrame=%2 picked=%3 filtered=%4 "
                               "restored=%5 pan=%6 zoom=%7 fit=%8 swap=%9/%10 "
                               "tab=%11 fallback=%12 single=%13; ")
                    .arg(count).arg(firstFrame).arg(picked).arg(filtered)
                    .arg(restored).arg(panPick).arg(zoomPick).arg(fitPick)
                    .arg(panToSwap).arg(zoomToSwap).arg(tabSurvived)
                    .arg(failClosed).arg(noRepeatedCallback);
            }
            records.append(QJsonObject{
                {"objects",count},{"pass",casePass},
                {"source_load_and_index_ms",loadMs},
                {"spatial_index_buckets",static_cast<qint64>(item->indexBucketsForTest())},
                {"spanning_routes",static_cast<qint64>(item->spanningTracksForTest())},
                {"picking_ns_p50",a019Percentile(pickNs,50)},
                {"picking_ns_p95",a019Percentile(pickNs,95)},
                {"pan_action_to_first_swap_ms",panToSwap},
                {"zoom_action_to_first_swap_ms",zoomToSwap},
                {"picking_exact",picked && panPick && zoomPick && fitPick},
                {"layer_filter_restored",filtered && restored},
                {"post_show_new_frame_verified",postShowFrame},
                {"repeated_tab_survived",tabSurvived},
                {"injected_error_cpu_fallback",failClosed && noRepeatedCallback},
                {"actual_gpu_driver_reset","NOT_RUN"}
            });
        }
    }
#else
    pass=false;error="QT_QUICK_UNAVAILABLE";
#endif
    const QJsonObject result{
        {"schema","boardsi-a019-interaction-v1"},{"pass",pass},
        {"cases",records},{"failure",error},
        {"source_sha",qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"scope","Native synthetic UI interaction, source picking and injected error; NOT a real GPU device reset"},
        {"gpu_vram","NOT_MEASURED"},
        {"physical_si","NOT_RUN"}
    };
    QFile file("qt-preview-a019-interaction-results.json");
    if(!file.open(QIODevice::WriteOnly) ||
       file.write(QJsonDocument(result).toJson(QJsonDocument::Indented))<1)
        return 2;
    qInfo().noquote()<<QJsonDocument(result).toJson(QJsonDocument::Compact);
    return pass?0:1;
}
