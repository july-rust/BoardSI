# D_030 — Exact Steep 3:4 Non-45° Via-B SOURCE Endpoint Qualification

**Scope:** `SOURCE_GEOMETRY_ONLY`. This document certifies an independently reconstructed layer-local contact candidate only, not fabricated Copper, plated Via barrel conduction or any electrical/solver capability.

**Tracker:** [Issue #431](https://github.com/july-rust/BoardSI/issues/431) · **Code PR:** [#432](https://github.com/july-rust/BoardSI/pull/432) · **Date:** 2026-11-04 KST.

## Frozen geometry and interpretation

PADS V9.4 MILS (254 nm per source tick), NET_A, L4: Main Trace `(15000,25000)→(35000,25000)`; Branch `(35000,25000)→(44000,37000)`; Via-B at `(35000,25000)`. Exterior displacement `(+9000,+12000)` is an exact steep 3:4 horizontal-to-vertical slope (>45°). Reverse Branch is accepted. Rust signed i128 requires nonzero cross/dot, `3*abs(cross)==4*abs(dot)`, shared Via-B endpoint, and exterior projection strictly greater than main length squared. No floating-point tolerance.

- Public fixture: [D_030 PADS V9.4 MILS](../../crates/boardsi-import/tests/fixtures/d030_non45_steep_via_b_endpoint_v94.asc).
- Explicit CLI opt-in: `--d030-non45-steep-via-b-endpoint-contact-v1 NET_A 1 4 0 1`.
- Rust witness type: `SOURCE_NON45_STEEP_VIA_B_ENDPOINT_CONTACT_CANDIDATE`; overall state `NON45_STEEP_VIA_B_ENDPOINT_CONTACT_SOURCE_CANDIDATE_ONLY`.
- Expected and qualified candidate topology: **8 Source Nodes / 6 same-layer Candidate Edges / 3 local Components / 0 proven Electrical Cross-layer Edges**.
- Qt independently reimports canonical Source and verifies complete SHA-bound wire; no trust escalation from Rust PASS.
- Fail-closed hostiles: 1-tick gap, lateral offset and angle drift, shallow 4:3/45°/orthogonal/collinear cross-modes, wrong Net/Layer/Via, source inventory changes, stale/forged receipts and electrical proof promotion.

## Exact code PR acceptance

Final PR HEAD `c294d979e8ef1dc945be4a9174934ad6f655d565`: **34/34 Check Runs SUCCESS · 11/11 pull_request Workflows SUCCESS**. D_030 Rust **Debug 3/3 and Release 3/3 PASS** ([Numerical #41793834432](https://github.com/july-rust/BoardSI/actions/runs/41793834432)), Linux Qt Offscreen **52/52 CTest PASS**, D_030 native Qt Windows x64 / Windows ARM64 / macOS ARM64 SUCCESS ([Qt Runtime #41793834529](https://github.com/july-rust/BoardSI/actions/runs/41793834529)). Independent Codex same-HEAD review: completed, zero findings, zero unresolved findings/threads.

Historical failures preserved: first superseded code HEAD `05ec9e4` failed Rustfmt; same-HEAD Codex review found a P1 Native Qt smoke CLI-flag mismatch. Both were corrected, original review thread resolved, and final code HEAD reviewed afresh without findings. Do not represent the initial failures as passing.

## Verified protected code-main and independent Push

[Code-main `f898ccc8ccedfca2be5876a986a521fb770d3dc1`](https://github.com/july-rust/BoardSI/commit/f898ccc8ccedfca2be5876a986a521fb770d3dc1) is GitHub **Verified=true, reason=valid**, one-parent Squash (parent `beeb47e8e70301d12ae77e8ea978f52e13f4f474`). Exact code-main SHA independent `event=push`: **8/8 Workflows SUCCESS and 28/28 Check Runs SUCCESS** ([Qt Push #41795317730](https://github.com/july-rust/BoardSI/actions/runs/41795317730), [Numerical Push #41795317722](https://github.com/july-rust/BoardSI/actions/runs/41795317722)). These results establish bounded SOURCE geometry code acceptance, not physical continuity.

## Separate Markdown-only closeout gate

Update `docs/development-checklist.md` plus this validation ledger in a distinct docs-only PR using existing docs-only classifier. Obtain exact docs PR HEAD CI, independent Codex no-findings review, zero unresolved threads, protected Verified one-parent Squash, and exact post-docs-main Push CI **before** administrative closure of Issue #431. Record final docs-main SHA and runs in Issue #431, not another recursive docs PR. Docs-only skipped native compilations are not new platform PASS evidence.

## Explicit unaccepted scope

**NOT_VERIFIED:** manufactured copper / plated Via Barrel conduction, global Open/Short, P009 / PH-08 Return Path, RLGC/S-parameter solver, physical Scenario/Cache, formal M1/M2 U/T or release. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN; legacy PR #35 remains CLOSED/UNMERGED.
