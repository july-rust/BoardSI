# PH-08: exact uniform-square quadratic HMM identity (scoped)

**Scope: independently transcribed PUBLIC SYNTHETIC exact arithmetic.**
**Not a complete original HMM executable replay, not a certified floating-point
error budget, not a new Gate Q acceptance.** Issue #51.

## Provenance and separation

An authorized *private* Stage 4 archive was inspected without publishing its
source or inputs. The external ZIP digest was independently matched as
`6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9`.
The idealized calculation below reflects a uniform square cell, homogeneous
epsilon=1, the manufactured quadratic `phi(x,z)=x²+z²`, source `-4`, and
Simpson Dirichlet face means. It is implemented **independently** using only
Python Fraction arithmetic. It does not execute private archived code on CI.

Original Stage 4 historical Gate Q remains FAIL (ratios 0.953506 and
0.459163 against 1.8). Historical Stage 5B FAIL remains unchanged.

## Analytic derivation

Let a square cell have center `c`, side `h>0`, four outward unit
normals `n_f`, side length `h`, and distance `h/2` to each face.
For the quadratic `phi(x,z)=x²+z²`, write `g=grad phi(c)=2c`.
On any straight face, Simpson's quadratic-exact mean satisfies:

```text
face_mean(phi) = phi(face_midpoint) + h²/12
               = phi(c) + (h/2) n_f.g + h²/3.
```

Consider the candidate exact discrete unknowns

```text
u_cell  = phi(c) - h²/6
u_face  = face_mean(phi).
```

For the ideal square local energy
`h² |G|² + 2 sum_f R_f²`, with

```text
G   = sum_f (h n_f / h²) (u_face_f - u_cell)
R_f = u_face_f - u_cell - (h/2) n_f.G,
```

the opposite-face gradient difference yields `G=g`; all four
remainders satisfy `R_f=h²/2` exactly.

Differentiating the symmetric local energy bilinear form yields the
discrete cell equation and the face-flux contribution

```text
(Ku)_cell   = -2 sum_f R_f                  = -4h²
(Ku)_face_f = h n_f.G + 2R_f
              - sum_k R_k (n_k.n_f)         = h n_f.g + h².
```

The cell load is `source * area = -4h²`: **cell residual zero**.

For two adjacent cells separated by `h n`, the gradients differ by
`2h n`; their outward normals are opposite. Hence the sum of both
contributions to each internal face equals zero. Boundary faces are
prescribed using the exact Simpson quadratic mean, so the Dirichlet
conditions are also met.

The nonnegative local energy vanishes only when all face and cell values
are equal on each cell; connectivity plus prescribed Dirichlet boundaries
removes the remaining constant mode. Thus the finite uniform-square
discrete system is uniquely solved by the candidate values in **exact
arithmetic**, under the stated energy and boundary assumptions.

A rank-six quadratic recovery using only cell-center unknowns sees the
same constant `-h²/6` added to all exact quadratic center samples.
Because the quadratic basis includes a constant and its gradient is zero,
its **exact-arithmetic** recovered gradient is unchanged.

## Independent numerical checks

`tools/validation/ph08_uniform_quadratic_identity.py` evaluates the
local equations and all shared-face/boundary equations with exact
`Fraction` arithmetic. No floating tolerances are used.

`tools/validation/test_ph08_uniform_quadratic_identity.py` covers
n=2,4,8,16,32; a total of 1,364 exact synthetic cells. Cell, internal
face, Dirichlet face and gradient residuals must equal exact rational zero.

Negative controls deliberately mutate cell bias, its sign, face mean,
source, boundary-midpoint policy and input types. Each must expose a
nonzero residual or reject malformed input.

Optional CI: `.github/workflows/m2-ph08-uniform-quadratic-identity.yml`.
The CI is not a required protected check and never imports original ZIP.

## What this establishes — and does not

**EXACT CONDITIONAL THEOREM:** in ideal arithmetic for the stated uniform
square local energy, exact Simpson quadratic boundary means, homogeneous
epsilon and constant source, the `-h²/6` cell shift and `+h²/12` face
mean shift satisfy every discrete equation.

**CONSISTENT WITH ORIGINAL OBSERVATION:** original uniform quadratic
cell shifts of approximately `-h²/6` and face shifts of `+h²/12`
match this ideal identity in sign and magnitude. This is stronger than
a numerical nine-point observation but still conditioned on the ideal
geometry/energy specification.

**NOT VERIFIED:** exact equality for captured binary64 geometries,
source/face assembly rounding, sparse global solve, all recovered
stencils, RMS accumulation, or any certified forward error bound.
Roundoff is a *plausible explanation*, not an a-priori bound or proof of
the original observed convergence-rate failure. Skew/graded meshes,
material interfaces, Gate M and open-domain tails are excluded.

**Do not change** Historical Stage 4/Stage 5B Gate Q FAIL, AC-P1
PENDING, T/R PROPOSED_NOT_FROZEN, PH-08 Contract NOT_FROZEN, Rust FEM
NO_GO, Gate M NOT_VERIFIED, physical BOUND_NOT_CERTIFIED. PR #35 stays
Draft/unmerged; Qt #46 is independent.

## Remaining proof obligations

1. Authenticate that the original complete active path (not a selected
   operator) matches all ideal assumptions for uniform quadratic MMS.
2. Construct end-to-end a-priori floating assembly, solver, coordinate,
   recovery and RMS bounds, not solely observed binary64 errors.
3. Repeat independent original-data checks privately in an approved,
   authenticated, isolated environment; expose only sanitized summaries.
4. Develop *prospectively preregistered* T/R criteria, hold out original
   data, retain historic FAIL as immutable evidence.
5. Independently prove or refute Gate M finite-box/open-domain conditions.

References: Issue #51 and
`docs/validation/m2-g2-ph08-original-hmm-root-cause-analysis.md`.
