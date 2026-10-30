# C_016 — Source-Exact Reference Stitch Via Pad / Mixed Cutout Clearance V12

**Tracker:** [Issue #284](https://github.com/july-rust/BoardSI/issues/284) · **Implementation:** [PR #285](https://github.com/july-rust/BoardSI/pull/285) · **Parent:** M1 [#12](https://github.com/july-rust/BoardSI/issues/12) OPEN.

**Stage acceptance:** PENDING. The proposed V12 implementation is not a physical electrical or manufacturing verification. It must not be checked complete without exact-final HEAD CI, completed independent Codex no-findings review, signed protected squash and independent post-main results.

## Supported source-only input

- PADS ASCII V9.4 MILS explicitly bounded subset. The actual original Source SHA-256, strict Source Area ID, Reference Net, two distinct Reference layers, selected Signal/Reference Stitch Through Vias and supported circular layer-specific conductive padstack are required. Re-imported original bytes must match.
- Reference Copper owner rectangle with 0–8 mutually disjoint, strictly enclosed same-owner explicit rectangular COPCUT and/or circular COPCCO. Unknown/unassociated standalone voids, arcs, non-rectangular outer and ambiguous Reference Area counts are rejected.
- Integer Source Tick (0.01 mil = 254 nm) and analytically exact disk-vs-outer-edge, disk-vs-rect, disk-vs-circle separation using squared distances (no display tessellation in decisions). States: STITCH_PAD_FULL_COPPER_FOOTPRINT_CANDIDATE, STITCH_PAD_CUTOUT_TANGENCY_RISK, STITCH_PAD_CUTOUT_INTERSECTION_RISK, STITCH_PAD_OUTER_TANGENCY_RISK, STITCH_PAD_OUTER_CROSSING_RISK. Centre lies strictly in Copper per original V11; full Pad may still reach cutout/outer boundary.
- New opt-in Rust --reference-route-v12 includes the **original byte-identical V11** returned from original bytes. Qt independently validates SHA, selected layers/nets/Source IDs, Source geometry, stitch footprint and nested V11→V8 contact candidate evidence. Qt CPU and RHI draw actual Source-sized Pad rim, and reimport, selection, source edit, options and late-IPC changes invalidate stale receipts.
- V1–V11 pre-existing commands/wires preserved. Unsupported requests fail closed; V12 does not create or modify Copper, Antipad, Thermal, DRC rule or manufactured clearance threshold.

## Test inventory / acceptance evidence

| Gate | Evidence to record | Status |
|---|---|---|
| Rust exact outer/cutout tangency/intersection, valid 8 and invalid 9 holes | crates/boardsi-import/src/reference_route_v12/geometry.rs | PENDING exact-HEAD CI |
| Original importer V12 positive rectangle/circle and tamper negative; V11 verbatim | crates/boardsi-import/tests/c016_reference_route_v12.rs | PENDING exact-HEAD CI |
| Qt Native genuine importer V12, actual Pad CPU original-size rim, invalidation | --c016-stitch-pad-selftest, qt-runtime.yml | PENDING native matrix |
| Exact HEAD, fmt/Clippy Debug/Release, Native OS and all required contexts | [PR #285 checks](https://github.com/july-rust/BoardSI/pull/285/checks) | PENDING |
| Codex independent exact HEAD and threads resolved | [PR #285](https://github.com/july-rust/BoardSI/pull/285) | PENDING |
| Strict Ruleset #25046595 Signed Verified protected Squash | SHA after merge | PENDING |
| Independent post-main Checks and Push Workflows | Exact main SHA | PENDING |
| Checklist first checked KST and documentation closure | docs/development-checklist.md | PENDING |

## Explicit nonclaims

- return_path_p009 = UNSUPPORTED
- physical_si = NOT_RUN
- solver_ready = false
- cache_reuse_eligible = false

No actual Reference Return Current topology, electrical connectivity or physical short/open DRC, fabricated Antipad/Thermal geometry, SI, RLGC, impedance or S-parameters. Formal U09/U10/U39/T02/T04/T24/T52 and M1 Issue #12 stay unchecked/open.
