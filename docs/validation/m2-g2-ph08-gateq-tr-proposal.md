# PH-08 — Prospective Gate Q T/R protocol (DRAFT / NOT FROZEN)

**This is NOT a pass rule and is not authorized to change historical gates.**
Issue #51; depends on PR #68 and PR #70.

## Historical immutable evidence

Original Stage 4 uniform quadratic bulk-gradient RMS n=8,16,32:
4.043940694889194e-15, 4.241127033930931e-15,
9.236645226944874e-15. Historical reduction ratios:
0.953506146487913, 0.45916314091601546, threshold >=1.8.
**Result: FAIL.** Stage 5B original Gate Q also FAIL.

## Proposed prospective structure — no thresholds chosen

- T (truncation): use independent, certified reference solutions and
  a preregistered convergence metric with a bound on contamination from
  floating arithmetic. A visually declining RMS is not a proof.
- R (roundoff): declare an a-priori absolute physical-unit forward
  error *bound*, separate from exact captured-input sensitivity and
  observed floating differences.
- Crossover: predefine and independently review criteria from
  mathematical bounds and held-out cases BEFORE collecting gate data.
- Every gate must distinguish PASS, FAIL, NOT_RUN and NOT_VERIFIED;
  an unavailable bound is NOT_VERIFIED, never implicit PASS.
- Hold out actual original HMM cases and materially different meshes.
  Neither PR #70's exact synthetic theorem nor a single nine-point
  stencil validates the original full-domain gate.
- Record input/source SHA, coordinate units, matrix and solver provenance,
  review outcome and complete row counts before gate decisions.
- Treat RMS error propagation and floating reduction explicitly; per-probe
  infinity norms alone do not enclose a full RMS metric.
- Independently qualify Gate M (boundary tails, equilibrated flux,
  whole-space properties); do not infer it from Gate Q.

## Open decisions (must be signed and independently reviewed)

1. Original full-domain source and geometry identification.
2. Certified HMM global-solve/assembly/recovery forward bounds.
3. Admissible hold-out datasets, units, condition-number budget.
4. New future-only numerical acceptance thresholds and fallback behavior.
5. Reviewer approval, phase ownership and written freeze authorization.

Statuses: T/R PROPOSED_NOT_FROZEN; historical Stage 4/5B Gate Q FAIL;
AC-P1 PENDING; PH-08 NOT_FROZEN; Rust FEM NO_GO; Gate M NOT_VERIFIED;
physical error BOUND_NOT_CERTIFIED.
