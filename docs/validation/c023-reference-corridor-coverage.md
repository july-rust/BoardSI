# C_023 — Source-exact V15 multi-area Reference corridor coverage and gaps

**Status: IN PROGRESS / NOT ACCEPTED** — 2026-10-31 KST. [Issue #309](https://github.com/july-rust/BoardSI/issues/309) and [PR #310](https://github.com/july-rust/BoardSI/pull/310). Parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN.

## Bounded function

V15 opt-in Rust `--reference-corridor-v15` shares the C_022 V14 full-source inventory and Canonical Source Area checks. It selects an imported straight signal trace on a different physical copper layer from reference, 3–6 exact hole-free rectangular reference areas on the same named net and layer, and computes exact integer source-tick open-interior interval coverage. Positive-length uncovered gaps, zero-length boundary contacts and separated area components are only **source geometry risks**, never metal fill or electrical evidence. No canonical net/contact graph edges are added.

Qt independently verifies the V14 area receipt against imported artwork, the V15 selected trace and its exact source coordinates, and recomputes intervals and risk classification. Source changes, selection changes and late worker completions must invalidate prior warnings.

## Acceptance gate (record evidence, not projections)

- [ ] Rust synthetic positive/negative cases in Debug and Release
- [ ] Rust formatting, strict Clippy, existing import and numerical regressions
- [ ] Native Qt actual-importer E2E on Windows x64/ARM64, macOS ARM64 and Linux Offscreen
- [ ] Exact final PR SHA CI required checks all successful
- [ ] Independent same-final-HEAD Codex completed, zero findings and unresolved threads
- [ ] Protected Signed Verified one-parent Squash merge
- [ ] Independently successful exact-merge-SHA post-main Push checks/workflows
- [ ] Final evidence on issue, dated first-check/evidence date on development checklist, issue closed

No check above is to be marked done before independently observed proof.

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

No manufactured copper fill/DRC, full-board electrical topology, physical Return Current, RLGC/S, material solver, cache or formal U09/U10/U39/T02/T04/T24/T52 qualification.
