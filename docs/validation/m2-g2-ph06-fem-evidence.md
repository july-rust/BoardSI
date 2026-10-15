# BoardSI G2 PH-06 — restricted asymmetric FEM acceptance evidence

**Run date:** 2026-10-15 KST. **Independent verification:** [Actions run 38669970099](https://github.com/july-rust/BoardSI/actions/runs/38669970099), 201 direct FEM/FVM acceptance checks **PASS** and 57 supplementary geometry/material/reflective invariants **PASS**. Frozen PH-06 Acceptance Contract SHA: `15da48ef3753e361d009decd03dbd4516dfd9868`, *before* new FEM and FVM source. **Approval scope is limited to a synthetic 2D electrostatic PEC/lossless quasi-TEM coupon.** Full M2, measured PCB, full-wave and 30 GHz accuracy remain **Not Run**.

## Physical model and independent checks

Propagation y; x/z 2D, PEC trace w=0.30 mm, t=0.05 mm, zc=0.30 mm; grounds z=0 and z=0.80 mm, giving **lower gap 0.275 mm, upper gap 0.475 mm**. Lateral x=±B: homogeneous natural Neumann with B=1.2/1.6/2.4 mm. PH-06A εr=3.4 both sides. PH-06B εr_bottom=3.4, εr_top=4.0, horizontal interface z=0.35 mm, mesh-conforming. Signal voltage 1 V, both GND 0 V.

The [new Rust PH-06 module](../../prototypes/m2-lossless-fem/src/asymmetric_stripline.rs) has its own geometry and material constraints; [existing lib.rs](../../prototypes/m2-lossless-fem/src/lib.rs) shares the original triangle stiffness, sparse CSR and CG iterative solver using a material-row callback. PH-05's public API and centered homogeneous solver/CLI remain compatible. Every triangle uses the correct region εr; no smearing across interface. Top/bottom ground free charges are separate Dirichlet reactions and *never* symmetrized. PEC internal fields zero. Positive conductor free charge and field energy integrate separately, and full charge conservation is enforced.

[Separate Python FVM reference](m2-g2-ph06-independent-reference.md) uses cell-centered, face-based conservative flux with harmonic series ε at the interface and SciPy sparse direct solve, not Rust FEM or its output as input. Full-geometry reflectivity tests compare FEM scalar outputs and swapped physical electrode charge. PH-05 recovered centered coupon is **regression**, not absolute-C external reference.

## Measured results (n=128, B=1.2 mm)

All values below were printed by **actual GitHub Actions** from the same physical geometry, in SI units. Rounded only for display.

| Metric | PH-06A FEM | PH-06A FVM | PH-06B FEM | PH-06B FVM |
| --- | ---: | ---: | ---: | ---: |
| C (pF/m) | 117.353652075 | 116.862304258 | 123.914484620 | 123.386465280 |
| C0 (pF/m) | 34.5157800222 | 34.3712659584 | 34.5157800222 | 34.3712659584 |
| Lext (nH/m) | 322.359817840 | derived from FVM C0 | 322.359817840 | derived from FVM C0 |
| Z0 (Ω) | 52.41096379 | 52.6313258 | 51.00460791 | 51.2209668 |
| Delay (ns/m) | 6.150618010 | 6.150618010 | 6.320209703 | 6.319974040 |
| εeff | 3.400000000 | 3.4 | 3.590082117 | between 3.4 and 4.0 |

**FEM vs FVM comparisons:** C/C0 discrepancy approximately 0.42% (both cases), Z0 discrepancy approximately 0.42%; delay A matches the exact homogeneous quasi-TEM identity, B differs by about 0.004%. The preregistered gate for each C,C0,Z0,Delay was <=2%. Lext is *not* independent magnetic-field validation. PH-06A `C/C0=3.4` is exact under its homogeneous, nonmagnetic idealization; PH-06B `εeff=C/C0` is not any single constituent permittivity.

## Physical non-symmetry and charge conservation (FEM, n=128)

| Metric | PH-06A | PH-06B |
| --- | ---: | ---: |
| Signal Q (pC/m for 1 V) | +117.353652075 | +123.914484620 |
| Upper GND Q (pC/m) | −43.6692025194 | −50.5717861793 |
| Lower GND Q (pC/m) | −73.6844495326 | −73.3426984266 |
| Normalized net charge imbalance | 1.99e−10 | 1.17e−10 |
| True CG residual | 9.83e−12 | 9.58e−12 |

**Interpretation:** The nearer bottom reference carries more free charge in both executed examples; bottom/top charges are *not* forced to equality. With upper ε increased in PH-06B, the top plane's magnitude increases. x mirror remains symmetric, while z mirror is *not* applied to the base result. Full geometric reflection moves the conductor and, for PH-06B, also reflects the interface and swaps dielectric assignments; top/bottom charges then exchange within the frozen mirror gate.

## Convergence, invariants, resource evidence

| Gate | Recorded result |
| --- | --- |
| Independent FVM/FEM n128 absolute C/C0/Z0/delay, each case | PASS within 2% |
| FEM mesh n32/n64/n128; n64→128 change C/C0 <=1.5% | PASS |
| Independent FVM n64→128 C/C0 change <=1.0% | PASS |
| B=1.2/1.6/2.4 mm n64 boundary sensitivity <=2% | PASS |
| True residual <=1e-8 and Energy/Charge <=1e-6 | PASS |
| 1-V PEC and 0-V GND error <=1e-12 V; x reflection <=1e-6 V | PASS |
| Distinct top/lower ground charge; total charge normalized <=1e-6 | PASS |
| PH-05 symmetric recovery and full PH-06A/PH-06B reflection | PASS, extra 57-gate sweep |
| zc=0.30/0.35/0.40/0.50 mm sweep | PASS |
| εbottom/εtop=3.4/3.4, 3.4/4.0, 4.0/3.4, 1.0/1.0 | PASS |
| Aligned equal-ε interface is physically neutral | PASS |
| Negative material/geometry/overlap/unsupported test cases | PASS in Rust units and PH-06 oversized CLI |
| Original PH-05 81 independent gates and G1/PH-03 reference checks | PASS on [Actions 38669726117](https://github.com/july-rust/BoardSI/actions/runs/38669726117) |
| Existing M0 Rust workspace tests | PASS on same existing FEM workflow |
| Rust strict Clippy `-D warnings` | PASS |

At n128 B=1.2 mm, FEM mesh has **49,665 nodes, 48,454 free DOF, 98,304 triangles and 241,130 CSR nonzeros**. Rust CG iterations: 732 (A) / 733 (B). Dielectric FEM solve time measured 0.309 s (A) / 0.307 s (B), per-solve measurement in release run. Entire Linux CLI process, including dielectric+vacuum field: elapsed 0.61 s (A) / 0.60 s (B), maximum RSS 32,720 kB (A) / 32,688 kB (B). These measurements are specific to the linked hosted Linux runner, not universal performance claims.

**Reproducibility:** See [independent PH-06 reference](m2-g2-ph06-independent-reference.md) and the `ph06-numerical-evidence` Actions artifact for raw JSON, source and fixture SHA-256, per-file hashes, gate observations, n/B sweeps and CPU/RSS. Artifacts have a 14-day retention period; preserve a copy for later audit if needed.

## Remaining limitations

- Full-wave 3D, via/pad discontinuities, surface roughness, finite copper conductivity, skin-effect R, dielectric conductance/loss G and material dispersion: **Unsupported**.
- Real PCB/imported stackup, port calibration, 30 GHz band applicability, measurement correlation and independent AC magnetic inductance: **Not Run**.
- PH-07 solder mask/multilayer, PH-08 alternative reference topology, G3/G4/G5 and overall M2: **Not Run**.
- CI warning from GitHub-hosted actions about Node.js compatibility is an **external action runtime notice**, not a Rust or numerical solver warning.
- No Qt GUI, Worker, IPC or Canonical Board Model source was modified; full Windows/macOS platform builds were intentionally **Not Run**.
- PR #35 and Issue #16 remain separate/open. Do not mark full M2 complete.

**Limited PH-06 verdict:** PASS for the executed synthetic 2D homogeneous/two-region ideal PEC electrostatic cases, subject to verifying required checks on the final merged code HEAD. Every unexecuted physics model is explicitly Not Run/Unsupported rather than assumed validated.
