# B_006 — Bounded PADS Through Via to COPCLS local contact evidence

Status: **B_006 LIMITED SOURCE-GEOMETRY CONTACT IMPLEMENTATION AND ORIGINAL CODE-MERGE POST-MAIN GATES VERIFIED**; documentation acceptance PR tracked separately (last audit 2026-10-28 KST).

Tracking: [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12).
Baseline: `main@92965d428fca4a3f1782273d86d41a571f8197c9`.
Prior scoped acceptance: [B_003](b003-b001-reference-topology.md), [B_004](b004-reference-copper-connectivity.md), [B_005](b005-reference-geometry.md).
[Living checklist](../development-checklist.md) read before implementation.

## Qualified limited scope — not physical reference

- Fresh reparsing of the exact original PADS V9.4 MILS source bytes, source SHA identity, Route/Via original inventory, Drill/Padstack definitions, and complete accepted Copper geometry.
- Only a single source COPCLS hole-free simple polygon on a layer, exact strict integer source-grid Via **centre** containment, same declared source Net, explicit plated Through Via, and explicitly conductive Padstack on that copper layer.
- Extend canonical graph with `ContactSite::ViaCentre`, `AreaInterior`, `SameLayerContact` and same-source `AreaContinuity`. A `ViaBarrel` extension needs another independently witnessed previously connected via layer. Reject source tampering and disconnected or unsupported shapes.
- Source-geometry local contact only, NOT actual manufactured fill/current/Return Path. Preserve `return_path_p009=UNSUPPORTED`, `solver_ready=false`, `physical_si=NOT_RUN`, `cache_reuse_eligible=false`.
- Existing B_005 original via-contact rejection remains enforced: no fabricated source-order geometry or P009 qualifier.
- Legacy Qt V1 counts and RHI artwork must remain unchanged.

## Explicit unsupported

Blind/buried/microvia, cutouts, thermal, antipads, clearance unverified, split/multiple source Areas per layer, pad-body/trace-width overlaps, arbitrary stacks, inter-plane stitching, physical SI/solver, material loss/roughness, scenario/cache, full U09/U10/U39 or T02/T04/T24/T52 acceptance.

## Scoped code acceptance evidence — 2026-10-27 KST

- [x] Final source HEAD `68cd29313d4b3ccfd340321196e286af0ef4ff80` Rustfmt, Clippy, importer/model tests SUCCESS; B_006 11/11 Debug and 11/11 Release public synthetic cases — [Numerical CI #40510123992](https://github.com/july-rust/BoardSI/actions/runs/40510123992)
- [x] B_003/B_004/B_005 + unchanged opt-in Qt V1 and GPU RHI regression in passing numerical and Qt CI — [Qt Native run #40510123737](https://github.com/july-rust/BoardSI/actions/runs/40510123737)
- [x] Rust Native Windows x64/ARM64, macOS ARM64; Qt Native Windows x64/ARM64, macOS ARM64, Linux offscreen SUCCESS — [Rust CI](https://github.com/july-rust/BoardSI/actions/runs/40510123992), [Qt CI](https://github.com/july-rust/BoardSI/actions/runs/40510123737)
- [x] Nine separate PR-head Actions workflows SUCCESS; independent Codex same final HEAD `68cd293` status completed, `has_any_findings=false`, `has_unresolved_findings=false` — [PR #229](https://github.com/july-rust/BoardSI/pull/229)
- [x] Both earlier Codex P2 review threads (source `connected_layers` spoof and duplicate area witness edges) resolved after code fixes and synthetic regression — [PR discussion](https://github.com/july-rust/BoardSI/pull/229)
- [x] GitHub accepted protected **Squash Merge** [PR #229](https://github.com/july-rust/BoardSI/pull/229), `main@d7a22f8230086cdf8aeb326882708dfee48d9ee1` (merged=true). Independent GitHub REST commit signature verification reports **verified=true, reason=valid**, verified at 2026-10-27T13:23:25Z.
- [x] Independently checked original code-merge SHA post-main: [commit Check runs](https://github.com/july-rust/BoardSI/commit/d7a22f8230086cdf8aeb326882708dfee48d9ee1/checks) **21/21 SUCCESS**, **9 post-merge workflow runs SUCCESS** including separate [Push on main #40511614420](https://github.com/july-rust/BoardSI/actions/runs/40511614420) SUCCESS, all from GitHub REST for that exact SHA. **최초 체크일: 2026-10-28 KST**, **근거일: 2026-10-27 KST**; distinct from prior PR-head evidence.
- [x] Scope-only first checklist check date **2026-10-27 KST**, evidence date **2026-10-27 KST**, exact PR/code/main references — [living checklist](../development-checklist.md). Post-main item remains unchecked.

**Only the bounded source-geometry B_006 code functionality is accepted; do not claim full B_001, physical return-current P009 or a qualified SI Solver.** [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN and formal U/T items unchecked. Original code-merge post-main Push CI and signature are now independently verified; documentation PR's own acceptance CI/merge and post-merge checks are tracked separately.
