# D_020 — Same-layer strictly contained collinear Trace Source contact

- Tracker: [Issue #400](https://github.com/july-rust/BoardSI/issues/400)
- Status: **bounded Source geometry code accepted on protected `main`; not physical electrical connectivity** (2026-11-03 KST). Separate documentation follow-up and tracker closure are tracked independently.
- Baseline: D_015/D_019 Source-owned synthetic PADS ASCII V9.4 MILS, exact 254-nm Source Tick, 2 Via / 4 Trace Canonical, SHA-256-bound receipt
- New opt-in: `--d020-contained-collinear-overlap-v1 NET_A 1 4 0 1`
- New fixture: `crates/boardsi-import/tests/fixtures/d020_contained_collinear_overlap_v94.asc`

## Geometry contract

Layer-4 main Trace runs from (150,250) to (350,250) **MILS** (internally 15000..35000 centimil Source Ticks). Its second Layer-4 Trace runs (200,250) to (250,250) MILS; both endpoints must be strictly interior to the original main Trace, with nonzero segment length and exact integer zero cross product. Reversed contained Trace direction is supported. Single endpoint touch, equal segments, boundary-touch, only partial (noncontained) overlap, one-tick skew and incompatible Net/Layer are intentionally not promoted.

The exact overlap is serialized as `OVERLAP|MAIN_TRACE|1|CONTAINED_TRACE|3|START|20000|25000|END|25000|25000` in the main Trace's direction. Qt reconstructs this independently from current Source and imported geometry, comparing the complete byte-for-byte receipt. The positive case retains **8 Source Nodes, 5 same-layer candidate edges, 3 local components, 0 proven cross-layer electrical edges** verified by CI (Linux Qt CTest 42/42 PASS, and native Windows x64/ARM64 + macOS ARM64 Qt tests), with an independent overlap edge kind `SOURCE_COLLINEAR_CONTAINED_OVERLAP_CANDIDATE`.

## Fail-closed acceptance

Rust hostile cases cover exact 1-tick misses, zero length, equal/boundary/partial overlap, cross-Net/Layer, unsupported Copper, extra Route Objects/Via definitions, invalid lexemes and bad selection. Existing D_015 and D_019 modes must reject this contained overlap while retaining their original positive fixtures. Qt native test covers actual importer and source reimport, SHA mutation/stale receipt, forged layer/contact/interval fields, electric-barrel promotion, reverse direction and decimal lexical SHA preservation.

**No physical electrical connectivity claim:** `SOURCE_GEOMETRY_ONLY`, `PLATING_IMPORTER_ASSUMED`, `BARREL_ELECTRICAL_NOT_VERIFIED`, `CROSS_LAYER_ELECTRICAL_NOT_PROVEN`, `SOLVER_READY_FALSE`; physical return path P009, PH-08, manufactured Copper, RLGC/S Solver, Scenario/Cache, global Open/Short and formal U/T still not accepted. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN; [Draft PR #35](https://github.com/july-rust/BoardSI/pull/35) untouched.

## Verified evidence — 2026-11-03 KST

- **Code PR:** [#401](https://github.com/july-rust/BoardSI/pull/401); exact final PR HEAD `2a5825ff83de399865bb4e2644c0079e00fd50de`, **34/34 checks successful** and **11/11 pull_request workflows successful** ([exact HEAD Checks](https://github.com/july-rust/BoardSI/commit/2a5825ff83de399865bb4e2644c0079e00fd50de/checks)).
- **Rust:** final-head `cargo fmt --all -- --check` + `clippy -D warnings` and numerical/model-integrity tests successfully completed in the Numerical Core workflow. Adversarial `d020_contained_collinear_overlap` fixture/tests and separated D015/D019 behavior remained covered.
- **Qt:** actual Linux Qt `ctest --test-dir build-qt --output-on-failure` **42/42 PASS** including `qt-preview-d020-contained-overlap-source`. Same-head dedicated D020 native Qt step successfully completed on Windows x64, Windows ARM64 and macOS ARM64 (all under [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/41626609600)).
- **Codex:** independent same-final-HEAD Code Review `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`, with **0 unresolved review threads**. The original P1 centimil Source Tick error was corrected and its stale review thread resolved.
- **Protected main:** [Verified Squash `8f29cd1a6fca6b2aef51934fd97a25afdce6a1e7`](https://github.com/july-rust/BoardSI/commit/8f29cd1a6fca6b2aef51934fd97a25afdce6a1e7), one parent `63be4c7158000fa906e573962fe35bf71efedfae`, GitHub `verification.verified=true,reason=valid`.
- **Independent post-main:** exact merged commit [Checks](https://github.com/july-rust/BoardSI/commit/8f29cd1a6fca6b2aef51934fd97a25afdce6a1e7/checks) **28/28 success**, **8/8 event=push workflows success** (including [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/41629270403) and [Numerical core](https://github.com/july-rust/BoardSI/actions/runs/41629270374)). No pending/failure/cancelled work.
- **Acceptance boundary:** solely strict Source Geometry + canonical importer contact candidate. Manufacturing Copper, barrel electrical proof, whole-board electrical Open/Short, Return Path P009, PH-08, Solver/Cache, RLGC, S parameters, and formal U/T acceptance remain NOT VERIFIED. Parent Issue #12 must stay open, Draft PR #35 unchanged.

## Audit status (only observed actions recorded)

- [x] Rust fmt + unit/hostile/regression suite green on final code HEAD
- [x] Linux Qt CTest and real Windows x64/ARM64 + macOS ARM64 Qt selftest green
- [x] All required Ruleset checks on final PR HEAD success
- [x] Independent latest-head Codex completed with 0 findings and 0 unresolved review threads
- [x] Protected signed Verified single-parent Squash
- [x] Independently verified exact merged SHA post-main push checks
- [ ] Follow-up documentation PR protected merge/post-main verification and Issue #400 closure (not yet evidenced here)

Do not pre-record the future documentation PR/head/merge/post-main CI or Issue #400 closure as completed.
