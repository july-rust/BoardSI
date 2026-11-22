# F_009 — PADS/ODB++ Native `.bsi` Source-Free Artwork Replay

**Implementation and PR acceptance: COMPLETE · 2026-11-23 KST. Stage final acceptance: PENDING exact merged-`main` Push CI receipt.** The F_009 result applies only to the explicitly supported F_005 PADS V9.4 / ODB++ Native Artwork subset, not to full fabrication, connectivity or SI qualification.

## Immutable contract and delivered capability

- Accepted Source -> Canonical -> original SQLite v1 / BSIG `.bsi` -> independently validated Native Reader -> actual Qt QProcess Artwork; the `.bsi` can reopen **without the original CAD Source**.
- Two-or-more Copper Layer restricted display subset: two-point straight Trace, exactly representable circular / axis-aligned rectangle conductive Pad, and uniform circular Via annulus with explicit pad-layer inventory. Native source-local `connected_layers` is preserved **without claiming manufactured copper contact or validated SI**. Via visibility is filtered by actual Padstack copper entries, never simply by the span or by a net label.
- Correct PADS / ODB source-to-Qt Y-axis handedness. The PADS public fixture Y=200 mil corresponds to Qt Y=-5.08 mm; a nonzero-Y ODB line terminates at Qt Y=-0.75 mm. Existing KiCad/IPC-2581/Gerber rendering conventions are unchanged.
- Unsupported arcs, areas, cutouts, arbitrary rotation, incompatible Padstack/Via geometry, single Copper Layer, nonintegral-nm projection, over-limit geometry or oversized JSON **never** produce an invented or silently partial Artwork. The structurally valid Native file remains loadable while GUI receipt explicitly records `ARTWORK NOT_RENDERED` / `BSI_F009_*`. The F_005 source import gate was **not** widened to accept single-Copper-layer PADS.
- Bound Preview only: at most 128 graphical primitives, 32 Copper layers, coordinates ±10^12 nm, dimensions at most 10^9 nm, and a full serialized Qt worker receipt cap below its 65,536-byte limit; these are *display* bounds, not Native format/Reader limits.
- Source SHA/SourceIdentity/Provenance and Canonical round-trip are preserved without claiming a renewed verification of inaccessible Source. SQLite v1, BSIG, required extension and F_004 Golden compatibility contracts stay unchanged.
- `solver_ready=false`, `physical_si=NOT_RUN`, and source-local/electrical contact remains `NOT_VERIFIED` unless separately proven. No physical PCB analysis, global copper connection or formal U/T acceptance is implied.

## Stage-local F009 acceptance

| Gate | Actual evidence/coverage | Result |
|:--|:--|:--|
| T01–T02 | F_005 real PADS V9.4 and bounded ODB++ Source Writer / independent `.bsi` readback | **PR CI PASS** |
| T03 | Rename/remove source CAD, then open saved Native without Source | **PR Rust + real Qt PASS** |
| T04 | Source-known exact nm dimensions, Trace/Pad/Via subset, asymmetric PADS/ODB screen Y | **PR Rust + Qt PASS** |
| T05 | Full Canonical/Provenance/Geometry native semantic equality, SourceIdentity | **PR Rust/F004 PASS** |
| T06 | Production Qt QProcess GUI Artwork, Layer Filter, scene selection/picking | **PR Qt 4-OS PASS** |
| T07 | Valid Native with unsupported orientation/single Layer remains loadable but Artwork NOT_RENDERED; exact nm/size limits | **Implemented; covered exercised subsets PASS**, not universal proof of every invalid input |
| T08 | Malformed/unsupported extension/source-change fail-closed inherited F_003/F_004/F_005 | **PR regression PASS** |
| T09 | Native Save As and reopened Artwork after source removal | **PR Rust/Qt PASS** |
| T10 | Existing Worker kill/cancel/timeout/stale-response and Qt responsiveness | **PR F003 Resilience/Qt PASS** |
| T11 | Linux x64, Windows x64, Windows ARM64, macOS ARM64 Native + actual Qt | **PR 4-OS PASS** |
| T12 | Previous F_001–F_008 regressions, exact-HEAD Codex, protected Squash and merged-`main` Push | **PR/Review/Merge PASS; post-`main` Push NOT_VERIFIED** |

This is the F_009 **local subset** ledger, not a product-wide U01–U48/T01–T54 completion claim.

## Exact review, CI and merge receipts

- [Implementation PR #671](https://github.com/july-rust/BoardSI/pull/671): final implementation HEAD `922a2d74a8202a83cbce49c0ddf149e2eea6448b`, **14/14 PR workflows SUCCESS**; zero failed/cancelled/pending.
- Exact-final-HEAD independent Codex review: `status=completed`, `commit_sha=922a2d74a8202a83cbce49c0ddf149e2eea6448b`, `has_any_findings=false`, `has_unresolved_findings=false`; all historical review threads resolved.
- [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/45001707265): Linux x64, native Windows x64/ARM64, native macOS ARM64 **all SUCCESS**. [Numerical core](https://github.com/july-rust/BoardSI/actions/runs/45001707398) and [F_004 compatibility](https://github.com/july-rust/BoardSI/actions/runs/45001707197): all 4-OS jobs SUCCESS. Native/FEM/Resilience and other PR workflows likewise SUCCESS.
- [Protected Squash implementation merge](https://github.com/july-rust/BoardSI/commit/bb53860c27108dcb14016aed77818988e50230b0): `bb53860c27108dcb14016aed77818988e50230b0`, GitHub `merged=true` for PR #671, **2026-11-23 08:12 KST**. Merge-signature verification and exact merged-`main` Push CI must be independently observed before being claimed.
- Documentation reconciliation uses a single checklist row and this ledger; no rewriting historic F_001–F_008 entries.

## Pending final close gate

- **Exact merged-main Push workflows for `bb53860c27108dcb14016aed77818988e50230b0`: NOT_VERIFIED (none returned by commit-scoped workflow lookup when checked).**
- **Final F_009 overall status: NOT_COMPLETE solely while the mandatory post-merge `main` receipt is unverified.** It must not be reinterpreted as failure of the verified 14/14 implementation PR checks; likewise PR checks must not be relabeled as post-merge evidence.
