# C_008 — Source-bounded Two-Layer Reference Coverage and Plated Via Transition Risk

**Tracker:** [Issue #258](https://github.com/july-rust/BoardSI/issues/258) · **Implementation:** [PR #259](https://github.com/july-rust/BoardSI/pull/259) · **Parent:** [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) (OPEN)  
**Baseline:** `main@f7bc4c1bb56f37ba76815f13b9561595995340d7` (accepted C_007).  
**Evidence date (KST):** 2026-10-29.  
**Status:** Source geometry and Qt risk display **LIMITED PASS (code-merged and independently post-main verified)**. The documentation PR's exact-HEAD acceptance and separate post-document-main CI are distinct gates; neither creates physical SI readiness.

## Bounded implementation

- Opt-in `--reference-route-v4` for explicitly selected **2–16** source-order Straight Traces across **exactly two distinct signal layers** with a single verified plated through Via at the one transition. The route has an explicit nonempty split index, one Signal Net, exact contiguous Source endpoints, and no inferred Net-name continuity.
- Verify Via source/Canonical object, centre, same Signal Net, actual Through kind and **Conductive** padstack entries on both selected layers. Fail closed on duplicate indices, unsupported arcs, self-intersection, backtracking/overlap, unselected same-layer branch, nonunique/unselected via, and a branch through a third conductive layer of the transition Via. Arbitrary PADS syntax, thermal connections, multi-Via topology and whole-board netlist are **not** supported.
- Keep C_005 V1, C_006 V2 and C_007 V3 byte contracts unchanged. The **additive opt-in V4 envelope** carries independently source-bound per-trace C_006 exact rational reference gap witnesses for two separately selected Reference Net/Layer configurations. Source SHA, trace indices/net/layer/geometry, Via name/Source ObjectId/coordinates/span and V2 segment receipts are included.
- Qt Widgets inspect an actual imported PADS source; Qt verifies exact SHA against accepted Import, V4 format and V2 segment witnesses, imported Via placement/net/layer and explicit split. CPU and Qt Quick/RHI share selectable source-gap spans and a separate **`REFERENCE_TRANSFER_UNVERIFIED`** Via marker, both marked **`P009 UNSUPPORTED`** and **`SI NOT_RUN`**. User edits, ReImport, source watcher, rejected route, and asynchronous prior-generation response clear existing risk.
- Rendering distances and projected spans are **display-only approximations**, not exact electromagnetic fields. Neither full Reference geometry coverage nor Plated Via presence certifies actual return-current transfer.

## Verified implementation-PR evidence (2026-10-29 KST)

- [x] [Exact final PR code HEAD `ee765510b1b6f761c31adea8990eb5ef42405f18`](https://github.com/july-rust/BoardSI/commit/ee765510b1b6f761c31adea8990eb5ef42405f18/checks): **27/27 checks SUCCESS**, including Rustfmt/Clippy, Rust Debug/Release/C_008 regressions, historical B/C regressions, Native Rust, Qt Linux and Windows x64/ARM64 and macOS ARM64 actual Rust importer→Qt `--c008-via-selftest`. The initial Rustfmt and two incorrect fixture expectations were repaired before this final HEAD. The F6 runner-side Rust component installation conflict passed on rerun; do not treat historical failed attempts as success.
- [x] Independent GitHub Codex review **completed on the SAME FINAL HEAD `ee765510...`**, `has_any_findings=false`, `has_unresolved_findings=false`; 3 prior review threads corrected and resolved (cutout fixture, false disjoint-Via expectation, third-conductive-layer via branch).
- [x] [Protected Signed Verified Squash PR #259](https://github.com/july-rust/BoardSI/pull/259): `main@5db1840aa267624d20693aef4e48186383e04a57` with REST `commit.verification.verified=true`, `reason=valid`, no Ruleset #25046595 bypass.
- [x] Independently re-read merged **code** [`main@5db1840aa267624d20693aef4e48186383e04a57`](https://github.com/july-rust/BoardSI/commit/5db1840aa267624d20693aef4e48186383e04a57/checks) — **21/21 GitHub Check Runs SUCCESS, 9/9 Workflow Runs SUCCESS** on **2026-10-29 KST**, including [Push on main #40791493502](https://github.com/july-rust/BoardSI/actions/runs/40791493502), [Qt Native #40791493610](https://github.com/july-rust/BoardSI/actions/runs/40791493610), [FEM Native #40791493531](https://github.com/july-rust/BoardSI/actions/runs/40791493531), Windows x64/ARM64 and macOS ARM64. Both previously queued macOS checks completed `success`; none pending or failed.

## Remaining documentary gates

- [x] Recorded independent **code** post-main 21/21 Check Runs and 9/9 Workflow Runs against exact merged SHA; no reused PR-only proof.
- [x] Restrict C_008 checklist approval to exact source-backed geometric/Qt risk visualization only. **First check date: 2026-10-29 KST; evidence date: 2026-10-29 KST**. Preserve all older first-check dates.
- [ ] [Documentation PR #260](https://github.com/july-rust/BoardSI/pull/260): verify its **own exact final HEAD** CI and same-HEAD independent Codex review, then Signed Verified protected Squash and independent post-doc-main Checks/Workflows. Do **not** confuse document-PR evidence with implementation PR #259.

## Explicitly unaccepted product physics and formal requirements

- [ ] Actual electromagnetic `P009` Return Path, Return Current continuity, via stitching/ground-transfer qualification or real flooded Copper geometry.
- [ ] Production RLGC/S parameters/impedance, physical Solver, electrical cache, actual PCB validation or arbitrary source formats.
- [ ] Formal U09/U10/U39 and T02/T04/T24/T52; full M1 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN.

```text
return_path_p009     = UNSUPPORTED
solver_ready         = false
physical_si          = NOT_RUN
cache_reuse_eligible = false
```
