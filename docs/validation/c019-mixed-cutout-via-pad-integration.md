# C_019 — Bounded Mixed Cutout / Via Pad V9–V12 Source Receipt Integration

**Tracker:** [Issue #293](https://github.com/july-rust/BoardSI/issues/293) · **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN · **Previous:** [C_018 #290](https://github.com/july-rust/BoardSI/issues/290) CLOSED.

**Baseline:** `main@38e50d9e5f97d025b28a2bb353bd19262bb59a05` (2026-10-30 KST). **Status: IN PROGRESS — no C_019 PASS or first-check claimed yet.**

## Exact bounded implementation

`crates/boardsi-import/tests/c019_mixed_cutout_via_pad_integration.rs` adds five public synthetic PADS V9.4 MILS cross-version regressions:

1. Single rectangular COPCUT: V9 embeds original V8; V10 embeds byte-identical V9; V11 embeds original V8; V12 embeds byte-identical V11, preserving SHA/source-only state.
2. Multiple rectangular COPCUTs: V9 fails closed; V10 uses genuine original V8 instead of fabricated V9; V11 and V12 retain exact ancestry and holes.
3. Mixed rectangular COPCUT/circular COPCCO: V9/V10 fail closed, V11/V12 include actual Source V8 contact candidate; independent V12 Stitch Pad rows.
4. Rectangle tangent/overlap: original V9/V10 risk states agree, exact nested receipts survive V12.
5. Changed source SHA, invalid Stitch Via/Reference Net/Layer/Area, overlapping circular void: reject unsupported or invalidate old receipt without physical safety upgrade.

Existing C_013–C_016 Qt CPU/RHI native selftests, stale/reimport and Linux offscreen tests are **reused**, not newly implemented. An unexecuted dedicated Qt C_019 test is not claimed.

## Required independent evidence (not pre-certified)

- [ ] Pinned Rustfmt and Clippy; C_019 5/5 Debug and 5/5 Release plus prior B/C regressions.
- [ ] Exact final code PR HEAD Check Runs/Workflows SUCCESS; same exact HEAD independent Codex completed/no findings/unresolved threads.
- [ ] Existing Qt Native Windows x64/ARM64, macOS ARM64 and Linux offscreen jobs actually completed.
- [ ] Protected one-parent Signed Verified Squash and independent post-code-main Check Runs and Push Workflow Runs.
- [ ] Checklist first-check (KST) and supporting evidence date, separate documentation exact-head review/CI, protected Squash and post-document-main checks.

## Non-claims

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

No physical return path P009, manufactured Copper/antipad/thermal DRC, RLGC/S-parameter Solver, real GPU performance or U09/U10/U39 / T02/T04/T24/T52 formal signoff. Keep parent #12 OPEN. C_020 separately audits B_007 final Codex/legacy connectivity evidence.
