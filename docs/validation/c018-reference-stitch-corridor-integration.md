# C_018 — Bounded Stitch Via / Copper Corridor V5–V8 Integration

**Tracker:** [Issue #290](https://github.com/july-rust/BoardSI/issues/290) · **Code PR:** [#291](https://github.com/july-rust/BoardSI/pull/291) · **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN.  
**Source baseline:** `main@96c3c9f74dd7326dd309341ed4a7bb4dfb04e667` · **Evidence / proposed limited first-check date:** 2026-10-30 KST.  
**Scope:** public synthetic PADS V9.4 MILS, supported source-only subset. No newly implemented source geometry, PCB electrical connectivity, physical return path, or solver.

## Five new integrated regressions

`crates/boardsi-import/tests/c018_reference_stitch_corridor_integration.rs` contains exactly five cross-version Rust cases:

1. One unchanged source SHA for opt-in V5, V6, V7 and V8. Their independent exact byte receipts are nested in order V8 ⊃ V7 ⊃ V6 ⊃ V5 with source-only output states.
2. Reference Stitch Via has strict local Source contact, but signal corridor and Via contact belong to two disconnected same-Net Source Area IDs. V6 `REFERENCE_AREA_MISMATCH` carries through V7 `V6_CORRIDOR_RISK` and V8 `V7_REFERENCE_FOOTPRINT_RISK`.
3. V6 corridor remains candidate, while an entire Reference Stitch Via circular Pad touches the Source outer boundary or nearby COPCUT. V7 `PAD_BOUNDARY_RISK` / `PAD_CUTOUT_RISK` survive V8.
4. Exact source-tick pad separation, tangency and overlap have distinct `SOURCE_PAD_SEPARATE_OBSERVED`, `SOURCE_PAD_TANGENT_RISK`, `SOURCE_PAD_OVERLAP_RISK`; V7 ancestry remains unchanged.
5. Changed Source route bytes change V5–V8 SHA; wrong Reference Via index, unsupported Reference layer or mismatched Reference Net fail closed.

Existing C_009–C_012 Qt CPU/RHI display, stale generation/reimport and Windows x64/ARM64/macOS ARM64 native selftests are **reused unchanged**, as is Linux Qt offscreen CTest. This is not a new dedicated Qt IPC selftest and does not claim real GPU-device performance.

## Independently inspected code gates (2026-10-30 KST)

| Gate | Result | Source |
| --- | --- | --- |
| Exact final code PR HEAD | `ad43b4703985f53c6dd5c1bcf202d412535f6fcf`; **22/22 Check Runs SUCCESS**; **8/8 PR Workflows SUCCESS** | [checks](https://github.com/july-rust/BoardSI/commit/ad43b4703985f53c6dd5c1bcf202d412535f6fcf/checks), [PR #291](https://github.com/july-rust/BoardSI/pull/291) |
| Pinned Rustfmt + Clippy | **PASS**, corrected original first-HEAD formatting failure before final HEAD | [Numerical Core](https://github.com/july-rust/BoardSI/actions/runs/41008869337) |
| New Rust integration regression | **Debug 5/5 PASS, Release 5/5 PASS**, and previous workspace regression PASS | [Numerical Core](https://github.com/july-rust/BoardSI/actions/runs/41008869337) |
| Qt Native + Linux offscreen | Windows x64, Windows ARM64, macOS ARM64 + Linux Qt jobs SUCCESS on final source HEAD | [Qt Runtime](https://github.com/july-rust/BoardSI/actions/runs/41008869338) |
| Independent Codex same exact HEAD | `status=completed, has_any_findings=false, has_unresolved_findings=false`; zero unresolved review threads | [Codex receipt](https://github.com/july-rust/BoardSI/pull/291#issuecomment-6515781997) |
| Protected Squash merge | `main@f387fbbda933479b765aa685f7cffe66a8bfff69`; **one parent**, REST signature `verification.verified=true, reason=valid` (verified_at 2026-10-30T05:41:41Z) | [merged commit](https://github.com/july-rust/BoardSI/commit/f387fbbda933479b765aa685f7cffe66a8bfff69) |

**Independent post-code-main CI VERIFIED on 2026-10-30 KST:** [exact signed merged `main@f387fbb` check-runs](https://github.com/july-rust/BoardSI/commit/f387fbbda933479b765aa685f7cffe66a8bfff69/checks) **21/21 SUCCESS**; **9/9 associated workflow runs SUCCESS**, including Qt Windows x64/ARM64, macOS ARM64, Linux Qt, Rust Native and dynamic `Push on main`. [Qt post-main workflow](https://github.com/july-rust/BoardSI/actions/runs/41009850278). Source-only C_018 code acceptance first checked **2026-10-30 KST**, evidence date **2026-10-30 KST**. Documentation PR review, merge and independent post-document-main CI remain separate pending gates, not implied by the code verdict.

## Exclusions and C_019 handoff

This acceptance is **source geometry and existing Qt risk preview only**. In particular a Stitch candidate or apparent pad separation is not return-current continuity, electrical contact, an RF stitching effectiveness measurement, manufactured Copper fill, thermal/antipad realization, an impedance/DRC pass, RLGC/S, or production Solver/Cache validation.

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

Formal U09/U10/U39, T02/T04/T24/T52, historical PH-08 Gate Q, and parent M1 [#12](https://github.com/july-rust/BoardSI/issues/12) remain unchecked/open. Next C_019 checks **existing C_013–C_016 V9–V12 mixed Cutout / Signal Via Pad risk contracts** as a separate bounded regression; no new physics capability is implied.

**Documentation PR gate (separate):** This ledger and dated checklist must pass their own exact-final-HEAD CI and independent completed same-HEAD zero-finding Codex review, protected signed Verified Squash, and independent post-document-main CI. Do not pre-certify those future checks.
