# PH-08 Gate Q — Recovery hardening evidence and acceptance limitations

**Issue #51 / PR #57. Status: PROPOSED / NOT FROZEN.**
**Historical Stage 4 Gate Q: FAIL. AC-P1: PENDING. Contract: NOT FROZEN.
Rust FEM: NO-GO. Stage 5B: NOT RUN. Float bound: BOUND_NOT_CERTIFIED.**

## Provenance and scope (2026-10-17 KST)

Review baseline: PR #56 squash merged, main
`86d7d869a51ed2d8e1c47f0f131455f9d2dedb7c`.
New code is **synthetic numerical tooling only**; do not interpret positive
`VERIFIED` results as original HMM, Gate Q, Gate M, or FEM approval.
No original/private Stage 4 archive was supplied to the GitHub CI run.
The Stage 4 historical ZIP SHA-256 remains
`6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9`.
Stage 5B archive and independent external hash remain unavailable.

## Source-review findings from PR #56

1. `diagnose()` and `rational_fit()` interpret **any** finite six-column
   matrix as a least-squares design; a 6×6 intercepted matrix is NOT evidence
   that it represents the original six-term polynomial basis. In particular,
   an existing normal matrix H must not be used as a second design A.
2. The original `PRIVATE_SCRIPT` only hooked selected NumPy/SciPy entrypoints;
   aliases bound before patching, custom QR/LU/Cholesky and separately bound
   methods could bypass it. Recovery invocation and actual matrix interception
   are distinct events. Solver instrumentation can observe a direct solve
   without proving that the physical problem equals the synthetic stencil.
3. Earlier adversarial tests asserted only that numeric differences were
   nonzero/large; there was no tested FAIL/NOT VERIFIED **verdict interface**.
   Constant-coefficient corruption is invisible in a gradient-only check,
   which must therefore also require explicit coefficient fidelity.
4. The original bridge emitted private child-process stderr on failure; this
   could leak source or path information. The revised bridge redacts stderr
   and limits public console output. A private explicit output-json file may
   contain provenance and aggregate diagnostics; keep it in an approved
   nonpublished location.

## Implemented tools

- `ph08_gateq_recovery_hardening.py`: `rational_operator()` requires
  explicit design/weighted-design/normal-equation/linear-system role. Design
  roles form A^T W A on *binary64 input operands*; existing 6×6 normal or
  other square systems use direct rational solve without another squaring.
  `observed_diagnostics()` separates cond(A) from cond(H), reports observed
  system backward residual, never a certified float error bound.
- `gradient_transfer()`: for user-declared six-term basis, sample order,
  positive diagonal W, and explicit 2×6 physical-gradient map D, compute
  exact rational T = D (A^T W A)^(-1) A^T W.
  Under the mathematical operator represented by the given binary64 operands:
  `delta_g = T delta_y`, so
  `||delta_g||_inf <= ||T||_inf ||delta_y||_inf`.
  This is an exact finite-dimensional algebraic envelope, NOT a certified
  bound on original HMM potential error, real coordinate/basis errors, or
  floating-point solver and gradient rounding. If the original uses a
  different operator, this formula is inapplicable.
- `verdict()` rejects nonfinite/malformed vectors (FAIL), coefficient or
  gradient mismatch beyond the two explicitly preregistered tolerances
  (FAIL), and absent provenance, semantics, D mapping or sensitivity
  preregistration (NOT VERIFIED). The public judge **cannot** certify
  original HMM merely from caller-provided `float_bound_certified=True`;
  `VERIFIED` is limited to `SYNTHETIC_IMPLEMENTATION_ONLY`.
- `ph08_gateq_hmm_bridge.py`: opt-in private Stage 4 adapter runs
  `mms_suite()` separately in baseline and instrumented child processes,
  verifies replay and unchanged output, counts Recovery entry and solver
  interceptions separately, and stores minimal operator metadata.
  Every captured operator is `UNCLASSIFIED`; hints such as "design" mean
  algebraic **hypotheses**, not confirmed cell/face sampling semantics.
  Imported aliases and custom solvers remain incompletely intercepted.
  Errors/failure to capture => NOT VERIFIED/ORIGINAL_RECOVERY_NOT_CAPTURED.
  Original mode has a nonzero process exit status even if unclassified
  operands were captured; no accidental physical qualification.

## Pre-registered *synthetic test* sensitivity

