# 전체 검증 보고서 색인 | Validation evidence index

> **보관 원칙:** 검증 기록은 역사적 사실을 증명하는 원장입니다. 이 목록에 문서가 존재한다는 사실은 해당 단계의 정식 제품 인수를 뜻하지 않습니다. 각 문서에 기록된 PASS/FAIL과 당시 SHA/실험 조건이 우선하며, **현재 상태의 유일한 정본은 [개발 체크리스트](../development-checklist.md)**입니다.
>
> **색인 범위:** 2026-11-07 KST 기준 기존 검증 Markdown 원장, [Validation 개요](README.md) 제외. 현재 파일 목록을 작성한 것이며 외부 웹 링크·내부 앵커까지 모든 접속을 보증한 것이 아닙니다.

## 3분 빠른 탐색

| 찾는 내용 | 바로가기 |
| :--- | :--- |
| **전체 개발 상태 / 정식 U/T** | [개발 체크리스트](../development-checklist.md) |
| **범용 Trace·Via·Pad 비중심 접촉** | [D_054–D_055 통합 검증](d054-d055-general-trace-contact-qualification.md) · [기하 알고리즘](../architecture/trace-copper-contact.md) |
| **제한 개발 Preview 완료 범위** | [A_035 P2-Z 감사](a035-p2z-final-preview-acceptance.md) |
| **M0 미완 근거** | [M0 완료 게이트](m0-completion-gate.md) |
| **M2 수치 쿠폰 기준과 실제 관측** | [PH-07 FEM](m2-g2-ph07-fem-evidence.md) · [M2 전체 한정 감사](m2-validation-audit-hardening.md) · [E_001 사후 감사 종결](e001-ph07-post-audit-closure.md) |
| **PH-08 원본 Gate Q FAIL** | [원본 Stage 4 재현](m2-g2-ph08-stage4-original-gateq-reexecution.md) · [Stage 5B 재실행](m2-g2-ph08-stage5b-original-replay-evidence.md) |

## 전체 파일 목록

아래는 Markdown 파일명 기준의 **문서 인벤토리**입니다. 과거 단계 이름이나 세부 테스트 개수는 문서의 작성 시점에 종속됩니다. 신규 개별 각도/모서리 D 작업을 권장하는 목록이 아닙니다.


<details>
<summary><strong>Stage A — Import · Artwork · Qt Preview</strong> — 13개 · PADS 서명/기하, Viewer·Importer 제한 검증</summary>

- [A_025 · p2p diagonal copper preflight](a025-p2p-diagonal-copper-preflight.md)
- [A_026 · p2q circular copper preflight](a026-p2q-circular-copper-preflight.md)
- [A_027 · p2r hollow circular copper](a027-p2r-hollow-circular-copper.md)
- [A_028 · p2s circle polygon mixed source copper](a028-p2s-circle-polygon-mixed-source-copper.md)
- [A_029 · p2t curved copper preflight](a029-p2t-curved-copper-preflight.md)
- [A_030 · p2u curved copper polygon cutout](a030-p2u-curved-copper-polygon-cutout.md)
- [A_031 · p2v copper stabilization](a031-p2v-copper-stabilization.md)
- [A_032 · 1 remediation](a032-1-remediation.md)
- [A_032 · 2 contact viewer](a032-2-contact-viewer.md)
- [A_032 · p2w local connectivity](a032-p2w-local-connectivity.md)
- [A_033 · p2x source artwork parity](a033-p2x-source-artwork-parity.md)
- [A_034 · p2y integration stabilization](a034-p2y-integration-stabilization.md)
- [A_035 · p2z final preview acceptance](a035-p2z-final-preview-acceptance.md)

</details>

<details>
<summary><strong>Stage B — Stackup · Reference · Copper</strong> — 11개 · Reference 형상과 Via/Copper 접촉의 한정 근거</summary>

