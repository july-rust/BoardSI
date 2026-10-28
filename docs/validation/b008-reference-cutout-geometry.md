# B_008 — Source-bound single polygon COPCUT Reference centreline coverage

Status: **SOURCE PR VERIFIED AND MERGED; POST-MAIN CHECKS PENDING** (2026-10-28 KST).
Tracking: [M1 parent Issue #12](https://github.com/july-rust/BoardSI/issues/12).
Baseline: protected `main@43c5d54d5af58d110e98a3260f0e93221c2e4896`.
Original living [development checklist](../development-checklist.md) and original B_005/B_006/B_007 acceptance ledgers read before work.

## Purpose and exact boundary

Extend B_005's source-bound, **source-order** adjacent-Layer and exact straight Trace centreline geometry audit with B_007's strictly owned **one straight polygon COPCUT** geometry. A positive state means only `SOURCE_ORDER_GEOMETRY_ONLY`, NOT reference current, manufactured fill, electrical continuity or physical P009.

The new, opt-in Rust function `reference_geometry::audit_cutout_aware` shares all B_005 guards and explicitly allows either zero or one source-owned polygon cutout. Original `reference_geometry::audit` remains hole-free for backward compatibility.

All of these must hold:
- Exact original PADS V9.4 MILS Source bytes, Source SHA, complete Source Copper inventory, Net IDs and labels, Trace/Arc source identities/coordinates, Source Layer order/kinds; no detached Canonical fabrication.
- Exactly one qualifying Source Reference COPCLS straight polygon with no more than one owned straight polygon COPCUT; both contours must match Canonical BoundarySegments and source frame/ticks exactly, including the internal hole.
- Every selected straight Trace segment has both endpoints strictly inside Reference Copper (not inside/on the hole or outer boundary); the **entire** segment must avoid every outer and cutout boundary edge. This rejects both-endpoints-safe but middle-crosses-hole false positives.
- Same explicitly distinct Signal/Reference Net rules, same-Layer path and exact B_003 graph joins, unselected Trace/Arc branches, source Via touching selected path, no ambiguous overlapping qualifying Source Areas.

## Explicit exclusions

Multiple cutouts, circle/curved contours, unsupported Via branches or cross-Layer signal paths, manufactured fill, reference stitching, area split/thermal/antipad, selected Trace width and pad-body physical overlap, material conductivity/loss/roughness, Solver/Cache/Scenario physics, real RF Return Path / P009 and whole-board Open/Short.

`return_path_p009=UNSUPPORTED`; `solver_ready=false`; `physical_si=NOT_RUN`; `cache_reuse_eligible=false` regardless of geometric positive result.

## Test plan

New public synthetic fixture: `crates/boardsi-import/tests/b008_reference_cutout_geometry.rs`.

- B_005 hole-free API retained, new opt-in API hole-free compatible.
- Single hole away from signal centreline succeeds only at geometry level.
- Straight path endpoints both outside a cutout but segment cuts through it: rejected.
- Tangent/hole edge, hole boundary endpoint, outer boundary and concave outer excursion: rejected.
- One original PADS source tick outside the hole edge remains eligible.
- Wrong Net, wrong Layer/source order, changed original SHA, mutated Canonical hole/Trace, multiple source cutouts: rejected.
- B_003–B_007, Qt legacy V1, CPU/RHI and multi-OS Native matrix regressions required.

## Source PR accepted evidence — 2026-10-28 KST

- [x] [PR #233](https://github.com/july-rust/BoardSI/pull/233) source HEAD `3dc3a681a2259369d3b1396a95f3d988fbad438c`: 23/23 checks and 9/9 workflows SUCCESS.
- [x] B_008 new Rust numerical tests 10/10 PASS; existing B_005/B_006/B_007 regression and Rust fmt/Clippy PASS.
- [x] Native Windows x64/ARM64 and macOS ARM64 Rust/Qt, Linux Qt passed in PR-head Numerical and Qt workflows.
- [x] Same-head independent Codex review completed with zero findings and zero unresolved review threads.
- [x] Signed protected squash merge to `main@9cba34f1eb1135da8e187c8ecb945ad90c78d073` verified=true (valid).
- [ ] Independent post-main 21 checks / 9 workflows still running; do not claim final acceptance before verification.
## Required post-implementation acceptance ledger (to fill ONLY with evidence)

- [ ] Same-head Rust fmt, Clippy, Debug/Release new and existing tests
- [ ] Windows x64/ARM64, macOS ARM64 and existing Linux Qt CI
- [ ] Independent Codex final source HEAD completed review, no unresolved issues
- [ ] Protected branch 17/17 required checks; do not bypass
- [ ] Verified signed Squash merge and separate post-main Push CI
- [ ] KST checklist first-check date/evidence date, PR/head SHA, actual checks
- [ ] Parent Issue #12 stays OPEN; formal U09/U10/U39 and T02/T04/T24/T52 remain unchecked

B_009 will consolidate the bounded B-series implementations and explicitly carry remaining physical/model/official acceptance work forward. Never claim completion of full B_001 merely because this geometry subset passes.
