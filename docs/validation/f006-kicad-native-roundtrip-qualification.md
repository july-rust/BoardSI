# F_006 — KiCad 9 Source → Canonical → Native .bsi → Qt acceptance

> **State: IMPLEMENTATION + TEST-ONLY REPAIRS MERGED; FINAL MAIN PUSH CI PENDING.** PR #662 delivered the bounded KiCad9 importer; PR #663 repaired F_006 concurrent test fixtures; PR #664 repaired the inherited F_005 fixture race. All three are Verified Squash-merged into `main`. First `100% COMPLETE` date remains UNSET until the exact `main` commit `f129eb89781a58fad8732efd3eafb7659b7b269c` passes all required post-merge workflows.

## Scope contract

- Input: KiCad 9 PCB S-expression file **version 20241229**, exact file SHA-256.
- Accepted in the bounded native adapter: layer declaration/order, logical net declaration, straight `segment` copper tracks (integer-nm geometry, UUID/tstamp when present), and unmodified standard through `via` geometry with nominal round padstack/drill.
- Existing E_026 Source inventory is retained. This F_006 adapter is **not** an XSD/vendor KiCad certification or an electrical graph importer.
- KiCad copper order: F.Cu, In1.Cu …, B.Cu. KiCad source *numeric ordinals* are retained in Layer source IDs, not used as physical order.
- Source-backed nominal general board thickness is preserved as a SourceIdentity condition, **not** independently verified individual copper/dielectric stackup z/thickness/Dk/Df.
- Standard through-via `Drill.plated=true` records **KiCad nominal CAD design intent**, not actual manufactured plating; `Via.connected_layers=[]`, physical graph empty, SI NOT_RUN.
- Explicitly reject unknown PCB geometry: footprint/pads/zones/arcs/graphic board outlines, blind/buried/microvias and pruning/locked/free via flags. No silent partial data loss.
- Native v1 SQLite DDL/BSIG unchanged; Writer/Reader checked semantic equality and guarded source SHA re-evaluation.
- Main Qt's `Import KiCad → .bsi` action runs the actual native file Worker asynchronously. The accepted source geometry may be displayed as a bounded source-only readout, not as verified physical/electrical connectivity.
- Independent real-world KiCad exporter evidence, full KiCad supported formats, manufacturing continuity, solver/RLGC/S parameters and formal M0 U/T acceptance remain **NOT_VERIFIED**.

## Reproduction

```sh
cargo +1.75.0 fmt --all -- --check
cargo +1.75.0 test -p boardsi-import --locked kicad9_source
cargo +1.75.0 test -p boardsi-native-file --locked --test f006_kicad_roundtrip
cargo +1.75.0 test --workspace --locked
# Qt build with BOARDSI_NATIVE_FILE_WORKER set to the built real binary:
ctest --test-dir <qt-build-directory> -R qt-f006-main-kicad-native-roundtrip --output-on-failure
```

## F_006 T01–T12 qualification

| ID | Gate | Status |
|:---|:---|:---|
| T01 | KiCad 9 version + malformed S-expression rejection | PASS — KiCad9 version/grammar rejection (E_026 + F_006) |
| T02 | 2/4/6/8/32-Cu accurate source order | PASS — E_026 2/4/6/8/32 copper inventory + F_006 four-layer Native order |
| T03 | straight Track exact nm/width/layer/net mapping | PASS — exact integer nm, rational width, Layer/Net and UUID |
| T04 | through Via nominal geometry, no physical contact invention | PASS — nominal through Via round padstack/drill; no invented contact |
| T05 | exact Canonical → .bsi → Canonical semantic roundtrip | PASS — real Native save/load semantic equality on four OS |
| T06 | v1 Golden and F_004 backward compatibility | PASS — F_004 v1 Golden + legacy compatibility on four OS |
| T07 | independent SQL/BSIG/Source geometry oracle | PASS — raw SQLite/BSIG/SHA independent oracle in F_006 test |
| T08 | unsupported geometry/flags/malformed input fail-closed | PASS — unsupported footprint/zone/arc/flags malformed input reject |
| T09 | SHA source mutation/cancel/lock/ENOSPC preserve old native | PASS — real pre-publish source race no-clobber four OS + shared native resilience |
| T10 | main Qt import/save/open/artwork picking/layer filtering | PASS — actual Main Qt import/save/open/drawn annulus/round track/picking/filter four OS |
| T11 | native Rust/Qt Windows x64, ARM64, macOS ARM64, Linux | PASS — Linux/Windows x64/Windows ARM64/macOS ARM64 native+Qt |
| T12 | existing F_001–F_005 + post-merge main regression | PENDING — PR #662/#663/#664 code and PR CI passed; exact final main Push `f129eb89` pending |

