# D_049 — SOURCE-Exact Mixed Round/Square SMD Pad Dual Finite-Width Contact Qualification

**First checked: 2026-11-07 KST. Evidence date: 2026-11-07 KST.** [Issue #490](https://github.com/july-rust/BoardSI/issues/490) · [Code PR #491](https://github.com/july-rust/BoardSI/pull/491) · [Canonical checklist](../development-checklist.md).

## 1. Qualification boundary and SOURCE inventory

This is an opt-in **`SOURCE_LOCAL_MIXED_PAD_DUAL_FINITE_WIDTH_ONLY`** diagnostic of two simultaneous strictly positive-area same-layer copper **contact candidates**. It does not prove manufactured electrical continuity or arbitrary physical PCB topology. The parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.

[Public synthetic PADS V9.4 MILS fixture](../../crates/boardsi-import/tests/fixtures/d049_mixed_round_square_dual_width_v94.asc), 0.01 mil = 254 nm per SOURCE tick, NET_A/NET_B, Layer 1. The complete bounded D_040 four-Pad/four-10-mil-width straight Trace baseline (two disjoint components) is preserved. Two additional width 8 mil, round-capped straight Traces are sourced in one board:

| Contact | Pad center | Conductive Pad | SOURCE Trace start → end | Positive-area geometry |
| :--- | :--- | :--- | :--- | :--- |
| U1.1 round SMD Pad | (200, 300) mil | Diameter 40 mil, radius 20 mil | (222,285)→(222,315) mil, pin pair U1.1/U2.1 | Centerline 22 mil from Pad center; 20 < 22 < 20+4 mil |
| U1.2 square SMD Pad | (200, 400) mil | 30×30 mil square, right side x=215 mil | (217,390)→(217,410) mil, pin pair U1.2/U2.2 | Centerline 2 mil from Pad right side; 2 < 4 mil half-width |

These simultaneous witnesses do **not** touch the baseline rails, remote U2 component or each other. Source-bound graph: **10 Nodes (4 Pad + 6 Trace), 8 same-layer candidate edges (6 baseline + 2 additional), 2 local components, 0 proven electrical cross-layer edges**. Added canonical-order edges:

- `E|6|0|8|POSITIVE_FINITE_WIDTH_TRACE_ROUND_PAD_CONTACT`
- `E|7|1|9|POSITIVE_FINITE_WIDTH_TRACE_PAD_CONTACT`

The boundary equality (zero-area tangency) and SOURCE unsupported geometry fail closed rather than being classified as physical connections.

## 2. Rust and independent Native Qt implementation

[Rust D_049 module](../../crates/boardsi-import/src/d049_mixed_pad_dual_width.rs) removes each unique additional SOURCE pin-pair witness into a separately validated reduced fixture, invokes bounded [D_047 square](../../crates/boardsi-import/src/d047_trace_square_pad_width.rs), [D_048 round](../../crates/boardsi-import/src/d048_trace_round_pad_width.rs) and [D_040 two-component baseline](../../crates/boardsi-import/src/d040_multi_component.rs) validations, then reconstructs **one fresh** combined local graph from the complete six imported Traces. It does not concatenate legacy receipts. Full inventory, both independent Pad shapes, NET/Layer/Owner/Pin-Pair, fixed point/width geometry, one-to-one Trace identity and SHA-256 source binding are required.

[Independent Native Qt SOURCE oracle](../../apps/boardsi-qt-prototype/d049_mixed_pad_dual_width_qt.h) uses [actual PreviewImportPanel reImport](../../apps/boardsi-qt-prototype/d049_qt_runner.cpp) and ImportedArtwork geometry. It independently checks four Pad shapes/centers/sizes, four baseline traces, two supplemental finite-width Trace-to-Pad candidates, supported SOURCE numeric-field semantics, declared pins and full exact-source SHA-256, and rebuilds its own candidate edges. It does not regard the Rust-produced wire as proof: exact independently rebuilt Receipt parity is required.

Supported PADS numeric aliases include optional leading `+`, zero-prefix decimal, 0–2 decimal precision (e.g. `8.00` mil = 800 ticks), supported straight-route flags and second-vertex terminal layer 65 inheriting the prior layer. Unsupported precision, wrong first-vertex layer, unsupported flags/shape/net/owner, extra/missing/duplicated route records, stale hash and forged Receipt fail closed.

[Rust integration tests](../../crates/boardsi-import/tests/d049_mixed_pad_dual_width.rs) exercise strict positive contact, cap/direction, NET_B/CRLF/tabs/header/numeric aliases and hostile alternatives. Native Qt runner actually loads input mutants through PreviewImportPanel before comparing its own reconstruction. Invalidity of either of the two contacts invalidates **the entire D_049 Receipt**; an individual good contact never yields a partial false PASS.

## 3. Exact code PR validation and historical failures

- [Final code PR HEAD `20f668f27dc2b56f45b0e020f06e2645dca1b4e1`](https://github.com/july-rust/BoardSI/commit/20f668f27dc2b56f45b0e020f06e2645dca1b4e1/checks): **11/11 pull_request workflows SUCCESS**, Rustfmt/Clippy, Linux Numerical and Rust Debug/Release regressions, Native Windows x64 / Windows ARM64 / macOS ARM64 numerical tests successful. Linux Native Qt **71/71 CTest PASS**, including `qt-preview-d049-mixed-pad-dual-finite-width-source` with `D049_NATIVE_QT_TRACE_PAD_WIDTH true`. Qt Native Windows x64 / Windows ARM64 / macOS ARM64 jobs SUCCESS.
- **Independent exact HEAD Codex:** completed with `has_any_findings=false`, `has_unresolved_findings=false` after a prior P2 finding; review threads resolved = all, unresolved = 0.
- Historical failed heads remain historical: initial Rustfmt differences, Qt header include filename mismatch, independent Codex P2 Rust/Qt SOURCE numeric-field alias inconsistency, and Windows Fixture-mutant test strings matched no records under CRLF. All were corrected and newly verified. A separate preexisting intermittent Windows D_006 temporary-file test failure was observed on an older HEAD and did not recur in final HEAD's successful Windows x64 matrix. Do not present past failures as passing retroactively.
- Protected GitHub single-parent **Verified Squash** [code-main `8c69df61b605fe10b464fd87bedff81348d2b096`](https://github.com/july-rust/BoardSI/commit/8c69df61b605fe10b464fd87bedff81348d2b096), sole parent `2e70337842471b2c81759ee7696994abf442eaf0`, GitHub REST `verification.verified=true` / `reason=valid`, main ref matches.
- **Independent exact code-main `event=push` workflows/checks** must be confirmed and recorded in the final [Issue #490](https://github.com/july-rust/BoardSI/issues/490) comment. Do not infer a push PASS from PR CI. The documentation PR must separately prove docs-only classification, exact docs PR HEAD checks, Codex, Verified Squash and exact docs-main event=push; do not generate another recursive docs PR.

## 4. Checklist and explicit nonacceptance

Canonical Stage D evidence counts move **69→70 checked**, 2 pending unchanged. Overall limited evidence count moves **140→141 checked**, 122 pending unchanged. These are bounded validation records, **not a percentage of product completion**.

**NOT_VERIFIED:** manufactured physical copper continuity, global Electrical Open/Short, arbitrary PCB/Plane/Via/Pour/Thermal, P009·PH-08 Return Path, physical Scenario/Cache, RLGC and S-parameter solver, formal U01–U48/T01–T54 acceptance, or release. **The parent Issue #12 remains OPEN**. Close D_049 Issue #490 only after all actually observed code/docs/main-push evidence has been posted without inventing any success state.
