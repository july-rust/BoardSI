# D_051 — SOURCE-Exact Dual Finite-Width Mixed-Pad Bridge Loop and Virtual Cut-Set

**First checked: 2026-11-07 KST. Evidence date: 2026-11-07 KST.**
[Issue #497](https://github.com/july-rust/BoardSI/issues/497) · [Code PR #498](https://github.com/july-rust/BoardSI/pull/498) · [Canonical checklist](../development-checklist.md).

## 1. Bounded SOURCE witness and exact copper geometry

This opt-in `SOURCE_LOCAL_DUAL_FINITE_WIDTH_REDUNDANCY_ONLY` diagnosis qualifies **same-layer synthetic SOURCE-local copper geometry contacts** and virtual graph removal, NOT manufactured physical continuity, arbitrary PCB electrical Open/Short, or global SI.

The synthetic [PADS V9.4 MILS fixture](../../crates/boardsi-import/tests/fixtures/d051_mixed_pad_dual_finite_width_loop_v94.asc) uses **254 nm per 0.01-mil SOURCE tick**, Layer 1, exactly one supported NET_A SOURCE group (and NET_B full-net variant). It preserves the complete D_050 fixture: four SMD Pads, seven Traces and first width-8-mil bridge U1.1/U2.1 **(226,300)→(378,300) mil**. It adds one independent width-8-mil U1.2/U2.2 SOURCE bridge **(222,400)→(382,400) mil**.

Two *additional* strictly positive-area contacts are required:

| Contact | Bounded integer geometry witness | Positive criterion |
| :--- | :--- | :--- |
| Second bridge ↔ inherited square-pad supplemental Trace | x=217 mil, y=390..410 mil, 8-mil-wide vertical Trace; new bridge left x=222 mil, y=400 mil, width 8 mil | Minimum centerline distance 5 mil < combined halfwidth 8 mil |
| Second bridge ↔ U2.2 square SMD Pad | Pad center (400,400) mil, side 30 mil, left edge x=385 mil; new bridge right endpoint x=382 mil | Horizontal distance 3 mil < round-cap radius 4 mil |

The left bridge endpoint must remain strictly to the right of U1.2 Pad direct-contact bound (x>219 mil), and the right endpoint strictly outside the U2.2 rectangle (x<385 mil). Boundary tangency at x=225 mil on the left or x=381 mil on the right is **zero-area** and rejected; a 1-tick inboard variant passes only when all other SOURCE records remain valid. No extra Pad, Trace or off-net contact is accepted.

## 2. Graph and single-bridge virtual removal

The Rust and Native Qt implementations each reconstruct an original-source-bound graph rather than trusting shared Net labels or merely concatenating previous receipts.

| Virtual scenario | Nodes | Same-layer candidate edges | Components | Cycle rank | U1↔U2 reachability |
| :--- | ---: | ---: | ---: | ---: | :--- |
| No removal | 12 | 12 | 1 | 1 | Reachable |
| First bridge Trace (node 10) + incident edges removed | 11 | 10 | 1 | 0 | Reachable |
| Second bridge Trace (node 11) + incident edges removed | 11 | 10 | 1 | 0 | Reachable |
| Both bridge Traces + incident edges removed | 10 | 8 | 2 | 0 | Separated |

The cycle rank is computed from **E − N + C**, not inferred from matching Net names. All edges are same-layer SOURCE-local candidates; **0 cross-layer electrical edges proven**. Virtual node removal is not a manufactured open-circuit test.

The full deterministic output is the `BOARDSI_D051_DUAL_FINITE_WIDTH_LOOP_V1` receipt: actual-source SHA-256, four Pad and eight Trace records, twelve candidate Edge records, four independently recomputed diagnostic records and explicit `GLOBAL_ELECTRICAL_NOT_VERIFIED` / `SI_NOT_RUN` limitations.

## 3. Rust and Native Qt evidence

- [Rust SOURCE validator](../../crates/boardsi-import/src/d051_dual_bridge_loop.rs): parses exact 8-Trace SOURCE, uniquely identifies U1.2/U2.2 extra route whose removal independently requalifies D_050, proves all original seven Traces one-to-one, applies integer-tick positive-area geometry and independent DSU cut-set diagnostics. CLI: `boardsi-import-artwork <source.asc> --d051-dual-finite-width-loop-v1`.
- [Rust hostile/positive tests](../../crates/boardsi-import/tests/d051_dual_bridge_loop.rs): source reversal, full NET_B, CRLF, tabs, numeric aliases and 1-tick strict contact positives; wrong Net/Layer/Owner/Pad shape/Width, missing/duplicate/extra routes, tangency, gaps, wrong geometry and forged SOURCE negatives.
- [Independent Qt SOURCE oracle](../../apps/boardsi-qt-prototype/d051_dual_bridge_loop_qt.h) with [actual Native Qt runner](../../apps/boardsi-qt-prototype/d051_qt_runner.cpp): separate full SOURCE and reduced D_050 `PreviewImportPanel` reImports, original ImportedArtwork identity/geometry checks, complete receipt parity, graph reconstruction and forged/stale receipt rejection. Explicit timeout-safe callback removal, importer cancellation/drain on timeout and Fail-Closed result.
- [Qt test registration](../../apps/boardsi-qt-prototype/CMakeLists.txt) in the existing Linux, Windows x64/ARM64 and macOS ARM64 workflows.

The first code HEAD contained an automatic Rustfmt mismatch and Windows CRLF-dependent inert mutations; those failures were corrected and retested on the exact final HEAD. Codex found a Qt runner callback-lifetime timeout risk on an earlier HEAD; corrected in the final HEAD and its review thread resolved. Past failures remain historical failures, not retroactive passes.

## 4. Code acceptance (exact verified evidence)

- [Code PR #498](https://github.com/july-rust/BoardSI/pull/498), exact final PR HEAD [`c99415758e1a8e593eb384f81a64644074628047`](https://github.com/july-rust/BoardSI/commit/c99415758e1a8e593eb384f81a64644074628047/checks): **34/34 check runs SUCCESS, 11/11 event=pull_request workflows SUCCESS**. Rustfmt/Clippy, Debug/Release, Native Rust Windows x64/ARM64 and macOS ARM64, Linux Qt, Windows x64/ARM64 and macOS ARM64 Qt and existing regression matrix passed. Linux Qt **73/73 CTest PASS**; D_051 runner logs `D051_NATIVE_QT_DUAL_BRIDGE_LOOP true GLOBAL_ELECTRICAL_NOT_VERIFIED`.
- Exact final-HEAD independent Codex code review: `completed`, `has_any_findings=false`, `has_unresolved_findings=false`; **0 unresolved review threads**.
- Protected one-parent **Verified Squash Merge**: [code-main `28a6f2b43e49a06e4eff0da1d1e41a46eb0d8b92`](https://github.com/july-rust/BoardSI/commit/28a6f2b43e49a06e4eff0da1d1e41a46eb0d8b92), sole parent `7300bc3a759d41f858996a163302ab42e3061e9a`, GitHub `verification.verified=true` / `reason=valid`, exact main ref matched immediately after merge.
- **Separate code-main event=push status must be verified independently**, and the final results recorded in [Issue #497](https://github.com/july-rust/BoardSI/issues/497). Do not substitute PR checks for push checks or treat pending work as success.

## 5. Documentation and exclusions

The separate documentation-only PR must modify **only** this evidence ledger and `docs/development-checklist.md`, preserve the optimized docs classifier (no native build runners), receive independent exact HEAD review, pass required checks, and merge with a protected Verified single-parent squash. Its exact docs-main Push checks and final code-main Push checks belong in the terminal Issue evidence comment; avoid recursive documentation PRs.

After all gates are observed, Stage D bounded checked records rise **71→72** (pending 2 unchanged) and overall bounded checked records **142→143** (pending 122 unchanged). These are **limited evidence counts**, not an estimate of BoardSI product completion.

**NOT_VERIFIED:** manufactured Copper continuity; global electrical Open/Short; arbitrary PCB/PADS geometry; Via/Plane/Pour/Thermal; P009/PH-08; physical Scenario/Cache; RLGC/S-parameter solver; formal U01–U48/T01–T54 acceptance; product release. A rejected cross-net SOURCE condition is not a proven manufactured electrical short. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains **OPEN**.
