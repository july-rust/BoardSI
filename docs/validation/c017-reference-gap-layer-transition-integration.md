# C_017 — Bounded Reference Gap / Layer Transition V1–V4 Integration

**Tracker:** [Issue #287](https://github.com/july-rust/BoardSI/issues/287) · **Baseline:** `main@345f25f15f607a6d28f20a46781a66e3558a71f8` · **Date:** 2026-10-30 KST.

**Status: IN PROGRESS — not yet independently accepted.** PR-head numerical/Qt CI, same-head Codex review, protected Verified merge and post-main checks must be recorded from actual results; this document does not pre-certify them.

## Scope and non-goals

This is intentionally **one bounded integration regression**, not a fresh B/C-series full audit. It preserves existing C_005 V1 topology risk, C_006 V2 exact rational gap intervals, C_007 V3 connected route segments, and C_008 V4 two-layer plated transition. No new geometry or source revisions, manufacturing clearance/DRC, electrical Reference Current P009, RLGC/S-parameters or physical scenario/cache are introduced.

`crates/boardsi-import/tests/c017_reference_gap_transition_integration.rs` adds four source-only cases:

1. On a single shared PADS V9.4 MILS synthetic source, verify V1/V2/V3 distinct opt-in headers, identical exact source SHA, V3's verbatim V2 per-segment receipts, explicit gap risk and physical UNSUPPORTED/NOT_RUN.
2. Mutate one source trace coordinate and verify source hashes differ across V1/V2/V3; no old receipt can be treated as the new source identity.
3. On a two-layer plated Via synthetic source, verify V4's exact V2 receipts for separate selected Reference Nets/Layers, explicit transition-unverified marker and gap risk.
4. Reimport a changed Via source or select a missing Reference Net: changed hash or UNSUPPORTED, never a physical PASS.

The new cross-version Rust test is exercised in existing `.github/workflows/numerical-core.yml` workspace debug/release and native matrices, **only if the current PR CI succeeds**. Existing Qt C_005/C_007/C_008 native smoke and Linux offscreen CTest are reused, not replaced or claimed as newly implemented Qt tests.

## Evidence / gating record

| Gate | Current state | Evidence |
|---|---|---|
| Pre-change main baseline | Observed | [`main@345f25f`](https://github.com/july-rust/BoardSI/commit/345f25f15f607a6d28f20a46781a66e3558a71f8) |
| Existing C_005 Qt source danger+stale | Prior limited proof only | [C_005 ledger](c005-qt-reference-risk-integration.md), [PR #249](https://github.com/july-rust/BoardSI/pull/249), [Native follow-up PR #250](https://github.com/july-rust/BoardSI/pull/250) |
| Existing C_007 Qt route risk | Prior limited proof only | [C_007 ledger](c007-multi-segment-reference-coverage.md), [PR #256](https://github.com/july-rust/BoardSI/pull/256) |
| Existing C_008 Qt Via layer transition | Prior limited proof only | [C_008 ledger](c008-multilayer-reference-via-transition.md), [PR #259](https://github.com/july-rust/BoardSI/pull/259) |
| C_017 test new exact-head debug/release | **PENDING** | Add final PR HEAD and checks here |
| Qt Native Win x64/ARM64/macOS ARM64 & Linux CTest | **PENDING** | Record which jobs actually ran |
| Same-head independent Codex review | **PENDING** | No prior-head review substitution |
| Protected signed Squash / post-main CI | **PENDING** | Record real merge SHA + push runs |

## Historical C_002 Qt checkbox reconciliation

The initial C_002 PR #240 was **Rust-only**, thus its original 2026-10-28 unchecked Qt marker row was correct at that time. Later C_005 PRs #249/#250 implemented and tested bounded Reference Split / Layer Transition source-risk Qt markers and stale handling; C_008 PR #259 added a two-layer Via transition with Native Qt tests. The checklist may mark **that specific later source-bound Qt scope** complete with first check date 2026-10-30 KST and original evidence dates, *without reclassifying the C_002 Rust PR itself as a Qt change or passing full electrical Reference or P009*.

## Retained unsupported boundaries

- Only the explicitly supported synthetic source geometry, PADS V9.4 MILS subset and source-exact selected objects are covered.
- Qt generation/reimport/stale-result behavior depends on existing Native Qt selftests; a Rust source SHA comparison alone is **not** an asynchronous Qt acceptance.
- `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`.
- Parent [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN. Formal U09/U10/U39 and T02/T04/T24/T52 are not checked.
- C_009–C_016 multi-cutout/pad changes belong to C_018/C_019, not this issue. B_007 review reconciliation belongs to C_020.

## Final disposition

**NOT YET ACCEPTED.** Record the actual PR-head workflow/check outcomes, latest-head independent review, the verified protected merge and post-merge CI before changing the C_017 checklist to `[x]`. Any unmet gate remains explicitly unchecked.
