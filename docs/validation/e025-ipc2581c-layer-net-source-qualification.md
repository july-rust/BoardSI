# E_025 — IPC-2581C Source Layer/LogicalNet Qualification

> Date: 2026-11-14 KST. **E_025 SOURCE-only LIMITED COMPLETE**: original [#628](https://github.com/july-rust/BoardSI/pull/628), corrected/reviewed [#630](https://github.com/july-rust/BoardSI/pull/630), and evidence/docs [#631](https://github.com/july-rust/BoardSI/pull/631) all protected Squash merged, GitHub Verified, and exact merged-main `event=push` workflows **8/8 SUCCESS for each**. #630 first Qt main-Push attempt was CANCELLED in Windows ARM64; same exact SHA rerun attempt 2 SUCCESS, final **28/28 checks SUCCESS**. This qualifies the bounded Layer/LogicalNet declaration inventory only; not full XSD, physical board, vendor oracle, electrical SI or product U/T.

## Requirement and source

- [E closure plan](../e-stage-closure-plan.md) E_025; [#532](https://github.com/july-rust/BoardSI/issues/532), [#23](https://github.com/july-rust/BoardSI/issues/23); parent #520 remains OPEN.
- IPC Consortium RevC test cases: https://www.ipc2581.com/ipc-2581-revc-test-cases/
- IPC Consortium XML/XSD validation limitations: https://www.ipc2581.com/ipc-2581-file-validation-tool/
- Official XSD directory: https://webstds.ipc.org/2581/
- Independently readable IPC-2581C XSD mirror (for schema-reference only): https://docs.rs/crate/ipc2581/0.3.3/source/IPC-2581C.xsd
- Verified structural names for this implementation: IPC-2581@revision=C, xmlns=http://webstds.ipc.org/2581, Ecad/CadData/Layer @name/@layerFunction/@side/@polarity, Ecad/CadData/Step @name /LogicalNet @name /PinRef @pin and optional @componentRef, Content/LayerRef @name.

## Actual bounded behavior

- `crates/boardsi-import/src/ipc2581c_source.rs` exposes opt-in `inspect(bytes)`.
- Hard 8 MiB document limit, 100,000 element limit, 64-level depth limit, 4,096 Layer and 65,536 LogicalNet limits. After #630, cumulative stored lexical path strings are additionally bounded at 8 MiB to block attacker-controlled ancestor-name memory amplification.
- Decodes built-in and numeric XML character references, rejects invalid UTF-8, malformed tags/attrs, repeated attrs, DTD/CDATA/external/unknown entities, prefixed elements and nested namespace overrides. An additional parser audit now rejects XML comments with an illegal interior `--` or terminal `-`, and prohibited `]]>` in character data, with negative regression cases.
- Rejects revision != C or noncanonical default namespace, malformed/missing required declaration attributes, duplicate Layer/Net IDs, unknown declared Content LayerRef, ambiguous multiple Step and LogicalNet with no PinRef.
- Preserves raw source declaration names, function/side/polarity and raw PinRef; `SourceIdentity` binds SHA-256 of exact bytes, selected Step and importer version; `ObjectId` binds the original Layer and Step/LogicalNet names.
- Unknown layer functions are retained as nonconductive `LayerKind::Other` with explicit count, **never converted to copper**.
- Returns only an `Inventory`, not `BoardModel`. Net, stackup declaration capability remain PARTIAL; geometry/PinPad/Reference/Model are MISSING; `solver_ready=false`, `physical_si=NOT_RUN`.

## Explicitly not established

This restricted reader **is not a full IPC-2581C XSD validator**. It does not validate the complete XSD content model, every schema enumerated value, all foreign references, arbitrary XML namespace prefixes, all legal multiple Step cases, source Stackup physical ordering, conductor thickness/z, padstack/via/plating/copper, physical Net Graph, independent Vendor export or cross-CAD Oracle, Qt integration, electrical Opens/Shorts or SI. Synthetic source-contract regression is not product or manufacturing acceptance. Synthetic minimal XML fixtures are not asserted to be fully XSD-valid.

## Tests and required evidence

**Executed code-head evidence (2026-11-14 KST):** [PR #628](https://github.com/july-rust/BoardSI/pull/628), exact code HEAD [`fab3652c`](https://github.com/july-rust/BoardSI/commit/fab3652c429260da5b4d3eef7ebe15698a460598): **30/30 GitHub check-runs SUCCESS, 0 failed** (includes applicable 17 protected checks, additional CodeQL and classifier). Numerical workflow [run 43559931382](https://github.com/july-rust/BoardSI/actions/runs/43559931382) Linux job `132341172270` ran **Rust Unit 349/349 Debug and 349/349 Release PASS (E_025 six source tests PASS in both)**, formatting/Clippy PASS. Rust native Windows x64/ARM64, macOS ARM64 SUCCESS. Qt runtime [run 43559931318](https://github.com/july-rust/BoardSI/actions/runs/43559931318) Linux/Windows x64/ARM64/macOS ARM64 SUCCESS; docs, FEM, FVM and import benchmarks also SUCCESS. Initial compile type inference, rustfmt, and strict Clippy failures at earlier HEAD were corrected; do not erase failures or relabel them PASS.

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

**Final original-PR evidence (independently read from GitHub API 2026-11-14 KST):** [PR #628](https://github.com/july-rust/BoardSI/pull/628) final HEAD [`a5de84e`](https://github.com/july-rust/BoardSI/commit/a5de84e79aea5bf7f35a8c895580ad574c3fb0ff) **30/30 check-runs SUCCESS**; protected [squash `e9ef6510`](https://github.com/july-rust/BoardSI/commit/e9ef651049524fa2dd2d2c8d06c1dfc53dc75548) has **exactly one parent**, GitHub commit verification `verified=true, reason=valid`, and was the observed `main` branch HEAD at audit. Exact SHA `e9ef6510` + `event=push` [Actions collection](https://github.com/july-rust/BoardSI/actions?query=branch%3Amain) **8/8 workflows completed SUCCESS**, **28/28 check-runs SUCCESS**, including Rust/Qt native runs. [Numerical core main Push](https://github.com/july-rust/BoardSI/actions/runs/43562468860); [Qt runtime main Push](https://github.com/july-rust/BoardSI/actions/runs/43562468978). No pre-merge CI is being substituted for post-merge evidence. Initial earlier failures and successful corrections remain historical evidence.

**Review evidence and boundary:** Original PR #628 exact final-HEAD *external* review remains historically **NOT_VERIFIED** (Codex quota exhausted; no completed Copilot review). This is not retroactively upgraded to approval. However, corrected final source HEAD `75b1da0831aa905942fbb7552e4c3d564c52fdb5` in [follow-up PR #630](https://github.com/july-rust/BoardSI/pull/630) received a **completed independent Codex review**, covering the whole E_025 parser including inherited source, with **0 unresolved findings and 0 unresolved threads** after P1/P2 remediation. The reviewed latest source code, not the original historical #628 revision, satisfies the E_025 source-code independent-review gate. Formal vendor/XSD Oracle and product U/T remain NOT_VERIFIED/NOT_RUN.

**Corrective parser hardening:** PR #630 corrected invalid attribute separators, opening/closing tag lexical whitespace, XML comments/control characters and optional component IDs; Codex P1 exposed severe repeated long ancestor-path memory amplification and P2 cases in numeric references, XML-only whitespace, literal attribute normalization and XML declaration grammar. Added a cumulative 8 MiB path-storage limit and targeted positive/negative regressions; final HEAD **30/30 GitHub check-runs SUCCESS** with Rust **353/353 Debug and 353/353 Release**, **10/10 E_025 tests in each**. [Numerical run 43566081704](https://github.com/july-rust/BoardSI/actions/runs/43566081704). GitHub signed, one-parent protected Squash [main `41890286`](https://github.com/july-rust/BoardSI/commit/41890286efcb87f37aad838fab54817c4d4f6353), `verified=true`. Exact code main commit SHA `41890286efcb87f37aad838fab54817c4d4f6353`: **8/8 `event=push` workflows SUCCESS, 28/28 checks SUCCESS** after [Qt Runtime run #43567771213](https://github.com/july-rust/BoardSI/actions/runs/43567771213) attempt **2**. Original attempt **1 CANCELLED** for Qt Windows ARM64 is preserved (not relabeled as success); targeted rerun on the exact same commit succeeded. Earlier `rustfmt` failures and fixes remain historical evidence.

## Closeout criteria

**E_025 SOURCE-only closeout: COMPLETE (evidence date 2026-11-14 KST).** Original #628 final HEAD **30/30 SUCCESS**, Verified Squash [`e9ef6510`](https://github.com/july-rust/BoardSI/commit/e9ef651049524fa2dd2d2c8d06c1dfc53dc75548), exact main Push **8/8 SUCCESS**. Corrected #630 final HEAD `75b1da08` **30/30 SUCCESS**; Rust Debug and Release **353/353 each**, E_025 **10/10 each**, and full corrected-source Codex review completed with **0 unresolved findings/threads**. Verified Squash [`41890286`](https://github.com/july-rust/BoardSI/commit/41890286efcb87f37aad838fab54817c4d4f6353), exact main Push **8/8 workflows and 28/28 checks SUCCESS** after Qt Windows ARM64 job rerun attempt 2. Docs #631 final HEAD **13 SUCCESS/16 docs-only SKIPPED, 0 failed**, Verified Squash [`3777c874`](https://github.com/july-rust/BoardSI/commit/3777c874cea45e898ad2b23a86c6b0ead22d8073), exact docs-main Push **8/8 workflows SUCCESS**. Docs-only skipped native jobs must **not** be treated as new Native PASS. Historical #628 external-review gap and initial #630 Qt cancellation remain documented. This limited completion is a **declaration inventory only**, not full IPC-2581/XSD-certified `BoardModel`, physical Layer/Via Copper/Net Graph, Qt artwork or SI. Parent issues #532/#23/#520 and formal U07/U33/U38/U40/T16/T18/T19/T20 remain **OPEN / NOT_ACCEPTED**.
