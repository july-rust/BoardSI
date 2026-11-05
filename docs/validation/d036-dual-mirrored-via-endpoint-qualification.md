# D_036 — PADS Dual Mirrored Negative Shallow 4:3 Via-A/Via-B SOURCE Contact Qualification

**Status:** BOUNDED CODE AND CODE-MAIN PUSH ACCEPTED; exact final code PR HEAD, independent Codex, protected GitHub Verified one-parent Squash, and exact code-main post-merge Push independently confirmed. Administrative docs-only PR and Issue #451 closure are separate gates. The separate Markdown-only documentation PR and its own verified push are administrative acceptance gates; avoid interpreting a bounded SOURCE candidate as electrical proof. **First checked / evidence date: 2026-11-05 KST.**

## Frozen SOURCE and geometry

- PADS PowerPCB V9.4 MILS, 1 source tick = **254 nm**; single `NET_A`, L4 main `(15000,25000) → (35000,25000)`.
- Via-A `(15000,25000)` and Via-B `(35000,25000)` are imported through-via SOURCE entities with L1/L4 Source Pad candidates, not demonstrated conductive barrels.
- **Both branches exist in the SAME imported PADS Source**: A `(15000,25000) → (3000,16000)`, B `(35000,25000) → (47000,16000)`. Input has 3 signal records, 2 Via instances, 5 Trace segments and an independently complete Source inventory; the selection is `NET_A 1 4 0 1`. Reversing either/both branch records preserves signed via-to-exterior classification.
- Main displacement `(+20000,0)`; Via-A exterior `(-12000,-9000)` gives `dot=-240000000`, `cross=-180000000`, exterior projection below zero. Via-B exterior `(+12000,-9000)` gives `dot=+240000000`, `cross=-180000000`, projection strictly greater than main length squared `400000000`.
- For both branches, exact integer `4*abs(cross)==3*abs(dot)`, nonzero dot, **signed cross < 0**, strict Via-owned exterior projection; no floating-point epsilon or one-source-tick forgiveness. The canonical Via→exterior ray is SOURCE-order-independent.
- Both L1 arms are checked with exact general segment intersection and Via-on-Trace predicates rather than a horizontal-only bounding shortcut. Crossing oblique L1 arms fail closed; a 180° reversed-main valid non-contact geometry passes.
- [Public synthetic fixture](../../crates/boardsi-import/tests/fixtures/d036_dual_mirrored_via_endpoint_v94.asc), opt-in CLI `--d036-non45-dual-mirrored-via-endpoint-contact-v1`, distinct witness `BOARDSI_D036_DUAL_MIRRORED_VIA_ENDPOINT_CONTACT_V1`. D_034/035 single-branch modes cannot accept this composite SOURCE.

## Qualified source-owned candidate graph

- Exactly **9 SOURCE nodes / 8 same-layer CONTACT CANDIDATE edges / 3 local connected components / 0 proven electrical cross-layer edges**.
- Nodes comprise 4 layer-specific Via-Pad entries and 5 Trace entries. Candidate SOURCE graph has 4 base Via/Trace touches, 2 L4 main/branch endpoints and 2 additional Via-A/B Pad/branch Source contacts. No Via-barrel cross-layer edge is materialized.
- SOURCE witness is SHA-256 bound to original file bytes and serializes exact Via/Trace coordinates, both contacts and all graph nodes/edges. Independent Qt reconstructs expected wire from imported geometry, validates its own SOURCE inventory and stale-source digest, then requires exact wire equality.
- Explicit `SOURCE_GEOMETRY_ONLY`, `PLATING_IMPORTER_ASSUMED`, `BARREL_ELECTRICAL_NOT_VERIFIED`, `CROSS_LAYER_ELECTRICAL_NOT_PROVEN`, `SOLVER_READY_FALSE`, `END|FAIL_CLOSED_NO_BARREL_PROOF`.
- Fail-closed adversarial coverage: one-tick slope, endpoint gap or offset; wrong signed quadrant/4:3 angle, wrong Net/Layer/Via, duplicate or unsupported SOURCE sections, wrong CLI mode, synthetic SHA/TRACE/contact/edge/graph count and electrical-promotion forgery; stale reimport; reversed trace record and CRLF Windows Source test mutations; unmodeled nonhorizontal Layer-1 arm crossing.

## Code PR evidence and independent review — ACCEPTED

