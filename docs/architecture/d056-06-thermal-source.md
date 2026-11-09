# D_056_06 — PADS RT/ST Thermal Spoke source diagnostic (candidate)

Status: **IN PROGRESS, NOT ACCEPTED**. This first slice is a bounded source-level thermal record reader and local geometry diagnostic. It does **not** wire Thermal copper into the electrical graph, canonical IPC, or Qt CPU/RHI main artwork. **It is not D_056_06 acceptance.**

## Vendor grammar and cross-check

- Primary: Mentor Graphics, *PADS Layout ASCII Format Specification*, V9.4 (2012), p.40, Via Definition Format: https://www.freecalypso.org/pub/CAD/PADS/pdfdocs/Ppcb_ASCII.pdf
- Layout: level size RT|ST spokeori outsize spokewidth spokenum. Example: -2 60 RT 45 84 15 4; angular spacing is 360/4=90 degrees. RT is round, ST square. Outsize represents plane void/outer size, not a solid copper disk.
- Independent: KiCad parser RT/ST branch: https://fossies.org/linux/kicad/pcbnew/pcb_io/pads/pads_parser.cpp, which stores orientation/outer diameter/width/count separately.

## Scope of new Rust module

crates/boardsi-import/src/thermal_source.rs provides strict seven-field parsing, bounded geometry parameters and SOURCE-local selectors, a V9.4-MILS VIA section scanner, conservative RT/ST point-zone diagnostics and unit tests for vendor RT sample, ST, invalid inputs and truncation. The scanner does not infer that VIA instances are placed, that the plane is filled, or that a barrel is plated.

Existing production via_source.rs deliberately returns D056_THERMAL_CONTACT_UNSUPPORTED and remains the safety gate. This diagnostic may not be presented as complete production support.

## Unfinished mandatory gates

1. Physical-layer mapping, placed VIA instances and source SHA/record identity.
2. Exact spoke/plane clearance geometry in Canonical model without lossy round-pad fallback.
3. Versioned Rust→Qt IPC and actual main CPU/RHI artwork integration.
4. Independent positive/negative source Copper contacts and UNKNOWN when plane fill, process or net is missing.
5. Native 4-platform tests, old regressions, CI and independent final main Push.
6. Update only the authoritative docs/development-checklist.md after accepted protected Verified Squash and push evidence.

Issue #530 and parent #520 remain OPEN. D_056_07 owns slot/NPTH/backdrill/microvia manufacturing grammar. Physical manufactured Open/Short, SI, RLGC/S remain NOT VERIFIED.
