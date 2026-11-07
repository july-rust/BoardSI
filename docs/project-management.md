# BoardSI 프로젝트 관리 · 실제 상태와 운영 규칙

> **2026-11-07 KST 현재 운영 안내.** [문서 홈](README.md) · [공식 단일 개발 체크리스트](development-checklist.md) · [요구사항 v1.0](specification/BoardSI_Development_Spec_v1.0.md) · [프로젝트 전반 문서 감사](audits/2026-11-07-documentation-review.md).
>
> **실제 완료 경계:** 개발자 프리뷰와 A/B/C/D 한정 시험은 존재하지만, 정식 M0/M1/M2 종료/제품 U/T는 완료되지 않았습니다. 새 D_054/D_055 범용 Copper 접촉은 **SOURCE-local** 계산만 한정 검증됐습니다. 이전 실험을 각도/Pad 모서리별로 무한 반복하는 것은 개발 정책이 아닙니다. [공통 기하 계약](architecture/trace-copper-contact.md).
>
> **오래된 운영 기록 정정:** G5 설계 [PR #35](https://github.com/july-rust/BoardSI/pull/35)는 기준일 현재 **Closed / Not Merged**입니다. 아래 2026-10월 이력에서 'Draft'라고 기술한 부분은 *당시의 상태*이며 지금 열려 있는 PR이나 현재 진행 중인 소스 구현으로 간주하지 않습니다.

Source of truth: [Development Specification v1.0](specification/BoardSI_Development_Spec_v1.0.md). Requirements U01–U48; acceptance tests T01–T54. A document's completion is NOT proof of program correctness.

**Living implementation tracker:** [A_036 Development Completion Checklist](development-checklist.md). For the normal workflow, ChatGPT alone maintains checklist content through reviewed GitHub PRs; the repository owner reads the Markdown tracker without manually ticking boxes. Every B-series and later feature PR must update the affected requirements/test rows and evidence, recording `체크일: YYYY-MM-DD KST` and the dated source for new checks, or explicitly record that checklist status does not change. This assigns editing responsibility but does not alter the owner's GitHub administrator permissions. A-series scoped Preview successes must not be promoted to full M0/M1/M2 acceptance without the applicable gate evidence.

**Documentation efficiency (2026-11-03 KST):** Document affected checklist states and exact-SHA evidence inside the code/feature PR rather than creating an automatic second documentation PR. For genuine Markdown-only corrections, the required Ubuntu `docs` check runs while all required native/FEM jobs are skipped **before allocating Windows/macOS runners**. Do not manufacture fresh release tests for an evidence correction. JSON, workflow, source and test-fixture changes still require the full applicable CI. The checklist's C_025_01–10 stage index includes audit-only rows without duplicating accepted [x] counts.

## Milestones and labels

[Project plan JSON](project-plan.json) defines 11 development milestone stages, 22 labels and 24 planning Issues. The workflow [bootstrap-management.yml](../.github/workflows/bootstrap-management.yml) creates missing GitHub Milestones/Labels and links existing Issues by their exact titles. Trigger: merge of setup PR into main, or Actions → Bootstrap GitHub Milestones and Labels → Run workflow. It is idempotent; it must show a successful Actions run before considering Milestones created.

## GitHub Projects v2

GitHub Projects v2 Board creation is **not part of this repository's bootstrap workflow**; a Project board's actual existence must be verified in GitHub. If the owner chooses to create it, the GitHub interface is https://github.com/users/july-rust/projects/new ; link this repository after creation. Recommended status columns: Backlog, Ready, In Progress, In Review, Blocked, Done. Add Priority (P0/P1/P2), Area and Requirement/Test fields when supported. Issues and Milestones remain the durable task records.

## Development policy

- M0: Qt/Rust binding, versioned IPC, canonical geometry model, native OS builds, public fixtures and offline confidentiality.
- M1/M1a/M1b: import, artwork, substrate geometry, port configuration and validation.
- M2/M2a/M2b: validated RLGC/direct S, progressive sweep, live statuses, bounded RAM/SSD cache and recovery — initial release gates.
- M2c/M3/M4/M5: **future scope only**; never imply implemented.
- All work on focused branches; merge only after tests and written evidence.
- No real company PCB/stackup/netlist/Touchstone data in repository, PRs, Issues, CI or artifacts.
- Native compilation and actual runtime acceptance are distinct; the docs-only CI intentionally checks neither.

## 2026-10-31 owner-priority reconciliation (C_025_04)

The original runnable-Preview-first policy was preserved from [legacy PR #153](https://github.com/july-rust/BoardSI/pull/153) as the dated [delivery-first playbook](preview-delivery-playbook.md), reconciled against current implementation status. Historical P0 [Issue #152](https://github.com/july-rust/BoardSI/issues/152) is Closed/Completed; A/B/C bounded work is separately recorded in the [development checklist](development-checklist.md). The former P0-specific deferral of GPU and other features is **not** a blanket restriction on current planned development. Formal M0 F1/F6/F7, PH-08 Gate Q, M1 #12 and M2 #16 remain independently gated; developer Preview is not release authorization. [C_025_04 audit](validation/c02504-legacy-open-pr-reconciliation.md) tracks superseded PRs. This paragraph changes development scheduling documentation only, not code or specification U/T acceptance.

## M2 G0 / G1 change history — 2026-10-15 KST

- [#33](https://github.com/july-rust/BoardSI/pull/33) G0 independent synthetic PEC-grounded-box FVM oracle and frozen mathematical/numerical gates were squash merged at `main@ec5fa4b048c0c00d8588dfd90f2c35e1107e220b`.
- [#34](https://github.com/july-rust/BoardSI/pull/34) G1 isolates a triangular Rust FEM prototype and an independent SciPy FVM / Python ABCD comparator. Final review hardened Mesh dimension overflow and enforced the frozen `1e-12` *complex* S tolerance, distinguishing matched PH-01 from mismatched 50-ohm PH-02. [G1 evidence and exact status ledger](validation/m2-g1-lossless-fem-evidence.md) records partial scoped PASS, Not Run, raw observed numbers, and an initial M0 Worker regression failure requiring clean subsequent CI before merge.
- All actual PCB lossy RLGC, DC, multiconductor, open-boundary/frequency qualification, full product T-tests, and the PH-03 exact FEM plate remain outside the accepted G1 scope. Issue [#16](https://github.com/july-rust/BoardSI/issues/16) stays **Open**.
- Subsequent order is **G2** PH-05..08 other Single-ended geometries; **G3** copper/dielectric losses; **G4** differential/multiconductor; **G5** Canonical Board Model→Solver integration. [#35](https://github.com/july-rust/BoardSI/pull/35) is a draft **G5 architecture preflight**, not proof that G2 or G5 is implemented.

## Post-PH07 validation audit

PH-03/05/06/07 restricted synthetic coupons are merged; see [audit #41 status and evidence](validation/m2-validation-audit-hardening.md). PH-08 was researched under Issue #51 (administratively CLOSED on 2026-10-18 KST), while the original physics Gate Q remains **FAIL**, NOT_FROZEN and BOUND_NOT_CERTIFIED; this is **not** completed product functionality. G5 design PR #35 is **closed without merge** (as checked 2026-11-07); the overall physics Issue #16 remains **OPEN**. Administrative main-rule enforcement is tracked separately from implemented CI.

## 2026-10-18 baseline audit checkpoint

[Whole-project reconciliation](audits/2026-10-18-pre-ph08-project-audit.md) distinguishes 17/17 successful current main CI from unverified full application/release. Stage 5B original HMM was privately re-executed with historical Gate Q **FAIL**, not run in public synthetic GitHub Actions. Issue #51 (original HMM) is administratively CLOSED, but its scientific Gate Q **FAIL** is unresolved. Keep #46 (Qt timer) and #16 (Solver) tracked as OPEN at this 2026-11-07 checkpoint. [G5 PR #35](https://github.com/july-rust/BoardSI/pull/35) is now **closed without merge**, and its historical design preflight is not accepted product G5 implementation. The present SVG [option B design master](branding/boardsi-icon-decision.md) is explicitly DESIGN_APPROVED by the owner (2026-10-18); previous PNG byte identity and release packaging are NOT VERIFIED.
