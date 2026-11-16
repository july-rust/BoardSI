# F_006 — KiCad 9 Source → Canonical → Native .bsi → Qt acceptance

> **State: IN PROGRESS / NOT YET ACCEPTED.** Do not infer a PASS from code changes alone. Actual CI, independent review, protected merge and post-merge checks must close this ledger.

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
| T01 | KiCad 9 version + malformed S-expression rejection | PENDING FINAL CI |
| T02 | 2/4/6/8/32-Cu accurate source order | PENDING FINAL CI |
| T03 | straight Track exact nm/width/layer/net mapping | PENDING FINAL CI |
| T04 | through Via nominal geometry, no physical contact invention | PENDING FINAL CI |
| T05 | exact Canonical → .bsi → Canonical semantic roundtrip | PENDING FINAL CI |
| T06 | v1 Golden and F_004 backward compatibility | PENDING FINAL CI |
| T07 | independent SQL/BSIG/Source geometry oracle | NOT_YET_VERIFIED |
| T08 | unsupported geometry/flags/malformed input fail-closed | PENDING FINAL CI |
| T09 | SHA source mutation/cancel/lock/ENOSPC preserve old native | PENDING FINAL CI |
| T10 | main Qt import/save/open/artwork picking/layer filtering | PENDING FINAL CI |
| T11 | native Rust/Qt Windows x64, ARM64, macOS ARM64, Linux | PENDING FINAL CI |
| T12 | existing F_001–F_005 + post-merge main regression | PENDING FINAL CI |

## Evidence receipts

- Implementation review candidate: [PR #662](https://github.com/july-rust/BoardSI/pull/662). **Draft until final HEAD CI/review and acceptance.**
- Prior accepted bounded subset evidence: [F_005](f005-cad-bsi-roundtrip-qualification.md), [E_026](e026-kicad-layer-via-source-qualification.md).
- **Exact HEAD check runs, independent reviewer findings, protected merge SHA, post-merge main workflow PASS: NOT YET RECORDED.**
- Evidence date of this ledger is the Git commit date; F_006 first COMPLETE date may only be added after all required gates actually pass.
