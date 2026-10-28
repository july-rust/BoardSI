# C_003 — Immutable Scenario / Stale Result provenance contract

Date: 2026-10-28 KST. Tracking: [Issue #242](https://github.com/july-rust/BoardSI/issues/242).

## Bounded implementation

- Opt-in Rust boardsi-import::scenario_contract. Owns a frozen deterministic SHA-256 input identity (conditions, engine compatibility, sweep-specific snapshot).
- Separates edit-capable ScenarioDraft from immutable FrozenScenario and independent Run ID.
- Length-prefixed, schema- and domain-separated deterministic fields. Rejects missing/oversized/invalid claimed SHA inputs, same Signal/Reference, duplicate/unordered/out-of-band/oversized frequency grids.
- ResultScope validates synthetic receipt schema, run, source conditions/snapshot, compatibility, strict contiguous sequence and sample frequencies **before** advancing its cursor; rejected old, corrupted, duplicate or out-of-order receipts never advance.
- ResultReceipt::freshness distinguishes Current (identity only), OutOfDate (input/sweep change), Incompatible (engine/model/backend change), and Rejected.
- User edits after capture cannot change FrozenScenario; change-and-revert recovers the same *content* identity but not Run ID. Zoom/theme are deliberately absent.
- A change of frequency grid changes snapshot identity while retaining the condition hash. **Does not permit reuse of the old direct samples**: cache_reuse_eligible() is false.

## Security and physics limitations

**Critical:** Source and Canonical model SHA values in ScenarioDraft are externally supplied *claims*. This contract does not independently reparse the PCB, serialize every BoardModel field, validate materials, or prove completeness. It **must not** be used as a physical Solver entry or Cache authorization.

Existing M0 Synthetic Snapshot v2 and IPC v1 are unchanged; this is a separate provisional C_003 contract, not a full Windows Qt GUI integration or real PCB Worker result transport. No real SI values are computed. C_002 Reference split risk and P009 electrical return-path remain separate unsupported qualifications.

Required unqualified states:
- solver_ready=false
- physical_si=NOT_RUN
- return_path_p009=UNSUPPORTED
- cache_reuse_eligible=false

No U09/U10/U39/T02/T04/T24/T52 formal signoff or complete M1 parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) closure. C_002 Qt Viewer markers and B_007 original final-HEAD Codex review remain independent TODOs.

## Exact-head validation and protected integration — 2026-10-28 KST

- [x] [Code PR #243](https://github.com/july-rust/BoardSI/pull/243) final exact Source HEAD [f1d9c2d66e94d7228326ee009c3c0ae902a1ba3f](https://github.com/july-rust/BoardSI/commit/f1d9c2d66e94d7228326ee009c3c0ae902a1ba3f/checks): 9/9 PR-head GitHub Actions Workflows SUCCESS; Rustfmt, Clippy -D warnings, Debug and Release full workspace, C_003 synthetic test suite **12/12 Debug and 12/12 Release PASS**, previous B_003–B_008/C_001/C_002 suites remain green. First/evidence date: **2026-10-28 KST**.
- [x] Rust Native Windows x64, Windows ARM64 and macOS ARM64; Qt Native Windows x64, Windows ARM64, macOS ARM64 and Linux: all PR-head workflow jobs SUCCESS.
- [x] Independent Codex latest final HEAD f1d9c2d6: review completed, findings false, unresolved false, open review threads **0**. Confirmed independently before merge.
- [x] Protected GitHub **Squash** [main@7edff0b9766a1ae70ab26e7010e356d64bad7046](https://github.com/july-rust/BoardSI/commit/7edff0b9766a1ae70ab26e7010e356d64bad7046) verified=true, reason=valid via REST commit.verification. Verified post-main exact SHA: [21/21 Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/7edff0b9766a1ae70ab26e7010e356d64bad7046/checks) and 9/9 Workflow Runs SUCCESS, including [Push on main #40648767875](https://github.com/july-rust/BoardSI/actions/runs/40648767875), [Qt runtime #40648767924](https://github.com/july-rust/BoardSI/actions/runs/40648767924), [Numerical core #40648767849](https://github.com/july-rust/BoardSI/actions/runs/40648767849).
- [x] First limited implementation checklist acceptance **2026-10-28 KST**, evidence **2026-10-28 KST**, linked to PR #243 and immutable code merge SHA; all prior first-check dates preserved.

**Scoped verdict:** PASS for deterministic immutable Rust provisional input field ownership, synthetic result provenance/freshness and failure handling only. Caller-provided source/model digests are NOT independently verified. Real PCB full Canonical physical snapshot, physical Solver/cache reuse and actual Qt Worker transport are **NOT IMPLEMENTED/NOT VERIFIED**. No physical SI computation, AC Return Path/P009 or U09/U10/U39/T02/T04/T24/T52 formal product PASS. M1 Issue #12 stays OPEN; B_007 original final-HEAD Codex and C_002 Qt Viewer overlays remain pending.

Prior B_002 audit_scenario_revision_sha256 is NOT this C_003 input identity. Future full physical Scenario/Cache, real Run receipt transport, geometry/port completeness, loss models and solver-ready criteria require separate evidence.