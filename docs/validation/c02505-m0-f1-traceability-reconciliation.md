# C_025_05 — M0 F1 specification and U/T traceability acceptance audit

**Audit date:** 2026-10-31 KST. **Source baseline examined:** `main@d083a98eb52641c80124bb1a82ee6ba6b530d05e` (after C_025_04 PR #320). **Disposition:** documentation/identifier reconciliation complete for the inspected source; **M0 F1 remains PARTIAL / acceptance BLOCKED, Issue #2 OPEN**. This is not a new test run, a product sign-off, M0 exit decision, or approval of M1–M5 features.

## Controlled inputs and mechanical reconciliation

- Frozen [specification v1.0](../specification/BoardSI_Development_Spec_v1.0.md): §2 defines exactly **U01–U48**, §13 exactly **T01–T54**. No missing or repeated IDs were found in these definition tables.
- The specification §18.2 maps exactly 48 distinct U IDs once each, with no missing or duplicate U groups. Its T group associations are **coarse** links, not individual case acceptance proofs.
- [M0 U/T traceability ledger](../audits/m0-requirements-traceability-ledger.md): 48 U rows and 54 T rows; neither identifier range has omissions or duplicates. [Existing fail-closed source validator](../../tools/validation/m0_traceability_check.py) is run by [docs-check workflow](../../.github/workflows/docs-check.yml), including the separate T44→U01 regression. This audit's inspection does not replace an exact-head CI execution.
- [Existing immutable 10-case evidence](m0-case-evidence.json) is pinned to source `2a08c167d89279e15f1e65b689a58ed0682afc6e` and full raw Git blob `43b88c0cfdfa2f0267fef8d3619af02a6b54879d` by [m0_case_evidence_check.py](../../tools/validation/m0_case_evidence_check.py) and its mutation tests. Do **not** edit this signed historical evidence or infer new runs from the older source.
- GitHub merged history: [PR #81](https://github.com/july-rust/BoardSI/pull/81), [PR #93](https://github.com/july-rust/BoardSI/pull/93), [PR #102](https://github.com/july-rust/BoardSI/pull/102) and [PR #127](https://github.com/july-rust/BoardSI/pull/127) are **merged**. The historical ledger instruction to resolve PR #81 is superseded, not a current outstanding issue.

## Existing M0 prototype case evidence — no new acceptance

The table quotes the **existing pinned** [M0 case JSON](m0-case-evidence.json); links to evidence are those already fixed in its source-code pin. The `PARTIAL` label means limited synthetic/native scope only, **not formal T acceptance**. Every complete T acceptance and case-specific sign-off remains `NOT_RUN` / `NOT_PERFORMED`.

| Test | Existing M0 prototype evidence | Full T acceptance | Concrete evidence / remaining boundary |
| --- | --- | --- | --- |
| T10 | PARTIAL | NOT_RUN | [Native Qt run](https://github.com/july-rust/BoardSI/actions/runs/39391822772); synthetic cancel/heartbeat, not full product recovery |
| T11 | NOT_RUN | NOT_RUN | No production GPU comparison |
| T13 | PARTIAL | NOT_RUN | [Qt native](https://github.com/july-rust/BoardSI/actions/runs/39391822772) and [Rust native](https://github.com/july-rust/BoardSI/actions/runs/39391823062); full Unicode/locale paths unqualified |
| T32 | PARTIAL | NOT_RUN | [Qt native](https://github.com/july-rust/BoardSI/actions/runs/39391822772); persistent pause/checkpoint/restart not qualified |
| T34 | PARTIAL | NOT_RUN | [Qt native](https://github.com/july-rust/BoardSI/actions/runs/39391822772); no real PCB Import/physical Solver-load UI acceptance |
| T35 | PARTIAL | NOT_RUN | [PR #127](https://github.com/july-rust/BoardSI/pull/127); 8 GiB synthetic Linux cgroup, not SSD spill / real reference-PC test |
| T39 | PARTIAL | NOT_RUN | [Qt native](https://github.com/july-rust/BoardSI/actions/runs/39391822772) and [Rust native](https://github.com/july-rust/BoardSI/actions/runs/39391823062); no three-platform same-PCB Import/complex S equivalence |
| T46 | PARTIAL | NOT_RUN | [Qt native](https://github.com/july-rust/BoardSI/actions/runs/39391822772); bounded synthetic immutable snapshot only |
| T50 | PARTIAL | NOT_RUN | [Qt native](https://github.com/july-rust/BoardSI/actions/runs/39391822772); provisional M0 global event-loop 1,500 ms, unchanged other limits, no real product acceptance |
| T54 | NOT_RUN | NOT_RUN | Persistent SSD checkpoint/recovery test unavailable in the M0 synthetic prototype |

**Other M0-adjacent cases, not silently covered by the pinned ten-case set:** T14 (offline), T15 (package/dependency consistency), T21 (Canonical structural identity) and T49 (confidential-data containment). [M0 Canonical model evidence](m0-canonical-board-model.md) establishes only a **partial** T21 synthetic model contract. [F6/F7 inventory](m0-f6-f7-qualification-evidence.md) explicitly retains packaging, licensing and reference-PC blockers. Complete T14/T15/T21/T49 acceptance remains **NOT_VERIFIED/NOT_RUN** for this audit; their final M0-specific applicability and immutable case-level proof must be adjudicated before F1 certification. T40 is a later release gate, not a manufactured M0 product acceptance requirement.

## M0 F1 / Issue #2 disposition and next gate

| Issue #2 definition-of-done element | C_025_05 result |
| --- | --- |
| Controlled v1.0 requirements baseline, numbered U/T mappings | **Verified document coverage** (48/48 U and 54/54 T; 48/48 unique §18.2 U group memberships) |
| Reproducible fixture, native job and immutable SHA for M0-applicable individual acceptance cases | **PARTIAL**. The pinned 10-case inventory has 8 restricted PARTIAL + 2 NOT_RUN; full per-case sign-off 0 |
| Numerical/import fidelity and unsupported scope declared | **Declared with explicit restrictions**; real-board fidelity, clean install and production physical solver **NOT_VERIFIED** |
| Individual formal PASS/FAIL/NOT_RUN and independent acceptance review | **Not established for full T cases**; all ten existing complete acceptance statuses NOT_RUN; review NOT_PERFORMED |
| Confidential company PCB, proprietary data or credentials committed | No such contents were introduced by this *documentation-only* audit; **this does not certify a complete historical repository security scan** |

**Conclusion:** Document/ID integrity alone is not the M0 F1 exit gate. Because per-case M0 proof and independent full acceptance are still absent, **Issue #2 must remain OPEN**. The [M0 exit gate](m0-completion-gate.md) stays **BLOCKED**, including F6 real reference hardware/approved boards and F7 packaging/licensing. Do not close #3–#7, #46, #51 or merge Draft [#35](https://github.com/july-rust/BoardSI/pull/35). The original PH-08 Gate Q remains FAIL / physical error BOUND_NOT_CERTIFIED.

**Follow-up, specifically scoped:** Review M0 applicability and obtain exact executed fixture+job+commit SHA and independent case-level review for the missing F1 cases, without requiring later-phase M1–M5 tests to pass as an M0 precondition. If an actual unfulfilled M0 prerequisite cannot be exercised, record it as BLOCKED and keep #2 OPEN. No PH-08, full PCB SI, or installer claim is implied.

## PR acceptance is a separate gate

The findings above are from the inspected `main` and pinned historical data. This documentation change still requires its own exact-HEAD required CI, review, protected Verified Squash merge and post-merge `main` checks before **C_025_05 documentation delivery** may be called complete. Never conflate that procedural acceptance with F1 or Issue #2 completion.
