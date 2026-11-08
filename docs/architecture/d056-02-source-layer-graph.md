# D_056_02 — Source-evidenced multilayer copper graph foundation

**Date:** 2026-11-08 KST · **Parent:** [#520](https://github.com/july-rust/BoardSI/issues/520) · **Implementation gate:** [#529](https://github.com/july-rust/BoardSI/issues/529) · **Precondition:** [D_056_01 source audit](d056-01-source-evidence-audit.md).

## Implemented design (opt-in Rust, *not* certified electrical graph)

`crates/boardsi-import/src/layered_source_graph.rs` adds `from_pads_v94_source(bytes)` with a byte-bound SHA-256, declared physical layer count, source-owned per-layer conductive nodes, deterministic local contact candidate edges and explicitly unqualified via process inventory.

| Element | Evidence and limitation |
| --- | --- |
| Trace nodes | Validated straight/arc SOURCE strokes, physical layer, declared Net, line and original width. |
| Via-pad nodes | Actual normalized per-layer R/S/A conductive pads (or normalized round layer pads); omit anti-pads, missing intermediary pads and drills without copper. No uniform pad downcast. |
| SMD nodes | Restricted parsed source part-pad geometry; **source line explicitly unknown** where importer cannot supply one. Pin Net stays Unknown without an accepted source assignment. |
| Local contacts | Reuse `trace_contact::contact_candidates` and `trace_attachment::contact_candidates`. Positive-area only; uncertain tangencies, unsupported primitives, invalid layer geometry and source inventory fail closed. |
| Net relation | Same/Different/Unknown reports declaration *only*, never inferred electrical short or complete net continuity. |
| Via barrel | `UnverifiedBarrel` stores declared span and actual landing-pad nodes but **does not produce electrical edges**. Parser `plated: true` is not manufacturing proof. |
| Electrical graph | `verified_electrical_edges` intentionally empty pending D_056_03 independent process and whole-board qualification. No manufactured continuity or SI claim. |

The geometry is handled by a layer-keyed generic algorithm, not fixed 4L branches. Source grammar support is **restricted PADS V9.4**, not generalized recognition of ODB++/IPC-2581/Gerber/KiCad and not guaranteed full PADS conductive coverage.

## Independent test acceptance and exclusions

The module includes source-owned 2/4/6/8/64-layer Through and partial L2–L3 synthetic cases, absence of an inferred middle pad, contradiction in drill span, changed digest and absence of verified barrel edges. Source fixtures specify expected node/edge counts independently of the production geometric predicates. CI must run on exact PR HEAD; record actual result before setting a checklist checkbox.

The current shape/contact kernel and resulting graph do **not** qualify polygon pour islands, copper fills, via-to-via or SMD-to-SMD area contact, manufacturing barrel/stack process, thermal spokes, source-confirmed slot/backdrill/NPTH, global open/short, Qt V1–V14 artwork, 30 GHz electrical performance or physical-board acceptance. It is not a full-board inventory: consumers may not promote its local candidates into canonical `ConnectivityGraph` verified edges.

## Next gate

[D_056_03 / Issue #529](https://github.com/july-rust/BoardSI/issues/529) must independently qualify electrical Open/Short/conflict/cut-set witnesses, missing source/physical process, polygon copper and graph connected components before claiming verified electrical connectivity. A Source-declared span and a same-name Net are insufficient.
