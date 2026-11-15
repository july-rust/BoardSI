# E_029 — Native Qt first PreciseTimer callback A/B experiment

**Scope:** Isolate a single falsifiable hypothesis for Issue [#46](https://github.com/july-rust/BoardSI/issues/46): does showing the first native QWidget prior to the first precise-timer callback correlate with the observed intermittent macOS ARM64 startup gap?

**Status:** **NATIVE EXPERIMENT QUALIFIED (2026-11-15 KST)** — final code-PR SHA, three native platforms, raw receipts, independent Codex and protected Squash confirmed. **Exact code-main event=push 8/8 remains NOT_VERIFIED by the available read interface**, so final operational closure is conditional; #46/EB-02 root cause remains OPEN / NOT_VERIFIED. This document must be reconciled with exact-head CI, PR review, protected Squash and exact merged-main Push before any E_029 completion check.

## Historical facts (immutable)

- macOS 1,134 ms and 1,331 ms **FAIL** under then-current 1,000 ms M0 gate.
- macOS 2,930 ms **FAIL** under explicitly approved provisional 1,500 ms M0 gate.
- The existing 10 ms `Qt::PreciseTimer`, all true M0 worker/IPC/UI criteria, and [provisional 1,500 ms policy](m0-f8-provisional-1500ms-policy.md) are untouched.
- Earlier [cold-start diagnostics](m0-qt-issue46-cold-start-diagnostics.md) and [dispatcher instrumentation](c02509-qt-global-timer-stabilization.md) show a first PreciseTimer callback occurring later than a posted zero-delay callback, but cannot separate Cocoa startup, dispatcher queuing, runner scheduling, or other causes.

## Deliberately narrow experiment

- New test-only C++ executable: `apps/boardsi-qt-prototype/tests/e029_timer_ab.cpp`.
- A/`control`: QApplication plus global PreciseTimer, no QWidget.
- B/`window`: identical executable, Qt and timer; create/show a native QWidget synchronously before first timer dispatch.
- One process per A or B; eight process sequence **A-B-B-A / A-B-B-A** on the *same* native platform/runner and final SHA.
- Three real native Qt platforms: Windows x64, Windows ARM64, macOS ARM64 in existing `.github/workflows/qt-runtime.yml` shared steps. The Windows variants are comparison platforms, not macOS physical-device evidence.
- All eight native JSON results are independently checked for schema, exact source SHA, platform, run order, timing invariants, window/control distinction, and truly observed callbacks by `.github/scripts/e029_analyze_timer_ab.py`. Raw files and derived summary are preserved as an Actions artifact even when the diagnostic step fails.

### Interpretation constraints

- `first_precise_gap_ms` measures callback delivery between arming and first callback, **not proven UI-thread blockage**.
- `zero_to_precise_ms` may be negative if the first PreciseTimer callback fires before the posted zero callback; this is not a fabricated failure.
- `dispatcher_max_wait_ms` includes operating system scheduling, and incomplete nested loop attribution cannot prove a kernel or timer bug.
- A/B correlation is **not causation**. Cold-host independence, thread CPU attribution, representative customer hardware, and definitive root cause all remain **NOT_VERIFIED**.
- Diagnostic `pass` validates instrument/receipt integrity only. It never evaluates the official M0 1,500 ms gate (`global_1500ms_acceptance=NOT_EVALUATED`).
- An intermittently green baseline does not erase historical failed runs or close #46.

## Reproducibility

Build with the standard Qt project and run one real native experiment:

```bash
cmake -S apps/boardsi-qt-prototype -B build-qt -DCMAKE_PREFIX_PATH="$QT_PREFIX" -DBOARDSI_WORKER="$BOARDSI_WORKER"
cmake --build build-qt --target boardsi-e029-timer-ab
BOARDSI_EXPECTED_QT_PLATFORM=cocoa BOARDSI_E029_HEAD_SHA="<40-char commit SHA>" \
  ./build-qt/boardsi-e029-timer-ab --mode control build-qt/e029-ab-01-control.json
```

The existing CI performs the entire eight-process ordered experiment automatically and runs the analyzer. Do **not** substitute `QT_QPA_PLATFORM=offscreen` for native platform qualification.

## Actual independent native evidence — 2026-11-15 KST

**Exact source-HEAD:** `7e3faee0cc3895322ad61ae0780f4d5b006eb2e8`; [Code PR #643](https://github.com/july-rust/BoardSI/pull/643), all **10/10 final-HEAD PR workflows SUCCESS** ([Qt Runtime #43660254165](https://github.com/july-rust/BoardSI/actions/runs/43660254165), all 5 Qt jobs PASS including Linux). Codex latest exact-HEAD review **COMPLETED / 0 findings / 0 unresolved**, after three earlier P2 findings corrected in follow-up commits and their three threads resolved. Protected **Squash** succeeded: [code-main `7bd8b8bffc1d8d9a8be4db30478800a83a7d2fe1`](https://github.com/july-rust/BoardSI/commit/7bd8b8bffc1d8d9a8be4db30478800a83a7d2fe1). Merge protection and one resulting SHA observed; *cryptographic Verified signature bit and exact post-merge push workflow outcome have not been independently read*, and must not be claimed without the corresponding API/raw evidence.

| Platform, Qt 6.8.3 native | A control first gap samples (ms) | B native QWidget-show first gap samples (ms) | A/B median (ms) | B–A (ms) | Original **unchanged** M0 whole-suite max gap (ms) | Native result |
| --- | --- | --- | --- | ---: | ---: | --- |
| macOS ARM64 Sequoia 15.8 / cocoa | 88, 88, 86, 86 | 66, 55, 54, 60 | 87 / 57.5 | −29.5 | **617** | PASS, [artifact #14140241723](https://github.com/july-rust/BoardSI/actions/runs/43660254165/artifacts/14140241723) |
| Windows x64 Server 2022 / windows | 0, 0, 7, 0 | 32, 32, 32, 30 | 0 / 32 | +32 | **62** | PASS, [artifact #14139689670](https://github.com/july-rust/BoardSI/actions/runs/43660254165/artifacts/14139689670) |
| Windows ARM64 Windows 11 25H2 / windows | 9, 6, 0, 5 | 17, 18, 16, 18 | 5.5 / 17.5 | +12 | **85** | PASS, [artifact #14140407044](https://github.com/july-rust/BoardSI/actions/runs/43660254165/artifacts/14140407044) |

Each native platform executed exactly **A-B-B-A / A-B-B-A: eight independent process invocations**, with native backend assertion, immutable source SHA in receipts, strict JSON ordering and source proof. An initial callback gap of 0 ms is the *integer millisecond* resolution of the monotonic clock, not zero processing time. B show completion was independently checked to precede the first precise callback. Artifact files were uploaded successfully; raw per-process data and analyzed summary are linked above. The Python summary's `pass` means receipt integrity, **not the M0 1500 ms gate**.

**Facts:** No historical intermittent >1,500 ms macOS outlier reproduced in these eight short independent experiments; the separate full original Qt Runtime macOS job observed **617 ms** and passed the unchanged provisional 1,500 ms rule. B was **faster** than A on the tested macOS runner but **slower** on both Windows runners. These values are real *single-runner observations*, not inter-platform controlled cause-isolation proof.

**Not supported:** Native window display alone as the unique cause, OS or Qt timer defect, thread CPU starvation diagnosis, independent cold-host/run repetition, representative customer macOS hardware, general release stability, recovery of earlier 1,134/1,331/2,930 ms historical FAILs, or source-level remediation. All remain **NOT_VERIFIED**; #46 and EB-02 stay open. No production GUI/Worker, timer rule or physics acceptance was changed.

**Distinct post-merge limitation:** The available GitHub integration can fetch PR-head workflows and native logs but does not expose the complete `event=push` run list for a newly merged SHA. The exact code-main `7bd8b8bf...` post-merge workflow 8/8 status is **NOT_VERIFIED** here. Do not use 10/10 PR-head workflows as substitute push proof. Reconcile only against independently captured Push Run IDs and exact SHA when available.

**Closeout contract:** The E_029 *experiment* may be COMPLETED once its scoped code and native evidence, review, protected merge and exact merged-main verification are all finished. Issue #46 and EB-02 stay OPEN/NOT_VERIFIED unless a causal remedy is separately demonstrated with representative regression. E_029 is neither a release certification nor a physical SI acceptance.

**Related:** [E-stage plan](../e-stage-closure-plan.md), [blocked register](../e-stage-blocked-register.md), [Issue #46](https://github.com/july-rust/BoardSI/issues/46).
