# ADR update: cancel/completion race and native Qt runtime matrix

Date: 2026-10-14 KST. PR #30 remains unmerged; Issues #3/#5/#6 remain partial.
This supersedes the strict-race/current-CI statements of the initial
[Qt runtime ADR](qt-native-runtime.md). Specification v1.0 unchanged.

## Race policy, Rust and Qt in agreement

CancelRequested is a local intent, not proof the Worker received or accepted it.
The existing Worker may legitimately write Completed before reading a queued Cancel.

| Event ordering | Before this update | Updated clients |
| --- | --- | --- |
| CancelRequested, all N results, Completed; no CancelAcknowledged | Rejected, discarded valid completed results | Completed wins; retain N results, await successful OS exit |
| CancelRequested, CancelAcknowledged, Completed | Rejected | Still rejected, even with N results; Cancelled is required |
| CancelRequested, fewer than N results, Completed | Rejected | Still rejected |
| Cancel write gets BrokenPipe / Qt WriteError before outcome drain | Immediately failed run | Keep CancelRequested, expose delivery diagnostic, drain scoped final outcome |
| Completed before ack, but OS exit hangs after Cancel | Rust independent guard hid force option | Preserve 10 s force option until OS exit |

Only a matching scope, snapshot, contiguous sequence, finite valid payloads,
complete result count and successful OS exit can produce a completed result.
Abnormal OS exit still invalidates results. Acknowledgement is the irreversible
boundary between "requested" and "accepted" cancellation. An acknowledgement
after terminal Completed remains an invalid extra frame; no terminal is rewritten.
Closed-pipe handling does not invent acknowledgement or completion and cannot
turn truncated/missing/failed outcomes into success.

Wire compatibility: framing, schema version 1, message fields, Snapshot identity,
Worker supervision and numerical algorithms are unchanged. Client acceptance is
widened for an existing legal Worker emission ordering. An older strict client
can still reject this ordering; no new message or silent wire-schema change is
introduced. Updated Qt and independent Rust clients share the same state rule.
The independent client's additive cancel_delivery_error field exposes a failed
delivery; cancel() remains a best-effort request, not an acknowledgement API.

Evidence includes state tests for incomplete/before-ack/after-ack completion and
a real closed OS stdin pipe followed by intentionally delayed complete frames.
The actual Qt fixture waits for Cancel, then exercises all three result orderings.

## Native environments and honest display scope

Qt 6.8.3 Windows x64/MSVC 2022 and Windows ARM64 binary SDKs were enumerated via
aqt 3.3.0. ARM package is win64_msvc2022_arm64_cross_compiled, installed under
msvc2022_arm64; it requires the desktop x64 SDK for host tools. The workflow
installs both and sets QT_HOST_PATH. This name describes how the vendor built
the SDK, not the validation application: Qt QSysInfo and Rust compile-time
architecture checks must confirm actual arm64/aarch64 executable execution.
Python/SDK host tooling can run under emulation without substituting an emulated
Qt/Rust target for the native acceptance test.

Windows builds use installed MSVC via VsDevCmd with explicit target/host arch
and Ninja, avoiding dependence on a Visual Studio generator version. CMake and
compiler versions are printed in CI. macOS builds explicitly set arm64 and use
cocoa. Windows uses windows backend. Linux runs both offscreen and xcb under Xvfb.

The initial Linux CI link failure was confirmed: missing libEGL.so.1, despite
successful SDK installation and compilation. libegl1 is now an explicit runtime
prerequisite. Linux desktop jobs additionally install Xvfb and xcb support libs.
No test disabling/continue-on-error/required-check change was used to hide failure.

The harness checks a non-null Widgets raster frame and records size/SHA-256. It
also measures 150 events under 2–8 busy background CPU threads, plus the previous
normal and burst 150+150 events. This covers the existing prototype under load,
not a heavy PCB field Solver that does not yet exist. Xvfb is a software virtual
display; Windows/macOS hosted native backends are not certification of physical
monitor presentation, hardware GPU acceleration or human input latency.

The local managed container cannot launch the desktop X server with its missing
keyboard setup/socket restrictions; local xcb desktop status is Not Run. Native
CI is the prepared environment used for additional actual executions. Their
current status is recorded separately from Linux local PASS in the validation report.

Sources: [Qt 6.8 supported platforms](https://doc.qt.io/qt-6.8/supported-platforms.html),
[Windows SDK guidance](https://doc.qt.io/qt-6.8/windows.html),
[CI workflow](../../.github/workflows/qt-runtime.yml),
[race regression](../../crates/boardsi-ipc/tests/cancel_race.rs),
[current evidence](../validation/m0-qt-runtime-followup.md).
