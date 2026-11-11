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

## Native/CI acceptance — final code PR #588

- **Code PR:** [#588](https://github.com/july-rust/BoardSI/pull/588), final HEAD [`a4c935ed74e4ce60fe1f4678a4dbbdb4a6f0175b`](https://github.com/july-rust/BoardSI/commit/a4c935ed74e4ce60fe1f4678a4dbbdb4a6f0175b).
- **Final exact HEAD checks:** **37/37 SUCCESS**; 0 failed/pending. Required Rust numerical/format checks, Qt QApplication and independent Worker (Linux offscreen + xcb, Windows x64/ARM64, macOS ARM64), applicable native Rust/Qt/field-workflow regressions actually completed. This confirms synthetic Qt/Worker states, not actual PCB physics or clean-host redistribution.
- **Independent PR review:** Codex review completed for the exact HEAD with **0 findings**, **0 unresolved findings**. An initial finding about the Qt failure diagnostic being overwritten was corrected before acceptance; final unresolved review threads **0**. Historical failed intermediate CI (Rust formatter and pre-existing cancel-race expectation) was corrected and is not relabeled PASS.
- **Protected merge:** Verified GitHub Squash [`c229eccdbbba41af5f4c4d622c7a1bb6421bdf02`](https://github.com/july-rust/BoardSI/commit/c229eccdbbba41af5f4c4d622c7a1bb6421bdf02), **single parent** `1ceaef92791f1fb4875b782c01ac0a6c1c976511`; verified GitHub signature, exact current code main ref confirmed.
- **Independent exact-code-main Push CI:** **8/8 workflows SUCCESS** at `main@c229eccdbbba41af5f4c4d622c7a1bb6421bdf02`, independently of the 37 PR checks. [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/42982428188), [Numerical core](https://github.com/july-rust/BoardSI/actions/runs/42982428141), [native FEM](https://github.com/july-rust/BoardSI/actions/runs/42982428389), [PH-07](https://github.com/july-rust/BoardSI/actions/runs/42982428186), [PH-06](https://github.com/july-rust/BoardSI/actions/runs/42982428116), [G0](https://github.com/july-rust/BoardSI/actions/runs/42982428140), [M2 isolated FEM](https://github.com/july-rust/BoardSI/actions/runs/42982428147), [docs](https://github.com/july-rust/BoardSI/actions/runs/42982428026). All recorded as completed/success; failed/cancelled/pending **0**.
- **Scoped E_005 closure:** **COMPLETE (2026-11-11 KST; native synthetic Worker post-ACK late-data and clean restart only)**. [Single authoritative checklist](../development-checklist.md#stage-e) records first check date and separate parent Issue #5 OPEN.

**Evidence boundary:** Real native CI executes regression harnesses, not a complete physical BoardSI Solver. The 37 PR checks and main Push workflows are distinct. Specific code and scope failures preserve their historical FAIL statuses, and `#5` stays OPEN for product-level acceptance.

## Exclusions and issue disposition

This is a bounded synthetic M0 transport/state-machine qualification, **not** full Issue #5, U22/U23/U44, T10/T34/T46/T50, heavy real PCB/solver scheduler, production snapshot/cache, real GPU behavior or product 30 GHz SI acceptance. Existing historical failures (including PH-08 Gate Q) and the macOS global timer question (#46/E_029) remain unchanged.

Issue #5 remains OPEN until its original full Definition of Done is independently met. Once this E_005 unit is accepted, other #5 obligations stay explicitly OPEN or BLOCKED with evidence and next action; do not close the parent issue just to reduce the count.
