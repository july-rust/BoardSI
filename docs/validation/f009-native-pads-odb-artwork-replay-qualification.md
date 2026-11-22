# F_009 — PADS/ODB++ Native .bsi Source-Free Artwork Replay

**Status: IN_PROGRESS — exact HEAD CI, independent Codex, protected Squash and post-merge main Push NOT_VERIFIED.**

## Scope and contract

- F_005 supported PADS V9.4/ODB++ limited Source -> Canonical -> SQLite Native v1 -> independent Reader -> Main Qt Artwork; source file is not consulted during Native Open or Save As.
- Native v1 SQLite/BSIG schema, opaque extensions and F_004 Goldens unchanged.
- Actual source-specific bounded display receipt from the independently loaded BoardModel, not from original CAD bytes.
- Exact integer-nm straight two-point traces, circular or axis-aligned rectangle conductive Pads, and uniform circular Via annuli (with declared span/drill and unverified connected-layers) only.
- Unsupported arcs, Area/cutout, arbitrary Orientation, thermal/antipad, nonuniform Via or non-nm-exact projection do not silently disappear; Native remains STRUCTURALLY LOADED but Artwork is NOT_RENDERED with diagnostic.
- Object bound 128, Copper Layer bound 32, coordinate bound ±10^12 nm, dimension bound 10^9 nm are only GUI display limits.
- Solver readiness remains false; physical contact, plating and SI NOT_VERIFIED / NOT_RUN. Never declare formal U01–U48/T01–T54 completed from this subset.
- Source SHA/SourceIdentity is preserved but is not a claim of renewed source verification. Synthetic/public fixtures only.

## Verification gates

| ID | Executable proof | Status |
|:--|:--|:--|
| F009-T01/T02 | PADS/ODB++ source-to-native v1 Writer | NOT_VERIFIED |
| F009-T03 | Remove/rename original source and independently Open .bsi | NOT_VERIFIED |
| F009-T04 | Exact known source dimensions, Trace/Pad/Via and source-free preview | NOT_VERIFIED |
| F009-T05 | Full Canonical/Provenance/Geometry Native equality | NOT_VERIFIED |
| F009-T06 | Production Qt QProcess and scene, Layer filter, picking | NOT_VERIFIED |
| F009-T07 | Unsupported/overflow/nonexact shapes without partial-success claim | NOT_VERIFIED |
| F009-T08 | Malformed/required-extension/source changes fail-closed | NOT_VERIFIED |
| F009-T09 | Save As/reopen without CAD, identical artwork semantics | NOT_VERIFIED |
| F009-T10 | Cancel/timeout/kill, stale response, Qt responsiveness | NOT_VERIFIED |
| F009-T11 | Linux x64, Windows x64/ARM64, macOS ARM64 actual native/Qt | NOT_VERIFIED |
| F009-T12 | F_001–F_008 regression, final-HEAD Codex, protected Squash, exact-main CI | NOT_VERIFIED |

Existing test entry points: cargo +1.75.0 fmt/clippy/workspace tests; cargo test -p boardsi-native-file --locked f005_; ctest --test-dir build-qt -R qt-f009- --output-on-failure.

## Evidence

- [F_001 Native contract](../architecture/f001-bsi-format-contract.md)
- [F_005 PADS/ODB native validation](f005-cad-bsi-roundtrip-qualification.md)
- [F_008 source-only validation](f008-gerber-excellon-native-roundtrip-qualification.md)
- Implementation branch: feature/f009-native-pads-odb-artwork-replay
- Exact implementation HEAD, 4OS CI, independent Codex, merge SHA, main Push: **NOT_VERIFIED until inspected**.

**Do not mark COMPLETE without executed evidence for each scoped item.**
