# B_001_3 — PADS-accepted Copper/Net inventory → fail-closed Reference candidate audit

Date: **2026-10-27 KST** · [Issue #12](https://github.com/july-rust/BoardSI/issues/12) · [PR #217](https://github.com/july-rust/BoardSI/pull/217)

## Bounded implementation

The Qt partial PADS V9.4-MILS import controller now exports a bounded, **source-owned inventory only after accepted Import and validated Artwork load**: individual source Copper Area Layer ID, original Area identifier, declared Net label, source line number, explicitly imported polygon/circle cutout count, and curve/circle markers. A bounded list of declared Net names comes from accepted Track, Arc, Via and Copper Area records, with a fail-closed completeness flag if more than 128 distinct names or an oversize name occur. The **SMD Pad transport does not expose a qualified Net**; no pad-to-net association is invented.

Every new import attempt (including a failed or cancelled reimport) clears the previous inventory. A successful import resets material state and supplies this fresh inventory to the Stackup / Reference editor. Explicit *user-selected* Signal and Reference Net labels are kept distinct from per-Layer role hints and the source inventory. Selection edits invalidate audit status immediately, and explicit labels are source SHA-bound when saved in a user Overlay.

The existing native `boardsi-stackup-config --audit-stdin-v1` rejects malformed source inventory, non-Copper layer Area, duplicate Area names, Net inventory/Area conflict, oversized/malformed IDs, and identical selected Signal/Reference Net. It distinguishes no source evidence, incomplete source net inventory, unknown/mismatched Net/Layer, visible Cutout or multiple disjoint Area identifiers, and source Net labels with **no proven electrical topology**.

**Hard invariant:** the Rust receipt continues to contain `reference_continuity=UNSUPPORTED_NO_COPPER_NET_EVIDENCE`, `return_path_p009=UNSUPPORTED`, `electrical_net_separation=NOT_VERIFIED`, `solver_ready=false` and `physical_si=NOT_RUN`. Source Net labels cannot certify connections or a GND Plane. This is not a complete topology import, geometric cutout-crossing detector, Port implementation or SI solve.

## Required tests and evidence (do not pre-certify)

| Test | Claim boundary | Exact HEAD result |
| --- | --- | --- |
| Native Rust strict inventory tests | Different source Net labels remain electrically unqualified; same Net denied, cutout/split Unsupported, truncated/missing inventory degrades without material failure | **PASS** at code SHA `d15b86db4cc044da07930fbc9baad41ea0bc891a` ([Numerical workflow](https://github.com/july-rust/BoardSI/actions/runs/40428481561)) |
| Native Qt-to-Rust selftest | Source candidate never qualifies return; explicit holes and split, same Net, empty declared Net, truncation and GUI rejection reason checks | **PASS**; [Linux 11/11 CTest](https://github.com/july-rust/BoardSI/actions/runs/40428481598/job/121940089361) with all B001 flags true |
| Windows x64 / ARM64, macOS ARM64 & Linux Qt | 3 Native plus Linux runtime, source Import Artwork GPU/RHI and packaging | **PASS** at code SHA `d15b86db4cc044da07930fbc9baad41ea0bc891a`: [Win x64](https://github.com/july-rust/BoardSI/actions/runs/40428481598/job/121940089822), [Win ARM64](https://github.com/july-rust/BoardSI/actions/runs/40428481598/job/121940089804), [macOS ARM64](https://github.com/july-rust/BoardSI/actions/runs/40428481598/job/121940089680), [Linux](https://github.com/july-rust/BoardSI/actions/runs/40428481598/job/121940089361) |
| Independent Codex latest SHA | Reviewer findings resolved | NOT_RUN until CI |
| Actual topology/Port/Return Path | Imported complete Copper connectivity, P009 T24 physical proof | NOT_IMPLEMENTED |
| Whole Issue #12 / U09 U10 U39 T02 T04 T24 T52 | Production complete definition of done | NOT_QUALIFIED |

No uploaded company PCB or vendor material; all test data synthetic and source identifiers fictitious. Do not mark full B_001 complete until actual imported geometry and full acceptance gates are independently demonstrated.

## Limited test checkpoint (2026-10-27 KST)

**Code SHA `d15b86db4cc044da07930fbc9baad41ea0bc891a` 27/27 GitHub Checks SUCCESS, 17 required contexts subject to latest docs HEAD reconfirmation.** The public synthetic `pads_copper_filled_v94.asc` import produced `source_copper_net_inventory=true`: the GND source Area remained declared while the other Area had no Net and **was not substituted**. A short/return path was neither connected nor certified. The native validator distinguishes `OBSERVED_LABELS_NO_TOPOLOGY_PROOF`, `CUTOUT_OR_SPLIT_UNSUPPORTED`, `NO_SOURCE_REFERENCE_EVIDENCE`, and `UNKNOWN_OR_MISSING_SOURCE_NET`; same-Net user choices are denied. Qt GUI preserves fixed Rust rejection reasons. Source/new board reimport clears stale evidence.

This ledger modification is a **new PR HEAD**, requiring its own exact-head CI, Codex review and protected signed merge. Formal U09/U10/U39/T02/T04/T24/T52 and physical P009 Return Path still unqualified.
