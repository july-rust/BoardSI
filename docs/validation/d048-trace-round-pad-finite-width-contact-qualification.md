# D_048 — SOURCE-Exact Finite-Width Trace-to-SMD Round Pad Contact Qualification

**First checked: 2026-11-07 KST. Evidence date: 2026-11-07 KST.** [Issue #487](https://github.com/july-rust/BoardSI/issues/487) · [Code PR #488](https://github.com/july-rust/BoardSI/pull/488) · [Canonical checklist](../development-checklist.md).

## 1. Qualification boundary

The opt-in `SOURCE_LOCAL_TRACE_ROUND_PAD_FINITE_WIDTH_ONLY` diagnostic establishes **strict positive-area SOURCE-local copper contact candidates only**. It does not establish manufactured board continuity, global Electrical Open/Short, arbitrary PCB connectivity, P009/PH-08 reference-return verification, physical Scenario/Cache, RLGC/S Solver, formal U01–U48/T01–T54 acceptance, or release. The parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.

[Source fixture](../../crates/boardsi-import/tests/fixtures/d048_trace_round_pad_width_v94.asc): public synthetic PADS V9.4 MILS, 0.01 mil = 254 nm per SOURCE tick, Layer 1 and NET_A/NET_B. Preserve the exact D_040 4 SMD Pad / 4 straight Trace / 2 local component baseline, and add precisely one 8-mil-width straight SOURCE-owned Trace with pair U1.1/U2.1 from (222,285) to (222,315) mil. It approaches **U1.1**, a 40-mil-diameter round SMD Pad centered at (200,300) mil, without crossing the Pad with its centerline or contacting fixed base rails. All four Pad and four baseline Trace coordinates, shape, width, owner, layer, net, and source pair are verified.

The nearest centerline–Pad-center separation is 22 mil; Pad radius = 20 mil; new round-capped Trace half-width = 4 mil. Thus strict `20 < distance < 24` mil yields one positive-area Pad–Trace contact. Expected output: **9 SOURCE Nodes, 7 same-layer candidate edges, 2 local components, 0 proven electrical cross-layer edges**. New edge: `E|6|0|8|POSITIVE_FINITE_WIDTH_TRACE_ROUND_PAD_CONTACT`.

## 2. Exact oracle and adversarial coverage

[Rust i128 implementation](../../crates/boardsi-import/src/d048_trace_round_pad_width.rs) independently computes exact SOURCE tick center-to-clamped-segment squared distance using endpoint projection and cross products. Its `positive_round_contact` requires strict `pad_radius² < distance² < (pad_radius + trace_half_width)²`. Equality at either bound fails closed: the centerline must remain outside the conductive Pad, and a mere outer tangency is not a positive overlap. SOURCE values are integers, and bounded intermediate products avoid floating-point tolerance and overflow.

[Native Qt independent qint64 oracle](../../apps/boardsi-qt-prototype/d048_trace_round_pad_width_qt.h) rebuilds source geometry from actual [PreviewImportPanel Qt reImport](../../apps/boardsi-qt-prototype/d048_qt_runner.cpp), independently regenerates the original SOURCE SHA-256-bound receipt, and never accepts Rust-generated candidate edges without exact receipt parity. Full pad/trace SOURCE inventory is bound before emitting the fixed two-component graph.

| Scenario | Centerline minimum to U1.1 center | Qualified result |
| :--- | ---: | :--- |
| 8-mil trace (222,285)→(222,315) mil | 22 mil | Strict positive contact |
| 8-mil cap (217,316)→(220,319) mil | √545 ≈ 23.35 mil | Round-cap positive contact |
| Trace at x=223.99 mil | 23.99 mil | 1 SOURCE tick inside outer threshold, positive |
| Trace at x=224 mil | 24 mil | Outer tangency; reject |
| Trace at x=224.01 mil | 24.01 mil | 1 SOURCE tick beyond tangency; reject |
| Trace at x=225 mil | 25 mil | Gap; reject |
| Trace at x=219 mil | 19 mil | Unsupported centerline-inside-Pad; reject |
| Trace (222,322)→(230,330) mil | √968 ≈ 31.11 mil | AABB-only overlap false positive; reject |

Source owner/pin-pair, wrong Net/Layer/Pad Shape/Width, moved second component, missing/duplicate source, unsupported sections, fabricated Receipt/edge and stale SHA must fail closed. Supported reverse direction, NET_B, CRLF, tabs, and supported header variants preserve the SOURCE-local result as covered by the dedicated [Rust tests](../../crates/boardsi-import/tests/d048_trace_round_pad_width.rs); the Native Qt runner additionally imports positive, cap, reverse, one-tick, tangent/gap, AABB-only, wrong owner/layer/pad shape, missing source and forged receipt variants.

## 3. Verified code and independent main-push evidence

- **Final code PR HEAD:** [`0b381936fc2415a209096583a2f86907911a8580`](https://github.com/july-rust/BoardSI/commit/0b381936fc2415a209096583a2f86907911a8580/checks), **34/34 pull_request Check Runs SUCCESS, 11/11 pull_request Workflows SUCCESS**. An earlier HEAD had four `cargo fmt --check` formatting differences; those exact logged differences were applied, and the final HEAD's `Format and warnings` job succeeded. Old failure is not retroactively marked successful.
- **Rust:** D_048 integration tests **Debug 3/3 and Release 3/3 PASS**; Rustfmt and Clippy check SUCCESS. Linux Native Qt **70/70 CTest PASS**; `qt-preview-d048-trace-round-pad-finite-width-source` passed with `D048_NATIVE_QT_TRACE_PAD_WIDTH true`. Existing platform matrix Windows x64 / Windows ARM64 / macOS ARM64 Native Qt also SUCCESS.
- **Independent Codex:** `@codex review` on final PR HEAD received `chatgpt-codex-connector[bot]` 👍 (no suggestions); GitHub unresolved review threads = **0**.
- **Protected Squash:** [code-main `19dcfe92af17392d2677ad02cf07c818c03266ee`](https://github.com/july-rust/BoardSI/commit/19dcfe92af17392d2677ad02cf07c818c03266ee), sole parent `884d7b522198612bd7a394a447db6ca115d877bc`, GitHub REST `verification.verified=true/reason=valid`, exact main-ref match.
- **Independent exact code-main SHA event=push:** **8/8 Workflows and 28/28 Check Runs SUCCESS** verified separately from PR checks. See [D_048 Issue #487](https://github.com/july-rust/BoardSI/issues/487) for final documentation gate chronology.

## 4. Checklist interpretation and separate documentation gate

Canonical Stage D limited-evidence checked count moves **68→69**, pending stays **2**; overall limited-evidence checked count moves **139→140**, pending stays **122**. These are **bounded test evidence counts, not a product completion percentage**.

This new ledger and the canonical checklist constitute the only two Markdown changes in the independent documentation PR. The existing lightweight docs-only CI classifier should skip unnecessary Native platform jobs. Require exact docs PR HEAD checks, independent Codex review, zero unresolved threads, protected GitHub Verified one-parent Squash and exact docs-main event=push validation. Close [Issue #487](https://github.com/july-rust/BoardSI/issues/487) only after recording all observed terminal evidence; do not recursively create another documentation PR. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN.