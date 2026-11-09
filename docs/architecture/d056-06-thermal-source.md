# D_056_06 — PADS RT/ST Thermal Spoke SOURCE / Canonical / Qt Integration

> **Candidate pending exact-head CI, protected Verified Squash and main-push audit.** No product/physical electrical or solver acceptance is claimed here.

Parent [#520](https://github.com/july-rust/BoardSI/issues/520) · PADS Thermal/Manufacturing [#530](https://github.com/july-rust/BoardSI/issues/530) · [PR #553](https://github.com/july-rust/BoardSI/pull/553) · [D_056_05 main Qt baseline](d056-05-qt-main-artwork.md)

## Source grammar (independently cross-checked)

**Primary vendor**: *PADS Layout ASCII Format Specification*, V9.4, Via Definition Format, pp. 38–40: [public PDF](https://www.freecalypso.org/pub/CAD/PADS/pdfdocs/Ppcb_ASCII.pdf). **Independent parser cross-check:** KiCad PADS parser [implementation reference](https://fossies.org/linux/kicad/pcbnew/pcb_io/pads/pads_parser.cpp).

SOURCE VIA pad row: `level size RT|ST spokeori outsize spokewidth spokenum`.

- `RT` round central land and round thermal-void envelope; `ST` square central land and square envelope.
- `spokeori` is the first direction (counter-clockwise from right / source +X). Uniform subsequent directions = initial angle + `360° * k / spokenum`. The vendor uses 1-degree precision; parser accepts the exact integer (including `.000` spelling), does **not** invent fractional-angle support.
- `outsize` is the thermal envelope/plane-void width, **not** a conductive solid copper disk. `size` is central land diameter or square width, `spokewidth` is each conductive radial spoke width.
- Vendor example: `-2 60 RT 45 84 15 4`. V9.4 PADS MILS parser preserves exact hundredth-mil source grid and SHA.
- PADS relative `-2/-1/0` selectors map to actual physical drill span; no fixed 4L assumptions. RT/ST shapes on missing levels are not manufactured.
- Actual metal plating, plane fill, CAM thermal generation and barrel contact depend on separate source/process evidence. The PADS ROUTE `THERMAL` flag alone does not prove thermal spokes exist in a manufactured pour.

## Implemented interfaces (candidate)

| Component | Implementation | Important limitation |
| :--- | :--- | :--- |
| `thermal_source.rs` | Bounded seven-field parser, source SHA + line diagnostic, RT/ST zone classification | Only supported V9.4 MILS subset; whole PADS variant coverage not proven |
| `via_source.rs` / `via_shapes.rs` | Actual placed RT/ST via, per-physical-layer spans, exact dimensions, copper vs clearance, no round downcast | D_056_07 process grammar is separate |
| `boardsi-model` `PadShape::Thermal` | Parametric round/square centre, outer relief, spoke width/count/origin | Geometry only, no manufactured process proof |
| `canonical_padstack_ipc.rs` | V1 unchanged; V2 used **only** if Thermal exists, P row with centre/outer/width/angle/count | Strict body SHA, source SHA, field bounds; unsupported variants fail closed |
| Qt `d056_canonical_ipc.h` | Transactional V1/V2 decoder and V2 negative protection | V1 cannot accept a Thermal shape |
| Main Qt `d05605_canonical_artwork.h` | CPU positive centre/spoke path, exact drilled void, outer relief outline; source/layer/net Properties | QPainterPath is a visual display, not measured physical copper |
| RHI `rhi_artwork.h` | **CPU fallback** for complex Thermal instead of incorrect solid-disk GPU drawing | Hardware GPU pixel-equivalence certification NOT_DONE |
| `thermal_contact.rs` | Strict interior SOURCE finite-width straight/arc trace–thermal positive witness; safe separated envelope/bore negative; otherwise UNKNOWN | Sampling is sound-positive but intentionally not exhaustive |
| `layered_area_contacts.rs` | Strict interior SOURCE-filled Copper–thermal pad/spoke candidate witness | A POUROUT boundary/declared net is never accepted as filled Plane copper |
| `layered_pad_contacts.rs` | RT/ST pair-contact kept UNKNOWN when overlap is unproven | No fabricated RT-to-RT electrical edge |

Only local-source geometry candidates can be promoted. Source-declared net membership does not establish electrical Open/Short; no actual barrel plating, microvia fabrication, Return Path, RLGC or S-parameter product acceptance.

## Versioned IPC rows

V1: `BOARDSI_CANONICAL_PADSTACK_V1` and original `P|stack|L|role|CIRCLE/RECT/ANNULUS|a|b` remain byte-compatible.

V2: `BOARDSI_CANONICAL_PADSTACK_V2`, with source-SHA, body-SHA, byte/record counts and positional records retained. RT/ST additional row:

`P|stack|L|CONDUCTIVE|THERMAL_RT/THERMAL_ST|centre_m|outer_m|spoke_width_m|first_angle|spoke_count`

Rational metres are exact integer numerator/denominator, angles/count bounded and validated by both Rust and Qt, without converting thermal outer void to a single solid copper circle. V2 only applies when at least one Thermal record exists.

## Regression and release gate

- Public synthetic PADS V9.4 fixture: `crates/boardsi-import/tests/fixtures/pads_via_rt_st_thermal_v94.asc` (RT L2, ST L3, ordinary landing L4 on six-layer buried span).
- Independent vendor-RT 45° 4-spoke oracle, ST 0° source, hole/clearance/tangency and invalid field/zero spokes, source mutation.
- Rust actual SOURCE import → Canonical model → V2 wire; Qt strict IPC → main CPU artwork roundtrip, including 2/4/6/8/64 synthetic layer invariants.
- V1 regression must remain unchanged; V2 zero-spoke forgery with independently recomputed body SHA must be rejected.
- Native Qt Linux x64 / Windows x64, ARM64 / macOS ARM64 and original D_001–D_055 validations in required CI.
- Merge code through protected Verified Squash **only** after exact final PR HEAD required checks pass, all reviews resolved; run independent main `event=push` validation. Reconcile sole `docs/development-checklist.md` only after accepted SHA evidence; docs-only jobs should skip unrelated native.

**Caveat:** Vendor grammar establishes RT/ST parameters, not source-complete CAM plane fill. An unresolved nonnegative geometry candidate remains UNKNOWN, not declared Open or Short. Parent #520 and Issue #530 must remain open for D_056_07 and downstream adapters. 
