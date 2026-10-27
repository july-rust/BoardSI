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
| Source-bound Qt Stackup material overlay | Limited PASS via #213/#215 | Audit session freshness and post-worker source check added (final PR HEAD 26/26 CI PASS; 2026-10-27 KST) | Qualified source Stackup vs assumptions, material/loss scope |
| Copper/Net Reference | Negative label-level tests via #217 | Existing fail-closed result preserved | Complete source geometry/contact graph and qualified split/return path |
| Scenario/cache | UI invalidates visible status on edit | Adds session audit revision identity (final PR HEAD 26/26 CI PASS; 2026-10-27 KST) | Immutable physical Scenario revisions, cache invalidation and result staleness |
| T02 | Limited input tests | New freshness regression (final PR HEAD 26/26 CI PASS; 2026-10-27 KST) | Full T02 scope |
| T04/T24 | Negative candidates only | No claim of physical return | Physical/geometry reference proofs on supported cases |
| T52 | Limited in-band Layer z checks | No new numerical qualification | Scenario geometry, loss/roughness and extrapolation gates |
| Issue #12 | OPEN | Auto-closed on #219 merge, immediately REOPENED because full gates remain unmet | Evidence-backed DoD and final review |

## Verified B_002 narrow-scope integration (2026-10-27 KST)

- [x] Final source PR [#219](https://github.com/july-rust/BoardSI/pull/219) HEAD `f9deda898b97a42886cfbdbdae972924c3fcdc56`: **26/26 GitHub check runs SUCCESS**, including three Native Qt platforms and Linux offscreen, Rust and supplementary workflows. This is a CI pass of supported synthetic test coverage, **not** full commercial CAD or physical SI acceptance.
- [x] Independent Codex review explicitly reported **"Didn't find any major issues"** at exactly `f9deda898b`; zero unresolved review threads.
- [x] GitHub verified signed protected squash merge [`main@6e795049b6de920f99053ae6361d487a7caeec96`](https://github.com/july-rust/BoardSI/commit/6e795049b6de920f99053ae6361d487a7caeec96) completed on 2026-10-27 KST.
- [ ] Separate post-merge main CI: **pending final complete check on this documentation checkpoint**. Do not infer results from the source PR HEAD.
- [x] GitHub automatically closed Issue #12 on source PR merge despite unfinished gates; Issue #12 was reopened explicitly to preserve accurate completion state.

**Acceptance decision:** limited Qt/Rust audit freshness fix **PASS**. **Full B_001 Issue #12 PARTIAL / OPEN**. Formal U09/U10/U39 and T02/T04/T24/T52 remain unchecked; physical Return Path and Scenario/Cache still unavailable.

## Pending full B_001 work

1. Complete qualified Source/Copper/Net/Area/Pad/Via contact topology and reference layer transitions with exact source-coordinate evidence; detect plane split, isolated islands, cutouts, anti-pads and unknown PADS records without false shorts.
2. Verify actual Reference Copper and Signal/Reference electrical separation, not mere matching text Net labels. Preserve `return_path_p009=UNSUPPORTED` until an independently tested supported electrical/physical return model exists.
3. Implement versioned immutable physical Scenario/Reference/Material snapshots, correct cache invalidation and Out-of-date result semantics; do not conflate the verified **Qt audit session digest** with a real solver-cache key.
4. Qualify Material conductivity/loss/roughness and frequency assumptions with provenance, numerical error and model-band gating; do not invent a loss solver.
5. Provide reproducible fixtures and per-case evidence for full T02/T04/T24/T52 and traceability to U09/U10/U39. Keep every unverified acceptance case NOT_RUN, not PASS.
6. Finish native CI, exact-head independent review, protected merge, post-main CI and dated checklist before final Issue closure.

Do not mark the larger features as complete or close Issue #12 without full independently checkable proof.

## B_002 continuation: immutable limited Stackup audit Scenario revision (candidate 2026-10-27 KST)

Implementation branch: `b002-b001-scenario-revision-contract` (starting from `main@9b7cb3c9e544b5ddbe25cff1833bf6579de889a0`; baseline post-main checks **21/21 SUCCESS**).

The existing Qt session digest detects UI edits but was **not** a Rust-owned audit Scenario revision. The new Rust Worker `boardsi-stackup-config` now constructs a deterministic SHA-256 identity over the **validated, accepted audit inputs**: source PCB SHA-256, all Layer order/thickness/Material Dk/Df/conductivity/bands and provenance strings, chosen Signal/Reference Nets, Source Copper/Net inventory, inventory completeness and requested audit frequency. It reuses the existing dependency-free SHA-256 from `boardsi-import::digest`.

- Rust emits `audit_scenario_revision_schema=boardsi-b002-limited-audit-snapshot-v1`, `audit_scenario_revision_sha256`, and `audit_revision_scope=SOURCE_BOUND_ASSUMED_MATERIAL_ONLY`.
- The UI verifies this contract and exposes a revision **only while** the source and exact last-audited overlay remain fresh; all UI edits invalidate the receipt, including changes when notifications are suppressed.
- Rust tests assert that Dk, thickness, provenance Revision, chosen Net, Net/Copper inventory and holes, source identity and frequency edits change revision. Unrelated zoom/theme changes do not.
- Qt Native selftest exercises change/revert stable identity, changed Reference inventory, changed Cutout, and stale-receipt refusal.
- For safety, **`cache_reuse_eligible=false`**: this is an immutable identity of an assumed Material/Reference **audit input**, **not** a full physics solver Scenario or reusable result cache. All `reference_continuity`, `return_path_p009`, `solver_ready` and `physical_si` hard stops remain unchanged.

### Candidate verification (do not mark PASS before CI)

- [ ] Rust Worker model and snapshot revision unit tests, Rust numerical checks.
- [ ] Native Qt Windows x64, Windows ARM64 and macOS ARM64 B_001 Stackup test.
- [ ] Linux x86_64 Qt regression, earlier PADS Artwork/IPC tests.
- [ ] Exact final HEAD required checks and independent Codex review.
- [ ] Protected merge and separate post-main verification.

**Remaining full B_001 blockers:** electrical Reference Copper/Net/Pad/Via/Port graph and planar cutouts, qualified Return Path, physical solver Scenario and cache, validated Material loss/roughness, formal full U09/U10/U39 + T02/T04/T24/T52. Issue #12 remains **OPEN**. This limited implementation alone is not full B_001 completion.
