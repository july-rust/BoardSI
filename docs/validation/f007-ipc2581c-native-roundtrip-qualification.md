# F_007 — IPC-2581C Source Copper → Native `.bsi` → Main Qt

**Status (2026-11-28 KST): IMPLEMENTATION + CURRENT PROTECTED MAIN REQUALIFICATION PASS (supported scope). Original 2026-11-17 post-merge Push CI remains NOT_OBSERVED.**
The original F_007 implementation was Squash-merged as PR [#666](https://github.com/july-rust/BoardSI/pull/666) after **15/15 PR workflow successes** at exact HEAD `75572a6a509872083cb133499ec9d2bbad1ff146`.
Merged commit: [`c847132dd1078873c370edc75294ac4d8f51169f`](https://github.com/july-rust/BoardSI/commit/c847132dd1078873c370edc75294ac4d8f51169f).
`main` source file blob SHA matches the accepted final PR code; GitHub reported `merged=true`.

## Strict source-only contract

F_007 reuses E_025's bounded IPC Revision C Source inventory and F_003 Native SQLite **v1** Writer/Reader; it does **not** change v1 BSIG envelopes, schema, or old Golden fixtures. The opt-in adapter covers declared positive Copper LayerFeature/Set/Features/Line and straight Polyline, with inline ROUND/SOLID LineDesc, source Decimal → checked integer nm, bounded layer/net/PinRef metadata and source SHA-256. The required RevC Step Datum is accepted only at exact zero origin; missing, duplicate or nonzero Datum fails closed. Per-Net PinRef parent names are preserved even when source LogicalNets are not sorted. A Copper layer named `*` is rejected before save because Qt reserves that sentinel.

Unsupported fabrication Pad, Via, Drill, Arc, Polygon, Location, Xform, negative polarity, external LineDescRef, unrecognized CadData geometry, manufacturing physical evidence and out-of-range/sub-nm coordinates fail closed rather than silently disappear. Layer declaration order is **not** proven physical z; logical net names are **not** an electrical connection graph. Full IPC XSD conformance, vendor-qualified specimens, actual barrel/plating, SI readiness and electrical RLGC/S-parameter acceptance remain **outside this F_007 source-only contract**.

**Standard/schema background:** [IPC-2581 test cases](https://www.ipc2581.com/ipc-2581-revc-test-cases/) · [RevC XSD](https://docs.rs/crate/ipc2581/0.3.3/source/IPC-2581C.xsd).

## Executed evidence — final PR HEAD

| Gate | Result | Actual evidence |
| --- | --- | --- |
| T01 RevC safe bounded XML and mandatory Datum | **PASS (supported subset)** | `ipc2581c_source::f007_native_tests`; Rust 1.75 CI |
| T02 Layer, Net and original PinRef-parent linkage | **PASS** | native nonalphabetical Net Z/A round-trip regression |
| T03 exact straight Line / Polyline Copper | **PASS** | source fixture, Canonical traces, actual Qt artwork |
| T04 checked nm and MILLIMETER/MICRON/INCH | **PASS** | integer unit and unsupported geometry negative tests |
| T05 Native v1 semantics round-trip | **PASS** | `f007_ipc2581c_roundtrip` (3/3) |
| T06 frozen F_004 legacy v1 Goldens | **PASS** | [F004 4 OS](https://github.com/july-rust/BoardSI/actions/runs/43990895101) |
| T07 independent raw SQLite / BSIG / SHA and exact vertices | **PASS** | independent SQL + BSIG PointList byte parser regression |
| T08 rejected unsupported/CAD geometry retains last good `.bsi` | **PASS (tested subset)** | importer negative tests and native no-clobber test |
| T09 stale SHA guard, crash, kill, lock/ENOSPC resilience | **PASS (covered cases)** | [F003 resilience](https://github.com/july-rust/BoardSI/actions/runs/43990895133) and native tests |
| T10 real main Qt Save/Open/Artwork/picking/filters | **PASS** | [Qt actual native four-platform jobs](https://github.com/july-rust/BoardSI/actions/runs/43990894855) |
| T11 Win x64/ARM64, macOS ARM64, Linux x64 | **PASS** | F004, Qt runtime, [F003 native](https://github.com/july-rust/BoardSI/actions/runs/43990895034) |
| T12 prior F_001–F_006 and post-merge status | **CURRENT MAIN REQUALIFICATION PASS; original Push NOT_OBSERVED** | [15/15 PR checks](https://github.com/july-rust/BoardSI/pull/666/checks); dated follow-up below |

**Independent review receipt:** Codex reviewed PR #666 and opened P1 mandatory RevC Datum plus P2 net PinRef identity and Qt reserved `*` layer findings. All three received substantive source/tests corrections, replies and `is_resolved=true`; final exact-HEAD Codex re-review did not produce a separate review receipt, so no claim is made that it did.

**Historical evidence boundary (retained):** the original merged commit's exact Actions run and check-run queries returned zero; the verified PR suite and source match do not reconstruct a historical Push CI result. On 2026-11-28 the REST commit verification field for `c847132dd1078873c370edc75294ac4d8f51169f` was observed as `verified=true`, reason `valid`. No historical Push PASS is asserted.

## Dated current-code requalification — 2026-11-28 KST

The protected, cryptographically verified code-main commit [`560f0d47e4482b63b689cc2caf4c88d6a0c16c4a`](https://github.com/july-rust/BoardSI/commit/560f0d47e4482b63b689cc2caf4c88d6a0c16c4a) has **12/12 actual Push workflow successes**. This new qualification is not evidence for the historical SHA.

| Evidence | Actual result |
| --- | --- |
| [Native Rust Push](https://github.com/july-rust/BoardSI/actions/runs/46095270687) | F007 round-trip integration tests: 4 per OS, 16/16 PASS |
| [Main Qt Push](https://github.com/july-rust/BoardSI/actions/runs/46095270665) | Actual IPC-2581C Native Main Qt selftest 4/4 PASS, Linux offscreen |
| Protected code-main signature | verified=true, reason=valid |
| Original F007 code-main Push | NOT_OBSERVED; deliberately retained |

Rust job IDs: Linux 140825056012, macOS ARM64 140825056033, Windows ARM64 140825056084, Windows x64 140825056094. Qt job IDs: macOS ARM64 140825094745, Windows x64 140825094760, Windows ARM64 140825094815, Linux x64 140825094786. The four Native cases were nonalphabetical Net PinRef parent identity, reserved display-name valid native structural open, unsupported geometry/no clobber, and exact source save/open oracle with stale guard. The current functional contract is requalified without changing production code, schema, fixtures or the supported IPC subset. No claim of full vendor IPC, physical connectivity, or SI acceptance. The final protected document merge and its own actual Push CI are to be recorded in the document PR closure receipt.

## Reproduction (Rust 1.75 / actual Qt)

```bash
cargo +1.75.0 fmt --all -- --check
cargo +1.75.0 clippy --locked --workspace --all-targets -- -D warnings
cargo +1.75.0 test -p boardsi-import --locked f007_native_tests
cargo +1.75.0 test -p boardsi-native-file --locked --test f007_ipc2581c_roundtrip
cargo +1.75.0 test --locked --workspace
ctest --test-dir build-qt -R qt-f007-main-ipc-native-roundtrip --output-on-failure
```

Workflows: `.github/workflows/f004-compatibility.yml`, `.github/workflows/qt-runtime.yml`, plus the remaining PR gate matrix. Confidential PCB sources were not committed. Broader format/physical acceptance remains tracked under #23, #532 and #520.
