# A_028 / P2-S — COPCIR with COPCUT polygon and mixed COPCCO

Baseline protected `main@cd667979d7e00ac82b150edce04db7a782afeed0` (A_027 PR #186). Issue #187.

## Bounded source contract

PADS V9.4 MILS `*LINES*` source-owned `COPPER` multi-piece records alone establish polygon or circle cutout ownership. `COPCIR` is the analytic circular primary with a strictly positive source-grid radius; `COPCUT` is an exact closed straight polygon and `COPCCO` is an exact source circle. Loose COPCUT, standalone CIRCUR, invalid piece count, wrong layer, unowned voids and unexpected grammar remain fail-closed. Provenance, line/layer, original source ticks (254 nm), exact Canonical polygon boundaries and circular quarter-Arcs are retained. The declared label is not proof of an electrical NetId (`net: None`).

### Exact Rust source topology

- A polygon strictly inside a circle must have every source vertex strictly inside the disk. The disk is convex, hence all polygon edges remain inside without sampled geometry.
- `polygon_topology::validate_disjoint_polygon_holes` checks each ring's source-grid bounds, simplicity and positive nonzero area, forbids pairwise contacts, overlaps and nesting, and includes 33 outer proxy vertices in the fixed complexity budget.
- Mixed circle/polygon holes use `circle_strictly_disjoint_from_polygon` for each pair plus prior `circles_strictly_disjoint` and `circle_strictly_in_circle`. Tangency, crossing and ambiguous boundaries fail closed.
- The existing strict PADS ownership, maximum eight holes, overall shape limits, bounded input and V11/V12 protocol logic are preserved.

### Conservative Qt display boundary

`a026::innerProxy` creates the 32-edge, source-grid INSCRIBED COPCIR preview outer (<=1% source-circle radial underfill); `COPCUT` remains exact source tick polygons; `a026::proxy` uses 32-edge CIRCUMSCRIBED source circular hole polygons (<=1% radial overfill). These are **display-only**, not Canonical geometry. The A_025 triangulation/area/overlap oracle verifies every CPU and RHI triangle of the conservative filled region. Unsafe or non-tessellable geometry must be rejected, not drawn with fictitious copper or silently omitted cutouts.

V11 `G|C` + indexed `H` can represent circle+polygon alone; mixed `G|C` + `H` + `J` reuses V12 without transport format change. Hole-aware CPU Graphics View and Qt Quick/RHI use the same mesh, picking, layer controls and source properties. GPU hardware proof requires a real backend; headless/software fallback is separate.

## Public synthetic fixtures and native acceptance

- `pads_copper_circular_polygon_v94.asc`: one circular primary with square and sloped triangular voids.
- `pads_copper_circular_mixed_v94.asc`: one circular primary with a source square void and one circular void.
- Rust negative mutations include circular boundary tangent, polygon sibling overlap, bow-tie/self-intersection, wrong layer, orphan/understated owner, mixed polygon-circle intersection, circular tangency, and bad ownership.
- Qt `--circle-polygon-selftest` and `--circle-mixed-selftest` must report imported RHI frame, CPU hollow pixels, exact void picking, layer hide/restore, source properties, malformed input refusal and cleanup.
- All existing A_022..A_027 regressions must pass on native Linux, Windows x64, Windows ARM64 and macOS ARM64.

## Evidence state (must be updated from actual GitHub runs)

Implementation commits are candidates **NOT YET ACCEPTED**. There is no claim of CI passing, source exporter compatibility, independent exact-head Codex review, verified Squash merge or post-merge main CI until source run IDs and commit SHA are recorded.

Production-exporter data, fabricated conductive copper, actual electrical connectivity/flood fill, 2D RLGC/S extraction, measured real GPU VRAM, driver reset and 1M / 60 FPS remain **NOT_VERIFIED / NOT_RUN**.
