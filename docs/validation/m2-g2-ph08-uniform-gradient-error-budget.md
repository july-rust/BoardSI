# PH-08 — Public synthetic uniform Gradient error budget (scope-limited)

**Issue #51. Precedent:** verified PR #68 exact two-operator algebra;
verified PR #70 ideal uniform-square exact identity. This is NOT a private
original HMM replay, not production FEM, not a physical uncertainty certificate.

## Motivation and rigor boundary

PR #70 proved under ideal assumptions that for phi=x²+z², source=-4,
homogeneous eps=1, uniform square cells of side h, exact Simpson Dirichlet
means and ideal discrete arithmetic:

- u_cell = phi(center) - h²/6;
- u_face = phi(face midpoint) + h²/12.

Exact common-potential offsets vanish through full-rank quadratic recovery
when the transfer annihilates constants. That theorem **does not** bound
the floating global HMM assembly, linear solve, cell reconstruction,
recovery, or RMS accumulation of the original private Stage 4 source.

## Independent executable PUBLIC synthetic scope

The new `tools/validation/ph08_uniform_gradient_error_budget.py` builds nine
off-origin normalized cell-center samples for n=8,16,32 from the ideal
quadratic discrete solution. Reference matrices and samples are exact
`fractions.Fraction`, including the generally non-dyadic -h²/6 bias.
Captured operands are binary64 and gradients are computed with synthetic
`numpy.linalg.lstsq` using actual double-precision arithmetic.

The numeric solver is NOT a global HMM solve. The reference manufactured
physical gradient is exactly evaluated at (cx+u*h,cz+v*h) with u=1/4,
v=-1/4. The diagnostic invokes the existing independently tested PR #68
four-term exact rational decomposition, including the mandatory operator
difference term (Tc-Tr)*y_ref.

For captured A,D, observed binary64 gradient g_obs, exact ideal cell samples
y_ideal, reference operator Tr and exact manufactured g_exact:

```text
g_obs-g_exact = (g_obs-Tc*y_num) + Tc*(y_num-y_ideal)
              + (Tc-Tr)*y_ideal + (Tr*y_ideal-g_exact)
```

All quantities in the identity are exact rational interpretations of the
supplied data. Decomposition terms mean, respectively, observed recovery
arithmetic defect, sample/input difference, operator perturbation, and
reference consistency defect. The identity and centered operator norm bound
are conditional on exact inputs; neither certifies the original data's
physical provenance nor bounds error introduced *before* operand capture.

Explicit adversarial synthetic ULP perturbations change A, D, or samples,
with an immutable nonphysical status. The ULP model is an experiment, NOT
a statistical uncertainty estimate, universal floating error floor, or a
worst-case forward error proof. NumPy's QR/SVD implementation is not asserted
to match any uninspected source/environment. A small observed residual or
a normal-equation residual does not by itself bound original forward error.

## Acceptance claims and limitations

This task may establish:
1. exact algebraic agreement for fully specified synthetic input;
2. reproducible NumPy floating output and bounded-input sensitivity examples;
3. distinction of sample, operator and arithmetic contributions;
4. negative controls and GitHub CI provenance.

This task may NOT claim:
1. full original HMM/PDE/grid/Dirichlet source-path identity;
2. original whole-domain floating/PDE/open-boundary error bound;
3. original original uniform/skew/graded gradient RMS root cause;
4. historical Gate Q PASS or retrospective exception;
5. Gate M certification, AC-P1 acceptance, frozen contract or Rust FEM GO.

## Source/provenance boundaries

The two private known archives are NOT uploaded to public GitHub. If the
actual private Stage 4 whole-cell execution cannot run in an authorized,
network-isolated environment, its status remains PRIVATE_EXECUTION_NOT_VERIFIED.
Original historical values are preserved in
`m2-g2-ph08-stage4-original-gateq-reexecution.md`.

Historical uniform quadratic ratios: 0.953506 and 0.459163 against 1.8.
Stage 4 and Stage 5B Gate Q: FAIL. AC-P1 PENDING; T/R
PROPOSED_NOT_FROZEN; PH-08 Contract NOT_FROZEN; Rust FEM NO_GO; Gate M
NOT_VERIFIED; Physical Bound BOUND_NOT_CERTIFIED.

## Remaining blockers before qualifying the physics

- Independently attest actual original full-domain A,D,y, physical sites,
  HMM sparse assembly, source/boundary and cell reconstruction against
  the conditional ideal theorem in a private approved environment.
- Derive mathematically *a-priori* assembly, sparse-solve forward,
  recovery and gradient/RMS bounds, not only an exact captured-data identity.
- Separately preregister a novel T/R acceptance rule before new held-out
  tests, without altering the archived historical FAIL.
- Gate M is separately blocked on finite-box bracketing, flux and open tails.

Source links: PR #68, PR #70, Issue #51.
