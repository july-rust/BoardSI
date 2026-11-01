# D_007 — Source plated Via Pad → Straight Trace → distinct COPCLS area

**Tracking:** [Issue #352](https://github.com/july-rust/BoardSI/issues/352), parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN. **Code:** [PR #353](https://github.com/july-rust/BoardSI/pull/353). Initial evidence date: 2026-11-01 KST.

## Scope (limited Source geometry candidate)

- The opt-in `boardsi-d007-via-trace-three-area` CLI and `boardsi-import-artwork --d007-via-trace-three-area-v1` inspect immutable PADS V9.4 MILS Source bytes.
- Requires one plated Through Via, supported declared round Conductive Pad on selected layer, one positive-width straight Trace with exactly one endpoint strictly inside the Via Pad, and exactly three same-net/layer Source `COPCLS` rectangular owners with at most one explicitly owned rectangular `COPCUT`.
- Reuses the D_003 three-area Source receipt, D_006 whole circular Via-pad/Area owner test, and D_004 endpoint attachment geometry. The D_004 standalone protocol continues to reject Vias.
- Full Via Pad footprint must be strictly inside one retained Area A. Opposite Trace endpoint must have a unique *different* Area B. Boundary, overlap, COPCUT, same owner, missing selected-layer Pad, source inventory mismatch and unsupported objects never promote a candidate.
- Rust envelope includes a complete embedded D_006 receipt, trace endpoint coordinates, area-owner and component identity, and SHA-256. Qt independently checks the original actual Import, conductive Via padstack, object identities, Source area/cutout geometries, endpoint ownership, forged result and stale re-import.
- Native selftest and Linux offscreen CTest use the existing built artwork importer opt-in; default artwork/canonical connectivity/Solver and previous D_001–D_006 wire contracts are unchanged.

## Verified bounded code and native matrix evidence — 2026-11-02 KST

- [x] [Code PR #353](https://github.com/july-rust/BoardSI/pull/353), exact final HEAD [`d0992ba6ea204d16c6e915fef32108856ed49087`](https://github.com/july-rust/BoardSI/commit/d0992ba6ea204d16c6e915fef32108856ed49087/checks): **27/27 Check Runs SUCCESS, 11/11 pull_request Workflows SUCCESS**, required #25046595 **17/17 PASS**. Independent same-head Codex state `completed/has_any_findings=false/has_unresolved_findings=false` and zero unresolved threads. Rust source and Qt independent decoder, Source SHA and stale/forged receipts, 30/30 Linux Qt CTest (including `qt-preview-d007-via-trace-area` true). These are source-limited test observations.
- [x] Protected one-parent [GitHub Verified squash `main@4b790edb0bcfdbded1c89b519e2e4b6b1ee35422`](https://github.com/july-rust/BoardSI/commit/4b790edb0bcfdbded1c89b519e2e4b6b1ee35422); Git commit verification `verified=true/reason=valid`, exactly one parent `28c7a914f25740723edb048979fed8d8c94cb492`. Independent exact-sha **21/21 Check Runs, 8/8 push Workflows SUCCESS**.
- [x] [Native-coverage PR #354](https://github.com/july-rust/BoardSI/pull/354), exact final HEAD [`fd6c422727df45f813a701bd563d80bc1231511f`](https://github.com/july-rust/BoardSI/commit/fd6c422727df45f813a701bd563d80bc1231511f/checks): **25/25 Check Runs SUCCESS, 9/9 pull_request Workflows SUCCESS** and required #25046595 **17/17 PASS**. The actual D_007 native Qt step **SUCCESS** independently on Windows x64, Windows ARM64 and macOS ARM64. Same-head Codex `completed/findings=false/unresolved=false`; zero review threads. The existing Linux Qt D_007 30/30 CTest remains passed.
- [x] Protected [one-parent GitHub Verified squash `main@aa6ddf744622e213c50e9e877019a02560ef8755`](https://github.com/july-rust/BoardSI/commit/aa6ddf744622e213c50e9e877019a02560ef8755), `verified=true/reason=valid`, parent `4b790edb0bcfdbded1c89b519e2e4b6b1ee35422`.
- [x] Independent exact [native-coverage merged `main@aa6ddf744622e213c50e9e877019a02560ef8755`](https://github.com/july-rust/BoardSI/commit/aa6ddf744622e213c50e9e877019a02560ef8755/checks) **21/21 Check Runs SUCCESS · 8/8 event=push Workflow Runs SUCCESS**, independently distinct from PR checks. Actual three-OS D_007 Native Qt step succeeded at PR #354; this post-main CI confirms protected integration, not electrical validity.

**Tracking closure rule:** Issue #352 can be marked `CLOSED/completed` only after the dedicated docs/checklist PR itself has exact-final-HEAD CI/Codex, GitHub Verified protected one-parent squash and independent post-main push CI success. Parent M1 #12 stays OPEN; Draft #35 stays unmerged.

## Excluded and expressly **NOT VERIFIED**

```
electrical_connection = NOT_VERIFIED
return_path_p009      = UNSUPPORTED
physical_si           = NOT_RUN
solver_ready          = false
cache_reuse_eligible  = false
```

No real manufactured fill, thermal relief, plated barrel electrical continuity, physical Open/Short, multi-Via/Arc/Branch, full topology, P009, PH-08, Solver/Cache or formal U09/U10/U39/T02/T04/T24/T52 acceptance. Parent #12 remains OPEN and Draft #35 remains unchanged. Source endpoint geometry alone cannot certify electrical continuity.
