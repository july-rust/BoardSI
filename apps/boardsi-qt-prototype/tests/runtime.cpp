#define BOARDSI_RUNTIME_TEST
#include "../main.cpp"
#include <QEventLoop>
#include <QFile>
#include <QSysInfo>
#include <QThread>
#include <QCryptographicHash>
#include <QImage>
#include <algorithm>
#include <atomic>
#include <chrono>
#include <functional>
#include <stdexcept>
#include <thread>
#include <vector>

class RuntimeValidation {
    QString rust, python, fixture;
    QJsonArray cases;
    QJsonObject metrics;
    int assertions = 0;
    using Steady = std::chrono::steady_clock;
    void check(bool ok, const char *message) {
        ++assertions;
        if (!ok) throw std::runtime_error(message);
    }
    // Nested Qt loops dispatch UI/IPC/timers; no synchronous process/read waits.
    bool until(const std::function<bool()> &predicate, int timeout = 3000) {
        QEventLoop loop;
        QTimer poll, deadline;
        poll.setInterval(2);
        deadline.setSingleShot(true);
        QObject::connect(&poll, &QTimer::timeout, &loop, [&] { if (predicate()) loop.quit(); });
        QObject::connect(&deadline, &QTimer::timeout, &loop, &QEventLoop::quit);
        poll.start(); deadline.start(timeout);
        if (!predicate()) loop.exec();
        return predicate();
    }
    void start(Window &w) {
        w.show();
        QElapsedTimer t; t.start(); w.start->click();
        check(w.status->text() == "Starting Worker", "Start display missing");
        metrics["start_display_max_ms"] = qMax(metrics.value("start_display_max_ms").toInt(), int(t.elapsed()));
    }
    void running(Window &w) {
        check(until([&] { return w.state == Window::Running; }), "Started not received");
    }
    void exited(Window &w, int timeout = 3000) {
        check(until([&] { return w.exitSeen && w.start->isEnabled(); }, timeout), "OS exit not drained/reaped");
        check(w.process.state() == QProcess::NotRunning, "Worker alive after exit");
        check(!w.cancel->isEnabled() && !w.force->isEnabled(), "Buttons not cleaned up");
    }
    void cancel(Window &w) {
        QElapsedTimer t; t.start(); w.cancel->click();
        check(w.state == Window::CancelRequested, "Cancel request state missing");
        metrics["cancel_display_max_ms"] = qMax(metrics.value("cancel_display_max_ms").toInt(), int(t.elapsed()));
    }
    void record(const QString &name, Window &w) {
        cases.append(QJsonObject{{"name", name}, {"pass", true}, {"exit_code", w.exitCode},
                                {"exit_status", int(w.exitKind)}, {"error", w.error->text()}});
    }
    void probes(Window &w, const QString &prefix, int count, int spacing) {
        std::vector<double> delays;
        QElapsedTimer tickClock; tickClock.start();
        qint64 previous = 0, maxGap = 0;
        QTimer tick;
        tick.setInterval(10);
        QObject::connect(&tick, &QTimer::timeout, &w, [&] {
            const auto now = tickClock.elapsed(); maxGap = qMax(maxGap, now - previous); previous = now;
        });
        tick.start();
        std::atomic<bool> posted{false};
        std::thread producer([&] {
            for (int i = 0; i < count; ++i) {
                const auto at = Steady::now();
                QMetaObject::invokeMethod(&w, [&, at] {
                    delays.push_back(std::chrono::duration<double, std::milli>(Steady::now() - at).count());
                    // Perform a real widget update for each queued interaction.
                    w.setWindowTitle(QString("UI probe %1").arg(delays.size()));
                }, Qt::QueuedConnection);
                std::this_thread::sleep_for(std::chrono::milliseconds(spacing));
            }
            posted.store(true);
        });
        const bool received = until([&] { return posted.load() && int(delays.size()) == count; }, 15000);
        // posted proves producer is done; joining cannot wait for Worker or GUI.
        producer.join(); tick.stop();
        check(received, "Queued UI probes timed out");
        std::sort(delays.begin(), delays.end());
        const double p95 = delays[(delays.size() * 95 + 99) / 100 - 1];
        metrics[prefix + "_count"] = count;
        metrics[prefix + "_p95_ms"] = p95;
        metrics[prefix + "_max_ms"] = delays.back();
        metrics[prefix + "_timer_max_gap_ms"] = maxGap;
        check(p95 <= 200 && delays.back() <= 1000 && maxGap <= 1000, "UI responsiveness target missed");
        check(w.isVisible(), "GUI did not survive probes");
    }
    void normal() {
        Window w(rust); start(w); running(w);
        probes(w, "normal_ui", 150, 10);
        exited(w, 8000);
        check(w.state == Window::Completed && w.resultCount == 100, "Rust completion/result mismatch");
        check(w.progress->value() == 100 && w.results->text() == "Synthetic results: 100", "Progress/results display mismatch");
        const auto expectedWorker = qEnvironmentVariable("BOARDSI_EXPECTED_WORKER_ARCH");
        if (!expectedWorker.isEmpty()) check(w.diagnostics.contains("arch="+expectedWorker+" "), "Rust Worker executable architecture mismatch");
        const QImage frame = w.grab().toImage();
        check(!frame.isNull() && frame.width()>0 && frame.height()>0, "Widget raster rendering failed");
        metrics["render_width_px"]=frame.width(); metrics["render_height_px"]=frame.height();
        metrics["render_sha256"]=QString::fromLatin1(QCryptographicHash::hash(QByteArray(reinterpret_cast<const char *>(frame.constBits()),frame.sizeInBytes()),QCryptographicHash::Sha256).toHex());
        std::vector<qint64> hb;
        qint64 maxUpdate = 0; int resultSamples = 0, progressSamples = 0;
        for (const auto &v : w.observations) {
            const auto o = v.toObject(); const auto type = o.value("type").toString();
            if (type == "Heartbeat") hb.push_back(qint64(o.value("received_ms").toDouble()));
            if (type == "Result" || type == "Progress") {
                maxUpdate = qMax(maxUpdate, qint64(o.value("updated_ms").toDouble()-o.value("received_ms").toDouble()));
                if (type == "Result") ++resultSamples; else ++progressSamples;
            }
        }
        check(hb.size() >= 4 && resultSamples == 100 && progressSamples == 100, "Missing measurement samples");
        qint64 minGap = 100000, maxGap = 0;
        for (size_t i=1; i<hb.size(); ++i) { minGap=qMin(minGap,hb[i]-hb[i-1]); maxGap=qMax(maxGap,hb[i]-hb[i-1]); }
        metrics["heartbeat_samples"] = int(hb.size()); metrics["heartbeat_min_gap_ms"] = minGap; metrics["heartbeat_max_gap_ms"] = maxGap;
        metrics["receive_to_widget_update_max_ms"] = maxUpdate;
        metrics["result_samples"] = resultSamples; metrics["progress_samples"] = progressSamples;
        check(minGap >= 800 && maxGap <= 1500 && maxUpdate <= 1000, "Heartbeat/update target missed");
        record("real-rust-normal", w);
    }
    void cancellation() {
        Window w(rust); QString previousRun;
        for (int i=0; i<5; ++i) {
            start(w); running(w); check(w.run != previousRun, "Run ID reused"); previousRun=w.run;
            if (i==0) check(until([&] { return w.resultCount>0; }), "No provisional result before Cancel");
            QElapsedTimer t; t.start(); cancel(w); exited(w);
            check(w.state == Window::Cancelled && w.resultCount == 0, "Cancel outcome mismatch");
            check(w.results->text()=="Synthetic results: 0", "Cancelled result display not cleared");
            qint64 ack=-1, terminal=-1;
            for (const auto &v : w.observations) {
                const auto o=v.toObject();
                if (o.value("type")=="CancelAcknowledged") ack=qint64(o.value("received_ms").toDouble())-w.cancelAt;
                if (o.value("type")=="Cancelled") terminal=qint64(o.value("received_ms").toDouble())-w.cancelAt;
            }
            check(ack>=0 && terminal>=ack && ack<=1000, "Cancel acknowledgement/terminal distinction missing");
            metrics["cancel_ack_max_ms"] = qMax(metrics.value("cancel_ack_max_ms").toInt(), int(ack));
            metrics["cancel_exit_max_ms"] = qMax(metrics.value("cancel_exit_max_ms").toInt(), int(t.elapsed()));
        }
        record("real-rust-5-rapid-start-cancel-restarts", w);
    }
    void fault(const QString &mode, bool failure) {
        Window w(python, {fixture, mode}); start(w);
        exited(w, 10000);
        check(w.state == (failure ? Window::Failed : Window::Completed), "Fixture final state mismatch");
        if (failure) check(w.results->text()=="Synthetic results: 0", "Failed result display not cleared");
        if (mode.startsWith("failed")) check(w.error->text() == "FixtureFailure: preserved context", "Failed context overwritten");
        if (mode=="exit7") check(w.exitCode==7 && w.exitKind==QProcess::NormalExit, "Nonzero normal exit misclassified");
        if (mode=="crash") check(w.exitKind==QProcess::CrashExit && w.isVisible(), "Crash not isolated");
        if (!failure) check(w.resultCount==100 && w.progress->value()==100, "Fragmented messages lost");
        record(mode, w);
    }
    void hang(const QString &mode, bool real = false) {
        Window w(real ? rust : python, real ? QStringList{"--fault=unresponsive"} : QStringList{fixture, mode});
        start(w); running(w);
        if (mode != "completed-hang") cancel(w);
        if (mode == "cancel-hang") check(until([&] { return w.state==Window::Cancelled; }), "Missing Cancelled");
        if (mode == "completed-hang") check(until([&] { return w.state==Window::Completed; }), "Missing Completed");
        check(w.process.state()!=QProcess::NotRunning && !w.start->isEnabled(), "Terminal event treated as OS exit");
        check(until([&] { return w.worker->text().startsWith("Warning:"); }, 6500), "5s warning missing");
        const auto warningAt = w.clock.elapsed();
        check(warningAt-w.lastRx>=5000 || warningAt-w.lastHeartbeat>=5000, "Warning before deadline");
        metrics[mode+"_warning_ms"] = warningAt;
        check(!w.force->isEnabled(), "Force stop enabled early");
        check(until([&] { return w.force->isEnabled(); }, 6000), "10s force-stop missing");
        const qint64 stopAt = w.cancelAt>=0 ? w.cancelAt : w.terminalAt;
        const auto forceDelay=w.clock.elapsed()-stopAt;
        check(forceDelay>=10000 && forceDelay<=10300, "Force-stop deadline outside tolerance");
        metrics[mode+"_force_delay_ms"] = forceDelay;
        w.force->click(); exited(w); check(w.isVisible(), "GUI lost on force stop");
        record(mode, w);
    }
    void burst() {
        Window w(python, {fixture, "burst"}); start(w); running(w);
        probes(w, "burst_ui", 150, 1); exited(w,10000);
        check(w.state==Window::Completed && w.resultCount==100 && w.progress->value()==100, "Burst buffer lost valid frames");
        record("20000-heartbeats-plus-200-result-progress-frames", w);
    }
    void closing() {
        Window w(rust); start(w); running(w); w.close();
        check(w.closing, "Active close not deferred"); exited(w);
        check(!w.isVisible(), "GUI not closed after Worker exit"); record("close-active-worker",w);
    }
    void noHeartbeat() {
        Window w(python,{fixture,"no-heartbeat"}); start(w); running(w);
        check(until([&] { return w.worker->text().startsWith("Warning:"); },6500), "Missing heartbeat warning during traffic");
        check(w.clock.elapsed()-w.lastRx<1000 && w.clock.elapsed()-w.lastHeartbeat>=5000, "Heartbeat and traffic silence conflated");
        metrics["missing_heartbeat_warning_ms"]=w.clock.elapsed();
        w.process.kill(); exited(w); record("heartbeat-missing-with-progress-traffic",w);
    }
    void realCrash() {
        Window w(rust,{"--fault=crash"}); start(w); exited(w);
        check(w.state==Window::Failed && w.exitKind==QProcess::CrashExit && w.isVisible(), "Real Rust crash not isolated");
        record("real-rust-crash",w);
    }
    void completionRace() {
        Window w(python,{fixture,"completion-race"}); start(w); running(w); cancel(w); exited(w);
        check(w.state==Window::Completed && w.resultCount==100, "Completed before CancelAcknowledged should win");
        record("completion-before-cancel-ack-preserves-complete-results",w);
        for (const auto &mode : {"completion-after-ack","completion-incomplete"}) {
            Window invalid(python,{fixture,QString::fromLatin1(mode)});
            start(invalid); running(invalid); cancel(invalid); exited(invalid);
            check(invalid.state==Window::Failed && invalid.resultCount==0,"Illegal/incomplete race completion accepted");
            record(QString::fromLatin1(mode),invalid);
        }
    }
    void cpuStress() {
        Window w(rust); start(w); running(w);
        const int threads=qBound(2,QThread::idealThreadCount(),8);
        std::atomic<bool> stop{false};
        std::vector<std::thread> load;
        for(int i=0;i<threads;++i) load.emplace_back([&,i] {
            volatile double value=double(i+1);
            while(!stop.load()) for(int j=0;j<2048;++j) value=std::sin(value+0.001);
        });
        try { probes(w,"cpu_stress_ui",150,10); }
        catch (...) { stop.store(true); for(auto &t:load) t.join(); throw; }
        stop.store(true); for(auto &t:load) t.join();
        metrics["cpu_stress_threads"]=threads;
        cancel(w); exited(w); check(w.state==Window::Cancelled,"CPU-load cancellation failed");
        record("real-worker-with-cpu-saturated-background-load",w);
    }
    void closedCancel() {
        Window w(python,{fixture,"closed-cancel"}); start(w); running(w);
        check(until([&] { return w.diagnostics.contains("CANCEL_PIPE_CLOSED"); }),"Closed-pipe fixture not ready");
        cancel(w); exited(w);
        check(w.state==Window::Completed && w.resultCount==100,"Closed-pipe Cancel destroyed complete outcome");
        // A delayed Qt write-error notification cannot rewrite terminal completion.
        w.process.errorOccurred(QProcess::WriteError);
        check(w.state==Window::Completed && w.resultCount==100,"Delayed write error invalidated completed run");
        record("actual-closed-stdin-cancel-with-completed-outcome",w);
    }
public:
    RuntimeValidation(QString r, QString p, QString f):rust(std::move(r)),python(std::move(p)),fixture(std::move(f)){}
    int run() {
        QJsonObject report{{"qt", qVersion()}, {"platform", QGuiApplication::platformName()},
                           {"os", QSysInfo::prettyProductName()}, {"cpu_arch", QSysInfo::currentCpuArchitecture()}};
        int code=0;
        QElapsedTimer entireClock; entireClock.start();
        qint64 lastTick=0, entireMaxGap=0;
        QTimer entireTick; entireTick.setInterval(10);
        QObject::connect(&entireTick,&QTimer::timeout,[&] {
            const auto now=entireClock.elapsed(); entireMaxGap=qMax(entireMaxGap,now-lastTick); lastTick=now;
        });
        entireTick.start();
        try {
            const auto arch=qEnvironmentVariable("BOARDSI_EXPECTED_QT_ARCH");
            if(!arch.isEmpty()) check(QSysInfo::currentCpuArchitecture()==arch,"Qt executable architecture mismatch");
            const auto backend=qEnvironmentVariable("BOARDSI_EXPECTED_QT_PLATFORM");
            if(!backend.isEmpty()) check(QGuiApplication::platformName()==backend,"Native Qt platform backend mismatch");
            normal(); cancellation(); realCrash();
            for (const auto &m : {"failed1","failed2","exit7","crash","schema","duplicate","reverse","scope","snapshot","frame","json","truncated"}) fault(QString::fromLatin1(m),true);
            fault("partial",false); burst(); completionRace(); closedCancel(); cpuStress();
            hang("unresponsive",true); hang("cancel-hang"); hang("completed-hang"); noHeartbeat(); closing();
            entireMaxGap=qMax(entireMaxGap,entireClock.elapsed()-lastTick);
            check(entireMaxGap<=1000,"Global event-loop gap exceeded 1s");
        } catch (const std::exception &e) { report["failure"]=QString::fromUtf8(e.what()); code=1; }
        entireTick.stop();
        metrics["entire_test_timer_max_gap_ms"]=entireMaxGap;
        metrics["entire_test_elapsed_ms"]=entireClock.elapsed();
        report["pass"]=code==0; report["assertions"]=assertions; report["cases"]=cases; report["metrics"]=metrics;
        const auto bytes=QJsonDocument(report).toJson(QJsonDocument::Indented);
        QFile out("qt-runtime-results.json");
        if (!out.open(QIODevice::WriteOnly) || out.write(bytes)!=bytes.size()) return 2;
        qInfo().noquote() << bytes;
        return code;
    }
};

int main(int argc,char **argv) {
    QApplication app(argc,argv);
    app.setQuitOnLastWindowClosed(false);
    if(argc!=4) return 2;
    RuntimeValidation test(QString::fromLocal8Bit(argv[1]),QString::fromLocal8Bit(argv[2]),QString::fromLocal8Bit(argv[3]));
    QTimer::singleShot(0,&app,[&] { app.exit(test.run()); });
    return app.exec();
}
