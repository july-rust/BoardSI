# D_056_03 — evidence-scoped design electrical graph qualification

**2026-11-08 KST.** Parent [#520](https://github.com/july-rust/BoardSI/issues/520), graph gate [#529](https://github.com/july-rust/BoardSI/issues/529), dependency [D_056_02](d056-02-source-layer-graph.md).

## Opt-in implementation and semantics

`crates/boardsi-import/src/electrical_graph_qualification.rs` consumes the restricted PADS V9.4 `LayeredSourceGraph` **without modifying or promoting its verified_electrical_edges**. It validates original-source SHA, physical-layer/node identity, contact-layer and declared-net consistency, rejects forged preverified barrel flags/edges, and calculates deterministic connected components, independent net conflicts (with path witnesses), expected-net disconnections, bridges and articulation points. Cut-set detection uses iterative Tarjan so deeply chained nets do not overflow the call stack.

- Supported **SOURCE-local positive-area** contacts come from D_054/D_055/D_056_02, not from equal XY, matching net names or viewer pixels.
- Plated-barrel edges are **not generated automatically**. The optional `BarrelProcessWitness` requires an exact SHA, via identity, declared start/end layers, and independent process/transition evidence references. It links successive *actually present* pad nodes; missing intermediate pads are not created.
- `SourceCoverage::Partial` is the ordinary D_056_02 default. `QualifiedSourceScope` requires a separate independent and same-digest coverage attestation. Even with a complete-source claim, missing process evidence forces an apparent disconnection to `CandidateOnly`.
- `SourceNetConflict` is a disagreement **inside the positive-contact/qualified-transition source graph**, not proof of a manufactured short. `OpenDisposition::QualifiedSourceScope` is a software-design open within an externally attested complete source scope, not an empirical open.
- A process reference is an engineering evidence claim accepted from a separately vetted caller. No test fixture or file metadata can prove that a manufactured board's plating is continuous. A library caller must independently audit the document's validity and scope.

## Independent oracle matrix

The integration tests have hand-enumerated expected nodes/components, net-conflict paths, bridge edges, articulation points and Open dispositions, rather than comparing an algorithm with itself. Test categories include 2/4/6/8/64 physical layers, arbitrary partial spans including L2–L4 and L17–L41, no phantom middle pads, missing plating witness, stale SHA, wrong span, missing endpoint, duplicate identity, forged preverified edge and changed net declaration, redundant cycle and a 12,000-node iterative chain.

## D_056_03 follow-up: source-owned CopperArea contact candidates (#540, 2026-11-08 KST)

[PR #540](https://github.com/july-rust/BoardSI/pull/540) extends the restricted PADS V9.4 SOURCE graph through `layered_area_contacts.rs`. SOURCE-owned filled `COPCLS` polygons and `COPCIR` circles become physical-layer-tagged `CopperArea` graph vertices, retaining declared Net (which may be unknown), source line, exact SHA, and cutout count. Only strictly SOURCE-valid *positive* interior witness points can add `TraceCopperArea`, `ViaPadCopperArea`, `SmdPadCopperArea` or `CopperAreaCopperArea` candidate edges. A circle's own source-owned holes must exclude any sample before it can be used as a positive area-to-area overlap witness. Source-ambiguous analytical curved area contacts are rejected. Anti-pad, via drill centre and cross-layer XY coincidence cannot generate copper contacts.

**Soundness is one-way:** a present witness supports a candidate of positive local copper overlap *within the supported geometry subset*; failure to discover a witness does **not** prove separation. The interior sample search is **not exhaustive** (including many off-centre finite-width intersections). As a consequence `qualify()` emits `COPPER_AREA_CONTACT_COVERAGE_PARTIAL` and may **never** upgrade an area-bearing graph disconnection to `OpenDisposition::QualifiedSourceScope`, even if an independent caller claims source-inventory completeness. Unknown process/barrel continuity continues to fail closed.

The independently authored public source-fixture oracles `d056_03_source_area_graph.rs` cover source polygon/straight trace, circle/layer/unknown Net, explicit COPCUT-hole negative, same-layer identical polygon cross-Net candidate, different-layer identical polygon negative, concentric circular void with isolated island false-short negative, an incomplete-area-contact Open gate, and an immediate 100,000-contact allocation limit. **8/8 PASS** at exact PR HEAD `e81e4fdfb58157a468d8cfab17b5a6a43ca1d179` under [Numerical Core #42475396805](https://github.com/july-rust/BoardSI/actions/runs/42475396805), with **9/9 PR workflow groups SUCCESS** and **0 unresolved review threads**. Protected signed one-parent code squash [main `1afbf1626d1adc93f6bb4ebedb028d65c0f86b39`](https://github.com/july-rust/BoardSI/commit/1afbf1626d1adc93f6bb4ebedb028d65c0f86b39) has GitHub `verified=true`, `reason=valid`. The code-main independent Push is a **separate** check.

## D_056_03 final contact-graph follow-ups: #542 / #543 (2026-11-09 KST)

**Direct source-layer ViaPad/SMD Pad contacts:** [PR #542](https://github.com/july-rust/BoardSI/pull/542) adds `layered_pad_contacts.rs`. Its single per-layer broad-phase and bounded pair scan emits SOURCE-local positive contact candidates for ViaPad/ViaPad, ViaPad/SmdPad and SmdPad/SmdPad. Circular annuli and solid rectangles use analytical strict overlap; mixed square/round/hollow contacts need an interior witness and retain UNKNOWN when none is found. The drilled centre, anti-pad, mere XY coincidence across physical layers and pure tangency do not cause copper edges. Public synthetic + original PADS per-layer R35/R40 direct contact tests **8/8 PASS**, exact PR HEAD `15a708dc58b2eb2ed6df76c01836e36a0891206a`, PR workflows **9/9 SUCCESS** with zero unresolved review threads. Signed, one-parent [protected main squash `0c45f5a3c361d1cf6aabc82b1ded911486aa90a7`](https://github.com/july-rust/BoardSI/commit/0c45f5a3c361d1cf6aabc82b1ded911486aa90a7) is verified; its exact-SHA independent code-main Push completed **8/8 SUCCESS** ([Numerical Core #42533804238](https://github.com/july-rust/BoardSI/actions/runs/42533804238), [Qt #42533804450](https://github.com/july-rust/BoardSI/actions/runs/42533804450), each 5/5 jobs).

**Off-sample SOURCE Trace-to-Copper positive evidence:** [PR #543](https://github.com/july-rust/BoardSI/pull/543) adds `straight_area_witness.rs`. Rather than assuming quarter-point trace samples cover arbitrary geometries, it analyzes polygon line-segment boundary crossings and circle roots, samples each resulting *strictly interior* interval and excludes owning COPCUT holes, tangency and unproved arc-chord interpolation. A finite positive-width straight trace with an independently verified centreline interior point has real positive 2D design-copper intersection. No inferred fabricated connectivity is claimed. Public independent 7/7 tests include a narrow PADS COPCLS midspan crossing not captured by old quarter-point sampling, arbitrary angle, concentric cutout and tangent negatives, invalid layer/width, and true arc not reduced to chord. PR HEAD `49a7b2fbd1e5ba17c37cf3f551adf1507e3c62a3` passed **9/9 workflow groups**, Numerical Core 5/5 jobs, Qt 5/5 jobs, review unresolved 0; signed verified one-parent [protected main squash `d939409947e297cf334f8570087d9e1f5c3e231c`](https://github.com/july-rust/BoardSI/commit/d939409947e297cf334f8570087d9e1f5c3e231c). Its exact-SHA independent code-main Push completed **8/8 SUCCESS** ([Numerical Core #42535600072](https://github.com/july-rust/BoardSI/actions/runs/42535600072), [Qt #42535600145](https://github.com/july-rust/BoardSI/actions/runs/42535600145), each 5/5 jobs).

**Test inventory:** Source graph qualification 10 + explicit filled-area 8 + source pad-pair 8 + arbitrary-angle event witness 7 = **33 independently stated integration tests** (excluding older D_001–D_055 regressions). These tests validate a software **SOURCE-scoped design graph**, not a guarantee that full PCB CAD Source, manufacturing process, plating or SI has been proven. Positive contact *presence* is source-evidenced; failure to find a candidate when the geometry inventory/algorithm is partial is **UNKNOWN**, never an authorized global Open. A source-net conflict is an electrical-design graph witness and not measured manufactured Short.

## Explicit gaps — not the acceptance of #529

The D_056_02 source input and #540 area extension are still **a partial conductive inventory and non-exhaustive contact inventory**: only certain explicit COPCLS/COPCIR filled shapes and owned cutouts have candidate graph nodes/positive samples. Full polygon/arc/flood/thermal-spoke/slot coverage, exhaustive Copper-to-Copper or Via-to-Via positive and negative contact proof, independent manufactured process evidence and full-board netlist coverage remain unverified. Unsupported/source-unknown conductive records must remain fail-closed. D_056_04/05 owns Canonical padstack transport and Qt integration; D_056_06/07 owns vendor-confirmed special manufacturing grammar.

**Therefore the graph algorithm and synthetic source-scope tests are a constrained implementation step, NOT full-board Open/Short/SI acceptance.** Keep #529 and parent #520 OPEN until the outstanding full input/positive-and-negative source coverage criteria are independently fulfilled. Do not set the D_056_03 checklist item to [x] before exact-head PR checks, protected signed Squash and independent post-main Push validation, and do not claim that synthetic tests meet unsupported native-grammar criteria.

## Commands and evidence to record

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo test --locked -p boardsi-import --test d056_03_graph_qualification
cargo test --locked -p boardsi-import --test d056_03_source_area_graph
cargo test --locked -p boardsi-import --test d056_03_source_pad_pair_graph
cargo test --locked -p boardsi-import --test d056_03_straight_area_witness
```

Record the actual PR HEAD, required CI, review/thread status and protected post-main commit SHA in the single `docs/development-checklist.md`, with KST evidence date and unverified scope explicitly retained. Do not rewrite historical D_056_02 results.
