# D_021 — Proper partial collinear Source Trace overlap qualification

- Tracker: [Issue #403](https://github.com/july-rust/BoardSI/issues/403)
- Status: **SOURCE_GEOMETRY_ONLY / bounded Source-contact candidate qualification COMPLETE (2026-11-03 KST)**; not physical electrical or product acceptance.
- Base: protected main `4fd71b4a63aab1ab2e73ccc08025c55bb1f6af03`, existing D_015/D_019/D_020 Source-only inventory.
- Opt-in: `--d021-partial-collinear-overlap-v1 NET_A 1 4 0 1`.
- Public synthetic fixture: `crates/boardsi-import/tests/fixtures/d021_partial_collinear_overlap_v94.asc`.

## Exact geometry contract

PADS ASCII V9.4 MILS Source tick equals 0.01 mil = 254 nm. Layer-4 main Trace is (15000,25000) → (35000,25000); partial Trace is (30000,25000) → (40000,25000) in exact integer ticks. Proper overlap is [30000,35000] in main Trace direction.

Only one endpoint of the branch may lie strictly inside main; the other must be strictly outside main (collinear beyond one endpoint). Nonzero dot and exact zero cross product prevent parallel-offset or zero-length promotion. Reverse branch and overlap at the other main endpoint are supported, and reported interval is main-direction normalized. Contained, equal, endpoint-only, gap, wrong Source geometry/Net/Layer/extra Artwork fail closed. Existing D_020 strict containment remains separate.

Output is SHA-256-bound Source-only Receipt, with exact `OVERLAP|MAIN_TRACE|1|PARTIAL_TRACE|3|START|30000|25000|END|35000|25000` and `SOURCE_COLLINEAR_PARTIAL_OVERLAP_CANDIDATE` edge kind. Graph nodes/edges/components are bounded candidate inventory, not a proof of all coincident physical interfaces or complete conductive connectivity. Via intersection cannot be promoted to a barrel electrical edge.

## Verified D_021 evidence (2026-11-03 KST)

- [Issue #403](https://github.com/july-rust/BoardSI/issues/403) CLOSED/completed at `2026-11-03T12:59:13Z` after explicit post-main success confirmation.
- [PR #404](https://github.com/july-rust/BoardSI/pull/404) final code HEAD [`60ba127d9435e82b1a27a7d7cc714be336888750`](https://github.com/july-rust/BoardSI/commit/60ba127d9435e82b1a27a7d7cc714be336888750/checks): **34/34 Check Runs SUCCESS; 11/11 pull_request Workflows SUCCESS**. Rust fmt / strict Clippy / hostile tests, Linux Qt offscreen CTest, native Windows x64/ARM64 and macOS ARM64 Qt Source reimport and independent Receipt reconstruction covered by existing CI.
- Final HEAD Codex independent code review `completed`, `has_any_findings=false`, `has_unresolved_findings=false`; unresolved review threads 0.
- GitHub protected [Signed Verified single-parent Squash main `516c5de1480b77ae726971f83c138932ba535fff`](https://github.com/july-rust/BoardSI/commit/516c5de1480b77ae726971f83c138932ba535fff), parent `4fd71b4a63aab1ab2e73ccc08025c55bb1f6af03`; `verified=true / reason=valid`.
- Exact merged SHA independent [post-main checks](https://github.com/july-rust/BoardSI/commit/516c5de1480b77ae726971f83c138932ba535fff/checks): **28/28 Check Runs SUCCESS; 8/8 push Workflows SUCCESS**. [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/41638787878) attempt 1 SUCCESS. [Numerical Core](https://github.com/july-rust/BoardSI/actions/runs/41638787928) attempt 2 SUCCESS after same-SHA failed-job-only Windows ARM64 rerun.
- Historical incident **not erased**: first post-main Windows ARM64 Numerical Core run failed *pre-existing* `d006_via_three_area_cli` tests `copper_cutout_blocks_candidate` and `different_source_bytes_cannot_reuse_sha`; logs included temporary-file NotFound and unexpected `HOLE|NONE`. Concurrent temporary-file naming is a **hypothesis, not verified root cause**. Retry success does not prove root-cause remediation; future D_006 stability can be tracked separately.
- Bounded candidate topology in synthetic two-Via/four-Trace model: **8 nodes / 5 layer-local contact-candidate edges / 3 components / 0 proven electrical cross-layer edges**. No electrical/barrel/copper manufacturing claim.

## Completed qualification gates

- [x] Final code HEAD `cargo fmt --all -- --check`, Clippy and Rust hostile regression validated.
- [x] Linux Qt Offscreen CTest including native D_021 pass.
- [x] Independent Windows x64, Windows ARM64 and macOS ARM64 Qt real-import and Receipt forgery/stale regressions pass.
- [x] 17 Strict required GitHub Checks and complete PR Workflows pass at exact final HEAD.
- [x] Independent Codex completed on same HEAD without findings or unresolved review threads.
- [x] Protected Signed Verified single-parent Squash and exact merged SHA independent push CI pass.
- [x] Issue #403 safely closed only after all prior evidence (GitHub auto-close was explicitly reopened until post-main CI completed).

**Explicitly out of scope:** manufactured copper continuity, actual electrical via barrel conduction, whole-board Open/Short, physical Return Path P009, PH-08, scenario/solver cache, RLGC, S parameters and formal U/T acceptance. Parent Issue #12 remains OPEN, existing Draft PR #35 untouched. No release acceptance.