- [B_001 · 2 canonical stackup audit](b001-2-canonical-stackup-audit.md)
- [B_001 · 3 source reference inventory](b001-3-source-reference-inventory.md)
- [B_001 · stackup reference](b001-stackup-reference.md)
- [B_002 · b001 completion](b002-b001-completion.md)
- [B_003 · b001 reference topology](b003-b001-reference-topology.md)
- [B_004 · reference copper connectivity](b004-reference-copper-connectivity.md)
- [B_005 · reference geometry](b005-reference-geometry.md)
- [B_006 · Via copper contact](b006-via-copper-contact.md)
- [B_007 · copper cutout connectivity](b007-copper-cutout-connectivity.md)
- [B_008 · reference cutout geometry](b008-reference-cutout-geometry.md)
- [B_009 · b series final acceptance](b009-b-series-final-acceptance.md)

</details>

<details>
<summary><strong>Stage C — Reference Coverage · Integration</strong> — 32개 · Reference/Scenario/Native Qt·프로젝트 감사</summary>

- [C_001 · multi cutout island connectivity](c001-multi-cutout-island-connectivity.md)
- [C_002 · reference split transition topology](c002-reference-split-transition-topology.md)
- [C_003 · immutable scenario stale result](c003-immutable-scenario-stale-result.md)
- [C_004 · material loss conductivity roughness](c004-material-loss-conductivity-roughness.md)
- [C_005 · Qt reference risk integration](c005-qt-reference-risk-integration.md)
- [C_006 · reference gap interval](c006-reference-gap-interval.md)
- [C_007 · multi segment reference coverage](c007-multi-segment-reference-coverage.md)
- [C_008 · multilayer reference Via transition](c008-multilayer-reference-via-transition.md)
- [C_009 · reference stitching Via candidate](c009-reference-stitching-via-candidate.md)
- [C_010 · reference corridor stitch continuity](c010-reference-corridor-stitch-continuity.md)
- [C_011 · reference stitch pad footprint](c011-reference-stitch-pad-footprint.md)
- [C_012 · signal reference Via pad clearance](c012-signal-reference-via-pad-clearance.md)
- [C_013 · signal Via reference cutout clearance](c013-signal-via-reference-cutout-clearance.md)
- [C_014 · multi cutout signal Via clearance](c014-multi-cutout-signal-via-clearance.md)
- [C_015 · circular mixed cutout source geometry](c015-circular-mixed-cutout-source-geometry.md)
- [C_016 · reference stitch Via pad clearance](c016-reference-stitch-via-pad-clearance.md)
- [C_017 · reference gap layer transition integration](c017-reference-gap-layer-transition-integration.md)
- [C_018 · reference stitch corridor integration](c018-reference-stitch-corridor-integration.md)
- [C_019 · mixed cutout Via pad integration](c019-mixed-cutout-via-pad-integration.md)
- [C_020 · legacy connectivity review audit](c020-legacy-connectivity-review-audit.md)
- [C_021 · reference area connectivity](c021-reference-area-connectivity.md)
- [C_022 · reference multi area connectivity](c022-reference-multi-area-connectivity.md)
- [C_023 · reference corridor coverage](c023-reference-corridor-coverage.md)
- [C_024 · c series final acceptance](c024-c-series-final-acceptance.md)
- [C_025 · 03 legacy PADS pr reconciliation](c025-03-legacy-pads-pr-reconciliation.md)
- [C02504 legacy open pr reconciliation](c02504-legacy-open-pr-reconciliation.md)
- [C02505 m0 f1 traceability reconciliation](c02505-m0-f1-traceability-reconciliation.md)
- [C02506 pre PH-08 audit issue78 closeout](c02506-pre-ph08-audit-issue78-closeout.md)
- [C02507 open issue d stage handoff](c02507-open-issue-d-stage-handoff.md)
- [C02508 m0 architecture issue3 acceptance](c02508-m0-architecture-issue3-acceptance.md)
- [C02509 Qt global timer stabilization](c02509-qt-global-timer-stabilization.md)
- [C02510 m0 f6 public fixture reconciliation](c02510-m0-f6-public-fixture-reconciliation.md)

</details>

<details>
<summary><strong>Stage D — SOURCE Copper Connectivity</strong> — 53개 · 과거 각도별 합성 실험은 이력으로 보존. **새 개발은 D_054/D_055 범용 엔진을 재사용**</summary>

