# E_014 — ODB++ GZIP tar.gz/tgz bounded transport qualification

**Date:** 2026-11-12 KST  
**Repository:** [july-rust/BoardSI](https://github.com/july-rust/BoardSI)  
**Parent:** [#578](https://github.com/july-rust/BoardSI/issues/578), remains OPEN  
**Code PR:** [#603](https://github.com/july-rust/BoardSI/pull/603)  
**Status:** IN_PROGRESS — do not mark COMPLETE before exact final PR HEAD, independent review, protected Verified Squash, and exact new main Push are evidenced.

## 1. Bounded acceptance contract

Add one independent GZIP (RFC 1952) source-transport path feeding the already-qualified limited ODB++ USTAR reader and existing immutable Source FileMap. Accept a **single** gzip member with optional RFC1952 FEXTRA/FNAME/FCOMMENT/FHCRC, reject reserved flag bits, malformed/truncated inputs, header metadata exceeding 64 KiB, CRC32/ISIZE mismatch, unconsumed compressed bytes, concatenated members and trailing garbage. RFC1952 text hint is ignored; header name/comment must never become a filesystem path.

Do not extract to disk, launch external tools, infer missing board geometry, or modify Qt/IPC/SI semantics. File extensions `.tgz` and `.tar.gz` are supported at the existing archive path call site; **signature determines the decoder**, not the extension. Raw in-memory `import_tar(bytes,...)` remains USTAR-only for compatibility.

## 2. Bounded resource and fail-closed policy

| Gate | Maximum / behavior |
| :--- | :--- |
| Compressed package | 256 MiB, checked before reading and at decoder |
| Expanded USTAR stream | 256 MiB actual bytes, checked **during** raw-DEFLATE |
| GZIP metadata header | 64 KiB |
| Individual logical Source file | 16 MiB, existing USTAR limit |
| Cumulative Source files | 128 MiB, existing USTAR limit |
| USTAR members | 8,192, existing limit |
| Arithmetic, truncation, incorrect ISIZE/CRC | Reject entire import, no partial Source FileMap |
| Unsafe path, link, duplicate, multiple product roots | Existing `odb_tar::read_tar()` rejects entire import |

GZIP CRC32 and original-byte SHA-256 serve different purposes; CRC verifies transport integrity and cannot prove exporter identity or physical truth. An archive identity is not claimed identical between GZIP, ZIP and raw USTAR; the **validated logical contents** and their Source SHA/provenance should match.

## 3. Implementation routes

- `crates/boardsi-import/src/odb_gzip.rs`: bounded single-member header and raw-DEFLATE, CRC32/ISIZE verification, strict input consumption.
- `crates/boardsi-import/src/odb_source.rs`: add GZIP magic dispatch to the existing read-once archive path. All later import and drill-source audit continue using the same immutable normalized `OdbFileMap`.
- `crates/boardsi-import/src/odb_tar.rs`: **unchanged** USTAR checksum/path/prefix/member validation, preserving legacy behavior.
- `crates/boardsi-import/src/odb_zip.rs`: **unchanged** E_013 ZIP 0/8 transport.
- `crates/boardsi-import/src/lib.rs`: register source-only transport module.

## 4. Synthetic and independent test matrix

| Test | Acceptance criterion | Evidence |
| :--- | :--- | :--- |
| GZIP+USTAR positive | 2 layer, 2 trace source import, no promoted electrical edges | PR #603 test/CI pending |
| Directory-equivalent USTAR bytes | Normalized file map and Canonical source path identical | PR #603 test/CI pending |
| `.tar.gz` and `.tgz` real paths | Both use archive source path and drill-source snapshot | PR #603 test/CI pending |
| GZIP optional metadata + FHCRC | Header metadata accepted but never trusted for paths; CRC verified | PR #603 test/CI pending |
| Independent Python gzip vector | RFC1952 compression cross-implementation decode | PR #603 test/CI pending |
| Truncated stream/header, flags, checksum, ISIZE | Deterministic fail-closed | PR #603 test/CI pending |
| Expansion limit | In-flight rejection before an excessive allocation | PR #603 test/CI pending |
| Concatenated member / trailing data | Reject the entire package | PR #603 test/CI pending |
| Unsafe / duplicate / multiple-root / invalid USTAR | Existing parser rejects after valid GZIP decode | PR #603 test/CI pending |
| ZIP STORED/DEFLATE and unrelated PADS | No regression | Final CI pending |
| Independent commercial exporter oracle | **NOT_RUN** (outside E_014) | EB-01 / E_023 |

**No test result, native execution, reviewer approval or workflow success is asserted by merely submitting these files.** Capture actual final statuses below at closeout.

## 5. Exact final operational evidence (to complete only after observation)

- Final PR HEAD: **PENDING**
- Rust checks and Native Windows x64 / ARM64 / macOS ARM64: **PENDING**
- Independent reviewer findings / unresolved threads: **PENDING**
- GitHub Verified single-parent protected Squash: **PENDING**
- Exact new code-main SHA and its 8 Push workflow outcomes: **PENDING**
- Documentation closeout final SHA and Push outcomes (if separate PR): **PENDING**

**Final E_014 status is NOT_ACCEPTED until all applicable gates succeed.**

## 6. Limitations and parent status

This task accepts **GZIP as transport**, not all ODB++ syntax. Unsupported ZIP extensions, GZIP multi-member, PAX/links/sparse USTAR, compressed `features.Z` (E_015), Negative/Island/Step Transform (E_016–E_017), complete Package/Pin, manufactured plating/via, Qt main ODB import, electrical Open/Short and physical SI are not promoted. Independent vendor and cross-CAD oracle remains unqualified (EB-01). Parent [#578](https://github.com/july-rust/BoardSI/issues/578) stays OPEN.

**Do not conflate E_014 bounded completion with D_056_09 / #578 product acceptance.**
