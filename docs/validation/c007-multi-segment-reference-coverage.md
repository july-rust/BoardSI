# C_007 — Source-bounded Multi-Segment Reference Coverage and Qt Route Risk

**Tracker:** [Issue #255](https://github.com/july-rust/BoardSI/issues/255) · **Code:** [PR #256](https://github.com/july-rust/BoardSI/pull/256)  
**Baseline:** protected `main@9acb804068a18fa86b943807ac09bf3b713e82e7`  
**First checked (KST):** 2026-10-29 · **Evidence date (KST):** 2026-10-29  
**Code verdict:** **PASS for bounded source geometry and Qt Native risk display ONLY**. **NOT** a physical SI/Return Path acceptance.

## Supported scope

- Opt-in `--reference-route-v3` for explicit source-order **2–16 consecutive straight traces** on one Signal Net and one Copper Layer in the supported synthetic PADS V9.4 MILS subset.
- Each chosen route segment must connect end-to-start in source order, with distinct indices, exactly matching source coordinates/Net/Layer. Duplicated/disconnected/reversed routes, collinear backtracking beyond prior start, self-intersections, unselected same-net contact/branch, source arcs and vias on the path Layer are rejected conservatively.
- Existing C_006 independently source-bound V2 rational reference-gap receipts are enclosed in a new bounded V3 envelope; **C_005 V1 and C_006 V2 remain unchanged**. Source SHA/selected indices/net/geometry are checked again by the Qt consumer, with asynchronous generation and ReImport/source-edit invalidation.
- Qt CPU and Qt Quick/RHI risk spans share the existing selectable gap-interval renderer, scene picking/Properties and source rational parameters. Route Segment and Gap counts are displayed. Projected coordinates/lengths are **approximate and display-only**.
- Route-level `SOURCE_GEOMETRY_ONLY` indicates only narrowly supported original-source geometry. No Net-name-only electrical connection, no automatic path extraction or actual GND return-current proof is implied.

## Actual evidence and bounded acceptance gates

- [x] **Source exact final code HEAD:** [`fdfad35931805e47d6658aeed7966ed3d778b741` 27/27 PR check runs SUCCESS](https://github.com/july-rust/BoardSI/commit/fdfad35931805e47d6658aeed7966ed3d778b741/checks), including Rustfmt, Clippy `-D warnings`, Debug/Release, C_007 synthetic regression, historical B/C regression and Native platforms. Earlier formatting failures and code-review finding were fixed; their historic failures are not final gate successes.
- [x] **Actual Rust process→Qt source-risk:** C_007 route smoke successfully executed on **Windows x64, Windows ARM64, macOS ARM64**, with Linux Qt CTest and historical C_005/C_006 regression; source-bound gap picking, invalid route and stale selection are exercised ([Qt source workflow on final PR HEAD](https://github.com/july-rust/BoardSI/commit/fdfad35931805e47d6658aeed7966ed3d778b741/checks)). Synthetic code and Qt Native acceptance are not proof of physical PCB behavior.
- [x] **Independent Codex same FINAL HEAD:** PR #256 review state `completed`, exact `fdfad35931805e47d6658aeed7966ed3d778b741`, `has_any_findings=false`, `has_unresolved_findings=false`. Initial P2 finding on overlong reverse-collinear backtracking was repaired with explicit regression and its [original review thread](https://github.com/july-rust/BoardSI/pull/256) marked resolved; zero remaining unresolved threads.
- [x] **Protected Signed Verified Squash:** [PR #256](https://github.com/july-rust/BoardSI/pull/256) merged to [`main@08849e3fdf048f60a8a06265792acd4f51316b2f`](https://github.com/july-rust/BoardSI/commit/08849e3fdf048f60a8a06265792acd4f51316b2f). GitHub commit verification `verified=true`, `reason=valid`; no Ruleset #25046595 bypass.
- [x] **Independent post-code-main exact SHA:** [21/21 GitHub check runs SUCCESS](https://github.com/july-rust/BoardSI/commit/08849e3fdf048f60a8a06265792acd4f51316b2f/checks) and **9/9 Workflow Runs SUCCESS** for the same code merge SHA. Includes [Push on main #40781834140](https://github.com/july-rust/BoardSI/actions/runs/40781834140), [Qt Native runtime #40781834617](https://github.com/july-rust/BoardSI/actions/runs/40781834617), [Numerical core #40781834518](https://github.com/july-rust/BoardSI/actions/runs/40781834518), [M2 Native FEM #40781834708](https://github.com/july-rust/BoardSI/actions/runs/40781834708).
- [x] **Living checklist bounded scope**: first checked **2026-10-29 KST**, evidence **2026-10-29 KST**, code HEAD, review, signed Squash and post-main links recorded without changing historical first-check dates.

**Documentation closeout PR remains a separately validated integration**: this code verdict does not pre-certify the documentation PR's own final SHA checks, independent Codex review, signed squash or post-document-main CI.

## Explicitly unaccepted

- [ ] Real manufactured/flooded copper net continuity or arbitrary PADS layouts / via / Arc / branching and reversed traversal.
- [ ] Actual Return Current, electrical P009 / Signal-to-Reference conductor qualification.
- [ ] Physical RLGC, S parameters, impedance, measured PCB validation, production Solver or physically reusable cache.
- [ ] Formal U09/U10/U39 and T02/T04/T24/T52; full [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains **OPEN**.

```text
return_path_p009     = UNSUPPORTED
solver_ready         = false
physical_si          = NOT_RUN
cache_reuse_eligible = false
```

**C_007 code verdict: LIMITED PASS**, no physical SI qualification.
