# D_035 — Exact Negative Shallow 4:3 Non-45° Via-A SOURCE Contact Qualification

**Status:** BOUNDED CODE ACCEPTED. Exact final code PR HEAD, independent Codex, protected Verified one-parent Squash and exact post-code-main Push independently confirmed. Separate docs-only protected PR, docs-main Push and Issue #448 administrative closure remain **PENDING** until independently verified. **First checked / evidence date: 2026-11-05 KST.**

## Frozen PADS SOURCE geometry and qualified scope

- PADS V9.4 MILS, one exact SOURCE tick = **254 nm**; Net `NET_A`; Layer L4.
- Main `(15000,25000)→(35000,25000)`; Via-A `(15000,25000)`; Branch `(15000,25000)→(3000,16000)`. Exact reverse SOURCE branch record accepted.
- Canonical Via-A→exterior vector `(-12000,-9000)`; main vector `(+20000,0)`; signed cross `-180000000`, dot `-240000000`, `4*abs(cross)==3*abs(dot)`, both nonzero; **canonical cross < 0**, exterior projection < 0. Use signed i128 in Rust, exact integer geometry; **no floating/epsilon/one-tick tolerance**.
- Existing [D_029 positive shallow 4:3 Via-A](d029-non45-oblique-via-a-endpoint-qualification.md) now requires `canonical_cross > 0` and rejects D_035. [D_034 negative shallow 4:3 Via-B](d034-non45-oblique-mirrored-via-b-endpoint-qualification.md) and [D_033 negative steep 3:4 Via-A](d033-non45-steep-mirrored-via-a-endpoint-qualification.md) remain distinct.
- [Public synthetic fixture](../../crates/boardsi-import/tests/fixtures/d035_non45_oblique_mirrored_via_a_endpoint_v94.asc), opt-in `--d035-non45-oblique-mirrored-via-a-endpoint-contact-v1 NET_A 1 4 0 1`.
- Dedicated wire `BOARDSI_D035_NON45_OBLIQUE_MIRRORED_VIA_A_ENDPOINT_CONTACT_V1`; source edge candidate `SOURCE_NON45_OBLIQUE_MIRRORED_VIA_A_ENDPOINT_CONTACT_CANDIDATE`.
- Exact candidate topology: **8 SOURCE nodes / 6 same-layer candidate edges / 3 local components / 0 proven electrical cross-layer edges**.
- SOURCE witness explicitly states `SOURCE_GEOMETRY_ONLY`, `PLATING_IMPORTER_ASSUMED`, `BARREL_ELECTRICAL_NOT_VERIFIED`, `CROSS_LAYER_ELECTRICAL_NOT_PROVEN`, `SOLVER_READY_FALSE`.

## Code PR #449 — exact final HEAD independently accepted

- [Issue #448](https://github.com/july-rust/BoardSI/issues/448); [code PR #449](https://github.com/july-rust/BoardSI/pull/449).
- Final code PR HEAD [`29e3a32237b576053eb59caf7fada1a806afc7b4`](https://github.com/july-rust/BoardSI/commit/29e3a32237b576053eb59caf7fada1a806afc7b4/checks): **34/34 actual pull_request Check Runs SUCCESS; 11/11 actual pull_request Workflows SUCCESS**. Separate synthetic dynamic PR overview run was not counted as an Actions workflow.
- [Numerical Core #41951441753](https://github.com/july-rust/BoardSI/actions/runs/41951441753): Rustfmt, Clippy, Rust Debug and Release tests, D_035 source-only exact candidate, branch reversal/SHA lexical binding, hostile source, D_029/D_033/D_034 mode isolation, and existing native Windows x64/ARM64/macOS ARM64 Rust jobs SUCCESS.
- [Qt Runtime #41951441750](https://github.com/july-rust/BoardSI/actions/runs/41951441750): Linux Qt 6.8.3 Offscreen **57/57 CTest PASS**. Native Windows x64, Windows ARM64 and macOS ARM64 Qt jobs **SUCCESS**; D_035 dedicated Qt selftest logs report all checked assertions true and `NO_ELECTRICAL_PROOF`. Independently reimported exact source, SHA, TRACE and topology; verified forged SHA+TRACE, wrong signed slope/angle, 1-tick shift, stale receipt, wrong mode and unjustified electrical promotion fail closed.
- Independent exact-final-HEAD Codex review receipt `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; **0 unresolved review threads**.
- **Historical failures retained:** first code HEAD `087246c17fde3c4c696b11dbbcc16c957419b315` Rustfmt failure; initial independent Codex P1 findings (missing D_035 CLI dispatch and erroneous Qt steep flag) were fixed through `29e3a322...`. The two original review threads were resolved; the latest exact HEAD was independently re-reviewed clean. Earlier failures are not retroactively reclassified as success.

## Protected code-main Squash and exact Push — PASS

- Protected GitHub **Verified one-parent Squash** [`main@51370cbb30cc5c9b3b8bf22ab243b367871bf486`](https://github.com/july-rust/BoardSI/commit/51370cbb30cc5c9b3b8bf22ab243b367871bf486): one parent `810ba2f342baf1bec9a41569c15255ff9389c8e4`, `commit.verification.verified=true`, `reason=valid`; protected `main` points to exact SHA at acceptance.
- Independently queried exact code-main SHA and `event=push`: **8/8 workflows completed SUCCESS and 28/28 check runs completed SUCCESS**. [Qt Push #41953244820](https://github.com/july-rust/BoardSI/actions/runs/41953244820), [Numerical Push #41953244926](https://github.com/july-rust/BoardSI/actions/runs/41953244926). No failed/skipped code-main latest checks claimed as passes.

## Separate two-Markdown documentation PR and Issue closeout — PENDING

- Exactly two files: [canonical checklist](../development-checklist.md) and this validation ledger. **No code, workflow, platform test, ruleset or required-context changes**.
- Use existing docs-only CI classifier: native compiler jobs intentionally **SKIPPED**, required contexts maintained; report SKIPPED separately from SUCCESS.
- Completion requires exact final docs PR HEAD CI, independent same-HEAD Codex completed/no findings/zero unresolved threads, protected Verified single-parent docs Squash, independently verified exact docs-main `event=push` workflows and checks. Record actual docs PR number, exact final SHA and administrative Issue #448 closure **in Issue Comment**; do not create recursive documentation PR.
- Parent Issue #12 stays OPEN; legacy PR #35 CLOSED/UNMERGED.

## Explicit exclusions — NOT_VERIFIED

Manufactured copper, plated Via barrel electrical continuity, full-board Open/Short, P009/PH-08 reference return-current path, RLGC/S-parameter solver fidelity, actual physical Scenario/Cache, formal U/T and product release are **NOT_VERIFIED**. `SOURCE_GEOMETRY_ONLY` is candidate geometry, not an electrical proof.
