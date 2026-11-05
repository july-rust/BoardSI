# D_042 — Same-Layer SOURCE Closed-Loop / Redundant Path Qualification

**Status:** limited source-geometry code and code-main Push accepted **2026-11-06 KST**. Documentation PR CI/review/merge and docs-main Push acceptance must be recorded in [Issue #469](https://github.com/july-rust/BoardSI/issues/469); do not make recursive docs PRs. The parent physical-connectivity [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains **OPEN**.

## 1. Exact bounded SOURCE and scope

Public synthetic fixture [`d042_local_closed_loop_v94.asc`](../../crates/boardsi-import/tests/fixtures/d042_local_closed_loop_v94.asc): PADS PowerPCB **V9.4 MILS**, 254 nm per Canonical Source Tick, Layer 1, NET_A with supported NET_B lexical variant.

The accepted [D_041 direct SOURCE bridge](d041-same-layer-copper-bridge-qualification.md) spans **J0=(200,350) mil** to **J1=(400,350) mil**. Exactly five additional source-owned positive-width straight detour Traces connect J0 to J1 by:

`(200,350) → (150,350) → (150,250) → (450,250) → (450,350) → (400,350)` mil.

The original 4 SMD Pads and 5 Traces remain. The added 5 detour Traces produce **4 Pad + 10 Trace = 14 local SOURCE geometry Nodes**, **14 local candidate Edges**, **1 local connected Component**, **1 independent local Cycle**. A graph traversal and `E − V + C` confirm the **bounded candidate topology**. The original 8 D_041 edges remain, with 6 detour attachment/continuation edges; a Net label alone never introduces an edge.

This feature is **fixture-specific and exact**: it does **not** qualify arbitrary imported real PCB closed-loop topologies, other layers, curved Copper, global network connectivity, or physical electrical redundancy. A Source-local cycle is not evidence of manufacturer copper continuity, nor a physical Short test.

## 2. Implementation and protocol

- Rust [`d042_local_closed_loop.rs`](../../crates/boardsi-import/src/d042_local_closed_loop.rs) reuses D_041 source provenance only after removing the **five specifically declared** detour records, reparses all 10 Source traces, and checks the complete supported inventory, exact ends, positive width, matching Layer/Net/owner, duplicate/missing segments, public fixture geometry, and graph cycle rank.
- An explicit CLI `boardsi-import-artwork --d042-local-closed-loop-v1` produces SHA-256-bound deterministic `BOARDSI_D042_LOCAL_CLOSED_LOOP_V1` diagnostic receipt; it is **opt-in**, not the normal Canonical Solver path.
- Native Qt [`d042_local_closed_loop_qt.h`](../../apps/boardsi-qt-prototype/d042_local_closed_loop_qt.h) reads fresh `ImportedArtwork` and independently rebinds all **10 SOURCE Pin-Pair declarations to exact source-coordinate traces** before reconstructing nodes, edges, component and cycle. The Qt decoder requires exact bytewise receipt equivalence with the source SHA. Qt does **not** accept Rust stdout blindly.
- Successful result explicitly retains `P009_UNSUPPORTED`, `SI_NOT_RUN` and `GLOBAL_ELECTRICAL_NOT_VERIFIED`; its local label `LOCAL_COPPER_CLOSED_LOOP_VERIFIED` refers **only** to the declared synthetic SOURCE geometry.

## 3. Reproducible tests and fail-closed

- Positive: one supported closed loop and two distinct local paths; reproducible deterministic receipt; reversed detour record orientation; NET_A/NET_B; V9.4 supported header variants, tab-delimited Pin-Pair and `*SIGNAL*`, CRLF; `E−V+C=1`. Rust source-specific tests **3/3 Debug PASS**, **3/3 Release PASS**.
- Negative: single Source Tick gap, wrong Net/Layer/width/owner/Pin-Pair, missing or duplicate Trace, displaced Junction/Part, extra/unsupported Section, Via/Area/Arc, stale original Source/SHA, forged edges/cycle/receipt, and header-only Pin-Pair ownership swap. Both Rust and Qt fail closed on unsupported evidence; geometry rejection must **not** be reinterpreted as an electrical Short.
- Native Qt dedicated [runner](../../apps/boardsi-qt-prototype/d042_qt_runner.cpp) registered in existing CMake/CTest/Native CI. Existing Linux offscreen **64/64 CTest PASS**, including D_042 and previous D_039–D_041; actual existing Native Qt Windows x64, Windows ARM64, macOS ARM64 workflow jobs **SUCCESS**. No additional workflow/runner/Ruleset or expanded platform matrix.

## 4. Verified CODE PR and independent acceptance

- [Code PR #470](https://github.com/july-rust/BoardSI/pull/470) **final exact HEAD** [`9cd97d4bc24deaf0c42a30e5c1a6d3999f1f6365`](https://github.com/july-rust/BoardSI/commit/9cd97d4bc24deaf0c42a30e5c1a6d3999f1f6365/checks): **34/34** pull_request Check Runs and **11/11** workflows SUCCESS; Rustfmt, Clippy with warnings denied, Debug/Release, and native Linux/Windows x64/Windows ARM64/macOS ARM64 all passed.
- Independent same-final-HEAD Codex Review **completed**, `has_any_findings=false`, `has_unresolved_findings=false`; **0 unresolved inline threads**. The initial Rustfmt failure, next Clippy `identity-op` failure, and two historical Codex P2 findings remain **historical failures**: (1) wrong Source loaded before forged-cycle test, (2) native Qt accepted swapped Pin-Pair declarations independent of geometry. Both defects were fixed and verified again on the new final HEAD.
- Protected GitHub **Verified one-parent Squash** commit [`932e5bcd872fab20393b01cd46ace71a0c685633`](https://github.com/july-rust/BoardSI/commit/932e5bcd872fab20393b01cd46ace71a0c685633), sole parent `51a36f173a3ad785151df30c9635997445dc1eaa`, GitHub REST `verification.verified=true/reason=valid`; exact `main` ref equality verified.
- Code-main exact commit actual `event=push` CI: **8/8 Actions Workflows SUCCESS · 28/28 Check Runs SUCCESS**, none failed/pending/skipped. [Qt Push #42130513630](https://github.com/july-rust/BoardSI/actions/runs/42130513630) · [Numerical Push #42130513899](https://github.com/july-rust/BoardSI/actions/runs/42130513899).

## 5. Documentation closeout and explicit exclusions

Only this file and [`docs/development-checklist.md`](../development-checklist.md) change in the separate **Markdown-only docs PR**. Stage D bounded checked **62 → 63**, total checked **133 → 134**, pending unchanged (**Stage D 2 / total 122**). Existing docs-only classifier must SKIP native compilations **before allocating runners**, preserving all **17 required status contexts** and distinguishing SUCCESS from SKIPPED. Accept the exact final docs HEAD only after independent review and passing required CI; use protected Verified one-parent Squash and verify exact docs-main `event=push`. Record final immutable evidence and close Issue #469 in the Issue comment, **not another docs PR**.

**NOT_VERIFIED**: manufactured Copper/Plated Via barrel continuity, whole-board electrical Open/Short, general multilayer Pad/Via/Plane/Pour/Thermal connectivity, P009/PH-08 actual return path, Physical Scenario/Cache/Reference, RLGC and S-parameter simulation/DC–30 GHz product correctness, formal U01–U48/T01–T54 product U/T acceptance, and final release. The additional single bounded checked item is **not** a product-completion percentage. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains **OPEN**.
