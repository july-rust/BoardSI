# B_007 — PADS Source-bound COPCUT-aware local contact acceptance ledger

Status: **BOUNDED IMPLEMENTATION/PR CI/PROTECTED SIGNED SQUASH/POST-MAIN CI VERIFIED (2026-10-28 KST); LATEST-HEAD CODEX REVIEW CLOSURE STILL PENDING.**

Tracking: [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12); prior B_003 [#224](https://github.com/july-rust/BoardSI/pull/224), B_004 [#226](https://github.com/july-rust/BoardSI/pull/226), B_005 [#227](https://github.com/july-rust/BoardSI/pull/227), B_006 [#229](https://github.com/july-rust/BoardSI/pull/229). Initial protected main: `f3fd85076c9d2cc57d69b21c91a3a2ce15e42bd9` (2026-10-28 KST). [Development checklist](../development-checklist.md) was read before any code changes.

## Exact limited implementation

- Original PADS V9.4 MILS *LINES* COPPER: **one** straight source COPCLS simple outer polygon and exactly **one** strictly enclosed source-owned straight COPCUT polygon; all source coordinates on the restricted integer tick grid.
- B_004 legacy `augment` remains hole-free. New `augment_source_bound` reparses the **exact original source bytes**, matches SHA-linked Canonical source, complete source Area inventory, contour geometry, layer order, traces and net definitions before allowing a hole. Qt import binary calls this new entrypoint.
- Model validates each complete polygon, source-grid frame/range, positive area, no backtracking or self-crossing, hole strict inclusion/no contour touching, and rejects both the void interior and **every hole boundary** for `AreaInterior` nodes.
- Existing B_004 source declared Trace Endpoint/SMD Pad Centre, B_006 plated Through Via Centre and their independent Layer/Net/Padstack/Drill qualification are retained. Only positive sites on remaining fill may join via a common source Area; a single strictly internal simple hole cannot divide the mathematically connected simple polygon fill.
- Qt V1 local contact counts and CPU/RHI Artwork wire contract stay unchanged.
- Conservative exclusions: multiple COPCUT, circle/curved cutout, curved/arc outer, split/overlapping areas, thermal/antipad clearance, pad body/trace width crossings, fabricated fill/Return Path. Signal/Reference same-named areas never join merely by labels.

## Actual verified acceptance evidence — 2026-10-28 KST

- [x] [PR #231](https://github.com/july-rust/BoardSI/pull/231) exact final code HEAD `6a75eb5c5616d86407a04a946bc994958fb55ea6`: **28/28 PR checks SUCCESS**, including native Windows x64, Windows ARM64, macOS ARM64, Linux Qt; Rustfmt, Clippy, numerical and existing B_003–B_006 plus Qt V1/GPU artwork regression. [Exact HEAD checks](https://github.com/july-rust/BoardSI/commit/6a75eb5c5616d86407a04a946bc994958fb55ea6/checks).
- [x] [Numerical core PR HEAD](https://github.com/july-rust/BoardSI/actions/runs/40600884752) runs `b007_cutout_contact`: Debug **11/11 PASS**, Release **11/11 PASS**. Added negative source Pad moved from original hole to Copper, Padstack spoof, altered source bytes, altered Canonical cutout, wrong Net/Layer, boundary/tick, disjoint area and Via Barrel conditions.
- [x] All three previous Codex P2 review threads fixed and resolved. **Do not confuse this with exact latest HEAD final Codex completed review.**
- [x] [Protected Squash Merge PR #231](https://github.com/july-rust/BoardSI/pull/231) to [`main@8aae7504417f6aa8d212a0906de658ed173c4ef9`](https://github.com/july-rust/BoardSI/commit/8aae7504417f6aa8d212a0906de658ed173c4ef9), GitHub REST verification `verified=true, reason=valid`, and protected `main` at this SHA confirmed.
- [x] Independent post-merge `main@8aae7504` [Check runs](https://github.com/july-rust/BoardSI/commit/8aae7504417f6aa8d212a0906de658ed173c4ef9/checks): **21/21 SUCCESS**; all 9 post-merge Actions workflow runs **SUCCESS**, [Push on main #40602499013](https://github.com/july-rust/BoardSI/actions/runs/40602499013) and [Qt Runtime #40602499442](https://github.com/july-rust/BoardSI/actions/runs/40602499442) included.
- [x] Initial limited implementation/post-main checklist check date **2026-10-28 KST**, evidence date **2026-10-28 KST**, SHA/PR/CI links documented in [living checklist](../development-checklist.md).
- [ ] **Independent Codex exact code HEAD `6a75eb5c` final review NOT VERIFIED** — automatic Codex review receipt currently `status=running` (previous reviewed HEAD `da0d2042` had `has_any_findings=false`; latest head differs in regression assertions). Do not mark final Code Review PASS until receipt is `status=completed` with no findings/unresolved threads.
- [ ] This **documentation PR** final HEAD protected CI, independent review, signed Squash Merge and its separate post-main checks must be recorded after they actually succeed.

## Safety boundary

`return_path_p009=UNSUPPORTED`, `solver_ready=false`, `physical_si=NOT_RUN`, `cache_reuse_eligible=false` are unchanged. No claim of actual manufactured copper fill, full-board Open/Short, continuous high-frequency current return, RLGC/S-parameters, loss/roughness physics, scenario/cache acceptance or formal U09/U10/U39/T02/T04/T24/T52 acceptance. **M1 B_001 parent Issue #12 stays OPEN.**


## C_020 retrospective evidence clarification — 2026-10-30 KST

[C_020 audit](c020-legacy-connectivity-review-audit.md) independently checked GitHub PR metadata and existing review artifacts. The historical unchecked line above referring to *this documentation PR* is preserved as an **at-the-time pending gate**; it does not describe current PR #232 state: [documentation PR #232](https://github.com/july-rust/BoardSI/pull/232) is now **CLOSED/MERGED** (`695e43f62fa911af4f6b278481bbb7fdcc043690` → [`main@43c5d54d5af58d110e98a3260f0e93221c2e4896`](https://github.com/july-rust/BoardSI/commit/43c5d54d5af58d110e98a3260f0e93221c2e4896)). Historical workflow/check figures remain credited to their recorded evidence; no new post-merge check-run enumeration is claimed here.

**Unchanged separate gap:** original code PR #231's *exact final* source HEAD `6a75eb5c5616d86407a04a946bc994958fb55ea6` latest Codex receipt still states `status=running`, findings=null; the submitted Codex review body names older `40faa94332`, although three review threads are resolved. **Original exact-HEAD Codex completed/zero-finding receipt = NOT_VERIFIED, not PASS.** C_020's new review cannot retroactively certify #231. Physical Return Path/P009, Solver/Cache, RLGC/S and formal U09/U10/U39/T02/T04/T24/T52 remain unaccepted; Issue #12 remains OPEN.
