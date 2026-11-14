# E_027 — Gerber X2 / Excellon Source-only qualification

**Scope (2026-11-14 KST):** opt-in Rust source inventory, not full Gerber/Excellon import, IPC/Qt integration, physical PCB acceptance or solver input. Parent issues [#532](https://github.com/july-rust/BoardSI/issues/532), [#23](https://github.com/july-rust/BoardSI/issues/23), [#520](https://github.com/july-rust/BoardSI/issues/520) remain open.

## Input and provenance contract

- The strict, size-bounded (8 MiB/file) Gerber X2 partial parser accepts absolute, leading-zero-omitted FS for X/Y, explicit MOMM, one TF.FileFunction Copper L1–L64 Top/Inr/Bot, a positive image (LPD), circular ADD apertures, Dnn selection, explicit X/Y D03 circular flashes and M02.
- It fails closed on D01/D02, arcs, region, macro, step-repeat, negative polarity, unsupported extended commands, duplicate declarations, malformed coordinates, unsafe control characters and truncated files.
- The separate Excellon source parser accepts M48 / METRIC / TnnCdecimal / % / Tnn / explicit XdecimalYdecimal round drill hits / M30. Zero-suppression guesses, routing, slots, backdrill, PTH/NPTH naming guesses, hidden unit conversions, unsupported tool commands and incomplete files are rejected.
- Coordinates and diameters are exact integral nm with checked arithmetic; at most 100k commands and 512 tool/aperture definitions. A SourceIdentity SHA-256 binds each original file's exact bytes, and Source ObjectMeta records deterministic source offsets and conversion provenance.
- CopperFlash uses ObjectKind::Pad as an **isolated source geometry label**, not a validated BoardModel Pad/Padstack. DrillHole uses ObjectKind::Drill as a **source hit**, not a fabricated Canonical Drill with manufactured span/plating.
- An inspect_pair call preserves two *independent* source identities. It does not verify common revision, origin alignment or ownership; even matching XY does not produce a via.

## Qualification and no-promotion rules

| Item | Result represented by this partial adapter |
| --- | --- |
| Copper layer attribution | Source-declared X2 L-number and Top/Inr/Bot only; no complete stackup |
| Flash/hole geometry | Restricted circular source geometry, exact integer nm |
| Plated/NonPlated status | UNKNOWN, not false / not assumed |
| Drill layer span | UNKNOWN |
| Net, component, material, dielectric, z/thickness | MISSING / UNKNOWN |
| Cross-file alignment / revision | UNVERIFIED |
| Connectivity graph / Solver / RLGC / S-parameters | NOT_RUN; solver_ready=false |
| Product-wide Gerber/Excellon, real vendor output, independent oracle | NOT_ACCEPTED |

The parsers do not create a BoardModel, Canonical Via, global electrical edges, manufactured Barrel witness or SI-ready source.

## Explicit regression receipt contract

Six unit-level synthetic qualification tests validate positive artwork/drill geometry and SHA, independent source IDs, source-only unknowns, declared inner/bottom X2 classification, unsupported graphics/drill mutations, truncation, size limits and tampering. These are not an independent CAD-tool or fabrication oracle.

**Execution/evidence gate:** add exact PR final HEAD SHA, Rust Debug/Release output, relevant Native CI job URLs, independent review outcome, Verified protected Squash SHA and exact post-merge main Push job results only once observed. Do not mark a pending CI or unavailable external oracle as PASS.

## Normative format references

- Ucamco, *Gerber Layer Format Specification* and X2 attributes: https://www.ucamco.com/en/guest/downloads/gerber-format
- The Excellon subset here is explicitly local and intentionally narrower than the family of Excellon/NC Drill dialects. Exact per-exporter dialect acceptance requires its own source reference and regression fixtures.

E_028 cross-format geometry equivalence, independent origin alignment, and multilayer via-span promotion are separate gates.
