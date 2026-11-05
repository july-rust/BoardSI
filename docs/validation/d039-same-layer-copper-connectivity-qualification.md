# D_039 — PADS Same-Layer Copper Connectivity Foundation & Fail-Closed Qualification

**Status: BOUNDED CODE AND EXACT CODE-MAIN PUSH ACCEPTED. DOCUMENTATION-ONLY CLOSEOUT REQUIRES SEPARATE GATES.**

First checked: **2026-11-05 KST**. Owner: [Issue #460](https://github.com/july-rust/BoardSI/issues/460). Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains **OPEN**.

## Strictly bounded source-owned geometry

The D_039 implementation uses a **single redistributable synthetic PADS PowerPCB V9.4 MILS SOURCE**, in which one tick is exactly **254 nm**. It represents **two distinct conductive top SMD Pads and two positive-width straight Traces** on Net NET_A / Layer 1. Both traces meet at one exact collinear interior junction, while each outer endpoint lies strictly inside a corresponding conductive Pad footprint.

- Fixture: [d039_local_copper_v94.asc](../../crates/boardsi-import/tests/fixtures/d039_local_copper_v94.asc).
- Rust geometry: [d039_local_copper.rs](../../crates/boardsi-import/src/d039_local_copper.rs). Explicit opt-in CLI mode: --d039-local-copper-v1.
- Rust positive and adversarial tests: [d039_local_copper.rs](../../crates/boardsi-import/tests/d039_local_copper.rs).
- Independent Qt actual Preview/ImportedArtwork re-import: [d039_local_copper_qt.h](../../apps/boardsi-qt-prototype/d039_local_copper_qt.h) and [d039_qt_runner.cpp](../../apps/boardsi-qt-prototype/d039_qt_runner.cpp).

The SHA-256 SOURCE-bound wire reports **4 source geometry nodes, 3 local Copper Geometry edges and 1 local component**. The only positive label is LOCAL_COPPER_GEOMETRY_VERIFIED, together with explicit GLOBAL_ELECTRICAL_NOT_VERIFIED, SI_NOT_RUN and P009_UNSUPPORTED. This is a **local geometry** receipt, not manufactured net continuity or global electrical Open/Short proof.

Both Rust and Qt validate exact Source/Owner/Net/Layer/pad geometry, positive trace width, strict Pad interior, endpoint coincidence, bounded inventory and SHA. Only the four supported PADS V9.4 MILS header variants (base, CP1252, 250L, 250L-CP1252) qualify. Source trace-record reversal, ASCII tab or blank rows and supported lexical source variations preserve geometry, but every byte difference changes the source digest. Repeated execution is deterministic.

## Fail-closed and native independent qualification

Positive: exact 2 Pad / 2 Trace graph, source-direction reversal, all four supported headers, tab/blank-row reimports, clean local overlay and its reimport invalidation.

Negative: one Source Tick gap; wrong endpoint, Net, Layer, dimension, Trace width or Pad ownership; duplicate Decal/Part definitions; missing, duplicate or unsupported Source; forged digest, graph edge, receipt or global Electrical PASS; stale source after reimport. Unsupported Via, Pour, Thermal, Cutout or Arc topologies are not granted a positive witness. Qt reconstructs complete serialized output from actual accepted imported geometry rather than trusting Rust alone.

The local overlay reuses an already supported SOURCE-only candidate marker; no new globally certified electrical result, physical solver, or GUI assurance is implied.

## Code PR — ACCEPTED

[Code PR #461](https://github.com/july-rust/BoardSI/pull/461), exact final PR HEAD [b8c9525ccbeb993525089a06ed9932849ba1a283](https://github.com/july-rust/BoardSI/commit/b8c9525ccbeb993525089a06ed9932849ba1a283/checks):

- **34/34 exact HEAD PR Check Runs SUCCESS**, **11/11 actual pull_request Actions Workflows SUCCESS**.
- [Numerical CI #41993386759](https://github.com/july-rust/BoardSI/actions/runs/41993386759): D_039 Rust Debug **5/5**, Release **5/5** PASS; rustfmt and Clippy PASS; native Rust CI maintained.
- [Qt CI #41993386772](https://github.com/july-rust/BoardSI/actions/runs/41993386772): Linux Qt offscreen **61/61 CTest PASS**, including D039 source local copper, forged/one-tick/stale checks. Actual native D039 Qt Windows x64, Windows ARM64 and macOS ARM64 steps SUCCESS.
- Independent same-final-HEAD Codex: completed, no findings, no unresolved findings; review threads unresolved = **0**.
- Historical failure evidence deliberately retained: early Rustfmt and Qt build/runtime failures and Codex P2 findings about duplicated part definitions, absent native execution, blank source rows, supported header mismatch and ASCII-tab tokenization were corrected on subsequent commits. Earlier HEAD results were never used as final-HEAD success.

Protected GitHub Verified single-parent Squash code-main [ad1f4f1992adb41b0ce47a54278ef8052b1b5909](https://github.com/july-rust/BoardSI/commit/ad1f4f1992adb41b0ce47a54278ef8052b1b5909) has sole parent 69db0a263337f8e05c4be2df99f1120bb86badb1, REST verification.verified=true/reason=valid, and main ref was confirmed equal to that SHA.

## Exact code-main Push — ACCEPTED

At exact **main@ad1f4f1992adb41b0ce47a54278ef8052b1b5909**, **8/8 actual event=push GitHub Actions Workflows SUCCESS and 28/28 Check Runs SUCCESS**. No failed, skipped, cancelled or pending status is counted as success.

- [Qt main Push #41995229580](https://github.com/july-rust/BoardSI/actions/runs/41995229580).
- [Numerical main Push #41995229505](https://github.com/july-rust/BoardSI/actions/runs/41995229505).

## Documentation-only PR — SEPARATE CLOSEOUT GATE

Only this ledger and [canonical checklist](../development-checklist.md) are modified; no code, CMake, workflow, CI runners or branch-protection changes. Stage D evidence goes from **59 checked / 2 pending to 60 checked / 2 pending**, and total from **130 checked / 122 pending to 131 checked / 122 pending**. These are strictly bounded checked-evidence counts, not product completion percentages.

The existing docs-only classifier must prevent allocation of native compilation runners for docs-only changes, with required contexts preserved and SUCCESS distinguished from SKIPPED. Independently verify exact latest docs PR HEAD CI, completed Codex with zero findings/threads, protected Verified one-parent Squash and independent exact docs-main Push. Record immutable final docs PR SHA, CI, independent review, docs-main verification and Push results in the **final Issue #460 comment** and close Issue only after the gates succeed. Do not generate a recursive documentation revision.

## Physical scope remains NOT_VERIFIED

PADS complete imported model, real manufactured conductive continuity, Via plating barrel, Copper Pour/Thermal/Reference, global Open/Short, P009/PH-08 Return Path, physical Scenario/Cache, RLGC/S-parameter solver (DC–30 GHz), formal U01–U48/T01–T54 acceptance and release are **NOT_VERIFIED**. Parent Issue #12 stays OPEN, legacy [PR #35](https://github.com/july-rust/BoardSI/pull/35) remains CLOSED/UNMERGED.
