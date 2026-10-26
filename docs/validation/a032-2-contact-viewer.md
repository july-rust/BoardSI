# A_032_2 — bounded local endpoint evidence transport and Qt Artwork overlay

Tracking: [Issue #195](https://github.com/july-rust/BoardSI/issues/195), [PR #196](https://github.com/july-rust/BoardSI/pull/196).
Status: **IMPLEMENTATION CANDIDATE — NOT FULLY ACCEPTED — DO NOT MERGE UNTIL EXACT-HEAD GATES PASS**.

## Shipped in candidate source

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

## Added synthetic native regressions

- Rust unit: deterministic opt-in contact rows; exact source grid; cross-net source conflict is diagnostic only; original A_032_1 negative padstack and 10,000-segment tests.
- Rust binary integration: default stdout is a byte-identical prefix of opt-in stdout on representative V10, V11, V12, V13 and V14 public Copper source fixtures; suffix count/trailer and stderr audit preserved, including previously missed V10–V14 early-return serializer.
- Qt imported P2A smoke: source trace-count unchanged in presence of contact overlay, local markers present; source Net/Layer filtering unaffected.
- Qt native decoder smoke: valid contact, valid zero-contact board, valid potential conflict; hostile SHA mismatch, altered source coordinate, mismatched Net, malformed trailer and invalid conflict claims rejected.
- All outcomes must be verified at the **final exact PR HEAD**, not inherited from an earlier candidate. Unknown results remain `NOT_RUN`.

## Acceptance and remaining gates

- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, full unit/integration suite.
- Linux x64 and Windows x64/ARM64 and macOS ARM64 native Qt/RHI+CPU regressions, including existing A_022–A_030 Copper imports.
- Repeated **optimized release** 10,000-segment Import latency measured against Qt's 10 s deadline: **NOT_VERIFIED until a reproducible result is attached**. Native test passing alone is not performance certification.
- Source-backed GPU/CPU marker visual/picking parity and actual non-software GPU evidence: distinguish tested from `NOT_RUN`.
- Review all P1/P2 comments and explicitly resolve only after confirming source fixes. Exact-final-HEAD independent Codex review with zero unresolved P1/P2, zero open threads.
- All 17 protected required status contexts SUCCESS at exact final HEAD. Keep branch protections and signed squash-only integration. Separately inspect protected-main signature and post-merge required contexts.
- Preserve unrelated Issue #9, #11, #165 OPEN. No proprietary fixture, credentials, or unsupported SI assertion.

**Do not merge PR #196 or close Issue #195 while any mandatory item is FAILED, BLOCKED or NOT_RUN.**
