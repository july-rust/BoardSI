# C_013 — Signal Via Pad vs explicit Reference COPCUT Source clearance (V9)

**Tracker:** [Issue #273](https://github.com/july-rust/BoardSI/issues/273) · **Parent:** [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN · **Predecessor:** [C_012 V8](c012-signal-reference-via-pad-clearance.md).

**Started 2026-10-29 KST. Status: code accepted by protected main; post-main and document acceptance pending.** The original protected baseline is `main@869d8d2deb0edfb1394137336ba3ed953612ad17`, C_012 #270 CLOSED after 21/21 checks and 9/9 workflows on final post-doc-main.

## Bounded Source geometry

- Opt-in `--reference-route-v9` on the existing two-layer C_012 Source route, with one unique Signal Through Via and one explicit Reference Stitch Via.
- Regenerate V8 using the original bytes and embed it *byte-for-byte*; V7/V6/V5/V4/V2 remain transitively embedded. Unmodified V1–V8 CLI paths retain previous protocol.
- Independently reparse Source PADS V9.4 MILS, Canonical Via and round conductive Pad, plated Drill span, two selected reference layers, and source-owned Reference Copper Area Net/Layer/ID.
- This incremental V9 supports **one rectangular Straight COPCLS** and **zero or exactly one explicitly same-owner rectangular Straight COPCUT per Reference layer**; rejects nonrectangular, curved, round, multiple/ambiguous or otherwise unsupported geometry.
- Each Source Signal Pad must be strictly contained within the matched Source Copper outer rectangle; for an explicit hole, compare its exact Source-tick minimum wall distance against half the full conductive Pad diameter. Equality is exact tangency. A missing COPCUT never implies electrical isolation or a positive clearance.
- `SOURCE_CUTOUT_GAP_OBSERVED` means Source geometry only. Other states: `SOURCE_CUTOUT_TANGENCY_RISK`, `SOURCE_REFERENCE_COPPER_OVERLAP_RISK`, `SOURCE_CUTOUT_MISSING_RISK`. Unsupported is error/fail-closed, not a fabricated gap.
- Exact classification is integer-based; any displayed distance in mm is approximate and never an acceptance predicate.
- Qt independently verifies the V8 receipt, source SHA, Source Area identity and imported original cutout rectangle, then draws the source-sized Signal Pad rim on both selected layers in CPU/RHI. Existing Reference Copper cutout is already displayed by the Artwork layer. Selected input changes, source watcher/reimport, process generations and late IPC must invalidate prior markers.

## Implementation and verified code evidence

- Rust: `crates/boardsi-import/src/reference_route_v9.rs`, executable opt-in in `src/bin/boardsi-import-artwork.rs`, publicly reproducible `tests/c013_signal_via_cutout_v9.rs`.
- Qt: `apps/boardsi-qt-prototype/c005_reference_risk.h`, `imported_artwork.h`, `rhi_artwork.h`; actual source-to-Importer-to-Viewer C_013 selftest in `preview_p0.cpp`.
- Native CMake test and native workflow for Windows x64, Windows ARM64, macOS ARM64; Linux offscreen.
- **Code evidence PASS**: [PR #274](https://github.com/july-rust/BoardSI/pull/274) exact reviewed HEAD `d15169e1c5b5367bc96ff9cb1cfbc37144db9d6e`, all **27/27** PR CI checks SUCCESS including native Windows x64/ARM64, macOS ARM64 Qt and Linux offscreen; Rust Debug/Release, fmt, Clippy, and public C_013 tests (4/4 Debug, 4/4 Release). Native F6/F7 and FEM checks also succeeded.
- **Codex PASS on exact final code HEAD**: `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`; prior 3 P2 threads resolved, latest [review receipt](https://github.com/july-rust/BoardSI/pull/274#issuecomment-6497326931).
- **Protected merge PASS**: GitHub Squash-only [main commit `96f49248d0cda95b729734683d9918cd9444a45a`](https://github.com/july-rust/BoardSI/commit/96f49248d0cda95b729734683d9918cd9444a45a), GitHub commit signature `verified=true`, `reason=valid`, parent `869d8d2deb0edfb1394137336ba3ed953612ad17`.
- **Still pending**: independent complete code-merge post-main 21 required check runs and 9 push workflow results, plus documentation PR/merge and separate final `main` verification. Preserve Issue #273 OPEN until all evidence exists.

## Explicit non-claims

```text
return_path_p009     = UNSUPPORTED
physical_si          = NOT_RUN
solver_ready         = false
cache_reuse_eligible = false
```

No manufactured Copper fill, antipad, thermals, DRC minimum rule, full-board electrical topology, physical Return Current/P009, RLGC, impedance, S-parameters, Solver or Cache readiness. Formal U09/U10/U39 and T02/T04/T24/T52 and parent M1 #12 remain unchecked/OPEN.

## Closure gates

- [x] V9 Rust and Native Qt implementation tests prove supported source cases and prior V8 byte compatibility (27/27 exact PR HEAD, 2026-10-29 KST).
- [x] Exact final code HEAD required CI + independent same-HEAD Codex completed with zero findings/unresolved (PR #274, SHA d15169e, 2026-10-29 KST).
- [x] Protected Squash of code is GitHub Signed Verified (main 96f49248; 2026-10-29 KST).\n- [ ] Code merge independent post-main checks/workflows all successful.
- [ ] Dated checklist and this record finalized; documentation PR if separate passes required CI/review/verified merge/post-main.
- [ ] Only then close Issue #273. Parent #12 stays OPEN.
