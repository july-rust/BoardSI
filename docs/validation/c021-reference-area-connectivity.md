# C_021 — Two source Reference Copper rectangles: bounded connectivity-risk audit

**Tracker:** [Issue #300](https://github.com/july-rust/BoardSI/issues/300) · Parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN · baseline `main@076f7e739db6ef1517882b3ab4c377c72d4d0922`.

**Status (2026-10-30 KST): LIMITED SOURCE-GEOMETRY CODE MERGED, HEAD CI + NATIVE + EXACT-HEAD CODEX VERIFIED. Post-code-main independent push Check Runs / Workflow Runs and commit-signature REST verification NOT_VERIFIED; final checklist completion gate PENDING.** Never infer electrical Return Path or formal M1 acceptance from a merged PR.

## Implemented candidate, strictly source-only

- Independent opt-in `reference_area_v13::encode` and CLI `--reference-area-v13 RA RB NET_REF 2` (two names, Net label, Source Reference Layer; CLI route coverage not requested). Rust API optionally checks one straight trace route.
- Reparse original bytes, preserve SHA256, Canonical Source Area IDs and explicit Layer/Net label. Exactly two source-owned hole-free COPCLS rectangles; both must have positive dimensions and authenticated Canonical polygon rings. No graph edges, no retroactive C_001 or V1–V12 changes.
- Distinguish positive 2D rectangle intersection from one-tick gaps, zero-area edge/corner touches and disconnected islands. Physical continuity and manufactured fill remain **not proved** even for overlapping source rectangles.
- A straight, adjacent-layer, axis-aligned signal centreline is a *separate* opt-in candidate: both rectangles must contribute strictly positive overlapping open-line intervals covering the entire finite trace, not just share a 2D overlap.
- Unsupported other shapes, holes, more than two Areas, different Net/Layer, stale Source, invalid trace or ambiguous ownership fail closed. Neither a shared Net name nor overlapping source geometry upgrades Canonical electrical Net or Return Current.
- Test file: `crates/boardsi-import/tests/c021_reference_area_v13.rs`, public synthetic PADS V9.4 MILS fixtures only.

## Verified source-code PR and follow-up gates (2026-10-30 KST)

- [x] Code [PR #301](https://github.com/july-rust/BoardSI/pull/301) exact last source HEAD [`a1c7a5a918d848c62a25d495f7e22d58a7513ff0`](https://github.com/july-rust/BoardSI/commit/a1c7a5a918d848c62a25d495f7e22d58a7513ff0/checks): **11/11 HEAD-associated PR workflows completed SUCCESS**, including Numerical Core, Qt Runtime, docs, native FEM, and legacy B/C regressions. Initial intermediate failures (Rustfmt, Qt async test assertion, Clippy) were corrected before final HEAD; do not count earlier failed/aborted heads as success.
- [x] Rustfmt and Clippy -D warnings, numerical Debug/Release and new C_021 six synthetic Rust tests run under final HEAD Numerical Core [#41048613327](https://github.com/july-rust/BoardSI/actions/runs/41048613327); Linux plus Windows x64/ARM64 and macOS ARM64 Native numerical jobs SUCCESS.
- [x] Qt Windows x64, Windows ARM64, macOS ARM64 and Linux Offscreen Native jobs SUCCESS; **dedicated C_021 Qt selftest** executed and passed on Windows x64/ARM64 and macOS ARM64, with Linux offscreen CTest included in the successful job. [Qt Runtime #41048613631](https://github.com/july-rust/BoardSI/actions/runs/41048613631).
- [x] Independent Codex summary on exact final HEAD `a1c7a5a...`: `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; one historical P1 async Qt negative-case thread fixed, marked resolved, no unresolved threads. [Code PR #301](https://github.com/july-rust/BoardSI/pull/301).
- [x] GitHub [PR #301](https://github.com/july-rust/BoardSI/pull/301) **squash merge API reported `merged=true`** and the resulting [`main@555ad01a1e49a60fefb48b6ef00493b6c8ce0b02`](https://github.com/july-rust/BoardSI/commit/555ad01a1e49a60fefb48b6ef00493b6c8ce0b02) appears in the GitHub commit list. The GitHub protection requirements were not bypassed manually. However this tool's commit metadata does **not** include the explicit REST `verification.verified` field or parent count; do NOT claim those two details independently checked.
- [ ] Exact merged main SHA **independent post-main Push CI Check Runs + Workflow Runs**: **NOT_VERIFIED**. Current GitHub connector's commit workflow query returns only PR-associated runs and returns an empty list for this `main` push; absence is **not** a failure or a PASS. Independently inspect [merged SHA checks](https://github.com/july-rust/BoardSI/commit/555ad01a1e49a60fefb48b6ef00493b6c8ce0b02/checks) and `event=push` workflow runs before final promotion.
- [ ] Explicit GitHub signed `verification.verified=true, reason=valid` and single-parent confirmation: **NOT_VERIFIED by current connector metadata**. Squash merge API success does not replace direct cryptographic/metadata verification.
- [ ] Complete dated checklist item and close C_021 Issue #300 only after the above gaps and this documentation PR's own exact HEAD CI/Codex, protected merge, independent post-doc-main checks. Preserve first checked date until actual acceptance.

## Original implementation candidate gates — superseded by above read-only evidence



- [x] Rustfmt, Clippy, Debug/Release C_021 tests and all existing B/C numerical regressions — see verified exact-HEAD results above.
- [x] Qt Native Windows x64/ARM64, macOS ARM64 and Linux offscreen CI; C_021 dedicated Qt UI tests executed on three desktop Native systems (Linux CTest also passed).
- [x] Independent completed Codex review of **exact final source HEAD** with zero findings/unresolved threads.
- [x] GitHub protected Squash merge API success, no explicit bypass — **signature verification is separately NOT_VERIFIED**, see above.
- [ ] Independently checked post-code-main Checks and workflows.
- [ ] Dated checklist completion (first check date and evidence date KST), and documentation-PR independent gate if split.

**Nonclaims:** physical `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`. No fabricated Copper Fill/DRC, Return Current, RLGC/S or Solver/Cache. Formal U09/U10/U39 and T02/T04/T24/T52 unchecked. [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. This is not a replacement for the original B_007 final-head Codex missing receipt.
