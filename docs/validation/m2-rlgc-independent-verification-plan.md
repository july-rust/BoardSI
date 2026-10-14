# M2 physical RLGC — independent reference and validation plan

**Plan status: NOT RUN for every NEW physical field/PCB verification case.**  
**Created:** 2026-10-14 KST; **source baseline:** `main@35285a47a72beabb4885a8e17f35867e6c20bac0` (merged PR #30).  
**Companion ADR:** [m2-rlgc-physical-model-adr.md](../architecture/m2-rlgc-physical-model-adr.md).  
**Immutable product specification:** [BoardSI v1.0](../specification/BoardSI_Development_Spec_v1.0.md), particularly §§6, 8, 8.2, 13, 13.1, 14, 15. Issues #7/#16/#17.

## 1. Strict separation of independent evidence

1. **Mathematical solver oracle (class M):** exact lossless distributed-line formula, independent high-precision ABCD, analytical symmetric/reference situations. Checks algebra/complex S from *given* R/L/G/C. Does **not** check the physics of geometry extraction.
2. **Independent field/physical oracle (class F):** separately implemented/solved 2D electrostatic/AC magnetic reference (e.g. independently scripted FEM or separately licensed Ansys 2D Extractor), field energy/charge and convergence with **fully identical physical boundary and materials**. Controls geometry-to-C/L, and eventually R/G. Not just the same BoardSI mesh/same code with different parameters.
3. **Independent full-wave/measurement oracle (class X):** a frequency-appropriate full-wave EM model or VNA-calibrated/de-embedded physically fabricated straight coupon with recorded manufacturing tolerances. Needed to validate quasi-TEM applicability, excitation/ports and frequency band or discontinuities. A 3D result is not automatically more accurate absent mesh/port convergence.
4. **Qualitative/formula sanity (class Q):** closed-form empirical microstrip/stripline formulas within their assumptions, energy bounds and dimensional invariance. Useful red flags; **not sufficient to label real stacked copper Validated**.
5. **Safety/property tests (class S):** symmetry, PSD, passivity, reciprocity, units, permutations, nonfinite rejection and unsupported geometry. Necessary but not a substitute for an independent physical oracle.

Existing evidence: [numerical report](m0-numerical-foundation.md) has 180 independent fixed scalar RLGC→S mathematical cases; [native report](m0-native-platforms.md) records numerical native results. These remain historical **M-class numerical PASS only**, not newly executed physical tests and **not** a substitute for F/X. Existing Qt/Worker evidence remains separate.

## 2. Proposed matrix — all new experiments Not Run

| ID | Coupon and varied parameters | Independent oracle class | Measured quantities / primary acceptance question | Status |
| --- | --- | --- | --- | --- |
| PH-01 | Analytic uniform lossless matched transmission line, varied l/f/Zref | M: telegrapher analytic and independent ABCD | S11/S21 phase, Zc, delay; specification T06/S mathematical target | **Not Run** (physical-plan case) |
| PH-02 | Uniform lossless mismatch, two real port references and arbitrary l | M; independently derived S | All complex S entries, power, reference conventions; initial product unequal refs must be Unsupported | **Not Run** |
| PH-03 | Homogeneous parallel plate / idealized stripline (analytic geometry) | M+F analytic electrostatic/field energy | C per length, L_ext from C0, energy and unit consistency; guard idealized edge-free limits | **Not Run** |
| PH-04 | Single microstrip w/h/t/εr sweep including thin/thick copper | F: independent 2D FEM/qualified extractor + Q formulas | C,C0,L_ext,Z0, delay, edge charge, field/boundary convergence | **Not Run** |
| PH-05 | Symmetric stripline w/t/h/εr sweep | F+Q | C,C0,L, impedance and field symmetry | **Not Run** |
| PH-06 | Asymmetric stripline unequal upper/lower gaps/ε | F | C,C0,L, propagation and convergence, no forced symmetry | **Not Run** |
| PH-07 | Multilayer microstrip with air and explicit solder mask on/off | F | ε distribution, C change, field-energy fractions, exact mask inclusion | **Not Run** |
| PH-08 | Finite copper thickness, trapezoid/surface shape and finite-width but longitudinally uniform reference plane | F | Cross-section sensitivity and topology; do not assume infinite reference | **Not Run** |
| PH-09 | Coupled microstrip P,N and gap/mask/ε variation | F + M 4-port | Full Maxwell C/C0 and L matrices; even/odd impedance for *symmetric* case, near/far coupling | **Not Run** |
| PH-10 | Symmetric coupled stripline with joint reference | F + M | Full matrix, modes and coupler directional S; no inferred mutual entries | **Not Run** |
| PH-11 | Asymmetric P/N widths/height/medium; one intentionally unequal termination | F+M+independent scikit-rf reference conversion | 4-port S then Sdd/Sdc/Scd/Scc; nonzero mode conversion, P/N inversion and port permutations | **Not Run** |
| PH-12 | DC copper conductor+return, known σ and cross-section, leakage intentionally different from RF tanδ | F independent conduction model | R_dc, separately bounded G_dc; no DC skin-loss extrapolation | **Not Run** |
| PH-13 | Skin depth vs copper t; low/high f and unequal σ | F independent AC magnetic reference and limiting theory | R(ω), L_int(ω), transition δ/t; no spurious R=0 | **Not Run** |
| PH-14 | Two adjacent conductors with return; near/far spacing | F independent magnetic field | Proximity redistribution, full R and L matrices; energy positive | **Not Run** |
| PH-15 | Homogeneous/multilayer dielectric loss and causal dispersion dataset | F+material measurements/doc | G(ω), C(ω), ε′/ε″, validity band, DC leakage separation | **Not Run** |
| PH-16 | Rough/flat copper pair with fully specified roughness model | F/X where suitable | Loss increment, parameter/band applicability; otherwise Unsupported | **Not Run** |
| PH-17 | Mesh refinement: uniform and corner-adaptive h levels, element order | F independent reference + self-convergence | C/C0/L, Z0/delay/S deltas, independent mesh residual, elapsed/RAM | **Not Run** |
| PH-18 | Exterior air domain size and alternative boundary conditions | F | Outer-boundary bias separate from mesh bias, comparison to enlarged-domain asymptote | **Not Run** |
| PH-19 | Positive (R,L,G,C), reciprocity, PSD and singular/ill-conditioned matrices | S + independent numerical matrix implementation | Eigenvalues, condition estimate, σmax(S), ||S−Sᵀ||, no silent conditioning repair | **Not Run** |
| PH-20 | Invalid: zero/negative dielectric or thickness, missing σ/Dk/Df, nonfinite and overlapping copper | S | Deterministic error and **no fabricated S** | **Not Run** |
| PH-21 | Via/Pad/plane slot/reference transition/bend/branch (full-route scope) | S; optionally X as future research | **Unsupported** for unmodeled 3D effects; never idealize silently | **Not Run** |
| PH-22 | Quasi-TEM f-band limits for increasing cross-section dimensions | X full-wave or de-embedded measurement + F | Model-specific max frequency, multimode/longitudinal effects; 30 GHz never implicit guarantee | **Not Run** |
| PH-23 | Cold/warm numerical runs: conductor count, mesh N, frequency F, RAM limit | independent instrumentation, not accuracy oracle | wall time, peak resident memory, mesh elements, stable failure under cap | **Not Run** |
| PH-24 | Independent Touchstone roundtrip, four-port mapping, partial frequency list | M+scikit-rf or separate parser | phase, all complex entries, refs, ordering, nonuniform sweep, missing samples blocked | **Not Run** |
| PH-25 | Unit transforms, P/N reversal, port permutation, reference-conductor changes | S+M | Consistent matrix congruences, modal sign conventions; invalid return rejected | **Not Run** |

These are **new** PH case identifiers; they do not renumber specification T01–T54. Traceability: T02=PH-20, T04/T24=PH-08/21, T05/T25/T28/T42=PH-09/10/11/25, T06=PH-01/03/04/05/06, T07/T43=PH-12..19, T09/T53=PH-02/12/15, T12/T44=PH-24, T14/T49=source/security audit, T36–T38=PH-21, T52=PH-07/08/15/20. Completion of a single PH is not automatically a full T-test PASS.

## 3. Required exact coupon definitions (to freeze before running)

Use **SI units only**. Each record MUST specify:
- Unique `coupon_id`, geometry source (fully synthetic design or licensed sample), immutable geometry hash SHA-256, license/reuse decision, expected topology and signal/reference conductor connectivity.
- Physical cross-section: coordinate system and origin; longitudinal propagation direction; individual polygons for copper, reference conductor(s), finite thickness/trapezoid parameters, dielectric/air/mask polygons, surrounding domain and boundary conditions; all measurements in metres.
- Each material: name/revision, εr(f) real/imag, **source time convention and complex-permittivity sign**, μr(f), σ(f), dielectric/conductor loss treatment (including separation of conductive versus polarization loss), Dk/Df measurement or table reference, temperature where relevant, source/version/hash, **calibrated frequency range**. Normalize passive material records to BoardSI `exp(+jωt)` using `ε*=ε′−jε″`, `ε″≥0`, and when applicable `Df=ε″/ε′` (`ε′>0`); record any conjugation from `exp(−jωt)` source data, reject ambiguous sign/convention, and do not double-count material conductivity in G. PH-15 remains **Not Run** until independently checked.
- Reference/ports: near/far terminal planes, P/N ordering, electrical reference conductor, 2-port vs 4-port, 1-based terminal order, power-wave definition, real reference impedances in Ω, port de-embedding and signs. Record any fixture transitions and their independent models.
- Solver/source of truth: tool, version/build/licensed use, independent formulation, mesh settings and size, field/wave solver options, boundary distance, excitation/return assignment, convergence sequence, solver residual and numerical precision.
- Frequencies: explicit list in Hz including f=0 only when the DC model is independently meaningful, DC method, all model applicability bands, and any extrapolation authorization.
- Outputs: R(ω),L(ω),G(ω),C(ω) units and conductor order, Maxwell vs SPICE matrix representation, γ/modes if meaningful, full complex physical S (Re/Im), mixed mode plus transform definition, temperature, known measurement/calibration and manufacturing uncertainty.
- Evidence: full unrounded reference/source archive hash, generated fixture content hash, tool config hash, invocation/command, OS/compiler/CPU/RAM, raw observed results, tolerance/error formula, Not Run/PASS/FAIL, model status and precise Unsupported reasons.

**Security:** public repo accepts only explicitly self-created synthetic data or artifacts for which source/license/redistribution were verified. Never upload company PCB/ODB/ASC, proprietary Stackup/material tables, extracted coordinates, anonymous measurements derived from secret boards, personal paths, lab credentials or private Touchstone. Local confidential validation remains offline; any published aggregate must separately pass disclosure review. No outside website converter. Do not infer 'publicly accessible' means 'redistributable'. Raw commercial/oracle results and proprietary source files require license review before committing.

## 4. Accuracy, metrology, and acceptance gates

**Specified mathematical goals (not measured physics):** v1.0 §13.1: maximum `|S_impl,ij−S_ref,ij|` over all complex entries and approved mathematical cases ≤ **1e-6**; impedance and phase-delay relative errors for mathematically exact simple lossless anchors ≤ **0.1%**; sampled S passivity and reciprocity residuals for the stated mathematical coupons ≤ **1e-6**. Correct port normalization is mandatory. Touchstone independent full-matrix finite-precision roundtrip target ≤ **1e-12**. Do not misapply these targets as general microstrip physical extraction error guarantees.

**Physical field targets are PENDING:** create separate signed, unit-consistent `max_rel_C`, `max_rel_L`, `max_rel_R`, `max_rel_G`, `max_abs_complex_S`, `max_Z0_rel`, `max_delay_rel`, and uncertainty intervals by structure/frequency/reference. Agree magnitudes only after a qualified external F/X oracle, mesh/domain convergence and material/geometry manufacturing tolerances. Do not invent "1%" or "30 GHz PASS". Near-zero values require absolute tolerance and floor policies rather than unstable percentage-only errors. Complex S reference tolerance is affected by uncertainty in fixture de-embedding.

**Required per-case report**:
- Independent oracle type M/F/X/Q/S; **M-only success never earns physical Validated**.
- Sample count, exact edge and frequency points, reported maximum and distribution (not only average); complex amplitude and phase error, endpoint/near-zero handling.
- Mesh and boundary sequence until *both* stabilized, with an independent accuracy reference where possible. Set expected monotonic/variance rule in advance; refinement need not strictly monotonically improve every output.
- C/L/G/R physical matrix symmetry / eigenvalue and energy, σmax(S) and reciprocity residual per f; no passivity "clamping".
- Actual CPU wall time, peak resident RAM, mesh DOF, number of factorization/RHS solves; quantify scaling on declared hardware instead of a guessed Big-O certification.
- Artifact/tool rights, limitations, unsupported cases, raw SHA-256 hashes and reproduction recipe.

**PASS** only for an executed case on a fixed commit with independent expected reference and documented tolerances; **FAIL** on executed mismatches; **Not Run** otherwise; **Unsupported** for unmodeled requested physical capability rather than numeric FAIL or fabricated comparison. Physics **Validated** additionally needs independent F/X evidence, convergence, calibrated materials, band, boundary and uncertainty. Failure of one case does not by itself reclassify an unrelated supported coupon.

## 5. Candidate reference and licensing plan

| Reference | Role | Available license evidence | Actual reference readiness |
| --- | --- | --- | --- |
| Exact lossless/ABCD high precision independent equations | Foundational mathematical S oracle | Self-created formulas/fixtures, provenance preserved | Prior scalar tests already PASS; **new PH runs Not Run** |
| Independent FEniCSx/DOLFINx 2D electrostatic PDE implementation | Class F C/C0 cross-check (not complete RLGC without separate AC formulation) | Project [LGPL-3.0-or-later statement](https://github.com/FEniCS/dolfinx/blob/main/README.md); dependencies to inventory | **PENDING** setup, independent formulation and case execution |
| Qualified Ansys 2D Extractor setup | Independent field RLGC and AC losses | [Official 2D technical notes](https://ansyshelp.ansys.com/public/Views/Secured/Electronics/v252/en/Subsystems/Q3DExtractor/Content/Q3D/2DExtractorTechnicalNotes.htm?TocPath=Q3D+Extractor+Help%7C2D+Extractor+Technical+Notes%7C_____0); commercial license required | **PENDING** license/access, geometry/port harmonization and permissible output use |
| scikit-rf | Independent 4-port/mixed-mode/Touchstone calculations | [BSD-3-Clause](https://github.com/scikit-rf/scikit-rf/blob/master/LICENSE.txt); [se2gmm](https://scikit-rf.readthedocs.io/en/latest/api/generated/skrf.network.Network.se2gmm.html) | **PENDING** version pin and fixture |
| openEMS full-wave reference | Optional out-of-band/3D check as standalone program | [GPL-3.0](https://github.com/thliebig/openEMS), license/redistribution review before use | **PENDING**, not needed for initial scalar field extraction |
| NIST published independent measurements | Published validation rationale and possible traceable dataset | [Multiconductor line metrology](https://www.nist.gov/publications/multiconductor-transmission-line-characterization), [materials metrology](https://www.nist.gov/publications/dielectric-and-conductor-loss-characterization-and-measurements-electronic-packaging) | **PENDING** actual raw geometry/data and redistribution rights |

**No third-party raw samples or code are included by this PR.** The project will build synthetic coupon coordinates itself; externally provided numeric/oracle tables may be committed only after explicit source-rights and redistribution checks. Document citation is not a data license. Any product-linked FEM library decision must be separately reviewed for distribution compatibility. Open-source external programs can be *independent test tools*, not assumed dependencies.

## 6. Validation order and implementation work gates

**Gate G0 (pre-implementation).** Freeze lossless scalar geometry input/ports, chosen independent F reference, explicit PEC/lossless metadata, external rights, definition of mesh/domain convergence, and PH-01/03/04 expected-reference generation. All currently PENDING; do not stamp Ready/Validated.

**G1 (microstrip electrostatics).** One straight single microstrip C/C0 and L_ext implemented; execute PH-01,03,04,17,18,19,20 (safety negative controls). Evidence tied to commit and hash. Release only specifically qualified ideal uniform coupon scope; no real lossy-board full-path results.

**G2 (other single ended geometries).** PH-05/06/07/08 and material/boundary validity controls. Finite-ground but longitudinally uniform segments may be supported after their own benchmarks; plane cutouts remain unsupported.

**G3 (lossy physics).** Implement and separately validate PH-12/13/14/15 and any optional PH-16. Declare supported f intervals, DC handling and material dispersion; test R,L_int,G not silently zero. Numerical scalar mapping baseline remains independent.

**G4 (coupled differential).** Implement joint multi-conductor field matrices and stable 4-port propagation; execute PH-09/10/11/19/24/25. Mixed mode conversion cannot be independently graded from guessed mutual coupling. Verify both P/N reversal and genuinely asymmetric mode conversion.

**G5 (product integration).** Canonical model → extractor → numerical → Worker/test report → export; execute critical T02,T05,T06,T07,T09,T12,T24,T25,T28,T36–T38,T42,T43,T44,T52,T53 as applicable. Maintain Worker/Qt nonblocking prototype and regression suite without redeveloping PR #30. Product-level 30 GHz eligibility requires PH-22 X-type physical/full-wave evidence.

**Open** #7 (source rights, baseline PC), #16 (actual RLGC physics) and #17 (DC, complete multiport/Touchstone) until appropriate executed tests and review are complete. Documentation alone meets none of their overall Definition of Done.

## 7. Initial documented resource targets: pending qualification

Benchmark at minimum n=1/n=2 field excitations across mesh sizes and F points; report compilation/release/debug, executable host architecture, wall clock, peak RSS, maximum matrix condition, solver residual, mesh DOF, C/C0 assemblies and reuse count. State physical data band/uncertainty in addition to CPU metrics. Use explicit memory pressure/timeout failure receipts; no silently undersampled sparse matrix/mesh. User's low-resource ambition is a **design goal**, not a demonstrated performance result. Reuse the already validated Worker control plane where appropriate.

## 8. Completion ledger for sixth documentation-only task

| Deliverable | Status |
| --- | --- |
| Review latest main/spec/existing numerical evidence | Reviewed as input, not rerun |
| Architecture direction and explicit pending decisions | **Documented** |
| Independent PH-01..25 cases and physical evidence levels | **Planned** |
| Physics FEM/EM/measurement executions | **Not Run** |
| New Rust numerical or physical code | **Not implemented** (out of scope) |
| Physical geometry→RLGC Validated claim | **None** |
| Qt/IPC code or existing v1.0 specification changes | **None** |
| PR merge, issue closure, new Actions jobs | **None requested** |
