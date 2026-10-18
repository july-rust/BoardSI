# BoardSI whole-project baseline audit — 2026-10-18 KST

**Issue:** [#78](https://github.com/july-rust/BoardSI/issues/78). **Starting SHA:** `4c8926d526f85b72f0659914aa8823050d5414c6`. Read-only historical/GitHub checks were performed; this audit is not a line-by-line formal verification of every source file, nor a new original HMM run.

## Independently checked project inventory

GitHub API returned **213 file/directory entries** in the complete current main tree (`truncated=false`), **47 historical PRs** and **17/17 required Check Runs success** at the verified main SHA. Active [Ruleset 25046595](https://github.com/july-rust/BoardSI/rules/25046595): strict checks, signatures, squash-only, review-thread resolution, deletion and non-fast-forward protections, **no bypass**. Required approving review count is **0** by documented single-maintainer choice. Protected passing CI is *scope-specific regression evidence*, not physical certification.

## Engineering scope ledger

| Stage | Verified implementation scope | NOT VERIFIED / remaining |
| --- | --- | --- |
| M0 specification | V1.0 requirements U01–U48, tests T01–T54 listed, 11-stage roadmap | IDs alone are not completed acceptance results; software licensing and binary distribution |
| M0 numerical | uniform supplied scalar RLGC→ABCD→S independent reference tests, native CI #27 | true CAD-derived R/L/G/C, loss, coupled ports, 30 GHz or measured PCB correlation |
| M0 board model | typed Canonical Board Model, structural integrity/coordinates and synthetic tests #28 | real PADS/ODB++ parsing, complete trace/pad/via Boolean, connectivity and field geometry |
| M0 Worker/Qt | framed local async Rust Worker + Qt Widgets/QProcess native prototypes #29/#30/#49 | final viewer, end-to-end GUI, cache, recoverable run and native installer; timer root cause #46 OPEN |
| M2 independent FEM | limited PEC/lossless 2D G0/G1/PH-03/05/06/07 math coupons #33/#34/#37-#40/#42/#45 | lossy G3, differential G4, adapter G5, real PCB physical validation, overall #16 OPEN |
| PH-08 Stage 4 original | 36/36 selected MMS rows privately reexecuted; original uniform quadratic gradient reductions **0.953506** and **0.459163** < preregistered **1.8** → **FAIL** | full original globally authenticated HMM coefficient/source/Recovery/gradient-RMS floating budget |
| PH-08 Stage 5B original | authorized PRIVATE original reexecution on 2026-10-17 reproduced original Gate Q **FAIL**, **12/12** rows, **1761/1761** deterministic scalars matching; 12 timing values excluded | public GitHub CI is *synthetic only*; no whole-domain physical certification, AC-P1 or Gate M |
| M1–M5 application | format priorities and architecture/issues | importer, artwork, stackup editor, ports, production sweep/Touchstone, cache, actual binaries |
| Branding | **official option B** selected in earlier 2026-10-14 conversation | original A–E icon design contact sheet found in Library but its raw PNG byte export denied; standalone B asset absent from GitHub; provisional **RECONSTRUCTION_ONLY** SVG not approved original |

**Immutable physics block:** Stage 4/Stage 5B original Gate Q **FAIL**; AC-P1 PENDING, T/R PROPOSED_NOT_FROZEN, PH-08 contract NOT_FROZEN, Rust FEM NO_GO, Gate M NOT_VERIFIED, physical BOUND_NOT_CERTIFIED. A public synthetic matrix test or perfect exact rational identity does not prove original HMM upstream roundoff bounds.

## Confirmed audit findings / priority

1. **P1 confirmed document defect:** `docs/development-history.md` claims **Stage 5B original physics rerun remains NOT RUN**, contradicted by `docs/validation/m2-g2-ph08-stage5b-original-replay-evidence.md` and #51 (private 2026-10-17 replay). Correct history to distinguish **private historical FAIL reproduced** from **NOT RUN in public synthetic CI** and **physical NOT CERTIFIED**.
2. **P2 confirmed:** `docs/architecture/README.md` still calls Qt vs Rust IPC decision open despite existing Worker/Qt Widgets implementation. Distinguish prototype from product decision.
3. **P2 confirmed:** root README says native platforms `not yet verified` though numerical/FEM/Qt prototype native jobs ran, while actual product binary unverified. Clarify scope.
4. **P2 confirmed:** project management text calls PH-08 a future task; current Issue #51 work is active but scientifically blocked. CONTRIBUTING describes planning/M0 despite accepted limited M2 coupons.
5. **P2 confirmed:** original chosen BoardSI icon B file was not in baseline GitHub tree. Store decision and marked reconstruction, then recover genuine original before release.
6. **P2 remaining:** docs status guard does not yet enforce current PH-08 historical replay cross-document consistency; improve without altering frozen PH-03/05/06/07 contracts.

## Open risks, controls and completion criteria

- **P0:** full original PH-08 A/b, float coefficient and sparse solve, all original recovery stencils/whole RMS, finite/open-domain Neumann/Dirichlet and source-to-field bounds are **NOT CERTIFIED**. Do not change historical 1.8 acceptance threshold retroactively.
- **P1:** macOS Qt native test once observed global UI gap **1134 ms** > unchanged **1000 ms** gate. Later passing runs do not prove root cause solved (#46).
- **P1:** native prototype CI 17/17 success does not equate to completed all 54 system tests or real import-to-solver pipeline.
- **P1:** ensure all audit commits are GitHub **Verified**, required strict checks PASS on exact head, threads resolved, squash-only, no bypass. **Never merge an unsigned PR**.
- **P2:** repository has no finalized distribution software license; inspect Qt, parser, dependency and icon IP terms before distributing binaries.
- **P2:** G5 [Draft PR #35](https://github.com/july-rust/BoardSI/pull/35) remains open/UNMERGED. Physics #51, Qt #46 and solver #16 remain OPEN.

Remediation must not modify frozen contracts, FEM/core numerical calculations, security limits, acceptance criteria or the 17 required checks. Confirm exact head and post-merge 17/17 before marking any item complete. Original accepted option-B art requires separately acquired original bytes and SHA-256 documentation; reconstructed concept must never masquerade as exact artwork.

**Sources:** [#16](https://github.com/july-rust/BoardSI/issues/16), [#41](https://github.com/july-rust/BoardSI/issues/41), [#46](https://github.com/july-rust/BoardSI/issues/46), [#51](https://github.com/july-rust/BoardSI/issues/51), [#78](https://github.com/july-rust/BoardSI/issues/78), [original Stage5B evidence](../validation/m2-g2-ph08-stage5b-original-replay-evidence.md), [original Stage4](../validation/m2-g2-ph08-stage4-original-gateq-reexecution.md).

### Additional source recovery after baseline

The earlier 2026-10-14 1536×1024 `BoardSI 아이콘 디자인 제안 보드.png` was **visually recovered** from user's personal Library (A–E proposals plus B-1 small icon). Its raw byte materialization was rejected by the available Files service, so it could not be committed bit-identically; a different vector version is **not** accepted as the original. The B-specific white circuit border/vias and smooth blue wave were used only to improve the explicitly provisional reconstructed SVG. Original SHA-256, font/creator provenance and final portable icon files are **NOT VERIFIED**.

### Additional P2 historical evidence correction

[Stage 5D historical synthetic report](../validation/m2-g2-ph08-stage5d-evidence.md) incorrectly described the **338 checksum / 20 source** nested Stage 4 inventory as though it belonged to Stage **5B**. Correct classification: Stage 5B outer archive **36 checksum / 13 source**, nested Stage 4 archive **338 checksum / 20 source**. The original Stage5D *session-local* `Not Run` was valid at that historical date and remains a valid historical statement; the later separate private Stage5B original replay reproduced **FAIL** on 2026-10-17. No historical test outcomes or acceptance criteria were altered.
