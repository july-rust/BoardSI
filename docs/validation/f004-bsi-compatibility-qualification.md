# F_004 — Native .bsi Golden, Compatibility and Migration Qualification

> Scope: independent SQLite v1 Golden, native 4-OS compatibility, explicitly
> versioned migration framework, unknown optional/required fail-closed and
> preserved source data. **Status: IN_PROGRESS** until exact HEAD CI,
> independent review, protected Squash and exact-main Push evidence are recorded.

## Contract and tested boundary

- Published native baseline is **SQLite `user_version=1`, file format 1.0**.
  At F_004 creation, there are **zero officially published legacy schema versions**.
- The F_004 `MigrationStep` production registry is intentionally **empty**:
  do not fabricate an official v0/v2 migration or silently downgrade future
  schemas. A synthetic, unit-test-only 0→1 step exercises the exact staging,
  pre/post version guards, semantic validation and rollback logic.
- The production `migrate_to_current(source, target)` does a validated **byte-
  preserving** v1 copy (or same-path no-op) and returns
  `transformed=false`. An unsupported older schema/major/required extension,
  future DDL or damaged payload is not automatically converted.
- Unknown optional extensions are preserved exactly by the raw-file copy,
  but the Reader remains **read-only**. Native reserialization stays prohibited.
  Unknown required extensions are rejected.
- Staging uses existing F_003_1 UUID + OS owner lock and safe replacement;
  its original input SHA is rechecked before replacement. No F_001/F_002
  schema/BSIG format changes.

## Frozen Golden v1 and independent Oracle

| Field | Frozen reference |
| --- | --- |
| Oracle | `tools/validation/f004_reference_fixture.py` (Python sqlite3 + independent BSIG TLV) |
| Fixture | `fixtures/bsi/v1/reference-v1.bsi.b85` (lossless compressed text mirror of actual SQLite file; frozen contents, not regenerated per Reader) |
| v1 app_id | `0x42534931` |
| DDL | `schemas/bsi/v1.sql` (immutable) |
| UUID | `00000000-0000-4000-8000-000000000004` |
| Source | public synthetic `synthetic/golden-v1.asc`; `sha_verified=0` |
| Geometry | Trace, 2 exact source points; Frame/Layer/Net and explicit Frame reference |
| Independent Oracle SHA-256 | `e43f30933c3640141ad583d1d3b91b8416e099bfed089be2fa9671db3bf4c2d7` |
| Complete v1 nine-kind Golden | `fixtures/bsi/v1/all-nine-v1.bsi.b85` |
| Nine-kind frozen SHA-256 | `6fcda969a94766b1f25d0f171c1d986965c9d3efef0fdd1950ac093a134eccc9` |
| Both decoded file sizes | 163840 bytes each |

The compressed checked-in fixture must decode to **exactly** the above SHA.
The independent Oracle also must regenerate the same bytes (rather than
merely generate a mutually compatible but mutable reference). Every OS reads
the *same* decoded frozen file, and checks exact expected source coordinates
and object IDs/relationships independently of the Rust writer.

**Qualification boundary:** The independent Python/SQLite Oracle freezes a minimal
v1 Trace sample. The complementary immutable all-nine-kind v1 Golden is
emitted once from the F_003 production Rust nine-kind model and then checked
by archived bytes/SHA and cross-OS Reader semantics. It is **not** an
independent Python reimplementation of all nine geometry codecs.
An independent vendor CAD source/accuracy Oracle remains F_005.
A future formally released format must add new immutable files and migration
steps **before** release.

## T01–T12 acceptance ledger

| ID | Condition | State |
| :--- | :--- | :--- |
| T01 | Two frozen v1 sources + SHAs + independently produced SQL/BSIG Trace Oracle | PENDING |
| T02 | Rust opens independent Golden | PENDING |
| T03 | Exact i64/i128, relationships and all 9 geometry kinds | PENDING |
| T04 | Same Golden read by Windows x64/ARM64, macOS ARM64 and Linux x64 | PENDING |
| T05 | Future schema/minimum_reader version matrix | PENDING |
| T06 | Unknown required feature rejection | PENDING |
| T07 | Unknown optional read-only and byte-exact copy | PENDING |
| T08 | Step registry and test-only 0→1 transition | PENDING |
| T09 | v1 same-file no-op and repeat copy idempotence | PENDING |
| T10 | Kill, stage recovery, injected failure, lock and ENOSPC protect files | PENDING |
| T11 | Existing Qt and F0031 worker lifecycle regressions | PENDING |
| T12 | All required existing PR CI and exact-main CI | PENDING |

## Exact evidence and closure

- Implementation: [PR #657](https://github.com/july-rust/BoardSI/pull/657).
- Dedicated [F_004 workflow](../../.github/workflows/f004-native-compatibility.yml):
  independent Python Oracle + frozen 9-kind producer + 4 native Rust
  golden/migration/fault jobs + Linux real ENOSPC.
- F_003_1 cross-OS GUI [qualification](f003-1-native-qt-recovery-qualification.md)
  remains separate and is not automatically upgraded.
- PR HEAD SHA, precise CI run IDs, independent review outcome, protected Squash,
  Verified status and exact-main Push results must be appended from observed
  GitHub evidence **before** marking the checklist COMPLETE.

**Explicit nonclaims:** no physical power-failure/disc-cache durability,
no independent production v0→v1/v1→v2 migration (such versions do not exist),
no full native artwork renderer, no electrical connectivity truth, no CAD
vendor-to-native roundtrip or product U/T acceptance.
