# E_002 — M0 F1 U/T traceability status reconciliation

**Date:** 2026-11-11 KST. **Inspected protected baseline:** `main@360da3a258e824ec94ce4121548f8368bf56c364`. **Scope:** one bounded documentation / fail-closed validation contract, **not M0 F1 or formal T01–T54 product acceptance**.

## Input and existing immutable evidence

- [Frozen BoardSI v1.0](../specification/BoardSI_Development_Spec_v1.0.md): 48 defined U IDs (U01–U48), 54 defined T IDs (T01–T54); §18.2 links broad stage groups and cannot certify individual cases.
- [M0 U/T ledger](../audits/m0-requirements-traceability-ledger.md): no claimed ID-count or reciprocal mapping deficit in previously [C_025_05 independently documented inspection](c02505-m0-f1-traceability-reconciliation.md). T44 remains attached to U01.
- [Pinned ten-case source evidence](m0-case-evidence.json): immutable source `2a08c167d89279e15f1e65b689a58ed0682afc6e`, 8 restricted `PARTIAL` and 2 `NOT_RUN` prototype statuses; **all 10 formal full T acceptance values `NOT_RUN`**, all individual case reviews `NOT_PERFORMED`. This signed historical JSON is intentionally unchanged.

## Exact discrepancy and correction

The T ledger column explicitly titled `Complete acceptance result` incorrectly carried the **prototype** value `PARTIAL` in three cases. This is a status-domain conflict, not proof of a failed original test or absence of all prototype observations.

| Test | Before: complete acceptance column | After: complete acceptance column | Preserved bounded evidence |
| :--- | :--- | :--- | :--- |
| **T13** | `PARTIAL` | `NOT_RUN` | Native Qt/Worker on target platforms (see #3/#6); full Unicode/locale/product test not accepted |
| **T39** | `PARTIAL` | `NOT_RUN` | Native Qt/Worker restricted prototype (see #3/#6); same real board / complex S comparison not accepted |
| **T46** | `PARTIAL` | `NOT_RUN` | Restricted IPC / worker snapshot contract (see #5/#46); complete production test not accepted |

The existing `M0 native Qt/Worker prototype only; ...; full T unqualified` evidence wording remains **unchanged in those T rows**, so a reader can still distinguish bounded execution from complete acceptance. Nothing turns previous PASS into FAIL, or prior historical FAIL into PASS. No other U/T status changes are made.

## Regression contract

The [existing strict coverage validator](../../tools/validation/m0_traceability_check.py) continues to check all 48/54 IDs, reciprocal stage mapping, Markdown table source visibility and T44↔U01 semantic invariants. **New E_002 fail-closed assertions:**

1. Every **T** row in the `Complete acceptance result` column must be `NOT_RUN` until a separately reviewed formal acceptance schema is implemented.
2. T13, T39, T46 `Evidence` cells must preserve **their own** exactly approved restricted prototype wording; generic or transplanted `PARTIAL` evidence is rejected.
3. Per-case `PARTIAL` or `PASS` promotion, or an altered `FULL T ACCEPTANCE PASS` evidence narrative must be rejected by explicit adversarial mutations.
4. Existing U implementation `PARTIAL` remains allowed **only with its exact per-row source wording**, not as a formal T completion state.
5. The independent immutable ten-case evidence validator still enforces historical JSON integrity and must pass.

**Execution results / final PR HEAD / independent review / protected Verified Squash / exact post-main Push CI must be recorded after GitHub execution in the final [Issue #2](https://github.com/july-rust/BoardSI/issues/2) comment.** This document cannot pre-assert unrun CI.

## Exit boundary and source of unresolved work

This finishes only **E_002 status-domain correction and regression protection**, contingent on its required CI and protected merge. **M0 F1 and Issue #2 remain open** until individual M0-scope cases receive expected vs observed results, fixtures, exact job and commit evidence, plus genuinely independent case-specific approval. The [EB-07 dependency](../e-stage-blocked-register.md) tracks that explicit reopening gate. F6/F7, macOS #46, PH-08 Gate Q FAIL, production solver and formal T01–T54 acceptance are not promoted or waived.

E_003 may proceed independently after the E_002 bounded PR is merged and its exact new main checked. No redundant document-only follow-up PR is part of this acceptance contract.