- [D_001 · source pad endpoint](d001-source-pad-endpoint.md)
- [D_002 · copper split contact qualification](d002-copper-split-contact-qualification.md)
- [D_003 · three area copcut connectivity](d003-three-area-copcut-connectivity.md)
- [D_004 · trace endpoint area attachment](d004-trace-endpoint-area-attachment.md)
- [D_005 · pad trace area qualification](d005-pad-trace-area-qualification.md)
- [D_006 · Via three area qualification](d006-via-three-area-qualification.md)
- [D_007 · Via trace area qualification](d007-via-trace-area-qualification.md)
- [D_008 · dual layer Via area qualification](d008-dual-layer-via-area-qualification.md)
- [D_009 · dual layer Via trace qualification](d009-dual-layer-via-trace-qualification.md)
- [D_010 · two trace four area qualification](d010-two-trace-four-area-qualification.md)
- [D_011 · source contact candidate graph qualification](d011-source-contact-candidate-graph-qualification.md)
- [D_012 · Via span plating provenance qualification](d012-via-span-plating-provenance-qualification.md)
- [D_013 · legacy barrel provenance qualification](d013-legacy-barrel-provenance-qualification.md)
- [D_014 · two Via progress](d014-two-via-progress.md)
- [D_015 · t junction qualification](d015-t-junction-qualification.md)
- [D_016 · x junction qualification](d016-x-junction-qualification.md)
- [D_017 · oblique x junction qualification](d017-oblique-x-junction-qualification.md)
- [D_019 · oblique t junction qualification](d019-oblique-t-junction-qualification.md)
- [D_020 · contained collinear overlap qualification](d020-contained-collinear-overlap-qualification.md)
- [D_021 · partial collinear overlap qualification](d021-partial-collinear-overlap-qualification.md)
- [D_022 · collinear endpoint contact qualification](d022-collinear-endpoint-contact-qualification.md)
- [D_023 · opposite Via a endpoint qualification](d023-opposite-via-a-endpoint-qualification.md)
- [D_024 · orthogonal endpoint contact qualification](d024-orthogonal-endpoint-contact-qualification.md)
- [D_025 · orthogonal Via a endpoint qualification](d025-orthogonal-via-a-endpoint-qualification.md)
- [D_026 · oblique Via b endpoint qualification](d026-oblique-via-b-endpoint-qualification.md)
- [D_027 · oblique Via a endpoint qualification](d027-oblique-via-a-endpoint-qualification.md)
- [D_028 · non45 oblique Via b endpoint qualification](d028-non45-oblique-via-b-endpoint-qualification.md)
- [D_029 · non45 oblique Via a endpoint qualification](d029-non45-oblique-via-a-endpoint-qualification.md)
- [D_030 · non45 steep Via b endpoint qualification](d030-non45-steep-via-b-endpoint-qualification.md)
- [D_031 · non45 steep Via a endpoint qualification](d031-non45-steep-via-a-endpoint-qualification.md)
- [D_032 · non45 steep mirrored Via b endpoint qualification](d032-non45-steep-mirrored-via-b-endpoint-qualification.md)
- [D_033 · non45 steep mirrored Via a endpoint qualification](d033-non45-steep-mirrored-via-a-endpoint-qualification.md)
- [D_034 · non45 oblique mirrored Via b endpoint qualification](d034-non45-oblique-mirrored-via-b-endpoint-qualification.md)
- [D_035 · non45 oblique mirrored Via a endpoint qualification](d035-non45-oblique-mirrored-via-a-endpoint-qualification.md)
- [D_036 · dual mirrored Via endpoint qualification](d036-dual-mirrored-via-endpoint-qualification.md)
- [D_037 · mixed sign dual Via endpoint qualification](d037-mixed-sign-dual-via-endpoint-qualification.md)
- [D_038 · mixed sign reversed dual Via endpoint qualification](d038-mixed-sign-reversed-dual-via-endpoint-qualification.md)
- [D_039 · same layer copper connectivity qualification](d039-same-layer-copper-connectivity-qualification.md)
- [D_040 · multi component copper isolation qualification](d040-multi-component-copper-isolation-qualification.md)
- [D_041 · same layer copper bridge qualification](d041-same-layer-copper-bridge-qualification.md)
- [D_042 · same layer closed loop qualification](d042-same-layer-closed-loop-qualification.md)
- [D_043 · same layer cutset qualification](d043-same-layer-cutset-qualification.md)
- [D_044 · parallel finite width contact qualification](d044-parallel-finite-width-contact-qualification.md)
- [D_045 · perpendicular finite width contact qualification](d045-perpendicular-finite-width-contact-qualification.md)
- [D_046 · oblique finite width contact qualification](d046-oblique-finite-width-contact-qualification.md)
- [D_047 · trace pad finite width contact qualification](d047-trace-pad-finite-width-contact-qualification.md)
- [D_048 · trace round pad finite width contact qualification](d048-trace-round-pad-finite-width-contact-qualification.md)
- [D_049 · mixed round square pad contact qualification](d049-mixed-round-square-pad-contact-qualification.md)
- [D_050 · finite width intercomponent bridge qualification](d050-finite-width-intercomponent-bridge-qualification.md)
- [D_051 · dual finite width bridge loop qualification](d051-dual-finite-width-bridge-loop-qualification.md)
- [D_052 · dual bridge midspan break qualification](d052-dual-bridge-midspan-break-qualification.md)
- [D_053 · dual bridge midspan reconnection qualification](d053-dual-bridge-midspan-reconnection-qualification.md)
- [D_054–D_055 · 범용 Trace·원호·Via 환형 패드·SMD Pad 접촉 검증](d054-d055-general-trace-contact-qualification.md)

