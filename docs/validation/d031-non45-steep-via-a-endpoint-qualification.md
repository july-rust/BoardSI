# D_031 — Exact Steep 3:4 Non-45° Via-A SOURCE Endpoint Qualification

**Status:** D_031 SOURCE_GEOMETRY_ONLY code and initial-document acceptance PASS after exact code/docs main Push and Verified single-parent Squash. This final documentation update is subject to its own protected PR and post-main Push audit in Issue #434. Date: 2026-11-04 KST.

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
- Numerical Core/Linux and native Windows x64/ARM64, macOS ARM64 final PR jobs all SUCCESS. Exact [Numerical Core PR logs](https://github.com/july-rust/BoardSI/actions/runs/41802722398) show D_031 **Debug 3/3 PASS and Release 3/3 PASS**.
- Linux Qt Offscreen: **53/53 CTest PASS** and exact D031 `D031_NATIVE_QT_NON45_STEEP_VIA_A_SOURCE true ... NO_ELECTRICAL_PROOF`.
- Dedicated D031 Native Qt steps **Windows x64, Windows ARM64, macOS ARM64 SUCCESS**. All five full Qt runtime jobs and its final pull_request workflow SUCCESS.
- Same-final-HEAD **independent Codex review completed / has_any_findings=false / has_unresolved_findings=false**; zero unresolved review threads.
- A prior PR HEAD had `cargo fmt --check` FAIL; a later independent review detected **P2: Qt hostile geometry not independently reimported/validated**. Corrected by Qt reimport of five hostile original Sources, SHA- and TRACE-matched forged witnesses (1 tick gap, 1 tick angle, exact shallow 4:3, inward 3:4, wrong net), preserving the original failures and resolving the thread after a fresh no-findings review.
- Other fail-closed Rust tests: branch reversal, wrong Net/Layer/Via, extra/missing source objects, 45°/orthogonal/collinear/wrong endpoint, source SHA/receipt changes, no electrical promotion.

## Verified code-main Squash and independent Push — PASS

- Code PR #435 merged with a one-parent **Verified** Squash commit [2e5f203f82d6bf86846456bc8f89b85a50f5cbea](https://github.com/july-rust/BoardSI/commit/2e5f203f82d6bf86846456bc8f89b85a50f5cbea), parent 9b1fd0c50a52ecef8a66881d2abded190399aa21. The GitHub REST commit and git-commit endpoints independently returned `verification.verified=true`, `reason=valid`.
- Independently filtered exact `head_sha=2e5f203f82d6bf86846456bc8f89b85a50f5cbea`, `event=push` workflow and commit check-run REST records: **8/8 workflows SUCCESS; 28/28 check runs SUCCESS**.
- **Cancellation history preserved:** initial code-main Push [Numerical Core #41804760404](https://github.com/july-rust/BoardSI/actions/runs/41804760404) and [Qt runtime #41804760291](https://github.com/july-rust/BoardSI/actions/runs/41804760291) were initially CANCELLED after a subsequent docs-main Push. Their YAML uses `concurrency.group` keyed by `github.ref` and `cancel-in-progress: true`. Existing failed/cancelled jobs were re-run with unchanged original Push `head_sha`, no new workflow and no altered protected rule. Both **attempt 2 SUCCESS** (including native Windows x64/ARM64 and macOS ARM64 Qt). Initially successful jobs were retained.

## Verified separate docs-main Squash and Push — PASS

- Documentation-only [PR #436](https://github.com/july-rust/BoardSI/pull/436) exact final HEAD `03c268766e99abdeb4e012c7eebeea26ac6a8ee3`: **8/8 PR workflows SUCCESS**, independent Codex `completed/findings=false/unresolved=false`, 0 unresolved threads. Two Markdown files only; native compiler steps skipped.
- Protected one-parent **Verified** Squash docs-main commit [3b9d4e65b6b17f2406e03bd67d697053139cba90](https://github.com/july-rust/BoardSI/commit/3b9d4e65b6b17f2406e03bd67d697053139cba90), parent 2e5f203f82d6bf86846456bc8f89b85a50f5cbea. Both REST commit representations returned `verification.verified=true/reason=valid`.
- Exact `head_sha=3b9d4e65b6b17f2406e03bd67d697053139cba90`, `event=push`: **8/8 workflow runs SUCCESS**, **28/28 check runs normal (12 SUCCESS / 16 intentional docs-only SKIPPED)**. See [Numerical Core Push #41805554838](https://github.com/july-rust/BoardSI/actions/runs/41805554838) and [Qt runtime Push #41805554853](https://github.com/july-rust/BoardSI/actions/runs/41805554853).
- D_031 A_036 checklist is now checked **[x]** within this narrow evidence scope: Stage D **52 checked / 2 pending**, entire ledger **123 checked / 122 pending**. This final two-file documentation acceptance PR must independently pass its checks and review and merge via protected Verified Squash. Its exact docs-main Push proof is recorded on [Issue #434](https://github.com/july-rust/BoardSI/issues/434) after merging, **without another recursive docs PR**. Close #434 only after that final administrative gate passes; Issue #12 remains OPEN.

## Explicit exclusions

**NOT_VERIFIED:** fabricated copper, plated barrel electrical conduction, whole-board Open/Short, P009 / PH-08 Return Path, RLGC/S-parameters and physical solver, physical Scenario/Cache, full M1/M2 U/T, product release. Legacy PR #35 stays CLOSED/UNMERGED.
