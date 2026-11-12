# E_008 — PADS V9.4 MILS format/readiness bounded source qualification

**Date:** 2026-11-12 KST. **Parent:** [Issue #8](https://github.com/july-rust/BoardSI/issues/8). **Scope:** opt-in PADS ASCII V9.4 MILS bounded format/version and **restricted ROUTE→Canonical subset** readiness. This is not product approval.

## 1. Existing implementation and additions

- Existing `probe_pads_reader` reads at most 514 bytes (512-byte first-line limit with CRLF lookahead). The default one-argument CLI still uses this path and keeps its historical stdout/exit codes; a matching family signature alone does not prove a supported revision or CAD import.
- E_008 adds `import_readiness::inspect_pads_readiness` and `boardsi-import-probe FILE --readiness`. The opt-in path bounds the actual file read to **8 MiB + 1 byte**, including after the file metadata check, and passes bytes to the established V9.4 MILS `route_preview::parse` and `into_canonical`. Source bytes are never edited.
- Only four exact supported importer header tokens are recognized as candidates: `!PADS-POWERPCB-V9.4-MILS!`, `!PADS-POWERPCB-V9.4-MILS-CP1252!`, `!PADS-POWERPCB-V9.4-MILS-250L!`, `!PADS-POWERPCB-V9.4-MILS-250L-CP1252!`. Other family/revision/unit or BOM/star variants are classified **FAMILY_ONLY**, not promoted to parser-qualified input.
- Source-only `G/N/P/S/R/M` values are copied from the **actual restricted Canonical adapter's** `ImportCapability` after a successful parser run. Before that, each is **UNKNOWN**; no header-only readiness inference. Once accepted, partial/missing applies only to the existing importer subset, not source-complete PCB.
- A parser-successful pad/outline-only source must also show at least one actual converted Canonical trace/arc/via; otherwise `NO_CANONICAL_ROUTE_GEOMETRY` is REJECTED and all six capabilities remain UNKNOWN. Public synthetic `pads_rf_rectangle_v94.asc` is the negative regression.
- Parser rejection and `skipped` unsupported source route records are exposed by static code, not by echoing private input. Skips cannot become a clean preview qualification. `solver_ready=false` and `physical_si=NOT_RUN` are invariant.

## 2. Verification matrix

| Synthetic case | Expected behavior |
| --- | --- |
| Exact V9.4 MILS restricted route | Source/canonical subset PARTIAL, no SI readiness |
| Recognized other revision/unit/code page | Family-only; parser not run, capability UNKNOWN |
| Schematic ASCII | PCB unsupported; capability UNKNOWN |
| Spoofed/embedded header, LTspice ASC | UNKNOWN; no geometry promotion |
| Missing `*END*` | REJECTED; no claimed capability |
| Oversize 8 MiB input | REJECTED before parse |
| Four accepted parser tokens | Restricted parser qualification, no FULL support |
| UTF-8 BOM only recognized at family level | Parser variant not silently accepted |
| Skipped/unsupported route records | Not clean preview; never SI-ready |
| Hidden net names and repeated runs | No source content leaked; deterministic status |

**Actual execution evidence:** record final exact PR HEAD checks, Rust and applicable native results, independent review and unresolved thread count, Verified protected Squash commit, and exact merged-main Push workflow in [Issue #8 final E_008 closeout comment](https://github.com/july-rust/BoardSI/issues/8). Do not convert planned tests into PASS until CI confirms them.

## 3. Explicit limitations and original gates

- This is not complete PADS vendor-export grammar support. Unsupported Source objects outside the restricted ROUTE subset may remain unmodeled; an accepted subset does **not** prove complete object inventory or safe electrical import.
- Format identity and geometry provenance are distinct. Source SHA is computed only by the existing Canonical route adapter after actual conversion; the readiness output itself does not assert independent authenticity or public redistribution rights.
- No true layer/material/stackup, complete pin/pad/netlist, continuous reference conductor, manufactured plating, global Open/Short or actual SI physics is established.
- T17 (same extension different contents), T20 (unsupported/lost objects), T22 (malformed/oversize safety) have only *focused synthetic subset* regression evidence; all formal T17/T20/T22 acceptance tests remain **NOT_RUN**.
- Full Issue #8 U38-U40/U48 plus format-wide G/N/P/S/R/M, native Qt GUI error-path, real public exporter evidence, and any compressed-archive budget are not closed by this task. Keep #8 **OPEN** until independently qualified original DoD. E_009 handles another PADS source-grammar gap; E_013–E_015 handle ODB++ compressed formats.
- No confidential customer artwork, board files, traces, original net names, paths, licenses, or secrets may be uploaded.

## 4. Closeout gate

**E_008 bounded COMPLETE only after** final-head required CI is SUCCESS, independent review findings are addressed, unresolved threads = 0, protection is not bypassed, the merge commit is GitHub Verified Squash-only, and all required exact merged-main Push workflows are confirmed. An unrun/skipped native job is not a PASS. If any condition fails, report **EVIDENCE_QUALIFIED / BLOCKED** rather than COMPLETE; document exact links and status in #8.

## 5. Independent retrospective evidence audit — 2026-11-12 KST

- **Original exact PR HEAD:** `b59a37c10e0507bffa724f40da828bc77aef8dda`, [PR #592](https://github.com/july-rust/BoardSI/pull/592): all **30/30 GitHub check-runs SUCCESS** (9 listed workflow runs plus independent CodeQL checks; includes native Windows x64/ARM64, macOS ARM64 Rust/Qt). PR Code review produced one P2 finding at intermediate HEAD; corrected with public synthetic pad-only regression; review thread resolved; **unresolved 0**. A *separate* final-head Codex review receipt was not evidenced, so do not claim one.
- **Original merge:** exact Squash `fdf0d09da6e8a4d72cfa7214e16dec44b75034fd` ([commit](https://github.com/july-rust/BoardSI/commit/fdf0d09da6e8a4d72cfa7214e16dec44b75034fd)), **GitHub `verification.verified=true;reason=valid`**, one parent. This resolves the prior `NOT_VERIFIED` signature field.
- **Exact original merge `main` Push:** GitHub Actions `event=push;head_sha=fdf0d09da6e8a4d72cfa7214e16dec44b75034fd` returns **8/8 completed SUCCESS**, including [Qt native](https://github.com/july-rust/BoardSI/actions/runs/43112017682), [Rust numerical/native](https://github.com/july-rust/BoardSI/actions/runs/43112017716), [docs](https://github.com/july-rust/BoardSI/actions/runs/43112017723), and the other five FVM/FEM workflows. This resolves prior `NOT_VERIFIED` post-merge evidence.
- **Small independent follow-up:** inspection found the *pre-read* non-regular/oversize path of `boardsi-import-probe FILE --readiness` rejected input with exit 3 but omitted required static `version/format_status/G/N/P/S/R/M/skipped` output fields. The small patch reuses the existing `ReadinessReport::diagnostic()` schema with `INVALID_FILE_SIZE`; Rust unit and real CLI sparse-oversize-file tests assert complete UNKNOWN/fail-closed fields. The original one-argument output and parser's `INPUT_TOO_LARGE` semantics are unchanged. This follow-up is **qualified only once its own final-head CI, review, protected Verified Squash and exact-main Push are independently verified**.
- **Strict original scope unchanged:** Complete vendor formats, GUI end-to-end, electrical graph, physical SI and U38-U40/U48/T17/T20/T22 are **NOT_ACCEPTED/NOT_RUN**; parent [#8](https://github.com/july-rust/BoardSI/issues/8) stays OPEN. Reclassifying the *bounded E_008* unit is not complete product acceptance.
