# F_010 — Unified Native Cross-Format Artwork Replay

Status: IMPLEMENTATION IN PROGRESS / QUALIFICATION NOT_OBSERVED. Reference main: `fd2b8c6e62ef9f83108c20f27cc1d8bcda7803bd`. No completion or physical SI claim.

Scope: existing F_005–F_009 supported source subsets only. SQLite v1, BSIG, extension payloads and Golden fixtures unchanged. Derived preview status and bounds do not gate structurally valid Native load/save. Source-less replay tests use real Worker processes and Main Qt. No fabrication, electrical topology or SI promotion.

| Gate | Evidence status |
|---|---|
| F010-T01–T05 | Test additions pending actual execution |
| F010-T06–T10 | Common status, bounds and document behavior implemented in part; negative and cross-format tests still required |
| F010-T11 | Four-OS Native and actual Qt pending |
| F010-T12 | Exact final HEAD review, protected merge, main Push pending |

F_007 historical exact merge SHA `c847132dd1078873c370edc75294ac4d8f51169f` Push Actions API returned zero runs when queried 2026-11-27 KST. Do not retroactively assert PASS. Existing evidence retained.

Update this single ledger with observed CI, final Codex review, actual test results, merge and main Push receipts. Do not report docs-only skipped jobs as executed Native/Qt PASS.

## Intermediate receipt — 2026-11-27 KST
- PR #674 HEAD `37b78469728f63668bb3985429e8c71f4959b791`: 14/14 PR workflows SUCCESS; independent Codex completed exact HEAD, zero findings and unresolved threads. This is intermediate evidence, not final qualification.
- Additional Native negative tests and source-free Save As, plus dedicated actual Main Qt cross-format transition and structural-open/NOT_RENDERED test added after that head. Final CI/review still required.
- Existing source parser limits and Gerber required-extension validation remain unchanged; valid Native presentation limitations are distinct from rejected source input or invalid extension data.
