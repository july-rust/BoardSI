# E_025 — IPC-2581C Source Layer/LogicalNet Qualification

> Date: 2026-11-14 KST. Status: **CODE CI PASS / INDEPENDENT REVIEW NOT_VERIFIED / MERGE PENDING**. Actual Rust/Qt/CI code HEAD `900e8ab066c830dd4f5e3ffb67afa6dc29b79f71` succeeded; final docs PR HEAD, protected merge and exact main Push remain separate obligations. No completed U/T/production gate is claimed.

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

**Executed code-head evidence (2026-11-14 KST):** [PR #628](https://github.com/july-rust/BoardSI/pull/628), exact code HEAD [`900e8ab0`](https://github.com/july-rust/BoardSI/commit/900e8ab066c830dd4f5e3ffb67afa6dc29b79f71): **30/30 GitHub check-runs SUCCESS, 0 failed** (includes applicable 17 protected checks, additional CodeQL and classifier). Numerical workflow [run 43556754146](https://github.com/july-rust/BoardSI/actions/runs/43556754146) Linux job `132331341212` ran **Rust Unit 348/348 Debug and 348/348 Release PASS**, formatting/Clippy PASS. Rust native Windows x64/ARM64, macOS ARM64 SUCCESS. Qt runtime [run 43556754052](https://github.com/july-rust/BoardSI/actions/runs/43556754052) Linux/Windows x64/ARM64/macOS ARM64 SUCCESS; docs, FEM, FVM and import benchmarks also SUCCESS. Initial compile type inference, rustfmt, and strict Clippy failures at earlier HEAD were corrected; do not erase failures or relabel them PASS.

The table below describes what these executed tests cover; `PASS (synthetic scope)` means only the bounded Source Inventory contract.

| Test scope | Intended outcome | Actual |
| --- | --- | --- |
| 2/4/6/8/64 Layer declaration inventory | deterministic counts, no inferred physical order | PASS (synthetic scope) |
| RevC Layer/Net/PinRef and original IDs | exact SourceIdentity and SHA-256, partial capability | PASS (synthetic scope) |
| byte mutation and reimport | distinct SHA and distinct source identity | PASS (synthetic scope) |
| other Revision/namespace, malformed XML | fail closed | PASS (synthetic scope) |
| duplicate/missing Layer/Net/PinRef, unknown references | fail closed | PASS (synthetic scope) |
| unknown layer function | `Other` / UNKNOWN, not Copper | PASS (synthetic scope) |
| malicious entity/DTD/nested namespace/oversize | reject without loading external resources | PASS (synthetic scope) |
| existing PADS/ODB++ and Rust/Qt native regressions | preserved behavior | PASS (Rust/Qt Native CI only) |
| Independent vendor RevC fixture / official full XSD | separate qualification | NOT_VERIFIED |

**Important evidence boundary:** the exact code PR HEAD passed CI; final documentation edit commits can result in docs-only Native SKIPPED and must not be counted as a new Native PASS. GitHub Codex independent code-review attempts failed due exhausted review quota (see PR #628 conversation), **not a clean review and not a finding-free approval**; no completed independent final-HEAD review is available. Merge verified signature and exact merged-main `event=push` are NOT_VERIFIED until independently checked. Official RevC XSD-valid vendor input, independent Oracle and formal product U/T are NOT_VERIFIED/NOT_RUN.

## Closeout criteria

Final E_025 SOURCE-only COMPLETE requires exact PR HEAD applicable checks, independent code review/threads resolved, protected Verified squash merge and exact merged-main Push verification. An unavailable proof is NOT_VERIFIED, not PASS. Parent issues #532/#23/#520 and formal U07/U33/U38/U40/T16/T18/T19/T20 remain OPEN / not product accepted.
