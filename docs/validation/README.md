# Independent verification

The v1.0 development specification contains T01–T54 acceptance test definitions. These tests are NOT implemented merely because IDs appear in documentation. Separate: format parsing fidelity, electrical/numerical reference solutions, coupling/port conventions, adaptive vs direct sweep, GUI responsiveness, crash recovery, confidentiality and native platform execution. Each future PASS must include sample provenance, reference, model validity, method/units, hash, OS/compiler and observed results.

## M2 PH-03 exact mathematical plate (2026-10-15)

- [Pre-registered PH-03 acceptance contract](m2-ph03-parallel-plate-acceptance-contract.md)
- [G1 FEM evidence and PH-03 test results](m2-g1-lossless-fem-evidence.md)
- [Independent M2 plan and G1→G2 order](m2-rlgc-independent-verification-plan.md)

PH-03 is a numerical mathematical anchor only; no real PCB or 30 GHz physical qualification is implied.

## M2 G2 PH-05 ideal symmetric stripline (2026-10-15)

- [Pre-registered PH-05 acceptance contract](m2-g2-ph05-acceptance-contract.md)
- [Independent FVM method, SHA-256 and numeric reference](m2-g2-ph05-independent-reference.md)
- [FEM PH-05 limited acceptance and regression report](m2-g2-ph05-fem-evidence.md)

The PH-05 class-F **limited synthetic PEC/lossless** coupon passed 81 independent acceptance checks in [CI 38666133102](https://github.com/july-rust/BoardSI/actions/runs/38666133102). It does NOT validate actual PCB loss/roughness, 30 GHz, discontinuities or overall M2.

## M2 G2 PH-06 asymmetric and two-region PEC stripline (2026-10-15)

- [PH-06 immutable acceptance contract](m2-g2-ph06-acceptance-contract.md) — preregistered SHA `15da48ef3753e361d009decd03dbd4516dfd9868`.
- [Independent cell-centred FVM, material interface, source/fixture SHA-256](m2-g2-ph06-independent-reference.md).
- [Measured FEM/FVM comparisons, independent upper/lower ground charge, mesh/geometry/material sweeps](m2-g2-ph06-fem-evidence.md).

PH-06 limited synthetic lossless 2D field acceptance recorded 201 numerical and 57 invariant PASS gates in [CI 38669970099](https://github.com/july-rust/BoardSI/actions/runs/38669970099). This **does not** validate a real PCB, 30 GHz, multilayer microstrip, loss or full M2. The existing PH-05 and PH-03 tests remain mandatory.

## M2 G2 PH-07 multilayer microstrip with mask — 2026-10-15

- [Pre-registered PH-07 acceptance contract](m2-g2-ph07-acceptance-contract.md) (first commit `c11440a1`)
- [Independent Python FVM reference and SHA-256](m2-g2-ph07-independent-reference.md)
- [Rust FEM actual calculation/mesh/boundary/charge evidence](m2-g2-ph07-fem-evidence.md)
- [PR #40](https://github.com/july-rust/BoardSI/pull/40), initial [PH-07 CI run 38674095933](https://github.com/july-rust/BoardSI/actions/runs/38674095933): 740 numerical checks PASS (8 C/C0 cross-method comparisons plus 8 derived Z0/delay comparisons), 0 FAIL.

Only restricted synthetic ideal PEC/quasi-TEM electrostatic class F is supported, with explicit rectangular mask/top/side and layered substrate. PR #40 merged at `c6dce055`; final HEAD `875f314a`, [CI 38674727369](https://github.com/july-rust/BoardSI/actions/runs/38674727369) PASS; real PCB, 30 GHz full-wave and R/G loss models remain Not Run.

## M2 validation audit hardening (#41)

[Executed audit, check taxonomy, native platform ledger and administrative limits](m2-validation-audit-hardening.md). Frozen acceptance contracts remain unchanged.

## PH-08 Stage 4 / Stage 5B (historical FAIL; no physical approval)

- [Original Stage 4 36-row HMM FAIL reproduction](m2-g2-ph08-stage4-original-gateq-reexecution.md): 36/36 rows reproduced, historical uniform quadratic gradient ratios below 1.8, FAIL unchanged.
- [Original Stage 4 Recovery operator selected-case audit](m2-g2-ph08-original-recovery-operator-evidence.md): selected captured binary64 matrices and exact rational algebra, not physical forward-error certification.
- [Gate Q captured binary64 T/R proposals](m2-g2-ph08-gateq-tr-exact-captured-proposed.md): proposed synthetic-only registry; original dataset/reduction proof and prospective policy NOT VERIFIED.
- [Stage 5B archive preflight / original replay boundary](m2-g2-ph08-stage5b-original-replay-evidence.md): previous private archive digest record, GitHub public synthetic integrity tests only. **Original Stage 5B physics replay NOT RUN**.

Stage 5B archived Gate Q remains FAIL; AC-P1 PENDING, PH-08 contract NOT FROZEN, Rust FEM NO-GO, Gate M NOT VERIFIED, physical bound BOUND_NOT_CERTIFIED. Synthetic GitHub CI does not change these statuses.
