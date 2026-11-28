# F_011 — Large Native Artwork, bounded resource usage and performance

**Status: IMPLEMENTATION CANDIDATE / QUALIFICATION PENDING.** Baseline main `246551ac133eef333b3a10d44850cd705231d82d`. [Implementation PR #676](https://github.com/july-rust/BoardSI/pull/676). Code and test definitions are not observed execution evidence. No merge or product U/T claim.

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
| T01 | Five-format and Golden regressions | NOT_OBSERVED |
| T02 | 128/129 boundary, legacy and framed output | NOT_OBSERVED |
| T03 | Actual Main Qt 1k/10k, trace-only and mixed trace/via | NOT_OBSERVED |
| T04 | Independent expected grid/width/layer/ID/annulus checks, canonical round-trip | NOT_OBSERVED |
| T05 | Actual wheel zoom, middle drag, Fit, Picking, Layer filter | NOT_OBSERVED |
| T06 | Native source-free Open, Save As, reopen, close/reopen | NOT_OBSERVED |
| T07 | Malformed frame, missing/duplicate batch, sequence, token, digest, trailing and size errors | NOT_OBSERVED |
| T08 | Real Worker lifecycle regression and staged-render cancellation retaining old scene | NOT_OBSERVED |
| T09 | Repeated opens/closes, worker sampled RSS and Qt peak RSS; leak/resource interpretation | NOT_OBSERVED |
| T10 | 100,000 structural stress and explicit NOT_RENDERED; measured phase times, payload, batch count and event loop gap | NOT_OBSERVED |
| T11 | Actual four-OS Native and Main Qt; mandatory regression | NOT_OBSERVED |
| T12 | Exact final HEAD CI and independent Codex, protected merge, exact code-main and document-main Push | NOT_OBSERVED |

## Measurement limitations and outstanding gates

The test receipt records end-to-end Open, worker+transfer and scene phase times, sampled timer gaps, payload bytes/batches, Qt process peak RSS and live worker RSS samples. A sampled live high-water value is a lower bound; macOS is sampled current RSS, Windows/Linux may expose OS high-water while the child is alive, -1 means unavailable. No claim of exact worker lifetime peak, controlled A/B speedup, UI latency acceptance, measured memory-leak bound, actual GPU qualification or completed physical SI until evidence exists. Compare with original 128-object policy only under identical conditions; 10,000 is the proposed supported display ceiling, not a measured product limit. Preserve existing timing thresholds and disclose failures.

The final ledger must contain actual CI jobs/logs, per-platform measurement values, exact final review SHA, verified protected merge and exact main Push. Do not promote this candidate or modify historical F007 post-merge evidence, EB gates or product U/T statuses. Use a single checklist row and a final PR receipt.
