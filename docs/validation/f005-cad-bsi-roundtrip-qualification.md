# F_005 — CAD Source → Canonical → Native .bsi → Canonical qualification

> **Status: IN PROGRESS / NOT YET QUALIFIED.** Evidence date: 2026-11-16 KST.
> Code candidate: [PR #660](https://github.com/july-rust/BoardSI/pull/660).
> Completion requires exact final-HEAD CI, independent review, protected Squash and verified merged-main push.

## Scope and ownership

- The supported PADS V9.4 subset uses the pre-existing strict `save-pads` path; F_005 adds a real independent worker-vs-Canonical round-trip assertion.
- The bounded ODB++ SOURCE-only `Directory + Single STEP` / positive straight Trace and round/rect Pad subset uses the same `odb_qt_wire::render` preflight for Qt and native saves. Unsupported copper is not silently deleted. The canonical model is owned by `odb_source::import_directory`; source manifest comes from verified logical path + raw SHA-256 entries, not the user's chosen directory name.
- New `boardsi-native-file save-odb <root> <step> <target.bsi> <accepted-manifest-sha256>` verifies the Qt-accepted manifest, repeats source read immediately before writing, delegates to the existing `boardsi_native::save`, then emits a bounded native receipt. It does not add a new SQLite schema or geometry codec.
- The main Qt `Save Project (.bsi)` uses the accepted ODB root, STEP and manifest, not the last attempted candidate. It uses the existing asynchronous QProcess and receipt validation. A stale source cannot be promoted without a successful new Import.
- KiCad 9, IPC-2581 and Gerber+Drill remain SOURCE-inventory-limited: not full Canonical `.bsi` import. The E_028 KiCad/PADS independent via oracle remains source-geometric, not electrical.
- Physically manufactured connectivity, return path, RLGC/S-parameters, vendor full-board qualification, EB-01..07, PH-08 and product U/T are **NOT ACCEPTED** by F_005.

## Canonical round-trip invariants

A real importer creates model A; native Writer stores A; separately loaded native Reader produces B. The shared `canonical_semantics_equal` compares SourceIdentity / raw SHA, coordinate frames / i128 rational, layers/materials, all geometry kinds, Net/Component/Pin, padstacks/drills/vias, outlines and connectivity evidence, independent of row/order. A structural match is not physical verification. No unsupported source geometry may be lost to make this comparison pass.

The synthetic ODB witness copies the E_021 exact two-copper-layer contract: three source files (matrix and two features), two straight traces, two round/rectangle pads, with independently asserted 1.5 mm coordinate. PADS uses existing source V9.4 fixture and the real `save-pads` worker. The source manifest is computed independently from raw file SHA-256, matching the Qt `hex(path)|hex(digest)\\n` byte protocol.

## F005 test ledger (candidate, verify against actual CI)

| Gate | Actual test | Status |
|---|---|---|
| T01 | PADS V9.4 real `save-pads`→load and exact Canonical semantics | NOT_RUN |
| T02 | ODB++ Directory/STEP `save-odb` worker | NOT_RUN |
| T03 | PADS model round-trip all stored fields | NOT_RUN |
| T04 | ODB model round-trip all stored fields, Native Save As | NOT_RUN |
| T05 | Exact decimal 1.5 mm source witness / Rational frame | NOT_RUN |
| T06 | Geometry, Layer/Pad/Net/source references in shared semantic comparison | NOT_RUN |
| T07 | Full SourceIdentity + capability in shared semantic comparison | NOT_RUN |
| T08 | Independent raw SHA-256 manifest + separate F_004 BSIG Python oracle | NOT_RUN |
| T09 | E_028 PADS/KiCad bounded Via cross-format oracle; native KiCad excluded | NOT_RUN |
| T10 | Changed manifest, unsupported copper, undeclared step, last-good preservation | NOT_RUN |
| T11 | Actual main Qt ODB Import/Native Save/Open/stale-source rejection, event loop | NOT_RUN |
| T12 | F_001~F_004 native Golden, migration, file failure, Qt regressions | NOT_RUN |

### Commands and evidence

- `cargo test -p boardsi-native-file --test f005_cad_roundtrip`
- `cargo test -p boardsi-import --test e028_cross_format_via_oracle`
- `python3 tools/validation/test_f004_golden_oracle.py`
- `ctest --test-dir <configured-native-qt-build> -R qt-f005-main-odb-native-roundtrip --output-on-failure`
- Four-platform CI on Windows x64 / Windows ARM64 / macOS ARM64 / Linux x64, with exact commit status and logs recorded upon successful execution.

Actual unsupported CAD constructs remain fail-closed. Normal vendor ODB++ full interchange and manufactured electrical graphs remain NOT_VERIFIED regardless of test outcome.

## Final acceptance receipts

Pending: final PR HEAD, per-platform CI checks and runs, independent final-HEAD review, protected Verified squash SHA, exact main Push CI and checklist first-check/evidence date. **Do not claim 100% COMPLETE before all gates are actually satisfied.**
