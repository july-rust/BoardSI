# C_009 — Source-bound Reference Stitching Via candidate and Qt risk (validation draft)

**Tracker:** [Issue #261](https://github.com/july-rust/BoardSI/issues/261) · **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN · **Predecessor:** [C_008 #258](https://github.com/july-rust/BoardSI/issues/258).

**Status:** IMPLEMENTED CANDIDATE / NOT YET VERIFIED by final PR-head/native CI/independent review/post-main. Do not treat this draft as acceptance.

## Bounded implementation

- Opt-in `--reference-route-v5` extends unchanged C_008 V4 source route receipt with one explicitly selected **source-via index** (not an automatic nearby Via search).
- Requires exactly two distinct Reference layers with **identical Reference Net** (different Reference Nets unsupported), explicitly plated Through Via with conductive padstack entries on both; source and Canonical identifiers/net/position must agree.
- Reparse original PADS ASCII V9.4 MILS Source; re-use B_006/C_001 source-bound strict COPCLS/COPCUT ViaCentre→AreaInterior SameLayerContact witnesses on both selected Reference layers. Unproven, cutout, disconnected/ambiguous or unsupported contact is fail-closed.
- Include exact source-grid squared Via separation and derived approximate distance in mm (display-only; no physical clearance threshold).
- Keep V1/V2/V3/V4 byte records unchanged; V5 embeds original V4 and its complete V2 receipts for independent Qt revalidation. The Source SHA-256 binds all records to the same successful import.
- Qt CPU/RHI distinct Reference candidate marker, separate from C_008 Signal Via risk and gap spans; reimport, source watcher, selections, changed Reference inputs and late IPC invalidate stale receipts.

## Source fixture and expected result

`crates/boardsi-import/tests/c009_reference_stitching_v5.rs` is synthetic public PADS V9.4 MILS with one Signal Via, one independently declared Reference Through Via, Reference COPCLS Areas on two separate layers and same Source Reference Net. No confidential PCB data is included.

| Test case | Expected |
| --- | --- |
| One valid Reference Via, two strict copper contacts and intact V4 receipt | source-only candidate |
| Signal Via chosen instead of Reference Via; unknown via index | rejected |
| Different Reference Net or Net on one area missing | rejected |
| Reference Via outside Copper or cutout at Via centre | rejected |
| Source changed, old SHA, Qt options/re-import changed | result invalidation; no stale markers |
| All existing B/C/Rust/Qt source-only regressions | no change |

## CI / acceptance gates (unverified until evidence)

- [ ] Final code-head Rustfmt/Clippy, Rust Debug+Release and C_009 negative fixtures PASS.
- [ ] Actual Rust importer→Qt Native Linux and Windows x64/ARM64, macOS ARM64 PASS; RHI/CPU marker coherence.
- [ ] Same **final code HEAD** independent Codex review completed with zero findings and unresolved threads.
- [ ] Ruleset #25046595 required CI all pass, protected Signed Verified Squash.
- [ ] Independent post-main CI Checks and Workflow Runs all SUCCESS and KST checklist evidence.

## Physics and formal signoff remain UNACCEPTED

- `return_path_p009=UNSUPPORTED`
- `solver_ready=false`
- `physical_si=NOT_RUN`
- `cache_reuse_eligible=false`

Neither a candidate Via nor nearby distance proves actual return current transfer or acceptable high-frequency behaviour. No RLGC/S, production Solver, P009 or full U09/U10/U39/T02/T04/T24/T52. M1 #12 remains OPEN.
