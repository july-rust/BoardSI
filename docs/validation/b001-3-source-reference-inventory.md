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
| Native Rust strict inventory tests | T04 different Net names remain distinct; equal name rejected; incomplete inventory / wrong reference Layer error | NOT_RUN until CI |
| Native Qt-to-Rust selftest | Source candidate remains unqualified; explicit hole/split candidate blocked, same Net rejected, incomplete input visible | NOT_RUN until CI |
| Windows x64 / ARM64, macOS ARM64 & Linux Qt | GUI and native companion round-trip, plus existing artwork RHI | NOT_RUN until CI |
| Independent Codex latest SHA | Reviewer findings resolved | NOT_RUN until CI |
| Actual topology/Port/Return Path | Imported complete Copper connectivity, P009 T24 physical proof | NOT_IMPLEMENTED |
| Whole Issue #12 / U09 U10 U39 T02 T04 T24 T52 | Production complete definition of done | NOT_QUALIFIED |

No uploaded company PCB or vendor material; all test data synthetic and source identifiers fictitious. Do not mark full B_001 complete until actual imported geometry and full acceptance gates are independently demonstrated.
