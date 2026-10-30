# C_019 — Bounded Mixed Cutout / Via Pad V9–V12 Source Receipt Integration

**Tracker:** [Issue #293](https://github.com/july-rust/BoardSI/issues/293) · **Code PR:** [#294](https://github.com/july-rust/BoardSI/pull/294) · **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN · **Previous:** [C_018 #290](https://github.com/july-rust/BoardSI/issues/290) CLOSED.

**Baseline:** `main@38e50d9e5f97d025b28a2bb353bd19262bb59a05`. **Verdict: LIMITED CODE PASS (2026-10-30 KST)**: original synthetic PADS source-only V9–V12 receipt and inherited Qt preview regression, not physical SI. **First checklist check date: 2026-10-30 KST; evidence date: 2026-10-30 KST.** This documentation PR's subsequent checks/merge are separate and not pre-certified here.

## Five bounded regressions

File: `crates/boardsi-import/tests/c019_mixed_cutout_via_pad_integration.rs` (public synthetic PADS V9.4 MILS, supported subset).

1. Single source-owned rectangle hole: V9 embeds byte-exact original V8, V10 embeds byte-exact V9, V11 embeds genuine original V8, and V12 embeds byte-exact V11. Each retains the original SHA and source-only state.
2. Multiple disjoint rectangular COPCUTs on the two Reference Layers: V9 deliberately errors, V10 embeds genuine V8 (not fabricated V9), and V11→V12 exact receipts remain valid.
3. Rectangle/COPCCO circle mix: V9 and V10 fail closed while V11/V12 retain genuine original V8 Source contact ancestry. V12 emits two Source Stitch Pad rows.
4. Exact rectangular COPCUT boundary tangency and one-source-tick overlap classification: V9 and V10 preserve the same risk classification. V12 embeds the original V11 byte-exactly.
5. Changed original Source Net bytes change all V9–V12 Source SHA; invalid selected Stitch Via, Layer, Reference Net/Area identity, and overlapping circular voids fail closed.

The tests exercise only existing source-geometry contracts, not any new V13 or IPC protocol. C_013–C_016 Qt Native CPU/RHI marker and stale/reimport tests are reused unchanged. This C_019 PR contains **no dedicated new Qt selftest**.

## Independently verified code gates (2026-10-30 KST)

| Gate | Source | Verified outcome |
| --- | --- | --- |
| Exact code PR final HEAD | [`07971e14761c178db6ca790b8851f391a155d591`](https://github.com/july-rust/BoardSI/commit/07971e14761c178db6ca790b8851f391a155d591/checks) / [PR #294](https://github.com/july-rust/BoardSI/pull/294) | **22/22 PR Check Runs SUCCESS, 9/9 associated PR Workflows SUCCESS** |
| Rust pinned format/lint and new integration | [Numerical Core #41014324613](https://github.com/july-rust/BoardSI/actions/runs/41014324613) | **Rustfmt/Clippy PASS; C_019 Debug 5/5 and Release 5/5 PASS**, existing B/C regressions PASS |
| Qt Native reused regression | [Qt runtime #41014324714](https://github.com/july-rust/BoardSI/actions/runs/41014324714) | **Windows x64, Windows ARM64, macOS ARM64, Linux Qt Offscreen PASS**, no C_019-specific Qt test claimed |
| Independent same-exact-HEAD Codex | [PR #294 review summary](https://github.com/july-rust/BoardSI/pull/294) | `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`, zero unresolved threads |
| Protected Signed Verified Squash | [`main@9f4b425f3a8f843e0ac9eade7f69329298d62de8`](https://github.com/july-rust/BoardSI/commit/9f4b425f3a8f843e0ac9eade7f69329298d62de8) | **One parent; REST signature `verified=true`, `reason=valid`** |
| Independently checked code post-main | [exact merged main checks](https://github.com/july-rust/BoardSI/commit/9f4b425f3a8f843e0ac9eade7f69329298d62de8/checks) / [Push on main #41015286806](https://github.com/july-rust/BoardSI/actions/runs/41015286806) | **21/21 Check Runs SUCCESS, 9/9 Workflows SUCCESS**, including native Windows x64/ARM64, macOS ARM64, Linux Qt |

The first code HEAD failed Rustfmt and was corrected at the final `07971e1` HEAD. Failed intermediate checks were not treated as a PASS. The exact final code HEAD was independently reviewed after the correction.

## Non-claims and handoff

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

No manufactured Copper Fill / Antipad / Thermals, DRC clearance, electrical Return Current, P009, RLGC/impedance/S-parameter validation, physical Solver or Cache. Formal U09/U10/U39 and T02/T04/T24/T52 are not accepted and parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) stays OPEN. C_020 handles the separate B_007 final Codex and older Connectivity evidence audit.

**Documentation closure is an independent gate:** this updated ledger and checklist must pass their own exact-final-HEAD required CI and completed same-HEAD independent Codex zero-finding review, protected Signed Verified Squash, and independently checked post-document-main CI. Do not infer that future documentation result from the code result.
