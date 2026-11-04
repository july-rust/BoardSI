# D_028 — Exact 4:3 Non-45° Via-B Source Endpoint Contact Qualification

- **Scope:** `SOURCE_GEOMETRY_ONLY`; this is not evidence of manufactured/plated copper, Via barrel conduction, return current or cross-layer electrical continuity.
- **Tracker:** [Issue #424](https://github.com/july-rust/BoardSI/issues/424), open until documentation acceptance and exact docs-main Push CI.
- **Code:** [PR #425](https://github.com/july-rust/BoardSI/pull/425), protected Verified single-parent Squash already MERGED.
- **First recorded date / code qualified:** 2026-11-04 KST.
- **Public synthetic fixture:** [D_028 PADS V9.4 MILS Source](../../crates/boardsi-import/tests/fixtures/d028_non45_oblique_via_b_endpoint_v94.asc).
- **Opt-in CLI:** `--d028-non45-oblique-via-b-endpoint-contact-v1 NET_A 1 4 0 1`.

## 1. Frozen bounded Source geometry

Exact PADS V9.4 MILS `0.01 mil = 254 nm` integer Source coordinate ticks; Layer 4, NET_A; Main Trace `(15000,25000) -> (35000,25000)`, Branch Trace `(35000,25000) -> (47000,34000)`, Via-B Pad `(35000,25000)`. Branch endpoint reversal is allowed. The positive main-projection branch has `dx=12000`, `dy=9000` ticks (exact non-45° 4:3 horizontal-to-vertical ratio; angle approximately 36.87°).

The bounded Rust mode independently checks `dot != 0`, `cross != 0`, `4 * abs(cross) == 3 * abs(dot)` using `i128` and strict `exteriorProjection > mainLengthSquared` beyond Via-B; no floating-point tolerance. Qt independently reconstructs geometry from the actually imported Source, verifies layer/net/Via-B and SHA-256-bound exact wire, and refuses stale/forged/foreign/electrically promoted evidence.

**Actual qualified Source candidate graph:** 8 Nodes, 6 same-layer Candidate Edges, 3 local Connected Components, **0 proven electrical cross-layer edges**. Distinct `SOURCE_NON45_OBLIQUE_VIA_B_ENDPOINT_CONTACT_CANDIDATE` Edge 4 (Main Trace node 5 to Branch Trace node 7) and Via-B Layer-4 Pad node 2 to Branch Trace node 7 at Edge 5. `OVERALL|NON45_OBLIQUE_VIA_B_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY`; `END|FAIL_CLOSED_NO_BARREL_PROOF`.

Fail closed for 1-tick gap/offset/angle drift, 45°/orthogonal/collinear wrong mode, wrong Via-A/Net/Layer, degenerate/additional/ambiguous Source, unsupported geometry, stale/forged/cross-mode Source-SHA Receipt and false electrical promotion. Legacy D_015–D_027 remains separate.

## 2. Exact-head code CI and independent review

- Final code PR [#425](https://github.com/july-rust/BoardSI/pull/425) HEAD [`8f6b690984be17be8392e0dc2102d8fd59e981b1`](https://github.com/july-rust/BoardSI/commit/8f6b690984be17be8392e0dc2102d8fd59e981b1/checks): **34/34 PR Check Runs SUCCESS, 11/11 pull_request Workflows SUCCESS**; existing 17 required strict Ruleset contexts retained.
- Rust `tests/d028_non45_oblique_via_b_endpoint.rs`: **3/3 tests PASS** in Debug and **3/3 PASS** in Release, negative/angle/cross-mode/lexical SHA tests ([Numerical CI #41772379408](https://github.com/july-rust/BoardSI/actions/runs/41772379408)).
- Linux Qt 6.8.3 Offscreen **50/50 CTest PASS** including `qt-preview-d028-non45-oblique-via-b-endpoint-source`, with native Qt receipt `D028_NATIVE_QT_NON45_OBLIQUE_VIA_B_SOURCE true ... NO_ELECTRICAL_PROOF` ([Qt CI #41772379363](https://github.com/july-rust/BoardSI/actions/runs/41772379363)).
- Windows x64, Windows ARM64 and macOS ARM64 Native Qt actual backend and D_028 smoke steps **SUCCESS** on the same PR HEAD; additional Qt RHI regression steps SUCCESS.
- Independent Codex review on exact final HEAD: `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; unresolved Review Threads **0**.
- Historical initial HEAD `78b4ae100d549f118bcc30ef485b432cd4fc4c2d` had a **real Rustfmt CI failure**, due solely to the new D_028 test formatting. Corrected in final HEAD and all checks rerun; preserve this provenance, do not pretend initial HEAD passed.

## 3. Protected code merge and post-main evidence

- Code-main protected GitHub **Verified one-parent Squash** [`7c91dab8c735d7b9e59ed4352e3f3354dabe6188`](https://github.com/july-rust/BoardSI/commit/7c91dab8c735d7b9e59ed4352e3f3354dabe6188), `commit.verification.verified=true`, `reason=valid`; sole parent `4511e8d002bfa37f06ba3947b85c75d601cc2d7c`.
- **Post-code-main independent Push CI verified SUCCESS** on exact merged `main@7c91dab8c735d7b9e59ed4352e3f3354dabe6188`: **28/28 Check Runs, 8/8 `event=push` Workflows**, every run on the exact SHA, zero failed/pending/cancelled/skipped Check Runs at final verification. [Qt Runtime Push #41773687613](https://github.com/july-rust/BoardSI/actions/runs/41773687613), [Numerical Core Push #41773687542](https://github.com/july-rust/BoardSI/actions/runs/41773687542). Verified 2026-11-04 KST; this is independent of the code PR's 34/34 and 11/11.

## 4. Explicit exclusions and documentation gate

**NOT VERIFIED / NOT ACCEPTED:** actual manufactured copper or plated barrel conduction, board-wide Open/Short, P009 Reference Return Path, PH-08, physical SI/RLGC/S solver, Scenario/Cache, formal M1/M2 U/T acceptance, production release. Keep parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN and legacy [PR #35](https://github.com/july-rust/BoardSI/pull/35) CLOSED/UNMERGED.

This separate documentation PR follows independently verified exact post-code-main Push success. It changes only `docs/development-checklist.md` and this Markdown ledger, with no native compiler job allocation (existing docs-only classifier; do not weaken required contexts). **Still required after opening this documentation PR:** verify exact final docs HEAD required checks, independent Codex no-findings review, zero unresolved threads, protected Verified Squash, exact post-doc-main Push CI, and only then close Issue #424. Do not pre-record unobserved tests or date/commit states.
