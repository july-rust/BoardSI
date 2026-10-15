# M2 validation audit hardening — Issue #41

Date: 2026-10-15 KST. Baseline `c6dce055bfe7fc3a22b169b159c5dadaab0da3c8`, merged PH-07 PR #40. No PH-08, G5 or product integration is implemented here. All preregistered acceptance contracts and their tolerances remain unchanged.

## Findings and remedies

| Finding at baseline | Correction / evidence |
| --- | --- |
| PH-07 FVM trusted Rust JSON geometry/material/mesh; fixture hash did not bind each run | `m2_ph07_inputs.py` independently defines all 44 named witnesses. Validate ALL before solving; reject missing, duplicate or unknown cases; construct FVM input from the manifest. Width/thickness, layers, mask, air, B/H, electrodes, boundaries and mesh topology are checked. |
| Analytic plate described but not actually implemented in FVM self-test | Same four-face assembly now solves 9 exact uniform/two-layer plates at n8/16/32: C, phi, Ex/Ez, Dn and extrapolated interface potential, residual, charge and energy checked. |
| Sampled fields / region energies emitted without enforced comparison | Check positivity, eps bounds, derived identities, charge signs, energy partition, finite samples and regions. Four n128 base cases compare nonsingular phi/Ex/Ez against FVM; all other meshes retain error diagnostics. |
| `740 PASS` could imply 740 independent oracle points | Original 740 checks are retained. Primary comparisons: **8 C/C0** at A-D n128 (the four C0 share identical vacuum geometry); **8 Z0/delay** are derived. Residual, symmetry, property and repeated mesh/boundary checks are separate internal/convergence evidence. |
| FVM per-case RSS was the cumulative high-water of a shared process | Fresh subprocess for each of 36 distinct coupon inputs; 44 witnesses include 8 reused equivalents. Coupon peak RSS includes interpreter imports and both dielectric/vacuum solves. Per-phase high-water is labelled through that phase, not incremental allocation. CPU user/system and wall time recorded. |
| Physical workflows lacked main-push runs and had path filters | Numerical, Qt and PH-03/05/06/07 workflows run on every PR and push to main, with stable job names; new four-platform native FEM matrix. No path-filter-dependent required checks. |
| New stand-alone FEM lacked Windows/macOS execution evidence | Native matrix verifies runner architecture, rustc host AND executing Rust binary architecture/OS. Executes debug/release tests and CLI plate/stripline/asymmetric/A-D n128 coupons. Full FVM recomputation runs on Linux; native results compare recorded independent n128 FVM C/C0. |
| Worker test assumed cancel must return BrokenPipe after child exit | Reproduced baseline assertion failures. Separate deterministic locally closed-input case from real OS-pipe case, where either accepted write or BrokenPipe must preserve delayed Completed/EOF. Only test code changes. |
| Stale Draft / no solver / unverified final-head wording | PR #40 final HEAD `875f314aad0841300cfb1544e181a02e3ccb03de`, [CI 38674727369](https://github.com/july-rust/BoardSI/actions/runs/38674727369), merged baseline recorded in all status documents. |

## Code review scope and limitations

Reviewed all PH-03/05/06/07 geometry builders and field extraction, common P1 element matrix, CSR duplicate accumulation, diagonal guards, CG and recomputed true residual, energy and conductor reactions, quasi-TEM identities, input/nonfinite/grid-count guards. Added exact P1 matrix/constant-nullspace and 2x2 CSR/CG tests and PH-07 PEC-exclusion/natural-boundary topology tests. Isolated Rust sources are formatted to enable a real fmt gate; mathematical solver code is otherwise unchanged.

All excitation is 1 V signal / 0 V reference. Positive signal reaction, negative ground reactions and charge conservation are checked. PH-03 sides and PH-05/06 sides are natural Neumann; PH-07 top/sides are natural Neumann, never fictitious reference electrodes. PH-07 excludes PEC cell triangles and gives no independent interior DOF. Legacy PH-05/06 retain triangles between prescribed equal-1-V PEC nodes: their gradient/energy/reaction contribution is zero, not an independent dielectric field solution; this audit preserves that representation. Conforming P1 shares interface nodes; weak residual and independent conservative FVM/analytic layered plates test integrated flux, not singular-corner pointwise flux.

