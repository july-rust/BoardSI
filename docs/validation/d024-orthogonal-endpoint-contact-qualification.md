# D_024 — PADS Same-Layer Orthogonal Via-B L-Bend Source Contact Qualification

- Tracker: [Issue #412](https://github.com/july-rust/BoardSI/issues/412)
- Code PR: [#413](https://github.com/july-rust/BoardSI/pull/413), **MERGED** to protected main
- Initial recorded date: **2026-11-04 KST**
- Code limited acceptance / evidence date: **2026-11-04 KST**
- Status: **LIMITED SOURCE-ONLY CODE QUALIFICATION ACCEPTED**. Full code-main push CI and docs-only administrative closeout are separate evidence gates.
- Base protected main: `6f1ed6678934f460a0b52e1ebc61cc362c42af78`
- Dedicated opt-in: `--d024-orthogonal-endpoint-contact-v1 NET_A 1 4 0 1`
- Source fixture: `crates/boardsi-import/tests/fixtures/d024_orthogonal_via_b_endpoint_v94.asc`

## Frozen bounded Source contract

PADS V9.4 MILS; exact Source Tick = 0.01 mil = 254 nm. Main Layer-4 Trace (15000,25000) → (35000,25000); Branch (35000,25000) → (35000,35000); Through Via-B Pad (35000,25000); both Trace primitives belong to declared `NET_A`. Exactly **90° orthogonal** with **zero positive-length centerline overlap**; reverse orientation of the Branch is accepted. Main Trace, Branch Trace and Via Pad remain separate Source-owned primitives.

Expected exact Source-only candidate graph: **8 nodes / 6 layer-local candidate edges / 3 local components / 0 proven electrical cross-layer edges**. New triangle consists of Via-B Layer-4 Pad (Node 2), Main Layer-4 Trace (Node 5), and Branch Layer-4 Trace (Node 7); new orthogonal edge 5–7 and separate Pad-to-Branch candidate 2–7. These are neither fabricated Cu continuity nor electrical barrel links.

The Rust mode uses bounded complete Source section inventory, exact `i128` dot/cross geometry and no floating tolerance; the raw complete Source bytes are SHA-256 bound into a receipt. Independent Qt importer re-imports Source/Canonical Geometry and reproduces the exact expected native wire receipt. Reject one source-tick gap/offset, non-orthogonal diagonal, collinear, contained/partial overlap, wrong Via/Net/Layer, degenerate Trace, unmodelled extra Source, stale/forged receipt and electrical-promotion attempts. D_015–D_023 remain strict and mutually disjoint.

## Exact code PR evidence (verified)

- Final code HEAD: [`b2ccd95573acc61a0e26bcf6534da4e7fef7ef0b`](https://github.com/july-rust/BoardSI/commit/b2ccd95573acc61a0e26bcf6534da4e7fef7ef0b/checks).
- **34/34 PR Check Runs SUCCESS, 11/11 event=pull_request Workflows SUCCESS**; Rust Format / Strict Clippy / Workspace regression and D024 dedicated Rust **3/3 PASS**.
- Linux Qt 6.8.3 Offscreen [PR workflow #41743871737](https://github.com/july-rust/BoardSI/actions/runs/41743871737): **46/46 CTest PASS**, including D_024.
- Dedicated native D_024 Qt tests: **Windows x64 PASS; Windows ARM64 PASS; macOS ARM64 PASS** on exact HEAD. Native Qt independently reconstructs Source SHA-bound witness and rejects stale, forged, promoted, and hostile cases.
- Exact-HEAD Codex Review Summary: **completed / has_any_findings=false / has_unresolved_findings=false / unresolved review threads=0**.
- Protected signed GitHub Verified one-parent Squash [`main@9829468384ebe94e3bbec979c9ae96b26a0a6b2e`](https://github.com/july-rust/BoardSI/commit/9829468384ebe94e3bbec979c9ae96b26a0a6b2e); sole parent `6f1ed6678934f460a0b52e1ebc61cc362c42af78`, GitHub commit `verification.verified=true` and `reason=valid`.

## Code-main push evidence

- Correct push commit: [`9829468384ebe94e3bbec979c9ae96b26a0a6b2e`](https://github.com/july-rust/BoardSI/commit/9829468384ebe94e3bbec979c9ae96b26a0a6b2e/checks).
- Independently verified exact merged SHA: **28/28 Check Runs SUCCESS**, **8/8 event=push Workflows SUCCESS**, without failed, cancelled, pending or skipped checks. [Qt Runtime run #41745544417](https://github.com/july-rust/BoardSI/actions/runs/41745544417) and [Numerical Core run #41745544598](https://github.com/july-rust/BoardSI/actions/runs/41745544598) finished SUCCESS on the identical merge SHA. This is separate from the PR's 34/34 and 11/11.

## Preserved earlier failure

The superseded initial HEAD `78fed521011fe556f0fada63e90c4e415a32acbe` failed Linux Numerical `cargo fmt --all -- --check` for newly added CLI and Rust test formatting. Corrected based on exact CI rustfmt hunk before clean final HEAD. The failure was not proof of a functional regression. Do not claim any other or older Windows ARM64 intermittent failure was root-cause fixed.

## Physical evidence exclusions and issue policy

Only a bounded PADS source-owned *geometry candidate* is accepted, NOT: manufactured copper, physical Via Barrel plating and electrical continuity, whole-board Open/Short, Reference Return Path / P009, PH-08, physical Scenario/Cache, validated RLGC/S-parameters, solver readiness, formal U/T or release.

Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN and Draft PR #35 unmerged. Administrative closure of #412 requires independent docs-only PR merge and post-doc-main verification. Docs-only changes must not unnecessarily allocate native compiler runners.
