# D_005 — Source-only SMD Pad / Straight Trace / Three-Area COPCUT candidate
**Started 2026-11-01 KST. Tracker:** [#345](https://github.com/july-rust/BoardSI/issues/345). **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. **Baseline:** main@e505211172e89a3eaf1f24458cb6a1340bc030a1.

## Bounded hypothesis
This opt-in audit composes the previously distinct D_001 Source Pin/Pad/Trace endpoint witness and D_004 opposite Trace endpoint / D_003 three source COPCLS component candidate. Exactly one straight Trace, exactly three Source-owned rectangular COPCLS on same declared Net/Layer, max one owned internal rectangular COPCUT, supported R/S/RF SMD pad, strict-internal endpoint and complete source/Canonical inventories. Pad and Area must occupy **opposite endpoints** of the **same Trace**. Multiple eligible routes or ambiguous Pad owner are rejected, not quietly selected. No actual filled Copper or physical connectivity is claimed.

Rust invocation: `boardsi-import-artwork <file.asc> --d005-pad-trace-area-v1 RA RB RC NET_A 1 0`. This produces a bounded SHA-256-tied D_005 wire including the D_004 evidence and one LINK verdict. Ordinary importer and canonical connectivity wire are unchanged. Qt opt-in `--d005-pad-trace-area-selftest` re-checks imported one-Trace/three-area D_004 evidence, Source SHA, actual pad footprint interior and candidate link, and rejects stale changes or forged positive/negative link. An apparent positive candidate is shown as a warning marker only.

## Explicit pending acceptance, no preclaims
- [ ] Exact final code HEAD Rust fmt/strict Clippy and Debug/Release tests.
- [ ] Native Qt/actual importer Windows x64, Windows ARM64, macOS ARM64, Linux offscreen D_005 CTest and prior D_001–D_004 regression.
- [ ] All required protected contexts and all completed PR checks/workflows independently verified on exact final code HEAD.
- [ ] Latest independent Codex exact HEAD review completed with zero unresolved findings and review threads.
- [ ] GitHub Verified one-parent Squash and independent exact-merge-SHA post-main Push Workflow and Checks.
- [ ] Separate docs-only reconciliation PR with its own exact-HEAD checks/review/protected merge/post-main; only then close #345.

## Mandatory nonclaims
```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
electrical_connection = NOT_VERIFIED
```
Actual PCB electrical continuity, manufactured poured copper/thermal/relief, Return Current/P009, SI solver/cache, PH-08 and complete U09/U10/U39/T02/T04/T24/T52 **NOT VERIFIED**. M1 #12 stays OPEN, Draft #35 stays unmerged. Synthetic fixture validation alone is not real-PCB qualification.
