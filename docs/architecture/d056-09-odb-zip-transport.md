# D_056_09 — ODB++ ZIP STORED transport (historical source baseline)

> Historical D_056_09 Method 0 record: preserved unchanged below. The **E_013 Method 8 successor** is documented at the end and in [E_013 validation](../validation/e013-odb-zip-deflate-qualification.md). Do not read the historical unsupported list as the current combined method 0/8 contract.

**First implementation:** 2026-11-11 KST. **Owner issue:** [#578](https://github.com/july-rust/BoardSI/issues/578). **Code PR:** [#579](https://github.com/july-rust/BoardSI/pull/579). **Status:** one bounded transport subset merged; **D_056_09 overall OPEN**.

## Authority and baseline

- [Siemens ODB++Design specification 8.1 Update 4](https://odbplusplus.com/wp-content/uploads/sites/2/2024/08/odb_spec_user.pdf): ODB product structure, `matrix/matrix`, `steps/<step>` and layers. This project is an independent limited subset, not a vendor certification.
- [PKWARE ZIP Application Note (APPNOTE.TXT)](https://www.pkware.com/documents/casestudies/APPNOTE.TXT): ZIP member/local header, central directory, end-of-central-directory, method 0 (STORED), CRC-32 and optional ZIP extensions. Only the explicit subset below is accepted.
- [D_056_08 architecture](d056-08-odb-source-adapter.md) and [single checklist](../development-checklist.md): D_056_08 eight verified source-only slices remain separate. [Main ODB issue #10](https://github.com/july-rust/BoardSI/issues/10), cross-format [#532](https://github.com/july-rust/BoardSI/issues/532) and universal parent [#520](https://github.com/july-rust/BoardSI/issues/520) are not closed by this transport change.

## Implemented source transport

Code: `crates/boardsi-import/src/odb_zip.rs`, shared product-root normalization in `odb_tar.rs`, file routing in `odb_source.rs` and diagnostic CLI entrypoints.

- `boardsi-odb-import <unpacked-directory-or-USTAR-or-STORED-ZIP> <step> [--eda-nets] [--source-holes]`
- `boardsi-odb-drill-audit <unpacked-directory-or-USTAR-or-STORED-ZIP> <step> [--hits]`

The ZIP reader accepts an **uncompressed STORED ZIP container only**, not general ZIP decompression: one disk, zero archive/member comments, no extra fields or descriptors, no encryption, no ZIP64, no executable/link/device type, one product root. It verifies:

1. Bounded archive length (maximum 256 MiB), file count (8192), individual file size (16 MiB), and cumulative member bytes (128 MiB).
2. Exact local-header ↔ central-directory file name, compression mode, flags, size and CRC-32 equality for all entries.
3. Actual file data CRC-32, nonoverlapping contiguous local member regions, no hidden unlisted prefix/appendix.
4. No duplicate files or unsafe logical paths; no traversal, absolute paths, or backslash escapes.
5. Common root normalization into one `OdbFileMap` and the same strict Canonical/optional EDA FID/drill-source-audit logic as Directory and USTAR.

Input is inspected without external decompression commands, shell, or extracting to the filesystem. Existing uncompressed USTAR and unpacked Directory handling remain available. Source SHA-256 identities are computed on each accepted logical file's original content bytes. The ZIP CRC-32 checks transport integrity, **not** cryptographic authenticity.

The API remains intentionally backwards compatible: `import_tar_file` accepts the narrow local ZIP subset via shared routing, while `import_archive_file` makes that expanded scope explicit. No versioned Qt main import/IPC contract is changed.

## Negative tests and qualification scope

Unit and path integration tests use synthetic, redistributable two-layer ODB source material:

- STORED ZIP → exact source file map → Canonical geometry with 2 copper layers and two traces.
- Local ZIP file path → Canonical import, optional source-hole observation, optional drill-hit audit through the same immutable read snapshot; no manufactured edges.
- Reject CRC mutation, duplicate file, traversal path, DEFLATE method, trailing bytes, changed local header, data-descriptor flag, mixed product roots, and declared Unix-origin symlink.

Existing TAR/Copper/EDA/Hole test suites and platform CI still apply. Synthetic fixtures are *not* independent Siemens exporter or CAM oracle evidence.

## Deliberately unsupported

- General ZIP **DEFLATE/compressed entries**, multi-disk, ZIP64, archive comments, extra fields, descriptors, and self-extracting ZIP prefixes.
- `.tgz`, `.tar.gz`, `features.Z` and arbitrary compressed member formats; this PR does not add a decompressor.
- Full ODB++ Design Symbol/Contour/Negative/Step-Repeat; complete PKG/PIN/Component, manufactured Drill/Via/Plating, Stackup z/Dk/Df/thickness, main Qt Artworks, physical electrical Graph/Open/Short, and SI readiness.
- Real commercial CAD exporter interoperability and independent physical/vendor evidence.

Unsupported cases remain **FAIL-CLOSED**, not silently skipped and not promoted to connected-layer or SI evidence.

## Acceptance separation

- **Code PR #579**: final HEAD `7a086ab050c1347a6ab2305371a51c2a18300dab`, 30/30 PR Checks SUCCESS; unresolved review threads 0.
- **Protected merge**: GitHub-verified, single-parent Squash `8069420d10403fb3a3f5faaab07df07ecd7bdb0f`.
- **Independent exact-code-main SHA Push CI**: exact commit `8069420d10403fb3a3f5faaab07df07ecd7bdb0f` has **8/8 successful workflows**, zero failures: [Numerical](https://github.com/july-rust/BoardSI/actions/runs/42936771992), [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/42936772184), [Docs](https://github.com/july-rust/BoardSI/actions/runs/42936772016), [PH-06](https://github.com/july-rust/BoardSI/actions/runs/42936772170), [PH-07](https://github.com/july-rust/BoardSI/actions/runs/42936772203), [Native FEM](https://github.com/july-rust/BoardSI/actions/runs/42936772055), [Lossless FEM](https://github.com/july-rust/BoardSI/actions/runs/42936771949), [G0](https://github.com/july-rust/BoardSI/actions/runs/42936772137). The Qt run was initially **CANCELLED** for Windows x64/ARM64 but both jobs were independently retried; final **attempt 3 SUCCESS**. Cancelled attempts are not counted as success. PR 30/30 Checks and post-squash Push 8/8 are separate proofs.
- **Independent docs-main Push CI**: [docs PR #580](https://github.com/july-rust/BoardSI/pull/580) was protected GitHub-verified Squash as `140349e29be84110a465d3ab95961264089dc849`, with its exact `main` Push **8/8 successful workflows**. Docs-only PR validated with 13 successful checks and 16 correctly skipped native checks.
- **D_056_09 whole-stage gate**: OPEN under #578. This file is an acceptance record for ZIP STORED transport only.

Do not report compressed ODB++ products generally, verified plated vias, electrical Open/Short, or solver-ready S-parameters as supported until independently validated.


## E_013 successor — ZIP raw DEFLATE Method 8 (2026-11-12 KST)

**Code review/CI:** [PR #601](https://github.com/july-rust/BoardSI/pull/601) source subset **COMPLETE (2026-11-12 KST; final docs-PR protected closeout conditional)**: final HEAD `9554db34` 38/38 Checks SUCCESS; independent Codex earlier P2 corrected, follow-up bot 👍 and unresolved threads 0; [GitHub Verified Squash `06265985`](https://github.com/july-rust/BoardSI/commit/06265985fdf7888e2b16ace7daa4b7e2bf48a0b2), one parent; exact code-main Push **8/8 workflow SUCCESS** including [Numerical native](https://github.com/july-rust/BoardSI/actions/runs/43165652402) and [Qt native](https://github.com/july-rust/BoardSI/actions/runs/43165652649). Historical D_056_09 STORED-only record above remains unchanged.

- Accept ZIP member methods **0 (STORED)** and **8 (raw DEFLATE)**, including mixed-method packages. ZIP data is decompressed **in memory** through the Rust `flate2 = 1.0.35` pure-Rust backend (`miniz_oxide = 0.8.2`), not via an external process, temporary file, zlib wrapper or gzip.
- Shared bounded original data policy: archive 256 MiB, per expanded file 16 MiB, total expanded files 128 MiB, 8192 entries. Actual output is limited **while decoding** to each member's header-advertised size and policy maximum, and cumulative declared sizes are checked before expanding.
- Enforce entire raw stream consumption and exact decoded byte count, then CRC32 on expanded data, and local/central header equality. Invalid or unsupported ZIP members fail the whole import, never yielding a partial Canonical board.
- Existing strict ZIP limits remain: no ZIP64, multiple disks, data descriptors, encryption, extra fields, comments, self-extracting prefixes, symlinks, or path traversal.
- All accepted decoded data feeds the **existing** root normalization, `OdbFileMap`, Source SHA-256, Canonical geometry and optional Drill Audit/EDA ownership. ZIP CRC32 is transport integrity, **not** cryptographic origin proof.

**Scope boundary:** This does **not** add tar.gz/tgz (E_014), features.Z (E_015), step transforms/copper negative (E_016–017), full EDA PKG/PIN, qualified plated vias, main Qt ODB++ import, electrical connectivity, physical SI, vendor oracle or complete #578 acceptance. Unsupported source semantics remain UNKNOWN/fail-closed.
