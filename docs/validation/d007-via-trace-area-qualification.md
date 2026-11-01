# D_007 — Source plated Via Pad → Straight Trace → distinct COPCLS area

**Tracking:** [Issue #352](https://github.com/july-rust/BoardSI/issues/352), parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN. **Code:** [PR #353](https://github.com/july-rust/BoardSI/pull/353). Initial evidence date: 2026-11-01 KST.

## Scope (limited Source geometry candidate)

- The opt-in `boardsi-d007-via-trace-three-area` CLI and `boardsi-import-artwork --d007-via-trace-three-area-v1` inspect immutable PADS V9.4 MILS Source bytes.
- Requires one plated Through Via, supported declared round Conductive Pad on selected layer, one positive-width straight Trace with exactly one endpoint strictly inside the Via Pad, and exactly three same-net/layer Source `COPCLS` rectangular owners with at most one explicitly owned rectangular `COPCUT`.
- Reuses the D_003 three-area Source receipt, D_006 whole circular Via-pad/Area owner test, and D_004 endpoint attachment geometry. The D_004 standalone protocol continues to reject Vias.
- Full Via Pad footprint must be strictly inside one retained Area A. Opposite Trace endpoint must have a unique *different* Area B. Boundary, overlap, COPCUT, same owner, missing selected-layer Pad, source inventory mismatch and unsupported objects never promote a candidate.
- Rust envelope includes a complete embedded D_006 receipt, trace endpoint coordinates, area-owner and component identity, and SHA-256. Qt independently checks the original actual Import, conductive Via padstack, object identities, Source area/cutout geometries, endpoint ownership, forged result and stale re-import.
- Native selftest and Linux offscreen CTest use the existing built artwork importer opt-in; default artwork/canonical connectivity/Solver and previous D_001–D_006 wire contracts are unchanged.

## Acceptance gate (PENDING)

- Exact final PR HEAD numerical / Rust fmt / Clippy / Debug / Release; Windows x64/ARM64, macOS ARM64 Native Qt build and independent D_007 smoke; Linux Qt offscreen CTest.
- Every Ruleset #25046595 required status context; independent Codex review on final exact HEAD with all P1/P2 resolved.
- Protected GitHub-verified, one-parent Squash merge followed by exact merged main SHA checks and push workflows.
- No completion box, evidence count, merge status or Issue closure is to be asserted before these are independently observed.

## Excluded and expressly **NOT VERIFIED**

```
electrical_connection = NOT_VERIFIED
return_path_p009      = UNSUPPORTED
physical_si           = NOT_RUN
solver_ready          = false
cache_reuse_eligible  = false
```

No real manufactured fill, thermal relief, plated barrel electrical continuity, physical Open/Short, multi-Via/Arc/Branch, full topology, P009, PH-08, Solver/Cache or formal U09/U10/U39/T02/T04/T24/T52 acceptance. Parent #12 remains OPEN and Draft #35 remains unchanged. Source endpoint geometry alone cannot certify electrical continuity.