At test authoring time, before CI execution, the standalone synthetic
nine-site test defined gradient infinity error tolerance **1e-8** and
coefficient infinity error tolerance **1e-8** in the dimensions of its
normalized synthetic basis. These values are **NOT** Gate Q physical
acceptance thresholds, and cannot be transferred to original HMM scales.
Mutations are chosen outside the specified synthetic budgets; a positive
test must remain within both. Source digest equality in the synthetic
fixture asserts internal provenance only, not independent physical truth.
Source/sample order, cell/face identities, W, coordinate origin and D still
require original-source forensic confirmation.

Negative coverage includes constant coefficient corruption (even with
unchanged gradient), gradient sign/scale/offset, individual coefficient
corruption, NaN/Inf, missing/replaced samples via pinned provenance,
wrong/absent weighting, rank deficiency, non-symmetric false normal matrix,
unexpected solver/role combination, unattested source, and unverified
original-scope attempts. Synthetic stand-ins execute `PRIVATE_SCRIPT`
with and without interception and exercise the bound-function/alias-miss case.
Tests ASSERT the **verdict** is FAIL/NOT VERIFIED, not just a difference.

## Synthetic-only observed-versus-certified split

`rational_operator()` treats each already-rounded binary64 operand as an
exact rational value. This does not recover the ideal physical coordinates
or exact manufactured potentials. `observed_diagnostics()` condition and
system backward residual are computed in float64; measured residuals,
and 100-digit mpmath agreement from prior tooling, are NOT outward-rounded
rigorous solver/geometry error bounds. In particular, for a full-column-rank
design, an exact rational inverse map may be derived, but it does not bound
the true HMM gradient until input potential/geometry representation bounds,
actual algorithm-specific solver error, rank decisions, gradient mapping,
and exact input sites are independently established.

A spatially constant potential shift vanishes under differentiation **only
if** the recovery basis includes a free constant column, W is unchanged,
the operator is full rank, and D annihilates that constant coefficient.
The synthetic test demonstrates exact row sums of T equal zero for its own
nine-site basis. The original uniform quadratic observed approximately
`-h²/6` is **OBSERVED_NOT_DERIVED**. Real HMM potential errors can
also contain nonconstant terms, which must be evaluated separately.

## Proposed Gate Q T/R rule — not frozen

- **T** requires identical original meshes/source and independently
  established physical gradient references, at least three resolutions,
  prospective absolute gradient limits, prospective decrease ratios and
  independent uncertainty estimates. Historical uniform quadratic
  0.953506/0.459163 against 1.8 remains FAIL.
- **R** requires independently attested original sample/operator semantics,
  exact/high-precision same-operand reference, certified outward-rounded
  bounds for quantization + assembly + actual solver + D scaling, verified
  potential-to-gradient transfer, prospective T/R crossover, and mutation
  detection. No retrospective threshold tuning. Missing independent
  source/bound/switch evidence => NOT VERIFIED or INDETERMINATE, never PASS.
- Gate M retains energy-bracketing hypotheses, flux equilibration,
  finite-precision energy interval and finite-box/open-domain tail/C/C0
  certification as separate GO blockers.

## GitHub CI and exact-head reporting

The **optional** existing `m2-ph08-gateq-independent.yml` was extended
to discover `test_ph08_gateq_*.py`, pin Python 3.13, NumPy 2.3.5,
SciPy 1.17.0, mpmath 1.3.0, report PR HEAD separately from checked-out
merge SHA, and prohibit private evidence ZIPs. It does not alter the 17
Strict required contexts. Full execution results, any fixes and signed PR
HEAD are verified only from matching GitHub Actions run evidence. Earlier
PR #56 30/30 synthetic PASS is historical, not proof of these new tests.

### Outstanding — NEVER silently promoted

Original Stage 4 ZIP rerun/new HMM A/B/C (3 grid types × 3 resolutions × 2
polynomials), original cell/face layouts/basis/weights and source line
fingerprint, actual HMM solver algorithm, QR/LU/Cholesky error model,
independent 100-digit vs exact rational against original operands, a
certified floating upper bound and HMM physical acceptance: **NOT VERIFIED
or NOT RUN**. Stage 5B: **NOT RUN**. The first public GH CI run must not be
reported as PASS until its final test count and exact checkout/PR HEAD
are verified. Do not modify Rust FEM.
