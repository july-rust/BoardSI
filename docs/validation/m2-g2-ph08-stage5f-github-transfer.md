# PH-08 Stage 5F — GitHub-first transfer (2026-10-16 KST)

## Scope
This change transfers Stage 5C/5D synthetic numerical diagnostics and Stage 5E integrity/provenance tooling from local Chat artifacts into versioned GitHub source. All new PH-08 files are under `tools/validation`, `docs/validation`, and an optional workflow; existing Rust FEM, Frozen PH-03/05/06/07 contracts, product specification and established 17 required checks remain unchanged.

## Verified / unresolved
- Stage 5E ZIP auditor: 22 synthetic positive/negative unit tests passed in local Chat test execution; exact remote GitHub Actions status must be separately checked.
- Stage 5C/5D: independent nine-site quadratic/cubic reconstruction and rounding/conditioning diagnostics. **Not** the original Stage 4/5B HMM, not a 2D electrostatic PDE solve, and not a physical accuracy reference.
- Original Stage 4/5B evidence ZIP bytes are still unavailable in this execution. **Do not claim** 338 checksums or 20 source records were checked.
- Original Gate Q **FAIL**, AC-P1 **PENDING**, Contract **NOT FROZEN**, Rust FEM **NO-GO**.
- Main baseline `f9d1e860bed2856049e0faac6e9861e7ee0ae160`. Ruleset 25046595 retains Strict 17/17, signed commits, Squash-only, review-thread resolution, no bypass. This evidence-only transfer must not weaken rules or auto-merge.
- The GitHub API commits on this working branch can be **unsigned**. The maintainer must sign the branch changes using the existing Verified SSH signing workflow, and review exact PR head required CI results, before any merge.

## Next actual work (GitHub-first)
1. Obtain original Stage 4/5B archives without publishing confidential data. Place files as private local evidence or controlled GitHub Actions artifacts only if their public redistribution and retention are approved.
2. Add format-aware tests against the real manifest schema and source hash inventory; do not guess a checksum for Stage 5B.
3. Reproduce original HMM Gate Q using the original source and raw gradient outputs; test exact polynomial reproduction, scaling, reconstruction, and finite-precision error separately.
4. Review quantitative bounds and full AC-P1 evidence independently. Only then consider Contract freeze and a separately scoped Rust FEM PR.

## PR merge restrictions
Draft only until signed and all required checks on exact HEAD are confirmed. PR #35 remains draft/unmerged. Keep Issue #46 separate.
