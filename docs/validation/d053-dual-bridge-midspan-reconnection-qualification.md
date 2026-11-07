# D_053 — PADS SOURCE-Exact Dual Finite-Width Bridge Midspan Copper Reconnection

**First checked: 2026-11-07 KST. Evidence date: 2026-11-07 KST.**
[Issue #503](https://github.com/july-rust/BoardSI/issues/503) · [Code PR #504](https://github.com/july-rust/BoardSI/pull/504) · [Canonical checklist](../development-checklist.md)

## 1. Qualified boundary

**SOURCE_LOCAL_DUAL_BRIDGE_REPAIR_ONLY** is a bounded synthetic PADS Layout ASCII V9.4 MILS **Layer 1** local copper-contact-candidate qualification. 0.01 mil tick = 254 nm; four original mixed round/square SMD pads, eight D_051 original traces, two independent 8-mil width bridge paths. It is not a physical manufacturing or global electrical continuity test.

D_052 midspan breaks are real source partitions, not virtual graph deletions. Bridge A left (226,300)→(290,300) and right (310,300)→(378,300) mil; Bridge B left (222,400)→(290,400) and right (310,400)→(382,400) mil. Positive copper gap in each case: 12 mil. Explicit independently authored source repair traces:

- Repair A: U1.1 U2.1 (288,300)→(312,300) mil, Layer 1 width 8 mil.
- Repair B: U1.2 U2.2 (288,400)→(312,400) mil, Layer 1 width 8 mil.

Each repair is a **separate actual SOURCE trace**, with strict positive-area copper overlap with the left AND right trace fragment, not a synthetic edge nor a net-label guess. A one-tick positive-area overlap is accepted; tangent or positive clearance is rejected for a completed repair.

## 2. SOURCE-local graph acceptance

| Actual SOURCE state | Nodes | Contact edges | Components | Cycle rank E − N + C | U1 ↔ U2 |
| :--- | ---: | ---: | ---: | ---: | :--- |
| BREAK_BOTH | 14 | 12 | 2 | 0 | SEPARATED |
| REPAIR_A_ONLY | 15 | 14 | 1 | 0 | REACHABLE |
| REPAIR_B_ONLY | 15 | 14 | 1 | 0 | REACHABLE |
| REPAIR_BOTH | 16 | 16 | 1 | 1 | REACHABLE |
| BREAK_A_REPAIR_A | 14 | 14 | 1 | 1 | REACHABLE |
| BREAK_B_REPAIR_B | 14 | 14 | 1 | 1 | REACHABLE |

The graph is reconstructed from separately identified real imported SOURCE traces, qualified local contact edges, and source pad/trace evidence, not from a manufactured or global electrical oracle.

## 3. Implementation / fail closed

- [Rust SOURCE encoder](../../crates/boardsi-import/src/d053_midspan_repair.rs), CLI opt-in `--d053-source-midspan-repair-v1`: verify bounded D_052 bare-SOURCE baseline independently, identify repair rows, require source-authored and parsed 1:1 inventory, compute dual end-contact geometry, graph and original-source SHA-256-bound deterministic full receipt (`BOARDSI_D053_SOURCE_MIDSPAN_REPAIR_V1`). No silent skipped/extra/duplicate tracks.
- [Rust tests](../../crates/boardsi-import/tests/d053_midspan_repair.rs): six correct graph states and NET_B, CRLF, reversed trace, 1-tick-positive boundary; hostile missing/partial/tangent/clearance, wrong owner/layer/width/net/pad shape, source extra/duplicate, forged records. The first versions had a bridge node-index bug and one invalid hostile-test expectation; these were fixed and independently requalified on the final HEAD, not retroactively relabeled PASS.
- [Independent Native Qt geometry](../../apps/boardsi-qt-prototype/d053_midspan_repair_qt.h) and [real runner](../../apps/boardsi-qt-prototype/d053_qt_runner.cpp): actual `PreviewImportPanel` reImport of original, bare, restored and reduced PADS SOURCE; own `ImportedArtwork` Pad/Trace ID matching, strict geometry and graph, exact full Rust receipt reconstruction, stale/forged rejection and hostile real reImport. [CMake](../../apps/boardsi-qt-prototype/CMakeLists.txt) registered Native tests on Linux x64, Windows x64/ARM64, macOS ARM64.
- Preserve D_039–D_052 regressions. Reject unsupported geometry in the bounded qualification, never manufacture a false electrical Open/Short result.

## 4. Verified code PR evidence

- [Code PR #504](https://github.com/july-rust/BoardSI/pull/504) exact final HEAD [944cd9d33dbf35d0354a957217f6c29d76b56c48](https://github.com/july-rust/BoardSI/commit/944cd9d33dbf35d0354a957217f6c29d76b56c48/checks) — **34/34 PR checks SUCCESS, 11/11 PR workflows SUCCESS**. Numerical, Rustfmt/Clippy, Rust Debug/Release, Native Windows x64/ARM64 and macOS ARM64, Native Qt Linux x64 (75/75 CTest PASS including `D053_NATIVE_QT_SOURCE_REPAIR true GLOBAL_ELECTRICAL_NOT_VERIFIED`), Qt Windows x64/ARM64, and Qt macOS ARM64.
- Same final HEAD independent Codex review: `completed / findings=false / unresolved=false`; **zero open review threads**, previous obsolete test finding corrected.
- Protected one-parent Verified squash merge to [code-main 2cbaac6af8845b89c087266ddcecab2824c75fa6](https://github.com/july-rust/BoardSI/commit/2cbaac6af8845b89c087266ddcecab2824c75fa6), parent `30d46fa2f81a9e5ed6516aa667c1f59233553b08`, GitHub verification `verified=true, reason=valid`. Independently inspect exact **event=push** code-main CI as a separate acceptance gate; record completed result in [Issue #503](https://github.com/july-rust/BoardSI/issues/503), never substitute PR checks.

## 5. Documentation gate and remaining scope

Exactly two Markdown-only files: this ledger and `docs/development-checklist.md` (restore missing D_052 overview row and add one new bounded D_053 check). After acceptance, Stage D **73→74 checked / 2 pending**, overall **144→145 checked / 122 pending**; these are ledger counts, **not product completion percentage**. Docs-only PR must verify optimized CI (skip native builds), independent final-head review, protected Verified squash and exact docs-main push. Results recorded in Issue #503.

**NOT_VERIFIED**: manufactured physical copper continuity; arbitrary/global electrical Open/Short or shorts between different nets; Via/Plane/Pour/Thermal/cross-layer copper; P009/PH-08 physical return path; physical scenario/cache; RLGC/S-parameters; formal U01–U48/T01–T54 acceptance; distribution/release. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.
