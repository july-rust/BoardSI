# D_020 — Same-layer strictly contained collinear Trace Source contact

- Tracker: [Issue #400](https://github.com/july-rust/BoardSI/issues/400)
- Status: **implementation proposed; independent CI and merge not yet accepted**
- Baseline: D_015/D_019 Source-owned synthetic PADS ASCII V9.4 MILS, exact 254-nm Source Tick, 2 Via / 4 Trace Canonical, SHA-256-bound receipt
- New opt-in: `--d020-contained-collinear-overlap-v1 NET_A 1 4 0 1`
- New fixture: `crates/boardsi-import/tests/fixtures/d020_contained_collinear_overlap_v94.asc`

## Geometry contract

Layer-4 main Trace runs from (150,250) to (350,250) Source Ticks. Its second Layer-4 Trace runs (200,250) to (250,250); both endpoints must be strictly interior to the original main Trace, with nonzero segment length and exact integer zero cross product. Reversed contained Trace direction is supported. Single endpoint touch, equal segments, boundary-touch, only partial (noncontained) overlap, one-tick skew and incompatible Net/Layer are intentionally not promoted.

The exact overlap is serialized as `OVERLAP|MAIN_TRACE|1|CONTAINED_TRACE|3|START|200|250|END|250|250` in the main Trace's direction. Qt reconstructs this independently from current Source and imported geometry, comparing the complete byte-for-byte receipt. The positive case retains **8 Source Nodes, 5 same-layer candidate edges, 3 local components, 0 proven cross-layer electrical edges** as a *test expectation until CI verifies it*, with an independent overlap edge kind `SOURCE_COLLINEAR_CONTAINED_OVERLAP_CANDIDATE`.

## Fail-closed acceptance

Rust hostile cases cover exact 1-tick misses, zero length, equal/boundary/partial overlap, cross-Net/Layer, unsupported Copper, extra Route Objects/Via definitions, invalid lexemes and bad selection. Existing D_015 and D_019 modes must reject this contained overlap while retaining their original positive fixtures. Qt native test covers actual importer and source reimport, SHA mutation/stale receipt, forged layer/contact/interval fields, electric-barrel promotion, reverse direction and decimal lexical SHA preservation.

**No physical electrical connectivity claim:** `SOURCE_GEOMETRY_ONLY`, `PLATING_IMPORTER_ASSUMED`, `BARREL_ELECTRICAL_NOT_VERIFIED`, `CROSS_LAYER_ELECTRICAL_NOT_PROVEN`, `SOLVER_READY_FALSE`; physical return path P009, PH-08, manufactured Copper, RLGC/S Solver, Scenario/Cache, global Open/Short and formal U/T still not accepted. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN; [Draft PR #35](https://github.com/july-rust/BoardSI/pull/35) untouched.

## Evidence ledger (fill with observed GitHub results only)

- [ ] Rust fmt + unit/hostile/regrression suite green on final code HEAD
- [ ] Linux Qt CTest and real Windows x64/ARM64 + macOS ARM64 Qt selftest green
- [ ] All required Ruleset checks on final PR HEAD success
- [ ] Independent latest-head Codex completed with 0 findings and 0 unresolved review threads
- [ ] Protected signed Verified single-parent Squash
- [ ] Independently verified exact merged SHA post-main push checks
- [ ] Checklist limited acceptance and tracker closure with observed evidence

Do not pre-record a future SHA, check count, review or merge as completed.
