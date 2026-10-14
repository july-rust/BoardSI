# M0 Qt runtime follow-up: native matrix, race fix and CPU load

Date: 2026-10-14 KST. Continues PR #30, initial head
`417f17f264dbc060d80c0e274c91596c42bb6116`. No merge or release.
Baseline specification v1.0, numerical core/model/Worker calculation unchanged.

## Resolved implementation issues

1. Initial Qt CI failed at link: missing libEGL.so.1. Added libegl1 explicitly,
   with Xvfb/xcb desktop prerequisites; retained all actual build/runtime checks.
2. Rust/Qt now accept fully received Completed before CancelAcknowledged after a
   local Cancel request. After acknowledgement, Completed remains invalid.
   Incomplete/foreign/duplicate/malformed output is still rejected. A late Cancel
   on a closed pipe drains the final outcome rather than discarding valid results.
3. Added 150 measured UI events under 8 CPU-load threads locally; shared harness
   runs on native Qt windows/cocoa and Linux xcb software desktop in CI. Target
   Qt and Rust executable identities must match the runner architecture.

[Before/after compatibility and decisions](../architecture/qt-runtime-followup.md).

## Actual local measurements

Same Linux x86_64 container / Qt 6.8.3 / GCC 13.3.0 C++17 / CMake 3.31.6 /
Rust 1.75.0 as the initial report. Actual QApplication::exec plus independent
Rust Worker: **27 scenarios / 255 assertions PASS**, **49,332 ms**. Native-job
architecture/backend assertions add checks when their expected environment is set.
Raw local [JSON](qt-runtime-linux-followup.json) is independent of CI results.

| Measurement | Final local value | Samples |
| --- | --- | --- |
| Normal UI p95 / maximum | 0.101 / 0.489 ms | 150 |
| Burst UI p95 / maximum | 4.479 / 6.749 ms | 150 |
| CPU-stress UI p95 / maximum | 0.026 / 0.228 ms | 150, 8 busy threads |
| Whole-test event-loop timer maximum gap | 37 ms | 49,332 ms test duration |
| Cancel acknowledgement / drained exit maximum | 4 / 10 ms | 5 real Rust runs |
| Heartbeat gap minimum / maximum | 1,001 / 1,005 ms | 6 samples |
| Receipt-to-widget result/progress update maximum | 2 ms | 100 + 100 |
| Force stop after Cancel / terminal | 10,044–10,089 ms | 3 hanging-process cases |
| Widgets raster frame | 440×300 pixels, non-null | SHA-256 in raw JSON |

All 450 UI events pass p95 ≤200 ms/max ≤1 s; the whole-test timer has no >1 s gap.
These are actual event/property-update measurements; frame grabbing is software
raster evidence, not screen presentation latency or a hardware GPU benchmark.

Rust Debug **69 PASS**, Release **69 PASS**, clippy -D warnings and fmt PASS.
Original 67 tests preserved; new IPC race and real closed-pipe client regressions
add two tests. Numerical 15/model 28 unchanged. Qt build warnings/errors 0 locally;
expected offscreen size-hint diagnostics remain disclosed as environmental output.

## CI execution status

Initial-head existing Rust four-platform CI and required docs succeeded.
[Initial Qt CI](https://github.com/july-rust/BoardSI/actions/runs/38560937655)
failed on missing EGL; it did not execute the GUI. The new source must be judged
by its own runs; installing a workflow is not native-runtime PASS.

| Environment | Current follow-up status |
| --- | --- |
| Linux local offscreen | PASS, measured above |
| Linux CI offscreen + xcb/Xvfb | Pending actual new-head CI |
| Windows x64 windows backend | Pending actual new-head CI |
| Windows ARM64 windows backend | Pending actual new-head CI |
| macOS ARM64 cocoa backend | Pending actual new-head CI |

Qt checks remain provisional, not mandatory branch protection checks. Existing
Rust native CI and docs checks remain active. Issues #3/#5/#6 remain open because
overall M0 and product acceptance exceed this prototype.

## Scope still requiring future product/hardware work

Physical monitor/GPU presentation, human input latency and a full Artwork
Viewer/geometry-based Solver cannot be validated from this minimal prototype.
Those components are not implemented and adding them is outside M0 fifth-stage
scope. CPU-load simulation and native Qt backends do not fabricate that evidence.
Next product stage needs the implemented Viewer/Solver plus prepared physical
Windows/macOS/Linux hosts; do not label complete T34/T39/T50 or M0 from these tests.
