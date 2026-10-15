# PH-07 preregistered acceptance contract — multilayer microstrip with rectangular solder mask

**FROZEN GATE BEFORE PH-07 FEM/FVM IMPLEMENTATION.** Baseline main `c1bdf95e37e6b0643382ed5e32edac2eebdd8d6d`. Commit containing this contract must precede all PH-07 solver and independent-reference commits. Changing tolerances after numerical results is prohibited. This validates only an ideal-PEC, nonmagnetic, isotropic, nondispersive, lossless *synthetic* 2D electrostatic model; no real-board, 3D, magnetic-current, 30 GHz or lossy claim.

## Geometric fixture, units and boundary conditions

All coordinates SI metres, y longitudinal; x–z cross-section. Domain `-B<=x<=B; 0<=z<=H`. Bottom continuous PEC ground at z=0, fixed 0 V. Top z=H and sides x=±B are zero-normal-gradient natural Neumann (NOT Dirichlet). Signal copper rectangle `-0.00015<=x<=0.00015; 0.00020<=z<=0.00025` PEC 1 V, no interior independent field DOFs. A zero field inside PEC is not a separate dielectric solution. Homogeneous Neumann top/sides give no fictitious conductor charge.

Default B=0.00120 m, H=0.00080 m, substrate height h=0.00020 m. Single-substrate material z in (0,h) epsilon_r=3.4. Two layers separated exactly at z=0.00010 m: lower epsilon_r=3.4 and upper epsilon_r=4.0. Air epsilon_r=1.0. Materials isotropic, positive, nonmagnetic mu_r=1. Copper ideal PEC (conductivity not modelled); substrate and mask Df=0, no dispersion.

A: one substrate, mask off. B: one substrate, mask on. C: two substrate layers, mask off. D: two substrate layers, mask on. Same conductor and domain in all four cases. Interface of equal epsilon is geometrically possible but cannot change numerical C,C0 by >1e-8 relative.

**Exact PH-07 mask union**, with t_m=0.000025 m, epsilon_r=3.3, and copper half-width a=0.00015 m. Outside the PEC: (i) blanket `h<=z<=h+t_m` across exposed substrate; (ii) side coating `a<=|x|<=a+t_m; h<=z<=h+t`; (iii) top coating `|x|<=a+t_m; h+t<=z<=h+t+t_m`, where copper thickness t=0.00005 m. Mask rectangles form a continuous material union; overlapping mask rectangles are harmless union, but mask never occupies PEC. Everything else between substrate/metal and upper boundary is air. This deliberately simplified square-corner geometry does not approximate actual deposition curves. Material priority: PEC first; then substrate; then mask union; then air.

## Mesh and independent discretizations

For H=0.0008, `dz=H/n` and `dx=dz`; mandatory n=32,64,128. All substrate/copper/mask interfaces must land exactly on grid faces, not be rounded into alignment. Every dielectric triangle has one material ID/constant epsilon_r; no averaging across an interface. A metal-interior triangle contributes no electric-field energy or artificial dielectric region. Fixed geometry during refinement. Reject non-conforming or memory-exceeding inputs with a diagnostic and no panic. Existing P1 triangles, CSR matrix and CG are reused in Rust; independent Python cell-centred FVM assembles 4-face transmissibilities, uses distance-weighted harmonic series permittivity for dielectric/dielectric faces, half-cell conductor distances, and a SciPy sparse DIRECT solve. No FEM value supplies a numerical FVM target.

Top H sweep 0.8,1.2,1.6 mm at **fixed dz=0.0125 mm**, using n=64,96,128 respectively. Side B sweep 1.2,1.6,2.4 mm with H=0.8 mm, n=64 and fixed dz=0.0125 mm. Top and side influences evaluated independently using the widest/tallest case as comparator. No changed metal dimensions or artificial ground at top.

## Predeclared acceptance thresholds

Denominator for `relative(a,b)` is strictly positive independent comparator b; relative = abs(a/b-1), no epsilon denominator shortcuts.

