# A_034 / P2-Y — Qt import lifecycle and Artwork Viewer stabilization

**Status: IN PROGRESS — implementation candidate; native CI, independent review and protected merge pending.**

Tracking: [Issue #201](https://github.com/july-rust/BoardSI/issues/201).
Baseline: protected `main@957b1ac8484158bb1b42872ad95359662abce2e2`, after verified A_033 PR #199/#200.

## First executable slice: two different source files through production Qt

Existing P2-A Preview smoke already exercises a complete 3-track source-import, exact Qt source-to-scene geometry, net/layer filter, local A_032 contact decoding, invalid/missing replacement and eventual successful recovery. A_034 adds the previously missing **successful different-board replacement** stress case between the first accepted board and invalid-header recovery:

1. Open `pads_route_straight_v94.asc` with the real native Rust importer and optional source-bound contact records. Verify 3 imported trace objects and local contact marker.
2. Select source Net `NET_A`, enable selected-Net-only, and hide all imported layers (deliberately dirty UI state).
3. Open `pads_bottom_mirror_v94.asc` using the same actual `PreviewWindow::openCandidate` → `PreviewImportPanel::openPath` → asynchronous `QProcess` → Canonical IPC → CPU/RHI UI path.
4. Confirm **0 stale traces/arcs, 10 accepted RF pads, 0 stale local contact markers**, no old Net selection or Net-only state, restored layer visibility (10 visible pads), and the new source filename in user-visible properties.
5. Open original route source again, confirm 3 tracks, 0 stale pads, source-bound local contact evidence restored, and original filename restored.
6. Repeat existing independent Qt geometry/net/zoom/fit checks and invalid/header failure recovery.

### Native package scope distinction (A_034 CI corrective review)

The first A_034 candidate failed Windows x64 **bundled standalone** smoke, even though the native installed-Qt test step passed. The copied standalone test has only one fixture named `public-pads-v94-synthetic.asc`, so its original one-file smoke must not require the second repository fixture. The production-source native CTest uses `pads_route_straight_v94.asc` and **must** run the cross-file gate. The smoke JSON now reports `a034_cross_file_reimport_scope=PASS|FAIL` for the source CTest and `NOT_RUN_SINGLE_FIXTURE` for the staged package; the latter is never reported as a cross-file PASS. This preserves and distinguishes both acceptance scopes without reducing either environment's existing validation. At the time of this update the corrective PR HEAD CI has not been certified.

All source fixtures are public synthetic samples in `crates/boardsi-import/tests/fixtures`, without private/exporter CAD data. The added assertion is reflected in `qt-preview-p0-results.json` as `a034_cross_file_reimport_verified` and only reports true when the entire native P2-A smoke succeeds.

## Expected verification and controls

- Native `qt-preview-p2a-partial-import` CTest step on Qt Linux offscreen, Windows x64/ARM64 and macOS ARM64 jobs; existing workflows continue to use production binaries and fixtures.
- Required protected GitHub 17 contexts and Rust unchanged regression.
- Independent exact-HEAD Codex review of the final PR commit, with any findings tracked/resolved.
- Signed protected squash merge and separate post-merge required CI.
- A_034 Issue #201 remains open until acceptance evidence is recorded.

## Scope limits / outstanding

No importer grammar, Canonical source fields, V1–V14 transport, Qt GPU shaders/algorithms or physical SI algorithms are changed by this first test slice. This is not a hardware GPU/VRAM or production vendor board certificate.

| Qualification | Current evidence |
| --- | --- |
| Synthetic cross-file Qt reImport | **IMPLEMENTED CANDIDATE / CI PENDING** |
| Qt RHI vs CPU behavior under existing CI | **PARTIAL**; previous synthetic fixtures, new test pending |
| Real PADS export (licensed, redistributable) | **NOT_VERIFIED** |
| Full board open/short or pad/copper/thermal connectivity | **NOT_VERIFIED** |
| Physical RLGC / S-parameters | **NOT_RUN** |
| GPU VRAM / device reset / million objects 60 FPS | **NOT_RUN** |
| Binary distribution licensing | **NOT_VERIFIED** |

The P2 umbrella issues #9, #11 and #165 are not closed by this slice.
