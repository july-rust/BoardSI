# C_010 — Source Reference Corridor / Stitching Via Source Area identity (limited candidate)

**Tracker:** [Issue #264](https://github.com/july-rust/BoardSI/issues/264) · Parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN. Baseline accepted C_009 code and documentation `main@d7768512a0dd98b54f4b28b3a0cd61727307f5c5` (signed Verified, post-doc-main 21/21 checks and 9/9 workflows SUCCESS, 2026-10-29 KST).

**Status:** IMPLEMENTATION CANDIDATE — NOT ACCEPTED until exact final PR HEAD and post-merge independent validation. Do not mark checklist `[x]` without evidence.

## Source-bounded behavior

- Opt-in `--reference-route-v6`; V6 embeds **byte-exact V5**, including V4/V2 and original source SHA-256.
- For each of the two explicitly selected reference layers, require a unique Source-owned Reference COPCLS/COPCUT area whose **entire selected signal-leg centreline** is strictly interior (no boundary or cutout intersections), using existing exact `reference_geometry::segment_strictly_inside` predicate on newly Source-parsed Canonical geometry.
- Require source order adjacency of signal and Reference layer. Compare this unique area Source ID against the corresponding C_009 V5 strict Source ViaCentre–AreaInterior copper-contact Area ID.
- Preserve matching Net names with separate island Area IDs as **unconnected**. A nearby Via is only a verified *candidate*, never Return Current proof.
- Fail closed for gap/cutout, mismatched Area ID, ambiguous overlapping candidate ownership, unsupported source/route, invalid Via/Net/contact and changed Source SHA.
- Qt CPU and RHI expose explicit candidate/risk marker and area IDs, with source watcher/reimport/selection/generation/late worker invalidation; no physical PASS.

## Public synthetic regression contract

`crates/boardsi-import/tests/c010_reference_corridor_v6.rs` covers same Source Area, independent same-Net Copper Island mismatch, signal corridor Cutout, boundary-touch refusal, changed Source SHA, invalid Via and Reference selections. Native process integration test `qt-preview-c010-reference-corridor-stitch` runs actual Rust importer→Qt and checks source-risk rendering/invalidation; no GPU driver-reset or real-board claim.

## Gate ledger — PENDING

- [ ] Exact final PR HEAD Rustfmt, Clippy, Debug/Release and all previous B/C source tests success.
- [ ] Windows x64/ARM64, macOS ARM64, Linux native Qt actual importer→Qt risk/stale success.
- [ ] Independent Codex SAME final HEAD completed, findings 0 and unresolved 0.
- [ ] Ruleset #25046595 strict required checks success, protected Signed Verified Squash.
- [ ] Independent post-code-main checks and push-triggered workflows all success.
- [ ] Evidence-backed first-checked/date KST update to `docs/development-checklist.md` after all proven gates, separate signed documentation PR if needed.
- [ ] Issue #264 close ONLY on supported limited acceptance; M1 #12 remains OPEN.

**Never claim:** true Return Current/P009, full copper flood/thermals, physical RLGC/S, production Solver/Cache, arbitrary board or formal U09/U10/U39/T02/T04/T24/T52; PH-08 prior FAIL unchanged.

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```
