# BoardSI G2 PH-05 — restricted FEM acceptance evidence

**Date:** 2026-10-15 KST. **Test run:** [38666133102](https://github.com/july-rust/BoardSI/actions/runs/38666133102), code SHA `264c06226f7bed520ec6d5ac8b668a8690955e42`; contract SHA `572c1915a65519fb379ba4e14066e99b99774de1`. Result: **PASS (limited synthetic PEC symmetric stripline G2/PH-05)**, **not** overall M2 or real-PCB validation.

## Implementation scope

- Rust [stripline.rs](../../prototypes/m2-lossless-fem/src/stripline.rs) extends existing P1 triangle, common CSR assembly and conjugate-gradient solver, without rewriting PH-03/Microstrip.
- Actual finite copper rectangle: w=0.30 mm, t=0.05 mm, z=0.375..0.425 mm; upper/lower PEC 0 V at z=0/0.80 mm, copper 1 V, homogeneous epsilon_r=3.4; lateral B=1.2/1.6/2.4 mm natural Neumann.
- Finite copper interior fixed 1 V (not a line or point), PEC/insulator DOFs distinct. Shape misalignment, shorts, bad material/sidewall settings and excessive meshes reject.
- Both dielectric and vacuum fields contain potential arrays, per-triangle [Ex,Ez], charge/energy metrics, ground reaction, symmetry, fringing and mesh/CG statistics. CLI reports summary JSON; full arrays are exposed in Rust structures.
- **Quasi-TEM identities**: C/C0=3.4, Lext=mu0*eps0/C0, Z0=sqrt(Lext/C), delay=sqrt(mu0*eps0*3.4); these identities do not certify the absolute capacitance.

## Executed acceptance results

| Gate or case | Evidence | Outcome |
| --- | --- | --- |
| Exact homogeneous dielectric/delay identities | Fixed 0.1% mathematical threshold | **PASS** |
| Rust vs independently assembled Python FVM C/C0/Z0 | Independent numerical geometry/field oracle, n128, B1.2 mm; 2% threshold | **PASS** |
| FEM n64→128 absolute C/C0 convergence | <=1.5% pre-registered | **PASS** |
| Independent FVM n64→128 absolute C/C0 convergence | <=1.0% pre-registered | **PASS** |
| FEM lateral B=1.2/1.6/2.4 mm absolute C/C0 sensitivity | <=2.0% pre-registered | **PASS** |
| FEM true residuals / charge-energy | <=1e-8 / <=1e-6 | **PASS** |
| Potential mirror symmetry and top/bottom ground charge | <=1e-6 V / <=0.1% | **PASS** |
| Conductor + ground charge balance | <=1e-6 normalized | **PASS** |
| Finite thickness, fringing field, copper PEC field | Dedicated Rust unit tests | **PASS** |
| Width/thickness/separation/epsilon sweep | Dedicated Rust monotonic/unit tests | **PASS** |
| Malformed/unsupported geometry and material, nonfinite and oversized grids | Panic-safe Rust negative unit tests; CLI PH-20 pathological mesh | **PASS** |
| Overall frozen PH-05 independent checker | 81 numerical gates, zero failed | **PASS** |
| Existing G1/PH-03 and S-parameter mathematical tests | Same Action: Rust + CLI reference checks | **PASS** |
| Strict Rust Clippy | Rust 1.75.0, all targets, `-D warnings` | **PASS** |
| Existing M0 Rust workspace | `cargo +1.75.0 test --workspace --locked --offline` | **PASS** |
| Windows/macOS native build or GUI/Worker IPC | Not part of this isolated change | **Not Run** |

**Reference numbers:** [independent PH-05 FVM report](m2-g2-ph05-independent-reference.md) includes n32/64/128 absolute C, C0, Z0, source/fixture/evidence SHA-256, 2D FVM construction, and reproduction commands. At n128 B1.2 mm, independent FVM C=111.764461981 pF/m, C0=32.8719005825 pF/m, Z0=55.0319654 ohm; these are **not** Rust FEM exact outputs. The matching Rust FEM values and measured machine-specific resources remain in run artifact **ph05-numerical-evidence**, so we do not invent unobserved metrics here.

**Regressions:** Original PH-03 insulating-sidewall model, microstrip FEM, PH-01/02 ideal S check, isolated Rust tests and M0 Rust tests passed in the linked run; this does not certify all of PH-01..PH-25 or Qt runtime tests.

## Scope and next gate

**PH-05: limited PASS.** PH-04 full microstrip sweep: Not Run; PH-06 asymmetric stripline: Not Run; PH-07 layered/mask: Not Run; PH-08 alternate copper/plane topology: Not Run; PH-17/18 complete independent generic suites: Not Run (PH-05 subset tested); G3 lossy Solver: Not Run; G4 differential/coupling: Not Run; G5 Canonical integration: Not Run. No frequency-dependent R/G, 3D Via/Pad, port transition, real board, actual substrate or 30 GHz physical accuracy claim.

Issue [#16](https://github.com/july-rust/BoardSI/issues/16) stays open; unrelated G5 preflight [PR #35](https://github.com/july-rust/BoardSI/pull/35) stays untouched. Next recommended separate feature: **PH-06 asymmetric stripline**, first freezing a distinct acceptance contract and validating loss of symmetry with an independent asymmetric FVM; keep PH-05 public API and tests unchanged.
