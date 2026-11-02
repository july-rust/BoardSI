# D_010 — PADS plated Through-Via two-layer / two-trace / four-area Source candidate

**First recorded:** 2026-11-02 KST. **Evidence state:** PENDING exact-head protected CI, independent review, verified squash and independent post-main CI. **Tracking:** [Issue #362](https://github.com/july-rust/BoardSI/issues/362), [Code PR #363](https://github.com/july-rust/BoardSI/pull/363).

## Narrow supported geometry

- Explicit PADS V9.4 MILS, exactly one declared plated Through-Via with two round conductive pads on separate selected copper layers A/B; no hypothetical pads from a net name.
- Exactly four distinct same-net rectangular source COPCLS areas: A/C on layer A and B/D on layer B, with zero or one source-owned rectangular COPCUT on each area.
- Exactly two positive-width, straight Source traces, one on each layer. Each has **exactly one endpoint strictly inside** its layer's Via pad and the opposite endpoint strictly inside its corresponding distinct target area C or D; target endpoint must not touch the Via-pad area or its own cutout, and target area must not intersect the Via Pad.
- Reject wrong ownership or source layer/net, missing/ambiguous Pad, unsupported geometry or arcs, COPCUT/boundary, extra via/trace/area/pour, coordinate overflow, source SHA mismatch, forged receipt, stale imported source, and source-import truncation. Negative geometry remains **NOT_VERIFIED**, never a qualified electrical path.
- Rust opt-in: \`boardsi-import-artwork <file.asc> --d010-dual-layer-two-trace-v1 RA RB RC RD NET_A 1 4 0\`. Test-only dedicated binary \`boardsi-d010-two-trace-four-area\`. Receipt is source-SHA-bound. Default importer and D_001–D_009 remain unchanged.
- Qt uses the *actually accepted imported artwork* (two tracks, via padstack, four copper owners, cutouts, source SHA) and independently recomputes contact candidates. Negative/re-import conditions must clear markers, never project old markers to a new source.

## Acceptance evidence ledger

- [ ] Exact final PR #363 HEAD: numerical/Rust fmt + Clippy + Debug/Release all targets and targeted D_010 positive/negative tests PASS.
- [ ] Linux Qt CTest (including D_010) and actual three Native Qt platforms Windows x64/ARM64, macOS ARM64 PASS.
- [ ] Exact HEAD independent Codex zero unresolved findings and all relevant review threads resolved.
- [ ] GitHub protected Ruleset 25046595 required CI pass, Verified one-parent Squash main SHA.
- [ ] Independent **exact merged main SHA** event=push CI pass, not inferred from PR runs.
- [ ] Checklist acceptance evidence / final issue closeout documented with dates and exact SHA (not backdated).

**Pending items are not evidence of test success.** Fail-closed on any open gate.

## Unsupported / deliberately not accepted

\`\`\`text
electrical_connection = NOT_VERIFIED
via_barrel_continuity = NOT_VERIFIED
return_path_p009 = UNSUPPORTED
physical_si = NOT_RUN
solver_ready = false
cache_reuse_eligible = false
\`\`\`

No manufactured pour-fill or thermal connection, no real inter-layer electrical connectivity, no global open/short, PH-08, P009 or solver/cache. Formal M1 U09/U10/U39 and T02/T04/T24/T52 not accepted; [parent #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN, Draft PR #35 unchanged.
