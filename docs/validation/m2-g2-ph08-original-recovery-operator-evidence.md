# PH-08 — Stage 4 original HMM Recovery operator audit (2026-10-17 KST)

**Status: selected original-operand operator audit COMPLETED; PHYSICAL Gate Q remains FAIL.**

Historical Stage 4 Gate Q **FAIL** / AC-P1 **PENDING** / PH-08 Contract **NOT FROZEN** / Rust FEM **NO-GO** / Stage 5B **NOT RUN** / floating-point upper bound **BOUND_NOT_CERTIFIED**. Gate M unresolved independently. Issue #51. Baseline main: `aa75b85c45c73f9081a7662a74c30b381c63124c` (GitHub Verified, PR #58). No Stage 4 ZIP, private snapshot, raw operands, private geometry or source excerpt is published in this repository.

## Provenance and original MMS

The authorized PRIVATE Stage 4 archive was read in an isolated temporary analysis environment, not in public GitHub Actions. External SHA-256 of actual bytes: `6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9` (MATCH), length 2,301,227 bytes. All 338/338 SHA256SUMS entries and 20/20 independent source-snapshot manifest digests matched; ZIP member count 339. The original archived `m2_ph08_stage4_qualification.mms_suite()` was re-executed from the private source snapshot: 36/36 saved MMS rows matched (scalar abs tolerance 2e-13, rel 5e-10), all failure records exactly identical. Historical uniform quadratic gradient reduction [0.953506146487913, 0.45916314091601546], required >=1.8: **FAIL**. This is reproduction of failure, not qualification.

## Original source path and semantics — private source reviewed, not copied

Private SHA-pinned source locations (1-based line numbers in the original archive):

- `m2_ph08_stage4_qualification.py:50-75,98-119` — mesh/manufactured PDE, original HMM `solve`, Recovery instantiation, probe gradient and 36-row acceptance.
- `m2_ph08_hmm_reference.py:198+` — sparse global HMM solve, face unknowns, reconstructed cell unknowns and returned state.
- `m2_ph08_stage3.py:120-165,167-195` — `point_row`, `Recovery.polynomial`, and off-origin `Recovery.at`.

Confirmed actual operator in the tested path: same-permittivity two-ring **cell-neighbor stencil**; cells ordered by increasing cell ID; fitted values are **cell-center unknown potentials**, not face potential samples. Faces provide neighborhood adjacency and x/z extent for the per-target origin/scale. The origin is the recovery cell centroid, and the two strictly positive scales come from maximum absolute x/z offsets of nodes on stencil faces. Six columns are precisely the normalized polynomial basis `[1,u,v,u²,u·v,v²]`. The original numerical solver is `numpy.linalg.lstsq(A, y, rcond=None)`; no explicit W is applied. It checks rank 6 and a returned singular-value ratio <=1e8. These facts come from the verified private source **and** direct runtime entrypoint capture; a six-column matrix shape alone was never interpreted as a design matrix. Other potential runtime paths/aliases are not generalized beyond this audited invocation.

For a target with normalized evaluation coordinate `(u,v)` and scales `(sx,sz)`, the physical potential-gradient map on `c=[c0,c1,c2,c3,c4,c5]` is

```text
D = [[0, 1/sx, 0, 2*u/sx,   v/sx,   0],
     [0, 0, 1/sz,     0, u/sz, 2*v/sz]]
gradient(phi) = D*c ; E = -gradient(phi).
```

This map is *evaluation-point dependent*; using only `c1/sx, c2/sz` off origin is wrong. Binary64 formation/rounding of `u,v,sx,sz` is not a certified physical-coordinate error bound.

## Independent original-input calculations (one sampled recovery probe for each mesh/grid)

One actual HMM PDE solution and one original `Recovery.at` site per case were run for quadratic potential. A hook captured the *actual* `numpy.linalg.lstsq` operands and coefficients from `Recovery.polynomial`. Each 6-column captured operator was independently solved via exact `Fraction.from_float` normal equations (full rank); independently checked by 100-decimal-digit mpmath. Gradient comparison uses the actual off-origin `D` from captured binary64 operands, preserving sign/scale conventions. These are **observed errors**, not certified bounds or all-cell quantification.

