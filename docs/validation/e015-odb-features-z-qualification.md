# E_015 — ODB++ UNIX Compress `features.Z` Source Qualification

**First verification date:** 2026-11-12 KST. **Parent:** [Issue #578](https://github.com/july-rust/BoardSI/issues/578), which remains OPEN. **Implementation:** [PR #605](https://github.com/july-rust/BoardSI/pull/605). **Scope:** bounded Source-only `features.Z` decoding and Canonical/Drill source diagnostic; *not* full vendor ODB++ acceptance.

## 1. Format evidence and implementation

- ODB++Design 8.1 Update 4: [Siemens specification](https://odbplusplus.com/wp-content/uploads/sites/2/2024/08/odb_spec_user.pdf), Feature Layer and package compression. UNIX Compress (`.Z`) uses magic `1F 9D` and LZW, **not** RFC 1952 GZIP (`1F 8B`). [UNIX compress implementation](https://github.com/vapier/ncompress).
- A pure Rust decoder `crates/boardsi-import/src/odb_compress_z.rs` supports valid 9–16 bit maximum width, Block Mode CLEAR reset, LSB-first code packing, width-transition group padding, non-block dictionary code 256, KwKwK and bounded dictionary expansion. No new runtime dependency, shell, temp extraction or unsafe path.
- The `normalized_feature_files` adapter in `odb_source.rs` enforces complete validation before publication, accepts precisely `steps/<step>/layers/<layer>/features.Z`, rejects same logical `features` collision, and rejects all unrecognized `.Z` member paths. Both exact compressed `.Z` SHA-256 and derived uncompressed `features` SHA-256 remain in SourceIdentity. Legacy uncompressed file identity remains unchanged.
- The existing Copper L/P/A/limited S Parser, Matrix, optional EDA FID and Drill Source Audit continue to run on the normalized FileMap. Import paths Directory/USTAR, ZIP method 0/8 and single-member GZIP/TGZ share this contract; original transport rules remain unchanged.

## 2. Bounds, negative cases and evidence limitations

| Control | Bound / fail-closed behavior |
| :--- | :--- |
| Physical archive input | 256 MiB, inherited existing ZIP/GZIP/USTAR policy |
| Individual compressed member | 16 MiB, inherited FileMap bound |
| Decompressed logical feature | 16 MiB, enforced as produced |
| Combined original and decoded snapshot | 128 MiB and 8,192 file entries; checked addition and exact path validation |
| Unsupported or malformed header, code width, Dictionary, path, duplicates and unsupported Copper syntax | Entire import ERROR; no partial BoardModel, no verified Copper connectivity |
| Original Source identity | Compressed and decompressed hashes separately recorded; same numeric Geometry does **not** imply identical Source Identity |
| Checksum and EOF | Standard UNIX Compress `.Z` **has no mandatory stream CRC/ISIZE or explicit EOS**. Some syntactically plausible truncation/corruption is inherently undetectable, so no absolute integrity, exporter originality or manufactured board claim |

Tests include independent GNU `gzip -dc` (accepting classic UNIX Compress) decoding of fixed `features.Z` fixture to exact text `UNITS=MM\nF 1\n$0 r100\nL 0 0 1 0 0 P 0\n`; LZW KwKwK, 9→10→11 width boundaries, Block Mode CLEAR, non-block 256, malformed header/code, output limit, duplicate normalized names, unsupported member path, unsupported Copper syntax, exact trace XY ticks/width, SHA identity differentiation and Drill Source Audit parity. Cross-container regression reads the same normalized geometry via Directory, USTAR, ZIP 0/8 and GZIP USTAR. Test Source IDs differ by design; geometry is compared by source-rational coordinates and width, not by full provenance-bearing Point equality.

## 3. GitHub execution gates

**This record must be reconciled against the actual final PR HEAD and exact merged main SHA. No pending check is a PASS.**

- Code PR [#605](https://github.com/july-rust/BoardSI/pull/605) final HEAD `80bf903a76dd1df0712ee2f85e2ef90142eb9744`: **30/30 required/native Checks SUCCESS**, zero failed. Native Rust/Qt Windows x64, Windows ARM64, macOS ARM64 SUCCESS. Earlier formatting, Clippy and provenance-equality test failures were fixed on later HEAD and retained in GitHub history.
- Independent Codex Code Review on final HEAD `80bf903a`: **completed, `has_any_findings=false`, `has_unresolved_findings=false`; review threads 0**.
- Protected squash-only merge [PR #605](https://github.com/july-rust/BoardSI/pull/605) **SUCCESS**, GitHub [Verified one-parent commit `c16d568013be9e5b2d146919d2374f1cad23fbf3`](https://github.com/july-rust/BoardSI/commit/c16d568013be9e5b2d146919d2374f1cad23fbf3); successful GitHub signature verification via commit API.
- Independent exact **code-main** Push workflows for `c16d5680`: **IN_PROGRESS**, do not count as success until all 8 concluded SUCCESS.
- Docs-only final PR required CI, Verified merge and exact docs-main Push: PENDING.
- Formal U/T, independent real Vendor ODB++ fixture, complete copper, EDA/PKG/PIN, plated vias, main Qt ODB Import, global Open/Short, SI solver and parent #578: **NOT_ACCEPTED / OPEN**.

## 4. Scope boundary / follow-on

E_015 covers compressed member syntax, deterministic decoded FileMap, strict source Parser/Drill audit and negative tests **only**. E_016 handles Negative/Island copper; E_017 step transforms; E_018–E_024 subsequent full source, Qt and electrical gates; E_023 independent commercial/cross-CAD oracle remains an external evidence dependency (EB-01). The sole authoritative Stage E completion state is [development-checklist.md](../development-checklist.md).