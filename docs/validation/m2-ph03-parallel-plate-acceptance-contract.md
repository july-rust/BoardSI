# M2 G1 PH-03 — preregistered ideal parallel-plate FEM acceptance contract

**Freeze point:** dedicated commit on `feature/m2-g1-ph03-parallel-plate` **before PH-03 FEM implementation or execution**. Never relax these thresholds after observing results. All other G0/G1 contracts remain unchanged.

## Physics and applicability

Independent class-M mathematical anchor only. Solve `div(epsilon_r grad(phi))=0` over a homogeneous rectangle `x in [0, 0.0003] m`, `z in [0, 0.0002] m`, invariant in propagation direction y. Upper horizontal PEC at `z=h` is 1 V; bottom PEC at `z=0` is 0 V. Left/right sides have homogeneous natural Neumann `n . epsilon grad(phi)=0`. Corner nodes use the horizontal Dirichlet value. `epsilon_r=3.4`, `mu_r=1`, all material explicitly lossless and PEC. Physical finite-width fringing and all R/G losses are **not** modeled. This is an insulated-sidewall mathematical coupon, not a claim about a physical edge-free copper strip.

Use immutable G0 constants `epsilon0=8.8541878188e-12 F/m`, `mu0=1.25663706127e-6 H/m`. Independently derive `phi=z/h`, `Ex=0`, `Ez=-1/h`, `C=epsilon0*epsilon_r*w/h`, `C0=epsilon0*w/h`, `Lext=mu0*h/w`, `Z0=sqrt(Lext/C)`, `delay=sqrt(Lext*C)`. G0 published C=4.515635787588e-11 F/m, C0=1.32812817282e-11 F/m, Lext=8.377580408467e-7 H/m (display rounding only). Independent Python checker MUST compute these from locked constants and dimensions, never import Rust, fitted factors or FEM-generated references.

## Frozen gates (apply to BOTH dielectric and vacuum where meaningful)

| ID | Criterion fixed before execution |
| --- | --- |
| A / PH-03-A | Maximum absolute nodal `|phi_FEM-z/h| <= 1e-8 V`; maximum span of nodal phi across x at same z <= 1e-8 V. Every prescribed PEC node equals its voltage to <= 1e-14 V. |
| B / PH-03-B | Maximum over ALL P1 triangle elements `|Ex|/(1/h) <= 1e-6` and `|Ez + 1/h|/(1/h) <= 1e-6`. E evaluated from potential gradients, not independently manufactured. |
| C / PH-03-C | Each of C, C0, Lext, Z0, delay relative error against the independent closed-form anchor <= **0.001 (0.1%)**; additional strict affine-field diagnostic <= 1e-6 relative for C and C0; `|C/C0-epsilon_r|/epsilon_r <= 1e-6`. |
| D / PH-03-D | Relative true residual `||A*x-b||2/||b||2 <= 1e-8` for EACH solved field. Relative `|C_charge-C_energy|/C_energy <= 1e-6`. BOTH charge and energy C must individually meet 0.1% mathematical acceptance. |
| E / PH-03-E | n=8,16,32 (vertical divisions) with matching horizontal integer subdivisions 12,24,48; report every C/C0/Lext/Z0/delay result and n8→16, n16→32 changes. Each refined-vs-prior relative change <= 1e-6 for C, C0, Lext, Z0, delay; every mesh independently satisfies A-D. Strict no-assumed-monotonic-improvement rule; for this exactly representable affine solution, changes must be near roundoff. Report actual total nodes, unknown DOF, triangles, nnz, field timings and process peak RSS from CI. |
| F / PH-03-F | Unsupported/mismatched boundary/material, top/bottom electrode positions, coincident electrodes, zero/negative/nonfinite dimensions, nonfinite epsilon/mu, nonlossless/PEC topology, nonconforming or oversized/overflow mesh must return diagnosed `Err` / failing CLI exit, not panic or invented result. |

**Mismatch refusal:** a grounded left/right PEC model, multilayer dielectric, conductor-loss model, finite-thickness/edge-fringing geometry, non-unit permeability or anything else outside this specified rectangle cannot be compared to the PH-03 anchor as if equivalent.

**Implementation invariants:** do not touch preexisting microstrip Dirichlet topology and its public API. Reuse existing element matrix/CSR/CG. Zero-flux side boundaries have no source term in the FEM weak form; do NOT replace with 0 V. Carry solution nodal potentials and independently computed element gradients in PH-03-only evidence. Calculate energy and top-electrode reaction charge from the **same discretized field** (two internal diagnostics, NOT independent external PDE oracles). Reference data must come from separately authored mathematical equations in Python.

**Acceptance state:** PH-03 is **Not Run** until actual Rust and independently computed Python comparison, G1 regression, M0 workspace and strict Clippy PASS are captured from identifiable commits. If any required test fails or CI is incomplete, do not merge. PR #35 (G5 design preflight) remains untouched, and Issue #16 remains open.

**Gates:** G0 complete → G1 limited microstrip prototype complete → G1 PH-03 completion/approval → G2 PH-05 symmetric stripline → PH-06 → PH-07 → PH-08. No G2 code in this branch. Real-PCB extraction, 30 GHz accuracy, losses, coupled lines, vias and pads remain unverified.
