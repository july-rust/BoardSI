# D_001 — Source-bound SMD Pad / Trace endpoint strict-interior witness

**Started:** 2026-11-01 KST. **Tracker:** [Issue #332](https://github.com/july-rust/BoardSI/issues/332). **Parent:** [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) **OPEN**. Baseline main 95e6af79144e3c1da95d870c609daa7be4049110.

## 1. Distinct scope

B_004 [local COPCLS witness](b004-reference-copper-connectivity.md) checks Source Copper Area strict interior for a Trace endpoint or SMD Pad *centre*, not a Trace endpoint against the **full conductive Pad footprint**. B_006 checks Via centre; C_011/C_016 check Reference stitching Via pad/copper risks. D_001 instead adds one opt-in source-declared **Trace endpoint inside R/S/RF conductive SMD Pad geometry**, without creating an electrical graph edge.

Only source-accepted PADS V9.4 MILS and drill-free restricted SMD shapes qualify. Rust separately reparses source bytes into Canonical Track/Pad/Net, requires source-declared Pin/Net, matching trace Net/Layer, positive Trace width and a **strictly interior** endpoint. Source coordinates are exact 0.01 mil integer ticks (=254 nm); circle uses 4*(dx²+dy²) < diameter² and rectangle 2*abs(dx) < width, 2*abs(dy) < height (i128). Tangency, outside, unknown Net and ambiguous multiple pad owners cannot qualify.

An opt-in stdout source-SHA-256-tied record is provided via --d001-pad-endpoint-v1, leaving default Artwork wire versions untouched. Qt opt-in --d001-pad-endpoint-selftest opens the actual source in the Preview Import panel, checks imported Track/Pad/Source SHA and the Rust record independently, exercises a non-physical candidate marker and rejects stale source after mutation. Source selection and physical results remain separate.

## 2. Deliberate unsupported scope

**Do not interpret this witness as made PCB Copper contact.** No manufactured filled copper, thermal/pour, COPCLS/COPCUT membership, full padstack anti-pads, layer-transition, arbitrary Trace body, complex footprints, clearance DRC, electrical shorts/opens, physical Reference return current, P009, RLGC, S, SI solver or cache readiness is claimed. Existing B/C COPCLS/COPCUT and disconnected-island risk validators remain independent and unchanged. A D_001 candidate cannot bypass them. This is an opt-in source-only diagnostic, not yet an automatically enabled general product GUI workflow.

The formal U09/U10/U39/T02/T04/T24/T52 boxes, parent #12, F1/F6/F7 and PH-08 remain unaccepted. Draft #35 is not merged. Hard flags stay unchanged: return_path_p009=UNSUPPORTED; physical_si=NOT_RUN; solver_ready=false; cache_reuse_eligible=false.

## 3. Verified limited code integration — 2026-11-01 KST

The source-only **D_001 code PR** [#333](https://github.com/july-rust/BoardSI/pull/333) was independently accepted at the exact final HEAD `c6136c18fbf6b64af77d8a0426c8d3f9fc06e02e` on 2026-11-01 KST. The following facts are specific to the bounded import/Qt candidate, **not formal physical or full M1 U/T acceptance**.

| Criterion | Independently observed result |
| --- | --- |
| Full PR HEAD checks | **27/27 Check Runs SUCCESS** — [exact HEAD](https://github.com/july-rust/BoardSI/commit/c6136c18fbf6b64af77d8a0426c8d3f9fc06e02e/checks) |
| PR Workflow Runs | **11/11 event=pull_request workflows SUCCESS**, including source Native Rust/Qt Windows x64/ARM64, macOS ARM64 and Linux offscreen |
| Codex independent review | **Same HEAD** `status=completed, has_any_findings=false, has_unresolved_findings=false`; original finding on skipped Pad inventory [fixed](https://github.com/july-rust/BoardSI/pull/333) by rejecting all skipped pad records, 1 thread resolved / 0 unresolved |
| Protected integration | [Verified one-parent Squash main@8314e52cabc76b070e6b793ac962120cca2cd093](https://github.com/july-rust/BoardSI/commit/8314e52cabc76b070e6b793ac962120cca2cd093), `verification.verified=true`, `reason=valid`, sole parent `95e6af79144e3c1da95d870c609daa7be4049110` |
| Post-main independent checks | Exact merge SHA **21/21 Check Runs SUCCESS** and **8/8 event=push workflows SUCCESS**, [merge checks](https://github.com/july-rust/BoardSI/commit/8314e52cabc76b070e6b793ac962120cca2cd093/checks) |
| Source integrity and fail-closed safety | Verified by code PR source unit/Native Qt regression: strict integer-grid round Pad off-centre/boundary, Layer/Net negative, source SHA-256 deterministic records, truncated 5,000-pad inventory rejection, Qt actual import/readback/source switch stale rejection |
| First verified code acceptance | **2026-11-01 KST**; objective evidence date **2026-11-01 KST** |

The original code PR specifically contains synthetic round-Pad positive and boundary tests. It implements square and rotated RF rectangle support through the same exact integer-grid predicate, but **the supplemental explicit positive/boundary test cases** are added only in the subsequent acceptance reconciliation PR. Never retroactively claim they were executed in PR #333.

## 4. Supplemental qualification and closeout gate

A separate protected follow-up PR adds **independent square SMD, rotated rectangular RF, boundary and simultaneous overlapping-owner synthetic cases** alongside the dated checklist update. Treat its new cases as **PENDING / NOT_RUN until its own exact-HEAD native Rust/Qt CI executes**.

- [ ] Verify supplemental exact final HEAD CI and all mandatory contexts.
- [ ] Independent latest-exact-HEAD Codex review with zero unresolved findings/threads.
- [ ] GitHub Verified one-parent Squash and independent post-main checks/push workflows.
- [ ] Close only D_001 tracker [#332](https://github.com/july-rust/BoardSI/issues/332) when the supplemental PR and its dated evidence ledger/checklist are accepted; **do not close M1 parent #12**.

D_001 remains a restricted **geometry candidate**; `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`. Other M0/M1/M2 scope is not promoted by current CI.
