// M0 F8 diagnostic-only cold native QWidget control.
// This is a separate process run AFTER the unchanged BoardSI runtime gate.
// Do not treat this control's timings as evidence that BoardSI passed F8.
#include <QApplication>
#include <QElapsedTimer>
#include <QFile>
#include <QGuiApplication>
#include <QJsonDocument>
#include <QJsonObject>
#include <QSysInfo>
#include <QTimer>
#include <QWidget>
#include <optional>
#if defined(Q_OS_WIN)
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#elif defined(Q_OS_MACOS)
#include <mach/mach.h>
#include <mach/thread_info.h>
#endif

static std::optional<quint64> guiThreadCpuUs() {
#if defined(Q_OS_WIN)
    FILETIME created{}, exited{}, kernel{}, user{};
    if (!GetThreadTimes(GetCurrentThread(), &created, &exited, &kernel, &user))
        return std::nullopt;
    const auto toUs = [](const FILETIME &v) -> quint64 {
        return ((quint64(v.dwHighDateTime) << 32) | v.dwLowDateTime) / 10;
    };
    return toUs(kernel) + toUs(user);
#elif defined(Q_OS_MACOS)
    thread_basic_info_data_t info{};
    mach_msg_type_number_t count = THREAD_BASIC_INFO_COUNT;
    const mach_port_t thread = mach_thread_self();
    const kern_return_t rc = thread_info(
        thread, THREAD_BASIC_INFO, reinterpret_cast<thread_info_t>(&info), &count);
    mach_port_deallocate(mach_task_self(), thread);
    if (rc != KERN_SUCCESS) return std::nullopt;
    return quint64(info.user_time.seconds + info.system_time.seconds) * 1000000
         + quint64(info.user_time.microseconds + info.system_time.microseconds);
#else
    return std::nullopt;
#endif
}

int main(int argc, char **argv) {
    QElapsedTimer lifecycle;
    lifecycle.start();
    QApplication app(argc, argv);
    app.setQuitOnLastWindowClosed(false);

    QJsonObject metrics{
        {"scope", "M0 F8 independent native QWidget control, diagnostic only"},
        {"qualification", "NOT_F8_ACCEPTANCE"},
        {"order", "Run in separate process AFTER unchanged BoardSI runtime validation"},
        {"platform", QGuiApplication::platformName()},
        {"qt", qVersion()},
        {"os", QSysInfo::prettyProductName()},
        {"cpu_arch", QSysInfo::currentCpuArchitecture()},
        {"app_constructor_wall_ms", lifecycle.elapsed()}
    };

    // No BoardSI production Window, Worker, IPC or alternate F8 pass path.
    // The control intentionally includes the very FIRST native QWidget show.
    QElapsedTimer clock;
    clock.start();
    QTimer periodic;
    periodic.setTimerType(Qt::PreciseTimer);
    periodic.setInterval(10);
    qint64 firstPeriodic = -1;
    int periodicCallbacks = 0;
    QObject::connect(&periodic, &QTimer::timeout, &app, [&] {
        if (firstPeriodic < 0) firstPeriodic = clock.elapsed();
        ++periodicCallbacks;
        if (periodicCallbacks >= 5) app.quit();
    });
    periodic.start();
    metrics["global_timer_armed_ms"] = clock.elapsed();

    QWidget control;
    control.setWindowTitle("BoardSI M0 F8 isolated native QWidget diagnostic");
    control.resize(440, 300);
    metrics["first_show_begin_ms"] = clock.elapsed();
    const auto cpuBegin = guiThreadCpuUs();
    control.show();
    const auto cpuEnd = guiThreadCpuUs();
    metrics["first_show_end_ms"] = clock.elapsed();
    const bool cpuValid = cpuBegin && cpuEnd && *cpuEnd >= *cpuBegin;
    metrics["first_show_gui_thread_cpu_valid"] = cpuValid;
    if (cpuValid)
        metrics["first_show_gui_thread_cpu_us"] = qint64(*cpuEnd - *cpuBegin);

    qint64 zeroCallback = -1;
    QTimer::singleShot(0, &app, [&] { zeroCallback = clock.elapsed(); });
    bool deadlineExpired = false;
    QTimer::singleShot(3000, &app, [&] { deadlineExpired = true; app.exit(2); });
    metrics["app_exec_entry_ms"] = clock.elapsed();
    const int rc = app.exec();
    metrics["first_zero_callback_ms"] = zeroCallback;
    metrics["first_periodic_callback_ms"] = firstPeriodic;
    metrics["periodic_callbacks"] = periodicCallbacks;
    metrics["deadline_expired"] = deadlineExpired;
    metrics["app_exec_return_code"] = rc;
    metrics["diagnostic_success"] = (rc == 0 && !deadlineExpired && periodicCallbacks >= 5);
    metrics["first_periodic_over_1000ms_observed"] = firstPeriodic > 1000;
    metrics["not_proven"] = "Does not isolate runner scheduling or Qt root cause; not a passing BoardSI runtime test";

    const QByteArray output = QJsonDocument(metrics).toJson(QJsonDocument::Indented);
    QFile evidence("f8-cold-show-control.json");
    if (!evidence.open(QIODevice::WriteOnly) || evidence.write(output) != output.size())
        return 3;
    return rc == 0 && !deadlineExpired && periodicCallbacks >= 5 ? 0 : 2;
}
