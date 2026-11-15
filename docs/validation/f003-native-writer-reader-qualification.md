# F_003 — Native Writer/Reader qualification

> 2026-11-15 KST · [F_001 contract](../architecture/f001-bsi-format-contract.md) · [F_002 schema](../architecture/f002-bsi-sqlite-schema.md) · [BSIG byte layout](../architecture/f002-bsi-geometry-binary-layout.md)

## Scope and evidence rules

- Implementation PR: [#651](https://github.com/july-rust/BoardSI/pull/651) — `crates/boardsi-native`, backed by `boardsi-model`, SQLite 3.37+ STRICT and BSIG v1.
- Code creation is not qualification; no tests may be called PASS until final exact HEAD run succeeds.
- No actual CAD fixture equivalence (F_005), no formal Golden migration (F_004), no U/T, PH-08 or EB-01–07 acceptance.
- Native read defaults to read-only, unknown required fails, unknown optional is read-only, and schema physical/semantic corruption must not return a partial board.

## F003-T01–T12 gates

| Test | Required evidence | Result |
| :--- | :--- | :--- |
| T01/T02 | Rust SQLite file create/open; all 9 BSIG kinds bytes and round-trip | **NOT_VERIFIED** |
| T03/T04 | i64/i128 exactness and full Canonical semantic round-trip | **NOT_VERIFIED** |
| T05/T06 | SourceIdentity, SourceEvidence and graph contact semantics | **NOT_VERIFIED** |
| T07/T08 | Deterministic encode, malicious/truncated/corrupt payload rejection | **NOT_VERIFIED** |
| T09 | Version, future/unknown features and read-only behavior | **NOT_VERIFIED** |
| T10 | Atomic staging replacement and preservation under interrupted write | **NOT_VERIFIED** |
| T11 | Cross-platform native compilation/runtime and cross-OS fixture interchange | **NOT_VERIFIED** |
| T12 | File/row/BLOB limits, resource safety and non-blocking API | **NOT_VERIFIED** |

## Final closure contract

Final exact-head CI, independent review and zero unresolved threads, strict verified squash merge, exact-main push CI and one checklist reconciliation are mandatory before **100% COMPLETE**.

No tests run in a chat/container environment that lacks the Rust compiler can be represented as passed. Check GitHub Actions logs for executable evidence.
