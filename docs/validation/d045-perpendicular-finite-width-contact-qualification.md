# D_045 — PADS Same-Layer Perpendicular Finite-Width SOURCE Contact Qualification

**First checked: 2026-11-06 KST; evidence date: 2026-11-06 KST.** [Issue #478](https://github.com/july-rust/BoardSI/issues/478) · [Code PR #479](https://github.com/july-rust/BoardSI/pull/479) · [Canonical checklist](../development-checklist.md).

## Boundary and exact SOURCE

This is an opt-in, narrowly bounded **SOURCE_LOCAL_FINITE_WIDTH_ONLY** diagnostic. The public [D_045 synthetic PADS V9.4 MILS fixture](../../crates/boardsi-import/tests/fixtures/d045_perpendicular_width_v94.asc) uses 0.01 mil = **254 nm per SOURCE Tick**, Layer 1, supported NET_A/NET_B, four SMD Pads, four original 10 mil **vertical** Traces from the D_040 validated two-component source, plus one horizontal bridge Trace, endpoints **(224,350)–(236,350) mil**, width **42 mil**. Original rail centerlines are **x=200 mil** and **x=260 mil**, ending at the common y=350 mil junctions. No vertical and bridge centerlines meet; this is not the D_015/016 centerline T/X-Junction case.

On each side the minimum centerline distance is **24 mil** and combined copper half-width is **(10+42)/2 = 26 mil**: strict positive-area rounded-stroke overlap, not centerline intersection. The synthetic witness has **9 SOURCE Nodes, 10 same-layer local geometry candidate Edges, 1 local Component, 0 proven cross-layer electrical Edges**. Four additional `POSITIVE_FINITE_WIDTH_PERPENDICULAR_CONTACT` edges join the verified original six.

## Exact geometry and Fail-Closed

The [Rust implementation](../../crates/boardsi-import/src/d045_perpendicular_width.rs) and [independent Native Qt reconstruction](../../apps/boardsi-qt-prototype/d045_perpendicular_width_qt.h) independently test minimum Euclidean segment distance for the supported one-horizontal/one-vertical round-capped strokes, using the strict integer relation:

```text
4 * min_centerline_distance_squared < (width_A + width_B)^2
```

Equality is **boundary only**, therefore **rejected**. Rust uses bounded SOURCE Tick `i128` intermediate products; Native Qt uses bounded ImportedArtwork nanometer `qint64` coordinates independently of Rust's candidate edge result. The fixture is expressly bounded by coordinate, width, net, layer, pad, track, ownership and SOURCE inventory constraints; no general-purpose geometry or overflow-safety claim for arbitrary inputs is made. Segment centerline intersections, diagonals, arcs, vias, pours, thermal, unknown layers or unsupported PADS records are rejected instead of being silently accepted.

Required negative examples:

- **38 mil** bridge: combined half-width **24 mil**, exactly tangent to the vertical rails → reject.
- **36 mil** bridge: combined half-width **23 mil**, a positive **1 mil** gap → reject.
- Bridge moved to **y=361 mil**: distance to the two ending vertical segments is `sqrt(24²+11²) ≈ 26.40 mil > 26 mil` even though expanded AABBs can overlap; necessary contact edges are absent → reject. Contacts to other vertical segments do not rescue the required complete four-edge qualification.
- Wrong Layer, Net, Pad/Pin-Pair Owner, inconsistent Width or Source inventory, missing/duplicated candidate, fabricated receipt, stale Source SHA and unsupported geometry → reject.

Support identity-preserving branch reversal, NET_B, supported header variants, CRLF and tabs only after the mutated SOURCE is independently parsed and bound to its own SHA. A SHA-256-bound `BOARDSI_D045_PERPENDICULAR_WIDTH_CONTACT_V1` receipt is compared exactly in the [Native Qt test runner](../../apps/boardsi-qt-prototype/d045_qt_runner.cpp), not inferred from a successful Rust CLI exit.

## D_044 independent negative test debt

This code PR also updates [D_044 Native Qt runner](../../apps/boardsi-qt-prototype/d044_qt_runner.cpp): previously CLI-only failures for corner gap, 48 mil gap, wrong owner/layer and missing SOURCE now exercise each hostile source through actual PreviewImportPanel reimport and reject the Qt-side reconstruction if SOURCE was imported. A Rust CLI rejection alone is not treated as proof of Native Qt parity. It does **not** retroactively rewrite the earlier D_044 checklist/validation claim; this independently verified remediation is recorded here under D_045.

## Evidence and integration chronology

- **Final code PR HEAD** [`5d66d20e93eeb6ceffc84e217f30d786c1a4f240`](https://github.com/july-rust/BoardSI/commit/5d66d20e93eeb6ceffc84e217f30d786c1a4f240/checks): **34/34 Check Runs SUCCESS**, **11/11 pull_request Workflows SUCCESS** across Rustfmt/Clippy/Debug/Release and actual Linux, Windows x64/ARM64, macOS ARM64 Native Qt; independent exact-HEAD Codex `completed/has_any_findings=false/has_unresolved_findings=false`, unresolved review threads **0**.
- Earlier superseded PR HEAD failures: initial Rustfmt check and subsequent Clippy `single_char_pattern` in CRLF test. Both fixed, then **the final HEAD alone** used for acceptance. Historical failures remain visible.
- Protected GitHub **Verified one-parent Squash** [code-main `99695309b1481c03a7e20480a5e97aa4677b18d6`](https://github.com/july-rust/BoardSI/commit/99695309b1481c03a7e20480a5e97aa4677b18d6), sole parent `0921df4c035ebb3643f4772d692b1fa69acdcfa0`; REST verification `verified=true/reason=valid` and exact `main` ref confirmed at merge.
- **Independent exact code-main `event=push` CI**, the separate Markdown-only documentation PR's checks/review/Verified Squash, and **docs-main Push CI** are distinct checkpoints: their terminal evidence and accurate chronology must be placed in the [Issue #478](https://github.com/july-rust/BoardSI/issues/478) final evidence comment after independently confirming results. The documentation must not infer Push success from PR success.
- Issue #478 was auto-closed by the code PR commit on **2026-11-06T03:36:37Z**, before final documentation. Record the chronology explicitly rather than pretending that code merge completed all documentation gates. Do not open a recursive documentation PR.

## Non-acceptance and checklist semantics

One new checked Stage D limited diagnostic: **Stage D 65 → 66 checked**; **overall 136 → 137 checked**; **Stage D 2 and overall 122 pending unchanged**. These are bounded evidence item counts, **not** estimated product completion percentages.

**NOT_VERIFIED:** physical manufactured Copper continuity, global Open/Short, arbitrary PADS board topology, cross-layer barrel/return path, arbitrary Via/Pour/Thermal, P009/PH-08 reference continuity, Physical Scenario/Cache, RLGC/S-parameter solver, U01–U48/T01–T54 formal acceptance, production release. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.
