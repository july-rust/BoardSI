# E_003 — Rust/Qt/IPC v1 wire-compatibility qualification

**Audit date:** 2026-11-11 KST. **Source baseline:** protected main SHA 769fb58d043df247bee68a95bc769e43f4d1c05a. **Tracking:** [Issue #3](https://github.com/july-rust/BoardSI/issues/3); [PR #585](https://github.com/july-rust/BoardSI/pull/585).

## Architecture decision and current source inspection

| Contract | Source | Bounded conclusion |
| --- | --- | --- |
| Desktop host | [CMake](../../apps/boardsi-qt-prototype/CMakeLists.txt), [C_025_08](c02508-m0-architecture-issue3-acceptance.md) | C++17 / Qt Core, Gui, Widgets. CMake *minimum* Qt 6.5 versus Qt *executed CI* 6.8.3; not an approved product compatibility range. |
| Worker boundary | [Qt host](../../apps/boardsi-qt-prototype/main.cpp), [Worker](../../apps/boardsi-worker/src/lib.rs) | Separate Rust executable, Qt QProcess asynchronous process supervision, local inherited stdin/stdout, no mandatory in-process Rust/Qt FFI. |
| IPC | [boardsi-ipc](../../crates/boardsi-ipc/src/lib.rs) | Protocol v1; big-endian u32 length; UTF-8 JSON payload length 1–65536; strict schema, scoped run/snapshot and contiguous sequence. Unknown versions/invalid data rejected, no silent downgrade. |
| Snapshot | [C_025_08](c02508-m0-architecture-issue3-acceptance.md) | IPC v1 synthetic snapshot and restricted opt-in model snapshot v2 are separate; full Canonical PCB transport not claimed. |
| Artwork | [CMake](../../apps/boardsi-qt-prototype/CMakeLists.txt) | Optional Qt Quick/RHI and OpenGLWidgets, CPU fallback; not physical GPU benchmarking. |
| Toolchain | [Native workflow](../../.github/workflows/qt-runtime.yml), CMake, Cargo.lock | Rust 1.75.0 locked Cargo; Qt SDK 6.8.3 / aqtinstall 3.3.0; CMake >=3.21 (Linux CI 3.31.6); C++17. Complete deployed compiler/SDK/license manifest NOT VERIFIED. |

The C_025_08 architecture choice was already recorded for a **synthetic M0 prototype**, not licensed redistribution or full Issue #3 acceptance. This task changes **tests, not IPC protocol or software architecture**.

## Added adversarial tests

- [Rust wire compatibility tests](../../crates/boardsi-ipc/tests/e003_wire_compat.rs): reject unsupported v2, unknown envelope/body fields, kind mismatch, invalid UTF-8, partial headers/bodies, zero-length/oversized frames; accept unchanged valid v1 bytes. Rejected versions/scopes/gaps must not move the accepted sequence cursor.
- [Qt test peer](../../apps/boardsi-qt-prototype/tests/fault_worker.py) and [QApplication harness](../../apps/boardsi-qt-prototype/tests/runtime.cpp): exercise actual QProcess/Qt parser against UTF-8 invalidity, short header, zero length, wrong kind, unknown field. Preserve all existing normal Rust process/cancel/crash/partial-message scenarios.
- The [Rust IPC decoder](../../crates/boardsi-ipc/src/lib.rs) retains direct strict Serde decoding (including duplicate envelope-key rejection), then rejects extra members on internally tagged unit messages before their acceptance. This is a fail-closed correction without an IPC v1 wire-format change.
- The fault peer is a public synthetic test tool and does not prove real PCB SI or formal product acceptance.

## Verification and exit boundary

**Evidence must not be pre-asserted.** The final Issue #3 comment shall record actual PR #585 exact final HEAD, relevant Rust/Qt/Native checks, independent review and unresolved threads, Verified Squash SHA, and precise new main Push CI. Passing a code-free docs check is not equivalent to executing the tests.

Existing historical native Qt/Worker synthetic M0 evidence is **PARTIAL**. Formal complete T13/T39/T46 are **NOT_RUN**. F1/F6/F7, Qt LGPL/third-party clean-host distribution review, product installer, actual PCB import/ports/complex S-parameters, GPU hardware #165, Qt timer #46, original PH-08 Gate Q FAIL and complete product U/T acceptance remain unchanged.

**Issue #3 remains OPEN** unless its original full DoD is met; E_003 is permitted to close only as a bounded verification after its own review/CI/merge gates. Outstanding clean-host/license eligibility stays attached to #6/#7 and the [E blocked register](../e-stage-blocked-register.md).
