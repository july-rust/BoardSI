# F_004 — Native .bsi v1 independent compatibility qualification

> Status: **PARTIAL / IN PROGRESS**, 2026-11-16 KST. This document does not declare the complete F_004 12-item acceptance.

## Implemented evidence

- Frozen v1 empty-board binary: [Golden .bsi](../../schemas/bsi/golden-v1-empty.bsi), [SHA-256 manifest](../../schemas/bsi/golden-v1-empty.sha256). Created once from a separate SQL oracle, then committed as fixed bytes; cross-OS Reader verifies the pinned digest.
- [Independent v1 SQL semantic seed](../../schemas/bsi/golden-v1-empty.sql), applied to the **unmodified published v1 DDL**; separate from production Rust Writer.
- [Seven Rust regressions](../../crates/boardsi-native/tests/f004_compatibility.rs): independent reader baseline; future minor Read-Only; optional extension preservation through save refusal; required/future DDL/minimum reader rejection; unversioned table rejection; unpublished v0 rejection.
- [Four-host native CI](../../.github/workflows/f004-compatibility.yml): Windows x64, Windows ARM64, macOS ARM64, Linux x64. Results are per exact PR HEAD and must be checked before recording PASS.
- All adversarial tests verify that attempted load and save cannot alter the original SQLite bytes.

## Limitations / outstanding F_004 acceptance

| Contract | Evidence status |
|---|---|
| Independent source-controlled SQL seed and **frozen v1 binary SHA-256** | Implemented; native CI results belong to PR #658 |
| Pinned immutable **empty-board** binary `.bsi` and SHA-256 | **IMPLEMENTED**; rich 9-geometry independent oracle still **NOT_DONE** |
| Full historical version migration engine, staging/crash injection/idempotence | **NOT_DONE**; no official pre-v1 files exist |
| Future/minor unknown extension read-only protection | Existing production behavior + additional test, not opaque rewrite |
| GUI lifecycle, entire CI, independent review, protected merge/main push | Needs exact execution evidence |
| Vendor/CAD cross-format equality, real power cut, physical SI, product U/T | F_005/external scope: **NOT_VERIFIED** |

This step must not be promoted to 100% COMPLETE until a rich 9-geometry independent binary Golden, migration framework, 12-test acceptance, review, protected merge, exact-main CI and one checklist reconciliation are truly evidenced. A synthetic v0 or v2 experiment cannot count as a released historical-file migration.

## Sources

[F_001 compatibility policy](../architecture/f001-bsi-format-contract.md) · [F_002 byte contract](../architecture/f002-bsi-geometry-binary-layout.md) · [F_003_1 baseline](f003-1-native-qt-recovery-qualification.md) · [PR #658](https://github.com/july-rust/BoardSI/pull/658).
