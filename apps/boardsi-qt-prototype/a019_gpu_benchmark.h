// BoardSI A_019: native Qt Quick/RHI frame benchmark. Public synthetic data ONLY.
// These are wall-time presentation intervals, not GPU-clock shader timings.
// GPU hardware claims come exclusively from the separate native adapter probe.
#pragma once
#include <QElapsedTimer>
#include <QEventLoop>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QFile>
#include <QSysInfo>
#include <QTimer>
#include <algorithm>
#include <numeric>
#include <cmath>
#ifdef Q_OS_WIN
#include <windows.h>
#include <psapi.h>
// Win32 compatibility aliases otherwise erase existing C++ variables called
// near/far in the preview tests, and may hijack std::min/std::max.
#if defined(near)
#undef near
#endif
#if defined(far)
#undef far
#endif
#if defined(min)
#undef min
#endif
#if defined(max)
#undef max
#endif
#elif defined(Q_OS_MACOS)
#include <sys/resource.h>
#endif

[[maybe_unused]] static qint64 boardSiPeakResidentBytes() {
#ifdef Q_OS_WIN
    PROCESS_MEMORY_COUNTERS c{};
    c.cb=sizeof(c);
    return GetProcessMemoryInfo(GetCurrentProcess(),&c,sizeof(c)) ?
        static_cast<qint64>(c.PeakWorkingSetSize) : -1;
#elif defined(Q_OS_MACOS)
    struct rusage usage{};
    return getrusage(RUSAGE_SELF,&usage)==0 ?
        static_cast<qint64>(usage.ru_maxrss) : -1;
#else
    return -1;
#endif
}

