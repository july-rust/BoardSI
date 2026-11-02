# D_010 — PADS plated Through-Via two-layer / two-trace / four-area Source candidate

**First recorded:** 2026-11-02 KST. **Code acceptance:** VERIFIED for restricted Source geometry only (2026-11-02 KST). Exact code PR, independent reviewer, protected merge and independent post-code-main CI verified; documentation PR/issue closeout is a separate gate. **Tracking:** [Issue #362](https://github.com/july-rust/BoardSI/issues/362), [Code PR #363](https://github.com/july-rust/BoardSI/pull/363).

## Narrow supported geometry

- Explicit PADS V9.4 MILS, exactly one declared plated Through-Via with two round conductive pads on separate selected copper layers A/B; no hypothetical pads from a net name.
- Exactly four distinct same-net rectangular source COPCLS areas: A/C on layer A and B/D on layer B, with zero or one source-owned rectangular COPCUT on each area.
- Exactly two positive-width, straight Source traces, one on each layer. Each has **exactly one endpoint strictly inside** its layer's Via pad and the opposite endpoint strictly inside its corresponding distinct target area C or D; target endpoint must not touch the Via-pad area or its own cutout, and target area must not intersect the Via Pad.
- Reject wrong ownership or source layer/net, missing/ambiguous Pad, unsupported geometry or arcs, COPCUT/boundary, extra via/trace/area/pour, coordinate overflow, source SHA mismatch, forged receipt, stale imported source, and source-import truncation. Negative geometry remains **NOT_VERIFIED**, never a qualified electrical path.
- Rust opt-in: `boardsi-import-artwork <file.asc> --d010-dual-layer-two-trace-v1 RA RB RC RD NET_A 1 4 0`. Test-only dedicated binary `boardsi-d010-two-trace-four-area`. Receipt is source-SHA-bound. Default importer and D_001–D_009 remain unchanged.
- Qt uses the *actually accepted imported artwork* (two tracks, via padstack, four copper owners, cutouts, source SHA) and independently recomputes contact candidates. Negative/re-import conditions must clear markers, never project old markers to a new source.

## Verified code integration evidence — 2026-11-02 KST

- [x] **체크일: 2026-11-02 KST · 근거일: 2026-11-02 KST.** [Final Code PR #363 HEAD `e88d67827131d705682953f819688dfdd7566684`](https://github.com/july-rust/BoardSI/commit/e88d67827131d705682953f819688dfdd7566684/checks): **27/27 Check Runs SUCCESS, 11/11 `event=pull_request` Workflows SUCCESS**; Rust 1.75 fmt/Clippy, Debug/Release, D_010 six positive/negative tests both configurations PASS, existing D_001–D_009 regressions preserved.
- [x] **체크일: 2026-11-02 KST · 근거일: 2026-11-02 KST.** Linux Qt 6.8.3 Offscreen **33/33 CTest PASS** including actual D_010 `D010_NATIVE_QT_TWO_TRACE_FOUR_AREA true true true true true true true true PHYSICAL_NOT_VERIFIED_P009_UNSUPPORTED`; actual D_010 Native Qt smoke on Windows x64, Windows ARM64 and macOS ARM64 each **SUCCESS**, using real accepted PADS Import artwork and SHA-bound receipt.
- [x] **체크일: 2026-11-02 KST · 근거일: 2026-11-02 KST.** [Independent Codex review on PR #363](https://github.com/july-rust/BoardSI/pull/363): exact final HEAD `e88d67827131d705682953f819688dfdd7566684`, `status=completed/has_any_findings=false/has_unresolved_findings=false`. Initial earlier-HEAD P1 Via route layer token concern corrected in both Rust fixture and embedded Qt Source fixture; thread marked resolved, no unresolved threads.
- [x] **체크일: 2026-11-02 KST · 근거일: 2026-11-02 KST.** Active main Ruleset **25046595** with **17 mandatory checks**, GitHub protected one-parent Squash [`main@dacffbba5ffe082df6ef7417073f6359a76608da`](https://github.com/july-rust/BoardSI/commit/dacffbba5ffe082df6ef7417073f6359a76608da), GitHub `verification.verified=true/reason=valid`, single parent `d24e48cee745026d1bf6b505ea0432ad9e8b8cbb`.
- [x] **체크일: 2026-11-02 KST · 근거일: 2026-11-02 KST.** **Independent exact post-code-main SHA `dacffbba5ffe082df6ef7417073f6359a76608da`** [21/21 Checks SUCCESS](https://github.com/july-rust/BoardSI/commit/dacffbba5ffe082df6ef7417073f6359a76608da/checks) and **8/8 separately checked `event=push` workflows SUCCESS**, all `head_sha` matches code merge, not PR checks; additional dynamic [CodeQL run #41386460966](https://github.com/july-rust/BoardSI/actions/runs/41386460966) **4/4 analysis Jobs SUCCESS**. [Post-main native Qt run #41386460832](https://github.com/july-rust/BoardSI/actions/runs/41386460832) SUCCESS including three Native environments.

**Documentation and issue closeout gate (separate):** This evidence reconciliation/document PR requires its **own** final exact-HEAD CI, independent Codex review, protected Verified one-parent Squash, and independent post-document-main `event=push` CI. Only after the document gate may [Issue #362](https://github.com/july-rust/BoardSI/issues/362) be closed; the eventual result belongs in the document PR / issue closure comments, not backdated into this code-evidence record.

## Unsupported / deliberately not accepted

```text
electrical_connection = NOT_VERIFIED
via_barrel_continuity = NOT_VERIFIED
return_path_p009 = UNSUPPORTED
physical_si = NOT_RUN
solver_ready = false
cache_reuse_eligible = false
```

No manufactured pour-fill or thermal connection, no real inter-layer electrical connectivity, no global open/short, PH-08, P009 or solver/cache. Formal M1 U09/U10/U39 and T02/T04/T24/T52 not accepted; [parent #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN, Draft PR #35 unchanged.
