# E_010 — Main Qt Canonical Import / IPC / Picking / ReImport qualification

> Work date: 2026-11-12 KST. Parent [Issue #11](https://github.com/july-rust/BoardSI/issues/11); implementation [PR #595](https://github.com/july-rust/BoardSI/pull/595). **Bounded SOURCE/Qt software qualification only; not full M1, actual GPU hardware, electric continuity, RLGC, S parameters, or formal U/T acceptance.**

## Frozen small scope

An actual main `PreviewImportPanel` invokes the real `boardsi-import-canonical-padstack` Rust process for public `crates/boardsi-import/tests/fixtures/pads_via_round_through_v94.asc`, receives versioned D_056_04 Canonical IPC, and populates the *existing* `ImportedArtworkPanel`. The added `qt-e010-main-canonical-picking-reimport` Qt CTest exercises SOURCE SHA-256 identity, drill-void exclusion and conductive-annulus `QGraphicsScene::itemAt` picking; main Layer/Net UI filters; same-controller distinct SOURCE NET_A→NET_B reImport; stale selection/Properties/Source identity invalidation; and malformed V9.3 header rejection by the actual Rust child process.

PADS V9.4 MILS text coordinates are *MILS*, normalized by `parse_fixed` into **centimil SOURCE ticks** (100 ticks per MILS). The canonical coordinate frame uses exactly 1 tick = 254 nm = 0.000254 mm; thus SOURCE (300,200) MILS becomes the viewer Via centre (7.62,-5.08) mm. The public via outer diameter 35 MILS is 0.889 mm, its drill diameter 15 MILS is 0.381 mm; a point 0.30 mm away from centre lies within conductive annulus, centre lies in nonconductive drill void. These are source-geometry assertions, *not* electrical continuity.

## Explicit failure chronology, retained

1. Initial E_010 test incorrectly located the Qt synthetic scene rather than the imported scene; Codex independent review found P1. Fixed by looking up `QGraphicsScene` under `ImportedArtworkPanel` directly, review thread resolved.
2. Another initial test used PADS numeric MILS text directly as centimil ticks, producing a 100× position error. Linux CTest 84/85 passing and new E_010 failing in PR runs; explicit phase diagnostics narrowed the failure to the first imported Source-A Via geometry hit test. Corrected source-vs-canonical units. Historical FAILED runs remain FAIL, not retroactively PASS.
3. Native workflow historically only executes full CTest on Linux. E_010 explicitly adds the actual CTest executable to native Windows x64, Windows ARM64 and macOS ARM64 `Execute actual native Qt backend` steps with offscreen Qt platform, in addition to the Linux CTest. Software/offscreen execution is not GPU hardware RHI parity certification.

## Acceptance evidence and status

**E_010 bounded SOURCE/Qt software scope COMPLETE (2026-11-12 KST).** Exact final-HEAD required CI and independent Codex review passed; protected GitHub Verified single-parent Squash and exact merged-code-main event=push **8/8 workflows SUCCESS** independently confirmed. The earlier macOS M0 global event-loop >1500 ms failure was preserved as historical FAIL; a rerun of failed Qt job on the same code HEAD passed, and subsequent exact code-main Qt Push passed independently. This is NOT Issue #46 root-cause resolution. Formal product U/T remain NOT_RUN.

| Evidence gate | Actual result |
| :--- | :--- |
| Implemented source | [PR #595](https://github.com/july-rust/BoardSI/pull/595) |
| Final exact code HEAD / tests | [PR #595](https://github.com/july-rust/BoardSI/pull/595) exact HEAD `50dedc2e2659381bbeb0a7d53fef84c390e19267`; 10/10 PR workflows ultimately SUCCESS (Qt macOS failed first attempt on #46 gap, then retry succeeded) |
| Native Linux/Windows x64/Windows ARM64/macOS ARM64 | SUCCESS at exact PR HEAD via Linux CTest and dedicated native E_010 executable; exact merged-main Qt Push SUCCESS |
| Independent Codex final HEAD / unresolved threads | Exact HEAD `50dedc2e2659381bbeb0a7d53fef84c390e19267` Codex receipt: no major issues; original earlier P1 scene-selection thread resolved; 0 unresolved threads |
| Protected Verified Squash SHA, single parent | [`0bc459ecfc5419c3eaef420c8f48dfa59fffa87e`](https://github.com/july-rust/BoardSI/commit/0bc459ecfc5419c3eaef420c8f48dfa59fffa87e); GitHub commit verification `verified=true/reason=valid`, single parent `50266bde12421667625bb8f805973d01ca0e138e` |
| Exact merged-main event=push workflow and jobs | **8/8 workflows SUCCESS** for `0bc459ecfc5419c3eaef420c8f48dfa59fffa87e`, including [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/43141803172) and [Numerical core](https://github.com/july-rust/BoardSI/actions/runs/43141803256); PR CI not substituted |
| Docs closeout PR and exact docs-main push | [PR #596](https://github.com/july-rust/BoardSI/pull/596) — final documentation-only CI, independent review, Verified Squash and exact docs-main Push must be separately recorded in #11 after execution; no new Native PASS claimed for docs-only |

## Unaccepted boundaries

The restricted source is one public synthetic PADS V9.4 Through Via, not independent vendor-export PCB evidence. The Qt scene only represents source-declared visible geometry; declared Net names/Through-span/Plating do not demonstrate manufactured electrical contacts or full Open/Short. No realistic board SI/PI solver, model numerical fidelity, 30 GHz performance, or formal U08/U22/U31 and T03/T34 product acceptance is claimed. Hardware GPU RHI frame/pixel/latency #165 remains separate. Parent Issue #11 stays OPEN until its actual complete Definition of Done is independently demonstrated.
