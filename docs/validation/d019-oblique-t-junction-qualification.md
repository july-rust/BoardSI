# D_019 — PADS same-layer strictly oblique T-Junction source contact

- Tracking: [Issue #397](https://github.com/july-rust/BoardSI/issues/397)
- Source: public synthetic PADS ASCII V9.4 MILS, source ticks of 254 nm
- Reused: D_015 strict complete inventory, D_014 two-Via/four-Trace Canonical + graph
- Regression neighbors: D_015 perpendicular T, D_016 perpendicular X, D_017 oblique X
- Exact final PR HEAD `5d0ce5f4c6be8b04317ecaeb17e2c982c8db882e` ([code PR #398](https://github.com/july-rust/BoardSI/pull/398)): **34/34 Check Runs SUCCESS, 11/11 pull_request Workflows SUCCESS**
- Independent same-HEAD Codex review: **completed, findings=false, unresolved=false**, open review threads **0**
- Protected GitHub **Signed Verified single-parent Squash**: [`main@b4aa05b90535a6b9d2284e4d4a3397e65257ffc8`](https://github.com/july-rust/BoardSI/commit/b4aa05b90535a6b9d2284e4d4a3397e65257ffc8), GitHub REST `verified=true`, `reason=valid`, parent `ece0acc0d37d62e133b419a929ca8d3add171f06`
- Same exact merged main SHA independent post-main: **28/28 Check Runs SUCCESS, 8/8 event=push Workflows SUCCESS** ([checks](https://github.com/july-rust/BoardSI/commit/b4aa05b90535a6b9d2284e4d4a3397e65257ffc8/checks), [Qt runtime push](https://github.com/july-rust/BoardSI/actions/runs/41607798627), [Numerical core push](https://github.com/july-rust/BoardSI/actions/runs/41607798614))

## Source-only candidate contract

Opt-in `--d019-oblique-t-junction-v1 NET_A 1 4 0 1`. The L4 main trace is continuous from first Via to second Via; the second trace has a free endpoint off the main and its other endpoint strictly inside it. Centreline dot != 0 and cross != 0, computed in bounded exact integer arithmetic. The independent Qt decoder reconstructs the expected byte-for-byte receipt using the SHA-256 of the currently imported Source bytes.

**Verified in bounded Rust/Qt synthetic fixture:** 8 Source Nodes, 5 same-layer candidate edges, 3 local components, 0 proven electrical cross-layer edges. A one-tick gap, overrun, main endpoint, perpendicular/collinear case, wrong Net/Layer, partial inventory, extra objects, stale import or modified proof must fail closed.

## Nonacceptance of physical SI

Source-only geometry is not manufactured copper connectivity, Via Barrel electrical conduction, actual Open/Short, P009 Return Path, PH-08, Solver/Cache, RLGC, S-parameters, or the complete U/T tests. Parent [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN; `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`.

## Completion gate

All code acceptance gates were independently observed on 2026-11-03 KST: exact PR HEAD 34/34 Checks and 11/11 Workflows; same-HEAD independent Codex review `completed`, findings=false/unresolved=false, 0 open review threads; Signed Verified single-parent Squash; exact post-code-main 28/28 Checks and 8/8 push Workflows. Linux Qt CTest 41/41 passed including `qt-preview-d019-oblique-t-junction-source`; standalone D019 native Qt step succeeded on Windows x64/ARM64 and macOS ARM64, including independent post-main re-execution. **Only the bounded source geometry candidate item may be checked**; physical barrel/Copper, P009, PH-08, Solver/Cache and full U/T acceptance remain unapproved. Tracking Issue #397 is closed only after the independent documentation reconciliation gates, not by this code merge.
