# C_009 — Source-bound Reference Stitching Via candidate and Qt risk (limited source-geometry acceptance)

**Tracker:** [Issue #261](https://github.com/july-rust/BoardSI/issues/261) · **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN · **Predecessor:** [C_008 #258](https://github.com/july-rust/BoardSI/issues/258).

**Status (2026-10-29 KST):** **LIMITED SOURCE-GEOMETRY/Qt DISPLAY ACCEPTED — CODE PR AND INDEPENDENT CODE POST-MAIN VERIFIED**. Documentation-only acceptance PR is a separate validation gate below; physical Return Path / P009 / Solver remains **NOT_ACCEPTED**.

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

## Verified code implementation evidence (2026-10-29 KST)

- [x] [Code PR #262](https://github.com/july-rust/BoardSI/pull/262) final exact HEAD [`bdd29c061086b5d68742b3915dddd9e2bcf83033`](https://github.com/july-rust/BoardSI/commit/bdd29c061086b5d68742b3915dddd9e2bcf83033/checks): **27/27 GitHub check runs SUCCESS**, **11/11 pull-request workflows SUCCESS**. Rust 1.75 fmt/Clippy (warnings denied), Rust Debug/Release C_009 positive/negative tests and existing B/C regression; Native Qt Linux provisional and Windows x64/ARM64, macOS ARM64 importer subprocess+stale regression.
- [x] Same **exact final code HEAD** GitHub independent Codex code review **completed**, `has_any_findings=false`, `has_unresolved_findings=false`. Three prior findings (candidate argument, shortest valid V5 envelope, initially misplaced positive fixture) **corrected**; 3/3 review threads **resolved**.
- [x] [Protected Squash merge PR #262](https://github.com/july-rust/BoardSI/pull/262), `main@a246bc9f15a3b27cf33cdd65370308ee6051fece` ([commit](https://github.com/july-rust/BoardSI/commit/a246bc9f15a3b27cf33cdd65370308ee6051fece)): REST signature `verification.verified=true`, `reason=valid`; one-parent Squash, protected `main`, no bypass claimed or required.
- [x] **Independent code post-main** [`main@a246bc9...` Checks](https://github.com/july-rust/BoardSI/commit/a246bc9f15a3b27cf33cdd65370308ee6051fece/checks) **21/21 SUCCESS**; **8/8 push-triggered Workflow Runs SUCCESS** on 2026-10-29 KST. [Qt Native push #40814575323](https://github.com/july-rust/BoardSI/actions/runs/40814575323) · [Numerical Core push #40814574904](https://github.com/july-rust/BoardSI/actions/runs/40814574904) · [FEM Native push #40814575032](https://github.com/july-rust/BoardSI/actions/runs/40814575032). None pending or failed at the verification snapshot.
- [x] Acceptance is **only** for public synthetic PADS V9.4 MILS supported Source-Copper-contact evidence, V5 source identity and display markers. Synthetic successful fixture has same-Reference-Net Copper interior on two layers, signal Via at (150,250) mil and explicitly selected Reference Via at (180,280) mil, display-only planar separation ~1.077631 mm. The exact squared separation is checked in source ticks, not interpreted as electrical Return Path quality.
- [x] Existing C_005–C_008 contracts remain unmodified; C_009 implementation PR changes only the additive V5 opt-in path. Post-doc-main verification, if any, must be independently checked for the documentation-only PR and must not be reported as completed beforehand.

## Documentation-only evidence gate

- [ ] This evidence/checklist update requires its **own final exact-HEAD CI**, independent same-HEAD Codex, protected Signed Verified Squash, and independent **post-document-main** Checks/Workflows. Do not conflate this gate with the already completed code PR #262.

## CI / code acceptance gates (verified independently)

- [x] Final code-head Rustfmt/Clippy, Rust Debug+Release and C_009 positive/negative fixtures PASS (27/27 code HEAD checks).
- [x] Real Rust importer→Qt native Linux and Windows x64/ARM64, macOS ARM64 subprocess/stale regression PASS; Qt CPU and RHI source-only rendering paths are present and compiled. GPU device measurement / physical EM results NOT claimed.
- [x] Same **final code HEAD** independent Codex review completed with zero findings and unresolved threads.
- [x] Ruleset #25046595 protected required CI and signed Verified Squash code PR #262 confirmed.
- [x] Independent **code post-main** 21/21 Checks and 8/8 Push Workflow Runs SUCCESS, evidence date 2026-10-29 KST; checklist dated update is in the separately validated documentation PR.

## Physics and formal signoff remain UNACCEPTED

- `return_path_p009=UNSUPPORTED`
- `solver_ready=false`
- `physical_si=NOT_RUN`
- `cache_reuse_eligible=false`

Neither a candidate Via nor nearby distance proves actual return current transfer or acceptable high-frequency behaviour. No RLGC/S, production Solver, P009 or full U09/U10/U39/T02/T04/T24/T52. M1 #12 remains OPEN.
