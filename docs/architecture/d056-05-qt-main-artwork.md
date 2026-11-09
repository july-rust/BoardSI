# D_056_05 — Main Qt CPU/RHI Canonical Multilayer Via Artwork

**First implementation:** 2026-11-09 KST · **Code merge:** ACCEPTED (bounded SOURCE/Qt software scope) · **Code-main event=push:** pending independent final results · **Full D_056 and product SI:** OPEN / NOT_VERIFIED.  
**Related:** parent [Issue #520](https://github.com/july-rust/BoardSI/issues/520), Qt artwork [Issue #531](https://github.com/july-rust/BoardSI/issues/531), code [PR #551](https://github.com/july-rust/BoardSI/pull/551), upstream [D_056_04 IPC contract](d056-04-canonical-padstack-ipc.md), [single official checklist](../development-checklist.md#d_056_05).

## Evidence ledger — code acceptance, not physical PCB certification

| Gate | Evidence / exact boundary |
| :--- | :--- |
| Code final HEAD | [`7f251a2d802cad92f84c1b28f4906ec6ba59a958`](https://github.com/july-rust/BoardSI/commit/7f251a2d802cad92f84c1b28f4906ec6ba59a958/checks): **35/35 GitHub checks SUCCESS**; **11/11 PR workflow runs SUCCESS** |
| Native Qt supported platforms | Linux x64, Windows x64, Windows ARM64, macOS ARM64 **SUCCESS**, [Qt Runtime PR #42568746352](https://github.com/july-rust/BoardSI/actions/runs/42568746352) |
| Independent GUI/IPC regression | Linux Qt **81/81 CTest PASS**, with `qt-d05605-main-canonical-artwork`, `qt-d05605-main-rust-canonical-roundtrip`, and `qt-d05605-main-properties-accepted-canonical` PASS. Native RHI source-layer hole/picking smoke also wired on Quick-enabled builds; real hardware pixel parity is not established by model/offscreen tests |
| Security and other regressions | CodeQL and Rust/Qt/numerical/FEM/F6/F7 required PR workflows PASS. Exact code HEAD review identified **four P2 findings** in earlier heads (draw-order/picking, source Properties, CPU inner-hole pen, importer deployment), with code fixes and **0 unresolved review threads**; this is **not** a claim of a fresh completed Codex review on the final HEAD |
| Protected code merge | [Verified single-parent Squash `6103a1b6d6f1068b548f4cbb13d7c9cfff6544dd`](https://github.com/july-rust/BoardSI/commit/6103a1b6d6f1068b548f4cbb13d7c9cfff6544dd), GitHub REST `verification.verified=true/reason=valid`, parent `31dd4f27b5bd5e6f3621d0560d7195bfcc83d940`, matched `main` immediately after merge |
| Independent code-main Push | **PENDING:** record only once all 8 `event=push` workflows at exact code-main SHA complete SUCCESS (separate from code PR checks) |
| Docs-only ledger merge/Push | **PENDING:** distinct docs PR, no unnecessary native platform runners; report actual results without treating docs as product proof |

## Implemented in the actual main Qt Viewer

1. **No separate substitute Viewer:** `PreviewImportPanel` has user-visible **Canonical layer vias** importing to the existing `ImportedArtworkPanel`. The original Rust `boardsi-import-artwork` and versioned Artwork V1–V14 remain separate and unchanged; no partial/generalized Via is silently coerced to a legacy uniform L1–L4 Through Via.
2. **SOURCE evidence first:** independent `boardsi-import-canonical-padstack` Rust producer emits versioned D_056_04 V1 IPC. Qt validates frame body, SHA-256 of the actual source file, typed Source/Via/Stack/Drill IDs, physical layer ordering and Rational dimensions before presenting any new scene. `d05605::make` is display geometry, **not** independent proof of fabricated copper connectivity.
3. **Source-specific layer geometry:** declared Copper circle/rectangle/annulus role and dimensions, physical ViaKind/span (Through/Blind/Buried), Drill hole and explicit AntiPad/clearance outlines; missing intermediate-layer pads remain absent. Nonconductive anti-pad is not filled, selected or counted as Copper. Conductive geometry excludes drill/annulus aperture; CPU does not stroke across the hole.
4. **Native CPU/RHI:** exact accepted `QPainterPath` CPU hit regions with bounded visual tessellation for native Qt Quick/RHI; preserve source/painter stacking order for overlapping layer Pads and source-bound picking; cap Canonical RHI eligibility and fall back to CPU for unsupported ring widths/resource envelopes or unavailable GPU/Quick. The RHI rendered mesh conservatively excludes the drill aperture, but *visual mesh fidelity is not physical electrical evidence*.
5. **Main UI and lifecycle:** per-layer visibility/hide/show, fit, net filter, selection, Properties/Tooltip and native RHI hover source details. Versioned Source provenance and unsafe guesses (plating/connectivity) are labeled. Separate bounded, asynchronous `QProcess` has a 4 MiB stdout cap, 256-byte stderr cap, timeout and user Cancel. It rejects stale source hashes, failed/unsupported payloads and cancel/reimport races. Native tests exercise same-controller cancel/reimport recovery.
6. **Developer Windows x64 package:** source-tree launcher looks up the Canonical executable beside the actual legacy importer; staging bundles `boardsi-import-canonical-padstack.exe` and verifies x64 PE architecture and payload manifest alongside existing Qt runtime. **This is a development test package, not a certified release.**

## Test matrix and negative protection

| Input/condition | Accepted behavior |
| :--- | :--- |
| 2/4/6/8/64 physical layers, L2–L3/L2–L4 partial spans, nonuniform per-layer copper | Model/view use real ordered Layer + Span/Pad data without fixed layer-count switchcases |
| Round/rectangular/annular pads and explicit AntiPad; empty intermediate pad | Copper vs void and absent layer cannot be confused; real drill/annulus opening excluded from picking |
| Layer Visibility and source Net selection; hovered/selected glyph | Native main Qt scene reflects permitted source geometry and shows Via/Padstack/Drill/SHA without claiming electrical contact |
| SHA mismatch, invalid V1, unsupported source geometry, stale output, excessive IPC or stderr, timeout/cancel | Fail-Closed; no fabricated SOURCE success or electrical continuity; no stale candidate published |
| Legacy Artwork V1–V14 scene after Canonical mode | Legacy import path and previous tested consumers remain available; new generalized vias are **not** reinterpreted as validated legacy barrels |
| GPU unavailable, ring too thin, rendering resource budget exceeded | Main Viewer uses CPU fallback. Do not label software RHI model test as a real hardware pixel/latency certification |

**Known limitations:** only qualified subset of PADS V9.4 SOURCE, limited pad shapes and frame conventions, capped payload/object count; not complete exporter/Netlist coverage. Plane-specific thermal RT/ST, slots, NPTH, backdrill, manufacturing microvia evidence, Custom PadShape, other CAD formats and arbitrary area-contact combinations remain next gates. Do not treat a visually connected Barrel as electrically VERIFIED. No product-scale full-board Open/Short, real PCB plating/continuity, Return Path, MIPI RLGC/S-parameter/port flow, PH-08 or U/T acceptance has been performed here.

## Tracking rule

Use **only** [`docs/development-checklist.md`](../development-checklist.md) for the official D_056_05 limited acceptance checkbox and exact PR/code-main/docs-main CI evidence. Earlier failures (notably Windows MSVC C4456 on a superseded HEAD) remain historical FAIL, and were fixed then rerun at the final HEAD; never retroactively turn the old run into PASS. The **parent Issue #520** remains OPEN for D_056_06–12 regardless of this limited Qt artwork gate. Only close #531 when the post-code-main Push and separate docs tracking are verifiably complete.
