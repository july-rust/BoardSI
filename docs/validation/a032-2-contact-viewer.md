# A_032_2 — bounded local endpoint evidence transport and Qt Artwork overlay

Tracking: [Issue #195](https://github.com/july-rust/BoardSI/issues/195), [PR #196](https://github.com/july-rust/BoardSI/pull/196).
Status: **COMPLETE — A_032_2 merged into protected main via PR #196 with 17/17 successful post-merge mandatory checks**.

## Shipped source (merged)

1. `boardsi-import-artwork INPUT --contact-records-v1` is strictly opt-in. The one-argument default's complete V1–V14 stdout remains unchanged. The suffix begins *after* its ordinary `END` newline; versioned `BOARDSI_CONTACT_RECORDS_V1` header has total record count and the original source-file SHA-256. `BOARDSI_CONTACT_END_V1` closes it.
2. `E|S` transports an already-validated Canonical exact same-layer, same-Net endpoint-to-endpoint contact; `E|B` transports qualified plated-through-via barrel evidence between two independently contacted conductive layers. Object kinds/indexes, endpoint site, original layer and exact source-grid nm coordinates are carried. These positions are **not** inferred from trace width, proximity, or display geometry.
3. `X|C` is **POTENTIAL_CONFLICT_ONLY** at exactly coincident supported same-layer endpoints with different source NetId. Qt resolves both original Net names from the imported source objects. `X|C` is never a Canonical electrical edge and never a certified Short.
4. Rust uses bounded per-grid buckets (32), finite graph/record limits (100,000), a 1 MiB evidence suffix cap and an existing 2 MiB combined stdout cap. Any unsupported source or over-budget import must fail closed, not truncate successful records.
5. Qt verifies suffix version, source-file hash, record count, original object-kind/index/site/layer/net/exact endpoint coordinate, audit counts, forbidden cross-net electrical evidence, and trailer; only then loads GPU/CPU overlay markers and source Properties. Worker invalid stderr/partial stream/failed re-import clears the previous scene. Invalid data may not be treated as verified.
6. GPU Qt Quick/RHI and CPU fallback share the same parsed evidence, layer/net visibility and source-nm positions. SameLayerContact, ViaBarrel, and potential-conflict markers use distinct visual styles. Overlapping picks prioritize frontmost potential conflict, then barrel, then same-layer. No real-world GPU driver reset is certified.

## Validated scope boundaries

- Source-declared same Net != positive physical contact. No electrical edge for different nets.
- No inference of T-junction/line-interior crossing, trace width or near miss, SMD Pad/Pin, POUROUT, copper flooded islands/thermal, arbitrary layer span, unqualified Via, complete PCB Open/Short.
- The Canonical board geometry and copper `net: None` remain intact; the default V1–V14 layout protocol is never silently reinterpreted.
- Physical SI, 2D RLGC/S-parameters, package discontinuity models, vendor real-export qualification and blanket M1/P2 completion are **NOT_VERIFIED**.

## Validated synthetic native regressions

- Rust unit: deterministic opt-in contact rows; exact source grid; cross-net source conflict is diagnostic only; original A_032_1 negative padstack and 10,000-segment tests.
- Rust binary integration: default stdout is a byte-identical prefix of opt-in stdout on representative V10, V11, V12, V13 and V14 public Copper source fixtures; suffix count/trailer and stderr audit preserved, including previously missed V10–V14 early-return serializer.
- Qt imported P2A smoke: source trace-count unchanged in presence of contact overlay, local markers present; source Net/Layer filtering unaffected.
- Qt native decoder smoke: valid contact, valid zero-contact board, valid potential conflict; hostile SHA mismatch, altered source coordinate, mismatched Net, malformed trailer and invalid conflict claims rejected.
- Outcomes were verified at exact PR HEAD `8d65fff379de6b64d03631ca48aa5d91fc52c11f`, with 23/23 PR checks SUCCESS. Hardware-only exercises that were not run remain expressly NOT_RUN.

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

