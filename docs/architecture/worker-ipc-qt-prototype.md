# ADR: M0 Rust Worker / Qt nonblocking IPC

Date: 2026-10-14 (KST). Status: **process boundary/protocol Accepted for M0;
final UI, Qt version, product toolchain and distribution licensing Pending**.
Related #3, #5, #6 (partial), specification v1.0 §§9.4,11.1–11.3,12,13.1,14.2,15.

## Decision and alternatives

Separate Rust executable, one synthetic run per process; local inherited stdin/stdout
pipes only. No listener, HTTP, network, shared BoardModel or FFI is introduced.
Rust computation runs on a thread, while the supervisor processes requests/heartbeat;
bounded queues (calculation 2, requests 8, independent client 8) limit pending traffic.
Client must keep reading. Pipe backpressure can block writes and delay heartbeat;
5 s warning/10 s force option remain client responsibilities. This is not a hard
real-time promise or an OS sandbox. Worker has the launching user's permissions.

Qt source uses C++17 Core/Gui/Widgets and QProcess asynchronous signals plus QTimer.
No `waitFor*`, solver operation or manual nested event loop occurs in UI source.
QProcess readyRead/finished/error separate protocol events from actual OS exit.
Absolute Worker path and SeparateChannels avoid PATH ambiguity and debug-log mixing.
Process restart creates a new Run UUID and resets all cursors/receive buffers.
Qt implementation is **uncompiled/unexecuted** here; its interoperability is Pending.

| Candidate | Engineering judgement (not measured product evidence) | M0 choice |
| --- | --- | --- |
| Qt Widgets | Small buttons/status prototype needs little machinery; desktop forms fit | Source prototype |
| Qt Quick | Scene/declarative UI may suit later accelerated Artwork and animated state; QML/render backend increases this experiment's scope | Pending benchmark |
| QProcess + JSON | Crash isolation, Rust/C++ toolchains independent, simple to inspect; framing/schema and copying must be maintained on both sides | M0 boundary |
| C ABI / CXX-Qt | Useful for an eventual in-process adapter, but sharing process failure/memory lifetime would weaken this boundary | Not introduced |

