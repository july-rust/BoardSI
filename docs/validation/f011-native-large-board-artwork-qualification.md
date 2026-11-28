# F_011 — Large Native Artwork, bounded resource usage and performance

**Status: implementation, exact PR HEAD qualification and protected code merge COMPLETE within the supported subset.** Baseline main `246551ac133eef333b3a10d44850cd705231d82d`. [Implementation PR #676](https://github.com/july-rust/BoardSI/pull/676) is merged. Final exact code-main and documentation-main Push acceptance is recorded in a separate final receipt on PR #676; do not infer that acceptance from this document alone. No product U/T claim.

## Implementation scope

- Native SQLite v1, BSIG v1, Reader/Writer, existing Golden and Source Adapter contracts unchanged.
- Explicit per-QProcess 32-hex request token, opt-in BSI_ARTWORK_STREAM_V1 NDJSON. Legacy CLI callers without a token still receive the existing single JSON receipt with 128-item preview limit.
- Strict frame length below 60,000 bytes including newline, 16 MiB aggregate, at most 128 items per batch, at most 10,000 rendered primitives, existing 32-layer and geometry limits. Begin receipt declares item count; sequential batches; terminal batch/item counts and SHA-256 over exact preceding line bytes. Complete transport validated before any scene publication.
- The Rust worker constructs the bounded serialized transport before writing; total decoded Qt data and the staged scene are bounded by byte/item policy. This is not a claim of a measured process RSS ceiling or an incremental zero-copy pipeline.
- Main Qt stages 128 items per event-loop turn, swaps the complete scene only after validation, preserves old scene on transport or staging failure/cancel. Oversize valid Native documents load structurally and report NOT_RENDERED. Zoom, middle-drag pan, fit, layer selection and picking remain presentation-only.
- Synthetic Native fixture generation through the actual Writer is qualification-feature-gated; no source dialect expansion or real CAD vendor equivalence claim.
- Native presentation is QGraphicsView; no new native GPU/driver hardware qualification.

## Acceptance ledger

| Gate | Evidence required | Current |
|---|---|---|
| T01 | Five-format and Golden regressions | PASS — F010 five-format Qt and F004 independent Golden regression |
| T02 | 128/129 boundary, legacy and framed output | PASS — 128/129 and three paired legacy/framed comparisons per OS |
| T03 | Actual Main Qt 1k/10k, trace-only and mixed trace/via | PASS — actual 1k/10k trace-only and mixed on four OS |
| T04 | Independent expected grid/width/layer/ID/annulus checks, canonical round-trip | PASS — synthetic independent grid/width/layer/source ID and annulus oracle; Native Save As and reopen |
| T05 | Actual wheel zoom, middle drag, Fit, Picking, Layer filter | PASS — actual wheel, middle-drag, Fit, picking and filter |
| T06 | Native source-free Open, Save As, reopen, close/reopen | PASS — source-free Save As, reopen, repeated close/open |
| T07 | Malformed frame, missing/duplicate batch, sequence, token, digest, trailing and size errors | PASS — bounded malformed stream rejection suite |
| T08 | Real Worker lifecycle regression and staged-render cancellation retaining old scene | PASS — existing lifecycle suite plus actual staged cancellation retaining prior scene |
| T09 | Repeated opens/closes, worker sampled RSS and Qt peak RSS; leak/resource interpretation | PASS (bounded exercise) — zero closed-scene items, current RSS samples; not long-duration leak certification |
| T10 | 100,000 structural stress and explicit NOT_RENDERED; measured phase times, payload, batch count and event loop gap | PASS — 100k structural open with NOT_RENDERED, measured phase/bytes/batches/timer |
| T11 | Actual four-OS Native and Main Qt; mandatory regression | PASS — four OS actual Native and Qt regression |
| T12 | Exact final HEAD CI and independent Codex, protected merge, exact code-main and document-main Push | PARTIAL — exact PR HEAD CI and Codex, protected verified merge complete; exact code-main and document-main Push final receipt required |

## Final PR evidence (2026-11-28 KST)

- Exact final HEAD `1d6ac0a7cf6c39d7a30de04ef1a070702f83ef89`: **14/14 PR workflows and 55/55 check runs SUCCESS**. Independent [Codex review](https://github.com/july-rust/BoardSI/pull/676#issuecomment-7229943729) completed with zero findings/unresolved; review threads zero. The merge-message check count 52 was an undercount, corrected in the PR receipt using REST's 55 returned successful check runs.
- [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/46092652053): all four jobs succeeded; actual checkout SHA verified before build, and each exact-head JSON validated and uploaded. [Linux x64 job](https://github.com/july-rust/BoardSI/actions/runs/46092652053/job/140816955118) / [Windows x64](https://github.com/july-rust/BoardSI/actions/runs/46092652053/job/140816955076) / [Windows ARM64](https://github.com/july-rust/BoardSI/actions/runs/46092652053/job/140816955116) / [macOS ARM64](https://github.com/july-rust/BoardSI/actions/runs/46092652053/job/140816955130). Artifact IDs respectively 15242272374, 15242194092, 15242915910, 15242420249; non-expired at verification. GitHub artifact retention applies; this ledger preserves selected results beyond retention.
- [Numerical core](https://github.com/july-rust/BoardSI/actions/runs/46092652045), [F004 compatibility](https://github.com/july-rust/BoardSI/actions/runs/46092651883), [F003 Native Rust](https://github.com/july-rust/BoardSI/actions/runs/46092651966), [F003 resilience](https://github.com/july-rust/BoardSI/actions/runs/46092652004) succeeded. Existing F007 historical main Push remains unobserved; no retrospective promotion.
- [Protected verified Squash](https://github.com/july-rust/BoardSI/commit/560f0d47e4482b63b689cc2caf4c88d6a0c16c4a): GitHub merged=true, verified=true, single parent `246551ac133eef333b3a10d44850cd705231d82d`; main protected=true; exact PR and merged commit tree `2638603741e50011b8956dbfd9470e8cff6ee96e` identical.

## Actual measurements, final PR HEAD

| Case (Open ms) | Linux x64 | Windows x64 | Windows ARM64 | macOS ARM64 |
|---|---:|---:|---:|---:|
| 128 trace | 50 | 98 | 126 | 57 |
| 129 trace | 50 | 96 | 123 | 64 |
| 1,000 trace | 222 | 393 | 485 | 260 |
| 10,000 trace | 2351 | 4078 | 4845 | 2310 |
| 10,000 mixed trace/via | 4342 | 6817 | 8089 | 4673 |
| 10,001 structural NOT_RENDERED | 1703 | 3031 | 3677 | 2202 |
| 100,000 structural NOT_RENDERED | 17591 | 30782 | 38433 | 18325 |

| Resource / transport metric | Linux x64 | Windows x64 | Windows ARM64 | macOS ARM64 |
|---|---:|---:|---:|---:|
| 10k mixed Qt peak RSS bytes | 134299648 | 118280192 | 131563520 | 155271168 |
| 10k mixed Worker sampled peak RSS bytes | 55001088 | 50266112 | 60325888 | 61259776 |
| 10k mixed max timer gap ms | 110 | 115 | 79 | 82 |
| 10k mixed stream bytes | 1834760 | 1834795 | 1834795 | 1834812 |
| 10k mixed batch count | 79 | 79 | 79 | 79 |
| 100k Worker sampled peak RSS bytes | 290287616 | 280719360 | 301588480 | 329564160 |
| 128 legacy / framed median ms | 43 / 47 | 82 / 84 | 114 / 114 | 54 / 50 |
| 128 legacy / framed bytes | 16889 / 17327 | 16924 / 17362 | 16924 / 17362 | 16941 / 17379 |
| Mixed closed RSS bytes, repeat 1 / 2 | 113242112 / 113242112 | 46895104 / 45977600 | 147574784 / 101306368 | 167903232 / 162365440 |
| Mixed reopened RSS bytes, repeat 1 / 2 | 113242112 / 113242112 | 94085120 / 93716480 | 101306368 / 98238464 | 162365440 / 163807232 |

All 12 paired legacy/framed comparisons reconstruct exactly equal receipts (438-byte overhead, about 2.6%). Observed median changes are small and mixed in direction; no claim of an old-versus-new full GUI speedup. All 16 repeated-close samples (four per OS, two per composition) recorded zero remaining scene items. RSS can retain allocator/Qt caches; two repetitions per composition do not prove absence of long-term leakage. Worker sample is a lower bound, particularly macOS current RSS. Timer intervals are observations, not a separate product latency guarantee. The 10k supported rendering policy and 100k structural limit exercise are synthetic qualification, not unrestricted real-board capacity, GPU performance, manufacturing/electrical contact or physical SI certification.

## Final administrative receipt

The final exact code-main Push for `560f0d47e4482b63b689cc2caf4c88d6a0c16c4a` and the final documentation PR's exact HEAD CI/Codex, verified protected Squash and exact documentation-main Push must all be verified. Record counts, resulting SHA and verdict in one final comment on [PR #676](https://github.com/july-rust/BoardSI/pull/676) so the ledger does not recursively reference its future SHA. This document records T12 as partial until that receipt exists. The main branch and documentation status must not be advanced ahead of confirmed CI. Preserve all existing EB gates and product U/T statuses.
