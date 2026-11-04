# D_026 — PADS Same-Layer 45° Oblique Via-B Endpoint Source Contact Qualification

- Tracker: [Issue #418](https://github.com/july-rust/BoardSI/issues/418); kept **OPEN until documentation-main evidence is verified**.
- Code PR: [#419](https://github.com/july-rust/BoardSI/pull/419), **MERGED** by protected Verified one-parent Squash.
- First recorded / bounded code acceptance / evidence date: **2026-11-04 KST**.
- Scope: **LIMITED SOURCE GEOMETRY CONTACT CANDIDATES ONLY**, **NOT** electrical proof.
- Original protected code base: `34663e350b207e434494d5f08cba3636b4903c1e`.
- Rust CLI: `--d026-oblique-via-b-endpoint-contact-v1 NET_A 1 4 0 1`.
- Fixture: [`d026_oblique_via_b_endpoint_v94.asc`](../../crates/boardsi-import/tests/fixtures/d026_oblique_via_b_endpoint_v94.asc).

## 1. Exact PADS Source-owned contract

PADS POWERPCB V9.4 MILS; **0.01 mil = 254 nm Source integer tick**. On **Layer 4 / NET_A**, Main Trace is `(15000,25000) → (35000,25000)`; independent Branch Trace is `(35000,25000) → (45000,35000)`. Contact coordinate **(35000,25000)** is the **Via-B** pad position, not Via-A. Reversing branch endpoint order is permitted only after independent exact geometry proof. Main and branch meet at exactly **45°**, distinguished from D_024 90°/D_025 Via-A and collinear/oblique-interior modes.

Rust bounds Source ticks, checks the section inventory and selection, verifies two Via records and exactly four canonical traces, and computes `i128` dot/cross. The dedicated D_026 check requires nonzero dot/cross, `abs(dot)==abs(cross)`, exact Via-B common endpoint, and branch exterior projection beyond the main segment. It rejects 1-tick endpoint offset, 1-tick angle change, wrong Net/Layer/Via, degenerate trace, orthogonal/collinear traces, hidden source shapes, unsupported source inventory and changed selections. No floating tolerance is admitted.

The resulting **SOURCE-ONLY** graph contains **8 nodes / 6 same-layer candidate edges / 3 local components / 0 proven electrical cross-layer edges**. In the Via-B local candidate triangle: Via-B Layer-4 pad is **Node 2**, Main L4 Trace **Node 5**, Branch L4 Trace **Node 7**; `EDGE|4|5|7|SOURCE_OBLIQUE_VIA_B_ENDPOINT_CONTACT_CANDIDATE|4` and `EDGE|5|2|7|SOURCE_ENDPOINT_CONTACT_CANDIDATE|4` remain geometric candidates only. No manufactured Copper or plated Via Barrel electrical connectivity is inferred.

The witness binds **SHA-256 of the original Source bytes**, carries `SOURCE_GEOMETRY_ONLY / BARREL_ELECTRICAL_NOT_VERIFIED / CROSS_LAYER_ELECTRICAL_NOT_PROVEN / SOLVER_READY_FALSE`, and ends `FAIL_CLOSED_NO_BARREL_PROOF`. The independent Qt importer reconstitutes source geometry and expected wire bytes, checks the accepted Source SHA, rejects stale/forged/cross-mode receipts and a manufactured electrical-promotion claim, and preserves D_015–D_025 mode separation.

## 2. Final code PR exact-HEAD acceptance — independently verified

- Final PR #419 HEAD: [`f0fd7b058cc133a31c92bd039b3616988ca264e8`](https://github.com/july-rust/BoardSI/commit/f0fd7b058cc133a31c92bd039b3616988ca264e8/checks).
- **34/34 PR Check Runs SUCCESS; 11/11 PR Workflows SUCCESS** (including numerical, Qt, native FEM and planning-doc checks).
- Rust dedicated D_026 tests: **3/3 Debug PASS, 3/3 Release PASS** ([Numerical workflow #41756564193](https://github.com/july-rust/BoardSI/actions/runs/41756564193)).
- Linux Qt 6.8.3 Offscreen: **48/48 CTest PASS**, including `qt-preview-d026-oblique-via-b-endpoint-source` and `D026_NATIVE_QT_OBLIQUE_VIA_B_SOURCE ... true ... NO_ELECTRICAL_PROOF` ([Qt Runtime workflow #41756564241](https://github.com/july-rust/BoardSI/actions/runs/41756564241)).
- Windows x64, Windows ARM64 and macOS ARM64 Native Qt jobs: **SUCCESS** on the exact PR HEAD.
- Independent Codex summary for the same exact HEAD: `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; unresolved review threads **0**.
- Protected **Verified single-parent Squash** code main: [`2608975de9237e9b8d0979bf875426dd05a8276f`](https://github.com/july-rust/BoardSI/commit/2608975de9237e9b8d0979bf875426dd05a8276f), sole parent `34663e350b207e434494d5f08cba3636b4903c1e`, GitHub `commit.verification.verified=true`, `reason=valid`.

## 3. Exact code-main post-merge Push acceptance — independently verified

- On exact merged-code `main@2608975de9237e9b8d0979bf875426dd05a8276f`: **28/28 Check Runs SUCCESS; 8/8 event=push Workflows SUCCESS**, no failed/cancelled/pending tests in final snapshot.
- Evidence: [Qt Runtime Push #41757913057](https://github.com/july-rust/BoardSI/actions/runs/41757913057), [Numerical Core Push #41757913282](https://github.com/july-rust/BoardSI/actions/runs/41757913282), and remaining six matched-SHA Push workflows SUCCESS.
- These **code-main** checks must not be confused with this document PR's separate docs-only validation or subsequent docs-main Push checks.

## 4. Historical first-HEAD failure, correction and strict evidence

- First PR HEAD `a5d8730d64ad9153050fe387e94fdab402a7c889` failed `cargo fmt --all -- --check` in the Numerical job due to new `d026_oblique_via_b_endpoint.rs` test formatting. The test formatting was corrected in `f0fd7b058cc133a31c92bd039b3616988ca264e8`, and **that exact final HEAD** passed all required PR CI and independent review. Do not present earlier failure as a functional correctness failure.
- D_026 was introduced using existing Rust/Qt workflows, with a dedicated native Qt step; no new platform or workflow was created.
- The docs-only change must contain **only Markdown files** and use existing docs-only classifier so native compiler jobs are not allocated solely for documentation.

## 5. Explicit exclusions and administrative finish gate

**Accepted:** bounded PADS V9.4 MILS, exact 45° Via-B same-layer Source contact candidates, original Source/SHA-bound Rust and independent Qt fail-closed qualification.

**NOT VERIFIED:** manufactured Copper/Pad electrical contact, plated Via Barrel conduction, full-board Open/Short, Reference Return Path/P009, PH-08 physical gate, RLGC/S-parameter Solver, production physical Scenario/Cache, formal U/T acceptance and release readiness. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN; Draft [PR #35](https://github.com/july-rust/BoardSI/pull/35) remains unmerged.

**Documentation closure requirement:** docs PR exact-HEAD mandatory check contexts SUCCESS (explicit classifier skips permitted only for native jobs), independent exact-HEAD Codex review finished with no findings/unresolved threads, protected Verified Squash, exact docs-main Push checks SUCCESS, and only then administrative closure of Issue #418. Do not claim those later gates completed merely by creating this ledger.
