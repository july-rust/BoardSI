# D_050 — SOURCE-Exact Mixed-Pad Finite-Width Inter-Component Bridge Qualification

**First checked: 2026-11-07 KST. Evidence date: 2026-11-07 KST.** [Issue #493](https://github.com/july-rust/BoardSI/issues/493) · [Code PR #494](https://github.com/july-rust/BoardSI/pull/494) · [Canonical checklist](../development-checklist.md).

## 1. Scope and fixed SOURCE witness

The opt-in **`SOURCE_LOCAL_MIXED_PAD_FINITE_WIDTH_BRIDGE_ONLY`** diagnosis establishes strictly positive-area **same-layer local copper geometry contact candidates** between two previously disjoint D_049 source-owned groups. It is **not** proof of manufactured physical conduction, arbitrary copper topology, or global electrical Open/Short.

The public synthetic [PADS V9.4 MILS fixture](../../crates/boardsi-import/tests/fixtures/d050_mixed_pad_finite_width_bridge_v94.asc) uses 0.01 mil = 254 nm per SOURCE tick, Layer 1, and one explicit same-net `*SIGNAL* NET_A` group (supported NET_B-wide variant). The inherited D_049 inventory remains exactly four SMD Pads and six base/supplementary Traces: four D_040 straight centerline attachments and two independently qualified width-8-mil round/square SMD contact witnesses. The **seventh SOURCE route** is explicitly declared `U1.1 U2.1`, width 8 mil, with an 8-mil round-ended horizontal Trace **(226,300) → (378,300) mil**. The inherited circular-Pad supplemental SOURCE route also has pin pair `U1.1 U2.1`, so the full route identity must be determined through SOURCE records and complete geometric inventory, never by pin-pair equality alone.

| New contact candidate | Exact geometry | Strict positive-area qualification |
| :--- | :--- | :--- |
| Bridge → U1.1 supplemental round-Pad-contact Trace | Existing width-8-mil vertical Trace, (222,285)→(222,315) mil, x=222; bridge begins x=226 at y=300 | Centerline distance 4 mil < sum of half-widths 8 mil |
| Bridge → U2.1 round SMD Pad | U2.1 center (400,300) mil, diameter 40 mil, radius 20 mil; bridge ends (378,300) mil | Endpoint-center distance 22 mil lies strictly between Pad radius 20 mil and Pad radius + Trace half-width 24 mil |

The bridge must **not** directly touch U1.1 round Pad, the baseline rails, either square Pad, the other supplemental square-contact Trace, or any unlisted shape. In particular, the bridge starting point is 26 mil from U1.1 round-Pad center, exceeding its combined 24-mil Pad radius + Trace half-width. A Net label without positive-area SOURCE geometry must never create an edge.

The fresh source-bound graph has **11 Nodes (4 Pads + 7 Traces), 10 same-layer Candidate Edges (6 original + 2 D_049 Pad contact + 2 new bridge contacts), exactly 1 Local Component, 0 proven cross-layer electrical edges**. New canonical-order edges:

- `E|8|8|10|POSITIVE_FINITE_WIDTH_TRACE_TRACE_BRIDGE`
- `E|9|2|10|POSITIVE_FINITE_WIDTH_TRACE_ROUND_PAD_BRIDGE`

The SOURCE-local result contains an explicit `GLOBAL_ELECTRICAL_NOT_VERIFIED` marker. Rejection of unsupported or cross-net geometry is a **qualification failure**, not a proven global electrical short.

## 2. Rust and independent Native Qt implementations

[Bounded Rust SOURCE validator](../../crates/boardsi-import/src/d050_mixed_pad_width_bridge.rs), exposed by `boardsi-import-artwork <source.asc> --d050-mixed-pad-width-bridge-v1`, parses and binds the full seven-Trace inventory, Net, Layer, Width, signed/decimal coordinates and SHA-256. It identifies exactly one candidate additional SOURCE route whose exclusion independently requalifies the six inherited SOURCE Traces with [D_049](../../crates/boardsi-import/src/d049_mixed_pad_dual_width.rs). It checks the inherited two supplemental Trace positions exactly, matches each reduced receipt Trace to one original imported Trace, evaluates both new positive-area inequalities using integer squared distances, and constructs a fresh unified receipt; no independent receipts are concatenated blindly.

[Native Qt SOURCE oracle](../../apps/boardsi-qt-prototype/d050_mixed_pad_width_bridge_qt.h) uses full original bytes and the [actual PreviewImportPanel reImport runner](../../apps/boardsi-qt-prototype/d050_qt_runner.cpp). Its ImportedArtwork independently checks 4 Pads / 7 Traces, exact Pad shapes and coordinates, owner and Net/Layer assignments, explicitly declared routes, independent SOURCE geometry of both new contacts, SHA-256-bound original source, reconstructed edges, and deterministic full receipt parity. The CMake target/test is registered in the [existing Qt runtime CMakeLists](../../apps/boardsi-qt-prototype/CMakeLists.txt), retaining the existing Linux, Windows x64/ARM64 and macOS ARM64 workflow matrix.

[Explicit Rust regressions](../../crates/boardsi-import/tests/d050_mixed_pad_width_bridge.rs) include accepted reversal, NET_B, CRLF, tabs, supported PADS numeric aliases, and one SOURCE tick within positive contact boundaries. Hostile regressions include x=230-mil bridge-to-trace tangency and gap, x=376-mil bridge-to-Pad tangency and gap, bridge moved toward direct U1 Pad contact, wrong Layer/Width/Net/Owner/Pin Pair/Shape, missing/duplicate/additional SOURCE, unsupported decimal flag, and mixed `NET_A` / `NET_B` route declarations. Native Qt independently performs actual reImport of positive and hostile variants, rejects a forged edge/count receipt, and proves that the old SOURCE receipt becomes stale after a changed SOURCE is imported. Neither one good contact nor an unchanged Net string can yield a partial false PASS.

## 3. Exact code PR acceptance and historical failure handling

- **Final exact code PR HEAD** [`880f63b358de86f2f71d06e57e212711b0970c8f`](https://github.com/july-rust/BoardSI/commit/880f63b358de86f2f71d06e57e212711b0970c8f/checks): **11/11 pull_request workflows and 34/34 check runs SUCCESS**. Rustfmt/Clippy, Rust Debug and Release, native Windows x64/ARM64 and macOS ARM64 numerical tests, Linux Qt offscreen, and Windows x64/ARM64 plus macOS ARM64 Qt succeeded. Linux Native Qt **72/72 CTest PASS**, including `qt-preview-d050-mixed-pad-finite-width-bridge-source` with actual `D050_NATIVE_QT_MIXED_PAD_BRIDGE true GLOBAL_ELECTRICAL_NOT_VERIFIED`.
- **Independent Codex exact HEAD review:** `completed`, `has_any_findings=false`, `has_unresolved_findings=false`; **0 unresolved review threads**. Reviews of superseded heads are not final evidence.
- Early HEAD failures remain in their historical CI: Rustfmt formatting differences, Windows CRLF fixture causing inert mutation tests, and missing Qt precompiled dependency `c005_reference_risk.h` include; each was corrected and independently rerun on the final exact HEAD. No past failure is retroactively marked successful.
- Protected single-parent **Verified Squash Merge**, [code-main `f1261fd7b8c8c1292ab1b25d6e54d590eadc9192`](https://github.com/july-rust/BoardSI/commit/f1261fd7b8c8c1292ab1b25d6e54d590eadc9192), sole parent `60facdc4015fd57a6e4b9eef58fbb9e6fe284b7f`, GitHub REST `verification.verified=true`, `reason=valid`, `main` ref equal.
- Separate **exact code-main event=push** verification of the merged SHA: **8/8 workflows SUCCESS, 28/28 checks SUCCESS**, including Native Qt and numerical CI. No PR-check inference was used as a substitute for Push evidence.

## 4. Checklist, documentation and explicit nonacceptance

Canonical Stage D bounded checked records **70 → 71**, with 2 pending unchanged. Overall bounded checked records **141 → 142**, with 122 pending unchanged. These are limited evidence counts and **not the fraction of complete BoardSI product development**.

The separate documentation-only PR changes **exactly two Markdown paths**, `docs/development-checklist.md` and this ledger. The existing docs classifier must satisfy the required checks **without allocating native build/test platform runners**; the exact docs PR HEAD checks, independent review, protected Verified Squash and exact docs-main event=push will be recorded in the terminal [Issue #493](https://github.com/july-rust/BoardSI/issues/493) evidence comment **after** completion. Do not create a recursive documentation PR merely to re-record those later results.

**NOT_VERIFIED:** manufactured physical copper continuity, global electrical Open/Short, arbitrary PCB or PADS geometry, plated Via, Plane/Pour/Thermal, P009/PH-08 return path, physical Scenario/Cache, RLGC or S-parameter solver, formal U01–U48/T01–T54 acceptance, and product release. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) **remains OPEN**. D_050 Issue #493 closes only after actual final code/docs/main-push evidence has been recorded.
