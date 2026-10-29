# C_014 — Multi-COPCUT Signal Via Pad Source geometry V10 (bounded code accepted; post-main audit pending)

**Issue:** [#276](https://github.com/july-rust/BoardSI/issues/276) · **Parent M1:** [#12](https://github.com/july-rust/BoardSI/issues/12) **OPEN** · **Predecessor:** [C_013](c013-signal-via-reference-cutout-clearance.md) verified limited Source geometry.

**Started:** 2026-10-30 KST. **Code acceptance (Source Geometry only): 2026-10-30 KST.** The implementation PR is merged and its exact code HEAD and protected code merge are verified below. **Final full stage closure remains pending independent post-code-main checks, dated checklist, documentation review and post-doc-main checks.**

## V10 scope and format

- PADS V9.4 MILS explicit Source owner, one rectangular Reference Area on each selected layer, 0–8 distinct strictly enclosed disjoint rectangular COPCUTs per area. Different hole counts per layer are permitted.
- Source integer Tick comparisons for circular conductive Signal Via Pad versus hole boundaries, with tangency and overlap risk; all holes must lie strictly inside the Source outer rectangle and cannot touch/overlap another hole.
- Record ordered `LAYER` and `HOLE` rows, source SHA / Canonical Area Source ID / net / layer / Via indices, classification, selected cutout index, and legacy Receipt with line count. Use V9 only if **all** layers have <=1 hole; otherwise byte-identical V8. Legacy V1–V9 opt-ins remain unchanged.
- `GAP_OBSERVED` is Source geometry observation **not** manufactured design-rule clearance. Missing cutout and copper-side Signal Via are never a positive isolation result. Unsupported polygons/arcs/circles, ambiguous source ownership or unvalidated geometry fail closed.
- Qt independently checks original Source / imported Area ID / Cutout exact rectangles and order, recalculates integer geometry, validates embedded legacy receipt, places original-size Signal Via Pad markers on each layer and invalidates on changed input, process generation, selection and source watcher.

## Reproducible tests (candidate, not yet accepted)

- Rust `crates/boardsi-import/tests/c014_signal_via_multi_cutout_v10.rs`: 0/1/2/4/8, layer-asymmetric counts, byte-identical legacy V8/V9, pad tangency/overlap, malformed/ambiguous ownership, nonrectangular/overlapping/outside holes.
- Rust unit geometry: exact one tick, touching and overlapping rectangle boundary.
- Qt: `--c014-multi-cutout-selftest` actual Importer → Qt Preview + CPU Pad rim and stale markers on Windows x64 / ARM64, macOS arm64 native and Linux offscreen.
- fmt, Clippy, Rust Debug / Release, all previous B/C contract tests, native Qt and protected required GitHub CI. **Results must be recorded after execution**; this document does not claim they have passed.

## Verified code PR evidence — 2026-10-30 KST

- [x] [Issue #276](https://github.com/july-rust/BoardSI/issues/276), [code PR #277](https://github.com/july-rust/BoardSI/pull/277): exact final code HEAD [`fd95810bb8e60ca57a0c42b59ba83c6297021bcb`](https://github.com/july-rust/BoardSI/commit/fd95810bb8e60ca57a0c42b59ba83c6297021bcb/checks), all **11/11 PR workflows SUCCESS** (required Rust fmt/Clippy/Debug/Release, Native Qt Windows x64/ARM64, macOS ARM64, Linux offscreen, prior B/C and M2 regressions).
- [x] **Same exact HEAD Codex independent review:** `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`, [final review receipt](https://github.com/july-rust/BoardSI/pull/277#issuecomment-6509354755). Initial P1 CPU/RHI Marker and P2 Signal Via receipt identity resolved, **2/2 threads resolved**. Draft-to-ready second automatic review of same HEAD also completed with zero findings / zero unresolved.
- [x] Protected [Squash code merge `main@83623c167c27cdb51c58d9a4feab434068d7ed40`](https://github.com/july-rust/BoardSI/commit/83623c167c27cdb51c58d9a4feab434068d7ed40): GitHub REST `commit.verification.verified=true`, `reason=valid`, **one parent** independently checked by [GitHub-native temporary evidence workflow](https://github.com/july-rust/BoardSI/actions/runs/40964398309). The run intentionally failed overall while post-main Native checks remained in progress; its *successful signature step* is separate from the pending CI gate.
- [ ] **Independent post-code-main CI** — Signed commit had 21 Check Runs, but a Qt Windows ARM64 run was still `in_progress` at the latest independent verification; **not PASS until 21/21 and 9/9 successful push workflows**. [Temporary evidence workflow](https://github.com/july-rust/BoardSI/actions/runs/40964398309) will be rechecked before final acceptance.

## Acceptance gates

- [x] Exact code PR final HEAD mandatory GitHub CI and Native OS run complete (2026-10-30 KST; PR #277).
- [x] Independent same-HEAD Codex completed / zero findings / zero unresolved threads (2026-10-30 KST; PR #277).
- [x] Signed Verified Squash through existing protected Ruleset; no bypass (2026-10-30 KST; main 83623c1).
- [ ] Independent code post-main Checks and Push Workflow runs successful.
- [ ] Final dated checklist with initial check date and evidence date updated for the limited verified C_014 scope; validation document protected review and merge/post-main independently verified.
- [ ] Issue #276 closed only after all evidence reconciled. Parent #12 remains OPEN.

## Non-claims

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

No fabricated Copper or thermal/antipad, no P009 return current, no electrical topology, no DRC minimum clearance, RLGC, impedance, S-parameters, solver, or cache. Formal U09/U10/U39/T02/T04/T24/T52 remain unchecked.
