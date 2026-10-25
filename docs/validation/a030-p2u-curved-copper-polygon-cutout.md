# BoardSI A_030 / P2-U — curved COPCLS + one convex straight COPCUT

## Status
**VERIFIED COMPLETE for the restricted A_030 contract** (2026-10-26 KST). Acceptance evidence is the final closeout comment on [Issue #191](https://github.com/july-rust/BoardSI/issues/191#issuecomment-6403878315); **this does not certify overall P2 or real PCB physics**.
- Source baseline: protected `main@0e4e465145479e3142c78ca344dc936284f921e8`, merged A_029 PR #190.
- Issue #191.
- Candidate branch `a030/p2u-curved-copper-polygon-cutout` at reviewed source HEAD `34c6b5759a8fceda56b2caa2ad4f20bee02576d0`.
- [PR #192](https://github.com/july-rust/BoardSI/pull/192) Squash merged as protected `main@bcdfa269233730efbbfa35f34f5ee43d18153b0b`.
- Original Issue #191 final record reports 21/21 final candidate checks, 17/17 post-merge required checks, successful exact-HEAD Codex review with no findings, resolved threads and GitHub Verified signature. See [main post-merge CI](https://github.com/july-rust/BoardSI/actions/runs/40193161416).

## Accepted narrow source grammar
PADS V9.4 MILS `*LINES*`: explicit same-name `COPPER` owner, declared **two** pieces, A_029 analytically certified outward convex cardinal quarter-arc `COPCLS`, one simple strictly convex straight `COPCUT` on the exact same layer. Closed rings, no boundary contact/overlap, no standalone voids, no extra/mixed circle cutout, no arbitrary/nonconvex arc.

Rust independently validates the source-owned analytical outer and conservatively inscribed display polygon, then rejects a hole whose vertex violates any strictly inward convex display half-plane. The exact source Arc/Line and Cutout Line boundaries remain separate in Canonical, with `net=None`. This is a source-layout preview, never a flooded conductor or proof of electrical connectivity.

Source-grid coordinates: exact 0.01 mil ticks = 254 nm. Predicates for strict containment and winding use i128 in Rust; file/vertex/coordinate/ring budgets are bounded. Qt consumes the validated exact nm wire, validates A_029 source arc consistency again, removes strictly redundant collinear **display-only** points, and invokes the existing A_025 independent triangle-hole oracle. The same resulting triangles feed Qt CPU GraphicsView and Qt Quick/RHI GPU/picking. Analytical source segments are untouched.

## Wire protocol
`BOARDSI_P2U_PARTIAL_V14|V9.4-MILS|<layers>`.
- Source curved owner: existing `G|R` format (V13).
- Source-owned indexed straight cutout: existing `H|<G index>|<vertex count>|x|y...` (V11).
- V14 END: 12 numeric counts, with curved-object count and owned polygon hole count as fields 11 and 12; `CANONICAL_PARTIAL|SI_NOT_RUN` as fields 13/14 (zero-based split indices).
- Earlier V1–V13 format behavior is unchanged; V14 is emitted only if the imported objects contain a curved outer with cutout.

## Public acceptance tests
- `pads_curved_copper_polygon_cutout_v94.asc`: one analytic four-quarter-arc outer and one strictly interior convex rectangular COPCUT.
- Rust independent exact-tick valid/invalid topology tests, real restricted importer parse, Canonical provenance, V14 CLI receipt + counts, adverse owner/layer/overlap/convexity tests.
- Qt `--curved-hole-selftest`: actual import -> Qt V14 decoder -> CPU empty hole and filled copper controls, provenance/properties, visibility toggles, malformed V14 refusal, cleanup and actual imported Qt Quick/RHI frame with void-point pick refusal.
- Mandatory native Qt workflow across Linux, Windows x64, Windows ARM64 and macOS ARM64 plus existing A_022–A_029 regressions.

## Exclusions
- Actual PADS vendor export comparison: NOT_VERIFIED.
- Electrical connectivity and actual filled/fabricated Copper: NOT_VERIFIED.
- Substrate/stackup, production RLGC/S-parameters and SI: NOT_RUN.
- Actual GPU VRAM, driver reset and 1M primitive 60 FPS: NOT_RUN.
- Multiple/concave/curved/circular holes in a curved outer: UNSUPPORTED.

## Acceptance gate
1. Full Rust checks, native Qt checks, adversarial decoder checks pass for the exact candidate SHA.
2. Independent Codex exact-final-HEAD rereview 0 P1/P2 findings and 0 unresolved threads.
3. All protected required checks SUCCESS; GitHub-Verified Squash Merge only.
4. Separate post-merge main SHA/signature and mandatory CI fully verified.
5. Issue #191 closed only after proof. Broad #9/#11/#165 stay OPEN.

**Final A_030 disposition:** Verified complete for explicitly supported single convex straight COPCUT inside source-owned cardinal outward curved COPCLS. See #191 final acceptance evidence. Real vendor exports, copper flood/electrical connectivity, RLGC/S and hardware GPU performance remain NOT_VERIFIED / NOT_RUN.
