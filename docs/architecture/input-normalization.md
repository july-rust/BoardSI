# D_056 — Generalized Input Normalization and Via Topology Contract

**First checked:** 2026-11-08 KST. **State:** THREE CODE SLICES MERGED, exact PR-head CI VERIFIED; D_056 OVERALL OPEN. Core foundation [PR #521](https://github.com/july-rust/BoardSI/pull/521) → [main `288b16e`](https://github.com/july-rust/BoardSI/commit/288b16e19054102e752d433ab60088e669ea235a); PADS source-backed partial circular via [PR #523](https://github.com/july-rust/BoardSI/pull/523) exact HEAD `b113e8a41b96d8d87e17f8d7cb6a91f9eddb8918` → verified single-parent squash [main `63183cb`](https://github.com/july-rust/BoardSI/commit/63183cbcf94e30134c08aee795b51ae9057bdc08). Additional source-shaped VIA [PR #526](https://github.com/july-rust/BoardSI/pull/526) exact final HEAD `2f65d08321d183f4a446163f7b05705d5b5fb7f9` (38/38 GitHub checks and 13/13 workflows SUCCESS, completed automatic code review with no findings, 0 unresolved threads) → protected single-parent verified squash [main `7e10d7b`](https://github.com/july-rust/BoardSI/commit/7e10d7ba07d6c9e0f959eb3e2e4be570af6dd88b). The separate post-merge main Push CI is a distinct gate, not inferred from PR-head results. This does NOT certify vendor-wide imports, electrical connectivity, per-layer Qt display or SI.

## D_056_01 inspected-source audit and future adapter contract (2026-11-08 KST)

[**D_056_01 source evidence audit, concrete codepath inventory, fail-closed contract, regression matrix and D_056_02–12 gates**](d056-01-source-evidence-audit.md) inspects protected `main@4118e2c0a525ba732e16ea9062d1b22e05c2af30` following merged [opt-in Qt source-layer viewer #528](https://github.com/july-rust/BoardSI/pull/528). It is a **separate source-code and architecture audit, not a new electrical graph implementation**.

Key non-equivalences to preserve:
- `SourceIdentity`/SHA and `ObjectId` already exist, but do not guarantee per-record provenance, complete import inventory, or native ODB++/IPC-2581/Gerber/KiCad readers.
- The PADS adapter's `plated: true` and Canonical `Drill.plated` boolean are **not independent plated-barrel fabrication proof**; actual connected layers require verified local landing and process evidence.
- Placeholder Copper layers synthesized from `MAXLAYER` are **not** a measured or source-qualified physical stackup.
- The D_056 Qt V2 **separate diagnostic viewer** now exists; the legacy main Qt Artwork V1–V14 transport is still not per-layer shape integrated. Review #531 before calling it done.
- Geometry-only PositiveArea, source Net label and shared XY are **not** full-board electrical verification; graph work remains #529.

The 12-stage named plan is D_056_01 through D_056_12. Parent #520 and child #529/#530/#531/#532 stay open until their separate acceptance tests are met.

## Why

D_055 implemented generic off-centre trace-to-via annulus overlap, but the source importer has a separate limitation. Its `scan_simple_via_defs` currently accepts only PADS V9.4 MILS round `-2/-1/0` definitions with **equal diameters**, and source route placements are interpreted as Through-Via `L1..MAXLAYER`. The original Qt-compatible `route_preview::parse` remains restricted. A separate opt-in PADS V9.4 source adapter was added in merged PR #523 and subsequently extended in merged PR #526 with source-backed R/S/A conductive and RA/SA plane-void shapes. This does not imply Microvia process identification, thermal reconstruction or full vendor grammar. `ViaKind` and `ViaSpan` in `boardsi-model` can represent more topology than the legacy PADS/Qt adapter.

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

## Third source-shaped copper/void geometry slice — merged PR #526

- `via_shapes.rs` introduces `NormalizedViaShapes` and per-layer shape/role footprints: **R** round copper, **S** square copper, **A** annular copper with **explicit distinct inner diameter**, **RA/SA** circular/square plane **AntiPad** (NOT conductive copper). All use source-owned real physical layer IDs and drill-span normalization.
- `via_source.rs` preserves the source-defined pad shape and role through `route_preview::parse_generalized`, `into_canonical` constructs `PadShape::{Circle,Rectangle,Annulus}` and `PadRole::{Conductive,AntiPad}` with exact Rational SI dimensions. Circular-only `normalize` fails closed for nonround profiles.
- SOURCE-local trace-to-via finite-width candidates account for actual square boundary minus drilled hole, and annular pad **inner opening rather than merely drill diameter**. Anti-pad layers cannot yield conductive candidates. Old endpoint-only barrel graph rejects the newly generalized provenance; `Via.connected_layers` is not invented.
- `boardsi-import-via-topology FILE.asc [--candidate-contacts]` emits **version V2** diagnostic for shaped via source records including annular inner diameter; R-only output remains V1. This wire is deliberately **separate from Qt Artwork V1–V14**.
- Final PR HEAD `2f65d08321d183f4a446163f7b05705d5b5fb7f9`: **38/38 GitHub Checks SUCCESS, 13/13 workflow groups SUCCESS**, automatic code review completed with no findings, no unresolved inline review threads. Verified one-parent protected squash main `7e10d7ba07d6c9e0f959eb3e2e4be570af6dd88b`. Post-merge `main` checks run separately and are not yet certified here.
- The verified scope is a **synthetic, restricted PADS V9.4 source format subset only**; no manufacturer-file, thermal-spoke, actual metallization, whole-board connectivity, production GUI or physical SI certification is implied.

## Still NOT IMPLEMENTED / NOT VERIFIED

- PADS **full** VIA grammar and actual vendor export acceptance: **RT/ST thermal spokes** and other unsupported shapes/constructs, backdrill, stacked/staggered physical geometry, source-proven microvia manufacturing class. PR #526 implements a restricted **R/S/A copper and RA/SA anti-pad source subset**, not the full format.
- ODB++ / IPC-2581 / Gerber / KiCad via and other import source adapters: independent implementation and format-specific validation required.
- Replacing the old `ViaInstance.diameter_ticks` / `inner_pad` representation throughout **Qt V1–V14**, local reference/area/routing consumers and hard-coded D_014/D_015/D_016/D_036/D_037/D_038 4-layer fixtures. The new separate opt-in CLI prevents legacy consumers from silently treating a nonuniform or partial via as one uniform through-hole.
- Expanding source-backed local contact evidence into independently qualified global electrical connectivity, including same/cross-net open/short without name-only/XY-only assumptions. No generalized SOURCE via is automatically promoted into the old barrel/contact graph.
- Remaining generic source pad/clearance shapes beyond the implemented circle/square/annulus/anti-pad subset, arbitrary slot/drill, thermal clearance/spokes, backdrill and stacked/staggered manufacturing structure.
- Full-board proven connected-layer inventory, actual plated barrel continuity, global connectivity graph promotion, solver/Port/S-parameter readiness, and measured validation.
- Full audit/remediation of every object importer including component placement and copper pours. These are separately gated follow-up work within Issue #520 and **must not** be marked complete by the first PR.

**Exit gate for D_056:** source adapters must independently prove actual source layer/shape/span/role, all downstream SOURCE/Qt consumers must preserve per-layer footprints or explicitly fail closed, and source-driven full connectivity plus independent negative/multi-format regression must pass. Keep [Issue #520](https://github.com/july-rust/BoardSI/issues/520) OPEN until those gates are met. A tested round-pad PADS source subset is **not** universal PCB input support.

## D_056_02 opt-in source layer copper graph (2026-11-08 KST)

[PR #535](https://github.com/july-rust/BoardSI/pull/535) adds a PADS V9.4 **source-local** `layered_source_graph` module and [scope/negative evidence contract](d056-02-source-layer-graph.md). It retains source SHA, declared Copper layer, Source record line where available, R/S/A conductive footprints, Trace/SMD geometry candidates, and explicit unverified Via Barrel spans. Native R/S/A/RA/SA source evidence is not a global electrical graph. Bare drill span and parser `plated=true` do not certify manufacture; `verified_electrical_edges` stays empty. This is the D_056_02 limited foundation, not D_056_03 Open/Short or D_056 full import acceptance. The protected code squash is [main 2eddeac](https://github.com/july-rust/BoardSI/commit/2eddeacbb985c14313544c958e2c5a0707a89aa0).

## D_056_03 opt-in source-scope graph qualification (2026-11-08 KST)

[PR #538](https://github.com/july-rust/BoardSI/pull/538) adds the standalone Rust [electrical qualification algorithm](../../crates/boardsi-import/src/electrical_graph_qualification.rs) and independent [source-scoped design and test contract](d056-03-electrical-graph-qualification.md). It validates source SHA, actual node/layer and local contact evidence, and admits cross-layer edges only with an independently referenced qualified design/process witness; unknown plating remains disconnected/unknown. Connected components, open *candidates* or scoped qualified disconnects, declared-net conflict paths, bridges and articulation points have an independent 2/4/6/8/64-layer test matrix. The protected [main code squash `ca87a4d`](https://github.com/july-rust/BoardSI/commit/ca87a4d6b08bc75ccff9827bcfdf6d0ffed64cc4) is signed/verified.

**Boundary:** D_056_02 still does not import a certified whole-board conductive inventory (polygon pours, special manufacturing forms and process evidence remain restricted or unknown). D_056_03 does not prove fabricated-barrel continuity, entire-board Open/Short or SI readiness. Keep #529 and #520 OPEN. The D_056_03 standalone code suite is a verified *subset*, not a closure of the full electrical graph gate.

## Evidence

- [D_056 parent issue](https://github.com/july-rust/BoardSI/issues/520)
- [Canonical model contract](canonical-board-model.md)
- [Generic D_054/D_055 contact geometry](trace-copper-contact.md)
- [D_054/D_055 SOURCE qualification](../validation/d054-d055-general-trace-contact-qualification.md)

The first two protected code PR heads (#521 and #523) each reported **30/30 GitHub Checks SUCCESS** and **10/10 pull-request workflow groups SUCCESS**; third code PR #526 final head reported **38/38 checks and 13/13 workflows SUCCESS**. These are repository-reported scoped CI results, not a claim of local real-PCB, measured hardware, or SI solver validation. The independent code-main push CI and documentation-only PR/push must be reviewed at their own exact SHA.