No demonstrated production FEM physics defect was found within these restricted coupons. The confirmed defects are missing/overstated validation and a nondeterministic Worker test expectation. Mesh/roundoff error is not real-PCB uncertainty.

## Executed local evidence

Linux x86_64, Rust 1.75.0, NumPy 2.3.5 / SciPy 1.17.0. Local resource wrapper uses Python child `getrusage` because GNU time is absent; GitHub Ubuntu jobs use `/usr/bin/time -v`. Native CI remains the authority for other platforms.

| Execution | Observed result |
| --- | --- |
| PH-03 exact analytic regressions | PASS n8/16/32 |
| PH-05 independent FVM | PASS 81 checks |
| PH-06 independent FVM and invariants | PASS 201 + 57 checks |
| PH-07 original + audit checks | Local pilot PASS 1756, FAIL 0; final source rerun recorded in CI |
| Input mutations | PASS 1108 rejected mutations across 44 real FEM JSON files, plus missing-key and unknown-case controls |
| FVM exact plate self-test | PASS 9 coupons; max C relative error 1.71e-14, max phi error 6.00e-15 V, max normalized E error 3.21e-14 |
| Baseline Worker stress | Sequential 100/100; parallel 490/500, 10 identical assertion failures |
| Corrected Worker stress | Sequential 100/100; parallel 500/500; 0 failures |
| Rust FEM, M0 workspace, fmt and strict Clippy | Local PASS; final HEAD CI required |

Pilot extra all-mesh field criterion of 0.02 V failed at n32 A (0.0222448997 V) and C (0.0228874662 V). Those coarse-grid differences are retained in `sample_diagnostics`, not labelled as satisfying that bound. Field acceptance uses the same A-D n128 cross-method comparison scope as the frozen contract: maximum potential difference <=0.02 V; max component E difference normalized by 1/h <=0.10. This is an additional audit check, not a modified contract tolerance. Measured n128 maxima: phi 0.00424079864 V; normalized Ex 0.00701753687; Ez 0.00801263163. Coarse/fine diagnostics and the original C/C0 mesh-convergence gates remain available. Field samples exclude PEC corners and stencils crossing material interfaces; they do not validate exhaustive local peak fields.

Maximum n128 primary FEM/FVM C/C0 relative difference: **0.009767283844 (0.9767283844%)**, case C capacitance; original limit 3%. C0 maximum 0.4116548584%, derived Z0 maximum 0.6890148847%, delay maximum 0.2809836958%. These are numerical discretization comparisons only.

## Native platform and final CI ledger

Native matrix and final PR HEAD results are populated after the corresponding Actions finish. Until then Windows x64, Windows ARM64 and macOS ARM64 new FEM runs are **Not Run**, not inferred from earlier M0 success. Linux local execution above is actual native execution. Full FVM is not claimed on non-Linux hosts.

## main protection / administrative constraint

Observed active [Protect main ruleset 25046595](https://github.com/july-rust/BoardSI/rules/25046595): only `docs` required; strict update=false; approving reviews=0; review-thread resolution=false; no bypass actor. The connected GitHub tools expose ruleset GET but no ruleset update operation; administration writes cannot be made through this connection. No alternate credential or ruleset bypass is attempted. A reviewable desired payload is in `m2-audit-protect-main-proposed.json`; application remains an administrative follow-up and Issue #41 must remain open until resolved. Proposed one approving review requires a genuinely separate reviewer; automated self-review is not substituted for it.

Code/CI completion and administrative enforcement are separate statuses. Merge only after all actually executed final-head checks pass and no numerical blocker remains. PR #35 stays unmerged, Issue #16 stays open, and actual PCB/loss/full-wave/30 GHz validation stays Not Run.
