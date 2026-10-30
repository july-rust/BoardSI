# C_017 — Bounded Reference Gap / Layer Transition V1–V4 Integration

**Tracker:** [Issue #287](https://github.com/july-rust/BoardSI/issues/287) · **Code:** [PR #288](https://github.com/july-rust/BoardSI/pull/288) · **Baseline:** `main@345f25f15f607a6d28f20a46781a66e3558a71f8`  
**First checked:** 2026-10-30 KST · **Evidence date:** 2026-10-30 KST.  
**Verdict: LIMITED CODE PASS — source-geometry V1–V4 integration only.** The documentation PR's exact-head review/CI and post-document-main validation are separately required, and are not pre-certified here.

## Actual bounded regression

The added `crates/boardsi-import/tests/c017_reference_gap_transition_integration.rs` exercises **four** integration cases on public synthetic PADS V9.4 MILS:

1. V1/V2/V3 same original byte source: separate opt-in headers; identical SHA; exact V2 per-segment receipts embedded inside V3; a COPCUT gap stays warning-only, physical result stays `UNSUPPORTED/NOT_RUN`.
2. A changed route coordinate changes V1/V2/V3 source hashes, invalidating original source-bound receipts.
3. V4 two-layer plated Via selected with two distinct Reference Nets/Layers embeds both independent V2 receipts verbatim and retains the gap and `REFERENCE_TRANSFER_UNVERIFIED` states.
4. Mutated via-route bytes produce different V4 source SHA; missing Reference Net stays unsupported rather than a physical PASS.

This is not a new Qt selftest or real asynchronous IPC integration test. Existing native Qt C_005/C_007/C_008 tests and Linux Qt CTest remain the evidence for selection, marker and stale lifecycle behavior. Existing V1–V4 contracts were not rewritten.

## Independently verified code PR and merge

| Gate | Verified result | Reference |
|---|---|---|
| Source PR final HEAD | `d85846799fa81bf236ad0e72deb9fbb8452b7e19`; **22/22 Check Runs SUCCESS**, **8/8 pull-request workflows SUCCESS** (plus CodeQL) | [Exact code HEAD checks](https://github.com/july-rust/BoardSI/commit/d85846799fa81bf236ad0e72deb9fbb8452b7e19/checks), [PR #288](https://github.com/july-rust/BoardSI/pull/288) |
| Rustfmt + Clippy | PASS after the first-head formatting correction | [Numerical core workflow](https://github.com/july-rust/BoardSI/actions/runs/41003351384) |
| Four new Rust source-only tests | **4/4 PASS in Debug and 4/4 PASS in Release**; independent prior B/C regression retained | [Numerical core workflow](https://github.com/july-rust/BoardSI/actions/runs/41003351384) |
| Native Rust + Qt | Windows x64, Windows ARM64 and macOS ARM64 native jobs SUCCESS; Linux x86_64 Qt offscreen SUCCESS (existing C_005/C_007/C_008 Qt tests reused) | [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/41003351260), [Numerical core](https://github.com/july-rust/BoardSI/actions/runs/41003351384) |
| Independent same-final-HEAD Codex | `status=completed`, `commit_sha=d85846799fa81bf236ad0e72deb9fbb8452b7e19`, `has_any_findings=false`, `has_unresolved_findings=false` | [PR #288 review summary](https://github.com/july-rust/BoardSI/pull/288) |
| Protected Squash-only merge | [`main@046ba46ba93fc6ced803519bba96982f3b7311d7`](https://github.com/july-rust/BoardSI/commit/046ba46ba93fc6ced803519bba96982f3b7311d7); one parent; GitHub signature `verified=true, reason=valid` | [Verified Squash commit](https://github.com/july-rust/BoardSI/commit/046ba46ba93fc6ced803519bba96982f3b7311d7) |
| Independent code post-main CI | **21/21 Check Runs SUCCESS, 8/8 Push Workflows SUCCESS** on the exact code merge SHA | [Code post-main checks](https://github.com/july-rust/BoardSI/commit/046ba46ba93fc6ced803519bba96982f3b7311d7/checks), [post-main Qt workflow](https://github.com/july-rust/BoardSI/actions/runs/41004290406) |

The initial source HEAD `8b0c333` failed Rustfmt and the dated-checklist docs assertion. Both were corrected and **the final exact `d858467` source HEAD passed**; the initial failures were not described as passes. This document makes no claim that the new source-only Rust test itself exercises an asynchronous Qt IPC stale reply.

## Historical C_002 checkbox reconciliation

The C_002 [PR #240](https://github.com/july-rust/BoardSI/pull/240) initially included **Rust-only risk API** and properly left Qt marker integration unchecked. Later [C_005 #249](https://github.com/july-rust/BoardSI/pull/249)/[Native #250](https://github.com/july-rust/BoardSI/pull/250) provided source-bound Qt CPU/RHI `REFERENCE_SPLIT_RISK` and `LAYER_TRANSITION_RISK` display, source state invalidation and Native smoke. [C_008 #259](https://github.com/july-rust/BoardSI/pull/259) added two-Layer Via source geometry/Qt risk. The C_002-related historical checkbox was updated **only for the later verified limited Qt marker scope** (first checked 2026-10-30, evidence from 2026-10-28/29); it is *not* a claim that C_002 originally implemented Qt or that P009 is validated.

## Boundaries / handoff

- Source-exact PADS V9.4 MILS **supported subset and synthetic fixtures only**. No whole-board electrical copper continuity, arbitrary PADS, manufactured clearance, thermal/anti-pad current, RLGC/S, or validated high-frequency return-current model.
- `return_path_p009=UNSUPPORTED`; `physical_si=NOT_RUN`; `solver_ready=false`; `cache_reuse_eligible=false`.
- [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN, and formal U09/U10/U39 / T02/T04/T24/T52 remain unchecked.
- The limited C_009–C_012 Stitch Via/Corridor/Pad integration belongs to C_018, C_013–C_016 mixed Cutout belongs to C_019, B_007 source review and older Connectivity to C_020.
- Documentation closure must independently pass exact HEAD CI/review, protected verified squash and post-document-main checks. Those future checks are not assumed in this code ledger.
