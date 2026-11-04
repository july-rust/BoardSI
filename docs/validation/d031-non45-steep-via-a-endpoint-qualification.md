# D_031 — Exact Steep 3:4 Non-45° Via-A SOURCE Endpoint Qualification

**Status:** bounded implementation merged and exact code PR gates verified; **final administrative acceptance PENDING** independent code-main Push verification and documentation acceptance. Date: 2026-11-04 KST.

**Scope:** `SOURCE_GEOMETRY_ONLY`. This ledger certifies a geometrically reconstructed layer-local Source contact *candidate*, not fabricated conductor, plated Via barrel conduction, physical reference return, electrical Open/Short or any SI solver capability.

- Issue: [#434](https://github.com/july-rust/BoardSI/issues/434).
- Code PR: [#435](https://github.com/july-rust/BoardSI/pull/435).
- Final code PR HEAD: [`6381e86d8671a557fe80f8b85e0e0e5c7e48d2ad`](https://github.com/july-rust/BoardSI/commit/6381e86d8671a557fe80f8b85e0e0e5c7e48d2ad).
- Protected Squash main commit: [`2e5f203f82d6bf86846456bc8f89b85a50f5cbea`](https://github.com/july-rust/BoardSI/commit/2e5f203f82d6bf86846456bc8f89b85a50f5cbea), **one commit ahead** of the prior main baseline `9b1fd0c50a52ecef8a66881d2abded190399aa21` based on independent commit comparison.

## Frozen Source geometry

- PADS Layout V9.4 ASCII in MILS; one source tick exactly 0.01 mil = **254 nm**.
- Net `NET_A`, Layer 4, Via-A `(15000,25000)`.
- Main Trace `(15000,25000)→(35000,25000)`.
- Branch `(15000,25000)→(6000,37000)` and exactly reversed Branch direction; displacement `(-9000,+12000)`.
- Signed `i128` exact `3*abs(cross)==4*abs(dot)`, nonzero cross and dot, shared Via-A endpoint and strict exterior projection `<0`. No floating-point or one-tick tolerance.
- Fixture: [`d031_non45_steep_via_a_endpoint_v94.asc`](../../crates/boardsi-import/tests/fixtures/d031_non45_steep_via_a_endpoint_v94.asc).
- Explicit CLI: `--d031-non45-steep-via-a-endpoint-contact-v1 NET_A 1 4 0 1`.
- Independent Rust and Qt witness; expected and qualified **8 Source Nodes / 6 same-layer candidate Edges / 3 local Components / 0 proven electrical cross-layer Edges**.

## Exact code PR evidence — PASS

- **11/11 final HEAD pull_request workflows SUCCESS** on `6381e86d8671a557fe80f8b85e0e0e5c7e48d2ad`. [Numerical core Actions #41802722398](https://github.com/july-rust/BoardSI/actions/runs/41802722398); [Qt runtime Actions #41802722396](https://github.com/july-rust/BoardSI/actions/runs/41802722396).
- Numerical Core/Linux and native Windows x64/ARM64, macOS ARM64 all final jobs SUCCESS. D_031 Rust 3/3 dedicated Debug/Release tests require per-profile log confirmation; **profile-specific count NOT independently transcribed here**.
- Linux Qt Offscreen: **53/53 CTest PASS** and exact D031 `D031_NATIVE_QT_NON45_STEEP_VIA_A_SOURCE true ... NO_ELECTRICAL_PROOF`.
- Dedicated D031 Native Qt steps **Windows x64, Windows ARM64, macOS ARM64 SUCCESS**. All five full Qt runtime jobs and its final pull_request workflow SUCCESS.
- Same-final-HEAD **independent Codex review completed / has_any_findings=false / has_unresolved_findings=false**; zero unresolved review threads.
- A prior PR HEAD had `cargo fmt --check` FAIL; a later independent review detected **P2: Qt hostile geometry not independently reimported/validated**. Corrected by Qt reimport of five hostile original Sources, SHA- and TRACE-matched forged witnesses (1 tick gap, 1 tick angle, exact shallow 4:3, inward 3:4, wrong net), preserving the original failures and resolving the thread after a fresh no-findings review.
- Other fail-closed Rust tests: branch reversal, wrong Net/Layer/Via, extra/missing source objects, 45°/orthogonal/collinear/wrong endpoint, source SHA/receipt changes, no electrical promotion.

## Bounded code merge — confirmed, but independent post-main gate pending

- Code PR #435 was merged using **Squash**. GitHub returned merged=true, SHA `2e5f203f82d6bf86846456bc8f89b85a50f5cbea`; a separate compare of pre-code-main and this SHA returned exactly **1 commit**, with **9 changed files**.
- **NOT VERIFIED:** this exact merged code-main SHA's independent `event=push` workflow/check run outcomes and the Git commit's `verified=true/reason=valid` status. The connected GitHub workflow history endpoint reports pull_request event runs only; do **not** substitute PR jobs for post-main Push jobs or claim signature verification from a squash label alone.

## Separate documentation-only closeout

This ledger and `docs/development-checklist.md` are the only documentation changes intended for the separate PR. Existing docs-only CI classifier must avoid native compiler allocation; all 17 protected required contexts remain intact. Final documentation PR exact-HEAD checks, independent Codex review, zero review threads, protected squash, independent exact docs-main Push, and the separate outstanding code-main Push/signature evidence must be verified before marking D_031 checked and closing Issue #434.

Keep Stage D D_031 **unchecked (0/1)** and parent Issue #12 **OPEN** while those gates remain unconfirmed. Do not create endlessly recursive administrative PRs; closeout verification SHA should be recorded on Issue #434 after independent evidence is obtained.

## Explicit exclusions

**NOT_VERIFIED:** fabricated copper, plated barrel electrical conduction, whole-board Open/Short, P009 / PH-08 Return Path, RLGC/S-parameters and physical solver, physical Scenario/Cache, full M1/M2 U/T, product release. Legacy PR #35 stays CLOSED/UNMERGED.
