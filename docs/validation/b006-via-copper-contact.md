# B_006 — Bounded PADS Through Via to COPCLS local contact evidence

Status: **CANDIDATE; NOT ACCEPTED UNTIL EXACT-HEAD CI, CODEX, SIGNED MERGE AND POST-MAIN CHECKS.**

Tracking: [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12).
Baseline: `main@92965d428fca4a3f1782273d86d41a571f8197c9`.
Prior scoped acceptance: [B_003](b003-b001-reference-topology.md), [B_004](b004-reference-copper-connectivity.md), [B_005](b005-reference-geometry.md).
[Living checklist](../development-checklist.md) read before implementation.

## Candidate scope (not yet qualified)

- Fresh reparsing of the exact original PADS V9.4 MILS source bytes, source SHA identity, Route/Via original inventory, Drill/Padstack definitions, and complete accepted Copper geometry.
- Only a single source COPCLS hole-free simple polygon on a layer, exact strict integer source-grid Via **centre** containment, same declared source Net, explicit plated Through Via, and explicitly conductive Padstack on that copper layer.
- Extend canonical graph with `ContactSite::ViaCentre`, `AreaInterior`, `SameLayerContact` and same-source `AreaContinuity`. A `ViaBarrel` extension needs another independently witnessed previously connected via layer. Reject source tampering and disconnected or unsupported shapes.
- Source-geometry local contact only, NOT actual manufactured fill/current/Return Path. Preserve `return_path_p009=UNSUPPORTED`, `solver_ready=false`, `physical_si=NOT_RUN`, `cache_reuse_eligible=false`.
- Existing B_005 original via-contact rejection remains enforced: no fabricated source-order geometry or P009 qualifier.
- Legacy Qt V1 counts and RHI artwork must remain unchanged.

## Explicit unsupported

Blind/buried/microvia, cutouts, thermal, antipads, clearance unverified, split/multiple source Areas per layer, pad-body/trace-width overlaps, arbitrary stacks, inter-plane stitching, physical SI/solver, material loss/roughness, scenario/cache, full U09/U10/U39 or T02/T04/T24/T52 acceptance.

## Required evidence (pending)

- [ ] Source Rustfmt, Clippy, import/model numerical tests; new B_006 synthetic suite
- [ ] B_003/B_004/B_005 and Qt V1 + GPU RHI regression
- [ ] Native CI Windows x64, Windows ARM64, macOS ARM64, Linux
- [ ] Exact final HEAD required checks SUCCESS and independent Codex latest-HEAD review
- [ ] All relevant review threads resolved
- [ ] GitHub protected Verified Squash Merge and separate post-main Push CI success
- [ ] Checklist first-check date (KST), evidence day, PR and SHA recorded after passing gates

**Do not mark B_006 or B_001 full complete before evidence.** Issue #12 remains OPEN, formal U/T items unchecked.
