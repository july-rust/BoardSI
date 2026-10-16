# PH-08 Gate Q — independent exact-arithmetic qualification proposal (NOT FROZEN)

**Status:** PROPOSED / NOT FROZEN (Issue #51). **Historical Stage 4 Gate Q:** FAIL. **AC-P1:** PENDING. **Rust FEM:** NO-GO. **Stage 5B original:** NOT RUN.
**Source archive:** historical Stage 4 private ZIP SHA-256 `6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9`; the ZIP is not included in this repository. No new original-HMM run is claimed here.

## Separation of evidence

1. The Stage 4 **original** (PR #54, tracked in Issue #51) preserved 36/36 original MMS rows and the preregistered FAIL: uniform/quadratic/bulk gradient RMS n=8/16/32 = 4.0439406949e-15 / 4.2411270339e-15 / 9.2366452269e-15, ratios 0.953506/0.459163 versus historical threshold 1.8. Historical verdict is immutable.
2. This PR implements a **separate independent nine-site polynomial LS oracle**; it does not execute original HMM `solve` or `Recovery`. Its exact arithmetic conclusions cannot be transferred to original HMM without an explicitly reviewed bridge on identical sites, data and reconstructed gradients.
3. Stage 4 uniform original HMM potential offset was observed approximately -h^2/6, constant across cells to numerical precision; this is **an observation**, not a proven universal PDE discretization property. A deliberately added constant -h^2/6 is tested synthetically as an exact-invariance example, not a proof of the original solver error.
4. Gate M (energy bracketing, flux equilibration, boundary and floating error) is entirely separate.

## Independently defined synthetic oracle

For each of uniform/skew/graded, n=8,16,32, use a nine-node 3×3 normalized stencil with coordinates (a,b), centered at rational (x0,z0)=(2/5,1/3), h=1/n. The normalized LS columns are (1,a,b,a²,ab,b²); solve exact normal equations using Python `fractions.Fraction` Gaussian elimination. The nine samples are exact rational evaluations of quadratic q=x²+z² or cubic c=x³+z³. Calculate (coefficient_a/h, coefficient_b/h) and compare with analytically exact gradients. All site positions and data are exact rationals. With `numpy==2.3.5`, solve an independent double-precision LS via `numpy.linalg.lstsq`, report normalized matrix 2-norm condition, rank, residual and absolute gradient error; with `mpmath==1.3.0` at 100 decimal digits, solve high-precision equations and compare to Fraction-based oracle. This is an **independent interpolation diagnostic**, not independent FEM/PDE solver evidence.

Grid construction:
- Uniform: (a,b)=(i,j).
- Skew: (a,b)=(i+j/5, j+i/7).
- Graded: (a,b)=(i+i²/10, j+j²/12).
- i,j∈{-1,0,1}; these prescribed geometries are **synthetic templates**, not original Stage 4 physical meshes.

Because quadratic samples are exactly representable in the six-term polynomial *as rational mathematical quantities*, the exact rational recovered gradient must be exact (all 9 cases). Adding any spatially constant potential offset, including -h²/6, changes only the constant basis coefficient in exact arithmetic. For cubic, the normalized stencil is scale invariant, so the exact recovered gradient error is proportional to h² provided its coefficient is nonzero; the *squared error* must decrease by exactly 16 under h halving (gradient error factor 4). All 9 cubic errors and both reductions for each shape are independently checked. These are derivations for the stated synthetic oracle, not observed HMM convergence guarantees.

**Important numerical nuance:** a small RMS near 1e-15 alone is not a mathematical proof of IEEE-754 error; the exact Fraction oracle and independently bounded evaluation/recovery errors are required. The double-precision and 100-digit discrepancies and conditions are diagnostics only.

## Prospective two-regime Gate Q acceptance design

**Entry prerequisites (both regimes)**: frozen prospective data set, exact HMM implementation identity/hash and independent F-class reference, recorded units, mesh definitions and coordinate scaling, software/library versions, strict nonfinite rejection, source provenance, and independent review approval. A missing prerequisite is NOT VERIFIED, never PASS. Pre-register all thresholds and the crossover criterion **before** candidate data are obtained.

### Regime T — truncation dominates

Use independently established absolute gradient errors E(h), not solely floating residuals, over at least three successive resolutions per material/geometry/grid class. Require a pre-registered absolute error ceiling and pre-registered lower bound on E(h)/E(h/2) (historical protocol was 1.8 and remains FAIL for Stage 4). No convergence-rate decision if the denominator is near the validated floating uncertainty. Where no true truncation regime is demonstrated, report INDETERMINATE rather than waive a failure.

### Regime R — roundoff dominates

Allow a possible classification only when: (i) an independent exact/high-precision oracle proves the *specific* polynomial gradient is reproduced by the mathematical recovery operator, (ii) a **certified**, conservative upper bound B(h) on input quantization, matrix formation and floating QR/linear solve error is derived for the *actual* implementation (with outward-rounded intervals or an independent validated a posteriori solver error), (iii) the measured absolute gradient error E(h) <= B(h) with consistent units and known physical scaling, and (iv) mutations exceeding a pre-registered sensitivity budget are reliably rejected. The crossover requires a proved bound on the truncation component, e.g. E_trunc,upper <= B_round,lower or another rigorously justified pre-registered dominance criterion. An estimated floor or a small observed error cannot trigger the classification.

**Sufficient bound template, not yet numerically certified:** for exact 6×6 normal system Hc=b (H=AᵀA, b=Aᵀy), let computed coefficients c_hat satisfy a residual r=b_hat-H_hat c_hat; let rigorous bounds η_H >= ||H_hat-H||₂ and η_b >= ||b_hat-b||₂ be proved, with ||H⁻¹||₂ η_H < 1. The perturbation identity yields

||c_hat-c||₂ <= ||H⁻¹||₂ /(1-||H⁻¹||₂η_H) [η_b+η_H||c||₂+||r||₂].

The relevant gradient bound also requires division by h, reference gradient evaluation uncertainty and any coordinate-transform error. Terms η_H, η_b and ||r|| must be bounded with **outward rounding**; a float64-computed residual alone is not a rigorous bound. Normal equations can square the condition number, so this formula is only a safe mathematical template; it is not permission to replace the actual QR solver's error model with a wishful threshold. Certified interval or exact rational evaluation of the relevant quantities is required.

**Current status:** the synthetic exact/high precision oracle and mutation tests can establish reproducibility and detect some errors; they do **not** provide all outward-rounded bounds for original HMM or certify a Regime R crossover. Do not claim newly accepted Gate Q from a green CI run.

## Mutation/negative controls

Reject injected constant gradient shift, sign reversal, factor error, missing 1/h basis scale, one corrupted quadratic sample (detected via exact residual and gradient), NaN/Inf, missing/extra samples, missing or incorrect evidence SHA-256 and unsupported grid/polynomial/precision. Positive checks cover exact quadratic reproduction, constant-offset invariance, exact cubic scaling, full-rank finite float64, and 100-digit-vs-exact agreement.

These mutations are intentionally detectable in this **oracle**, not measured sensitivity on original HMM; eventual HMM bridge must independently demonstrate fault detection on the production recovery pathway.

## Original HMM bridge — absent/required

- Obtain verified original Stage 4 input source and agree on a redistribution-safe CI mechanism (e.g., private workflow with approved encrypted artifact access), compare identical site coordinates, weight matrices, boundary data, recovered coefficients, gradients and residuals with exact/high-precision calculations.
- Separately qualify uniform/skew/graded HMM quadratic and cubic, including grid-dependent LS stencils, condition values and solver error, at n=8,16,32 or broader **preregistered** grid set. Do not silently repurpose nine-site surrogate results.
- Stage 4 original historical FAIL remains, even if a **new future protocol** is approved.
- The Stage 5B original archive has no independently established external SHA-256 and must remain NOT RUN until obtained and audited.

## Gate M blockers and Rust FEM GO condition

A Gate Q numeric success **does not** validate Neumann/Dirichlet energy enclosure sign/monotonicity for the chosen PDE boundary and finite domain, flux equilibration, finite precision outward bounds, domain truncation and far-field energy, or certified C/C0 enclosure. Require: original solver evidence and independent physical oracle, preregistered Gate Q acceptance and independent review, demonstrated Gate M interval bounds under validated conditions, AC-P1 approval, PH-08 contract freeze and a separate signed Rust FEM implementation PR. No 61-setting physical rerun and no new FEM solver in this PR.

## Reproduction and output provenance

```sh
python -m pip install numpy==2.3.5 mpmath==1.3.0
python -m unittest discover -s tools/validation -p 'test_ph08_gateq_independent.py' -v
python tools/validation/ph08_gateq_independent.py --output-json /tmp/ph08-gateq-synthetic.json
```

The optional GitHub Action pins dependencies, records source SHA-256 and package versions, publishes synthetic JSON and checksum manifest as workflow artifacts and never uploads proprietary/private ZIPs. The optional job does not alter 17 required CI checks. A green job only asserts these tests pass; physical acceptance remains NOT FROZEN.

Review before freezing: verify QR/numerical perturbation constants, polynomial sample model vs original HMM, loss/ε units, strict tolerance preregistration, and independent mutation sensitivity. Any unresolved defect gets a separate tracked issue.
