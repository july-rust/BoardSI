# PH-05 independent Python finite-volume field reference

**Date:** 2026-10-15 KST. **Scope:** synthetic PEC/lossless homogeneous symmetric stripline, 2D electrostatic quasi-TEM only.
**Frozen contract:** [m2-g2-ph05-acceptance-contract.md](m2-g2-ph05-acceptance-contract.md), preregistered in commit `572c1915a65519fb379ba4e14066e99b99774de1` before FEM/FVM implementation.

## Different numerical algorithms

- **BoardSI FEM:** Rust P1 triangular elements; nodal scalar potential, shared sparse CSR/CG solver. Prescribed electrode nodal values; lateral natural Neumann. Separate dielectric and vacuum solves. Capacitance from integrated field energy and conductor reaction. Copper occupies a conforming, finite-width and finite-thickness rectangular set.
- **Independent Python FVM:** [source](../../tools/validation/m2_ph05_stripline_reference.py); scalar potential unknown at **dielectric cell centres**, conservative four-face flux stencil, directly assembled COO/CSR matrix, SciPy **sparse direct solve**. Signal charge from half-cell conductor-face flux; top/bottom ground flux from boundary faces; left/right outer flux omitted to enforce zero Neumann. Neither Rust FEM stiffness nor its field/potential is used to construct reference values.
- Same rectangular physical boundary and ground electrodes; fundamentally **different** discretization, matrix, solve and charge extraction.
- The homogeneous-dielectric C/C0=epsilon_r ratio is a mathematical identity (class M), **not** independent C accuracy evidence. Lext from C0 is a mathematical quasi-TEM transform, **not** an AC magnetic FEM.

## Frozen physical geometry

| Quantity | Value |
| --- | --- |
| Propagation | y |
| Cross-section | x,z, z in [0,0.0008] m |
| Signal PEC (1 V) | x in [-0.00015,0.00015] m, z in [0.000375,0.000425] m |
| Bottom/top reference PEC (0 V) | z=0 / z=0.0008 m |
| Finite lateral window | x in [-B,+B], B=0.0012/0.0016/0.0024 m |
| Left/right | Homogeneous Neumann (not grounded) |
| Material | epsilon_r=3.4 (or vacuum 1), mu_r=1, ideal zero loss |
| Mesh | n=32,64,128 means dz=0.0008/n; explicit face conformance |

## Execution and fixed reproducibility hashes

CI: [run 38666133102](https://github.com/july-rust/BoardSI/actions/runs/38666133102), Linux ubuntu-24.04, Python 3.13, NumPy 2.3.5, SciPy 1.17.0, Rust 1.75.0. Exact run metadata and Linux max RSS records are stored in the **ph05-numerical-evidence** Actions artifact. SHA-256 of FVM Python source: `e38f9851d5c17684d92aae94cbd84196e5465873c15d4e92f92662cb272e0a61`; immutable physical fixture JSON: `96887d01ac171ee9c35f526a64e307a825b4103ea8b83d62ac20f3ca9e0ae790`. Independent evidence JSON from this run: `cc2187524ad6eee7d0aef11c36c1527594f943137a40d07025930a8a1646cde2`. The artifact also contains each FEM input SHA-256 and individual solver observations. These hashes refer to this **specific code run**, not arbitrary future modified code.

Reproduce from repository root with frozen sources:

```bash
python -m pip install numpy==2.3.5 scipy==1.17.0
cargo +1.75.0 build --manifest-path prototypes/m2-lossless-fem/Cargo.toml --release
BIN=prototypes/m2-lossless-fem/target/release/boardsi-m2-lossless-fem
for n in 32 64 128; do "$BIN" stripline "$n" 0.0012 > "ph05-n$n-B12.json"; done
"$BIN" stripline 64 0.0016 > ph05-n64-B16.json
"$BIN" stripline 64 0.0024 > ph05-n64-B24.json
python tools/validation/m2_ph05_stripline_reference.py --self-test
python tools/validation/m2_ph05_stripline_reference.py \
  --fem ph05-n32-B12.json ph05-n64-B12.json ph05-n128-B12.json \
        ph05-n64-B16.json ph05-n64-B24.json \
  --output ph05-independent-fvm-evidence.json
```

## Independently observed numeric reference

These are **Python FVM numbers**, not Rust FEM measurements:

| n | B (mm) | C (pF/m) | C0 (pF/m) | Z0 (ohm) |
| ---: | ---: | ---: | ---: | ---: |
| 32 | 1.2 | 110.494861256 | 32.4984886047 | 55.6642901 |
| 64 | 1.2 | 111.412010955 | 32.7682385163 | 55.2060586 |
| 128 | 1.2 | 111.764461981 | 32.8719005825 | 55.0319654 |
| 64 | 1.6 | 111.436438717 | 32.7754231521 | 55.1939570 |
| 64 | 2.4 | 111.437540846 | 32.7757473076 | 55.1934112 |

At n=128, B=1.2 mm, derived **FVM-based** Lext=3.384805978e-7 H/m and delay=6.150618010e-9 s/m, using the frozen mu0*eps0/C0 identity. The independent FVM's n64-to-n128 absolute-C relative change is 0.31535%; the B=1.2 vs B=2.4 mm C difference (n64) is 0.02291%. Both values are **numerical sensitivity indicators**, not proof of real-stripline manufacturing accuracy.

The strict checker compared C/C0/Z0 from Rust FEM against independent FVM at n=128 and verified separate mesh, boundary, residual, charge and symmetry constraints. The CI log reports **81 PASS, 0 FAIL**, not just a homogeneous epsilon identity.

## Physical validity limits

Stripline PH-05 does **not** represent finite-resistance copper, roughness, dielectric loss/dispersion, via/pad, plane cutouts, asymmetric dielectric, multilayer mask or 3D discontinuities. No frequency-band or 30 GHz full-wave validity is established; C0-derived Lext is not independently magnetically solved. PH-06 and G3/G4/G5 remain Not Run.
