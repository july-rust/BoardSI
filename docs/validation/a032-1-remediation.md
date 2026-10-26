# A_032_1 — P2-W connectivity corrective implementation ledger

Status: **COMPLETE — reviewed A_032_1 corrective code integrated in A_032 PR #196, 2026-10-26**.

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

## Implemented corrective code (historical review findings resolved)

- `crates/boardsi-model/src/validation.rs`: per-validation indexed graph node / Trace / Arc / Pad / Via / CopperArea and Net lookups; remove repeated whole-via validation in node/edge path because the same `BoardModel::validate` already validates every via. Same-layer, exact-coordinate, source-net and connected-layer checks remain active. **Optimized Release Rust importer measured at exact PR HEAD on a public synthetic 10k-segment board: 0.6213/0.6038/0.5814 s; not a whole Qt GUI deadline measurement.**
- `crates/boardsi-import/src/connectivity_preview.rs`: guard `MAX_SITES_PER_POINT` before each insertion of Trace/Arc/Via contact records, and add 33-site limit and 10,000-segment positive regression cases.
- `apps/boardsi-qt-prototype/preview_import.h`: remember cumulative stderr overflow across chunks, drain stderr on process completion, and refuse import on overflow. Extract the exact bounded chunk collector for targeted native regression.
- `apps/boardsi-qt-prototype/preview_p0.cpp`: split-chunk and 256/257-byte collector regressions plus an actual spawned Python mock-worker QProcess integration smoke that emits valid V1 stdout and >256-byte malicious stderr, verifying `INVALID_CONTACT_AUDIT` and no imported view.

The native test suite and fixes were accepted on the final reviewed PR head by GitHub CI; a separate optimized Rust importer release benchmark is linked in the closeout evidence below. Whole Qt GUI import and vendor real boards remain unmeasured.

## CP3 features subsequently delivered by A_032_2

- Per-object, source-identity-backed Trace/Arc/Via contact status and source coordinate.
- Versioned, bounded per-object worker-to-Qt evidence transport (legacy V1–V14 stdout unchanged).
- CPU/RHI shared marker rendering, filtered picking, layer/net and mirror parity.
- Cross-Net potential conflict location and source Net IDs; never claim a certified Short.
- Native Qt regressions for reimport/cancel/overflow and hostile audit payloads.

**The listed CP3 work was implemented in A_032_2 via source-bound versioned contact evidence transport, strict Qt validation, and CPU/RHI overlays, with the conservative unsupported boundaries retained.**

## Final verified closeout (2026-10-26 KST)

**A_032 / A_032_1 / A_032_2: COMPLETE for the bounded P2-W deliverable, not for all P2/M1.** The earlier candidate/pending language below has been reconciled to the following exact evidence:

- Implementation PR: [#196](https://github.com/july-rust/BoardSI/pull/196), final reviewed PR head `8d65fff379de6b64d03631ca48aa5d91fc52c11f`, squash-merged 2026-10-26 03:08:09 UTC.
- Protected `main` merge commit: [`51360acac465cbff394edf63b1363c60b635478c`](https://github.com/july-rust/BoardSI/commit/51360acac465cbff394edf63b1363c60b635478c), GitHub signature **Verified** (`valid`), one parent `54dc90ccd68a80ec1455c6947ce8a2229924ee26`.
- The merged commit's [post-merge checks](https://github.com/july-rust/BoardSI/commit/51360acac465cbff394edf63b1363c60b635478c/checks) completed **17/17 SUCCESS, 0 failed/pending**, matching the 17 required contexts of active main ruleset `#25046595`. At the exact PR head, 23/23 checks passed (including the 17 required native contexts).
- Independent Codex review of exact PR head `8d65fff`: completed, no remaining new findings; all six PR review threads **resolved (6/6)**. Issue [#195](https://github.com/july-rust/BoardSI/issues/195) **CLOSED**.
- [Release 10,000-segment synthetic route benchmark](https://github.com/july-rust/BoardSI/actions/runs/40261462513/job/121374561848): Rust importer 0.6213 / 0.6038 / 0.5814 s; peak child RSS ~57,452 KiB; output 955,611 bytes. **This is not end-to-end Qt GUI latency or real-board performance certification.**
- Synthetic Rust/Qt native tests and V10–V14 opt-in compatibility are covered by the exact-head PR checks. Hardware-GPU reset, VRAM capacity and 1M-object 60 FPS were **NOT_RUN/NOT_VERIFIED**, not release blockers for this scoped local-evidence milestone.
- Remains **NOT_VERIFIED**: whole-board Open/Short, interior/T-junction/trace-width or copper flood/thermal contacts, component Pad/Pin electrical contact, real vendor-export qualification, full SI/RLGC/S-parameters, complete M1/P2. Broad Issues [#9](https://github.com/july-rust/BoardSI/issues/9), [#11](https://github.com/july-rust/BoardSI/issues/11), [#165](https://github.com/july-rust/BoardSI/issues/165) stay open.

This closeout edits validation documentation only. Historical review issues, baseline numbers, and historical pre-merge candidate states are retained as history, not current blockers.

