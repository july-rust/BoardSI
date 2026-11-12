# E_013 — ODB++ ZIP DEFLATE Method 8 bounded transport qualification

**Date:** 2026-11-12 KST. **Repository:** july-rust/BoardSI. **Parent Issue:** [#578](https://github.com/july-rust/BoardSI/issues/578) (OPEN). **Code PR:** [#601](https://github.com/july-rust/BoardSI/pull/601).

**Current state: IN_PROGRESS — NOT ACCEPTED.** This is the technical/test contract submitted to CI. Replace this status only after independent evidence confirms final PR HEAD, independent review, protected Verified Squash, and exact merged code-main Push. Never count pending/skipped/cancelled checks as PASS.

## Code and source contract

- Rust: `crates/boardsi-import/src/odb_zip.rs`, `odb_source.rs`, ODB Import/Drill Audit CLIs, pinned `flate2 = 1.0.35` using `rust_backend` and locked `miniz_oxide = 0.8.2`. Pure Rust; current project MSRV 1.75. GitHub crates.io index provides checksums; actual native compiler/lock verification requires CI.
- Methods supported: STORED (0), DEFLATE (8), mixed entries; only single disk, standard local+central headers without ZIP64, descriptors, extra fields, comments, encryption or links.
- Raw ZIP DEFLATE (no zlib/gzip header), chunked output with checks before extending the result; precise expanded length, consumed compressed length, CRC32 of expanded bytes; per-entry 16 MiB, total 128 MiB, archive 256 MiB, 8192 entries.
- Source: normalized decoded ODB FileMap content passes to the original geometry/EDA/drill subset; SHA-256 source identity is over **uncompressed logical source bytes**, not ZIP archive bytes. No arbitrary via edges, physical copper proof or SI readiness.

## Submitted regressions (not PASS until CI)

| Type | Test | Evidence state |
| :--- | :--- | :--- |
| Independent oracle | Python zlib raw `hello` vector `cb48cdc9c90700`; exact stream exhaustion | SUBMITTED |
| Positive | ZIP DEFLATE all entries, mixed STORED/DEFLATE, existing STORED | SUBMITTED |
| Transport parity | Directory/USTAR/STORED/DEFLATE normalized Source FileMap equality | SUBMITTED |
| Source integrity | Canonical layers/traces, Source SHA file IDs, empty electrically-connected graph | SUBMITTED |
| Program path | Archive import and optional fabrication/drill-source audit | SUBMITTED |
| Corruption | Header CRC mutation, local/central method, sizes, compressed bit corruption | SUBMITTED |
| Decompression bomb | Highly compressible 1 MiB member with forged small declared size; bounded decoder | SUBMITTED |
| Unsafe paths | Traversal, duplicate entries, multiple roots | SUBMITTED |
| Legacy | Existing STORED invalid flags, symlink, prefix/trailer, USTAR and unrelated PADS tests | PENDING CI |

`ODB_ZIP_DEFLATE_INVALID`, `ODB_ZIP_DEFLATE_SIZE_MISMATCH`, and `ODB_ZIP_DEFLATE_TRAILING_DATA` are the new bounded error classes. Invalid members abort the whole source import.

## External specification and evidence boundaries

- [PKWARE ZIP APPNOTE](https://www.pkware.com/documents/casestudies/APPNOTE.TXT): general ZIP headers, method and CRC framing.
- [ODB++Design 8.1 Update 4](https://odbplusplus.com/wp-content/uploads/sites/2/2024/08/odb_spec_user.pdf): source product structure; not proof of whole exporter compliance.
- [Historical D_056_09 Method 0 baseline](../architecture/d056-09-odb-zip-transport.md).
- Independently licensed public Vendor Export and cross-format geometry oracle: **NOT_VERIFIED**, retained in EB-01/E_023. Synthetic data is not vendor qualification.
- GZIP/tgz, `features.Z`, compressed-internal member parsing, negative copper, step repeat, full Package/Pin, plated via, main Qt ODB++, global Open/Short, RLGC/S, formal U/T and full Issue #578: **NOT_ACCEPTED**.

## Protected completion gate (fill with actual evidence)

- [ ] Final PR HEAD SHA and required checks; Windows x64/ARM64 and macOS ARM64 actual result
- [ ] Independent review evidence and unresolved review threads 0
- [ ] GitHub-Verified protected Squash merge, SHA and single parent
- [ ] Exact code-main Push workflows, failures/skips/cancelled recorded truthfully
- [ ] Sole checklist, plan, validation index final limited COMPLETE reconciliation
- [ ] PR closed/merged and #578 OPEN with remaining bounded E_014–E_024 dependencies

**Do not promote this document or E_013 to COMPLETE based solely on a submitted patch.**
