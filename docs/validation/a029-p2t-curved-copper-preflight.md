# BoardSI A_029 / P2-T — curved COPCLS source copper integration

## Baseline and status

- Protected main baseline: `e86b124c28f7622fdd4cfa30d675e81f6cfb782b`, completed A_028 PR #188.
- Scope Issue #189; implementation draft PR #190; current candidate branch `a029/p2t-curved-copper-outline`.
- **Production code is implemented in this PR, NOT accepted or merged until exact final HEAD native CI and independent review both pass.** This supersedes the previous source-only preflight description.
- Any PASS cited here is for named source/test checks, not automatically for a future commit.

## Strict supported source subset

Only PADS V9.4 MILS explicit same-owner `*LINES*` `COPPER` single-piece `COPCLS` with 2-field line points and exact cardinal ±90° 8-field arc points. The source ring must be closed, simple and convex, with outward arcs matching winding. At least one arc is required; each source radius must be at least 1,000 × 254 nm = 0.254 mm. At most 65 point rows, 64 segments and 256 preview vertices per area. Maximum 64 bounded copper objects and global existing parser budgets remain. Arbitrary-angle arcs, non-convex arcs, curved cutouts and curved-outer combinations with any other copper cutouts are strictly rejected.

All PADS ownership, unit, layer, origin, closedness, source tick, angle, end-point and intersection gates are validated in Rust before import. Source net names remain **unverified labels**, never electrical connectivity. Other A_022–A_028 supported shapes remain unchanged. An unsafe/ambiguous curved shape causes the import to fail closed rather than silently rendering a straight surrogate.

## Canonical and Qt transport

- `copper_preview::CopperGeometry::Curved` contains two **separate** forms: exact source `outline_topology::Segment::Line/Arc` and conservative display-only points.
- `copper_preview::augment` stores analytic `BoundarySegment::Line/Arc` in `BoardModel.areas[].outer`; the display mesh never replaces the Canonical source boundary.
- New Qt wire header `BOARDSI_P2T_PARTIAL_V13`, source-owned `G|R` record with a bounded source line/arc array, and explicit curve count in `END`. Existing V1–V12 formats remain valid. The decoder rejects malformed, underspecified or incompatible version records.
- `a029_curved_copper.h` independently reconstructs the conservative source-grid preview from exact source arcs, validates radial containment, convexity, closure, directions, segment count, and exact polygon fan-area equality. Per quarter arc 8 display chords are used; intermediate vertices are rounded inward by two 254-nm source ticks. Display-only radial underfill bound is <0.8% for accepted source radii, but topology requires additional exact checks.
- `ImportedCopperArea::hollowTriangles` is one common validated mesh consumed by QPainterPath CPU fill and Qt Quick/RHI GPU drawing and picking. This is preview geometry, not a conductivity or flood-fill computation.
- Qt area Properties explicitly marks conservative arc preview; layer visibility, re-import and cleanup continue to use the shared area pipeline.

## Required native acceptance

The synthetic, redistributable fixture `pads_curved_copper_candidate_v94.asc` covers four quarter-circle corners. Rust tests include exact source topology, adversarial arc mutations, source owner, failed unsupported combinations, Canonical boundary, and source net unverified. The Qt `--curved-copper-selftest` exercises the real Rust import executable, actual Qt V13 decoder, CPU filled/not-filled control points, Properties, layer hide/restore, rejected malformed V13, cleanup, imported-current RHI frame and triangle-based picking. A_022–A_028 regressions remain mandatory on Linux, Windows x64/ARM64 and macOS Apple Silicon.

Source CI reference before this document update: [Qt runtime 40187015909](https://github.com/july-rust/BoardSI/actions/runs/40187015909) and [PR #190](https://github.com/july-rust/BoardSI/pull/190). These are **intermediate** results; the final acceptance record must identify the exact reviewed HEAD, every required check run ID and the post-merge main validation.

## Hard acceptance and exclusions

A_029 remains **NOT COMPLETED** until exact final HEAD CI PASS, independent Codex review with zero unresolved P1/P2, protected GitHub Verified Squash Merge, and separate protected-main CI success. Do not bypass ruleset or retag previous tests.

Real commercial PADS exporter certification, conductor flood, electrical connectivity, board fabrication, production RLGC/S, 3D EM, physical GPU VRAM/driver reset and million-object 60 FPS remain **NOT_VERIFIED / NOT_RUN**. Keep the broad roadmap Issues #9/#11/#165 and deferred #46 open.
