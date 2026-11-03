# D_022 — Collinear Source Trace endpoint contact qualification

- Tracker: [Issue #406](https://github.com/july-rust/BoardSI/issues/406)
- Initial recorded date: 2026-11-03 KST; evidence dates and acceptance: **PENDING**
- Base protected main: `c0a41cfbd968ac16bbdef95878b5501be2cf61ed`
- Opt-in: `--d022-collinear-endpoint-contact-v1 NET_A 1 4 0 1`
- Synthetic fixture: `crates/boardsi-import/tests/fixtures/d022_collinear_endpoint_v94.asc`

## Frozen limited contract

PADS V9.4 MILS Source tick = 0.01 mil = 254 nm. Main Layer-4 trace (15000,25000) → (35000,25000), branch (35000,25000) → (45000,25000), same NET_A. Shared endpoint (35000,25000), exactly zero centreline overlap length. The main endpoint is also pre-existing Through-Via B's Layer-4 pad; **all three** Source primitives are represented. Candidate graph: 8 nodes / 6 layer-local candidate edges / 3 components; one legitimate three-node Source contact triangle, zero proven electrical cross-layer edges. No manufactured copper or plated Via barrel conduction inferred.

The current bounded topology intentionally supports the main Trace's Via-B endpoint only. Opposite Via-A endpoint would require a different third candidate edge and is fail-closed, not silently accepted. Strict i128 integer collinearity and exterior projection, native source complete inventory and SHA binding. Reverse branch orientation accepted, 1-tick gap/offset, noncollinear bend, zero length, partial/contained overlap, different Net/Layer, unsupported artwork and forged/stale receipts rejected.

## Acceptance ledger — not yet evidenced

- [ ] Rust format, Clippy and hostile regression at exact final HEAD.
- [ ] Real Qt Import/Reimport, Source SHA/complete Receipt, forgery and stale rejection on Linux.
- [ ] Windows x64/ARM64 and macOS ARM64 independent native Qt tests.
- [ ] 17 required Strict GitHub checks and all PR Workflows at exact final HEAD.
- [ ] Independent same-HEAD Codex review complete, no unresolved finding or review thread.
- [ ] Protected Signed Verified single-parent Squash and independent exact post-main push checks.
- [ ] Issue #406 CLOSED only after all gates with evidence.

**Explicit exclusions:** manufactured copper, electrical barrel proof, whole-board Open/Short, P009, PH-08, scenario/cache, RLGC/S-parameters, formal U/T or release acceptance. Parent #12 OPEN and Draft #35 unchanged. Historical D_021 Windows ARM64 D006 failed-job retry is not interpreted as root-cause correction.
