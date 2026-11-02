# D_009 — Dual-layer plated Through-Via Pad / single Trace / third Source Area
**Tracking:** [Issue #359](https://github.com/july-rust/BoardSI/issues/359); parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN. First check/evidence date: 2026-11-02 KST.

## Strict opt-in source geometry candidate
- Input is immutable original PADS Layout ASCII V9.4 MILS bytes. One plated Through-Via with explicit declared circular conductive pad on two *different* selected Copper layers; three named rectangular COPCLS owners, A on Layer A and B/C on Layer B, all declared same net, each with no more than one source-owned rectangular COPCUT.
- One positive-width nonzero Straight Trace on Layer B. Exactly one endpoint lies strictly within the Via Pad; the opposite endpoint is strictly inside different Area C and not touching Area B. Whole Via Pad footprints must fit strictly inside A and B, must not touch their cutouts, and must not intersect C. Center-point endpoint candidate does *not* establish full-width trace conductor continuity.
- Reject unsupported source objects, omitted pads, inconsistent layer/net, ambiguous owners, borders/cutouts, stale source SHA-256, forged contact/link/coordinates and out-of-range tick geometry. No inferred Copper fill, Via Span pad manufacturing, or electrical short/open PASS.
- Rust emits bounded D_009-only receipt with source SHA, source object IDs, Via, per-layer Source area/cutout boundaries, Contact states, trace endpoints, Link state and explicit physical/SI exclusion. Existing D_001–D_008 formats and default artwork importer remain unchanged.
- Native Qt independently compares receipt against *actual* current ImportPanel Source SHA-256, imported Via Pad ownership, source Copper polygons/cutouts, Trace and endpoints; markers are explicitly warning candidates and stale markers cleared after reimport.

## Verification ledger
- [ ] Exact PR HEAD: Rust format, Clippy/warnings, Debug/Release, source fixture positive and negative.
- [ ] Linux Qt offscreen CTest including actual D_009 Rust→Qt importer selftest.
- [ ] Native Qt Windows x64, Windows ARM64 and macOS ARM64 D_009 smoke.
- [ ] Required Ruleset #25046595 CI 17/17 and exact-head independent Codex 0 unresolved findings.
- [ ] Protected GitHub Verified one-parent Squash to main.
- [ ] Independent post-main exact SHA Push CI.
- [ ] Issue #359 CLOSED/completed after independent evidence.

Current status is **implementation under test; not accepted** until checks above are independently confirmed.

## Out of scope
```text
electrical_connection = NOT_VERIFIED
return_path_p009      = UNSUPPORTED
physical_si           = NOT_RUN
solver_ready          = false
cache_reuse_eligible  = false
```

No verified manufactured Copper flood, plated barrel electrical continuity, arbitrary Via/Trace topology, global Open/Short, PH-08, SI Solver/Cache, or formal M1 U09/U10/U39 and T02/T04/T24/T52 acceptance. Parent Issue #12 stays OPEN and Draft #35 remains unmerged.