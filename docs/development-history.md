# BoardSI — verified M2 development history

This is a phase-scoped engineering record, not an assertion of overall M2 completion. Detailed evidence and raw references are linked. A PH-case numerical PASS does not imply actual PCB physical validity.

| Date (KST) | Change | Evidence / scope |
| --- | --- | --- |
| 2026-10-15 | G0 independent acceptance and numerical baseline approved (#33) | [G0 reference](validation/m2-g0-reference-evidence.md); mathematical plate anchor frozen but FEM PH-03 still Not Run |
| 2026-10-15 | G1 restricted PEC-box Microstrip FEM (#34) | [G1 evidence](validation/m2-g1-lossless-fem-evidence.md), main `7f87b8b3c1f192ca310811cf07b4efe11ab0f47c`; full PCB physical accuracy NOT validated |
| 2026-10-15 | PH-03 pre-run acceptance contract frozen, then separate Neumann plate FEM + independent mathematical validation in PR #37 | Contract first `0769ede0e40861d6b587b910f85b2d9d18073276`; source test run [38663357466](https://github.com/july-rust/BoardSI/actions/runs/38663357466) PASS, 3 mesh levels n8/16/32, original G1/Clippy/M0 Rust regression PASS. [PH-03 detailed results](validation/m2-g1-lossless-fem-evidence.md). This is **mathematical coupon only**, not finite-width copper/30 GHz validity. |

| 2026-10-15 | G2 PH-05 symmetric lossless PEC stripline FEM and independent Python FVM (PR #38) | Frozen contract in commit `572c1915`, independent 81/81 checks PASS on [CI 38666133102](https://github.com/july-rust/BoardSI/actions/runs/38666133102), Rust unit/integration, Clippy and M0 regression PASS. [G2 FEM evidence](validation/m2-g2-ph05-fem-evidence.md), [FVM source/fixture hashes and reference](validation/m2-g2-ph05-independent-reference.md). Limited homogeneous coupon only, **not** complete M2/real-board validation. |

| 2026-10-15 | G2 PH-06 asymmetric homogeneous/two-region lossless PEC stripline FEM and independent cell-centred Python FVM (PR #39) | [Frozen PH-06 contract](validation/m2-g2-ph06-acceptance-contract.md) committed **before implementation** at `15da48ef`; [run 38669970099](https://github.com/july-rust/BoardSI/actions/runs/38669970099) 201/201 numerical and 57/57 geometry/reflection invariant gates PASS. [Independent numerical reference](validation/m2-g2-ph06-independent-reference.md), [measured FEM evidence](validation/m2-g2-ph06-fem-evidence.md). Limited synthetic quasi-TEM only; full M2, loss/30 GHz and physical board validation remain pending. |

| 2026-10-15 | G2 PH-07 multilayer/air/mask microstrip lossless ideal-PEC FEM and independent cell-centred FVM (PR #40) | Contract first `c11440a1`; initial [CI 38674095933](https://github.com/july-rust/BoardSI/actions/runs/38674095933) 740/740 independent gates, PH-05/PH-06 regressions PASS. [Reference](validation/m2-g2-ph07-independent-reference.md), [FEM evidence](validation/m2-g2-ph07-fem-evidence.md). Synthetic only; final HEAD/merge must be verified, #16 stays Open. |

## Future required development sequence

**G0 complete → G1 limited prototype complete → G1 PH-03 mathematical PASS → G2 PH-05 restricted class-F PASS → PH-06 restricted asymmetric/two-region class-F PASS → PH-07 restricted class-F PASS → PH-08 finite reference topology.** Continue only with new independently frozen oracle/acceptance. G3 losses, G4 differential, G5 Canonical/Worker integration and physical validation remain future separate gates. The G5-only design [PR #35](https://github.com/july-rust/BoardSI/pull/35) must not be included in this implementation. [Issue #16](https://github.com/july-rust/BoardSI/issues/16) remains Open.
