# M2 Canonical Board Model → physical Solver adapter — design preflight (NOT G5 implementation)

**Date:** 2026-10-15 (KST)  
**Status:** **DESIGN REVIEW / PENDING**. No native input parsing, Port extraction, runtime Solver call, or product physical validation was implemented.  
**Baseline:** `main@ec5fa4b048c0c00d8588dfd90f2c35e1107e220b` (PR #33 G0 merged); [G1 isolated FEM PR #34](https://github.com/july-rust/BoardSI/pull/34) remains **open, not merged**.  
**Authoritative contracts:** [BoardSI development specification v1.0](../specification/BoardSI_Development_Spec_v1.0.md) §§4,4.6,6,8,11,13–15; [ADR-M2-001](m2-rlgc-physical-model-adr.md); [M2 verification plan](../validation/m2-rlgc-independent-verification-plan.md); [M0 Canonical contract](canonical-board-model.md).

## 1. Gate identity / dependency clarification

Do **not** rename gates to make progress appear faster:
- **G0** is source-locked independent ideal PEC coupon reference (within its *finite grounded-box* scope); now incorporated in main.
- **G1** is the independent initial ideal microstrip physical prototype; the [#34 Rust FEM report](https://github.com/july-rust/BoardSI/blob/feature/m2-lossless-fem-prototype/docs/validation/m2-g1-lossless-fem-evidence.md) confirms one constrained coupon and partial PH cases. It **does not** certify a real copper/PCB geometry.
- **G2** in the merged PH plan is *additional single-ended geometry qualification*, namely PH-05/06/07/08 stripline, multilayer, mask, finite ground; **not** Canonical Model integration.
- **G3** loss physics; **G4** coupled/multiconductor; **G5** **Canonical Board Model → geometry/Port → solver → Worker/Qt/output integration**. This document is a **G5 interface preflight** that may be reviewed while G2–G4 remain pending. It does not authorize bypassing gates or feeding lossy real-board paths into an ideal prototype.

**Decision:** G5 adapter design can be drafted now; a product Solver call from arbitrary imported geometry remains **NO-GO**. User's requested "G2 Canonical connection" means **early integration feasibility review**, while the official G2 label remains unchanged.

## 2. Current, verified source contract and gaps

`crates/boardsi-model/src/lib.rs` is a dependency-free format-neutral M0 data contract; **not** an importer, 2D geometry/mesh extractor, or physical readiness oracle:

| Existing typed data | Can supply | Must NOT silently infer |
| --- | --- | --- |
| `BoardModel`, `ImportSource`, `ObjectMeta`, immutable model IDs | Provenance, board kind (PCB/substrate), cross-format identity | That a parsed/validated model is electromagnetically suitable |
| `CoordinateFrame`, `Point`, `Rational`, `solver_m()` | Precision-checked SI x/y cross-section basis | Rounding, unsupported arbitrary rotation, lost geometry/coordinates |
| `Trace` (`centreline`, `width_m`, `layer`, `net`) | Candidate straight segment + signal identity | Guaranteed finite copper cross-section, exact corners or longitudinal uniformity |
| `Stackup.layers`: `Layer.kind`, `order`, `z_m`, `thickness_m`, `material` | Layer sequence, z position and thickness | Automatic dielectric fill, copper shape profile or finite plane height |
| `Material.dk/df/conductivity_s_per_m/valid_band` as `Property<T>` | Explicit Missing/Assumed/Confirmed and source/band | Treating **real Dk/Df/σ** as **PEC and zero loss** or inferring absent mask |
| `Net.role`, `CopperArea.outer/cutouts`, `ConnectivityGraph` | Ground/return candidates and declared connectivity evidence | Continuous physical reference return from net name, polygon label or connectivity alone |
| `Via/Pad/Arc/Cutout` | Unsupported geometry markers needing independent treatment | Via/pad capacitance, 3D discontinuity, plane-slot return, branch, bend |

M0 `validated_view()` indicates no *model-integrity* Error; Missing/Assumed properties remain. It is **not** a physical SI Run approval. `validate_materials_at(f)` reports material completeness/band, not frequency-valid extraction. The historical `boardsi-numerics` accepts already-supplied scalar RLGC and was mathematically validated; it cannot certify geometry→C/C0.

## 3. Proposed typed adapter: no modifications to existing M0 types yet

In a **future separate crate**, e.g. `boardsi-geometry-extract`, define a new versioned, immutable intermediate contract. Do not mutate source `BoardModel`, reinterpret ODB++ as internal format, or alter existing `boardsi-numerics` scalar signatures.

Proposed data shape (interface proposal only, not compiled API):

```rust
struct SectionSelection {
    board_revision_hash: [u8; 32],
    signal_trace_id: ObjectId,
    reference_conductor_ids: Vec<ObjectId>,
    start_m: [f64; 2],
    end_m: [f64; 2],
    scenario_id: String,
}
enum Eligibility {
    VerifiedSyntheticPECOnly,
    Unsupported { reasons: Vec<String> },
    PendingGeometryEvidence { missing: Vec<String> },
}
struct CrossSectionProvenance {
    source_ids: Vec<ObjectId>,
    material_sources: Vec<PropertySource>,
    coordinate_error_bounds_m: Vec<f64>,
    solver_assumption_id: String,
    input_revision_hash: [u8; 32],
}
// A future CrossSection has explicit conductor polygons, dielectric/air/mask
// regions, reference topology proof, near/far Port order, domain boundary,
// SI units, loss model and excitation. No inferred PEC/zero loss.
```

`ObjectId` and metadata must be referenced from the **same** `Arc<BoardModel>` snapshot; no cloned/mutating BoardModel or UI-only simplified geometry. Canonical board z increases **top→bottom**, while isolated FEM uses a local upward z from its declared PEC reference plane. An explicit traceable coordinate transform with unit precision checks is mandatory; never use layer order as physical z. Source-geometry IDs and Scenario overlay/version must participate in cache keys.

### Eligibility steps before allowing even an **ideal** field extraction

1. Validate BoardModel structural integrity and provenance *without* promoting `Assumed` or `Missing` to Confirmed.
2. Require an unambiguous straight, uniform segment and full *conductor surface* cross-section (actual thickness/profile). Reject bends, pads, vias, neckdowns, branches, reference changes, partial geometry and unknown mask/air.
3. Identify a distinct, physically continuous reference PEC surface with positive gap; `NetRole::Ground` **alone is insufficient**. Reject a plane cutout crossing the return, floating/ambiguous reference, and missing return topology verification.
4. Require all dielectric regions to have explicitly specified epsilon, boundary type, frequency/temperature applicability and loss behavior; do not convert a real-board Df or conductivity to zero. Explicit synthetic PEC/lossless inputs may be accepted only with synthetic-provenance flag and matching bounded G1 coupon topology.
5. Construct local x/z conductor + layered material polygons and near/far physical ports with independent reference and wave-normalization metadata; preserve exact coordinate/error and material provenance.
6. Use **a new typed, gate-checked interface** to the field solver. Return `Unsupported`/Pending with reasons and **no synthesized S**, rather than silent default or fake 30 GHz Validated. Worker IPC and Qt must remain untouched until G5 design is independently reviewed.

For ordinary imported PCB, steps 2–4 **cannot yet be certified** by the current M0 prototype. Therefore an implementation that merely reads `Trace.width_m`, `Layer.thickness_m` and a ground Net and calls G1 with `R=G=0` would be **incorrect and prohibited**.

## 4. Design-only tests for later PR — do not claim executed

| ID | Future synthetic test | Expected status |
| --- | --- | --- |
| AD-01 | Identical geometry from two independently normalized input formats + same Scenario | Equal exact physical cross-section input/provenance; **Not Run** (no importers) |
| AD-02 | Coordinate transform, x/y-to-cross-section orientation and top-down z → upward local z | Numerically stable SI units & no inverted layers; **Not Run** |
| AD-03 | Straight, explicitly synthetic PEC coupon + continuous reference + declared 0 V enclosure | Can produce exactly the approved G1 ideal input; **Not Run** |
| AD-04 | Real copper conductivity or nonzero/missing dielectric loss parameter | Return Unsupported, never zero-loss full S; **Not Run** |
| AD-05 | Missing/Assumed thickness, Dk, mask, top-wall boundary, undefined reference | No default fill, explicit diagnostics; **Not Run** |
| AD-06 | Split reference plane/cutout, via/pad transition, bend, coupled lines, floating nets | No scalar ideal-coupon run and no fabricated two-port S; **Not Run** |
| AD-07 | Same board Revision with changed Scenario width/stackup and re-run | Immutable base, solver geometry changes, cache invalidates; **Not Run** |
| AD-08 | Confidential board file / unknown sample rights / unsupported importer format | No public GitHub artifacts or network upload; **Not Run** |

These are **adapter review identifiers**, not invented PASS values and not replacements for specification T01–T54 or physical PH IDs.

## 5. Execution milestones and approval criteria

**Preflight review now (documentation only):** Agree naming, canonical provenance, section detection, reference continuity evidence, geometry eligibility, Unsupported contract, cache/Scenario invalidation, error categories, mandatory test fixtures and minimal dependency strategy.

**Later (not yet approved G5 integration):**
- A. Qualify geometry/port projection using synthetic `BoardModel` fixtures with explicit continuous return, cross-section/z transform, error/status propagation and independent geometry checks. No real-PCB results.
- B. Expand and qualify **formal G2** structures PH-05–08, with new F references and mesh/exterior tests; no broad physical applicability asserted from one G1 box.
- C. Qualify **G3** conductor/dielectric loss independently and **G4** multiconductor matrices/ports before claiming lossy or coupled real-board paths. Stop rather than assume lossless.
- D. **G5** versioned typed adapter integration, then existing mathematical M0 contract/Worker/Qt product verification under specification T-gates and independent physical references.
- E. Source rights and offline-only handling of private PCB remain mandatory at every step.

**Decision ledger:** adapter contract **Proposed**; physical continuity extractor **Not Implemented**; conversion tests **Not Run**; production G5 link **NO-GO**. G1 restricted synthetic numerical proof does not remove this block. Keep Issue #16 OPEN; do not merge unrelated Rust/UI changes or run needless platform CI for this document-only proposal.
