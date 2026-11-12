# E_016 — ODB++ Positive Multi-Island SOURCE qualification

**최초 기록일:** 2026-11-12 KST. **최종 근거 확인:** 2026-11-13 KST. **상태: COMPLETE — Positive Multi-Island SOURCE 부분 기능 한정.** [PR #608](https://github.com/july-rust/BoardSI/pull/608) exact HEAD CI, final-HEAD independent review, protected Verified Squash and exact merged-main Push all succeeded; evidence independently recorded in [#578 closeout](https://github.com/july-rust/BoardSI/issues/578#issuecomment-6839190698) and [#10 importer gate](https://github.com/july-rust/BoardSI/issues/10#issuecomment-6839191366). #578 was administratively CLOSED by its owner *before* E_016 began (2026-11-12T12:23:49Z); its closed status does NOT imply whole ODB++ acceptance. Product-wide importer issue #10 remains OPEN.

## Scope and exact contracts

- **Positive S only:** one source `S P <dcode>` record with 2–16 disjoint, strictly clockwise straight-edge `OB ... I` contours, each closed with `OE`, entire S with `SE`. Combined 512 vertex and previous exact coordinate bounds remain. Legacy single-island with up to 32 internally disjoint H holes still supported.
- **Fail-closed:** multi-island + any H hole, negative `S N`, OC curved polygon, wrong winding, self-intersection, overlap, boundary touch, nesting, unclosed/malformed/oversized input are rejected, not ignored. The parser is transactional; no partial Canonical model returns after failure.
- **Canonical:** multiple disconnected `CopperArea` objects preserve original S record as first ID and additional deterministic child IDs for remaining islands. Each additional object's ObjectMeta witnesses its own source OB line; all areas bind SourceIdentity file SHA, layer and exact rational-frame coordinates. No fabricated Net or Via.
- **EDA:** one zero-based FID feature ordinal per source S. Opt-in `SNT PLN` design ownership applies to *every* Canonical child area, preserving each FID evidence witness. Actual connectivity graph remains empty; common net name is **not** proof of physical contact.
- **Input parity:** existing Directory/USTAR/ZIP0/ZIP8/GZIP and `features.Z` all use the same normalized source parser; no format-specific Island special case.
- **Out of scope:** Negative copper Boolean subtraction, curved contours, complex multi-island holes, full commercial ODB++ interoperability, complete EDA PKG/PIN, Main Qt ODB Import, plating/barrel, material stackup, electrical Open/Short, SI. EB-01 independent vendor oracle remains unqualified.

## Expected deterministic synthetic fixtures

| Case | Independent expectation |
| --- | --- |
| Two clockwise 4×4 source-unit rectangles at (0,0) and (10,10) | Exactly two canonical 4-segment polygons, each area 16 source-unit², no touching segments or electrical edges |
| Three separated islands | Exactly three distinct area IDs, one Source FID |
| One legacy island with strict internal hole | Exactly one area and one Cutout, original first area ID |
| EDA PLN/FID C 0 3 | Both areas share declared design Net identity and independent provenance witnesses; graph still empty |
| Intersect/touch/contain/reverse winding | Explicit reject; never silently merge or bridge |
| Multiple islands plus holes | Explicit unsupported; do not generalize unsupported topology |
| S N negative or OC arc | Explicit unsupported |
| Same bytes imported twice | Deterministic Canonical source-based identities |

The bounded positive/negative synthetic regression suites and independent exact-integer rectangular-geometry assertions passed on PR #608 final HEAD `ba26bde470b61e3d0a72e6c5093d204458a2375b`: **30/30 Checks SUCCESS**, including actual Windows x64/ARM64 and macOS ARM64 Rust/Qt Native jobs. This is **not** independent commercial Vendor Export/CAM cross-CAD verification; that remains NOT_VERIFIED (EB-01).

## Final completion gates

- [x] Final PR HEAD `ba26bde470b61e3d0a72e6c5093d204458a2375b`: **30/30 Checks SUCCESS**, exact Rust import and regression CI
- [x] Windows x64/ARM64, macOS ARM64 relevant Native Rust/Qt successful (PR plus exact main Push)
- [x] Codex independent final HEAD review **0 findings / 0 unresolved**, GraphQL threads 0
- [x] Protected GitHub **Verified single-parent Squash** [`ff6fecb0af3b4e37f7a08859342286c4d85a3656`](https://github.com/july-rust/BoardSI/commit/ff6fecb0af3b4e37f7a08859342286c4d85a3656); PR #608 merged/closed
- [x] Exact merged code-main `ff6fecb0af3b4e37f7a08859342286c4d85a3656`: **8/8 event=push Workflow SUCCESS** ([Qt](https://github.com/july-rust/BoardSI/actions/runs/43225216787), [Numerical](https://github.com/july-rust/BoardSI/actions/runs/43225216846), [Docs](https://github.com/july-rust/BoardSI/actions/runs/43225217018), [PH-06](https://github.com/july-rust/BoardSI/actions/runs/43225217173), [PH-07](https://github.com/july-rust/BoardSI/actions/runs/43225216716), [Native FEM](https://github.com/july-rust/BoardSI/actions/runs/43225216768), [Lossless FEM](https://github.com/july-rust/BoardSI/actions/runs/43225216828), [G0](https://github.com/july-rust/BoardSI/actions/runs/43225217101))
- [x] Sole checklist and plan reconciled with original scope/dates; this docs-only evidence closeout does not claim new Native execution
- [x] Exact code evidence recorded on [historically closed #578](https://github.com/july-rust/BoardSI/issues/578#issuecomment-6839190698) and [open #10](https://github.com/july-rust/BoardSI/issues/10#issuecomment-6839191366); whole-format acceptance unverified

**Completion scope:** Only the E_016 positive multi-island Source subset is COMPLETE. It is not full ODB++ product, Vendor Oracle, manufactured plating, inter-layer connectivity, main Qt ODB Import, or physical SI acceptance. Historic failed/cancelled earlier attempts remain historic and are not relabelled PASS. The E_016 implementation PR is merged and closed; the independent code-main Push is complete. Any docs-only closeout PR must independently satisfy its own protected CI and exact docs-main Push before claiming that separate administrative action complete.
