# F_003 — Native Rust SQLite Writer/Reader qualification

> 2026-11-15 KST · [F_001 contract](../architecture/f001-bsi-format-contract.md) · [F_002 schema](../architecture/f002-bsi-sqlite-schema.md) · [BSIG layout](../architecture/f002-bsi-geometry-binary-layout.md)

## 확정 결과와 범위

- Native `.bsi` v1 Writer/Reader·BSIG 9종 구현 및 CI 실행 검증: **CORE COMPLETE**.
- 구현 [PR #651](https://github.com/july-rust/BoardSI/pull/651) 마지막 HEAD `5766601ba632ab12ce3013ee1040cdf57f23d727`: **47/47 SUCCESS**, review threads **0 unresolved**.
- 보호 Squash [main `cdbf57bd4c596e2643c78b59eee122accfe123d3`](https://github.com/july-rust/BoardSI/commit/cdbf57bd4c596e2643c78b59eee122accfe123d3): **Verified**, exact main push Checks **41/41 SUCCESS**.
- [Native four-OS PR evidence](https://github.com/july-rust/BoardSI/actions/runs/43702554917) · [exact-main native evidence](https://github.com/july-rust/BoardSI/actions/runs/43704091593).
- Product U/T, F_004 historical Golden migrations, F_005 vendor CAD equivalence, Qt native menu integration, EB-01~07 and PH-08 statuses are **not** upgraded by F_003.

## F003-T01–T12 — tested evidence versus untested scenarios

| Gate | Evidence status | Test boundary |
| :--- | :--- | :--- |
| T01/T02 | **PASS** | Rust SQLite save/open, BSIG nine kinds and Debug/Release round trips |
| T03/T04 | **PASS (fixture)** | Exact i64 values, i128 rational, Canonical semantic equality on exercised models |
| T05/T06 | **PASS (synthetic fixture)** | SourceIdentity, material/layer evidence and connectivity references; independent vendor-source proof is out of scope |
| T07/T08 | **PASS (tested negatives)** | Deterministic payload, corrupt digest/kind/unversioned SQL schema fail-closed |
| T09 | **PASS** | Required feature reject, unknown optional/read-only preservation, SHA evidence and future minor Save As guard |
| T10 | **PASS (tested paths); PARTIAL (full failure gate)** | Staging/replace, invalid-model preservation, relative-path sync. **Power loss, forced kill, real disk full and multi-process lock are NOT_RUN** |
| T11 | **PASS** | Ubuntu Writer fixture read across Windows x64, Windows ARM64, macOS ARM64 and Linux |
| T12 | **PASS (tested limits); PARTIAL (full resource gate)** | Source file count rejection, file/geometry guards. **512 MiB worst-case throughput/memory, cancellation/timeout and Qt non-blocking integration are NOT_RUN** |

Do not turn a PARTIAL gate into PASS merely because broad CI was green. The implementation and tested native-core scope are complete; **full original F_003 acceptance of every physical failure and resource condition is not yet evidenced**.

## Closing evidence

- PR final HEAD: `5766601ba632ab12ce3013ee1040cdf57f23d727`; verified protected Squash: `cdbf57bd4c596e2643c78b59eee122accfe123d3`.
- 2026-11-15 KST main push: 41/41 checks SUCCESS. All historical PR failed attempts remain visible, with final run passing.
- Exact technical scope and exceptions are reflected in the single authoritative [development checklist](../development-checklist.md).
