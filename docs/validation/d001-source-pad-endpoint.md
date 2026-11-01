# D_001 — Source-bound SMD Pad / Trace endpoint strict-interior witness

**Started:** 2026-11-01 KST. **Tracker:** [Issue #332](https://github.com/july-rust/BoardSI/issues/332). **Parent:** [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) **OPEN**. Baseline main 95e6af79144e3c1da95d870c609daa7be4049110.

## 1. Distinct scope

B_004 [local COPCLS witness](b004-reference-copper-connectivity.md) checks Source Copper Area strict interior for a Trace endpoint or SMD Pad *centre*, not a Trace endpoint against the **full conductive Pad footprint**. B_006 checks Via centre; C_011/C_016 check Reference stitching Via pad/copper risks. D_001 instead adds one opt-in source-declared **Trace endpoint inside R/S/RF conductive SMD Pad geometry**, without creating an electrical graph edge.

Only source-accepted PADS V9.4 MILS and drill-free restricted SMD shapes qualify. Rust separately reparses source bytes into Canonical Track/Pad/Net, requires source-declared Pin/Net, matching trace Net/Layer, positive Trace width and a **strictly interior** endpoint. Source coordinates are exact 0.01 mil integer ticks (=254 nm); circle uses 4*(dx²+dy²) < diameter² and rectangle 2*abs(dx) < width, 2*abs(dy) < height (i128). Tangency, outside, unknown Net and ambiguous multiple pad owners cannot qualify.

An opt-in stdout source-SHA-256-tied record is provided via --d001-pad-endpoint-v1, leaving default Artwork wire versions untouched. Qt opt-in --d001-pad-endpoint-selftest opens the actual source in the Preview Import panel, checks imported Track/Pad/Source SHA and the Rust record independently, exercises a non-physical candidate marker and rejects stale source after mutation. Source selection and physical results remain separate.

## 2. Deliberate unsupported scope

**Do not interpret this witness as made PCB Copper contact.** No manufactured filled copper, thermal/pour, COPCLS/COPCUT membership, full padstack anti-pads, layer-transition, arbitrary Trace body, complex footprints, clearance DRC, electrical shorts/opens, physical Reference return current, P009, RLGC, S, SI solver or cache readiness is claimed. Existing B/C COPCLS/COPCUT and disconnected-island risk validators remain independent and unchanged. A D_001 candidate cannot bypass them. This is an opt-in source-only diagnostic, not yet an automatically enabled general product GUI workflow.

The formal U09/U10/U39/T02/T04/T24/T52 boxes, parent #12, F1/F6/F7 and PH-08 remain unaccepted. Draft #35 is not merged. Hard flags stay unchanged: return_path_p009=UNSUPPORTED; physical_si=NOT_RUN; solver_ready=false; cache_reuse_eligible=false.

## 3. Objective evidence gate

| Gate | Status |
| --- | --- |
| Rust strict geometry / Net / Layer / SHA fixture regressions | PENDING exact HEAD CI |
| Qt original importer → native Source artwork readback / stale handling | PENDING Native CI |
| Windows x64, Windows ARM64, macOS ARM64 and Linux Qt | PENDING |
| 17 mandatory contexts, all complete checks/workflows | PENDING |
| Independent latest exact-HEAD Codex review / zero unresolved threads | PENDING |
| Protected Verified one-parent Squash and independent post-main CI | PENDING |
| Tracker #332 code closeout | OPEN until evidence independently checked |

Do not convert proposed tests, historic CI or docs to PASS. Record first-check KST date, evidence KST date, exact head, completed review receipt, signed merge SHA and post-main links in the tracker/checklist after actual completion.