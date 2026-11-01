# D_004 — three Source COPCLS/COPCUT candidate groups with one Trace endpoint pair
**First record:** 2026-11-01 KST. **Tracker:** [#342](https://github.com/july-rust/BoardSI/issues/342), parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. Baseline `main@4e2e3ec03e20190ba4a3622e42c809357d676dec`.

## Source-only implementation proposal
This opt-in D_004 path reuses the strict D_003 three-area full-source SHA, source/canonical ring identity, one-owned-COPCUT residual candidate edges and component IDs. Exactly one same-layer declared-Net straight Source Trace contributes its two **point** endpoints; strictly retained unique Area interior permits a source attachment candidate. Internal COPCUT, tangent/outer boundaries, ambiguous multiple retained Area owners, different Net/Layer, unsupported arcs/vias, extra Trace inventory and stale source fail closed. A positive geometrical candidate is **not** a manufactured/electrical contact.

Rust `boardsi-import-artwork <file.asc> --d004-endpoint-attachment-v1 RA RB RC NET_REF 2 0` is a separate opt-in wire; ordinary import/Copper electrical edges unchanged. Qt's `--d004-endpoint-attachment-selftest` independently checks D_003's complete Source geometry and selected imported Trace as integer 254 nm ticks, recomputes endpoint owners/groups, rejects forged endpoint states, shows warning-only markers and clears them on changed Source.

## Independently verified code PR evidence — 2026-11-01 KST

Code [PR #343](https://github.com/july-rust/BoardSI/pull/343), exact final HEAD `749bd4cb5db935961a115a3d56282b453707096f`: **27/27 Check Runs SUCCESS**, **11/11 same-head pull_request workflows SUCCESS**, all 17 protected required contexts independently compared to Ruleset 25046595. Numerical Rustfmt/strict Clippy, Debug/Release, and Windows x64/ARM64 macOS ARM64 Native Rust succeeded. Dedicated D_004 Native Qt step SUCCESS on Windows x64, Windows ARM64, macOS ARM64; Linux Qt 6.8.3 offscreen CTest `qt-preview-d004-endpoint-attachment` PASS, with actual Rust importer and `D004_NATIVE_QT_ENDPOINT_ATTACHMENT true true true true true true true` log. The test verifies forged same-SHA receipt with skipped PADS route geometry is rejected, plus stale ReImport risk clearing.

Latest-final-HEAD independent Codex summary `status=completed`, findings=false, unresolved=false. Historical P2 missing Qt imported-skipped gate fixed by exposing `importedSkippedCount()`, negative forged-receipt regression, documented review reply and resolved thread. Protected [one-parent GitHub Signed Verified Squash](https://github.com/july-rust/BoardSI/commit/3bde575048d9abb4c6a8cd9a5ccc8df50f857e46): REST `commit.verification.verified=true`, `reason=valid`, sole parent `4e2e3ec03e20190ba4a3622e42c809357d676dec`. **Independent post-code-main Push CI verified 2026-11-01 KST** for exact merged `main@3bde575048d9abb4c6a8cd9a5ccc8df50f857e46`: [21/21 Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/3bde575048d9abb4c6a8cd9a5ccc8df50f857e46/checks) and [8/8 `event=push` Workflow Runs SUCCESS](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=push&head_sha=3bde575048d9abb4c6a8cd9a5ccc8df50f857e46&per_page=100), all head_sha exact match. This establishes acceptance of **D_004 restricted code-only source geometry**; documentation PR acceptance and Issue #342 closure are separately gated.

## Acceptance (no preclaims)
- [x] Rust Debug/Release synthetic positive, COPCUT, tangent, disjoint, owner ambiguity, wrong source/net/layer/selection and SHA cases PASS — exact code HEAD CI #343.
- [x] Native Qt Windows x64/ARM64 and macOS ARM64 actual Rust importer smoke PASS — exact code HEAD #343.
- [x] Linux Qt offscreen CTest actual Rust importer PASS — exact code HEAD #343.
- [x] Exact final PR HEAD 17/17 required contexts, 27/27 checks and 11/11 workflows SUCCESS.
- [x] Independent same-final-head Codex code review completed zero findings/unresolved threads.
- [x] Protected GitHub Verified one-parent Squash and independent exact-merge-SHA **21/21 checks, 8/8 push workflows SUCCESS** (2026-11-01 KST).
- [ ] Separate documentation PR final-head CI/Codex/Verified Squash/post-main acceptance and Issue #342 safe closure (must be independently verified after this document is submitted).

## Nonclaims
```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```
No manufactured filled copper, terminal-level electrical connectivity, physical reference return current, SI solver/cache, PH-08 Gate, full U09/U10/U39 or T02/T04/T24/T52 acceptance. Parent #12 OPEN and Draft #35 unmerged. An unchecked box MUST remain unchecked until the exact evidence is independently observed.

## First accepted checkbox scope
Restricted Source geometry implementation first checked **2026-11-01 KST**; code CI, Codex, protected merge and independent code-main Push evidence date **2026-11-01 KST**. This is not U09/U10/U39, T02/T04/T24/T52, complete B_001 #12, physical Return Path/P009, or an SI solver PASS.
