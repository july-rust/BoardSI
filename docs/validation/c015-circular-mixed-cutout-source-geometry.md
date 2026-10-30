# C_015 — mixed rectangular COPCUT / circular COPCCO Source geometry V11 (candidate; NOT ACCEPTED)

- Issue: [#280](https://github.com/july-rust/BoardSI/issues/280).
- Baseline: protected `main@8701b8b20548c92e4e429472791f2ce2c3abf57b`; C_014 limited acceptance [#276](https://github.com/july-rust/BoardSI/issues/276).
- Started: **2026-10-30 KST**. Full C_015 acceptance: **PENDING**.
- Parent M1: [#12](https://github.com/july-rust/BoardSI/issues/12) **OPEN**.

## Source-exact preliminary geometry

`crates/boardsi-import/src/reference_route_v11/geometry.rs` introduces source-tick integer-only:

- Bounded 0–8 rectangular/circular hole identities; strict outer rectangle containment; no pairwise overlap, nesting or tangency.
- Exact circle–circle and rectangle–circle squared-distance comparisons with `i128`, no Qt display tessellation for classification.
- Circular conductive Via Pad versus circular / rectangular cutout: GAP_OBSERVED, TANGENCY_RISK, COPPER_OVERLAP_RISK, CUTOUT_MISSING_RISK. Missing hole never means a qualified return path.
- Synthetic positive / tangency / overlap / mixed / invalid / boundary tests.

**No V11 protocol, CLI or Qt feature may be described as implemented on the basis of these primitives alone.**

## Critical integration dependency (do not bypass)

Existing V8 embeds V7 → V6 → V5. Existing `reference_route_v7::encode` rejects any non-polygon Source cutout with `V7_CUTOUT_UNSUPPORTED`. `via_copper_contact::augment_multi`, used by V5/V6, likewise rejects non-polygon Source cutouts for the strict reference Via-to-Area contact evidence.

Therefore, **a real COPCCO input cannot currently produce a byte-identical V8 receipt**. Silently dropping circles, replacing them with rectangles, or forging a V8 receipt would invalidate the Source SHA, area-contact proof and safety contract. Production V11 wire/Qt and protected merge must remain blocked until exact legacy dependency is resolved with independent, correctly scoped evidence. Do not modify existing V1–V10 output or hide this gap.

## Required remaining work

- [ ] Design a defensible opt-in source-exact contact/legacy dependency for circular voids without changing V1–V10 outputs on existing inputs.
- [ ] Wire a new V11 encoder, exact Source SHA/Area identity, original hole order and verifiable actual V8 legacy receipt.
- [ ] Implement Qt independent geometry/receipt decoder, CPU/RHI risk marker and stale state; native importer→Qt smoke.
- [ ] Native Windows x64/ARM64/macOS arm64/Linux offscreen and Rust fmt/Clippy/Debug/Release all successful on exact final HEAD.
- [ ] Independent Codex review on exact final HEAD: completed, zero findings/unresolved.
- [ ] Strict Ruleset #25046595 required contexts, signed protected Squash, independent post-main checks/push workflows.
- [ ] Date-limited evidence checklist update, documentary follow-up if needed, close #280 only after all gates.

## Unchanged nonclaims

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

No fabricated copper, DRC, thermal/antipad, physical SI or finished U09/U10/U39/T02/T04/T24/T52 acceptance.
