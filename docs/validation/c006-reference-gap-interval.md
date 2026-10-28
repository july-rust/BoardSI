# C_006 — Source-rational Reference Gap Interval / Qt risk spans

**Tracker:** [Issue #252](https://github.com/july-rust/BoardSI/issues/252) · **Implementation candidate:** [PR #253](https://github.com/july-rust/BoardSI/pull/253) · **Start:** 2026-10-28 KST · **Status:** PENDING exact-head CI, independent review, protected merge and post-main checks.

## Implemented candidate, subject to acceptance

- Existing C_005 V1 source geometry warning remains unchanged for two-trace plated Via transitions. Opt-in V2 `--reference-gap-v2` for one selected straight Signal Trace and one explicitly chosen adjacent source Reference Layer/Net.
- Reuses C_005 V1 independently source-bound PADS import/audit and source SHA/Trace witness. Rust exact rational t intersections of one source-owned straight COPCLS polygon and up to 8 straight COPCUT edges, bounded at 32 gaps. Each proper crossing toggles inside/outside. Inner/outer boundary tangent, collinear line, vertex crossing, overlapping source Reference areas and out-of-range coordinates are refused or UNSUPPORTED, never fabricated as safe. Unknown Source IDs and layers do not produce positive source results.
- Rust wire V2 emits ordered exact t fractions. Display positions/approximate lengths in Qt are only projections of these original rational parameters and must **not** be called exact nanometre intersection coordinates. Source hash, original Track index, Signal Net/Layer, entire Trace endpoints, bounded fractions and all no-physics guards checked at Qt receive.
- CPU GraphicsScene selectable gap spans, Qt Quick/RHI thick line spans and segment proximity picking. Existing UI reimport/source-hash/selected layer/material/Net/Worker late reply staleness policy reused. Old V1 remains callable and unchanged.
- Public synthetic Rust tests include covered, fully uncovered, one/two cutouts, outer crossing, diagonal, collinear unsupported, invalid Reference and V1 compatibility. Qt actual Rust process synthetic cutout native source-risk smoke checks exact `1/6..1/2` and stale source lifecycle, run on Windows x64/ARM64, macOS ARM64; Linux Qt regression remains in existing CTest job.

## Important limitations

- **Not actual manufactured/flooded Copper topology:** Copper Island separation remains conservative and no continuity connection between distinct source areas is inferred. Polyline curves, touching polygon vertices/edges and unsupported geometry are rejected. Polygon arithmetic is exact **within bounded source coordinates**; rendered endpoint/length is approximate for screen visualization.
- **No physical solver qualification:** `return_path_p009=UNSUPPORTED`, `solver_ready=false`, `physical_si=NOT_RUN`, `cache_reuse_eligible=false`. No AC Return Path, RLGC/S, loss physics, S-parameter, full SI physical Cache, arbitrary commercial PADS, or U09/U10/U39/T02/T04/T24/T52 acceptance. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.

## Acceptance ledger — fill only with independently verified evidence

- [ ] Rustfmt / Clippy / Debug / Release and old B/C source regression on **final PR head**.
- [ ] C_006 Rust synthetic test counts and expected exact fractions checked from actual CI.
- [ ] Qt Native Windows x64, Windows ARM64, macOS ARM64 and Linux CTest verified on final PR HEAD (not compile-only).
- [ ] Independent Codex same-final-HEAD review completed, zero findings and zero unresolved threads.
- [ ] Strict ruleset protected Signed Verified Squash merge and independently checked post-main workflows / check runs.
- [ ] Only then update the live checklist entry to [x], adding **first checked date** and distinct **evidence date**, exact PR HEAD, merge SHA and tested links. Existing first-check dates remain immutable.

**No full product/physics acceptance should be inferred from these source geometry tests.**
