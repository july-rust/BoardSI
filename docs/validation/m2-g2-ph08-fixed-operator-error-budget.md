# PH-08 fixed-operator gradient error decomposition (public synthetic only)

Issue: #51. Status: mathematical identity and PUBLIC SYNTHETIC regressions; NOT a run of private original HMM, and NOT a certified PH-08 physical model.

## Exact algebraic identity

Let A be m-by-6 full-column-rank captured binary64 design matrix, D the 2-by-6 physical potential-gradient matrix at the actual evaluation point. Interpret already-rounded captured binary64 operands as exact rationals and define T = D (A^T A)^(-1) A^T. Exact rational normal equations are used as an algebraic *reference*, never to recommend floating normal-equations solution.

For exact reference sampled potential y_ref, observed numerical samples y_num, independently justified exact target physical gradient g_physical and already-captured observed gradient g_obs:

g_obs - g_physical =
  (g_obs - T y_num)
  + T (y_num - y_ref)
  + (T y_ref - g_physical).

The three terms mean, respectively: observed output error relative to the exact captured-input fixed operator; propagation of the sampled-potential difference; and captured-reference polynomial/geometry defect. This equality is exact in rational arithmetic.

The exact fixed-operator inequalities are:
  norm_inf(T*(y_num-y_ref)) <= norm_inf(T)*norm_inf(y_num-y_ref)
  norm_inf(g_obs-g_physical) <=
    norm_inf(g_obs-T*y_num)
    + norm_inf(T)*norm_inf(y_num-y_ref)
    + norm_inf(T*y_ref-g_physical).

These do not certify original-source or mesh identity, upstream coordinates/assembly error, original floating solver a-priori bounds, PDE sample errors, physical field accuracy, Gate M, or any unobserved HMM cell. Numeric accuracy of supplied captured operands does not prove physical accuracy.

## What was implemented

- tools/validation/ph08_recovery_fixed_operator_budget.py:
  exact rational fixed-matrix diagnostic, six-term rank check, fixed T norm,
  exact gradient error decomposition, exact triangle/sample bounds, independent
  manufactured reference Fraction input requirement and fail-closed statuses.
- tools/validation/test_ph08_recovery_fixed_operator_budget.py:
  synthetic full-rank quadratic basis, 8 stencil points, exact off-origin
  D for u=1/2, v=1/4, sx=2, sz=4; tests include constant shift,
  input perturbation, observed output perturbation, fake reference,
  rank defect, nonfinite numbers and oversized exact rational reference.
- .github/workflows/m2-ph08-fixed-operator-budget.yml:
  separate OPTIONAL public-only tests, never a protected-required status.
  No private original archive is uploaded or executed.
- tools/validation/ph08_private_archive_guard.py plus six synthetic guard
  tests run in the EXISTING mandatory docs check on EVERY pull request and
  main push. This detects the two known original ZIP basenames at any tracked
  path and exact known SHA-256 bytes even when renamed. It does NOT prevent
  all forms of private data leakage (e.g. recompressed or extracted copies)
  and must not be advertised as comprehensive DLP.

Resource safety: bounded ingestion consumes no more than MAX_CAPTURE_ROWS+1
matrix rows or seven entries per six-column row; vector and D iterables
are bounded before materialization. Infinite iterables fail closed.

An input named a reference is not itself evidence of independent provenance:
Fraction types do NOT authenticate original HMM samples, mesh, generator or
coordinate semantics. Output always marks source as not independently
authenticated and original physical error bound as NOT CERTIFIED.

## Historical original evidence (NOT rerun in this PR)

Original Stage 4 uniform quadratic gradient RMS for n=8,16,32:
4.043940694889194e-15, 4.241127033930931e-15,
9.236645226944874e-15 respectively. Observed reduction ratios
0.953506146487913 and 0.45916314091601546, each below historical 1.8:
Gate Q FAIL. Nine selected local exact captured operator audits found exact
constant-shift annihilation, while the nearly spatially constant
-h^2/6 potential offset was observed, NOT mathematically proved from full
HMM PDE elimination and boundaries. See:
- docs/validation/m2-g2-ph08-original-recovery-operator-evidence.md
- docs/validation/m2-g2-ph08-stage4-original-gateq-reexecution.md

Stage 5B private historical FAIL was reproduced separately: 12/12 rows,
1761/1761 deterministic scalars match, 12 timings excluded, physical_solves=0
and pilot NOT RUN. The newer PR #64 wrapper has NOT yet been independently
private-smoke-tested against the original ZIP. See:
docs/validation/m2-g2-ph08-stage5b-private-replay-runbook.md

## Future acceptance blockers

1. Private Runner smoke test: only authorized immutable original Stage 5B ZIP,
   approved isolated runtime, sanitized status-only evidence. Otherwise
   PRIVATE_SMOKE_NOT_VERIFIED; public GitHub Actions are synthetic only.
2. For actual original per-cell HMM: bind y_ref, y_num, observed g, physical
   evaluation site, A,D and source/mesh SHA. Do not publish raw private rows.
3. Prove/certify upstream solver, assembly, input potential and coordinate
   errors and full-cell quantization bounds, not only captured algebra.
4. Derive/reject observed -h^2/6 from original equations including boundaries;
   independently inspect cubic/skew/graded original paths.
5. Prove Gate M finite-box bracketing preconditions, equilibrated flux and
   open-domain tail bound; a finite-box conditional estimate is not an
   infinite-domain physical certificate.
6. No retrospective rate threshold or T/R crossover reclassification.

Frozen decision: Stage 4/5B Gate Q FAIL, AC-P1 PENDING, PH-08 Contract
NOT FROZEN, Rust FEM NO-GO, Gate M NOT VERIFIED and BOUND_NOT_CERTIFIED.
