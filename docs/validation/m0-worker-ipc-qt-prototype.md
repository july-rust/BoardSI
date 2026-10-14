# M0 Worker / IPC / Qt prototype validation

Date: 2026-10-14 (KST). Base main: `79bbc2994ec3157b346058acdd46a316f9185cc1`;
PR #28 merged verified. Related #3/#5/#6 remain partial/open. Specification v1.0,
boardsi-numerics and boardsi-model source/tests/fixtures/toolchain are preserved.
No real PCB, company data, importer, geometry RLGC, Progressive Sweep or release.

## Actual local environment and tests

Ubuntu 24.04.3 Linux x86_64 container; Intel Xeon Platinum 8573C, 9 visible logical CPUs;
visible RAM 10,451,460,096 bytes (snapshot available 9,174,577,152 bytes), no swap;
overlay filesystem total 33,770,192,896 bytes, available 28,739,014,656 bytes.
Underlying SSD type/performance and host resource allocation are **unknown**.
Rust 1.75.0 (82e1608df), cargo 1.75.0. Reused available extracted Rust tools with their
library search path; no product toolchain change. Display/Qt SDK: unavailable.
This container is not the final reference desktop/low-RAM product benchmark machine.

| Check | Actual result |
| --- | --- |
| fmt --all -- --check | PASS |
| clippy --workspace --all-targets --locked --offline -- -D warnings | PASS |
| workspace debug | PASS 65 tests: numerics 15 + model 28 + IPC 10 + Worker/process 12 |
| workspace release | PASS same 65 tests, FAIL 0 |
| existing numerical independent reference | PASS all 180 points; unchanged max complex-S absolute error 4.0953603625310199e-10 |
| git diff --check / preservation check | PASS |
| compiler/test warning: / error: in final local logs | 0 / 0 |
| Qt C++ build / QApplication execution | **Not Run** |

Qt/CMake/qmake/pkg-config Qt6Widgets were not present. No large SDK installation or
repeated build attempts were made. C++ source includes minimal buttons/status/progress,
QProcess signals and timers; source review does not establish build or UI correctness.

## Evidence and timing (independent Rust client, not GUI)

| Scenario | Test / result | Specification scope |
| --- | --- | --- |
| serialization/schema | all 10 body kinds roundtrip; unknown field/version/kind/nonfinite rejected | T13/T39 partial |
| frame handling | byte-at-a-time reception, multiple frames, truncated header/body, invalid JSON, zero/oversize payload rejected | T13 partial |
| run/snapshot/sequence | duplicate/reverse/gap, foreign run, changed snapshot and restarted session rejected; cursor unchanged | T39/T46 partial |
| normal process | 10 synthetic points, Completed + exit 0, results count 10 | T10 partial |
| cancel | request → acknowledgement → Cancelled → exit 0; acknowledgement not terminal | T10/T50 partial |
| crash/kill | real abort and kill; client test process alive, Failed, results discarded | T10/T46 partial |
| unresponsive | real fault Worker stops commands/heartbeat; warning then force option, no fabricated acknowledgement | T10/T50 partial |
| heartbeat omitted with progress | separate heartbeat warning despite recent progress | T50 partial |
| cleanup | kill/wait + pipe/reader cleanup, including full queue at Drop | T10 partial |
| actual Worker request rejection | duplicate/gapped sequence/foreign run/snapshot produce Failed(code) + nonzero exit | T13/T39/T46 partial |
| unsupported version at entry | separate stderr error and nonzero exit, empty protocol stdout | T13 partial |
| incomplete/abnormal completion | incomplete Completed rejected; abnormal exit invalidates collected results | T39/T46 partial |

One Debug and one Release run, automated tests run in parallel. Samples are limited;
these are observed values, not percentile guarantees. The cancellation observation
is an upper bound for request-to-ack because it includes actual Cancelled/exit.

| Metric | Debug | Release | Sample |
| --- | --- | --- | --- |
| Cancel request through acknowledged/cancelled/exit | 6.266 ms | 4.639 ms | 1 per profile |
| Heartbeat observed inter-arrival | 999.692 / 1001.302 / 1001.911 ms | 999.166 / 1005.795 / 1000.041 ms | 3 intervals per profile |
| 5 s silence/heartbeat warning | 5003.679 ms | 5000.137 ms | 1 per profile |
| 10 s Cancel force option observed | 10005.935 ms | 10006.917 ms | 1 per profile, includes subsequent reap polling |
| UI p95≤200 ms / max≤1 s (100+ interactions) | Not Run | Not Run | no Qt Event Loop |
| actual UI Cancel receipt indication≤1 s / live update≤1 s | Not Run | Not Run | no Qt Event Loop |
| Pause/Resume, full PCB stress/saturated CPU | Not Run | Not Run | outside minimum implemented task |

