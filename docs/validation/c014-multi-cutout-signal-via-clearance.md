# C_014 — Multi-COPCUT Signal Via Pad Source geometry V10 (PENDING)

**Issue:** [#276](https://github.com/july-rust/BoardSI/issues/276) · **Parent M1:** [#12](https://github.com/july-rust/BoardSI/issues/12) **OPEN** · **Predecessor:** [C_013](c013-signal-via-reference-cutout-clearance.md) verified limited Source geometry.

**Started:** 2026-10-30 KST. This is a candidate development record, **NOT a verification or product release claim**. The GitHub PR code, Native CI, Codex review, protected merge, and independent post-merge checks must be proven before marking completed.

## V10 scope and format

- PADS V9.4 MILS explicit Source owner, one rectangular Reference Area on each selected layer, 0–8 distinct strictly enclosed disjoint rectangular COPCUTs per area. Different hole counts per layer are permitted.
- Source integer Tick comparisons for circular conductive Signal Via Pad versus hole boundaries, with tangency and overlap risk; all holes must lie strictly inside the Source outer rectangle and cannot touch/overlap another hole.
- Record ordered `LAYER` and `HOLE` rows, source SHA / Canonical Area Source ID / net / layer / Via indices, classification, selected cutout index, and legacy Receipt with line count. Use V9 only if **all** layers have <=1 hole; otherwise byte-identical V8. Legacy V1–V9 opt-ins remain unchanged.
- `GAP_OBSERVED` is Source geometry observation **not** manufactured design-rule clearance. Missing cutout and copper-side Signal Via are never a positive isolation result. Unsupported polygons/arcs/circles, ambiguous source ownership or unvalidated geometry fail closed.
- Qt independently checks original Source / imported Area ID / Cutout exact rectangles and order, recalculates integer geometry, validates embedded legacy receipt, places original-size Signal Via Pad markers on each layer and invalidates on changed input, process generation, selection and source watcher.

## Reproducible tests (candidate, not yet accepted)

- Rust `crates/boardsi-import/tests/c014_signal_via_multi_cutout_v10.rs`: 0/1/2/4/8, layer-asymmetric counts, byte-identical legacy V8/V9, pad tangency/overlap, malformed/ambiguous ownership, nonrectangular/overlapping/outside holes.
- Rust unit geometry: exact one tick, touching and overlapping rectangle boundary.
- Qt: `--c014-multi-cutout-selftest` actual Importer → Qt Preview + CPU Pad rim and stale markers on Windows x64 / ARM64, macOS arm64 native and Linux offscreen.
- fmt, Clippy, Rust Debug / Release, all previous B/C contract tests, native Qt and protected required GitHub CI. **Results must be recorded after execution**; this document does not claim they have passed.

## Acceptance gates

- [ ] Exact code PR final HEAD mandatory GitHub CI and Native OS run complete.
- [ ] Independent same-HEAD Codex completed / zero findings / zero unresolved threads.
- [ ] Signed Verified Squash through existing protected Ruleset; no bypass.
- [ ] Independent code post-main Checks and Push Workflow runs successful.
- [ ] Final dated checklist with initial check date and evidence date updated for the limited verified C_014 scope; validation document protected review and merge/post-main independently verified.
- [ ] Issue #276 closed only after all evidence reconciled. Parent #12 remains OPEN.

## Non-claims

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

No fabricated Copper or thermal/antipad, no P009 return current, no electrical topology, no DRC minimum clearance, RLGC, impedance, S-parameters, solver, or cache. Formal U09/U10/U39/T02/T04/T24/T52 remain unchecked.
