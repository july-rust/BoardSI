# C_011 — Source-exact Reference Stitch Via circular Pad Footprint risk (V7)

**Tracker:** [Issue #267](https://github.com/july-rust/BoardSI/issues/267) · **Code PR:** [#268](https://github.com/july-rust/BoardSI/pull/268) · **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) **OPEN** · **Predecessor:** [C_010](c010-reference-corridor-stitch-continuity.md).  
**Limited CODE acceptance:** **2026-10-29 KST** (first checked and evidence date). **Status:** **source-geometry risk candidate code ACCEPTED on signed protected main**; formal electrical/physical M1 acceptance **NOT RUN / OPEN**. A separate documentation PR and its independent post-doc-main gate are tracked by Issue #267 until explicitly verified.

## Accepted bounded engineering scope (NOT electrical Return Path)

- Opt-in `--reference-route-v7` for public synthetic PADS Layout ASCII V9.4 MILS in the *existing C_010-supported subset*: two explicitly selected Reference layers, Source-owned simple COPCLS/COPCUT polygon Copper Areas, two to sixteen straight Signal segments, one signal plated-through transition and one explicitly selected plated Reference stitching Via.
- Reparse original bytes and Source SHA. Verify source Via index/Net, Canonical Through Via, plated Drill span and positive drill diameter, and exactly one matching *circular conductive Pad* on each selected Reference layer. Width is Source Pad diameter (not an invented clearance or manufactured fill).
- Compare the **entire exact Source-tick circular Pad footprint** with the source polygon outer boundary and supported polygon Cutouts. Centre strictly inside alone is insufficient. Tangency, outer boundary intersection, Cutout contact and unsupported/mismatched geometry become Source risk or fail-closed Unsupported.
- The geometric acceptance predicate uses source-grid integer coordinates, checked construction and `i128` squared distances/cross-products; the comparison does **not** use drawing pixels or floating-point tolerance. Unsupported topology is rejected; no general PCB flood or thermal/antipad geometry is synthesized.
- Independently recompute and **embed the byte-identical V6 receipt**, with its intact V5/V4/V2 evidence. V1–V6 CLI/wire behavior remains unchanged. Two areas with an identical declared Reference Net but different Source Area identities are **not** joined into an electrical path.
- Qt opt-in V7 independently validates Via diameter/drill against the imported source model and accepted SHA, embeds Source Area identity and dimensions in Properties, draws a **source-sized round Pad rim on each selected Reference layer** in CPU and RHI, and uses a bounded adaptive RHI chord tessellation and consistent chord hit-testing. These drawings are source-risk **visualizations**, not EM/solver results.
- Qt invalidates old risk markers/Properties on watcher changes, import replacement, source SHA mismatch, selected trace/Net/Layer/geometry options changes, late generation/IPC results and reset. No persistent physical result or Solver/Cache approval is emitted.

## Source-only states and explicit non-claims

| State | Strictly supported interpretation |
| --- | --- |
| `PAD_FOOTPRINT_CONTAINED_CANDIDATE` | Supported round Pad is fully strictly interior to the same source-owned Reference Copper Area, in **both** selected Source layers; geometric *candidate only* |
| `PAD_BOUNDARY_RISK` | Round Pad touches or intersects an outer Source Copper boundary |
| `PAD_CUTOUT_RISK` | Round Pad touches or intersects a supported Source Copper Cutout |
| `V6_CORRIDOR_RISK` | Prior V6 Source corridor→stitch area identity/gap is not accepted |
| CLI error/Unsupported | Unsupported/wrong Source version, disconnected or ambiguous ownership, non-round/absent Pad, invalid Via/Net/Layer, Source mismatch, other unsupported geometry |

Source Area membership is **not** fabricated/manufactured Copper, thermal spoke or antipad construction, proven electrical contact, RF stitching effectiveness, Return Current, impedance, loss, crosstalk, RLGC/S, or solver-ready geometry. The following flags remain:

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

Formal **U09/U10/U39/T02/T04/T24/T52**, **PH-08 Gate Q**, and the parent **M1 #12** remain **unchecked / OPEN**. No confidential board has been supplied to public CI.

## Verified code evidence — 2026-10-29 KST

- [x] **Exact PR final HEAD:** [`e7f61e3984b9f84a7cfbd3fcf25480c423a41620`](https://github.com/july-rust/BoardSI/commit/e7f61e3984b9f84a7cfbd3fcf25480c423a41620/checks), **27/27 GitHub Check Runs SUCCESS**, **11/11 PR workflow runs SUCCESS**; Rust fmt/Clippy, Debug/Release and prior B/C tests covered by CI. [Native Qt PR run #40838399036](https://github.com/july-rust/BoardSI/actions/runs/40838399036).
- [x] **Public synthetic test suite:** `crates/boardsi-import/tests/c011_reference_stitch_pad_v7.rs` plus inline exact tangent/half-tick tests in `reference_route_v7.rs`; compare positive full-circle containment against tangency, outer overhang, Cutout contact, same-Net disconnected Area, invalid/non-round Pad/Drill and source SHA change. Preserve V6 embedded byte text.
- [x] **Actual Qt Rust Importer→GUI:** `apps/boardsi-qt-prototype/preview_p0.cpp` `--c011-pad-selftest`, `CMakeLists.txt`, `.github/workflows/qt-runtime.yml`. Explicit native C_011 and C_010 tests **SUCCESS** on Windows x64, Windows ARM64 and macOS ARM64; Linux Qt offscreen **SUCCESS**. CPU source Pad diameter check uses 0.889 mm synthetic Pad, with separate two-layer Source rim markers.
- [x] **Independent Codex same HEAD:** [PR #268 review summary](https://github.com/july-rust/BoardSI/pull/268#issuecomment-6490577376) reports `code_review.commit_sha=e7f61e3984b9f84a7cfbd3fcf25480c423a41620`, `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`. Two historical P2 RHI overlay hit-test/tessellation findings were corrected, review threads **2/2 resolved**, then re-reviewed on the exact final HEAD.
- [x] **Protected Signed Verified Squash:** [PR #268](https://github.com/july-rust/BoardSI/pull/268) merged as one-parent commit [`main@32485198fbb24058435f8ef1821f561edef6a793`](https://github.com/july-rust/BoardSI/commit/32485198fbb24058435f8ef1821f561edef6a793), GitHub REST `commit.verification.verified=true`, `reason=valid`. [Active Ruleset #25046595](https://github.com/july-rust/BoardSI/rules/25046595): Strict **17** required status checks, Squash only, required signatures, zero bypass actors.
- [x] **Independent code post-main:** [`main@32485198` **21/21 Check Runs SUCCESS**](https://github.com/july-rust/BoardSI/commit/32485198fbb24058435f8ef1821f561edef6a793/checks); **9/9 associated Workflow Runs SUCCESS** (8 push workflows plus dynamic `Push on main`). [Native Qt push run #40839520313](https://github.com/july-rust/BoardSI/actions/runs/40839520313); all four Linux/Windows x64/Windows ARM64/macOS ARM64 Qt jobs passed. No pending or failed checks in the final code snapshot.

**Limited code acceptance first-check date: 2026-10-29 KST; evidence date: 2026-10-29 KST.** These dates do **not** retroactively change pre-existing checked-item dates. Source-only acceptance does not qualify the parent M1.

## Separate documentation closure record

The protected documentation-only PR that updates this ledger and `docs/development-checklist.md` must independently pass its own **exact-final-HEAD CI and completed zero-finding Codex review**, Signed Verified Squash, then post-document-`main` Checks and Workflow Runs. **Document closure status and resulting SHA are recorded in [Issue #267](https://github.com/july-rust/BoardSI/issues/267), not presumed by this file before the docs merge.** Only then close #267. Parent M1 #12 stays OPEN.

## Scope exclusions

**Not accepted:** physical Return Current/P009, any made Copper flood or thermal/antipad realization, arbitrary commercial PADS, whole-board Open/Short, RLGC, S-parameters, product Solver/Cache, actual GPU driver/reset/VRAM quality, complete M1/PH-08 Gate, or full formal U/T requirements.
