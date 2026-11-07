# D_056 — Generalized Input Normalization and Via Topology Contract

**First checked:** 2026-11-08 KST. **State:** PARTIAL IMPLEMENTATION, CI/PUSH/PRODUCT acceptance not established.

## Why

D_055 implemented generic off-centre trace-to-via annulus overlap, but the source importer has a separate limitation. Its `scan_simple_via_defs` currently accepts only PADS V9.4 MILS round `-2/-1/0` definitions with **equal diameters**, and source route placements are interpreted as Through-Via `L1..MAXLAYER`. There is no verified native blind/buried/microvia PADS parser. `ViaKind` and `ViaSpan` in `boardsi-model` can represent more topology than the current PADS adapter.

## Target decomposition

1. **Source format adapter:** Parse exact syntax, source layer identifiers, pad definitions, drill and actual span. Reject unsupported grammar with diagnostics. PADS special `-2/-1/0` identifiers are not physical Layer IDs.
2. **Normalizer:** One `NormalizedVia` contract holds total copper layer count, first/last physical layer, explicit kind, plating, drill and separately declared conductive layer pad diameters. Validate span-kind consistency, dimensions, actual pad layers and duplicates; no guessed default FR-4, no fabricated inner pad or connected layer.
3. **Canonical:** Resolve layer-order identities, build per-layer padstack shapes and drill span with immutable provenance, preserve the separate `connected_layers` evidence. No graph edge solely from matching XY, net or span.
4. **Geometry and topology:** Use generic D_054/D_055 distance/intersection on *the actual copper layer*, not via-centre coincidence or a universal diameter. Hold back old SOURCE-local fixed-4L qualification paths unless their exact required assumptions are established.
5. **Result gating:** Structurally valid but unsupported sources are `PARTIAL` or `UNSUPPORTED`, not verified electrical/physical contacts or SI.

## Implemented first safe slice

- New `crates/boardsi-import/src/via_topology.rs` with `NormalizedVia`, `CircularLayerPad` and validated copper-layer bounds/kind/diameters.
- A reusable PADS V9.4 through pad mapping, with independently supplied top, interior (optional) and bottom pad diameters at normalization level. **The shipping source parser still accepts only equal-diameter PADS records** until all legacy consumers can preserve nonuniform geometry.
- The existing PADS restricted Through-Via importer now uses the shared normalizer to build canonical padstack entries. Its existing SOURCE grammar, normal Qt artwork output and electrical qualification level are unchanged.
- Generated-style parameter tests cover 2, 4, 6, 8 and 64 copper-layer through normalization and L2–L4, L2–L3 buried, L1–L3 and L3–L6 blind, and adjacent-span microvia data **at the normalization layer**. Additional PADS restricted parser matrix covers 2, 4, 6, 8.
- Invalid duplicate/out-of-span pad, drill/pad overlap, inconsistent kind/span and missing inner pads fail closed.

## Still NOT IMPLEMENTED / NOT VERIFIED

- Parsing actual blind/buried/microvia records from native PADS, ODB++, IPC-2581, Gerber, KiCad; this requires source format-specific evidence.
- Replacing the old `ViaInstance.diameter_ticks` / `inner_pad` transport assumptions throughout local preview, legacy fixed-4-layer fixtures, reference-area/routing modules and Qt wire types; nonuniform source export remains deliberately disallowed rather than risking a false contact.
- Generic per-layer shapes beyond circles; arbitrary slot/drill, anti-pad, clearances, backdrill and stacked/staggered manufacturing structure.
- Full-board proven connected-layer inventory, actual plated barrel continuity, global connectivity graph promotion, solver/Port/S-parameter readiness, and measured validation.
- Full audit/remediation of every object importer including component placement and copper pours. These are separately gated follow-up work within Issue #520 and **must not** be marked complete by the first PR.

**Exit gate for D_056:** native supported source adapters must prove actual layer/shape/span mapping, all SOURCE-dependent downstream consumers must use qualified per-layer footprints or explicitly fail closed, and a source-backed canonical topology regression matrix must pass. Keep Issue #520 OPEN until that gate is met.

## Evidence

- [D_056 parent issue](https://github.com/july-rust/BoardSI/issues/520)
- [Canonical model contract](canonical-board-model.md)
- [Generic D_054/D_055 contact geometry](trace-copper-contact.md)
- [D_054/D_055 SOURCE qualification](../validation/d054-d055-general-trace-contact-qualification.md)

No locally executed compiler or tests are claimed by this document. GitHub CI statuses must be read separately on the PR head and main push.
