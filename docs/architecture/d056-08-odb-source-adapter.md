# D_056_08 — ODB++Design source-evidence adapter (bounded, two implemented slices)

**First implemented:** 2026-11-10 KST. **Status:** source-limited code [PR #562](https://github.com/july-rust/BoardSI/pull/562) **MERGED**; verified protected Squash [main `1f33e75beced7115cb65453aca4c6e9fc1d6df36`](https://github.com/july-rust/BoardSI/commit/1f33e75beced7115cb65453aca4c6e9fc1d6df36) with independent Push **8/8 SUCCESS**. **Acceptance:** D_056_08 NOT COMPLETE; this document does not assert whole-board ODB++ support, Qt import, electrical connectivity or SI.

Parent [#520](https://github.com/july-rust/BoardSI/issues/520) · Multi-format gate [#532](https://github.com/july-rust/BoardSI/issues/532) · [Source contract](d056-01-source-evidence-audit.md) · [Single checklist](../development-checklist.md#d_056).

## 1. Authoritative format evidence

Siemens EDA, [ODB++Design Format Specification Release 8.1 Update 4, August 2024](https://odbplusplus.com/wp-content/uploads/sites/2/2024/08/odb_spec_user.pdf). This implementation independently parses a limited record subset; it does not embed or redistribute the specification or imply Siemens validation or license.

- Product model pp.43–46: product root / steps / layer containers.
- matrix/matrix pp.62–66: STEP and LAYER blocks; ROW, NAME, TYPE, CONTEXT, POLARITY.
- Layer features pp.164–172: UNITS, symbol table, F, L, P, A records, optional ID and line provenance.
- Standard symbols pp.187–189: round, square, rectangle; explicit I (mils) and M (micrometres) symbol units or source-file unit.
- Public synthetic grammar tests are **not** independent real CAD exporter evidence.

## 2. Runtime entrypoints and limits

Rust source: crates/boardsi-import/src/odb_source.rs. Public library functions are import_unpacked(OdbFileMap, step) and import_directory(Path, step). The diagnostic CLI boardsi-odb-import accepts an unpacked product directory and explicit step. Not a main Qt Import yet.

- Bounded product-root-relative file map; no symlink, nonregular-file or unsafe path traversal.
- File size at most 16 MiB; cumulative at most 128 MiB; at most 8192 files; directory depth at most 16.
- Exact decimal coordinates up to nine fractional digits. Metric frame is 1e-12 metre per tick, inch frame 2.54e-11 metre per tick, retaining exact rational transforms.
- Source SHA-256 per file and logical relative source path:line provenance for Canonical objects.
- Positive L lines with round symbols -> Trace; positive P with round/square/rectangle -> Pad/Padstack; positive A with round symbol and common exact radius -> analytic Arc; bounded positive S straight-edge surface with a single island and strictly enclosed holes -> Canonical CopperArea / Cutout (PR #564).
- Matrix BOARD Copper Layer order accepts SIGNAL, POWER_GROUND and MIXED for 2–64 layers.
- No fabricated net, material, connected-layer evidence, manufacture connectivity or S-parameter is produced.

ImportCapability: geometry PARTIAL, net MISSING, pin-pad MISSING, stackup PARTIAL (layer order only; no z/thickness), reference MISSING, model MISSING. All Graph nodes/edges remain empty.

## 3. Explicitly unqualified / unsupported

- Archive .tgz/.tar.gz/ZIP, compressed features.Z or compressed native ODB++ data.
- Nested Step Repeat / flip/mirror beyond the supported pad-local orthogonal transform, custom symbols, arbitrary-angle/scaled pads.
- Subtractive negative copper; ODB S feature with negative polarity; circular/arc OC contour; multiple islands; unclosed/degenerate/crossing outer or hole contour, boundary contact or nested/outside hole; copper text/barcodes and unsupported Copper features: source import **fails closed**. A bounded subset of positive S/OB/OS/OE/SE straight-edge copper is now supported as documented below.
- EDA/Net mapping, Feature-to-Net mapping, Component/Pin, PTH/NPTH drill, blind/buried/microvia manufacturing information and plated barrels.
- Actual layer z/material/Dk/Df/thickness, whole-board electronic connectivity, verified open/short and SI.
- Main Qt Artwork IPC / CPU / RHI / picking and vendor-export qualification.
- Concurrent filesystem mutations are NOT atomically snapshotted; TOCTOU race and adversarial mount changes remain NOT_VERIFIED.

This is a strict importer for **selected supported Copper records**, not a complete import of all objects present in a CAD product. Any unparsed evidence must not be promoted to COMPLETE.

## 3A. Additional bounded SOURCE Surface / Cutout implementation (PR #564)

**Code:** `crates/boardsi-import/src/odb_surface.rs` (bounded exact integer topology), `odb_source.rs` (multiline S/OB/OS/OE/SE source reader and Canonical conversion), `src/bin/boardsi-odb-import.rs` (area/hole diagnostic counts). The legacy PADS parser, Qt IPC, display pipeline and global electrical Graph are not changed.

**Supported narrow grammar:** one positive `S P <dcode>` record on a selected BOARD Copper layer, followed by `OB x y I` (single copper outer island), `OS x y` closed straight-edge vertices, `OE`; optionally up to 32 strictly internal `OB x y H` hole contours, then `SE`. Source-tick validation uses exact `i128` orientation, segment-contact and winding-free strict inclusion; a conservative maximum of 512 total contour vertices and coordinates bounded to ±1,000,000,000,000 source ticks. Accepting a record requires a nondegenerate, closed, simple outer contour and distinct strictly contained non-touching holes; unknown roles, missing `OE`/`SE`, curved `OC`, multiple outer islands, overlap, nesting and out-of-range coordinates **fail closed**, not silently discarded.

**Canonical meaning:** a successful surface produces a `CopperArea` with exact line-segment `Boundary` and its true voids as `Cutout` objects, with the file SHA bound to SourceIdentity and the surface source location bound to ObjectMeta. The hole object inherits the parent surface's source record location; this is **not** an independent per-hole OB-record locator. `net=None`; no inter-layer connection, plated-barrel proof, fabricated surface fill or SI result is asserted. Source-file units and matrix layer remain attached to the emitted points. A local call to `BoardModel::validate` checks reference and boundary structure, **not** independent source-to-copper equivalence.

**Tests:** positive four-edge outer + one internal cutout, standalone hole/island geometry preflight, self-intersection, unclosed surface, unsupported negative/circular surface, hole outside, and regression that unsupported non-S copper still fails closed. These are **synthetic grammar fixtures only**; an external ODB++Design exporter and independent same-board CAM geometry oracle are required for vendor interoperability acceptance.

**CI and merge proof:** [code PR #564](https://github.com/july-rust/BoardSI/pull/564), final PR HEAD `d1f2ce51ab21764de7ef5cac2698179a0755636a`, **30/30 PR checks SUCCESS**, review threads resolved; [protected Verified single-parent Squash `0d832dd92d86ea1d6e375b89506cf74be9b252d0`](https://github.com/july-rust/BoardSI/commit/0d832dd92d86ea1d6e375b89506cf74be9b252d0). Independent code-main Push for exact squash `0d832dd92d86ea1d6e375b89506cf74be9b252d0`: **8/8 workflows SUCCESS** and 0 FAIL ([Numerical](https://github.com/july-rust/BoardSI/actions/runs/42742249073), [Qt](https://github.com/july-rust/BoardSI/actions/runs/42742249075), [Docs](https://github.com/july-rust/BoardSI/actions/runs/42742249504), [Native FEM](https://github.com/july-rust/BoardSI/actions/runs/42742249106), [PH-07](https://github.com/july-rust/BoardSI/actions/runs/42742249161), [PH-06](https://github.com/july-rust/BoardSI/actions/runs/42742249272), [Lossless](https://github.com/july-rust/BoardSI/actions/runs/42742249340), [G0](https://github.com/july-rust/BoardSI/actions/runs/42742249316)). The PR checks and post-merge Push are separate, not a claim of whole-ODB acceptance.

## 4. Test coverage and acceptance

Implemented tests: 2/4/6/8/64-layer source maps; round line, rotated rectangle pad, CW/CCW arc and canonical coordinates; SHA mutation binding; missing source file; bad feature count; unsupported/negative copper; wrong arc radius; **full-circle rejected before invalid Canonical Arc creation**; **zero-length line rejected before invalid Canonical Trace creation**; bad pad rotation; unsafe path; decimal overflow; wrong step; real unpacked directory loading and Unix symlink rejection; complete `BoardModel::validate()` structural checks on accepted 2–64 layer fixtures.

**Verified evidence (2026-11-10 KST):** exact PR #562 final head `8502bc0fa8a3ed9ff92b6c55e365cf1b97a0c411`: **30/30 Checks SUCCESS**; 9 pull-request workflow runs SUCCESS, separate code-scanning PR job SUCCESS. Four review findings (unit inheritance, fractional symbol dimensions, unsupported full circle and zero-length line) corrected, all review threads resolved. Protected single-parent squash commit `1f33e75beced7115cb65453aca4c6e9fc1d6df36`: GitHub verification `verified=true`, reason `valid`. Distinct main `push` workflow set: **8/8 SUCCESS**: [Numerical](https://github.com/july-rust/BoardSI/actions/runs/42733742478), [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/42733742501), [Docs](https://github.com/july-rust/BoardSI/actions/runs/42733742895), [M2 PH-06](https://github.com/july-rust/BoardSI/actions/runs/42733743062), [M2 PH-07](https://github.com/july-rust/BoardSI/actions/runs/42733742499), [G0](https://github.com/july-rust/BoardSI/actions/runs/42733743052), [Lossless FEM](https://github.com/july-rust/BoardSI/actions/runs/42733742971), [Native FEM](https://github.com/july-rust/BoardSI/actions/runs/42733742572). Both categories are separate evidence, not a claim of full ODB++ import readiness. Code tests alone do **not** qualify real independent CAD exports or physical connection.

Remaining D_056_08 gates: external valid ODB++ fixture + reference oracle, full geometry inventory including arbitrary Surface/OC/multi-island/negative handling, EDA/Net/Part/Drill mappings, safe archive handling, Main Qt/IPC integration and source-qualified complete electrical Graph behavior. D_056_08 remains OPEN until its agreed acceptance contract is actually met; Issue #532 and parent #520 remain OPEN through later format stages.

Avoid filling the sole checklist with unverifiable success claims.