## Evidence receipts

- **PR #662**: [implementation and review](https://github.com/july-rust/BoardSI/pull/662), final PR HEAD `9340ecc60b8769728bf1eda4e2861943d1f77ab6` **56/56 GitHub checks SUCCESS**, **15/15 workflows SUCCESS**. Codex independently re-reviewed the exact HEAD with no new major issues; **0 unresolved review threads**.
- **Protected merge**: [Verified Squash `9e68bb595cace4a33190e61cd817b6b5b64e79ee`](https://github.com/july-rust/BoardSI/commit/9e68bb595cace4a33190e61cd817b6b5b64e79ee), recorded 2026-11-16 KST.
- **Four OS native and guard**: [F_004 compatibility CI](https://github.com/july-rust/BoardSI/actions/runs/43864065409) — F_006 actual worker golden round-trip + staged-source mutation no-clobber; all 4 OS PASS.
- **Four OS Qt**: [Qt runtime CI](https://github.com/july-rust/BoardSI/actions/runs/43864064892) — F_006 production Main Qt actual artwork/picking/layer filter/save/open tests PASS on Linux, Windows x64/ARM64, macOS ARM64.
- **Common regressions**: F_001–F_005 native v1 SQLite/BSIG, existing main Qt IPC, source continuity policies remain unchanged; PR CI verifies no regressions.
- **Test-only fix #1**: [PR #663](https://github.com/july-rust/BoardSI/pull/663) addressed actual Windows x64 [main F_004 compatibility #43867948114](https://github.com/july-rust/BoardSI/actions/runs/43867948114) `AlreadyExists` in F_006 test directory generation. Added atomic nonce, exclusive mkdir/retry, 24-thread collision test. PR **12/12 workflows, 49/49 checks SUCCESS**, independent Codex no major issues; [Verified Squash `9df9f6ed`](https://github.com/july-rust/BoardSI/commit/9df9f6edb6e25a07414a277e17e413323aeb3ba2).
- **Test-only fix #2**: [PR #664](https://github.com/july-rust/BoardSI/pull/664) addressed an inherited F_005 ODB++ fixture `AlreadyExists` on macOS ARM64 in [main F_004 compatibility #43876419343](https://github.com/july-rust/BoardSI/actions/runs/43876419343). Same guard plus separate 24-thread test. PR **12/12 workflows, 49/49 checks SUCCESS**, no unresolved Codex review findings; [Verified Squash `f129eb89`](https://github.com/july-rust/BoardSI/commit/f129eb89781a58fad8732efd3eafb7659b7b269c).
- **Exact final main Push**: `f129eb89781a58fad8732efd3eafb7659b7b269c` GitHub Actions **PENDING**. Do not mark T12 PASS or assign first COMPLETE date until actual run evidence is complete.
- **Scope boundary**: Both `AlreadyExists` findings occurred in test fixture path creation, not verified production parser/Writer/Reader failures. SQLite v1 DDL/BSIG and product runtime unchanged.
- Source E_026: [KiCad source fixture/qualification](e026-kicad-layer-via-source-qualification.md). Existing F_005: [CAD native roundtrip](f005-cad-bsi-roundtrip-qualification.md).
- **Not in F_006 acceptance scope**: full KiCad boards with unsupported source objects, independent physical KiCad GUI export/real PCB fixture, actual plating/contact topology, solver/RLGC/S-parameters, industrial U/T. These remain explicitly **NOT_VERIFIED**, never silently promoted to PASS.
