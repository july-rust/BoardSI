# D_047 — SOURCE-Exact Finite-Width Trace-to-SMD Square Pad Contact Qualification

**First checked 2026-11-06 KST; evidence date 2026-11-06 KST.** [Issue #484](https://github.com/july-rust/BoardSI/issues/484) · [Code PR #485](https://github.com/july-rust/BoardSI/pull/485) · [Canonical checklist](../development-checklist.md).

## 1. Bounded synthetic SOURCE and expected graph

This is an opt-in **SOURCE_LOCAL_TRACE_PAD_FINITE_WIDTH_ONLY** source-geometry diagnostic, not an arbitrary PCB connectivity implementation or manufactured electrical proof. The [public PADS V9.4 MILS fixture](../../crates/boardsi-import/tests/fixtures/d047_trace_square_pad_width_v94.asc) has 0.01 mil = 254 nm per SOURCE Tick, Layer 1, NET_A/NET_B, fixed D_040 four SMD Pads and four 10 mil baseline rails in two independent local components. Exactly one source-owned new 8 mil round-capped Trace is declared by the U1.2/U2.2 SOURCE pin pair from **(217,390) to (217,410) mil**. The supported **30 mil square SMD Pad U1.2** is centered at **(200,400) mil**.

The pad right edge x=215 mil is 2 mil from the trace centerline x=217 mil. A 4 mil trace radius therefore overlaps the pad interior positively. The trace does NOT meet the square with its centerline and does NOT touch the fixed baseline rails or remote U2. Expected output: **4 Pad Nodes + 5 Trace Nodes = 9 Nodes, 6 baseline edges + 1 trace-to-square Pad edge = 7 SOURCE-local Candidate Edges, 2 Local Components, 0 proven electrical cross-layer edges**. Added edge: E|6|1|8|POSITIVE_FINITE_WIDTH_TRACE_PAD_CONTACT.

## 2. Exact independent oracle and fail-closed tests

[Rust implementation](../../crates/boardsi-import/src/d047_trace_square_pad_width.rs) uses bounded SOURCE Tick i128. [Native Qt counterpart](../../apps/boardsi-qt-prototype/d047_trace_square_pad_width_qt.h) independently reconstructs imported artwork in 254 nm Tick units with bounded qint64 operations, without trusting Rust candidate edges.

The entire supported centerline must lie strictly right of the fixed square. Only in this bounded region is the minimum segment-to-square distance exactly the minimum to the **right square edge**. Exact clamped projection compares all four endpoint-to-segment distances using rational squared integer arithmetic. Accept only **4 × minimum_distance² < trace_width²**; equality is boundary-only and rejected. This is not a general rotated-rectangle, arbitrary-polygon or 3D continuity oracle.

| Witness | Nearest separation | Expected |
| :--- | ---: | :--- |
| 8 mil width at x=217, y=390..410 mil | 2 mil | Positive square-side contact |
| 8 mil width from (217,417) to (225,425) mil | √8 mil at corner | Positive round-cap corner contact |
| 8 mil width at x=219 mil | 4 mil | Exact tangent: reject |
| 8 mil width at x=220 mil | 5 mil | Gap: reject |
| 8 mil width from (216,419) to (224,427) mil | √17 mil at corner | AABB-only overlap: reject |

Rust requires the **entire exact original four-Pad/four-baseline-Trace inventory**. Earlier independent Codex P2 found that U2 could be relocated and create an unrecorded additional contact if only U1.2 were checked. The entire baseline SOURCE coordinates, widths, owner, layers and pin pairs are now bounded before contact acceptance. Moved U2, wrong Net/Layer/Owner, unsupported Pad Shape, zero/changed width, missing/duplicated SOURCE, modified receipt, mismatched source SHA, stale import and unsupported sections are fail closed.

The [dedicated Rust integration tests](../../crates/boardsi-import/tests/d047_trace_square_pad_width.rs) cover variants. The [Native Qt runner](../../apps/boardsi-qt-prototype/d047_qt_runner.cpp) reimports every hostile SOURCE into PreviewImportPanel and independently verifies edge construction; Rust CLI rejection alone is never counted as Qt negative-proof.

## 3. Verified code PR and exact main push

- **Final code PR HEAD:** [8797e34e39813060e59cc154cf835dbb4fc87c4b](https://github.com/july-rust/BoardSI/commit/8797e34e39813060e59cc154cf835dbb4fc87c4b/checks). 34/34 PR Check Runs and 11/11 PR Workflows SUCCESS, Rustfmt/Clippy/Debug/Release regression successful, Linux Qt Offscreen **69/69 CTest PASS** (native D_047 runner returned true), Windows x64/ARM64 and macOS ARM64 Native Qt SUCCESS.
- **Independent exact HEAD Codex:** completed, findings=false, unresolved=false; Review Thread unresolved count 0. Previous baseline-inventory P2 resolved.
- **Historical rejected attempts remain recorded:** rustfmt layout failures, an invalid assumption that a hostile relocated-source D_040 test would pass, and unrelated intermittent macOS D_006 temporary-file NOT_FOUND. The D_006 root cause is **not** claimed to be solved.
- **Protected GitHub Verified single-parent Squash:** [code-main 2e45fc5e8ea664956fdbdf240797025b12b8b61f](https://github.com/july-rust/BoardSI/commit/2e45fc5e8ea664956fdbdf240797025b12b8b61f); sole parent dc84008baca4ea1ab461d828af659cc91c9c082b, REST verification.verified=true/reason=valid, main ref identical.
- **Independent exact code-main event=push:** 8/8 Workflows and 28/28 Check Runs SUCCESS, not inferred from pull_request checks.

## 4. Acceptance boundary and documentation gate

Canonical checklist Stage D moves **67→68 checked / 2 pending unchanged**. Overall evidence-items move **138→139 checked / 122 pending unchanged**. These are counts of bounded proofs, **not product completion percent**.

**NOT_VERIFIED:** fabricated PCB electrical continuity, all-board Open/Short, arbitrary PADS/rotated Pad/Area/Via/Pour/Thermal, P009/PH-08 Return Path, actual Scenario/Cache, RLGC/S Solver, U01–U48/T01–T54 formal product qualification, product release.

This documentation PR must contain only two Markdown files, use existing lightweight docs CI and independently verified exact-head Codex, then protected Verified Squash and exact docs-main push. Put the terminal evidence and closure chronology in [Issue #484](https://github.com/july-rust/BoardSI/issues/484) without a recursive third docs PR. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN.
