# M0 fifth stage: actual Qt / Rust Worker execution

Historical evidence for initial PR #30 commit `417f17f264dbc060d80c0e274c91596c42bb6116`.
Later runtime/CI/race updates supersede the current-status statements here; see
[follow-up validation](../validation/m0-qt-runtime-followup.md).

Date: 2026-10-14 KST. Baseline main `78924b79ad7d7ecd1ea85d0ea85b9f444cc35141`;
PR #29 confirmed merged. Related #3, #5, #6 remain open. Specification v1.0 unchanged.

**PASS for Linux x86_64 offscreen synthetic prototype integration.**
An actual QApplication::exec() ran the existing Window and separate QProcess
Rust Worker. Not a source-only or Rust-only substitution. The final harness run
passed **23 scenarios / 221 assertions** in **47,269 ms**. The earlier run before
adding whole-test timer monitoring also passed; figures below are the final run.
Raw evidence: [JSON](qt-runtime-linux-offscreen.json).

## Environment and scope

Ubuntu 24.04.3 LTS, Linux 6.18.44 x86_64; Intel Xeon Platinum 8573C,
9 visible logical CPUs, MemTotal 10,206,504 kB (container-visible capacity,
not a dedicated/reference desktop). Qt 6.8.3 official gcc_64 SDK, CMake 3.31.6,
GCC 13.3.0 C++17. Qt Core/Gui/Widgets only. Rust 1.75.0 unchanged.
Qt build: default single-config flags; C++ `-Wall -Wextra -Wpedantic -Werror`.
Normal Worker: 100 synthetic lossless RLGC points, nominal 50 ms per-point delay;
there is no PCB import/field extraction/production Solver in this experiment.
Burst: 20,000 heartbeat frames followed by 100 Result and 100 Progress frames.

| Qt native platform | Build | Actual GUI / Worker execution |
| --- | --- | --- |
| Linux x86_64 | PASS | PASS, offscreen |
| Windows x64 | Not Run | Not Run |
| Windows ARM64 | Not Run | Not Run |
| macOS ARM64 | Not Run | Not Run |

Offscreen exercises actual Widgets property updates, QApplication event dispatch,
timers, process pipes and lifecycle. **Screen presentation, GPU/render latency,
desktop input, Artwork Pan/Zoom/tabs and CPU-saturated real Solver are Not Run.**
T10/T46/T50 have measured partial coverage; T13/T34/T39 overall acceptance remains
incomplete. This does not close the M0 milestone or validate actual PCB SI accuracy.

## Measured responsiveness

Queued UI probes originate on a separate producer thread and timestamp enqueue
to GUI callback; each callback updates a real widget title. 150 probes at 10 ms
spacing during actual Rust work; 150 at 1 ms spacing during the burst scenario.
p95 is nearest-rank percentile. A 10 ms timer runs across the whole test to find
event-loop dispatch gaps. Host receipt-to-widget-property update is measured
inside the production accept handler; it is **not** remote Worker emission latency
or physical screen paint/presentation latency. Timestamp resolution is 1 ms
except the steady-clock UI probes.

| Measurement | Actual final measurement | Samples / scope |
| --- | --- | --- |
| Normal UI p95 / max | 0.090 / 1.149 ms | 150 events |
| Burst UI p95 / max | 4.330 / 31.482 ms | 150 events |
| Normal / burst timer max gap | 11 / 32 ms | 10 ms timer during probe windows |
| Entire-test timer max gap | 99 ms | 47,269 ms total; no >1 s gap |
| Start / Cancel local state-display max | 0 / 0 ms at 1 ms resolution | normal + repeated runs and faults |
| Cancel acknowledgement max | 5 ms | 5 real Rust runs |
| Cancel to observed/drained exit max | 10 ms | 5 real Rust runs |
| Result/Progress receipt-to-widget-update max | 1 ms | 100 Result + 100 Progress |
| Heartbeat receipt gap min / max | 1,001 / 1,002 ms | 6 heartbeats, 5 intervals |
| Rust unresponsive warning / force-stop delay | 5,076 / 10,073 ms | 1 scenario; force measured from Cancel |
| Cancelled-but-alive warning / force-stop delay | 5,092 / 10,070 ms | 1 scenario; force measured from Cancel |
| Completed-but-alive warning / force-stop delay | 5,097 / 10,041 ms | 1 scenario; force measured from Completed |
| Missing heartbeat despite traffic warning | 5,096 ms | 1 scenario, continued Progress traffic |

