# D_038 — PADS Reversed Mixed-Sign Shallow 4:3 Via-A Positive / Via-B Negative SOURCE Contact Qualification

**Status: BOUNDED CODE AND EXACT CODE-MAIN PUSH ACCEPTED.** Final documentation-only PR and Issue #457 administrative closeout are separate acceptance gates. **First checked / evidence date: 2026-11-05 KST.** Scope is SOURCE geometry contact candidates only, never fabricated electrical continuity.

## Frozen one-source PADS geometry

- Original input: a single synthetic, redistributable **PADS PowerPCB V9.4 MILS** source, one SOURCE tick = **254 nm**, Net `NET_A`, L4 main trace `(15000,25000)->(35000,25000)`.
- Two Via SOURCE instances: Via-A at `(15000,25000)`, Via-B at `(35000,25000)`. Each has L1 and L4 source Pad candidates; neither real plated Barrel nor cross-layer electrical continuity is demonstrated.
- **Positive Via-A branch:** `(15000,25000)->(3000,34000)`, canonical main displacement `(+20000,0)`; exterior `(-12000,+9000)`; exact `dot=-240000000`, `cross=+180000000`, main-start projection `-240000000 < 0`.
- **Negative Via-B branch:** `(35000,25000)->(47000,16000)`, exterior `(+12000,-9000)`; exact `dot=+240000000`, `cross=-180000000`, main-start projection `640000000 > main length² 400000000`.
- Each branch independently satisfies signed **i128** exact equation `4*abs(cross)==3*abs(dot)`, with nonzero dot, strict exterior projection, and Via-A canonical `cross>0`, Via-B canonical `cross<0`. Input SOURCE trace record direction reversal must not alter classification. **No float tolerance or one-tick forgiveness**.
- Strict SOURCE inventory **3 SIGNAL / 2 Via / 5 Trace segments**, 0 skipped SOURCE constructs, 0 arcs; L1 arms checked with exact general segment intersection and Via-on-arm exclusion.
- Public fixture [d038_mixed_sign_dual_via_endpoint_v94.asc](../../crates/boardsi-import/tests/fixtures/d038_mixed_sign_dual_via_endpoint_v94.asc). Dedicated opt-in mode `--d038-non45-mixed-sign-reversed-dual-via-endpoint-contact-v1` and wire tag `BOARDSI_D038_DUAL_MIXED_SIGN_VIA_ENDPOINT_CONTACT_V1`.

## Independent SOURCE candidate graph

- Exactly **9 SOURCE nodes / 8 same-layer SOURCE contact CANDIDATE edges / 3 local components / 0 proven electrical cross-layer edges**, not a production net-continuity assertion.
- Four SOURCE Via-Pad nodes (A/B each L1/L4) and five Trace nodes. Four candidate Via-Pad/Main-or-Arm contacts, two branch/Main SOURCE contacts, two Via-Pad/Branch SOURCE contacts.
- Rust independently parses the original SOURCE and validates strict bounded inventory, exact signed geometry, Canonical geometry, graph shape and the byte-level **SHA-256** source digest.
- Native Qt independently reimports SOURCE through its actual Preview panel, checks its accepted SHA and imported inventory, recomputes signed geometry and the complete serialized NODE/EDGE/CONTACT graph, and refuses contradictory, stale, or forged wire.
- Wire explicitly says `SOURCE_GEOMETRY_ONLY`, `PLATING_IMPORTER_ASSUMED`, `BARREL_ELECTRICAL_NOT_VERIFIED`, `CROSS_LAYER_ELECTRICAL_NOT_PROVEN`, `SOLVER_READY_FALSE`; summary `OVERALL|DUAL_MIXED_SIGN_POS_A_NEG_B_4_3_SOURCE_CANDIDATE_ONLY`, final `END|FAIL_CLOSED_NO_BARREL_PROOF`.

## Fail-closed adversarial and positive qualification

- Both SOURCE branch record reversals, lexical source-byte changes with independently recomputed SHA, disjoint L1 source arms, and valid 180° rotation of the fixture are accepted as bounded equivalent cases.
- Hostile single-tick slopes, endpoint/gap/offset deviations, wrong signs or wrong 4:3 ratio, wrong Net/Layer/Via/mode, missing/additional/duplicate/unsupported SOURCE content, overlapping/crossing L1 arms, forged SHA/Trace/Contact/Edge/Component counts, stale reimport, and fake electrical promotion are rejected.
- Previously accepted D_037 opposite mixed-sign and D_036 dual negative modes do **not** qualify D_038 input; D_038 does **not** qualify the D_037 fixture.
- **Historical failures preserved:** first code HEAD had Rustfmt line-wrapping failure and two independent Codex P1 findings (Qt CLI mode disagreed with Rust importer; Qt hostile TRACE wire coordinates still used D_037 orientation). Both repaired on later commits, prior review threads resolved, all tests rerun on the **new latest HEAD**.

