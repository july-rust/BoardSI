# C_001 — PADS multi-COPCUT and disconnected Copper Island connectivity

**Status:** implementation candidate; independent exact-HEAD CI, Codex and protected main acceptance **PENDING**. 2026-10-28 KST.

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

## Immutable evidence ledger

- [ ] Final exact source HEAD Rust fmt/Clippy/Debug+Release C_001 and B_003–B_008 regressions PASS.
- [ ] Windows x64/ARM64, macOS ARM64 and Linux Qt Native matrix PASS.
- [ ] Independent Codex same-final-HEAD review completed with zero unresolved findings.
- [ ] Protected signed Verified Squash Merge accepted.
- [ ] Independent post-main CI PASS on exact merge SHA.
- [ ] Checklist first-check and evidence dates/PR/SHA/test links updated with preserved historical dates.

Do not mark this gate PASS based on code creation or prior B-series CI. B_007 exact-final-HEAD independent review closure is tracked separately and remains NOT_VERIFIED until proven.
