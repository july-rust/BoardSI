# BoardSI architecture — current prototypes and open product choices

**2026-10-18 KST evidence checkpoint:** [complete-project audit](../audits/2026-10-18-pre-ph08-project-audit.md). Version 1.0 specification, frozen physics contracts and numerical thresholds remain unchanged.

## Built and scope-tested (not full product)

- M0 Rust scalar uniform RLGC/ABCD/S numerical engine: independent reference tests, [numerical contract](numerical-contract.md).
- M0 canonical board geometry/provenance typed Rust model: synthetic integrity/validation, [canonical model](canonical-board-model.md). Neither PADS ASCII nor ODB++ real import adapter is production-qualified.
- M0 asynchronous **Qt Widgets + QProcess** to isolated Rust Worker with versioned local framed IPC: native prototype tests, [IPC contract](worker-ipc-qt-prototype.md) and [Qt evidence](qt-native-runtime.md). This does **not** include complete layout viewer, crash-safe cache, production release or installer.
- M2 research G0/G1 and PH-03/05/06/07: independent restricted **PEC/lossless** 2D FEM validation only, [validation index](../validation/README.md).

## Still open

Qt Widgets is the implemented **prototype choice**; final Widgets/Quick UX/render architecture, Qt SDK distribution and licensing are not frozen. C ABI/CXX-Qt was not introduced; the process boundary was selected for M0. Actual PCB adapter/model→physical solver G5, true 30 GHz material and conductor losses G3, coupled lines G4, real artwork/ports, progressive sweep and packaging are unqualified.

Original PH-08 HMM Stage 4 and privately re-executed Stage 5B Gate Q **FAIL**; AC-P1 PENDING, T/R PROPOSED_NOT_FROZEN, contract NOT_FROZEN, Rust FEM NO_GO, Gate M NOT_VERIFIED, physical **BOUND_NOT_CERTIFIED**. [Issue #51](https://github.com/july-rust/BoardSI/issues/51) stays open, G5 [Draft PR #35](https://github.com/july-rust/BoardSI/pull/35) **DO NOT MERGE**. Separate cross-platform Qt intermittent >1000 ms global event-loop gap [#46](https://github.com/july-rust/BoardSI/issues/46) remains OPEN: macOS ARM64 originally observed 1134 ms; a 2026-10-18 Windows ARM64 staging run also measured 1255 ms (1240 ms within the first synchronous window-show interval). The original 1000 ms limit is unchanged. Same-SHA rerun passing does not identify or fix the root cause; ROOT_CAUSE_NOT_CONFIRMED.

Normalize each future CAD adapter to common Canonical Board Model; ODB++ is an input format, not internal representation. Never report a physical PCB Validated status from synthetic-only tests.

## M0 fast closeout gate (2026-10-19)

The [M0 completion gate and blocking evidence matrix](../validation/m0-completion-gate.md) distinguishes the existing native synthetic prototypes from **unresolved** baseline PC, public board fixtures, Qt redistributable-deployment/licensing and intermittent event-loop diagnostics. **Current phase status: BLOCKED**, not product or physical-accuracy acceptance. For M0, Qt Widgets + QProcess IPC v1 is the retained implemented technology choice; production packaging/Qt Quick enhancements and restricted synthetic Snapshot v2 do not expand the original M0 gate.
