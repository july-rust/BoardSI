# F_012 — Native board metadata, stackup and object inspection

**Status: implementation, exact PR HEAD qualification and protected code merge COMPLETE for the supported fixture/source subset.** Baseline `2a6857aa5252a0338098d7206aa37336ead1e671`; [implementation PR #678](https://github.com/july-rust/BoardSI/pull/678). Exact code-main Push and final documentation-main acceptance are separate mandatory gates. The final receipt on PR #678 will establish administrative closure; do not infer it from this ledger alone.

## Scope and operation

Open a supported Native .bsi, use the Inspector toggle in the Main Qt Native panel, choose category and Inspect, or pick a single supported artwork object. Parent/child indices select nested detail pages; Next page follows the returned offset. Exact JSON retains integer tick strings and rational metre numerators/denominators. The structured tree displays the stored values, not an independent measurement. Geometric centreline length is an explicitly approximate binary64 value, not electrical length. Source hashes are retained provenance, not proof the missing CAD files were re-read. Missing values remain missing; a contiguous stackup total is derived only when stored layer faces and dimensions are available and representable, and assumed input stays ASSUMED. Drill observations with unknown plating/span/alignment stay unknown. Main Qt can inspect structurally valid NOT_RENDERED documents.

The Worker uses BSI_NATIVE_INSPECT_V1, accepted snapshot SHA-256, request UUID, category/parent/child/offset, complete ObjectId or drill observation identity, and a bounded page of up to 128 rows. Each response is under 60,000 bytes; row budget 32,000 bytes. Native Reader validates before inspection, digest checked before and after. No CAD source reopening or Native write. The measured fresh read-only worker reloads the snapshot for each request; it is the bounded alternative to a persistent cache, not an interactive-speed optimization. Stale selections, invalid receipts, cancel, timeout, crash and close clear results without publishing stale data. Qualification-only pause uses the existing gated feature; no production fault hook. No changes to SQLite v1, BSIG v1 or frozen Golden files.

## Acceptance ledger

| Gate | Evidence | Verdict |
|---|---|---|
| T01 | Board counts, source identity/hash, metadata on five source-free formats | PASS (supported fixtures) |
| T02 | Frozen geometry and independent SQL stackup/material states, exact total and provenance | PASS |
| T03 | Full ObjectId, source index, part, exact picking, wrong identity rejection | PASS |
| T04 | Literal frozen arc/area/cutout/outline/padstack/via coordinates/dimensions, source widths/drills and exact synthetic trace oracle | PASS (fixture scope; not exhaustive CAD geometry equivalence) |
| T05 | Exact i64/rational strings, bounded pages, geometric length oracle | PASS (exact stored units; no arbitrary rounded display guarantee) |
| T06 | Five supported CAD fixture imports, delete source, actual fresh Main Qt Open, Save As and reopen | PASS |
| T07 | Real Qt artwork picking and latest selection, failure resynchronization, NOT_RENDERED inspection | PASS |
| T08 | 128/129/1k/10k/10001/100k first/tail/repeat queries; measured fresh-worker resource gates | PASS (bounded structural inspection; not interactive latency) |
| T09 | Actual QProcess cancel/timeout/crash/close-reopen and unchanged digest | PASS |
| T10 | Golden SHA, hostile categories/offset/identity, existing F003/F004 fail-closed compatibility regressions | PASS |
| T11 | Rust Debug/Release and Main Qt four platforms, full regression | PASS |
| T12 | Exact PR HEAD CI/Codex, verified protected squash, exact code-main and docs-main Push | PARTIAL — PR HEAD and protected squash verified; final receipt required |

## Exact implementation evidence — 2026-11-28 KST

- HEAD `3c769475a10521a992d20496723b191da70e20a4`: 14/14 PR workflows, 55/55 checks SUCCESS, [independent Codex](https://github.com/july-rust/BoardSI/pull/678#issuecomment-7236365040) no findings, both historical P2 threads resolved. [Acceptance receipt](https://github.com/july-rust/BoardSI/pull/678#issuecomment-7237791152).
- [Numerical core](https://github.com/july-rust/BoardSI/actions/runs/46124079383): four F012 test functions, debug/release, four OS = 32/32 executions, including independent frozen geometry/SQLite oracle 8/8. Linux job 140914739729, Win x64 140914739767, Win ARM64 140914739698, macOS ARM64 140914739717.
- [Main Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/46124079454): 4/4 platform jobs, five-format actual Inspector and 72/72 bounded query measurements; lifecycle cancel/timeout/crash/close-reopen/unchanged SHA all true per OS = 20/20. Linux x64 offscreen provisional job 140914782918, Win x64 140914782967, Win ARM64 140914782949, macOS ARM64 140914782931. The existing F011 rendering limit remains 10k; 100k structural open/inspection is NOT_RENDERED.
- [Protected squash](https://github.com/july-rust/BoardSI/commit/d3f2889beac8ccdcb6137d94c715108705fbb9c3): merged=true, verified=true (valid), main protected=true, identical PR and merged tree `8768e2d1119ed9efa68019fca1b9d3c38ce0fb95`.

## Measured query limits — final PR HEAD

| Platform | Max page ms | Max timer gap ms | Max response bytes | Worker sampled RSS bytes | Qt peak/sample RSS bytes |
|---|---:|---:|---:|---:|---:|
| Linux x64 offscreen | 23318 | 64 | 57980 | 290889728 | 191393792 |
| Windows x64 | 30790 | 70 | 57980 | 280838144 | 170958848 |
| Windows ARM64 | 38939 | 51 | 57980 | 301776896 | 177184768 |
| macOS ARM64 | 23512 | 55 | 57980 | 330317824 | 246595584 |

Fixed acceptance: 20,000 ms through 10k, 60,000 ms larger, 500 ms event-loop gap, less than 60,000 response bytes, Worker and Qt at most 1 GiB in measured samples; all 72 samples passed. Sampled RSS is not a proof of all transient peaks or long-term leak absence. First/tail/repeated first pages preserve exact fixture identity and coordinates; SHA unchanged. Full Native reload makes 100k queries slow; no cache/speedup claim. No new physical SI, electrical contact, vendor-wide CAD, actual GPU hardware, installer or product U/T qualification. Preserve EB/U/T statuses.

## Final administrative receipt

Record exact code-main Push for `d3f2889beac8ccdcb6137d94c715108705fbb9c3`, exact documentation HEAD CI and independent Codex, verified documentation squash and exact documentation-main Push in a single final comment on [PR #678](https://github.com/july-rust/BoardSI/pull/678). No recursive documentation PR is needed for the final receipt. Until both Push runs and documentation gates are confirmed, do not claim unconditional final completion.
