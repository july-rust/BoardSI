# C_025_08 — M0 Rust/Qt/IPC architecture baseline and Issue #3 acceptance audit

**Audit date:** 2026-10-31 KST. **Tracker:** [#325](https://github.com/july-rust/BoardSI/issues/325). **Target:** [M0 Issue #3](https://github.com/july-rust/BoardSI/issues/3). **Inspected baseline:** protected `main@f207bec1cfd7519e2eab382b578d5289c66ec8e6` ([C_025_07 squash PR #324](https://github.com/july-rust/BoardSI/pull/324)). **Change type:** bounded architecture/evidence decision only; no new Qt/Rust executable, native fixture, installer or physical validation. A proposed PR cannot certify its own review, CI or merge.

## 1. Current engineering decision (accepted for the M0 prototype, NOT for redistributable product)

| Boundary | Decision | Evidence / limits |
| --- | --- | --- |
| Desktop GUI | **Qt Widgets with C++17** as host/application shell | [CMake](../../apps/boardsi-qt-prototype/CMakeLists.txt) requires Qt6 Core/Gui/Widgets >= 6.5; Qt 6.8.3 was actually built in [native runtime workflow](../../.github/workflows/qt-runtime.yml). This is not all-OS installer verification. |
| Computation/process | **Separate Rust Worker executable** behind **Qt QProcess** asynchronous process signals; retain crash/exit/cancel isolation | [Worker/IPC ADR](../architecture/worker-ipc-qt-prototype.md), [native runtime evidence](m0-qt-runtime-followup.md), [M0 F4 closeout](m0-completion-gate.md). The full production PCB snapshot is not qualified. |
| Transport | Existing **versioned, size-bounded, length-prefixed UTF-8 JSON** local stdin/stdout IPC; scopes/run identities and monotonic message sequencing checked. Keep stdout for frames and stderr for diagnostics | [IPC v1 contract](../architecture/worker-ipc-qt-prototype.md). F4 restricted model-bound v2 snapshot-to-Worker receipt [PR #102](https://github.com/july-rust/BoardSI/pull/102); this is not a claim that IPC v1 serializes all Canonical PCB geometry. |
| Artwork acceleration | **Qt Quick/RHI optional** for `boardsi-preview-p0` when its Qt module is present; OpenGLWidgets optional; CPU fallback retained | [Current CMake](../../apps/boardsi-qt-prototype/CMakeLists.txt). RHI availability is not equivalent to real hardware Metal/Direct3D benchmarking or commercial release proof; [#165](https://github.com/july-rust/BoardSI/issues/165) remains OPEN. |
| Rust ↔ Qt binding | **No mandatory in-process Rust/Qt FFI/CXX-Qt**. Communicate by independent process/IPC; revisit only on measured need | [Original alternatives](../architecture/worker-ipc-qt-prototype.md), [M0 exit gate decision](m0-completion-gate.md). This is an engineering selection, not a universal performance comparison. |
| Canonical board data | Rust owns Canonical Board Model and synthetic immutable snapshot. No large cross-process shared mutable model | [Model baseline PR #28](https://github.com/july-rust/BoardSI/pull/28), [F4 PR #102](https://github.com/july-rust/BoardSI/pull/102). Geometry-to-physical SI remains unvalidated. |

**Alternative disposition:** Qt Quick as the *entire* desktop shell and synchronous/in-process Qt↔Rust binding are **not adopted for M0**; no exhaustive comparative performance proof is asserted. A later rendering acceleration choice must not silently alter the already proven Worker process isolation. Do not change these interfaces under this documentation-only task.

## 2. Toolchain pins versus deployment qualification

| Item | Observed source/CI version or requirement | Current conclusion |
| --- | --- | --- |
| Rust Worker | Rust **1.75.0** and `cargo --locked` in [Qt runtime native CI](../../.github/workflows/qt-runtime.yml); actual Cargo.lock governs resolved dependencies | **PASS for tested synthetic executable**, not a complete bundled dependency/license SBOM |
| C++/Qt | **C++17**; CMake `find_package(Qt6 6.5 ... Core Gui Widgets)`; **Qt 6.8.3** used by CI | **PASS for source/native prototype**; minimum version and executed version are distinct |
| CMake | CMakeLists `>=3.21`, Linux workflow installs **3.31.6**; native OS CMake version is printed at runtime | **Provisional**: do not assert all native OS compiler minor/SDK versions are frozen for distribution |
| Qt installation | Native workflow `aqtinstall==3.3.0`; Windows x64, Windows ARM64 and macOS ARM64 matrices | Actual CI runner build/exec is **not** a Qt-SDK-free clean-host test |
| Linux | CI Linux x86_64 offscreen and xcb/Xvfb prototype | Additional developer/test environment; not a promised commercial target platform |

**F7 is BLOCKED:** identify every actually shipped Qt/Quick/OpenGL/platform-plugin and third-party license and verify its real linkage per build. Qt's [official licensing](https://doc.qt.io/qt-6.8/licensing.html), [LGPL conditions](https://www.qt.io/development/open-source-lgpl-obligations) and [third-party terms](https://doc.qt.io/qt-6.8/licenses-used-in-qt.html) are the normative starting points, **not a legal clearance**. Dynamic linking, ability to replace/relink libraries, license texts/notices and source obligations are *proposed compliance controls*, not independently demonstrated for a Windows/macOS binary bundle. Need clean host tests without SDK, runtime plugin/dll/dylib load proof, license manifest and signed distribution policy. The historical F6/F7 evidence ledger is [here](m0-f6-f7-qualification-evidence.md).

## 3. Three-target prototype evidence and associated spec trace

[PR #30](https://github.com/july-rust/BoardSI/pull/30) and [Qt native test report](m0-qt-runtime-followup.md) document actual Qt 6.8.3 / Rust Worker prototype execution on **Windows x64**, **Windows ARM64**, **macOS ARM64** (plus Linux x86_64); [PR #102](https://github.com/july-rust/BoardSI/pull/102) supplies restricted synthetic model/snapshot handoff. Historical [final PR Qt CI run 38567925575](https://github.com/july-rust/BoardSI/actions/runs/38567925575) is evidence of its historical SHA only. This C_025_08 audit adds **no independent native execution**; require exact new PR HEAD CI before integration, and distinguish CI regression coverage from formal case acceptance.

| Requirement or test | Evidence-bounded judgment | Full U/T acceptance |
| --- | --- | --- |
| U02 Rust core + Qt UI | **PARTIAL**: real separate Rust/Qt prototype; physical PCB Solver missing | **NOT VERIFIED** |
| U03 Windows x64/ARM64 and macOS Apple Silicon | **PARTIAL**: native synthetic Qt/Worker CI on all three | **NOT VERIFIED** for clean installed product |
| U23 modular UI/Solver separation | **PARTIAL**: process isolation, QProcess/Worker event handling | **NOT VERIFIED** for full heavy PCB run |
| U44 immutable snapshot/IPC/late response isolation | **PARTIAL**: F4 synthetic v2 model-snapshot/Worker, stale identity checks | **NOT VERIFIED** for full production run/recovery |
| T13 Native and Unicode/locale paths | **PARTIAL** native prototype; Unicode/decimal path case unqualified | **NOT RUN** as complete T13 |
| T39 3-platform same-PCB Import/Point/Port/Complex S | **PARTIAL** platform execution only; not the specified same-PCB comparison | **NOT RUN** as complete T39 |
| T46 late IPC, duplicate, edit-during-run | **PARTIAL** synthetic scope/late response gates; no product immutable snapshot and persistent cache test | **NOT RUN** as complete T46 |

This is consistent with [C_025_05 case ledger](c02505-m0-f1-traceability-reconciliation.md): the pinned historical 10-case record had **8 PARTIAL / 2 NOT_RUN** and **no fully accepted formal T case**. Do not edit the frozen case JSON to make a higher pass count.

## 4. Issue #3 and M0 gate decision

**Engineering decision for this limited prototype: RECORDED. Issue #3: OPEN / BLOCKED for full DoD.** Prior [C_025_07](c02507-open-issue-d-stage-handoff.md) explicitly found licensed redistributable toolchain/deployment choice unproven; this ADR does not independently prove it. Formal T13/T39/T46 product case acceptance, fully qualified SDK-free three-target packaging and verified Qt/third-party compliance are **not** turned into PASS by writing a decision. If issue owner determines #3 should cover *only* a technology selection, a future scope amendment must explicitly transfer each unfinished requirement to #6/#7/F7 with traceable evidence before any safe issue closure. Do not change its DoD retroactively or close it merely to reduce the issue count.

**M0 total: BLOCKED on F1/F6/F7.** [#2](https://github.com/july-rust/BoardSI/issues/2), [#6](https://github.com/july-rust/BoardSI/issues/6), [#7](https://github.com/july-rust/BoardSI/issues/7), [#46](https://github.com/july-rust/BoardSI/issues/46), [#165](https://github.com/july-rust/BoardSI/issues/165) remain independent. Qt historical >1 s macOS cold-gap root cause still unverified; only provisional M0 whole-test **1,500 ms** policy applies, not a general product acceptance. Draft [PR #35](https://github.com/july-rust/BoardSI/pull/35) stays open/unmerged. No P009 return path, actual PCB SI, 30 GHz, native installer or GPU performance certificate is claimed.

## 5. Scope acceptance and final-PR gate (MUST be filled from actual evidence, not self-certified)

- [x] Pre-change direct GitHub Issue #3 OPEN and base SHA were read; architecture, CMake and native workflow contents inspected **2026-10-31 KST**.
- [x] Prototype decision and partial/full acceptance separation documented; **no new executable or T-case implementation**.
- [ ] Exact *final PR HEAD* 17/17 strict required contexts PASS and actually observed native jobs.
- [ ] **Independent latest exact-HEAD Codex review** completed; zero unresolved substantive findings/threads.
- [ ] Signed **Verified**, one-parent **protected Squash Merge**, Ruleset 25046595 intact.
- [ ] Resulting exact `main` SHA post-merge PUSH workflows and required checks SUCCESS.
- [ ] Final Issue #325 audit closeout comment cites actual immutable evidence. Close #325 only after documented audit delivery passes; Issue #3 stays open while its DoD is not met.

An audit proposal, green docs CI or an internally drafted review does **not** substitute for native execution or independent review. If any gate cannot be read or met, record **BLOCKED** rather than claim delivery complete.
