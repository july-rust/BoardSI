# PH-05 — preregistered symmetric stripline acceptance contract

Status: **FROZEN BEFORE G2 IMPLEMENTATION AND ACCEPTANCE EXECUTION**.
Baseline: `main@1f736ad91af718b35e73652bffc929706f84b07f` (PR #37 merged).
Scope: ideal homogeneous, isotropic, lossless, nonmagnetic (mu_r=1), single-PEC-signal quasi-TEM 2D stripline. **Not** full-wave, lossy, 30 GHz or PCB validation.

## Geometry, coordinates and boundaries

Propagation along y, transverse coordinates x/z in metres; z=0 and z=b are PEC ground at 0 V.
Signal: rectangular PEC (1 V), x in [-w/2,+w/2], z in [(b-t)/2,(b+t)/2].
Parameters: w=0.00030 m, t=0.00005 m, b=0.00080 m, relative epsilon=3.4; gaps each 0.000375 m. Outer domain x in [-B,+B], B=0.0012 m initially. Lateral sides have zero-normal-flux natural Neumann. The lateral boundary is a **finite computational approximation**, not infinite plane. Domain is uniform dielectric without solder mask or air.
Signal boundary must align with a geometry-conforming discretization; **never** silently snap. Bottom/top Dirichlet precedes natural lateral conditions at corners. Signal must not touch ground/boundary. Mesh level n means b/n for grid step, with aligned n=32,64,128.

## Exact mathematical checks (class M)

For homogeneous epsilon and fixed geometry/boundaries, C/C0=epsilon_r, Lext=mu0*eps0/C0, Z0=sqrt(Lext/C), delay=sqrt(Lext*C)=sqrt(mu0*eps0*epsilon_r). The identity holds without giving C absolute correctness. Lext reconstructed from C0 is NOT independent magnetic field validation. Relative error for C/C0 and delay against mathematical identities: <=0.1%. Finite physical constants EPS0=8.8541878188e-12 F/m and MU0=1.25663706127e-6 H/m.

## Independent class F numerical reference

A separately authored Python cell-centered 2D finite-volume flux discretization with sparse linear solve, identical PEC/dielectric/outer Neumann conditions, calculates conductor flux C, and separately vacuum C0. It must not import FEM stiffness/matrix/output as its oracle. Fix source/fixture SHA-256 in evidence. Compare C, C0 and Z0 with Rust P1 FEM for matching n/B/geometry: absolute relative discrepancy <=2.0% at n=128, B=0.0012 m. Report C/C0 and delay as algebraic checks rather than sole field checks. Independently enforce FVM n=64->128 relative change of **absolute C and C0** <=1.0%, and residual <=1e-8. A discrepancy or incomplete run is FAIL/Not Run, never PASS.

## FEM field and numerical constraints (class S)

- True relative linear residual <=1e-8; FEM charge/energy relative discrepancy <=1e-6 for material and vacuum.
- n=64->128: absolute C and C0 relative change <=1.5%. Report n=32,64,128 and distinct mesh statistics.
- B=0.0012, 0.0016 and 0.0024 m at fixed n=64 (and identical materials/BCs): absolute C and C0 relative variation against B=0.0024 <=2.0%. The C/C0 invariant must also hold; do NOT use invariant alone to claim mesh/boundary convergence.
- Mirror symmetry: max |phi(x,z)-phi(x,b-z)| <=1e-6 V, max |phi(x,z)-phi(-x,z)| <=1e-6 V. Signal and ground imposed potential error <=1e-12 V; upper/lower total ground charge magnitude asymmetry <=0.1%. Ground+signal total free charge normalized imbalance <=1e-6. Tangential/normal field parity and fringing reported qualitatively without implying singular corner peak convergence.
- All copper interfaces must be conforming, including finite thickness, with zero electric field *inside* the PEC.
- Geometry monotonicity: larger width or copper thickness should increase C and reduce Z0; greater plane gap should reduce C; epsilon=1 and 3.4 should scale C by 3.4, holding fixed structure. Sweep w=0.20/0.30/0.40 mm; t=0.05 mm plus aligned additional case; b=0.80 mm plus aligned additional case.
- Negative controls: missing/shorted signal/ground, offset/asymmetric signal, invalid side BC, nonconforming coordinates, zero/negative/nonfinite dimensions or material, finite conductivity, loss, layered dielectric, non-unit mu, excessive node counts. Each must return a diagnostic error and must not panic.
- Collect C,C0,Lext,Z0,delay; potential and element E; node, triangle, DOF, CSR nnz, iterations, true residual, wall time, and `/usr/bin/time -v` max RSS on Linux. CPU/RSS are machine-specific and NOT performance guarantees.

## Gates and statuses

For PH-05 limited approval **all** exact math, independent class F, numeric, symmetry, negative, existing PH-03 and G1/M0 Rust regression, strict Clippy, and final-head required CI must pass. Existing contract tolerances may NOT be relaxed after implementation. Any gate absent/failed means no squash merge; leave PR Draft. PH-04, PH-06..08, G3(lossy), G4(coupled), G5(canonical integration), 30 GHz model accuracy remain Not Run/Unsupported. Issue #16 stays open. PR #35 stays untouched.