These UI suitability statements are **inferences/design opinions**. No final Qt Quick
versus Widgets selection is made. Official framework descriptions and QProcess API:
[Widgets](https://doc.qt.io/qt-6.8/qtwidgets-index.html),
[Quick](https://doc.qt.io/qt-6.8/qtquick-index.html),
[QProcess](https://doc.qt.io/qt-6.8/qprocess.html). Qt explicitly warns that blocking
wait calls on the GUI thread can freeze UI; asynchronous source alone does not prove latency.

## IPC v1 contract

Frame = unsigned 32-bit big-endian payload byte length + UTF-8 JSON, no delimiter.
Length must be 1..65536 before allocation; incomplete header/body at EOF is an error.
Readers support partial reception and multiple frames. Invalid UTF-8/JSON/unknown
fields/version/envelope/body are rejected. Do not attempt stream resynchronization.
Stdout contains **only framed messages**; stderr diagnostics contain no CAD data.
Before a valid Start establishes identity, protocol errors produce stderr + exit 2;
after Start, rejected requests produce scoped Failed(code,context) + exit 2.
Normal Completed/Cancelled exits 0; calculation Failed exits 1. Crash/kill/exit status
is reported by the client OS process API, never fabricated as a final Worker message.

Envelope: `version=1`, nonempty `run_id` (≤128 UTF-8 bytes), `snapshot_id` (≤256 bytes),
`seq` (1..2^53−1), `kind` (request/response/event), tagged `body`.
Each direction has its own contiguous sequence starting at 1; duplicate, reverse or
gapped sequence is rejected. Scope mismatch never advances a valid client's cursor.
A fresh session must choose a fresh run ID; IDs do not depend on elapsed time alone.

| Direction / kind | Body | Meaning |
| --- | --- | --- |
| Client / request | Start(snapshot), Cancel | One Start and at most one Cancel |
| Worker / response | Started, CancelAcknowledged | Run started / request accepted, not terminal |
| Worker / event | Result, Progress, Heartbeat | Provisional synthetic point / definite count / liveness |
| Worker / event | Completed, Failed(code,context), Cancelled | Explicit run terminal status |
| OS / client-local | exit code, abnormal exit, EOF, forced kill | Process lifecycle, distinct from run messages |

Snapshot = `{points:1..1000, delay_ms:0..1000}`. Its immutable ID is the full canonical
string `synthetic-rlgc-v1:points=N:delay_ms=D`, not a truncated hash. The v1 task fixes
R=G=0, L=250e-9 H/m, C=100e-12 F/m, length=0.1 m, Zref=50 Ω, f(i)=i×1e6 Hz.
Task/model/version changes require a new identity/protocol decision. Start identity
must agree with the snapshot's exact fields. No full BoardModel serialization exists.
Result includes point/frequency/S11 and S21 real/imaginary components and mandatory
`synthetic=true`; finite-value checks are required. There is no physical Validated flag.

Run state: Awaiting → Running → Completed, or Running → CancelRequested →
CancelAcknowledged → Cancelled, or Failed. Ordered results are provisional; Completed
requires exactly all requested points. Terminal states reject later events. Client
failure/abnormal exit discards all collected results, even after Completed. Cancelled
also discards temporary results in this minimal prototype (persistent confirmed chunks
are outside scope). Completion racing Cancel is conservatively rejected if the client
already requested cancellation; future protocol can explicitly settle this race.
A received Completed event is provisional execution evidence until exit 0 is observed.

Heartbeat scheduler: 1 s, independent of point production. Clients distinguish ≥5 s
since any **accepted** message and ≥5 s since last Heartbeat. Progress cannot hide a
missing heartbeat. These are warnings, not automatic numerical failure. ≥10 s after
Cancel request exposes force termination, not automatic kill. Partial headers do not
reset clocks. Qt timers/parser batches implement these policies in source only.

## Toolchain, dependencies, licensing and deployment

Rust 1.75.0 remains unchanged; exact direct pins serde 1.0.197 / serde_json 1.0.114.
Cargo.lock pins all transitive versions; MSRV of resolved dependencies is ≤1.75.
No async runtime, external process library, CXX-Qt or networking dependency added.
CI fetches the lockfile once, then builds/tests offline. Existing `docs` check unchanged.
No duplicate workflows or artifact uploads. Qt SDK is not added to CI.
Qt source requires Qt≥6.5 and CMake≥3.21; Qt 6.8 is a **candidate**, not a tested baseline.

| Locked dependency | Version | Manifest license |
| --- | --- | --- |
| serde / serde_derive | 1.0.197 | MIT OR Apache-2.0 |
| serde_json | 1.0.114 | MIT OR Apache-2.0 |
| itoa | 1.0.18 | MIT OR Apache-2.0 |
| ryu | 1.0.23 | Apache-2.0 OR BSL-1.0 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| syn | 2.0.119 | MIT OR Apache-2.0 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |

Evidence: each downloaded crate's Cargo.toml/packaged LICENSE files and repository
Cargo.lock, not an assumed license based on package name. serde_derive/proc-macro
support crates are build-time; runtime uses serde/json/itoa/ryu. Future binary/source
redistribution must include applicable notices/license texts; this PR ships no binaries.

Qt offers commercial and open-source licensing, including LGPLv3 with conditions;
some modules are GPL-only under the open-source option. Core/Gui/Widgets are the only
requested modules here. Future LGPL distribution review must check actual Qt build
licenses/third-party notices, provide applicable Qt source/modifications, allow users
to replace/relink the library and avoid conflicting EULA restrictions. Dynamic linking
is the provisional deployment preference, not proof of compliance. Static linking or
store restrictions require separate review; commercial licensing is an alternative.
Do not infer that Rust process separation removes the GUI's Qt obligations. BoardSI's
own distribution license and final Qt package compliance remain Pending.
Official sources: [Qt licensing](https://doc.qt.io/qt-6.8/licensing.html),
[LGPL obligations](https://www.qt.io/development/open-source-lgpl-obligations),
[third-party code](https://doc.qt.io/qt-6.8/licenses-used-in-qt.html).

Worker must be shipped next to the GUI with matching architecture/protocol versions;
launch from an installation-resolved absolute path, detect missing executable/version
mismatch, and handle dynamic Qt/platform plugin dependencies. macOS bundle paths,
Windows runtime DLLs, signing/notarization and installers are future work.
Qt 6.8 documents Windows x64 and Windows ARM64/MSVC 2022 (ARM64EC is separate):
[Windows platform reference](https://doc.qt.io/qt-6.8/windows.html).
That is framework capability, not BoardSI verification. Native Rust results and Qt
Not Run status for each platform belong to the [validation report](../validation/m0-worker-ipc-qt-prototype.md).

## Three remaining risks

1. Qt source interoperability, real Event Loop p95/max/live-update latency and native
   GUI deployment are unverified. Qt's parser/burst/exit behavior needs actual tests.
2. Pipe backpressure, cancel/completion races and saturated computation are only
   partially covered; this one-run synthetic prototype is not a production scheduler.
3. Full board snapshot hashes, confirmed result chunks/checkpoint recovery and package
   license/SBOM decisions must be resolved in later milestones before distribution.