</details>

<details>
<summary><strong>M0 — Core · IPC · Runtime</strong> — 11개 · 기초 모델·수치·Worker/Qt 실행 및 미완 게이트</summary>

- [M0 · canonical board model](m0-canonical-board-model.md)
- [M0 · completion gate](m0-completion-gate.md)
- [M0 · f1 synthetic executable evidence](m0-f1-synthetic-executable-evidence.md)
- [M0 · f6 f7 qualification evidence](m0-f6-f7-qualification-evidence.md)
- [M0 · f8 provisional 1500ms policy](m0-f8-provisional-1500ms-policy.md)
- [M0 · native platforms](m0-native-platforms.md)
- [M0 · numerical foundation](m0-numerical-foundation.md)
- [M0 · Qt issue46 cold start diagnostics](m0-qt-issue46-cold-start-diagnostics.md)
- [M0 · Qt native runtime](m0-qt-native-runtime.md)
- [M0 · Qt runtime followup](m0-qt-runtime-followup.md)
- [M0 · worker ipc Qt prototype](m0-worker-ipc-qt-prototype.md)

</details>

<details>
<summary><strong>Stage E — Audit closeout</strong> — 1개 · PH-07 사후 감사 한정 종결, 미해결 #46 별도 이관</summary>

- [E_001 · PH-07 post-audit closure and independent disposition](e001-ph07-post-audit-closure.md)

</details>

<details>
<summary><strong>M2 — FEM·RLGC Restricted Research</strong> — 15개 · 정해진 PEC/lossless 수학·쿠폰 검증이며 제품 PCB Solver 인수가 아님</summary>

- [M2 g0 acceptance protocol](m2-g0-acceptance-protocol.md)
- [M2 g0 reference evidence](m2-g0-reference-evidence.md)
- [M2 g1 lossless FEM evidence](m2-g1-lossless-fem-evidence.md)
- [M2 g2 ph05 acceptance contract](m2-g2-ph05-acceptance-contract.md)
- [M2 g2 ph05 FEM evidence](m2-g2-ph05-fem-evidence.md)
- [M2 g2 ph05 independent reference](m2-g2-ph05-independent-reference.md)
- [M2 g2 ph06 acceptance contract](m2-g2-ph06-acceptance-contract.md)
- [M2 g2 ph06 FEM evidence](m2-g2-ph06-fem-evidence.md)
- [M2 g2 ph06 independent reference](m2-g2-ph06-independent-reference.md)
- [M2 g2 ph07 acceptance contract](m2-g2-ph07-acceptance-contract.md)
- [M2 g2 ph07 FEM evidence](m2-g2-ph07-fem-evidence.md)
- [M2 g2 ph07 independent reference](m2-g2-ph07-independent-reference.md)
- [M2 ph03 parallel plate acceptance contract](m2-ph03-parallel-plate-acceptance-contract.md)
- [M2 RLGC independent verification plan](m2-rlgc-independent-verification-plan.md)
- [M2 validation audit hardening](m2-validation-audit-hardening.md)

