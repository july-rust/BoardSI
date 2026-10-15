# Issue #46 — Qt native runtime cold-start timing evidence (2026-10-15 KST)

Scope: **diagnosis only**, no production GUI/Worker changes or M2 physics changes. The frozen global Qt event-loop callback-gap acceptance bound remains **1000 ms**. The UI-probe and Worker/IPC acceptance tests are unchanged.

## Independent log inspection

| Native execution | OS/backend | Qt | Entire-test timer max | Normal UI timer max | Result |
| --- | --- | --- | ---: | ---: | --- |
| [38684133789 attempt 1](https://github.com/july-rust/BoardSI/actions/runs/38684133789/attempts/1), job 116255017064 | macOS 14.8 arm64 / cocoa | 6.8.3 | 1134 ms | 35 ms | **FAIL** |
| [38684133789 attempt 2](https://github.com/july-rust/BoardSI/actions/runs/38684133789/attempts/2), job 116256698924 | Same SHA, macOS arm64 / cocoa | 6.8.3 | 728 ms | 18 ms | PASS |
| [38686672072](https://github.com/july-rust/BoardSI/actions/runs/38686672072), job 116262996731 | macOS arm64 / cocoa | 6.8.3 | 662 ms | 48 ms | PASS |
| [38687811465](https://github.com/july-rust/BoardSI/actions/runs/38687811465), job 116266593375 | signed main aecc069 / macOS arm64 / cocoa | 6.8.3 | 843 ms | 42 ms | PASS |
| [38687811465](https://github.com/july-rust/BoardSI/actions/runs/38687811465), job 116266593451 | Windows x64 / windows | 6.8.3 | 176 ms | 11 ms | PASS |
| [38687811465](https://github.com/july-rust/BoardSI/actions/runs/38687811465), job 116266593508 | Windows ARM64 / windows | 6.8.3 | 204 ms | 11 ms | PASS |
| [38687811465](https://github.com/july-rust/BoardSI/actions/runs/38687811465), job 116266593200 | Linux x64 / xcb | 6.8.3 | 60 ms | 11 ms | PASS |

The two phase-tagged macOS diagnostic runs (662 ms and 843 ms) locate the **largest callback-delivery gap** at `initialization -> normal`, ending at the first timer callback. They **do not** prove that the historical 1134 ms gap had the same location. The original run lacked phase tagging. The exact-SHA retry establishes intermittency but not remediation.

## Competing hypotheses, evidence and limits

- **Cold startup / native window creation/show:** plausible. The test installs its global `QTimer` *inside* a `QTimer::singleShot(0)` handler, then synchronously constructs/shows its first native `Window` before the first nested `QEventLoop`. Previous phase tagging cannot distinguish constructor vs `show()` vs Start click.
- **First timer dispatch / nested harness loop:** plausible. Until a nested `QEventLoop::exec()` is reached, the outer event dispatcher cannot process the registered timer; wall-clock elapsed time contributes to the global gap by design. This is *not* by itself proof of a production GUI stall.
- **Runner scheduling or CPU contention:** possible but not isolated. Existing logs contain no per-thread runnable/wait/CPU time decomposition.
- **Qt rendering:** possible during native `show()` but not individually quantified.
- **Worker launch/IPC:** less directly supported for the first gap; Worker events occur after Start, and synthetic Worker/UI probes have much smaller measured timer gaps during normal operation. Not ruled out.
- **Harness instrumentation/attribution:** confirmed **limitation**: the global metric spans pre-first-callback synchronous setup and runtime, and the phase label only describes the enclosing test, not which operation consumed the time. It is a valid callback-delivery measurement, not a causal attribution.

## Additive startup diagnostic contract

`apps/boardsi-qt-prototype/tests/runtime.cpp` now reports two *diagnostic-only* objects:

1. `app_bootstrap_ms`: duration to create `QApplication`, elapsed time to enter `app.exec()`, and the delivery time of the `singleShot(0)` test callback. Qt does not guarantee that this zero-timer callback is the first event processed. This interval is **not** silently added to or removed from the established global acceptance interval.
2. `first_window_startup_milestones_ms`: timestamps on an elapsed clock started with `RuntimeValidation::run()`. Includes global timer armed, first native Window constructor begin/end, `show()` begin/end, Start click begin/end, native `QProcess::started` signal, Worker Running state observed and first global timer callback.

All timing points use monotonic `QElapsedTimer`; callback and startup marker instrumentation consist of a small number of clock reads and JSON field writes. The diagnostic presence alone does not establish negligible perturbation; compare repeated runs before drawing performance conclusions. This is a **test harness change only** and adds no busy waiting, synchronously blocking Worker read, new retry, skipped test, or threshold change.

## Diagnostic PR #49 first CI observation (not a final-head PASS)

First diagnostic PR head `cb362be1b0b3cf89435a1598812d549e48b5920f`, [Qt run 38706346963](https://github.com/july-rust/BoardSI/actions/runs/38706346963), macOS arm64 native job 116326602083: **FAIL**, but the global callback gap was **818 ms** (below 1000 ms), with first `show()` from 8 to 238 ms after run entry, first genuine timer callback at 818 ms, and initial Worker Running observed at 828 ms. The separate failure was `Started not received` after the subsequent `completed-hang` phase spent 3064 ms; the last completed case was `cancel-hang`. No evidence currently isolates whether the later synthetic Python Worker failed to start, delayed its framed Started reply, or was affected by runner scheduling. The old 3000 ms Worker-start timeout stays unchanged. Final follow-up head adds only the **failure-phase** and **QProcess state/error/stderr/stdout** diagnostics on such a timeout.

Do not mislabel this as another global >1000 ms failure, do not infer the original 1134 ms's phase from it, and do not merge the PR until its latest HEAD has passing required CI and review.

## Next decision gate

Run the final PR head on native macOS arm64 and Windows x64/arm64 plus Linux xcb. Examine startup order and `show_end - show_begin` versus `first_global_timer_callback - global_timer_armed`. If the >1000 ms outlier cannot be reproduced, report **intermittent root cause unconfirmed** and keep #46 open; do not characterize an instrumented PASS as a fix. Compare process/OS scheduling only when matching logs can isolate it. Verify all 17 required checks, exact PR HEAD, review thread resolution and signed squash merge before merging any diagnostic PR. Keep #41 open, #35 draft/unmerged and PH-08 excluded.

## Independent review correction

Codex review on PR #49 identified a concrete P2 diagnostic-label defect: Qt does not promise that a `QTimer::singleShot(0)` callback is the first event dispatched by the application event loop. The bootstrapping key is therefore named `test_single_shot_callback_ms` rather than `first_event_dispatch_ms`. It measures only the time the test callback itself begins; no claim about earlier dispatched platform events is made. The code/timer acceptance and runtime behavior are unchanged.
