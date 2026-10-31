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

## Verified PR #328 closeout — bounded diagnostic scope only

- [x] **Exact PR final HEAD `26f93d8eb7f6afe95b1f7163b171048960ba9ba7`: 26/26 GitHub Check Runs SUCCESS**, including the 17 strict required ruleset contexts and native Windows x64/ARM64, macOS ARM64 and Linux Qt. [PR #328](https://github.com/july-rust/BoardSI/pull/328), [exact-head checks](https://github.com/july-rust/BoardSI/commit/26f93d8eb7f6afe95b1f7163b171048960ba9ba7/checks), [Qt run #41206844425](https://github.com/july-rust/BoardSI/actions/runs/41206844425). No unsuccessful or pending checks in the final source check readback.
- [x] **Independent latest exact-head Codex** review `completed` on the same SHA, `has_any_findings=false`, `has_unresolved_findings=false` and 0 unresolved review threads. [Codex receipt and review request](https://github.com/july-rust/BoardSI/pull/328).
- [x] Protected **Verified signed one-parent Squash** into [`main@903b61cf9850c2c50a7a32658686d4db2451db49`](https://github.com/july-rust/BoardSI/commit/903b61cf9850c2c50a7a32658686d4db2451db49), `verification.verified=true`, `reason=valid`, one parent `c75f6641f5ad46efa8607a3ecf43ce8f59ca5324`. Active Ruleset #25046595 still has 17 required strict contexts, signed commits and Squash-only no bypass.
- [x] **Independent post-merge main SHA** [21/21 checks SUCCESS](https://github.com/july-rust/BoardSI/commit/903b61cf9850c2c50a7a32658686d4db2451db49/checks) and **8/8 push workflows SUCCESS**, including [native Qt #41207619823](https://github.com/july-rust/BoardSI/actions/runs/41207619823), Numerical Core, native FEM, PH-06/07 and docs. These were separate from PR-head checks.
- [x] **Actual new-metric PR native results**: Linux x64 `entireMaxGap=186 ms`, dispatcher max wait 8 ms; Windows x64 76/3 ms; Windows ARM64 85/5 ms; macOS ARM64 647/33 ms, first zero at 232 ms then first global precise timer at 647 ms (415 ms delta). All reported PASS under existing 1500 ms M0-only limit. On **post-main macOS** (run #41207619823) the global maximum was 827 ms, dispatcher wait max 35 ms, first-zero-to-precise delta 441 ms, normal UI p95 0.609792 ms. **The initial delayed precise-timer callback still occurs; no causal fix established.**
- [x] **Issue #46 and #41 individually re-read and annotated** with reproducible evidence. Both deliberately **OPEN**: no confirmed root cause, targeted remedy, or representative real-device/release test. Older 1134/1331 ms under their respective 1000 ms thresholds remain historical FAIL. Issue #43 ruleset work is already CLOSED and is not a fresh blocker.
- [ ] **Audit tracker #327 final administrative closure**, only after this post-verification *documentation-only follow-up PR* itself passes exact-head required checks, independent review, protected Squash and independent post-main success. This is separate from the completed code PR #328 gate and does not certify Issue #46/#41 closure.

**Closeout status:** The C_025_09 diagnostic code and source acceptance are VERIFIED on main, while the underlying Issue #46 macOS causal stabilization, Issue #41 full issue-level acceptance, actual product UI latency and broader M0 F1/F6/F7 remain **OPEN/BLOCKED as before**. The follow-up record does not modify compiler/Qt code, synthetic test thresholds, historical findings, PH-08 physics, P009 or Draft PR #35.
