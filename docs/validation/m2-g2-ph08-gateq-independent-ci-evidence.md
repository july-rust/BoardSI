# PH-08 Gate Q independent synthetic oracle — observed GitHub Actions evidence

**Issue:** #51. **PR:** #55. **Initial exact source HEAD:** `5a4144ace651af56146cd8624fec22dc3afa4449` (GitHub Verified, parent `main@484b1953ff0d9e81836bea226e385dc7e8b5d42c`).
**Evidence run:** [38894594045](https://github.com/july-rust/BoardSI/actions/runs/38894594045), optional `ph08-gateq-independent (optional, synthetic only)`.
**Artifact:** `ph08-gateq-independent-synthetic`, GitHub Actions artifact ID **12042773820**, JSON `gateq-independent-synthetic.json`.

## Directly observed, reproducible results

- GitHub Actions `unittest`: **16 test methods PASS, 0 FAIL, 0 ERROR**, including positive and mutation-negative cases; each shape/resolution is also evaluated in loops/subtests.
- Deterministic independent case matrix: **18 rows** = uniform/skew/graded × quadratic/cubic × n=8/16/32.
- Exact rational quadratic recovered gradient error: **0 for all 9 combinations**. High precision at 100 decimal digits differs from Fraction fit by <1e-65 for all 18 cases.
- Exact rational cubic recovered gradient error reductions: **4.0, 4.0 per shape**; exact squared gradient error ratios **16, 16** per shape. This follows from exact polynomial projection on scale-invariant normalized synthetic templates, **not** original HMM PDE convergence.
- All Float64 fit matrices have rank 6. Conditions reported below are for the **independent 9×6 scaled LS matrix**, NOT original Stage 4 HMM.
- Environment: GitHub-hosted Ubuntu 24.04, Python 3.13.16, NumPy 2.3.5, mpmath 1.3.0; IEEE-754 binary64 unit roundoff 1.1102230246251565e-16.

### Quadratic — float64 absolute Gradient RMS difference to analytic gradient

| Synthetic grid | n=8 | n=16 | n=32 | normalized LS 2-norm condition |
| --- | ---: | ---: | ---: | ---: |
| Uniform | 2.482534e-16 | 7.850462e-17 | 7.850462e-16 | 4.243 |
| Skew | 1.032567e-15 | 8.950904e-16 | 8.082546e-16 | 5.178 |
| Graded | 6.473657e-16 | 8.345156e-16 | 9.155134e-16 | 4.381 |

The nonmonotonic floating error is **consistent with** roundoff; its numeric values **do not** prove a certified bound for the original HMM or justify changing historic Gate Q acceptance.

### Cubic — exact rational Gradient RMS difference to analytic gradient

| Synthetic grid | n=8 | n=16 | n=32 | exact rate (8/16, 16/32) |
| --- | ---: | ---: | ---: | ---: |
| Uniform | 0.0156250000 | 0.0039062500 | 0.0009765625 | 4.0, 4.0 |
| Skew | 0.0187930777 | 0.0046982694 | 0.0011745674 | 4.0, 4.0 |
| Graded | 0.0154926399 | 0.0038731600 | 0.0009682900 | 4.0, 4.0 |

## Attestation and traceability

The initial Actions artifact's `SHA256SUMS` lists:
- `tools/validation/ph08_gateq_independent.py`: `1f06fe52f59b0f892e67568e2f1896bffe7e340b0193d05d186bdf098ce93da2`
- `tools/validation/test_ph08_gateq_independent.py`: `f56e01ad4c2f96e58bd3a9a34297e1088b14ec26e12fcecd66056671c0d2dbe5`
- `docs/validation/m2-g2-ph08-gateq-independent-proposed.md`: `a0c72ab869bc766e45c9a1f38f8087d48bdab41ac88e5345fe92fd06507411ba`

**PR checkout SHA caveat:** The first artifact `github-context.txt` had `commit=e1a664b38d1dfba9157eaf3a45eb7dc4fe113ccd`, which is the GitHub Actions PR merge checkout SHA, *not* the signed PR source HEAD. The optional workflow is amended in this PR to record the PR head and PR test checkout SHA **separately**. Do not misreport the merge checkout SHA as the signed source revision.

## Fault-injection and negative tests

Positive: exact quadratic recovery, constant offset invariance (-h²/6 synthetic shift), exact cubic rate, full-rank finite float64 recovery, independent mpmath cross-check.
Negative: nonzero gradient bias, sign inversion, factor-of-two change, incorrect recovery basis scaling, one sample tamper (exact overdetermined residual detects), invalid sample count, NaN/Inf, nonfinite JSON, missing evidence file, bad SHA-256, unsupported settings. These controls operate on this **synthetic oracle**, not the original HMM acceptance engine.

## Explicitly NOT established

- Original Stage 4 HMM is **not** rerun inside the public Actions job; old private ZIP known external SHA-256 `6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9` stays private. Prior PR #54 already independently replayed the historical **FAIL** (uniform/quadratic 0.953506/0.459163 versus 1.8). Do not retroactively change it.
- There is no outward-rounded, rigorously certified floating original-HMM gradient error ceiling, no mathematically approved regime switch, no F-class original source oracle and no approval to freeze.
- The original Stage 5B ZIP external hash and raw bytes were **not found** in available verified sources. Stage 5B is NOT RUN.
- Gate M flux equilibration, energy bracketing assumptions, floating outward enclosures, far-field boundary error and C/C0 certification remain open.

**Physical disposition: Stage 4 Gate Q FAIL; AC-P1 PENDING; PH-08 Contract NOT FROZEN; Rust FEM NO-GO.** The synthetic tools can merge after required checks without granting physics approval. The complete signed head must be rechecked for all required/optional CI after any review correction.
