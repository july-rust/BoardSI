# D_056 — Generalized Input Normalization and Via Topology Contract

**First checked:** 2026-11-08 KST. **State:** FIRST CODE SLICE MERGED; D_056 OVERALL OPEN, NATIVE PARTIAL-VIA INPUT / FULL CONNECTIVITY / PRODUCT NOT VERIFIED. **Code PR-head:** [#521](https://github.com/july-rust/BoardSI/pull/521) 30/30 checks and 10/10 workflows SUCCESS; **protected squash main:** [288b16e](https://github.com/july-rust/BoardSI/commit/288b16e19054102e752d433ab60088e669ea235a) (one parent, GitHub verified=true/reason=valid). Post-main push checks are a distinct evidence gate.

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
- `trace_attachment::trace_to_normalized_via` reuses the D_055 annular-copper kernel with the actual layer-specific pad diameter for bounded SOURCE-grid geometry; L2–L4 / L2–L3 and missing-layer-pad tests are included. This remains a geometry result only; it is not wired to native PADS blind/buried parsing or verified global electrical graph edges.
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

The protected code PR-head reported 30/30 GitHub Checks SUCCESS and all 10 workflow groups SUCCESS (Rust `fmt`/Clippy, Debug/Release workspace regression, Windows x64/ARM64/macOS ARM64 native Rust, Linux/Windows/macOS native Qt regressions, docs, A032, M2). These are repository-reported CI results, not a claim of local or measured hardware validation. Code-main push CI is separate and is not inferred from the PR-head results.
