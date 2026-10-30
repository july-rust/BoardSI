# C_022 — Bounded multi-area Source Reference Copper components (V14)

**Status: IN PROGRESS; acceptance NOT VERIFIED.** [Issue #304](https://github.com/july-rust/BoardSI/issues/304); formal parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.

## Scope and source-only contract

Separate opt-in Rust `reference_area_v14::encode` and `--reference-area-v14 RA,RB,RC NET_REF 2` importer, independent of legacy C_021 V13/V1–V12 and canonical net connectivity. Require PADS V9.4 MILS complete inventory of 3–6 explicit Source-owned hole-free straight COPCLS rectangles, same declared Net/Layer, exact source SHA-256, Canonical original ring and source identity. Declared Net equality is NOT electrical contact.

Check all rectangle pairs using exact i64/i128 integer source ticks. Only strictly positive 2D area intersection joins source-geometry candidate groups; zero-area edge/corner touching is boundary-only risk. Transitive overlaps form deterministic component IDs (minimum selected position). Disjoint components are disconnected-island **risk**, not established manufactured copper islands or a complete electrical graph.

Qt independently re-reads exact imported Source Area outer rings and confirms each V14 rectangle, SHA, identity, inferred component IDs, boundary count and classification before displaying warning-only markers; stale changed source, selection and late process responses invalidate results.

## Negative and integration gates (to be completed independently)

- [ ] Synthetic Rust Debug and Release V14 integration tests (3–6 areas, chain, islands, boundary, mismatch, malformed selection and changed source)
- [ ] Qt Linux Offscreen V14 end-to-end via actual Rust importer
- [ ] Windows x64, Windows ARM64 and macOS ARM64 native Qt V14 E2E
- [ ] Exact FINAL PR HEAD rustfmt, Clippy -D warnings, required CI SUCCESS
- [ ] Independent same-final-HEAD Codex completed, zero findings, zero unresolved threads
- [ ] Protected Signed Verified Squash merge with independent post-main Check Runs and Push CI
- [ ] Dated development checklist acceptance update, documentary closure gate and Issue #304 closure

Do not record checked dates until evidence exists. All tests are source-only; actual return current/P009, manufactured fill/DRC, electrical AreaContinuity, physical SI/RLGC/S, Solver/Cache, full formal U09/U10/U39/T02/T04/T24/T52 acceptance remain **NOT VERIFIED**.

`return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`.

Existing C_021 first checked date (2026-10-30 KST) is unchanged.
