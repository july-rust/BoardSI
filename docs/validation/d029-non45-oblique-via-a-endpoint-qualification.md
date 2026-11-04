# D_029 — Exact 4:3 Non-45° Via-A Source Endpoint Contact Qualification

> **Scope:** `SOURCE_GEOMETRY_ONLY`. This is an independently reconstructed **candidate** contact, never manufactured copper proof, plated barrel conduction, a global Open/Short result, or cross-layer electrical continuity.

- **Tracker:** [Issue #427](https://github.com/july-rust/BoardSI/issues/427) — remain OPEN until full documentary acceptance.
- **Code:** [PR #428](https://github.com/july-rust/BoardSI/pull/428) — merged on 2026-11-04 KST.
- **Exact final code PR HEAD:** `3f07ef8f0ee7ddcc7044f5ed3eb66acabcf698e0`.
- **GitHub squash merge SHA:** [`54d345bfa5c4a60eb8d7c0dc2e9c3be7e7aeacf6`](https://github.com/july-rust/BoardSI/commit/54d345bfa5c4a60eb8d7c0dc2e9c3be7e7aeacf6).
- **Recorded date:** 2026-11-04 KST.
- **Public fixture:** [PADS V9.4 MILS D_029](../../crates/boardsi-import/tests/fixtures/d029_non45_oblique_via_a_endpoint_v94.asc).
- **Explicit CLI opt-in:** `--d029-non45-oblique-via-a-endpoint-contact-v1 NET_A 1 4 0 1`.

## 1. Frozen, bounded Source geometry

PADS V9.4 MILS source coordinates use exact `0.01 mil = 254 nm` integer ticks. `NET_A`, layer 4, Main Trace `(15000,25000) → (35000,25000)`, Branch Trace `(15000,25000) → (3000,34000)`; same coordinate at the **Via-A** pad `(15000,25000)`. Reversed Branch point order remains valid.

The Branch exterior has relative displacement `(-12000,+9000)` ticks, exactly 4:3, non-45°; Rust uses signed `i128` dot/cross, `dot != 0`, `cross != 0`, `4*abs(cross) == 3*abs(dot)`, and `exterior_projection < 0` beyond Via-A. No floating-point tolerance or inferred physical contact.

The qualified **source candidate** witness format contains **8 Source Nodes / 6 Layer-local Candidate Edges / 3 Local Components / 0 proven electrical cross-layer edges**; the sixth edge is **Via-A Layer-4 Pad node 1 ↔ Branch node 7**, never Via-B node 2. Distinct edge type `SOURCE_NON45_OBLIQUE_VIA_A_ENDPOINT_CONTACT_CANDIDATE`; `OVERALL|NON45_OBLIQUE_VIA_A_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY`; `END|FAIL_CLOSED_NO_BARREL_PROOF`.

Qt independently reimports and reconstructs exact canonical Source and SHA-bound receipt, not trusting the Rust output alone. Reject one-tick endpoint gaps, offsets or angle changes; wrong Net/Layer/Via; 45°/90°/collinear and D_028 Via-B cross-mode; malformed/incomplete Source; extra conductive objects; stale/forged wire; and false electrical promotion. Existing D_015–D_028 behavior remains separate.

## 2. Confirmed code PR validation

- At exact final code PR HEAD `3f07ef8f0ee7ddcc7044f5ed3eb66acabcf698e0`, **11/11 GitHub pull_request Workflows SUCCESS**: Qt runtime, Numerical core, F6/F7, A032, planning-doc checks and M2/PH regression workflows. [PR #428](https://github.com/july-rust/BoardSI/pull/428).
- Rust `tests/d029_non45_oblique_via_a_endpoint.rs` **3/3 Debug PASS and 3/3 Release PASS**, including reverse Branch, lexical SHA binding, one-tick hostiles and cross-mode rejection; evidenced in [Numerical Workflow #41777287021](https://github.com/july-rust/BoardSI/actions/runs/41777287021).
- Linux Qt 6.8.3 offscreen **51/51 CTest PASS**, including `qt-preview-d029-non45-oblique-via-a-endpoint-source` and `D029_NATIVE_QT_NON45_OBLIQUE_VIA_A_SOURCE true ... NO_ELECTRICAL_PROOF`. [Qt Workflow #41777287248](https://github.com/july-rust/BoardSI/actions/runs/41777287248).
- Native Qt D_029 Source smoke step verified **SUCCESS** on Windows x64, Windows ARM64, and macOS ARM64 in the same Qt Workflow; all jobs and the complete Qt Workflow SUCCESS.
- Independent Codex code review on the **same final HEAD**: `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; unresolved review threads **0**. [Review summary](https://github.com/july-rust/BoardSI/pull/428#issuecomment-6623071078).
- Initial superseded code HEAD had real Rustfmt and dead-code warnings. They were corrected before the final HEAD checks. Do not erase failed-history provenance.

## 3. Code merge and evidence still pending

- GitHub reported PR #428 `merged=true`, method `squash`, commit `54d345bfa5c4a60eb8d7c0dc2e9c3be7e7aeacf6`.
- **NOT YET INDEPENDENTLY VERIFIED IN THIS RECORD:** exact merged commit signature `verified=true`, sole-parent ancestry, exact SHA post-code-main `event=push` workflow/check-run acceptance. The available connected workflow listing is limited to PR event runs; do not infer Push results from successful PR jobs.
- **Separate Markdown-only documentation PR** is restricted to this ledger and `docs/development-checklist.md`; existing docs-only CI classifier must avoid native compiler jobs while retaining required check contexts.
- Before completion: independently verify exact code-main Push CI and signature/parent, exact docs PR HEAD checks, same-HEAD Codex review, zero unresolved threads, protected docs Squash and exact docs-main Push CI. Only then mark D_029 complete and close Issue #427.

## 4. Explicit exclusions

**NOT VERIFIED / NOT ACCEPTED:** actual manufactured copper or plated Via barrel electrical conduction, board-wide Open/Short, P009 Reference Return Path, PH-08, RLGC / S-parameter solver, physical Scenario/Cache, formal M1/M2 U/T acceptance, and production release. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN; legacy [PR #35](https://github.com/july-rust/BoardSI/pull/35) remains CLOSED/UNMERGED.
