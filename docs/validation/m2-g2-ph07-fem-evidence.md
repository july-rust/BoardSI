# PH-07 multilayer microstrip FEM and actual field evidence

**Date:** 2026-10-15 KST. [Draft PR #40](https://github.com/july-rust/BoardSI/pull/40). The [PH-07 acceptance contract](m2-g2-ph07-acceptance-contract.md) was frozen **before solver implementation** at commit `c11440a1`. Measured results from actual [initial CI run 38674095933](https://github.com/july-rust/BoardSI/actions/runs/38674095933). Verify latest PR HEAD separately before claiming final approval/merge.

## Method

Finite rectangular PEC signal (0.30 mm wide, 0.05 mm thick; z=0.20–0.25 mm); bottom GND z=0 Dirichlet 0 V; signal 1 V; top H=0.8 mm and sides x=±1.2 mm natural zero-Neumann. Single substrate εr=3.4 (0–0.20 mm) or two layers εr=3.4/4.0 (0–0.10/0.10–0.20 mm). Solder mask ON adds explicitly bounded 25 μm rectangular blanket/side/top material εr=3.3; rest is air εr=1. Rust reuses P1 triangles, CSR CG sparse solver, per-triangle isotropic εr, reaction charge and field energy. No independent DOF/triangle material inside ideal PEC. Vacuum C0 separately solved on unchanged metal/outer geometry with εr=1.

C_energy=EPS0·integral(εr·|grad φ|²)dA at 1 V, C_charge=signal Dirichlet reaction; ground reaction independently aggregated. Region energy is a partition of total, NOT individual net capacitance per dielectric. εeff=C/C0; Lext=MU0 EPS0/C0; Z0=sqrt(Lext/C); delay=sqrt(Lext C). These latter terms are quasi-TEM derived, NOT directly solved R(f), Lint(f), G(f), or AC magnetic fields. Sparse sampled element-centroid (x,z,φ,Ex,Ez,material ID) values in final JSON evidence are diagnostic P1 fields, not exhaustive plots.

## n128 FEM vs independent FVM, B=1.2 mm H=0.8 mm

| Case | FEM C (pF/m) | FVM C (pF/m) | C relative error | FEM C0 (pF/m) | FEM Z0 (Ω) | FEM delay (ns/m) | FEM εeff approx. |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| A | 87.1043944021 | 86.3034973531 | 0.9280% | 33.7635781671 | 61.508485178 | 5.357659352 | 2.5798 |
| B | 94.1002812492 | 93.6663880727 | 0.4632% | 33.7635781671 | 59.177907341 | 5.568657725 | 2.7870 |
| C | 94.7457408831 | 93.8292836370 | 0.9767% | 33.7635781671 | 58.975986774 | 5.587723561 | 2.8062 |
| D | 101.6995669669 | 101.1889103837 | 0.5047% | 33.7635781671 | 56.924013421 | 5.789147515 | 3.0122 |

Physical trends independently confirmed: A→B and C→D mask ON increases C/εeff/delay, reduces Z0 and leaves C0 invariant; C/D upper dielectric εr=4 increases C vs lower-only A/B, unequal-layer swap is not a symmetry; larger mask thickness/εr monotonically increases C. All-vacuum C=C0, same-er artificial layer boundary invariant.

## Acceptance evidence ledger

| Criterion | Initial CI result |
| --- | --- |
| PH-07 numerical, independent FVM comparison, charge/energy, symmetry, mask/layer sweeps, n32/64/128 mesh and side/top boundary convergence | **PASS 740 / FAIL 0** |
| Rust unit tests, strict Clippy (-D warnings), M0 Rust workspace | **PASS** |
| Negative unsupported/invalid CLI inputs | **PASS 13 cases** |
| Existing PH-05 81 and PH-06 201+57 independent checks, G0/G1/PH-03 regression | **PASS in associated workflows** |
| Representative PH07-A n128 Rust CLI wall and RSS | **0.74 s, 34,344 kB** (Ubuntu runner; not platform guarantee) |
| Final HEAD pinned source SHA and sampled Ex/Ez field data | **Not Run in initial CI** — mandatory final CI follow-up |
| Real PCB or 30 GHz frequency accuracy, finite-conductivity/magnetic losses, PH-08/G3/G4/G5 | **Not Run / Unsupported** |

Actual per-case charges, solver residuals, iteration/DOF/triangle/NNZ, full masks/layers, CPU/RSS and FVM differences are stored in workflow `ph07-numerical-evidence` artifact. Failure in final HEAD gate blocks merging. Issue #16 remains Open. No overall M2 completion asserted.
