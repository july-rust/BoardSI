# E_029 — Native Qt first PreciseTimer callback A/B experiment

**Scope:** Isolate a single falsifiable hypothesis for Issue [#46](https://github.com/july-rust/BoardSI/issues/46): does showing the first native QWidget prior to the first precise-timer callback correlate with the observed intermittent macOS ARM64 startup gap?

**Status:** IMPLEMENTED / NATIVE PR TESTS AND INDEPENDENT REVIEW PENDING. This document must be reconciled with exact-head CI, PR review, protected Squash and exact merged-main Push before any E_029 completion check.

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

## Evidence reconciliation (complete only after real runs)

| Gate | Evidence | Status |
| --- | --- | --- |
| Final PR HEAD / eight A/B results | Linked exact-source CI and Actions artifact | NOT_VERIFIED |
| Mac native Cocoa startup gap | Eight actual per-process macOS JSON files | NOT_VERIFIED |
| Windows x64/ARM64 comparison | Eight actual per-platform JSON files | NOT_VERIFIED |
| Existing Qt/Worker full suite | Unchanged native jobs | NOT_VERIFIED |
| Independent exact-head code review | Review SHA, findings, unresolved threads | NOT_VERIFIED |
| Strict/Verified protected Squash | Signed one-parent merge SHA | NOT_VERIFIED |
| Separate post-merge push check | Exact merged-main SHA/run/artifacts | NOT_VERIFIED |

**Closeout contract:** The E_029 *experiment* may be COMPLETED once its scoped code and native evidence, review, protected merge and exact merged-main verification are all finished. Issue #46 and EB-02 stay OPEN/NOT_VERIFIED unless a causal remedy is separately demonstrated with representative regression. E_029 is neither a release certification nor a physical SI acceptance.

**Related:** [E-stage plan](../e-stage-closure-plan.md), [blocked register](../e-stage-blocked-register.md), [Issue #46](https://github.com/july-rust/BoardSI/issues/46).
