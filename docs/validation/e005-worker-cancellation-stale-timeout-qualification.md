# E_005 — Non-blocking Worker post-ack late-response qualification

**Audit / implementation date:** 2026-11-11 KST. **Tracking:** [Issue #5](https://github.com/july-rust/BoardSI/issues/5). **Source baseline:** `main@1ceaef92791f1fb4875b782c01ac0a6c1c976511` (observed before the E_005 branch). **Scope:** one post-CancelAcknowledged late-data path and clean same-window restart in the **synthetic** M0 Qt/Worker prototype.

## Evidence contract and code

| Boundary | Source change | Qualified behavior to verify |
| :--- | :--- | :--- |
| Rust IPC RunGuard | [boardsi-ipc](../../crates/boardsi-ipc/src/lib.rs) | Result/Progress permitted while Running/CancelRequested; rejected after CancelAcknowledged, without advancing sequence, receive timestamp, or result count |
| Qt QApplication/QProcess | [Qt host](../../apps/boardsi-qt-prototype/main.cpp) | Same post-ACK semantic gate; rejected calculation data produces visible fail-closed result and provisional count zero |
| Independent Rust regression | [E_005 tests](../../crates/boardsi-ipc/tests/e005_cancel_late_response.rs) | In-flight pre-ACK Result preserved as provisional; post-ACK data rejected; Heartbeat and Cancelled still legal; prior run ID cannot move next run cursor |
| Native Qt fixture | [fault_worker.py](../../apps/boardsi-qt-prototype/tests/fault_worker.py), [runtime.cpp](../../apps/boardsi-qt-prototype/tests/runtime.cpp) | Deterministic post-ACK Result/Progress rejection; same QProcess/Window safely restarts a real Rust Worker after fixture failure |

**Intentionally unchanged:** IPC v1 framing and message kinds, pre-ACK completion-race behavior, correct asynchronous Cancel/Cancelled versus OS exit, watchdog warning at 5 seconds and Force Stop availability after 10 seconds. Actual Rust Worker already suppresses calculation outputs after acknowledging Cancel (see `apps/boardsi-worker/src/main.rs`); the test-only peer intentionally violates that contract.

## Native/CI acceptance — fill from exact final PR HEAD

- PR number / final HEAD: **NOT_VERIFIED until created and CI completes**
- Native Rust test results (target OS + Debug/Release): **NOT_RUN in this report yet**
- Native Qt QApplication/QProcess test results (target OS, backend, SHA): **NOT_RUN in this report yet**
- Required PR checks, independent review, unresolved threads: **NOT_VERIFIED**
- Verified Squash SHA and exact new-main Push CI: **NOT_VERIFIED**
- Final E_005 scoped closure status: **PENDING**, never inferred from source changes alone.

The operator must place actual URLs, SHAs, per-platform PASS/FAIL/NOT_RUN, review disposition, and merge/main Push checks in Issue #5 before changing the sole checklist's E_005 status to COMPLETE. Do not claim an independent review from a self-comment or an unexecuted offscreen test.

## Exclusions and issue disposition

This is a bounded synthetic M0 transport/state-machine qualification, **not** full Issue #5, U22/U23/U44, T10/T34/T46/T50, heavy real PCB/solver scheduler, production snapshot/cache, real GPU behavior or product 30 GHz SI acceptance. Existing historical failures (including PH-08 Gate Q) and the macOS global timer question (#46/E_029) remain unchanged.

Issue #5 remains OPEN until its original full Definition of Done is independently met. Once this E_005 unit is accepted, other #5 obligations stay explicitly OPEN or BLOCKED with evidence and next action; do not close the parent issue just to reduce the count.
