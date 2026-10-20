# M0 F8 provisional Qt global event-loop timing policy — user decision 2026-10-20

## Decision and scope

The BoardSI project owner approved changing **only the M0 Qt native test's maximum whole-runtime global event-loop timer callback gap** from **1,000 ms** to **1,500 ms**. This is a temporary M0/prototype acceptance decision to unblock practical progress, **not proof that the intermittent cold-start delay was fixed**. Revisit the bound when the application is released and stabilized, with real-machine profiling and agreed performance targets.

- Target: `apps/boardsi-qt-prototype/tests/runtime.cpp`, `entireMaxGap` over the unchanged whole native test from before first QWidget show until the final measurement.
- New condition: measured `entire_test_timer_max_gap_ms <= 1500`. `global_event_loop_gap_limit_ms=1500` is emitted alongside the original raw observation.
- Keep `Qt::PreciseTimer`, callback sampling start, end-of-test final sample, native Windows x64/ARM64 and macOS ARM64 matrix, and PASS/FAIL behavior. No dropped samples, delayed start, skipped phases, synthetic metric adjustment or forced-success retry.
- Preserve all other acceptance conditions as written: UI interaction p95 <=200 ms, per-interaction latency limits, cancel/acknowledgment/display and live-result response budgets, Heartbeat and process deadlines, Worker/IPC recovery and error isolation.
- This provisional F8 threshold does **not** silently revise specification v1.0 §12 or its one-second user-interaction/product targets. Such changes require a separate user decision and full requirement-trace update.

## Historical evidence and new decision

The earlier 1,000 ms acceptance was the correct rule *at the time* of historical CI runs. Preserve their status, source SHA, raw measurements, failure descriptions and logs as they were. In particular, the macOS ARM64 **1,331 ms** failure from GitHub Actions [39375357028 / job 118472164680](https://github.com/july-rust/BoardSI/actions/runs/39375357028/job/118472164680) was a genuine **FAIL under the old 1,000 ms rule**. It would be numerically under 1,500 ms, but it is **not retroactively converted to CI PASS**, and it does not prove a causal remedy.

Issue [#46](https://github.com/july-rust/BoardSI/issues/46) may remain open as a **deferred root-cause/performance-hardening follow-up**. For M0, F8 may qualify as **PASS under the provisional threshold** only with **fresh exact-head native CI**, unchanged other checks, independently reviewed code and correct protected-main evidence. If a measured run exceeds 1,500 ms or another Qt requirement fails, F8 remains **BLOCKED**.

## Release/stabilization revisit criteria

Re-evaluate this temporary threshold before treating the program as production-performance-ready, after an actual release/stabilization workload exists. Collect representative startup, first-window/show, Qt periodic dispatch, UI interactions, Worker and cancellation metrics on appropriate machines (including Windows ARM64 and macOS ARM64). Separate cold start/OS scheduling issues from real GUI-thread blocking. Decide explicitly whether to restore the stricter 1,000 ms, introduce a phase-specific startup allowance, or retain a better-justified target. Do **not** silently extend the M0 provisional threshold into a production acceptance contract.

## Other outstanding M0 gates

This timing decision alone does not close F1/F6/F7, qualify a real ~8 GiB baseline PC, complete two redistributable reference-board fixtures, certify a Qt deployment or legal compliance, or validate any physical PCB SI result. Keep PH-08 original HMM findings and Draft PR #35 untouched.

## Change control

Record this exception in the M0 completion gate and Issue #46. Promote only after normal 17 required CI checks, fresh exact-source code review, strict GitHub protected-branch signature/rules and Verified-only Squash promotion. A code branch or PR existing is not evidence that protected main has adopted the new threshold.
