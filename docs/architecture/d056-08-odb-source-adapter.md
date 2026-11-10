# D_056_08 — ODB++Design source-evidence adapter (five bounded source-only slices)

**First implemented:** 2026-11-10 KST. **Status:** five separately bounded source-only code slices merged: [#562](https://github.com/july-rust/BoardSI/pull/562) Matrix/L/P/A, [#564](https://github.com/july-rust/BoardSI/pull/564) S Surface, [#566](https://github.com/july-rust/BoardSI/pull/566) opt-in design Net, and [#568](https://github.com/july-rust/BoardSI/pull/568) drill tools/declared layer span, followed by [#569](https://github.com/july-rust/BoardSI/pull/569) independent drill P-hit-to-tool cross-reference. All five have protected verified Squash; prior four code-main Push sets passed 8/8 independently while the latest #569 code-main Push must be audited separately; detailed scope and evidence are listed below. **Acceptance:** D_056_08 NOT COMPLETE; source drill tooling does not import physical holes or Canonical Vias, and this document does not assert whole-board ODB++, Qt import, electrical connectivity or SI.

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

ImportCapability: geometry PARTIAL, net MISSING **by default** or PARTIAL with opt-in `--eda-nets` (design ownership only), pin-pad MISSING, stackup PARTIAL (layer order only; no z/thickness), reference MISSING, model MISSING. All Graph nodes/edges remain empty.

## 3. Explicitly unqualified / unsupported

- Archive .tgz/.tar.gz/ZIP, compressed features.Z or compressed native ODB++ data.
- Nested Step Repeat / flip/mirror beyond the supported pad-local orthogonal transform, custom symbols, arbitrary-angle/scaled pads.
- Subtractive negative copper; ODB S feature with negative polarity; circular/arc OC contour; multiple islands; unclosed/degenerate/crossing outer or hole contour, boundary contact or nested/outside hole; copper text/barcodes and unsupported Copper features: source import **fails closed**. A bounded subset of positive S/OB/OS/OE/SE straight-edge copper is now supported as documented below.
- Full EDA/Net and Feature-to-Net mapping (only explicitly opt-in NET/SNT/FID is supported), Component/Pin, PTH/NPTH *drill hit geometry* (tools and declared layer span are audited separately as a source-only subset), blind/buried/microvia manufacturing proof and plated barrels.
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

## 3B. Strict opt-in EDA Net / FID source ownership (code PR #566)

**Versioned inputs:** `import_unpacked_with_eda(&OdbFileMap, step)`, `import_directory_with_eda(root, step)`, and CLI `boardsi-odb-import <unpacked-product-root> <step> --eda-nets`. The legacy geometry-only functions and corresponding SourceIdentity version/conditions remain unchanged.

**Normative grammar:** Siemens EDA, ODB++Design Format Specification 8.1 Update 4, `steps/<step>/eda/data` (pp.105–116). The strict supported subset parses `HDR`, **required** `UNITS=MM|INCH` before `LYR`, ordered `LYR`, design `NET`, `SNT TRC/VIA/PLN`, and `FID C <zero-based layer> <zero-based source-feature>`. Each positive copper `L/P/A/S` receives one feature ordinal in source-layer order; multiline surface `S/OB/OS/OE/SE` counts as one Feature. `SNT TRC` supports Trace/Arc, `VIA` supports Pad design identity only, `PLN` supports bounded Area. Unsupported EDA `PKG/PIN/TOP`, out-of-range FID, duplicate FID, wrong feature/subnet kind, malformed/duplicate unit, and duplicate **named** Net fail closed. The specification permits multiple `$NONE$` unassigned NET groups; they generate no canonical owned net while duplicate FIDs still reject.

**Canonical contract:** Generate `Net` objects with `NetRole::Unknown` and assign the matching Trace/Arc/Pad/CopperArea `net` references; bind source-file SHA in SourceIdentity and exact EDA `FID` source line witness to owned geometry `ConversionStep`. Only `capability.net=PARTIAL`. NO `ConnectivityGraph` nodes/edges are created; `SNT VIA` **does not** prove actual drilled/plated barrel, connected layers, fabricated Open/Short or SI. A failed EDA parser discards the entire transactional result. No cryptographic authenticity proof of real CAD exporter beyond source bytes.

**Tests:** 4-copper-layer NET to Trace/Arc/Pad mapping, correctly indexed Surface Area ownership and hole, `$NONE$` repeated groups, wrong FID/layer/type, duplicate Net/FID, absent EDA, duplicate/missing/unknown UNITS, SOURCE mutation changing Net/Copper IDs, preserved independent geometry-only behavior and all existing project CI tests. Synthetic source fixtures, **not independently qualified commercial CAD files**.

**Verification:** [PR #566](https://github.com/july-rust/BoardSI/pull/566) final head `518eefc9aa9e5f5ae66d40b3c3d4228e3971290e` with **30/30 PR Checks success**, automated review P1 mandatory Units and P2 repeated `$NONE$` findings corrected, 0 unresolved. Protected [single-parent GitHub-verified Squash `ffdad981aa96a2a89eee9c8f569b3d7b74916f58`](https://github.com/july-rust/BoardSI/commit/ffdad981aa96a2a89eee9c8f569b3d7b74916f58). The distinct **post-squash main Push** on `ffdad981aa96a2a89eee9c8f569b3d7b74916f58` has **8/8 workflow SUCCESS, 0 failure**: [Docs](https://github.com/july-rust/BoardSI/actions/runs/42749304686), [G0](https://github.com/july-rust/BoardSI/actions/runs/42749304770), [Lossless FEM](https://github.com/july-rust/BoardSI/actions/runs/42749304631), [PH-06](https://github.com/july-rust/BoardSI/actions/runs/42749304964), [Native FEM](https://github.com/july-rust/BoardSI/actions/runs/42749304703), [PH-07](https://github.com/july-rust/BoardSI/actions/runs/42749304989), [Numerical](https://github.com/july-rust/BoardSI/actions/runs/42749304963), [Qt](https://github.com/july-rust/BoardSI/actions/runs/42749304706). PR checks and Push checks are separate proof, not whole ODB++ acceptance.

**Still missing:** complete ODB EDA packages, component placements/pins, all subnet/FID forms, explicit plated Drill and Via, actual stackup materials, full contours/Step Repeat, compressed product import, independently qualified vendor oracle, and a real Main Qt Import/IPC/Artwork route. Do **not** mark D_056_08 as fully done based on these three limited slices.

## 3C. ODB++ MATRIX-to-tools drill process intent audit (code PR #568)

**Exact Source evidence only.** [Code PR #568](https://github.com/july-rust/BoardSI/pull/568) introduced `audit_unpacked_drill_tools`, `audit_directory_drill_tools` and `boardsi-odb-drill-audit <unpacked-product-root> <step>`. It parses selected BOARD DRILL layer records in `matrix/matrix` with explicit `START_NAME`/`END_NAME` into declared Copper-layer order, source `steps/<step>/layers/<drill-layer>/tools` units and `TOOLS { NUM, TYPE, TYPE2, FINISH_SIZE, DRILL_SIZE }` fields, with exact source SHA-256, source line and bounded grammar validation. It rejects reversed/partial spans, unknown tool types/subtypes, duplicate NUM, unit conflicts and inconsistent source declaration instead of manufacturing implicit connections.

**Normative reference:** Siemens ODB++Design 8.1 Update 4, MATRIX LAYER `START_NAME`/`END_NAME` and `ADD_TYPE` pp.67–69, `layer/tools` pp.184–186. The declared intent `PLATED`, `NON_PLATED`, `VIA`, `BACKDRILL` or `DUAL_DIAMETER` is **not** a hole location, actual drill outcome, copper plating confirmation, connected-layer proof or ready-to-simulate via.

**Verified CI:** PR HEAD `8e09dd464ca9633b65afa4238f104e4f4568fd96` has 30/30 check runs SUCCESS; protected Squash [main `badc4fb23011aea5193a5d39e8401e120c451ac5`](https://github.com/july-rust/BoardSI/commit/badc4fb23011aea5193a5d39e8401e120c451ac5) followed by independent `push` [8/8 successful workflows](https://github.com/july-rust/BoardSI/actions?query=event%3Apush+branch%3Amain) on that exact SHA (including Qt run #42760798834 attempt 2). Tests are synthetic 6-layer positive/negative source fixtures, not independent vendor CAM evidence. Canonical `Drill`/`Via`, plated Graph, Main Qt IPC and SI remain **NOT_VERIFIED**.

## 3D. Independent drill P hit-to-tool inventory (code PR #569)

**Added source-only API:** `audit_unpacked_drill_hits`, `audit_directory_drill_hits` and diagnostic CLI `boardsi-odb-drill-audit <unpacked-product-root> <step> --hits`. These parse drill layer `features` independently of unsupported Copper artwork, requiring explicit matching `layer/tools` `NUM` for every positive P feature's nonzero `dcode`. The bounded inventory stores feature source-file SHA-256/line/index, exact source decimal coordinate text/UNITS, tool file SHA/line and MATRIX SHA/line. Invalid/missing tool, duplicate ID, unit mismatch, negative/non-circular geometry, malformed P, count mismatch and unsupported conductive features fail closed. Synthetic six-copper-layer full-span and source mutation cases are tested.

**Normative source:** Siemens ODB++Design 8.1 Update 4 Feature P record pp.169–170 and layer/tools pp.184–186. **Limits:** P/tool cross-reference is evidence of a declared hole *position/intent* only. It does not generate Canonical Drill/Via or plated layer-to-layer edges, perform manufactured Backdrill process reconstruction, calculate Open/Short or integrate with Main Qt/IPC/SI.

**PR and main verification:** [#569](https://github.com/july-rust/BoardSI/pull/569) final HEAD `cd5e49a7a51289a9b2bd09fa30d9e7bc4cbc1861`, **30/30 PR checks SUCCESS** after retrying a cancelled provisional Linux Qt job; latest code review reports no unresolved findings; [protected verified single-parent Squash `f07442a1d97d11b49419f32655c03b1714d4840f`](https://github.com/july-rust/BoardSI/commit/f07442a1d97d11b49419f32655c03b1714d4840f). **Independent exact-code-main Push verification pending** at initial documentation edit; update only after all workflow conclusions are checked. No real vendor fixture or electrical validation claimed.

## 4. Test coverage and acceptance

Implemented tests: 2/4/6/8/64-layer source maps; round line, rotated rectangle pad, CW/CCW arc and canonical coordinates; SHA mutation binding; missing source file; bad feature count; unsupported/negative copper; wrong arc radius; **full-circle rejected before invalid Canonical Arc creation**; **zero-length line rejected before invalid Canonical Trace creation**; bad pad rotation; unsafe path; decimal overflow; wrong step; real unpacked directory loading and Unix symlink rejection; complete `BoardModel::validate()` structural checks on accepted 2–64 layer fixtures.

**Verified evidence (2026-11-10 KST):** exact PR #562 final head `8502bc0fa8a3ed9ff92b6c55e365cf1b97a0c411`: **30/30 Checks SUCCESS**; 9 pull-request workflow runs SUCCESS, separate code-scanning PR job SUCCESS. Four review findings (unit inheritance, fractional symbol dimensions, unsupported full circle and zero-length line) corrected, all review threads resolved. Protected single-parent squash commit `1f33e75beced7115cb65453aca4c6e9fc1d6df36`: GitHub verification `verified=true`, reason `valid`. Distinct main `push` workflow set: **8/8 SUCCESS**: [Numerical](https://github.com/july-rust/BoardSI/actions/runs/42733742478), [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/42733742501), [Docs](https://github.com/july-rust/BoardSI/actions/runs/42733742895), [M2 PH-06](https://github.com/july-rust/BoardSI/actions/runs/42733743062), [M2 PH-07](https://github.com/july-rust/BoardSI/actions/runs/42733742499), [G0](https://github.com/july-rust/BoardSI/actions/runs/42733743052), [Lossless FEM](https://github.com/july-rust/BoardSI/actions/runs/42733742971), [Native FEM](https://github.com/july-rust/BoardSI/actions/runs/42733742572). Both categories are separate evidence, not a claim of full ODB++ import readiness. Code tests alone do **not** qualify real independent CAD exports or physical connection.

Remaining D_056_08 gates: external valid ODB++ fixture + reference oracle, full geometry inventory including arbitrary Surface/OC/multi-island/negative handling, **complete** EDA/Net/Part/Pin/Drill mappings beyond the restricted design FID contract, safe archive handling, Main Qt/IPC integration and source-qualified complete electrical Graph behavior. D_056_08 remains OPEN until its agreed acceptance contract is actually met; Issue #532 and parent #520 remain OPEN through later format stages.

Avoid filling the sole checklist with unverifiable success claims.