## Code PR — ACCEPTED

- [Issue #457](https://github.com/july-rust/BoardSI/issues/457), [code PR #458](https://github.com/july-rust/BoardSI/pull/458), final HEAD [`52d5ddc59e4878e9ebc3399a561eff1bbddfb734`](https://github.com/july-rust/BoardSI/commit/52d5ddc59e4878e9ebc3399a561eff1bbddfb734/checks).
- Exact final PR HEAD: **34/34 completed SUCCESS Check Runs**, **11/11 actual `pull_request` Actions Workflows SUCCESS**; 0 failed/in-progress.
- [Numerical CI #41975273279](https://github.com/july-rust/BoardSI/actions/runs/41975273279): Rustfmt and Clippy PASS, D_038 Rust **Debug 4/4** and **Release 4/4** PASS, native Windows x64/ARM64 and macOS ARM64 Rust PASS.
- [Qt CI #41975273361](https://github.com/july-rust/BoardSI/actions/runs/41975273361): Linux Qt 6.8.3 Offscreen **60/60 CTest PASS** including `qt-preview-d038-mixed-sign-dual-via-source` and `D038_NATIVE_QT_DUAL_MIXED_SIGN_VIA_SOURCE true true true true true true true true true NO_ELECTRICAL_PROOF`; actual native Qt Windows x64, Windows ARM64, macOS ARM64 D038 steps SUCCESS.
- Exact same-final-HEAD independent Codex review `completed`, `has_any_findings=false`, `has_unresolved_findings=false`; **0 unresolved review threads**. The two previous P1 findings were fixed and independent re-review succeeded.
- Protected GitHub **Verified single-parent Squash** code-main [`1de0cb2af0e7b0847441ea9fdca71ac413e0967e`](https://github.com/july-rust/BoardSI/commit/1de0cb2af0e7b0847441ea9fdca71ac413e0967e), sole parent `7096bc2f7611f9feb7f205c2b944d57f4f163664`, REST GitHub `verification.verified=true`, `reason=valid`. The actual `main` reference was independently confirmed at this exact SHA.

## Exact code-main Push — ACCEPTED

- At exact `main@1de0cb2af0e7b0847441ea9fdca71ac413e0967e`, **8/8 actual `event=push` Actions Workflows completed SUCCESS** and **28/28 Check Runs completed SUCCESS**; no SKIPPED or failure counted as SUCCESS.
- [Qt Push #41976797229](https://github.com/july-rust/BoardSI/actions/runs/41976797229) and [Numerical Push #41976797281](https://github.com/july-rust/BoardSI/actions/runs/41976797281). Existing native Qt and Rust platform checks succeeded.

## Documentation-only PR and administrative closeout — PENDING SEPARATE GATE

- Documentation changes **exactly two Markdown files**: this ledger and [canonical checklist](../development-checklist.md). All historical evidence and original checked dates retained. Stage D **59 checked / 2 pending**, overall **130 checked / 122 pending**; these counts record bounded cases, **not** product readiness.
- Use existing docs-only classifier to **skip whole native compiler jobs before runner allocation**, keep all required 17 contexts and distinguish `SUCCESS` vs `SKIPPED`. No changes to code, CMake, workflows, branch rules, or runner matrix.
- Require actual exact final docs PR HEAD CI and completed same-HEAD independent Codex review with no findings/threads, GitHub Verified one-parent Squash, exact docs-main `event=push` checks/workflows; close [Issue #457](https://github.com/july-rust/BoardSI/issues/457) **only after** all gates.
- Record immutable final docs PR number/commit, SUCCESS vs SKIPPED, Codex, docs-main verification and Issue closeout in the **final Issue #457 comment**, not a recursively generated docs PR.
- Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN; legacy [PR #35](https://github.com/july-rust/BoardSI/pull/35) remains CLOSED/UNMERGED.

## Explicit exclusions — NOT_VERIFIED

Manufactured Copper/Pad/Trace and plated Via-barrel electrical connectivity, global Open/Short, P009/PH-08 physical return-current continuity, immutable physical Scenario/Cache proof, RLGC/S parameter accuracy, formal product U01–U48 / T01–T54 acceptance, and general distribution/release remain **NOT_VERIFIED**. SOURCE-only geometry candidates shall never become electrical proof by inference.
