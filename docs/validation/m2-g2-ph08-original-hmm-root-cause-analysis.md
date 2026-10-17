# PH-08 original HMM Gate Q: two-operator gap analysis

**Status: PUBLIC SYNTHETIC EXACT ALGEBRA ONLY; ORIGINAL HMM ROOT CAUSE NOT PROVEN.**
Issue #51. Starting baseline main dea2e58d03685f42c4d96c635637cf755474426a.
No original private geometry, source, sampled cells or ZIP were uploaded.

## Evidence already confirmed, separated from inference

- Original Stage 4 uniform/quadratic bulk-gradient RMS for n=8,16,32:
  4.043940694889194e-15, 4.241127033930931e-15, 9.236645226944874e-15.
  Reduction ratios 0.953506146487913 and 0.45916314091601546 against the
  historical rate criterion 1.8: FAIL.
- A separately authorized original Stage 5B replay matched 12/12 rows and
  1761/1761 deterministic numerical scalars (12 timing scalars excluded).
  Historical Gate Q FAIL reproduced; physical_solves=0, pilot=NOT RUN.
- Nine selected original Stage 4 recovery operators had exact constant-shift
  cancellation for *captured* full-rank rational operators. Original cell
  potential minus manufactured potential was observed near -h^2/6, with
  nonconstant spreads growing at finer grids. It was NOT derived from the
  private original full discrete system.
- Roundoff dominance for the quadratic gradient is a plausible INFERENCE,
  not a certified floating point, assembly or PDE error theorem.
- The PR #64 private replay wrapper itself remains PRIVATE_SMOKE_NOT_VERIFIED;
  its predecessor's independent original-source replay is separate evidence.

## Algebraic extension beyond PR #66 fixed A,D

Let A_c,D_c be binary64 captured design and gradient matrices, interpreted
exactly as rationals. Let A_r,D_r be separately supplied exact rational
reference matrices, and let y_num (binary64), y_r (exact), g_obs (binary64),
and g_phys (exact) denote observed and reference values.

For both full-column-rank six-column systems:

    T_c = D_c (A_c^T A_c)^-1 A_c^T
    T_r = D_r (A_r^T A_r)^-1 A_r^T

The following identity is exact *conditional on supplied values*:

    g_obs - g_phys
      = [g_obs - T_c y_num]              (observed output arithmetic)
      + T_c (y_num - y_r)                (sample/input difference)
      + (T_c - T_r) y_r                  (A,D operator difference)
      + [T_r y_r - g_phys].             (reference consistency defect)

Unlike the fixed-T identity, the third term cannot be merged into sample
error when the design or physical-derivative operator changes.

In infinity norm,

    ||(T_c-T_r)y_r|| <= ||T_c-T_r||_inf * ||y_r||_inf

When both maps annihilate an exactly common constant potential mode, then
(T_c-T_r)1=0. Centering by the exact sample midrange c gives

    ||(T_c-T_r)y_r||
       <= ||T_c-T_r||_inf * (max(y_r)-min(y_r))/2.

The latter is invariant to a uniform reference-potential offset and may be
substantially tighter than the naive bound. All algebra is exact Fraction
arithmetic; exact inversion of normal equations is a *reference*, not a
recommendation for floating least-squares implementation.

Neither reference A_r/D_r nor the true original PDE value is authenticated by
providing Fraction data. This is not a certified uncertainty interval for the
original assembly, geometry, mesh, coordinate rounding or true potential.
It is a CONDITIONAL identity and exact captured-input bound only.

## -h^2/6 uniform quadratic offset: precise theorem boundary

At fixed full-rank quadratic recovery with the constant basis and derivative
of a constant equal to zero, T*ones=0. Therefore IF the original HMM
cell-potential error is identical -h^2/6 for every stencil point, and IF
captured operator and physical evaluation point equal the desired operator,
then the constant component alone contributes zero to exact gradient error.

The original source's interior and boundary HMM cell/face elimination,
Dirichlet face means, and all relevant coordinate choices have not been
independently re-derived in this public task. A nine-site numerical
observation does not prove a full-domain -h^2/6 identity. Nor is an exact
constant offset guaranteed after each independently rounded binary64 sample.
The synthetic fixture h=1/8, h^2/6=1/384 highlights that this rational
shift is not representable as an exact dyadic input.

Verdict:
- constant-shift cancellation: EXACTLY PROVEN CONDITIONALLY in recovery;
- original uniform HMM -h^2/6 law: OBSERVED_NOT_DERIVED;
- root cause of archived Gate Q failure: NOT VERIFIED;
- original solver/PDE/assembly/coordinate forward bound: BOUND_NOT_CERTIFIED.

## Synthetic regression and validation

Implementation: tools/validation/ph08_recovery_operator_perturbation.py
Tests: tools/validation/test_ph08_recovery_operator_perturbation.py

Cases include:
- exact off-origin 2D quadratic reproduction;
- changed captured A, changed captured D and nonuniform sample shifts;
- exact dyadic uniform potential offset and synthetic h^2/6 rounding;
- reference-offset-invariant centered operator bound;
- sign/axis mutations, wrong reference gradient;
- rank defects, nonfinite input, non-Fraction references, resource caps;
- explicit prohibition of physical certification from these tests.

No synthetic negative control is claimed to have executed through the
original private HMM solver.

## Original data and new Gate Q T/R prerequisites

1. Approved privately network-isolated HMM execution with authenticated
   source and full-cell stencils, actual A,y,D, gradient evaluation sites and
   reference geometry, not selected nine-site substitution.
2. Independent derivation or refutation of uniform -h^2/6 from original
   interior and all boundary discrete equations, including face means.
3. Separately bounded mesh/coordinate rounding, HMM global solve, sample
   potential, recovery least-squares and gradient/RMS accumulation errors,
   avoiding ex-post exact-float comparisons as substitute for a-priori bounds.
4. Independently preregistered new original polynomial/cubic/float/mutation
   criteria before unseen data, with signed dataset-to-reduction evidence.
   Old Stage 4 and Stage 5B FAIL must not be reclassified.
5. Separately prove Gate M Neumann/Dirichlet bracket admissibility, numerical
   flux correction and open-domain tail before physical field certification.

**Unchanged:** historical Stage 4/5B Gate Q FAIL, AC-P1 PENDING,
T/R PROPOSED_NOT_FROZEN, PH-08 Contract NOT FROZEN, Rust FEM NO-GO,
Gate M NOT VERIFIED, physical error BOUND_NOT_CERTIFIED, Issue #51 OPEN.
Qt Issue #46 remains separate; Draft PR #35 must not be merged.

References:
- docs/validation/m2-g2-ph08-original-recovery-operator-evidence.md
- docs/validation/m2-g2-ph08-stage4-original-gateq-reexecution.md
- docs/validation/m2-g2-ph08-fixed-operator-error-budget.md
- https://github.com/july-rust/BoardSI/pull/66
- https://github.com/july-rust/BoardSI/issues/51
