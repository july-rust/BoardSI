# D_037 — PADS Mixed-Sign Shallow 4:3 Via-A Negative / Via-B Positive SOURCE Contact Qualification

**Status: BOUNDED CODE AND EXACT CODE-MAIN PUSH ACCEPTED.** Final documentation-only PR and Issue #454 administrative closeout are independent acceptance gates. **First checked / evidence date: 2026-11-05 KST.** This document does not promote SOURCE geometry candidates into electrical continuity proof.

## Frozen one-source PADS geometry

- Original input is a single synthetic, redistributable **PADS PowerPCB V9.4 MILS** Source. One Source tick = **254 nm**. Main Net `NET_A`, L4 main trace `(15000,25000) -> (35000,25000)`.
- Two through-Via Source instances: Via-A `(15000,25000)` and Via-B `(35000,25000)`, imported L1/L4 Source Pad candidates only. No plated Barrel electrical continuity is demonstrated.
- **Negative Via-A Branch:** `(15000,25000) -> (3000,16000)`, main displacement `(+20000,0)`, exterior ray `(-12000,-9000)`, exact `dot=-240000000`, `cross=-180000000`, strict projection below 0.
- **Positive Via-B Branch:** `(35000,25000) -> (47000,34000)`, exterior ray `(+12000,+9000)`, exact `dot=+240000000`, `cross=+180000000`, strict projection above `main length² = 400000000`.
- Each branch independently satisfies exact signed `i128` equation `4*abs(cross)==3*abs(dot)` with nonzero dot; Via-A canonical cross **<0**, Via-B canonical cross **>0**. Source record direction reversal must preserve classification. **No floating-point epsilon, 1-tick forgiveness or geometric ambiguity.**
- Strict inventory: **3 SIGNAL records / 2 Via instances / 5 Trace segments**, Source Net/Layer/Pad provenance, 0 skipped Source constructs, 0 arcs; Layer-1 arms require exact general segment intersection and Via-on-arm exclusion.
- Public fixture: [d037_mixed_sign_dual_via_endpoint_v94.asc](../../crates/boardsi-import/tests/fixtures/d037_mixed_sign_dual_via_endpoint_v94.asc). Opt-in CLI: `--d037-non45-mixed-sign-dual-via-endpoint-contact-v1`. Unique wire tag `BOARDSI_D037_DUAL_MIXED_SIGN_VIA_ENDPOINT_CONTACT_V1`.

## Independent candidate graph and witness

- Exactly **9 SOURCE nodes, 8 same-layer CONTACT CANDIDATE edges, 3 local connected components, 0 proven electrical cross-layer edges**.
- Four Via-Pad nodes (Via-A/B each L1/L4) and five trace nodes. Four pad/main-or-arm contacts, two branch/main Source contacts, two Via-Pad/branch Source contacts. Components are Source-local graph connectivity, **not** plated-barrel or real-world electrical conduction.
- Independent Rust parse / Canonical geometry / bounded Source inventory; SHA-256 binds original Source bytes; serialized Via/Trace coordinates, graph nodes, contacts and edges. Independent Qt reimports Source, checks its own inventory and SHA against panel state, recomputes exact signed contact predicates and full expected wire, rejects all mismatch.
- Wire is explicitly `SOURCE_GEOMETRY_ONLY`, `PLATING_IMPORTER_ASSUMED`, `BARREL_ELECTRICAL_NOT_VERIFIED`, `CROSS_LAYER_ELECTRICAL_NOT_PROVEN`, `SOLVER_READY_FALSE`, ends with `END|FAIL_CLOSED_NO_BARREL_PROOF`.

## Fail-closed adversarial qualification

- Both branches reversed and original-source lexical variant accepted only with matching recomputed SHA; valid 180-degree main rotation and disjoint L1 arms accepted.
- One Source tick gap/offset/slope errors, wrong sign (+/+) or (-/-), bad 4:3 ratio, wrong Net/Layer/Via/Mode, unexpected extra/missing/duplicate Source and unsupported Section, unmodeled L1 crossing, forged SHA/Trace/contact/node/edge/component count, stale reimport and fake electrical promotion rejected.
- Existing D_036 all-negative mode does **not** accept D_037 input. D_035 and D_034 single-source modes also remain isolated.
- Rust and Qt independently verified hostile geometry. Earlier two independent review P1 issues were corrected: overlapping string substitutions in the 180-degree rotation test; wrong Via-B positive endpoint Y in native Qt hostile wire checks. Old review threads resolved, **new final SHA fully retested**.

