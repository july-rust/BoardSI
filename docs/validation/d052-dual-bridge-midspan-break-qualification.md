# D_052 — PADS SOURCE-Exact Dual-Bridge Midspan Copper Break Qualification

**First checked: 2026-11-07 KST. Evidence date: 2026-11-07 KST.**
[Issue #500](https://github.com/july-rust/BoardSI/issues/500) · [Code PR #501](https://github.com/july-rust/BoardSI/pull/501) · [Canonical checklist](../development-checklist.md)

## 1. Bounded SOURCE witness

This is an opt-in **SOURCE_LOCAL_DUAL_BRIDGE_MIDSPAN_BREAK_ONLY** qualification of synthetic same-layer local copper-contact candidates, **not** a manufactured open-circuit test, global electrical Open/Short or SI simulation.

PADS ASCII V9.4 MILS, Layer 1, **254 nm per 0.01-mil SOURCE tick**, exactly one NET_A group with full NET_B variant; four round/square SMD pads and D_051 eight intact traces. Both bridges have width 8 mil.

- Bridge A intact: (226,300)→(378,300) mil. Midspan split: (226,300)→(290,300) and (310,300)→(378,300) mil.
- Bridge B intact: (222,400)→(382,400) mil. Midspan split: (222,400)→(290,400) and (310,400)→(382,400) mil.
- Each gap has **20 mil endpoint-centre separation** and **12 mil positive copper separation** after the round-cap radii (4+4 mil). A valid break requires strictly g > 8 mil; g=8 mil tangent or g<8 mil overlap must not be accepted as a qualified break. Old auxiliary-trace/pad contacts remain positive area; no unexpected contacts are permitted.

## 2. Actual parsed SOURCE topology

| SOURCE state | Nodes (pads + source traces) | Same-layer candidate edges | Components | Cycle rank E-N+C | U1↔U2 |
| :--- | ---: | ---: | ---: | ---: | :--- |
| INTACT | 12 | 12 | 1 | 1 | REACHABLE |
| BREAK_A | 13 | 12 | 1 | 0 | REACHABLE |
| BREAK_B | 13 | 12 | 1 | 0 | REACHABLE |
| BREAK_BOTH | 14 | 12 | 2 | 0 | SEPARATED |

Actual SOURCE trace fragments, not D_051 virtual node removal or a shared Net label, are the basis for these state graphs. The right-side pad contact is assigned to a real right fragment, preserving exactly 12 local edges. Zero cross-layer electrical edges proven.

Deterministic SHA-256-bound receipt **BOARDSI_D052_DUAL_BRIDGE_MIDSPAN_BREAK_V1** includes complete 4 Pad / 8–10 Trace / 12 Edge inventory, individual positive gap witnesses, complete graph counts and explicit P009_UNSUPPORTED, GLOBAL_ELECTRICAL_NOT_VERIFIED and SI_NOT_RUN.

## 3. Implementation and fail-closed coverage

- [Rust SOURCE encoder](../../crates/boardsi-import/src/d052_bridge_midspan_break.rs), opt-in CLI option **--d052-dual-bridge-midspan-break-v1**: independently parse actual full SOURCE; re-qualify an intact D_051 baseline as a supporting source inventory oracle; one-to-one match actual 8–10 unique traces; strict integer tick positive separation; compute DSU connectivity/cycle from actual fragment edges. Binds receipt to actual original SOURCE SHA-256.
- [Synthetic PADS fixture](../../crates/boardsi-import/tests/fixtures/d052_dual_bridge_midspan_break_both_v94.asc) and [Rust tests](../../crates/boardsi-import/tests/d052_bridge_midspan_break.rs): four states, reversed segments, NET_B, CRLF/tab, supported numeric aliases and one-tick positive gap, missing/duplicate/extra/forged SOURCE, tangent/overlap, shape, layer, width, owner, pin pair and wrong net.
- [Independent Native Qt oracle](../../apps/boardsi-qt-prototype/d052_bridge_midspan_break_qt.h) and [runner](../../apps/boardsi-qt-prototype/d052_qt_runner.cpp): actual PreviewImportPanel reImport of original SOURCE plus reconstructed D_051 intact/second-bridge-removed baselines; independently rebuild ImportedArtwork inventory and local contact graph and compare against complete Rust receipt; forged/stale receipt rejected. Timeout-safe callback disconnect and cancellation/drain.
- [Native CMake registration](../../apps/boardsi-qt-prototype/CMakeLists.txt) on Linux, Windows x64/ARM64 and macOS ARM64. Preserve D_039–D_051 regression tests.

Historical Rustfmt and Windows CRLF fixture-string test failures were corrected; **only exact final HEAD tests count as PASS**, not earlier failed runs. This is not a claim of correcting PADS-source or manufactured electrical failures.

## 4. Code PR acceptance evidence

- [Code PR #501](https://github.com/july-rust/BoardSI/pull/501) final HEAD [15d672c2d9d7bbc8e33d32786c5c062788c8246a](https://github.com/july-rust/BoardSI/commit/15d672c2d9d7bbc8e33d32786c5c062788c8246a/checks): **34/34 checks SUCCESS**, **11/11 pull_request workflows SUCCESS**, Rustfmt/Clippy, Rust Debug/Release and Native Rust Win x64/ARM64, macOS ARM64; Native Qt Linux/Windows x64/ARM64/macOS ARM64. Linux Qt **74/74 CTest PASS** including D_052 runner **D052_NATIVE_QT_REAL_MIDSPAN_BREAK true GLOBAL_ELECTRICAL_NOT_VERIFIED**.
- Same HEAD independent Codex review: completed, findings=false, unresolved=false; **0 unresolved review threads**.
- Protected Verified **single-parent Squash** [code-main 310563cc375a899c96800136a19eed5ae93f667b](https://github.com/july-rust/BoardSI/commit/310563cc375a899c96800136a19eed5ae93f667b); sole parent f6e93642d60ea46f2c3f418999fcb1b11bf39411, GitHub REST verification.verified=true/reason=valid; main ref matched immediately after merge.
- Independent **code-main event=push** CI is a **separate gate**. Record exact results in [Issue #500](https://github.com/july-rust/BoardSI/issues/500) after completion, without substituting PR checks or inventing a PASS.

## 5. Documentation gate and scope limitations

The separate docs-only PR changes exactly **docs/development-checklist.md** and this D_052 ledger, runs optimized Docs-only CI without Native Rust/Qt builds, requires independent exact-HEAD review, Verified one-parent Squash and independent exact docs-main push CI. Final outcome is recorded in Issue #500 without recursively generating new documentation PRs.

Limited evidence counts after code acceptance: Stage D **72→73 checked** (2 pending unchanged); overall **143→144 checked** (122 pending unchanged). Not a product-completion estimate.

**NOT_VERIFIED:** manufactured physical copper continuity or actual PCB open-circuit; global electrical Open/Short; arbitrary PCB/PADS, unsupported Via/Plane/Pour/Thermal/contact geometry; P009/PH-08 physical Return Path; physical Scenario/Cache; RLGC/S solver; formal U01–U48/T01–T54 acceptance; release. Rejected cross-net/ambiguous geometry is not a proved electrical short. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN.
