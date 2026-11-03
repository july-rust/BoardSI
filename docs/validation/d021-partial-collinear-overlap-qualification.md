# D_021 — Proper partial collinear Source Trace overlap qualification

- Tracker: [Issue #403](https://github.com/july-rust/BoardSI/issues/403)
- Status: **PENDING** exact-final-HEAD CI, independent review, protected merge and post-main verification. No results pre-accepted.
- Base: protected main `4fd71b4a63aab1ab2e73ccc08025c55bb1f6af03`, existing D_015/D_019/D_020 Source-only inventory.
- Opt-in: `--d021-partial-collinear-overlap-v1 NET_A 1 4 0 1`.
- Public synthetic fixture: `crates/boardsi-import/tests/fixtures/d021_partial_collinear_overlap_v94.asc`.

## Exact geometry contract

PADS ASCII V9.4 MILS Source tick equals 0.01 mil = 254 nm. Layer-4 main Trace is (15000,25000) → (35000,25000); partial Trace is (30000,25000) → (40000,25000) in exact integer ticks. Proper overlap is [30000,35000] in main Trace direction.

Only one endpoint of the branch may lie strictly inside main; the other must be strictly outside main (collinear beyond one endpoint). Nonzero dot and exact zero cross product prevent parallel-offset or zero-length promotion. Reverse branch and overlap at the other main endpoint are supported, and reported interval is main-direction normalized. Contained, equal, endpoint-only, gap, wrong Source geometry/Net/Layer/extra Artwork fail closed. Existing D_020 strict containment remains separate.

Output is SHA-256-bound Source-only Receipt, with exact `OVERLAP|MAIN_TRACE|1|PARTIAL_TRACE|3|START|30000|25000|END|35000|25000` and `SOURCE_COLLINEAR_PARTIAL_OVERLAP_CANDIDATE` edge kind. Graph nodes/edges/components are bounded candidate inventory, not a proof of all coincident physical interfaces or complete conductive connectivity. Via intersection cannot be promoted to a barrel electrical edge.

## Test and protection plan — do not pre-mark completed

- [ ] Final code HEAD `cargo fmt --all -- --check`, Clippy and Rust hostile regression validated.
- [ ] Linux Qt Offscreen CTest including native D_021 pass.
- [ ] Independent Windows x64, Windows ARM64 and macOS ARM64 Qt real-import and Receipt forgery/stale regressions pass.
- [ ] 17 Strict required GitHub Checks and complete PR Workflows pass at exact final HEAD.
- [ ] Independent Codex completed on same HEAD without findings or unresolved review threads.
- [ ] Protected Signed Verified single-parent Squash and exact merged SHA independent push CI pass.
- [ ] Issue #403 safely closed only after all prior evidence.

**Explicitly out of scope:** manufactured copper continuity, actual electrical via barrel conduction, whole-board Open/Short, physical Return Path P009, PH-08, scenario/solver cache, RLGC, S parameters and formal U/T acceptance. Parent Issue #12 remains OPEN, existing Draft PR #35 untouched. No release acceptance.
