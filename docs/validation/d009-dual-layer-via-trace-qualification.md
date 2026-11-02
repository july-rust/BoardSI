# D_009 — PADS plated Through-Via dual-layer Pad → Layer-B Trace → third COPCLS Area

**Evidence / first check:** 2026-11-02 KST. **Tracking:** [D_009 Issue #359](https://github.com/july-rust/BoardSI/issues/359); [Code PR #360](https://github.com/july-rust/BoardSI/pull/360). This is **restricted Source geometry qualification only**, not a PCB electrical or SI acceptance.

## Verified scope and fail-closed contract

- Strict opt-in original PADS Layout ASCII V9.4 MILS geometry: exactly one plated Through Via with explicit round conductive pads on two different selected Copper layers, exactly three same-Net source-owned rectangular COPCLS Areas (A on layer A; B and C on layer B), each with zero or one source-owned rectangular COPCUT; and one Layer-B straight positive-width Trace.
- The Via Pad must lie **wholly and strictly** within A and B and avoid owned COPCUT. Exactly one Trace endpoint is strictly within the Via Pad; the other is strictly inside C, disjoint from B and any C cutout. The geometry is only a source contact candidate, not proof of a full-width trace conductor or manufactured inter-layer electrical connection.
- Reject wrong Net/Layer/Via kind, missing selected-layer Pad, incomplete Source import, unsupported shapes, owner ambiguity, COPCUT / tangency / boundary, wrong trace endpoints, Source SHA mismatch, stale re-import, forged receipt/link or out-of-domain coordinate. Do not fabricate omitted pad/copper/barrel continuity.
- Rust opt-in `--d009-dual-layer-via-trace-v1` emits a bounded SHA-256-bound receipt. Native Qt independently re-evaluates the **actual accepted artwork import** (Via, Padstack, Trace, Area ownership, cutouts, geometry) against that receipt and clears stale markers after re-import.
- Default Import, legacy D_001–D_008 wire protocols, electrical connectivity, P009 and solver behavior remain unchanged.

## Verified code PR evidence — 2026-11-02 KST

- [x] [Final exact code PR #360 HEAD `9d73bac244457aa47a536f9d8c6db9bd5a48aa9d`](https://github.com/july-rust/BoardSI/commit/9d73bac244457aa47a536f9d8c6db9bd5a48aa9d/checks): **27/27 Check Runs SUCCESS**, **11/11 pull_request workflows SUCCESS**. Rust format/Clippy and numerical/model integrity Job SUCCESS; source-only Rust fixture positive and negative tests.
- [x] Linux Qt Offscreen **32/32 CTest PASS**, including actual `qt-preview-d009-dual-layer-via-trace` with `D009_NATIVE_QT_DUAL_LAYER_VIA_TRACE true true true true true true true true PHYSICAL_NOT_VERIFIED_P009_UNSUPPORTED`. Native Qt D_009 test step SUCCESS on Windows x64, Windows ARM64 and macOS ARM64. Existing Qt D_001–D_008 coverage was retained.
- [x] Independent Codex review at exactly `9d73bac244457aa47a536f9d8c6db9bd5a48aa9d`: `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; zero unresolved review threads. This is review evidence only, not a physical verification.
- [x] Protected GitHub Squash [`main@ca3dd2224b9bc851914b38f05240ad8b95d1602c`](https://github.com/july-rust/BoardSI/commit/ca3dd2224b9bc851914b38f05240ad8b95d1602c), one parent `aaff99a4d7eac72b2a3fbe3c4c4f5364ca9082cc`, GitHub REST `verification.verified=true`, `reason=valid`.
- [x] Independent post-code-main [exact SHA `ca3dd2224b9bc851914b38f05240ad8b95d1602c`](https://github.com/july-rust/BoardSI/commit/ca3dd2224b9bc851914b38f05240ad8b95d1602c/checks): **21/21 Check Runs SUCCESS** (17 regular + 4 CodeQL), **8/8 `event=push` workflow runs SUCCESS**, each workflow `head_sha` equal to the merged SHA and distinct from the PR run. Additional dynamic `Push on main` CodeQL run **4/4 Jobs SUCCESS**. Representative [post-main Qt run #41379115121](https://github.com/july-rust/BoardSI/actions/runs/41379115121) / [post-main numerical #41379115213](https://github.com/july-rust/BoardSI/actions/runs/41379115213) / [post-main CodeQL #41379115057](https://github.com/july-rust/BoardSI/actions/runs/41379115057).

**Documentation-only closeout:** This record/checklist is a post-code-main evidence reconciliation. Its own documentation PR must independently pass exact-head required CI, independent Codex, protected Verified one-parent Squash and a separate post-document-main push CI. Only then may [Issue #359](https://github.com/july-rust/BoardSI/issues/359) be closed; document verification is recorded in the GitHub PR and Issue final evidence comment rather than asserted in advance.

## Unsupported and not accepted

```text
electrical_connection = NOT_VERIFIED
return_path_p009      = UNSUPPORTED
physical_si           = NOT_RUN
solver_ready          = false
cache_reuse_eligible  = false
```

No manufactured Copper fill/thermal spokes, plated barrel continuity, global Open/Short, complex routing topology, P009/PH-08, SI Solver/Cache, or formal M1 U09/U10/U39 and T02/T04/T24/T52 acceptance. [Parent M1 #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN and Draft PR #35 stays untouched.