| Mesh | n | Samples | cond₂(A), observed | max coefficient error vs rational | max gradient error (physical-gradient units) | exact `||T||∞`, reported decimal |
|---|---:|---:|---:|---:|---:|---:|
| uniform | 8 | 12 | 11.142 | 1.212e-16 | 3.947e-16 | 7.451 |
| uniform | 16 | 13 | 11.604 | 4.918e-15 | 1.772e-15 | 12.903 |
| uniform | 32 | 13 | 11.604 | 2.539e-15 | 7.255e-15 | 30.727 |
| skew | 8 | 12 | 14.497 | 3.220e-16 | 8.908e-16 | 7.898 |
| skew | 16 | 13 | 14.625 | 1.294e-16 | 7.639e-16 | 14.468 |
| skew | 32 | 13 | 14.625 | 4.135e-16 | 2.498e-15 | 32.027 |
| graded | 8 | 13 | 18.142 | 3.656e-16 | 1.005e-15 | 7.230 |
| graded | 16 | 13 | 16.681 | 5.220e-16 | 1.284e-15 | 12.247 |
| graded | 32 | 13 | 14.853 | 5.559e-16 | 6.752e-16 | 20.843 |

100-decimal-digit mpmath and Fraction coefficients differed by at most 4.96e-102 **on the nine inspected cases**, as an observed cross-check, not an a priori guarantee. The rational transfer `T=D(AᵀA)⁻¹Aᵀ`, formed from the captured binary64 operands, had **exactly zero** row sum for a common constant shift in y in all nine cases. Therefore `δg=T δy`, `||δg||∞ ≤ ||T||∞ ||δy||∞` holds for this fixed algebraic map; it does **not** certify `||δy||`, input coordinate quantization, PDE discretization, floating assembly or actual solver errors.

## Uniform quadratic bias — observed, not yet proven as an HMM theorem

For the ORIGINAL uniform quadratic HMM full-grid cell unknowns, `cell_u - phi(cell_center)` was approximately `-h²/6` at n=8,16,32; the total spread across cells was ~4.01e-16, ~8.09e-16, ~4.14e-15 respectively. Face-potential minus quadratic midpoint potential was approximately `+h²/12` on sampled faces at n=8. A mathematical derivation from the full HMM cell elimination and all boundary equations is still **NOT VERIFIED**. Numerically constant error is annihilated by the gradient transfer at fixed A, but the observed nonconstant/rounding remainder is not certified.

## Gate Q T/R preregistration and unverified cases

- Historical `T` rate criterion >=1.8 remains in force for the archived Stage 4 run: **FAIL**. No ex post threshold adjustment.
- New `R` roundoff-aware criterion is **PROPOSED / NOT FROZEN**. A predeclared crossover rule and physical-unit forward-error budgets remain unavailable; this audit establishes an exact local *linear sensitivity operator*, **not** a rigorous floating-point solver bound.
- Cubic recovery on actual HMM PDE output, multiple recovery cells per grid, guaranteed quantization/assembly/solver error intervals, and certified `T` → `R` crossover are **NOT RUN / NOT VERIFIED**. Independent nine-site synthetic cubic tests do not become original HMM.
- Stage 5B remains **NOT RUN** in this task, regardless of separate Library archival material. Independent Stage 5B provenance would require a separate scoped audit; its hash must never be assumed equal to Stage 4.
- Gate M energy bracketing, equilibrated flux, finite-box/open-domain tails and rigorous whole-space C/C0 bound remain **NOT VERIFIED**.

## Public test/CI and publication

This report is sanitized; verification of private inputs has been performed outside GitHub only because the original archive must not be published. Public synthetic regression tests and GitHub Actions are separate steps and must be judged against the actual new PR head. Merely observing small coefficients or exact-rational arithmetic must never imply original physical PASS.

**Verdicts:** original provenance **PASS**; historical replay **PASS (FAIL reproduced)**; selected original operator semantics **PASS (scoped)**; nine selected exact-vs-float diagnostics **PASS (scoped observations)**; floating bound **NOT VERIFIED**; new Gate Q T/R **INDETERMINATE / NOT FROZEN**; historical Gate Q **FAIL**.
