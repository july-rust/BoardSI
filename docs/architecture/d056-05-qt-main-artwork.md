# D_056_05 — Canonical Padstack in Main Qt Artwork (in verification)

**First implementation:** 2026-11-09 KST · **Status:** DRAFT / NOT ACCEPTED until exact PR HEAD CI + verified merge and post-main push.  
Parent [#520](https://github.com/july-rust/BoardSI/issues/520), gate [#531](https://github.com/july-rust/BoardSI/issues/531), implementation [PR #551](https://github.com/july-rust/BoardSI/pull/551), prerequisite [D_056_04](d056-04-canonical-padstack-ipc.md).

## What actually changes

- Keep the existing **main** `ImportedArtworkPanel`/ `PreviewImportPanel` and the unchanged legacy Artwork V1–V14 wire. No newly qualified source record is downgraded to a uniform-diameter, all-layer Via.
- User-accessible **Canonical layer vias** command starts the opt-in, already existing Rust `boardsi-import-canonical-padstack` producer in its own bounded `QProcess`. The separate legacy artwork importer continues as before.
- Reuse D_056_04 `d056ipc::decode` and require the actual original-file SHA-256 before GUI installation. The Canonical source’s frame, physical ordered layers, Via/Padstack/Drill ids, per-layer shapes, roles and exact rational geometry are retained.
- New `d05605::make` constructs an independent **view geometry model**; all source-grid acceptance remains in the existing strict V1 IPC. One glyph per explicitly present pad, with no fabricated missing landings or plane contact.
- CPU draws actual circles/rectangles/annuli as `QPainterPath`, subtracting the explicit larger of source annulus inner aperture and drill hole. Nonconductive antipads/clearances are dashed outlines, not copper. Rendered picking is limited to accepted conductive footprints.
- Where safe under explicit tessellation/resource limits, existing main Qt Quick/RHI Scene Graph draws the same Copper extents through bounded triangle batches with a conservatively circumscribed inner hole: rendering never fills the declared void. Hover/selection are on source-bound geometry, not screenshot RGB. Unsupported narrow ring, no Quick or device loss falls back to CPU.
- Existing UI layer visibility, hide/show/solo/restore, per-object properties and Fit are reused. Backend switching/re-import invalidates prior model; a cancelled or digest-changed subprocess must never publish its result.

## What must NOT be asserted

The SOURCE model can carry exact rational pad geometry; CPU/RHI rendering discretizes it for visibility. RHI is a conservative visualization approximation and its lack of coverage of a thin outer rim is not proof of electrical isolation. A hardware adapter, actual fabricator drill/plating, real board connectivity/whole-board Open or Short, P009 return path, RLGC/S solvers and product U/T remain **NOT_VERIFIED / NOT_RUN**. Declared `plated: true` is only a SOURCE declaration. Unsupported PADS thermal spokes, slot/backdrill and other native CAD formats are not silently accepted.

## Independent tests (acceptance pending exact CI evidence)

- `qt-d05605-main-canonical-artwork`: actual main ImportedArtworkPanel for 2/4/6/8/64 physical layers; L2–L3/L2–L4 and Through/Blind/Buried spans; asymmetric rectangle and annulus; missing middle-layer copper and nonconductive antipad; drill-hole hit-test, layer hide/show, SHA rejection, legacy V1–V14 scene reload.
- `qt-d05605-main-rust-canonical-roundtrip`: actual Rust CLI to main PreviewImportPanel process, strict SHA/IPC/geometry to Qt and stale scene rejection.
- `qt-d05605-rhi-holes-layer-picking`: Qt Quick scene model per-layer RHI source-bound geometry, annulus hole and nonconductive anti-pad picking, legacy scene reset. Offscreen/native CI cannot by itself certify real GPU adapter pixels or manufacturing.
- Existing Qt offscreen and native Windows x64/ARM64, macOS ARM64, Linux GUI/Worker regression, Rust release/debug, Security and protected required checks must all pass **at the final PR HEAD** before acceptance.

## Remaining issue gate and change hygiene

PR #551 must be independently reviewed at the final exact SHA, with zero unresolved material findings, every required GitHub check successful, protected **Verified single-parent Squash** and separately verified exact-sha `main` event=push workflows. Update the **single** [development checklist](../development-checklist.md) only with genuine GitHub evidence and dates; the `D_056_05` acceptance and Issue #531 remain **OPEN** while any gate remains incomplete. Parent Issue #520 stays OPEN through D_056_06–12. Docs-only amendments must skip unrelated native runners while retaining essential docs/security checks.
