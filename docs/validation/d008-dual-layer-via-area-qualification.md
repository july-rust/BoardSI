# D_008 — Plated Through-Via Pad to two Source COPCLS layers

**Scope:** explicitly selected **two different** Source Copper layers, each with exactly one
same-Net source-owned rectangular COPCLS and zero or one same-owner rectangular COPCUT;
exactly one plated Through Via with an explicitly declared circular Conductive Pad on both
selected layers. This is a **bounded source geometry candidate only**, not physical or
electrical connectivity.

**Tracking:** [D_008 Issue #356](https://github.com/july-rust/BoardSI/issues/356);
parent [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN;
Draft PR #35 remains untouched.
**First check / evidence date:** 2026-11-02 KST.
**Code implementation:** [PR #357](https://github.com/july-rust/BoardSI/pull/357).

## Verified restricted behavior

- Opt-in Rust CLI and artwork importer `--d008-dual-layer-via-area-v1` use original immutable
  PADS Layout ASCII V9.4 MILS bytes and explicitly selected Area names, Net, Layer IDs and Via
  index. Existing default Import, D_001–D_007 protocols and physical Solver are unchanged.
- The Rust source and Canonical contract requires one plated Through Via and source-defined
  Conductive round Pad entries for **both** selected Layers, matching Drill/Padstack Span,
  Net/Layer/coordinates, Source Area provenance, exact closed rectangle and optional owned
  rectangular COPCUT. No extrapolation from Via Span to a missing inner-layer Pad.
- A complete circular Pad footprint must be strictly internal to each selected, retained
  Copper rectangle. Edge/tangency, cutout overlap, disjoint Pad, missing Pad, wrong Net/Layer,
  unsupported geometry, incomplete import and stale Source evidence are rejected or remain
  `SOURCE_DUAL_LAYER_NOT_VERIFIED`, never silent electrical PASS.
- The receipt independently binds Source SHA-256, Via and Area IDs, Layer, Net, source tick
  geometry, COPCUT and per-layer contact labels.
- Actual Qt imported Artwork is independently checked against the **currently accepted
  PreviewImportPanel SHA**. Stale replay after semantically unchanged artwork is rejected,
  forged receipt/wrong state is rejected, and out-of-domain forged Via coordinates are
  rejected **before multiplication or geometry math**.
- Qt native selftest shows two explicitly source-only markers when qualified and clears
  stale markers on ReImport. All markers retain physical/SI warning semantics.

## Exact implementation code acceptance

- [x] Final [PR #357 HEAD `2ca548bf9844c77f53845420ad7757b2d29543d9`](https://github.com/july-rust/BoardSI/commit/2ca548bf9844c77f53845420ad7757b2d29543d9/checks):
  **27/27 GitHub Check Runs SUCCESS**, **11/11 pull_request Workflow Runs SUCCESS**,
  including Rust format/warnings, numerical and Native Rust Windows x64/ARM64/macOS ARM64,
  Qt Linux and Native Qt Windows x64/ARM64/macOS ARM64; exact-HEAD independent Codex
  `completed/has_any_findings=false/has_unresolved_findings=false`, zero unresolved
  review threads. Earlier two P2 Codex findings (accepted SHA replay and receipt overflow)
  were corrected, reproduced and independently rereviewed on this final SHA.
- [x] Linux Qt **31/31 CTest PASS**, including actual
  `qt-preview-d008-dual-layer-via-area` and
  `D008_NATIVE_QT_DUAL_LAYER_SOURCE true` with all source/replay/forgery/boundary
  detail flags true. Separate three-OS Native Qt CI jobs SUCCESS.
- [x] Protected **GitHub Verified**, one-parent **Squash** code merge
  [`main@f088c5f2e7ef03fd8d5accd5d46b5db1bbf813de`](https://github.com/july-rust/BoardSI/commit/f088c5f2e7ef03fd8d5accd5d46b5db1bbf813de):
  `verification.verified=true`, `reason=valid`, exactly one parent.
- [ ] Post-code-main push checks and workflow runs must be independently evaluated
  on exact `main@f088c5f2e7ef03fd8d5accd5d46b5db1bbf813de`
  before final D_008 Issue closure. Do not substitute PR CI for this gate.
- [ ] This *documentation PR* also requires its own exact HEAD required CI,
  independent Codex review, GitHub Verified protected Squash, and independent
  post-document-main push CI before closing Issue #356.

## Explicitly not accepted

```text
source_dual_layer_contact = SOURCE_GEOMETRY_CANDIDATE (restricted)
electrical_connection     = NOT_VERIFIED
return_path_p009          = UNSUPPORTED
physical_si               = NOT_RUN
solver_ready              = false
cache_reuse_eligible      = false
```

No manufactured copper flood, thermal spokes, electroplating/barrel electrical proof,
multi-Via topology, arbitrary PADS versions, whole-board Open/Short, physical P009
Return Path, PH-08, product Solver/Cache, or formal U09/U10/U39 and
T02/T04/T24/T52 acceptance is claimed. Parent M1 Issue #12 must remain OPEN;
Draft PR #35 remains unchanged.
