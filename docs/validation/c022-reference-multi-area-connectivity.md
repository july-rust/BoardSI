# C_022 — Bounded multi-area Source Reference Copper components (V14)

**Status: COMPLETE for bounded C_022 Source Geometry risk audit (2026-10-31 KST); final Issue #304 closure is recorded separately after reconciliation.** [Issue #304](https://github.com/july-rust/BoardSI/issues/304); formal parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.

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
- [x] Independent documentation [PR #307](https://github.com/july-rust/BoardSI/pull/307) exact-final-HEAD CI 22/22 checks SUCCESS and 8/8 workflows SUCCESS; same-final-HEAD independent Codex `completed`, findings=false, unresolved=false, zero review threads; protected GitHub-signed Verified one-parent Squash and independently successful post-document-main CI (details below). **Issue #304 closure remains a separate final action after this reconciliation PR is independently accepted.**

**First accepted source-only checklist check date: 2026-10-31 KST; evidence date: 2026-10-31 KST.** Existing C_021 first-check date is preserved. The documentation gate has now been independently observed and checked; close #304 only after this reconciliation PR's own required gates and post-main acceptance. All tests are source-only; actual return current/P009, manufactured fill/DRC, electrical AreaContinuity, physical SI/RLGC/S, Solver/Cache, full formal U09/U10/U39/T02/T04/T24/T52 acceptance remain **NOT VERIFIED**.

`return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`.

Existing C_021 first checked date (2026-10-30 KST) is unchanged.

## Independent documentation-merge acceptance (observed 2026-10-31 KST)

- [x] Documentation [PR #307](https://github.com/july-rust/BoardSI/pull/307) reviewed final HEAD `e264b306d3aa5887f1b21af280993a3dbef6702a`: [22/22 exact-head Checks SUCCESS](https://api.github.com/repos/july-rust/BoardSI/commits/e264b306d3aa5887f1b21af280993a3dbef6702a/check-runs?per_page=100), [8/8 exact-head Pull Request workflows SUCCESS](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=pull_request&head_sha=e264b306d3aa5887f1b21af280993a3dbef6702a&per_page=100).
- [x] Same-final-HEAD independent Codex review recorded in [PR #307 review-state summary](https://github.com/july-rust/BoardSI/pull/307): `code_review.status=completed`, `commit_sha=e264b306...`, `has_any_findings=false`, `has_unresolved_findings=false`. Inline review comments: 0.
- [x] GitHub [protected one-parent Signed Verified Squash `main@c4e083b9f2d4bd056990e036c05fa3e28b818ddc`](https://github.com/july-rust/BoardSI/commit/c4e083b9f2d4bd056990e036c05fa3e28b818ddc): [direct REST](https://api.github.com/repos/july-rust/BoardSI/commits/c4e083b9f2d4bd056990e036c05fa3e28b818ddc) `commit.verification.verified=true`, `reason=valid`, exactly one parent `242293dda118024697b6d1cb3bde5a521e6fa2cf`.
- [x] Independent documentation [post-main 21/21 Checks SUCCESS](https://api.github.com/repos/july-rust/BoardSI/commits/c4e083b9f2d4bd056990e036c05fa3e28b818ddc/check-runs?per_page=100) and [8/8 `event=push` workflows SUCCESS](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=push&head_sha=c4e083b9f2d4bd056990e036c05fa3e28b818ddc&per_page=100), all 8 `head_sha` exact match, run_attempt 1.

**Reconciliation changes only documentation; it does not change source code, product claims, or previously recorded first-check date.** The current reconciliation PR also needs its own independent CI/Codex/protected merge/post-main checks; its final evidence can be recorded in Issue #304 after merge, avoiding any unverified forward claims in this document.

## Evidence links and strict limitations

- [Code PR #305](https://github.com/july-rust/BoardSI/pull/305) **exact final reviewed HEAD** `e077eeaa8f31766ae94757bb90a90dcdd129cb34`: [27/27 PR Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/e077eeaa8f31766ae94757bb90a90dcdd129cb34/checks); **11/11 PR workflows SUCCESS**, including [Numerical core Rustfmt/strict Clippy/Debug+Release/Native](https://github.com/july-rust/BoardSI/actions/runs/41141859569) and [Qt runtime Native Windows x64/ARM64, macOS ARM64, Linux offscreen](https://github.com/july-rust/BoardSI/actions/runs/41141859505); dedicated C_022 Native importer→Qt selftest succeeded on all 3 requested native targets.
- Independent Codex exact final HEAD: `code_review.status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; historical 2 P1 fixture/asynchronous rejection defects fixed and both review threads resolved. Source-only tests cover six synthetic cases, transitive groups, disjoint island risk, edge/corner non-contact, invalid inventory and changed-source SHA.
- [Signed protected one-parent Squash `main@242293dda118024697b6d1cb3bde5a521e6fa2cf`](https://github.com/july-rust/BoardSI/commit/242293dda118024697b6d1cb3bde5a521e6fa2cf): independent GitHub REST `commit.verification.verified=true`, `reason=valid`, **single parent** `b6cb674f4fbd95baa78d8b49dcbd050e6dc2a0bb`.
- Independently queried code `main` [post-merge Checks **21/21 SUCCESS**](https://api.github.com/repos/july-rust/BoardSI/commits/242293dda118024697b6d1cb3bde5a521e6fa2cf/check-runs?per_page=100) and [post-main Push Workflows **8/8 SUCCESS**](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=push&head_sha=242293dda118024697b6d1cb3bde5a521e6fa2cf&per_page=100), exact SHA, including [Numerical Push](https://github.com/july-rust/BoardSI/actions/runs/41142924382) and [Qt Push](https://github.com/july-rust/BoardSI/actions/runs/41142924437).
- GitHub documented mandatory `macos-14` brownout **2026-10-30 14:00 UTC—2026-10-31 00:00 UTC** and retirement on 2026-11-02 ([official notice](https://github.blog/changelog/2026-10-01-github-actions-macos-14-runner-image-retirement/)); three required macOS ARM64 CI matrices moved to supported `macos-15`. All four platform Qt source/Native and FEM checks then succeeded. This infrastructure fix is not an electrical feature.

**Source-only limitations remain:** Same-source Net label and geometric overlapping rectangles DO NOT demonstrate physically plated contact, manufactured fill/DRC, true electrical Copper connectivity or continuous Reference Return Current. `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`. Formal U09/U10/U39 and T02/T04/T24/T52 remain unchecked; parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.
