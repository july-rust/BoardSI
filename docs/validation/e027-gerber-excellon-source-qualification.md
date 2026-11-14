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

## E_027 code PR #636 protected merge receipt (2026-11-14 KST)

- PR final reviewed HEAD: `69f3e5aed3a604535d2bba2d6f217b49541aa483`; 30/30 GitHub Checks SUCCESS and 9/9 PR Workflows SUCCESS, including Linux/Windows x64/Windows ARM64/macOS ARM64 Rust/Qt Native. Independent Codex final-HEAD review `completed`, no findings and no unresolved threads.
- The original P2 FileFunction attribute byte-offset provenance error was fixed and its inline review thread resolved. The earlier `dfaff409` Windows ARM64 unrelated D_006 `copper_cutout_blocks_candidate` failure is retained as historic FAIL; a rerun was cancelled following branch synchronization, while the exact final PR HEAD passed Windows ARM64. Do not relabel the historic failure as PASS.
- Protected merge commit [`cb898e56827bdc8eda25e9957903c0be606d10c8`](https://github.com/july-rust/BoardSI/commit/cb898e56827bdc8eda25e9957903c0be606d10c8): verified GitHub signature `valid`, exactly one parent `7402d35a15e653a93133717fdd8bc98952784d42`. Code PR #636 merged and closed.
- **Exact merged-main Push CI CERTIFIED:** [Numerical #43592214120](https://github.com/july-rust/BoardSI/actions/runs/43592214120), [Qt #43592214083](https://github.com/july-rust/BoardSI/actions/runs/43592214083) and the six other code-main workflow runs at `cb898e56` finished **8/8 SUCCESS** and **28/28 Checks SUCCESS**. Numerical first-run macOS ARM64 failed the preexisting D_006 `pad_perimeter_boundary_and_multiple_area_owner_are_not_accepted` test (`D006_INCOMPLETE_SOURCE_INVENTORY`); exact-SHA job rerun **attempt 2 SUCCESS**. Preserve the initial failure and the successful rerun separately; it is not an E_027 Gerber/Drill parser failure.
- **Documentation review availability:** PR [#639](https://github.com/july-rust/BoardSI/pull/639) encountered `Codex usage limits for code reviews` on its initial docs-only HEAD. Automated Codex review of this **documentation PR is NOT_VERIFIED**; it is not equivalent to the completed clean independent code review of PR #636. The E_027 limited source software implementation and post-merge CI are technically qualified, but the formal E_027 document-review operational closure gate cannot be marked independently reviewed without that external proof. Record blocker truthfully; do not invent a review or delete the original failed receipt.
- This receipt qualifies only the restricted *source inventories*, not full Gerber artwork, production Excellon dialects, a fabricated Canonical BoardModel, cross-format geometry, Qt importing of Gerber, manufactured connectivity or SI solver input. Parent #23/#532/#520 remain open.
