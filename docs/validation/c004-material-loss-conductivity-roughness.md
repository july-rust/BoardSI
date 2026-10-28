# C_004 — Bounded Canonical Material Loss, Conductivity and Roughness Input Contract

Date: 2026-10-28 KST. Tracker: [Issue #245](https://github.com/july-rust/BoardSI/issues/245).
Predecessor: [C_003](c003-immutable-scenario-stale-result.md), [B_009 handoff](b009-b-series-final-acceptance.md).
Architecture: [M2 physical-model ADR](../architecture/m2-rlgc-physical-model-adr.md).

## Supported input contract (opt-in; not a solver)

The implementation in boardsi-import::material_contract::audit_material accepts a real Canonical Layer and Material, bound through their complete ObjectId/source identity. It validates user-Assumed materials only. Caller-supplied source identity and underlying material measurement claims remain unauthenticated. It does not invent a Confirmed property.

- Copper layer: positive finite conductivity in S/m, distinct from dielectric Dk/Df; reciprocal resistivity in Ω·m is a dimensional diagnostic, not DC field resistance.
- Dielectric/mask layer: finite positive Dk, nonnegative finite Df, Df=0 only with an explicit ideal-lossless input. No implicit FR-4 or copper default.
- Mandatory provenance on every active property and valid band: document/revision/method, reference frequency (within recorded band), optional finite temperature. Requested frequency must also lie within the band.
- Optional relative complex permittivity is checked at the same frequency against Dk/Df: under exp(+jωt), passive εr*=εr′−jεr″ and εr″>=0. Exp(−jωt) source may supply a positive imaginary part, converted with explicit sign; unknown sign or inconsistent passive sign is rejected. The source convention remains included in the revision.
- Separate bulk-conduction already included in complex ε is rejected by this limited input path, which cannot safely split AC polarization and DC leakage.
- Roughness: Unknown, explicitly sourced ideal smooth, or sourced measured Rq/Ra/Rz with bounded model identifiers and validity range. These metrics are not interchangeable. A measured profile is metadata only, no Hammerstad/Huray physics or correction multiplier is applied.
- Revision SHA-256 is domain-separated and length-prefixed over Canonical identity and all accepted material/evidence/model fields. The separate audit revision also covers frequency; changes invalidate past audit receipts. Identical validated contents recover identical revisions. It is a synthetic identity, not authenticated material physics or C_003 physical cache permission.

Errors distinguish missing evidence, invalid material, out-of-band requests and unsupported/conflicting physical loss conventions. No result S, RLGC or field solution is calculated.

## Explicitly NOT qualified

- Physical Return Path P009, real PCB connectivity/ports, validated loss models, AC magnetic skin/proximity, conducting DC resistance, real dispersive Dk/Df and roughness loss, 30 GHz performance and product qualification.
- This module is opt-in Rust and does not change Qt viewer, Worker IPC, physical solver or C_003 sealed scenario/cache behavior.
- solver_ready=false, cache_reuse_eligible=false, physical_si=NOT_RUN, return_path_p009=UNSUPPORTED.
- PH-12–PH-16 and U09/U10/U39/T02/T04/T24/T52 remain Not Run / unchecked, B_001 parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN.
- C_002 Qt risk markers and B_007 original final-HEAD Codex review remain separate incomplete items.

## Acceptance evidence — 2026-10-28 KST

- [x] Code [PR #246](https://github.com/july-rust/BoardSI/pull/246), exact final PR HEAD [3c17b723b5d2508e71808d186c78869ece56ee06](https://github.com/july-rust/BoardSI/commit/3c17b723b5d2508e71808d186c78869ece56ee06/checks): **23/23 Check Runs SUCCESS**, 9/9 PR Workflow Runs SUCCESS. Rustfmt, Clippy -D warnings, Debug/Release and existing B/C regression gates succeeded, including 10 scoped synthetic C_004 regression tests.
- [x] Windows x64/ARM64 and macOS ARM64 Rust + Qt Native, Linux Qt Native, and related cross-platform regressions: exact PR HEAD CI SUCCESS.
- [x] Independent Codex exact final HEAD 3c17b723b5: completed, has_any_findings=false, has_unresolved_findings=false, all historical inline threads resolved (0 open). Two prior real/imag loss consistency findings were fixed and independently re-reviewed.
- [x] Protected Squash Merge [main@02cc35b51dfe4263f3e3d3f02f9151d47d696849](https://github.com/july-rust/BoardSI/commit/02cc35b51dfe4263f3e3d3f02f9151d47d696849), verified=true, reason=valid (GitHub REST commit verification).
- [ ] Independent post-main 21 Check Runs and 9 Workflow Runs SUCCESS — **pending**, no claim before exact merged SHA is verified.
- [ ] Final dated living checklist acceptance and protected documentation PR closeout — pending post-main evidence.

This is a limited input-contract implementation only, not an independent F/X oracle nor verified real-PCB lossy RLGC. **Issue #12 OPEN**, PH-12–PH-16/physical loss/P009/Cache still blocked. C_002 Qt risk overlay and B_007 original exact-head review stay pending.
