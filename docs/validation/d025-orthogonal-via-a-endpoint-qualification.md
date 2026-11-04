# D_025 — PADS Same-Layer Orthogonal Via-A L-Bend Source Contact Qualification

- Tracker: [Issue #415](https://github.com/july-rust/BoardSI/issues/415)
- Code PR: [#416](https://github.com/july-rust/BoardSI/pull/416), **MERGED** to protected `main`
- First recorded / code limited acceptance / evidence date: **2026-11-04 KST**
- Status: **LIMITED SOURCE-ONLY CODE QUALIFICATION ACCEPTED**; this ledger records independent merged-code Push CI. Docs PR's own checks, merge, post-doc-main checks and Issue administrative closure are separate later gates, not assumed here.
- Base protected main: `a050481f410f7942f7fbcda839f52086f26e4b8b`
- Dedicated opt-in: `--d025-orthogonal-via-a-endpoint-contact-v1 NET_A 1 4 0 1`
- Source fixture: `crates/boardsi-import/tests/fixtures/d025_orthogonal_via_a_endpoint_v94.asc`

## Frozen bounded Source-only contract

PADS POWERPCB V9.4 MILS; exact Source tick = **0.01 mil = 254 nm**, integer coordinates without floating-tolerance admission. Main Layer-4 Trace **(15000,25000) → (35000,25000)**; independent Branch Layer-4 Trace **(15000,25000) → (15000,35000)**; common Layer-4 **Via-A Pad (15000,25000)**, all declared `NET_A`. Exact 90° orthogonal L-bend contact is at **Via-A**, not Via-B; reversed Branch orientation is admitted after independent geometric validation. The two Traces and Via Pad are separate Source primitives, not a proven manufactured electrical junction.

The bounded Source contact graph has **8 nodes / 6 same-layer candidate edges / 3 local components / 0 proven electrical cross-layer edges**. Its local Via-A triangle comprises Layer-4 Via-A Pad **Node 1**, Main Trace **Node 5**, Branch Trace **Node 7**. The Source-only orthogonal candidate **5–7** uses `SOURCE_ORTHOGONAL_VIA_A_ENDPOINT_CONTACT_CANDIDATE`; the Pad-to-Branch Source candidate **1–7** remains separate. No Via barrel edge or physical continuity proof is introduced.

The Rust mode reuses full bounded Source-section inventory, explicit Net/Layer/Via selection, integer coordinates, and exact `i128` dot/cross calculations. Witness contains an SHA-256 digest of the original Source bytes and explicit `SOURCE_GEOMETRY_ONLY / BARREL_ELECTRICAL_NOT_VERIFIED / CROSS_LAYER_ELECTRICAL_NOT_PROVEN / SOLVER_READY_FALSE` qualifiers. Qt independently imports Source geometry, verifies SHA provenance and reconstructs exact expected witness bytes. Both reject one-tick gaps/offsets, diagonal/non-orthogonal, collinear or overlap modes, wrong Via/Net/Layer, degeneracy, extra/unmodelled artwork, mismatched selections, stale/forged receipts and fabricated electrical-promotion claims. D_015–D_024 have distinct mode/receipt gates; D_024 Via-B CLI remains reachable.

## Exact final code PR acceptance (verified)

- Final code PR HEAD: [`48d22751b2943bc56ff27b5241684d18c6257e10`](https://github.com/july-rust/BoardSI/commit/48d22751b2943bc56ff27b5241684d18c6257e10/checks).
- **34/34 HEAD check runs SUCCESS; 11/11 `event=pull_request` workflows SUCCESS** on exact code HEAD, including strict rustfmt/Clippy and native Qt/platform checks.
- D_025 dedicated Rust tests: **3/3 PASS** in debug and **3/3 PASS** in release ([Numerical workflow #41750646047](https://github.com/july-rust/BoardSI/actions/runs/41750646047)). Includes reverse orientation, lexical SHA binding, hostile/mode isolation.
- Linux Qt 6.8.3 offscreen: **47/47 CTest PASS** including `qt-preview-d025-orthogonal-via-a-endpoint-source` ([Qt runtime workflow #41750646261](https://github.com/july-rust/BoardSI/actions/runs/41750646261)).
- D_025 native Qt and existing regression suite on **Windows x64, Windows ARM64, macOS ARM64: SUCCESS**, exact final HEAD.
- Independent exact HEAD Codex review: **`completed`, `has_any_findings=false`, `has_unresolved_findings=false`**, unresolved review threads **0**.
- Protected one-parent Squash [`main@0206095de79a8a258b6d87beb4f0fd640366a7cc`](https://github.com/july-rust/BoardSI/commit/0206095de79a8a258b6d87beb4f0fd640366a7cc): sole parent `a050481f410f7942f7fbcda839f52086f26e4b8b`; GitHub `commit.verification.verified=true`, `reason=valid`.

## Independently verified code-main Push acceptance

- Exact merged-code SHA: [`0206095de79a8a258b6d87beb4f0fd640366a7cc`](https://github.com/july-rust/BoardSI/commit/0206095de79a8a258b6d87beb4f0fd640366a7cc/checks), the protected `main` SHA at code acceptance.
- **28/28 same-SHA GitHub Check Runs SUCCESS** and **8/8 `event=push` Workflows SUCCESS**; no failed/cancelled/pending checks in the completed evidence snapshot.
- Separate Push evidence: [Qt Runtime #41752196370](https://github.com/july-rust/BoardSI/actions/runs/41752196370), [Numerical Core #41752196540](https://github.com/july-rust/BoardSI/actions/runs/41752196540), and remaining six same-SHA Push workflows all SUCCESS. This is *post-code-merge* evidence, not documentation-merge evidence.

## Historical failures and remediation preserved

- Initial PR HEAD `dca72a2e29fd3e3a81a65f6e4ad0777a7c46cab9` failed Linux Numerical **`cargo fmt --check`**, fixed with exact Rust formatting.
- Superseded HEAD `6f61d80494ac0418e18c2f5e8cd3b799d395f04e` received a **Codex P1 finding**: new D_025 CLI condition intercepted D_024 flag, breaking both modes. Corrected by dedicated `--d025-orthogonal-via-a-endpoint-contact-v1` branch in `c910f325...`; thread resolved, exact final HEAD reviewed independently with no findings.
- Superseded HEAD `c910f325f353ac0d4cee8c153966af1f6fc1a10b` separately failed Rustfmt for long CLI condition. Corrected according to CI diff in final HEAD `48d22751...`; final same-HEAD Numerical checks and tests PASS.
- Earlier failures are **not** rewritten as functional failures or attributed to previously observed unrelated Windows ARM64 intermittency.

## Explicit physical exclusions and administrative gate

**Accepted:** strictly bounded PADS Source geometry contact *candidates* and fail-closed Rust/Qt witness validation.

**NOT VERIFIED:** manufactured Copper/Pad continuity, plated Via barrel electrical proof, whole-board Open/Short, Reference Return Path/P009, PH-08 physical gate, RLGC/S-parameters, production Solver, physical Scenario/Cache, formal U/T requirements or release readiness.

Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains **OPEN**; Draft [PR #35](https://github.com/july-rust/BoardSI/pull/35) remains **OPEN/unmerged**. Close D_025 Issue #415 administratively **only after** this Markdown-only evidence PR is protected-merged and its own exact docs-main Push checks have succeeded. Do not create an extra native-workflow or allocate native compiler runners for these documentation-only changes; retain the existing docs-only classifier and mandatory check contexts.
