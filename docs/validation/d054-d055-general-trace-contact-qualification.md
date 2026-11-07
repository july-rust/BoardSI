# D_054–D_055 — Generalized SOURCE Trace Contacts (straight, circular arc, Via annulus, SMD pad)

**First checked: 2026-11-07 KST. Evidence date: 2026-11-07 KST.**

- D_054: [Issue #506](https://github.com/july-rust/BoardSI/issues/506), [Code PR #507](https://github.com/july-rust/BoardSI/pull/507), main squash `45878db712f2786c0852c2fb2b0153b584a4ce29`.
- D_055: [Issue #508](https://github.com/july-rust/BoardSI/issues/508), [Code PR #509](https://github.com/july-rust/BoardSI/pull/509), main squash `983f2b871d9a96b361899573207ca3a89c529c27`.
- D_055 PR tested head `9aed939b747cd621af10776cfcc8be9239539d9f`. Nine PR workflow groups reported **success**: Numerical core, Qt runtime, M2 synthetic G0, M2 isolated FEM, M2 native FEM, PH-06 independent FVM, PH-07 independent FVM, A032 import benchmark, and Validate planning documents. PR-head native Rust Debug/Release covered Windows x64, Windows ARM64, macOS ARM64 and Linux; existing Qt native regression covered Windows x64/ARM64, macOS ARM64 and Linux.

## Why the design changed

Do **not** enumerate 0°, 45°, 90°, diagonal quadrant, Via-centred endpoint, Pad-centred endpoint, individual edge or individual corner as separate production connection logic. A trace can enter a Via or SMD pad at *any supported SOURCE-grid coordinate*, including its side, corner, midspan or a non-centred point. The common kernel computes **finite-width copper footprint intersection**.

### Reusable geometrical rules

- `trace_contact.rs`: one analytical straight-segment / directed CW-or-CCW circular arc centreline engine, a shared conservative envelope and uncertainty policy. `centreline_point_distance`, `centreline_point_farthest`, and doubled-tick `centreline_rectangle_distance` are reused by the new attachment kernel. Doubled ticks preserve rectangular Pad edges at half-source-tick locations instead of truncating them.
- `trace_attachment.rs`: source-exact plated Via has an **outer conductive pad diameter** *and* a **drilled aperture**. A finite-width Trace must overlap the annulus, not merely the empty bore. On intermediate layers, a Via pad is eligible only where the restricted source actually declares an inner conductive pad. A Through-Via span alone is not proof of layer contact.
- Circle SMD Pad: segment/arc centreline minimum distance to pad centre is compared with the sum of the trace and pad radii. Square and rectangular SMD Pad: full filled rectangle (interior, four edges, all corners) compared with finite trace width. No trace-centre-to-pad-centre identity requirement.
- A strict positive-area overlap yields `PositiveArea`, a certified gap yields `Separated`, and a near-tangent uncertainty yields `BoundaryUncertain` and **aborts candidate reporting**. Source inventory issues, malformed geometry, invalid radii, an unsupported Part, missing plating evidence, pair-budget overflow and skipped routing are never reclassified as valid geometry.
- Trace/Pad declared nets use the existing source-owned PADS ROUTE pin-assignment parser. Different-net geometric overlap is labelled as a **conflict candidate**, and a Pad with no declared Net remains `PadNetUnverified`. Neither class becomes an electrical graph edge automatically.

### Available opt-in diagnostic (not the production Qt transport)

```text
boardsi-import-artwork <supported-source.asc> --trace-pad-via-contacts-v1
```

Its bounded output begins `BOARDSI_TRACE_PAD_VIA_CONTACT_V1|<source-sha256>|...` and enumerates `VIA` and `SMD_PAD` SOURCE-local positive-copper-area candidates. Existing Qt Artwork V1–V14 stdout remains unchanged. Neither this diagnostic nor the D_054 `--general-trace-contacts-v1` endpoint asserts global board connectivity or a physical SI result.

### Direct regression evidence

- D_055 adds **10 Rust tests**, each PASS in both Debug and Release on the PR-head Linux workspace run: off-centre annular Via contact; hole-only non-contact; one-tick interior/exterior and tangency; explicit Via layer; SMD round/square/rect Pad edges/corners/midspan; circular Trace to Via/Pad; different/unknown declared Net; 360 generated directional samples using **one** algorithm; invalid footprints; and two modified **original public PADS V9.4 ASC fixtures** proving off-centre Via route and corner-to-SMD Pad contact without centre coincidence.
- All previously qualified D_001–D_054 Rust and Qt regressions were retained in the mandatory PR workflow results. The Qt CI result is a **regression pass** for the existing GUI; it does not certify that the new candidates have been promoted to the canonical global graph or displayed by the Qt GUI.

## Explicit limitations / NOT VERIFIED

This is bounded **same-layer SOURCE-local copper overlap classification** for the existing PADS V9.4 MILS parser subset. It does **not** establish manufactured copper plating, arbitrary Via padstack shapes, buried/blind/microvia definitions, rotated custom or oval SMD pads outside the supported parser, thermal spokes, fills/pours, all PADS revisions, full-board net connectivity, electrical Open/Short certification, Reference return paths, numerical RLGC/S-Parameter accuracy, performance on arbitrary dense boards, or U/T official product acceptance.

**Next product-integration boundary:** after provenance-confirmed geometry and source-net checks, transfer independently verified local candidates into the canonical graph with dedicated evidence types and validate their representation in the actual Qt connectivity UI. This shall be treated as **integration work**, not an invitation to resume one-development-step-per-angle/edge/corner enumeration.

**Post-main push CI** is a distinct evidence gate and is not inferred from the PR-head workflow PASS above.