Watchdog thresholds remain 5,000 / 10,000 ms; 100 ms monitor scheduling explains
the small measured delay beyond the threshold. All responsiveness targets tested
here passed. Probe timestamps and the global timer are actual runtime measurements.
Human mouse-to-screen latency and remote emission-to-receipt are not measured.

## Integration and fault coverage

- Real Rust: Start sends fresh Run UUID, exact immutable Snapshot ID and schema 1;
  Started/Progress/Result/Heartbeat accepted, 100 results and progress displayed.
  Five rapid Start/Cancel/restart runs have distinct identities and cleared buffers.
  The first cancellation occurs after a provisional result; its displayed count
  is verified cleared. Failure paths likewise clear displayed results.
  CancelRequested, acknowledgement, Cancelled and OS exit are separately checked.
- Actual Rust abort is CrashExit and GUI survives. Fixture normal exit 7 remains
  NormalExit with nonzero code, distinct from CrashExit. Fixture scoped Failed
  events preserve `FixtureFailure: preserved context` across normal exit 1/2.
- Actual unresponsive Rust ignores Cancel; warning appears and Force stop kills
  and reaps it. Fixture Cancelled/Completed while alive must keep Start disabled;
  watchdog and Force stop remain active until OS exit. Active GUI close kills
  asynchronously, drains/reaps the Worker and then closes the window.
- Test-only Python peers: wrong schema, wrong snapshot, foreign/late Run ID,
  duplicate/reverse sequence, oversized/malformed/truncated frame fail closed.
  Split headers/payloads succeed; 20,200 burst frames succeed with GUI probes.
- Completion after CancelRequested fails closed under the existing strict Rust
  and Qt state contract; no provisional results survive. This is a tested policy
  limitation, not a new IPC change or successful cancellation.

Each rejected scenario is expected to display an error; those errors are test
inputs, not unexpected validation failures. Whole-process exits are checked after
stdout drain; acknowledgement and terminal messages are never treated as OS exit.
Detailed exit codes/status and error text are retained in the raw JSON.

## Regression, warnings and setup errors

Unchanged Rust Debug **67 PASS**, Release **67 PASS**:
numerics 15, model 28, IPC 11, independent-client race 1, process 12 per profile.
`cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets --locked
--offline -- -D warnings` PASS. Rust test/clippy and final Qt compile warnings 0,
errors 0. No regression test was removed or disabled.

The final local Qt runtime emits 113 expected offscreen `propagateSizeHints`
diagnostics and 0 XDG_RUNTIME_DIR diagnostics. These platform messages are disclosed;
the run is not described as warning-free. Optional XKB/Vulkan headers were absent
locally, reported during CMake discovery but not required by offscreen compilation.
CI installs those prerequisites. No unexpected runtime failure remained.

Setup obstacles were resolved locally: system apt sandbox/cache permissions,
aqt Unix-socket restriction (official archive extraction instead), missing
OpenGL/ICU files, and stale/invalid local Rust artifacts (clean rebuild of affected
packages). These are not changes to Rust algorithm/toolchain or disguised passes.
Qt archive SHA-256 and reproducible CI installation are in the
[ADR](../architecture/qt-native-runtime.md).

## CI and remaining work

New `.github/workflows/qt-runtime.yml` builds both the real GUI and harness, builds
the Rust Worker with 1.75.0, runs CTest and prints measurements. It fails on build,
runtime assertion, timeout, percentile/deadline or report-write failure. It does
not use continue-on-error, skip the test or upload binaries/artifacts. Existing
Rust native CI and required `docs` check are untouched. Qt is not a required check.
Local success does not predeclare Actions success; use the PR's current checks.

Remaining, maximum three groups:
1. Windows x64/ARM64 and macOS ARM64 actual Qt execution and deployment are Not Run.
2. Desktop rendering/GPU, full Artwork interactions and saturated real Solver
   responsiveness are Not Run; synthetic offscreen acceptance is limited.
3. Completed/Cancel race remains strict rejection; coordinate a state-contract
   decision before production semantics, with Rust/Qt regression tests.

Recommended next M0 stage: prepared native desktop execution matrix plus the
completion/cancel race policy. Keep Issues #3/#5/#6 open and do not merge this PR.
