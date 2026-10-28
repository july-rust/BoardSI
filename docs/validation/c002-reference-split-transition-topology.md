# C_002 — Source-bound Reference Split and Layer Transition Topology Risk

**Started and scoped-accepted:** 2026-10-28 KST. **Status:** LIMITED RUST SOURCE-TOPOLOGY AUDIT PASS, protected signed code merge and independent post-main CI verified. Qt Viewer overlay and physical/full M1 acceptance remain NOT_IMPLEMENTED / NOT_VERIFIED.

Parent: [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) (OPEN). Sources: [B_009 handoff](b009-b-series-final-acceptance.md), [C_001](c001-multi-cutout-island-connectivity.md), [development checklist](../development-checklist.md).

## Bounded API

`boardsi_import::reference_topology::audit` is opt-in: 1 straight signal Trace and one source-order adjacent reference Layer, or 2 source Traces on different Layers meeting at exactly one source-declared plated Through Via with explicit conductive pad layers. The caller supplies all Trace IDs, Reference Net name and ordered Reference Layer IDs. This does not automatically choose reference paths.

- Reparse PADS V9.4 MILS bytes. Compare Source SHA, all canonical Traces/Arcs/Vias/Drills/Padstacks/Pads, layer order, Nets, reference Area rings/holes and imported Area inventory. Unexpected mutations are Unsupported.
- Supported reference areas are straight source-owned COPCLS polygons with at most 8 straight source COPCUT holes. Exact i128 predicates check both Trace endpoints and every outer/hole segment crossing or tangency; same-Net separate Areas do not become one conductor.
- Conservative bounding-box overlap/tangency between source Reference areas on the same Layer is **Unsupported**, even if actual polygons are separate. Zero/ambiguous reference inventory never reports a positive.
- A supported straight Trace not wholly covered by one source reference area is labeled `REFERENCE_SPLIT_RISK`, which includes gap, outer-boundary and hole crossing. This is an engineering risk indicator, not proof of electrical discontinuity.
- The limited cross-layer Source Via witness is labeled `LAYER_TRANSITION_RISK` regardless of whether the same Reference Layer is selected on both sides; neither return-via proximity nor RF return current is calculated.

## Output reason and safety contract

States: `SOURCE_GEOMETRY_ONLY`, `REFERENCE_SPLIT_RISK`, `LAYER_TRANSITION_RISK`, `UNSUPPORTED`. Deterministic reason codes, source geometry witness and Source ID references accompany supported results. The location for a Split is the Trace midpoint **risk indicator**, not an exact split intersection solution. No actionable false precision is implied.

```text
return_path_p009     = UNSUPPORTED
solver_ready         = false
physical_si          = NOT_RUN
cache_reuse_eligible = false
```

**Out of scope:** actual AC return-path, P009 quantitative proof, stitching improvement, arbitrary branch/arc or whole-board connectivity, nonpolygon/circular or curved electrical copper, manufactured pour, antipad/thermal, physical solver/cache, loss/roughness, formal U09/U10/U39/T02/T04/T24/T52 product acceptance. No Qt V1 IPC/schema changes; consumer-facing Viewer risk overlays remain pending separate integration.

## Regression and protected acceptance gates — independently verified 2026-10-28 KST

- [x] [Code PR #240](https://github.com/july-rust/BoardSI/pull/240) exact final source HEAD [`c6591c14842b55fb48ba070c9f719998dd37d195`](https://github.com/july-rust/BoardSI/commit/c6591c14842b55fb48ba070c9f719998dd37d195/checks) **23/23 Check Runs SUCCESS**, including all 17 required protected contexts.
- [x] Rust `cargo fmt --all -- --check`, Clippy `-- -D warnings`, full Workspace Debug and Release tests SUCCESS. C_002 public synthetic `tests/c002_reference_topology.rs` **10/10 Debug and 10/10 Release PASS** in Numerical core workflow, existing B_003–B_008 and C_001 suites PASS.
- [x] Native Windows x64, Windows ARM64, macOS ARM64 Rust/Qt, Linux Qt and other PR-head checks PASS.
- [x] Independent Codex review of the **same last source HEAD `c6591c1`** `completed`, `has_any_findings=false`, `has_unresolved_findings=false`, unresolved review threads 0. Earlier P2 issue involving unsupported 80-edge containment was corrected and retested with a new negative fixture.
- [x] Protected Squash merged [`main@304015008112b082f96829f763ad4e77e109094c`](https://github.com/july-rust/BoardSI/commit/304015008112b082f96829f763ad4e77e109094c), GitHub Git Commit REST signature `verified=true`, `reason=valid`, original main HEAD matched. No branch protection bypass.
- [x] **Independent code-merge post-main CI** for exact `3040150`: [21/21 Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/304015008112b082f96829f763ad4e77e109094c/checks) and **9/9 Actions Workflow Runs SUCCESS**, including [Push on main #40640594957](https://github.com/july-rust/BoardSI/actions/runs/40640594957) and [Qt runtime #40640594732](https://github.com/july-rust/BoardSI/actions/runs/40640594732).
- [x] Limited Rust scope checklist first-check **2026-10-28 KST** and evidence date **2026-10-28 KST**, earlier first-check dates preserved. This documentation closeout PR has its **own** separate CI/Codex/Verified Merge and post-main gates; do not claim those as complete until separately checked.
- [x] Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN and original B_007 exact-head Codex closure stays NOT_VERIFIED. Formal U09/U10/U39/T02/T04/T24/T52 remain unchecked.
- [ ] Qt Viewer risk marker/status integration, real AC Return Path/P009, manufactured Copper, Solver/Cache/RLGC, full M1 acceptance and physical accuracy — all outside this accepted core-only scope.

**Scoped acceptance: PASS for source-bound Rust Topology Risk only. Product electrical or GUI capability not accepted.**
