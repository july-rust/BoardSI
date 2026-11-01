# D_006 — Source plated Via Pad / three-area COPCUT geometry candidate qualification

**Scope:** bounded, source-only geometric candidate; **not** electrical / manufactured copper continuity, P009, physics solver, or M1 acceptance. **Check date / evidence date: 2026-11-01 KST.** See [D_006 tracking #348](https://github.com/july-rust/BoardSI/issues/348), parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) (**OPEN**).

## Provenance and code acceptance

- Baseline: `main@4b4ce296a439cd9d23518ca7f1f8e32cd0ed700b` before D_006.
- Code PR: [#349](https://github.com/july-rust/BoardSI/pull/349), **reviewed exact HEAD `252405dfccf5602f158649aa79bee00cd614b78a`**. GitHub Actions: **27/27 pull_request Check Runs SUCCESS**, all 17 required Ruleset #25046595 contexts successful. [Exact source HEAD checks](https://github.com/july-rust/BoardSI/commit/252405dfccf5602f158649aa79bee00cd614b78a/checks).
- Independent Codex on exact HEAD: `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; initial P2 issue (Qt selected-layer conductive Pad missing) fixed through independent PADS `*VIA*` source padstack readback, added two-row/no-interior-Pad negative test, and review thread resolved ([PR discussion](https://github.com/july-rust/BoardSI/pull/349)).
- Code merged by protected **one-parent Squash** into [`main@19d3a877c6c4d85fe99c829120958c8fc49ef8c7`](https://github.com/july-rust/BoardSI/commit/19d3a877c6c4d85fe99c829120958c8fc49ef8c7). GitHub commit reports `verification.verified=true`, `reason=valid`; sole parent `4b4ce296a439cd9d23518ca7f1f8e32cd0ed700b`.
- **Post-main push CI ACCEPTED (2026-11-01 KST):** independently queried exact merged `main@19d3a877c6c4d85fe99c829120958c8fc49ef8c7`: **21/21 Check Runs SUCCESS**, **8/8 event=push Workflow Runs SUCCESS**, each matching the exact head SHA; all Ruleset #25046595 required contexts passed. This evidence is separate from the PR's 27/27 pre-merge success. [Post-main checks](https://github.com/july-rust/BoardSI/commit/19d3a877c6c4d85fe99c829120958c8fc49ef8c7/checks).

## Implementation and bounded observations

- Added opt-in `boardsi-d006-via-three-area` Rust diagnostic binary; it does **not** alter normal Board Import, canonical electrical contact graph, solver input, or caches.
- Exactly one explicitly declared Through Via, selected source net and layer, circular conductive pad / plated drill, Source-to-Canonical padstack/size/layer identity, SHA-256 provenance. Exactly three explicit same-net/same-layer rectangular `COPCLS` areas and at most one source-owned rectangular `COPCUT`. D_003 three-area Source component group is reused **only as geometry evidence**.
- Complete Via pad footprint must lie strictly within a single remaining Copper Area; boundary touching, ambiguous owners, cutout overlap/tangency, wrong net/layer, malformed padstack and unsupported source objects fail closed. Integer source-grid / widened integer-square distance checks avoid floating point contact promotion.
- Rust 1.75 strict fmt/Clippy and Debug/Release regression tests, synthetic PADS positive / boundary / multiple-area / COPCUT / net-layer-invalid / source SHA change; **all passed on reviewed PR HEAD**, including `numerical` CI.
- Native Qt independent Import/Source SHA, Via pad/Drill/Net/Layer, explicit `*VIA*` selected-layer conductive Pad, three-area receipt, cutout, forged result, re-import stale marker invalidation: actual D_006 step **SUCCESS** on Windows x64, Windows ARM64, macOS ARM64; Linux x86_64 Qt 6.8.3 offscreen workflow **SUCCESS** on the reviewed PR HEAD. Existing D_001–D_005 tests preserved.
- All receipts remain `PHYSICAL_NOT_VERIFIED`, `P009_UNSUPPORTED`, `SI_NOT_RUN`; positive results are only **SOURCE_VIA_PAD_AREA_CANDIDATE**, never electrical contact.

## Unaccepted / no promotion

```text
electrical_connection = NOT_VERIFIED
return_path_p009      = UNSUPPORTED
physical_si           = NOT_RUN
solver_ready          = false
cache_reuse_eligible  = false
```

No manufactured copper fill/thermal relief, via barrel electrical continuity, multi-Via/Blind/Buried/non-circular production padstack, full PADS source cases, M1 P009, PH-08, physics/solver/cache or formal U09/U10/U39 / T02/T04/T24/T52 acceptance. Parent issue #12 **OPEN**; legacy Draft PR #35 unchanged. All code acceptance above is limited to this independent Source geometry candidate. **Tracking #348 closure is gated on separate documentation PR, signed merge, final main push CI.**
