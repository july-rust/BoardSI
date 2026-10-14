# ADR: executed Linux Qt Widgets / Rust Worker prototype

Date: 2026-10-14 KST. Related #3, #5, #6; baseline main
`78924b79ad7d7ecd1ea85d0ea85b9f444cc35141` / merged PR #29.

## Verified decision

The existing Widgets/QProcess design **compiled and executed** on Linux x86_64
with Qt 6.8.3, CMake 3.31.6, GCC 13.3.0, C++17 and unchanged Rust 1.75.0.
Only Core/Gui/Widgets are linked. The test harness shares the actual Window
implementation; its outer main calls QApplication::exec(). Its finite nested
QEventLoop waits dispatch IPC, timers and UI without synchronous process waits.
The production Window has no nested loops, waitForFinished, blocking reads,
synchronous Solver or unbounded message dispatch loop. Python is a test-only
separate adversarial peer. Normal computation/cancel/crash/silence use the actual
Rust executable; fixture success is never treated as a numerical validation.

This is an accepted **M0 Linux offscreen prototype** decision, not a final
product UI, distribution, packaging, Qt Quick/Widgets or cross-platform approval.
The existing CMake Qt 6.5 minimum is preserved; the executed/CI version is 6.8.3.
The v1.0 specification, Rust source, protocol, dependencies and toolchain are unchanged.

## Runtime fixes, without changing IPC v1

- Read at most 65,540 bytes and 8 frames per callback. Limit the application
  buffer to 131,080 bytes by reading only available capacity; defer remaining
  work. Previously a valid burst accumulated faster than eight-frame batches
  and could incorrectly hit the buffer limit. QProcess/OS pipe buffering is
  separate; this is not a claim of a fixed total process memory bound.
- Tag deferred parse callbacks with the run generation so old scheduled work
  cannot consume a restarted run's stream. Scope, snapshot and contiguous
  sequence checks still fail closed on invalid frames.
- On QProcess Crash, do not enable Start until finished and output drain.
  FailedToStart alone has no finished signal and can enable retry immediately.
- A result count QLabel updates alongside the existing progress/state controls,
  and clears on cancellation or failure.
  Host receipt/update timestamps are bounded to 4,096 observations; they measure
  widget property changes, not display presentation or GPU completion.

CancelRequested, CancelAcknowledged, Cancelled, OS exit remain separate. Completed
or Cancelled does not suppress the process-lifetime watchdog. Scoped Failed code
and context survive normal exit 1/2. CrashExit and nonzero NormalExit are reported
separately. Existing strict policy rejects Completed after CancelRequested and
discards provisional results. This race remains an explicitly tested limitation;
changing that policy requires a coordinated Rust/Qt protocol-state decision.

## Reproducible installation

CI: Ubuntu 24.04 x86_64, `aqtinstall==3.3.0`, `cmake==3.31.6`,
`aqt install-qt linux desktop 6.8.3 linux_gcc_64 --archives qtbase icu`.
Qt Test is unnecessary for this harness. apt provides OpenGL/XKB/Vulkan headers;
the tests explicitly run `QT_QPA_PLATFORM=offscreen`. Existing required docs
and Rust native checks remain unchanged. The new Qt check is provisional and
has **not** been made required. No SDK, archive, executable or CI artifact upload
is committed or distributed.

The local managed environment rejected apt privilege switching/system cache
writes and aqt's multiprocessing Unix socket. Its safe alternative was to
download and extract the same official Qt archives with Python py7zr and extract
Ubuntu OpenGL packages into a writable directory. Both Qt archives were checked
against their official SHA-256 sidecar:

| Qt archive | SHA-256 |
| --- | --- |
| qtbase 6.8.3-0-202503201345 Linux RHEL 8.10 GCC x86_64 | d8fe882cae71284a7ca906b70b42840376882fe96f22fce1ba5a7a70ceeec799 |
| icu 6.8.3-0-202503201345 Linux RHEL 8.6 x86_64 | a2543496e1280c6fa348e0791def5026d5bde9e5749d9fc2244866ab0bee8aed |

Official source: [Qt repository](https://download.qt.io/online/qtsdkrepository/linux_x64/desktop/qt6_683/qt6_683/qt.qt6.683.linux_gcc_64/).
Only temporary local SDK paths/LD_LIBRARY_PATH differ from standard CI installation.
Windows x64, Windows ARM64 and macOS ARM64 Qt execution remain **Not Run**.

Evidence: [runtime report](../validation/m0-qt-native-runtime.md),
[raw measurements](../validation/qt-runtime-linux-offscreen.json),
[QProcess API](https://doc.qt.io/qt-6.8/qprocess.html),
[QApplication event loop](https://doc.qt.io/qt-6.8/qapplication.html#exec),
[runtime source](../../apps/boardsi-qt-prototype/tests/runtime.cpp),
[CI](../../.github/workflows/qt-runtime.yml).
