# F_008 — Gerber X2 + Excellon Source Evidence → Native `.bsi` → Main Qt

**First completion/evidence date: 2026-11-17 KST. Status: F_008 `100% COMPLETE` (defined Gerber X2 D03 flash + Excellon source-evidence subset only), including exact-final-HEAD independent Codex re-review, protected Squash merges, and verified post-merge `main` CI.** This document is the F_008 single validation ledger, not evidence of full fabrication, global connectivity, or SI acceptance.

## Strict source-only contract

- Existing E_027 source parsers accept a **single Gerber X2 Copper layer** with positive circular D03 flash and one explicitly-decimal METRIC Excellon round-drill source file in the supported grammar. Unsupported commands and lossy unqualified geometry fail closed.
- Gerber Source is represented as exact integer-nm Canonical circular Pad/Padstack geometry; the independently hashed Excellon tool number, byte offset, diameter and round hole XY are preserved as **typed required SQLite v1 extension** `org.boardsi.source.gerber-excellon-drill` (version 1). Original F_002 SQLite v1 schema and BSIG envelopes remain unchanged.
- Exact Source Identity, source-file SHA-256 and extension integrity hash survive load and Save As. Unknown required extensions fail closed. Drill observations are *not* `BoardModel.drills` or `vias`: plating, layer span, common revision and coordinate alignment are **unknown**, even if drill and pad XY match. `solver_ready=false`; no verified connectivity edges, manufacturing continuity, RLGC or S-parameter analysis.
- Real native worker command `save-gerber-drill` re-checks both files and hashes before staging publish, aborts source changes without overwriting the previous good `.bsi`, and supplies a bounded source-only preview.
- Main Qt uses the actual QProcess worker and visualizes supported copper flash and drill observation in a source-only artwork panel, with a non-electrical status, picking and layer filter. The implementation does **not** represent general multi-layer Gerber sets, full Excellon dialects or all fabrication outputs.

## Acceptance ledger

| Gate | Required observed proof | Current result |
|---|---|---|
| T01–T04 | Strict source import, exact flash/drill coordinates and independent SHA/identity | PR CI PASS (supported subset) |
| T05–T07 | Native save/load/Save As, independent raw SQLite/BSIG/typed extension oracle | PR CI PASS (supported subset) |
| T08–T09 | Mutation, malformed input, required-extension corruption and source guards | PR CI PASS (covered cases) |
| T10 | Actual Main Qt artwork, picking/filter, save/open and failure propagation | PR Qt CI PASS (four OS jobs) |
| T11 | Linux x64, Windows x64/ARM64 and macOS ARM64 native/Qt | PR CI PASS (four OS) |
| T12 | Prior F stages, protected merge and exact `main` Push CI | **PASS** — implementation PR 15/15, merged final `main` 12/12 workflows SUCCESS; 28 jobs SUCCESS and 16 intentional docs-only SKIP; real F003/F004/SQLite cross-OS checks PASS |

## Immutable implementation evidence

- [Implementation PR #668](https://github.com/july-rust/BoardSI/pull/668): final reviewed code HEAD `85d546282bf66094d44dbe48e5434f75ed3d6615`, **15/15 PR-triggered workflows SUCCESS**.
- **Final exact-HEAD independent Codex re-review COMPLETED (2026-11-17 02:43:43 UTC)**: [Codex review receipt](https://github.com/july-rust/BoardSI/pull/668#issuecomment-6936389372) explicitly names `85d546282b`; the [machine-readable review summary](https://github.com/july-rust/BoardSI/pull/668#issuecomment-6935113160) records `code_review.commit_sha=85d546282bf66094d44dbe48e5434f75ed3d6615`, `status=completed`, `has_any_findings=false`, `has_unresolved_findings=false`. Original 3 P1/P2 findings were fixed, all 3 inline review threads resolved. Earlier separate quota-exhaustion messages were real historical attempts **before the later successful completed review**; retain them as history, not as the current F_008 gate state.
- [Protected Squash merge `8a45d5ce641029651509026390db651990225fb0`](https://github.com/july-rust/BoardSI/commit/8a45d5ce641029651509026390db651990225fb0): GitHub reported `merged=true` for PR #668 with expected HEAD precondition; REST commit verification reported `verified=true`, `reason=valid`, exactly one parent.
- [Documentation reconciliation PR #669](https://github.com/july-rust/BoardSI/pull/669): **12/12 PR workflows SUCCESS**; protected, signed Verified Squash merged as [`3d749b11c6f7124bc7c03d71efb7a51f3a5ff8af`](https://github.com/july-rust/BoardSI/commit/3d749b11c6f7124bc7c03d71efb7a51f3a5ff8af), exactly one parent.
- **Exact post-merge `main` Push** for `3d749b11c6f7124bc7c03d71efb7a51f3a5ff8af`: **12/12 workflows SUCCESS**, **28 job successes**, **16 intentional docs-only skips**, **0 failures** ([F003 native](https://github.com/july-rust/BoardSI/actions/runs/44006022537), [F004 compatibility](https://github.com/july-rust/BoardSI/actions/runs/44006022599), [F003 resilience](https://github.com/july-rust/BoardSI/actions/runs/44006022705), [Qt runtime](https://github.com/july-rust/BoardSI/actions/runs/44006022635), [Numerical](https://github.com/july-rust/BoardSI/actions/runs/44006022766), [PR #669 final receipt](https://github.com/july-rust/BoardSI/pull/669#issuecomment-6936625253)).
- The **actual four-OS Qt and source tests** were exercised on F_008 code PR #668 final exact HEAD; docs-only `main` Push intentionally skipped four Qt runtime jobs and does **not** count as a second four-OS execution. Earlier code merge `8a45d5ce` Push ended with 10 successful workflows and 2 cancelled when the later docs merge took precedence; the latest final `main` pass and the exact source PR pass are the authoritative bounded acceptance evidence.
- **F_008 closure = T01–T12 (12/12 PASS), supported source-only scope.** No inference of independent vendor PCB, electrical plated-via connectivity, multi-layer production data, actual material stackup, RLGC/S-parameters or formal product U/T acceptance.

## Reproduction

```bash
cargo +1.75.0 fmt --all -- --check
cargo +1.75.0 clippy --locked --workspace --all-targets -- -D warnings
cargo +1.75.0 test -p boardsi-import --locked gerber_excellon_source
cargo +1.75.0 test -p boardsi-native-file --locked --test f008_gerber_drill_roundtrip --features f004-qualification
cargo +1.75.0 test --locked --workspace
# After building real Qt with the native file worker:
ctest --test-dir build-qt -R qt-f008-main-gerber-drill-native-roundtrip --output-on-failure
```

CI sources: [F004 compatibility workflow](https://github.com/july-rust/BoardSI/blob/main/.github/workflows/f004-compatibility.yml) and [Qt runtime workflow](https://github.com/july-rust/BoardSI/blob/main/.github/workflows/qt-runtime.yml).

**No promotion:** The F_008 implementation is a *supported-source subset acceptance*, not a production Gerber/Excellon importer, external vendor-golden conformance test, verified plating/stackup, global net connectivity, SI solver, or formal M0 U/T acceptance.
