# D_004 — three Source COPCLS/COPCUT candidate groups with one Trace endpoint pair
**First record:** 2026-11-01 KST. **Tracker:** [#342](https://github.com/july-rust/BoardSI/issues/342), parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. Baseline `main@4e2e3ec03e20190ba4a3622e42c809357d676dec`.

## Source-only implementation proposal
This opt-in D_004 path reuses the strict D_003 three-area full-source SHA, source/canonical ring identity, one-owned-COPCUT residual candidate edges and component IDs. Exactly one same-layer declared-Net straight Source Trace contributes its two **point** endpoints; strictly retained unique Area interior permits a source attachment candidate. Internal COPCUT, tangent/outer boundaries, ambiguous multiple retained Area owners, different Net/Layer, unsupported arcs/vias, extra Trace inventory and stale source fail closed. A positive geometrical candidate is **not** a manufactured/electrical contact.

Rust `boardsi-import-artwork <file.asc> --d004-endpoint-attachment-v1 RA RB RC NET_REF 2 0` is a separate opt-in wire; ordinary import/Copper electrical edges unchanged. Qt's `--d004-endpoint-attachment-selftest` independently checks D_003's complete Source geometry and selected imported Trace as integer 254 nm ticks, recomputes endpoint owners/groups, rejects forged endpoint states, shows warning-only markers and clears them on changed Source.

## Acceptance (no preclaims)
- [ ] Rust Debug/Release synthetic positive, COPCUT, tangent, disjoint, owner ambiguity, wrong source/net/layer/selection and SHA cases PASS.
- [ ] Native Qt Windows x64/ARM64 and macOS ARM64 actual Rust importer smoke PASS.
- [ ] Linux Qt offscreen CTest actual Rust importer PASS.
- [ ] Exact final PR HEAD all 17 required contexts and actual checks/workflows SUCCESS.
- [ ] Independent same-final-head Codex code review completed zero findings/unresolved threads.
- [ ] Protected GitHub Verified single-parent Squash and independently confirmed exact-merge-SHA post-main checks/push workflows PASS.
- [ ] Separate documentation acceptance and Issue #342 safe closure.

## Nonclaims
```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```
No manufactured filled copper, terminal-level electrical connectivity, physical reference return current, SI solver/cache, PH-08 Gate, full U09/U10/U39 or T02/T04/T24/T52 acceptance. Parent #12 OPEN and Draft #35 unmerged. An unchecked box MUST remain unchecked until the exact evidence is independently observed.