</details>

<details>
<summary><strong>PH-08 — Historical HMM Gate Q FAIL and Proposals</strong> — 22개 · **원본 Gate Q FAIL은 유지.** Private replay, synthetic tests, proposed contracts and historical diagnostics should not be conflated</summary>

- [M2 g2 PH-08 fixed operator error budget](m2-g2-ph08-fixed-operator-error-budget.md)
- [M2 g2 PH-08 gateq independent ci evidence](m2-g2-ph08-gateq-independent-ci-evidence.md)
- [M2 g2 PH-08 gateq independent proposed](m2-g2-ph08-gateq-independent-proposed.md)
- [M2 g2 PH-08 gateq tr exact captured proposed](m2-g2-ph08-gateq-tr-exact-captured-proposed.md)
- [M2 g2 PH-08 gateq tr proposal](m2-g2-ph08-gateq-tr-proposal.md)
- [M2 g2 PH-08 global assembly diagnostic](m2-g2-ph08-global-assembly-diagnostic.md)
- [M2 g2 PH-08 HMM bridge proposed](m2-g2-ph08-hmm-bridge-proposed.md)
- [M2 g2 PH-08 original HMM root cause analysis](m2-g2-ph08-original-hmm-root-cause-analysis.md)
- [M2 g2 PH-08 original recovery operator evidence](m2-g2-ph08-original-recovery-operator-evidence.md)
- [M2 g2 PH-08 recovery hardening evidence](m2-g2-ph08-recovery-hardening-evidence.md)
- [M2 g2 PH-08 stage4 integrity recheck](m2-g2-ph08-stage4-integrity-recheck.md)
- [M2 g2 PH-08 stage4 original gateq reexecution](m2-g2-ph08-stage4-original-gateq-reexecution.md)
- [M2 g2 PH-08 stage5b original replay evidence](m2-g2-ph08-stage5b-original-replay-evidence.md)
- [M2 g2 PH-08 stage5b private replay runbook](m2-g2-ph08-stage5b-private-replay-runbook.md)
- [M2 g2 PH-08 stage5c chat evidence](m2-g2-ph08-stage5c-chat-evidence.md)
- [M2 g2 PH-08 stage5c q protocol proposal](m2-g2-ph08-stage5c-q-protocol-proposal.md)
- [M2 g2 PH-08 stage5d evidence](m2-g2-ph08-stage5d-evidence.md)
- [M2 g2 PH-08 stage5d gateq protocol notes](m2-g2-ph08-stage5d-gateq-protocol-notes.md)
- [M2 g2 PH-08 stage5e audit evidence](m2-g2-ph08-stage5e-audit-evidence.md)
- [M2 g2 PH-08 stage5f github transfer](m2-g2-ph08-stage5f-github-transfer.md)
- [M2 g2 PH-08 uniform gradient error budget](m2-g2-ph08-uniform-gradient-error-budget.md)
- [M2 g2 PH-08 uniform quadratic identity](m2-g2-ph08-uniform-quadratic-identity.md)

</details>

<details>
<summary><strong>Security / Administrative Evidence</strong> — 1개 · 보안 경보·관리 감사</summary>

- [SEC001 codeql markdown comment scan](sec001-codeql-markdown-comment-scan.md)

</details>

---

**목록 집계:** 158개 그룹 분류 + 0개 미분류 = **158개** 기존 Markdown 근거 파일. 개별 기록을 일괄 수정하거나 과거 `FAIL`을 변경하지 않았습니다. 신규 보고서는 이 색인에 등록하고, 정식 완료 판정은 [단일 체크리스트](../development-checklist.md)에만 반영합니다. 검증 방법은 [Validation 설명](README.md), 문서 정책은 [문서 표준](../documentation-standards.md)을 참조하십시오.
