# C_023 — Source-exact V15 multi-area Reference corridor coverage and gaps

**Status: bounded C_023 Source Geometry code ACCEPTED on 2026-10-31 KST; documentation final acceptance PR and issue closure pending** — 2026-10-31 KST. [Issue #309](https://github.com/july-rust/BoardSI/issues/309) and [PR #310](https://github.com/july-rust/BoardSI/pull/310). Parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN.

## Bounded function

V15 opt-in Rust `--reference-corridor-v15` shares the C_022 V14 full-source inventory and Canonical Source Area checks. It selects an imported straight signal trace on a different physical copper layer from reference, 3–6 exact hole-free rectangular reference areas on the same named net and layer, and computes exact integer source-tick open-interior interval coverage. Positive-length uncovered gaps, zero-length boundary contacts and separated area components are only **source geometry risks**, never metal fill or electrical evidence. No canonical net/contact graph edges are added.

Qt independently verifies the V14 area receipt against imported artwork, the V15 selected trace and its exact source coordinates, and recomputes intervals and risk classification. Source changes, selection changes and late worker completions must invalidate prior warnings.

## Independently verified code PR evidence — 2026-10-31 KST

- [Code PR #310](https://github.com/july-rust/BoardSI/pull/310) reviewed exact final HEAD `42ef22f32bdd247a473b7c29d8a6dc4c59c954db`: **27/27 final-HEAD Check Runs SUCCESS** and **11/11 matching `event=pull_request` workflow runs SUCCESS**, including Rust format, strict Clippy, numerical Debug/Release tests, FEM, CodeQL, Qt Linux offscreen and Windows x64/ARM64 plus macOS ARM64 native runs.
- Native Windows x64, Windows ARM64 and macOS ARM64 `C_023 Native multi-area Reference corridor coverage and stale smoke` steps individually completed **SUCCESS**; the Linux offscreen CTest also passed with actual Rust importer.
- Independent Codex same-final-HEAD `code_review.status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; 0 unresolved review threads.
- [GitHub protected one-parent Signed Verified Squash `main@69c37dc4bb0815e13baae59f3b416c95546a38b9`](https://github.com/july-rust/BoardSI/commit/69c37dc4bb0815e13baae59f3b416c95546a38b9) validated by independent GitHub REST `commit.verification.verified=true`, `reason=valid`, single parent `2e9da132f9a77891b70169684ae520d4b7d6e48a`.
- **Independently verified post-code-main gate on 2026-10-31 KST:** [exact `main@69c37dc4bb0815e13baae59f3b416c95546a38b9` Check Runs](https://api.github.com/repos/july-rust/BoardSI/commits/69c37dc4bb0815e13baae59f3b416c95546a38b9/check-runs?per_page=100) **21/21 SUCCESS**, [exact `event=push` workflow runs](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=push&head_sha=69c37dc4bb0815e13baae59f3b416c95546a38b9&per_page=100) **8/8 completed SUCCESS**, each matching the complete merge SHA. Source-only checklist first accepted/check date **2026-10-31 KST**, evidence date **2026-10-31 KST**. This independent post-main gate qualifies only the bounded V15 geometry risk feature, not the physical return path.

Final documentation [PR #311](https://github.com/july-rust/BoardSI/pull/311) and Issue #309 closure remain separate pending acceptance gates; do not pre-certify its exact final-HEAD checks/Codex/protected merge/post-main outcomes in this ledger.

## Acceptance gate (record evidence, not projections)

- [x] Rust synthetic positive/negative cases in Debug and Release (code PR #310 exact-head Numerical native CI)
- [x] Rust formatting, strict Clippy, existing import and numerical regressions (code PR #310 exact-head checks)
- [x] Native Qt actual-importer E2E on Windows x64/ARM64, macOS ARM64 and Linux Offscreen (code PR #310 exact-head CI)
- [x] Exact final PR SHA CI required checks all successful (27/27 checks, 11/11 workflows)
- [x] Independent same-final-HEAD Codex completed, zero findings and unresolved threads
- [x] Protected Signed Verified one-parent Squash merge
- [x] Independently successful exact-merge-SHA post-main Push checks/workflows — **21/21 exact-merge-SHA Check Runs SUCCESS** and **8/8 same-SHA `event=push` workflows SUCCESS**
- [ ] Final evidence on issue and documentation PR #311 protected acceptance/issue closure — **PENDING**; checklist first check 2026-10-31 KST already recorded only for source-only code scope

No check above is to be marked done before independently observed proof.

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

No manufactured copper fill/DRC, full-board electrical topology, physical Return Current, RLGC/S, material solver, cache or formal U09/U10/U39/T02/T04/T24/T52 qualification.
