# C_024 — C-series bounded integration evidence audit and D-stage handoff

**Work started:** 2026-10-31 KST. **Tracker:** [Issue #313](https://github.com/july-rust/BoardSI/issues/313). **Immutable starting baseline:** protected `main@4b4de41a920e23014deacefc0e7c0785dd68f34b`. **C_024 limited C-series evidence-audit source/PR gate ACCEPTED on 2026-10-31 KST**, independently verified for [PR #314](https://github.com/july-rust/BoardSI/pull/314) and protected `main@d985c94ae6b04f0be11b1db0da6871359dd5c199`; **evidence-only reconciliation PR has its own separate protected checks tracked on [Issue #313](https://github.com/july-rust/BoardSI/issues/313)**. Not a physical SI/M1 acceptance.

## 1. Scope and evidence classification

This is an *evidence and regression-coverage audit*, not a new V16 feature, new electrical contact algorithm, new test implementation, physical Return Path P009 model, or solver qualification. Read [development checklist](../development-checklist.md), historical C-series validation ledgers, `.github/workflows/qt-runtime.yml`, and `apps/boardsi-qt-prototype/CMakeLists.txt` from the baseline. **All 23 existing C_001–C_023 limited-scope entries are [x] in the existing checklist**, including C_021–C_023 despite their nonchronological ordering there. These marks are accepted historical evidence for *their own narrowly stated scope*, **not newly rerun C_024 test results**, and not proof of total M1/C physical readiness.

| Task | Previously recorded bounded outcome | Source and first-check KST |
| --- | --- | --- |
| C_001 | Multi-COPCUT / island local candidates | Checklist bounded [x], first checked 2026-10-28 KST; [ledger](c001-multi-cutout-island-connectivity.md) |
| C_002 | Reference split / layer transition source risk | Checklist bounded [x], first checked 2026-10-28 KST; [ledger](c002-reference-split-transition-topology.md) |
| C_003 | Immutable scenario audit / stale result receipt | Checklist bounded [x], first checked 2026-10-28 KST; [ledger](c003-immutable-scenario-stale-result.md) |
| C_004 | Material provenance, loss input gates | Checklist bounded [x], first checked 2026-10-28 KST; [ledger](c004-material-loss-conductivity-roughness.md) |
| C_005 | Qt reference-risk readback/staleness | Checklist bounded [x], first checked 2026-10-28 KST; [ledger](c005-qt-reference-risk-integration.md) |
| C_006 | Exact reference gap intervals | Checklist bounded [x], first checked 2026-10-28 KST; [ledger](c006-reference-gap-interval.md) |
| C_007 | Multi-segment reference coverage | Checklist bounded [x], first checked 2026-10-29 KST; [ledger](c007-multi-segment-reference-coverage.md) |
| C_008 | Two-layer plated via transition | Checklist bounded [x], first checked 2026-10-29 KST; [ledger](c008-multilayer-reference-via-transition.md) |
| C_009 | Reference stitching via candidates | Checklist bounded [x], first checked 2026-10-29 KST; [ledger](c009-reference-stitching-via-candidate.md) |
| C_010 | Corridor-to-stitch candidate parity | Checklist bounded [x], first checked 2026-10-29 KST; [ledger](c010-reference-corridor-stitch-continuity.md) |
| C_011 | Reference stitch via pad footprint | Checklist bounded [x], first checked 2026-10-29 KST; [ledger](c011-reference-stitch-pad-footprint.md) |
| C_012 | Signal/stitch via pad clearance | Checklist bounded [x], first checked 2026-10-29 KST; [ledger](c012-signal-reference-via-pad-clearance.md) |
| C_013 | Signal via to rectangular COPCUT | Checklist bounded [x], first checked 2026-10-29 KST; [ledger](c013-signal-via-reference-cutout-clearance.md) |
| C_014 | Multiple cutout clearance | Checklist bounded [x], first checked 2026-10-30 KST; [ledger](c014-multi-cutout-signal-via-clearance.md) |
| C_015 | Circular and mixed cutout geometry | Checklist bounded [x], first checked 2026-10-30 KST; [ledger](c015-circular-mixed-cutout-source-geometry.md) |
| C_016 | Reference stitch via pad / mixed cutout | Checklist bounded [x], first checked 2026-10-30 KST; [ledger](c016-reference-stitch-via-pad-clearance.md) |
| C_017 | V1–V4 limited regression (inherited Qt) | Checklist bounded [x], first checked 2026-10-30 KST; [ledger](c017-reference-gap-layer-transition-integration.md) |
| C_018 | V5–V8 limited regression (inherited Qt) | Checklist bounded [x], first checked 2026-10-30 KST; [ledger](c018-reference-stitch-corridor-integration.md) |
| C_019 | V9–V12 limited regression (inherited Qt) | Checklist bounded [x], first checked 2026-10-30 KST; [ledger](c019-mixed-cutout-via-pad-integration.md) |
| C_020 | B_007/B_003–B_008 historical review audit | Checklist bounded [x], first checked 2026-10-30 KST; [ledger](c020-legacy-connectivity-review-audit.md) |
| C_021 | V13 two-area reference overlap candidates | Checklist bounded [x], first checked 2026-10-30 KST; [ledger](c021-reference-area-connectivity.md) |
| C_022 | V14 3–6 area candidate components | Checklist bounded [x], first checked 2026-10-31 KST; [ledger](c022-reference-multi-area-connectivity.md) |
| C_023 | V15 3–6 area corridor interval union/gap | Checklist bounded [x], first checked 2026-10-31 KST; [ledger](c023-reference-corridor-coverage.md) |

**Important qualifications:**
- C_017–C_019 specifically reused pre-existing native Qt regression and their new Rust integration cases; no separately implemented C_017/C_018/C_019 Qt IPC/E2E test may be inferred. See each ledger.
- C_020 is an *audit of evidence*; its original B_007 PR #231 latest final HEAD `6a75eb5c5616d86407a04a946bc994958fb55ea6` **independent completed Codex receipt remains NOT_VERIFIED**. Repaired later PRs do not retroactively certify it. See [C_020 ledger](c020-legacy-connectivity-review-audit.md).
- V13, V14, V15 are **separate, opt-in, source-owned rectangle geometry risk audits**, not proof of manufactured overlapping Copper/electrical continuity, even when labels/geometry agree.
- Several historical ledgers intentionally distinguish a code-only gate from subsequent docs/issue-closeout gates. A checklist [x] must never overwrite that distinction or a historical first-check date.

## 2. Current repository CI and regression mapping (inspected, not rerun by this audit)

**Existing test implementation/registration inspected at baseline:**

| Test family | Source of actual executable checks | Evidence boundary |
| --- | --- | --- |
| Rust source geometry/IPC/Canonical + Debug/Release | Historical C-series ledgers, `Numerical Core` workflow | Prior CI proven per frozen historic SHA, not newly rerun until C_024 PR jobs report success |
| C_005/C_007–C_014, C_021–C_023 Qt importer readback and stale selftests | Qt `--cNNN-...-selftest` calls in [Qt native workflow](../../.github/workflows/qt-runtime.yml), Linux `add_test` in [CMake](../../apps/boardsi-qt-prototype/CMakeLists.txt) | Actual native Win x64/ARM64 and macOS ARM64 step plus Linux offscreen CTest evidence must be checked for **new final PR SHA** |
| C_015/C_016 Qt mixed cutout/pad smoke | Explicit Native Qt workflow selftest steps | Native steps exist; dedicated Linux C_015/C_016 CTest **is not asserted** by existing source |
| C_017–C_019 integration | Rust added integration cases and previously existing Qt selftests | **No newly independent full-path Qt combined-operation selftest**; proof limited to reused smoke |
| C_020 review retrospective | GitHub metadata and checklist/ledger | No execution claim, and no B_007 final review completion claim |
| C_021/C_022/C_023 | Qt native `--c021-area-selftest`, `--c022-area-selftest`, `--c023-corridor-selftest` plus Linux CMake registrations; actual Rust Importer → Qt | Tests available; complete PR-HEAD and exact run results needed for THIS audit |
| Old source vs late selection, file/project switch and stale warning | Respective Qt selftest paths and prior ledgers | Only exercised fixture cases; not arbitrary real PCB data |

Do not confuse presence of a test, historic green CI, successful *new PR-head* runs, and exact *post-main* runs. These are distinct evidence grades. Running a docs-only PR with the full existing CI is a new regression against the current code, but **does not create a new C_024-specific Rust/Qt scenario**. If native steps are skipped, mark NOT_RUN.

## 3. Integration risk matrix and fail-closed contracts

| Domain | Safe established scope | Missing before true physical readiness |
| --- | --- | --- |
| Original PADS imports | Explicit PADS V9.4 MILS public/synthetic subset, source SHA and Canonical provenance | Version/shape-complete production PADS import, real copper fill, all padstack/span geometries |
| Copper/Net/Pad/Via | Selected local centre/edge/source geometrical witnesses | Source-complete electrical graph, real manufactured contact, DRC, split/cutout/fill semantics |
| Reference corridors | V13–V15 positive-area/interval candidates and source Gap/Island warnings | Electrical Reference conductor continuity and physics-aware signal/reference current return |
| Via transition | Limited plated-via/reference candidate Source layer checks | Physical transition impedance/return stitching and actual P009 qualification |
| Scenario/material/cache | B/C limited assumed/source/audit input contracts, provenance and immutable receipts | Solver-qualified physical revision model, cache key coherence for real physics, validated model applicability |
| User workflow | Qt CPU/RHI source risk display, explicit stale warnings | Actual productive DC–30 GHz RLGC/impedance/S-parameter board workflow |

**Hard safety boundary remains:**
```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

The full **U09/U10/U39 and T02/T04/T24/T52** requirements are **unchecked**, and [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) must remain OPEN. Physical/noise/RLGC/S acceptance cannot be manufactured by editing checklist, QA labels, or CI passing unrelated geometry tests.

## 4. Risk-prioritized handoff: bounded D-series, no monolithic promise

- **D_001 (recommended first separate implementation):** one precisely supported Source-owned same-layer electrically plausible trace-endpoint/conductive pad / reference area local witness, with strict positive Copper interior/overlap, provenance, explicit unsupported fallbacks and a separate native Qt readback regression. Specify exactly what counts as a conductor witness; avoid labeling it `P009`. Confirm before developing how this differs from B_004/B_006 so it is not redundant.
- **D_002 (later, separate):** cutout/split-aware **electrical** component hypothesis across more than one area, with manufactured-pour, thermal and via-padstack gaps marked unsupported rather than silently promoted.
- **D_003 (later, separate):** qualify the smallest physical signal/reference transition/return-path model with independent oracle and explicit limits; actual solver/caching enabled only after contract and numerical verification.
- Further release stabilization includes U/T traceability, performance/OS native evidence, packaging and unsupported UX. Never close #12 at D_001 merely because a limited contact works.

These are *handoff recommendations*, not approved acceptance criteria or completed code.

## 5. Independently verified C_024 audit PR #314 acceptance — 2026-10-31 KST

The following applies **only** to the evidence-bounded C-series source/Qt audit in [PR #314](https://github.com/july-rust/BoardSI/pull/314). It is NOT retrospective physical qualification of any C_001–C_023 original code, B_007 original last-head Codex, or formal M1 U/T.

- [x] Checklist and all 23 C-series historical validation ledgers inspected at baseline protected `main@4b4de41a920e23014deacefc0e7c0785dd68f34b` (2026-10-31 KST).
- [x] Existing Qt CMake and Native workflow source registration audited; C_017–C_019 inherited Qt testing and C_020 evidence-only nature recorded (not newly executed tests).
- [x] Historical original B_007 final-HEAD Codex `NOT_VERIFIED` preserved, not upgraded by later review.
- [x] **Exact final PR HEAD `f523e89d345ff84922b3a0a082baf5d1e0afc274`**: independently observed **22/22 PR Check Runs SUCCESS** and **8/8 matching `event=pull_request` workflow runs SUCCESS**. Existing Rust Debug/Release, Native Windows x64/ARM64 and macOS ARM64 Qt, Linux Qt Offscreen and other configured regression jobs completed SUCCESS. [Exact PR SHA Check Runs](https://api.github.com/repos/july-rust/BoardSI/commits/f523e89d345ff84922b3a0a082baf5d1e0afc274/check-runs?per_page=100), [exact PR SHA workflows](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=pull_request&head_sha=f523e89d345ff84922b3a0a082baf5d1e0afc274&per_page=100).
- [x] Independent Codex latest **same exact final HEAD** review `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; zero unresolved review threads, [PR #314](https://github.com/july-rust/BoardSI/pull/314).
- [x] Protected **Signed Verified, one-parent Squash Merge**: [`main@d985c94ae6b04f0be11b1db0da6871359dd5c199`](https://github.com/july-rust/BoardSI/commit/d985c94ae6b04f0be11b1db0da6871359dd5c199). Independent GitHub commit REST `verification.verified=true`, `reason=valid`, sole parent `4b4de41a920e23014deacefc0e7c0785dd68f34b`. No protection bypass.
- [x] **Independent exact-merge-SHA post-main CI:** [21/21 Check Runs SUCCESS](https://api.github.com/repos/july-rust/BoardSI/commits/d985c94ae6b04f0be11b1db0da6871359dd5c199/check-runs?per_page=100) and [8/8 `event=push` Workflow Runs SUCCESS](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=push&head_sha=d985c94ae6b04f0be11b1db0da6871359dd5c199&per_page=100), all bound to the exact complete merge SHA.
- [x] C_024 **bounded audit** first checklist accepted date **2026-10-31 KST**; evidence date **2026-10-31 KST**. This fact is updated in the evidence-reconciliation PR and must not be confused with this PR's later independent merge gate.

**Evidence-only follow-up rule:** the PR that edits this paragraph and the checklist must itself pass final-HEAD CI, independent exact-HEAD Codex, protected Signed Verified Squash and post-main CI before Issue #313 may close. Record that later PR's SHA-bound gate in [Issue #313](https://github.com/july-rust/BoardSI/issues/313); no recursive self-certification by this document. Do not silently upgrade unsupported scope.

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

**No new C_024 Rust/Qt source tests were implemented.** PR CI provides a fresh actual existing-code regression only. Native results are checked for the PR and post-main exact SHAs, not fabricated for a dedicated C_024 scenario.
