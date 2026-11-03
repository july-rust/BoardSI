# D_023 — Opposite Via-A collinear endpoint Source contact qualification

- Tracker: [Issue #409](https://github.com/july-rust/BoardSI/issues/409)
- First recorded: **2026-11-04 KST**
- Acceptance check date: **2026-11-04 KST**
- Evidence date: **2026-11-04 KST**
- Scope status: **LIMITED SOURCE-ONLY CODE QUALIFICATION ACCEPTED**; Issue closure and documentation-only CI require independent status confirmation
- Protected code baseline: `29c3b20b11b8df72a108c7f012af1b8c1acc53cc`
- Dedicated opt-in: `--d023-collinear-opposite-endpoint-contact-v1 NET_A 1 4 0 1`
- Synthetic fixture: `crates/boardsi-import/tests/fixtures/d023_opposite_via_a_endpoint_v94.asc`

## Frozen Source-only qualification contract

PADS V9.4 MILS fixed integer Source tick = 0.01 mil = 254 nm. Layer-4 main Trace (15000,25000) → (35000,25000); separate branch (15000,25000) → (5000,25000); same declared `NET_A`. The two traces share exactly the Via-A Pad endpoint and have **zero positive centreline overlap length**. Main Trace and Via-A Pad remain distinct Source primitives.

Expected exact native Source candidate graph: **8 Nodes / 6 same-layer Candidate Edges / 3 local Components / 0 proven electrical cross-layer Edges**. Contact triangle comprises Node 1 (Via-A Layer-4 Pad), Node 5 (main Layer-4 Trace), and Node 7 (branch Layer-4 Trace). D_023 adds the distinct Layer-4 Pad-to-Branch edge 1–7; all six edges are *candidates*, not manufactured Copper or plated Barrel conduction.

Source inventory is complete in the supported bounded PADS subset. Rust uses exact i128 collinearity and negative exterior projection. SHA-256 binds the full original Source byte stream to the receipt, which Qt independently reconstructs from imported Source/Canonical geometry. Reverse Branch orientation and lexically distinct yet numerically equivalent Source represent valid but different SHA receipts. 1-tick endpoint gaps/offsets, noncollinearity, wrong Net/Layer, partial/contained overlap, degenerate Trace, unmodelled sections or route primitives, stale/forged receipt and unproven electrical promotion fail closed. D_022's Via-B opt-in must reject this Via-A case; D_023 must reject D_022's Via-B case.

## Verified code and CI evidence

- [Code PR #410](https://github.com/july-rust/BoardSI/pull/410): **MERGED** to protected main. Exact final code HEAD [`a39b3da592d60bd66d751f5d6295afc42e009bf8`](https://github.com/july-rust/BoardSI/commit/a39b3da592d60bd66d751f5d6295afc42e009bf8/checks): **34/34 PR Check Runs SUCCESS and 11/11 event=pull_request Workflows SUCCESS**. Rustfmt, strict Clippy and workspace tests, bounded D_023 hostile Source regression and prior D_015–D_022 regression included.
- [Qt Runtime PR run #41728379414](https://github.com/july-rust/BoardSI/actions/runs/41728379414): Linux x86_64 Qt 6.8.3 Offscreen **45/45 CTest PASS**, including the D_023 test. The dedicated D_023 native Qt test succeeded on **Windows x64, Windows ARM64, macOS ARM64** on exact final HEAD; independent Qt Source reconstruction, reimport, SHA-bound receipt, stale/forged/electrical-promotion rejection were exercised.
- Independent exact-final-HEAD Codex Review Summary: **status=completed; has_any_findings=false; has_unresolved_findings=false; unresolved review threads=0**. The earlier superseded P1 Qt flag comment was replied to and resolved, not silently discarded.
- Protected GitHub Signed Verified one-parent Squash: [`main@4de7ba2004e951ac1d6fabf600213e1a7ba4d4be`](https://github.com/july-rust/BoardSI/commit/4de7ba2004e951ac1d6fabf600213e1a7ba4d4be), sole parent `29c3b20b11b8df72a108c7f012af1b8c1acc53cc`, `verification.verified=true` and `reason=valid`.
- Independent exact merged `main` SHA push CI: **28/28 Check Runs SUCCESS and 8/8 event=push Workflows SUCCESS**, with no failed, cancelled or pending checks. This is separate evidence from the PR's 34/34 and 11/11.

## Preserved superseded failures and corrections

- The initial code HEAD's Numerical Core job failed `rustfmt`; the final code HEAD contains the formatting correction.
- Initial Codex P1 found a Qt smoke-test flag that did not match the CLI; corrected before final independent same-HEAD review, and review thread resolved.
- A superseded Qt native D_023 hostile input case mistakenly changed the Via-A/main coordinate along with the branch, forming another valid geometry while expecting rejection. Corrected to change only the Branch in the final code HEAD. This is a **test fixture mutation correction**, not evidence that production manufactured electrical connectivity was ever proven.
- Older D_021 / D_006 Windows ARM64 intermittent test history is not claimed to be root-cause corrected by D_023.

## Explicit excluded acceptance and issue policy

Only these *bounded Source-geometry candidate contacts* have been qualified. This does **not** establish fabricated/manufactured Copper connectivity, Via Barrel plating or electrical continuity, whole-board Open/Short, Reference Plane Split/Cutout return-path continuity, P009/PH-08, physical Scenario/Cache, validated RLGC/S-parameters, solver-ready state or formal U/T/release acceptance.

Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) must stay OPEN. Legacy Draft PR #35 must remain unmerged. Issue #409 administrative closure and docs-only workflow outcome are separately checked from the code qualification and are not inferred from PR merge.
