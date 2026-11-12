# E_013 — ODB++ ZIP DEFLATE Method 8 bounded transport qualification

**Date:** 2026-11-12 KST. **Repository:** july-rust/BoardSI. **Parent Issue:** [#578](https://github.com/july-rust/BoardSI/issues/578) (OPEN). **Code PR:** [#601](https://github.com/july-rust/BoardSI/pull/601).

**Current state: BOUNDED CODE COMPLETE — final documentation closeout conditional.** [PR #601](https://github.com/july-rust/BoardSI/pull/601) exact final HEAD `9554db340e09b662b93b2d98257db9a2adcad073` has 38/38 checks SUCCESS, 0 unresolved review threads, protected GitHub Verified single-parent Squash `06265985fdf7888e2b16ace7daa4b7e2bf48a0b2`, and exact code-main Push 8/8 workflows SUCCESS. E_013 sole-checklist COMPLETE becomes unconditional after this final documentation PR also passes required docs CI, protected merge and exact docs-main Push. Never count initial failed/cancelled checks or docs-only native SKIPPED as PASS.

## Code and source contract

- Rust: `crates/boardsi-import/src/odb_zip.rs`, `odb_source.rs`, ODB Import/Drill Audit CLIs, pinned `flate2 = 1.0.35` using `rust_backend` and locked `miniz_oxide = 0.8.2`. Pure Rust; current project MSRV 1.75. GitHub crates.io index provides checksums; actual native compiler/lock verification requires CI.
- Methods supported: STORED (0), DEFLATE (8), mixed entries; only single disk, standard local+central headers without ZIP64, descriptors, extra fields, comments, encryption or links.
- Raw ZIP DEFLATE (no zlib/gzip header), chunked output with checks before extending the result; precise expanded length, consumed compressed length, CRC32 of expanded bytes; per-entry 16 MiB, total 128 MiB, archive 256 MiB, 8192 entries.
- Source: normalized decoded ODB FileMap content passes to the original geometry/EDA/drill subset; SHA-256 source identity is over **uncompressed logical source bytes**, not ZIP archive bytes. No arbitrary via edges, physical copper proof or SI readiness.

## Verified regressions — exact code-main source SHA

| Type | Test | Evidence state |
| :--- | :--- | :--- |
| Independent oracle | Python zlib raw `hello` vector `cb48cdc9c90700`; exact stream exhaustion | PASS (final PR HEAD and code-main CI) |
| Positive | ZIP DEFLATE all entries, mixed STORED/DEFLATE, existing STORED | PASS (final PR HEAD and code-main CI) |
| Transport parity | Directory/USTAR/STORED/DEFLATE normalized Source FileMap equality | PASS (final PR HEAD and code-main CI) |
| Source integrity | Canonical layers/traces, Source SHA file IDs, empty electrically-connected graph | PASS (final PR HEAD and code-main CI) |
| Program path | Archive import and optional fabrication/drill-source audit | PASS (final PR HEAD and code-main CI) |
| Corruption | Header CRC mutation, local/central method, sizes, compressed bit corruption | PASS (final PR HEAD and code-main CI) |
| Decompression bomb | Highly compressible 1 MiB member with forged small declared size; bounded decoder | PASS (final PR HEAD and code-main CI) |
| Unsafe paths | Traversal, duplicate entries, multiple roots | PASS (final PR HEAD and code-main CI) |
| Legacy | Existing STORED invalid flags, symlink, prefix/trailer, USTAR and unrelated PADS tests | PASS (workspace CI, no unrelated product acceptance) |

`ODB_ZIP_DEFLATE_INVALID`, `ODB_ZIP_DEFLATE_SIZE_MISMATCH`, and `ODB_ZIP_DEFLATE_TRAILING_DATA` are the new bounded error classes. Invalid members abort the whole source import.

## External specification and evidence boundaries

- [PKWARE ZIP APPNOTE](https://www.pkware.com/documents/casestudies/APPNOTE.TXT): general ZIP headers, method and CRC framing.
- [ODB++Design 8.1 Update 4](https://odbplusplus.com/wp-content/uploads/sites/2/2024/08/odb_spec_user.pdf): source product structure; not proof of whole exporter compliance.
- [Historical D_056_09 Method 0 baseline](../architecture/d056-09-odb-zip-transport.md).
- Independently licensed public Vendor Export and cross-format geometry oracle: **NOT_VERIFIED**, retained in EB-01/E_023. Synthetic data is not vendor qualification.
- GZIP/tgz, `features.Z`, compressed-internal member parsing, negative copper, step repeat, full Package/Pin, plated via, main Qt ODB++, global Open/Short, RLGC/S, formal U/T and full Issue #578: **NOT_ACCEPTED**.

## Independent CI, review and merge evidence (2026-11-12 KST)

- **Final PR HEAD:** `9554db340e09b662b93b2d98257db9a2adcad073`. [PR #601](https://github.com/july-rust/BoardSI/pull/601) 38/38 exact HEAD checks SUCCESS; **12/12 ODB ZIP unit regressions PASS**, including initial STORED behavior and new raw-DEFLATE/mixed/CRC/bomb/Source parity. Rust 1.75 `cargo fmt --check` and locked offline clippy both PASS.
- **History kept:** an earlier test failed because a synthetic mutated 32-bit ZIP header size field was written with an 8-byte `usize`; fixed in `380f2c9`. A later earlier HEAD failed Rust formatting; exact rustfmt hunks applied in `9554db34`, then final CI passed. Earlier cancelled workflow runs are not reclassified as successes.
- **Independent review:** Codex previously reported a P2 defect for Method 8 compression-option header bits `0x0006`; fixed in `115b6267`, regression added for DEFLATE legal options and encryption/descriptor failures, thread answered and resolved. After final HEAD was authored (05:46Z), the Codex bot posted 👍 at 05:48Z; 0 unresolved threads. That reaction does not expose an explicit reviewed commit SHA, so it is not represented as an SHA-bound written review.
- **Protected merge:** [GitHub Verified single-parent Squash `06265985fdf7888e2b16ace7daa4b7e2bf48a0b2`](https://github.com/july-rust/BoardSI/commit/06265985fdf7888e2b16ace7daa4b7e2bf48a0b2), exact `main` HEAD at validation, parent `8f9c4889cd2f12d79945d05beb8d7e8db4a05002`.
- **Independent merged-code Push:** exact `head_sha=06265985...`, `event=push`, 8/8 workflows completed **SUCCESS**, no failed/skipped/cancelled workflows: [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/43165652649), [Numerical core](https://github.com/july-rust/BoardSI/actions/runs/43165652402), [Docs](https://github.com/july-rust/BoardSI/actions/runs/43165652452), [Native FEM](https://github.com/july-rust/BoardSI/actions/runs/43165652530), [PH-06](https://github.com/july-rust/BoardSI/actions/runs/43165652371), [PH-07](https://github.com/july-rust/BoardSI/actions/runs/43165652387), [Lossless FEM](https://github.com/july-rust/BoardSI/actions/runs/43165652461), [G0](https://github.com/july-rust/BoardSI/actions/runs/43165652408). Rust native Windows x64/ARM64 and macOS ARM64 PASS; Qt native Windows x64/ARM64 and macOS ARM64 PASS, Linux Qt PASS.
- **Documentation:** this report, singular checklist, E closure plan and historical architecture were reconciled without changing Stage A/B/C/D U/T totals or false assumptions about ODB++ product support. Final docs-only PR must pass protected docs CI, verified Squash and exact docs-main Push; docs-only skipped native jobs do **not** constitute new code tests.

## Closeout boundary

- [x] Final HEAD CI 38/38 SUCCESS and exact Native Windows x64/ARM64, macOS ARM64 records
- [x] Codex P2 fixed; subsequent bot 👍; unresolved review threads 0
- [x] Protected GitHub-Verified Squash, single-parent main commit
- [x] Exact code-main Push 8/8 SUCCESS
- [x] Source subset and old failed/cancelled histories accurately separated
- **Final docs gate (conditional):** this document's PR required CI, protected GitHub-Verified Squash and exact docs-main Push are independently verified in the parent [Issue #578](https://github.com/july-rust/BoardSI/issues/578) closeout comment after merge; no post-facto PASS preclaim.

**Only ZIP methods 0 and 8 / source-limited Canonical transport are complete.** Parent #578, real vendor oracle, manufacturing/graph/SI and E_014–E_024 remain OPEN/NOT_VERIFIED. After final docs gate completes, report the bounded E_013 unit COMPLETE without closing #578.
