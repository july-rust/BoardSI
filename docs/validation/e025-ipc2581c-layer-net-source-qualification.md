# E_025 — IPC-2581C Source Layer/LogicalNet Qualification

> Date: 2026-11-14 KST. Status: **IMPLEMENTED / CI_PENDING** until the exact final PR HEAD, independent review, protected squash merge and merged-main Push receipts are confirmed. No completed U/T/production gate is claimed.

## Requirement and source

- [E closure plan](../e-stage-closure-plan.md) E_025; [#532](https://github.com/july-rust/BoardSI/issues/532), [#23](https://github.com/july-rust/BoardSI/issues/23); parent #520 remains OPEN.
- IPC Consortium RevC test cases: https://www.ipc2581.com/ipc-2581-revc-test-cases/
- IPC Consortium XML/XSD validation limitations: https://www.ipc2581.com/ipc-2581-file-validation-tool/
- Official XSD directory: https://webstds.ipc.org/2581/
- Independently readable IPC-2581C XSD mirror (for schema-reference only): https://docs.rs/crate/ipc2581/0.3.3/source/IPC-2581C.xsd
- Verified structural names for this implementation: IPC-2581@revision=C, xmlns=http://webstds.ipc.org/2581, Ecad/CadData/Layer @name/@layerFunction/@side/@polarity, Ecad/CadData/Step @name /LogicalNet @name /PinRef @pin and optional @componentRef, Content/LayerRef @name.

## Actual bounded behavior

- `crates/boardsi-import/src/ipc2581c_source.rs` exposes opt-in `inspect(bytes)`.
- Hard 8 MiB document limit, 100,000 element limit, 64-level depth limit, 4,096 Layer and 65,536 LogicalNet limits.
- Decodes built-in and numeric XML character references, rejects invalid UTF-8, malformed tags/attrs, repeated attrs, DTD/CDATA/external/unknown entities, prefixed elements and nested namespace overrides.
- Rejects revision != C or noncanonical default namespace, malformed/missing required declaration attributes, duplicate Layer/Net IDs, unknown declared Content LayerRef, ambiguous multiple Step and LogicalNet with no PinRef.
- Preserves raw source declaration names, function/side/polarity and raw PinRef; `SourceIdentity` binds SHA-256 of exact bytes, selected Step and importer version; `ObjectId` binds the original Layer and Step/LogicalNet names.
- Unknown layer functions are retained as nonconductive `LayerKind::Other` with explicit count, **never converted to copper**.
- Returns only an `Inventory`, not `BoardModel`. Net, stackup declaration capability remain PARTIAL; geometry/PinPad/Reference/Model are MISSING; `solver_ready=false`, `physical_si=NOT_RUN`.

## Explicitly not established

This restricted reader **is not a full IPC-2581C XSD validator**. It does not validate the complete XSD content model, every schema enumerated value, all foreign references, arbitrary XML namespace prefixes, all legal multiple Step cases, source Stackup physical ordering, conductor thickness/z, padstack/via/plating/copper, physical Net Graph, independent Vendor export or cross-CAD Oracle, Qt integration, electrical Opens/Shorts or SI. Synthetic source-contract regression is not product or manufacturing acceptance. Synthetic minimal XML fixtures are not asserted to be fully XSD-valid.

## Tests and required evidence

Initial test definitions (results remain NOT_RUN until CI evidence arrives):

| Test scope | Intended outcome | Actual |
| --- | --- | --- |
| 2/4/6/8/64 Layer declaration inventory | deterministic counts, no inferred physical order | NOT_RUN |
| RevC Layer/Net/PinRef and original IDs | exact SourceIdentity and SHA-256, partial capability | NOT_RUN |
| byte mutation and reimport | distinct SHA and distinct source identity | NOT_RUN |
| other Revision/namespace, malformed XML | fail closed | NOT_RUN |
| duplicate/missing Layer/Net/PinRef, unknown references | fail closed | NOT_RUN |
| unknown layer function | `Other` / UNKNOWN, not Copper | NOT_RUN |
| malicious entity/DTD/nested namespace/oversize | reject without loading external resources | NOT_RUN |
| existing PADS/ODB++ and Rust/Qt native regressions | preserved behavior | NOT_RUN |
| Independent vendor RevC fixture / official full XSD | separate qualification | NOT_VERIFIED |

No observed Native CI, exact PR checks or code-main Push success is claimed at report creation time.

## Closeout criteria

Final E_025 SOURCE-only COMPLETE requires exact PR HEAD applicable checks, independent code review/threads resolved, protected Verified squash merge and exact merged-main Push verification. An unavailable proof is NOT_VERIFIED, not PASS. Parent issues #532/#23/#520 and formal U07/U33/U38/U40/T16/T18/T19/T20 remain OPEN / not product accepted.