Watchdog exact boundary policy also tested with injected monotonic times (4999 ms,
5 s, 10 s). That deterministic policy test is separate from real process timing.
Fault modes require explicit `--fault=crash/unresponsive/no-heartbeat` CLI argument;
not part of the Start schema. Synthetic delay is an experiment, not PCB performance.
Results are provisional counts/data, never PCB SI or physical Validated.

## Native CI status

Existing numerical-core workflow extended for IPC/Worker/Qt source path changes;
locked fetch followed by offline Debug/Release workspace checks. Native identity
checks remain and Worker/client tests log their actual architecture/OS. No duplicate
workflow, no artifact uploads, no `docs` required-check change. Qt SDK not installed.

| Platform | Rust actual native execution | Qt build / actual GUI |
| --- | --- | --- |
| Linux x86_64 local | PASS Debug + Release, 65 tests each | Not Run |
| Linux x86_64 GitHub Actions | PASS Debug + Release, 65 each | Not Run |
| Windows x64 | PASS native CI Debug + Release, 65 each | Not Run |
| Windows ARM64 | PASS native CI Debug + Release, 65 each | Not Run |
| macOS ARM64 | PASS native CI Debug + Release, 65 each | Not Run |

Tested source commit: `6c52774116d9731be1b9e47b10fcf372d0e9e951`.
[Four-platform CI](https://github.com/july-rust/BoardSI/actions/runs/38552339713) and
[required docs CI](https://github.com/july-rust/BoardSI/actions/runs/38552339916) succeeded.
All four full job logs were read: each has Debug 65 + Release 65 PASS and compiler/test
`warning:`/`error:` counts 0. Windows/macOS compiler host, runner architecture and
actual executable identity were verified; Worker/client native identity appears in logs.
Checkout test-merge commit `2a18e33` was fetched and has zero file differences from
the source commit. Prior PR #27/#28 CI results are not reused.

| Native job | Compiler host / client executable | Debug / Release |
| --- | --- | --- |
| [Linux 115813050311](https://github.com/july-rust/BoardSI/actions/runs/38552339713/job/115813050311) | x86_64 Linux / x86_64 linux | 65 / 65 PASS |
| [Windows x64 115813050693](https://github.com/july-rust/BoardSI/actions/runs/38552339713/job/115813050693) | x86_64-pc-windows-msvc / x86_64 windows | 65 / 65 PASS |
| [Windows ARM64 115813050549](https://github.com/july-rust/BoardSI/actions/runs/38552339713/job/115813050549) | aarch64-pc-windows-msvc / aarch64 windows | 65 / 65 PASS |
| [macOS ARM64 115813050528](https://github.com/july-rust/BoardSI/actions/runs/38552339713/job/115813050528) | aarch64-apple-darwin / aarch64 macos | 65 / 65 PASS |

This added CI evidence changes only this report. Code/lockfile/workflow remain exactly
the tested source. GitHub pull_request path filters consider the full PR diff, so a report-only update
can also retrigger this same numerical workflow. This is not a second workflow or
artifact upload; distinguish HEAD CI status from the completed source-specific evidence.

## Reproduce and limits

```sh
cargo fetch --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline -- --nocapture
cargo test --workspace --release --locked --offline -- --nocapture
```

Source evidence: [IPC tests](../../crates/boardsi-ipc/tests/contract.rs),
[actual process tests](../../apps/boardsi-worker/tests/process.rs),
[Worker](../../apps/boardsi-worker/src/main.rs),
[independent client](../../apps/boardsi-worker/src/lib.rs),
[Qt source/instructions](../../apps/boardsi-qt-prototype/README.md),
[ADR/protocol/license](../architecture/worker-ipc-qt-prototype.md).

Transport limits queues/frame allocation, but OS pipe/QProcess buffer backpressure,
large-board memory/RAM caps, persisted results/recovery and final deployment are not
fully validated. Qt source only, so **GUI–Worker interoperability and GUI nonblocking
acceptance remain Not Run**. T10/T13/T34/T39/T46/T50 and M0 overall are not complete.
Next M0 priority: on a prepared native Qt host compile/run this small prototype,
measure ≥100 interactions, inject crash/stall/bursts and verify restart isolation;
then choose Widgets/Quick using actual Viewer/renderer evidence before expanding UI.
