# BoardSI — verified M2 development history

This is a phase-scoped engineering record, not an assertion of overall M2 completion. Detailed evidence and raw references are linked. A PH-case numerical PASS does not imply actual PCB physical validity.

| Date (KST) | Change | Evidence / scope |
| --- | --- | --- |
| 2026-10-15 | G0 independent acceptance and numerical baseline approved (#33) | [G0 reference](validation/m2-g0-reference-evidence.md); mathematical plate anchor frozen but FEM PH-03 still Not Run |
| 2026-10-15 | G1 restricted PEC-box Microstrip FEM (#34) | [G1 evidence](validation/m2-g1-lossless-fem-evidence.md), main `7f87b8b3c1f192ca310811cf07b4efe11ab0f47c`; full PCB physical accuracy NOT validated |
| 2026-10-15 | PH-03 pre-run acceptance contract frozen, then separate Neumann plate FEM + independent mathematical validation in PR #37 | Contract first `0769ede0e40861d6b587b910f85b2d9d18073276`; source test run [38663357466](https://github.com/july-rust/BoardSI/actions/runs/38663357466) PASS, 3 mesh levels n8/16/32, original G1/Clippy/M0 Rust regression PASS. [PH-03 detailed results](validation/m2-g1-lossless-fem-evidence.md). This is **mathematical coupon only**, not finite-width copper/30 GHz validity. |

## Future required development sequence

**G0 complete → G1 limited prototype complete → G1 PH-03 (mathematical) → G2 PH-05 symmetric stripline → PH-06 asymmetric → PH-07 multilayer/mask → PH-08 finite reference topology.** Continue only with new independently frozen oracle/acceptance. G3 losses, G4 differential, G5 Canonical/Worker integration and physical validation remain future separate gates. The G5-only design [PR #35](https://github.com/july-rust/BoardSI/pull/35) must not be included in this implementation. [Issue #16](https://github.com/july-rust/BoardSI/issues/16) remains Open.
