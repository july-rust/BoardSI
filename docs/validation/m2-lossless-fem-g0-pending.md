# M2 lossless 2D FEM microstrip prototype — G0 pre-implementation audit

**Audit date:** 2026-10-15 (KST)  
**Decision:** **G0 = PENDING / NO-GO** for physical Solver implementation. This is a verification gate record, **not** a working FEM implementation and **not** a physical PASS.  
**Immutable baseline:** `main@35285a47a72beabb4885a8e17f35867e6c20bac0` (merged PR #30); development specification `docs/specification/BoardSI_Development_Spec_v1.0.md`.  
**Design inputs (not merged into main at audit):** [PR #31](https://github.com/july-rust/BoardSI/pull/31), [ADR-M2-001](https://github.com/july-rust/BoardSI/blob/feature/m2-rlgc-physics-validation-design/docs/architecture/m2-rlgc-physical-model-adr.md), [PH independent verification plan](https://github.com/july-rust/BoardSI/blob/feature/m2-rlgc-physics-validation-design/docs/validation/m2-rlgc-independent-verification-plan.md).  
**Tracking:** [Issue #16](https://github.com/july-rust/BoardSI/issues/16) remains OPEN.

## 1. Verified repository state versus claims

1. At audit, PR #31 was **OPEN and unmerged**; thus neither its ADR nor its PH plan belonged to the audited `main`. The documents' design status is PROPOSED; the PH plan explicitly says **G0 currently PENDING**.
2. The independent numerical baseline in `boardsi-numerics` concerns **externally supplied scalar RLGC -> complex two-port S**, not geometry-to-C/C0, PEC field energy, meshing, physical microstrip or loss verification.
3. The `boardsi-model`, `boardsi-ipc`, worker and Qt prototypes do not constitute a 2D EM extraction/independent field oracle.
4. PR #31's inline review identifies an **unresolved complex-permittivity sign-convention issue** for future PH-15 (lossy material); this is not a missing lossless FEM formula, but must not be silently inherited by future lossy expansion.
5. This audit includes **no new solver, no benchmark run, no field extraction, no fixture oracle, no recorded CPU/RAM result and no changed Rust/Qt files**.

## 2. G0 decision table

| Mandatory G0 item | Evidence required to leave PENDING | Status as audited |
| --- | --- | --- |
| Adopted architecture and reviewed source | ADR-M2-001 reviewed/approved and incorporated through normal PR process | **PENDING** (PR #31 open; ADR proposed) |
| Frozen single-microstrip geometry and conductor identity | Versioned synthetic cross-section file: x/z axis, origin, propagation, trace width **w**, finite thickness **t** with exact metal profile, dielectric height **h**, εr, air, finite or explicitly idealized reference plane, material-interface coordinates and conductor shorts/overlap checks, all SI units | **PENDING** (requirements only, no frozen case) |
| Explicit signal/reference ports | One signal, an identified electrically continuous reference PEC, scalar near/far two-port order and orientation, definition of 1 V/0 V electrostatic excitations, positive real port reference, length and frequency samples, reference termination and cross-section-to-2-port handoff | **PENDING** |
| Material and idealization contract | Explicit **PEC**, **nonmagnetic** (μr=1), explicitly **lossless** εr, no inferred σ/Df/mask; retained provenance and unsupported real-PCB cases | **PENDING** for a locked fixture (principle documented) |
| Independent field oracle F | Independently run FEniCSx/DOLFINx reference or qualified 2D extractor, pinned version, independent weak form, excitation/geometry match, recorded actual C, C0 and independent convergence; rights to redistribute fixture/results | **PENDING** (candidates only, no runs) |
| Independent mathematical/analytical anchors | Reproducible PH-01 exact lossless two-port/ABCD reference and PH-03 explicitly valid homogeneous parallel-plate limit; PH-04 independent field plus qualified formula sanity checks | **PENDING** for new PH fixtures |
| Mesh convergence contract | Mesher/version, triangular basis/element order, corner/material-interface refinement levels, DOFs, residual/energy diagnostics, pre-agreed per-metric stopping rules and error estimator; numerical tolerance not retrofitted after results | **PENDING** |
| Exterior boundary convergence contract | Outer domain type/placement, PEC/Neumann/open approximation, independent domain-size sequence with matched mesh controls, bias assessment, rejection limits | **PENDING** |
| Frozen independent physical thresholds and model envelope | Externally justified C/C0/L/Z0/delay tolerances, unit conventions, numerical vs physical acceptance, geometry/material/boundary/frequency envelope | **PENDING** |
| Reproducible execution and resources | Version/hash for all scripts, sources and synthetic geometry, commands, actual host/compiler, wall time, peak RSS, mesh statistics, failure/timeout receipts | **Not Run** |

**Decision rule:** G0 cannot be considered approved from an ADR recommendation, an existing mathematical S regression, or an unexecuted field study. Approve only when the above artefacts are frozen with reviewable links and explicitly authorized numerical/physical acceptance criteria. Afterward implement G1 in a *separate* source change.

## 3. Proposed technical contract — NOT approved input or solver implementation

For a **uniform straight** single trace referenced to a continuous PEC plane, consider the 2D scalar electrostatic problem with trace at 1 V and reference plane at 0 V, piecewise constant explicitly lossless ε; use the **identical cross-section and external conditions** with all dielectric regions replaced by ε0 for C0. The electrostatic field energy per unit length `U' = 1/2 ∫ ε|∇φ|² dA` provides `C' = 2 U' / (1 V)²`, cross-checked independently with conductor charge. If the grounded C0 is positive and assumptions hold, compute `L_ext' = μ0 ε0 / C0'` for nonmagnetic PEC quasi-TEM, and `Z0 = sqrt(L_ext'/C')`, `v = 1/sqrt(L_ext' C')`, `delay per length = sqrt(L_ext' C')`. All are *per-unit-length* quantities where appropriate.

**These equations do not establish field-solver correctness or practical PCB frequency validity.** A 2D electrostatic method does not calculate finite-conductivity R(ω), internal L(ω), dielectric G(ω), skin/proximity or longitudinal via/pad/plane-slot discontinuities. No real-board full-path R=G=0 substitution is allowed. No frequency range (including 30 GHz) is claimed physically validated.

Candidate review choices (each **PENDING**, not a silent default): a version-pinned independent FEniCSx electrostatic reference; explicit bounded-box boundary treatment and independently enlarged-domain study; adaptive corner refinement with a pre-registered convergence metric; fixture-specific vs ideal infinite-plane geometry differentiated in IDs. A homogeneous parallel-plate analytic anchor is valid only in its documented no-fringing/edge-free limiting assumptions, not as an exact finite-width microstrip oracle.

## 4. Requested PH verification ledger (this audit)

| Test | Required proof | Actual result |
| --- | --- | --- |
| PH-01 | Independent analytic lossless 2-port S, Zc, phase/delay with declared reference and length | **Not Run** |
| PH-03 | Analytic limiting electrostatic C/C0/L/energy versus independent field calculation | **Not Run** |
| PH-04 | Independent F microstrip sweep for w/h/t/εr; C,C0,L,Z0,delay/edge fields | **Not Run** |
| PH-17 | Mesh-size/order/corner refinement and external F comparison | **Not Run** |
| PH-18 | Outer-domain distance/type convergence independent of mesh changes | **Not Run** |
| PH-19 | Physical/algorithmic positivity, energy, passivity/reciprocity and condition handling | **Not Run** |
| PH-20 | Invalid geometry/material/ref-input rejection without fabricated complex S | **Not Run** |

**Complex S, C, C0, L, characteristic impedance and propagation delay:** **Not computed here**; no example result is relabeled PASS.  
**CPU duration, peak RAM, mesh DOFs, solver residual, iteration history and domain convergence:** **Not Run / no measurements**.  
**PH-12..16 lossy, PH-09..11 differential, 3D via/pad, product 30 GHz validity:** **Unsupported/unvalidated scope**, not silently approximated.

## 5. Minimal unblock sequence

1. Review and independently merge PR #31 (or amend and separately approve its technical design); keep its unresolved review observation visible until resolved. This audit PR does **not** merge #31.
2. Commit small **synthetic, redistributable** geometry/port/material fixtures and a deterministic G0 manifest with source and SHA-256 records; freeze reference plane type, metal thickness/shape and 1-V/0-V convention.
3. Select and actually execute independent field oracle F with provenance/license review; produce PH-03/04 C/C0 datasets and mesh/exterior-domain sequences. Build PH-01 analytic S reference separately.
4. Pre-register acceptable *numerical field* and *physical* errors, bounding frequency/topology/material scope and exact measurement method. Distinguish dimensionless errors from per-unit-length quantities; avoid invented percentages.
5. Review G0 results explicitly. Only a **G0 Ready** outcome authorizes the separate G1 Rust FEM prototype PR and PH-01/03/04/17/18/19/20 execution.

## 6. Change impact and safety

- Source/fixtures/workflows/Qt/IPC/model/numerics: **unchanged**.
- Existing M0 tests and Windows/macOS Qt runtime: **not re-run** by this audit; historical evidence is neither overwritten nor promoted to physical PASS.
- New solver code: **intentionally absent** while G0 is PENDING.
- No privileged company PCB/cable/stackup data, commercial reference output, license-encumbered geometry or fabricated benchmark numbers committed.
- Issue #16: **OPEN**; PR auto-merge: **NOT ENABLED**. Only existing documentation checks should run for this documentation-only PR.
