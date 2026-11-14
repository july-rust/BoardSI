# E_022 — ODB++ Source-qualified local contact / design graph qualification

**Evidence date:** 2026-11-14 KST. **Stage:** E_022. **Scope:** opt-in positive L straight Trace versus positive round P Pad, one native STEP, supported ODB++ 8.1-U4 grammar subset; same physical copper layer. This is a limited SOURCE *design* graph, not a manufactured PCB or SI proof.

## Implemented architecture

- Source: exact ODB++ OdbFileMap -> import_unpacked or import_unpacked_with_eda -> existing format-neutral BoardModel. Unsupported arc, surface, repeated STEP, other pad shapes, fabricated via/drill and uncertain grammar fail closed for this specifically scoped adapter. The general ODB importer remains unchanged.
- One immutable board and file-SHA manifest bound to step, SHA-256, Layer ObjectId, Trace/Pad ObjectId, actual source line and optional EDA FID design Net. The digest returned by a previous import can be passed as an expected digest to reject a mutated/re-imported board.
- Format-neutral representation: preexisting LayeredSourceGraph (CopperNode, GeometryContact/TraceSmdPad, DeclaredNetCheck SAME/DIFFERENT/UNKNOWN) and preexisting electrical_graph_qualification::qualify with SourceCoverage::Partial. A confirmed local positive overlap yields only QualifiedEdgeKind::SourceLocalPositiveArea. Never prepopulate verified_electrical_edges.
- Actual finite-width Straight Trace with round endcaps versus solid Round Pad: strict interior disk/capsule positive overlap. Calculated from relative source-coordinate ticks using conservative binary64 guard. Near tangent, source-tick precision limit, overflow or unsupported frame returns UNKNOWN/no edge. A missing candidate is **not** proof of electrical Open.
- No cross-layer XY contact, Net-name-only contact, plating or barrel qualification. No canonical Graph or SI solver mutation, no global open/short, no actual vendor/export or fabrication oracle.

## Independent synthetic oracle matrix

The code adds these tests inside boardsi-import/src/odb_local_graph.rs:

| Test | Oracle | Required |
| --- | --- | --- |
| independent_2_4_6_8_64_layer_positive_oracle | N strict positive local edges and N connected components for N layers | PASS |
| offcentre_endpoint_and_midspan_contacts | Midspan/off-center/general-angle/endpoint positive contact | PASS |
| separate_and_tangent_are_not_electrical_edges | No positive edge from distinct/tangent copper | PASS |
| same_xy_other_layer_is_not_contact | No layer-crossing connection from identical XY | PASS |
| declared_cross_net_conflict_and_unknown_net | Positive local different-Net conflict, Unknown Net preserved | PASS |
| same_net_does_not_invent_separated_edge | Same Net cannot fabricate contact | PASS |
| stale_sha_reimport_and_bad_eda_fail_closed | Stale manifest and unsupported FID rejected | PASS |
| unsupported_shapes_and_conductors_fail_closed | Arc/unsupported round-line symbol rejected | PASS |
| exact_source_identity_and_repeat_determinism | Source identity and edge order stable | PASS |

Additional required regressions: preexisting PADS graph, ODB Source Import/E_019/E_020, E_021 Qt Artwork/Picking/Import/ReImport and native runtime. All results must be recorded from actual CI, never inferred from this plan.

## CI/review/merge evidence ledger

- Code PR: **PENDING**
- Exact code PR HEAD SHA: **PENDING**
- fmt/clippy/workspace tests: **NOT_RUN until CI evidence**
- Linux x64 / Windows x64 / Windows ARM64 / macOS ARM64 native CI: **NOT_RUN until CI evidence**
- Independent review and unresolved threads: **NOT_VERIFIED**
- Protected Verified Squash and exact merged code-main Push: **NOT_VERIFIED**
- Commercial ODB++/cross-format oracle, manufactured barrel, electrical/global Open/Short and SI: **NOT_VERIFIED / excluded from this slice**

Update this section with real, exact evidence before marking the E_022 scope complete. Do not promote any status from an unsupported source.

## Closure boundary

E_022 is only one SOURCE-local Trace/RoundPad positive contact slice. Historic issue #578 is administratively CLOSED; #10 and #520 remain OPEN until their full acceptance gates are met. E_023 independently owns real Vendor fixture + cross-format oracle. E_024 independently owns whole-product ODB gate. E_021 has separate unresolved merged-main CI/signature evidence; do not repair E_021 by claiming E_022 CI also proves E_021.
