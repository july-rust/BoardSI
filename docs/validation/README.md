# BoardSI verification hub — evidence, exclusions and negative results

[← Documentation hub](../README.md) · [Development checklist](../development-checklist.md) · [Developer Quickstart](../developer-quickstart.md) · [Architecture](../architecture/README.md)

> **As of 2026-11-07 KST:** This directory contains **bounded verification evidence and historical executed reports**, not a blanket statement that the BoardSI product is validated. **Current implementation and the authoritative first-check/evidence dates belong to the [single checklist](../development-checklist.md).** Frozen PH case contracts and original historical failure results must not be redefined as part of documentation cleanup.

**전체 원본 보고서를 파일 단위로 찾으려면 [158개 검증 보고서 전체 색인](INDEX.md)을 사용하십시오.** 이 색인은 과거 단위 테스트와 미완·실패 데이터를 보존하며 공식 제품 인수 PASS를 의미하지 않습니다.

## Which evidence are you looking for?

| Test family | Entry points | Qualified scope / still blocked |
| :--- | :--- | :--- |
| M0 scalar math | [Numerical foundation](m0-numerical-foundation.md), [numerical contract](../architecture/numerical-contract.md) | Externally supplied scalar R/L/G/C → mathematical two-port S. **NOT** PCB geometry→R/L/G/C |
| M0 Canonical geometry | [Canonical model verification](m0-canonical-board-model.md), [model contract](../architecture/canonical-board-model.md) | Type/identity/coordinate/integrity invariants; not vendor exporter compatibility or whole-board Open/Short |
| M0 Qt, Worker, IPC | [Qt native evidence](m0-qt-native-runtime.md), [follow-up](m0-qt-runtime-followup.md), [M0 gate](m0-completion-gate.md) | Synthetic Native GUI and Worker; **M0 exit BLOCKED** and redistributability NOT_VERIFIED |
| A-series / Preview | [A_035 scoped handoff](a035-p2z-final-preview-acceptance.md), [Qt Preview guide](../../apps/boardsi-qt-prototype/README.md) | Public synthetic PADS subset/Qt 2D Viewer, no SI product acceptance |
| B-series / Reference | [B_009 bounded audit](b009-b-series-final-acceptance.md), [checklist stage B](../development-checklist.md) | Source-owned candidate reference geometry, not full physical return-path or solver approval |
| C-series / Reference & Qt | [Single checklist stage C](../development-checklist.md), [development history](../development-history.md) | Restricted Copper/Gap/Layer/Qt risk evidence, not real PCB electrical acceptance |
| D-series / local Copper | [D_039 contact foundation](d039-same-layer-copper-connectivity-qualification.md), [D_054–D_055 generalized contacts](d054-d055-general-trace-contact-qualification.md) | Any-angle Trace/arc; off-centre Via annulus/SMD pad overlap candidates, **not** canonical full-board electrical graph |
| M2 numerical and synthetic fields | [Physical RLGC independent verification plan](m2-rlgc-independent-verification-plan.md) and PH-03/05/06/07 sections below | Ideal mathematically bounded PEC/lossless coupons; not 30 GHz measured PCB or qualified losses |
| PH-08 historical original HMM | [Stage 5B original replay](m2-g2-ph08-stage5b-original-replay-evidence.md) and PH-08 section below | **Gate Q FAIL** retained; no physical go/no-go promotion |
| Formal product acceptance | [U01–U48/T01–T54 checklist](../development-checklist.md) and [v1.0 specification](../specification/BoardSI_Development_Spec_v1.0.md) | A requirement/test number, fixture, or prototype gate is **not** a completed official product acceptance test |

## Evidence label rules

| Label | Meaning | Required documentation |
| :--- | :--- | :--- |
| **PASS (scoped)** | Precisely stated cases and checks passed on specified code/source and host | Date, fixture provenance, exact SHA, OS/toolchain, expected/observed evidence, excluded inputs |
| **PARTIAL** | Supported subset only | What is implemented *and* what fails closed/not accepted |
| **FAIL** | The acceptance condition did **not** pass | Preserve failing evidence and causal uncertainty; no reclassification from newer unrelated tests |
| **BLOCKED** | Required dependency/approval is missing | Named missing gate and how to verify it later |
| **NOT_RUN** | No executed test result | Never convert into PASS by writing a specification or synthetic test |
| **NOT_VERIFIED** | Insufficient evidence for claimed product scope | Do not equate an algorithmic candidate with physical electrical contact |
| **PROPOSED** | Design or reference method, not frozen/accepted | State who/what must approve, and separate from executed cases |

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
- [Stage 5B private original HMM replay runbook](m2-g2-ph08-stage5b-private-replay-runbook.md): SHA-pinned executable source runs only on an approved isolated private host, never in public GitHub CI; it reproduces the historical FAIL rather than physics approval.
- [Stage 5B archive preflight / original replay boundary](m2-g2-ph08-stage5b-original-replay-evidence.md): previous private archive digest record, GitHub public synthetic integrity tests only. **Original Stage 5B privately re-executed after initial preflight: historical FAIL reproduced with 12/12 rows and 1761/1761 deterministic numerical scalars; public CI remains synthetic only.**

Stage 5B archived and newly reproduced Gate Q remain FAIL; AC-P1 PENDING, PH-08 contract NOT FROZEN, Rust FEM NO-GO, Gate M NOT VERIFIED, physical bound BOUND_NOT_CERTIFIED. Synthetic GitHub CI does not change these statuses.
