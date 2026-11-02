# D_013 — PADS legacy V1 barrel compatibility provenance qualification

**First recorded:** 2026-11-02 KST. **Issue:** [#371](https://github.com/july-rust/BoardSI/issues/371). **Code PR:** [#372](https://github.com/july-rust/BoardSI/pull/372). **Scope:** bounded opt-in, exact-source **legacy compatibility projection provenance ONLY**, not physical electrical continuity.

## Qualified fixture and representation

- Requalifies the original PADS V9.4 MILS Source SHA, one Through Via Net/position/source span/drill, its explicitly selected Layer 1/4 conductive pads, two same-net straight Trace geometries and four Source-owned COPCLS areas through D_010 → D_011 → D_012.
- The accepted D_011 graph remains **8 nodes, 6 layer-local candidate edges, 2 local components, 0 source-verified cross-layer electrical edges**.
- `route_preview::into_canonical` sets `Drill.plated=true` as an **IMPORTER ASSUMPTION**, not proof of original PADS manufacturing plating. The Legacy V1 `contact_wire::legacy_v1_barrels` projection is a backwards-compatibility connection over exact TraceVertex/ViaCentre contacts; it is **not evidence of manufactured conductive barrel**.
- An opt-in D_013 receipt embeds the exact D_012 span witness, independently requalifies the restricted Source/Canonical via-contact geometry, and records `LEGACY|0|VIA:0:L1|VIA:0:L4|V1_COMPATIBILITY_PROJECTION|PLATING_IMPORTER_ASSUMED|BARREL_ELECTRICAL_NOT_VERIFIED`. The single legacy V1 link is **not** inserted into the D_011 electrical candidate graph.
- Fail-closed fixed tags: `PLATING_IMPORTER_ASSUMED`, `BARREL_ELECTRICAL_NOT_VERIFIED`, `CROSS_LAYER_ELECTRICAL_NOT_PROVEN`, `SOLVER_READY_FALSE`, `END|FAIL_CLOSED_NO_BARREL_PROOF`.
- The receipt is deterministic, source SHA-bound, limited to **72,000 bytes**. Existing ordinary importer outputs, earlier D_001–D_012 opt-in commands, and public Canonical Model contracts remain unchanged.
- CLI: `boardsi-import-artwork board.asc --d013-legacy-barrel-provenance-v1 RA RB RC RD NET_A 1 4 0`, separate Rust binary `boardsi-d013-legacy-barrel-provenance` and Qt selftest `--d013-legacy-barrel-provenance-selftest`.

## Independent verification and negative gate

- Rust 1.75 `fmt`/Clippy/workspace Debug+Release, deterministic receipt, source reimport SHA changes, incorrect Via/Net/Layer, forged geometry, overflow and source-contact failures were exercised through existing Numerical Core CI. Added an off-centre endpoint regression: a Source Trace endpoint strictly inside the Via pad but not **at ViaCentre** can pass D_012 geometry yet must not qualify for the legacy V1 barrel; the D_013 Rust witness rejects it. The test fixture normalizes Windows CRLF/LF before constructing this change.
- Qt reconstructs D_012 from **actually imported PADS** (not just a provided receipt) and independently checks both ImportedTrack endpoint positions for exact ImportedVia centre equality on the two selected layers before accepting a legacy compatibility projection. Forged/stale/source-reimport/wrong-span/false-plating, legacy record and count mutation fail-closed; stale risk markers are not retained.
- Linux Offscreen Qt CTest on exact code HEAD: **36/36 PASS**, including `qt-preview-d013-legacy-barrel-provenance`; recorded `D013_NATIVE_QT_LEGACY_BARREL true … IMPORTER_ASSUMED_LEGACY_BARREL_NOT_ELECTRICAL`.
- Native Qt **Windows x64, Windows ARM64, macOS ARM64** each executed the existing D_012 Source-only smoke and its independently chained D_013 selftest successfully on the same exact final PR HEAD. No additional GitHub Actions workflow was introduced.

## Verified code PR acceptance — 2026-11-02 KST

- [x] [PR #372](https://github.com/july-rust/BoardSI/pull/372) final exact code HEAD `3fefba3be82282ef9253c713607d6a4fd72b0c60`: **27/27 Check Runs SUCCESS, 11/11 pull-request Workflows SUCCESS**, active repository Ruleset requirements enforced.
- [x] Exact final-HEAD independent Codex `status=completed, has_any_findings=false, has_unresolved_findings=false`; all inline review threads resolved. An original P2 finding on Qt pad-containment vs exact ViaCentre endpoint was corrected and re-reviewed.
- [x] Protected **GitHub Verified single-parent Squash** `main@b657403c4e0f30f844001c65c5d5d9adc1571f4e`, `verification.verified=true, reason=valid`, sole parent `b18a46e0879b67ed8b2788a1a0c20eb3286b9443`.
- [ ] Independent exact post-code-main **event=push** Workflow Runs and Check Runs: **not yet accepted at the time of initial document drafting**. Check separately on the immutable code-merge SHA, not a PR result.

## Explicit exclusions, fail-closed state

```text
source_via_span = QUALIFIED_ON_RESTRICTED_FIXTURE
legacy_v1_compatibility_projection = OBSERVED_SOURCE_ENDPOINT_MATCH
plating_provenance = IMPORTER_ASSUMED
manufactured_plating = NOT_VERIFIED
via_barrel_electrical_continuity = NOT_VERIFIED
cross_layer_electrical_edges = NOT_PROVEN
manufactured_copper_and_fill = NOT_VERIFIED
global_open_short = NOT_VERIFIED
physical_return_path_p009 = UNSUPPORTED
ph08_and_physical_si = NOT_RUN
solver_ready = false
cache_reuse_eligible = false
formal_m1_u09_u10_u39_t02_t04_t24_t52 = NOT_ACCEPTED
```

A missing candidate does **not** establish a manufactured open; a legacy barrel count does **not** establish a physical short, return path or continuity. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN and Draft [PR #35](https://github.com/july-rust/BoardSI/pull/35) remains unmerged.

## Document PR and Issue #371 closeout gate

This document and `docs/development-checklist.md` require their own final-HEAD CI, exact-HEAD independent Codex review with zero unresolved findings, protected GitHub Verified one-parent Squash and **independent post-document-main push CI**. Close Issue #371 only **after** both code-main and document-main push CI are green; never backdate or infer this from PR checks.
