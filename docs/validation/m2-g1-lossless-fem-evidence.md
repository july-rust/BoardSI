# BoardSI M2 G1 — isolated lossless 2D P1 FEM prototype evidence

**Date:** 2026-10-15 KST. **Status:** Working, numerically cross-checked **restricted experimental prototype**; not integrated into the product and NOT physically validated for real PCB.  
**Feature PR:** [#34](https://github.com/july-rust/BoardSI/pull/34) (DRAFT, stacked on G0 [#33](https://github.com/july-rust/BoardSI/pull/33)). **Measured tested source commit:** 7b858f3cdbcacb6765e6f6e5ee0a0c18a97f1fe3.  
**Final recorded CI for that source:** [M2 isolated lossless FEM run 38653846512](https://github.com/july-rust/BoardSI/actions/runs/38653846512) **PASS** (Rust 1.75 build, 5 Rust tests, Clippy -D warnings, independent Python ABCD comparison). Document-only later commits do not alter those physical execution results.

## 1. Source and exact accepted scope

- Rust P1 **triangle finite-element** weak form, piecewise homogeneous nonmagnetic nondispersive epsilon, one explicitly PEC rectangular signal, continuous bottom PEC reference, 0 V top and side PEC walls. Direct sparse accumulation into CSR and Jacobi-preconditioned conjugate-gradient linear solve; charge/energy internal consistency.
- Compute dielectric C, identical-geometry vacuum C0, exterior idealized inductance Lext = mu0*epsilon0/C0, characteristic Z0=sqrt(Lext/C), and per-metre delay sqrt(Lext*C). **No internal conductor inductance**. No assumption of real copper R=0 or real dielectric G=0: model **refuses** a non-PEC/non-lossless request.
- Optional analytic **lossless mathematical** 2-port S at +jωt, equal positive real port reference, one uniform straight idealized section; independent Python ABCD/complex-S validation. Neither full-wave propagation nor physical frequency applicability is certified.
- Code: [isolated Rust prototype](../../prototypes/m2-lossless-fem/), [independent Python S check](../../tools/validation/m2_s2_analytic_check.py). G0 [pre-registered acceptance](m2-g0-acceptance-protocol.md), [Python FVM reference](../../tools/validation/m2_fvm_reference.py), and [unrounded oracle data](../../tools/validation/m2_g0_holdout.csv).
- No other structures/ports/materials have passed. Not a production PCB model; copper resistance, dielectric attenuation, multilayer stacks, physical open-space boundaries, via/pad, reference switching, bends, differential and 3D effects remain Unsupported.

## 2. Independent results and pre-registered thresholds

Specimen (entirely synthetic): width 0.3 mm, conductor thickness 0.05 mm, dielectric height 0.2 mm, epsilon_r=3.4, PEC signal/reference, explicitly zero-loss nonmagnetic dielectric, 0 V grounded PEC box of half-width/top height 1.2 mm. Mesh n=32 subdivisions per dielectric height. Differences against independent **cell-centered FVM** (separate field formulation and SciPy sparse direct solve, not a second Rust call).

| Property | Observed Rust FEM | Independent FVM | Difference | Pre-registered G1 limit |
| --- | ---: | ---: | ---: | ---: |
| C | 8.9251239079867028e-11 F/m | 8.842386680772379e-11 F/m | **0.935689%** | <=2.0% |
| C0 | 3.6708540153695572e-11 F/m | 3.654801591437518e-11 F/m | **0.439215%** | <=2.0% |
| Lext | 3.0310386939834007e-7 H/m | derived independently from FVM C0 | diagnostic comparison only | no separate universal field tolerance |
| Z0 | 58.27584952131 ohm | FVM 58.67628954408 ohm | diagnostic comparison only | no universal physical field tolerance |
| delay | 5.201191778208533e-9 s/m | FVM derived | diagnostic comparison only | no universal physical field tolerance |

**FEM self mesh 16 -> 32:** C **0.575816%**, C0 **0.328017%**, both within pre-registered 1.5% tolerance. Mesh order P1 and uniform corner resolution only; not yet a full adaptive-meshing accuracy certificate.

**FEM PEC-box sensitivity B=1.6 -> 2.4 mm, n=12:**
C **0.333827%**, C0 **1.168449%**, Lext **1.154954%**, Z0 **0.744687%**, delay **0.413345%**; each within pre-registered 2.0% box sensitivity tolerance. All side/top boundaries remain grounded PEC, and this does NOT establish open-air microstrip accuracy.

**Charge/energy consistency:** relative difference **5.00710584e-14**; field/vacuum true relative linear-system residuals **9.58488576e-12** and **9.49959068e-12**, respectively. No silent regularization, pseudo-inverse, or passivity clamp.

**Independent complex S mathematical comparison:** at f=1 GHz, length 0.01 m, 50 ohm equal real references, full physical 2-port S from a separate Python ABCD formula (not the Rust scattering routine) gives absolute S11 discrepancy **1.10546e-14**, S21 **3.00397e-14**. By reciprocity and a symmetric line, both directions S12/S21 and S22/S11 were checked. This is mathematics of a *given* C/L, not proof that an actual PCB is lossless or quasi-TEM accurate at 1 GHz.

## 3. Measured resource evidence

Single GitHub Actions Ubuntu 24.04 Linux x64 run, Rust 1.75.0 release optimized build, **n=32, B=1.2 mm**, two field solves (material and vacuum). Environment logs pinned by [CI run 38653846512](https://github.com/july-rust/BoardSI/actions/runs/38653846512).

| Metric | Observed result |
| --- | ---: |
| Conductor/field unknowns (each RHS) | **72,712** |
| P1 triangular elements | **147,456** |
| Assembled nonzero sparse entries | **362,296** |
| PCG iterations material/vacuum | **846 / 878** |
| Solver wall time material / vacuum | **0.546285 / 0.534045 s** |
| External /usr/bin/time executable wall | **1.08 s** |
| Peak process resident memory | **50,240 KiB** |

Results are process-/runner-specific. Do not compare resource data directly with the FVM n=48 experiment as if workload and method were identical. No other OS/CPU native FEM benchmark executed. No new workers/UI runtime operations executed.

## 4. Explicit status ledger (not artificially promoted)

| PH ID | Actually executed evidence | Honest scope |
| --- | --- | --- |
| PH-01 | Ideal S complex ABCD comparison and mathematical energy/passivity tests | **Partial PASS** for one ideal mathematical case; full PH frequency/length/port sweep Not Run |
| PH-03 | Independent edge-free parallel plate analytical values from G0; no exact edge-free Rust FEM boundary coupon | **Not Run** for FEM exact plate anchor |
| PH-04 | One synthetic grounded-box microstrip vs separate FVM C/C0 | **Partial PASS**; width/height/thickness/Dk parameter sweep Not Run |
| PH-17 | n=16→32 uniform FEM self-mesh and n32 FVM comparison | **Partial PASS**; corner adaptive and order study Not Run |
| PH-18 | Grounded PEC enclosure width/top sweep, one boundary type | **Partial PASS**; open/Neumann alternative boundary Not Run |
| PH-19 | Positive C/C0, energy-charge equality, true residual, analytic lossless S unitarity/reciprocity | **Partial PASS**; general matrix eigenvalues/ill-conditioning not studied |
| PH-20 | Deterministic rejection of nonfinite/negative dielectric, lossless not specified, non-PEC conductor, broken reference, nonuniformity and copper/box collision | **Partial PASS**; comprehensive invalid combinations Not Run |

Five Rust tests were executed (3 unit + 2 integration), no failed tests, Clippy with -D warnings **PASS**, plus separate Python S benchmark **PASS**. The existing M0 boardsi-numerics, boardsi-model, boardsi-ipc, Worker and Qt were not modified or rerun. Full product T-tests remain Not Run; Issue #16 remains Open. Do not set ModelStatus::Validated for any real-board result or claim a 30 GHz validated physical frequency band.

## 5. Reproduction

Run from repository checkout with Rust 1.75.0 and no external Rust dependencies:

- Cargo test: cargo +1.75.0 test --manifest-path prototypes/m2-lossless-fem/Cargo.toml --release -- --nocapture
- Clippy warnings: cargo +1.75.0 clippy --manifest-path prototypes/m2-lossless-fem/Cargo.toml --all-targets -- -D warnings
- Example: cargo +1.75.0 run --manifest-path prototypes/m2-lossless-fem/Cargo.toml --release -- 32 0.0012
- Analytic comparison: save the last command's standard output JSON and run python tools/validation/m2_s2_analytic_check.py output.json
- Separate FVM oracle: follow [G0 execution instructions](m2-g0-acceptance-protocol.md). Product Rust code intentionally does not import SciPy.

Keep PR #34 draft, stacked until G0 PR #33 is independently reviewed. Neither PR is auto-merged. Future M2 G2/G3 loss-related models require entirely separate field evidence and independently approved conditions.
