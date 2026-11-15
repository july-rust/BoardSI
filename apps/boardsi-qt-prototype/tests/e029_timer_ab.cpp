// E_029 diagnostic-only cold-start A/B (not an M0 acceptance test).
// Control: no QWidget; experimental: a native QWidget shown before first timer dispatch.
// Each invocation is a fresh native process; artifacts preserve every observation.
#include <QAbstractEventDispatcher>
#include <QApplication>
#include <QElapsedTimer>
#include <QFile>
#include <QGuiApplication>
#include <QJsonDocument>
#include <QJsonObject>
#include <QSysInfo>
#include <QThread>
#include <QTimer>
#include <QWidget>
#include <QDebug>
#include <memory>

int main(int argc, char **argv) {
    if (argc != 4 || QString::fromLocal8Bit(argv[1]) != "--mode")
        return 2;
    const QString mode = QString::fromLocal8Bit(argv[2]);
    const QString output = QString::fromLocal8Bit(argv[3]);
    if (mode != "control" && mode != "window")
        return 2;

    QElapsedTimer monotonic;
    monotonic.start();
    QApplication app(argc, argv);
    app.setQuitOnLastWindowClosed(false);
    const qint64 appReady = monotonic.elapsed();
    const QString expected = qEnvironmentVariable("BOARDSI_EXPECTED_QT_PLATFORM");
    const QString actual = QGuiApplication::platformName();

    qint64 startCallback = -1, timerArmed = -1, showBegin = -1, showEnd = -1;
    qint64 firstZero = -1, firstPrecise = -1, maxGap = 0, lastPrecise = -1;
    qint64 waitBegin = -1, maxWait = 0, dispatcherPairs = 0, timerCallbacks = 0;
    QTimer precise;
    precise.setTimerType(Qt::PreciseTimer);
    precise.setInterval(10);
    std::unique_ptr<QWidget> window;

    QObject::connect(&precise, &QTimer::timeout, &app, [&] {
        const qint64 now = monotonic.elapsed();
        if (firstPrecise < 0) firstPrecise = now;
        if (lastPrecise >= 0 && now - lastPrecise > maxGap)
            maxGap = now - lastPrecise;
        lastPrecise = now;
        ++timerCallbacks;
    });
    if (auto *dispatcher = QAbstractEventDispatcher::instance(app.thread())) {
        QObject::connect(dispatcher, &QAbstractEventDispatcher::aboutToBlock, &app, [&] {
            waitBegin = monotonic.elapsed();
        });
        QObject::connect(dispatcher, &QAbstractEventDispatcher::awake, &app, [&] {
            if (waitBegin < 0) return;
            const qint64 delay = monotonic.elapsed() - waitBegin;
            if (delay > maxWait) maxWait = delay;
            ++dispatcherPairs;
            waitBegin = -1;
        });
    }
    QTimer::singleShot(0, &app, [&] {
        startCallback = monotonic.elapsed();
        timerArmed = monotonic.elapsed();
        precise.start();
        if (mode == "window") {
            showBegin = monotonic.elapsed();
            window = std::make_unique<QWidget>();
            window->resize(160, 100);
            window->show();
            showEnd = monotonic.elapsed();
        }
        QTimer::singleShot(0, &app, [&] { firstZero = monotonic.elapsed(); });
        QTimer::singleShot(1700, &app, [&] { app.quit(); });
    });
    const int appRc = app.exec();
    precise.stop();

    const bool nativePlatform = !expected.isEmpty() && actual == expected;
    const bool valid = appRc == 0 && nativePlatform && timerArmed >= 0 &&
                       firstPrecise >= timerArmed && firstZero >= timerArmed &&
                       timerCallbacks > 0 && (mode != "window" ||
                       (showBegin >= timerArmed && showEnd >= showBegin && showEnd <= firstPrecise));
    QJsonObject result{
        {"schema", "boardsi-e029-timer-ab-v1"},
        {"diagnostic_only", true},
        {"mode", mode},
        {"source_sha", qEnvironmentVariable("BOARDSI_E029_HEAD_SHA")},
        {"os", QSysInfo::prettyProductName()},
        {"architecture", QSysInfo::currentCpuArchitecture()},
        {"qt", qVersion()},
        {"platform", actual},
        {"expected_platform", expected},
        {"native_backend_verified", nativePlatform},
        {"app_ready_ms", appReady},
        {"start_callback_ms", startCallback},
        {"timer_armed_ms", timerArmed},
        {"show_begin_ms", showBegin},
        {"show_end_ms", showEnd},
        {"first_zero_ms", firstZero},
        {"first_precise_ms", firstPrecise},
        {"first_precise_gap_ms", firstPrecise >= timerArmed ? firstPrecise - timerArmed : -1},
        {"zero_to_precise_ms", firstPrecise >= 0 && firstZero >= 0 ? firstPrecise - firstZero : -1},
        {"max_subsequent_precise_gap_ms", maxGap},
        {"dispatcher_max_wait_ms", maxWait},
        {"dispatcher_wake_pairs", dispatcherPairs},
        {"timer_callback_count", timerCallbacks},
        {"total_elapsed_ms", monotonic.elapsed()},
        {"cold_host_independently_verified", false},
        {"pass", valid},
        {"global_1500ms_acceptance", "NOT_EVALUATED"}
    };
    QFile file(output);
    if (!file.open(QIODevice::WriteOnly | QIODevice::Truncate))
        return 2;
    const QByteArray json = QJsonDocument(result).toJson(QJsonDocument::Indented);
    if (file.write(json) != json.size())
        return 2;
    qInfo().noquote() << json;
    return valid ? 0 : 1;
}
