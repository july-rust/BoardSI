# E_019 — ODB++ Manufacturing Drill → Source-only Via Candidate Qualification

**Status:** COMPLETE — bounded ODB++ SOURCE-only via-candidate code acceptance verified at [PR #615](https://github.com/july-rust/BoardSI/pull/615); final documentation publication is subject to docs PR #616 protected CI/merge/exact docs-main Push. **Scope:** only round `TYPE=PLATED`, `TYPE2=STANDARD` drill hits with explicit source span and finished diameter. **Evidence date:** 2026-11-13 KST.

## Implementation boundary

- `crates/boardsi-import/src/odb_fabrication.rs` adds `OdbPlatedViaCandidate` as an opt-in **sidecar** on `OdbSourceImport`; existing `BoardModel` is not mutated.
- A candidate preserves `Rational` XY, `FINISH_SIZE` (separate from optional `DRILL_SIZE`), physical Copper ordinal span, source file path+SHA-256, tool file path+SHA-256, matrix SHA, original feature ordinal/tool number/source line, and a complete `SourceIdentity` binding.
- `crates/boardsi-model/src/source_layer_via.rs` adds `observe_source_via_holes_by_path`: one full board validation and physical layer ordering per batch; exact logical source path + SHA avoids ambiguity when distinct files have identical bytes. The old SHA-only observation API remains fail-closed on collisions.
- Source-only `SourceDeclaredPlated` **never** proves manufactured Barrel continuity; `BoardModel::vias`, `drills`, `connectivity.edges`, `Via.connected_layers`, IPC solver inputs and SI are never promoted. Tool `TYPE=VIA`, `NON_PLATED`, `PRESS_FIT`, unknown `FINISH_SIZE=-1` stay source observations but do not produce plated candidates.
- The already bounded Directory/USTAR/ZIP/GZIP FileMap handling, Source drill parsing and E_004 span validation are reused; no shell decompression or importer-wide overhaul.

## Scoped regressions

The existing `crates/boardsi-import/tests/d056_08_odb_fabrication.rs` now includes E_019 regression tests for:

1. 2/4/6/8/64 Copper layers; complete and L2–L5 partial span; exact rational XY/finished/tool diameter; physical envelope != connected-layer set.
2. Deterministic candidate key + complete SourceIdentity, mutation changes provenance; no electrical edge, no manufactured barrel or SI approval.
3. `NON_PLATED`, `VIA`, `PRESS_FIT`, missing `FINISH_SIZE` never promoted.
4. Byte-identical content under different logical paths; qualified path resolves uniquely, SHA-only observer remains conservative.
5. 256 extra plated drill hits in one observation batch; stable ordering and no board connectivity mutation.
6. Existing D_056_08 negative tests (span, backdrill, bad tool, missing diameter) stay in effect.

**Verified test evidence (2026-11-13 KST):** final HEAD `a6fbf0c03714d2f0f6d838f865b2951089ee82e5`, macOS ARM64 native Rust rerun [Numerical workflow #43375256402](https://github.com/july-rust/BoardSI/actions/runs/43375256402), Debug **5/5** and Release **5/5** E_019 focused regressions PASS, previous D006 failure preserved in run attempt 1 and rerun SUCCESS in attempt 2. Linux numerical, Windows x64/ARM64 Native Rust PASS for same HEAD. Final PR HEAD **38/38 Checks SUCCESS** and 12/12 PR Workflow SUCCESS; macOS Qt SDK download corruption failed an earlier attempt and passed native Qt on retry. Failed prior attempts remain visible; do not count them as successes.

## Independent code review

Initial Codex P2 suggestions on PR #615: (1) path-qualified provenance with SHA duplicates, (2) repeated full BoardModel validation across holes. Both fixed with separate regression tests; GitHub threads resolved 2/2, unresolved 0. Final HEAD `a6fbf0c037` independent [Codex review receipt in PR #615](https://github.com/july-rust/BoardSI/pull/615) reports no major issues, plus bot +1 reaction (2026-11-13 KST).

## Acceptance and non-acceptance

- **E_019 bounded SOURCE software candidate:** COMPLETE only after final HEAD required CI, independent review, zero unresolved threads, protected GitHub Verified one-parent Squash and independent exact merged-code main `push` workflows.
- **NOT_ACCEPTED:** physical plating/manufactured barrel; contact to Copper Pads; CAM tool/subtype comprehensive support (including PRESS_FIT, microvia, laser, slot, backdrill); full EDA CMP/TOP; actual product Qt ODB++ Import (E_021), stackup material/z (E_020), verified electrical graph (E_022), vendor/cross-CAD oracle (E_023), S-parameters, product U/T.
- Parent [#10](https://github.com/july-rust/BoardSI/issues/10), [#4](https://github.com/july-rust/BoardSI/issues/4), [#520](https://github.com/july-rust/BoardSI/issues/520) remain separately tracked. Historical [#578](https://github.com/july-rust/BoardSI/issues/578) remains administratively closed, not treated as full ODB++ acceptance.
- The only source-of-truth completion checklist is [docs/development-checklist.md](../development-checklist.md); no second checklist is created.

## Final GitHub evidence (fill only after verified)

| Gate | Status / exact evidence |
| --- | --- |
| Code PR final HEAD SHA | [`a6fbf0c03714d2f0f6d838f865b2951089ee82e5`](https://github.com/july-rust/BoardSI/pull/615/commits/a6fbf0c03714d2f0f6d838f865b2951089ee82e5), merged as #615 |
| Code PR final required Checks | `a6fbf0c037` **38/38 SUCCESS**, 12/12 GitHub PR Workflows SUCCESS (native macOS Qt SDK download failure recovered on rerun, old failed attempts retained) |
| Final HEAD independent review | Codex: no major issues at `a6fbf0c037`, 0 unresolved threads; [PR #615](https://github.com/july-rust/BoardSI/pull/615) |
| Unresolved reviewer threads | 0 (initial two P2 threads resolved) |
| Verified one-parent Squash SHA | [`9efeeb6bb004e737aeaba7bd8a8af89866385254`](https://github.com/july-rust/BoardSI/commit/9efeeb6bb004e737aeaba7bd8a8af89866385254), GitHub verification `valid`, 1 parent `6ef94a10ffdb650b91e18876b84a941f048889be` |
| Exact merged code-main SHA event=push 8/8 workflows | **8/8 SUCCESS** at `9efeeb6bb004e737aeaba7bd8a8af89866385254` — all event=`push`, no failed/unfinished workflow (links below) |
| Docs-only final PR CI/review/Verified Squash/Push | [PR #616](https://github.com/july-rust/BoardSI/pull/616) — subject to final-head documentation gate, verified squash, and independently confirmed exact docs-main Push; publish final documentation SHA in PR/issue closure evidence |

## Independently verified code-main Push — exact merged SHA

The following **eight** GitHub Actions workflows are independently checked with `event=push`, `head_sha=9efeeb6bb004e737aeaba7bd8a8af89866385254` and `conclusion=success` on 2026-11-13 KST. These are **post-merge runs**, not reused PR checks.

| Workflow | Exact Push Run |
| --- | --- |
| Qt runtime | [#43378402580](https://github.com/july-rust/BoardSI/actions/runs/43378402580) |
| Numerical core | [#43378403040](https://github.com/july-rust/BoardSI/actions/runs/43378403040) |
| Validate planning documents | [#43378402592](https://github.com/july-rust/BoardSI/actions/runs/43378402592) |
| M2 PH-06 independent FVM | [#43378402572](https://github.com/july-rust/BoardSI/actions/runs/43378402572) |
| M2 PH-07 independent FVM | [#43378402615](https://github.com/july-rust/BoardSI/actions/runs/43378402615) |
| M2 native FEM regression | [#43378402651](https://github.com/july-rust/BoardSI/actions/runs/43378402651) |
| M2 isolated lossless FEM prototype | [#43378402666](https://github.com/july-rust/BoardSI/actions/runs/43378402666) |
| M2 synthetic G0 reference | [#43378402589](https://github.com/july-rust/BoardSI/actions/runs/43378402589) |

**Scope is strictly SOURCE-only.** Electrical via connectivity, physical barrel proof, E_020–E_024, vendor oracle and product U/T remain separately unaccepted.
