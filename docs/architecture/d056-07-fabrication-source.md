# D_056_07 — PADS V9.4 Part-Decal PTH/NPTH/Slot SOURCE and Fabrication Contract

> 2026-11-09 KST. **Implementation scope is limited** to independently documented *PARTDECAL* padstack drill syntax and a format-neutral, non-electrically qualified fabrication model. This is **not** D_056_07 full acceptance. Do not claim production CAM, finished barrel, global Open/Short, Qt RHI rendering or SI solver support.

Parent [#520](https://github.com/july-rust/BoardSI/issues/520) · Manufacturing gate [#530](https://github.com/july-rust/BoardSI/issues/530) · Thermal baseline [D_056_06](d056-06-thermal-source.md)

## Authoritative, cross-checked PADS syntax

1. **Mentor Graphics**, *PADS Layout ASCII Format Specification*, V9.4, **Part Decal Definition Format**, printed pp. 36–38 (PDF pp. 35–37): [original PDF](https://www.freecalypso.org/pub/CAD/PADS/pdfdocs/Ppcb_ASCII.pdf).
   - `PAD pinno stacklines`, followed by the shape and optional drill fields.
   - `plated=N` explicitly denotes an unplated component pad drill; `P` or omitted means plated only in the documented applicable field position.
   - `slotori` angle 0–179.999 degrees, `slotlength` 1–1000 source units, `slotoffset` along the drilling axis. These parameters are for *part-decal padstack drills*, **not** a VIA-section record.
2. **KiCad** independent PADS parser: [pads_parser.cpp](https://docs.kicad.org/doxygen/pads__parser_8cpp_source.html), `parseSectionPARTDECAL`, checks P/N, drill and the three slot parameters. Independent cross-check does **not** prove every PADS revision.
3. Vendor **Via Definition Format**, printed pp. 39–40: `name drill stacklines [drill_start] [drill_end]`; does not independently establish any `P/N slotori slotlength slotoffset` tuple for *VIA* rows. Never transplant PARTDECAL syntax into the existing VIA parser.
4. **Backdrill / stacked/staggered microvia**: no independent native V9.4 declaration grammar confirmed in this scoped source review. A name containing `MICROVIA` is not process evidence. Any invented native field would be unsafe.

## Implemented bounded, transactional SOURCE reader

`crates/boardsi-import/src/pads_fabrication_source.rs`:

- Explicit V9.4 MILS header gate; UTF-8 only; bounded 8 MiB source, 200k lines, 4096-byte line length and 100k records.
- Strict zero-outline `I` part-decal, `T` terminals, `PAD` count, `-2` top row plus `-1/0` passive rows.
- **Intentionally restricted round-pad subset**: a positive drill number followed by explicit `P` or `N`, plus either no slot or an entire angle/length/offset triplet. Zero drill has no hole witness.
- Exact 0.01 mil dimension ticks and 0.001-degree angles; checks unsupported extra precision, invalid slot tuple and malformed records.
- Emits declaration, dimensions, decal/pin selector, physical SOURCE line and full source SHA-256; no footprint placement/board coordinates are inferred.
- The scanner refuses unsupported shapes, copper graphics, ambiguous optional fields, malformed/truncated data and version mismatch. These are a limited support boundary, **not** a claim that all real V9.4 PARTDECAL exports are supported.
- If the source is supported, `to_canonical_hole(span, copper_layers)` converts ticks to **exact rational metres**. A separately sourced physical span is mandatory; it does not claim the decal's drill spans the entire PCB.

## Canonical manufacturing representation

`crates/boardsi-model/src/fabrication.rs`:

- `FabricationHoleShape::Round | Slot`, physical `PhysicalLayerSpan`, independent `FabricationPlating`, qualified source SHA/line, and process intentions `Drilled | Backdrill | MicroviaTransitions`.
- Cross-stack structural validation for 2/4/6/8/64 copper layers, partial spans and backdrill inward direction; microvia stacking remains unknown unless independently declared.
- A nominal plated drill, apparent adjacent microvia positions, visual overlap, net assignment, anti-pad, or hole geometry **never** prove a manufactured conductive barrel or a verified graph edge.
- **No automatic electrical edge** is created. Even SOURCE-declared PTH is a candidate pending process evidence.
- Existing `BoardModel::drills` (round plated holes only), V1/V2 IPC and Qt artwork are **unchanged** until a versioned transport and the real viewer can represent slots without downcasting or false conduction.

## Tested and explicitly not implemented

| Capability | Current status |
| --- | --- |
| PADS V9.4 PARTDECAL limited R P/N round | Implemented SOURCE scanner / canonical candidate |
| PADS V9.4 PARTDECAL limited R P/N slotted tuple | Implemented SOURCE scanner / geometry parameters |
| Exact rational conversion and 2/4/6/8/64-layer span validation | Implemented / unit-test covered |
| Full PADS PARTDECAL shapes / graphics / mirrored placement | UNSUPPORTED; hard fail rather than silent skip |
| PADS VIA/ROUTE native NPTH / slot declaration | UNSUPPORTED; grammar not independently established |
| Native PADS Backdrill / stacked or staggered microvia parsing | UNSUPPORTED; grammar not independently established |
| Actual CAM slot endcap construction and rendered copper boolean | NOT_VERIFIED |
| Versioned slot/backdrill Qt IPC, CPU/RHI integrated artwork/picking | NOT_IMPLEMENTED |
| Global graph after physical barrel removal, full Open/Short | NOT_VERIFIED |
| Manufactured PCB conductive continuity, RLGC/S, measurement | NOT_VERIFIED |

## Regression and acceptance requirements

- Positive: declared PTH, NPTH, slotted NPTH with fractional orientation/length/offset; source provenance and exact rational dimensions across 2/4/6/8/64.
- Negative: invalid P/N, missing values, extra precision, malformed span, source mutation, token ambiguity and unsupported pad shape; all fail closed.
- No silent under-parse of a source record followed by a fabricated Copper edge.
- Code-change PR must pass repository-required Rust/Qt CI with exact final HEAD, review threads resolved and Verified Squash. Confirm independent `main` Push on the **exact merge SHA** separately.
- D_056_07 checklist remains **unchecked** until all Issue #530 acceptance criteria are completed, including real native grammar/fixtures and main Qt renderer path. This document records partial work rather than certifying completion.
