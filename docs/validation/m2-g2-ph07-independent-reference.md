# PH-07 independent FVM numerical reference

**Date:** 2026-10-15 KST. **Acceptance frozen first:** commit `c11440a1fa2a1401d7a7294217aa5697a2722990` ([contract](m2-g2-ph07-acceptance-contract.md)). All numerical values below come from actual [GitHub Actions run 38674095933](https://github.com/july-rust/BoardSI/actions/runs/38674095933); this is a restricted ideal-PEC synthetic 2D electrostatic coupon, not real-PCB/full-wave/30 GHz/loss validation.

## Independent numerical formulation

[Python cell-centred FVM source](../../tools/validation/m2_ph07_multilayer_microstrip_reference.py) constructs geometry from explicit metres-valued input: x/z cell centres, one region per cell (air, substrate 1/2, mask, metal exclusion). Four-face conservative flux matrix, harmonic epsilon at dielectric/dielectric interfaces, half-cell distances to signal PEC=1 V and bottom GND=0 V, zero-normal flux top/sides. Solve sparse matrix by **SciPy spsolve direct** rather than Rust P1/CSR/CG. Compute independent signal and bottom GND charges and C, repeat with every epsilon_r=1 for C0. Rust FEM field/potential/capacitance is never used as an FVM numerical input or target; explicit geometry metadata is shared to enforce identical physical setup.

Geometry: copper x ±0.15 mm, z 0.20–0.25 mm; ground z=0; domain B=1.20 mm H=0.80 mm. PH-07A single εr=3.4 substrate 0–0.20 mm, mask OFF; B same substrate with mask ON; C two substrate layers 0–0.10 mm εr=3.4 and 0.10–0.20 mm εr=4.0, mask OFF; D two layers, mask ON. Mask ON is a union of 25 μm exposed-substrate blanket, copper side and top coating, εr=3.3, air elsewhere. No fabricated boundary top ground.

Recorded Python 3.13, NumPy 2.3.5, SciPy 1.17.0, Rust 1.75, ubuntu-24.04. **Source SHA-256** `7e896050de8a0c220ee3c6132c49d452bb02fe4a338feb571c8bd75a5a1afd9a`. **Canonical fixture SHA-256** `e85f99cba980cb9eb3eea0db436ca6d3a8a6b2026f5c1fcdfe7fa2e3f2b35489`. Final CI explicitly checks both pinned hashes.

## Independently calculated at n128, B=1.2 mm, H=0.8 mm

| Case | C FVM (pF/m) | C0 FVM (pF/m) | Z0 FVM (Ω) | Delay FVM (ns/m) |
| --- | ---: | ---: | ---: | ---: |
| A | 86.3034973531 | 33.6251585682 | 61.920282471 | 5.343936934 |
| B | 93.6663880727 | 33.6251585682 | 59.436775405 | 5.567228071 |
| C | 93.8292836370 | 33.6251585682 | 59.385159361 | 5.572066962 |
| D | 101.1889103837 | 33.6251585682 | 57.184807943 | 5.786468406 |

These Z0 and delay values and Lext=MU0*EPS0/C0 are **mathematically derived from independently solved C,C0**, not independent AC magnetic FEM. All four cases had n32/64/128 refinement, side B=1.2/1.6/2.4 mm, top H=0.8/1.2/1.6 mm at fixed dx, mask thickness 12.5/25/50 μm, mask εr=1/3.3/4, and substrate swaps/equal/vacuum. Full FVM charge, true residual, wall time, memory and field comparisons are in `ph07-numerical-evidence` artifact (14-day retention) of CI run. **740/740 initial independent gates PASS**; verify final PR HEAD before approval.

Unverified: measured boards, real solder-mask coating shapes, PEC skin/proximity losses, dielectric loss/dispersion, 3D return current, full-wave and physical 30 GHz.
