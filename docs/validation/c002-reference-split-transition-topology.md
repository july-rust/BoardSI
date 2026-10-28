# C_002 — Source-bound Reference Split and Layer Transition Topology Risk

**Started:** 2026-10-28 KST. **Status:** IMPLEMENTATION CANDIDATE — NOT ACCEPTED until exact-head CI, independent Codex and protected signed integration.

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

## Regression and protected acceptance gates

- [ ] New `crates/boardsi-import/tests/c002_reference_topology.rs` synthetic split/transition tests, Debug + Release.
- [ ] Existing B_003–B_008 and C_001 suites on exact last source HEAD, Rust fmt/Clippy.
- [ ] Native Windows x64, Windows ARM64, macOS ARM64, Linux Qt test matrix.
- [ ] Same-head independent Codex review `completed`, zero blocking/unresolved findings.
- [ ] Protected signed `Squash` merge, Verified `main` SHA and independent post-main Checks/Workflows.
- [ ] Development checklist KST first-check/evidence dates **only after successful evidence**, preserve historical first-check dates.
- [ ] Parent Issue #12 OPEN; B_007 original final-head Codex closure NOT_VERIFIED until separately proven.

**Do not mark C_002 complete based on merely creating this candidate file or PR.**
