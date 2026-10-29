# C_010 — Source Reference Corridor to Stitching Via: limited Source Area identity acceptance

**Tracker:** [C_010 Issue #264](https://github.com/july-rust/BoardSI/issues/264) · **Implementation PR:** [#265](https://github.com/july-rust/BoardSI/pull/265) · **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) **OPEN** · **Previous phase:** [C_009](c009-reference-stitching-via-candidate.md).  
**Evidence / first-check date:** **2026-10-29 KST**. **Status:** **LIMITED SOURCE-GEOMETRY CODE ACCEPTED**, independently verified on signed protected main; documentation PR and its independent post-doc-main validation are **separate outstanding gates** until completed.

## Accepted engineering scope (not electrical Return Path)

- Additive opt-in `--reference-route-v6` on public synthetic PADS V9.4 MILS with 2–16 explicit Straight Signal traces in source order, one signal plated Via transition between two layers, two selected Reference layers, same explicitly selected Reference Net, and one explicitly selected plated Reference Stitch Via candidate.
- **Each selected signal-leg centreline** must be strictly interior to exactly one source-owned simple COPCLS/COPCUT Reference Copper Area on its configured Reference layer. Exact supported integer/rational Source predicates exclude boundary/cutout intersection or ambiguous ownership. Adjacent Signal–Reference layer order is explicitly required.
- Reparse original Source bytes, preserve Source SHA-256 and provenance; compare the corridor Source Area ID on each layer with the **corresponding C_009 V5 ViaCentre–AreaInterior verified contact Area ID**. Source area ownership is geometry-only and **does not assert Canonical electrical connection**. Identical Net label alone does not join detached Copper Islands.
- Reject Source change/stale generation, unsupported route/geometry, ambiguous overlapping source ownership, wrong Via/Net/Layer, corridor cutout/boundary/gap, same-net separate island Area mismatch or mismatched V5 contact.
- Embed **byte-exact original V5, V4, V2 receipts** in bounded V6 output. No changes to existing V1–V5 bytes or opt-in behavior.
- Qt CPU/RHI display Corridor/Via source-risk information and Source Area identities, and invalidate previous results on Source watcher/reimport, user selection, Reference inputs, generation and late worker messages. No positive physical SI label.

## Public synthetic evidence

- `crates/boardsi-import/tests/c010_reference_corridor_v6.rs`: supported same-source-area candidate, detached same-Net Copper Island mismatch, Cutout, boundary touch, changed Source SHA, invalid Reference Via / Net and previously accepted B/C contracts.
- `apps/boardsi-qt-prototype/preview_p0.cpp` and `c005_reference_risk.h`: opt-in Rust importer-to-Qt V6 receipt handling, CPU/RHI risk markers and stale-result tests.
- `.github/workflows/qt-runtime.yml`: explicit `--c010-corridor-selftest` on native Windows x64, Windows ARM64, macOS ARM64; Linux offscreen CTest `qt-preview-c010-reference-corridor-stitch` in `CMakeLists.txt`. Tested code is a source-only synthetic Preview, not production hardware/GPU-device signoff.

## Independently verified code acceptance — 2026-10-29 KST

- [x] [PR #265 **exact final code HEAD** `4e050874fbf621b2e8f28d50c3ab06ff93d77b05`](https://github.com/july-rust/BoardSI/commit/4e050874fbf621b2e8f28d50c3ab06ff93d77b05/checks): **27/27 Check Runs SUCCESS; 12/12 PR Workflow Runs SUCCESS**, all completed on the exact SHA; fmt/Clippy, Rust Debug/Release, prior B/C regressions, Native Qt Windows x64/ARM64, macOS ARM64, Linux offscreen. [Native Qt PR run #40822519592](https://github.com/july-rust/BoardSI/actions/runs/40822519592).
- [x] [Codex independent **same final HEAD** summary](https://github.com/july-rust/BoardSI/pull/265#issuecomment-6488503225): `code_review.commit_sha=4e050874fbf621b2e8f28d50c3ab06ff93d77b05`, `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`. Historical P1 Source Net ownership false Gap and P2 native test omission corrected; review threads **2/2 resolved**.
- [x] [PR #265 protected Squash merged](https://github.com/july-rust/BoardSI/pull/265) to [`main@565e63cb2f3cc4deaca7f09543340747f6024bb0`](https://github.com/july-rust/BoardSI/commit/565e63cb2f3cc4deaca7f09543340747f6024bb0), **one parent** and GitHub REST `commit.verification.verified=true`, `reason=valid` (signed by GitHub); active Ruleset [#25046595](https://github.com/july-rust/BoardSI/rules/25046595) has 17 required checks, strict, Squash-only, signatures required, zero bypass actors.
- [x] **Independent code post-main** [`565e63c...` **21/21 Check Runs SUCCESS**](https://github.com/july-rust/BoardSI/commit/565e63cb2f3cc4deaca7f09543340747f6024bb0/checks); **9/9 associated Workflow Runs SUCCESS** (8 push workflows + `Push on main` dynamic workflow). [Qt Native push #40823686048](https://github.com/july-rust/BoardSI/actions/runs/40823686048), [Numerical Core push #40823686042](https://github.com/july-rust/BoardSI/actions/runs/40823686042), [main push gate #40823685703](https://github.com/july-rust/BoardSI/actions/runs/40823685703). None pending or failed in snapshot.
- [x] Limited Source Geometry + Qt risk candidate only; **first checked 2026-10-29 KST; evidence date 2026-10-29 KST**. Existing checklist entries retain their original first-check dates.

## Separate documentation-only closure gate

- [ ] The PR modifying this acceptance ledger and `docs/development-checklist.md` must itself pass **exact-final-doc-HEAD** required checks, have a **completed Codex review on that same HEAD with zero findings/unresolved threads**, protected **Signed Verified Squash**, and **independent post-doc-main Checks / Workflow Runs**. Mark only after actual verification; code PR evidence does not substitute for this gate.
- [ ] Close only C_010 Issue #264 after these document gates. Parent M1 Issue #12 remains OPEN.

## Excluded / fail-closed claims

**Not accepted:** physical Return Current/P009, stitching-Via SI quality, complete manufactured copper fill, thermal spokes/antipad discontinuity, arbitrary commercial PADS, whole-board Open/Short, RLGC or S-parameters, production Solver and Cache, actual GPU driver/VRAM performance, formal U09/U10/U39/T02/T04/T24/T52, PH-08 Gate Q and M1 #12 full completion.

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```
