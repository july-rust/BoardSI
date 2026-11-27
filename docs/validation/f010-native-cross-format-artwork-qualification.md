# F_010 — Unified Native Cross-Format Artwork Replay

**Implementation and exact merged-code-main acceptance 100% COMPLETE within the supported subset — confirmed 2026-11-28 KST.** Final documentation acceptance is recorded separately in the [PR #675 final receipt](https://github.com/july-rust/BoardSI/pull/675); until that receipt is verified, overall administrative closure remains pending. Supported F_005–F_009 source subsets only; no full CAD dialect, manufacturing, electrical contact, physical SI, installer or product U/T promotion.

## Delivered scope

Native SQLite v1, BSIG, required/optional extension contracts and F004 Golden fixtures unchanged. The Native Reader, not Qt graphics, owns structural validity. Valid supported models use explicit RENDERABLE or NOT_RENDERED; unsupported presentation never silently publishes a partial picture or prevents structural Open/Save As. Existing source-parser and required Gerber extension validation limits remain in force. Preview bounds: 128 items, 32 copper layers, coordinate magnitude ≤10^12 nm, dimensions ≤10^9 nm, full renderable receipt <60,000 bytes (Qt cap 65,536 bytes). No inference of manufacturing or electrical contact; physical_si=NOT_RUN. Existing format-specific screen orientation preserved.

## Stage-local evidence

| Gate | Observed evidence |
|:--|:--|
| T01 | All five supported Source → Native save paths exercised by actual Worker; prior per-format Qt import tests preserved and passed |
| T02 | All five source files/directories removed, fresh actual Main Qt window opens Worker-seeded Native files; separate Worker invocations. This is not a separate Qt-process restart of each original import session |
| T03 | Existing F005–F009 Canonical/provenance, independent SQLite/BSIG source oracles and Save As regressions passed |
| T04 | Existing independent exact geometry/orientation assertions passed; tested fixture geometries only |
| T05 | Actual Main Qt five-format scene, layer filter, selectable item and source identity checks passed |
| T06 | Existing PADS single-layer and ODB rotation tests plus new KiCad sub-nm trace, IPC reserved display-name and Gerber sub-nm flash tests passed; valid Native opens without Artwork |
| T07 | Existing malformed/required extension/source-change tests passed; corrupt Native Open retains last-good Qt document in F010 |
| T08 | New full receipt size and preview coordinate/degenerate geometry unit tests passed; no partial picture in exercised cases |
| T09 | All five Source-Free Save As and reopen; limited Native Save As; existing read-only and cancellation/timeout regressions passed |
| T10 | Actual Qt five-format switch, corrupt Open retention, valid NOT_RENDERED clears scene, subsequent valid PADS replay passed |
| T11 | Native and actual Qt on Linux x64, Windows x64, Windows ARM64, macOS ARM64; existing F001–F009 regressions passed |
| T12 | Exact final implementation HEAD CI/Codex, protected code merge and exact merged-code-main Push PASS (12/12 workflows, 44/44 jobs); documentation acceptance via PR #675 final receipt |

The fresh Qt test process seeds files using real Worker child processes before constructing its first PreviewWindow, deletes the sources and then performs actual Main Qt Open/Save As/switch; the existing per-format Main Qt import tests independently cover original CAD import and source-free reopen. No independent vendor fixture coverage beyond the existing scope. F007 historical main push evidence remains unobserved: exact c847132dd1078873c370edc75294ac4d8f51169f query returned zero runs on 2026-11-27 KST; no retroactive historical PASS.

## Verified receipts

- [Implementation PR #674](https://github.com/july-rust/BoardSI/pull/674), final HEAD `e6d019b03a5e390c791b96f447d945d7c3258ff7`: **14/14 PR workflows SUCCESS; 55/55 check runs SUCCESS**.
- [Final independent Codex summary](https://github.com/july-rust/BoardSI/pull/674#issuecomment-7218615087) and [exact-HEAD receipt](https://github.com/july-rust/BoardSI/pull/674#issuecomment-7219111148): completed, findings 0, unresolved 0, review threads 0.
- [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/46004314906): actual four OS jobs successful. Linux [CTest evidence](https://github.com/july-rust/BoardSI/actions/runs/46004314906/job/140523594870): qt-f010-cross-format-source-free-native-artwork PASS, 5 formats, source_absent=true, not_rendered_open=true. macOS [native job](https://github.com/july-rust/BoardSI/actions/runs/46004314906/job/140523594910), Windows [x64](https://github.com/july-rust/BoardSI/actions/runs/46004314906/job/140523594917) and [ARM64](https://github.com/july-rust/BoardSI/actions/runs/46004314906/job/140523594938): dedicated F010 step SUCCESS.
- [F004 compatibility](https://github.com/july-rust/BoardSI/actions/runs/46004314881): four OS + Linux ENOSPC success. [Linux independent Golden and F010 negative tests](https://github.com/july-rust/BoardSI/actions/runs/46004314881/job/140523517583). [Numerical core](https://github.com/july-rust/BoardSI/actions/runs/46004315075): Rustfmt, Clippy -D warnings, workspace debug/release PASS, common receipt tests PASS.
- [Protected Squash](https://github.com/july-rust/BoardSI/commit/df0eb7eb75e5fc1dd17a9ba16e1af5b5d59ff5f1): GitHub merged=true, verified=true, one parent fd2b8c6e62ef9f83108c20f27cc1d8bcda7803bd, identical tree to final PR HEAD, branch main protected=true.
- Exact implementation main Push SHA `df0eb7eb75e5fc1dd17a9ba16e1af5b5d59ff5f1`: **12/12 workflows and 44/44 jobs completed SUCCESS**, no skipped, failed, cancelled or pending jobs, confirmed 2026-11-28 KST. Actual four-OS Qt and numerical jobs executed successfully on this exact code SHA; the dedicated F010 Qt test passed on Linux, macOS ARM64, Windows x64 and Windows ARM64. [Native](https://github.com/july-rust/BoardSI/actions/runs/46010106955), [F004](https://github.com/july-rust/BoardSI/actions/runs/46010107178), [Qt](https://github.com/july-rust/BoardSI/actions/runs/46010107416), [Numerical](https://github.com/july-rust/BoardSI/actions/runs/46010107118). The code-main Qt and Native runs above are execution evidence, not documentation-only classification. Do not claim docs-only skipped jobs as real Native/Qt passes.

**Final administrative closure:** PR #675 must pass CI and independent Codex on its exact final HEAD, then protected verified Squash and exact documentation-main Push CI. Record the resulting SHA, workflow/job counts, and verdict in a single final receipt on PR #675, rather than recursively amending the ledger to reference its own future SHA. No further code changes are required by this documentation-only gate. Keep one checklist row and this ledger; do not rewrite historical A–E or product U/T evidence.
