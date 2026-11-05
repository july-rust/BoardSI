# D_040 — PADS Same-Layer Multi-Component Copper Isolation & Net Separation Qualification

**Status: BOUNDED CODE + EXACT CODE-MAIN PUSH ACCEPTED; DOCS-ONLY FINAL GATE RECORDED IN ISSUE #463.**

First checked: **2026-11-05 KST**. [Issue #463](https://github.com/july-rust/BoardSI/issues/463). Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains **OPEN**. The limited result is local SOURCE geometry; there is no global Electrical, Return Path, physical or product acceptance.

## Scope — source-owned bounded multi-component graph

- Synthetic redistributable PADS PowerPCB V9.4 MILS Source: [d040_multi_component_v94.asc](../../crates/boardsi-import/tests/fixtures/d040_multi_component_v94.asc); exact **254 nm per Source Tick**.
- Source supports exactly two U1/U2 instances of the bounded D2/TYPEA SMD footprint, four individually owned top Pads, four positive-width straight Layer-1 Traces and two spatially disjoint Pad–Trace–Trace–Pad paths.
- Each path has two independently identified Pad interior contact points and one shared exact junction, using SOURCE coordinates; Net labels do not create geometric edges.
- Independent U1 and U2 groupings yield **8 geometry nodes / 6 local copper edges / 2 connected components**, each component **4 nodes / 3 edges**, with **0 proven global electrical/cross-layer edges**.
- **Same-Net** NET_A/NET_A and NET_B/NET_B are still **two disconnected local components**; **different-Net** NET_A/NET_B is still **two isolated local components**. Separated groups must not be merged through matching Net names. This does not prove a global electrical Open.
- Conservative cross-group pad/trace bounding-box envelope overlap is **rejected** as unsupported, including cross-Net contact. An envelope collision is not an exact Copper short certification.

### Implementation and provenance

| Evidence | Path / interpretation |
| :--- | :--- |
| Rust diagnostic | [d040_multi_component.rs](../../crates/boardsi-import/src/d040_multi_component.rs), opt-in **`--d040-multi-component-v1`** on `boardsi-import-artwork` |
| Rust adversarial regression | [d040_multi_component.rs](../../crates/boardsi-import/tests/d040_multi_component.rs) |
| Qt independently imported artwork | [d040_multi_component_qt.h](../../apps/boardsi-qt-prototype/d040_multi_component_qt.h) and [d040_qt_runner.cpp](../../apps/boardsi-qt-prototype/d040_qt_runner.cpp) |
| Existing CI route | [Qt runtime](../../.github/workflows/qt-runtime.yml); existing native Windows x64/ARM64, macOS ARM64 and Linux offscreen matrix, no added workflow |
| Serialized receipt | `BOARDSI_D040_MULTI_COMPONENT_V1`, exact original SOURCE SHA-256, Component C / Pad P / Trace T / Edge E / Count/End records; `LOCAL_COPPER_COMPONENT_ISOLATION_VERIFIED`, `P009_UNSUPPORTED`, `SI_NOT_RUN`, `GLOBAL_ELECTRICAL_NOT_VERIFIED` |

The Rust diagnostic reuses the existing D_001 pad endpoint SOURCE/Canonical audit and existing part/route preview parsers; there is no new global electrical graph or proof of manufactured continuity. Qt does not accept Rust stdout merely because the digest or status matches: it independently reimports SOURCE and reconstructs the complete component and wire output from ImportedArtwork, then compares byte-for-byte.

### Positive and adversarial tests

**Accepted:** two separate same-Net groups, mixed NET_A/NET_B groups, NET_B-only groups, supported four V9.4 MILS headers, ASCII tab lexical header, reversed trace direction and deterministic topology, exact SOURCE digest changes.

**Rejected / Fail-Closed:** one-tick endpoint junction gap; wrong owner, part, pin, net, layer or width; same-Net name-only bridge; cross-group pad/trace envelope collision, including different nets; unsupported Via/Pour/Thermal/Cutout/Arc; duplicate/missing extra source records; empty or duplicated SIGNAL section; trailing PADS-like header after `*END*`; forged graph edge/Component/receipt; stale SOURCE reimport and mismatched SHA; promotion of local geometry result into global Electrical PASS.

A positive fixture proves only the explicitly bounded geometry. A rejected envelope collision means **not qualified**, not a proven electrical Short. No physical plating, reference plane, global connected-net or solver assertion is made.

## Code PR — ACCEPTED

- [Code PR #464](https://github.com/july-rust/BoardSI/pull/464), exact final HEAD [df45b9f467fa0dac29d7d3d09e38a8989cea508e](https://github.com/july-rust/BoardSI/commit/df45b9f467fa0dac29d7d3d09e38a8989cea508e/checks): **34/34 PR Check Runs SUCCESS and 11/11 actual pull_request Workflows SUCCESS**.
- Rustfmt, Clippy, Debug and Release native suites, plus Linux Qt Offscreen, Windows x64/ARM64 and macOS ARM64 native Qt matrix completed SUCCESS; native Qt D_040 scenario executed within the existing Qt runtime job.
- Final exact-HEAD independent Codex review: **no additional major issues**. Previous **3 P1/P2** findings corrected and all GitHub review threads resolved (**0 unresolved**).
- Historical intermediate HEAD failures kept: Rustfmt, missing per-SIGNAL inventory validation, Windows CRLF-only test input, NET_B-only Qt parity, Qt accepting a header-like row after `*END*`. Fixes were reverified on later heads; earlier PASS/FAIL evidence was not recycled as final HEAD proof.
- Protected GitHub **Verified, single-parent Squash Merge**: [code-main f65d6abbdb7e289a27e6b548798d5fcc4c03df9a](https://github.com/july-rust/BoardSI/commit/f65d6abbdb7e289a27e6b548798d5fcc4c03df9a); sole parent `804d5a8e3839683ff06dc6e68d891d3ea6830d44`, REST verification.verified=true/reason=valid, main ref equal.

## Exact code-main Push — ACCEPTED

The independent `event=push` CI for **main@f65d6abbdb7e289a27e6b548798d5fcc4c03df9a** finished **8/8 Workflows SUCCESS and 28/28 Check Runs SUCCESS**, with no pending, failed, skipped or cancelled check.

- [Qt runtime main Push #42026344040](https://github.com/july-rust/BoardSI/actions/runs/42026344040): Linux Qt, Windows x64, Windows ARM64 and macOS ARM64 all SUCCESS.
- [Numerical core main Push #42026344056](https://github.com/july-rust/BoardSI/actions/runs/42026344056): Rust native matrix SUCCESS.
- All remaining required M2/PH-06/PH-07/FEM, docs and CodeQL checks SUCCESS for this exact code-main SHA.

## Documentation / administrative closeout

Only these two Markdown files are eligible for the D_040 docs-only PR:

1. [canonical checklist](../development-checklist.md), Stage D adds exactly one bounded checked item **60 → 61**, whole checklist **131 → 132**, pending counts unchanged (**Stage D 2; total 122**).
2. This D_040 qualification ledger.

Use existing docs-only CI classification to **SKIP** native compilers before runner allocation while preserving all required branch protection contexts. The exact docs PR HEAD CI, independent Codex review, protected Verified Squash and exact docs-main Push, plus final Issue closure evidence, are recorded on [Issue #463](https://github.com/july-rust/BoardSI/issues/463) after their independent gates. Do not create a recursive docs PR to self-update this ledger.

## Explicit non-goals and next development

No proof of physical copper connection, global Electrical Open/Short, arbitrary PADS/ODB++ Net connectivity, manufactured Plated Via Barrel, Pour/Thermal or Reference Plane Split/Cutout P009, PH-08, verified RLGC/S solver, physical Scenario/Cache, or U01–U48/T01–T54 formal acceptance is provided. Do not present the checklist count as product completion percentage. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN for actual physical/network connectivity and reference return path work.
