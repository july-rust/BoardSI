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

## Acceptance evidence (must be completed, not assumed)

- [ ] Exact code PR head Rustfmt and Clippy -D warnings, Debug/Release tests incl. C_004 synthetic cases and all old B/C regressions
- [ ] Windows x64, Windows ARM64, macOS ARM64 Rust/Qt Native and Linux Qt Native tests
- [ ] Same-final-HEAD independent Codex review completed, zero unresolved findings/threads
- [ ] Protected Squash Merge, GitHub Verified signature, independent post-main CI
- [ ] Dated original checklist acceptance only for narrow input contract, preserving all earlier check dates

A successful test is limited input-contract acceptance only; it must not be presented as an independent F/X oracle or real PCB lossy RLGC completion.
