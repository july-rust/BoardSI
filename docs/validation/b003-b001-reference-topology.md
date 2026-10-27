# B_003 — B_001 source-backed Trace/Pad/Via connectivity continuation

Date: 2026-10-27 KST. Tracking: [Issue #12](https://github.com/july-rust/BoardSI/issues/12). Code: [Draft PR #224](https://github.com/july-rust/BoardSI/pull/224).

## Baseline and bounded candidate

Starting protected `main@fca3ffda5fcd03a44510c57a4a253caa603d991c`, verified signed. The [development checklist](../development-checklist.md) was read before coding. Existing [A_032 exact endpoint graph](a032-p2w-local-connectivity.md) and [B_002 limited audit revision](b002-b001-completion.md) were reused.

First implementation increment: strictly collinear original PADS source-tick straight-trace T-junction witnesses, validated again by the Canonical model using i128 cross and dot products. Same Layer plus equal source-declared Net IDs are still required for a contact edge; a cross-Net coincidence adds only a potential-conflict count and never an edge. One tick off centreline must not connect. This evidence does not validate trace widths, Pad contacts, arbitrary crossings, Area/Plane, reference return path or whole-board Open/Short.

The importer currently assigns `Pad.net=None` to restricted source PARTDECAL/PART pads, as the Net association is not qualified. B_003 MUST NOT fabricate Pad-Net membership or accept Pad electrical connectivity based on proximity. This is an explicit blocker for the rest of B_003 until an independently source-backed pin/net binding is available.

## Verification ledger — do not pre-accept

- [ ] Exact HEAD Rust fmt/clippy and importer tests: **PENDING**.
- [ ] Native Qt Windows x64, Windows ARM64, macOS ARM64, Linux: **PENDING**.
- [ ] 10k route and previous A_032 regression: **PENDING**.
- [ ] Independent Codex same-HEAD review: **NOT_RUN**.
- [ ] Protected signed squash merge + independent post-main checks: **NOT_RUN**.
- [ ] Re-read checklist after test completion, add only accepted scoped subitem with first-check KST and evidence date; otherwise leave formal boxes unchanged.

**State:** B_003 PARTIAL / B_001 PARTIAL. Issue #12 stays OPEN. T02/T04/T24/T52 and U09/U10/U39 are NOT accepted; `return_path_p009=UNSUPPORTED`, `solver_ready=false`, `physical_si=NOT_RUN` remain unchanged.

## Additional work remaining for bounded B_003

1. Verify actual accepted PADS Pad-to-source-Net association rather than inventing connectivity from a pad label or geometric overlap.
2. Implement qualified Trace-to-Pad contact for supported Pad geometries, including negative tests for unknown-net pads.
3. Extend qualified Via-to-Trace contact beyond matching endpoints only where exact geometric support and validation are available; preserve Via Span/anti-pad fail-closed boundaries.
4. Verify same-HEAD native CI, independent review, protected merge, post-main checks and then update dated checklist.

No code capability is checked complete until exact-head CI and independent review pass.
