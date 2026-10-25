# A_026 P2-Q — COPCCO source semantics and acceptance preflight

Issue: #183. Base protected main `b78bbb4ebdb0baefefc0b792668c040fe9e5d984`, A_025 merged at #182.

## Source semantics — confirmed, but production exporter compatibility NOT VERIFIED

1. [PADS Layout ASCII Format Specification 9.4](https://www.freecalypso.org/pub/CAD/PADS/pdfdocs/Ppcb_ASCII.pdf), Lines item table: `COPPER/COPCCO` is a *circular void in copper*. `COPCUT/CIRCUR` is a distinct unassociated circular void in copper pour; do not attach it by geometry.
2. [KiCad PADS parser](https://docs.kicad.org/doxygen/pads__parser_8cpp_source.html), copper-piece branch: `PIECETYPE CORNERS WIDTH FLAGS LEVEL`; `COPCIR`, `COPCCO` and `CIRCUR` use two diameter endpoint coordinates relative to the parent record's source origin. This is independent implementation evidence, not validation of every PADS export.
3. A_025 `copper_preview.rs` accepts a `COPCIR` primary circle on strict horizontal/vertical equal-grid diameter but intentionally rejects `COPCCO` cutout. Canonical `Cutout.boundary` supports `BoundarySegment::Arc`; current `augment` has a polygon-only `unreachable!` arm. Qt V11 `H` encodes source polygon rings, `validateCopperArea` and Qt RHI consume polygons only. **Never just enable `COPCCO` in the parser without Canonical + Qt parity.**

## Acceptance contract before activation

- Retain exact 0.01-mil source tick coordinates, origin, layer, owner multi-piece and declared net label distinct from verified electrical connectivity.
- Require explicit same-COPPER ownership, 2 bounded source endpoints, exact midpoint on source grid, strictly positive integer radius, and layer equality. Keep unassociated `CIRCUR` and orphan `COPCCO` fail-closed.
- Test strict circular void containment in outer straight polygon or source circle. Use checked i128 squared-distance and point-to-segment comparisons, rejecting tangent/contact (including diagonals), intersections, outer crossings, nested/overlapping/touching holes and arithmetic overflow.
- Extend Canonical `Cutout.boundary` with four exact circular quarter-arc segments as already used by source `COPCIR` primary shapes. Do not replace source geometry with display tessellation.
- Only accept in production parser after versioned Qt IPC can transmit circles losslessly, validate CPU/GPU shared mesh and frame-presented GPU picking, and enforce a documented maximum approximation-error bound; reject unsupported circle combinations rather than fabricating fill.
- Native Windows x64, Windows ARM64, macOS ARM64 Qt selftests and Rust tests, existing A_022–A_025 regressions, exact-HEAD independent Codex review, all protected required CI and Verified Squash merge, separate post-merge required CI.

## Conservative rendering plan (candidate, not accepted)

Use screen-space error bounded circle tessellation. For cutout safety, tessellate the void as an outward conservative polygon, so approximate triangles never fill any part of the true source void. Bound tessellation count and error; do **not** claim an exact source-tick area equality for a mathematical circle. CPU/GPU both use the same validated mesh for display and picking. Exact mathematical circle predicates remain authoritative for source-space eligibility. If bounded approximation cannot be independently proven, refuse import.

**Progress:** CP1 grammar grounding / contract recorded; CP2–CP6 not passed. Scope is source-authored Preview artwork only. Physical SI, actual PADS exports and actual GPU memory/reset remain NOT_VERIFIED/NOT_RUN.
