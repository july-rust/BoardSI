# B_002 — B_001 Issue #12 completion and evidence ledger

Date: **2026-10-27 KST** · [Issue #12](https://github.com/july-rust/BoardSI/issues/12) · [Development checklist](../development-checklist.md)

## Decision and objective

B_002 supersedes the earlier proposed M1a Pad/Via/Anti-pad work. It is an **exclusive continuation of incomplete B_001** and prioritizes the existing M1 Stackup/Material/Reference acceptance gates. It does **not** close Issue #12 or qualify the product.

Starting protected main: `c0fbf46ac52ded52c21591771f422d9453025715` (2026-10-27 KST); its 21 displayed check runs are SUCCESS, but these are pre-B_002 checks and prove only the prior accepted restricted scope.

## First B_002 remediation: source- and scenario-bound audit freshness

The limited Qt Stackup editor previously displayed an audited receipt and reset its message when the user edited cells or Net selections, but had no explicit session audit-revision identity. This change:
- Records a session-local SHA-256 digest of the exact accepted Qt-to-Rust JSON audit request; clears it on Material/Layer edits, frequency changes, Signal/Reference selection, imported Net/Copper inventory updates, Overlay load, and new import.
- Rechecks the original PCB source SHA-256 **after** the native Rust audit subprocess, so a file modified during validation cannot leave a successful stale receipt.
- Provides a narrow `currentAuditIsFresh()` probe for native self-tests; source mutation, edit and inventory replacement must invalidate it.
- Preserves the existing hard blocks: Rust receipt `reference_continuity=UNSUPPORTED_NO_COPPER_NET_EVIDENCE`, P009 unsupported, `solver_ready=false`, and `physical_si=NOT_RUN`.

**Boundary:** This is *not* a physical Scenario/Cache implementation and not a solver cache key. It is a GUI audit freshness safeguard. A digest is not a proof of electrical connectivity. No Pad/Via/Port or reference return path is invented.

## Acceptance and blockers

| Item | Before B_002 | First remediation | Remaining for B_001 full acceptance |
| --- | --- | --- | --- |
| Source-bound Qt Stackup material overlay | Limited PASS via #213/#215 | Audit session freshness and post-worker source check added (CI pending) | Qualified source Stackup vs assumptions, material/loss scope |
| Copper/Net Reference | Negative label-level tests via #217 | Existing fail-closed result preserved | Complete source geometry/contact graph and qualified split/return path |
| Scenario/cache | UI invalidates visible status on edit | Adds session audit revision identity (CI pending) | Immutable physical Scenario revisions, cache invalidation and result staleness |
| T02 | Limited input tests | New freshness regression (CI pending) | Full T02 scope |
| T04/T24 | Negative candidates only | No claim of physical return | Physical/geometry reference proofs on supported cases |
| T52 | Limited in-band Layer z checks | No new numerical qualification | Scenario geometry, loss/roughness and extrapolation gates |
| Issue #12 | OPEN | OPEN | Evidence-backed DoD and final review |

## Required verification for this PR

- [ ] Native Qt Windows x64 / ARM64 and macOS ARM64 B001 selftest including newly added receipt freshness assertions.
- [ ] Linux Qt offscreen B001 selftest and Rust numerical/test suite.
- [ ] Same HEAD complete required GitHub checks.
- [ ] Independent Codex exact-HEAD review, no unresolved threads.
- [ ] Protected squash merge and post-main checks; update the evidence status only after actual results.

Never mark U09/U10/U39/T02/T04/T24/T52 full `[x]` or close Issue #12 on this limited change. Continue development after this checkpoint in B_002 until those full gates have independent evidence. If tools, tests, or data prevent completion, record BLOCKED explicitly, not PASS.
