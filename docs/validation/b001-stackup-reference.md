# B_001 — Stackup & Reference overlay (restricted Qt Preview)

**Tracking:** [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) ·
[development checklist](../development-checklist.md) ·
[specification v1.0 §6](../specification/BoardSI_Development_Spec_v1.0.md).

**Base:** `main@b9054a9b1f0499209f438e5a19076d3bcd8536bf` (2026-10-26 KST).

## Scope actually implemented and CI-verified in the B_001 limited UI slice

- A real **Stackup / Reference** tab in `boardsi-preview-p0`, alongside the existing Artwork/Result views.
- After a successful restricted PADS ASCII import, the editor creates one **unqualified, Unknown** row per imported Layer ID. No material, copper type, thickness, role or reference is silently inferred.
- User overlay fields are layer kind, thickness in µm, material, Dk, Df, conductivity, role *hint*, reference Layer ID and free-form property source.
- Local validation rejects missing or non-positive thickness, invalid Dk/Df, missing source, invalid conductivity, invalid layer reference and missing layer kind. Even after fields are valid, output remains **PHYSICAL SI RUN BLOCKED**.
- Export and import **local user overlay JSON**, schema `boardsi-b001-stackup-overlay-v1`, written atomically via `QSaveFile`; reloading requires an exact current source-file SHA-256 match and Layer count/IDs, otherwise rejects before changing visible data.
- Beginning another import, including failed import, clears stale overlay rows. No PCB file is modified or uploaded.
- `--b001-stackup-selftest` exercises the real Qt editor, invalid and unknown fields, save/reload, board change and wrong-source rejection on public synthetic bytes, via three-native-platform Qt CI jobs, plus the Linux offscreen CTest.

## Boundaries that remain **not implemented / not qualified**

- This overlay is **not yet bound to the Rust Canonical Board Model** or Solver IPC; it is not physical Stackup/Material readiness and not a signed off reference path. The existing Rust `Property` / `validate_materials_at` model remains the normative future physics data contract.
- PADS importer transports layer IDs and artwork but does not import certified copper thickness, material properties, full Ground/Power net roles, electrical plane connectivity or layer-z stacking.
- The editor's role is a **manual layer-level hint**, **not** a proven Copper Net role. It does not confirm Signal-to-Reference return paths. No full P009, split-plane connectivity, all-Port validation or production RLGC/S computation has been performed.
- The first overlay schema does not contain frequency-dependent dielectric data, material revision/temperature/frequency qualification, scenario-wide mutation/revision, full canonical snapshot serialization or lossless project roundtrip.
- No physical frequency accuracy, formal `T02/T04/T24/T52` acceptance, arbitrary vendor PADS qualification, GPU VRAM/device-loss claim, or actual-product Run is warranted.

## Evidence ledger / result policy

| Check | Expected scope | Observed result |
| --- | --- | --- |
| Qt Windows x64 | Native B_001 source-safe overlay regression | **PASS**, [job 121590877587](https://github.com/july-rust/BoardSI/actions/runs/40328219147/job/121590877587) |
| Qt Windows ARM64 | Native B_001 source-safe overlay regression | **PASS**, [job 121590877836](https://github.com/july-rust/BoardSI/actions/runs/40328219147/job/121590877836) |
| Qt macOS ARM64 | Native B_001 source-safe overlay regression | **PASS**, [job 121590877261](https://github.com/july-rust/BoardSI/actions/runs/40328219147/job/121590877261) |
| Linux Qt 6.8.3 offscreen | `qt-preview-b001-stackup-reference` and existing CTest | **PASS 11/11**, [job 121590877594](https://github.com/july-rust/BoardSI/actions/runs/40328219147/job/121590877594) |
| Protected-head CI | Exactly 17 required contexts / 26 total checks, including 4 CodeQL Analyze jobs | **17/17 required PASS, 26/26 total SUCCESS** at [candidate `e5f449398c9f22a02878e397d69bb8b34319b252`](https://github.com/july-rust/BoardSI/commit/e5f449398c9f22a02878e397d69bb8b34319b252/checks) |
| Source safety | Source overwrite and alias, oversized JSON / 512-character field, mutated source, wrong board, missing Dk/Df | **PASS** in four Qt OS job receipts; based on generated synthetic local files |
| Rust Canonical/Material/Reference integration | End-to-end physical configuration | **NOT_IMPLEMENTED** |
| Formal U09/U10/U39 and T02/T04/T24/T52 | Whole specification | **PARTIAL / NOT_VERIFIED**; keep unmarked |
| Independent Codex review | Same candidate SHA / no unresolved threads | **PASS** for `e5f449398c9f22a02878e397d69bb8b34319b252`, [PR #213 Codex summary](https://github.com/july-rust/BoardSI/pull/213); older P1 source overwrite / P2 save-versus-load limits fixed, both review threads resolved |

**Evidence date:** 2026-10-26 KST (CI from 2026-10-26 13:12–13:20 UTC = 22:12–22:20 KST). **Checklist documentation/reconfirmation:** 2026-10-27 KST. The 26 successful checks are for the exact code candidate `e5f4493`; documentation changes after this ledger are a **new PR head** and require new exact-head CI and Codex review before protected merge. This ledger is not a certificate of physical PCB SI validation.

### Independent review / corrective history
1. Codex P1: destination equal to PCB original / symlink alias could overwrite source; addressed via path canonicalization and native overwrite/alias regression.
2. Codex P2: serialized overlay could exceed importer bounds; writer now rejects fields > 512 characters and file > 65,536 bytes, with regression for oversized inputs.
3. Native Windows x64 previously failed an overlay selftest using an already open temporary destination; test now uses a fresh JSON path in a temporary directory. Exact `e5f4493` Windows x64 / ARM64 / macOS ARM64 jobs verified PASS. No stale-head green status was reused.

## GitHub protected merge evidence (2026-10-27 KST confirmation)

- [B_001 PR #213](https://github.com/july-rust/BoardSI/pull/213) is **MERGED**, protected signed squash result **[`main@ed9fc66e5b0ab0072fcb93103adc18c2729ae0e3`](https://github.com/july-rust/BoardSI/commit/ed9fc66e5b0ab0072fcb93103adc18c2729ae0e3)**; GitHub commit verification reports `verified=true`, `reason=valid`.
- PR final source `head@a789ff22ad025c83ec6357a81ea2bbe9b3069b32` was checked independently: 26/26 GitHub checks SUCCESS; 17/17 required contexts SUCCESS; all three Native Qt and Linux Qt PASS; independent Codex review for this identical SHA completed with zero findings; both older review threads resolved.
- A merge into `main` has separate new checks. Its final post-merge results must **not** be inferred from the pre-merge SHA. All product/physics gates described above remain unchanged.

## Follow-up gate before Issue #12 can close

A subsequent implementation must couple the Qt editor to the **same Rust Canonical Board Model**, preserve exact provenance plus material validity frequency/source revision, confirm a concrete Reference **Net and Copper region**, and validate source-specific Return Path and scenario/reimport revision. The implementation must then qualify T02/T04/T24/T52 with relevant independent tests. Keep Issue #12 OPEN until that evidence exists.
