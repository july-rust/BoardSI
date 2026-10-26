# A_034 / P2-Y — PADS Import, local connectivity evidence and Artwork Viewer stabilization

Tracking: [Issue #201](https://github.com/july-rust/BoardSI/issues/201).
Scope: the already-supported **synthetic PADS V9.4 MILS partial Artwork subset only**. This is **not** a vendor CAD importer, complete electrical topology audit, physical SI or real-hardware GPU benchmark.

## Accepted slice — cross-file native Qt import lifecycle

- [PR #202](https://github.com/july-rust/BoardSI/pull/202), reviewed PR HEAD `b24abfea66e05958afe3dcbf1243671807b5f482`, protected signed squash to `main@1607d177263a0fc60423f9bec3d086be0de70fb4` with GitHub Verified signature.
- 21/21 PR checks SUCCESS, including **17/17 protected mandatory**, same-HEAD independent Codex review completed with no findings and 0 unresolved threads. **Separate merge-SHA main-push 17/17 mandatory SUCCESS**.
- Production Qt/Rust import smoke opens source-supported `pads_route_straight_v94.asc` (3 traces, exactly one A_032 local contact), `pads_bottom_mirror_v94.asc` (10 RF pads), and original route again. Previously selected Net and hidden Layers, obsolete pads, traces, markers and source filename must not survive replacement. Invalid-header and missing-file fail-closed/recovery tests are retained.
- The Windows Qt-bundled **standalone** smoke stages only `public-pads-v94-synthetic.asc`. It **cannot** check cross-file reimport without the second repository fixture and explicitly declares `NOT_RUN_SINGLE_FIXTURE`; it still runs all preexisting one-file packaging checks. The separate source-native CTest strictly requires the two available synthetic files, and its `a034_cross_file_reimport_verified` gate passed.

## Final bounded viewer parity audit — candidate (not yet accepted)

The second A_034 slice extends the **actual native production-source** P2-A imported Qt smoke rather than inventing a new CAD parser or a mock electrical netlist:

1. Demand exactly one source SHA-bound endpoint contact from the public original route. Validate overlay did **not** change its 3 primitive tracks or underlying original Golden geometry.
2. On the same real loaded source, Net A local marker is visible, Net B + Net-only hides it, Net A restores it; hidden Top layer hides it and restoring Top shows it, clearing the filter leaves exactly one marker.
3. Compare CPU scene-visible marker count against the equivalent filtered RHI-model contact count when that renderer exists; CPU-only builds do **not** claim RHI execution.
4. Select the actual Qt CPU source-backed marker and require Properties to identify a bounded local endpoint, with complete connectivity explicitly `NOT_VERIFIED`.
5. If native RHI canvas is **active**, drive the real QQuickItem mouse press/release at the exact 300/200 mil source contact point and require the same bounded source-backed Properties. Record `NOT_RUN_NO_ACTIVE_RHI` when not available rather than a fabricated PASS.
6. On different-board reimport, require 0 old source markers in CPU *and existing RHI item*; on return, require exactly one correct restored marker.
7. Audit existing RHI frame/picking/CPU fallback, Copper hole picking, mirror, per-layer/net explorer, scene resize, 10k synthetic interaction and controlled injected failure regression **by specific CI steps**, without inferring a physical GPU stress qualification.

**Acceptance pending:** exact candidate HEAD native Qt/Windows x64+ARM64/macOS ARM64/Linux offscreen checks, source CTest receipt flags, full 17 protected contexts, independent exact-HEAD Codex review, merge and main-push checks.

**Explicit active-RHI gate:** `.github/workflows/qt-runtime.yml` now runs the full source `--selftest` again **after** Qt Quick is installed and rebuilt on Windows x64/ARM64 and macOS ARM64. Each job requires `a034_local_contact_cpu_rhi_filter_verified=true`, `a034_local_contact_rhi_pick_scope=PASS`, `a034_cross_file_reimport_scope=PASS` and native test PASS. The receipt is archived as `qt-preview-a034-contact-results.json`. Merely seeing a CPU-only CTest success is not accepted as native RHI picking evidence; a real hardware GPU *adapter* is still a separately scoped qualification. Complete this paragraph only on real evidence.

## Existing regression coverage reused (not newly certified)

| Existing suite or native workflow step | Supported evidence type | Limit |
| --- | --- | --- |
| A_019 RHI interaction/benchmark | Synthetic picking, pan/zoom/filter, indexed 10k-1M model and scene frame timing when hardware confirmed | Hardware VRAM/reset and a universal 60 FPS promise NOT_MEASURED |
| A_020 source explorer | Imported filtered objects, Net names, selected Properties and panel resets | No physical electrical NetIds |
| A_021–A_030 Qt native source geometry | Explicit PADS copper, cutouts, circular and curved source-owned geometry; bounded shared CPU/RHI meshes | No automatic flooded, fabricated or thermal contacts |
| A_032 local contact-record decoder/transport | Source SHA, original exact endpoint Net/Layer/coordinates and local marker transport | No whole-board Open/Short/Pad/Pin inference |
| A_033 source grid Golden fixtures | Exact independently-authored source coordinates, object row sets and fail-closed mutation | Synthetic fixtures only; vendor export NOT_VERIFIED |
| A_034 first native source cross-file smoke | UI reset/fail-closed, no stale imported objects or local markers | Standalone one-file package cannot certify cross-file |

## Qualification matrix

- **PASS (previous merge):** bounded source-backed cross-board Qt import lifecycle at `main@1607d177`.
- **PENDING (current candidate):** CPU/RHI contact visibility, active RHI contact picking, exact one-marker restoration, reviewed native CI.
- **NOT_VERIFIED:** genuine redistributable vendor-produced PADS V9.4 export fidelity, complete PCB Open/Short, Net-to-Pin/Area/thermal contacts.
- **NOT_RUN:** measured SI/RLGC/S-parameters, physical GPU device reset/VRAM, qualified 1-million-object 60 FPS on customer hardware, redistribution licensing certification.
- **Out-of-scope:** broad M1/P2 umbrella Issues #9/#11/#165 stay open; P2 final acceptance is a **separate A_035 gate**.

All tests use existing public synthetic fixtures, offline in CI. No proprietary PCB source, sensitive Net names, paths or CAD exports are uploaded.
