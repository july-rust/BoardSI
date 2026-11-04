# D_029 — Exact 4:3 Non-45° Via-A Source Endpoint Contact Qualification

> **Scope:** `SOURCE_GEOMETRY_ONLY`. This is an independently reconstructed **candidate** contact, never manufactured copper proof, plated barrel conduction, a global Open/Short result, or cross-layer electrical continuity.

- **Tracker:** [Issue #427](https://github.com/july-rust/BoardSI/issues/427) — tracked for administrative closure after exact final closeout-docs main Push CI (do not pre-record Issue closure).
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

## 3. Protected code-main and documentary acceptance — independently verified (2026-11-04 KST)

### Code PR #428 and exact merged code SHA

- Final code PR HEAD `3f07ef8f0ee7ddcc7044f5ed3eb66acabcf698e0`: **34/34 PR Check Runs SUCCESS and 11/11 pull_request Workflows SUCCESS**. Independent same-HEAD Codex `completed` / `has_any_findings=false` / `has_unresolved_findings=false`, **0 unresolved threads**. [Code PR #428](https://github.com/july-rust/BoardSI/pull/428).
- Protected GitHub one-parent Squash: [`main@54d345bfa5c4a60eb8d7c0dc2e9c3be7e7aeacf6`](https://github.com/july-rust/BoardSI/commit/54d345bfa5c4a60eb8d7c0dc2e9c3be7e7aeacf6), sole parent `bb689a47685018c1ee4eba54412eb596f0489f94`; REST commit verification **`verified=true`, `reason=valid`**.
- Independently queried `event=push` at exact code-main SHA: **8/8 Push Workflows SUCCESS and 28/28 final GitHub Check Runs SUCCESS**. [Qt Runtime Push #41778597784](https://github.com/july-rust/BoardSI/actions/runs/41778597784), [Numerical Core Push #41778597746](https://github.com/july-rust/BoardSI/actions/runs/41778597746). Each returned run has exact `head_sha=54d345bfa5c4a60eb8d7c0dc2e9c3be7e7aeacf6`.
- **Historical cancellation preserved:** the first code-main Qt Runtime Push attempt was **CANCELLED** (3 Native Qt jobs), not SUCCESS. The existing workflow sets `concurrency.group: qt-runtime-${{ github.ref }}` and `cancel-in-progress: true`; later document-main push on the same `main` ref started during the earlier run. The cancellation timing is consistent with that concurrency policy; this is not evidence of a functional Qt defect. Invoked existing **re-run failed jobs**, resulting in attempt **2**, where Linux Qt 6.8.3 Offscreen and Windows x64 / ARM64 / macOS ARM64 Qt jobs all **SUCCESS**. Final Workflow `status=completed, conclusion=success, run_attempt=2`. No new native workflow family or relaxed check requirement.

### Separate initial documentation PR #429 and exact merged docs SHA

- [Docs PR #429](https://github.com/july-rust/BoardSI/pull/429), exact docs HEAD `5f7933a57175a10fcfb52a51dfc997b7c4c22a3d`: **8/8 pull_request Workflows SUCCESS**, independent same-HEAD Codex `completed`, findings=false, unresolved=false, **0 review threads**. Changed **only** `docs/development-checklist.md` and this ledger. Existing Docs-only classifier skipped native compiler jobs; required contexts were not weakened.
- Protected GitHub one-parent Verified Squash: [`main@96cfe39bdd794a2c526c6c610a6e4c52dfe87a5c`](https://github.com/july-rust/BoardSI/commit/96cfe39bdd794a2c526c6c610a6e4c52dfe87a5c), sole parent `54d345bfa5c4a60eb8d7c0dc2e9c3be7e7aeacf6`, `verified=true/reason=valid`.
- Independently queried exact docs-main SHA: **8/8 event=push Workflows SUCCESS**; **28/28 Check Runs completed (12 SUCCESS, 16 expected SKIPPED)**, preserving efficient Docs-only CI behavior. [Qt Docs Push #41779146552](https://github.com/july-rust/BoardSI/actions/runs/41779146552), [Numerical Docs Push #41779146592](https://github.com/july-rust/BoardSI/actions/runs/41779146592). Skip is not a compiler-test pass; true platform qualification came from the verified code PR and code-main Push runs.

### Final administrative closeout

This ledger upgrades the **bounded D_029 Source-only acceptance** from pending to checked only after the exact code-main rerun and initial docs-main independent Push verification. The final follow-up Markdown-only reconciliation PR itself must still satisfy exact-HEAD CI, independent Codex review, Verified one-parent Squash and exact post-merge Push CI. Record that *final PR* SHA and acceptance in [Issue #427](https://github.com/july-rust/BoardSI/issues/427); close the Issue only after its final Push CI success. Do **not** recursively create documentation PRs solely to insert a later administrative commit SHA into this file.

## 4. Explicit exclusions

**NOT VERIFIED / NOT ACCEPTED:** actual manufactured copper or plated Via barrel electrical conduction, board-wide Open/Short, P009 Reference Return Path, PH-08, RLGC / S-parameter solver, physical Scenario/Cache, formal M1/M2 U/T acceptance, and production release. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN; legacy [PR #35](https://github.com/july-rust/BoardSI/pull/35) remains CLOSED/UNMERGED.
