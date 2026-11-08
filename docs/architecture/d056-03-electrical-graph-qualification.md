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

## Explicit gaps — not the acceptance of #529

The D_056_02 input is still **a partial conductive inventory**: polygon fills / thermal spokes / slots, Copper-to-Copper and Via-to-Via area contacts, native manufacturing proof and full-board netlist coverage cannot be certified by this module. Unsupported/source-unknown conductive records must remain fail-closed. D_056_04/05 owns Canonical padstack transport and Qt integration; D_056_06/07 owns vendor-confirmed special manufacturing grammar.

**Therefore the graph algorithm and synthetic source-scope tests are a constrained implementation step, NOT full-board Open/Short/SI acceptance.** Keep #529 and parent #520 OPEN until the outstanding full input/positive-and-negative source coverage criteria are independently fulfilled. Do not set the D_056_03 checklist item to [x] before exact-head PR checks, protected signed Squash and independent post-main Push validation, and do not claim that synthetic tests meet unsupported native-grammar criteria.

## Commands and evidence to record

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo test --locked -p boardsi-import --test d056_03_graph_qualification
```

Record the actual PR HEAD, required CI, review/thread status and protected post-main commit SHA in the single `docs/development-checklist.md`, with KST evidence date and unverified scope explicitly retained. Do not rewrite historical D_056_02 results.
