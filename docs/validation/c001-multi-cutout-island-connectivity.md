# C_001 — PADS multi-COPCUT and disconnected Copper Island connectivity

**Status:** C_001 limited Source Geometry and local connectivity CODE ACCEPTED on protected signed main, with independent post-main CI. Not full M1/physical acceptance. 2026-10-28 KST.

Tracking: [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12), [B_009 C-stage handoff](b009-b-series-final-acceptance.md), [development checklist](../development-checklist.md).

## Scoped engineering evidence

- C_001 opt-in `copper_contact::augment_source_bound_multi` supports at most eight **straight polygonal source-owned COPCUT** contours per simple Source COPCLS. Exact PADS V9.4 MILS original bytes, source SHA, Source Area/Net/Pad/Via/Layer inventory and ordered Canonical rings are independently compared.
- Canonical integer model validates closed, simple, bounded polygons; complete hole-in-outer containment; pairwise hole intersections/nesting; void interior **and boundary** exclusion. Existing hole-free B_004 and single-hole B_007 entrypoints retain their restricted acceptance scopes.
- Multiple independent Source Area objects on one Layer are not joined by their shared Net label. Exact endpoint/pad/via centre joins remain tied to the same source-owned Area and verified Layer/Net. Overlapping or touching same-Layer outer bounding boxes are conservatively Unsupported, including cases that might actually be disjoint complex polygons.
- `via_copper_contact::augment_multi` maintains source-exact plated Through Via centre contact with bounded geometry. Existing legacy Via API retains its original limits.
- The Qt artwork import binary selects the opt-in multi-COPCUT paths without changing V1 IPC, CPU/RHI data contracts or B_005/B_008 Reference Geometry audit.
- Public synthetic `c001_multi_cutout_islands.rs` covers valid multiple holes, boundary/interior refusal, separated same-Net Area IDs, overlap refusal, changed original SHA / altered Canonical hole, wrong Net, nested holes, forged Area witnesses and plated Via candidates.

**Explicit exclusions:** manufactured fill, thermal/antipad, Trace width or Pad body overlap, circle/curved cutout electrical contact, overlapping source areas, board-wide Open/Short, real P009 return current, M2 RLGC/S, loss/roughness and physical solver cache.

Safety invariants remain `return_path_p009=UNSUPPORTED`, `solver_ready=false`, `physical_si=NOT_RUN`, `cache_reuse_eligible=false`. Parent Issue #12 remains OPEN, and U09/U10/U39/T02/T04/T24/T52 full acceptance is NOT_VERIFIED.

## Immutable evidence ledger — verified on 2026-10-28 KST

- [x] Exact final [PR #237](https://github.com/july-rust/BoardSI/pull/237) Source HEAD `1db862985505427f8f3fcd58b6ae6c9530cbac6a`: **28/28 GitHub Check Runs SUCCESS**. Numerical workflow [#40628842300](https://github.com/july-rust/BoardSI/actions/runs/40628842300) Rust fmt/Clippy and C_001 Debug **10/10**, Release **10/10** PASS, B_003–B_008 regressions PASS.
- [x] Windows x64, Windows ARM64, macOS ARM64, Linux Qt Native PR checks PASS at the same HEAD, Qt IPC and artwork regressions.
- [x] Same-head independent Codex review receipt `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; a previous fixture P2 review comment was fixed and its sole stale thread explicitly resolved, zero unresolved threads.
- [x] Protected [Squash Merge `main@df1fde8648fd48f9ee884a3d9ad68fdf266e5f35`](https://github.com/july-rust/BoardSI/commit/df1fde8648fd48f9ee884a3d9ad68fdf266e5f35) GitHub `verified=true`, `reason=valid`. No bypass, `main` protected.
- [x] Independent post-main **21/21 Check Runs SUCCESS**, **9/9 Workflow Runs SUCCESS** at exact merged SHA. [Push on main #40629876739](https://github.com/july-rust/BoardSI/actions/runs/40629876739), [Qt Runtime #40629877037](https://github.com/july-rust/BoardSI/actions/runs/40629877037), [commit checks](https://github.com/july-rust/BoardSI/commit/df1fde8648fd48f9ee884a3d9ad68fdf266e5f35/checks).
- [x] Scoped checklist first check: **2026-10-28 KST**; evidence date **2026-10-28 KST**. Historical first dates preserved.

## C_001 documentary closeout independently verified (2026-10-28 KST)

- [x] [Final checklist / ledger PR #238](https://github.com/july-rust/BoardSI/pull/238), exact docs source HEAD `2f788cf771dae55ca0e78d57c7c0ef8752d2543e`: **22/22 Check Runs SUCCESS**, same-head Codex `completed`, findings 0 and unresolved 0.
- [x] Protected Verified signed squash `main@1167b0d531a685705900117f6097217f6a8e2312`; independent **21/21 post-main Checks SUCCESS and 9/9 Workflows SUCCESS** ([exact main checks](https://github.com/july-rust/BoardSI/commit/1167b0d531a685705900117f6097217f6a8e2312/checks)).
- [x] C_001 scope completed and dated in living checklist. This final heading clarification does not expand the limited physical acceptance or close parent Issue #12.

**Not accepted:** [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN; B_007 independent exact-final-HEAD Codex receipt unresolved; true electrical Reference and P009 return current, physical SI solver/cache, and full U09/U10/U39/T02/T04/T24/T52 all remain unchecked. No claim of whole-board copper islands beyond bounded disjoint outer bounding boxes.

