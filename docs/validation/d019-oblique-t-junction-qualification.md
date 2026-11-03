# D_019 — PADS same-layer strictly oblique T-Junction source contact

- Tracking: [Issue #397](https://github.com/july-rust/BoardSI/issues/397)
- Source: public synthetic PADS ASCII V9.4 MILS, source ticks of 254 nm
- Reused: D_015 strict complete inventory, D_014 two-Via/four-Trace Canonical + graph
- Regression neighbors: D_015 perpendicular T, D_016 perpendicular X, D_017 oblique X
- Exact final PR HEAD required CI: **PENDING**
- Independent same-HEAD Codex review and unresolved threads: **PENDING**
- Protected GitHub Verified Squash / merged-main Push CI: **PENDING**

## Source-only candidate contract

Opt-in `--d019-oblique-t-junction-v1 NET_A 1 4 0 1`. The L4 main trace is continuous from first Via to second Via; the second trace has a free endpoint off the main and its other endpoint strictly inside it. Centreline dot != 0 and cross != 0, computed in bounded exact integer arithmetic. The independent Qt decoder reconstructs the expected byte-for-byte receipt using the SHA-256 of the currently imported Source bytes.

**Expected, not yet CI-verified:** 8 Source Nodes, 5 same-layer candidate edges, 3 local components, 0 proven electrical cross-layer edges. A one-tick gap, overrun, main endpoint, perpendicular/collinear case, wrong Net/Layer, partial inventory, extra objects, stale import or modified proof must fail closed.

## Nonacceptance of physical SI

Source-only geometry is not manufactured copper connectivity, Via Barrel electrical conduction, actual Open/Short, P009 Return Path, PH-08, Solver/Cache, RLGC, S-parameters, or the complete U/T tests. Parent [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN; `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`.

## Completion gate

Do **not** mark the single D_019 checklist item complete before exact-head CI, same-head independent review, Signed Verified protected Squash and post-main CI have actually been observed. This ledger records planned criteria, not invented results.
