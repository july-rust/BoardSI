# D_005 — Source-only SMD Pad / Straight Trace / Three-Area COPCUT candidate
**Started 2026-11-01 KST. Tracker:** [#345](https://github.com/july-rust/BoardSI/issues/345). **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. **Baseline:** main@e505211172e89a3eaf1f24458cb6a1340bc030a1.

## Bounded hypothesis
This opt-in audit composes the previously distinct D_001 Source Pin/Pad/Trace endpoint witness and D_004 opposite Trace endpoint / D_003 three source COPCLS component candidate. Exactly one straight Trace, exactly three Source-owned rectangular COPCLS on same declared Net/Layer, max one owned internal rectangular COPCUT, supported R/S/RF SMD pad, strict-internal endpoint and complete source/Canonical inventories. Pad and Area must occupy **opposite endpoints** of the **same Trace**. Multiple eligible routes or ambiguous Pad owner are rejected, not quietly selected. No actual filled Copper or physical connectivity is claimed.

Rust invocation: `boardsi-import-artwork <file.asc> --d005-pad-trace-area-v1 RA RB RC NET_A 1 0`. This produces a bounded SHA-256-tied D_005 wire including the D_004 evidence and one LINK verdict. Ordinary importer and canonical connectivity wire are unchanged. Qt opt-in `--d005-pad-trace-area-selftest` re-checks imported one-Trace/three-area D_004 evidence, Source SHA, actual pad footprint interior and candidate link, and rejects stale changes or forged positive/negative link. An apparent positive candidate is shown as a warning marker only.

## Independently observed limited **code** evidence — 2026-11-01 KST

- [x] [Code PR #346](https://github.com/july-rust/BoardSI/pull/346) exact final HEAD `2b7eaa30edab5145addacb5229b1659069149cac`: **27/27 GitHub Check Runs SUCCESS** and **11/11 `event=pull_request` Workflows SUCCESS**, all head_sha exact match. [Exact HEAD check runs](https://github.com/july-rust/BoardSI/commit/2b7eaa30edab5145addacb5229b1659069149cac/checks).
- [x] [Ruleset #25046595](https://github.com/july-rust/BoardSI/rules/25046595) active; all **17/17** specified required contexts success at final PR HEAD. Separate CodeQL checks also succeeded. Rust formatter, strict Clippy and Debug/Release tests are included in the successful numerical Rust workflow.
- [x] Actual Source-bounded D_005 Native Qt steps **SUCCESS** on Windows x64, Windows ARM64 and macOS ARM64, reusing the actual built Rust importer; [Qt Native CI run](https://github.com/july-rust/BoardSI/actions/runs/41295455528). Linux Qt 6.8.3 offscreen CTest `qt-preview-d005-pad-trace-area` **PASS**, with `D005_NATIVE_QT_PAD_TRACE_AREA true` log. Distinct D_001–D_004 regression steps also succeeded. This is test execution evidence, not board-level electrical accuracy.
- [x] Independent Codex final-head [receipt](https://github.com/july-rust/BoardSI/pull/346#issuecomment-6556462329) says **"Didn't find any major issues"**, reviewed commit `2b7eaa30ed`; no new final-head inline findings. Five earlier P2 source-parity findings were addressed and their review threads resolved. The fixes cover source-declared Pad Pin/Net membership, reversed Trace direction, later Pin-pair rows, `*REMARK*`, and annotated `*SIGNAL* NET 1024 0`.
- [x] Protected **one-parent GitHub Verified squash** merged [`main@60d71061b135c9a7a4f77e6f0912452fbe6e3a96`](https://github.com/july-rust/BoardSI/commit/60d71061b135c9a7a4f77e6f0912452fbe6e3a96): GitHub REST `commit.verification.verified=true/reason=valid`, sole parent `e505211172e89a3eaf1f24458cb6a1340bc030a1`; no bypass asserted. Merge actually completed 2026-11-01 KST.
- [x] **Independent post-code-main exact-SHA Push CI SUCCESS (2026-11-01 KST):** [Verified `main@60d71061b135c9a7a4f77e6f0912452fbe6e3a96`](https://github.com/july-rust/BoardSI/commit/60d71061b135c9a7a4f77e6f0912452fbe6e3a96/checks) independently returned **21/21 Check Runs SUCCESS** and **8/8 `event=push` Workflow Runs SUCCESS** ([exact-SHA runs](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=push&head_sha=60d71061b135c9a7a4f77e6f0912452fbe6e3a96&per_page=100)), with all heads exactly matching the merged SHA. This is a source-only code integration result; it does not approve physical connectivity, P009 or the SI solver.
- [ ] **Separate docs-only reconciliation and Issue #345 closure**: this ledger/checklist evidence must itself be independently reviewed/checked/merged and post-main verified. Tracker stays OPEN until those gates pass.

**Code source-only scope first checked:** 2026-11-01 KST; code PR CI, reviewed HEAD and Verified squash evidence dates: 2026-11-01 KST. No full M1 physical/solver acceptance.

## Mandatory nonclaims
```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
electrical_connection = NOT_VERIFIED
```
Actual PCB electrical continuity, manufactured poured copper/thermal/relief, Return Current/P009, SI solver/cache, PH-08 and complete U09/U10/U39/T02/T04/T24/T52 **NOT VERIFIED**. M1 #12 stays OPEN, Draft #35 stays unmerged. Synthetic fixture validation alone is not real-PCB qualification.
