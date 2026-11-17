# F_008 — Gerber X2 + Excellon Source Evidence → Native `.bsi` → Main Qt

**Evidence date: 2026-11-17 KST. Status: implementation protected Squash-merged; exact merged-`main` Push CI verification pending.** This document is the F_008 single validation ledger, not evidence of full fabrication, global connectivity, or SI acceptance.

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
| T12 | Prior F stages, protected merge and exact main Push CI | Protected merge PASS; main Push CI **pending independent verification** |

## Immutable implementation evidence

- [Implementation PR #668](https://github.com/july-rust/BoardSI/pull/668): final reviewed code HEAD `85d546282bf66094d44dbe48e5434f75ed3d6615`, **15/15 PR-triggered workflows SUCCESS**.
- Three independent Codex findings were addressed and all three inline review threads resolved. **Final exact-HEAD Codex re-review is NOT_VERIFIED**: subsequent `@codex review` requests returned quota errors, not a successful independent re-review. Preserve those failures; previous successful reviews do not prove a clean review of the final HEAD.
- [Protected Squash merge `8a45d5ce641029651509026390db651990225fb0`](https://github.com/july-rust/BoardSI/commit/8a45d5ce641029651509026390db651990225fb0): GitHub reported `merged=true` for PR #668 with expected HEAD precondition. **Cryptographic signature verification field remains unobserved** unless separately fetched.
- Exact merged-`main` Push-triggered workflow runs were observed to be queued/in progress on 2026-11-17 KST. Do not declare their success without checking each completed conclusion on the exact merge SHA.

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
