# D_041 — PADS Same-Layer SOURCE Bridge Copper Qualification

**Code and code-main Push accepted (2026-11-05 KST). Docs-only acceptance is recorded on [Issue #466](https://github.com/july-rust/BoardSI/issues/466).** Parent [Connectivity Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.

## Bounded scope

Synthetic [PADS V9.4 MILS fixture](../../crates/boardsi-import/tests/fixtures/d041_local_bridge_v94.asc) (254 nm/source tick). Starting with accepted D_040 SOURCE inventory and two disjoint Layer-1 SMD Pad–Trace–Trace–Pad local components, an explicit fifth, positive-width same-Net Layer-1 Bridge Trace connects **both original junctions exactly**. Expected/verified: **4 Pads + 5 Traces = 9 geometry Nodes, 8 local Edges, 1 SOURCE-owned local Component**; original 4 Pad–Trace + 2 exact local Trace–Trace edges, plus 2 explicit Bridge endpoint edges. Net name alone never creates an edge. Verified supported synthetic NET_A/NET_B only; no global Electrical claim.

The Rust opt-in `boardsi-import-artwork --d041-local-bridge-v1` [encoder](../../crates/boardsi-import/src/d041_local_bridge.rs) reuses D_040 base inventory, source pad-contact audit and two-component isolation, then checks the imported fifth trace and emits an original-SOURCE SHA-256 bound stable receipt. Qt [independent ImportedArtwork reconstruction](../../apps/boardsi-qt-prototype/d041_local_bridge_qt.h) derives all P/T/E records from a fresh Import and compares byte-for-byte, rather than trusting Rust stdout.

Receipt: `BOARDSI_D041_LOCAL_BRIDGE_V1`, `LOCAL_COPPER_BRIDGE_VERIFIED`, `P009_UNSUPPORTED`, `SI_NOT_RUN`, `GLOBAL_ELECTRICAL_NOT_VERIFIED`. All candidate edges are same-layer local SOURCE geometry.

## Regression

Positive: exact Junction-to-Junction Bridge; NET_A and NET_B; reversed Bridge record orientation, four supported V9.4 MILS headers, CRLF and tab-delimited SIGNAL and pin-pair Source; deterministic graph.

Fail-closed: one-Tick endpoint gap; wrong Layer, Net, Width, Pad/Pin/Owner, duplicates or missing records, unsupported Area/Via/Arc/Cutout, bridge crossing Pad envelope, forged receipt/edges and stale Import/SHA. A conservative geometry envelope rejection is **not** physical Short proof.

- Rust [D_041 tests](../../crates/boardsi-import/tests/d041_local_bridge.rs): **3/3** PASS; D_040 predecessor **5/5** PASS. 
- Native Qt [runner](../../apps/boardsi-qt-prototype/d041_qt_runner.cpp): existing Linux offscreen **63/63 CTest** and Windows x64/ARM64 plus macOS ARM64 Native jobs. Existing CMake CI-only test execution; **no new Runner, Workflow or Ruleset**.

## Verified code acceptance

- [PR #467](https://github.com/july-rust/BoardSI/pull/467), exact final HEAD `56f9d8b75c79e471f0caf3a8f322747239775532`: **34/34 PR Check Runs, 11/11 Workflows SUCCESS**; independent exact-HEAD Codex Code Review completed, no findings, **0 unresolved review threads**.
- Earlier failures accurately retained: Rustfmt/Clippy character pattern; Qt mixed-type `auto`; macOS Qt `signals` macro. Earlier Codex P2: CRLF/tab parity and native Windows/macOS actual execution corrected and reverified. No historical FAIL is reclassified.
- Protected one-parent **Verified Squash** code-main [`154e8e24556459044053a9b099ea9c3a5557d824`](https://github.com/july-rust/BoardSI/commit/154e8e24556459044053a9b099ea9c3a5557d824), sole parent `dbc82fa2a315f6fada8d918f49e20869ccdb2a79`, GitHub REST `verification.verified=true/reason=valid`, equal `main` ref.
- Actual code-main **event=push 8/8 Workflows + 28/28 Check Runs SUCCESS** (independent [Qt Push](https://github.com/july-rust/BoardSI/actions/runs/42036225504) and [Numerical Push](https://github.com/july-rust/BoardSI/actions/runs/42036225380)).

## Documentation closeout and limits

Only this validation ledger and [canonical checklist](../development-checklist.md) may change in the separate docs-only PR. Stage D bounded checks **61 → 62**; overall checks **132 → 133**; pending unchanged (**Stage D 2, overall 122**). Docs-only must SKIP native compilation before runner allocation, retain all protected check contexts, require exact final docs PR HEAD review/checks, Verified Squash and independently verified docs-main Push. Immutable final evidence belongs in Issue #466, **not another docs PR**.

**NOT VERIFIED:** manufactured copper continuity, global electrical Open/Short, full Layer/Via/Pour/Thermal connectivity, P009/PH-08 return path, physical Solver/Scenario/Cache, RLGC/S/DC–30 GHz, U01–U48/T01–T54 product acceptance and release. This bounded check count is **not** a product completion percentage. Parent Issue #12 stays OPEN.
