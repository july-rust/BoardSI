# Qt Widgets M0 Worker prototype

Actual Linux x86_64 build and QApplication/Rust Worker execution: **PASS, Qt 6.8.3 offscreen**.
Windows x64/ARM64 windows backend and macOS ARM64 cocoa backend: **PASS**.
Linux xcb/Xvfb software desktop: **PASS**. Verified code commit:
`838203bdd286296c3b93ce06091c21af3206045d`; see the current report for scope.
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
# Required for qt-model-v2-identity: generate immutable synthetic fixtures.
# Run from the repository root; do not use real or proprietary PCB data.
cargo run --locked -p boardsi-worker --example m0_model_snapshot_fixture -- \
  build-qt/qt-model-a.frame build-qt/qt-model-b.frame
QT_QPA_PLATFORM=offscreen ctest --test-dir build-qt --output-on-failure
# Run the actual GUI on a prepared desktop:
./build-qt/boardsi-qt-prototype /absolute/path/to/target/debug/boardsi-worker
```

If needed, include the SDK ICU/Qt library directory in LD_LIBRARY_PATH on Linux.
BUILD_TESTING=ON requires Python3 for the test-only fault peer and an absolute
BOARDSI_WORKER path. Use `-DBUILD_TESTING=OFF` to compile only the GUI.
Windows multi-config executable locations and `.exe` need matching paths.
The `qt-model-v2-identity` CTest **requires** the two generated synthetic
`build-qt/qt-model-a.frame` and `build-qt/qt-model-b.frame` files shown above.
They are not shipped in Git, must be regenerated after changing model fixture
sources, and contain no real board geometry or measured SI results. Running
`ctest` without generating these files first is an incomplete test setup.
On Windows PowerShell, set `$env:QT_QPA_PLATFORM="offscreen"` before CTest
instead of using the POSIX environment-variable prefix.

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


## Developer Preview P0 (native desktop, NOT PCB SI)

Build the existing Rust Worker and two offline synthetic Canonical Model fixtures
using the commands above, then compile the new `boardsi-preview-p0` Qt target
from the same CMake build. The preview executable is a native Qt Widgets window.

Run it on a developer machine with a matching installed Qt runtime:

```sh
# Use absolute paths on each operating system; add .exe on Windows.
./build-qt/boardsi-preview-p0 /absolute/boardsi-worker \
  /absolute/qt-model-a.frame /absolute/qt-model-b.frame
```

Windows example from PowerShell after generating the two fixtures:

```powershell
.\build-qt\boardsi-preview-p0.exe "$PWD\target\debug\boardsi-worker.exe" "$PWD\build-qt\qt-model-a.frame" "$PWD\build-qt\qt-model-b.frame"
```

A no-argument launch is permitted to inspect the shell, but Worker and fixture
files are then sought beside the Qt executable; missing files are reported and
no synthetic Canonical A/B test is available. This is NOT a self-contained
installer, nor an approved distribution package.

- **Run** starts the existing M0-tested independent Worker synthetic progress
  demonstration; **Stop** requests Cancel. The Progress tab retains Force stop.
- **Canonical Model A/B** is a separate, existing immutable model-v2 receipt
  demonstration, with no progress protocol and no physical SI output.
- **Open PCB** only chooses a local filename. It does NOT inspect the file,
  recognize PADS syntax, import geometry, or upload files. **Save Project**,
  Artwork, S-Parameters and Impedance are explicitly unsupported.
- Every result is marked **SYNTHETIC / NOT PCB SI**. No physical PCB results
  have been produced.

The native CI calls `boardsi-preview-p0 --selftest WORKER FRAME_A FRAME_B`.
It opens the real Qt window (native Windows/macOS backend, offscreen on Linux),
checks widget rendering, clicks Run/Stop and restart, verifies synthetic
progress/completion, a missing-Worker error, and runs Canonical A/B receipt.
It writes `qt-preview-p0-results.json` with the source SHA, architecture,
Qt version, platform, result and failure phase. P0 also runs under Linux CTest.
Native CI uploads only JSON evidence, **not any executable or Qt runtime**.

**License boundary:** this implementation is a developer source/build
prototype. Qt LGPLv3 dynamic-link compliance, all Qt/ICU/other dependencies,
clean target-machine deployment, and redistribution rights are NOT verified.
Binary redistribution remains BLOCKED pending F7 legal/package review;
a CI compilation or selftest PASS does not authorize public release.
