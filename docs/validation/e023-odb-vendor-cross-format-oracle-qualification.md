# E_023 — Independent public Vendor ODB++ fixture and cross-format Oracle: dependency qualification

> Evidence date: **2026-11-14 KST**. Audit baseline: protected `main@45226d16bf63a28e28a931eb2abf8023e13984a5` (E_022 evidence reconciliation). This is a **source discovery / dependency disposition**, **not** a successfully executed product oracle test. No new BoardSI importer code, private data, external board bytes, or manufactured/SI claim is introduced.

## 1. Original acceptance contract

The sole [E checklist](../development-checklist.md) and [E plan](../e-stage-closure-plan.md) require **one independently produced, genuine Vendor ODB++ package, matching same-revision CAD source of a different format, permitted use, immutable raw SHA-256 and cross-format independent geometry Oracle**. Historical [#578](https://github.com/july-rust/BoardSI/issues/578) is administratively CLOSED, while [#10](https://github.com/july-rust/BoardSI/issues/10) remains OPEN. [E_022](e022-odb-local-electrical-graph-qualification.md) establishes synthetic SOURCE-local copper contacts, not commercial Vendor coverage or whole-board electrical proof.

### Candidate discovery (public repository metadata only)

| Candidate | Pinned evidence / actual listing | What is established | Missing mandatory evidence |
| :-- | :-- | :-- | :-- |
| [chikonnuggets/fab-to-kicad](https://github.com/chikonnuggets/fab-to-kicad), commit [`6b8a6642`](https://github.com/chikonnuggets/fab-to-kicad/tree/6b8a6642d6736092150ca1c0e720b833fd67c996) | `fab_files/microwave-odb.tgz` (53,313 B, Git blob SHA-1 `e59b8973615261cb30ab87ae1b7226ad9a016e09`), `fab_files/microwave.xml` (464,689 B, blob `4883703f9296012352ef2dca5f0d3bea6c74c129`), `original_PCB/microwave.kicad_pcb` (85,718 B, blob `7763a306cc5a503b04285e61fc2d081a6ed7fde0`). `CM5_MINIMA_3` similarly has ODB/IPC/KiCad files. | Named candidate artifacts of three formats coexist; project README describes conversion/limitations. | Commercial/vendor-independent exporter and exact version; same-revision immutable design identity; raw SHA-256; exact rights/notice clearance; independently computed geometry and actual BoardSI result. GitHub API repository metadata reports `license=null` and root listing has **no LICENSE file**; this is **not** proof there are no rights. |
| [FixturFab/ODB2kicad](https://github.com/FixturFab/ODB2kicad) | `samples/test-odb.tgz`, `samples/test-odb.zip`, `samples/test.kicad_pcb`, extracted ODB sample directories; repository advertises GPL-3.0 code license. | Public ODB samples and a KiCad-named file exist. | Identical origin/revision, real Vendor generator, independent Oracle, board-specific redistribution rights and raw SHA-256 not established. Repository code LICENSE is not an independent board data clearance. |
| [Delta-Proto/delta-odbpp](https://github.com/Delta-Proto/delta-odbpp) | `examples/designodb_rigidflex.tgz` has Git blob `21b43585a3ba11429a67768d8a1aa6aeff4c44d6`, **identical** to FixturFab `samples/designodb_rigidflex.tgz` blob. Other example archives exist. | Sample files and a third-party ODB reader are published. | The identical example blob is not independent export evidence. Matching original CAD, exporter/version, rights and independent BoardSI comparison unverified. |

**Interpretation:** A public GitHub path and Git blob SHA-1 are **not** an original file's calculated SHA-256; **no candidate binary has been newly downloaded, hashed, parsed with BoardSI, or compared to an independent CAD reader during this audit**. No file name or converter-generated reverse engineering output proves same-board revision or exporter independence. README statements about KiCad exporter vintage do not establish the specific archived binary's exporter tool/version.

## 2. Verification ledger

| Mandatory gate | Actual evidence | Verdict |
| :-- | :-- | :-- |
| Public ODB++ candidate location | Exact external repository file listings | CANDIDATE_IDENTIFIED |
| Matching alternative CAD/IPC-2581 candidate location | Identically named files present in one repository | CANDIDATE_IDENTIFIED; REVISION_NOT_VERIFIED |
| Real independent Vendor-origin export + version | No reproducible exporter receipt or verified tool/version metadata | DEPENDENCY_UNVERIFIED |
| Reuse / CI download / redistribution rights | No sufficiently scoped fixture authorization | BLOCKED |
| Raw exact bytes + SHA-256 | Git blob SHA-1 and metadata sizes only | NOT_RUN |
| External original CAD Oracle | No independently executed reference reader / geometry result | NOT_RUN |
| BoardSI actual imported output and same-revision delta | No authorized qualified pair / actual executable comparison | NOT_RUN |
| Positive/negative/UNKNOWN tests on independent fixture | Cannot run without qualified sources | NOT_RUN |
| Whole electrical graph, manufactured plating, full #10, SI | Never part of this dependency-discovery audit | NOT_ACCEPTED |

**No inferred PASS.** A future matching design must be checked by exact origin + version + raw bytes, then compared only for the implemented SOURCE-supported feature subset, with unsupported features reported as such. A KiCad CLI ODB export from the same board, if later qualified, can be a cross-format *tool-written regression* but must not silently be relabelled an **independent commercial Vendor** export.

## 3. Disposition / resume criteria

**Technical E_023: `BLOCKED / DEPENDENCY_UNVERIFIED`; administrative source-discovery audit: completed 2026-11-14 KST.** No implementation PR, no test run, no fixture redistribution, no fake formal U/T PASS. E_024 whole-ODB Gate cannot use this audit as Vendor Oracle acceptance.

Resume E_023 only after a separately verified public board satisfies **all**:

1. Genuine source-owned ODB++ CAD/Vendor export, named exporter/version, immutable direct download and provenance.
2. Independently produced, same-board/same-revision other CAD format, unambiguous board identity or manufacturing manifest.
3. Explicit applicable test/CI retrieval rights and redistribution approval if any bytes will be copied into project/release; complete third-party notice handling.
4. Raw SHA-256 and immutable reference for *both* original files, computed from actual bytes, not Git blob SHA-1 or a named path.
5. Independent CAD/third-party reader output, units/transforms and defensible tolerances; actual BoardSI import and supported-object cross-comparison.
6. Positive and adversarial fail-closed results, exact PR-head native CI, review, protected Verified Squash and distinct exact merged-main Push evidence, if implementing a fixture test.

Keep the dependency in [EB-01](../e-stage-blocked-register.md), the one [development checklist](../development-checklist.md), and this report. Avoid opening a long-lived implementation PR without the fixture prerequisites. Parent [#10](https://github.com/july-rust/BoardSI/issues/10) and [#520](https://github.com/july-rust/BoardSI/issues/520) remain independent OPEN product issues; historical [#578](https://github.com/july-rust/BoardSI/issues/578) remains CLOSED.

## 4. Evidence links

- [Source repository `fab_files` at pinned commit](https://github.com/chikonnuggets/fab-to-kicad/tree/6b8a6642d6736092150ca1c0e720b833fd67c996/fab_files), [`original_PCB`](https://github.com/chikonnuggets/fab-to-kicad/tree/6b8a6642d6736092150ca1c0e720b833fd67c996/original_PCB), [README limitations](https://github.com/chikonnuggets/fab-to-kicad/blob/6b8a6642d6736092150ca1c0e720b833fd67c996/README.md)
- [FixturFab sample folder](https://github.com/FixturFab/ODB2kicad/tree/main/samples), [GPL-3.0 repository LICENSE](https://github.com/FixturFab/ODB2kicad/blob/main/LICENSE)
- [Delta-Proto public examples](https://github.com/Delta-Proto/delta-odbpp/tree/main/examples)
- [Official KiCad ODB++ export documentation](https://docs.kicad.org/master/en/cli/cli.html)
