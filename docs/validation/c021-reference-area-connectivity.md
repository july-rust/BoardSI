# C_021 — Two source Reference Copper rectangles: bounded connectivity-risk audit

**Tracker:** [Issue #300](https://github.com/july-rust/BoardSI/issues/300) · Parent [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN · baseline `main@076f7e739db6ef1517882b3ab4c377c72d4d0922`.

**Status: IMPLEMENTATION CANDIDATE — CI / independent exact-HEAD Codex / protected Signed Verified Squash / independent post-main NOT VERIFIED.** Do not claim PASS before actual receipts.

## Implemented candidate, strictly source-only

- Independent opt-in `reference_area_v13::encode` and CLI `--reference-area-v13 RA RB NET_REF 2` (two names, Net label, Source Reference Layer; CLI route coverage not requested). Rust API optionally checks one straight trace route.
- Reparse original bytes, preserve SHA256, Canonical Source Area IDs and explicit Layer/Net label. Exactly two source-owned hole-free COPCLS rectangles; both must have positive dimensions and authenticated Canonical polygon rings. No graph edges, no retroactive C_001 or V1–V12 changes.
- Distinguish positive 2D rectangle intersection from one-tick gaps, zero-area edge/corner touches and disconnected islands. Physical continuity and manufactured fill remain **not proved** even for overlapping source rectangles.
- A straight, adjacent-layer, axis-aligned signal centreline is a *separate* opt-in candidate: both rectangles must contribute strictly positive overlapping open-line intervals covering the entire finite trace, not just share a 2D overlap.
- Unsupported other shapes, holes, more than two Areas, different Net/Layer, stale Source, invalid trace or ambiguous ownership fail closed. Neither a shared Net name nor overlapping source geometry upgrades Canonical electrical Net or Return Current.
- Test file: `crates/boardsi-import/tests/c021_reference_area_v13.rs`, public synthetic PADS V9.4 MILS fixtures only.

## Required acceptance gates (update only from actual GitHub evidence)

- [ ] Rustfmt, Clippy, Debug/Release C_021 tests and all existing B/C numerical regressions.
- [ ] Qt Native Windows x64/ARM64, macOS ARM64 and Linux offscreen CI; C_021 *dedicated* Qt risk UI test if implemented, otherwise clearly mark UI integration **NOT IMPLEMENTED**.
- [ ] Independent completed Codex review of **exact final source HEAD** with zero findings/unresolved threads.
- [ ] Protected Ruleset-compliant Signed Verified Squash merge, no bypass.
- [ ] Independently checked post-code-main Checks and workflows.
- [ ] Dated checklist completion (first check date and evidence date KST), and documentation-PR independent gate if split.

**Nonclaims:** physical `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`. No fabricated Copper Fill/DRC, Return Current, RLGC/S or Solver/Cache. Formal U09/U10/U39 and T02/T04/T24/T52 unchecked. [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. This is not a replacement for the original B_007 final-head Codex missing receipt.
