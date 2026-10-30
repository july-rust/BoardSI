# C_022 — Bounded multi-area Source Reference Copper components (V14)

**Status: SOURCE-ONLY CODE ACCEPTED on 2026-10-31 KST; separate documentation closeout PR and Issue #304 final closure pending.** [Issue #304](https://github.com/july-rust/BoardSI/issues/304); formal parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.

## Scope and source-only contract

Separate opt-in Rust `reference_area_v14::encode` and `--reference-area-v14 RA,RB,RC NET_REF 2` importer, independent of legacy C_021 V13/V1–V12 and canonical net connectivity. Require PADS V9.4 MILS complete inventory of 3–6 explicit Source-owned hole-free straight COPCLS rectangles, same declared Net/Layer, exact source SHA-256, Canonical original ring and source identity. Declared Net equality is NOT electrical contact.

Check all rectangle pairs using exact i64/i128 integer source ticks. Only strictly positive 2D area intersection joins source-geometry candidate groups; zero-area edge/corner touching is boundary-only risk. Transitive overlaps form deterministic component IDs (minimum selected position). Disjoint components are disconnected-island **risk**, not established manufactured copper islands or a complete electrical graph.

Qt independently re-reads exact imported Source Area outer rings and confirms each V14 rectangle, SHA, identity, inferred component IDs, boundary count and classification before displaying warning-only markers; stale changed source, selection and late process responses invalidate results.

## Independent code HEAD and post-main evidence — 2026-10-31 KST

- [x] Synthetic Rust Debug and Release V14 integration tests (3–6 areas, chain, islands, boundary, mismatch, malformed selection and changed source)
- [x] Qt Linux Offscreen V14 end-to-end via actual Rust importer
- [x] Windows x64, Windows ARM64 and macOS ARM64 native Qt V14 E2E
- [x] Exact FINAL PR HEAD rustfmt, Clippy -D warnings, required CI SUCCESS
- [x] Independent same-final-HEAD Codex completed, zero findings, zero unresolved threads
- [x] Protected Signed Verified Squash merge with independent post-main Check Runs and Push CI
- [ ] Documentation closeout PR exact-final-HEAD CI, independent Codex, protected Signed Verified Squash and independently successful post-document-main CI, followed by Issue #304 closure. **Not claimed in advance.**

**First accepted source-only checklist check date: 2026-10-31 KST; evidence date: 2026-10-31 KST.** Existing C_021 first-check date is preserved. Do not close #304 or check the documentation closure gate until its own post-main acceptance. All tests are source-only; actual return current/P009, manufactured fill/DRC, electrical AreaContinuity, physical SI/RLGC/S, Solver/Cache, full formal U09/U10/U39/T02/T04/T24/T52 acceptance remain **NOT VERIFIED**.

`return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`.

Existing C_021 first checked date (2026-10-30 KST) is unchanged.

## Evidence links and strict limitations

- [Code PR #305](https://github.com/july-rust/BoardSI/pull/305) **exact final reviewed HEAD** `e077eeaa8f31766ae94757bb90a90dcdd129cb34`: [27/27 PR Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/e077eeaa8f31766ae94757bb90a90dcdd129cb34/checks); **11/11 PR workflows SUCCESS**, including [Numerical core Rustfmt/strict Clippy/Debug+Release/Native](https://github.com/july-rust/BoardSI/actions/runs/41141859569) and [Qt runtime Native Windows x64/ARM64, macOS ARM64, Linux offscreen](https://github.com/july-rust/BoardSI/actions/runs/41141859505); dedicated C_022 Native importer→Qt selftest succeeded on all 3 requested native targets.
- Independent Codex exact final HEAD: `code_review.status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; historical 2 P1 fixture/asynchronous rejection defects fixed and both review threads resolved. Source-only tests cover six synthetic cases, transitive groups, disjoint island risk, edge/corner non-contact, invalid inventory and changed-source SHA.
- [Signed protected one-parent Squash `main@242293dda118024697b6d1cb3bde5a521e6fa2cf`](https://github.com/july-rust/BoardSI/commit/242293dda118024697b6d1cb3bde5a521e6fa2cf): independent GitHub REST `commit.verification.verified=true`, `reason=valid`, **single parent** `b6cb674f4fbd95baa78d8b49dcbd050e6dc2a0bb`.
- Independently queried code `main` [post-merge Checks **21/21 SUCCESS**](https://api.github.com/repos/july-rust/BoardSI/commits/242293dda118024697b6d1cb3bde5a521e6fa2cf/check-runs?per_page=100) and [post-main Push Workflows **8/8 SUCCESS**](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=push&head_sha=242293dda118024697b6d1cb3bde5a521e6fa2cf&per_page=100), exact SHA, including [Numerical Push](https://github.com/july-rust/BoardSI/actions/runs/41142924382) and [Qt Push](https://github.com/july-rust/BoardSI/actions/runs/41142924437).
- GitHub documented mandatory `macos-14` brownout **2026-10-30 14:00 UTC—2026-10-31 00:00 UTC** and retirement on 2026-11-02 ([official notice](https://github.blog/changelog/2026-10-01-github-actions-macos-14-runner-image-retirement/)); three required macOS ARM64 CI matrices moved to supported `macos-15`. All four platform Qt source/Native and FEM checks then succeeded. This infrastructure fix is not an electrical feature.

**Source-only limitations remain:** Same-source Net label and geometric overlapping rectangles DO NOT demonstrate physically plated contact, manufactured fill/DRC, true electrical Copper connectivity or continuous Reference Return Current. `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`. Formal U09/U10/U39 and T02/T04/T24/T52 remain unchecked; parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.
