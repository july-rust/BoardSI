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

## Required validation evidence (fill after actual checks)

- [ ] Exact source-HEAD Rust fmt, Clippy, cargo tests in Debug and Release, C_003 synthetic test suite and B/C regressions.
- [ ] Windows x64 / Windows ARM64 / macOS ARM64 / Linux Qt Native matrix green.
- [ ] Independent Codex final same-HEAD review completed with no unresolved findings or threads.
- [ ] Protected signed Verified Squash; independent post-main Check Runs and Workflow Runs.
- [ ] Final dated checklist first-check/evidence dates with exact PR, SHA, test references **after** acceptance.

Prior B_002 audit_scenario_revision_sha256 is NOT this C_003 input identity. Future full physical Scenario/Cache, real Run receipt transport, geometry/port completeness, loss models and solver-ready criteria require separate evidence.