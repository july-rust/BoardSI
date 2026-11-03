# D_017 — Source-only Oblique X-Junction Qualification

Recorded: 2026-11-03 KST. Tracking [Issue #386](https://github.com/july-rust/BoardSI/issues/386), code [PR #387](https://github.com/july-rust/BoardSI/pull/387). This ledger reports **bounded source-geometry contact candidates**, not electrical continuity.

## Scope and geometry

D_017 extends D_016 with a strictly oblique (non-perpendicular) **interior–interior** crossing of two straight Layer-4 Trace centrelines. The public synthetic fixture `crates/boardsi-import/tests/fixtures/d017_oblique_x_junction_v94.asc` retains four PADS V9.4 MILS layers, two Through-Via placements and four straight Traces. The Layer-4 main Trace connects (150,250) to (350,250), and the fourth branch runs from (220,350) to (300,150); the exact SOURCE coordinates are in PADS MILS tick units, not fabricated electrical dimensions.

The explicit opt-in `boardsi-import-artwork <file.asc> --d017-oblique-x-junction-v1 NET_A 1 4 0 1` reuses D_016's complete strict Source section inventory, SHA-256 binding, Via/Trace Net/Layer/geometry checks and Canonical consistency. An exact integer orient/dot check requires strict crossing with dot product nonzero, so D_015 endpoint–interior T, D_016 perpendicular X, collinear overlap, endpoint touch, net/layer mismatch and near misses cannot be promoted as D_017. Real Qt independently reimports Source and checks SHA, all candidate rows and imported objects rather than trusting the Rust receipt. Rust/Linux Qt/Windows x64 & ARM64/macOS ARM64 native tests include forged/stale receipt and unsupported geometry rejection. Ordinary default importer and GitHub Actions workflow contracts remain unchanged.

The restricted deterministic Source graph reports **8 nodes / 5 SAME-LAYER candidate edges / 3 local components / 0 electrically PROVEN cross-layer edges**. Source `PLATING_IMPORTER_ASSUMED` and `BARREL_ELECTRICAL_NOT_VERIFIED` are **not manufacturing or electrical barrel proof**. `SOLVER_READY_FALSE` stays mandatory.

## Exact verified code acceptance — 2026-11-03 KST

- Code PR #387, final HEAD `e963d070619f0f137528b50eae927856f4cef2fd`: **27/27 GitHub PR Checks and 11/11 event=pull_request Workflows SUCCESS**, same SHA, no incomplete or failed checks.
- Independent same-final-HEAD Codex code review: `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; zero unresolved review threads. Initial Rustfmt/warnings and Windows CRLF fixture false-negative failures were corrected and checked again at final HEAD; the separate F6 toolchain Clippy installation conflict passed failed-only rerun on the same SHA.
- GitHub protected Verified, valid, one-parent code squash: [`main@236617055cc1bd7c4eb961f0853bac0caa9d0ce7`](https://github.com/july-rust/BoardSI/commit/236617055cc1bd7c4eb961f0853bac0caa9d0ce7), sole parent `ce2f63d41a4ff43b115e73bba25f78e8543757f8`.
- Independent **exact code-main SHA** post-squash: **21/21 GitHub Check Runs SUCCESS and 8/8 event=push Workflows SUCCESS**, every head SHA matching `236617055cc1bd7c4eb961f0853bac0caa9d0ce7`. This is independent of the PR checks.

## Documentation acceptance gate — pending

- [ ] This documentation PR's exact final HEAD: all required checks/Workflows SUCCESS, independent same-head Codex no findings, zero unresolved threads.
- [ ] Protected one-parent GitHub Verified Squash for this document PR.
- [ ] Independent exact documentation-main SHA event=push Checks/Workflows SUCCESS.
- [ ] Only after all the above, independently close Issue #386 and reconcile its actual status in a later evidence-only checklist change. Do not preclaim doc-main results or closure.

**Explicit non-acceptance:** manufactured plated Via barrel electrical continuity, copper fill/manufactured pad physical continuity, whole-board Open/Short, P009/PH-08, 2D/3D physical SI, SI Solver, Scenario/Cache or formal M1 U/T. Parent Issue #12 remains OPEN. Draft PR #35 remains OPEN and unmerged.
