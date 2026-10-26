# B_001 — Stackup & Reference overlay (restricted Qt Preview)

**Tracking:** [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) ·
[development checklist](../development-checklist.md) ·
[specification v1.0 §6](../specification/BoardSI_Development_Spec_v1.0.md).

**Base:** `main@b9054a9b1f0499209f438e5a19076d3bcd8536bf` (2026-10-26 KST).

## Scope actually implemented in the B_001 candidate

- A real **Stackup / Reference** tab in `boardsi-preview-p0`, alongside the existing Artwork/Result views.
- After a successful restricted PADS ASCII import, the editor creates one **unqualified, Unknown** row per imported Layer ID. No material, copper type, thickness, role or reference is silently inferred.
- User overlay fields are layer kind, thickness in µm, material, Dk, Df, conductivity, role *hint*, reference Layer ID and free-form property source.
- Local validation rejects missing or non-positive thickness, invalid Dk/Df, missing source, invalid conductivity, invalid layer reference and missing layer kind. Even after fields are valid, output remains **PHYSICAL SI RUN BLOCKED**.
- Export and import **local user overlay JSON**, schema `boardsi-b001-stackup-overlay-v1`, written atomically via `QSaveFile`; reloading requires an exact current source-file SHA-256 match and Layer count/IDs, otherwise rejects before changing visible data.
- Beginning another import, including failed import, clears stale overlay rows. No PCB file is modified or uploaded.
- `--b001-stackup-selftest` exercises the real Qt editor, invalid and unknown fields, save/reload, board change and wrong-source rejection on public synthetic bytes, via three-native-platform Qt CI job.

## Boundaries that remain **not implemented / not qualified**

- This overlay is **not yet bound to the Rust Canonical Board Model** or Solver IPC; it is not physical Stackup/Material readiness and not a signed off reference path. The existing Rust `Property` / `validate_materials_at` model remains the normative future physics data contract.
- PADS importer transports layer IDs and artwork but does not import certified copper thickness, material properties, full Ground/Power net roles, electrical plane connectivity or layer-z stacking.
- The editor's role is a **manual layer-level hint**, **not** a proven Copper Net role. It does not confirm Signal-to-Reference return paths. No full P009, split-plane connectivity, all-Port validation or production RLGC/S computation has been performed.
- The first overlay schema does not contain frequency-dependent dielectric data, material revision/temperature/frequency qualification, scenario-wide mutation/revision, full canonical snapshot serialization or lossless project roundtrip.
- No physical frequency accuracy, formal `T02/T04/T24/T52` acceptance, arbitrary vendor PADS qualification, GPU VRAM/device-loss claim, or actual-product Run is warranted.

## Evidence ledger / result policy

| Check | Expected scope | Observed result |
| --- | --- | --- |
| Qt three-platform native build and new `qt-preview-b001-stackup-reference` CTest | Public synthetic UI/save/load/reimport behavior | **NOT_RUN until exact-head CI verified** |
| Existing importer/Qt RHI/Worker regressions | No breaks introduced | **NOT_RUN until exact-head CI verified** |
| Rust Canonical/Material/Reference integration | End-to-end physical configuration | **NOT_IMPLEMENTED** |
| Formal U09/U10/U39 and T02/T04/T24/T52 | Whole specification | **PARTIAL / NOT_VERIFIED** |
| Independent Codex review | Same SHA / no unresolved threads | **NOT_VERIFIED** |

When CI results become available, add exact commit SHA, run/job URLs and observed PASS/FAIL, without confusing synthetic editor tests with physical acceptance.

## Follow-up gate before Issue #12 can close

A subsequent implementation must couple the Qt editor to the **same Rust Canonical Board Model**, preserve exact provenance plus material validity frequency/source revision, confirm a concrete Reference **Net and Copper region**, and validate source-specific Return Path and scenario/reimport revision. The implementation must then qualify T02/T04/T24/T52 with relevant independent tests. Keep Issue #12 OPEN until that evidence exists.