static int runA019NativeRhiBenchmark() {
    QJsonArray cases;
    QString failure;
    bool pass=true;
    const QString platform=QGuiApplication::platformName();
#ifdef BOARDSI_RHI_ARTWORK
    if (platform=="offscreen" || platform=="minimal") {
        failure="HEADLESS_NOT_MEASURED";
        pass=false;
    } else {
        constexpr qint64 origin=90000000000LL;
        for (int count : {10000,100000,1000000}) {
            // 95% traces, 4% drill-free pads, 1% true via geometry.
            // Coordinates deliberately use large nm absolute offsets and are
            // arranged in a deterministic 1000-column raster.
            std::vector<ImportedTrack> tracks;
            std::vector<ImportedPad> pads;
            std::vector<ImportedVia> vias;
            tracks.reserve(static_cast<size_t>(count)*95/100);
            pads.reserve(static_cast<size_t>(count)*4/100);
            vias.reserve(static_cast<size_t>(count)/100);
            for (int i=0;i<count;++i) {
                const qint64 x=origin+static_cast<qint64>(i%1000)*200000;
                const qint64 y=origin+static_cast<qint64>(i/1000)*200000;
                if (i%100==0) {
                    ImportedVia v;
                    v.name="V";v.net="SYNTHETIC";
                    v.x_nm=x;v.y_nm=y;v.diameter_nm=75000;v.drill_nm=25000;
                    v.firstLayer=1;v.lastLayer=4;vias.push_back(v);
                } else if(i%100<5) {
                    ImportedPad p;
                    p.reference="U";p.pin="1";p.shape="RF";p.side="T";
                    p.layer=1;p.x_nm=x;p.y_nm=y;
                    p.width_nm=60000;p.height_nm=40000;pads.push_back(p);
                } else {
                    ImportedTrack t;
                    t.layer=1;t.net="SYNTHETIC";
                    t.x0_nm=x;t.y0_nm=y;t.x1_nm=x+120000;t.y1_nm=y;
                    t.width_nm=10000;tracks.push_back(t);
                }
            }
            QElapsedTimer loadTimer;loadTimer.start();
            BoardSiRhiView view;
            view.resize(820,520);
            view.show();
            view.load(4,tracks,vias,pads);
            const qint64 sourceLoadMs=loadTimer.elapsed();
            const bool countOkay=view.primitiveCount()==count &&
                vias.size()==static_cast<size_t>(count/100) &&
                pads.size()==static_cast<size_t>(count*4/100) &&
                tracks.size()==static_cast<size_t>(count*95/100);
            // Require the generation-aware native frame receipt to become
            // available. No previous frame can qualify a later artwork load.
            QEventLoop firstLoop;
            QTimer firstPoll;
            QObject::connect(&firstPoll,&QTimer::timeout,&firstLoop,[&] {
                if(view.status().startsWith("RHI_FRAME_") || view.failed())
                    firstLoop.quit();
            });
            QTimer::singleShot(10000,&firstLoop,&QEventLoop::quit);
            firstPoll.start(12);
            if(!view.status().startsWith("RHI_FRAME_")) firstLoop.exec();
            firstPoll.stop();
            const qint64 firstReceiptMs=loadTimer.elapsed();
            const QString firstStatus=view.status();
            const bool firstFrame=firstStatus.startsWith("RHI_FRAME_");
            const bool hardware=firstStatus.endsWith("_HARDWARE_CONFIRMED");
            // The Qt signal is emitted when a frame is presented. Measure
            // several actual successive frame-swap intervals on the SAME
            // populated scene; do not report these as GPU timer queries.
            std::vector<qint64> intervals;
            if(firstFrame) {
                QEventLoop loop;
                QElapsedTimer frameClock;frameClock.start();
                auto *item=view.itemForTest();
                const auto connection=QObject::connect(
                    view.quickWindowForTest(),&QQuickWindow::frameSwapped,
                    &loop,[&] {
                        intervals.push_back(frameClock.restart());
                        if(intervals.size()>=8) {loop.quit();return;}
                        QTimer::singleShot(0,item,[item]{item->update();});
                    },Qt::QueuedConnection);
                QTimer::singleShot(10000,&loop,&QEventLoop::quit);
                item->update();
                loop.exec();
                QObject::disconnect(connection);
            }
            std::sort(intervals.begin(),intervals.end());
            const qint64 p50=intervals.empty() ? -1 :
                intervals[(intervals.size()-1)/2];
            const qint64 p95=intervals.empty() ? -1 :
                intervals[static_cast<size_t>(std::ceil(intervals.size()*0.95))-1];
            const qint64 peak=boardSiPeakResidentBytes();
            const bool casePass=countOkay && firstFrame && hardware &&
                                intervals.size()>=4 && peak>0;
            if(!casePass) {
                pass=false;
                failure+=QString("%1 objects: %2, frames=%3, peak=%4; ")
                    .arg(count).arg(firstStatus).arg(intervals.size()).arg(peak);
            }
            cases.append(QJsonObject{
                {"objects",count}, {"trace_objects",static_cast<int>(tracks.size())},
                {"pad_objects",static_cast<int>(pads.size())},
                {"via_objects",static_cast<int>(vias.size())},
                {"source_load_and_setup_ms",sourceLoadMs},
                {"first_frame_receipt_ms",firstReceiptMs},
                {"backend_status",firstStatus},
                {"hardware_adapter_confirmed",hardware},
                {"presented_frame_samples",static_cast<int>(intervals.size())},
                {"frame_swap_interval_p50_ms",p50},
                {"frame_swap_interval_p95_ms",p95},
                {"peak_resident_bytes",peak}, {"pass",casePass}
            });
        }
    }
#else
    pass=false;failure="QT_QUICK_UNAVAILABLE";
#endif
    QJsonObject result{
        {"schema","boardsi-a019-native-rhi-benchmark-v1"},
        {"pass",pass},{"cases",cases},{"failure",failure},
        {"host_arch",QSysInfo::currentCpuArchitecture()},
        {"native_platform",platform},
        {"source_sha",qEnvironmentVariable("BOARDSI_SOURCE_HEAD_SHA")},
        {"metric_scope","Synthetic scene; CPU load and frame-present wall-time; not GPU shader timings"},
        {"gpu_vram","NOT_MEASURED"},
        {"input_privacy","PUBLIC_SYNTHETIC_ONLY"},
        {"physical_si","NOT_RUN"}
    };
    QFile out("qt-preview-a019-native-benchmark.json");
    if(!out.open(QIODevice::WriteOnly) ||
       out.write(QJsonDocument(result).toJson(QJsonDocument::Indented))<1)
        return 2;
    qInfo().noquote()<<QJsonDocument(result).toJson(QJsonDocument::Compact);
    return pass?0:1;
}
