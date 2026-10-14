# M2 G0 evidence — independent synthetic grounded-box field reference

**Evidence date:** 2026-10-15 KST. **Main baseline:** 65c397d99b2ebf8dc3e55dcdb0ef6e168be32342 (merged PR #31).  
**Decision:** Independently exercised **synthetic PEC-box reference** passes the pre-registered FVM mesh/domain criteria on one Linux x64 environment. The Rust FEM and physical PCB validity are NOT thereby verified. The *portable reference suite* independently reproduced in GitHub Actions: **PASS** ([focused CI run 38653297996](https://github.com/july-rust/BoardSI/actions/runs/38653297996)); the G0 reference is **READY ONLY for this restricted synthetic PEC-box coupon**, not for physical open-air or real PCB.

Sources: [frozen acceptance protocol](m2-g0-acceptance-protocol.md), [independent FVM implementation](../../tools/validation/m2_fvm_reference.py), [committed unrounded holdout table](../../tools/validation/m2_g0_holdout.csv), [independent-result checker](../../tools/validation/m2_g0_verify.py). Both synthetic inputs and newly authored implementation are redistributable. SciPy uses BSD-3-Clause. FVM is NOT FEniCSx/Ansys; do not misrepresent reference independence as commercial field validation.

## 1. Held-out FVM observations

All values below are per unit length for the artificial enclosure with ideal copper and nondispersive lossless dielectric. Mesh n means cells per substrate height. Unknown counts exclude metal-cell faces. Outputs are not rounded before the committed CSV.

| B (mm) | n | unknowns | C (pF/m) | C0 (pF/m) | Lext (nH/m) | Z0 (ohms) | delay (ns/m) |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1.2 | 12 | 10,314 | 87.53666 | 36.32643 | 306.29215 | 59.15248 | 5.17801 |
| 1.2 | 24 | 41,256 | 88.25916 | 36.51005 | 304.75176 | 58.76155 | 5.18624 |
| 1.2 | 48 | 165,024 | 88.57945 | 36.58181 | 304.15394 | 58.59765 | 5.19055 |
| 1.6 | 12 | 18,378 | 87.09940 | 35.73797 | 311.33552 | 59.78700 | 5.20741 |
| 1.6 | 24 | 73,512 | 87.81622 | 35.91615 | 309.79100 | 59.39461 | 5.21581 |
| 2.4 | 12 | 41,418 | 86.81374 | 35.33063 | 314.92503 | 60.22951 | 5.22875 |

**Pre-registered mesh acceptance:** B=1.2 mm, n=24 to n=48, maximum percentage deviations (relative to finer level):
- C **0.36159%**, C0 **0.19617%**, Lext **0.19655%**, Z0 **0.27969%**, delay **0.08291%**; all within <=1.0% criterion — FVM reference mesh **PASS**.

**Pre-registered box-size acceptance:** B=1.6 vs 2.4 mm at n=12, relative to larger box:
- C **0.32905%**, C0 **1.15294%**, Lext **1.13980%**, Z0 **0.73471%**, delay **0.40808%**; all within <=2.0% criterion — FVM box-size sensitivity **PASS**. This does NOT show exact open-space boundary convergence; top and sides remain modeled PEC walls.

FVM discrete relative residuals for material and vacuum range from 7.48e-16 to 2.00e-15, below 1e-9. Every sampled C/C0 is finite and positive, with 1 <= C/C0 <= 3.4; all derived Lext/Z0/delay are positive.

## 2. Independent mathematical anchors

- **PH-03 (edge-free plate, not a physical microstrip):** C=4.515635787588e-11 F/m; C0=1.32812817282e-11 F/m; Lext=8.377580408467e-7 H/m, from exactly declared w/h/er and eps0/mu0.
- **PH-02 (the FROZEN 50-ohm mismatched near/far ports):** f=1 GHz, length=0.01 m, Z0≈58.59765467 ohms from the B=1.2 mm, n=48 field reference, exp(+jωt). Expected **S11=S22≈0.01651839692653288+j0.048232430752762685** and **S21=S12≈0.9448265269029662−j0.3235793708615182**. Independently derived ABCD and multiple-reflection formulas must agree within 1e-12 with the 50-ohm basis and satisfy lossless power conservation; S11 is **NOT ZERO** for this declared reference. **PH-01** is the separate matched-to-Z0 analytic fixture: S11=0 and S21≈0.94728874331−j0.32038108059. Its Zref≈58.59765 Ω is explicitly not the frozen 50-Ω basis used in PH-02.
- PH-01/PH-02/PH-03 have independently generated mathematical anchors; full PH sweep and BoardSI product Rust physical qualification are **not** established here.

## 3. Runtime and environment (single observed holdout run)

- Platform: Linux x86_64, kernel 6.18.44, Python 3.13.5, NumPy 2.3.5, SciPy 1.17.0. Sparse direct solve; no OpenMP multithreading benchmark claim. Main benchmark used OPENBLAS_NUM_THREADS=1.
- Six material/vacuum pairs of field solves: **7.615 s observed total wall time** (Python process including mesh assembly and SciPy calls).
- Peak process resident RAM from Linux resource.getrusage(RUSAGE_SELF).ru_maxrss: **474,328 KiB** (measured, includes interpreter/SciPy; not equivalent to process-isolated solver-only RAM).
- Maximum unknowns: 165,024 (B=1.2 mm, n=48); actual residuals and solver timestamps are in observed output and committed fixture columns. No guaranteed low-resource production budget established.
- Frozen independent oracle source digest (SHA-256): **c5e05fed106cc78e22150b0c90a828951a439940bb1a8dc8843bd5eb73786929**. Its value is committed in [m2_g0_source.sha256](../../tools/validation/m2_g0_source.sha256); the focused CI runs sha256sum -c and the verifier recomputes the hash before G0 readiness. This is separate from variable run-output digests.
- Local experimental full JSON SHA-256: **82a8cc3c10bc04b116f279b11de20d22b5b32d96afee65a0ba51a373dd4ffc4a**; that hash refers to one **environment/timing-dependent local observation file**, not the deterministic committed CSV or future CI reproduction. The committed CSV and Git commit are the stable fixture references.
- Reproduce within the versioned PR branch: install Python 3.13, NumPy 2.3.5 and SciPy 1.17.0; run python tools/validation/m2_fvm_reference.py --suite holdout --output g0-reproduction.json, then python tools/validation/m2_g0_verify.py tools/validation/m2_g0_holdout.csv g0-reproduction.json. The focused workflow prints /usr/bin/time -v peak memory.

## 4. Actual PH status and model boundary

| Test | Evidence so far | Status for BoardSI Rust FEM |
| --- | --- | --- |
| PH-01 | Matched-to-Z0 mathematical S11=0, S21 traveling-wave/ABCD reference | **Not Run** against Rust FEM in this G0 source snapshot |
| PH-02 | Frozen 50-ohm mismatch, complex S11/S21 ABCD and multiple-reflection reference | **Not Run** against Rust FEM in this G0 source snapshot |
| PH-03 | Independent mathematical edge-free plate reference generated | **Not Run** against Rust FEM |
| PH-04 | FVM electrostatic bounded-box reference and numerical results | **Not Run** against Rust FEM |
| PH-17 | Independent FVM mesh refinement and preregistered threshold met | **Not Run** for Rust FEM |
| PH-18 | Independent FVM grounded-box-size sweep met | **Not Run** for Rust FEM |
| PH-19 | FVM positivity/residual checks, no Rust matrix/passivity test | **Not Run** for Rust FEM |
| PH-20 | Input checks in FVM, no Rust negative-case matrix | **Not Run** for Rust FEM |

**G0 decision limited to ideal-box reference**: after focused CI confirmation, suitable to start **an isolated Rust prototype** and compare **different** discretizations. Does not authorize a product claim of real-PCB lossless physics over any GHz band. Full-wave/measurement reference X, air-open boundary, finite conductivity, dielectric loss, vias/pads, differential and physical frequency envelope remain **Pending / Unsupported**.

## 5. Change safety

No changes to specification v1.0, existing numerics/model/ipc/Worker/Qt or the prior 180 mathematical cases. Issue #16 remains open. This G0 PR is intentionally not auto-merged. G1 work is staged in a separate branch/PR, not implicitly validated by this report.
