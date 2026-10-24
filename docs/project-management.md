# BoardSI Project Management

Source of truth: [Development Specification v1.0](specification/BoardSI_Development_Spec_v1.0.md). Requirements U01–U48; acceptance tests T01–T54. A document's completion is NOT proof of program correctness.

## Milestones and labels

[Project plan JSON](project-plan.json) defines 11 development milestone stages, 22 labels and 24 planning Issues. The workflow [bootstrap-management.yml](../.github/workflows/bootstrap-management.yml) creates missing GitHub Milestones/Labels and links existing Issues by their exact titles. Trigger: merge of setup PR into main, or Actions → Bootstrap GitHub Milestones and Labels → Run workflow. It is idempotent; it must show a successful Actions run before considering Milestones created.

## GitHub Projects v2

The connected GitHub service does not offer a direct Projects creation action. User: create a Board project **BoardSI Development** at https://github.com/users/july-rust/projects/new and link the BoardSI repository. Recommended status columns: Backlog, Ready, In Progress, In Review, Blocked, Done. Add Priority (P0/P1/P2), Area and Requirement/Test fields when supported. Issues and Milestones remain the durable task records.

## Development policy

- M0: Qt/Rust binding, versioned IPC, canonical geometry model, native OS builds, public fixtures and offline confidentiality.
- M1/M1a/M1b: import, artwork, substrate geometry, port configuration and validation.
- M2/M2a/M2b: validated RLGC/direct S, progressive sweep, live statuses, bounded RAM/SSD cache and recovery — initial release gates.
- M2c/M3/M4/M5: **future scope only**; never imply implemented.
- All work on focused branches; merge only after tests and written evidence.
- No real company PCB/stackup/netlist/Touchstone data in repository, PRs, Issues, CI or artifacts.
- Native compilation and actual runtime acceptance are distinct; the docs-only CI intentionally checks neither.

## 2026-10-21 owner priority update: runnable Preview first

**Current operating priority: a working BoardSI developer Preview, not more standalone research or repeated M0 administrative passes.** See the controlled [Preview delivery playbook](preview-delivery-playbook.md) and executable [Preview P0 Issue #152](https://github.com/july-rust/BoardSI/issues/152).

- **P0 first:** reuse the Qt Widgets + independent Rust Worker synthetic prototype, demonstrate an actually launched native desktop executable (Windows x64 first), status/progress/Cancel and unambiguous SYNTHETIC / NOT PCB SI labeling. A green compile or CI is not itself a runnable application.
- **User-visible vertical slices thereafter:** P1 synthetic Artwork/Net/Layer viewer → P2 one qualified PADS ASC revision/subset → P3 independently cross-checked limited uniform RLGC/2-port S → P4 Preview plots/export and lawful packaging. The complete M1 ODB++ and release requirements remain unchanged.
- **Dual-track:** formal M0 F1/F6/F7 acceptance remains `BLOCKED` until original evidence, qualified reference-PC/board rights and Qt legal/clean-install gates pass. Preview development does not declare M0 complete. Preserve F8 provisional 1,500 ms M0-only timing and historical failures; PH-08 failure remains unfixed and research is deferred.
- **No protection or data compromise:** Ruleset 25046595, 17 required CI, Verified squash promotion, and no proprietary PCB files in public GitHub; never bypass checks. Reuse older tests and review relevant open PRs #115/#117/#118 before new implementation.
- **Efficiency/reporting:** one delivery slice plus at most one independent M0 evidence track; stop blind retries after two unchanged fails, diagnose the root cause; report runnable link where lawful, native measured results, PASS/PARTIAL/FAIL/NOT_RUN, real blockers and next user-visible outcome.
- **Legal distinction:** developer source/runtime technical proof is not redistributable product readiness; don't distribute unqualified Qt/third-party packages.

This owner scheduling update supersedes the blanket "no M1 work until M0" **only to permit separated Preview engineering**, and does not amend Specification v1.0 ``14.2/15 or the formal M0 gate criteria. Historical research records below are kept intact.

## M2 G0 / G1 change history — 2026-10-15 KST

- [#33](https://github.com/july-rust/BoardSI/pull/33) G0 independent synthetic PEC-grounded-box FVM oracle and frozen mathematical/numerical gates were squash merged at `main@ec5fa4b048c0c00d8588dfd90f2c35e1107e220b`.
- [#34](https://github.com/july-rust/BoardSI/pull/34) G1 isolates a triangular Rust FEM prototype and an independent SciPy FVM / Python ABCD comparator. Final review hardened Mesh dimension overflow and enforced the frozen `1e-12` *complex* S tolerance, distinguishing matched PH-01 from mismatched 50-ohm PH-02. [G1 evidence and exact status ledger](validation/m2-g1-lossless-fem-evidence.md) records partial scoped PASS, Not Run, raw observed numbers, and an initial M0 Worker regression failure requiring clean subsequent CI before merge.
- All actual PCB lossy RLGC, DC, multiconductor, open-boundary/frequency qualification, full product T-tests, and the PH-03 exact FEM plate remain outside the accepted G1 scope. Issue [#16](https://github.com/july-rust/BoardSI/issues/16) stays **Open**.
- Subsequent order is **G2** PH-05..08 other Single-ended geometries; **G3** copper/dielectric losses; **G4** differential/multiconductor; **G5** Canonical Board Model→Solver integration. [#35](https://github.com/july-rust/BoardSI/pull/35) is a draft **G5 architecture preflight**, not proof that G2 or G5 is implemented.

## Post-PH07 validation audit

PH-03/05/06/07 restricted synthetic coupons are merged; see [audit #41 status and evidence](validation/m2-validation-audit-hardening.md). PH-08 is active research under Issue #51, while the original physics gate is blocked and is **not** completed product functionality. G5 PR #35 and overall physics Issue #16 remain open. Administrative main-rule enforcement is tracked separately from implemented CI.

## 2026-10-18 baseline audit checkpoint

[Whole-project reconciliation](audits/2026-10-18-pre-ph08-project-audit.md) distinguishes 17/17 successful current main CI from unverified full application/release. Stage 5B original HMM was privately re-executed with historical Gate Q **FAIL**, not run in public synthetic GitHub Actions. Keep Issue #51 (original HMM), #46 (Qt timer), #16 (Solver) OPEN and [G5 Draft PR #35](https://github.com/july-rust/BoardSI/pull/35) unmerged. The present SVG [option B design master](branding/boardsi-icon-decision.md) is explicitly DESIGN_APPROVED by the owner (2026-10-18); previous PNG byte identity and release packaging are NOT VERIFIED.
