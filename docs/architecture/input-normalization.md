# D_056 — Generalized Input Normalization and Via Topology Contract

**First checked:** 2026-11-08 KST. **State:** TWO CODE SLICES MERGED AND SCOPED CI VERIFIED; D_056 OVERALL OPEN. Core foundation [PR #521](https://github.com/july-rust/BoardSI/pull/521) → [main `288b16e`](https://github.com/july-rust/BoardSI/commit/288b16e19054102e752d433ab60088e669ea235a); PADS source-backed partial circular via [PR #523](https://github.com/july-rust/BoardSI/pull/523) exact HEAD `b113e8a41b96d8d87e17f8d7cb6a91f9eddb8918` → verified single-parent squash [main `63183cb`](https://github.com/july-rust/BoardSI/commit/63183cbcf94e30134c08aee795b51ae9057bdc08). This does NOT certify vendor-wide imports, electrical connectivity, full Qt GUI or SI.

## Why

D_055 implemented generic off-centre trace-to-via annulus overlap, but the source importer has a separate limitation. Its `scan_simple_via_defs` currently accepts only PADS V9.4 MILS round `-2/-1/0` definitions with **equal diameters**, and source route placements are interpreted as Through-Via `L1..MAXLAYER`. The original Qt-compatible `route_preview::parse` remains restricted. A separate opt-in PADS V9.4 round-pad partial-span source adapter is being added in PR #523; this does not imply Microvia process identification or full vendor grammar. `ViaKind` and `ViaSpan` in `boardsi-model` can represent more topology than the legacy PADS/Qt adapter.

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


## D_056 source-backed PADS partial-via slice — PR #523

The official [PADS Layout ASCII Format Specification 9.4, Via Definition pp. 39–40](https://www.freecalypso.org/pub/CAD/PADS/pdfdocs/Ppcb_ASCII.pdf) declares `name drill stacklines [drill_start] [drill_end]` plus `level size shape [inner_diameter]`. For partial vias, `drill_start` and `drill_end` are interpreted as the top and bottom of the **actual via span**; `-2/-1/0` are relative pad-stack selectors, and supported explicit inner physical layers override the uniform interior.

- New `via_source.rs` accepts **R circular conductive** definitions with source-owned full/partial spans; validates duplicate, malformed, out-of-range pad rows and inconsistent route-layer transitions. A `last_layer=4` assumption is not present in this parsing core.
- `route_preview::parse_generalized` is explicitly **opt-in** and populates `NormalizedVia` with `ViaKind::Through/Blind/Buried`, actual layer pad sizes and drill span. Adjacent-layer blind/buried spans do **not** by themselves identify Microvia manufacturing technology.
- `into_canonical` creates the via's actual `ViaSpan`, `Drill` and per-layer `PadstackLayer`; `connected_layers` remains empty unless independently proven. Existing center-based barrel graph paths intentionally exclude these generalized records.
- `trace_attachment::source_candidates_generalized` checks local positive-area annulus contact using each layer's actual diameter, preserving drill-hole exclusion, unrelated layer separation, declared net comparison and uncertain-boundary fail-closed.
- The distinct `boardsi-import-via-topology FILE.asc [--candidate-contacts]` diagnostic is **not** the Qt V1–V14 artwork protocol and does not assert global electrical connectivity. Original legacy Qt preview uses `route_preview::parse` unchanged.
- Synthetic actual-source-format and independent core tests cover 2/4/6/8-layer through vias, L2–L4/L2–L3 buried, L1–L3/L3–L6 blind spans, unequal layer pad diameters, missing middle pads, wrong route-layer transitions, and invalid source rows.

**Verified #523 HEAD:** 30/30 GitHub Checks SUCCESS, 10/10 pull-request workflow groups SUCCESS (Rustfmt/Clippy, Debug/Release workspace, Windows x64/ARM64 and macOS ARM64 native Rust, Linux/Windows/macOS Native Qt, docs, A032, independent M2 groups). A P2 independent review identified an ambiguous duplicate-name bypass; fixed in final HEAD and source regression added, review thread resolved, no remaining unresolved review threads. Protected squash main `63183cb` is verified=true/reason=valid with one parent. The code-main push event CI is separately tracked and must not be inferred from PR head CI.

## Still NOT IMPLEMENTED / NOT VERIFIED

- PADS **full** VIA grammar and actual vendor export acceptance: S square, A annular inner-diameter, RA/SA anti-pad, RT/ST thermal, backdrill, stacked/staggered physical geometry, source-proven microvia process class. PR #523 handles only **R circular** PADS V9.4 partial spans in an opt-in path.
- ODB++ / IPC-2581 / Gerber / KiCad via and other import source adapters: independent implementation and format-specific validation required.
- Replacing the old `ViaInstance.diameter_ticks` / `inner_pad` representation throughout **Qt V1–V14**, local reference/area/routing consumers and hard-coded D_014/D_015/D_016/D_036/D_037/D_038 4-layer fixtures. The new separate opt-in CLI prevents legacy consumers from silently treating a nonuniform or partial via as one uniform through-hole.
- Expanding source-backed local contact evidence into independently qualified global electrical connectivity, including same/cross-net open/short without name-only/XY-only assumptions. No generalized SOURCE via is automatically promoted into the old barrel/contact graph.
- Generic per-layer shapes beyond circles; arbitrary slot/drill, anti-pad, clearances, backdrill and stacked/staggered manufacturing structure.
- Full-board proven connected-layer inventory, actual plated barrel continuity, global connectivity graph promotion, solver/Port/S-parameter readiness, and measured validation.
- Full audit/remediation of every object importer including component placement and copper pours. These are separately gated follow-up work within Issue #520 and **must not** be marked complete by the first PR.

**Exit gate for D_056:** source adapters must independently prove actual source layer/shape/span/role, all downstream SOURCE/Qt consumers must preserve per-layer footprints or explicitly fail closed, and source-driven full connectivity plus independent negative/multi-format regression must pass. Keep [Issue #520](https://github.com/july-rust/BoardSI/issues/520) OPEN until those gates are met. A tested round-pad PADS source subset is **not** universal PCB input support.

## Evidence

- [D_056 parent issue](https://github.com/july-rust/BoardSI/issues/520)
- [Canonical model contract](canonical-board-model.md)
- [Generic D_054/D_055 contact geometry](trace-copper-contact.md)
- [D_054/D_055 SOURCE qualification](../validation/d054-d055-general-trace-contact-qualification.md)

Both protected code PR heads (#521 and #523) reported **30/30 GitHub Checks SUCCESS** and **10/10 pull-request workflow groups SUCCESS** each. These are repository-reported scoped CI results, not a claim of local real-PCB, measured hardware, or SI solver validation. The independent code-main push CI and documentation-only PR/push must be reviewed at their own exact SHA.
