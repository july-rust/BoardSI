# M0 F1 — executable scoped native case verification

**Scope: M0 technology prototype only. Full product T acceptance NOT_RUN; formal F1 PARTIAL and M0 BLOCKED until independent case-specific acceptance is recorded.**

This additive gate does not rewrite the immutable `docs/validation/m0-case-evidence.json` v1 baseline (historical source `2a08c167...`) or its fail-closed SHA pin. It verifies current *actual* Qt Widgets 6.8.3 and separately launched Rust Worker testcase output on **Windows x64, Windows ARM64, and macOS ARM64**, checks the real platform and CPU architecture, and publishes a reproducible per-platform JSON proof linked to that workflow run. It is deliberately not an approval of the v1.0 full product tests.

## Exact executed scope

| ID | M0-only executable contract | Full test explicitly outside this proof |
| --- | --- | --- |
| T10 | Real Rust Worker normal, five rapid cancel/restart, crash, CPU-load cancel; measured cancellation deadline | Real PCB solver memory and persistent recovery |
| T13 | Real native GUI/Worker and observed Qt backend/CPU target | Unicode filenames/locale across actual importer |
| T32 | Crash/unresponsive/cancel-hang/OS-exit/close and heartbeat-negative scenarios | Pause/resume and durable checkpoints |
| T34 | Exactly 150 queued Qt widget updates per normal/burst/CPU-stress scenario, p95/max/timer deadlines | Artwork Viewer real import and pan/zoom |
| T46 | Scope/snapshot/duplicate/reverse/malformed IPC and illegal completion-race refusal | Full production Run Snapshot and cache |
| T50 | M0-only global 1,500 ms timer, unchanged p95/max, heartbeat, cancel and update deadlines | Full product responsiveness / physical SI workloads |

T11 optional GPU equivalence, T35 actual RAM/SSD exhaustion and spill, T39 common real PCB/S response and T54 checkpoint rollback **remain NOT_RUN as full tests**. Their later-phase full features must not be smuggled into the minimum synthetic M0 acceptance. Any applicable *M0-only* subcase still needs its own reviewer signoff and evidence.

The source `.github/scripts/m0_f1_native_subcases.py` rejects missing/duplicate/failing native scenario records, missing measurements, false or NaN numeric values, incorrect Qt version, wrong Windows/ARM64/macOS architecture and any global timer exceeding 1,500 ms. UI p95 (200 ms), max (1,000 ms), per-scenario timer (1,000 ms), heartbeat (800–1,500 ms), update (1,000 ms) and cancel ack (1,000 ms) limits remain unchanged. `test_m0_f1_native_subcases.py` injects invalid/falsified evidence and the historical 2,930 ms gap, requiring a FAIL. The signed main branch's prior failures stay historical failures.

Each native Qt job calls the verifier **after the real native Qt runtime harness and real model-to-Worker selftest**, then uploads `m0-f1-native-subcases-<target>` JSON containing executed case names, numerical observations, attempted source SHA and actual checkout SHA, workflow-run ID and input SHA-256. Reports always record `independent_case_specific_acceptance_review=NOT_PERFORMED`, `full_product_T_acceptance=NOT_RUN`, `formal_F1=PARTIAL`, `formal_M0=BLOCKED`; a failed verifier fails that native CI job and does not produce a false PASS.

## Remaining original M0 exit conditions

F1 requires independent M0-specific individual reviewer acceptance, including exact native GitHub run/job, committed source and fixture, expected/observed values. F6 real/equivalent reference PC baseline and board redistribution rights/notices remain separate. F7 native clean-environment installation and exact Qt/third-party license/relink/legal approval remain separate. No paid cloud hardware, private PCB, Draft PR #35, M1 or PH-08 work is authorized by this gate.
