# BoardSI M2 G0: synthetic ideal microstrip holdout acceptance contract

Date: 2026-10-15 KST. Baseline: merged PR #31, main 65c397d99b2ebf8dc3e55dcdb0ef6e168be32342.

**Contract frozen BEFORE the holdout reference run.** The small previously executed FVM pilot was exploratory, NOT an acceptance run; the new holdout mesh levels 12, 24, 48 remain separate. Do not revise tolerance thresholds after observing holdout measurements.

## Geometry and port contract

| Property | Synthetic value |
| --- | --- |
| Cross section axes | x across trace width; z vertical from bottom reference; propagation +y |
| Copper signal | PEC rectangle centered x=0, w=0.0003 m, t=0.00005 m, bottom z=h |
| Dielectric | h=0.0002 m, relative permittivity=3.4, lossless, nondispersive, permeability ratio=1 |
| Ground/reference | continuous ideal bottom PEC z=0, phi=0 V |
| Signal excitation | phi=+1 V signal PEC, one conductor |
| Exterior | PEC side walls x=±B, top wall z=B, all phi=0 V |
| Tested box B | 0.0012, 0.0016, 0.0024 m |
| Vacuum C0 | same geometry/ports/boundaries, all dielectric replaced by free space |
| Model class | longitudinally uniform scalar 2D electrostatic, NOT true open microstrip |
| Math two-port | near port 1, far port 2, +y, equal real reference 50 ohms, length=0.01 m, frequency=1e9 Hz, exp(+jωt) |
| Explicit idealization | signal/reference PEC, exactly zero dielectric loss; NO estimate of actual copper sigma, Df, R/G, internal L or high-frequency validity |

The dielectric fills 0≤z<h and air fills the rest, with no solder mask. All features are synthetically specified and redistributable; geometry boundaries must conform exactly to grid faces (FVM) or element edges (FEM). Reject any conductor overlap, negative dielectric parameter, nonfinite coordinate or absent reference. Side and upper walls are grounded, not open boundary; increasing B measures sensitivity but does not prove the open-space limit.

Constants: epsilon0=8.8541878188e-12 F/m, mu0=1.25663706127e-6 H/m (recorded numerical constants).

## Independent source and method

The independent field oracle F is tools/validation/m2_fvm_reference.py, a cell-centered face-flux finite-volume discretization, independently developed in Python using NumPy and SciPy sparse direct solve. It does **not** claim to be FEniCSx/DOLFINx or Ansys; the Rust prototype will use separate P1 triangular FEM with different nodes, assembly and linear solver. These constitute independently implemented field discretizations, not the same BoardSI FEM exercised twice. SciPy is BSD-3-Clause; no vendor artwork or licensed field outputs are distributed. Pin source, interpreter, NumPy and SciPy versions in recorded evidence.

Mathematical independent limit PH-03: an *edge-free homogeneous parallel plate*, for which C=epsilon0*epsilon_r*w/h, C0=epsilon0*w/h, Lext=mu0*h/w. This is not an exact closed-form result for finite-width grounded-box microstrip. Mathematical PH-01: for strictly positive C,C0, Lext=mu0*epsilon0/C0, Z0=sqrt(Lext/C), vp=1/sqrt(Lext*C). The **frozen 50-ohm** port basis is mismatched to this coupon's Z0 (about 58.6 ohms), and therefore S11/S22 are nonzero. Compute all complex S using independent ABCD and multiple-reflection formulas and require agreement, complex reciprocity and energy conservation. The formula S11=0, S21=exp(-j*omega*length/vp) applies **only** to a separate explicitly *matched-to-Z0* auxiliary cross-check and must not be substituted for 50-ohm reference data. Mathematical identities do not independently prove physical extraction accuracy.

## Pre-registered numerical readiness checks

1. **Holdout mesh:** B=0.0012 m and n=24 vs n=48 cells per substrate height: relative changes in C, C0, Lext, Z0, delay each <= 1.0%, divided by the finer grid result.
2. **Holdout exterior:** B=0.0016 vs B=0.0024 m at n=12: relative changes in C, C0, Lext, Z0, delay each <= 2.0%, divided by the larger-domain result. This is *box-size sensitivity*, not an open-boundary error bound.
3. **Physical and algebraic checks:** relative linear residual <=1e-9; finite positive C/C0; 1 <= C/C0 <= 3.4; positive Lext/Z0/delay. Report explicit units and idealized physical scope.
4. **Mathematical anchors:** generate PH-01 on the explicitly fixed **50-ohm mismatched** port basis and PH-03 edge-free plate. Require independently derived ABCD and multiple-reflection complex S11/S21 to agree within 1e-12, and lossless power sum within 1e-12. Explicit matched-to-Z0 S11=0 is a separate optional mathematical sanity check, never the 50-ohm expected value.
5. **Reproduction:** record raw output SHA-256, source SHA-256, tool versions, elapsed time, mesh cells/DOF, linear residual and peak observed process RSS, with measurement limits. The exact source digest is pinned and verified in tools/validation/m2_g0_source.sha256 (SHA-256 c5e05fed106cc78e22150b0c90a828951a439940bb1a8dc8843bd5eb73786929). The verifier recomputes the SHA-256 before declaring G0 Ready. A single end-of-run RSS reading must be called current RSS, not peak RSS.
6. **Status discipline:** these checks authorize *only* a G0 Ready decision for the restricted idealized coupon oracle. PH-04, PH-17 and PH-18 for BoardSI Rust FEM are Not Run until implementation; no PCB loss or 30 GHz capability validated.

**Pre-registered subsequent G1 comparison:** at B=0.0012 and n=32, triangular Rust FEM and independent FVM must agree in C and C0 each within 2.0%; Rust relative linear residual <=1e-8; FEM n=16->32 changes for C/C0 each <=1.5%. These coarse **numerical prototype** thresholds are NOT universal electromagnetic accuracy tolerances and do not replace the separate mathematical 0.1% targets in specification v1.0. After the first G1 comparison, test outer-box sensitivity separately.

If a holdout criterion fails: record G0 Pending/Fail, preserve actual values and never move goalposts or label the model Validated. Any future physically open microstrip, dispersive/lossy or frequency qualification requires separate evidence and change approval.