- [Tracking Issue #451](https://github.com/july-rust/BoardSI/issues/451); [Code PR #452](https://github.com/july-rust/BoardSI/pull/452).
- Exact final PR HEAD [`59a6712267f4c96b5b9c6041fa6915112c9e60c3`](https://github.com/july-rust/BoardSI/commit/59a6712267f4c96b5b9c6041fa6915112c9e60c3/checks): **34/34 completed SUCCESS check runs**; **11/11 completed SUCCESS pull_request workflows** (not substituting SKIPPED for SUCCESS).
- [Numerical PR workflow #41960615047](https://github.com/july-rust/BoardSI/actions/runs/41960615047): Rustfmt and Clippy; D036 4/4 Debug and 4/4 Release regression PASS, Windows x64/ARM64 and macOS ARM64 Rust jobs SUCCESS. An initial Windows x64 failure in an existing D_006 temporary-file test was rerun, and the final workflow succeeded. Early D036 Rustfmt, CRLF hostile mutation and Clippy lint failures were fixed and then independently rechecked.
- [Qt PR workflow #41960615058](https://github.com/july-rust/BoardSI/actions/runs/41960615058): Linux Qt 6.8.3 Offscreen **58/58 CTest PASS**; native Windows x64, Windows ARM64, macOS ARM64 Qt jobs SUCCESS. Dedicated independent native D036 Qt log: `D036_NATIVE_QT_DUAL_MIRRORED_VIA_SOURCE true true true true true true true true true NO_ELECTRICAL_PROOF`.
- Codex independent code review on exact final `59a6712...` completed with `has_any_findings=false` and `has_unresolved_findings=false`; both earlier P2 geometry review threads resolved. Historically reported P2 issues: nonhorizontal Layer-1 arms crossing and reversed-main one-sided interval false rejection; both fixed using exact general L1 segment intersection and targeted regression tests. No failing intermediate HEAD is represented as passing.
- Protected GitHub **Verified one-parent Squash** code-main [`ed802d2de37b2a2701db333c625d10f916eeede0`](https://github.com/july-rust/BoardSI/commit/ed802d2de37b2a2701db333c625d10f916eeede0), parent `d00d866b0904a8c37dc2ef68fdcaf171c56a3ca1`; REST `commit.verification.verified=true`, `reason=valid`. The exact protected `main` ref was confirmed at this code commit.

## Code-main push acceptance — PASS

- Independently queried the **exact** code-main SHA [`ed802d2de37b2a2701db333c625d10f916eeede0`](https://github.com/july-rust/BoardSI/commit/ed802d2de37b2a2701db333c625d10f916eeede0/checks): **8/8 `event=push` Actions workflows completed SUCCESS; 28/28 commit check runs completed SUCCESS**. No queued/in-progress/skipped/failed check is represented as passed.
- [Qt Push #41963718339](https://github.com/july-rust/BoardSI/actions/runs/41963718339) succeeded on Linux x86_64, Windows x64, Windows ARM64, macOS ARM64. [Numerical Push #41963718255](https://github.com/july-rust/BoardSI/actions/runs/41963718255) succeeded on Windows x64, Windows ARM64, macOS ARM64, plus numerical job. All six additional actual Push workflows succeeded as recorded by GitHub.
- Synthetic dynamic `Push on main` PR-overview run is **not** counted among the 8 Actions workflows. Protected `main` was independently read at this exact commit before docs branch work.

## Separate documentation PR and Issue closeout

- This ledger and [canonical development checklist](../development-checklist.md) are the **only** files in the docs PR. Do not edit code, CMake, workflows, CI required contexts or settings.
- Existing documentation-only classifier must skip native platform compilation **without allocating platform runners**; evaluate SKIPPED separately from SUCCESS and preserve all required contexts.
- Require exact docs PR HEAD CI, independent latest-HEAD Codex `completed/findings=false/unresolved=false`, 0 unresolved review threads; GitHub Verified single-parent protected Squash; exact docs-main `event=push` workflow/check-run verification.
- Record the actual final docs PR number/SHA, checks, Verified proof and closure of Issue #451 as **Issue #451 final comment**, not by spawning a recursively updating documentation PR.
- Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN; legacy PR #35 CLOSED/UNMERGED.

## Out of scope — NOT_VERIFIED

Manufactured Cu/Pad/Trace/Via-barrel physical electrical continuity, full-board Open/Short, P009/PH-08 return-current topology, actual physical Scenario/Cache, RLGC/S-parameter solver fidelity, full U01–U48 / T01–T54 formal product acceptance and product release are **NOT_VERIFIED**. Same-layer **SOURCE geometry contact candidates are not physical conduction proof**.
