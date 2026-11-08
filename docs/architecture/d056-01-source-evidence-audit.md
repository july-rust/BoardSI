# D_056_01 — Existing Implementation Audit & Universal PCB Source Evidence Contract

**First audited:** 2026-11-08 KST. **Audit baseline:** protected `main@4118e2c0a525ba732e16ea9062d1b22e05c2af30` (verified squash of [PR #528](https://github.com/july-rust/BoardSI/pull/528)). **Status of this record:** source inspection and interface/acceptance design; not a runtime test or implementation of D_056_02–12.

[Parent #520](https://github.com/july-rust/BoardSI/issues/520) · [Canonical model](canonical-board-model.md) · [D_056 implementation chronology](input-normalization.md) · [D_054/D_055 evidence](../validation/d054-d055-general-trace-contact-qualification.md) · [Sole checklist](../development-checklist.md#d_056)

## 1. Findings — code paths, not PR-title assumptions

| Subsystem and inspected source | Current verified-in-code capability | Scope / debt / subsequent owner |
| :--- | :--- | :--- |
| [`boardsi-model/src/identity.rs`](../../crates/boardsi-model/src/identity.rs) | `SourceFormat`, `SourceFile{logical_path,sha256}`, `SourceIdentity` (format/version/files/importer/conditions), compound `ObjectId`, `ConversionQuality` and `ImportCapability` are already defined. | The data types are **not** proof that non-PADS adapters exist or that every source record is covered. Per-record source line is not a mandatory `ObjectId` field. D_056_02/08–11. |
| [`boardsi-model/src/lib.rs`](../../crates/boardsi-model/src/lib.rs) and [`validation.rs`](../../crates/boardsi-model/src/validation.rs) | `PadShape::{Circle,Rectangle,Annulus,Custom}`, `PadRole::{Conductive,AntiPad,Clearance}`, `ViaKind`, `ViaSpan`, `Drill`, `Via.connected_layers`, `ConnectivityGraph` and limited `ConnectionEvidence`. Validator checks span, pad role, plated flag and references. | Structural validation and edge labels cannot alone establish a manufactured conductive path. Current `SameLayerContact` validator constrains positions to equality, and the preview graph still handles an endpoint-centred subset; full finite-width/area graph is #529. |
| [`route_preview.rs`](../../crates/boardsi-import/src/route_preview.rs) | Legacy `parse()` is restricted PADS V9.4 MILS Artwork; `parse_generalized()` is opt-in and preserves layer-resolved Via shapes. Source SHA is calculated in `into_canonical`. | `scan_simple_via_defs` requires equal round pads and `-2/-1/0` selectors. Generic data must never be passed to that legacy uniform-diameter transport as if complete. `MAXLAYER` creates placeholder Copper layers with missing z/thickness/material; not a proven physical stackup. #531/#532. |
| [`via_source.rs`](../../crates/boardsi-import/src/via_source.rs), [`via_shapes.rs`](../../crates/boardsi-import/src/via_shapes.rs), [`via_topology.rs`](../../crates/boardsi-import/src/via_topology.rs) | PADS V9.4 supported R/S/A copper + RA/SA anti-pad and source-declared through/partial drill spans; geometry is normalized by actual participating layer, independently validated within 2–64 Copper layers. | `normalize_shapes` assigns `plated: true` on the supported PADS VIA path: **interpret it only as the restricted parser's expected plated-via geometry, not independent fabrication/plating proof**. RT/ST, special hole processes, source-confirmed microvia process and vendor-file qualification remain #530. |
| [`trace_attachment.rs`](../../crates/boardsi-import/src/trace_attachment.rs) | Generic straight/arc finite-width SOURCE-local trace-to-via/pad positive-area candidates, with per-layer copper pads, hole exclusion and anti-pad rejection. | `PositiveArea` is a geometry classification; unknown nets, incomplete pours, plating and near-boundary uncertainty must not become verified graph edges or shorts. #529. |
| [`connectivity_preview.rs`](../../crates/boardsi-import/src/connectivity_preview.rs) | Legacy SOURCE endpoint and selected barrel witnesses. Explicit guard excludes SOURCE-declared generalized vias from old Through-only graph path. | Graph is not global Open/Short proof; `ViaBarrel` is added only for the restricted qualifying legacy path. Needs evidence-qualified via barrel and polygon/finite-width contacts. #529. |
| [`d056_via_layers.h`](../../apps/boardsi-qt-prototype/d056_via_layers.h), [`d056_via_viewer.cpp`](../../apps/boardsi-qt-prototype/d056_via_viewer.cpp) | Separate opt-in Qt layer-aware geometry diagnostic with explicit non-SI state and source validation. | Separate viewer, **not** main Artwork V1–V14/RHI integration. #531. |
| [`boardsi-import/src/lib.rs`](../../crates/boardsi-import/src/lib.rs) | Header signature detection plus PADS-specific modules. | No production ODB++ / IPC-2581 / Gerber+drill / KiCad input adapter proven by this inventory. A format discriminator is not an importer. #532. |

**Classification (do not conflate with defects):** `GENERIC` = contract supports arbitrary eligible layer identifiers/footprints; `LEGACY_LIMITED` = intentional proven subset; `HARD_CODED/REWORK` = fixed semantics leak into a generalized consumer; `UNSUPPORTED` = known unimplemented source grammar; `UNKNOWN` = required evidence absent. Fixed-size **synthetic tests** are not product hardcoding; PADS reserved `-2/-1/0` are selectors, **not physical layer numbers**.

**Specific risks to close in later slices:**
1. Uniform `ViaInstance.diameter_ticks/inner_pad` and legacy `parse()` are an intentional compatibility gate, not a suitable generic padstack representation (#531).
2. `MAXLAYER`-derived `Layer` placeholders do not prove the board's actual Stackup z/material/thickness (#532).
3. `plated: true` in the PADS shape normalizer and the model's boolean `Drill.plated` must not be treated as independent evidence of a manufactured barrel (#529/#530).
4. `connectivity_preview` guards exclude generalized source via records; full electrical graph absent (#529).
5. Source identity is available, but per-record parser diagnostics and full import coverage need adapter-specific contracts (#529/#532).
6. Separate Qt geometry diagnostic does not carry the generalized data into the main production transport (#531).
7. PADS thermal/spoke/slot/backdrill processing cannot be invented from undocumented tokens (#530).

## 2. Universal Source Evidence Contract — target, not shipping API

**Data pipeline:**

`Source bytes → native parser → source-evidenced normalized objects → Canonical BoardModel → local geometry candidates → evidence-qualified electrical graph → versioned Qt transport → independent SI gate`

### 2.1 Identity and coverage

A future format adapter MUST supply or explicitly mark unavailable:

| Field / semantic | Required policy |
| :--- | :--- |
| Source identity | Native format/version, bounded input, verified SHA-256 of relevant source files, stable **logical** path (no private workstation absolute paths), importer version, deterministic parse conditions. Existing `SourceIdentity` remains owner. |
| Object identity / location | `ObjectId` with stable native object key and part ID; native line/record/section pointer when the format permits; retain source-to-canonical mapping. Absent location = explicit unavailable, **not** fabricated line 0. |
| Layer mapping | Distinguish native layer selector/name, resolved physical Copper layer order and source evidence; unknown/missing/mismatched stackup cannot be silently mapped. PADS `-2/-1/0` are relative padstack tokens. |
| Shapes / roles | Source-defined width, boundary, per-layer pad shape, drill hole, copper vs anti-pad vs clearance; no inferred intermediate pads. Preserve unhandled conductive tokens as a **fail-closed diagnostic**. |
| Via manufacturing | Declared span and bore are geometry; plating, barrel continuity, stacked/microvia process, backdrill extent are **separate assertions** requiring source/process evidence. `plated=true` alone is insufficient physical qualification. |
| Net and graph | Native net name = declaration only. No verified connectivity solely from shared net, XY, nominal span, screenshot or renderer overlap. |
| Completeness | Use existing `ImportCapability` independently for geometry/net/pin-pad/stackup/reference/model and explicitly list source object coverage, skipped objects, unsupported features and mutation evidence. |

### 2.2 Qualification and severity

**Do not add redundant enums in D_056_01.** Map conceptual terms onto existing types, adding a versioned public API only with independent later tests:

- Input semantics: **SUPPORTED** = accepted source grammar for a specified subset; **PARTIAL** = proven partial inventory; **UNSUPPORTED** = recognized but not safely interpreted; **UNKNOWN** = required source/process record absent; **INVALID** = contradictory/malformed source. Existing `CapabilityState::{Complete,Partial,Missing,Conflicting}`, `ValidationIssue::Severity`, and guarded parse errors should be reused before creating new public discriminants.
- Geometric semantics: **CANDIDATE** = proven positive-area overlap for supported source-local shapes, without plating or whole-board evidence; **REJECTED** = independently proved separation, anti-pad/void, out-of-span, invalid geometry; **UNVERIFIED** = boundary/precision ambiguity, missing geometry or missing process evidence.
- Electrical semantics: **VERIFIED** only with explicit positive conductive landings and a qualified plated-barrel/transition process contract, valid graph vertices, identity/layer consistency and independent negative cases. `ConnectionEvidence` is a representation, not an external physical measurement.
- Source provenance and validation are orthogonal to the experimental/physical SI acceptance gate. **Never claim measured hardware electrical continuity or 30 GHz S-parameters from synthetic graph CI.**

### 2.3 Proposed adapter interface obligations (architecture only)

Each native adapter MUST report `source_identity`, `capability`, deterministic `objects`, `layer_mapping`, `diagnostics`, `unsupported_records`, and `source_location` per available item; normalized Copper, padstack, hole and Via geometry MUST be mapped into a shared Canonical model without format-specific graph logic. Reject duplicate/conflicting identifiers, missing mandatory layers, non-finite/overflow dimensions, unrecognized conductive forms and stale file hashes.

No native parser code is added here. An adapter can return a **PARTIAL canonical view** only if unsupported portions remain explicit and all electrical/SI outputs depending on them are blocked. Gerber+drill normally cannot guarantee net names, layer order or full stackup, so the adapter must preserve those as absent instead of inferring a netlist.

## 3. Regression / evidence matrix for D_056_02–12

| Case family | Positive oracle | Fail-closed / negative oracle | Stage |
| :--- | :--- | :--- | :--- |
| Physical Copper layer counts 2/4/6/8/64 | Same geometry with layer-order invariant normalization; through and partial L2–L3/L2–L4 | Missing or duplicate physical layer, out-of-range native selector, reversed span | 02/03, 04/05, 08–11 |
| Via pads and process | Per-layer different R/S/A dimensions, drill-only bore removed, explicit conductive roles | No inner pad, only RA/SA void, plating unknown, conflicting drill/stack, false microvia manufacturing class | 02/03, 06/07 |
| Arbitrary geometry | One generic straight/arc overlap algorithm, off-centre landed pad, polygon with hole | Tangency/boundary uncertainty, overlapping drill opening only, disconnected polygon island, unrelated layers | 02/03, 05 |
| Graph / declarations | Independent expected nodes/edges/components for same-net evidence | Cross-net conflict **candidate**, no graph edge from shared name or XY, disconnected barrel, no manufactured proof | 02/03 |
| Qt | Versioned per-layer shape payload; 2/4/6/8/64 and CPU/RHI equivalent display | No pad layer, non-copper RA/SA, truncation, reimport cancellation/stale SHA | 04/05 |
| Native sources | Public/spec-authorized valid fixture per adapter, source mutation, cross-format canonical geometry equivalence where possible | Unknown tokens, missing net/stackup, ambiguous drill/pad map, unsupported source revision; no private board upload | 06–11 |
| Invariant / performance | D_001–D_055 historical test suites stay runnable; bounded files/objects, deterministic IDs and reimports | Unbounded bytes/records, invalid UTF/encoding, object-order mutation loss, stale source digest | 02–12 |

**Oracle requirements:** independent expected components/contacts by layer and source ID; intentional negative and mutation fixtures; no oracle derived exclusively from the same production function. Cross-format tests may compare only fields genuinely present in both native sources. Tested shape/layer combinations are samples of the generic algorithm, not a claim of exhaustive physical manufacturing coverage.

## 4. Twelve-step execution ownership

| Step | Concrete engineering output / prerequisite | Evidence gate |
| :--- | :--- | :--- |
| D_056_01 | This audited source-evidence contract + regression matrix; architecture and sole checklist linked. | Docs-only PR, lightweight required checks, verified protected squash, independent main push audit. No graph claim. |
| D_056_02 | #529 source-evidenced graph vertices and reusable per-layer contact edges. | 2/4/6/8/64 positive/negative, real source SHA, fail-closed. |
| D_056_03 | #529 graph Open/Short conflict/cut-set and independent oracle. | Correct negative witnesses, D_001–D_055 regression, close #529 only when accepted. |
| D_056_04 | #531 versioned Canonical padstack + via IPC. | No lossy downcast; wire length/identity/reimport rejection. |
| D_056_05 | #531 actual main Qt CPU/RHI integration. | Native supported platforms; no anti-pad copper and no false graph; close #531 when accepted. |
| D_056_06 | #530 source-cited PADS RT/ST thermal spoke grammar. | Exact count/orientation/clearance or explicitly unsupported. |
| D_056_07 | #530 source-proven slot/NPTH/backdrill/microvia manufacturing metadata. | Negative process evidence; no fabricated grammar; close #530 only when accepted. |
| D_056_08 | #532 ODB++ adapter. | Authorized format specification/fixture and canonical oracle. |
| D_056_09 | #532 IPC-2581 adapter. | Format-specific negative and versioned source contract. |
| D_056_10 | #532 KiCad PCB adapter. | Valid source layer/net/footprint/via mapping and mutation tests. |
| D_056_11 | #532 Gerber+drill adapter. | Net/stackup missing remain unknown; no manufactured inter-layer proof. |
| D_056_12 | Cross-source geometry comparison, whole regression, verified docs and protected merge. | Close #532 then parent #520 **only** after all original independent issue acceptance criteria pass. |

## 5. Evidence boundary and follow-up protocol

This **D_056_01 record** is an inspected code/design audit; it does **not** claim a fresh Rust/Qt build, empirical PCB measurement, full file corpus audit, compliance with unsupported vendor grammar, or completed D_056_02–12. Check CI against the exact PR SHA and the protected post-merge `main` push event separately. Source inspection identifies actionable assumptions; a fixed 4-layer test fixture is legitimate if it does not leak into production logic. Preserve [the single checklist](../development-checklist.md) and the existing verified historical records.
