# Qt Widgets M0 Worker prototype

**Build and actual execution: Not Run in the implementation environment.**
No Qt SDK, qmake, pkg-config Qt6Widgets or CMake was available. No installer/large SDK
was added. This is reviewable C++17 source, not evidence of GUI responsiveness.
Only Qt Core/Gui/Widgets are requested; no QML, Charts, network or Rust FFI.

In a prepared Qt 6.5+ development environment (Qt 6.8 is an ADR candidate):

```sh
cargo build --locked -p boardsi-worker
cmake -S apps/boardsi-qt-prototype -B build-qt -DCMAKE_PREFIX_PATH=/absolute/qt/prefix
cmake --build build-qt
# Use the actual built executable path; Windows multi-config builds may use Debug/.
./build-qt/boardsi-qt-prototype /absolute/path/to/target/debug/boardsi-worker
```

The Worker path must be absolute; pass `.exe` on Windows. Start requests 100 synthetic
uniform lossless RLGC points with a 50 ms per-point test delay. The GUI displays only
execution status/counts; these are not actual PCB results. Each Start creates a UUID,
resets receive buffers/cursors and uses the immutable synthetic snapshot identity.
Stdout is parsed as framed JSON; stderr is separate. QProcess signals, bounded parse
batches and QTimer handle work; no blocking wait or solver operation runs on the UI.
Close while active requests kill and waits **asynchronously** for process finished.
Cancel request, acknowledgement, cancellation event and process exit are distinct.
Scoped Worker Failed(code,context) is retained across expected normal exit 1/2;
lifecycle warning/force-stop monitoring continues after a terminal event until OS exit.
The 5 s timer warning and 10 s Force stop enablement are source logic only until measured.

Next prepared-host validation must compile with warnings as errors and run a real
QApplication event loop, record OS/CPU/RAM/Qt/compiler/display backend, and measure
at least 100 queued button/interaction responses during work. Record p95/max UI delay,
request-to-ack, heartbeat gap, result-to-display latency, actual warning/force enablement,
crash/close/restart and partial/malformed/burst-message cases. Desktop Pan/Zoom/tab
coverage and CPU-saturated workload are separate from this button-only prototype.
Qt Widgets versus Quick and public packaging remain Pending. Do not mark T34/T50 PASS
from Rust tests or source inspection.
