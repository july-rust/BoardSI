# D_003 — bounded 3-COPCLS / 1-COPCUT source-geometry candidate graph

**First audit:** 2026-11-01 KST. **Tracker:** [Issue #339](https://github.com/july-rust/BoardSI/issues/339). **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. Baseline protected main `cb9c6e213130cee807edbd3817be691ada7e06e7`.

## New case / no duplicated physical claims

- C_022: 3–6 same-layer, same-net rectangles **without** COPCUT; groups only.
- D_002: exactly two Source-owned COPCLS with zero or one owned rectangular COPCUT.
- **D_003:** exactly three complete Source-owned same-declared-Net/Layer rectangular COPCLS and at most one internally owned rectangular COPCUT, with pairwise retained positive-area candidate edges AB/AC/BC and transitive components. This reuses D_002 exact integer-tick pairwise cells; no float, edge-only or bbox-only false contact.
- Area Source Name/Canonical ID and outer/hole segment parity, original SHA-256, full Copper inventory and unfilled-pour rejection. Explicit bounds on wire/selection, strict errors for unsupported geometry.
- Opt-in Rust command: `boardsi-import-artwork <source.asc> --d003-three-area-copcut-v1 RA RB RC NET_REF 2`. Ordinary importer stdout/wire untouched.
- Qt `--d003-three-area-selftest` independently recomputes 3 pairs from actual imported geometry and ownership, checks each wire edge, component group/overall state, and rejects stale SHA after file change; warning-only risk markers.

### Public synthetic tests
- Rust `crates/boardsi-import/tests/d003_three_area_copcut.rs`: transitive AB/BC and no AC; cutout eliminates AB; three separated/tangent rectangles; wrong Net/Layer, extra area, invalid shapes, more than one hole, source SHA mutation.
- Native Windows x64/ARM64, macOS ARM64 Qt dedicated smoke and Linux CTest run the real Rust executable/Qt importer and confirm markers removed after reimport.
- Actual execution evidence **PENDING**. A checked box, CI pass, independent review, protected merge, post-main verification and issue closure MUST NOT be pre-claimed by this document.

## Mandatory limits
These source-geometry graph edges are only candidates for a visual warning, **not manufactured or electrical conductor connectivity evidence**. Filled copper, thermal relief, padstack, split reference current, electrical Net, P009, PH-08 and production solver remain unverified. The geometric annulus-with-inner-rectangular-hole is connected as ideal geometry; this does not prove manufactured copper.

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

Full U09/U10/U39 and T02/T04/T24/T52 remain unchecked, parent #12 OPEN; Draft #35 unmerged. **No performance/30 GHz or physical correctness qualification.**

## Evidence gate (all pending as of creation)
- [ ] Exact latest code PR HEAD SHA and 17 strict required statuses, full Check Runs and Workflow Runs SUCCESS.
- [ ] Native Qt Win x64, Win ARM64, macOS ARM64 dedicated D_003 steps and Linux offscreen, Rust Debug/Release actual SUCCESS, not skipped.
- [ ] Independent Codex review **on exact latest HEAD** completed, 0 substantive unresolved, review threads 0.
- [ ] Protected signed GitHub Verified one-parent Squash, verified without bypass.
- [ ] Independent exact merged SHA main Push Checks/Workflow SUCCESS.
- [ ] Dated checklist (first checked/evidence dates KST, source links) and Issue #339 safe closure only after above gates.

Historic D_002 acceptance is documented in [D_002 ledger](d002-copper-split-contact-qualification.md) and does not automatically prove new 3-area cases.
