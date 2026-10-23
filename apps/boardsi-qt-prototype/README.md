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


## Developer Preview P1 — synthetic artwork UI (Issue #11 subset)

The existing **boardsi-preview-p0** executable name and P0 Qt/Rust Worker
interfaces are retained for compatibility while the Preview P1 UI is developed.

The Artwork Viewer is now a Qt `QGraphicsScene` / `QGraphicsView` with a
strictly **hard-coded, offline, illustrative two-layer PCB fixture**. It is
not the Rust Canonical Board Model, imported copper, or a solver input.
The synthetic board dimensions and positions in the scene are illustrative
millimetres, not measurements of a manufactured PCB.

- Left mouse drag pans the canvas; mouse wheel and +/- zoom; Fit restores full view.
- Top / Bottom layer checkboxes hide and show matching mock copper.
- The Net selector highlights only fictional DATA_P0, CLK_N0 and CTRL0
  primitives. Clicking a copper primitive selects its fictional net.
- The Properties panel reports selected mock net, layer visibility and
  primitive counts, always marked **SYNTHETIC / NOT IMPORTED**.
- **Open PCB** remains local filename selection only, without reading a
  file, detecting its signature or replacing the synthetic fixture.
- Physical SI (impedance/RLGC/S-parameters), real CAD artwork, Save Project,
  PADS ASC parser and generated Touchstone remain **NOT IMPLEMENTED / NOT_RUN**.

The existing `boardsi-preview-p0 --selftest` native workflow now asserts
the scene contains mock copper, toggles both layers, confirms zero visible
copper when both are hidden, selects/highlights a net, checks zoom and Fit
controls, restores full mock view, and then runs the **unchanged** Worker
progress/cancel/restart/completion/error and Canonical A/B receipt exercises.
The `qt-preview-p0-results.json` file adds `p1_synthetic_artwork`, without
replacing the old output schema. Native Windows x64/ARM64 and macOS ARM64
runs provide the release-independent technical UI acceptance evidence.

P1 is a narrow runnable UI slice of the existing [M1 Artwork issue](https://github.com/july-rust/BoardSI/issues/11).
It does NOT close that issue's full real-board U08/T03/T34 acceptance contract.
The independent M0 F1/F6/F7 legal/hardware gates and the Qt binary
redistribution restriction remain unchanged; no Qt executables are published.


## GPU-first Artwork rendering policy (A_013 OpenGL follow-up; PR #164)

**Default: GPU preferred, not GPU guaranteed.** The existing Qt Widgets
QGraphicsScene/QGraphicsView retained all Canonical artwork selection, layers,
pan/zoom and Properties behavior. The common `SyntheticArtworkCanvas` now
checks real OpenGL context/surface capability on first visible display and,
when the renderer matches a known hardware adapter and is not a software driver, switches to a QOpenGLWidget
viewport. Qt frameSwapped is required before reporting `GPU_OPENGL_ACTIVE`.
This is an initial acceleration path, **not** evidence of batching, FPS, or
whole-scene GPU rasterization: individual QGraphicsItem/QPainter primitives may
remain CPU constrained.

- `BOARDSI_ARTWORK_RENDERER=auto` or unset: GPU preferred (default);
  hardware unavailable, invalid context or headless => explicit CPU fallback.
- `BOARDSI_ARTWORK_RENDERER=gpu`: same GPU-first *request*; still fails safely
  to CPU when no usable hardware can be verified, without falsely claiming GPU.
- `BOARDSI_ARTWORK_RENDERER=cpu`: deterministic software viewport for
  troubleshooting and CI. Unsupported setting fails to CPU.
- Qt OpenGLWidgets is an optional CMake component for build compatibility:
  module missing => explicit CPU fallback, never silently claim GPU.
- Properties include `Artwork renderer` actual state. The
  `--renderer-selftest` receipt `qt-preview-renderer-results.json` includes
  configured policy, backend, adapter, first confirmed GL frame and source SHA.
  Offscreen CI asserts CPU fallback; native desktop CI records real backend.
- `GPU_OPENGL_PENDING_FRAME` does not count as GPU acceleration. A 3-second
  no-frame timeout reverts to CPU. Hidden/imported tabs are probed only when
  first displayed rather than timing out while inactive.
- `QGraphicsView` uses FullViewportUpdate with QOpenGLWidget per Qt guidance.

**Future high-volume PCB renderer:** do not use a QOpenGLWidget as a permanent
cross-platform Metal/Direct3D engine. Preserve the Canonical source and Qt
Widgets UI while isolating a renderer interface, then profile 10k/100k/1M
synthetic geometry for first render, pan/zoom, memory, selection, culling and
frame times on Windows x64/ARM64 and macOS ARM64. Compare Qt Quick Scene Graph
/ Qt RHI (Metal on macOS, Direct3D on Windows) to current CPU/GL paths, with
LOD/tiling/batching/spatial indexes. A tentative target of 60 FPS is a goal,
**not** an acceptance measurement. Qt Quick 3D exists but BoardSI does not
implement 3D PCB visualization or 3D EM analysis in this change.

Documentation:
- Qt QGraphicsView OpenGL viewport: https://doc.qt.io/qt-6/qgraphicsview.html
- Qt Graphics/RHI and Quick 3D: https://doc.qt.io/qt-6/topics-graphics.html
- Qt Quick embedding Widgets caveats:
  https://doc.qt.io/qt-6/qtquick-embeddedinwidgets-example.html
- Reuses BoardSI Qt 6.8.3 deployment LGPLv3 prototype policy; all distribution
  obligations and dependencies require separate verification.
