# E_016 — ODB++ Positive Multi-Island SOURCE qualification

**최초 기록일:** 2026-11-12 KST. **상태: NOT_VERIFIED** until the exact PR HEAD CI, independent review, GitHub Verified protected squash, and exact merged-main Push CI all pass. Parent [#578](https://github.com/july-rust/BoardSI/issues/578) and [#10](https://github.com/july-rust/BoardSI/issues/10) remain OPEN.

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

Current automated unit/integration tests exercise the above supported subset and negative cases; **do not claim these PASSED until exact GitHub CI checks are observed**. Vendor exporter fixture is not available, so independent commercial CAD geometry equivalence is NOT_VERIFIED.

## Final completion gates

- [ ] Final HEAD Rust import and regression CI successful
- [ ] Windows x64/ARM64, macOS ARM64 relevant Native evidence successful
- [ ] Independent reviewer findings addressed; 0 unresolved review threads
- [ ] Protected GitHub Verified, single-parent squash merge
- [ ] Exact final merged-main SHA: independent Push workflows successful
- [ ] Sole checklist and plan consistent with verified scope and dates
- [ ] Parent #578/#10 comment records exact status; parents OPEN

**No staged implementation success equals full ODB++ qualification.** Upon success, only E_016's positive multi-island source subset may be marked bounded COMPLETE, not the parent/whole product.
