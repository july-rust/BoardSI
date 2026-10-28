# C_007 — Source-bounded Multi-Segment Reference Coverage and Qt Route Risk

**Tracker:** [Issue #255](https://github.com/july-rust/BoardSI/issues/255)  
**Baseline:** protected main@9acb804068a18fa86b943807ac09bf3b713e82e7  
**Status:** IMPLEMENTED CANDIDATE, acceptance pending exact-head CI/review/signed merge and post-main.

## Changes in scope

- Opt-in source-only V3 route envelope for explicit source-order 2–16 consecutive straight traces; one Signal Net and Source Layer.
- Confirm source endpoint equality, full segment indices, reject repeated indices, self-intersection, unselected same-net branch/crossing and conservative Arc/Via ambiguity. Never infer a connected route from Net naming.
- Wrap existing C_006 independently source-verified V2 per-segment rational gaps. No changes to V1/V2 wire contracts. Outer header binds single source SHA; Qt validates all nested V2 receipts against original ImportedTrack and accepted SHA, with generation/stale-source protection.
- Qt line span, selection/picking and Property explanation use the existing CPU and Qt Quick/RHI C_006 path. Display distances are approximate only.
- Synthetic Rust tests and actual Rust importer→Qt C_007 native smoke added to Windows x64, Windows ARM64, macOS ARM64 workflow and Linux Qt CTest.
- This is limited to forward source-order same-layer straight routes. No reversed routes, via/arc, branch, route auto discovery, general copper continuity or unsourced electrical claims.

## Evidence gates — DO NOT MARK COMPLETE UNTIL ACTUALLY VERIFIED

- [ ] Actual final-code HEAD SHA with Rustfmt, Clippy, Debug/Release and existing B/C tests successful.
- [ ] Windows x64/ARM64, macOS ARM64 and Linux Qt actual C_007 source-route smoke successful.
- [ ] Independent Codex completed on SAME final HEAD, findings 0, unresolved threads 0.
- [ ] Protected Signed Verified squash with Ruleset #25046595 and 17 checks satisfied.
- [ ] Post-main exact merge SHA independent checks and workflow PASS.
- [ ] Dated first checked KST, proof URLs, main SHA in checklist, with previous dates preserved.

**Guard contract:** return_path_p009=UNSUPPORTED, solver_ready=false, physical_si=NOT_RUN, cache_reuse_eligible=false. No actual measured PCB, return current, P009, RLGC, S parameters, physical cache, official U09/U10/U39/T02/T04/T24/T52 or full M1 #12 acceptance. Parent #12 OPEN.

**Gate verdict: NOT_VERIFIED** until all acceptance evidence is independently obtained.
