# C_025_09 — macOS Qt event-dispatcher latency diagnostics and bounded Issue #46/#41 disposition

**First audit date:** 2026-10-31 KST. **Tracker:** [#327](https://github.com/july-rust/BoardSI/issues/327). **Starting protected main:** `c75f6641f5ad46efa8607a3ecf43ce8f59ca5324`; all promotion claims must be verified independently on the **final** SHA. This is a diagnostic and acceptance-boundary exercise, not a causal fix claim or a new product-release benchmark.

## Immutable observed evidence

- [PR #326 exact-head Qt native workflow #41204123648](https://github.com/july-rust/BoardSI/actions/runs/41204123648), macOS ARM64 job `124545621846`: **completed/success** on 2026-10-31. Source PR head `275300e0188547f8ffeec86b443b916e44a10c1a`. Qt 6.8.3; native `cocoa`; `entire_test_timer_max_gap_ms=554`, provisional whole-test gate `1500`, normal UI p95 `0.161375 ms`, normal UI maximum `0.76175 ms`, UI timer max `21 ms`, burst UI timer max `25 ms`, CPU stress timer max `24 ms`.
- At run time `t=0` the global precise 10 ms timer was armed; the first nested queued Qt zero-delay timer dispatched at **205 ms** and the first global precise timer at **554 ms**, with `first_nested_zero_timer_callback_order=1` and `first_global_timer_callback_order=2`. QProcess started signal was at 209 ms and Worker Running observed at 572 ms. The max global gap was a single **initialization -> normal** first-callback interval; all other measured phase maxima were substantially lower. A real Qt callback was processed earlier than the first periodic global timer callback.
- This sequence is **consistent with** timer-specific delayed delivery while Qt is handling startup, but does **not establish** a timer bug, OS scheduling cause, missing UI redraws, root cause, user-device behavior or a permanent correction. A first global timer interval measures callback delivery, not pure UI-thread busy time. Multiple causes remain possible.
- Earlier macOS runs measuring **1134 ms** (run #38684133789, initial attempt) and **1331 ms** (run #39375357028/job #118472164680) under the then-applicable **1000 ms** gate were genuine FAIL at their source commits; a later PASS does not reclassify those executions. The owner-approved **provisional M0 only** 1500 ms gate ([policy](m0-f8-provisional-1500ms-policy.md)) remains unchanged by this work. Specification v1.0 product UI latency, all other process/heartbeat/cancel limits remain untouched.

## New diagnostic-only source change

`apps/boardsi-qt-prototype/tests/runtime.cpp` now records, on the GUI thread:
1. `dispatcher_awaited_wake_pairs` — observed paired Qt event dispatcher `aboutToBlock` → `awake` signals.
2. `dispatcher_about_to_block_to_awake_max_ms` and `dispatcher_worst_wait_phase` — longest observed dispatcher waiting/wakeup interval and the harness phase when signaled.
3. `first_zero_before_precise_delta_ms` — when both startup callbacks are seen in order, delta of first precise timer callback relative to first zero-delay callback. Otherwise explicitly `NOT_OBSERVED`.

**Interpretation limits:** `aboutToBlock` → `awake` spans Qt dispatcher waiting **and OS scheduling**; nested Qt event loops can complicate pairing. This alone **cannot** distinguish host starvation from native-event queue behavior, rendering, or a GUI-thread computation bottleneck. Diagnostics are observational and **not gates**, so zero observed pairs or a large wait cannot automatically pass/fail or establish cause. Independent representative hardware profiling would still be necessary for release reliability certification.

The original global 10 ms `Qt::PreciseTimer`, its 1500 ms M0 gate and final observation order, 250 ms LOG-ONLY long-gap threshold, 28 Qt runtime test cases / 450 UI probes, Rust process isolation and all other existing acceptance checks are kept unchanged. The source change does not intentionally slow or exempt the event loop.

## Assessment / safe closeout

| Area | State before proposed change | Condition for closing |
| --- | --- | --- |
| #46 intermittent macOS Qt gap | **OPEN / ROOT_CAUSE_NOT_CERTIFIED** | Demonstrated mechanism, bounded remedy, reproducible pre/post regression, representative native + production-related validation. A green synthetic CI run alone is not enough. |
| #41 PH-07 CI/audit and policy hardening | **OPEN**, active ruleset #25046595 and #43 previously confirmed | Close only when the independent #46-related condition and any remaining own issue DoD are actually met. |
| M0 F8 | **provisional synthetic PASS on previous approved sources** | The new branch must pass its own native matrix; no production global-gap acceptance is inferred. |
| M0 F1/F6/F7, #2/#3/#6/#7 | **BLOCKED/OPEN** | Not affected by dispatcher-only diagnostics. |
| G5 Draft #35, PH-08, P009, physical SI, U/T | **NO PROMOTION** | Out of scope. |

## PR and acceptance ledger (must complete with actual links, not projections)

- [ ] Exact **final PR HEAD**: at least 17 strict required checks all SUCCESS; record complete observed job/workflow counts and native Qt source SHA, platform and diagnostic metrics.
- [ ] Independent **latest exact-head Codex** review complete; no unresolved substantive findings; all review threads resolved. Any changed HEAD resets this check.
- [ ] Protected **Squash** into main; check `verification.verified=true` /`reason=valid`, single parent and exact merge SHA. Never bypass ruleset #25046595.
- [ ] Fresh independent **post-main** check runs and push workflows SUCCESS on that merge SHA.
- [ ] Independent Issue #46 / #41 re-read and evidence-scoped updates. Keep them OPEN if root cause and permanent remediation are unproven.
- [ ] Close bounded tracker #327 only after the diagnostic PR itself passes the entire integration gate; closing it must not imply resolution of #46/#41 or actual macOS product performance.

The protected main currently has the earlier diagnostics (first zero timer vs first precise timer and phase tags); **new dispatcher signals are proposed only until the branch is accepted**. Never write the above future-gate items as PASS before observing real CI/review/merge.
