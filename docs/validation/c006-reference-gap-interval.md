# C_006 — Source-rational Reference Gap Interval / Qt risk spans

**Tracker:** [Issue #252](https://github.com/july-rust/BoardSI/issues/252) · **Code:** [PR #253](https://github.com/july-rust/BoardSI/pull/253) · **Start, first checked and evidence date:** 2026-10-28 KST · **Verdict:** PASS within explicitly bounded Source Geometry + Qt Native scope only. Parent [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.

## Bounded implementation accepted after protected verification

- Existing C_005 V1 source geometry warning remains unchanged for two-trace plated Via transitions. Opt-in V2 `--reference-gap-v2` for one selected straight Signal Trace and one explicitly chosen adjacent source Reference Layer/Net.
- Reuses C_005 V1 independently source-bound PADS import/audit and source SHA/Trace witness. Rust exact rational t intersections of one source-owned straight COPCLS polygon and up to 8 straight COPCUT edges, bounded at 32 gaps. Each proper crossing toggles inside/outside. Inner/outer boundary tangent, collinear line, vertex crossing, overlapping source Reference areas and out-of-range coordinates are refused or UNSUPPORTED, never fabricated as safe. Unknown Source IDs and layers do not produce positive source results.
- Rust wire V2 emits ordered exact t fractions. Display positions/approximate lengths in Qt are only projections of these original rational parameters and must **not** be called exact nanometre intersection coordinates. Source hash, original Track index, Signal Net/Layer, entire Trace endpoints, bounded fractions and all no-physics guards checked at Qt receive.
- CPU GraphicsScene selectable gap spans, Qt Quick/RHI thick line spans and segment proximity picking. Existing UI reimport/source-hash/selected layer/material/Net/Worker late reply staleness policy reused. Old V1 remains callable and unchanged.
- Public synthetic Rust tests include covered, fully uncovered, one/two cutouts, outer crossing, diagonal, collinear unsupported, invalid Reference and V1 compatibility. Qt actual Rust process synthetic cutout native source-risk smoke checks exact `1/6..1/2` and stale source lifecycle, run on Windows x64/ARM64, macOS ARM64; Linux Qt regression remains in existing CTest job.

## Important limitations

- **Not actual manufactured/flooded Copper topology:** Copper Island separation remains conservative and no continuity connection between distinct source areas is inferred. Polyline curves, touching polygon vertices/edges and unsupported geometry are rejected. Polygon arithmetic is exact **within bounded source coordinates**; rendered endpoint/length is approximate for screen visualization.
- **No physical solver qualification:** `return_path_p009=UNSUPPORTED`, `solver_ready=false`, `physical_si=NOT_RUN`, `cache_reuse_eligible=false`. No AC Return Path, RLGC/S, loss physics, S-parameter, full SI physical Cache, arbitrary commercial PADS, or U09/U10/U39/T02/T04/T24/T52 acceptance. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN.

## Final acceptance ledger — independently verified 2026-10-28 KST

- [x] **Code final exact HEAD** [`ed5e137bbb8b1bebcfde832961038d43286a7753`](https://github.com/july-rust/BoardSI/commit/ed5e137bbb8b1bebcfde832961038d43286a7753/checks) from [PR #253](https://github.com/july-rust/BoardSI/pull/253): **27/27 GitHub Check Runs SUCCESS**. `cargo fmt --all -- --check`, Clippy `--workspace --all-targets -D warnings`, Rust Debug and Release workspace tests PASS; public source-bound C_006 dedicated **9/9 synthetic fixtures** exercised in Debug and Release; existing B/C source and IPC regressions PASS. Historical failed candidates were corrected (Copper Piece declaration, Rustfmt, Clippy type alias, MSVC Qt RHI base member shadowing) and do not count as final PASS.
- [x] **Native Qt test and actual Rust importer process proof** on identical final HEAD: Native Windows x64, Native Windows ARM64 and Native macOS ARM64, each `C_006 Native source-rational Reference gap and stale source smoke` **PASS** along with C_005 prior risk test; public synthetic cutout expected interval `1/6..1/2` and stale/source-edit marker removal. Existing Linux Qt CTest and actual offscreen QApplication/Worker test PASS. CPU/RHI code rebuild on native targets PASS (MSVC strict shadow warning fixed). No real PCB hardware/GPU measurement qualified.
- [x] **Independent Codex same-final-HEAD review**: `status=completed`, `commit_sha=ed5e137bbb8b1bebcfde832961038d43286a7753`, `has_any_findings=false`, `has_unresolved_findings=false`. Earlier P2 5-mm CPU scene pen warning fixed with cosmetic 5-pixel pen and review thread resolved. Evidence [PR #253 review](https://github.com/july-rust/BoardSI/pull/253).
- [x] **Strict protected signed Squash Merge**: [`main@5e4de472a90539221f69e5cf11b88e20d9fd7361`](https://github.com/july-rust/BoardSI/commit/5e4de472a90539221f69e5cf11b88e20d9fd7361), GitHub commit `verification.verified=true` / `reason=valid`. No Ruleset #25046595 bypass.
- [x] **Independent post-merge main SHA verification**: [21/21 Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/5e4de472a90539221f69e5cf11b88e20d9fd7361/checks), **9/9 Actions Workflows SUCCESS** including [Push on main #40697041065](https://github.com/july-rust/BoardSI/actions/runs/40697041065), [Qt runtime #40697040984](https://github.com/july-rust/BoardSI/actions/runs/40697040984), numerical core and FEM native runs. Independently checked on signed `main` after merge, not inferred from pull request.
- [x] **Living checklist limited C_006 entry** completed only for this narrow source-geometry and Qt display contract, with **체크일: 2026-10-28 KST** (first checked) and **근거일: 2026-10-28 KST**; original prior first-check dates preserved. This closeout **documentation-only PR also needs its own** exact-head CI, independent review, Signed Verified Squash and post-main checks before the documentation PR is independently accepted.

## Explicitly NOT accepted

- [ ] Reference electrical continuity / AC Return current / validated physical P009.
- [ ] Arbitrary Source copper island merging, manufactured pours, Pad/Port/whole-board physical net completeness and arbitrary PADS versions.
- [ ] Physical RLGC/S-parameters, PH-12–16, production Solver/Cache and measured PCB validation.
- [ ] Full M1 Issue #12, U09/U10/U39 and T02/T04/T24/T52 product acceptance.

**C_006 verdict: PASS (bounded Source-exact rational gap intervals and tested Qt risk-only display). Physical SI and product gates remain NOT_RUN / UNSUPPORTED.**
