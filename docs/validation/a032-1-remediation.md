# A_032_1 — P2-W connectivity corrective implementation ledger

Status: **IN_PROGRESS — NOT ACCEPTED — DO NOT MERGE**.

Repository: https://github.com/july-rust/BoardSI
Issue: https://github.com/july-rust/BoardSI/issues/195
Existing PR: https://github.com/july-rust/BoardSI/pull/196
Baseline protected main at start: `54dc90ccd68a80ec1455c6947ce8a2229924ee26`.
Branch: `a032/p2w-endpoint-connectivity`.

## Actual review findings (not invented)

- [P2 validation time complexity](https://github.com/july-rust/BoardSI/pull/196#discussion_r4373055713): repeated scans of graph nodes and source conductors, with a reported 10,000-segment **11.9 s vs 0.27 s** release baseline, exceeding Qt's **10 s** import deadline.
- [P2 insertion-time contact bucket](https://github.com/july-rust/BoardSI/pull/196#discussion_r4373069769): 32-site constraint previously evaluated only after potentially hundreds of thousands of sites had been allocated.
- [P2 truncated stderr accepted](https://github.com/july-rust/BoardSI/pull/196#discussion_r4373069773): Qt collected only the first 256 bytes and forgot overflow, allowing malformed success diagnostics.
- [P2 via-barrel fixture](https://github.com/july-rust/BoardSI/pull/196#discussion_r4373010292): two-pad top/bottom fixture cannot imply inner-layer conductive pad. Existing A_032 candidate already has positive three-pad and negative absent-inner-pad regression; native evidence must be rechecked.

## Corrective code candidate

- `crates/boardsi-model/src/validation.rs`: per-validation indexed graph node / Trace / Arc / Pad / Via / CopperArea and Net lookups; remove repeated whole-via validation in node/edge path because the same `BoardModel::validate` already validates every via. Same-layer, exact-coordinate, source-net and connected-layer checks remain active. **Runtime speed is NOT_VERIFIED until measured on exact final HEAD.**
- `crates/boardsi-import/src/connectivity_preview.rs`: guard `MAX_SITES_PER_POINT` before each insertion of Trace/Arc/Via contact records, and add 33-site limit and 10,000-segment positive regression cases.
- `apps/boardsi-qt-prototype/preview_import.h`: remember cumulative stderr overflow across chunks, drain stderr on process completion, and refuse import on overflow. Extract the exact bounded chunk collector for targeted native regression.
- `apps/boardsi-qt-prototype/preview_p0.cpp`: split-chunk and 256/257-byte stderr collector regressions in the existing real Qt smoke, separate from decoder-only tests.

These are **source changes**, not a claim that runtime CI or an end-to-end hostile QProcess run has passed. A direct spawned-malicious-worker integration test and independently measured release import latency remain required for full A_032_1 acceptance.

## Outstanding owner-required CP3 features

- Per-object, source-identity-backed Trace/Arc/Via contact status and source coordinate.
- Versioned, bounded per-object worker-to-Qt evidence transport (legacy V1–V14 stdout unchanged).
- CPU/RHI shared marker rendering, filtered picking, layer/net and mirror parity.
- Cross-Net potential conflict location and source Net IDs; never claim a certified Short.
- Native Qt regressions for reimport/cancel/overflow and hostile audit payloads.

**These are NOT_IMPLEMENTED and must not be represented as completed by global audit counters.**

## Exact-head verification gates

- Rust workspace fmt, clippy and test, including the 10k test.
- Native Qt Linux x64, Windows x64, Windows ARM64 and macOS ARM64. Native GPU/CPU coverage must be named explicitly.
- Repeated optimized/release 10,000-segment import measurements against the actual Qt 10-second deadline; debug tests alone are not timing certification.
- Full 17 required GitHub check contexts SUCCESS at the final PR HEAD, with accurate PASS/FAIL/NOT_RUN states.
- Exact-final-head independent Codex review, no P1/P2 findings or unresolved threads.
- Unchanged ruleset #25046595; GitHub Verified squash-only merge followed by separate protected-main required checks and signature verification.

Do not close Issue #195, do not merge PR #196 and do not assert P2 complete while any gate is missing. Preserve unrelated broad Issues #9/#11/#165 and all 2D RLGC/S / vendor export / full-board electrical exclusions.
