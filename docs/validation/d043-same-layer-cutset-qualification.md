# D_043 — Same-Layer SOURCE Virtual Trace Cut-Set / Redundancy Qualification

**Status:** source-local code and exact code-main Push verified **2026-11-06 KST**. Separate docs-only PR status and final Issue closure evidence belong in [Issue #472](https://github.com/july-rust/BoardSI/issues/472), not a recursive documentation PR. The parent [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.

## 1. Scope and original-source preservation

The baseline is the public synthetic [D_042 PADS V9.4 MILS fixture](../../crates/boardsi-import/tests/fixtures/d042_local_closed_loop_v94.asc), 254 nm per Canonical Source Tick, Layer 1, NET_A (NET_B lexical regression). It contains four SMD Pads and ten straight Trace segments with 14 geometry Nodes, 14 local candidate Edges, one local Component and one independent Cycle. D_042 independently validates a complete original SOURCE with exact owner/Pin-Pair/Net/Layer/width/geometry and SHA-256 evidence **before** D_043 may run.

D_043 preserves the full SOURCE bytes and SHA. A separate **counterfactual graph diagnostic** removes only selected Trace Nodes and all corresponding incident Edges; it must never represent a physically missing SOURCE record as an authorized user-selected cut. Trace Tn maps to node `4+n`; the Pad nodes remain. This bounded graph model does **not** certify manufactured Copper damage or continuous electrical conductivity.

## 2. Exact scenarios and computed topology

| Scenario | Excluded Traces | Nodes | Edges | Components | Cycles | U1/U2 local reachability |
| :--- | :--- | ---: | ---: | ---: | ---: | :--- |
| Baseline | none | 14 | 14 | 1 | 1 | YES |
| Single direct cut | T4 | 13 | 12 | 1 | 0 | YES |
| Single detour cut | each T5, T6, T7, T8, T9 separately | 13 | 12 | 1 | 0 | YES |
| Double cut | each T4+T5 through T4+T9 separately | 12 | 10 | 2 | 0 | NO |

Total **12 executable distinct bounded scenarios** (1+6+5). The Graph Union-Find/Traversal computes active-node root count, edge count, independent cycle count, and reachability between Pads U1/U2; `E−V+C` independently cross-checks cycle rank. The expected values are asserted in tests but **not hard-coded as algorithm return values**. U1 local Pad pair and U2 local Pad pair must each remain connected.

Excluded Trace identifiers are limited to exact single T4..T9, or ordered double [T4,T5..T9]; invalid, duplicated, unsupported, or out-of-order identifiers are rejected, as are malformed tokens (including leading zeros).

## 3. Implementation and explicit fail-closed

- [Rust `d043_local_cutset.rs`](../../crates/boardsi-import/src/d043_local_cutset.rs) reconstructs and validates D_042 receipt and original SOURCE, parses the existing 14 edges, removes selected Trace Nodes and incident Edges, and computes components, cycles and reachability by graph traversal. Rust test [`d043_local_cutset.rs`](../../crates/boardsi-import/tests/d043_local_cutset.rs) covers all 12 cases, deterministic receipt, wrong source/owner/geometry, invalid ID/cut, reversed SOURCE Trace orientation, NET_B, CRLF, and supported header variant.
- Opt-in offline CLI `boardsi-import-artwork FILE --d043-local-trace-cutset-v1 [T4 [T5]]` produces deterministic SHA-bound `BOARDSI_D043_LOCAL_CUTSET_V1` diagnostic output only; it does not change normal Canonical Solver/Viewer/Project paths.
- Native Qt [`d043_local_cutset_qt.h`](../../apps/boardsi-qt-prototype/d043_local_cutset_qt.h) independently requires the full D_042 Qt `ImportedArtwork` and SOURCE Pin-Pair/Owner/SHA/freshness verification, then performs its **own** graph computation. It compares a byte-identical receipt against Rust rather than trusting Rust stdout. The native [runner](../../apps/boardsi-qt-prototype/d043_qt_runner.cpp) rejects forged metric receipt, stale imported SHA, incomplete SOURCE, owner swap, unsupported cut, duplicate and reversed order; includes all 12 positive scenarios.
- All accepted receipts explicitly retain `SOURCE_LOCAL_COUNTERFACTUAL_ONLY`, `P009_UNSUPPORTED`, `SI_NOT_RUN`, and `GLOBAL_ELECTRICAL_NOT_VERIFIED`; final marker `SOURCE_LOCAL_DIAGNOSTIC_NO_PHYSICAL_OPEN_SHORT`.

## 4. Exact verified code integration and platform evidence

- [Code PR #473](https://github.com/july-rust/BoardSI/pull/473), exact final HEAD [`ca46c3be13db0d88380d2cbc89c73c7921fa57ab`](https://github.com/july-rust/BoardSI/commit/ca46c3be13db0d88380d2cbc89c73c7921fa57ab/checks): **34/34** pull_request Check Runs and **11/11** pull_request Workflows SUCCESS. Rustfmt/Clippy warnings denied, Native Qt and Rust Debug/Release; dedicated Linux CTest **65/65 PASS**, including `qt-preview-d043-local-trace-cutset-source` and prior regressions; native Windows x64, Windows ARM64 and macOS ARM64 jobs SUCCESS.
- Independent Codex review exactly on final HEAD `completed/has_any_findings=false/has_unresolved_findings=false`; open review threads 0. Historical intermediate Rust type error, Rustfmt and Clippy failures remain documented and were fixed before the final HEAD passed; do not reclassify past failures.
- Protected, **Verified**, GitHub one-parent Squash merged [`main@4dd458e5670dcb8fafd705c233bfedc25adfd517`](https://github.com/july-rust/BoardSI/commit/4dd458e5670dcb8fafd705c233bfedc25adfd517), sole parent `528c67fdb632b26c35f9ba956e582d5ca89e0a4e`. GitHub REST `verification.verified=true/reason=valid`, precise main-ref equality.
- Exact code-main commit's separate actual `event=push` verification: **8/8 Actions Workflows SUCCESS and 28/28 Check Runs SUCCESS**, with no pending, failed or skipped checks. (Do not conflate PR checks with push checks.)

## 5. Documentation policy and explicit non-acceptance

Only [canonical checklist](../development-checklist.md) and this Markdown validation ledger change in the separate docs-only PR. Stage D bounded checks **63 → 64**, total bounded checks **134 → 135**; Stage D pending **2** and total pending **122** remain unchanged. Keep the established docs-only classifier/17 status-context contract and bypass unnecessary Native platform build allocation where supported. The code PR is done, but the documentation PR requires separate same-HEAD CI, independent review, protected Verified Squash, and exact docs-main push acceptance; place final evidence/Issue closure in Issue #472, not in a third Markdown-only PR.

**NOT_VERIFIED:** physical Trace damage and manufactured copper conductivity, global electrical Open/Short, arbitrary PCB Copper/Via/Plane/Pour/Thermal or cross-layer continuity, P009/PH-08 return path, immutable physical Scenario/Cache readiness, RLGC/S-parameter simulation, formal U01–U48/T01–T54 full product U/T, and product release. This single bounded checklist item is **not a percentage of product completion**. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.
