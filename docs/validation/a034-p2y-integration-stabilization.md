# A_034 / P2-Y — Import, Connectivity Evidence and GPU Artwork Viewer integration audit

Tracking: [Issue #201](https://github.com/july-rust/BoardSI/issues/201). Completion is **bounded to the already-supported public synthetic PADS V9.4 MILS import subset and native Qt/CPU/RHI regression scope**. This document records accepted code and reviewed test evidence; final post-merge mandatory check acceptance is tracked in Issue #201 and the exact commit check-runs.

## Verified implementation history

| Slice | Exact independently reviewed PR HEAD | Protected signed squash to main | PR acceptance |
| --- | --- | --- | --- |
| [PR #202](https://github.com/july-rust/BoardSI/pull/202): Source-backed cross-board re-import | `b24abfea66e05958afe3dcbf1243671807b5f482` | `1607d177263a0fc60423f9bec3d086be0de70fb4` (GitHub Verified) | **21/21 checks SUCCESS**, including all 17 required; Codex review no findings; post-merge **17/17** required SUCCESS |
| [PR #203](https://github.com/july-rust/BoardSI/pull/203): GPU/CPU source contact and Picking parity | `123ac7f7136b97cbe37ec2b6822c59635344fc04` | `3f24db7c5a8797d7c2518b735b29592921ca8ae8` (GitHub Verified) | **26/26 checks SUCCESS**, including all 17 required; exact-HEAD Codex review no findings; no unresolved review threads |

The PR #203 merge commit's direct parent is `1607d177263a0fc60423f9bec3d086be0de70fb4`. GitHub `main` was verified to point at `3f24db7c5a8797d7c2518b735b29592921ca8ae8`. **Separate post-merge check-run evidence for PR #203 must be read at [this exact main SHA](https://github.com/july-rust/BoardSI/commit/3f24db7c5a8797d7c2518b735b29592921ca8ae8/checks).** Issue #201 may be marked complete only after required checks at that SHA are confirmed successful, without branch-protection bypass.

## A_034 validated behaviors

1. **Source reImport and state reset:** the native production PADS Rust importer / QProcess / Qt UI opens public `pads_route_straight_v94.asc` (3 Trace primitives and exactly one SHA-bound A_032 local-contact marker), switches to `pads_bottom_mirror_v94.asc` (10 mirrored RF pads), and restores the original route. Stale traces, pads, source label, local contact markers, Net-only filter and Layer-hidden state are forbidden. The test preserves the older malformed-header/invalid-path fail-closed recovery checks.
2. **CPU/RHI contact overlay parity:** the original source contact is visible for Net A, hidden when filtering Net B, restored with Net A, hidden and restored by Layer visibility, and preserved exactly once on same-source return. RHI filtered contact counts must match the CPU scene for an existing RHI view.
3. **Picking and provenance:** Qt CPU source marker selection and active Qt Quick mouse click at the original 300/200 mil endpoint report the same bounded local evidence and explicitly state `complete connectivity NOT_VERIFIED`.
4. **Real presented RHI frame:** the native Quick source PADS self-test is repeated after Qt Quick modules are rebuilt on Windows x64, Windows ARM64 and macOS ARM64. A contact mouse pick can PASS only after `frameSwapped` is observed for a fresh Scene Revision and the matching Source Generation, and the viewer reports an RHI frame without CPU fallback. Required receipt fields: `a034_local_contact_cpu_rhi_filter_verified=true`, `a034_local_contact_rhi_frame_scope=PASS`, `a034_local_contact_rhi_pick_scope=PASS`, `a034_cross_file_reimport_verified=true`, and `a034_cross_file_reimport_scope=PASS`.
5. **Visible-canvas Zoom/Fit:** the user-visible Qt Quick RHI zoom factor is used when GPU canvas is active; the hidden CPU GraphicsView transform is not misinterpreted as RHI zoom. Native tests require Zoom In followed by Fit to restore the displayed canvas scale. Pre-existing geometry, rotation, mirror, layer and Net source identity assertions remain unchanged.
6. **Fallback / scale and Copper geometry regressions:** existing A_019 synthetic native 10k-to-1M model benchmark, active spatial picking, pan/zoom/filter and injected error recovery, A_020 explorer, A_021–A_030 source-owned Copper/hole/curve/mirror GPU/CPU parity, A_032 exact local contact transport and A_033 independent source-coordinate Golden fixtures continue as exact-HEAD native CI gates.

## Corrections found during independent review and native testing

- First A_034 cross-board release package check required a second fixture missing from the intentional standalone one-file bundle. Source CTest now requires both source fixtures; standalone package retains the original single-file smoke and reports `NOT_RUN_SINGLE_FIXTURE`, **never a false cross-board PASS**.
- Codex identified a P2 risk where mere active RHI canvas could pass mouse Picking before a frame; fixed by requiring a revision-matched actual frame swap and no fallback. Exact later HEAD was independently reviewed without findings.
- New test's CPU selection disrupted older Qt Net/Properties assertions; the CPU contact assertions now run after the legacy checks and Net A is explicitly restored for deliberate reImport state.
- Active RHI Zoom/Fit had incorrectly been judged through hidden CPU canvas transform; test oracle was corrected to the visible renderer. Windows x64, Windows ARM64 and macOS ARM64 native Qt jobs passed the final reviewed PR #203 HEAD after the fixes.

## Test scope and qualifications

| Classification | Evidence or boundary |
| --- | --- |
| **PASS** | Public synthetic PADS importer to Canonical/IPC/Qt integration, cross-board lifecycle, local source contact CPU/RHI filters, actual native Qt Quick frame-gated picking, existing supported Copper and native interaction regressions |
| **PARTIAL** | Native Direct3D/Metal frame-and-picking benchmarks on CI-host GPUs are coverage for those CI environments only, not all customer graphics adapters and drivers |
| **NOT_VERIFIED** | General commercial/vendor PADS Export compatibility, full PCB Open/Short/Net-to-Pin connectivity, Cu pour/thermal/pad junctions and fabricated connectivity, licensing/legal clearance |
| **NOT_RUN / NOT_MEASURED** | Physical SI/RLGC/S-parameter solver acceptance; independent customer's GPU memory/driver reset; guaranteed 1M objects at 60 FPS on target PCs |

Layer and source-declared Net labels are *not* proof of electrical connection. A_032 contact markers represent **only verified local endpoints**; cross-Net conflict remains **POTENTIAL_CONFLICT**, never a certified short. Canonical source coordinates must not be inferred from GPU display meshes. The default V1–V14 IPC and opt-in source-SHA suffix remain separate.

Tests use public offline synthetic fixtures only. No confidential vendor or customer PCB files were supplied.

**A_034 acceptance is separate from P2 completion**: A_035 P2-Z must decide final P2 scope, release acceptance and explicit unverified items; do **not** automatically close umbrella Issues #9, #11 or #165.
