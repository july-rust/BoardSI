# C_025_04 — Legacy PR #153 / #223 reconciliation and safe closure

**KST date:** 2026-10-31. **Baseline:** protected [main@d428ceef076c484a61aad0b72c49bb574faedbb4](https://github.com/july-rust/BoardSI/commit/d428ceef076c484a61aad0b72c49bb574faedbb4). This is a **historical PR reconciliation**, not a code feature or M0/M1/M2 product acceptance.

## Pre-change evidence and disposition

| Legacy PR | Exact head (read before action) | Comparison and safe decision |
| --- | --- | --- |
| [#153](https://github.com/july-rust/BoardSI/pull/153) | `caf275c991ca0f77abfb13975e54d9ec4a0185f1` | OPEN, unmerged. Its new `docs/preview-delivery-playbook.md` did not exist on main; closing without preserving current policy would drop useful decisions. Carry policy into the new playbook and project-management link, **updated for completed P0 #152 and current A/B/C work**. Retain dual M0/Preview gates, conservative evidence, licensing/IP, retry and GitHub protection policy. No source PR direct merge. |
| [#223](https://github.com/july-rust/BoardSI/pull/223) | `2dd2931754dfe1d9ff5e792543ef3c4104350228` | OPEN Draft, unmerged. Reconcile against [merged #224](https://github.com/july-rust/BoardSI/pull/224) exact head `e211b2e626afd89ac7a880eaeace40e91c8abc50` and protected signed squash `1c11d08be6e7c83d3f6b697f515f61a8c1a43ba8`. The later implementation is not byte-identical and is **not** a justification for silently claiming identical test coverage. Do not merge older branch. |
| [#35](https://github.com/july-rust/BoardSI/pull/35) | `358d74e447769cf1b3e508786a6964895658f7c6` | OPEN Draft G5 design review. **Untouched; not in closure scope**. |

## #223 versus #224 — scoped equivalence table

| Earlier #223 candidate | Current #224/main evidence | Disposition |
| --- | --- | --- |
| Orthogonal straight Trace centreline interior T-contact, source ticks | `crates/boardsi-import/src/connectivity_preview.rs` includes indexed horizontal/vertical source-grid witnesses and near-miss tests | Implemented in later B_003 limited scope. Not general body/width overlap. |
| Plated Through-Via centre and two independent conductive-layer local contacts | `connectivity_preview.rs` has layer-indexed Via candidate selection, plated Via barrel gating and unit coverage | Implemented in later limited scope, not all Via/anti-pad/thermal cases. |
| Canonical exact collinearity and strict interior witness verification with overflow handling | `crates/boardsi-model/src/validation.rs`: `ContactSite::TraceInterior(Point)` and checked i128 arithmetic; note different enum shape from older #223 | Functional purpose carried forward; not a byte-for-byte transplant. |
| Old Qt V1 endpoint-only output and audit count consistency | `crates/boardsi-import/src/contact_wire.rs`, existing CLI compatibility tests, and #224 P1 review remediation | Endpoint-only transport retained, but **do not assert identical CLI test case coverage**. |
| #223's added `a032_contact_wire_cli.rs::b003_canonical_interior_edges_do_not_break_legacy_qt_audit_counts` and two standalone `b003_*_v94.asc` fixtures | At baseline current `a032_contact_wire_cli.rs` tests V10–V14 opt-in compatibility but does **not** contain that exact process-level B_003 T/Via test or those standalone fixtures; #224 contains other importer unit regressions | **Unique process-level regression remains unported / NOT_VERIFIED as equivalent.** Track in open [Issue #12](https://github.com/july-rust/BoardSI/issues/12) as optional focused regression gap; do not revive/merge old candidate or claim test PASS. |

Source [B_003 acceptance ledger](b003-b001-reference-topology.md) confirms limited accepted #224 functionality only. No whole-board Open/Short, physical Reference Return Path or solver readiness follows from exact geometry.

## Checklist and closure procedure

Pre-task `docs/development-checklist.md` was read on 2026-10-31 KST at main `d428ceef`; B_003 bounded [x] is already dated 2026-10-27 KST. **Do not alter its first-check date**. Formal U09/U10/U39 and T02/T04/T24/T52 remain unchecked; Issue #12 stays OPEN. This policy/audit PR does not add a new product functionality checkbox.

1. Preserve the reconciled #153 policy on protected main through normal Verified squash / required checks / review.
2. Add the explicit CLI process-regression coverage delta to open Issue #12; no claim of code regression test execution in this docs-only task.
3. Explain the exact replacement and follow-up links on legacy PRs; then close #153 and #223 **without merging either**.
4. Independently read both PR states and #35 Draft status, inspect checklist before/after, and record protected main / CI and review evidence separately. Until observed, closure and CI **are pending**, not PASS.

**Current acceptance:** READ-ONLY RECONCILIATION COMPLETE; GitHub PR closure and protected merge remain PENDING until independently confirmed. Product scope and U/T acceptance unchanged.
