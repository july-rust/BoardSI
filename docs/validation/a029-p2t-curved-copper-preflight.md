# A_029 / P2-T — curved COPCLS copper, gated candidate

Baseline: protected `main@e86b124c28f7622fdd4cfa30d675e81f6cfb782b` (A_028 PR #188). Issue #189.

## Verified source observations

- Existing `outline_arc::preflight_closed` parses exact-source-grid PADS V9.4 cardinal 90-degree arcs; `outline_topology::ring_valid` conservatively checks line/quarter-circle crossings.
- Current `copper_preview::point` requires exactly two fields, intentionally rejecting all eight-field COPCLS corner arcs. The existing V10–V12 `G|P` and `G|C` messages have no source-arc preservation contract. The current Qt copper decoder/GPU path must not silently reinterpret the chord proxy as literal source copper.
- Existing A_028 COPCIR/COPCUT/COPCCO capability and physical exclusions remain unchanged.

## First implementation increment

`copper_arc_preflight::validate` accepts only a deliberately limited candidate:
- strictly closed COPCLS-shaped point sequence with 2-field lines or exact 8-field cardinal quarter-circle arcs;
- outward-curving arcs of a convex positive-or-negative winding boundary;
- at least one arc, 4–65 source rows, radius >=1,000 source ticks, at most 256 display points;
- exact source arc center, endpoints, clockwise direction and integer-tick grid;
- existing exact integer topology oracle for arc crossings and source chords.

A separate display-only preview candidate uses eight segments per 90-degree arc, with each inner sample two source ticks toward the centre. The chord endpoints are exact source endpoints. The maximum nominal radial underfill is bounded by `1 - cos(pi/32) + 3/r`, below 0.8% for `r >= 1,000` source ticks. Exact implementation verifies all intermediate vertices strictly inside the true source disk and refuses degenerate or non-convex proxy topology.

**Important limitation:** this helper does NOT modify production `copper_preview::parse`, the Canonical BoardModel, or Qt IPC/CPU/RHI acceptance. Neither import nor hardware GPU capability is claimed by this increment.

## Required next gates before feature activation

1. Strict source ownership and no Cutouts for curved COPCLS production input.
2. Exact analytical mixed Arc/Line Canonical source boundary, net remains `None`.
3. New versioned Qt IPC source/proxy distinction; CPU and GPU draw/pick same conservative validated fill.
4. Synthetic PADS fixtures (including malformed and adversarial) and native frame/picking/filter/reset tests on all required OS architectures.
5. All A_022–A_028 regressions, independent final HEAD Codex review, verified squash-only and post-merge main CI.

## Evidence state

- Rust source candidate: **IN PROGRESS**, do not mark production accepted.
- `cargo fmt` / `cargo test` / Qt native CI: **NOT RUN** at initial commit; update from actual run IDs.
- Actual vendor exports, electrical connectivity/flood, production RLGC/S and hardware GPU metrics: **NOT_VERIFIED / NOT_RUN**.

Do not merge or close #189 on source preflight alone.
