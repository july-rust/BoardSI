# D_056_04 — Versioned Canonical Padstack/Via Rust → Qt IPC contract

**First checked:** 2026-11-09 KST · **Evidence date:** 2026-11-09 KST · **Scope:** supported geometry-only, source-bound software transport. Parent [#520](https://github.com/july-rust/BoardSI/issues/520), Canonical/Qt implementation [#531](https://github.com/july-rust/BoardSI/issues/531), code [PR #549](https://github.com/july-rust/BoardSI/pull/549). Depends on accepted [D_056_03 SOURCE graph gate](d056-03-electrical-graph-qualification.md); **DOES NOT certify fabricated Via-barrel continuity, board-wide Open/Short, real-board netlists, physical RLGC/S or SI**.

## In-scope implementation (merged into protected main)

- Producer: `crates/boardsi-import/src/canonical_padstack_ipc.rs` (`encode`, `encode_pads_source`) and opt-in `boardsi-import-canonical-padstack FILE.asc` CLI, isolated from the old artwork protocol.
- Consumer: `apps/boardsi-qt-prototype/d056_canonical_ipc.h` contains independent, bounded, transactional V1 decoding. `d056_canonical_ipc_test.cpp` checks independently authored multilayer/negative payloads and executes the actual Rust CLI against the public PADS fixture.
- Build: `apps/boardsi-qt-prototype/CMakeLists.txt` adds `boardsi-d05604-canonical-ipc-test` and CTest `qt-d05604-canonical-padstack-ipc`; when `BOARDSI_CANONICAL_IMPORTER` is supplied, adds `qt-d05604-rust-canonical-roundtrip`. The native [Qt runtime workflow](../../.github/workflows/qt-runtime.yml) builds the Rust producer and runs both checks across supported native runners. Existing Artwork V1–V14 and its acceptance tests remain unchanged.
- The producer validates the Canonical board before serialization and obtains the original single-file SHA-256 from `SourceIdentity`. It retains source format/version, importer version, relative source path, import conditions and coordinate frames with **exact Rational** metres-per-tick and origin. The consumer verifies the expected source SHA and the complete payload SHA before installing the result. Source-grid XY ticks are carried as **signed integers**; no float rounding is used for their wire representation.
- Each physical copper layer has a stable identifier and actual Canonical stackup order. Via records link separate Padstack/Drill objects and an explicit ViaKind + first/last physical layer; each pad carries its own **physical layer, role, shape and Rational dimensions**. Absence of a middle-layer landing is **not replaced by a synthesized pad**. Annulus `inner_diameter` remains distinct from Drill diameter. AntiPad/clearance records are never promoted to conductive copper.
- Source `Drill.plated` is transported **only as a declaration** together with the explicit `PLATING_UNVERIFIED` marker. A Via must carry `CONNECTIVITY_UNKNOWN`; any pre-populated `connected_layers` makes the V1 encoder reject the board. No barrel, verified electrical edge or solver result is invented.

## Stable V1 wire framing

A UTF-8, newline-delimited protocol is used **only for the opt-in Canonical padstack transport** (not the separate D_056 V1/V2 via viewer wire and not the legacy Qt Artwork V1–V14).

Header field order:

```text
BOARDSI_CANONICAL_PADSTACK_V1
|original_source_SHA256
|hex(source_format)
|hex(format_version)
|hex(importer_version)
|hex(relative_logical_path)
|coordinate_frame_count
|physical_copper_layer_count
|padstack_count
|drill_count
|via_count
|source_condition_count
|body_byte_length
|body_record_count
|body_SHA256
|GEOMETRY_ONLY
|SI_NOT_RUN
```

This is **one physical header line**, shown split only for readability. The body contains ordered records:

| Record | Required preservation |
| :--- | :--- |
| `I` | SOURCE import-condition key/value, hex UTF-8 |
| `F` | frame ID, decoded UTF-8 unit label, Rational scale/origin, quarter-turn, mirror, layer-order provenance |
| `L` | physical copper layer ordinal, stable ID, Canonical order |
| `S` | padstack stable ID and number of subsequent pads |
| `P` | owner padstack, layer ordinal, Conductive/AntiPad/Clearance role, Circle/Rect/Annulus shape, exact Rational dimensions |
| `D` | drill ID, Rational diameter, actual span, source-declared plated state; explicitly `PLATING_UNVERIFIED` |
| `V` | via ID, padstack ID, drill ID, declared/unknown Net, frame, signed ticks, span, kind; explicitly `CONNECTIVITY_UNKNOWN` |
| `END` | exact Via count, `SOURCE_ONLY`, `GLOBAL_CONNECTIVITY_NOT_VERIFIED` |

Identifiers and text fields are encoded as hex UTF-8 (IDs separate source-object and part by one NUL); the decoder must decode the unit label, not retain wire hex. Record order is validated; Qt refuses a partial/changed/truncated/extra record or wrong complete-payload digest. Decoding clears the old result first and publishes the new object **only after complete validation**, so a stale/failed source is never reported as accepted.

**Bounds:** at most 64 physical copper layers, 10,000 Vias/Padstacks/Drills, and 4 MiB of V1 payload. Limits are resource fences, not proof that an arbitrary vendor board is fully supported.

## Qualified positive and negative matrix

- Independent Qt synthetic V1 cases: 2/4/6/8/64 copper layers and various Through, Blind, Buried spans (including L17–L41). Tests preserve per-layer unequal pads, a missing physical landing, a non-conductive intervening AntiPad, square/rectangular geometry and annulus with an independent inner aperture.
- Negative Qt cases include wrong source SHA, altered version, forged connectivity/plating state, impossible annulus, incorrect Via/END count, truncation and extra bytes. Rust source fixture tests check deterministic encoding and source-change SHA differentiation. A real Rust CLI → Qt decoder test validates the same original PADS V9.4 MILS frame unit label, pad/drill inventory and source SHA.
- The Code Review discovered that one early Qt version kept the frame unit *hex text* instead of decoding it; final HEAD corrects this and adds an actual roundtrip assertion. Review thread was resolved before merge.
- [Code PR #549](https://github.com/july-rust/BoardSI/pull/549), exact final HEAD `5170669e0e71c3722bc225a35b65a2c44f2fdc13`: **36/36 GitHub check runs SUCCESS**, including Rust Numerical and Qt Linux, Windows x64/ARM64 and macOS ARM64 native checks, and CodeQL; unresolved review threads **0**. The first restricted F6 test failed on Ubuntu package repository HTTP timeout, then the **same SHA** job rerun passed; original failure logs remain. [Numerical Core PR workflow](https://github.com/july-rust/BoardSI/actions/runs/42552019047), [Qt runtime PR workflow](https://github.com/july-rust/BoardSI/actions/runs/42552019073), [F6 rerun evidence](https://github.com/july-rust/BoardSI/actions/runs/42552019201).
- Protected single-parent **Verified Squash** of #549: [`main@6eb35680a6e7775f62cd0fd763b559955810e7d6`](https://github.com/july-rust/BoardSI/commit/6eb35680a6e7775f62cd0fd763b559955810e7d6), `verification.verified=true / reason=valid`, parent `94afaf1c6a2b053dff96e79918e3427b10dd6022`. The exact-SHA post-merge Push CI **must be independently checked**; do not equate PR checks with main Push.

## Explicitly excluded and remaining D_056_05 gate

- This V1 adapter handles the supported single-source, source-orthogonal frame and `Circle`/`Rectangle`/`Annulus` layer-pad subset. Multi-file SourceIdentity, unsupported frame rotation, `PadShape::Custom` boundary, source grammars other than the explicitly supported importer, thermal spokes RT/ST, slot, backdrill and manufacturer-qualified microvia process **fail closed or remain unimplemented**. Never claim arbitrary CAD completeness.
- V1 does not transport a verified connected-layer set or qualified global electrical graph: it will reject this scenario rather than silently discard evidence. A future transport revision must provide an explicit proven contract for such information.
- The Qt decoder is a **model/test adapter**, not the production Qt Artwork main viewer. Main CPU/RHI geometry, layer selector/picking, cancellation of running import, cross-project GUI state, downstream consumer consistency and V1–V14 integration require [D_056_05 / Issue #531](https://github.com/july-rust/BoardSI/issues/531) separately.
- **D_056 parent #520 stays OPEN.** Preserve the separately accepted D_056_03 45-oracle SOURCE-software gate, but never call the fabricated board's plating, actual Open/Short or SI solver validated by D_056_04.

**Acceptance state:** D_056_04 source-supported IPC implementation, contract and independent CI are accepted in #549; final code-main Push and docs-only PR/main Push are recorded independently in the sole [development checklist](../development-checklist.md). No statement here upgrades D_056_05, manufacturing or product U/T status.
