# BoardSI M2 G1 — isolated lossless 2D P1 FEM prototype evidence

**Date:** 2026-10-15 KST. **Status:** Working, numerically cross-checked **restricted experimental prototype**; not integrated into the product and NOT physically validated for real PCB.  
**Feature PR:** [#34](https://github.com/july-rust/BoardSI/pull/34), based on merged G0 main [#33](https://github.com/july-rust/BoardSI/pull/33) commit `ec5fa4b048c0c00d8588dfd90f2c35e1107e220b`. **Current implementation and CI workflow source revision:** `b64f240788e50fb02ba16eab2b8d76104ec5c746` (includes Mesh input hardening, matched/mismatched complex ABCD checks, and dependency-trigger CI fix). Later changes to this evidence document only do not alter the solver or workflow source. The reproducible *full tree* for final validation is the PR's terminal head SHA and its [commit-specific checks](https://github.com/july-rust/BoardSI/pull/34/checks); never use the obsolete `a2a35b2` as a reproduction revision.  
**Earlier baseline CI:** [M2 isolated lossless FEM run 38656159358](https://github.com/july-rust/BoardSI/actions/runs/38656159358) **PASS** (prior 5-test implementation; historical, superseded by section 6). **Post-fix CI:** [run 38658840270](https://github.com/july-rust/BoardSI/actions/runs/38658840270) **PASS** (6 Rust prototype tests, strict Clippy, fresh independent FVM and ABCD, 69 unchanged M0 Rust workspace tests); [documentation check 38658840271](https://github.com/july-rust/BoardSI/actions/runs/38658840271) **PASS**. Workflow dependency paths were additionally hardened at source revision `b64f240` after this run, so **the final PR HEAD must separately show passing checks before merge**. No Qt native runtime was rerun.

## 1. Source and exact accepted scope

- Rust P1 **triangle finite-element** weak form, piecewise homogeneous nonmagnetic nondispersive epsilon, one explicitly PEC rectangular signal, continuous bottom PEC reference, 0 V top and side PEC walls. Direct sparse accumulation into CSR and Jacobi-preconditioned conjugate-gradient linear solve; charge/energy internal consistency.
- Compute dielectric C, identical-geometry vacuum C0, exterior idealized inductance Lext = mu0*epsilon0/C0, characteristic Z0=sqrt(Lext/C), and per-metre delay sqrt(Lext*C). **No internal conductor inductance**. No assumption of real copper R=0 or real dielectric G=0: model **refuses** a non-PEC/non-lossless request.
- Optional analytic **lossless mathematical** 2-port S at +jωt, equal positive real port reference, one uniform straight idealized section; independent Python ABCD/complex-S validation. Neither full-wave propagation nor physical frequency applicability is certified.
- Code: [isolated Rust prototype](../../prototypes/m2-lossless-fem/), [independent Python S check](../../tools/validation/m2_s2_analytic_check.py). G0 [pre-registered acceptance](m2-g0-acceptance-protocol.md), [Python FVM reference](../../tools/validation/m2_fvm_reference.py), and [unrounded oracle data](../../tools/validation/m2_g0_holdout.csv).
- **G1 independence hardening:** [m2_g1_field_check.py](../../tools/validation/m2_g1_field_check.py) computes **new actual n=32 FVM solves** for dielectric and vacuum each CI run, confirms its FVM SHA-256 and previously frozen independent n=32 values, then compares observed Rust C/C0 with the preregistered 2% limit. This removes dependence on Rust tests' hardcoded constants alone.
- No other structures/ports/materials have passed. Not a production PCB model; copper resistance, dielectric attenuation, multilayer stacks, physical open-space boundaries, via/pad, reference switching, bends, differential and 3D effects remain Unsupported.

## 2. Independent results and pre-registered thresholds

Specimen (entirely synthetic): width 0.3 mm, conductor thickness 0.05 mm, dielectric height 0.2 mm, epsilon_r=3.4, PEC signal/reference, explicitly zero-loss nonmagnetic dielectric, 0 V grounded PEC box of half-width/top height 1.2 mm. Mesh n=32 subdivisions per dielectric height. Differences against independent **cell-centered FVM** (separate field formulation and SciPy sparse direct solve, not a second Rust call).

| Property | Observed Rust FEM | Independent FVM | Difference | Pre-registered G1 limit |
| --- | ---: | ---: | ---: | ---: |
| C | 8.9251239079867028e-11 F/m | 8.842386680772379e-11 F/m | **0.935689%** (fresh FVM CI recomputation confirmed) | <=2.0% |
| C0 | 3.6708540153695572e-11 F/m | 3.654801591437518e-11 F/m | **0.439215%** (fresh FVM CI recomputation confirmed) | <=2.0% |
| Lext | 3.0310386939834007e-7 H/m | derived independently from FVM C0 | diagnostic comparison only | no separate universal field tolerance |
| Z0 | 58.27584952131 ohm | FVM 58.67628954408 ohm | diagnostic comparison only | no universal physical field tolerance |
| delay | 5.201191778208533e-9 s/m | FVM derived | diagnostic comparison only | no universal physical field tolerance |

**FEM self mesh 16 -> 32:** C **0.575816%**, C0 **0.328017%**, both within pre-registered 1.5% tolerance. Mesh order P1 and uniform corner resolution only; not yet a full adaptive-meshing accuracy certificate.

**FEM PEC-box sensitivity B=1.6 -> 2.4 mm, n=12:**
C **0.333827%**, C0 **1.168449%**, Lext **1.154954%**, Z0 **0.744687%**, delay **0.413345%**; each within pre-registered 2.0% box sensitivity tolerance. All side/top boundaries remain grounded PEC, and this does NOT establish open-air microstrip accuracy.

**Charge/energy consistency:** relative difference **5.00710584e-14**; field/vacuum true relative linear-system residuals **9.58488576e-12** and **9.49959068e-12**, respectively. No silent regularization, pseudo-inverse, or passivity clamp.

**Independent complex S mathematical comparison:** at f=1 GHz, length 0.01 m, 50 ohm equal real references, full physical 2-port S from a separate Python ABCD formula (not the Rust scattering routine) gives absolute S11 discrepancy **1.10546e-14**, S21 **3.00397e-14**. By reciprocity and a symmetric line, both directions S12/S21 and S22/S11 were checked. This is mathematics of a *given* C/L, not proof that an actual PCB is lossless or quasi-TEM accurate at 1 GHz.

## 3. Measured resource evidence

Single GitHub Actions Ubuntu 24.04 Linux x64 run, Rust 1.75.0 release optimized build, **n=32, B=1.2 mm**, two field solves (material and vacuum). Environment logs pinned by [CI run 38656159358](https://github.com/july-rust/BoardSI/actions/runs/38656159358).

| Metric | Observed result |
| --- | ---: |
| Conductor/field unknowns (each RHS) | **72,712** |
| P1 triangular elements | **147,456** |
| Assembled nonzero sparse entries | **362,296** |
| PCG iterations material/vacuum | **846 / 878** |
| Solver wall time material / vacuum | **measured per-run; see updated CI log (values vary with runner load)** |
| External /usr/bin/time executable wall | **1.08 s** |
| Peak process resident memory | **50,432 KiB** |

Results are process-/runner-specific. Do not compare resource data directly with the FVM n=48 experiment as if workload and method were identical. No other OS/CPU native FEM benchmark executed. No new workers/UI runtime operations executed.

## 4. Explicit status ledger (not artificially promoted)

| PH ID | Actually executed evidence | Honest scope |
| --- | --- | --- |
| PH-01 | Rust matched-to-Z0 S11=0 and unit-magnitude S21 mathematical test, G0 matched analytic oracle | **Partial PASS**, full lengths/frequencies sweep Not Run |
| PH-02 | Rust 50-ohm mismatched S plus independent Python complex ABCD for both directions and passivity | **Partial PASS**, unequal reference impedances Unsupported |
| PH-03 | Independent edge-free parallel plate analytical values from G0; no exact edge-free Rust FEM boundary coupon | **Not Run** for FEM exact plate anchor |
| PH-04 | One synthetic grounded-box microstrip vs separate FVM C/C0 | **Partial PASS**; width/height/thickness/Dk parameter sweep Not Run |
| PH-17 | n=16→32 uniform FEM self-mesh and n32 FVM comparison | **Partial PASS**; corner adaptive and order study Not Run |
| PH-18 | Grounded PEC enclosure width/top sweep, one boundary type | **Partial PASS**; open/Neumann alternative boundary Not Run |
| PH-19 | Positive C/C0, energy-charge equality, true residual, analytic lossless S unitarity/reciprocity | **Partial PASS**; general matrix eigenvalues/ill-conditioning not studied |
| PH-20 | Deterministic rejection of nonfinite/negative dielectric, lossless not specified, non-PEC conductor, broken reference, nonuniformity and copper/box collision | **Partial PASS**; comprehensive invalid combinations Not Run |

Five Rust tests were executed (3 unit + 2 integration), no failed tests, Clippy with -D warnings **PASS**, fresh independent FVM numerical comparison **PASS**, plus separate Python S benchmark **PASS**. The unchanged existing M0 Rust workspace tests were also executed on the revised base and **PASS** (test group counts 1+11+1+27+15+2+12 = **69**; each group 0 failures; auxiliary 0-test groups excluded). M0 boardsi-numerics/model/ipc/Worker and Qt **source was not modified**; Qt GUI native runtime was deliberately **Not Run**. Full product T-tests remain Not Run; Issue #16 remains Open. Do not set ModelStatus::Validated for any real-board result or claim a 30 GHz validated physical frequency band.

## 5. Reproduction

Run from repository checkout with Rust 1.75.0 and no external Rust dependencies:

- Cargo test: cargo +1.75.0 test --manifest-path prototypes/m2-lossless-fem/Cargo.toml --release -- --nocapture
- Clippy warnings: cargo +1.75.0 clippy --manifest-path prototypes/m2-lossless-fem/Cargo.toml --all-targets -- -D warnings
- Example: cargo +1.75.0 run --manifest-path prototypes/m2-lossless-fem/Cargo.toml --release -- 32 0.0012
- Analytic comparison: save the last command's standard output JSON and run python tools/validation/m2_s2_analytic_check.py output.json
- Separate FVM oracle: follow [G0 execution instructions](m2-g0-acceptance-protocol.md). Product Rust code intentionally does not import SciPy.

**Scoped G1 technical judgment:** the isolated synthetic PEC/lossless grounded-box FEM prototype and its preregistered numerical checks are suitable for review as a **restricted research prototype**; this is **not full physical G1/real-board Validated**. G0 PR #33 was squash merged, and PR #34 now targets the current main with exactly G1-only changed files. PR #34 is not merged without further review/approval. **G2 per independent validation plan means more single-ended structures (PH-05..08), not Canonical Model integration**; integration belongs to G5. An adapter feasibility/design-only study may precede G2/G5 without connecting this restricted prototype to live PCB data.

## 6. Final review hardening (2026-10-15 KST)

**Review changes relative to baseline `main@ec5fa4b0`:** Codex findings on [PR #34](https://github.com/july-rust/BoardSI/pull/34) were addressed by bounding finite mesh ratios **before** converting to `usize`, checking zero/underflowed grid spacing, and applying `checked_add` / `checked_mul` to node counts. Added PH-20 invalid-input tests include `box_m=1e300`, maximum finite, infinity/NaN, negative/zero, oversized and nonconforming domains, mesh subdivisions including `usize::MAX`, and invalid width, thickness, dielectric height and permittivity. The release CLI also checks that `32 1e300` exits unsuccessfully with a diagnosis and **without a panic**.

The independent Python complex-ABCD comparator now enforces the **pre-registered absolute `1e-12`** threshold for all checked PH-01 and PH-02 complex S components, phase (PH-01), reciprocity/symmetry and power conservation, with deliberate `>1e-12` perturbation negative controls. The CLI explicitly emits **PH-01 matched-to-extracted-Z0** witnesses and retains the original **PH-02 50-ohm** witnesses, so the two physical port-reference bases cannot be confused. No frozen FVM, G0 or other G1 acceptance threshold was relaxed.

[Initial corrective CI run 38658516551](https://github.com/july-rust/BoardSI/actions/runs/38658516551) on code commit `4b4c30af2938635bfe21581c68a513d04da9fe8f` produced the following **executed** results:

| Evidence on this code commit | Observed | Gate | Result |
| --- | --- | --- | --- |
| Rust G1 tests | 4 unit + 2 integration | zero failures | **PASS** |
| Strict Rust Clippy | `--all-targets -- -D warnings` | no warnings | **PASS** |
| PH-20 oversized release `box_m=1e300` | explicit failure, no panic, no JSON output | reject | **PASS** |
| Frozen complex S threshold self-check | `1e-12` accepted, larger perturbations rejected | frozen `1e-12` | **PASS** |
| PH-01 matched S11 / S21 complex error | 0 / 4.32916e-14 | each <=1e-12 | **PASS** (scoped) |
| PH-01 complex reciprocity / power error | 4.32916e-14 / 7.99361e-14 | each <=1e-12 | **PASS** (scoped) |
| PH-02 50-ohm S11 / S21 complex error | 1.10546e-14 / 3.00397e-14 | each <=1e-12 | **PASS** (scoped) |
| PH-02 complex reciprocity / power error | 3.00397e-14 / 7.32747e-15 | each <=1e-12 | **PASS** (scoped) |
| Fresh independently recomputed source-locked SciPy FVM, C / C0 | 0.935689% / 0.439215% | each <=2% | **PASS** (scoped) |
| FEM n16 to n32, C / C0 | 0.575816% / 0.328017% | each <=1.5% | **PASS** (scoped) |
| PH-18 PEC-box n12 sensitivity maximum | 1.168449% | <=2% | **PASS** (scoped) |
| True FEM residual material / vacuum | 9.58489e-12 / 9.49959e-12 | <=1e-8 | **PASS** (scoped) |
| Charge-energy relative discrepancy | 5.00711e-14 | <=1e-6 | **PASS** (scoped) |
| Existing M0 Rust workspace (69 tests) on initial corrective run | 1 Worker cancel-delivery assertion failed | all PASS needed to merge | **FAIL on initial run; investigate/recheck** |
| Qt native GUI/Worker execution | not executed | separate existing evidence | **Not Run** |
| PH-03 exact edge-free parallel plate in Rust FEM | incompatible with current finite grounded-box microstrip geometry | separate appropriate coupon required | **Not Run** |

**CPU and memory on this corrective CI run:** Rust FEM n32 = 72,712 DOF, 147,456 triangles, 362,296 matrix nonzeros, material/vacuum PCG iterations 846/878, executable wall **0.91 s**, peak RSS **50,532 KiB**, individual field solver wall 0.455206 s / 0.451324 s. Measurements are for one Linux runner and do not imply target-platform performance.

**CI follow-up:** [post-fix final-head run 38658840270](https://github.com/july-rust/BoardSI/actions/runs/38658840270) fully **PASS** on 69 unchanged M0 tests, after the initial one-time failure; the additional FVM-dependency path correction must still pass its new head checks before merging. These outcomes show a possibly timing-sensitive pre-existing Worker test, not proof of a definitive root cause.\n\n**Existing M0 intermittent failure disclosure:** the initial corrective workflow's only failure was `apps/boardsi-worker/src/lib.rs:224`, where a completed shell child did not produce the test's expected `cancel_delivery_error` after a late cancel. No M0 source or interface was changed by G1; this result must not be hidden or represented as PASS. A subsequent passing M0 regression and green CI on the final head are mandatory before PR #34 merge. See the PR's check runs and final review comment for the latest verdict.

**G1 approval boundary:** approve only the numerically compared, artificial one-conductor PEC/lossless finite-grounded-box research prototype. PH-01/02/04/17/18/19/20 are **partial scoped proofs**, not completed general PH certifications. PH-03 exact FEM plate test remains **Not Run**. Real copper and dielectric losses, dispersive dielectric, reference cutouts, via/pad/3D discontinuities, open-space microstrip, differential lines, and any 30 GHz physical-band correctness remain **Unsupported/Not Run**; no actual real PCB was analyzed.

**Following gate recommendations (do not implement in this PR):** G2 should first freeze new independent coupons for PH-05 symmetric stripline, PH-06 asymmetric stripline, PH-07 multilayer/mask microstrip, then PH-08 finite copper/reference topology; use independent field calculations, energy and boundary convergence on each before declaring a supported geometry. G3 models R/G/internal L; G4 multiconductor and mixed-mode; G5 the Canonical→Solver/Worker adapter. [PR #35](https://github.com/july-rust/BoardSI/pull/35) is *G5 adapter design preflight only*, not G2 implementation or proof.


---

## PH-03 G1 completion addendum — 2026-10-15 KST, PR #37

**This later addendum supersedes ONLY the historical PH-03 `Not Run` entries above.** Original #34 restricted grounded-PEC-box G1 evidence and earlier PH-01/02/04/17/18/19/20 results are preserved; no general physical validity is inferred. The PH-03 model has **top PEC 1 V, bottom PEC 0 V and natural homogeneous-Neumann left/right sides**, unlike the original finite grounded-box microstrip. Mathematically it is a rectangular insulated-sidewall plate, *not* a physical finite-width copper geometry with fringing.

### Preregistered contract and independent evidence

- Contract committed **first**, before FEM implementation or PH-03 runs: [PH-03 acceptance contract](m2-ph03-parallel-plate-acceptance-contract.md), commit `0769ede0e40861d6b587b910f85b2d9d18073276`. All G0/G1 tolerances unchanged.
- Independent class-M mathematical comparator [m2_ph03_plate_check.py](../../tools/validation/m2_ph03_plate_check.py) computes directly from G0 constants `eps0=8.8541878188e-12 F/m`, `mu0=1.25663706127e-6 H/m`, `w=0.0003 m`, `h=0.0002 m`, `eps_r=3.4`; it does **not** import FEM source, stiffness, or calculated reference outputs.
- Baseline *code-head* Actions [38663357466](https://github.com/july-rust/BoardSI/actions/runs/38663357466) on `2ac349c2e47f9c2271efb4f620bd8e8be9c50194`: **PASS** in Ubuntu 24.04 / Rust 1.75.0 release, strict Clippy `-D warnings`, existing synthetic microstrip FVM/S checks, PH-03 n8/16/32 independent analytical comparator and negative CLI, plus the unchanged M0 Rust workspace (69 tests). Corresponding [documentation CI 38663357473](https://github.com/july-rust/BoardSI/actions/runs/38663357473) **PASS**. Later final-head CI must also be checked before merger.
- G0 analytical reference: `C=4.515635787588e-11 F/m`; `C0=1.32812817282e-11 F/m`; `Lext=8.377580408467e-7 H/m`; `Z0=136.20713227098116 ohm`; `delay=6.1506180100757495e-9 s/m`. Voltage `phi=z/h`, `Ex=0`, `Ez=-5000 V/m`.

### Raw FEM observations from executable (no fitted factors)

| n | C (F/m) | C0 (F/m) | Lext (H/m) | Z0 (ohm) | Delay (s/m) |
|---|---:|---:|---:|---:|---:|
| 8 | 4.5156357875879966e-11 | 1.3281281728200012e-11 | 8.3775804084666607e-7 | 136.20713227098119 | 6.1506180100757454e-9 |
| 16 | 4.5156357875879966e-11 | 1.3281281728199972e-11 | 8.3775804084666861e-7 | 136.20713227098139 | 6.1506180100757545e-9 |
| 32 | 4.5156357875880470e-11 | 1.3281281728200182e-11 | 8.3775804084665537e-7 | 136.20713227097954 | 6.1506180100757404e-9 |

The **n32 relative errors** against independently generated, full-precision mathematical values were: `C 1.04e-14`, `C0 1.35e-14`, `Lext 1.35e-14`, `Z0 1.19e-14`, `delay 1.33e-15` (dimensionless, minor last-bit changes depend on decimal serialization). Every value is vastly below the frozen **0.001 (0.1%)** requirement, with C/C0 below the additional 1e-6 relative diagnostic. For mesh changes n8→16 and n16→32, respectively: maximum over the five derived parameters `3.11e-15` and `1.58e-14`, both below `1e-6`.

### Field, charge and resource evidence

| n | DOF | Nodes | Triangles | CSR nnz | FEM dielectric/vacuum solve wall time (s) | Peak process RSS (MB, decimal) |
|---|---:|---:|---:|---:|---:|---:|
| 8 | 91 | 117 | 192 | 415 | 0.000095809 / 0.000049222 | 2.67 |
| 16 | 375 | 425 | 768 | 1795 | 0.000283642 / 0.000227556 | 2.78 |
| 32 | 1519 | 1617 | 3072 | 7435 | 0.001462580 / 0.001136430 | 3.44 |

Times are `Instant` wall times per field solve, not a separately measured CPU-time benchmark. GNU `time -v` recorded process CPU user/system time as 0.00 s at its displayed resolution for each of these very small cases, so precise CPU time is **not determined**. Process RSS values are from peak `ru_maxrss` in KiB (2604/2712/3364 KiB) converted to decimal MB; not peak solver heap alone.

At n32: largest nodal potential error `5.882e-13 V`; x-row spread `1.093e-12 V`; PEC potential error `0 V`; worst normalized Ex error `3.153e-11`, worst normalized Ez error `3.445e-11` (both dielectric/vacuum included). Both fields' true linear residuals `4.618e-12` and `5.164e-12`; largest n32 energy/charge relative discrepancy `4.230e-14`. Charge-derived C and C0 independently meet the fixed 0.1% mathematical reference limit. Energy and reaction charge are two **diagnostics of the same FEM discretization**, not separate independent mathematical or physical field solvers.

Negative input cases in Rust include wrong/coincident electrode positions or voltages, grounded sidewalls, layered/lossy materials, nonunit permeability, non-PEC topology, nonfinite/negative/zero/nonconforming dimensions, NaN/Infinity, and oversized or overflowing mesh; deliberate catch-unwind checks show errors rather than panic. Additional release-CLI invalid-mesh negative check PASS.

### Final scope / gate status after the code-head test

| Gate / test | Observed conclusion |
|---|---|
| PH-03-A potential | **PASS** — mathematical rectangle only |
| PH-03-B element E | **PASS** — homogeneous ideal medium only |
| PH-03-C C/C0, Lext, Z0, delay | **PASS** — exact analytical anchor |
| PH-03-D energy/charge/residual | **PASS** — discrete diagnostics |
| PH-03-E n8→16→32 | **PASS** — expected exactly affine field |
| PH-03-F negative controls | **PASS** |
| Original G1 S/microstrip/PEC-box + Clippy + M0 Rust regressions | **PASS** on referenced Actions |
| PH-05..08 G2 stripline/additional geometries | **Not Run**, no code implemented |
| Real copper R/G, dielectric dispersion, 30 GHz band, differential, via/pad, 3D, measured PCB accuracy | **Unsupported/Not Run** |
| Qt GUI/Worker runtime newly exercised by this PH-03 run | **Not Run** (no interface changes; existing Rust M0 tests run) |

**No physical product-level `Validated` status is justified.** PR #35 remains an untouched G5 design Draft, Issue #16 remains Open. Required development order is **G0 → G1 limited prototype → PH-03 completion → G2 PH-05 symmetric stripline → PH-06 asymmetric → PH-07 multilayer/mask → PH-08 topology controls**. Each later stage needs its own preregistered independent references and reviewed tolerance.
