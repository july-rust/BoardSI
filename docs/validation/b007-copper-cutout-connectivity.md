# B_007 — PADS Source-bound COPCUT-aware local contact acceptance ledger

Status: **IMPLEMENTATION CANDIDATE; NOT ACCEPTED PENDING EXACT-HEAD NATIVE CI, INDEPENDENT REVIEW, PROTECTED SIGNED SQUASH MERGE AND POST-MAIN TESTS.**

Tracking: [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12); prior B_003 [#224](https://github.com/july-rust/BoardSI/pull/224), B_004 [#226](https://github.com/july-rust/BoardSI/pull/226), B_005 [#227](https://github.com/july-rust/BoardSI/pull/227), B_006 [#229](https://github.com/july-rust/BoardSI/pull/229). Initial protected main: `f3fd85076c9d2cc57d69b21c91a3a2ce15e42bd9` (2026-10-28 KST). [Development checklist](../development-checklist.md) was read before any code changes.

## Exact limited implementation

- Original PADS V9.4 MILS *LINES* COPPER: **one** straight source COPCLS simple outer polygon and exactly **one** strictly enclosed source-owned straight COPCUT polygon; all source coordinates on the restricted integer tick grid.
- B_004 legacy `augment` remains hole-free. New `augment_source_bound` reparses the **exact original source bytes**, matches SHA-linked Canonical source, complete source Area inventory, contour geometry, layer order, traces and net definitions before allowing a hole. Qt import binary calls this new entrypoint.
- Model validates each complete polygon, source-grid frame/range, positive area, no backtracking or self-crossing, hole strict inclusion/no contour touching, and rejects both the void interior and **every hole boundary** for `AreaInterior` nodes.
- Existing B_004 source declared Trace Endpoint/SMD Pad Centre, B_006 plated Through Via Centre and their independent Layer/Net/Padstack/Drill qualification are retained. Only positive sites on remaining fill may join via a common source Area; a single strictly internal simple hole cannot divide the mathematically connected simple polygon fill.
- Qt V1 local contact counts and CPU/RHI Artwork wire contract stay unchanged.
- Conservative exclusions: multiple COPCUT, circle/curved cutout, curved/arc outer, split/overlapping areas, thermal/antipad clearance, pad body/trace width crossings, fabricated fill/Return Path. Signal/Reference same-named areas never join merely by labels.

## Mandatory acceptance evidence — remain unchecked until directly proved

- [ ] Exact PR HEAD Rustfmt, Clippy, Numerical Debug/Release B_007 tests and all B_003–B_006 regression SUCCESS.
- [ ] Exact PR HEAD Windows x64/Windows ARM64/macOS ARM64 Rust/Qt Native and Linux CI SUCCESS; legacy Qt V1 and RHI artwork regression.
- [ ] Independent Codex **exact latest HEAD** review no blocking findings; any historical review threads resolved.
- [ ] All protected GitHub required checks SUCCESS.
- [ ] Protected ruleset preserved; **Signed Verified Squash Merge** to main.
- [ ] Independently verified post-main exact merged SHA Checks and separate Push on main workflow.
- [ ] First checked KST date, evidence date, exact PR/CI/commit references reconciled in [checklist](../development-checklist.md).

## Safety boundary

`return_path_p009=UNSUPPORTED`, `solver_ready=false`, `physical_si=NOT_RUN`, `cache_reuse_eligible=false` are unchanged. No claim of actual manufactured copper fill, full-board Open/Short, continuous high-frequency current return, RLGC/S-parameters, loss/roughness physics, scenario/cache acceptance or formal U09/U10/U39/T02/T04/T24/T52 acceptance. **M1 B_001 parent Issue #12 stays OPEN.**