## Code PR evidence — ACCEPTED

- [Issue #454](https://github.com/july-rust/BoardSI/issues/454), [code PR #455](https://github.com/july-rust/BoardSI/pull/455), exact final HEAD [`632fb034155f75defc7acc5c10ab4a2df61d2fee`](https://github.com/july-rust/BoardSI/commit/632fb034155f75defc7acc5c10ab4a2df61d2fee/checks).
- Exact final HEAD: **34/34 completed SUCCESS Check Runs**, **11/11 PR-triggered Actions Workflows SUCCESS**; no prior failing SHA represented as passing.
- [Numerical CI #41969889694](https://github.com/july-rust/BoardSI/actions/runs/41969889694): Rustfmt/Clippy PASS; D037 Rust **Debug 4/4** and **Release 4/4** PASS; native Rust platforms SUCCESS.
- [Qt CI #41969889612](https://github.com/july-rust/BoardSI/actions/runs/41969889612): Linux Qt 6.8.3 Offscreen **59/59 CTest PASS**, D037 native Qt log `D037_NATIVE_QT_DUAL_MIXED_SIGN_VIA_SOURCE true true true true true true true true true NO_ELECTRICAL_PROOF`; real native Qt Windows x64, Windows ARM64, macOS ARM64 SUCCESS.
- Independent Codex **latest exact final HEAD** review status `completed`, `has_any_findings=false`, `has_unresolved_findings=false`, 0 unresolved threads; see [PR latest review summary](https://github.com/july-rust/BoardSI/pull/455#issuecomment-6649895779). Prior P1 findings repaired and re-reviewed, early Rustfmt FAIL repaired.
- Protected GitHub one-parent **Verified Squash** code-main [`09a407258fc330b6a17895e983b233612a1e07bf`](https://github.com/july-rust/BoardSI/commit/09a407258fc330b6a17895e983b233612a1e07bf), sole parent `61da3167db0b3239d1cbb1d29d963b6418977f28`, REST commit verification `verified=true`, `reason=valid`; actual `main` ref independently confirmed.

## Exact code-main push evidence — ACCEPTED

- **8/8 actual `event=push` Actions Workflows SUCCESS and 28/28 Check Runs SUCCESS** at exact `main@09a407258fc330b6a17895e983b233612a1e07bf`. No skipped/failed/in-progress native or numerical checks represented as success.
- [Qt Push #41971330552](https://github.com/july-rust/BoardSI/actions/runs/41971330552) and [Numerical Push #41971330514](https://github.com/july-rust/BoardSI/actions/runs/41971330514) SUCCESS, including existing Linux, Windows x64/ARM64, macOS ARM64 platform matrix. Synthetic dynamic `Push on main` overview is **not** counted among the eight actual Actions workflows.

## Documentation-only PR and administrative closeout — PENDING SEPARATE GATE

- **Exactly two Markdown files**: this ledger plus [canonical checklist](../development-checklist.md); no code, CMake, workflows, ruleset, required context or platform runner additions.
- Existing docs-only classifier should skip whole native jobs **before allocating platform runners**. Report `SKIPPED` separately from `SUCCESS`, with all required contexts satisfied.
- Verify exact docs PR final HEAD CI, independent same-HEAD Codex clean and no unresolved review threads, GitHub Verified protected one-parent Squash and exact docs-main push CI.
- Record final docs PR number/SHA, actual docs-only SUCCESS/SKIPPED counts, Verified merge/push, and Issue #454 closure evidence as the **last Issue #454 comment**; no recursively updated documentation PR.
- Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN; legacy [PR #35](https://github.com/july-rust/BoardSI/pull/35) CLOSED/UNMERGED.

## Explicit exclusions — NOT_VERIFIED

Manufactured copper/pad/trace and plated Via-barrel continuity, global Open/Short, P009/PH-08 return-current topology, physical Scenario/Cache, RLGC/S-parameter solver fidelity, formal U01–U48 / T01–T54 full product acceptance and distribution/release are **NOT_VERIFIED**. These Source candidates are geometry only.
