# Qt Widgets M0 Worker prototype

Actual Linux x86_64 build and QApplication/Rust Worker execution: **PASS, Qt 6.8.3 offscreen**.
Windows x64/ARM64 and macOS ARM64 Qt runtime: **Not Run**.
This is a synthetic RLGC prototype, not PCB SI or full M0 acceptance.

## Build and test

Qt 6.8.3 gcc_64 SDK with qtbase and ICU, CMake 3.31.6 and C++17 are the measured
configuration. Existing Qt 6.5 minimum remains unchanged. Core/Gui/Widgets only;
no QML/Charts/FFI or Qt Test dependency. See the pinned Linux workflow and ADR.

```sh
cargo build --locked -p boardsi-worker
cmake -S apps/boardsi-qt-prototype -B build-qt \
  -DCMAKE_PREFIX_PATH=/absolute/qt/prefix \
  -DBOARDSI_WORKER=/absolute/path/to/target/debug/boardsi-worker
cmake --build build-qt --parallel 2
QT_QPA_PLATFORM=offscreen ctest --test-dir build-qt --output-on-failure
# Run the actual GUI on a prepared desktop:
./build-qt/boardsi-qt-prototype /absolute/path/to/target/debug/boardsi-worker
```

If needed, include the SDK ICU/Qt library directory in LD_LIBRARY_PATH on Linux.
BUILD_TESTING=ON requires Python3 for the test-only fault peer and an absolute
BOARDSI_WORKER path. Use `-DBUILD_TESTING=OFF` to compile only the GUI.
Windows multi-config executable locations and `.exe` need matching paths.

The actual Window starts QProcess from the Start button; UUID/immutable snapshot,
framed JSON, sequence/scope checking, bounded parse callbacks and QTimer separate
UI and computation. Cancel request/acknowledgement/terminal/OS exit are distinct.
Close during work kills asynchronously and waits for drained process exit.
Failed code/context survives expected exit 1/2. Watchdog remains active after
Completed/Cancelled until exit; Force stop is offered after the 10 s deadline.
Completed before CancelAcknowledged wins only after every result is received.
After acknowledgement, Cancelled is required; incomplete completion remains invalid.
A late Cancel hitting a closed pipe drains the Worker outcome before classification.

CTest calls a real QApplication::exec(), drives this same Window, launches the real
Rust Worker and separate test peers, checks 28 scenarios and posts 450 measured
UI events. Results are written to `build-qt/qt-runtime-results.json`; failures
return nonzero. Receipt/update measurements cover widget properties, not GPU paint.
Offscreen diagnostics are recorded; desktop rendering is not validated.

[Current follow-up report](../../docs/validation/m0-qt-runtime-followup.md) ·
[Current raw measurements](../../docs/validation/qt-runtime-linux-followup.json) ·
[ADR and race update](../../docs/architecture/qt-runtime-followup.md) ·
[CI](../../.github/workflows/qt-runtime.yml).