| Mandatory numerical gate | Frozen threshold |
|---|---|
| FEM versus independent FVM at n128, B=1.2 mm, H=0.8 mm, cases A–D | C, C0, Z0, Delay EACH <=3.0% |
| Both solvers' true relative linear residual for all compared runs | <=1e-8 |
| FEM field-energy / signal conductor-charge C discrepancy | <=1e-6 |
| Each solver's net electrode charge imbalance, denominator abs(signal charge) | <=1e-6 |
| FEM n64→128 C,C0 change, cases A–D | <=2.0% |
| FVM n64→128 C,C0 change, cases A–D | <=1.5% |
| B=1.2/1.6 vs 2.4 mm, n64 fixed dz: C,C0 | <=2.0% |
| H=0.8/1.2 vs 1.6 mm, fixed dz: C,C0 | <=2.0% |
| Left/right potential mirror absolute difference (symmetric geometries) | <=1e-6 V |
| Exact prescribed PEC=1 V, GND=0 V nodal error | <=1e-12 V |
| Identical-epsilon artificial interface versus one-layer at identical geometry | C,C0,Z0,Delay relative <=1e-8 |
| All invalid, out-of-domain, unsupported and excessive-grid cases | explicit error, no panic |

Additional mandatory properties:
- C>0, C0>0, Lext>0, Z0>0, Delay>0 and finite. `C0` independently recomputed with all relative epsilon values set to 1 but the EXACT same metal, domain, BC and mesh.
- `epsilon_eff=C/C0`, `Lext=MU0*EPS0/C0`, `Z0=sqrt(Lext/C)`, `Delay=sqrt(Lext*C)`, identity relative <=1e-10. Derived Lext is NOT independently magnetic-solved.
- Epsilon bounds `min(material_eps)<=epsilon_eff<=max(material_eps)` with 1e-8 allowance.
- C0 invariant under mask ON/OFF and layer-epsilon changes (fixed geometry), relative <=1e-8.
- Replacing air with a mask εr>1 must yield C ON > C OFF for this nondegenerate fixture, C0 same, epsilon_eff higher, Z0 lower, Delay higher (independently checked by both FEM/FVM). Increasing mask epsilon or thickness cannot DECREASE C for same fixed PEC and domain geometry. Likewise increasing a substrate epsilon cannot decrease C. Swapping lower/upper unequal epsilon is NOT a symmetry.
- x symmetry of potentials; nonzero physically plausible fringe Ex outside signal; zero PEC interior electric field. Material interface potential continuous (P1 conforming); flux continuity tested in weak/conservative integrated form, NEVER with pointwise singular PEC-corner E.
- Signal and bottom-GND reaction/flux charges opposite, conservative to above bound; never invent top/side reference electrode charges.
- n32,64,128 all four cases record actual C,C0,Z0,Delay, epsilon_eff, signal/GND charges, field energy, triangle/node/DOF/NNZ/iterations/true residual, dx/nx/nz, CPU and peak RSS. Independently generated FVM source and canonical fixture SHA-256 recorded in evidence.
- Mask parameter sweeps: OFF; ON t=0.025 mm epsilon=3.3; t=0.0125 and 0.05 mm (at n64 or finer); epsilon=1.0,3.3,4.0 at fixed mask footprint. Off-mesh requests rejected or Unsupported, never silently snapped.
- Layer sweeps: single 3.4; bottom/top 3.4/4.0, 4.0/3.4, 3.4/3.4; all materials vacuum. Same-epsilon interface is numerically invariant. Report region energies as partition of total, not independent capacitance.
- Negative cases include zero/negative/nonfinite geometry, dielectric <=0, PEC short/intersection, missing/misplaced mask or dielectric layer, off-grid faces, unsupported boundaries/mu_r!=1, loss, finite conductivity, dispersion and overflow/memory limits. Return diagnostic without panic.

## Stop conditions, regressions and reproducibility

Actual independent Python FVM computations and Rust FEM outputs MUST be retained in JSON workflow artifacts, with source/fixture hashes and nonzero exit on any failed numerical gate. Report every acceptance item PASS/FAIL/Not Run/Unsupported. No claim of PASS from merely emitting a log. PH-03, PH-05 81 gates, PH-06 201+57 gates, G0/G1, workspace, legacy CLI/API, Clippy -D warnings and HEAD Actions must pass before approval. G5 PR #35, PH-08, Qt, IPC, board integration, copper/dielectric loss and measured 30 GHz performance excluded. PH-07 is one limited G2 prototype, not completion of M2. Issue #16 remains open.
