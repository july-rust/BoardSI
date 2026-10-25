# A_025 P2-P — source geometry acceptance preflight (IN PROGRESS)

Baseline: protected `main@b7d4f62c9b3278ed9f190931dca5207aea73431f`, merged A_024 PR #180. Related Issue #181.

## Observed current implementation (confirmed in source at baseline)

- `copper_preview::parse`: only PADS V9.4 MILS, source-owned multi-piece `COPPER`, primary `COPCLS/COPCIR`, followed by up to eight same-layer `COPCUT`. Each closed source ring is preserved as 0.01-mil ticks. Standalone/misdeclared `COPCUT`, `COPCCO` and arcs fail closed.
- `orthogonal_copper_holes_valid`: requires every edge of the outer and cutout contours to be orthogonal; uses exact source-tick topology and 4096 grid-cell complexity cap.
- `imported_artwork.h::validateCopperArea`: Qt V11 decoder's hollow mesh uses integer axis-aligned grid cells; **not** a generic diagonal tessellation algorithm. `rhi_artwork.h` consumes those generated triangles and checks hole/concave picking.
- Therefore enabling diagonal COPCUT in Rust without a matching Qt protocol, independent mesh-area validation and RHI/current-frame picking tests would break the A_024 fail-closed contract.

## New isolated validator (CP2 initial, NOT wired into production)

`polygon_topology.rs` introduces an **independent preflight predicate and twice-area oracle** for straight-line diagonal polygons. It uses exact `i128` orientation, segment-contact and shoelace operations, winding-independent containment, and bounded source-tick coordinates (±5,600,000). Total closed-ring vertices are bounded at 128, individual closed rings at 65 and hole count at eight. All touching, intersecting, disjoint-outside and nested holes are rejected. No floating point predicates or bounding-box substitute fills.

This module is deliberately **not called by** `copper_preview::parse`. No import acceptance has been widened and no GPU geometry is claimed.

## Remaining before A_025 can pass

1. Cross-check the official PADS V9.4 explicit piece syntax and input limits; source: [PADS Layout ASCII Format Specification 9.4](https://www.freecalypso.org/pub/CAD/PADS/pdfdocs/Ppcb_ASCII.pdf). Public synthetic fixtures must not imply vendor-export validation.
2. Select and qualify bounded triangulation of non-orthogonal polygons **with holes** that preserves source vertices. Intersections created during decomposition may have rational (non-integer-tick) coordinates: represent them exactly through the geometry oracle, then test any floating conversion used only for GPU graphics. Do not silently round them onto the source grid.
3. Validate independent equality of exact filled polygon area and triangle mesh area, positive triangle orientation and absence of triangles within source voids; malicious and degenerate cases must fail closed.
4. Implement equivalent strict decoder acceptance, CPU/RHI shared mesh, current-import presented-frame evidence, source-based net/layer filters and hole/concave negative picking.
5. Add public V9.4 synthetic fixture(s) and native Qt smoke on Windows x64, Windows ARM64 and macOS ARM64, Linux offscreen plus Rust test and all previous regressions.
6. Run independent Codex final-HEAD review, strict status checks, Verified Squash merge and separate post-merge `main` checks.

**Excluded / NOT VERIFIED:** circular `COPCCO`, arcs, stand-alone voids, flood fill, electrical connectivity, real PADS production-export comparison, physical SI, RHI hardware FPS/memory and 3D EM.

The successful CP2 topology predicate alone **does not constitute completed A_025**.
