# PH-06 — preregistered asymmetric stripline acceptance contract

**Status: FROZEN BEFORE PH-06 FEM/FVM CODE AND ACCEPTANCE EXECUTION.**
Baseline main: `aca670fed127b634e5a8307ff73e06c60e13ee52`.
Scope: synthetic 2D scalar electrostatic quasi-TEM, lossless, rectangular ideal PEC and continuous reference planes only. Does not validate 30 GHz, real PCB, AC copper losses or 3D discontinuities.

## Geometry and fixture
Propagation axis y; field cross section (x,z). Signal x=[-0.00015,+0.00015] m, z=[zc-0.000025,zc+0.000025] m, 1 V. Bottom GND z=0 and top GND z=0.0008 m, both 0 V. x boundaries [-B,+B] with homogeneous natural Neumann; B=0.0012 m except boundary sweep (0.0016,0.0024 m). No forced z symmetry. Outer domain is **finite**.

PH-06A: zc=0.00030 m; epsilon_r_bottom=epsilon_r_top=3.4. Bottom gap=0.000275 m; top gap=0.000475 m.
PH-06B: same geometry, epsilon_r_bottom=3.4 (z<0.00035), epsilon_r_top=4.0 (z>0.00035), with horizontal conforming material interface z=0.00035 m, full lateral span. All materials isotropic, nonmagnetic (mu_r=1), strictly real, lossless. Vacuum C0 repeats **same exact geometry** with both epsilon values set to 1.0. If epsilon values are equal the material boundary remains a geometric/metadata boundary.

Mesh height subdivision n in {32,64,128}; dz=0.0008/n. All copper faces, side boundaries and interface must land on nodes/faces; reject any mismatch instead of snapping. Triangles cannot cross a material interface. Cell-centred FVM must keep PEC/ground face flux conservative and use harmonic dielectric face transmissibility (distance-weighted series epsilon) for different adjacent materials. Both solvers use consistent relative permittivity and zero-normal flux on sides. PH-05 remains separate and unchanged.

## Frozen tolerances and PASS gates
Relative discrepancy is |a/b-1|, with b=independent FVM and no tiny-denominator substitution for strictly positive C/C0/Z0/delay. Relative convergence is |fine/coarse-1| with strictly positive values. For mirror comparisons use |mirror/original-1|, for charge symmetry normalized to total |upper|+|lower|.

- FEM vs independent *separately discretized and directly solved* Python FVM: **<=2.0% each** for absolute C, C0, Z0 and delay, at n128 B=0.0012 m, **both** PH-06A and PH-06B.
- True relative linear residual <=1e-8 for all FEM/FVM solves used in acceptance; relative field-energy/conductor-charge discrepancy <=1e-6 for FEM.
- FEM 64→128 |C,C0| variation <=1.5%; independent FVM 64→128 |C,C0| variation <=1.0%, each material model. Record n32 too.
- Lateral B sensitivity at fixed n64, compared to B=0.0024 m: <=2.0% absolute C and C0 for B=0.0012 and 0.0016 m, each material model.
- x mirror potential max absolute difference <=1e-6 V, all principal FEM runs. Prescribed PEC and both GND voltage errors <=1e-12 V. Total charge imbalance |Qsignal+Qtop+Qbottom|/|Qsignal| <=1e-6 for FEM and FVM.
- PH-06A *must* show true asymmetry: |Qbottom| >= 1.05*|Qtop|, and max z-reflection potential error >=0.01 V at n32 B=0.0012 m; no artificial symmetrization. PH-06B direction is checked against independent FVM (at n128: signs and which plane has larger |Q| must agree), no unsupported universal 'nearest ground always' rule with unequal epsilon.
- PH-06A homogeneous identity C/C0=3.4 and delay=sqrt(MU0*EPS0*3.4), relative <=0.1%; PH-06B 3.4 <= epsilon_effective <=4.0 with small numerical allowance 1e-8; Lext=MU0*EPS0/C0, Z0=sqrt(Lext/C), delay=sqrt(Lext*C) to relative <=1e-10. Lext is *derived*, not an independently simulated magnetic field.
- Symmetric recovery zc=0.0004, epsilon_bottom=epsilon_top=3.4 matches PH-05 at n32 B=0.0012 in C,C0,Z0,delay <=1e-8 relative. PH-06A mirrored geometry zc=0.0005 matches its unmirrored counterpart in all four outputs <=1e-6 relative, while top/bottom ground charges exchange to <=1e-5 relative against original signal charge. PH-06B reflected zc=0.0005, interface z=0.00045, epsilon_bottom=4.0,epsilon_top=3.4 must have same scalar outputs <=1e-6 and exchanged ground charges <=1e-5 against original signal charge. Reflected material-swap alone without moving geometry is **not** a symmetry.
- Sweeps: zc=0.0004/0.0003/0.00035/0.0005 m (at n32), epsilon_bottom/top=(3.4,3.4),(3.4,4.0),(4.0,3.4),(1.0,1.0) at z_interface=0.00035 m. Check positivity, C monotonic under material increase for *identical geometry*, physical effective-epsilon bounds; only geometrically conforming sweeps are eligible. All requested cases must be PASS or explicitly documented Unsupported/Not Run.
- Negative tests: out-of-bounds/electrode shorts, negative gap, zero/negative/nonfinite width, thickness, height, coordinate, dielectric, half-span and voltage; mu_r !=1, non-PEC, lossy/conducting/dispersion, unsupported boundary, missing interface for layered material, interface out of domain or off-mesh, oversized grids. All reject with deterministic diagnostic, no panic.
- Record n32/64/128 physical values and per-case mesh nodes/triangles/DOF/nnz/iterations, true residual, charge and field metrics, wall time and /usr/bin/time -v Linux peak RSS. Freeze source/fixture SHA-256 in the actual evidence, not invented here.

PH-05's 81 gates, PH-03, G1 and M0 regression, Rust strict Clippy and final HEAD GitHub Actions must all PASS prior to merging; failures block merging even if PH-06 numerical gate passes. Evidence must explicitly distinguish PASS/FAIL/Not Run/Unsupported. Source/toolchain and fixture hashes are recorded at execution. No change to frozen tolerances after this commit; deficiencies are reported as FAIL, not repaired by weakening gates. PR #35 and issue #16 are not closed or merged by this gate.
