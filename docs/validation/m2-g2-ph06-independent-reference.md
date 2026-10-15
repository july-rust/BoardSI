# PH-06 independent FVM — asymmetric and two-region reference

**Date:** 2026-10-15 KST. **Physical class:** independently computed 2D electrostatic field reference (limited synthetic class F). **Prerecorded gate:** [PH-06 Acceptance Contract](m2-g2-ph06-acceptance-contract.md), separately committed at `15da48ef3753e361d009decd03dbd4516dfd9868` before solver or FVM code. None of this proves real-board/30 GHz/loss/3D validation.

## Geometry and algorithms

- y-direction propagation; x–z cross-section; rectangular PEC signal x=[-0.15,+0.15] mm, z=[0.275,0.325] mm; upper/bottom PEC at z=0.80/0 mm, respectively.
- Signal PEC=1 V, grounds=0 V; natural zero-Neumann left/right x=±1.2 mm (swept to 1.6 and 2.4 mm); w=0.30 mm, t=0.05 mm, b=0.80 mm.
- PH-06A homogeneous epsilon=3.4; PH-06B epsilon(bottom)=3.4 to z=0.35 mm and epsilon(top)=4.0 above. Both nonmagnetic, isotropic, no finite conductivity/dielectric loss or dispersion. Material interface is aligned with n=32/64/128 mesh. Same copper/ground geometry for vacuum C0.
- **Rust FEM:** piecewise-element epsilon, P1 triangles, nodal Dirichlet PEC, natural Neumann, shared CSR/CG. C from integrated energy, positive signal free-charge reaction; independent upper and lower electrode reactions. Solve dielectric and vacuum separately.
- **Python FVM independent formulation:** [m2_ph06_asymmetric_stripline_reference.py](../../tools/validation/m2_ph06_asymmetric_stripline_reference.py) uses cell-centered unknowns, 4-face conservative finite-volume flux, sparse direct SciPy spsolve (not Rust CG). Dielectric-to-dielectric face: harmonic series conductance `2 εa εb/(εa+εb)`; metal/ground face at half-cell distance: `2 εcell`. Left/right Neumann faces contribute zero. Signal/upper/lower charges each integrate FVM face flux with proper sign. Rust FEM potential or stiffness is never a supplied FVM reference.
- PH-06 reflected full-geometry and material sweep references are checked independently in [m2_ph06_invariant_sweeps.py](../../tools/validation/m2_ph06_invariant_sweeps.py). PH-05 symmetric recovery is a regression anchor, not an independent absolute-C oracle.
- P1 has continuous nodal potential across material interfaces, and its dielectric coefficient is assigned by triangle row (no triangle may cross interface). Flux continuity holds in FEM weak/conservative integrated sense; no misleading pointwise normal-flux certification at singular copper corners.

## Independently computed representative values

CI [run 38669970099](https://github.com/july-rust/BoardSI/actions/runs/38669970099) used Python 3.13, NumPy 2.3.5, SciPy 1.17.0, Rust 1.75.0, Linux ubuntu-24.04. n=128; B=1.2 mm.

| Parameter | PH-06A independent FVM | PH-06B independent FVM |
| --- | ---: | ---: |
| C, pF/m | 116.862304258 | 123.386465280 |
| C0, pF/m | 34.3712659584 | 34.3712659584 |
| Z0, Ω | 52.6313258 | 51.2209668 |
| Delay, ns/m | 6.15061801 | 6.31997404 |

These are **independent FVM calculations**, not copied FEM predictions. Quasi-TEM `Lext=MU0*EPS0/C0`, `Z0=sqrt(Lext/C)`, `delay=sqrt(Lext*C)` are *derived* from independent electrostatic C and C0; an independent magnetic AC solver was not implemented.

## Reproducibility

FVM source SHA-256, as printed by the actual 2026-10-15 CI: `25996b9315f5aaf547a16cb6685429821a3144696b6abd9418c0190f5f890011`.

FVM fixture canonical JSON SHA-256: `01653c029ab80170b7ba95d049b1ff6849fbe5d25c1a07a4f2535553746b482f`.

Immutable copies of **actual** source, fixture hashes, individually hashed FEM input JSONs, both independent FVM field records, residuals, charge conservation, gate values and CPU time are in the workflow artifact `ph06-numerical-evidence` for the linked run (GitHub artifact retention 14 days). The JSON `ph06-independent-fvm-evidence.json` includes source/fixture hashes, model parameters, per-mesh results, comparisons and PASS/FAIL outcomes. Do not treat a future edited FVM source as carrying the same source hash.

Reproduce:

```bash
python -m pip install numpy==2.3.5 scipy==1.17.0
cargo +1.75.0 build --manifest-path prototypes/m2-lossless-fem/Cargo.toml --release
BIN=prototypes/m2-lossless-fem/target/release/boardsi-m2-lossless-fem
for n in 32 64 128; do
  "$BIN" asymmetric "$n" 0.0012 > "ph06-A-n$n-B12.json"
  "$BIN" asymmetric "$n" 0.0012 0.00030 3.4 4.0 0.00035 > "ph06-B-n$n-B12.json"
done
for code in 16 24; do
  if test "$code" = 16; then B=0.0016; else B=0.0024; fi
  "$BIN" asymmetric 64 "$B" > "ph06-A-n64-B$code.json"
  "$BIN" asymmetric 64 "$B" 0.00030 3.4 4.0 0.00035 > "ph06-B-n64-B$code.json"
done
python tools/validation/m2_ph06_asymmetric_stripline_reference.py \
  --fem ph06-A-n32-B12.json ph06-A-n64-B12.json ph06-A-n128-B12.json \
        ph06-A-n64-B16.json ph06-A-n64-B24.json \
        ph06-B-n32-B12.json ph06-B-n64-B12.json ph06-B-n128-B12.json \
        ph06-B-n64-B16.json ph06-B-n64-B24.json \
  --output ph06-independent-fvm-evidence.json
```

## Results and limitations

Frozen independent gate: **201 PASS / 0 FAIL**, plus supplementary geometry/dielectric/reflection invariants **57 PASS / 0 FAIL** as run in CI 38669970099. Includes C/C0 absolute comparisons, Z0/delay, field/electrode/charge limits, n32/64/128 mesh refinement and B=1.2/1.6/2.4 mm side-boundary truncation.

**Not Run / Unsupported:** true magnetic FEM, finite conductivity, nonzero dielectric loss, dispersive ε, multilayer mask/air, complex PCB stacks, discontinuities, vias, 3D full-wave, calibrated 30 GHz bandwidth, physical metrology and actual board signal-integrity accuracy.
