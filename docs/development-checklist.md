# BoardSI 개발 사양서 이행 체크리스트 — 단일 정본 (A_036 이후 · E 준비)

> 작성: 2026-10-26 KST · 추적 [A_036 Issue #209](https://github.com/july-rust/BoardSI/issues/209)  
> 기준: [개발 사양서 v1.0](specification/BoardSI_Development_Spec_v1.0.md) (U01–U48 / T01–T54), 착수 시 보호 `main@8168298e5a2f64966220ead49017aeef2b4844ad`, [A_035 범위 감사](validation/a035-p2z-final-preview-acceptance.md).  
> **사양서의 요구사항 확정 ≠ 프로그램 구현 완료 ≠ 정식 인수시험 PASS.** 아래 상태는 검증된 범위만 표시하며 사양서·PH-08 물리 Gate·릴리즈 정책을 변경하지 않습니다.

## 통합 현황판 — A/B/C/D 증거 및 E 실행 계획
<a id="stage-overview"></a>

> **단일 정식 체크리스트:** 이 파일만 수정·운영합니다. 기존의 별도 조회용 `development-checklist-stage-view.md`는 통합 후 폐지합니다. 아래 표는 상세 원장의 체크 상태를 읽기 쉽게 정리한 것으로, 체크일·근거일·PR/CI/SHA와 제한 조건은 아래 원문을 우선합니다.
>
> **검증 범위:** 체크된 항목의 *한정 검증 PASS*는 실제 PCB의 전기적 Open/Short, P009 Return Path, 제품 RLGC/S Solver, PH-08 물리 Gate 또는 U/T 정식 제품 인수와 다릅니다. 현황판 수치는 2026-11-11 KST 기준이며 체크 항목 변경 시 함께 갱신합니다.

**빠른 이동:** [Stage A](#stage-a) · [A_001–A_036 전번호 색인](#a-history-ledger) · [Stage B](#stage-b) · [Stage C](#stage-c) · [Stage D](#stage-d) · [Stage E](#stage-e) · [보안](#stage-security) ·
  [제품 마일스톤](#stage-milestones) · [정식 U/T](#stage-acceptance) · [문서·CI 관리](#documentation-ci-evidence) · [상세 원장](#목차)

| 영역 | 작업 의미 | 한정 확인 | 미확인 |
| :--- | :--- | ---: | ---: |
| [**A**](#stage-a) | Preview · Import · Artwork | 12 | 5 |
| [**B**](#stage-b) | Stackup · Copper/Reference 후보 | 17 | 6 |
| [**C**](#stage-c) | Reference 기하 · Qt 통합 · 감사 | 40 | 8 |
| [**D**](#stage-d) | PADS Source 접촉 후보 · 국소 Copper · 다층·CAD Source Adapter | 95 | 8 |
| [**E**](#stage-e) | 미완 감사·기능 30개 소형 종결 단위 (제품 인수 외) | 3 | 27 |
| [**SEC_001**](#stage-security) | CodeQL Alert #1 한정 Fixed 확인 | 1 | 0 |
| [**U01–U48**](#stage-acceptance) | 제품 요구사항 | 1 | 47 |
| [**T01–T54**](#stage-acceptance) | 정식 인수시험 | 0 | 54 |

**전체 체크 상태: 166건 확인 · 128건 미확인.**

> E_001은 제한된 사후 감사 종결, E_002는 M0 F1 인수상태 정합화 한정 완료로 기록하며, E_003은 Rust/Qt/IPC v1 제한 호환성 회귀로 별도 인수하며 E_004~E_030은 미착수 27건입니다. E 수치는 기존 **166건 확인 · 128건 미확인**의 A/B/C/D/SEC/U/T 통계에 포함하지 않습니다. 제한 범위의 완료 기록은 제품 완료율을 뜻하지 않습니다.

<a id="stage-e"></a>
### Stage E — 미완 작업 종결 실행 현황

> **D 작업은 D_056_09에서 종료**하며 과거 D_056_10~13은 역사적 계획 번호만 보존합니다. **E_001은 2026-11-11 KST 제한 범위로 완료했습니다([PR #583](https://github.com/july-rust/BoardSI/pull/583)). E_002는 동일 날짜의 한정 추적성 정합화 결과이며 [검증 기록](validation/e002-m0-f1-traceability-status-reconciliation.md) 및 Issue #2 최종 작업 댓글에서 PR/CI 종료 근거를 확인합니다.** E_003은 [제한 호환성 검증](validation/e003-rust-qt-ipc-compatibility-qualification.md)을 PR #585의 최종 CI·리뷰·보호 병합 후에만 **COMPLETE**로 판정합니다. E_004~E_030은 **NOT_STARTED**입니다. 기존 A/B/C/D/SEC/U/T의 166건 확인·128건 미확인 통계에는 아직 집계하지 않습니다.
>
> [**E_001~E_030 실행 계획·완료 관문**](e-stage-closure-plan.md) · [**실행 불가·외부 의존성 별도 관리**](e-stage-blocked-register.md) · [현재 GitHub 열린 이슈](https://github.com/july-rust/BoardSI/issues?q=is%3Aissue+is%3Aopen). **E 계획 수립 당시(2026-11-11 KST) 스냅샷: Open Issue 30건 · Open PR 0건.** 이후 변경은 최신 GitHub 상태를 우선합니다.

| 단위 | 끝낼 작업 한 가지 | 관련 이슈 | 단계 상태 |
| :--- | :--- | :--- | :--- |
| **E_001** | PH-07 사후 감사 #41 독립 종결 판단 | [#41](https://github.com/july-rust/BoardSI/issues/41), [#46 → E_029](https://github.com/july-rust/BoardSI/issues/46) | **COMPLETE (2026-11-11 KST · [PR #583](https://github.com/july-rust/BoardSI/pull/583))** |
| **E_002** | M0 U/T 추적성 정합화 | [#2](https://github.com/july-rust/BoardSI/issues/2), [검증 기록](validation/e002-m0-f1-traceability-status-reconciliation.md) | **COMPLETE (한정 정합화 · 2026-11-11 KST; PR/CI는 #2 종료 댓글 참조)** |
| **E_003** | Rust/Qt/IPC v1 호환성 근거·부정 회귀 | [#3](https://github.com/july-rust/BoardSI/issues/3), [검증](validation/e003-rust-qt-ipc-compatibility-qualification.md) | **COMPLETE (한정; PR #585 검증·보호 병합·main Push 통과 시; 전체 #3은 OPEN)** |
| **E_004** | Canonical Source/Via 근거 | [#4](https://github.com/july-rust/BoardSI/issues/4) [#520](https://github.com/july-rust/BoardSI/issues/520) | NOT_STARTED |
| **E_005** | Worker 비차단 단일 회귀 | [#5](https://github.com/july-rust/BoardSI/issues/5) | NOT_STARTED |
| **E_006** | 3 Native 플랫폼 근거 | [#6](https://github.com/july-rust/BoardSI/issues/6) | NOT_STARTED |
| **E_007** | 공개 Fixture·라이선스 | [#7](https://github.com/july-rust/BoardSI/issues/7) | NOT_STARTED |
| **E_008** | 입력 포맷 readiness | [#8](https://github.com/july-rust/BoardSI/issues/8) | NOT_STARTED |
| **E_009** | PADS 한 Source 문법 | [#9](https://github.com/july-rust/BoardSI/issues/9) | NOT_STARTED |
| **E_010** | Qt 메인 Picking 한 경로 | [#11](https://github.com/july-rust/BoardSI/issues/11) | NOT_STARTED |
| **E_011** | Stackup 편집/무효화 | [#12](https://github.com/july-rust/BoardSI/issues/12) | NOT_STARTED |
| **E_012** | 실기기 GPU 근거 1개 | [#165](https://github.com/july-rust/BoardSI/issues/165) | NOT_STARTED |
| **E_013** | ODB++ ZIP DEFLATE | [#578](https://github.com/july-rust/BoardSI/issues/578) | NOT_STARTED |
| **E_014** | ODB++ GZIP/TGZ | [#578](https://github.com/july-rust/BoardSI/issues/578) | NOT_STARTED |
| **E_015** | ODB++ features.Z | [#578](https://github.com/july-rust/BoardSI/issues/578) | NOT_STARTED |
| **E_016** | ODB++ Negative/Island | [#578](https://github.com/july-rust/BoardSI/issues/578) [#10](https://github.com/july-rust/BoardSI/issues/10) | NOT_STARTED |
| **E_017** | ODB++ Step Transform | [#578](https://github.com/july-rust/BoardSI/issues/578) | NOT_STARTED |
| **E_018** | ODB++ EDA PKG/PIN | [#578](https://github.com/july-rust/BoardSI/issues/578) | NOT_STARTED |
| **E_019** | ODB++ Drill→Via | [#578](https://github.com/july-rust/BoardSI/issues/578) | NOT_STARTED |
| **E_020** | ODB++ Stackup Material/z | [#578](https://github.com/july-rust/BoardSI/issues/578) | NOT_STARTED |
| **E_021** | ODB++ Qt 메인 Import | [#578](https://github.com/july-rust/BoardSI/issues/578) [#11](https://github.com/july-rust/BoardSI/issues/11) | NOT_STARTED |
| **E_022** | ODB++ SOURCE Graph | [#578](https://github.com/july-rust/BoardSI/issues/578) [#520](https://github.com/july-rust/BoardSI/issues/520) | NOT_STARTED |
| **E_023** | ODB++ 공개 Vendor Oracle | [#578](https://github.com/july-rust/BoardSI/issues/578) [#10](https://github.com/july-rust/BoardSI/issues/10) | NOT_STARTED |
| **E_024** | ODB++ 전체 Gate 재평가 | [#578](https://github.com/july-rust/BoardSI/issues/578) [#10](https://github.com/july-rust/BoardSI/issues/10) | NOT_STARTED |
| **E_025** | IPC-2581 Source 소형 Slice | [#532](https://github.com/july-rust/BoardSI/issues/532) [#23](https://github.com/july-rust/BoardSI/issues/23) | NOT_STARTED |
| **E_026** | KiCad PCB 소형 Slice | [#532](https://github.com/july-rust/BoardSI/issues/532) [#23](https://github.com/july-rust/BoardSI/issues/23) | NOT_STARTED |
| **E_027** | Gerber/Drill 소형 Slice | [#532](https://github.com/july-rust/BoardSI/issues/532) [#23](https://github.com/july-rust/BoardSI/issues/23) | NOT_STARTED |
| **E_028** | 교차 형식 Canonical 한 케이스 | [#532](https://github.com/july-rust/BoardSI/issues/532) [#520](https://github.com/july-rust/BoardSI/issues/520) | NOT_STARTED |
| **E_029** | macOS Qt 타이머 원인 실험 | [#46](https://github.com/july-rust/BoardSI/issues/46) | NOT_STARTED |
| **E_030** | E 최종 전수 감사 | 전체 30개 | NOT_STARTED |

**E_001 종결 근거:** 2026-11-11 KST, [PH-07 감사 근거·범위](validation/e001-ph07-post-audit-closure.md), [#41](https://github.com/july-rust/BoardSI/issues/41), [#43](https://github.com/july-rust/BoardSI/issues/43), [#46](https://github.com/july-rust/BoardSI/issues/46), [PR #583](https://github.com/july-rust/BoardSI/pull/583). **신규 물리 FEM/FVM 재실행이 아닌** 이전 감사 증거와 현행 GitHub Ruleset의 독립 상태 재확인입니다. #46 타이머 원인은 미규명이며 [EB-02](e-stage-blocked-register.md)에 보존하여 E_029로 인계합니다. 과거 PH-08 Gate Q FAIL과 정식 U/T 미인수는 그대로 유지합니다.

**E_003 제한 종결 근거:** 2026-11-11 KST, [PR #585](https://github.com/july-rust/BoardSI/pull/585) · [기술/부정 시험](validation/e003-rust-qt-ipc-compatibility-qualification.md). 기존 Qt Widgets/C++17 + 독립 Rust Worker + QProcess + Versioned JSON IPC v1을 유지하며, malformed UTF-8·Header·Frame·Version·Unknown Field·Kind와 Sequence Cursor의 Fail-Closed 회귀만 추가합니다. 이 행의 COMPLETE는 **정확한 PR HEAD 필수 CI, 독립 리뷰, Verified Squash 및 새 main Push CI 확인 후에만 유효**하며 실행 결과는 #3의 마지막 댓글을 우선합니다. **Issue #3은 F7/배포·라이선스 및 정식 T13/T39/T46 미인수로 OPEN**; 전체 M0/제품 인수, 물리 PH-08 Gate Q, GPU #165 및 Qt 원인 #46을 완료로 승격하지 않습니다.

**E 종결 규칙:** 각 번호의 코드/문서·부정 테스트·PR 최종 HEAD CI·Verified Squash·정확한 main Push 근거를 같은 작업에서 끝냅니다. 상위 이슈가 여전히 미완이면 거짓 Closed 대신 [차단/이관 원장](e-stage-blocked-register.md)에 재개 조건을 남깁니다. 작업이 분할되더라도 이미 시작한 E 번호의 수락
  범위를 무기한 Open으로 남기지 않습니다.

<a id="documentation-ci-evidence"></a>
### 문서 정합성·Docs-only CI 운영 검증 (비집계 관리 이력)

> **최초 정리일: 2026-11-08 KST · 근거 확인일: 2026-11-08 KST.** 이 표는 공식 요구사항·A/B/C/D 구현 항목을 추가한 것이 아닌 **문서와 CI 운영의 변경 추적표**입니다. 따라서 해당 관리 이력 자체는 A/B/C/D 체크 수에 영향을 주지 않습니다. D_056_01 및 D_056_02의 **한정 인수** 체크 항목을 현황판에 각각 1건씩 집계했습니다. 이 수치는 전체 PCB 전기적 연결성/제조/SI 인수를 의미하지 않습니다. 모든 Markdown 파일을 개별 체크박스로 중복 관리하지 않으며, 실제 파일별 길찾기는 [문서 허브](README.md)와 [검증 색인](validation/INDEX.md)에 맡깁니다.

#### 문서 전수 목록 및 현황

- **완료 근거:** 보호된 main SHA **0faed4133e2cb726860aa021b4c95db9bc900a2a** Git Tree의 **Markdown 189개**: docs/validation/ **160개**(기존 검증 보고서 **158개** + README.md + INDEX.md), 그 외 **29개**. [문서 허브](README.md) ·
  [158개 보고서 색인](validation/INDEX.md) · [날짜별 전체 문서 감사](audits/2026-11-07-documentation-audit.md). 2026-11-07의 182개는 **당시 스냅샷**이지 현재 파일 개수가 아님
- **검증 경계/미확인:** **189개 모든 문장의 독립 기술 재검증, 모든 외부 URL·내부 Anchor 인증은 NOT_VERIFIED**. 과거 FAIL/PASS 기록은 보존

#### 프로젝트 문서 구조·기술 설명 정정

- **완료 근거:** [문서 정리 Issue #511](https://github.com/july-rust/BoardSI/issues/511) · [PR #512](https://github.com/july-rust/BoardSI/pull/512) →
  [main 7f02600](https://github.com/july-rust/BoardSI/commit/7f0260007c81ea29127df8ee3c347e37401a2875). 문서 허브·Importer/Qt README, Quickstart, 보안·기여 정책 정정. 해당 PR **필수 10개 워크플로 SUCCESS**
- **검증 경계/미확인:** 초기 P0/P2-A 당시 상태와 실제 현재 기능 구분. 문서 보완이 보드 SI 정식 인수라는 뜻은 아님

#### 검증 보고서 색인·범용 기하 계약 보강

- **완료 근거:** [문서 감사 Issue #514](https://github.com/july-rust/BoardSI/issues/514) · [PR #515](https://github.com/july-rust/BoardSI/pull/515) →
  [main 97f5c69](https://github.com/july-rust/BoardSI/commit/97f5c6941b103ca8039f2bd841bd79d1a181d4ed). 보고서 **158/158개** 색인, 변경 문서의 상대경로 **323개 점검 / 파일 누락 0건**. 필수 PR 8개 및 해당 main Push **8개 워크플로 SUCCESS**.
  [D_054/D_055 기하 계약](architecture/trace-copper-contact.md) · [문서 작성 표준](documentation-standards.md)
- **검증 경계/미확인:** SOURCE-local 구리 접촉 후보와 전체 전기 연결성/물리 SI 구분. 과거 시험 수치를 모두 재계산한 것은 아님

#### README-only CI 자원 절감과 필수 검사 보존

- **완료 근거:** [CI Issue #513](https://github.com/july-rust/BoardSI/issues/513) **Closed** · [코드 PR #516](https://github.com/july-rust/BoardSI/pull/516) →
  [main 4618ce2](https://github.com/july-rust/BoardSI/commit/4618ce2220eccabeb36174b482fee57a1bc705da) (필수 **12개 PR 워크플로 SUCCESS**, 신규 분류기 회귀시험 **8개 PASS**) ·
  [실증 문서 PR #517](https://github.com/july-rust/BoardSI/pull/517) → [main 0faed41](https://github.com/july-rust/BoardSI/commit/0faed4133e2cb726860aa021b4c95db9bc900a2a) (**11개 PR 워크플로 SUCCESS**, 최종 main
  Push **8개 SUCCESS**). 실제 컴포넌트 README만 바뀐 PR에서 **Qt/Rust/FEM/F6/F7 무거운 작업 SKIPPED**, 경량 분류기와 Docs Check SUCCESS. [F6 실증](https://github.com/july-rust/BoardSI/actions/runs/42379930266) ·
  [F7 실증](https://github.com/july-rust/BoardSI/actions/runs/42379930151) · [Qt 실증](https://github.com/july-rust/BoardSI/actions/runs/42379930142)
- **검증 경계/미확인:** 코드·JSON·YAML·Fixture·Rename·SHA 불명확시 **Fail-Closed 전체 검증 실행**. 과거 중간 main 4618ce2 Push의 Qt/Numerical **CANCELLED 2개**는 PASS로 집계하지 않음; 최종 main 0faed41의 8/8 SUCCESS와 구분

**관리 원칙:** 구현 상태는 기존 상세 원장의 최초 체크일·근거일·원본 SHA 기준을 유지합니다. 이 표는 *문서와 운영 변경의 추적성만* 관리합니다. 후속 문서 추가는 [문서 허브](README.md)·[검증 색인](validation/INDEX.md)에 반영하고, 실제 기능 구현과 U/T 수락에 영향이 있을 때만 위 A/B/C/D 또는 제품 인수의 공식 체크
  상태를 변경합니다.

<a id="stage-a"></a>
### Stage A — 제한 Preview / Import / Artwork

**12건 확인 · 5건 미확인.** [A_001–A_036 전번호 역사 색인](#a-history-ledger)을 별도로 복원했습니다. 공개 합성 PADS Import subset, 제한 Qt Artwork/Worker IPC 및 Native CI 증거를 확보했습니다. 미확인 핵심은 상용 PADS 전체 지원, 실 PCB 전기 접속, 실제 SI Solver, GPU
  실기기 성능, 설치/라이선스입니다. [A 단계 근거와 미확인 체크 항목](#2-a시리즈-구현-확인-목록)

<a id="stage-b"></a>
### Stage B — Stackup / Copper / Reference

**17건 확인 · 6건 미확인.** 실제 Reference/Return Path 전체 인수는 아직 완료되지 않았습니다.

| 작업 | 제한 검증 범위 | 확인 | 미확인 |
| :--- | :--- | ---: | ---: |
| [**B_001**](#b_001) | Stackup·Reference 제한 검증 (B_002 별도) | 3 | 1 |
| [**B_002**](#b_002) | Qt 감사 최신성·Rust Scenario Revision 제한 검증 | 2 | 0 |
| [**B_003**](#b_003) | 제한 완료 — Trace/Pad/Via 원본 접촉 근거 (Issue #12 연속 작업) | 1 | 1 |
| [**B_004**](#b_004) | 제한 완료 — 단일 단순 Copper Area 국소 접촉 검증 | 1 | 0 |
| [**B_005**](#b_005) | 제한 완료 — Source-bound Reference 기하 커버리지 | 2 | 0 |
| [**B_006**](#b_006) | 제한 완료 — Source-bound Through Via/COPCLS 국소 접촉 감사 | 2 | 0 |
| [**B_007**](#b_007) | COPCUT-aware bounded local contact implementation and integration | 2 | 2 |
| [**B_008**](#b_008) | 단일 COPCUT Reference 경로 기하 감사 | 2 | 0 |
| [**B_009**](#b_009) | B-series 제한 범위 최종 감사 및 C 단계 인계 | 2 | 2 |

<a id="stage-c"></a>
### Stage C — Reference 기하 / Qt 통합 / 감사

**40건 확인 · 8건 미확인.** Source-bound 합성 사례 및 제한 통합 회귀의 증거이고, 미해결 완료 조건은 [상세 원장](#3-후속-개발-체크리스트)에서 관리합니다.

| 작업 | 제한 검증 범위 | 확인 | 미확인 |
| :--- | :--- | ---: | ---: |
| [**C_001**](#c_001) | 다중 Source-owned COPCUT / Disconnected Island | 1 | 0 |
| [**C_002**](#c_002) | Reference Split 및 Layer Transition Topology Risk | 2 | 1 |
| [**C_003**](#c_003) | Immutable Scenario and Stale Result | 1 | 1 |
| [**C_004**](#c_004) | Material Provenance / Loss Input Gate | 2 | 0 |
| [**C_005**](#c_005) | Qt Source Reference Risk 표시/무효화 제한 인수 | 1 | 0 |
| [**C_006**](#c_006) | Source-exact Reference Gap Interval / Qt CPU/RHI 제한 기능 완료 | 1 | 0 |
| [**C_007**](#c_007) | Multi-Segment Source Reference Route / Qt Risk | 1 | 0 |
| [**C_008**](#c_008) | Two-Layer Source Plated Via / Reference Gap Qt Risk | 1 | 0 |
| [**C_009**](#c_009) | Source Reference Stitching Via Candidate 및 Qt Risk 표시 | 1 | 0 |
| [**C_010**](#c_010) | Reference Copper Corridor / Stitch Source Area ID 정합성 | 1 | 0 |
| [**C_011**](#c_011) | Source Stitch Via Pad Footprint V7 (한정 코드 범위 완료) | 1 | 0 |
| [**C_012**](#c_012) | Source Signal/Stitch Via Pad Clearance V8 (한정 코드 범위 완료) | 1 | 0 |
| [**C_013**](#c_013) | Source Signal Via Pad / Reference COPCUT V9 | 1 | 0 |
| [**C_014**](#c_014) | Multi-COPCUT Signal Via / Reference Source geometry V10 | 1 | 0 |
| [**C_015**](#c_015) | Circular COPCCO / rectangular COPCUT mixed Source Geometry V11 | 1 | 0 |
| [**C_016**](#c_016) | Reference Stitch Via full circular Pad / mixed cutout Source V12 | 1 | 0 |
| [**C_017**](#c_017) | Reference Gap/Layer Transition V1–V4 한정 통합 안정화 | 1 | 0 |
| [**C_018**](#c_018) | Reference Stitch Via / Copper Corridor V5–V8 제한 통합 회귀 인수 | 1 | 0 |
| [**C_019**](#c_019) | PADS Mixed Cutout / Signal and Reference Via Pad V9–V12 한정 통합 회귀 인수 | 1 | 0 |
| [**C_020**](#c_020) | B_007 최종 리뷰 증거와 B_003–B_008 Connectivity 제한 감사 | 3 | 0 |
| [**C_021**](#c_021) | Source Reference Copper Area-to-Area 제한 감사 | 1 | 0 |
| [**C_022**](#c_022) | PADS Multi-Area Source Copper Component Candidates V14 | 2 | 0 |
| [**C_023**](#c_023) | Multi-Area Reference Corridor V15 | 1 | 0 |
| [**C_024**](#c_024) | C단계 통합 증거 감사 및 D단계 인계 | 1 | 0 |
| [**C_025_01**](validation/c02507-open-issue-d-stage-handoff.md) | A_023 Copper/COPCUT Issue #177 한정 감사·종료 | — | — |
| [**C_025_02**](validation/sec001-codeql-markdown-comment-scan.md) | SEC_001 CodeQL Alert #1 Fixed 검증·Issue #205 종료 (보안에 별도 집계) | — | — |
| [**C_025_03**](#c_025_03) | 구형 PADS Signature/Qt Import PR 정합화 및 안전 종료 (기능 인수와 별개) | 2 | 0 |
| [**C_025_04**](#c_025_04) | 구형 정책·B_003 Draft PR 정합화 및 안전 종료 (문서·PR 관리 한정) | 2 | 0 |
| [**C_025_05**](#c_025_05) | M0 F1 사양서/U-T 추적성 및 Issue #2 완료 조건 감사 | 1 | 1 |
| [**C_025_06**](validation/c02506-pre-ph08-audit-issue78-closeout.md) | 전체 프로젝트/PH-08 이전 감사·Issue #78 한정 종료 | — | — |
| [**C_025_07**](validation/c02507-open-issue-d-stage-handoff.md) | 27개 Open Issue/PR 감사·D 단계 인계·Issue #323 종료 | — | — |
| [**C_025_08**](validation/c02508-m0-architecture-issue3-acceptance.md) | M0 Rust/Qt/IPC 결정·감사 Issue #325 종료, 원래 Issue #3 OPEN | — | — |
| [**C_025_09**](#c_025_09) | macOS Qt 글로벌 타이머 지연 분리 진단 및 #46/#41 종료 안전성 | 2 | 2 |
| [**C_025_10**](#c_025_10) | M0 F6 공개 Fixture 증거 및 Issue #7 안전 판정 | 4 | 3 |

> **C_025 누락 행 복원 (2026-11-03 KST):** _01/_02/_06/_07/_08 작업은 누락된 개발 단계가 아니라 기존 검증 원장·Issue/PR·변경 이력에 존재했던 제한 감사 및 행정 정리입니다. 과거 통합 현황판에서 별도의 체크리스트 체크 항목이 없다는 이유로 이 다섯 줄이 빠졌습니다. 표의 `—`는 **독립 체크 항목을 다시 만들거나 완료 건수를 중복 계산하지 않음**을 뜻합니다. 특히 C_025_02의 `[x]` 근거는 기존 SEC_001 집계(1건)에 포함됐고, C_025_08 감사 종료에도 원래 M0 [Issue #3](https://github.com/july-rust/BoardSI/issues/3)은 OPEN입니다. 각 행 링크와 기존 변경 이력이 정확한 근거입니다.

<a id="stage-d"></a>
### Stage D — PADS Source Contact / Junction

**95건 확인 · 8건 미확인 (D_056 단계별 작업 진행 중, SOURCE 한정 구현과 전체 인수 구별).** D_054/D_055는 각도·접속 위치별 개별 분기 추가를 중단하고, 임의 방향 직선/원호 Trace와 Via의 도금 환형 패드 및 SMD 원형·사각 Pad 전체 구리 면적을 사용하는 공통 SOURCE 기하 판정 엔진을 Rust에 구현했습니다. 실제
  PADS 원본 2종 오프센터/모서리 변형과 신규 D_055 테스트 10종을 Debug/Release에서 PASS했으며 PR 필수 CI 모두 PASS입니다. 그러나 **국소 기하 후보만 한정 확인**한 것이고 실제 Qt 전체 연결 그래프·Open/Short·SI 계산과 U/T 공식 인수는 여전히 미검증입니다. D_053은 실제 SOURCE Repair Trace를
  독립 레코드로 추가해 D_052의 두 단선을 한쪽 또는 양쪽에서 국소적으로 복구하는 6개 합성 상태를 Rust와 Native Qt에서 한정 검증했습니다. BREAK_BOTH는 14/12/2/0, 단일 Repair는 15/14/1/0, 양쪽 Repair는 16/16/1/1, 단일 단선 복구는 14/14/1/1 (Nodes/Edges/Components/Cycle
  Rank)입니다. 완전한 SOURCE SHA-256·Inventory·양 끝 Copper 양의 접촉·Fail-Closed를 검증했지만 실물·전역 전기 도통은 NOT_VERIFIED입니다.

D_052는

D_051의 가상 Bridge Node 제거를 실제 PADS V9.4 SOURCE의 중간 Copper 간격(폭 8 mil, 중심선 간격 20 mil, Copper 간격 12 mil)으로 확장해 INTACT/Break-A/Break-B/Break-Both의 SOURCE 국소 연결성을 Rust·독립 Native Qt에서 한정 검증했습니다. 그래프는 12/12/1/1,
  13/12/1/0, 13/12/1/0, 14/12/2/0 (Nodes/Edges/Components/Cycle Rank)이고, 실물 전기적 도통·전역 Open/Short는 NOT_VERIFIED입니다.

D_051은 동일층 혼합 원형·정사각형 SMD Pad에 두 개의 독립 유한 선폭 Bridge를 구성하고 12 Nodes / 12 Same-Layer Candidate Edges / 1 Component / Cycle Rank 1 및 단일 Bridge 가상 제거 후 연결 유지, 양 Bridge 제거 후 2 Components를 Rust·독립 Native Qt에서
  SOURCE 국소 기하 한정 검증했습니다. 실물 전기적 도통은 NOT_VERIFIED입니다.

D_050은

D_049의 분리된 혼합 원형·정사각형 SMD Pad 국소 Component 두 개를 폭 8 mil의 명시적인 동일 Layer SOURCE Bridge Trace 하나로 접촉시켜 **11 Nodes / 10 Same-Layer Candidate Edges / 1 Local Component**를 Rust·Native Qt에서 독립 검증했습니다.

D_049는 D_047/D_048의 원형·정사각형 SMD Pad와 별개 SOURCE Trace 접촉 후보 둘을 하나의 원본 PADS에서 독립 검증하여 10 Nodes/8 Edges/2 Local Components로 결합합니다. 인수는 SOURCE-local 한정이고 실제 전기적 도통은 NOT_VERIFIED입니다.

D_048은 SOURCE 소유 동일 Layer의 원형 SMD Pad와 유한 선폭 둥근 끝단 Trace 간 양의 면적 중첩을 정확한 최소거리로 한정 검증합니다.

D_047은 SOURCE 소유 동일 Layer의 8 mil 유한 선폭 Trace가 SMD 정사각형 Pad에 양의 면적으로 중첩되는 후보를 정확한 선분-직사각형 거리로 검증합니다.

D_046은 비직교·비평행 두 Trace의 선폭 중첩을 정확한 SOURCE Tick 기반 최소 선분 거리로 한정 검증합니다.

D_045는 서로 직교하는 Trace의 중심선이 만나지 않아도 둥근 끝단 Copper 선폭이 양의 면적으로 중첩되는 한정 SOURCE 후보를 정확한 최소거리 제곱으로 검증합니다.

D_044의 Qt 적대적 변형 재Import 누락도 독립 회귀 테스트에서 보완했습니다.

D_044는 동일 층의 서로 다른 Trace 중심선 사이에 생기는 양의 폭 접촉 후보를 둥근 끝단의 정확한 거리 제곱으로 한정 검증합니다. 단순 AABB 교차나 실물 전기 도통은 인수하지 않습니다.

D_043은

D_042의 완전한 SOURCE를 먼저 검증한 뒤 별도 진단에서 Trace 노드·Incident Edge를 제외하여 12개 가상 단선 시나리오의 국소 Component/Cycle/Reachability 변화를 검증합니다. 실제 Copper 파손·전기적 Open/Short는 검증하지 않습니다.

D_042는 D_041 직접 Bridge에 5개 SOURCE-owned 우회 Trace를 추가하여 하나의 국소 Closed Loop(14 Nodes/14 Edges/1 Component/1 Cycle)가 형성되는 정확한 지원 기하만 검증합니다.

D_041은 독립 Component 2개의 국소 병합,

D_040은 두 독립 Component의 분리 검증입니다.

D_039를 포함한 Stage D의 체크는 지원된 공개 합성 PADS Source의 한정 결과이며 제조 실물 도통·글로벌 Open/Short·Return Path/SI를 인증하지 않습니다.

| 작업 | 제한 검증 범위 | 확인 | 미확인 |
| :--- | :--- | ---: | ---: |
| [**D_001**](#d_001) | Trace endpoint / conductive SMD Pad footprint Source witness | 2 | 1 |
| [**D_002**](#d_002) | Source COPCUT-aware two-area hypothesis | 1 | 0 |
| [**D_003**](#d_003) | Three-Area COPCUT Source Geometry Candidate | 1 | 0 |
| [**D_004**](#d_004) | Source Trace Endpoint / Three-Area Candidate Attachment | 1 | 0 |
| [**D_005**](#d_005) | SMD Pad → Trace → Three-Area COPCUT Source candidate | 2 | 1 |
| [**D_006**](#d_006) | Plated Via Pad → Three-Area COPCUT Source contact candidate | 3 | 0 |
| [**D_007**](#d_007) | Plated Via Pad → Straight Trace → Three-Area COPCUT Source chain | 3 | 0 |
| [**D_008**](#d_008) | Plated Through Via Pad → Dual-Layer Source COPCLS/COPCUT contacts | 3 | 0 |
| [**D_009**](#d_009) | Plated Through-Via Dual-Layer Pad → Layer-B Straight Trace → Third COPCLS | 2 | 0 |
| [**D_010**](#d_010) | Plated Through-Via Dual-Layer / Two-Trace / Four-Area Source qualification | 1 | 0 |
| [**D_011**](#d_011) | PADS Dual-Layer Source Contact Candidate Graph | 2 | 0 |
| [**D_012**](#d_012) | Through-Via Source Span & Plating Provenance Witness | 3 | 0 |
| [**D_013**](#d_013) | PADS Legacy V1 Barrel Compatibility Provenance & Fail-Closed Qualification | 3 | 0 |
| [**D_014**](#d_014) | PADS Two-Via Source Contact Chain & Fail-Closed Qualification | 3 | 0 |
| [**D_015**](#d_015) | PADS Same-Layer Trace T-Junction Source Contact & Fail-Closed Qualification | 3 | 0 |
| [**D_016**](#d_016) | PADS Same-Layer Trace X-Junction SOURCE Qualification | 3 | 0 |
| [**D_017**](#d_017) | PADS Same-Layer Oblique Trace X-Junction SOURCE Qualification | 3 | 0 |
| [**D_018**](https://github.com/july-rust/BoardSI/pull/396) | 문서·Docs-only Native CI 최적화 (별도 체크 항목 미집계) | — | — |
| [**D_019**](#d_019) | Same-Layer Oblique T-Junction Source 접촉 후보 (한정 검증 완료) | 1 | 0 |
| [**D_020**](#d_020) | Same-Layer Strictly Contained Collinear Overlap Source 후보 (한정 검증 완료) | 1 | 0 |
| [**D_021**](#d_021) | Same-Layer Partial Collinear Overlap Source 후보 (한정 검증 완료) | 1 | 0 |
| [**D_022**](#d_022) | Same-Layer Collinear Endpoint-to-Endpoint Source 후보 (한정 검증 완료) | 1 | 0 |
| [**D_023**](#d_023) | Opposite Via-A Collinear Endpoint Source 후보 (한정 코드 인수 완료) | 1 | 0 |
| [**D_024**](#d_024) | Orthogonal Via-B L-Bend Endpoint Source 후보 (한정 검증 완료) | 1 | 0 |
| [**D_025**](#d_025) | Orthogonal Via-A L-Bend Endpoint Source 후보 (한정 코드·Push 인수 완료) | 1 | 0 |
| [**D_026**](#d_026) | 45° Oblique Via-B L-Bend Endpoint Source 후보 (한정 코드·Push 인수 완료) | 1 | 0 |
| [**D_027**](#d_027) | 45° Oblique Via-A L-Bend Endpoint Source 후보 (한정 코드·Push 인수 완료) | 1 | 0 |
| [**D_028**](#d_028) | Non-45° exact 4:3 Via-B L-Bend Endpoint Source 후보 (한정 코드·main Push 인수 완료) | 1 | 0 |
| [**D_029**](#d_029) | Non-45° exact 4:3 Via-A L-Bend Endpoint Source 후보 (코드·문서 Push 한정 인수 완료) | 1 | 0 |
| [**D_030**](#d_030) | Non-45° exact steep 3:4 Via-B Source 후보 (code-main Push 인수 완료) | 1 | 0 |
| [**D_031**](#d_031) | Exact steep 3:4 Via-A Source 후보 (코드·문서·main Push 한정 인수 완료) | 1 | 0 |
| [**D_032**](#d_032) | Mirrored negative steep 3:4 Via-B SOURCE 후보 (코드·문서·main Push 한정 인수 완료) | 1 | 0 |
| [**D_033**](#d_033) | Mirrored negative steep 3:4 Via-A SOURCE 후보 (코드·main Push 한정 인수 완료) | 1 | 0 |
| [**D_034**](#d_034) | Mirrored negative shallow 4:3 Via-B SOURCE 후보 (코드·main Push 한정 인수 완료) | 1 | 0 |
| [**D_035**](#d_035) | Mirrored negative shallow 4:3 Via-A SOURCE 후보 (코드·main Push 한정 인수 완료) | 1 | 0 |
| [**D_036**](#d_036) | Dual mirrored negative shallow 4:3 Via-A/Via-B SOURCE 복합 후보 (코드·main Push 한정 인수 완료) | 1 | 0 |
| [**D_037**](#d_037) | Mixed-sign negative Via-A / positive Via-B shallow 4:3 SOURCE 복합 후보 (코드·main Push 한정 인수 완료) | 1 | 0 |
| [**D_038**](#d_038) | Mixed-sign positive Via-A / negative Via-B shallow 4:3 SOURCE 복합 후보 (코드·main Push 한정 인수 완료) | 1 | 0 |
| [**D_039**](#d_039) | Same-layer SMD Pad–Trace–Trace–SMD Pad 국소 Copper Geometry Graph (코드·main Push 한정 인수 완료) | 1 | 0 |
| [**D_040**](#d_040) | Same-layer Multi-Component Copper Isolation / Net Separation (코드·main Push 한정 인수 완료) | 1 | 0 |
| [**D_041**](#d_041) | Same-layer SOURCE Bridge Trace / Local Copper Component Merge (코드·main Push 한정 인수 완료) | 1 | 0 |
| [**D_042**](#d_042) | Same-layer SOURCE Closed Loop / Redundant Path (코드·main Push 한정 인수 완료) | 1 | 0 |
| [**D_043**](#d_043) | Same-layer SOURCE Virtual Trace Cut-Set / Redundancy (코드·main Push 한정 인수 완료) | 1 | 0 |
| [**D_044**](#d_044) | Finite-width parallel SOURCE Copper contact candidate (코드 한정 인수) | 1 | 0 |
| [**D_045**](#d_045) | Finite-width perpendicular SOURCE Copper contact + D_044 native Qt negative reimport (코드 한정 인수) | 1 | 0 |
| [**D_046**](#d_046) | Finite-width oblique SOURCE Copper contact / exact segment-distance Rust·Native Qt qualification (코드·main Push 한정 인수) | 1 | 0 |
| [**D_047**](#d_047) | Finite-width SOURCE Trace-to-SMD Square Pad positive-area contact / independent Rust·Native Qt | 1 | 0 |
| [**D_048**](#d_048) | Finite-width SOURCE Trace-to-SMD Round Pad positive-area contact / independent Rust·Native Qt (code-main Push accepted) | 1 | 0 |
| [**D_049**](#d_049) | Mixed Round/Square SMD Pad dual finite-width SOURCE-local contacts / independent Rust·Native Qt | 1 | 0 |
| [**D_050**](#d_050) | Mixed Round/Square SMD Pad SOURCE finite-width inter-component bridge / independent Rust·Native Qt | 1 | 0 |
| [**D_051**](#d_051) | Mixed-pad dual finite-width bridge SOURCE-local redundant loop / virtual cut-set / independent Rust·Qt | 1 | 0 |
| [**D_052**](#d_052) | PADS SOURCE-real dual midspan copper break / Rust + Native Qt | 1 | 0 |
| [**D_053**](#d_053) | PADS SOURCE-real dual finite-width midspan repair / Rust + Native Qt | 1 | 0 |
| [**D_054**](#d_054) | General-angle finite-width straight/circular SOURCE Trace local contact Rust engine (code/PR regression limited) | 1 | 0 |
| [**D_055**](#d_055) | Off-centre straight/circular Trace to plated Via annulus and round/square/rect SMD Pad copper footprint (code/PR regression limited) | 1 | 0 |
| [**D_056**](#d_056) | 범용 Layer/Span + PADS 원형·사각·환형 구리/R·S·A 및 RA/SA 안티패드 (코드 #521/#523/#526 인수, 전체 입력 미완료) | 0 | 1 |
| [**D_056_01**](#d_056_01) | 기존 코드 감사·Source Evidence 계약과 후속 12단계 검증 게이트 (문서 전용 PR #533 / Verified main Push 인수) | 1 | 0 |
| [**D_056_02**](#d_056_02) | PADS Source 증거 기반 다층 Copper 후보 Graph (코드 PR #535, Verified main Push 한정 인수) | 1 | 0 |
| [**D_056_03**](#d_056_03) | PADS SOURCE 기반 다층 전기 설계 Graph·Open 후보/Cut-set 제한 인수 (45 독립 Oracle) | 1 | 0 |
| [**D_056_04**](#d_056_04) | SOURCE SHA 결합 Canonical Padstack/Via Rust→Qt IPC V1 독립 한정 인수 (PR #549) | 1 | 0 |
| [**D_056_05**](#d_056_05) | Main Qt CPU/RHI Layer별 Via/Padstack/Drill Artwork, 원본 SOURCE SHA, Native 4플랫폼·Legacy V1~V14 소프트웨어 한정 인수 (PR #551) | 1 | 0 |
| [**D_056_06**](#d_056_06) | PADS RT/ST Thermal SOURCE/Canonical V2 · Qt 한정 구현 | 2 | 1 |
| [**D_056_07**](#d_056_07) | PADS PARTDECAL PTH/NPTH/Slot Source·제조 의도, 전체 Native 미인수 | 2 | 2 |
| [**D_056_08**](#d_056_08) | ODB++ SOURCE L/P/A/S·EDA Net·Drill tools/hits·USTAR·Hole 관측 통합 (전체 미인수) | 8 | 1 |
| [**D_056_09**](#d_056_09) | ODB++ 잔여 전체 인수 OPEN; ZIP STORED SOURCE 코드 PR 30/30 · code-main Push 8/8 한정 인수 | 2 | 1 |

<a id="stage-security"></a>
### 보안 — SEC_001

CodeQL `py/bad-tag-filter` Alert #1의 **Fixed 한정 증거 1건**이 있으며 전체 제품 보안 인수가 아닙니다. [검증 상세](#sec_001-실제-code-scanning-상태-인수--전체-제품-보안-인수와-별개)

<a id="stage-milestones"></a>
### 제품 마일스톤과 제한 검증의 차이

| 마일스톤 | 현재 해석 |
| :--- | :--- |
| **M0** | 구조/기술 일부 시연; 제품 종료 Gate는 BLOCKED |
| **M1 / M1a / M1b** | PADS Import/Reference 후보·Via/Pad/Port 일부만 입증; 정식 미완 |
| **M2 / M2a–c** | 합성 FEM 연구와 제품 SI/RLGC/S-parameter 인수 별개; 제품 미완 |
| **M3–M5** | CAD 확장과 고급 기능은 후속 요구사항 |

<a id="stage-acceptance"></a>
### U01–U48 / T01–T54 정식 인수

| 체계 | 확인 | 미확인 | 세부 |
| :--- | ---: | ---: | :--- |
| **U 요구사항** | 1 | 47 | [요구사항 원장](#4-요구사항-u01u48-체크리스트) |
| **T 인수시험** | 0 | 54 | [시험 원장](#5-인수시험-t01t54-체크리스트) |

**다음 핵심 미완:** PADS 전체 물리 도체/Pad/Pour/Thermal 연결, Reference Plane Split/Cutout Return Path, 검증된 RLGC/S Solver 및 DC–30 GHz, 실기기 배포와 U/T 전체 정식 인수.

---

## 목차

- [현재 현황](#1-현재-현황) · [A_001~A_036 색인](#a-history-ledger) · [A 기능 확인](#2-a시리즈-구현-확인-목록) · [B/C/D](#3-후속-개발-체크리스트) · [E 계획](#stage-e)
- [U01–U48](#4-요구사항-u01u48-체크리스트) · [T01–T54](#5-인수시험-t01t54-체크리스트) · [관리 규칙](#6-갱신-및-완료-판정-규칙)

## 1. 현재 현황

| 영역 | 확보된 증거 | 정식 완료 여부 |
| --- | --- | --- |
| A_001–A_035 P2 Preview | 제한된 PADS V9.4 MILS 합성 원본 Import→Canonical/IPC→Qt CPU/RHI Artwork; 3 Native CI ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md)) | **부분 기능 PASS**, 범용 PADS/실 PCB 해석 아님 |
| M0 구조·기술 | Rust/Qt·IPC·Canonical·3 Native 프로토타입 ([M0 종료 게이트](validation/m0-completion-gate.md)) | **BLOCKED**: F1/F6/F7 등 독립 종료 게이트 남음 |
| M1 PADS/Artwork | PADS subset, Copper/Cutout, Source parity, 제한 국소 Route 접점 ([A_034 Native 통합](validation/a034-p2y-integration-stabilization.md)) | **PARTIAL**, ODB++ 및 전체 Connectivity 미완 |
| M1 Stackup/Reference (B_001) | Qt Overlay UI + Rust Canonical **사용자 가정 Material 전용** 감사 PASS ([#213](https://github.com/july-rust/BoardSI/pull/213), [#215](https://github.com/july-rust/BoardSI/pull/215)); 실제 Copper/Net/Return Path·물리 Solver 미검증 | **두 제한 기능 PASS / M1 정식 미완** |
| M1a / M1b | Via/Pad 일부 기하 연구, 실 Port 자동/수동 전체 범위 증빙 없음 | **미완** |
| M2 연구·제품 Solver | PEC/lossless FEM 합성 쿠폰 수학 검증만 존재 ([M2 검증 원장](validation/README.md)) | **제품 Solver 미완**; PH-08 원본 Gate Q **FAIL 유지** |
| M2a/M2b/M2c, M3–M5 | 단계별 요구 명세 있음; 일부 기술 시연만 존재 | **미완 / 후속** |
| GPU 렌더 | 제한 RHI Native Frame/Picking 회귀 | 고객 GPU 드라이버·VRAM·Reset·1M@60FPS **미검증** |
| Security / 출시 | SEC_001 코드 수정 [PR #206](https://github.com/july-rust/BoardSI/pull/206), [CodeQL Alert #1 실제 Fixed 읽기 감사](https://github.com/july-rust/BoardSI/actions/runs/41183754657), [SEC_001 기록](validation/sec001-codeql-markdown-comment-scan.md) | **SEC_001 경고 #1 Fixed 직접 확인** (2026-10-26 08:25:02 UTC 수정, 2026-10-31 08:14:14 UTC 재확인), 동일 규칙 미해결 Open 경고 0; **전체 보안/라이선스·배포 미완** |

**날짜 기준:** 모든 `[x]`는 `체크일`(처음 체크리스트에 기록된 날짜, KST)과 `근거일`(참조 PR의 병합일 또는 사양서 개정일, KST)을 별도로 표시합니다. 최초 체크 13개는 2026-10-26에 등록했습니다. 선행 근거일이 전체 제품 인수 완료를 의미하지는 않습니다.

**표시 기준:** `[x]`는 **체크된 문장에 명시된 한정된 범위만 완료**를 뜻합니다. `부분 증빙`은 최종 인수 미완입니다. `[ ]`를 `[x]`로 변경하려면 해당 범위의 실행 결과·원본·SHA·리뷰·미해결 문제를 대조해야 합니다. 사양서에서 `확정`된 요구도 개발 완료로 자동 처리하지 않습니다.

## 2. A시리즈 구현 확인 목록

<a id="a-history-ledger"></a>
### A_001~A_036 전체 단계 번호·과거 근거 색인

> **복원일: 2026-11-11 KST.** 이는 과거 A 단계를 모두 **보이게 만든 역사 인벤토리**이며, 검증 없는 A 번호를 새로 [x] 처리하는 표가 아닙니다. A_035의 합성 Preview 총괄 PASS는 A_001–A_034 각각의 별도 CI PASS를 자동 증명하지 않습니다. A_036은 체크리스트 도입 관리 단계입니다. 개별 코드 출처만 확인된 항목에는 **개별 인수 미확인**이라고 명시합니다.
>
> 기존 Stage A의 **12건 제한 확인·5건 미확인**은 아래 36개 번호의 완료 통계가 아닌 **이 파일의 기존 17개 명시적 기능 체크 항목 집계**입니다. [A_035 최종 Preview 제한 감사](validation/a035-p2z-final-preview-acceptance.md) · [검증 보고서 색인](validation/INDEX.md).

| 번호 | 식별된 범위 (Source에서 못 찾은 것은 불명으로 표시) | 직접 근거 | 상태 해석 |
| :--- | :--- | :--- | :--- |
| **A_001** | A_001–A_034 통합 범위 외 개별 완료 증거 미확인 | [A_035 범위 감사](validation/a035-p2z-final-preview-acceptance.md) | 개별 단계 증거 미확인 |
| **A_002** | 개별 단계명·독립 인수 근거 미확인 | [A_035 통합 감사](validation/a035-p2z-final-preview-acceptance.md) | 개별 검증 미확인 |
| **A_003** | 개별 단계명·독립 인수 근거 미확인 | [A_035 통합 감사](validation/a035-p2z-final-preview-acceptance.md) | 개별 검증 미확인 |
| **A_004** | 개별 단계명·독립 인수 근거 미확인 | [A_035 통합 감사](validation/a035-p2z-final-preview-acceptance.md) | 개별 검증 미확인 |
| **A_005** | PADS V9.4 P2-A Parser 보완 | [Importer README](../crates/boardsi-import/README.md) | 소스·설명 흔적; 독립 전체 인수 미확인 |
| **A_006** | 개별 단계명·독립 인수 근거 미확인 | [A_035 통합 감사](validation/a035-p2z-final-preview-acceptance.md) | 개별 검증 미확인 |
| **A_007** | PADS V9.4 명세 부합 Parser 보완 | [Importer README](../crates/boardsi-import/README.md) | 소스·설명 흔적; 독립 전체 인수 미확인 |
| **A_008** | 개별 단계명·독립 인수 근거 미확인 | [A_035 통합 감사](validation/a035-p2z-final-preview-acceptance.md) | 개별 검증 미확인 |
| **A_009** | 개별 단계명·독립 인수 근거 미확인 | [A_035 통합 감사](validation/a035-p2z-final-preview-acceptance.md) | 개별 검증 미확인 |
| **A_010** | 제한 PADS Preview Parser | [Importer README](../crates/boardsi-import/README.md) | 구현 흔적; 개별 완료 근거 별도 감사 필요 |
| **A_011** | 제한 Component/Pad Preview | [part_preview.rs](https://github.com/july-rust/BoardSI/blob/main/crates/boardsi-import/src/part_preview.rs) | 구현 근거 존재; 독립 인수 미확인 |
| **A_012** | 제한 직사각형 SMD Finger Preview | [Importer README](../crates/boardsi-import/README.md) | 구현 흔적; 독립 전체 인수 미확인 |
| **A_013** | GPU 우선 Renderer 정책 결정 | [Qt README](../apps/boardsi-qt-prototype/README.md) | 정책·구현 흔적; GPU 실기기 인수 아님 |
| **A_014** | Qt Quick/RHI Artwork 부분 구현 | [RHI 소스](https://github.com/july-rust/BoardSI/blob/main/apps/boardsi-qt-prototype/rhi_artwork.h) | 부분 코드·CI 흔적; 실제 GPU 성능 미확인 |
| **A_015** | PADS Arc Preview/Worker→Qt | [Qt 소스](https://github.com/july-rust/BoardSI/blob/main/apps/boardsi-qt-prototype/preview_p0.cpp) | 부분 코드·CI 흔적; 범용 입력 미검증 |
| **A_016** | PADS Board Outline Preview | [outline_preview.rs](https://github.com/july-rust/BoardSI/blob/main/crates/boardsi-import/src/outline_preview.rs) | 부분 코드 증거, 개별 인수 미확인 |
| **A_017** | Qt Outline/Preview 한정 회귀 | [Qt 소스](https://github.com/july-rust/BoardSI/blob/main/apps/boardsi-qt-prototype/preview_p0.cpp) | 명칭·개별 수락 게이트 재감사 필요 |
| **A_018** | Quarter-Arc 기하 Preflight | [outline_arc.rs](https://github.com/july-rust/BoardSI/blob/main/crates/boardsi-import/src/outline_arc.rs) | 소스 범위 확인, 별도 전체 수락 미검증 |
| **A_019** | Synthetic 10K/100K/1M GPU/Qt Benchmark | [a019 benchmark](https://github.com/july-rust/BoardSI/blob/main/apps/boardsi-qt-prototype/a019_gpu_benchmark.h) | 합성 실행 범위; VRAM/고객 GPU 성능 미증명 |
| **A_020** | Layer/Net Explorer Preview | [a020 explorer](https://github.com/july-rust/BoardSI/blob/main/apps/boardsi-qt-prototype/a020_explorer_validation.h) | 합성/GUI 경로, 범용 기능 인수 아님 |
| **A_021** | POUR/POUROUT Source Preflight | [pour_preview.rs](https://github.com/july-rust/BoardSI/blob/main/crates/boardsi-import/src/pour_preview.rs) | 전기적 Pour 연결성 NOT_VERIFIED |
| **A_022** | Source COPCLS/COPCIR 한정 Copper | [A_031 지원 범위 매트릭스](validation/a031-p2v-copper-stabilization.md) | 지원 부분집합, 물리 Copper 미검증 |
| **A_023** | Source 직사각형 COPCUT | [A_031 지원 범위 매트릭스](validation/a031-p2v-copper-stabilization.md) | Source 부분집합; 원본 범용 미검증 |
| **A_024** | Source Orthogonal Concave Copper/Cutout | [A_025 선행 근거](validation/a025-p2p-diagonal-copper-preflight.md) | Source 부분집합; 개별 현재 수락 재감사 필요 |
| **A_025** | Nonorthogonal Copper/Cutout Preflight | [A_025 검증](validation/a025-p2p-diagonal-copper-preflight.md) | 개별 날짜 고정 보고서 존재; 한정 범위 |
| **A_026** | Circular Copper Void Preflight | [A_026 검증](validation/a026-p2q-circular-copper-preflight.md) | 개별 보고서 존재; 한정 범위 |
| **A_027** | Hollow Circular Copper | [A_027 검증](validation/a027-p2r-hollow-circular-copper.md) | 개별 보고서 존재; 한정 범위 |
| **A_028** | Circle/Polygon Mixed Copper | [A_028 검증](validation/a028-p2s-circle-polygon-mixed-source-copper.md) | 개별 보고서 존재; 한정 범위 |
| **A_029** | Curved Copper Preflight | [A_029 검증](validation/a029-p2t-curved-copper-preflight.md) | 개별 보고서 존재; 한정 범위 |
| **A_030** | Curved Copper Polygon Cutout | [A_030 검증](validation/a030-p2u-curved-copper-polygon-cutout.md) | 개별 보고서 존재; 한정 범위 |
| **A_031** | Copper Stabilization | [A_031 검증](validation/a031-p2v-copper-stabilization.md) | 과거 보고서 작성 당시 CI PENDING; 최신 통합 증거와 대조 필요 |
| **A_032** | Local Connectivity/Remediation/Qt Contact | [A_032 주요 검증](validation/a032-p2w-local-connectivity.md), [보완 1](validation/a032-1-remediation.md), [보완 2](validation/a032-2-contact-viewer.md) | Source 국소 후보, 보드 전역 Open/Short 아님 |
| **A_033** | Source↔Artwork Parity | [A_033 검증](validation/a033-p2x-source-artwork-parity.md) | Source subset만 증명 |
| **A_034** | Native Qt/RHI Integration | [A_034 검증](validation/a034-p2y-integration-stabilization.md) | 일부 Native/Source 범위만 증명 |
| **A_035** | P2-Z Developer Preview 범위 종료 감사 | [A_035 검증](validation/a035-p2z-final-preview-acceptance.md) | 제한 Preview 총괄, M0/M1/U/T 정식 인수 아님 |
| **A_036** | 개발 이행 단일 체크리스트 개설·관리 | [A_036 이슈 #209](https://github.com/july-rust/BoardSI/issues/209) | 관리 작업. 별도 제품 구현/수락 시험 아님 |

**역사적 원장 보존:** A_025–A_035 보고서의 과거 CI PENDING/한정 PASS 등 상태를 현재 문서 정리만으로 소급 갱신하지 않습니다. 검증 범위가 추가 확인되면 해당 **원본 Issue/PR/SHA**에 근거해 다음 E 감사에서 보강합니다. 아래에 있는 원래 A 기능별 [x]/[ ] 체크는 그대로 유지합니다.

### 실제 실행 증빙이 있는 제한 Preview

- [x] Qt Widgets Preview와 별도 Rust Worker / Versioned IPC의 제한 작동 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-19 KST**
  - [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) ([PR #102](https://github.com/july-rust/BoardSI/pull/102))
- [x] 공개 합성 PADS Layout ASCII V9.4 MILS의 **명시된 subset만** 파싱 및 비지원 입력 거절 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-22 KST**
  - [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) ([PR #159](https://github.com/july-rust/BoardSI/pull/159))
- [x] 지원 Straight Trace/Arc/Via/Component·RF Pad 및 Bottom Mirror 변환 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-23 KST**
  - [A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md) ([PR #167](https://github.com/july-rust/BoardSI/pull/167))
- [x] 지원 Board Outline과 다각형·곡선·복합 외곽 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST**
  - [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) ([PR #208](https://github.com/july-rust/BoardSI/pull/208))
- [x] 지원 Copper Area/원형·곡선·비직교 외곽, Cutout/Hole와 CPU/RHI 표시 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-25 KST**
  - [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) ([PR #192](https://github.com/july-rust/BoardSI/pull/192))
- [x] 원본 Source tick·좌표·원본 개체별 Provenance와 Artwork 독립 검증 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST**
  - [A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md) ([PR #200](https://github.com/july-rust/BoardSI/pull/200))
- [x] 제한 Layer·Net 표시, Filtering, Picking, Zoom/Fit 및 Qt Quick RHI Presented Frame · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST**
  - [A_034 Native 통합](validation/a034-p2y-integration-stabilization.md) ([PR #203](https://github.com/july-rust/BoardSI/pull/203))
- [x] 원본 정확히 일치하는 일부 Trace/Arc/Via **국소 접점만** 그래프 근거 생성 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST**
  - [A_032 국소 접점](validation/a032-p2w-local-connectivity.md) ([PR #196](https://github.com/july-rust/BoardSI/pull/196))
- [x] CPU/RHI 국소 접점 마커와 소스 정보/Visible Frame 정합성 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST**
  - [A_034 Native 통합](validation/a034-p2y-integration-stabilization.md) ([PR #203](https://github.com/july-rust/BoardSI/pull/203))
- [x] 연속 다른 PADS 합성 보드 재Import의 stale 객체·필터 상태 제거 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST**
  - [A_034 Native 통합](validation/a034-p2y-integration-stabilization.md) ([PR #202](https://github.com/july-rust/BoardSI/pull/202))
- [x] 10K·100K·1M **합성** 객체의 렌더/상호작용 회귀 시험(성능 보증 아님) · **체크일: 2026-10-26 KST** · **근거일: 2026-10-24 KST**
  - [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) ([PR #173](https://github.com/july-rust/BoardSI/pull/173))
- [x] Windows x64·Windows ARM64·macOS ARM64 **Native CI 시험 환경** 범위 PASS · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST**
  - [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) ([PR #208](https://github.com/july-rust/BoardSI/pull/208))

### SEC_001 실제 Code Scanning 상태 인수 — 전체 제품 보안 인수와 별개

- [x] **SEC_001 CodeQL `py/bad-tag-filter` Alert #1 실제 `Fixed` 검증** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST**
  - 기존 [수정 PR #206](https://github.com/july-rust/BoardSI/pull/206) 보호 병합 `main@6208db7083f4c56f91810e79d74ae22074221ecd`,
    [최신 기준 main@477428da의 21/21 CI](https://github.com/july-rust/BoardSI/commit/477428da3895ee4a2ff19dda5d53932754e3a6ba/checks),
    [독립 GitHub Actions 읽기 전용 API 감사](https://github.com/july-rust/BoardSI/actions/runs/41183754657): `state=fixed`, `fixed_at=2026-10-26T08:25:02Z`, 동일 `py/bad-tag-filter` 규칙으로 현재 열려 있는 Alert 0건.
  - . Code Scanning Alert #1 한정 인수이며 전체 보안 규칙/새 취약점 무결성 또는 릴리즈 인증을 의미하지 않습니다. [Issue #205](https://github.com/july-rust/BoardSI/issues/205).

### A_036 이후에도 반드시 남겨 두는 미완 항목

- [ ] 임의 상용 PADS 파일·버전의 전체 객체를 자격 검증
  - [Issue #9](https://github.com/july-rust/BoardSI/issues/9)
- [ ] Whole-board Open/Short, Pad/Pin/Pour/Thermal 접속의 물리 판정
  - [A_032 국소 접점](validation/a032-p2w-local-connectivity.md)
- [ ] GPU VRAM/driver reset/실장 기기/1M@60 FPS를 정량 측정
  - [Issue #165](https://github.com/july-rust/BoardSI/issues/165)
- [ ] 실제 PCB/Substrate 기반 RLGC/S·DC/30 GHz·Touchstone 검증
  - [M2 검증 원장](validation/README.md)
- [ ] 정식 설치/저사양 실물 PC/재배포 라이선스 검증
  - [M0 종료 게이트](validation/m0-completion-gate.md)

<a id="b_001"></a>
### B_001 추가 검증 — 제품 물리 자격과 구별

- [x] **B_001 제한 Qt Stackup/Reference Overlay UI** · **체크일: 2026-10-27 KST** · **근거일: 2026-10-26 KST**
  - Import Layer 번호 기반 Unknown 초기화, 수동 Material/Dk/Df/두께 입력 검사, 원본 SHA-256에 바인딩된 JSON 저장·복원, ReImport 초기화 및 원본 파일 덮어쓰기 방지의 **공개 합성 소스 Native 테스트 범위만** 완료
  - ([PR #213](https://github.com/july-rust/BoardSI/pull/213), [exact-head 26/26 검사](https://github.com/july-rust/BoardSI/commit/e5f449398c9f22a02878e397d69bb8b34319b252/checks),
    [Qt 3 Native + Linux 증거](validation/b001-stackup-reference.md)). **물리 Stackup/Net Reference 및 U09/U10/T02/T04/T24/T52 전체 PASS가 아님.**
- [x] **B_001_2 Rust Canonical Stackup/Material의 제한된 사용자 가정 검증** · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST**
  - 명시적 Layer·Material·출처 Revision·측정 방법·기준/유효 주파수 → 실제 `boardsi-model::BoardModel`의 `Property::Assumed`와 연속 z/전체 두께 계산, 대역 밖 차단, 변경 시 감사 무효화, Windows x64·ARM64/macOS ARM64/Linux native Qt→Rust 경로 PASS.
    별도의 검증된 **Reference Net/Copper 및 물리 SI 실행은 미구현**
  - ([PR #215](https://github.com/july-rust/BoardSI/pull/215), [native Qt CI](https://github.com/july-rust/BoardSI/actions/runs/40419571718),
    [Rust Numerical CI](https://github.com/july-rust/BoardSI/actions/runs/40419571744), [한정 범위·검증 원장](validation/b001-2-canonical-stackup-audit.md)). 정식 U09/U10/U39/T02/T04/T24/T52 `[x]` 아님.
- [x] **B_001_3 제한 PADS 원본 Copper/Net Reference 후보 감사** · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST**
  - 성공 Import의 source Area ID·Layer·Net 레이블(누락 `-`는 Unknown)·Cutout 목록을 Native Rust 감사로 전달. 동일 Net Signal/Reference 조합 거부, 분할/Cutout·Net 누락·불완전 목록은 전기적 Return Path를 **Unsupported로 차단**; 실제 PADS 공개 합성
    Import/GPU Artwork 및 Native 3 OS+Linux Qt 시험 **코드 HEAD 27/27 PASS**
  - ([PR #217](https://github.com/july-rust/BoardSI/pull/217), [코드 CI](https://github.com/july-rust/BoardSI/commit/d15b86db4cc044da07930fbc9baad41ea0bc891a/checks),
    [원장](validation/b001-3-source-reference-inventory.md)). **Reference 전기적 접속/Return Path P009 및 정식 T04/T24 인수 완료 아님.**
- [ ] **B_001 전체 Issue #12 완료**
  - 제한 Qt Stackup UI [#213](https://github.com/july-rust/BoardSI/pull/213), Rust Canonical 가정 Material [#215](https://github.com/july-rust/BoardSI/pull/215), 원본 Copper/Net **음성·Unsupported 판정**
    [#217](https://github.com/july-rust/BoardSI/pull/217)만 검증 완료. **실제 Copper/Net/Area/Pad/Port의 완전한 Canonical 연결 및 전기적 Reference/Return Path, Scenario/Cache Revision, 손실·거칠기 모델과 전체 정식 U/T 시험은 미완**
    ([#12](https://github.com/july-rust/BoardSI/issues/12), [B_001_3 원장](validation/b001-3-source-reference-inventory.md)).

<a id="b_002"></a>
- [x] **B_002의 제한적 B_001 Qt 감사 최신성 보강** · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST**
  - Stackup/Material/주파수·Reference Net 및 원본 Copper/Net 정보 변경 뒤 과거 Rust 감사 결과 무효화, 입력 JSON SHA-256 비교, 검증 중 외부 PCB 변경 거절을 Qt Native 테스트로 확인.
  - ([PR #219](https://github.com/july-rust/BoardSI/pull/219), 최종 PR HEAD `f9deda898b97a42886cfbdbdae972924c3fcdc56` 26/26 검사 PASS, 동일 HEAD Codex 리뷰 주요 지적 0,
    [서명된 병합 `6e795049b6de920f99053ae6361d487a7caeec96`](https://github.com/july-rust/BoardSI/commit/6e795049b6de920f99053ae6361d487a7caeec96), [B_002 원장](validation/b002-b001-completion.md)). **UI 감사 세션
    한정**이며 실제 물리 Scenario/Cache, Copper/Pad/Port 연결 및 Return Path, Material 손실/거칠기 모델과 정식 T02/T04/T24/T52는 아직 미검증. **B_001 전체 Issue #12는 미완료/OPEN 유지.**

- [x] **B_002 후속 — 제한적 Rust 감사 Scenario Revision 계약** · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST**
  - 원본 SHA-256, Stackup/Material 두께·Dk/Df·출처, Reference Net, Copper Inventory 및 주파수 변경을 구분하는 결정적 감사 Revision을 Rust Worker가 생성하며, Qt는 최신 감사 입력에만 이를 연결. 무관한 Zoom/Theme 변경은 Revision에 영향 없음.
  - ([PR #221](https://github.com/july-rust/BoardSI/pull/221), 최종 HEAD `dd5a2ac7efb0007e4624fb19a275a281b5381128` **27/27 CI PASS**, 동일 HEAD Codex 주요 지적 0·미해결 스레드 0,
    [GitHub 서명된 병합 `222552c9f92808e67ba06c80cb6de50756fcf3c2`](https://github.com/july-rust/BoardSI/commit/222552c9f92808e67ba06c80cb6de50756fcf3c2), [B_002 원장](validation/b002-b001-completion.md)). **감사
    입력 전용**이며 `cache_reuse_eligible=false`; 실제 물리 Scenario/Cache, Reference/Return Path, Copper/Pad/Port 연결 및 정식 T02/T04/T24/T52는 미검증. **B_001 전체 Issue #12는 미완료/OPEN 유지.**

<a id="b_003"></a>
### B_003 제한 완료 — Trace/Pad/Via 원본 접촉 근거 (Issue #12 연속 작업)

- [x] **B_003 지원 PADS V9.4 MILS Trace/Pad/Via 정확 좌표 접촉 그래프** · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST**
  - 제한 Straight Trace 내부 T-Junction, PADS ROUTE/SIGNAL Pin-pair에서 명시된 SMD Pad/Pin Net, Pad centre와 Straight Trace centreline 동일 층·동일 선언 Net 접점, 명시된 도금 Through Via의 해당 Conductive Layer 접점과 검증된 Layer 간
    Barrel Edge를 Canonical Rust 증거로 구성. 동일 Net 레이블만으로 일반 접촉을 가정하지 않고, 충돌/미확인 Net·미지원 형상은 Fail-closed. 기존 Qt V1 Source Endpoint Audit/Record 호환과 성능 회귀 유지.
  - [PR #224](https://github.com/july-rust/BoardSI/pull/224), 최종 HEAD `e211b2e626afd89ac7a880eaeace40e91c8abc50` **28/28 GitHub Checks SUCCESS**,
    [Codex 무결점 코드 Tree 동일성 비교](https://github.com/july-rust/BoardSI/compare/e40d45d157277d81ac8f50afef4a6b038c63f7bc...e211b2e626afd89ac7a880eaeace40e91c8abc50) (검토된 `e40d45d1`과 최종 HEAD의 Git Tree SHA 둘 다
    `aa5ac263f50044c79d97f27585e883010cab5633`; 리뷰 6개 스레드 해결),
    [Verified 서명된 Squash Merge `1c11d08be6e7c83d3f6b697f515f61a8c1a43ba8`](https://github.com/july-rust/BoardSI/commit/1c11d08be6e7c83d3f6b697f515f61a8c1a43ba8),
    [세부 검증 원장](validation/b003-b001-reference-topology.md). **일반 Pad 몸체·Trace 폭 겹침, 전체 Plane/Cutout/Reference Return Path, 물리 Solver/Cache, 정식 T04/T24 및 B_001 전체는 여전히 미완료.** 병합 후 `main` CI는 별도 확인 대상.

- [ ] **B_001 전체 Issue #12 정식 인수**
  - B_003 제한 접속 기능 병합이 곧 전체 Copper/Plane 연결과 고주파 Return Path 물리 검증을 의미하지 않음. U09/U10/U39, T02/T04/T24/T52 전체 승인 보류. Issue #12 OPEN 유지.

<a id="b_004"></a>
### B_004 제한 완료 — 단일 단순 Copper Area 국소 접촉 검증

- [x] **B_004 제한 PADS Source Copper Area 내부 접촉** · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST**
  - 공개 합성 PADS V9.4 MILS의 소스 소유 단일 Hole-free Straight COPCLS Polygon과 명시적 ROUTE Net이 일치할 때, 정수 Source Tick 내부의 Trace Endpoint/선언된 SMD Pad Centre 접촉을 Canonical `AreaInterior` 증거로 검증하고 같은 Area 안의 두 국소 접점만
    `AreaContinuity`로 연결. 직선상 정방향 Collinear 꼭짓점 허용·경계/역방향 중첩 거부, 타 Layer/Cross-Net/분리 Island 거짓 연결 방지, 변조된 증거와 구형 Qt V1 호환을 교차 시험.
  - [B_004 PR #226](https://github.com/july-rust/BoardSI/pull/226),
    [코드 HEAD `7c68f9523b65999271fff5b510e984ec30dac72a`의 12 워크플로·23 Job 전부 SUCCESS](https://github.com/july-rust/BoardSI/commit/7c68f9523b65999271fff5b510e984ec30dac72a/checks), 동일 HEAD Codex 추가 지적 0·과거
    스레드 4건 해결, [검증 원장](validation/b004-reference-copper-connectivity.md). **이 체크는 제한된 Rust Source-Geometry 기능에만 적용**되며 Arc/Hole/Cutout·Via-to-Plane·Pad 몸체/Trace 폭·Plane Return Path/P009/물리 SI/Cache 및 정식
    U09/U10/U39·T02/T04/T24/T52는 여전히 미검증. 보호된 Signed Squash Merge 및 병합 후 main CI는 별도 확인하여 기록한다.

<a id="b_005"></a>
### B_005 제한 완료 — Source-bound Reference 기하 커버리지

- [x] **B_005 제한 Source-bound Reference Plane Trace Centreline 기하 커버리지 감사** · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST**
  - 공개 합성 PADS V9.4 MILS의 지원 Straight/COPCLS Hole-free 단일 Polygon에 한정. 원본 파일 Source SHA·Net ID/이름·Layer 순서·Trace/Arc 원본 완전성·Copper Source ID/Net 및 원본 경계를 재파싱해 검증, 단일 Layer의 명시적 비분기 Trace 경로 전체가 하나의
    Reference Polygon 엄격 내부인지 정수 기하로 판정. Backtrack/T/Arc/Via 분기, 분리 Island·Cutout·원본 변조는 거부하며 결과는 `SOURCE_ORDER_GEOMETRY_ONLY`일 뿐 실제 P009/Return Path 아님.
    [PR #227](https://github.com/july-rust/BoardSI/pull/227),
    [코드 HEAD `225489bedabc146e31b83137267aaa29d86f8503` 23/23 CI 성공](https://github.com/july-rust/BoardSI/commit/225489bedabc146e31b83137267aaa29d86f8503/checks),
    [Rust Debug/Release B_005 17/17 PASS 및 원장](validation/b005-reference-geometry.md)
  - . 
  - (최초 체크일) 독립 Codex 최종 PR HEAD `284f5f04fe` 리뷰 주요 이슈 0; 역사적 지적 11건 해결·리뷰 스레드 0건. 보호 Squash Merge [PR #227](https://github.com/july-rust/BoardSI/pull/227),
    [Signed Verified `main@7a1ed48bf905df8357691a1f51b73e130a74537c`](https://github.com/july-rust/BoardSI/commit/7a1ed48bf905df8357691a1f51b73e130a74537c). 보호된 `main` 병합 후 Push CI **21/21 SUCCESS** 및 Push
    on main 워크플로 SUCCESS 확인 (별도 완료 체크 참조). `return_path_p009=UNSUPPORTED`, `solver_ready=false`, `physical_si=NOT_RUN`, `cache_reuse_eligible=false`. B_001 Issue #12 OPEN, 공식 U09/U10/U39 및 T02/T04/T24/T52
    미완.

- [x] **B_005 post-main Push CI 최종 확인** · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST**
  - [Signed Squash Merge `7a1ed48`](https://github.com/july-rust/BoardSI/commit/7a1ed48bf905df8357691a1f51b73e130a74537c/checks)의 병합 후 GitHub Check **21/21 SUCCESS** 및 별도 **Push on main SUCCESS** 확인.
  - . [메인 Push 검사 근거](https://github.com/july-rust/BoardSI/commit/7a1ed48bf905df8357691a1f51b73e130a74537c/checks)
  - [Push on main 워크플로 #40501517050](https://github.com/july-rust/BoardSI/actions/runs/40501517050). 코드·제한 B_005 구현 `[x]`와 구분한다.

<a id="b_006"></a>
### B_006 제한 완료 — Source-bound Through Via/COPCLS 국소 접촉 감사

- [x] **B_006 공개 합성 PADS V9.4 MILS Plated Through Via–단일 Straight Hole-free COPCLS Polygon의 제한 국소 접촉** · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST**
  - 원본 ROUTE/Net/Layer/Via/Drill/Padstack 및 Copper Source SHA·기하를 재파싱해 동일 원본과 비교하며, 명시적 도금 Through Via의 Conductive Padstack Layer상 Via Centre가 같은 선언 Net의 단일 Source Polygon 엄격 내부인 경우에만
    `ViaCentre`–`AreaInterior` SameLayer 증거를 Canonical Graph에 추가. 기존 독립 접점과의 ViaBarrel·AreaContinuity를 검증하며 Source 변조/거짓 Layer/동일 위치 Edge 중복/잘못된 Net·Cutout·Boundary를 차단, Qt V1 회귀 보존. (최초)
  - . [구현 PR #229](https://github.com/july-rust/BoardSI/pull/229),
    [최종 코드 HEAD `68cd29313d4b3ccfd340321196e286af0ef4ff80`의 CI](https://github.com/july-rust/BoardSI/commit/68cd29313d4b3ccfd340321196e286af0ef4ff80/checks) **9개 워크플로 SUCCESS**,
    [Rust Numerical Debug/Release B_006 각각 11/11 PASS](https://github.com/july-rust/BoardSI/actions/runs/40510123992),
    [Qt Native Windows x64/ARM64·macOS ARM64·Linux SUCCESS](https://github.com/july-rust/BoardSI/actions/runs/40510123737), Codex 동일 HEAD findings 0·두 과거 P2 스레드 해결, GitHub 보호 Squash
    [`main@d7a22f8230086cdf8aeb326882708dfee48d9ee1`](https://github.com/july-rust/BoardSI/commit/d7a22f8230086cdf8aeb326882708dfee48d9ee1), [B_006 검증 원장](validation/b006-via-copper-contact.md). **이 체크는 좁은
    원본 기하 국소 접촉 기능만** 의미함. 실제 Copper fill/전기적 Return Path/P009/물리 Solver·Cache 및 정식 U09/U10/U39/T02/T04/T24/T52 전부 미인수. Issue #12 OPEN.
- [x] **B_006 protected main post-merge Push CI 및 최종 서명 Verified 확인** · **체크일: 2026-10-28 KST** · **근거일: 2026-10-27 KST**
  - B_006 코드 [보호 Squash Merge `d7a22f8230086cdf8aeb326882708dfee48d9ee1`](https://github.com/july-rust/BoardSI/commit/d7a22f8230086cdf8aeb326882708dfee48d9ee1/checks)의 REST
    `commit.verification.verified=true`, `reason=valid`, `verified_at=2026-10-27T13:23:25Z`를 별도 확인. 해당 SHA의
    [GitHub Check Runs](https://github.com/july-rust/BoardSI/commit/d7a22f8230086cdf8aeb326882708dfee48d9ee1/checks) **21/21 SUCCESS**, 병합 후 **9개 workflow runs 전부 SUCCESS** 및 별도
    [Push on main #40511614420](https://github.com/july-rust/BoardSI/actions/runs/40511614420) SUCCESS (코드 PR HEAD 검증과 독립 확인).
  - . B_006의 제한 국소 접촉 기능에만 적용하며 B_001 Issue #12, 물리 Return Path/P009, Solver/Cache, 공식 U09/U10/U39/T02/T04/T24/T52는 미완.

<a id="b_007"></a>
### B_007 — COPCUT-aware bounded local contact implementation and integration

- [x] **B_007 original PADS single source-owned polygon COPCUT-aware Trace/Pad/Plated Through Via centre local evidence — limited implementation/test/merge only** · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**
  - [PR #231](https://github.com/july-rust/BoardSI/pull/231), exact source HEAD
    [`6a75eb5c5616d86407a04a946bc994958fb55ea6`](https://github.com/july-rust/BoardSI/commit/6a75eb5c5616d86407a04a946bc994958fb55ea6/checks) **28/28 PR checks SUCCESS**, B_007 source/signed area Rust
    Numerical Debug **11/11 PASS** and Release **11/11 PASS**, Native Rust Windows x64/ARM64, macOS ARM64 and Qt Windows x64/ARM64/macOS ARM64/Linux CI SUCCESS; original PADS
    Source/Net/Pad/Padstack/Drill/Via/Area geometry and strict single Hole contour validated. Protect Signed Verified Squash merged to
    [`main@8aae7504417f6aa8d212a0906de658ed173c4ef9`](https://github.com/july-rust/BoardSI/commit/8aae7504417f6aa8d212a0906de658ed173c4ef9), GitHub REST `commit.verification.verified=true, reason=valid`.
  - . The latest-head automatic independent Codex review is separately pending; this check certifies bounded implementation/CI/merge, **not** final independent-review closure, physical electrical SI,
    general hole/cutout connectivity or full B_001 acceptance. [Validation ledger](validation/b007-copper-cutout-connectivity.md).
- [x] **B_007 protected main post-merge Checks and Push on main** · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**
  - independently inspected exact merged [`main@8aae7504` Check runs](https://github.com/july-rust/BoardSI/commit/8aae7504417f6aa8d212a0906de658ed173c4ef9/checks): **21/21 SUCCESS**, all **9/9 post-main
    workflow runs SUCCESS**, including [Push on main #40602499013](https://github.com/july-rust/BoardSI/actions/runs/40602499013) and
    [Qt Runtime #40602499442](https://github.com/july-rust/BoardSI/actions/runs/40602499442).
  - . Tested synthetic source geometry subset only.
- [ ] **B_007 exact latest HEAD independent Codex final-review closure**
  - prior three P2 findings (source Pad spoof, two negative/positive fixture expectations) were fixed and all three PR threads resolved. Automatic latest-head review receipt for `6a75eb5c` still reports
    `status=running`; **do not claim completed/zero findings** until rechecked as `status=completed` with `has_any_findings=false`, `has_unresolved_findings=false` and zero unresolved PR threads. Parent
    [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN.


- [ ] **B_001 전체 Issue #12 정식 인수**
  - B_006이 제한적으로 성공하더라도 물리 Reference Return Path/P009, Scenario/Cache, Material loss/roughness 및 공식 U09/U10/U39/T02/T04/T24/T52는 남는다. 원래 OPEN 상태 유지.

<a id="b_008"></a>
### B_008 — 단일 COPCUT Reference 경로 기하 감사 (2026-10-28 KST)

- [x] **B_008 제한 단일 COPCUT Reference 전체 Centreline 기하 검사 구현 및 보호된 병합** · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**
  - B_005 원본 검증과 B_007 단일 Source-owned Hole을 결합해 전 구간의 Cutout Boundary/내부 교차를 차단; 물리 Return Path 자격 아님. [PR #233](https://github.com/july-rust/BoardSI/pull/233), exact final Source HEAD
    `3dc3a681a2259369d3b1396a95f3d988fbad438c` **23/23 Checks, 9/9 workflows SUCCESS**, Numerical [#40610946199](https://github.com/july-rust/BoardSI/actions/runs/40610946199) B_008 10/10 PASS, Qt Native
    [#40610946377](https://github.com/july-rust/BoardSI/actions/runs/40610946377) SUCCESS, same-HEAD independent Codex Review completed/no findings or unresolved threads,
    [Signed Verified Squash main `9cba34f1`](https://github.com/july-rust/BoardSI/commit/9cba34f1eb1135da8e187c8ecb945ad90c78d073).
  - . 지원된 Source Geometry의 제한 기능만 [x]; P009/Return Path, 물리 Solver/Cache, 정식 U09/U10/U39/T02/T04/T24/T52 및 B_001 Issue #12 미완.
- [x] **B_008 보호된 main 병합 후 독립 Push/Native CI 최종 확인** · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**
  - 정확한 [Signed Verified Squash `main@9cba34f1eb1135da8e187c8ecb945ad90c78d073`](https://github.com/july-rust/BoardSI/commit/9cba34f1eb1135da8e187c8ecb945ad90c78d073/checks)에서 **21/21 GitHub Check Runs
    SUCCESS**, 9/9 Workflow Runs SUCCESS, 별도 [Push on main #40612009982](https://github.com/july-rust/BoardSI/actions/runs/40612009982) 및
    [Qt Runtime #40612010054](https://github.com/july-rust/BoardSI/actions/runs/40612010054) SUCCESS를 GitHub REST로 확인.
  - . 제한 Source Geometry 기능만 완료. [B_008 검증 원장](validation/b008-reference-cutout-geometry.md).


<a id="b_009"></a>
### B_009 — B-series 제한 범위 최종 감사 및 C 단계 인계 (2026-10-28 KST)

- [x] **B_009 실행 전 보호 main 및 체크리스트 감사** · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**
  - 보호 `main@972c6663a5d3cd0fd9d7f9cfe2d22c24dbc21ef3` GitHub 서명 `verified=true, reason=valid`, [동일 SHA 검사](https://github.com/july-rust/BoardSI/commit/972c6663a5d3cd0fd9d7f9cfe2d22c24dbc21ef3/checks)
    **21/21 Check Runs SUCCESS**, 9/9 Workflow Runs SUCCESS를 실제 GitHub REST에서 조회하고 이 문서 및 [B_008 원장](validation/b008-reference-cutout-geometry.md),
    [B_007 원장](validation/b007-copper-cutout-connectivity.md), [Issue #12](https://github.com/july-rust/BoardSI/issues/12)를 대조했다.
  - . 이는 기존 main 증거이며 **B_009 신규 PR 시험 결과는 아님**.
- [x] **B_009 B_001–B_008 제한 구현 통합 감사·C 단계 인계 및 보호된 main 최종 검증** · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**
  - [B_009 검증 원장](validation/b009-b-series-final-acceptance.md)과 C_001–C_004 소규모 개발 인계. 최종 [PR #235](https://github.com/july-rust/BoardSI/pull/235) 정확한 소스 HEAD `a38facad54b457947009e3a364ce5bfdb3a6a37d`
    **22/22 PR Checks SUCCESS** 및 동일 HEAD Codex 자동 리뷰 `completed`, `has_any_findings=false`, `has_unresolved_findings=false`, 리뷰 미해결 Thread 0.
    [보호된 Squash Merge `main@47f4a6de7125eb49cb504c3b5d91aa59d908031f`](https://github.com/july-rust/BoardSI/commit/47f4a6de7125eb49cb504c3b5d91aa59d908031f) 서명 `verified=true, reason=valid`;
    [main 동일 SHA 병합 후 21/21 Checks SUCCESS](https://github.com/july-rust/BoardSI/commit/47f4a6de7125eb49cb504c3b5d91aa59d908031f/checks), 9/9 Workflows SUCCESS
    ([Push on main #40623750981](https://github.com/july-rust/BoardSI/actions/runs/40623750981), [Qt Native #40623751051](https://github.com/july-rust/BoardSI/actions/runs/40623751051)).
  - . **제한 문서/원본 기하 개발 감사만 [x]**; B_007 최종 Source HEAD 리뷰 및 B_001 Issue #12 전체·물리 P009/정식 U/T 인수는 여전히 [ ]/OPEN.
- [ ] **B_007 최종 Source HEAD 독립 Codex 리뷰 종결**
  - 원본 PR [#231](https://github.com/july-rust/BoardSI/pull/231)의 정확한 `6a75eb5c5616d86407a04a946bc994958fb55ea6`에 대한 [Codex 최신 상태](https://github.com/july-rust/BoardSI/pull/231#issuecomment-6455692908)는
    `status=running`. 3개 이전 스레드 resolved 및 28/28 Checks PASS는 확인되나 **최종 HEAD 리뷰 완료가 아님**. `completed`, 발견 사항/미해결 0을 확인하기 전 체크 금지.
- [ ] **B_001 Issue #12 전체 정식 인수**
  - 제한 Source Geometry와 실제 전기적 Reference/Return Path, Material Loss, 물리 Solver Scenario/Cache, U09/U10/U39 및 T02/T04/T24/T52 전체 시험을 혼동하지 않는다. [Issue #12](https://github.com/july-rust/BoardSI/issues/12)
    OPEN 유지. 미완 사항은 C단계 후보로 명확히 이관하지만 정식 M1 종료로 간주하지 않는다.

<a id="c_001"></a>
### C_001 — 다중 Source-owned COPCUT / Disconnected Island (제한 기능 완료 · 2026-10-28 KST)

- [x] **C_001 제한 다중 Source-owned Straight Polygon COPCUT 및 분리 Copper Island 국소 접점 — 정식 한정 구현 인수** · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**
  - [C_001 검증 원장](validation/c001-multi-cutout-island-connectivity.md), [코드 PR #237](https://github.com/july-rust/BoardSI/pull/237), 최종 PR Source HEAD `1db862985505427f8f3fcd58b6ae6c9530cbac6a`의 **28/28
    Checks SUCCESS**, Rust C_001 신규 Debug·Release **각 10/10 PASS**, 독립 Codex 동일 HEAD `completed`/findings 0/unresolved 0, 과거 P2 스레드 resolved;
    [Verified 보호된 Squash `main@df1fde8648fd48f9ee884a3d9ad68fdf266e5f35`](https://github.com/july-rust/BoardSI/commit/df1fde8648fd48f9ee884a3d9ad68fdf266e5f35/checks) `verified=true, reason=valid`, **독립 병합
    후 21/21 Checks·9/9 Workflows SUCCESS** ([Push main #40629876739](https://github.com/july-rust/BoardSI/actions/runs/40629876739),
    [Qt Native #40629877037](https://github.com/july-rust/BoardSI/actions/runs/40629877037)).
  - . (최초 체크일) 지원된 Source Geometry/국소 접촉만 `[x]`; 분리 Island는 Net 레이블만으로 연결하지 않으며 모호한 중첩 Area는 Unsupported. **B_007 원본 최종 HEAD Codex 리뷰 및 B_001 Issue #12 전체/P009/Solver/Cache/정식 U09/U10/U39/T02/T04/T24/T52는
    여전히 `[ ]`/OPEN.**

<a id="c_002"></a>
### C_002 — Reference Split 및 Layer Transition Topology Risk (Rust 제한 기능 검증 완료 · 2026-10-28 KST)

- [x] **C_002 Source-backed Reference Gap/Cutout 및 Plated Through Via Transition Risk의 제한 Rust Audit·공개 합성 시험·보호 병합 및 독립 post-main 검증** · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**
  - [검증 원장](validation/c002-reference-split-transition-topology.md), [코드 PR #240](https://github.com/july-rust/BoardSI/pull/240), 최종 Source HEAD
    [`c6591c14842b55fb48ba070c9f719998dd37d195`](https://github.com/july-rust/BoardSI/commit/c6591c14842b55fb48ba070c9f719998dd37d195/checks) **23/23 PR Checks SUCCESS**, C_002 신규 Rust Debug
    **10/10**·Release **10/10 PASS**, 기존 B_003–B_008/C_001 회귀, Rustfmt·Clippy 및 Windows x64/Windows ARM64/macOS ARM64/Linux Qt Native PASS. 독립 Codex **동일 최종 HEAD `completed`, findings 0, unresolved 0**,
    Review Thread 미해결 0. 보호된 [Verified Squash `main@304015008112b082f96829f763ad4e77e109094c`](https://github.com/july-rust/BoardSI/commit/304015008112b082f96829f763ad4e77e109094c)
    `verified=true, reason=valid`; 독립 [post-main 21/21 Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/304015008112b082f96829f763ad4e77e109094c/checks), **9/9 Workflows SUCCESS**
    ([Push main #40640594957](https://github.com/july-rust/BoardSI/actions/runs/40640594957), [Qt runtime #40640594732](https://github.com/july-rust/BoardSI/actions/runs/40640594732)).
  - (최초). 지원된 Source 정확 기하/Topology Risk 판정만 `[x]`; 전기적 Return Path·P009 물리 자격 또는 공식 U/T 전체 인수가 아님.
- [x] **C_002 이후 Source-bound Qt Reference Split/Layer Transition 위험 표시의 한정 인수 확인** · **체크일: 2026-10-30 KST** · **근거일: 2026-10-29 KST**
  - 최초 C_002 [PR #240](https://github.com/july-rust/BoardSI/pull/240)은 Rust 전용으로 Qt 기능 미완이었으나, 후속 C_005
    [PR #249](https://github.com/july-rust/BoardSI/pull/249)·[Native #250](https://github.com/july-rust/BoardSI/pull/250)의 Source-bound `REFERENCE_SPLIT_RISK`/`LAYER_TRANSITION_RISK` Qt CPU/RHI Marker와
    Stale 처리 및 C_008 [PR #259](https://github.com/july-rust/BoardSI/pull/259)의 두 Layer Via Native Qt 검증으로 **해당 한정 기능만** 구현·병합 후 검증된 사실을 확인. [C_005 원장](validation/c005-qt-reference-risk-integration.md)
  - [C_008 원장](validation/c008-multilayer-reference-via-transition.md). 
  - . 임의 PADS 형상·실제 전기적 Return Path/P009·정식 M1·U/T 인수는 미완이며 이 체크가 최초 C_002 PR에 Qt 코드가 있었다는 뜻은 아님.
- [ ] **B_001 Issue #12 및 정식 U09/U10/U39·T02/T04/T24/T52 전체 인수**
  - Reference 전기적 연속성, 실제 P009 Return Path, Solver/Cache/Material Loss는 모두 미승인. [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN 유지. B_007 원본 마지막 HEAD Codex 리뷰도 별도 NOT_VERIFIED 상태 보존.

<a id="c_003"></a>
### C_003 — Immutable Scenario and Stale Result (Rust 제한 기능 검증 완료 · 2026-10-28 KST)

- [x] **C_003 제한 Rust Immutable Scenario/ResultReceipt 계약의 정확한 Source HEAD 시험·독립 리뷰·보호 병합 및 독립 post-main CI** · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**
  - [Issue #242](https://github.com/july-rust/BoardSI/issues/242), [검증 원장](validation/c003-immutable-scenario-stale-result.md), [코드 PR #243](https://github.com/july-rust/BoardSI/pull/243), 최종 Source HEAD
    [`f1d9c2d66e94d7228326ee009c3c0ae902a1ba3f`](https://github.com/july-rust/BoardSI/commit/f1d9c2d66e94d7228326ee009c3c0ae902a1ba3f/checks) **9/9 GitHub Actions Workflows SUCCESS**, 신규 C_003 Synthetic 회귀
    Rust **Debug 12/12·Release 12/12 PASS**, Rustfmt·Clippy 및 기존 B/C 회귀 PASS, Numerical Native Windows x64/ARM64·macOS ARM64와 Qt Native Windows x64/ARM64·macOS ARM64/Linux PASS. 독립 Codex **동일 최종 HEAD
    completed, findings 0, unresolved 0**, Review Thread 미해결 0. 보호된 **Signed Verified Squash**
    [`main@7edff0b9766a1ae70ab26e7010e356d64bad7046`](https://github.com/july-rust/BoardSI/commit/7edff0b9766a1ae70ab26e7010e356d64bad7046) **`verified=true, reason=valid`**,
    [동일 main SHA 독립 21/21 Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/7edff0b9766a1ae70ab26e7010e356d64bad7046/checks) 및 **9/9 Workflow Runs SUCCESS**
    ([Push on main #40648767875](https://github.com/july-rust/BoardSI/actions/runs/40648767875), [Qt runtime #40648767924](https://github.com/july-rust/BoardSI/actions/runs/40648767924),
    [Numerical core #40648767849](https://github.com/july-rust/BoardSI/actions/runs/40648767849)).
  - (최초 기록). 이는 Caller-provided SHA 주장과 Synthetic Receipt의 **제한 입력 최신성 계약**만 완료한 것이며, 실 PCB 전체 Canonical 직렬화·실행 가능한 물리 Solver/Cache 재사용·P009, 정식 U09/U10/U39/T02/T04/T24/T52 인수와 동일하지 않다.
- [ ] **C_003 이후 실 PCB 완전 Solver Snapshot/Worker transport 및 실제 재사용 캐시**
  - 전 물리 입력 Canonical Serialize/Source Rehash·Port/Load·Material·엔진 호환성 검증과 물리 정확도/Cache 별도 Gate 필요. [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN, C_002 Qt Reference 위험 마커 미구현 및
    B_007 원본 HEAD Codex 최종 미확인 유지.

<a id="c_004"></a>
### C_004 — Material Provenance / Loss Input Gate (제한 코드 및 post-main 검증 완료 · 2026-10-28 KST)

- [x] **C_004 Canonical Material/Layer의 제한 Rust 입력 계약 구현·최종 코드 PR 검증·보호 병합** · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**
  - [Issue #245](https://github.com/july-rust/BoardSI/issues/245), [PR #246](https://github.com/july-rust/BoardSI/pull/246), [검증 원장](validation/c004-material-loss-conductivity-roughness.md). Dk/Df·σ·거칠기
    출처/유효대역·복소 유전율 부호와 오버플로·Material Revision을 제한 Native Rust로 검증. 정확한 코드 HEAD
    [`3c17b723b5d2508e71808d186c78869ece56ee06`](https://github.com/july-rust/BoardSI/commit/3c17b723b5d2508e71808d186c78869ece56ee06/checks) **23/23 CI SUCCESS·9/9 Workflows SUCCESS**, 3 Native Rust/Qt +
    Linux Qt, 동일 HEAD 독립 Codex completed / 주요 지적 0 / 미해결 스레드 0.
    [Verified Squash `main@02cc35b51dfe4263f3e3d3f02f9151d47d696849`](https://github.com/july-rust/BoardSI/commit/02cc35b51dfe4263f3e3d3f02f9151d47d696849), GitHub `verified=true`, `reason=valid`.
  - . 이 체크는 제한 입력 계약/코드 병합만 의미하며 실제 물리 Loss/PH-12~16, C_003 물리 Solver/Cache, P009 및 정식 U09/U10/U39/T02/T04/T24/T52 인수는 미완, Issue #12 OPEN.
- [x] **C_004 코드 보호 병합 후 독립 `main@02cc35b` 검증** · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**
  - [동일 코드 병합 SHA](https://github.com/july-rust/BoardSI/commit/02cc35b51dfe4263f3e3d3f02f9151d47d696849/checks) **21/21 Check Runs SUCCESS·9/9 Workflow Runs SUCCESS**
    ([Push on main #40673838011](https://github.com/july-rust/BoardSI/actions/runs/40673838011), [Qt runtime #40673838211](https://github.com/july-rust/BoardSI/actions/runs/40673838211),
    [Numerical core #40673838276](https://github.com/july-rust/BoardSI/actions/runs/40673838276), [FEM Native #40673838163](https://github.com/july-rust/BoardSI/actions/runs/40673838163)). FEM macOS ARM64
    최초 취소 후 재실행 attempt 2 성공 포함.
  - . 문서 자체의 보호 병합/후속 CI는 별도 PR에서 검증. 이 체크는 PH-12~16이나 실제 PCB 물리 Loss/전체 M1 인수와 무관.

<a id="c_005"></a>
### C_005 — Qt Source Reference Risk 표시/무효화 제한 인수 (2026-10-28 KST)

- [x] **C_005 Source-bound Rust Reference 기하 위험 감사 → Qt CPU/RHI Warning Marker 및 Stale 입력 무효화** · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**
  - 공개 합성 PADS V9.4 MILS의 C_002 지원 Straight Trace와 명시적 도금 Through Via 전이·Reference COPCLS/COPCUT 일부에 한정. Source SHA-256·원본 ObjectId·Layer/Net·정수 Endpoint 일치 및 명시 Reference 선택을 Rust/Qt 간 재검증하고
    `REFERENCE_SPLIT_RISK`/`LAYER_TRANSITION_RISK` 주의 마커, 대표 Midpoint 표시(정확한 교차 위치 아님), CPU/RHI Filter/Picking/Properties, 원본 변경·ReImport·선택/Stackup 수정·지연 응답 Stale 차단을 구현. `SOURCE_GEOMETRY_ONLY`는 전기적 PASS
    아님, `UNSUPPORTED`는 긍정 판정 금지. [Issue #248](https://github.com/july-rust/BoardSI/issues/248)
  - [검증 원장](validation/c005-qt-reference-risk-integration.md). 코드 [PR #249](https://github.com/july-rust/BoardSI/pull/249) (정확한 HEAD `43204eab6c6b204a93a7e36ad957bad772187fd1`: 11/11 Workflow/22/22 Job 성공,
    동일 HEAD Codex 0 findings·미해결 0, 과거 지적 3건 해결), [Verified Squash `main@338f5c5f9d8590ba84d56803036590158a8f87c4`](https://github.com/july-rust/BoardSI/commit/338f5c5f9d8590ba84d56803036590158a8f87c4) +
    독립 Main 21/21 Check 및 Push-on-main 성공. Native 전용 C_005 회귀시험 보강 [PR #250](https://github.com/july-rust/BoardSI/pull/250) (HEAD `b7f6e550f1cd9bf8d6ca32ad8bdf43ae7e8222f2`: 25/25 GitHub Checks, 9/9
    Workflow, Windows x64/ARM64·macOS ARM64의 실제 C_005 test + Linux Qt CTest 성공, Codex 0 findings/미해결 0),
    [Verified Squash `main@0c653bb6c01eba10bc0d1e145bc27d5228bd24f3`](https://github.com/july-rust/BoardSI/commit/0c653bb6c01eba10bc0d1e145bc27d5228bd24f3). **후속 main@0c653bb의 독립 21/21 Checks SUCCESS 및 9/9
    Main Workflows SUCCESS 확인** ([post-main 검사](https://github.com/july-rust/BoardSI/commit/0c653bb6c01eba10bc0d1e145bc27d5228bd24f3/checks),
    [Push on main #40686719331](https://github.com/july-rust/BoardSI/actions/runs/40686719331), [Qt Native #40686719649](https://github.com/july-rust/BoardSI/actions/runs/40686719649)). (최초)
  - . 기존 모든 최초 체크일 불변. **실제 Reference 전류·P009, 물리 RLGC/S Solver·Cache, 정식 U09/U10/U39 및 T02/T04/T24/T52는 미완; 상위 M1 [Issue #12](https://github.com/july-rust/BoardSI/issues/12)는 OPEN 유지.**

<a id="c_006"></a>
### C_006 — Source-exact Reference Gap Interval / Qt CPU/RHI 제한 기능 완료 (2026-10-28 KST)

- [x] **C_006 공개 합성 PADS Source Reference 미피복 구간의 정확한 유리수 교차 매개변수 계산·Qt CPU/RHI 구간 경고 및 Stale 무효화** · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**
  - C_005의 단일 대표 Midpoint만 표시하던 지원 Straight Signal Trace에 대해, 명시 Reference Net/Layer의 단일 Source COPCLS 및 최대 8개 Polygon COPCUT과의 교차를 원본 정수 Source Tick 기반 유리수 t 구간으로 구하여, 0~32개 미피복 구간을 V2 계약으로 전달하고 CPU/RHI
    선택/Properties·Source SHA/Trace/Net/Layer·기하·구간/비동기 무효화 검증. 화면 좌표와 길이는 **표시용 근삿값**이며 경계 접촉·다중 Source Reference Island·비지원 형상은 Fail-closed. 기존 C_005 V1 Plated Via 위험판정 및 Qt Regression 보존.
    [Issue #252](https://github.com/july-rust/BoardSI/issues/252), [검증 원장](validation/c006-reference-gap-interval.md), [코드 PR #253](https://github.com/july-rust/BoardSI/pull/253). **최종 HEAD
    [`ed5e137bbb8b1bebcfde832961038d43286a7753`](https://github.com/july-rust/BoardSI/commit/ed5e137bbb8b1bebcfde832961038d43286a7753/checks) 27/27 CI SUCCESS** (Rustfmt·Clippy -D warnings, Rust
    Debug/Release C_006 9/9 synthetic case 및 기존 B/C 회귀, 3 Native Windows x64/ARM64·macOS ARM64 Qt C_006 실프로세스 smoke와 Linux Qt), **동일 HEAD Codex completed·지적 0·미해결 0**; 보호된
    [Signed Verified Squash `main@5e4de472a90539221f69e5cf11b88e20d9fd7361`](https://github.com/july-rust/BoardSI/commit/5e4de472a90539221f69e5cf11b88e20d9fd7361) (`verification.verified=true`,
    `reason=valid`). **독립 post-main [21/21 Checks SUCCESS](https://github.com/july-rust/BoardSI/commit/5e4de472a90539221f69e5cf11b88e20d9fd7361/checks) 및 9/9 Workflow SUCCESS**
    ([Push on main #40697041065](https://github.com/july-rust/BoardSI/actions/runs/40697041065), [Qt runtime #40697040984](https://github.com/july-rust/BoardSI/actions/runs/40697040984)). (최초)
  - . **상위 M1 Issue #12 OPEN**, 물리 P009/실제 Return Current·전체 전기적 Copper 접속, RLGC/S/제품 Solver/Cache, 정식 U09/U10/U39·T02/T04/T24/T52는 미검증 유지.

## 3. 후속 개발 체크리스트

| 단계 | 상태 | 구체적 잔여 작업 (GitHub) |
| --- | --- | --- |
| M0 | ☐ BLOCKED | [M0 종료 게이트](validation/m0-completion-gate.md) — 사양/시험 개별 인수·실물 기준 PC/공개 샘플·라이선스/배포 |
| M1 Import/Artwork | ☐ PARTIAL | [#8](https://github.com/july-rust/BoardSI/issues/8), [#9](https://github.com/july-rust/BoardSI/issues/9), [#10](https://github.com/july-rust/BoardSI/issues/10), [#11](https://github.com/july-rust/BoardSI/issues/11): 포맷별 Import 완성도·ODB++·전체 연결 |
| **B_001 / M1 Stackup·Reference** | ◐ 제한 UI PASS / 전체 미완 | [PR #213](https://github.com/july-rust/BoardSI/pull/213) · [B_001 실행 근거](validation/b001-stackup-reference.md): 공개 합성 Source-bound 편집·검증·저장·재Import는 Native 검증 PASS; 물리 Material/Canonical 연결·Reference 도체·인수시험은 미완 |
| M1a | ☐ PARTIAL | [#13](https://github.com/july-rust/BoardSI/issues/13): Substrate Pad/Via/Anti-pad/Plane·Pad-to-Pad |
| M1b | ☐ 미완 | [#14](https://github.com/july-rust/BoardSI/issues/14), [#15](https://github.com/july-rust/BoardSI/issues/15): Port 후보/수동·Single/Differential·P001–P012 |
| M2 | ☐ 미완 | [#16](https://github.com/july-rust/BoardSI/issues/16), [#17](https://github.com/july-rust/BoardSI/issues/17): 물리 검증된 RLGC, Direct S, DC, Touchstone |
| M2a | ☐ 미완 | [#18](https://github.com/july-rust/BoardSI/issues/18), [#19](https://github.com/july-rust/BoardSI/issues/19): Progressive Sweep/Region·결과 Live |
| M2b | ☐ 미완 | [#20](https://github.com/july-rust/BoardSI/issues/20), [#21](https://github.com/july-rust/BoardSI/issues/21): RAM/SSD Cache·Snapshot·강제종료/복구 |
| M2c | ☐ 미완 | [#22](https://github.com/july-rust/BoardSI/issues/22): Via/Pad 불연속 독립 물리 검증 |
| M3 | ☐ 미착수 | [#23](https://github.com/july-rust/BoardSI/issues/23): IPC-2581/KiCad/Gerber/Drill/Netlist |
| M4 | ☐ 미착수 | [#24](https://github.com/july-rust/BoardSI/issues/24): Altium/Allegro/EAGLE/EasyEDA 평가 |
| M5 | ☐ 미착수 | [#25](https://github.com/july-rust/BoardSI/issues/25): 기타 CAD·성능·모델 확대 |

**주의:** M0/M1/M2는 사양서의 제품 개발 단계이며 A_001…A_036, B_001… 명령 번호와 다릅니다. 제한 P2 Preview를 마쳤다고 M0/M1/M2 전체의 정식 인수를 완료 처리하면 안 됩니다.

<a id="c_007"></a>
### C_007 — Multi-Segment Source Reference Route / Qt Risk (제한 코드 인수 완료 · 2026-10-29 KST)

- [x] **C_007 2–16개 Source-order Straight Trace 경로의 Reference Gap Source Geometry 검사 및 Qt CPU/RHI 표시 — 한정 코드·Native 및 병합 후 인수** · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST**
  - [Issue #255](https://github.com/july-rust/BoardSI/issues/255), [C_007 검증 원장](validation/c007-multi-segment-reference-coverage.md), [코드 PR #256](https://github.com/july-rust/BoardSI/pull/256). 동일 Signal
    Net/Layer, 정확한 Source Endpoint 연결·Source SHA와 C_006 V2 Rational 각 Segment 증명, 미접촉/분기/역행 겹침·Arc/Via 미지원 차단 및 Qt stale-result 제거. **최종 코드 HEAD**
    [`fdfad35931805e47d6658aeed7966ed3d778b741` 27/27 CI SUCCESS](https://github.com/july-rust/BoardSI/commit/fdfad35931805e47d6658aeed7966ed3d778b741/checks) (Rustfmt/Clippy, Debug/Release Rust 및 기존 B/C,
    Qt Windows x64/ARM64·macOS ARM64 Native 실프로세스 C_007 테스트, Linux Qt). **독립 Codex 동일 HEAD completed / findings 0 / unresolved 0**, 기존 역방향 중첩 리뷰 수정·스레드 해결. **보호된 Signed Verified Squash**
    [`main@08849e3fdf048f60a8a06265792acd4f51316b2f`](https://github.com/july-rust/BoardSI/commit/08849e3fdf048f60a8a06265792acd4f51316b2f) (`verified=true, reason=valid`), **독립 post-main
    [21/21 Checks SUCCESS](https://github.com/july-rust/BoardSI/commit/08849e3fdf048f60a8a06265792acd4f51316b2f/checks) 및 9/9 Workflow SUCCESS**
    ([Push main #40781834140](https://github.com/july-rust/BoardSI/actions/runs/40781834140), [Qt Native #40781834617](https://github.com/july-rust/BoardSI/actions/runs/40781834617)). (최초)
  - . **제한 Source Geometry와 Qt 위험 표시만 [x]**, 물리 Reference 전류·P009·RLGC/S·Solver/Cache·정식 U09/U10/U39·T02/T04/T24/T52는 [ ] 유지, 상위 [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN.


<a id="c_008"></a>
### C_008 — Two-Layer Source Plated Via / Reference Gap Qt Risk (한정 코드 및 병합 후 인수 완료 · 2026-10-29 KST)

- [x] **C_008 한정 PADS Source Two-Layer Via 전이 경로·Layer별 Reference Gap 및 Qt CPU/RHI Risk 표시** · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST**
  - [Issue #258](https://github.com/july-rust/BoardSI/issues/258), [C_008 원장](validation/c008-multilayer-reference-via-transition.md), [코드 PR #259](https://github.com/july-rust/BoardSI/pull/259), 최종 코드
    HEAD [`ee765510b1b6f761c31adea8990eb5ef42405f18`](https://github.com/july-rust/BoardSI/commit/ee765510b1b6f761c31adea8990eb5ef42405f18/checks) **27/27 Check Runs SUCCESS**, 동일 HEAD Codex 완료·지적 0·미해결 0,
    보호 Signed Verified Squash [`main@5db1840aa267624d20693aef4e48186383e04a57`](https://github.com/july-rust/BoardSI/commit/5db1840aa267624d20693aef4e48186383e04a57) 확인. Source SHA·Trace Net/Layer·연속
    Endpoint와 단일 도금 Via Canonical Conductive Padstack, 기존 C_006 V2 Rational Gap, Source/Generation stale 차단 및 Qt CPU/RHI Marker를 **공개 합성 PADS V9.4 MILS 제한 범위만** 확인. **코드 post-main
    [`5db1840aa267624d20693aef4e48186383e04a57` 21/21 Checks SUCCESS](https://github.com/july-rust/BoardSI/commit/5db1840aa267624d20693aef4e48186383e04a57/checks) 및 9/9 Workflow SUCCESS**
    ([Push main #40791493502](https://github.com/july-rust/BoardSI/actions/runs/40791493502), [Qt Native #40791493610](https://github.com/july-rust/BoardSI/actions/runs/40791493610),
    [FEM Native #40791493531](https://github.com/july-rust/BoardSI/actions/runs/40791493531)) 독립 확인. (최초)
  - — 한정 코드·Qt 위험 표시만 완료. 문서 PR #260 별도 검증·병합 예정. 실제 Return Current/P009/RLGC/S·물리 Solver/Cache/전체 U09/U10/U39·T02/T04/T24/T52는 미승인, [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN 유지.

<a id="c_009"></a>
### C_009 — Source Reference Stitching Via Candidate 및 Qt Risk 표시 (한정 코드·병합 후 인수 완료 · 2026-10-29 KST)

- [x] **C_009 공개 합성 PADS V9.4 MILS 두 Reference Layer의 Source-exact Through Stitch Via 후보 접점 증명·Qt CPU/RHI 위험 마커·Stale 무효화 — 제한 Source Geometry 기능 인수** · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST**
  - [Issue #261](https://github.com/july-rust/BoardSI/issues/261), [코드 PR #262](https://github.com/july-rust/BoardSI/pull/262), [검증 원장](validation/c009-reference-stitching-via-candidate.md). 같은 Source
    Reference Net의 두 Layer, 도금 Through Via, Canonical/Source 동일성, 각 Layer의 별도 COPCLS/COPCUT strict interior ViaCentre↔AreaInterior 접점, V4 및 V2 계약 불변, 정확한 Source Tick 거리²/표시용 mm 근삿값, Qt CPU/RHI 경고 및 Source
    SHA/선택/재Import/비동기 stale 차단만 포함. **최종 코드 HEAD**
    [`bdd29c061086b5d68742b3915dddd9e2bcf83033` 27/27 Check Runs·11/11 PR Workflows SUCCESS](https://github.com/july-rust/BoardSI/commit/bdd29c061086b5d68742b3915dddd9e2bcf83033/checks), 독립 Codex **동일 HEAD
    completed
  - findings 0
  - unresolved 0**, 과거 지적 3건 수정 및 스레드 3/3 해결. **보호 Squash Signed Verified**
    [`main@a246bc9f15a3b27cf33cdd65370308ee6051fece`](https://github.com/july-rust/BoardSI/commit/a246bc9f15a3b27cf33cdd65370308ee6051fece) (`verified=true, reason=valid`), **독립 코드 post-main
    [21/21 Checks SUCCESS](https://github.com/july-rust/BoardSI/commit/a246bc9f15a3b27cf33cdd65370308ee6051fece/checks)
  - 8/8 Push Workflows SUCCESS** ([Qt Native #40814575323](https://github.com/july-rust/BoardSI/actions/runs/40814575323),
    [Numerical Core #40814574904](https://github.com/july-rust/BoardSI/actions/runs/40814574904)). (최초)
  - . 본 기록의 별도 문서 PR/병합 후 검증은 분리 게이트. 전기적 Return Current/P009/RLGC/S, 물리 Solver/Cache, 정식 U09/U10/U39/T02/T04/T24/T52 전체 인수는 **[ ]/미검증**; [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN.

<a id="c_010"></a>
### C_010 — Reference Copper Corridor / Stitch Source Area ID 정합성 (한정 코드·병합 후 인수 · 2026-10-29 KST)

- [x] **C_010 공개 합성 PADS V9.4 MILS 지원 Straight Signal 경로의 두 Reference Layer Copper Corridor Source Area와 C_009 Stitch Via 접촉 Source Area ID 동일성 검증 및 Qt CPU/RHI 위험 표시 — Source Geometry 한정 인수** · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST**
  - [Issue #264](https://github.com/july-rust/BoardSI/issues/264), [코드 PR #265](https://github.com/july-rust/BoardSI/pull/265), [C_010 검증 원장](validation/c010-reference-corridor-stitch-continuity.md). 전체
    Signal leg 중심선의 Source-Owned Reference COPCLS/COPCUT Area 엄격 내부 포함, Cutout·Boundary·동일-Net 분리 Island·서로 다른 Area ID 및 변조 Source 거부, V5/V4/V2 기존 계약 불변 및 Qt 변경/재Import/비동기 stale 결과 무효화 **한정**. **최종 코드
    HEAD** [`4e050874fbf621b2e8f28d50c3ab06ff93d77b05` **27/27 Checks·12/12 PR Workflows SUCCESS**](https://github.com/july-rust/BoardSI/commit/4e050874fbf621b2e8f28d50c3ab06ff93d77b05/checks), 동일 HEAD
    [Codex `completed`, findings 0, unresolved 0](https://github.com/july-rust/BoardSI/pull/265#issuecomment-6488503225), 기존 P1/P2 리뷰 스레드 2/2 해결. 보호된 Signed Verified Squash
    [`main@565e63cb2f3cc4deaca7f09543340747f6024bb0`](https://github.com/july-rust/BoardSI/commit/565e63cb2f3cc4deaca7f09543340747f6024bb0) (`verification.verified=true, reason=valid`, one-parent), **독립
    post-code-main 21/21 Checks·9/9 Workflow Runs SUCCESS** ([Qt Native #40823686048](https://github.com/july-rust/BoardSI/actions/runs/40823686048),
    [Numerical Core #40823686042](https://github.com/july-rust/BoardSI/actions/runs/40823686042)). (최초)
  - . 문서 PR 자체의 동일 HEAD 리뷰·CI·보호 병합 및 post-doc-main 검증은 별도 게이트. **실제 전류/P009, RLGC/S, 물리 Solver/Cache 및 전체 U09/U10/U39/T02/T04/T24/T52는 미검증**. [상위 M1 #12](https://github.com/july-rust/BoardSI/issues/12)는
    OPEN.

<a id="c_011"></a>
### C_011 — Source Stitch Via Pad Footprint V7 (한정 코드 범위 완료)

- [x] **C_011 한정 Source-exact 원형 Reference Stitch Via Conductive Pad 전체 면적 / COPCLS·COPCUT 외곽·Cutout 위험 판정 및 Qt CPU/RHI 원본 크기 외곽 표시·선택 검증** · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST**
  - . [Issue #267](https://github.com/july-rust/BoardSI/issues/267), [코드 PR #268](https://github.com/july-rust/BoardSI/pull/268), [V7 검증 원장](validation/c011-reference-stitch-pad-footprint.md). 최종 HEAD
    `e7f61e3984b9f84a7cfbd3fcf25480c423a41620` **27/27 PR Checks·11/11 PR Workflows SUCCESS**, 동일 HEAD Codex 리뷰 **completed / findings=0 / unresolved=0**, RHI P2 2건 해결. 서명된 Squash
    `main@32485198fbb24058435f8ef1821f561edef6a793` **Verified/valid**, 병합 후 독립 **21/21 Checks·9/9 Workflows SUCCESS**. *이 체크는 공개 합성 PADS V9.4 MILS Source 기하/Qt Preview 기능의 제한적 코드 인수만 표시한다.* 물리 Return
    Path/P009, 실제 Copper Fill·Thermal/Antipad, RLGC/S·제품 Solver/Cache, 정식 U09/U10/U39/T02/T04/T24/T52 및 상위 M1 [#12](https://github.com/july-rust/BoardSI/issues/12)는 모두 **미완료/OPEN**. C_011 **문서 PR 최종 보호
    병합·post-doc-main 증거와 Issue 종료는 [#267](https://github.com/july-rust/BoardSI/issues/267)에서 별도로 판정**하며 코드 체크가 이를 대체하지 않는다.

<a id="c_012"></a>
### C_012 — Source Signal/Stitch Via Pad Clearance V8 (한정 코드 범위 완료)

- [x] **C_012 이종 Net Signal/Reference Via 원형 Conductive Pad의 Source Tick 기준 겹침·접촉·분리 판정 및 Qt CPU/RHI 위험 표시 — 한정 Source-geometry 코드 인수** · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST**
  - . [Issue #270](https://github.com/july-rust/BoardSI/issues/270), [코드 PR #271](https://github.com/july-rust/BoardSI/pull/271), [V8 검증 원장](validation/c012-signal-reference-via-pad-clearance.md). 최종 코드
    HEAD [`5874567053f2f7320e69fc674f0260808bc1deaa`](https://github.com/july-rust/BoardSI/commit/5874567053f2f7320e69fc674f0260808bc1deaa/checks) **27/27 PR Checks·11/11 PR Workflows SUCCESS**. 동일 HEAD
    Codex **completed / findings=0 / unresolved=0**, 기존 P1/P2 스레드 **2/2 해결**. 서명된 보호 Squash
    [`main@ae187f1f4437181676239106260b64238bc0afd8`](https://github.com/july-rust/BoardSI/commit/ae187f1f4437181676239106260b64238bc0afd8) **Verified/valid**·부모 1개, 독립 병합 후 **21/21 Checks·9/9 Workflows
    SUCCESS**, Ruleset #25046595 Strict/17개 필수/Bypass 없음. **이 체크는 공개 합성 PADS V9.4 MILS Source 기하 및 Qt Preview의 한정된 코드 완료만 표시한다.** 문서 PR [#272](https://github.com/july-rust/BoardSI/pull/272)의 최종
    CI·Codex·서명된 병합·post-doc-main 및 Issue #270 종료는 **별도 미완료 게이트**로 유지한다. P009/Return Current, 제조 Copper·Antipad/Thermal, RLGC/S, 제품 Solver/Cache, U09/U10/U39/T02/T04/T24/T52 및 상위 M1
    [#12](https://github.com/july-rust/BoardSI/issues/12)는 **미완료/OPEN**.

<a id="c_013"></a>
### C_013 — Source Signal Via Pad / Reference COPCUT V9 (제한 기능 PASS · 2026-10-29 KST)

- [x] **C_013 한정 Source COPCUT 직사각형과 Signal Via 원형 Pad Source Tick 간격 및 Qt CPU/RHI 마커** · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST**
  - [Issue #273](https://github.com/july-rust/BoardSI/issues/273)
  - [코드 PR #274](https://github.com/july-rust/BoardSI/pull/274)
  - [검증 원장](validation/c013-signal-via-reference-cutout-clearance.md). 최종 코드 HEAD `d15169e1c5b5367bc96ff9cb1cfbc37144db9d6e`의 **PR CI 27/27 SUCCESS**, 동일 HEAD Codex 완료·Findings 0·미해결 Thread 0(기존 P2 3건 모두
    해결), Native Qt Windows x64/ARM64/macOS ARM64/Linux offscreen 및 Rust V9 공개 합성 테스트/기존 회귀.
    [보호 Squash Verified `main@96f49248`](https://github.com/july-rust/BoardSI/commit/96f49248d0cda95b729734683d9918cd9444a45a) GitHub 서명 `verified=true, reason=valid`, 독립 **post-main 21/21 Check Runs
    SUCCESS + 9/9 Push Workflows SUCCESS**. Source 원본 정수 Tick Gap/Tangency/Overlap/Missing 분류 및 기존 V8 Byte-exact Receipt 보존이라는 **한정 기능만 PASS**. P009 Return Current/제조 Copper/실제 DRC/실물 PCB
    SI/Solver/Cache/정식 U/T 및 M1 #12는 **미완료**. 문서 자체의 보호 병합·최종 `main` 검증과 Issue 종료는 [#273](https://github.com/july-rust/BoardSI/issues/273)에서 별도로 증빙.

<a id="c_014"></a>
### C_014 — Multi-COPCUT Signal Via / Reference Source geometry V10 (한정 코드 범위 PASS · 2026-10-30 KST)

- [x] **C_014 Source-owned Reference Area별 독립 직사각형 COPCUT 0–8개/레이어 및 원형 Signal Via Pad Source Tick 간격·Qt CPU/RHI 위험 표시(한정 코드 인수)** · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST**
  - . [Issue #276](https://github.com/july-rust/BoardSI/issues/276)
  - [코드 PR #277](https://github.com/july-rust/BoardSI/pull/277)
  - [검증 원장](validation/c014-multi-cutout-signal-via-clearance.md). 코드 최종 HEAD
    [`fd95810bb8e60ca57a0c42b59ba83c6297021bcb`](https://github.com/july-rust/BoardSI/commit/fd95810bb8e60ca57a0c42b59ba83c6297021bcb/checks) **PR 11/11 Workflow SUCCESS**, Native Qt Windows
    x64/ARM64·macOS ARM64·Linux offscreen, Rust Debug/Release/fmt/Clippy 및 이전 B/C 회귀 PASS. **독립 동일 HEAD Codex completed / findings 0 / unresolved 0**, 기존 P1/P2 2/2 스레드 해결. 보호 Squash
    [`main@83623c167c27cdb51c58d9a4feab434068d7ed40`](https://github.com/july-rust/BoardSI/commit/83623c167c27cdb51c58d9a4feab434068d7ed40)는 GitHub 서명 **Verified/valid
  - 부모 1개**. [독립 post-main 증거 #40965201300](https://github.com/july-rust/BoardSI/actions/runs/40965201300): **21/21 Check Runs·GitHub API `event=push` 8/8 Workflow Runs SUCCESS**. 0~1개 Cutout 시 V9, 다중 시 V8
    Receipt Byte-exact 호환; Cutout 중첩/경계 접촉/Source 불명확 시 Fail-closed. **이 체크는 공개 합성 PADS V9.4 MILS Source Geometry·Qt 한정 인수만 뜻하며 문서 PR #278의 최종 병합·CI 검증은 별도이다.** 물리 Return Current P009, 제조
    Copper/Antipad/Thermal, 실제 DRC·SI·Solver·Cache, 공식 U09/U10/U39/T02/T04/T24/T52와 상위 M1 [#12](https://github.com/july-rust/BoardSI/issues/12)는 **미완료/OPEN**.

<a id="c_015"></a>
### C_015 — Circular COPCCO / rectangular COPCUT mixed Source Geometry V11 (한정 인수 완료 · 2026-10-30 KST)

- [x] **C_015 원형·직사각형 혼합 Cutout Source Geometry·Qt 독립 V11 한정 인수 완료** · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST**
  - . [Issue #280](https://github.com/july-rust/BoardSI/issues/280)
  - [Code PR #281](https://github.com/july-rust/BoardSI/pull/281)
  - [Verified protected Squash `273417a7555a1dddc5724b15b062029ddf3b6e52`](https://github.com/july-rust/BoardSI/commit/273417a7555a1dddc5724b15b062029ddf3b6e52)
  - [검증 원장](validation/c015-circular-mixed-cutout-source-geometry.md). 원본 PADS COPCCO/COPCUT 혼합 형상에 대한 i128 정수(Source Tick=254 nm) 검사, Canonical/cardinal-circle Source-local ViaCentre 접속, **진짜 원본 V8 필수
    계보**, Qt 독립 Signal/Stitch Via/void·nested CONTACT ID 검증과 원래 크기의 CPU/RHI 위험 표시·캐시 무효화 구현. 정확한 코드 HEAD `95daabc02339aa37a3c5ddb84af7e4798c54e96b`에서 Rust Debug/Release 원형 V8 및 음성 fixture PASS,
    [Numerical run 40984559607](https://github.com/july-rust/BoardSI/actions/runs/40984559607), [Qt Native run 40984559702](https://github.com/july-rust/BoardSI/actions/runs/40984559702), **PR 12/12
    Workflow SUCCESS**; Codex 해당 HEAD 리뷰 완료 findings=0·unresolved=0, P2 수정 스레드 해결; 서명 `verified=true, reason=valid`인 Squash 보호 병합. 원본 main code commit `273417a…`의 **사후 Checks 21/21 SUCCESS, Push Workflows
    8/8 SUCCESS**. **C_015 제한적인 원본 형상/접속 후보 결과만 인수**: `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`. 정식 **U09/U10/U39/T02/T04/T24/T52** 및 상위 M1
    [#12](https://github.com/july-rust/BoardSI/issues/12)는 계속 미완료/OPEN; 어떠한 SI·리턴전류·실제 제조품 DRC 인수도 의미하지 않음.

<a id="c_016"></a>
### C_016 — Reference Stitch Via full circular Pad / mixed cutout Source V12 (한정 코드 인수 PASS · 2026-10-30 KST)

- [x] **C_016 Source Reference Copper 외곽·COPCUT/COPCCO 대비 Reference Stitch Via 원형 Conductive Pad 전체의 경계·접촉·중첩 위험 및 Qt 독립 V12 한정 인수** · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST**
  - (최초 체크일). [Issue #284](https://github.com/july-rust/BoardSI/issues/284)
  - [코드 PR #285](https://github.com/july-rust/BoardSI/pull/285)
  - [검증 원장](validation/c016-reference-stitch-via-pad-clearance.md). **정확한 최종 코드 HEAD
    [`21dc6f8347568e777ad1182f38c00ae41f0c4436`](https://github.com/july-rust/BoardSI/commit/21dc6f8347568e777ad1182f38c00ae41f0c4436/checks)**: Rust 정수 Source 기하·원본 V11/V8 Receipt·Native Qt 실 Importer·기존
    회귀 및 **Windows x64/ARM64·macOS ARM64의 C_016 직접 selftest PASS**, Linux offscreen 기존 등록 Qt CTest 통과(**C_016 전용 Linux Qt selftest는 미실행·인수 주장하지 않음**). 전체 **Check Runs 27/27 SUCCESS, PR Workflows 12/12
    SUCCESS**. 독립 동일 HEAD Codex `completed`, findings=0, unresolved=0, 이전 3/3 리뷰 스레드 해결. 보호된 **Signed Verified Squash
    [`main@4b5cb04ca537f8d9b4400978dfb3104ef744c59f`](https://github.com/july-rust/BoardSI/commit/4b5cb04ca537f8d9b4400978dfb3104ef744c59f)** (`verified=true, reason=valid`, 부모 1개). **독립 post-code-main
    Check Runs 21/21 SUCCESS 및 `event=push` Workflows 8/8 SUCCESS**. 이 체크는 **공개 합성 PADS V9.4 MILS subset의 Source Geometry/Qt Preview 한정 코드 인수**이며 문서 PR의 별도 최종 검증을 대신하지 않는다. 실제 Reference Return Path P009·전기
    접속·제조 Clearance/DRC·임피던스/RLGC/S-parameter 및 Solver/Cache 인수는 **미완료** (`return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`). 정식
    U09/U10/U39/T02/T04/T24/T52와 M1 [#12](https://github.com/july-rust/BoardSI/issues/12)는 **미완료/OPEN**.

<a id="c_017"></a>
### C_017 — Reference Gap/Layer Transition V1–V4 한정 통합 안정화 (검증 진행 중 · 2026-10-30 KST)

- [x] **C_017 V1–V4 Source SHA·Gap·Plated Via Risk 한정 통합 회귀 및 기존 Qt Native 증거 정합성 인수** · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST**
  - . [Issue #287](https://github.com/july-rust/BoardSI/issues/287), [코드 PR #288](https://github.com/july-rust/BoardSI/pull/288), [검증 원장](validation/c017-reference-gap-layer-transition-integration.md). 최종
    Source HEAD [`d85846799fa81bf236ad0e72deb9fbb8452b7e19`](https://github.com/july-rust/BoardSI/commit/d85846799fa81bf236ad0e72deb9fbb8452b7e19/checks) **22/22 Checks SUCCESS**, Rust 신규 **Debug/Release
    4/4 각각 PASS**, 기존 Qt Windows x64/ARM64·macOS ARM64 및 Linux Qt 검증 SUCCESS. 동일 HEAD Codex `completed`, findings=0, unresolved=0. 보호된 Signed Verified Squash
    [`main@046ba46ba93fc6ced803519bba96982f3b7311d7`](https://github.com/july-rust/BoardSI/commit/046ba46ba93fc6ced803519bba96982f3b7311d7) 검증, **독립 post-code-main 21/21 Checks·8/8 Push Workflows
    SUCCESS**. **Source-only 한정 코드 인수 [x]**이며 이 문서 PR의 병합/CI는 별도 Gate. 전체 실제 P009·전기적 Return Path·Solver/Cache·정식 U/T는 미완료로 유지.

<a id="c_018"></a>
### C_018 — Reference Stitch Via / Copper Corridor V5–V8 제한 통합 회귀 인수 (2026-10-30 KST)

- [x] **C_018 V5–V8 Source SHA/Area ID/Pad Footprint/Pad Clearance 한정 통합 회귀 및 Qt Native 기존 증거 정합성 인수** · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST**
  - . [Issue #290](https://github.com/july-rust/BoardSI/issues/290), [코드 PR #291](https://github.com/july-rust/BoardSI/pull/291), [검증 원장](validation/c018-reference-stitch-corridor-integration.md). **코드**
    최종 HEAD `ad43b4703985f53c6dd5c1bcf202d412535f6fcf` 22/22 PR Checks·8/8 PR Workflows SUCCESS, Rust 신규 Debug/Release 각각 5/5 PASS, 기존 Windows x64/ARM64·macOS ARM64 및 Linux Qt Native/Offscreen PASS, 동일
    HEAD Codex 완료·지적 0·미해결 스레드 0, Signed Verified Squash `main@f387fbbda933479b765aa685f7cffe66a8bfff69`. **독립 post-code-main
    [21/21 Checks SUCCESS](https://github.com/july-rust/BoardSI/commit/f387fbbda933479b765aa685f7cffe66a8bfff69/checks), 9/9 Workflows SUCCESS** (Windows x64/ARM64·macOS ARM64/Linux Qt 포함). 이 [x]는
    **Source-only/Qt Preview 제한 코드 인수**를 나타내며 문서 PR의 추가 보호 병합과 독립 post-doc-main 검증은 별도 Gate로 남는다. 물리 P009·전기적 Return Path·제조 DRC, Solver/Cache, 정식 U09/U10/U39/T02/T04/T24/T52와 상위
    [#12](https://github.com/july-rust/BoardSI/issues/12)는 미완/OPEN 유지.

<a id="c_019"></a>
### C_019 — PADS Mixed Cutout / Signal and Reference Via Pad V9–V12 한정 통합 회귀 인수 (2026-10-30 KST)

- [x] **C_019 V9–V12 Source SHA·COPCUT/COPCCO·Signal/Reference Via Pad 위험 및 기존 Qt Native 회귀 한정 인수** · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST**
  - (최초 체크일). [Issue #293](https://github.com/july-rust/BoardSI/issues/293), [코드 PR #294](https://github.com/july-rust/BoardSI/pull/294), [검증 원장](validation/c019-mixed-cutout-via-pad-integration.md). 최종 코드
    HEAD [`07971e14761c178db6ca790b8851f391a155d591`](https://github.com/july-rust/BoardSI/commit/07971e14761c178db6ca790b8851f391a155d591/checks) **22/22 PR Check Runs·9/9 PR Workflows SUCCESS**, 합성 PADS
    V9.4 MILS 통합 Rust **Debug 5/5 및 Release 5/5 PASS**, Rustfmt/Clippy·기존 B/C 회귀 PASS, 기존 Qt Windows x64/ARM64·macOS ARM64 및 Linux offscreen CI SUCCESS (C_019 신규 Qt Selftest는 없음). 독립 동일 HEAD Codex
    `completed`, findings=0, unresolved=0, 리뷰 스레드 0. 보호된 Signed Verified one-parent Squash
    [`main@9f4b425f3a8f843e0ac9eade7f69329298d62de8`](https://github.com/july-rust/BoardSI/commit/9f4b425f3a8f843e0ac9eade7f69329298d62de8) (`verified=true, reason=valid`). **독립 post-code-main
    [21/21 Checks SUCCESS](https://github.com/july-rust/BoardSI/commit/9f4b425f3a8f843e0ac9eade7f69329298d62de8/checks) 및
    [9/9 Workflows SUCCESS](https://github.com/july-rust/BoardSI/actions/runs/41015286806)**. 이 `[x]`는 **Source Geometry/기존 Qt Preview 한정 코드 회귀 인수**이며 본 문서 후속 PR의 보호 병합·post-document-main은 별도 검증 대상. 실제
    P009·전기적 Return Path·제조 Clearance/DRC, 물리 RLGC/S·Solver/Cache, 정식 U09/U10/U39/T02/T04/T24/T52와 [상위 #12](https://github.com/july-rust/BoardSI/issues/12)는 미완/OPEN.

<a id="c_020"></a>
### C_020 — B_007 최종 리뷰 증거와 B_003–B_008 Connectivity 제한 감사 (2026-10-30 KST)

- [x] **C_020 B_007 원본 최종 HEAD Codex 상태/리뷰 스레드 및 Legacy B_003–B_008 보호 병합 증거 재감사** · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST**
  - (최초 체크일). [C_020 Issue #296](https://github.com/july-rust/BoardSI/issues/296)
  - [감사 원장](validation/c020-legacy-connectivity-review-audit.md)
  - [B_007 PR #231](https://github.com/july-rust/BoardSI/pull/231)
  - [B_007 Codex 원본 요약](https://github.com/july-rust/BoardSI/pull/231#issuecomment-6455692908)
  - [B_007 문서 병합 PR #232](https://github.com/july-rust/BoardSI/pull/232). 직접 확인: #231 최종 HEAD `6a75eb5c5616d86407a04a946bc994958fb55ea6`, latest original Codex `status=running` / findings=null, 제출된 Codex
    리뷰는 더 이른 `40faa94332` 대상, 3개 구 리뷰 Thread resolved, 원본 최종 HEAD **리뷰 종료 NOT_VERIFIED**. B_003~B_008 코드 PR #224/#226/#227/#229/#231/#233 모두 GitHub merged 상태. 역사적 Source Geometry 제한 기능/CI 합격 근거는 이전 원장에
    귀속하며 이 C_020 감사에서 Rust/Qt/물리 SI 테스트를 새로 실행했다고 주장하지 않는다. **이 항목 [x]는 정확한 이력 조회 및 미확정 판정 기록만 완료했다는 뜻**이며 원본 B_007 최신 HEAD Codex PASS 또는 신규 C_020 PR CI/병합의 완료가 아니다.
- [x] **C_020 감사 PR의 정확한 최종 HEAD CI·독립 Codex·보호 Squash·독립 post-main 검증 제한 인수** · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST**
  - . [감사 PR #297](https://github.com/july-rust/BoardSI/pull/297)의 정확한 HEAD
    [`8c777a3ffc24c63a78ba83fae813feafa20bd76f`](https://github.com/july-rust/BoardSI/commit/8c777a3ffc24c63a78ba83fae813feafa20bd76f/checks) **22/22 GitHub Checks SUCCESS·8/8 PR Workflows SUCCESS**, 동일
    HEAD 독립 Codex `completed`/findings=0/unresolved=0/Review Threads 0, Qt Windows x64/ARM64·macOS ARM64·Linux offscreen 및 Rust Numerical 기존 회귀 PASS. 보호된 단일 부모 Signed Verified Squash
    [`main@fee8ae63e912ced0f8fb823374246949bc0f7edc`](https://github.com/july-rust/BoardSI/commit/fee8ae63e912ced0f8fb823374246949bc0f7edc) REST `verified=true`/`reason=valid`; 독립
    [post-main **21/21 Checks SUCCESS**](https://github.com/july-rust/BoardSI/commit/fee8ae63e912ced0f8fb823374246949bc0f7edc/checks) 및 **9/9 Workflows
    SUCCESS**([Push on main #41020421444](https://github.com/july-rust/BoardSI/actions/runs/41020421444)). [감사 원장](validation/c020-legacy-connectivity-review-audit.md). 이 [x]는 제한 B_003–B_008 Source
    Geometry **증거 감사**와 검증된 문서 병합만 뜻한다. B_007 원본 최종 HEAD Codex 리뷰는 여전히 `NOT_VERIFIED`; 새 C_020 리뷰로 소급 대체 금지. `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`,
    `cache_reuse_eligible=false`; Issue #12 OPEN, 정식 U09/U10/U39·T02/T04/T24/T52 미완.
- [x] **C_020 최종 문서 PR #298 독립 최종 HEAD CI·Codex·보호 Signed Verified Squash·post-document-main 인수** · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST**
  - (본 항목 최초 체크일). [최종 문서 PR #298](https://github.com/july-rust/BoardSI/pull/298)의 정확한 HEAD
    [`b8932646810ca4a2c4452a9527463cc192f251ce`](https://github.com/july-rust/BoardSI/commit/b8932646810ca4a2c4452a9527463cc192f251ce/checks): **22/22 PR Check Runs SUCCESS·8/8 Workflows SUCCESS**, 동일 최종
    HEAD Codex `completed`/findings=0/unresolved=0, 리뷰 미해결 스레드 0. 보호된 단일 부모 Verified/valid Squash
    [`main@0dbcd03eac482b78d3fc04c88dbe8168c991db00`](https://github.com/july-rust/BoardSI/commit/0dbcd03eac482b78d3fc04c88dbe8168c991db00): GitHub REST `verified=true`/`reason=valid`, 단일 부모
    `fee8ae63e912ced0f8fb823374246949bc0f7edc`. 같은 병합 SHA의 독립 [post-document-main **21/21 Checks SUCCESS**](https://github.com/july-rust/BoardSI/commit/0dbcd03eac482b78d3fc04c88dbe8168c991db00/checks),
    **9/9 Workflows SUCCESS** ([Push on main #41022470172](https://github.com/july-rust/BoardSI/actions/runs/41022470172)); [C_020 Issue #296](https://github.com/july-rust/BoardSI/issues/296) CLOSED. 기존
    감사·첫 체크일 보존. **이 문서 증거 대조는 역사적 B_007 원본 최종 HEAD Codex 미확정 상태를 바꾸지 않으며 실제 Return Path/P009, Solver/Cache 또는 정식 U09/U10/U39·T02/T04/T24/T52 승인도 아니다.** 이 최종 체크 표시 정합화 PR 자체는 별도의 검증/병합 게이트를 통과해야 한다.


<a id="c_022"></a>
### C_022 — PADS Multi-Area Source Copper Component Candidates V14 (소스 전용 범위 완료 · 최초 체크 2026-10-31 KST)

- [x] **C_022 3~6개 Source-owned Reference Copper 직사각형 간 양의 면적 중첩 전이적 후보 그룹과 분리 Island 위험 표시** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST**
  - [검증 원장](validation/c022-reference-multi-area-connectivity.md)
  - [Issue #304](https://github.com/july-rust/BoardSI/issues/304) (문서 PR #307 최종 HEAD 검증·보호 병합·post-main 승인; Issue #304는 후속 정합화 PR 검증 후 종료)
  - [코드 PR #305](https://github.com/july-rust/BoardSI/pull/305). 정확한 코드 최종 HEAD `e077eeaa8f31766ae94757bb90a90dcdd129cb34`: **27/27 PR Check Runs, 11/11 PR workflows SUCCESS**, 동일 최종 HEAD 독립 Codex
    `completed`·findings=0·unresolved=0, 모든 리뷰 스레드 해결. Synthetic Rust Debug/Release 6개 Source-only V14 회귀, Qt Windows x64/ARM64·macOS ARM64 실제 Rust Importer 연결 Selftest, Linux Qt Offscreen 및 전체 Native/FEM
    CI PASS. 보호된 단일 부모 [Signed Verified Squash `main@242293dda118024697b6d1cb3bde5a521e6fa2cf`](https://github.com/july-rust/BoardSI/commit/242293dda118024697b6d1cb3bde5a521e6fa2cf) GitHub REST
    `verified=true/reason=valid`, 부모 `b6cb674...`; 독립
    [post-code-main **21/21 Check Runs SUCCESS**](https://api.github.com/repos/july-rust/BoardSI/commits/242293dda118024697b6d1cb3bde5a521e6fa2cf/check-runs?per_page=100),
    [8/8 Push workflows SUCCESS](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=push&head_sha=242293dda118024697b6d1cb3bde5a521e6fa2cf&per_page=100). GitHub
    [`macos-14` 예정 중단](https://github.blog/changelog/2026-10-01-github-actions-macos-14-runner-image-retirement/) 대응으로 필수 Native 세 워크플로를 `macos-15`로 교체해 모든 검증 성공. **한정 인수**: Source Geometry만 해당. 전기적
    AreaContinuity/제조 Copper Fill/DRC/물리 P009·Return Current/RLGC/S Solver/Cache·정식 U09/U10/U39/T02/T04/T24/T52 **미승인**. 상위 [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN.
    `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`.


- [x] **C_022 문서 PR #307 독립 완료 게이트** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST**
  - [PR #307](https://github.com/july-rust/BoardSI/pull/307) 최종 리뷰 HEAD `e264b306d3aa5887f1b21af280993a3dbef6702a`,
    [22/22 PR Checks SUCCESS](https://api.github.com/repos/july-rust/BoardSI/commits/e264b306d3aa5887f1b21af280993a3dbef6702a/check-runs?per_page=100),
    [8/8 PR Workflows SUCCESS](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=pull_request&head_sha=e264b306d3aa5887f1b21af280993a3dbef6702a&per_page=100), 동일 HEAD 독립 Codex
    `completed`/findings=false/unresolved=false, 리뷰 스레드 0.
    [Signed Verified one-parent Squash `main@c4e083b9f2d4bd056990e036c05fa3e28b818ddc`](https://github.com/july-rust/BoardSI/commit/c4e083b9f2d4bd056990e036c05fa3e28b818ddc) (`verified=true`,
    `reason=valid`, parent `242293dda118024697b6d1cb3bde5a521e6fa2cf`), 독립
    [post-document-main 21/21 Checks SUCCESS](https://api.github.com/repos/july-rust/BoardSI/commits/c4e083b9f2d4bd056990e036c05fa3e28b818ddc/check-runs?per_page=100) 및
    [8/8 Push workflows SUCCESS](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=push&head_sha=c4e083b9f2d4bd056990e036c05fa3e28b818ddc&per_page=100). **소스 기하 한정 체크만 갱신**, M1 #12
    OPEN·P009/물리 SI/Solver/Cache/정식 U/T 미승인. 본 정합화 PR은 별도 CI·Codex·보호 병합·post-main 확인 후 Issue #304에 최종 종료 근거 기재.
<a id="c_021"></a>
### C_021 — Source Reference Copper Area-to-Area 제한 감사 (소스 전용 범위 완료 · 최초 체크 2026-10-30 KST)

- [x] **C_021 두 Source-owned 직사각형 Reference Area 간 양의 면적 중첩·분리 Island 위험 감사** · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST**
  - [검증 원장](validation/c021-reference-area-connectivity.md)
  - [Issue #300](https://github.com/july-rust/BoardSI/issues/300) (최종 문서 PR 보호 병합·post-main 확인 후 종료)
  - [코드 PR #301](https://github.com/july-rust/BoardSI/pull/301)
  - [증거 문서 PR #302](https://github.com/july-rust/BoardSI/pull/302). 소스 HEAD `a1c7a5a918d848c62a25d495f7e22d58a7513ff0`: **PR 11/11 workflows PASS**, 독립 최신 Codex completed/findings=0/unresolved=0;
    Debug/Release 6개 Rust C_021 자체 시험, Qt Windows x64/ARM64·macOS ARM64 C_021 전용 Selftest, Linux offscreen와 전체 Native·FEM 기존 회귀 PASS. 문서 HEAD `b2ef4517ef949490569ade167076ca8d0fbe30fa`: **PR 8/8 workflows
    PASS**, 최신 Codex findings=0/unresolved=0. 병합 코드 `main@555ad01a1e49a60fefb48b6ef00493b6c8ce0b02`
    [Commit REST](https://api.github.com/repos/july-rust/BoardSI/commits/555ad01a1e49a60fefb48b6ef00493b6c8ce0b02) 및 문서 `main@4c585e7efae6d0f8ab812647e7d409be59e5901d`
    [Commit REST](https://api.github.com/repos/july-rust/BoardSI/commits/4c585e7efae6d0f8ab812647e7d409be59e5901d): 각각 **1 parent, GitHub `verified=true/reason=valid`** (2026-10-30 KST 확인). 코드/문서 양쪽
    [post-main Check Runs 21/21 PASS](https://api.github.com/repos/july-rust/BoardSI/commits/555ad01a1e49a60fefb48b6ef00493b6c8ce0b02/check-runs?per_page=100),
    [문서 post-main Check Runs 21/21 PASS](https://api.github.com/repos/july-rust/BoardSI/commits/4c585e7efae6d0f8ab812647e7d409be59e5901d/check-runs?per_page=100), 각각 **Push workflow 8/8 PASS**
    `event=push`, SHA 일치 ([GitHub Actions REST](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=push&per_page=100)). 원본 PADS SHA/Canonical Area ID/Net·Layer/Source Tick 양의 면적 중첩만 검증한 독립
    V13 `SOURCE_GEOMETRY_ONLY` 구현; C_001 일반 그래프·기존 V1–V12 불변. **범위 밖/미승인:** 실제 Copper 접촉·Return Current/P009, 제조 Fill/DRC, 물리 SI/RLGC/S Solver/Cache, 정식 U09/U10/U39/T02/T04/T24/T52, 상위
    [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`.

<a id="c_023"></a>
### C_023 — Multi-Area Reference Corridor V15 (소스 기하 한정 기능 완료 · 2026-10-31 KST)

- [x] **C_023 V15 PADS Multi-Area Reference Corridor Coverage/Gap 및 Qt Source-only 위험 표시** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST**
  - . [Issue #309](https://github.com/july-rust/BoardSI/issues/309) (**Closed / Completed**, 2026-10-31 KST), [코드 PR #310](https://github.com/july-rust/BoardSI/pull/310) 정확한 최종 HEAD
    `42ef22f32bdd247a473b7c29d8a6dc4c59c954db` **27/27 PR Checks + 11/11 PR Workflows SUCCESS**, 독립 Codex 동일 HEAD `completed`·findings 0·unresolved 0, Windows x64/ARM64·macOS ARM64 V15 Native Qt E2E 및
    Linux offscreen PASS. [보호된 단일 부모 Signed Verified Squash `main@69c37dc4bb0815e13baae59f3b416c95546a38b9`](https://github.com/july-rust/BoardSI/commit/69c37dc4bb0815e13baae59f3b416c95546a38b9)
    `verified=true/reason=valid`; [독립 post-main Checks 21/21 SUCCESS](https://api.github.com/repos/july-rust/BoardSI/commits/69c37dc4bb0815e13baae59f3b416c95546a38b9/check-runs?per_page=100),
    [Push Workflows 8/8 SUCCESS](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=push&head_sha=69c37dc4bb0815e13baae59f3b416c95546a38b9&per_page=100) (2026-10-31 KST).
    [C_023 검증 원장](validation/c023-reference-corridor-coverage.md), [문서 정합화 PR #311](https://github.com/july-rust/BoardSI/pull/311) (**22/22 CI·8/8 PR Workflows 성공, Codex 지적 0건, Signed Verified Squash
    `8f56099d`, post-main 21/21 CI·8/8 Push Workflows 성공**). **완료 범위는 3–6개 Source-owned hole-free 직사각형 Reference Area와 선택된 단일 직선 Signal Trace의 Source Tick 양의 Gap/Boundary/Island 기하 감사와 Qt 위험 표시만**. Copper
    실제 접속/제조 Fill·DRC, Return Current/P009, 물리 RLGC/S 및 Solver/Cache, U09/U10/U39·T02/T04/T24/T52 정식 인수는 **미완료**. `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`,
    `cache_reuse_eligible=false`. 상위 M1 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN.


<a id="c_024"></a>
### C_024 — C단계 통합 증거 감사 및 D단계 인계 (한정 감사 인수 · 최초 체크 2026-10-31 KST)

- [x] **C_024 C_001–C_023 Source Geometry/Qt 통합 회귀 증거 감사 및 D단계 인계 — 한정 감사 인수만 완료** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST**
  - (최초 체크일). [Issue #313](https://github.com/july-rust/BoardSI/issues/313), [C_024 검증/인계 원장](validation/c024-c-series-final-acceptance.md), [감사 PR #314](https://github.com/july-rust/BoardSI/pull/314)의 최종
    HEAD f523e89d345ff84922b3a0a082baf5d1e0afc274 **22/22 PR Checks SUCCESS·8/8 PR Workflows SUCCESS**, 동일 HEAD 독립 Codex completed·findings 0·unresolved 0·리뷰 스레드 0. 보호된 단일 부모
    [Signed Verified Squash main@d985c94ae6b04f0be11b1db0da6871359dd5c199](https://github.com/july-rust/BoardSI/commit/d985c94ae6b04f0be11b1db0da6871359dd5c199) (verification.verified=true, reason=valid,
    parent 4b4de41a920e23014deacefc0e7c0785dd68f34b), 독립 post-main
    [**21/21 Check Runs SUCCESS**](https://api.github.com/repos/july-rust/BoardSI/commits/d985c94ae6b04f0be11b1db0da6871359dd5c199/check-runs?per_page=100) 및
    [**8/8 Push Workflows SUCCESS**](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=push&head_sha=d985c94ae6b04f0be11b1db0da6871359dd5c199&per_page=100). C_001–C_023 최초 체크일 보존;
    C_017–C_019의 새 전용 Qt 시험은 없음(기존 Qt Native 회귀 재사용), B_007 원본 최종 HEAD Codex 완료 NOT_VERIFIED 유지. **이 [x]는 기존 한정 기능의 문서/증거 감사 인수이지 신규 Rust/Qt 물리 기능·실제 Return Path/P009·Solver/Cache 인수가 아니다.** 본 정합화 문서 PR
    자체의 최종 CI·동일 HEAD Codex·보호 Squash·post-main 검증 결과는 [Issue #313](https://github.com/july-rust/BoardSI/issues/313)에 별도로 기록하고 검증 전 Issue 종료 금지. 상위 [M1 #12](https://github.com/july-rust/BoardSI/issues/12)
    OPEN, 정식 U09/U10/U39 및 T02/T04/T24/T52 [ ] 유지. return_path_p009=UNSUPPORTED, physical_si=NOT_RUN, solver_ready=false, cache_reuse_eligible=false.


<a id="c_025_05"></a>
### C_025_05 — M0 F1 사양서/U-T 추적성 및 Issue #2 완료 조건 감사 (2026-10-31 KST)

- [x] **C_025_05 v1.0 사양서/추적 원장 U/T 정의와 그룹 ID 정합성 재확인(문서 검사만)** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST**
  - [감사 문서](validation/c02505-m0-f1-traceability-reconciliation.md) 및 [기존 M0 U/T 원장](audits/m0-requirements-traceability-ledger.md) 대조: U01–U48 정의 48/48, T01–T54 정의 54/54, §18.2 U group의 중복 없는 48/48 매핑, 원장
    U 48개·T 54개 행 확인; [기존 검증 코드](../tools/validation/m0_traceability_check.py)는 CI에서 별도 실행 필요. [기존 10-case 증거](validation/m0-case-evidence.json) 8 PARTIAL/2 NOT_RUN, **정식 개별 인수 0건**.
    [PR #81](https://github.com/july-rust/BoardSI/pull/81) 병합 상태 재확인 및 기존 미완 표기 수정.
  - . 이 항목은 **감사·ID 검사의 제한된 완료만 [x]**, C_025_05 문서 PR HEAD CI/리뷰/보호 병합 및 post-main 증빙은 별도 필요. Issue #2/F1/전체 M0 또는 T01–T54 완료 표시 아님.
- [ ] **Issue #2 및 M0 F1 정식 종료**
  - 실제 M0 적용 시험별 기대 결과·Fixture·Job·SHA·독립 수용 리뷰 미충족. M0 인접 T14/T15/T21/T49의 범위/증빙 추가 정합화 필요. 기존 10개 Case의 T11/T54 NOT_RUN 포함, 전부 Complete Acceptance NOT_RUN. **Issue #2 OPEN / M0 F1 PARTIAL·BLOCKED** 유지.
    F6/F7/정식 제품 시험 및 PH-08 Gate Q 미완 유지.

<a id="c_025_04"></a>
### C_025_04 — 구형 정책·B_003 Draft PR 정합화 및 안전 종료 (문서·PR 관리 한정)

- [x] **C_025_04 PR #153·#223 정책 보존·기능 중복 비교 후 Closed/Unmerged 상태 확인** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST**
  - PR [#153](https://github.com/july-rust/BoardSI/pull/153) `caf275c9`의 유효한 Preview-first 정책은 수정·현행화해 [PR #319](https://github.com/july-rust/BoardSI/pull/319) 문서로 보존했고, 기존 B_003 Draft
    [#223](https://github.com/july-rust/BoardSI/pull/223) `2dd29317`의 기능은 [병합 #224](https://github.com/july-rust/BoardSI/pull/224)와 대조하여 *후속 구현의 제한된 중복*으로 확인했다. 기존 #223 전용 B_003 CLI 프로세스 테스트 및 두 Fixture의
    정확한 동등 범위는 **NOT_VERIFIED**로 [Issue #12](https://github.com/july-rust/BoardSI/issues/12#issuecomment-6542281072)에 남김. GitHub 재조회에서 #153·#223 모두 `closed, merged=false`,
    [Draft #35](https://github.com/july-rust/BoardSI/pull/35) `open, draft=true` 유지.
  - . [감사 원장](validation/c02504-legacy-open-pr-reconciliation.md). **문서/PR 정리만 [x]; 테스트 동등성, 제품 U/T, P009/Return Path, 실제 SI 구현 완료 아님.**
- [x] **C_025_04 정책 문서 보호 병합의 제한적 인수** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST**
  - [PR #319](https://github.com/july-rust/BoardSI/pull/319) 최종 HEAD `3d74d2c49a42477725d21a647f96e5f89a6195a1`, **8/8 PR 워크플로 SUCCESS, 17/17 수집한 워크플로 Job SUCCESS**, 같은 HEAD Codex
    `completed/findings=false/unresolved=false`, Review Thread 0. GitHub 보호 Squash Merge
    [main@e098405d14bcb88588f4601f726c3d7186604c9e](https://github.com/july-rust/BoardSI/commit/e098405d14bcb88588f4601f726c3d7186604c9e) `verification.verified=true/reason=valid`, 단일 부모
    `d428ceef076c484a61aad0b72c49bb574faedbb4`.
  - . *별도 post-main 21개 Check Runs 및 8개 push Workflow 결론은 후속 독립 확인 대상이며 이 체크로 자동 PASS 주장하지 않음.*

<a id="c_025_03"></a>
### C_025_03 — 구형 PADS Signature/Qt Import PR 정합화 및 안전 종료 (기능 인수와 별개)

- [x] **C_025_03 구형 PR #115·#117·#118의 근거 기반 분류 및 미병합 종료** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST**
  - 기존 Signature Probe 핵심 코드 원본 184줄·CLI Blob과 공개 fixture 원본 Blob을 현재 `main`에 대조하여 #115/#117을 SUPERSEDED로 종료하고, 구형 Qt 독립 Signature Shell #118은 현재 비동기 Preview로 아키텍처가 바뀐 **비동등 구형 기능**임을 밝히고 미병합 종료. 각 PR
    `state=closed, merged=false`를 GitHub 응답에서 확인.
  - . [PR #115](https://github.com/july-rust/BoardSI/pull/115), [PR #117](https://github.com/july-rust/BoardSI/pull/117), [PR #118](https://github.com/july-rust/BoardSI/pull/118),
    [감사 원장](validation/c025-03-legacy-pads-pr-reconciliation.md), 기준 [main@d17b562a](https://github.com/july-rust/BoardSI/commit/d17b562a6932355438fa9bd80ca740d88eabe20f). **이 [x]는 구형 PR 상태 정리만 뜻하며 네이티브
    재실행, Signature-only GUI 동등성, U/T 공식 인수, SI/Return Path 또는 배포 인수가 아님.** [M1 #8](https://github.com/july-rust/BoardSI/issues/8), [#9](https://github.com/july-rust/BoardSI/issues/9) OPEN 유지.
- [x] **C_025_03 감사 문서 PR 보호 Squash 병합 및 Post-main 검증 — 문서 정리 한정** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST**
  - [PR #317](https://github.com/july-rust/BoardSI/pull/317) 최종 코드 HEAD
    [`2adf25fd6da8cb84e1949cd6eb6035a9f2f5531d`](https://github.com/july-rust/BoardSI/commit/2adf25fd6da8cb84e1949cd6eb6035a9f2f5531d/checks)의 **22/22 Check Runs SUCCESS**, **8/8 PR workflows SUCCESS**, 동일
    HEAD 독립 Codex **completed/findings 0/unresolved 0**, PR review threads 0. GitHub 보호 규칙 17개 strict checks, signed-only, squash-only 준수: 서명 유효·단일 부모
    [main@c2c4335bec4b7835e19b1fc9bb9c752786a92f20](https://github.com/july-rust/BoardSI/commit/c2c4335bec4b7835e19b1fc9bb9c752786a92f20) (`verification.verified=true`, `reason=valid`; parent
    `d17b562a6932355438fa9bd80ca740d88eabe20f`). 별도
    [post-main **21/21 checks SUCCESS**](https://api.github.com/repos/july-rust/BoardSI/commits/c2c4335bec4b7835e19b1fc9bb9c752786a92f20/check-runs?per_page=100),
    [post-main **8/8 push workflows SUCCESS**](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=push&head_sha=c2c4335bec4b7835e19b1fc9bb9c752786a92f20&per_page=100) 직접 확인.
  - . 이 `[x]`는 **PR #317 감사 문서 통합 게이트만** 뜻하며, 오래된 #118 전용 native signature GUI 테스트/완전한 PADS Import/실측 SI·P009/공식 U/T 자격은 여전히 미완. [감사 원장](validation/c025-03-legacy-pads-pr-reconciliation.md), Issue #8/#9
    OPEN.

## 4. 요구사항 U01–U48 체크리스트

원문: [사양서 2장 요구사항 표](specification/BoardSI_Development_Spec_v1.0.md). `[ ]`는 전체 요구의 정식 인수 **미완료**. U16만 실제 제품명 적용 여부를 확인할 수 있어 체크하였으며 상표권·법률 검증은 별개입니다.

### 기본 기능 U01–U16

- [ ] **U01**
  - Full-wave 3D EM을 제외한 경량 RLGC·임피던스·S-parameter — **부분 증빙 / 최종 미완료**: 제한 수학/무손실 쿠폰만 검증; 실제 PCB SI 해석 미검증 ([M2 검증 원장](validation/README.md))
- [ ] **U02**
  - Rust 계산 코어 + Qt UI — **부분 증빙 / 최종 미완료**: Qt/Rust/Worker Preview 시험 PASS, 최종 배포·전체 제품 미검증 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **U03**
  - Windows x64·ARM64, macOS Apple Silicon — **부분 증빙 / 최종 미완료**: 3개 Native CI 환경에서 Preview 구동, Clean PC 배포 미검증 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **U04**
  - 저사양 PC에서도 가벼운 동작 — **부분 증빙 / 최종 미완료**: 8 GiB Linux 합성 실험, 기준 실물 PC 미검증 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **U05**
  - 공개 GitHub에서 개발, 사내 데이터 비공개 — **부분 증빙 / 최종 미완료**: 공개 GitHub 운영; 기밀 데이터 무유출 최종 인수는 별도 ([SEC_001 코드 검사](validation/sec001-codeql-markdown-comment-scan.md))
- [ ] **U06**
  - PADS Layout ASCII (*.ASC) 지원 — **부분 증빙 / 최종 미완료**: PADS V9.4 MILS 공개 합성 subset만 검증 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **U07**
  - 다양한 PCB 형식 확장(ODB++, IPC-2581, KiCad, Gerber 등) — **정식 인수 미검증**
- [ ] **U08**
  - 자체 2D Artwork, Layer/Net 확인 — **부분 증빙 / 최종 미완료**: CPU/Qt RHI Preview·Layer/Net 필터 제한 검증 ([A_034 Native 통합](validation/a034-p2y-integration-stabilization.md))
- [ ] **U09**
  - Stackup 두께, Copper, Dk/Df, Material 편집 — **제한 Qt + 실제 Rust Canonical Assumed Material 연동 PASS / 정식 인수 미완료**: Native 3 OS/Linux; 주파수 출처·유효 대역 검사 PASS
    ([PR #215](https://github.com/july-rust/BoardSI/pull/215)); 원본 전기적 Copper 전체 형상 및 해석 적용 미완 ([B_001 검증](validation/b001-stackup-reference.md), [PR #213](https://github.com/july-rust/BoardSI/pull/213))
- [ ] **U10**
  - GND·Power·Reference 지정 — **제한 Layer 역할 힌트+사용자 Net 선택 및 Import 기반 Copper 후보 음성 판정 PASS / 전체 인수 미완료**: Source Net 누락·Cutout·분할·동일 Net 설정은 차단; 실제 접속·전원 분류·완전한 Return Path 자격 미검증
    ([B_001_3 원장](validation/b001-3-source-reference-inventory.md)) ([B_001 검증](validation/b001-stackup-reference.md))
- [ ] **U11**
  - Single/Differential 선택, 선폭·두께 가상 변경 — **정식 인수 미검증**
- [ ] **U12**
  - DC~30 GHz 등 사용자 주파수 범위 — **정식 인수 미검증**
- [ ] **U13**
  - CPU/GPU 등 계산 자원 선택 UI — **정식 인수 미검증**
- [ ] **U14**
  - 영어 기본, 한국어 옵션 — **정식 인수 미검증**
- [ ] **U15**
  - 공개 검증 샘플, 기밀 PCB 오프라인 검증 — **부분 증빙 / 최종 미완료**: 공개 합성 fixture 존재, 기밀 실보드 오프라인 전 경로 시험 미완 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [x] **U16** · **체크일: 2026-10-26 KST** · **근거일: 2026-10-14 KST**
  - 제품명 BoardSI 사용 — **현재 제품명 적용 확인** ([사양서 개정일](specification/BoardSI_Development_Spec_v1.0.md))

### 실행/기하/입력 구조 U17–U32

- [ ] **U17**
  - Progressive/Hierarchical Sweep: 초깃값 1 GHz 간격 예시 → 절반 구간 세분화 — **정식 인수 미검증**
- [ ] **U18**
  - 계산 완료 주파수 샘플 캐시 재사용, 동일 조건 재계산 방지 — **정식 인수 미검증**
- [ ] **U19**
  - 실행 중 새로운 계산 결과가 나오면 SI 그래프 실시간 갱신 — **정식 인수 미검증**
- [ ] **U20**
  - Group × Frequency Region별 독립적인 수렴·상태 판정 — **정식 인수 미검증**
- [ ] **U21**
  - 일부 그룹/대역 수렴만으로 전체 실행 조기 종료 금지 — **정식 인수 미검증**
- [ ] **U22**
  - 긴 작업 중 GUI 응답·진행률·취소/일시정지 유지 — **정식 인수 미검증**
- [ ] **U23**
  - 기능별 소형 모듈, UI와 Solver 분리(필요시 별도 프로세스) — **부분 증빙 / 최종 미완료**: 별도 Rust Worker와 versioned Qt IPC 프로토타입 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **U24**
  - RAM/SSD 하이브리드 캐시, 체크포인트, 장애 후 복구 — **정식 인수 미검증**
- [ ] **U25**
  - 분석 전 배선·리턴경로·모델 한계 위험 표시 — **정식 인수 미검증**
- [ ] **U26**
  - 일반 PCB 외 BGA/FC-BGA Substrate SI 대상 — **부분 증빙 / 최종 미완료**: Canonical/Substrate 일부 형상 연구; 전체 Pad-to-Pad 미구현 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **U27**
  - Substrate Trace, Pad/Land, Via 및 Anti-pad, Plane 포함 — **부분 증빙 / 최종 미완료**: Via/Pad/Copper 일부 기하 구현; 전 Substrate 전기적 검증 미완 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **U28**
  - Bump/Cu Pillar/Solder Ball/Wirebond 해석 영역에서 제외 — **정식 인수 미검증**
- [ ] **U29**
  - Substrate 분석 Port 기준은 양 끝 금속 Pad-to-Pad — **정식 인수 미검증**
- [ ] **U30**
  - 검증된 모델 없는 3D 불연속은 정확한 해석인 것처럼 계산 금지 — **부분 증빙 / 최종 미완료**: 미검증 Solver를 Validated로 광고하지 않는 정책; 제품 게이트 미완 ([M2 검증 원장](validation/README.md))
- [ ] **U31**
  - ODB++를 기준 내부 형식으로 사용하지 않고 자체 Canonical Model 사용 — **부분 증빙 / 최종 미완료**: Format-neutral Canonical과 PADS subset 경로 구현 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **U32**
  - Format-specific Importer를 분리하고 동일한 Viewer/Port/Solver 사용 — **부분 증빙 / 최종 미완료**: PADS importer/Canonical/Viewer 경계 구현; 다형식+Solver 통합 미완 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))

### Port·검증·복구 U33–U48

- [ ] **U33**
  - Altium, Allegro, EAGLE, EasyEDA, CADSTAR 등 Import 확대 검토 — **정식 인수 미검증**
- [ ] **U34**
  - Auto Port 후보 생성 + Manual 수정 + 공통 자동 검증 — **정식 인수 미검증**
- [ ] **U35**
  - Single 기본 2-Port, Differential 기본 물리적 4-Port — **정식 인수 미검증**
- [ ] **U36**
  - Reference 도체와 기준 임피던스 분리, Pair Mapping 명시 — **정식 인수 미검증**
- [ ] **U37**
  - Error Port는 Solver 차단, Warning은 표시 및 확인 — **정식 인수 미검증**
- [ ] **U38**
  - Format/Version/객체별 지원 수준과 누락 정보 명시 — **부분 증빙 / 최종 미완료**: PADS 제한 지원표·진단, 기타 파일 형식 미완 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **U39**
  - Import 원본 불변, 정규화 및 Provenance 보존 — **부분 증빙 / 최종 미완료**: subset Source grid·provenance 대조 ([A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md))
- [ ] **U40**
  - Import할 때 보조 데이터(Netlist/Stackup/Drill) 연계 — **정식 인수 미검증**
- [ ] **U41**
  - 고정 Linear 직접 Sweep과 수렴 검사의 독립 기준선 — **부분 증빙 / 최종 미완료**: 수학 직접 기준선 일부, 제품 Direct sweep 없음 ([M2 검증 원장](validation/README.md))
- [ ] **U42**
  - 전체 행렬 및 NEXT/FEXT의 방향·종단·결합 그룹 고정 — **정식 인수 미검증**
- [ ] **U43**
  - 실행·수렴·모델·Limit·입력 신뢰도 상태 분리 — **정식 인수 미검증**
- [ ] **U44**
  - 불변 Run Snapshot, IPC 버전과 늦은 응답 격리 — **부분 증빙 / 최종 미완료**: 제한 immutable Snapshot/Worker IPC/late-message proof ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **U45**
  - 프로젝트 스키마·이전 버전 이관·재개·저장 무결성 — **정식 인수 미검증**
- [ ] **U46**
  - Sweep만 변경 시 동일 물리 조건 직접 계산점 재사용 — **정식 인수 미검증**
- [ ] **U47**
  - 수치 정확도·GUI 응답 목표 및 단계별 측정 게이트 — **부분 증빙 / 최종 미완료**: 일부 수치·Native CI; 전체 제품 정량 인수 미완 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **U48**
  - Import 좌표 정밀도·ODB Step·보조 자료 충돌 통제 — **부분 증빙 / 최종 미완료**: PADS 좌표 grid fidelity 일부; ODB Step/파일충돌 미검증 ([A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md))

## 5. 인수시험 T01–T54 체크리스트

원문: [사양서 13장 인수시험](specification/BoardSI_Development_Spec_v1.0.md). 각 Txx의 *전체* 원문 기준과 실제 실행 자료가 일치하는 경우에만 체크. `정식 인수 미검증`은 아무 시험도 실행하지 않았다는 뜻이 아니라 **해당 T 전체 범위의 증거가 부족**하다는 뜻입니다. 특정 하위 테스트 PASS는 별도 표기하되
  Txx를 체크하지 않습니다.

### 기본 시험 T01–T15

- [ ] **T01**
  - ASC 형식·버전·단위 — **부분 증빙 / 정식 인수 미완료**: 제한 PADS 버전/Signature fixture ([A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md))
- [ ] **T02**
  - 필수 물성·기하 누락 — **제한 Qt 필드 검사 PASS / 정식 인수 미완료**: 실제 PCB·Canonical 입력 전체 누락 검증 아님 ([B_001 검증](validation/b001-stackup-reference.md))
- [ ] **T03**
  - Layer·Via·Padstack·Copper — **부분 증빙 / 정식 인수 미완료**: 일부 Pad/Via/Copper와 국소 접점만 ([A_032 국소 접점](validation/a032-p2w-local-connectivity.md))
- [ ] **T04**
  - Reference 분할·전원 분류 — **정식 인수 미검증**: B_001 역할 힌트는 실제 Copper/Net/분할 판정이 아님 ([B_001 원장](validation/b001-stackup-reference.md))
- [ ] **T05**
  - P/N·Mixed-mode — **정식 인수 미검증**
- [ ] **T06**
  - 균일 선로 수치 정확도 — **부분 증빙 / 정식 인수 미완료**: 합성 수학/무손실 쿠폰 ([M2 검증 원장](validation/README.md))
- [ ] **T07**
  - 손실·수동성·상호성 — **부분 증빙 / 정식 인수 미완료**: 무손실 기준 검사만; 손실 미검증 ([M2 검증 원장](validation/README.md))
- [ ] **T08**
  - Scenario·다중 파일 — **정식 인수 미검증**
- [ ] **T09**
  - DC·Log·주파수 — **정식 인수 미검증**
- [ ] **T10**
  - 자원·취소·복구 — **부분 증빙 / 정식 인수 미완료**: 비차단 Worker/Qt 일부; 제품 Crash 복구 미완 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **T11**
  - GPU 선택 기능 — **정식 인수 미검증**
- [ ] **T12**
  - 프로젝트·Export — **정식 인수 미검증**
- [ ] **T13**
  - 3 플랫폼과 언어 — **부분 증빙 / 정식 인수 미완료**: 3 Native CI 시험; 실제 배포·언어 기능 미완 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **T14**
  - 오프라인·기밀 보호 — **정식 인수 미검증**
- [ ] **T15**
  - 패키지 일관성 — **정식 인수 미검증**

### Importer/Port T16–T28

- [ ] **T16**
  - ASC·ODB++ 등 동일 보드 Cross-Format — **정식 인수 미검증**
- [ ] **T17**
  - 동일 확장자 다른 형식 — **부분 증빙 / 정식 인수 미완료**: 지원 PADS Header 구분, 전체 .brd/.asc 형식 아님 ([A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md))
- [ ] **T18**
  - Gerber Only Import — **정식 인수 미검증**
- [ ] **T19**
  - Gerber+Drill+IPC-D-356B+Stackup — **정식 인수 미검증**
- [ ] **T20**
  - 미지원/유실 객체 강제 입력 — **부분 증빙 / 정식 인수 미완료**: 제한 비정상 입력 Fail-closed; 전체 객체 분류 미완 ([A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md))
- [ ] **T21**
  - Canonical Model 안정성 — **부분 증빙 / 정식 인수 미완료**: Canonical synthetic 구조검증; Cross-format ID 미완 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **T22**
  - Import 압축/비정상 파일 — **정식 인수 미검증**
- [ ] **T23**
  - 수동 Port Copper 외부 클릭 — **정식 인수 미검증**
- [ ] **T24**
  - GND Plane Cutout/분리 — **정식 인수 미검증**: 물리 Return 경로 검증 미실시 ([B_001 원장](validation/b001-stackup-reference.md))
- [ ] **T25**
  - Pair P/N 역전/Port 번호 변화 — **정식 인수 미검증**
- [ ] **T26**
  - Branch/Series R·C 포함 신호 — **정식 인수 미검증**
- [ ] **T27**
  - Reimport 후 Port 이동/Net 변경 — **정식 인수 미검증**
- [ ] **T28**
  - Near/Far 4-Port 행렬 — **정식 인수 미검증**

### Sweep·복구·플랫폼 T29–T40

- [ ] **T29**
  - 단순 Microstrip·Stripline 1차 Sweep — **정식 인수 미검증**
- [ ] **T30**
  - Group×Region 독립 수렴 — **정식 인수 미검증**
- [ ] **T31**
  - 노치·급변 응답 — **정식 인수 미검증**
- [ ] **T32**
  - Abort/Pause/Resume/Crash Restart — **정식 인수 미검증**
- [ ] **T33**
  - Stackup/Port/Load 변경 후 캐시 — **정식 인수 미검증**
- [ ] **T34**
  - GUI 스트레스(무거운 Import/해석) — **부분 증빙 / 정식 인수 미완료**: Artwork UI/Viewer 스트레스 일부 ([A_034 Native 통합](validation/a034-p2y-integration-stabilization.md))
- [ ] **T35**
  - RAM 부족 및 SSD Spill — **정식 인수 미검증**
- [ ] **T36**
  - Substrate Pad-to-Pad 모델 경계 — **정식 인수 미검증**
- [ ] **T37**
  - Microvia/IVH/Plane Anti-pad — **정식 인수 미검증**
- [ ] **T38**
  - 3D 구조의 모델 미지원 — **정식 인수 미검증**
- [ ] **T39**
  - 세 플랫폼 결과·Import 재현성 — **부분 증빙 / 정식 인수 미완료**: 3 Native subset 실행만; 실 PCB 동일 S 행렬 아님 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **T40**
  - 정식 공개 릴리즈 산출물 — **정식 인수 미검증**

### 고급 인수 T41–T54

- [ ] **T41**
  - 고정 Direct와 Progressive 교차 비교 — **정식 인수 미검증**
- [ ] **T42**
  - NEXT/FEXT와 결합 그룹 — **정식 인수 미검증**
- [ ] **T43**
  - 비수동·비상호·특이행렬 입력 — **정식 인수 미검증**
- [ ] **T44**
  - 불균등 격자·부분 Export·한글 Net — **정식 인수 미검증**
- [ ] **T45**
  - 완료·수렴·모델·Limit 상태 조합 — **정식 인수 미검증**
- [ ] **T46**
  - 늦은 IPC·중복 결과·편집 중 Run — **부분 증빙 / 정식 인수 미완료**: Worker stale reply 격리 일부; 제품 전체 Run 미완 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **T47**
  - 프로젝트 이관·동시 저장·경로 변경 — **정식 인수 미검증**
- [ ] **T48**
  - 세분화·Region 경계·재개 — **정식 인수 미검증**
- [ ] **T49**
  - 진단 출력·캐시·공개 패키지 — **정식 인수 미검증**
- [ ] **T50**
  - UI·Heartbeat·Cancel 시간 계측 — **부분 증빙 / 정식 인수 미완료**: Qt 타이밍 회귀 일부; 원인/출시 목표 미확정 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **T51**
  - 미세 Substrate·ODB Step·극성 — **부분 증빙 / 정식 인수 미완료**: PADS source tick 좌표 확인; ODB Step/미세 Substrate 미완 ([A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md))
- [ ] **T52**
  - 치수·물성 범위와 외삽 — **제한 Rust Canonical Layer z/두께 및 Material 유효 대역 차단 PASS / 전체 인수 미완료**: 물성 모델별 외삽·손실/거칠기·실 PCB 기하/Scenario 재계산은 미검증 ([B_001_2 감사](validation/b001-2-canonical-stackup-audit.md))
    ([B_001 검증](validation/b001-stackup-reference.md))
- [ ] **T53**
  - DC 미지원·깊은 노치·긴 지연 — **정식 인수 미검증**
- [ ] **T54**
  - SSD 부족·쓰기 중 강제 종료 — **정식 인수 미검증**

## 6. 갱신 및 완료 판정 규칙

1. **문서 관리 담당은 챙(ChatGPT)**입니다. 사용자는 일상적으로 문서를 열람만 하고, 체크/날짜/근거 갱신은 챙이 GitHub PR로 수행합니다. `B_001`을 포함한 모든 후속 기능·수정 PR에서 이 문서를 함께 검토하며, 해당 Uxx·Txx·단계·검증 한계·Issue/PR 링크를 **동일 PR**에서 최신화합니다. 변경이 없다면 PR 본문에
  `체크리스트 변경 없음`과 범위·이유를 적습니다.
2. 체크 전 **사양서 요구와 검증 범위가 일치**하는지 확인합니다. 증거: 원본/공개 Fixture·PR·최종 Commit SHA·CI/실행 로그·OS·독립 검토 결과. Green CI/PR Merge만으로 실 PCB Solver/제품을 PASS로 승격하지 않습니다.
3. 판정: `완료`=해당 문장 전체 수용, `부분 증빙`=일부 구현/시험, `정식 인수 미검증`=제품 게이트 부족, `BLOCKED`=실패 또는 명시적 외부 조건 미해결. 제한 완료와 제품 전체 완료를 섞지 않습니다.
4. 물리 Gate Q **historical FAIL**, M0 **BLOCKED**, GPU 측정/라이선스·배포 미완 상태는 별도 근거가 확인되기 전 변경하지 않습니다. CodeQL Alert #1은 **2026-10-31 KST에 GitHub Actions의 `security-events: read`로 실제 `fixed` 상태 확인**
  ([감사 실행](https://github.com/july-rust/BoardSI/actions/runs/41183754657)); 이후 재발 여부는 새 분석 시 별도 검증하며 전체 보안 인수로 일반화하지 않습니다.
5. 체크를 새로 표시할 때는 반드시 실제 KST 날짜의 `**체크일: YYYY-MM-DD KST**`와 근거 링크·근거일을 기록합니다. 이미 체크된 항목을 재검증하더라도 최초 체크일은 유지하며, 정정은 변경 이력에 남깁니다. GitHub 문서 CI가 체크일 누락 및 날짜 형식 오류를 검사합니다.
6. 위 운영 방식은 GitHub 계정의 기술적 접근권한 변경이 아닙니다. 저장소 소유자 권한은 그대로 유지되며, 현재 동일 계정 인증으로 챙의 수정과 소유자의 직접 수정을 GitHub가 기술적으로 구별하지는 못합니다.
7. 각 단계 종료 후 아래 이력에 작업 번호, 변경된 U/T, 최종 `main` SHA, Native 테스트·Codex 리뷰 증빙 및 아직 미검증인 항목을 기록합니다.

8. **문서 CI 최소 실행 정책 (2026-11-03 KST):** 구현과 동시에 변경되는 체크 상태·근거·범위는 **해당 코드 PR에서 함께** 갱신합니다. 코드 PR 뒤에 동일한 사실을 다시 쓰기 위한 별도 문서 PR을 자동 생성하지 않습니다. 과거 누락·표시 오류·새 독립 근거를 고칠 때만 별도의 Markdown-only PR로 묶어 한 번 처리합니다.
9. **문서 전용 검증:** 지원 Markdown/RST/MDX 변경만 있는 경우, Ubuntu의 `docs` 문서 검사로 체크일·집계·요구사항 추적성과 보안 문서 무결성을 검증합니다. GitHub 필수 검사 이름은 보존하되 플랫폼별 Rust/Qt/FEM Job은 **러너 할당 이전에 Skip**합니다. 코드·설정·워크플로·JSON·테스트 Fixture 변경 또는
  분류 불확실 시 전체 필수 시험을 실행합니다.
10. **중복 시험·문서 리뷰 회피:** 단순 체크리스트 표기·날짜·근거 링크 수정으로 Native 빌드, 물리 Solver 시험 또는 릴리스를 재요구하지 않습니다. 코드 변경 시 확보한 플랫폼 결과는 정확한 SHA와 범위를 기록하여 재사용합니다. 별도 경미 문서 PR마다 새로운 독립 코드 리뷰를 자동 생성하지 않되, 보호 Ruleset의 Verified
  서명·Squash·필수 상태 검사·미해결 리뷰 스레드 해결은 유지합니다. 새로운 제품 인수 주장이나 보안 정책 변경은 별도 근거 검토가 필요합니다.

### 변경 이력

| 2026-10-31 | **C_025_08 M0 Rust/Qt/IPC 아키텍처 확정 및 Issue #3 제한 인수 감사 (PR 최종 검증 전)** | [Issue #325](https://github.com/july-rust/BoardSI/issues/325), [Issue #3](https://github.com/july-rust/BoardSI/issues/3), [범위·기술결정 원장](validation/c02508-m0-architecture-issue3-acceptance.md), [M0 종료 게이트](validation/m0-completion-gate.md), [C_025_05](validation/c02505-m0-f1-traceability-reconciliation.md) | **최초 감사 기록일: 2026-10-31 KST · 근거일: 2026-10-31 KST.** Qt Widgets/C++17+독립 Rust Worker/QProcess/Versioned IPC 채택 결정은 현행 구현과 Native 합성 검증으로 추적. Qt Quick/RHI는 선택적 Artwork 기능, 3 Native는 프로토타입에 한정. T13/T39/T46 전체 인수 NOT_RUN, F1/F6/F7 및 Issue #3 OPEN 유지. **기능 또는 U/T 완료 체크 [x] 추가 없음**. 본 PR의 독립 Codex/정확 HEAD CI/보호 Squash/post-main 및 #325 감사 종료는 객관적 근거가 확보되기 전 PENDING. 기존 체크일·PH08 원본 FAIL 및 PR #35 상태 보존. |

| 2026-10-31 | **C_025_07 전체 Open Issue·PR 최종 감사 및 D-stage 인계 (PR 최종 검증 전)** | [Issue #323](https://github.com/july-rust/BoardSI/issues/323), [27개 전수 감사](validation/c02507-open-issue-d-stage-handoff.md), [C_024](validation/c024-c-series-final-acceptance.md), [C_025_05](validation/c02505-m0-f1-traceability-reconciliation.md), [C_025_06](validation/c02506-pre-ph08-audit-issue78-closeout.md) | **최초 감사 기록일: 2026-10-31 KST · 근거일: 2026-10-31 KST.** 감사 전 기존 Open 27개 및 Draft PR #35 확인; 모든 27개가 개별 DoD 미충족으로 종료 대상 **0**. 본 감사 Issue #323 추가 후 일시 Open 28개. **신규 Uxx/Txx 또는 완료 체크 [x] 없음**. M0 F1/F6/F7 BLOCKED, #12/#16/#41/#46 OPEN, P009/물리 SI/Cache 미검증 유지. PR 정확 HEAD CI·독립 Codex·Signed Verified Squash·post-main 검증 및 #323 최종 종료는 별도 증거 확인 전까지 **PENDING**; 인계 D_001은 계획이지 구현 아님. |

| 2026-10-31 | C_025_06 pre-PH08 Issue #78 감사·현행 이슈 상태 정합성 보강 | [감사](validation/c02506-pre-ph08-audit-issue78-closeout.md), [기존 보호 병합 #81](https://github.com/july-rust/BoardSI/pull/81), [U/T #93](https://github.com/july-rust/BoardSI/pull/93), [Issue #78](https://github.com/july-rust/BoardSI/issues/78), [Issue #51](https://github.com/july-rust/BoardSI/issues/51) | **최초 체크일: 2026-10-31 KST · 근거일: 2026-10-31 KST.** 과거 감사 문서·승인 SVG·정합성 CI 근거를 대조하고, #51 행정 CLOSED와 PH-08 원본 Gate Q FAIL을 구별. **제품 U/T·물리 Solver·ICO/ICNS·배포 자격 검증은 체크하지 않음.** 본 C_025_06 PR 정확 HEAD CI/독립 리뷰/보호된 병합/post-main 및 #78 종료는 별도 결과 확인 후에만 인정. |

| 2026-10-31 | C_025_05 M0 F1 사양서/요구사항 추적성 및 Issue #2 감사 착수·제한 검증 | [감사](validation/c02505-m0-f1-traceability-reconciliation.md), [사양서 v1.0](specification/BoardSI_Development_Spec_v1.0.md), [기존 M0 10-case](validation/m0-case-evidence.json), [Issue #2](https://github.com/july-rust/BoardSI/issues/2) | **최초 체크일: 2026-10-31 KST · 근거일: 2026-10-31 KST.** ID 정의/매핑 확인만 [x]. 정식 M0 F1 및 Issue #2 OPEN/NOT_RUN 유지. 현행 PR 리뷰·CI·main 병합 후 검증은 별도. |


| 2026-10-31 | C_025_03 감사 문서 PR #317 정식 인수 | [PR #317](https://github.com/july-rust/BoardSI/pull/317), [최종 HEAD 22/22](https://github.com/july-rust/BoardSI/commit/2adf25fd6da8cb84e1949cd6eb6035a9f2f5531d/checks), [Signed Verified main@c2c4335](https://github.com/july-rust/BoardSI/commit/c2c4335bec4b7835e19b1fc9bb9c752786a92f20), [post-main 21/21 checks](https://api.github.com/repos/july-rust/BoardSI/commits/c2c4335bec4b7835e19b1fc9bb9c752786a92f20/check-runs?per_page=100), [push 8/8](https://api.github.com/repos/july-rust/BoardSI/actions/runs?event=push&head_sha=c2c4335bec4b7835e19b1fc9bb9c752786a92f20&per_page=100) | **최초 체크일: 2026-10-31 KST · 근거일: 2026-10-31 KST.** 17 필수 strict 보호 검사 준수, 22/22 PR SUCCESS, Codex latest HEAD 0 findings, Squash Verified single parent, post-main 21/21 and push 8/8 SUCCESS. **문서 감사 통합 완료만 [x]**. 원본 #118 단독 GUI selftest·제품 U/T·P009·물리 SI는 미인수. |


| 2026-10-31 | C_025_03 구형 PADS Import PR 3건 안전 정리 | [PR #115](https://github.com/july-rust/BoardSI/pull/115), [#117](https://github.com/july-rust/BoardSI/pull/117), [#118](https://github.com/july-rust/BoardSI/pull/118), [원장](validation/c025-03-legacy-pads-pr-reconciliation.md), [Issue #8](https://github.com/july-rust/BoardSI/issues/8) | **최초 체크일: 2026-10-31 KST · 근거일: 2026-10-31 KST.** PR 3건 모두 `closed/unmerged` 확인 및 기각 이유 기록. 과거 Signal Probe 핵심은 main에 보존, #118 단독 shell은 동등성 미인수. 감사 문서 PR 자체 CI/보호 병합 별도 검증 전 NOT_VERIFIED. U/T·물리 Solver·P009 승격 없음. |


| 2026-10-31 | C_025_02 SEC_001 실제 Code Scanning Fixed 검증 | [기존 수정 PR #206](https://github.com/july-rust/BoardSI/pull/206), [실제 GitHub API 감사 #41183754657](https://github.com/july-rust/BoardSI/actions/runs/41183754657), [Issue #205](https://github.com/july-rust/BoardSI/issues/205), [main@477428da 21/21 Checks](https://github.com/july-rust/BoardSI/commit/477428da3895ee4a2ff19dda5d53932754e3a6ba/checks) | **최초 체크일: 2026-10-31 KST · 근거일: 2026-10-31 KST.** Alert #1 실제 `fixed` (2026-10-26T08:25:02Z), 동일 규칙 Open 0. 라이선스·배포, M0, PH-08 및 전체 U/T 등 범위 변경 없음. |

| 2026-10-31 | C_022 V14 3~6 Source Rectangle Copper Component/Island 위험 한정 인수 | [Issue #304](https://github.com/july-rust/BoardSI/issues/304), [Code PR #305](https://github.com/july-rust/BoardSI/pull/305), [Verified `main@242293dd`](https://github.com/july-rust/BoardSI/commit/242293dda118024697b6d1cb3bde5a521e6fa2cf), [27/27 exact PR HEAD](https://github.com/july-rust/BoardSI/commit/e077eeaa8f31766ae94757bb90a90dcdd129cb34/checks), [21/21 post-main](https://api.github.com/repos/july-rust/BoardSI/commits/242293dda118024697b6d1cb3bde5a521e6fa2cf/check-runs?per_page=100), [원장](validation/c022-reference-multi-area-connectivity.md) | **최초 체크일: 2026-10-31 KST · 근거일: 2026-10-31 KST.** 11/11 PR workflows, independent exact HEAD Codex findings=0/unresolved=0, protected Verified Squash 1 parent, 8/8 push workflows; 제한 Source Geometry 후보 그룹/Island 위험 [x]. 문서 PR 자체는 별도 CI·리뷰·병합·post-main 검증 필요. Parent M1 #12 OPEN, P009/전기적 접촉/실물 Fill/DRC/RLGC/S Solver/Cache, U/T 인수 미완. |



| 2026-10-29 | C_009 Reference Stitching Via Source Contact·Qt Native 한정 인수 | [Issue #261](https://github.com/july-rust/BoardSI/issues/261), [코드 PR #262](https://github.com/july-rust/BoardSI/pull/262), [정확한 코드 HEAD `bdd29c0` 27/27](https://github.com/july-rust/BoardSI/commit/bdd29c061086b5d68742b3915dddd9e2bcf83033/checks), [Signed Verified `main@a246bc9` post-main 21/21](https://github.com/july-rust/BoardSI/commit/a246bc9f15a3b27cf33cdd65370308ee6051fece/checks), [Qt Native Push](https://github.com/july-rust/BoardSI/actions/runs/40814575323), [원장](validation/c009-reference-stitching-via-candidate.md) | **최초 체크일: 2026-10-29 KST · 근거일: 2026-10-29 KST.** PR Workflow 11/11, 동일 HEAD Codex 지적/미해결 0, 리뷰 스레드 3/3 해결, 독립 Main Push 8/8 워크플로 성공. **단일 Reference Via 후보/Source Geometry·Qt Risk만 [x]**, P009/실전 Return Path/물리 SI·Solver/Cache/Uxx·Txx 미완, M1 #12 OPEN. 문서 PR 자체 검증은 별도. |


| 2026-10-29 | C_007 다중 Straight Route Source Reference Gap / Qt Native 제한 인수 | [Issue #255](https://github.com/july-rust/BoardSI/issues/255), [PR #256](https://github.com/july-rust/BoardSI/pull/256), [정확한 코드 HEAD 27/27](https://github.com/july-rust/BoardSI/commit/fdfad35931805e47d6658aeed7966ed3d778b741/checks), [Signed Verified main@08849e3f, post-main 21/21 Checks](https://github.com/july-rust/BoardSI/commit/08849e3fdf048f60a8a06265792acd4f51316b2f/checks), [Push on main #40781834140](https://github.com/july-rust/BoardSI/actions/runs/40781834140), [Qt runtime #40781834617](https://github.com/july-rust/BoardSI/actions/runs/40781834617) | **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST**. 독립 Codex 최신 HEAD 지적 0·미해결 0, Windows x64/ARM64·macOS ARM64 실프로세스와 Linux Qt PASS, post-code-main 9/9 Workflow SUCCESS. **Source Geometry/Qt 한정**, P009/Return Current/물리 Solver 및 정식 U/T 불승인, M1 #12 OPEN. |

| 2026-10-28 | C_006 Source Reference gap intervals / Qt Native 제한 인수 | [Issue #252](https://github.com/july-rust/BoardSI/issues/252), [코드 PR #253](https://github.com/july-rust/BoardSI/pull/253), [exact HEAD 27/27 checks](https://github.com/july-rust/BoardSI/commit/ed5e137bbb8b1bebcfde832961038d43286a7753/checks), [Signed Verified main 5e4de47 + 21/21 post-main checks](https://github.com/july-rust/BoardSI/commit/5e4de472a90539221f69e5cf11b88e20d9fd7361/checks), [main Push](https://github.com/july-rust/BoardSI/actions/runs/40697041065) | **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**. C_006 한정 Source-geometry interval 및 CPU/RHI·Native 3 OS + Linux Qt PASS. 동일 HEAD Codex 0 findings/0 unresolved; post-main 9/9 Workflow SUCCESS. M1 #12와 정식 U/T 및 P009·물리 Solver 미완. |

| 2026-10-27 | B_005 post-main Push CI 완료 체크 | [Verified main@`7a1ed48`](https://github.com/july-rust/BoardSI/commit/7a1ed48bf905df8357691a1f51b73e130a74537c/checks) 21/21 SUCCESS, [Push on main #40501517050](https://github.com/july-rust/BoardSI/actions/runs/40501517050) SUCCESS | **체크일: 2026-10-27 KST**, **근거일: 2026-10-27 KST**. B_005 제한 기능 병합 후 CI 최종 확인, Issue #12 OPEN과 P009/물리 SI 공식 U/T 미완 유지. |

| 2026-10-27 | B_005 제한 Source-bound Reference 기하 감사 최초 완료 체크 | [PR #227](https://github.com/july-rust/BoardSI/pull/227), [Verified Squash `7a1ed48`](https://github.com/july-rust/BoardSI/commit/7a1ed48bf905df8357691a1f51b73e130a74537c), [PR HEAD `284f5f04` 23/23 CI](https://github.com/july-rust/BoardSI/commit/284f5f04fe3e1d5a07826c0c158893a6c685ea87/checks), Codex 동일 HEAD 주요 지적 0, Rust Debug/Release 17/17 | 최초 체크일 **2026-10-27 KST**, 근거일 **2026-10-27 KST**. `SOURCE_ORDER_GEOMETRY_ONLY` 제한. Post-main CI 별도 확인 필요; B_001 Issue #12 OPEN, 물리 P009 및 U/T 항목은 미완. |

| 2026-10-27 | B_005 Source-bound Reference 기하 감사 코드 HEAD 검증(정식 체크 보류) | [PR #227](https://github.com/july-rust/BoardSI/pull/227), [`225489bedabc146e31b83137267aaa29d86f8503` 23/23 checks](https://github.com/july-rust/BoardSI/commit/225489bedabc146e31b83137267aaa29d86f8503/checks), Rust Debug/Release B_005 17/17 PASS | 원본 Net/Layer/Trace/Arc/Area 재검증, Return Path 기하 커버리지 한정 코드 PASS. 동일 코드 최신 Codex 후속 리뷰·보호 병합·main CI 확인 전에는 신규 [x] 및 최초 체크일 없음; B_001 Issue #12 OPEN/공식 U/T 미검증 유지. |

| 2026-10-27 | B_004 제한 Copper Area 국소 접점 코드 인수 | [PR #226](https://github.com/july-rust/BoardSI/pull/226), [정확한 코드 HEAD `7c68f9523b65999271fff5b510e984ec30dac72a` CI](https://github.com/july-rust/BoardSI/commit/7c68f9523b65999271fff5b510e984ec30dac72a/checks), [검증 원장](validation/b004-reference-copper-connectivity.md) | 공개 합성 Straight COPCLS 내부 Trace/Pad Centre 및 동일 소스 Area의 제한적 Graph `[x]` 최초 체크일·근거일 **2026-10-27 KST**. 12 workflow/23 job 모두 성공, 동일 HEAD Codex findings 0·기존 4건 해결. 보호 병합·Post-main SHA는 PR 완료 후 별도 확인; B_001 Issue #12 OPEN, 정식 U/T·P009·물리 Solver 체크 없음. |


| 날짜(KST) | 작업 | 기준/증거 | 변경 사항 |
| --- | --- | --- | --- |
| 2026-10-26 | A_036 | 사양서 v1.0, [PR #210](https://github.com/july-rust/BoardSI/pull/210), `main@d23f2d4001130cf654e3071cfeb01a7df473ac1d` | U01–U48 / T01–T54 목록 및 최초 체크 13개 생성. 병합 후 21/21 CI PASS |
| 2026-10-26 | A_036 후속 날짜 보강 | [Issue #211](https://github.com/july-rust/BoardSI/issues/211) | 기존 체크 13개에 최초 체크일 및 개별 선행 근거일 기록, 향후 체크 날짜 필수화, 챙 관리·사용자 열람 운영 원칙 추가 |
| 2026-10-26 | B_001 착수 (PR 검증 전) | [Issue #12](https://github.com/july-rust/BoardSI/issues/12), [검증 원장](validation/b001-stackup-reference.md) | Qt 사용자 Stackup Overlay 후보 코드. U09/U10/T02/T52 부분 증빙 후보; 정식 체크박스 변경 없음. Native CI/독립 리뷰/실제 물리 준비도 미확인 |
| 2026-10-27 | B_001_3 보호된 main 병합·검증 기록 | [병합 PR #217](https://github.com/july-rust/BoardSI/pull/217), 유효 서명 **`main@7448edfe68f5bf50af4d1f3e580f84fca98af3d2`** ([commit](https://github.com/july-rust/BoardSI/commit/7448edfe68f5bf50af4d1f3e580f84fca98af3d2)), [최종 PR HEAD `ac02dfe57f885d2d33b496bdc93ef061f99935f6` 27/27 PASS](https://github.com/july-rust/BoardSI/commit/ac02dfe57f885d2d33b496bdc93ef061f99935f6/checks), Codex 독립 검토 0 findings·리뷰 스레드 4건 해결 | 제한 원본 Copper/Net Reference **음성 판정만** 최초 체크일 **2026-10-27 KST 유지** (시험 근거 2026-10-27 KST); 전기적 연결, Return Path P009, 완전한 B_001 및 정식 U/T 인수 미완. 병합 후 main CI는 별도 판정 필요 |
| 2026-10-27 | B_001_3 제한 Import Reference 후보 안전감사 체크 추가 | [PR #217](https://github.com/july-rust/BoardSI/pull/217), [code SHA `d15b86db4cc044da07930fbc9baad41ea0bc891a` 27/27 CI](https://github.com/july-rust/BoardSI/commit/d15b86db4cc044da07930fbc9baad41ea0bc891a/checks), [Qt Native+Linux](https://github.com/july-rust/BoardSI/actions/runs/40428481598), [검증 원장](validation/b001-3-source-reference-inventory.md) | B_001_3 좁은 음성/Unsupported 검증만 **최초 체크일 2026-10-27 KST**, 시험일 2026-10-27 KST로 체크. 물리 접속, P009, 전원 Net 및 U09/U10/U39/T02/T04/T24/T52 전체 공식 인수는 미완; 문서 새 HEAD 보호 검사 후 병합 필요 |
| 2026-10-27 | B_001_3 PADS Copper/Net Source Inventory 착수 | [PR #217](https://github.com/july-rust/BoardSI/pull/217), [검증 원장](validation/b001-3-source-reference-inventory.md) | 실제 성공 Import의 Layer·Net·Area ID·Hole 목록을 Rust 후보 감사에 전달. PASS 전까지 신규 완료 체크 없음. Reference 전기적 접속·Return Path P009·정식 T04/T24는 미검증 |
| 2026-10-27 | B_001_2 보호된 main 병합 증빙 확정 | [병합 PR #215](https://github.com/july-rust/BoardSI/pull/215), 서명된 `main@5f795b7f4609304827d4d6ca6acb4ff7ecf73cc8`, [정확한 PR HEAD 28/28 CI](https://github.com/july-rust/BoardSI/commit/6ce79d685a26cd3b26f1cd9b9ca065850aea5ca5/checks), [Qt Native 3+Linux](https://github.com/july-rust/BoardSI/actions/runs/40420950767), 독립 Codex same-HEAD 0 findings | 제한 Stackup Material `Assumed` 변환의 최초 체크일 **2026-10-27 KST 유지** (근거일 2026-10-27 KST); 별도 물리 Reference/Net/Copper 미자격 및 Issue #12 OPEN. 최종 main merge SHA 보존; 정식 U/T 체크박스 미변경 |
| 2026-10-27 | B_001_2 제한 Rust Canonical 감사 체크 추가 | [PR #215](https://github.com/july-rust/BoardSI/pull/215), [Native Qt 검사](https://github.com/july-rust/BoardSI/actions/runs/40419571718), [Rust fmt/Clippy/모델 검사](https://github.com/july-rust/BoardSI/actions/runs/40419571744), [원장](validation/b001-2-canonical-stackup-audit.md) | **검증된 좁은 B_001_2 항목만 최초 체크일 2026-10-27 KST / 근거일 2026-10-27 KST로 [x] 추가**, 물리 Reference·실 Port·Solver/정식 인수 미완. 최종 HEAD 필수 CI 및 Codex 재검증 필요 |
| 2026-10-27 | B_001_2 추가 착수 / 검증 전 | [PR #215](https://github.com/july-rust/BoardSI/pull/215), [Rust Canonical 검증 원장](validation/b001-2-canonical-stackup-audit.md) | UI 출처·물성 대역 필드를 Rust Canonical Layer/Material로 연결하는 후보 코드. 이번 PR의 필수 CI/독립 리뷰 증빙 전이라 [x] 추가 없음. T04/T24/실제 PADS Copper Net/Return Path 계속 미완 |
| 2026-10-27 | B_001 제한 UI 증빙 확정 / #12 OPEN | [병합 PR #213](https://github.com/july-rust/BoardSI/pull/213), **`main@ed9fc66e5b0ab0072fcb93103adc18c2729ae0e3`** ([서명된 squash merge](https://github.com/july-rust/BoardSI/commit/ed9fc66e5b0ab0072fcb93103adc18c2729ae0e3)), 사전 병합 `head@a789ff22ad025c83ec6357a81ea2bbe9b3069b32` (26/26 검사 PASS, 17/17 필수, Codex same-head findings 0), [Qt Native 3+Linux](validation/b001-stackup-reference.md) | 제한 Qt Overlay [x] 1개만 추가 (최초 체크일 2026-10-27 KST / 시험 근거일 2026-10-26 KST); U09/U10/U39 및 T02/T04/T24/T52 전체 인수 체크는 미완. 병합된 main 신규 CI는 별도 게이트로 확인 |

| 2026-10-29 | C_008 두 Signal Layer 단일 도금 Via Source Reference 기하·Qt Risk 제한 인수 | [Issue #258](https://github.com/july-rust/BoardSI/issues/258), [PR #259](https://github.com/july-rust/BoardSI/pull/259), [정확한 코드 HEAD `ee765510` 27/27 CI SUCCESS](https://github.com/july-rust/BoardSI/commit/ee765510b1b6f761c31adea8990eb5ef42405f18/checks), [Signed Verified `main@5db1840a` post-main 21/21](https://github.com/july-rust/BoardSI/commit/5db1840aa267624d20693aef4e48186383e04a57/checks), [Qt Native](https://github.com/july-rust/BoardSI/actions/runs/40791493610), [FEM Native](https://github.com/july-rust/BoardSI/actions/runs/40791493531), [원장](validation/c008-multilayer-reference-via-transition.md) | **최초 체크일: 2026-10-29 KST · 근거일: 2026-10-29 KST**. Codex 동일 HEAD 지적/미해결 0, post-main 9/9 Workflow SUCCESS. **PADS 합성 Source Geometry 및 Qt 표시만 [x]**; 실제 Return Current/P009·물리 SI·Solver/Cache 및 정식 U/T 미승인. 상위 M1 #12 OPEN. |

<a id="c_025_09"></a>
### C_025_09 — macOS Qt 글로벌 타이머 지연 분리 진단 및 #46/#41 종료 안전성 (2026-10-31 KST)

- [x] **C_025_09 기존 Native 실패/성공 근거 확인 및 진단 방향 확정 (현행 로그 읽기 한정)** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST**
  - [Issue #327](https://github.com/july-rust/BoardSI/issues/327), [Issue #46](https://github.com/july-rust/BoardSI/issues/46),
    [Qt native PR #326 실행 41204123648](https://github.com/july-rust/BoardSI/actions/runs/41204123648), [계측 원장](validation/c02509-qt-global-timer-stabilization.md).
  - . macOS ARM64 최초 zero timer 205 ms, 최초 precise timer 554 ms, 전체 최대 554 ms/PASS(현행 M0 1500 ms), normal UI p95 0.161375 ms; 과거 1134/1331 ms는 당시 1000 ms 기준 실제 FAIL. 이 [x]는 **기존 증빙 조회만** 의미하며 새 진단 실행/원인
    수정/정식 M0·제품 인수 아님.
- [x] **C_025_09 Qt dispatcher 계측 코드 최종 인수 및 보호된 main 병합 (진단만)** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST**
  - [PR #328](https://github.com/july-rust/BoardSI/pull/328), [최종 HEAD 26/26 Check Runs](https://github.com/july-rust/BoardSI/commit/26f93d8eb7f6afe95b1f7163b171048960ba9ba7/checks),
    [Verified signed Squash main@903b61cf](https://github.com/july-rust/BoardSI/commit/903b61cf9850c2c50a7a32658686d4db2451db49),
    [post-main 21/21 Check Runs](https://github.com/july-rust/BoardSI/commit/903b61cf9850c2c50a7a32658686d4db2451db49/checks),
    [post-main Qt Native](https://github.com/july-rust/BoardSI/actions/runs/41207619823), [검증 원장](validation/c02509-qt-global-timer-stabilization.md).
  - . 17 strict 필수 context 전부 성공, 최신 exact-HEAD Codex findings 0/unresolved 0, review threads 0, signed Verified one-parent Squash, 독립 post-main 8/8 push workflows SUCCESS. Win x64/ARM64·macOS ARM64·Linux
    Qt 진단 실행 PASS, 기존 M0 1500 ms gate 유지. **이 [x]는 새 진단 코드 통합만 의미하며 #46 근본 원인, #41 전체 종료, 실기기 배포, 물리 SI 및 U/T PASS가 아님.**
- [ ] **Issue #46 코드 원인 및 근본 해결 입증**
  - macOS 실기기/부하 대표 조건의 재현성 있는 수정 전후 회귀 결과 미확보. 단발 CI PASS/1500 ms M0 예외/디스패처 계측을 원인 해결로 과대 해석하지 않음; Issue #46 OPEN.
- [ ] **Issue #41 종료**
  - #46 및 자체 감사 완료 조건 독립 재평가 전 OPEN; Issue #43 Ruleset 적용은 이미 확인됨. Uxx/Txx, PH-08 물리 승인, #35 Draft, M0 F1/F6/F7 변경 금지.

<a id="c_025_10"></a>
### C_025_10 — M0 F6 공개 Fixture 증거 및 Issue #7 안전 판정 (2026-10-31 KST)

- [x] **두 공개 KiCad 후보의 원본 바이트/SHA-256 및 Git Blob SHA-1 검증 근거 재조정 (기존 실행만)** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-21 KST**
  - [Issue #7의 2026-10-21 실행 기록](https://github.com/july-rust/BoardSI/issues/7), [원본 해시 CI job](https://github.com/july-rust/BoardSI/actions/runs/39501592390/job/118900157258),
    [현재 고정 검사 코드](../.github/scripts/m0_f6_board_integrity.py), [C_025_10 원장](validation/c02510-m0-f6-public-fixture-reconciliation.md).
  - . 정확한 크기/파일 SHA-256/Git Blob SHA-1을 서로 구별; 이번 C_025_10에서 과거 원본 job 로그 직접 재열람은 API 404이므로 **새 Native 실행 PASS를 주장하지 않음**.
- [x] **두 후보의 제한적 공식 KiCad 외곽/객체 수 Native 증거 재조정 (기존 실행만)** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-21 KST**
  - [KiCad 9.0.9 job](https://github.com/july-rust/BoardSI/actions/runs/39501592463/job/118900157449), [Native 검사 코드](../.github/scripts/m0_f6_native_kicad_outline.py),
    [원장](validation/c02510-m0-f6-public-fixture-reconciliation.md).
  - . 닫힌 외곽/크기/객체 수의 기존 제한 기술 증거만 [x]; 소형·중형의 정식 수락 클래스, BoardSI CAD Import, Reference 전기 접속, 물리 SI는 PASS 아님.
- [x] **공개 소스 라이선스 계열 제목 검사 계약 존재 확인** · **체크일: 2026-10-31 KST** · **근거일: 2026-10-21 KST**
  - [해시/라이선스 스크립트](../.github/scripts/m0_f6_board_integrity.py), [C_025_10 원장](validation/c02510-m0-f6-public-fixture-reconciliation.md). . 출처에 표시된 CERN-OHL-2-P / Apache-2.0 계열 제목 검사만 의미; **해당 보드의 전체
    저작권/제3자 고지 및 재배포 적법성 승인은 아님**.
- [ ] **Issue #7 F6 공개 Fixture 배포 법적 승인**
  - 저작권자별 3rd-party asset·NOTICE·라이선스 의무, 실제 Bundle 배포 방식의 별도 승인이 없으므로 **LEGAL_REVIEW_REQUIRED**. 원본 보드는 BoardSI 저장소에 동봉하지 않음.
- [ ] **Issue #7 F6 실제 약 8 GiB 기준 PC 인수**
  - 기존 Linux 8 GiB/2 CPU 제한 합성 컨테이너와 별도로 실제 기준 환경의 OS/CPU/실가용 RAM/SSD 및 UI·디스크 시험 누락. 유료 클라우드/하드웨어는 승인 없이 사용하지 않음.
- [ ] **Issue #7 전체 DoD 및 정식 T14/T15/T49**
  - 외부 승인/실기기 측정 미완: **Issue #7 OPEN / M0 F6 BLOCKED**. Issue #6 F7 라이선스·배포와 Issue #46/#41은 각각 별개로 OPEN.
- [x] **C_025_10 보호 브랜치 문서 감사 최종 제한 인수** · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST**
  - [PR #331](https://github.com/july-rust/BoardSI/pull/331) 최종 HEAD `6037287621d1712010ce1b44a7b1a4911d4e3c07` 22/22 Checks SUCCESS 및 8/8 PR Workflows SUCCESS; 같은 HEAD Codex
    `completed/findings=false/unresolved=false`, 리뷰 Thread 0. [Verified 1-parent Squash main@95e6af7](https://github.com/july-rust/BoardSI/commit/95e6af79144e3c1da95d870c609daa7be4049110) (GitHub
    `verified=true/reason=valid`), 병합 후 21/21 Checks SUCCESS 및 8/8 Push Workflows SUCCESS 독립 확인. [추적 Issue #330](https://github.com/july-rust/BoardSI/issues/330) CLOSED/completed (2026-11-01 KST).
  - (PR·병합·CI는 2026-10-31 KST, 독립 재검증·추적 Issue 종료는 2026-11-01 KST). 기존 기술 Fixture/License 헤더 재조정 문서만 인수; 외부 재배포 법적 승인, 실기기 기준 PC, M0 F6 정식 T14/T15/T49 미확인 및 Issue #7 OPEN/BLOCKED 유지.

<a id="d_002"></a>
### D_002 — Source COPCUT-aware two-area hypothesis (started 2026-11-01 KST)

- [x] **D_002 제한 Source COPCUT-aware 두 Copper Area 접촉 후보 Rust/Qt 코드 및 Native 통합 인수** · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST**
  - [PR #337](https://github.com/july-rust/BoardSI/pull/337) 최종 HEAD `2ee4735bd7323e5971a542dd198e34c06f20b924`: **27/27 Check Runs SUCCESS**, **11/11 PR Workflows SUCCESS** (필수 17개 포함). Rust 1.75
    fmt/strict Clippy 및 Debug/Release 회귀시험; Windows x64·Windows ARM64·macOS ARM64 D_002 Native Qt 전용 테스트 각각 SUCCESS, Linux Qt Offscreen SUCCESS. 동일 HEAD 독립 Codex
    `completed/findings=false/unresolved=false`, 리뷰 스레드 0건.
    [GitHub Verified 단일 부모 Squash `main@c70d2309cb4aee1168762b51d7c76eb9d7ca42e4`](https://github.com/july-rust/BoardSI/commit/c70d2309cb4aee1168762b51d7c76eb9d7ca42e4),
    `verification.verified=true/reason=valid`, **정확 병합 SHA post-main 21/21 Checks SUCCESS 및 8/8 Push Workflows SUCCESS** 독립 검증. 두 개의 명시 Source COPCLS 직사각형과 최대 한 개의 동일 소유자 직사각형 COPCUT을 제외한 Source-grid 양의 잔여
    면적 겹침, Canonical/Qt Source SHA 검증 및 재Import Stale 무효화에 한정. [D_002 검증 원장](validation/d002-copper-split-contact-qualification.md), [추적 Issue #336](https://github.com/july-rust/BoardSI/issues/336). **최초
    기록일: 2026-11-01 KST.** 실제 제조 Copper 연결, 전기적 접속/P009, SI Solver·Cache, U09/U10/U39/T02/T04/T24/T52는 여전히 **NOT_VERIFIED/UNSUPPORTED/NOT_RUN**, M1 #12 OPEN; Draft PR #35 변경 없음.

<a id="d_001"></a>
### D_001 — Trace endpoint / conductive SMD Pad footprint Source witness (2026-11-01 KST)

- [x] **D_001 제한 Source-owned SMD Pad/Trace endpoint 후보 코드 및 Native 통합 인수** · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST**
  - [PR #333](https://github.com/july-rust/BoardSI/pull/333) 최종 HEAD `c6136c18fbf6b64af77d8a0426c8d3f9fc06e02e`: **27/27 Check Runs SUCCESS**, **11/11 PR Workflows SUCCESS**, Windows x64/ARM64·macOS ARM64
    Qt Native 및 Linux Offscreen/Rust. 동일 HEAD 독립 Codex `completed/findings=false/unresolved=false`, 리뷰 스레드 전부 해결.
    [Verified 단일 부모 Squash main@8314e52](https://github.com/july-rust/BoardSI/commit/8314e52cabc76b070e6b793ac962120cca2cd093), GitHub `verification.verified=true/reason=valid`, **정확 병합 SHA post-main 21/21
    Checks SUCCESS, 8/8 Push Workflows SUCCESS** 독립 확인. Source Pad 원형·정사각형·제한 RF 직사각형 내부 Trace 끝점과 명시 Net/Layer Source 기하 후보, Pad 수량 절단시 fail closed, 소스 SHA/Qt readback·재입력 Stale 무효화에 한정.
    [D_001 검증 원장](validation/d001-source-pad-endpoint.md), [추적 Issue #332](https://github.com/july-rust/BoardSI/issues/332).
  - . **M1 전기적 접속, COPCLS/COPCUT 제조 연속성, Return Path P009, SI Solver, 실제 제품 GUI 기능 정식 인수는 모두 미완**. 추가 S/RF 형상별 긍정/경계 회귀시험은 원 PR #333의 시험으로 소급하지 않고 보강 PR #334의 별도 최신 HEAD·CI 근거로 인수한다. D_001 추적 Issue #332
    종료는 별도 최종 문서 PR의 보호 병합 및 병합 후 CI 검증까지 확인.
- [x] **D_001 S/RF Pad 회전 형상·중복 소유자 보강 회귀시험과 보호 통합 인수** · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST**
  - [PR #334](https://github.com/july-rust/BoardSI/pull/334) 최종 HEAD `45564c928ab27711541f96803bc3ad96856c931b`: **23/23 Check Runs SUCCESS**, **9/9 pull_request Workflows SUCCESS**, Windows
    x64/ARM64·macOS ARM64 실제 D_001 Native Qt 진단 및 Rust Numerical Core 포함. 동일 정확 HEAD 독립 Codex `status=completed/has_any_findings=false/has_unresolved_findings=false`, 미해결 리뷰 스레드 0.
    [Verified one-parent Squash main@75a4a62bec4ab7dc6db1993edb0cca91b66732d2](https://github.com/july-rust/BoardSI/commit/75a4a62bec4ab7dc6db1993edb0cca91b66732d2),
    `verification.verified=true/reason=valid`, 단일 부모 `8314e52cabc76b070e6b793ac962120cca2cd093`. **main 정확 SHA 21/21 Check Runs SUCCESS, 8/8 Push Workflows SUCCESS**를 별도 재검증. 원형 Pad 기존 #333 회귀시험과 달리 **정사각형
    SMD·90도 회전 RF 직사각형 내부/접선·중복 Pad 소유자** 보강시험은 #334에서 수행. [검증 원장](validation/d001-source-pad-endpoint.md), [Issue #332](https://github.com/july-rust/BoardSI/issues/332).
  - . 이 체크는 Source 기하 후보만 인수하며 제조 전기 접속, M1 #12, P009, SI Solver/Cache, full U/T 불인수. 이 증빙 문서 자체의 별도 CI/리뷰/병합 결과는 추적 Issue에 남기고 종료 판정.
- [ ] **M1 전체 및 물리 전기 접속, P009 인수**
  - D_001은 제조 Copper/Fill/Thermal/COPCUT/Return Path를 보증하지 않는 Source 기하 후보. U09/U10/U39 및 T02/T04/T24/T52는 미완, [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. Solver/cache 미승인 유지.

### 필수 참고 근거

- [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) · [A_034 Native 통합](validation/a034-p2y-integration-stabilization.md) · [A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md) ·
  [A_032 국소 접점](validation/a032-p2w-local-connectivity.md)
- [M0 종료 게이트](validation/m0-completion-gate.md) · [M0 추적 원장](audits/m0-requirements-traceability-ledger.md) · [M2 검증 원장](validation/README.md)
- [원본 PH-08 FAIL](validation/m2-g2-ph08-stage5b-original-replay-evidence.md) · [SEC_001 코드 검사](validation/sec001-codeql-markdown-comment-scan.md) ·
  [Code scanning alert #1](https://github.com/july-rust/BoardSI/security/code-scanning/1)

<a id="d_004"></a>
### D_004 — Source Trace Endpoint / Three-Area Candidate Attachment (pending, 2026-11-01 KST)

- [x] **D_004 정확히 3개 Source COPCLS/최대 1개 소유 COPCUT와 Straight Trace 끝점 2개의 Source 기하 접촉 후보 연결 및 Qt 독립 재검증 한정 인수** · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST**
  - . [Issue #342](https://github.com/july-rust/BoardSI/issues/342), [코드 PR #343](https://github.com/july-rust/BoardSI/pull/343), 최종 HEAD `749bd4cb5db935961a115a3d56282b453707096f` **27/27 Check Runs
  - 11/11 PR Workflows SUCCESS**, Ruleset 25046595 필수 **17/17**, 동일 HEAD Codex `completed` findings 0/unresolved 0·기존 P2 스레드 해결, Rust Debug/Release 및 Windows x64/ARM64·macOS ARM64 Native Qt, Linux Qt
    Offscreen D_004 CTest PASS, [GitHub Signed Verified one-parent Squash `main@3bde5750`](https://github.com/july-rust/BoardSI/commit/3bde575048d9abb4c6a8cd9a5ccc8df50f857e46), 독립 **post-main 21/21 Checks
  - 8/8 Push Workflows SUCCESS**, [검증 원장](validation/d004-trace-endpoint-area-attachment.md). **Source 기하 후보만 승인**, 실제 제조 Copper/전기 접속, P009, 물리 SI, Solver/Cache, 정식 U09/U10/U39·T02/T04/T24/T52 미인수. 부모 #12
    OPEN, Draft #35 미병합. 문서 PR 자체 최종 검증·#342 종료는 별도.

<a id="d_003"></a>
### D_003 — Three-Area COPCUT Source Geometry Candidate (verification pending, 2026-11-01 KST)

- [x] **D_003 정확히 세 개 Source COPCLS 및 최대 한 개 소유 COPCUT의 Source 기하 접촉 후보 그래프 및 Qt 독립 재검증 한정 인수** · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST**
  - . [Issue #339](https://github.com/july-rust/BoardSI/issues/339), [코드 PR #340](https://github.com/july-rust/BoardSI/pull/340),
    [최종 코드 HEAD 27/27 Checks](https://github.com/july-rust/BoardSI/commit/3e608c495b9685fedbef25d607a2839372759d69/checks), 11/11 PR Workflows, 동일 HEAD Codex findings 0/미해결 0/스레드 0, Windows x64/ARM64·macOS
    ARM64 Native Rust/Qt 및 Linux Offscreen Qt PASS, [GitHub Signed Verified Squash main@7f439851](https://github.com/july-rust/BoardSI/commit/7f43985124f4b422fc002dba8633aab06e8f7402), 독립 post-main **21/21
    Checks
  - 8/8 Push Workflow SUCCESS**, [검증 원장](validation/d003-three-area-copcut-connectivity.md). **Source 기하 후보만 완료**. 제조 Copper/전기 접속, P009, 물리 SI, Solver/Cache, 정식 U09/U10/U39·T02/T04/T24/T52는 미인수이고 M1 #12
    OPEN. 문서 PR/Issue #339 종료 게이트는 별도.


<a id="d_005"></a>
### D_005 — SMD Pad → Trace → Three-Area COPCUT Source candidate (2026-11-01 KST)

- [x] **D_005 제한 Source 기하 Pad–Trace–Three-Area 접촉 후보 코드·Qt Native 인수 (전기적 연결 인수 아님)** · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST**
  - . [Issue #345](https://github.com/july-rust/BoardSI/issues/345), [코드 PR #346](https://github.com/july-rust/BoardSI/pull/346),
    [최종 코드 HEAD `2b7eaa30edab5145addacb5229b1659069149cac`](https://github.com/july-rust/BoardSI/commit/2b7eaa30edab5145addacb5229b1659069149cac/checks): 27/27 Check Runs SUCCESS·11/11 PR Workflows
    SUCCESS·Ruleset #25046595 필수 17/17. Rust fmt/Clippy, Native Qt Windows x64·ARM64/macOS ARM64 실제 D_005 단계, Linux Qt Offscreen CTest PASS, 기존 D_001~D_004 회귀 포함.
    [동일 HEAD Codex receipt](https://github.com/july-rust/BoardSI/pull/346#issuecomment-6556462329) "Didn't find any major issues", 과거 P2 5건 보완·스레드 5/5 해결.
    [Signed Verified 단일 부모 Squash `main@60d71061`](https://github.com/july-rust/BoardSI/commit/60d71061b135c9a7a4f77e6f0912452fbe6e3a96) 확인. **오직 Source 기하 후보 검증**. 실제 제조 Copper/전기 접속, P009, PH-08, SI
    Solver/Cache, 정식 U09/U10/U39·T02/T04/T24/T52 미인수; Issue #12 OPEN, Draft #35 미병합.
- [x] **D_005 보호된 코드 병합 후 정확한 main SHA 독립 검증 완료** · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST**
  - . [Verified one-parent Squash `main@60d71061b135c9a7a4f77e6f0912452fbe6e3a96`](https://github.com/july-rust/BoardSI/commit/60d71061b135c9a7a4f77e6f0912452fbe6e3a96/checks)에서 **21/21 Check Runs SUCCESS
  - 8/8 `event=push` Workflow Runs SUCCESS**, 모두 정확한 head_sha 일치. [D_005 검증 원장](validation/d005-pad-trace-area-qualification.md). 이 [x]는 제한 Source 기하 후보 코드 CI 통합만 인수. 전기/제조 Copper, P009, Solver/Cache,
    U09/U10/U39·T02/T04/T24/T52 미검증, Issue #12 OPEN.
- [ ] **D_005 별도 문서 PR #347 보호 병합 및 Issue #345 종료**
  - [문서 PR #347](https://github.com/july-rust/BoardSI/pull/347), 자체 정확한 최종 HEAD CI, Codex 독립 리뷰, GitHub Verified Squash, 별도 병합 후 Push Workflow 성공 확인 전에는 Issue #345를 종료하거나 완료 체크하지 않음.


<a id="d_006"></a>
### D_006 — Plated Via Pad → Three-Area COPCUT Source contact candidate (2026-11-01 KST)

- [x] **D_006 한정 Source 도금 Through Via 원형 Pad–3개 COPCLS/최대 1개 소유 COPCUT 접촉 후보 Rust/Qt Native 코드 인수 (전기 접속 인수 아님)** · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST**
  - . [Issue #348](https://github.com/july-rust/BoardSI/issues/348), [코드 PR #349](https://github.com/july-rust/BoardSI/pull/349),
    [정확 최종 HEAD `252405dfccf5602f158649aa79bee00cd614b78a`](https://github.com/july-rust/BoardSI/commit/252405dfccf5602f158649aa79bee00cd614b78a/checks): PR **27/27 Check Runs SUCCESS**, Ruleset #25046595
    필수 **17/17**; 동일 HEAD 독립 Codex `completed/findings=false/unresolved=false`, 과거 P2 패드 누락 개선·리뷰 스레드 해결. Rust 1.75 fmt/Clippy·Debug/Release·합성 PADS COPCUT/경계/모호한 Owner/Net/Layer/Source SHA 회귀시험, 실제 D_006
    Windows x64·ARM64·macOS ARM64 Qt Native 단계 및 Linux Qt Offscreen PASS. [Verified 단일 부모 Squash `main@19d3a877`](https://github.com/july-rust/BoardSI/commit/19d3a877c6c4d85fe99c829120958c8fc49ef8c7)
    (`verification.verified=true`, `reason=valid`). **오직 Source 기하 후보**로 인수하고 제조 Copper·Via Barrel 전기 연속성, P009, 물리 SI, PH-08, Solver/Cache, U09/U10/U39 및 T02/T04/T24/T52는 **미인수**.
    [D_006 검증 원장](validation/d006-via-three-area-qualification.md); M1 #12 OPEN, Draft #35 불변.
- [x] **D_006 병합 후 정확 `main@19d3a877` 독립 Push CI 최종 인수** · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST**
  - . [Verified 단일 부모 Squash `main@19d3a877c6c4d85fe99c829120958c8fc49ef8c7`](https://github.com/july-rust/BoardSI/commit/19d3a877c6c4d85fe99c829120958c8fc49ef8c7/checks)의 정확 HEAD에서 **21/21 Check Runs
    SUCCESS** 및 **8/8 event=push Workflow Runs SUCCESS**를 PR 체크와 독립 조회함. 전체 필수 17/17 PASS, Windows x64/ARM64·macOS ARM64 Qt Native 및 Linux Offscreen 포함. 제조 Copper 전기 접속/P009/SI Solver/Cache는 미검증이며 #12
    OPEN.
- [x] **D_006 별도 문서 PR 보호 병합·병합 후 CI 및 Issue #348 종료** · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST**
  - . [문서 PR #350](https://github.com/july-rust/BoardSI/pull/350) 최종 HEAD `f555f88e749adfc8eab15b13e87f3869a57a5a79`: **22/22 Check Runs 및 8/8 PR Workflows SUCCESS**, 동일 HEAD 독립 Codex
    `completed/findings=false/unresolved=false` 및 리뷰 스레드 0.
    [GitHub Verified 단일 부모 Squash `main@3e2e5fb9299ab0f0ed2477a8673bf0d5312c9f9d`](https://github.com/july-rust/BoardSI/commit/3e2e5fb9299ab0f0ed2477a8673bf0d5312c9f9d/checks) 확인 후 **독립 post-main 21/21
    Check Runs
  - 8/8 event=push Workflows SUCCESS**; [Issue #348](https://github.com/july-rust/BoardSI/issues/348) 실제 `CLOSED/completed` (2026-11-01 KST), 최종 증거 코멘트 기록. 문서 검증 [x]는 한정 Source Via Pad–Copper 후보 인수만 의미하며
    실제 전기 접속/P009/PH-08, SI Solver/Cache, M1 U/T는 불인수; 부모 #12 OPEN, Draft #35 미병합.

<a id="d_007"></a>
### D_007 — Plated Via Pad → Straight Trace → Three-Area COPCUT Source chain (2026-11-01~02 KST)

- [x] **D_007 한정 Source 소유 도금 Through Via 원형 Pad–Straight Trace–서로 다른 COPCLS Area 후보 코드 및 Qt 독립 검증** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - . [Issue #352](https://github.com/july-rust/BoardSI/issues/352), [코드 PR #353](https://github.com/july-rust/BoardSI/pull/353),
    [최종 코드 HEAD `d0992ba6ea204d16c6e915fef32108856ed49087`](https://github.com/july-rust/BoardSI/commit/d0992ba6ea204d16c6e915fef32108856ed49087/checks) **27/27 PR Check Runs·11/11 PR Workflows SUCCESS**,
    Ruleset #25046595 필수 17/17, 동일 HEAD Codex `completed/findings=false/unresolved=false`, 리뷰 스레드 0. Rust/Qt Source SHA 및 Via Pad/Trace/COPCLS/COPCUT 증거, Linux Qt Offscreen CTest **30/30 PASS**(실제 D_007 판정
    포함). [GitHub Verified 단일 부모 Squash `main@4b790edb0bcfdbded1c89b519e2e4b6b1ee35422`](https://github.com/july-rust/BoardSI/commit/4b790edb0bcfdbded1c89b519e2e4b6b1ee35422/checks)의 **독립 21/21 Push Check
    Runs·8/8 Push Workflows SUCCESS** 확인. 실제 전기 접속/P009/Solver 인수 아님.
- [x] **D_007 실제 Windows x64·Windows ARM64·macOS ARM64 Qt Native Selftest 보강 및 보호 병합** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - . [Native 검증 PR #354](https://github.com/july-rust/BoardSI/pull/354),
    [정확 최종 HEAD `fd6c422727df45f813a701bd563d80bc1231511f`](https://github.com/july-rust/BoardSI/commit/fd6c422727df45f813a701bd563d80bc1231511f/checks): **25/25 PR Check Runs, 9/9 PR Workflows SUCCESS**,
    필수 17/17, 세 Native OS에서 `D_007 Native Qt via trace to distinct area source and stale smoke` 실행 자체 **SUCCESS**, 동일 HEAD Codex `completed/findings=false/unresolved=false`·미해결 리뷰 0.
    [GitHub Verified 단일 부모 Squash `main@aa6ddf744622e213c50e9e877019a02560ef8755`](https://github.com/july-rust/BoardSI/commit/aa6ddf744622e213c50e9e877019a02560ef8755/checks) 확인. Source 기하 후보만 인수; 제조
    Copper 전기 접속, Via Barrel, P009, PH-08, SI Solver/Cache, U09/U10/U39·T02/T04/T24/T52 불인수. 부모 #12 OPEN, Draft #35 미병합.
- [x] **D_007 Native 보강 병합 후 정확한 `main@aa6ddf744622` 독립 Push CI 인수** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - . [GitHub Verified 단일 부모 Squash `main@aa6ddf744622e213c50e9e877019a02560ef8755`](https://github.com/july-rust/BoardSI/commit/aa6ddf744622e213c50e9e877019a02560ef8755/checks)의 정확 SHA에서 PR과 독립적으로 **21/21
    Check Runs SUCCESS
  - 8/8 event=push Workflows SUCCESS**를 확인. D_007 상세 검증 원장: [문서](validation/d007-via-trace-area-qualification.md). 실제 전기 접속·제조 Copper·P009·물리 SI·Solver·Cache·정식 M1 U/T는 미인수. 문서 PR 자체 보호 병합 및 후속 Push CI 전에는
    Issue #352 종료 금지.

<a id="d_008"></a>
### D_008 — Plated Through Via Pad → Dual-Layer Source COPCLS/COPCUT contacts (2026-11-02 KST)

- [x] **D_008 한정 Source 소유 도금 Through Via Pad–서로 다른 두 Copper Layer COPCLS/선택적 COPCUT 기하 후보 구현·검증** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - . [Issue #356](https://github.com/july-rust/BoardSI/issues/356), [코드 PR #357](https://github.com/july-rust/BoardSI/pull/357),
    [최종 Code HEAD `2ca548bf9844c77f53845420ad7757b2d29543d9`](https://github.com/july-rust/BoardSI/commit/2ca548bf9844c77f53845420ad7757b2d29543d9/checks) **27/27 Check Runs
  - 11/11 Workflows SUCCESS**. Linux Qt Offscreen **31/31 CTest PASS**, Native Qt Windows x64·ARM64, macOS ARM64 모두 성공; 동일 HEAD Codex `completed/findings=false/unresolved=false`, 미해결 리뷰 0. Code
    [GitHub Verified 단일 부모 Squash `main@f088c5f2e7ef03fd8d5accd5d46b5db1bbf813de`](https://github.com/july-rust/BoardSI/commit/f088c5f2e7ef03fd8d5accd5d46b5db1bbf813de/checks) 확인. 두 Layer 원형 Via Conductive
    Pad·원본 SHA·COPCUT/경계·좌표 Overflow 거부 및 재Import 후 캐시된 Source Receipt Replay 거부를 실제 Qt로 검사함.
- [x] **D_008 코드 보호 병합 후 동일 `main@f088c5f2...` 독립 Push CI** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - . **21/21 Check Runs SUCCESS
  - 8/8 event=push Workflows SUCCESS** (추가 Push on main dynamic 성공). [독립 Push 근거](https://github.com/july-rust/BoardSI/actions/runs/41367156646). PR CI와 구분하여 검증. 상세 검증 원장
    [D_008](validation/d008-dual-layer-via-area-qualification.md).
- [x] **D_008 문서 PR #358 독립 심사·보호 Squash 및 문서 병합 후 main Push CI·Issue #356 종료** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - . [PR #358](https://github.com/july-rust/BoardSI/pull/358) 최종 HEAD `ee9e63d7d3efc980124eea05753d1b74741fcfb2`의 **22/22 PR Checks·8/8 PR Workflows SUCCESS** 및 동일 HEAD Codex 지적·미해결 0.
    [GitHub Verified one-parent Squash `main@aaff99a4`](https://github.com/july-rust/BoardSI/commit/aaff99a4d7eac72b2a3fbe3c4c4f5364ca9082cc) 확인, 독립 post-doc-main **21/21 Check Runs·8/8 push Workflows
    SUCCESS**. [Issue #356](https://github.com/july-rust/BoardSI/issues/356) 실제 `CLOSED/completed`, 종료 근거 코멘트와 교차 확인. **Source 기하 후보 한정**이며 실제 제조/전기 접속·P009·물리 SI·Solver/Cache·정식 M1 U/T 미인수; 부모 #12 OPEN,
    Draft #35 유지.

<a id="d_009"></a>
### D_009 — Plated Through-Via Dual-Layer Pad → Layer-B Straight Trace → Third COPCLS (2026-11-02 KST)

- [x] **D_009 Source-only dual-layer Through-Via Pad → Layer-B Straight Trace → third COPCLS geometry candidate 구현 및 검증** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - . [Issue #359](https://github.com/july-rust/BoardSI/issues/359), [Code PR #360](https://github.com/july-rust/BoardSI/pull/360), [검증 원장](validation/d009-dual-layer-via-trace-qualification.md). 최종 HEAD
    `9d73bac244457aa47a536f9d8c6db9bd5a48aa9d`: **27/27 PR Checks, 11/11 PR Workflows SUCCESS**, 독립 Codex `completed/findings=false/unresolved=false`, 리뷰 스레드 0. Linux Qt **32/32 CTest PASS**; D_009 실제 Qt
    Native Windows x64·ARM64·macOS ARM64 **SUCCESS**; Source SHA·forged/stale/boundary/COPCUT negative 포함. 승인 대상은 **Source 기하 접점 후보만**.
- [x] **D_009 보호 코드 병합 및 독립 post-code-main Push CI 검증** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - . GitHub [Verified one-parent Squash `main@ca3dd2224b9bc851914b38f05240ad8b95d1602c`](https://github.com/july-rust/BoardSI/commit/ca3dd2224b9bc851914b38f05240ad8b95d1602c)
    (`verification.verified=true`, `reason=valid`, parent `aaff99a4`); 정확 HEAD 독립 **21/21 Check Runs SUCCESS·8/8 `event=push` Workflows SUCCESS** 및 별도
    [CodeQL 4/4 Jobs SUCCESS](https://github.com/july-rust/BoardSI/actions/runs/41379115057), [Qt Native post-main](https://github.com/july-rust/BoardSI/actions/runs/41379115121) 포함. 전기/도금/실제 제조
    Copper/P009/PH-08/물리 SI/Solver/Cache 및 정식 U/T는 **NOT VERIFIED/UNSUPPORTED/NOT RUN**. Parent Issue #12 OPEN, Draft #35 미병합.
- **D_009 문서 증거 정리 및 Issue 종료:** 이 코드 인수와 별도로 문서 PR 자체 최종 HEAD CI, Codex 리뷰, GitHub Verified Squash 및 독립 post-document-main Push CI를 마친 후에만 [Issue #359](https://github.com/july-rust/BoardSI/issues/359)를
  닫는다. 이 후속 절차의 실제 최종 결과는 문서 PR과 Issue #359 종료 코멘트에 남긴다.

<a id="d_010"></a>
### D_010 — Plated Through-Via Dual-Layer / Two-Trace / Four-Area Source qualification (2026-11-02 KST)

- [x] **D_010 제한 Source-only Plated Through-Via / 두 Layer Conductive Pad / Layer별 Trace 2개 / Source COPCLS Area 4개·선택적 COPCUT 후보 코드·Qt Native 인수** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - **최초 기록일: 2026-11-02 KST**
  - . [Issue #362](https://github.com/july-rust/BoardSI/issues/362), [코드 PR #363](https://github.com/july-rust/BoardSI/pull/363), [검증 원장](validation/d010-two-trace-four-area-qualification.md). 최종 Code HEAD
    `e88d67827131d705682953f819688dfdd7566684` **27/27 PR Check Runs
  - 11/11 PR Workflows SUCCESS**, Rust fmt/Clippy/Debug/Release 및 D_010 테스트 6/6, Linux Qt Offscreen **33/33 CTest PASS**, 실제 D_010 Windows x64·Windows ARM64·macOS ARM64 Native Qt 단계 **SUCCESS**, 같은 정확 HEAD
    독립 Codex `completed/findings=false/unresolved=false`, 기존 지적 수정 및 모든 리뷰 스레드 해결. Active Ruleset 25046595 필수 17개 통과,
    [GitHub Verified 단일 부모 Squash `main@dacffbba5ffe082df6ef7417073f6359a76608da`](https://github.com/july-rust/BoardSI/commit/dacffbba5ffe082df6ef7417073f6359a76608da),
    `verification.verified=true/reason=valid`, parent `d24e48ce`. 정확한 병합 SHA 기준 독립 **21/21 Checks
  - 8/8 `event=push` Workflows SUCCESS** 및 [dynamic CodeQL 4/4](https://github.com/july-rust/BoardSI/actions/runs/41386460966). **Source 기하 후보만 [x]**; 실제 제조 Copper/도금 Barrel 전기 연속성, P009/PH-08, 물리 SI,
    Solver/Cache, 정식 M1 U09/U10/U39/T02/T04/T24/T52 미인수. Parent #12 OPEN, Draft PR #35 미병합. 별도 문서 PR 자체의 HEAD CI/독립 리뷰/Verified Squash/병합 후 Push CI까지 확인한 뒤 Issue #362 종료; 문서 PR/Issue에 그 증거 별도 기재.

<a id="d_011"></a>
### D_011 — PADS Dual-Layer Source Contact Candidate Graph (2026-11-02 KST)

- [x] **D_011 Source-only Contact Candidate Graph 및 Native Qt 검증** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - **최초 기록일: 2026-11-02 KST**
  - . [Issue #365](https://github.com/july-rust/BoardSI/issues/365), [Code PR #366](https://github.com/july-rust/BoardSI/pull/366), [검증 원장](validation/d011-source-contact-candidate-graph-qualification.md).
    최종 Code HEAD `d1fed0ca9b8a93ebc8eea7a801b1f6642db52591`: **27/27 PR Checks
  - 11/11 PR Workflows SUCCESS**, Rust 1.75 fmt/Clippy 및 Debug/Release D_011 테스트 각 3/3, Linux Qt **34/34 CTest PASS**, Native Qt Windows x64·Windows ARM64·macOS ARM64 D_011 **SUCCESS**, 독립 정확 HEAD Codex
    `completed/findings=false/unresolved=false`, 리뷰 스레드 0. 실제 Import된 D_010 Source 기하와 SHA·Source ID를 독립 재검증해 **8 Nodes / 6 Layer-local Candidate Edges / 2 Local Components / 0 Cross-layer electrical
    edges** 구성. 이는 물리적 전기 연결성이 아닌 **Source 기하 접촉 후보만 인수**.
- [x] **D_011 보호 코드 병합 및 독립 post-code-main Push CI 검증** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - **최초 기록일: 2026-11-02 KST**
  - . [GitHub Verified 단일 부모 Squash `main@53efa8b64681eccf0c6ba0309206a3871752a93d`](https://github.com/july-rust/BoardSI/commit/53efa8b64681eccf0c6ba0309206a3871752a93d), parent `e3bd0e02`,
    `verification.verified=true/reason=valid`. 정확 병합 SHA 기준 독립 **21/21 Check Runs SUCCESS
  - 8/8 event=push Workflows SUCCESS**. 제조 Copper Fill/Plated Via Barrel 전기 연속성/Global Open-Short/P009/PH-08/물리 SI/Solver/Cache 및 정식 M1 U09/U10/U39/T02/T04/T24/T52는 **NOT VERIFIED/UNSUPPORTED/NOT RUN**;
    Parent Issue #12 OPEN, Draft PR #35 미병합.
- **D_011 문서/Issue 종료 게이트:** 별도 문서 PR 최종 HEAD CI, 독립 Codex 리뷰, 보호 Verified Squash 및 정확 문서 Main SHA의 독립 post-main Push CI를 확인한 뒤에만 [Issue #365](https://github.com/july-rust/BoardSI/issues/365)를 종료한다. 문서 인수
  결과는 해당 PR 및 Issue 종료 증거에 별도로 기록한다.


<a id="d_012"></a>
### D_012 — Through-Via Source Span & Plating Provenance Witness (2026-11-02 KST)

- [x] **D_012 제한 PADS Source Via Span 및 Importer-assumed 도금 Provenance 구현 코드 PR 검증·보호 병합** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - **최초 기록일: 2026-11-02 KST**
  - . [Issue #368](https://github.com/july-rust/BoardSI/issues/368), [코드 PR #369](https://github.com/july-rust/BoardSI/pull/369), [검증 원장](validation/d012-via-span-plating-provenance-qualification.md). 정확
    HEAD `8ecf5fefc83898a2c0696d0fa2bdae63f739b62d`: **27/27 PR Check Runs·11/11 PR Workflows SUCCESS**, Codex `completed/findings=false/unresolved=false`, 리뷰 스레드 0, D_012 Rust 3개 테스트 및 Linux Qt/Windows
    x64·ARM64/macOS ARM64 실제 Native D_012 smoke 성공.
    [GitHub Verified 단일 부모 Squash `main@cee3941f977a48c4de16b4837a78597b18918034`](https://github.com/july-rust/BoardSI/commit/cee3941f977a48c4de16b4837a78597b18918034). **D_011의 8 Node/6 Layer-local
    Edge/2 Component 유지; 새 층간 전기 Edge는 없음. `plated: true`는 SOURCE VERIFIED가 아닌 `PLATING_IMPORTER_ASSUMED`**. 제조 Via Barrel 전기 연속성/제조 Copper/Global Open-Short/P009/PH-08/SI Solver/Cache 및 정식 M1 U/T 인수 아님;
    Issue #12 OPEN, Draft #35 유지.
- [x] **D_012 정확한 code-main Push CI 최종 인수** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - **최초 기록일: 2026-11-02 KST**
  - . [GitHub Verified 단일 부모 Squash `main@cee3941f977a48c4de16b4837a78597b18918034`](https://github.com/july-rust/BoardSI/commit/cee3941f977a48c4de16b4837a78597b18918034/checks)의 정확 SHA에서 **21/21 Check
    Runs SUCCESS
  - 8/8 `event=push` Workflow SUCCESS**, 모든 head_sha 일치. 초기 1건 기존 D_006 Windows x64 `NotFound` 실패는 [Numerical Core Run #41428236843](https://github.com/july-rust/BoardSI/actions/runs/41428236843) 실패 Job만
    재실행한 **attempt 2가 `completed/success`**, Windows x64 포함 4/4 jobs SUCCESS로 회복; 코드 SHA 변경이나 기존 검증 우회 없음. Source Via Span과 Importer-assumed plating 범위 한정 인수이며 실제 제조 도금·전기 연결/P009/물리 Solver는 미검증.
- [x] **D_012 별도 문서 PR 및 Issue #368 종료** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - . [문서 PR #370](https://github.com/july-rust/BoardSI/pull/370) 최종 HEAD `41233672926e07bf7541c248c4222b8d108b14d1`: **22/22 PR Checks, 9/9 PR Workflows SUCCESS**, 같은 HEAD Codex
    `completed/findings=false/unresolved=false`·리뷰 스레드 0.
    [GitHub Verified 단일 부모 Squash `main@b18a46e0879b67ed8b2788a1a0c20eb3286b9443`](https://github.com/july-rust/BoardSI/commit/b18a46e0879b67ed8b2788a1a0c20eb3286b9443): 독립 post-doc-main **21/21 Check
    Runs·8/8 event=push Workflows SUCCESS**. [Issue #368](https://github.com/july-rust/BoardSI/issues/368) 실제 CLOSED/completed (2026-11-02 KST). Source Via Span/Importer-assumed plating 한정 인수이며 전기
    Barrel/P009/물리 SI 미인수; Parent #12 OPEN, Draft #35 유지.


<a id="d_013"></a>
### D_013 — PADS Legacy V1 Barrel Compatibility Provenance & Fail-Closed Qualification (2026-11-02 KST)

- [x] **D_013 제한 Source-only Legacy Barrel Projection 검증·보호 코드 병합** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - **최초 기록일: 2026-11-02 KST**
  - . [Issue #371](https://github.com/july-rust/BoardSI/issues/371), [코드 PR #372](https://github.com/july-rust/BoardSI/pull/372), [검증 원장](validation/d013-legacy-barrel-provenance-qualification.md). 정확한 최종
    코드 HEAD `3fefba3be82282ef9253c713607d6a4fd72b0c60` **27/27 PR Checks
  - 11/11 PR Workflows SUCCESS**, Rust 1.75 fmt/Clippy 및 Debug/Release (Windows x64·ARM64, macOS ARM64), Linux Qt Offscreen **36/36 CTest PASS**, 실제 Windows x64·ARM64/macOS ARM64 Qt D_013 연계 Native 단계
    PASS. Codex 같은 정확 HEAD `completed/findings=false/unresolved=false`, 리뷰 스레드 0, off-centre Source Pad 접촉만으로는 Legacy ViaCentre Link를 허용하지 않도록 Qt 독립 검증 보완.
    [GitHub Verified 단일 부모 Squash `main@b657403c4e0f30f844001c65c5d5d9adc1571f4e`](https://github.com/july-rust/BoardSI/commit/b657403c4e0f30f844001c65c5d5d9adc1571f4e), parent `b18a46e`. **8 D_011 Node /
    6 Local Edge / 2 Component / 0 전기 Cross-layer Edge**, `PLATING_IMPORTER_ASSUMED` 및 `BARREL_ELECTRICAL_NOT_VERIFIED` 고정. 실제 제조 도금/Barrel 전기 연속성/P009/PH-08/Solver/Cache/정식 M1 U/T 불인수; Parent #12 OPEN,
    Draft #35 미병합.
- [x] **D_013 정확 code-main SHA 독립 Push CI 최종 인수** · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST**
  - . [GitHub Verified 단일 부모 Squash `main@b657403c4e0f30f844001c65c5d5d9adc1571f4e`](https://github.com/july-rust/BoardSI/commit/b657403c4e0f30f844001c65c5d5d9adc1571f4e/checks)에서 독립적으로 **21/21 Check Runs
    SUCCESS·8/8 `event=push` Workflows SUCCESS**를 확인했고 모두 정확한 head_sha 일치. 기존 PR HEAD 27/27 및 11/11과 구분됨. Source-only Legacy Barrel Projection 한정; 물리 전기 연속성, 도금, P009, SI Solver/Cache 및 M1 U/T 미인수.
- [x] **D_013 별도 문서 PR 보호 병합·문서 main Push CI·Issue #371 종료** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-02 KST**
  - . [별도 문서 PR #373](https://github.com/july-rust/BoardSI/pull/373) 최종 HEAD `0d7d6835dae3a76c7da335df28f5504263eeefe1`: **22/22 PR Checks, 8/8 PR Workflows SUCCESS**, 정확 HEAD Codex
    `completed/findings=false/unresolved=false`. GitHub **Verified 단일 부모 Squash**
    [main@35cf89a0616edab57c9db3b65c55af6669bf04c4](https://github.com/july-rust/BoardSI/commit/35cf89a0616edab57c9db3b65c55af6669bf04c4), parent `b657403c`. 문서 main 독립 **21/21 Check Runs·8/8 event=push
    Workflows SUCCESS**. [Issue #371](https://github.com/july-rust/BoardSI/issues/371) 실제 CLOSED/completed (2026-11-02 KST). 이번 체크 표시 변경은 이미 검증·완료된 D_013 증빙에 대한 후속 기록 정정이며 물리적 Barrel 전기 검증 아님.

<a id="d_014"></a>
### D_014 — PADS Two-Via Source Contact Chain & Fail-Closed Qualification (2026-11-03 KST)

- [x] **D_014 제한 2-Via/3-Trace Source-only Contact Candidate Graph 구현 및 보호 코드 병합** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - **최초 기록일: 2026-11-03 KST**
  - . [Issue #374](https://github.com/july-rust/BoardSI/issues/374), [코드 PR #375](https://github.com/july-rust/BoardSI/pull/375), [검증 원장](validation/d014-two-via-progress.md). 최종 코드 HEAD
    `ee1ce7fab79216866a1f6ce471f7c6fe98cf025c` **27/27 PR Check Runs·11/11 PR Workflows SUCCESS**; 정확 HEAD Codex `completed/findings=false/unresolved=false`, 리뷰 스레드 0. Linux Qt Offscreen **37/37 CTest
    PASS**, Windows x64/ARM64·macOS ARM64 Native Qt 및 Rust 시험 SUCCESS. 4 Layer / 2 Via / 3 Trace / 7 Nodes / 4 Layer-local Edges / 3 Local Components / **0 proven Cross-layer Electrical Edges**, Source
    SHA-bound deterministic witness, 추가 Contact·내부 Pad·Arc·Remark·선분 내부 교차 fail-closed 검증.
    [GitHub Verified 단일 부모 Squash main@03203494cb99b83291e3c80cfb22bd5c98b1c4f4](https://github.com/july-rust/BoardSI/commit/03203494cb99b83291e3c80cfb22bd5c98b1c4f4).
- [x] **D_014 code-main 정확 SHA 독립 Push CI 최종 인수** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - . 정확한 `main@03203494cb99b83291e3c80cfb22bd5c98b1c4f4` (GitHub `verification.verified=true`, 유일한 parent `35cf89a`)에서 **21/21 Check Runs SUCCESS·8/8 event=push Workflows SUCCESS**, 모든 `head_sha` 동일. 기존
    D_006 macOS arm64 `NotFound` 실패는 최종 PR HEAD에서 해당 실패 Job만 재실행해 SUCCESS로 회복, 검증 생략이나 코드 SHA 변경 없음. 이는 제조 도금/Barrel 전기 연속성·Global Open/Short·P009·PH-08·Solver/Cache·정식 M1 U/T 인수가 아님. Parent #12 OPEN,
    Draft #35 유지.
- [x] **D_014 별도 문서 PR 보호 병합·문서 main Push CI·Issue #374 종료** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - . [문서 PR #376](https://github.com/july-rust/BoardSI/pull/376) 최종 HEAD `52d4f5cccf5dbcbc054bb20da5b9c3af37de186d` 자체 **22/22 PR Check Runs, 8/8 PR Workflows SUCCESS**, 정확 HEAD Codex
    `completed/findings=false/unresolved=false`, 미해결 리뷰 스레드 0.
    [GitHub Verified 단일 부모 Squash main@741da21d46811bd09f7e4211005006a5d7059249](https://github.com/july-rust/BoardSI/commit/741da21d46811bd09f7e4211005006a5d7059249), 유일한 parent
    `03203494cb99b83291e3c80cfb22bd5c98b1c4f4`, `verification.verified=true`/`reason=valid`; **해당 문서 main SHA의 독립 event=push CI 21/21 Check Runs·8/8 Workflows SUCCESS**.
    [Issue #374](https://github.com/july-rust/BoardSI/issues/374) 실제 CLOSED/completed (GitHub `closed_at=2026-11-03T00:31:09Z`/KST 09:31:09). D_014는 제한된 2-Via SOURCE-local 후보 접점 검증의 완료이며 제조 도금/Barrel 전기 연결
    또는 물리 Solver 합격으로 확대하지 않음; Parent Issue #12 OPEN, Draft PR #35 그대로 유지. 이 체크 표시는 선행 문서 main Push CI와 실제 Issue 종료가 확인된 뒤 별도 보호 PR에서 사후 기록 정정한 것이다.

<a id="d_015"></a>
### D_015 — PADS Same-Layer Trace T-Junction Source Contact & Fail-Closed Qualification (2026-11-03 KST)

- [x] **D_015 제한 Source-only T-Junction 검증 코드·Native Qt 및 보호 병합** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - **최초 기록일: 2026-11-03 KST**
  - . [Issue #378](https://github.com/july-rust/BoardSI/issues/378), [코드 PR #379](https://github.com/july-rust/BoardSI/pull/379), [검증 원장](validation/d015-t-junction-qualification.md). 코드 최종 HEAD
    `d87a19658eaae82a7907e32502215ecf8a34b9de`: **27/27 PR Check Runs·11/11 PR Workflows·22/22 Workflow Jobs SUCCESS**; 독립 동일 HEAD Codex `completed/findings=false/unresolved=false`, 미해결 리뷰 스레드 0. Rust/Qt
    Source SHA·Net·Layer·Via/Trace 완전성, 추가 Source COPCLS/중복 VIA·Pad·소수점 MILS 및 음성 케이스 검증. Windows x64/ARM64, macOS ARM64 Native, Linux Qt Offscreen SUCCESS.
    [GitHub Verified 단일 부모 Squash `main@ae89e46a8fe0568a113b5501d870d8d0711a0f10`](https://github.com/july-rust/BoardSI/commit/ae89e46a8fe0568a113b5501d870d8d0711a0f10), parent `ece21f0`,
    `verification.verified=true/reason=valid`. **8 Source candidate nodes·5 동일 층 후보 edges·3 local components·0 전기 Cross-layer edge**; 제조 Barrel 전기 연속성, Global Open/Short, P009, PH-08, Solver/Cache 및 정식 M1
    U/T 미인수.
- [x] **D_015 code-main 독립 Push CI 최종 인수** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - . 코드 main 병합 SHA [`ae89e46a8fe0568a113b5501d870d8d0711a0f10`](https://github.com/july-rust/BoardSI/commit/ae89e46a8fe0568a113b5501d870d8d0711a0f10) 기준 **21/21 GitHub Check Runs·8/8 `event=push`
    Workflows SUCCESS**, 모두 동일한 정확 SHA·완료 상태이며 실패/대기/스킵/취소 없음. 코드 PR 자체 27/27·11/11 성공과 구분하여 실제 main Push 성공을 독립 확인했다.
- [x] **D_015 별도 문서 PR 보호 병합·독립 Push CI·Issue #378 종료** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - . [문서 PR #380](https://github.com/july-rust/BoardSI/pull/380) 최종 HEAD `69cc019b9757ccd5c6951cede8a645789ea8f133`: **22/22 PR Check Runs·8/8 PR Workflows SUCCESS**, 동일 HEAD Codex
    `completed/findings=false/unresolved=false`, 미해결 리뷰 스레드 0.
    [GitHub Verified 단일 부모 Squash `main@ea187a3afead8fb1c13c4931df14860ba51914b8`](https://github.com/july-rust/BoardSI/commit/ea187a3afead8fb1c13c4931df14860ba51914b8) (`parent=ae89e46a`,
    `verification.verified=true/reason=valid`) 이후 **해당 문서 main SHA의 독립 21/21 Check Runs·8/8 `event=push` Workflows SUCCESS**, HEAD 일치·실패/대기 없음. [Issue #378](https://github.com/july-rust/BoardSI/issues/378)
    실제 GitHub `CLOSED/completed` (`closed_at=2026-11-03T02:30:10Z`, KST 11:30:10). 이는 선행 문서 main Push CI·Issue 종료가 확인된 뒤 별도 보호 문서 PR에서 사후 근거를 반영한 것이며, 제조 Barrel 전기 연속성·Global
    Open/Short·P009·PH-08·Solver/Cache·정식 M1 U/T 인수가 아니다. Parent Issue #12 OPEN, Draft PR #35 유지.


<a id="d_016"></a>
### D_016 — PADS Same-Layer Trace X-Junction SOURCE Qualification (2026-11-03 KST)

- [x] **D_016 limited code verification / protected merge** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - **최초 체크일: 2026-11-03 KST**
  - . [Issue #382](https://github.com/july-rust/BoardSI/issues/382), [code PR #383](https://github.com/july-rust/BoardSI/pull/383), [validation record](validation/d016-x-junction-qualification.md). Exact
    code HEAD `8f189b8228d114cf14bdcafeed002f2edcb07197`: 27/27 PR Check Runs, 11/11 PR Workflows SUCCESS; same-HEAD Codex completed/zero findings and zero unresolved review threads; protected
    [Verified single-parent Squash `main@ccffcb6598e07dd95cc37b526c8ff015207b3546`](https://github.com/july-rust/BoardSI/commit/ccffcb6598e07dd95cc37b526c8ff015207b3546). 8 SOURCE nodes, 5 SAME-LAYER
    candidate edges, 3 local components, 0 proven electrical cross-layer edges. Perpendicular strict centreline X only; D015 T case and near-misses fail closed. Not physical plating/copper/P009/SI Solver.
- [x] **D_016 code-main exact-SHA independent Push CI** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - . [main@ccffcb6598e07dd95cc37b526c8ff015207b3546](https://github.com/july-rust/BoardSI/commit/ccffcb6598e07dd95cc37b526c8ff015207b3546/checks) has independently verified **21/21 GitHub Check Runs
    SUCCESS and 8/8 event=push Workflows SUCCESS**, completed on the exact merged head SHA with no failed, pending, cancelled or skipped checks. This is distinct from the code PR's 27/27 and 11/11
    acceptance; source-only geometry, no manufactured electrical contact, PH-08, P009, SI Solver/Cache, or M1 U/T acceptance.
- [x] **D_016 separate documentation PR, independent doc-main Push CI and Issue #382 state reconciliation** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - . [Documentation PR #384](https://github.com/july-rust/BoardSI/pull/384) final HEAD `97d9678dc1212dcd212aaededad7645265700b27`: **22/22 PR Check Runs and 8/8 pull-request Workflows SUCCESS**,
    exact-HEAD Codex `completed/has_any_findings=false/has_unresolved_findings=false`, unresolved review threads 0. Protected
    [Verified one-parent Squash `main@48d51443ce12c601bc4d3a3fe45346c3e92646be`](https://github.com/july-rust/BoardSI/commit/48d51443ce12c601bc4d3a3fe45346c3e92646be), sole parent
    `ccffcb6598e07dd95cc37b526c8ff015207b3546`, verified=true/reason=valid. Exact document-main SHA independently verified **21/21 Check Runs SUCCESS and 8/8 event=push Workflows SUCCESS**; all heads
    match, no failures/pending/skips. [Issue #382](https://github.com/july-rust/BoardSI/issues/382) independently observed **CLOSED/completed** (`closed_at=2026-11-03T04:20:10Z`, KST 13:20:10). Chronology:
    Issue closure timestamp **precedes final post-document-main CI reconciliation**; closure was confirmed and its audit evidence updated subsequently, so no claim that CI preceded closure. Source geometry
    only, with zero proven cross-layer electrical edges; manufactured plating/Copper, global Open/Short, P009, PH-08, Solver/Cache and M1 U/T NOT ACCEPTED. Parent #12 remains OPEN; Draft PR #35 unmerged.


<a id="d_017"></a>
### D_017 — PADS Same-Layer Oblique Trace X-Junction SOURCE Qualification (2026-11-03 KST)

- [x] **D_017 제한 Source-only 비직각 X-Junction 코드/Qt 검증·보호 병합** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - **최초 체크일: 2026-11-03 KST**
  - . [Issue #386](https://github.com/july-rust/BoardSI/issues/386), [코드 PR #387](https://github.com/july-rust/BoardSI/pull/387), [검증 원장](validation/d017-oblique-x-junction-qualification.md). 정확한 최종 코드
    HEAD `e963d070619f0f137528b50eae927856f4cef2fd`: **27/27 PR Check Runs·11/11 event=pull_request Workflows SUCCESS**. 독립 Codex 동일 HEAD `completed/has_any_findings=false/has_unresolved_findings=false`,
    미해결 리뷰 스레드 0. D_016 strict Source/Canonical inventory를 보존하며 비직각 엄격 interior/interior 중심선 교차를 별도 opt-in으로 검증; 기존 D_015 T/D_016 90° X 거부·동작 유지. 공개 합성 PADS V9.4 MILS, Rust hostile 검사, Linux Qt CTest,
    Windows x64/ARM64 및 macOS ARM64 실제 Native Qt 검증 성공. **8 Source Nodes/5 Same-Layer Candidate Edges/3 Local Components/0 proven electrical cross-layer edges**.
    [GitHub Verified 단일 부모 Squash `main@236617055cc1bd7c4eb961f0853bac0caa9d0ce7`](https://github.com/july-rust/BoardSI/commit/236617055cc1bd7c4eb961f0853bac0caa9d0ce7), parent
    `ce2f63d41a4ff43b115e73bba25f78e8543757f8`; verified=true/reason=valid.
- [x] **D_017 코드 main 독립 Push CI 최종 검증** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - . 실제 보호 병합 SHA [`236617055cc1bd7c4eb961f0853bac0caa9d0ce7`](https://github.com/july-rust/BoardSI/commit/236617055cc1bd7c4eb961f0853bac0caa9d0ce7/checks) 기준 별도 **21/21 GitHub Check Runs·8/8 event=push
    Workflows SUCCESS**, 모든 HEAD SHA 정확 일치, FAILED/PENDING/CANCELLED 없음. 중간 실패는 최초 Rustfmt·테스트 모듈 unused·Windows CRLF fixture 테스트 문제를 각각 수정했고 최종 코드 HEAD CI에서 재검증; F6 Qt/Worker 독립 Workflow의 toolchain Clippy
    설치 충돌은 실패 Job만 재실행해 동일 코드 SHA에서 성공. 이는 SOURCE 기하 증거만 해당; 제조 도금/Barrel 전기 연속성, 물리 Copper, Global Open/Short, P009, PH-08, 3D EM, SI Solver, Scenario/Cache, 정식 M1 U09/U10/U39/T02/T04/T24/T52 인수가 아님.
- [x] **D_017 별도 문서 PR/문서 main Push CI/Issue #386 종료 — 최종 확인** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - . [문서 PR #388](https://github.com/july-rust/BoardSI/pull/388) 정확한 HEAD `bc8a696d790b4bc1329104e6ff1946994a0ba9a9`: **22/22 Check Runs, 8/8 event=pull_request Workflows SUCCESS**, 독립 동일 HEAD Codex
    `completed/findings=false/unresolved=false`, 미해결 리뷰 스레드 0. 보호 Squash
    [문서 main `6fce15ffe5cf7967c143bb988dfef19c1c4b8063`](https://github.com/july-rust/BoardSI/commit/6fce15ffe5cf7967c143bb988dfef19c1c4b8063) GitHub `verified=true/reason=valid`, 단일 부모
    `236617055cc1bd7c4eb961f0853bac0caa9d0ce7`. 별도 **문서-main 정확 SHA 21/21 Check Runs 및 8/8 event=push Workflows SUCCESS**, 모든 SHA 일치, 실패·대기·취소 없음.
    [Issue #386](https://github.com/july-rust/BoardSI/issues/386) 실제 `CLOSED/completed`, `closed_at=2026-11-03T05:56:08Z` (KST 14:56:08). 이 종료는 문서-main Push CI가 완료된 뒤 실행·확인되었다. 이번 증거 전용 후속 PR 자체의 CI나 미래
    병합은 완료로 선기록하지 않는다. Source-only의 8/5/3/0 후보 그래프만 인정하고 제조 Copper/Via Barrel 전기적 증명, 전역 Open/Short, P009, PH-08, Solver/Cache, M1 U/T 인수 아님. Parent #12 OPEN, Draft PR #35 OPEN·미병합 유지.

<a id="d_019"></a>
### D_019 — PADS Same-Layer Oblique Trace T-Junction SOURCE Qualification (한정 검증 완료 · 2026-11-03 KST)

- [x] **D_019 비직교 T-Junction Source-only Rust/Qt 구현·보호 병합 한정 인수** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - **최초 체크일: 2026-11-03 KST**
  - . [Issue #397](https://github.com/july-rust/BoardSI/issues/397), [코드 PR #398](https://github.com/july-rust/BoardSI/pull/398), [검증 원장](validation/d019-oblique-t-junction-qualification.md). 정확한 코드 HEAD
    `5d0ce5f4c6be8b04317ecaeb17e2c982c8db882e` **34/34 PR Check Runs·11/11 pull_request Workflows SUCCESS**, 동일 HEAD 독립 Codex `completed`/findings=false/unresolved=false/미해결 Review Thread 0. 공개 합성 PADS
    V9.4 MILS의 Source Tick strict interior·비직교·비공선 T 후보를 기존 D_015/016/017과 분리, Rust hostile/fail-closed 회귀, Linux Qt CTest **41/41 PASS**, Windows x64·ARM64 및 macOS ARM64의 전용 D_019 Native Qt Selftest 모두
    PASS. Source-only **8 nodes/5 same-layer candidate edges/3 local components/0 proven electrical cross-layer edges**. 보호된 Verified 단일 부모 Squash
    [`main@b4aa05b90535a6b9d2284e4d4a3397e65257ffc8`](https://github.com/july-rust/BoardSI/commit/b4aa05b90535a6b9d2284e4d4a3397e65257ffc8) `verified=true/reason=valid`, 독립
    [post-main **28/28 Check Runs SUCCESS**](https://github.com/july-rust/BoardSI/commit/b4aa05b90535a6b9d2284e4d4a3397e65257ffc8/checks) 및 **8/8 event=push Workflows SUCCESS**
    ([Qt Runtime](https://github.com/july-rust/BoardSI/actions/runs/41607798627), [Numerical](https://github.com/july-rust/BoardSI/actions/runs/41607798614)). **이 체크는 제한된 Source Geometry만 인정한다.** 제조
    Copper/Barrel 전기 접속, 전체 Open/Short, P009/Return Current, PH-08, RLGC/S Solver, Scenario/Cache, 정식 U/T 인수는 미완, 부모 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN.

<a id="d_020"></a>
### D_020 — PADS Same-Layer Strictly Contained Collinear Trace Overlap SOURCE Qualification (한정 검증 완료 · 2026-11-03 KST)

- [x] **D_020 Source-only 엄격 포함형 공선 Trace 중첩 Rust/Qt 구현·보호 병합 한정 인수** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - **최초 체크일: 2026-11-03 KST**
  - . [Issue #400](https://github.com/july-rust/BoardSI/issues/400), [코드 PR #401](https://github.com/july-rust/BoardSI/pull/401), [검증 원장](validation/d020-contained-collinear-overlap-qualification.md). 최종
    코드 HEAD `2a5825ff83de399865bb4e2644c0079e00fd50de`의 **34/34 Check Runs·11/11 PR Workflows SUCCESS**, 동일 HEAD 독립 Codex `completed/findings=false/unresolved=false`, 미해결 Review Thread 0. 기존 D_015/D_019
    Source-owned 4-layer·2 Through-Via·4 Trace Inventory를 재사용해 Layer-4 공선 양 끝점의 **strict interior·positive-length·Source Tick** 중첩만 opt-in 허용. 1 tick gap, 단일 종점, duplicate/경계/비포함 중첩, 다른 Net/Layer, 추가
    Artwork 등 Fail-Closed; Rust hostile 테스트·실제 Qt Import/SHA-bound 완전 Receipt 재구성 검증. Linux Qt CTest **42/42 PASS**, Windows x64·ARM64/macOS ARM64 D_020 Native Qt 테스트 각각 SUCCESS. **Source-only 8 Nodes / 5
    same-layer candidate Edges / 3 Local Components / 0 proven electrical cross-layer edges.** GitHub 보호 Verified 단일 부모 Squash
    [`main@8f29cd1a6fca6b2aef51934fd97a25afdce6a1e7`](https://github.com/july-rust/BoardSI/commit/8f29cd1a6fca6b2aef51934fd97a25afdce6a1e7) (`verified=true/reason=valid`, parent
    `63be4c7158000fa906e573962fe35bf71efedfae`). 별도 정확한 **Post-main 28/28 Check Runs·8/8 event=push Workflows SUCCESS** ([Qt Runtime](https://github.com/july-rust/BoardSI/actions/runs/41629270403),
    [Numerical](https://github.com/july-rust/BoardSI/actions/runs/41629270374)). **제조 Copper/Plated Barrel 실제 전기적 접속, Whole-board Open/Short, P009 Return Path, PH-08, RLGC/S Solver/Cache 및 정식 U/T 인수는 여전히
    NOT_VERIFIED**; 부모 Issue #12 OPEN 유지. 이 체크는 코드·Source 후보 범위만 인정하며, 후속 문서 증거 정리 PR의 병합·Issue 종료를 선기록하지 않는다.


<a id="d_021"></a>
### D_021 — PADS Same-Layer Proper Partial Collinear Trace Overlap SOURCE Qualification (한정 검증 완료 · 2026-11-03 KST)

- [x] **D_021 Source-only 부분 공선 Trace 중첩 Rust·Qt 검증 및 보호 병합 — 한정 완료** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - **최초 체크일: 2026-11-03 KST**
  - . [Issue #403](https://github.com/july-rust/BoardSI/issues/403) CLOSED/completed (2026-11-03T12:59:13Z), [코드 PR #404](https://github.com/july-rust/BoardSI/pull/404),
    [검증 원장](validation/d021-partial-collinear-overlap-qualification.md). 최종 코드 HEAD
    [`60ba127d9435e82b1a27a7d7cc714be336888750`](https://github.com/july-rust/BoardSI/commit/60ba127d9435e82b1a27a7d7cc714be336888750/checks) **34/34 PR Check Runs·11/11 PR Workflows SUCCESS**. 동일 HEAD 독립
    Codex `completed/findings=false/unresolved=false`, 리뷰 스레드 0. PADS V9.4 MILS 정수 Source Tick에서 D_020 완전 포함형과 분리된 Layer-4 공선 양의 부분 중첩 및 역방향 정규화, 1 Source Tick offset/gap·엔드포인트·Net/Layer·Source inventory
    Fail-Closed, Rust hostile 회귀와 Linux Qt Offscreen·Windows x64/ARM64·macOS ARM64 Native Qt Import/SHA-bound Receipt PASS. **8 Source Nodes / 5 same-layer Candidate Edges / 3 local Components / 0 proven
    electrical cross-layer edges**. 보호 Verified 단일 부모 Squash [`main@516c5de1480b77ae726971f83c138932ba535fff`](https://github.com/july-rust/BoardSI/commit/516c5de1480b77ae726971f83c138932ba535fff), parent
    `4fd71b4a63aab1ab2e73ccc08025c55bb1f6af03`, `verified=true/reason=valid`. 정확한 병합 SHA에서 **post-main 28/28 Check Runs·8/8 Push Workflows SUCCESS**
    ([Qt Runtime](https://github.com/july-rust/BoardSI/actions/runs/41638787878), [Numerical Core](https://github.com/july-rust/BoardSI/actions/runs/41638787928): 동일 SHA의 Windows ARM64 실패 Job 재실행 후 성공).
    **초기 Windows ARM64 Numerical CI에서 기존 D_006 테스트 2개 실패 이력 유지:** 임시 파일 충돌 가설은 미확정, 재실행 성공이 원인 해결은 아님. **Source Geometry 후보만 완료:** 실제 제조 Copper/Barrel의 전기 연속성·전역 Open/Short·P009·PH-08·RLGC/S Solver·물리
    Scenario/Cache·정식 U/T는 미완; 부모 Issue #12 OPEN, Draft #35 미병합.


<a id="d_022"></a>
### D_022 — PADS Same-Layer Collinear Endpoint-to-Endpoint Source Candidate (한정 검증 완료 · 2026-11-03 KST)

- [x] **D_022 동일 층 공선 Trace 끝점과 Via Pad Source 접촉 후보 — 한정 완료** · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST**
  - **최초 기록일: 2026-11-03 KST**
  - . [Issue #406](https://github.com/july-rust/BoardSI/issues/406) CLOSED/completed (코드 병합 시 자동 종료, post-main 증거는 이후 독립 확인), [코드 PR #407](https://github.com/july-rust/BoardSI/pull/407),
    [검증 원장](validation/d022-collinear-endpoint-contact-qualification.md). 최종 코드 HEAD
    [`7b7c1828af94e1692d5404dc00bf7e41b546bed6`](https://github.com/july-rust/BoardSI/commit/7b7c1828af94e1692d5404dc00bf7e41b546bed6/checks)에서 **34/34 PR Check Runs
  - 11/11 PR Workflows SUCCESS**, 독립 동일 HEAD Codex `completed/findings=false/unresolved=false`, 미해결 Review Thread **0**. Linux Qt Offscreen CTest **44/44 PASS** 및 Windows x64/ARM64·macOS ARM64 전용 Native Qt
    D_022 테스트 **SUCCESS**. PADS V9.4 MILS Source Tick (254 nm) 기반 Layer-4 Via-B 끝점 (35000,25000), 두 공선 Trace와 Via Pad의 **8 Source Nodes / 6 Layer-local Candidate Edges / 3 Components / 0 Proven Electrical
    Cross-layer Edges**를 Source-SHA-bound Receipt로 재구성. 역방향 Branch 통과, 1-Tick Gap/Offset·Net/Layer·적대적 Source·Stale/Forged Receipt·기존 D_015–D_021 모드 혼동은 Fail-Closed. 보호 Verified 단일 부모 Squash
    [`main@f7c1d183a91bfac536f07038d89ae8909247b4f9`](https://github.com/july-rust/BoardSI/commit/f7c1d183a91bfac536f07038d89ae8909247b4f9) (`verified=true/reason=valid`, parent
    `c0a41cfbd968ac16bbdef95878b5501be2cf61ed`), 정확한 병합 SHA의 **post-main 28/28 Check Runs
  - 8/8 Push Workflows SUCCESS** ([Qt Runtime](https://github.com/july-rust/BoardSI/actions/runs/41648213420), [Numerical Core](https://github.com/july-rust/BoardSI/actions/runs/41648213657)). **초기 실패 이력
    별도 보존:** Rustfmt/Clippy, Windows CRLF 회귀, Qt Receipt 줄바꿈은 최종 코드에서 수정; 임시 파일 문제 등을 추가 원인으로 확정하지 않음. **Source 기하 후보에만 한정:** 실제 Copper 제조·Via Barrel 도통·전역 Open/Short·P009·PH-08·RLGC/S Solver·물리
    Scenario/Cache·정식 U/T 불인수. [부모 Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN 및 Draft PR #35 미병합.

<a id="d_023"></a>
### D_023 — PADS Opposite Via-A Collinear Endpoint Source Contact (한정 코드 인수 완료 · 2026-11-04 KST)

- [x] **D_023 Source-only Via-A 공선 Trace 끝점 접촉 후보 — 한정 코드 검증 완료** · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST**
  - **최초 기록일: 2026-11-04 KST**
  - . [Issue #409](https://github.com/july-rust/BoardSI/issues/409), [코드 PR #410](https://github.com/july-rust/BoardSI/pull/410), [검증 원장](validation/d023-opposite-via-a-endpoint-qualification.md). 최신 최종 코드
    HEAD [`a39b3da592d60bd66d751f5d6295afc42e009bf8`](https://github.com/july-rust/BoardSI/commit/a39b3da592d60bd66d751f5d6295afc42e009bf8/checks)에서 **34/34 PR Checks
  - 11/11 PR Workflows SUCCESS**, 독립 동일 HEAD Codex `completed/findings=false/unresolved=false`, Review Thread 미해결 0. Linux Qt Offscreen **45/45 CTest PASS**, D_023 전용 Native Qt Windows x64·ARM64/macOS
    ARM64 **SUCCESS**. Layer-4 Via-A에서 정확한 공선 끝점 접촉 **8 Nodes / 6 Layer-local Candidate Edges / 3 Components / 0 Proven Electrical Cross-layer Edges**. D_022 Via-B 분리, 1 Source Tick Gap/Offset, Net/Layer,
    Stale/Forged Receipt 및 전기적 승격 Fail-Closed. 보호 Verified 단일 부모 Squash
    [`main@4de7ba2004e951ac1d6fabf600213e1a7ba4d4be`](https://github.com/july-rust/BoardSI/commit/4de7ba2004e951ac1d6fabf600213e1a7ba4d4be) (`verified=true/reason=valid`, parent
    `29c3b20b11b8df72a108c7f012af1b8c1acc53cc`), 정확한 코드 병합 SHA **post-main 28/28 Checks
  - 8/8 Push Workflows SUCCESS**. **과거 실패를 삭제하지 않음:** 초기 Rustfmt, Qt CLI 모드 P1, Qt 적대적 Fixture 좌표 변경의 오류는 최종 HEAD에서 수정되었으며, 기존 D_021 Windows ARM64 D_006 간헐 실패의 원인 해결로 단정하지 않음. **Source Geometry 후보만 인수:**
    제조 Copper·Via Barrel 실제 도통·전역 Open/Short·Reference Return Path·P009·PH-08·RLGC/S Solver·Scenario/Cache·정식 U/T 미인수. 부모 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN 및 Draft #35 미병합.
    문서 PR과 #409 행정 종료 상태는 코드 인수와 분리 확인.

<a id="d_024"></a>
### D_024 — PADS Same-Layer Orthogonal Via-B L-Bend Endpoint SOURCE Contact (한정 코드 인수 완료 · 2026-11-04 KST)

- [x] **D_024 Source-only Via-B 직교 Trace 끝점 접촉 후보 — 한정 코드 검증 완료** · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST**
  - **최초 기록일: 2026-11-04 KST**
  - . [Issue #412](https://github.com/july-rust/BoardSI/issues/412), [코드 PR #413](https://github.com/july-rust/BoardSI/pull/413), [검증 원장](validation/d024-orthogonal-endpoint-contact-qualification.md). 최종
    코드 HEAD [`b2ccd95573acc61a0e26bcf6534da4e7fef7ef0b`](https://github.com/july-rust/BoardSI/commit/b2ccd95573acc61a0e26bcf6534da4e7fef7ef0b/checks)에서 **34/34 PR Checks
  - 11/11 PR Workflows SUCCESS**, 동일 HEAD 독립 Codex `completed/findings=false/unresolved=false`, 미해결 Review Thread 0. Linux Qt Offscreen **46/46 CTest PASS** 및 D_024 전용 Native Qt Windows x64·Windows
    ARM64·macOS ARM64 **SUCCESS**. PADS V9.4 MILS 0.01 mil(254 nm) Source Tick, Layer-4 Via-B (35000,25000)에서 수직 L-Bend Trace 접촉 **8 Source Nodes / 6 same-layer Candidate Edges / 3 Local Components / 0
    Proven Electrical Cross-layer Edges**; 기존 D_022 공선 및 D_023 Via-A 후보와 분리. 1-Tick Gap/Offset, diagonal/collinear, Net/Layer, Source inventory, stale/forged SHA Receipt, 전기적 증거 승격 모두 Fail-Closed. 보호
    Verified Single-parent Squash [`main@9829468384ebe94e3bbec979c9ae96b26a0a6b2e`](https://github.com/july-rust/BoardSI/commit/9829468384ebe94e3bbec979c9ae96b26a0a6b2e) (`verified=true/reason=valid`,
    parent `6f1ed6678934f460a0b52e1ebc61cc362c42af78`). 정확한 코드 main SHA에서 **Post-main 28/28 Check Runs
  - 8/8 event=push Workflows SUCCESS** ([Qt Runtime](https://github.com/july-rust/BoardSI/actions/runs/41745544417), [Numerical Core](https://github.com/july-rust/BoardSI/actions/runs/41745544598)). 이 체크는
    Source Geometry 한정 인수이며 실제 전기 접속 인수가 아니다. 초기 superseded HEAD `78fed521...`의 `cargo fmt --check` 실패는 신규 CLI/테스트 formatting 수정으로 해결; 원인 범위를 확대하지 않음. **제조 Copper·Via Barrel 전기 도통, 전체 Open/Short,
    P009/PH-08 Return Path, RLGC/S Solver, Scenario/Cache, 정식 U/T 인수는 NOT_VERIFIED**. 부모 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN 및 Draft #35 미병합 유지. 문서 PR과 #412 행정 종료는 Post-main
    CI와 구분하여 판정한다.


<a id="d_025"></a>
### D_025 — PADS Same-Layer Orthogonal Via-A L-Bend Endpoint SOURCE Contact (한정 코드 인수 완료 · 2026-11-04 KST)

- [x] **D_025 Source-only Via-A 직교 Trace 끝점 접촉 후보 — 한정 코드·main Push 검증 완료** · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST**
  - **최초 기록일: 2026-11-04 KST**
  - . [Issue #415](https://github.com/july-rust/BoardSI/issues/415), [코드 PR #416](https://github.com/july-rust/BoardSI/pull/416), [검증 원장](validation/d025-orthogonal-via-a-endpoint-qualification.md). 최종 코드
    HEAD [`48d22751b2943bc56ff27b5241684d18c6257e10`](https://github.com/july-rust/BoardSI/commit/48d22751b2943bc56ff27b5241684d18c6257e10/checks)의 **34/34 PR Checks
  - 11/11 PR Workflows SUCCESS**, 동일 HEAD 독립 Codex `completed/findings=false/unresolved=false`, 미해결 Review Thread 0. Rust D_025 전용 **3/3 Debug 및 3/3 Release PASS**, Linux Qt Offscreen **47/47 CTest PASS**,
    Windows x64/ARM64·macOS ARM64 Native Qt **SUCCESS**. PADS V9.4 MILS 254 nm Source Tick의 Layer-4 **Via-A (15000,25000)** 에서 Main/Branch 직교 Trace 끝점·Pad의 **8 Source Nodes / 6 same-layer Candidate Edges /
    3 Components / 0 Proven Electrical Cross-layer Edges**를 재구성. D_024 Via-B 분리, 역방향 Branch, 1-tick Gap/Offset, diagonal/collinear, Net/Layer, Source inventory, stale/forged SHA Receipt,
    electrical-promotion Fail-Closed. 보호 Verified 단일 부모 Squash [`main@0206095de79a8a258b6d87beb4f0fd640366a7cc`](https://github.com/july-rust/BoardSI/commit/0206095de79a8a258b6d87beb4f0fd640366a7cc)
    (`verified=true/reason=valid`, parent `a050481f410f7942f7fbcda839f52086f26e4b8b`). 정확한 코드 병합 SHA에서 **post-main 28/28 Checks
  - 8/8 event=push Workflows SUCCESS** ([Qt Runtime](https://github.com/july-rust/BoardSI/actions/runs/41752196370), [Numerical Core](https://github.com/july-rust/BoardSI/actions/runs/41752196540)). **과거
    실패 기록 유지:** 초기 Rustfmt, 중간 D_024 CLI 가로채기 Codex P1, 추가 CLI Rustfmt는 각각 수정 후 최종 HEAD 재검증. **Source 기하 후보만 인수:** 제조 Copper/Plated Via Barrel 전기 도통·전역 Open/Short·P009/PH-08 Return Path·RLGC/S
    Solver·Scenario/Cache·정식 U/T 미인수. [부모 Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN, Draft #35 미병합. 문서 PR 및 Issue #415 행정 종료는 별도의 문서 검증·보호 병합 후 확인한다.

<a id="d_026"></a>
### D_026 — PADS Same-Layer 45° Oblique Via-B Endpoint-to-Endpoint SOURCE Contact (한정 코드·main Push 인수 완료 · 2026-11-04 KST)

- [x] **D_026 Source-only Via-B 45° 사선 Trace 끝점 접촉 후보 — 한정 코드 및 main Push 검증 완료** · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST**
  - **최초 기록일: 2026-11-04 KST**
  - . [Issue #418](https://github.com/july-rust/BoardSI/issues/418), [코드 PR #419](https://github.com/july-rust/BoardSI/pull/419), [검증 원장](validation/d026-oblique-via-b-endpoint-qualification.md). 최종 코드
    HEAD [`f0fd7b058cc133a31c92bd039b3616988ca264e8`](https://github.com/july-rust/BoardSI/commit/f0fd7b058cc133a31c92bd039b3616988ca264e8/checks) **34/34 PR Checks
  - 11/11 PR Workflows SUCCESS**, 독립 동일 HEAD Codex `completed/has_any_findings=false/has_unresolved_findings=false`, 미해결 Review Thread **0**. Rust D_026 전용 **Debug 3/3
  - Release 3/3 PASS**, Linux Qt 6.8.3 Offscreen **48/48 CTest PASS**, Windows x64/Windows ARM64/macOS ARM64 Native Qt **SUCCESS**. PADS V9.4 MILS **254 nm Source Tick**, L4 Main
    `(15000,25000)→(35000,25000)`, Branch `(35000,25000)→(45000,35000)`, **Via-B (35000,25000)** 의 정확한 45° Source 끝점 접촉. 역방향 Branch 허용, i128 Dot/Cross 정수 판정, 1 Tick Gap/Offset/각도 편차·직교/공선/잘못된
    Via/Net/Layer/Source Inventory·변조 및 Stale SHA Receipt·전기적 승격 Fail-Closed. **8 Source Nodes / 6 Layer-local Candidate Edges / 3 Local Components / 0 Proven Electrical Cross-layer Edges**. 보호 **Verified
    단일 부모 Squash** [`main@2608975de9237e9b8d0979bf875426dd05a8276f`](https://github.com/july-rust/BoardSI/commit/2608975de9237e9b8d0979bf875426dd05a8276f) (`verified=true/reason=valid`, parent
    `34663e350b207e434494d5f08cba3636b4903c1e`), 동일 병합 SHA에서 **post-main 28/28 Check Runs
  - 8/8 event=push Workflows SUCCESS** ([Qt Runtime](https://github.com/july-rust/BoardSI/actions/runs/41757913057), [Numerical Core](https://github.com/july-rust/BoardSI/actions/runs/41757913282)). **초기
    HEAD `a5d8730...`의 Rustfmt 실패**는 후속 HEAD에서 수정·전체 CI 재검증했으며 초기 실행 이력을 유지한다. **Source Geometry 후보 한정 인수**: 실제 Copper·Plated Via Barrel 전기 도통, 전역 Open/Short, P009/PH-08 Return Path, RLGC/S Solver,
    Physical Scenario/Cache, 정식 U/T **NOT_VERIFIED**. 부모 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN, Draft PR #35 미병합 유지. 문서 PR·문서-main Push 검증 및 Issue #418 행정 종료는 별도 확인한다.

<a id="d_027"></a>
### D_027 — PADS Same-Layer 45° Oblique Via-A Endpoint-to-Endpoint SOURCE Contact (한정 코드·main Push 인수 완료 · 2026-11-04 KST)

- [x] **D_027 Source-only Via-A 45° 사선 Trace 끝점 접촉 후보 — 한정 코드 및 main Push 검증 완료** · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST**
  - **최초 기록일: 2026-11-04 KST**
  - . [Issue #421](https://github.com/july-rust/BoardSI/issues/421), [코드 PR #422](https://github.com/july-rust/BoardSI/pull/422), [D_027 검증 원장](validation/d027-oblique-via-a-endpoint-qualification.md).
    **최종 코드 HEAD** [`d163bfc66a2e4a2a76ed6dd028d6718021d7ba80`](https://github.com/july-rust/BoardSI/commit/d163bfc66a2e4a2a76ed6dd028d6718021d7ba80/checks) **34/34 PR Checks
  - 11/11 PR Workflows SUCCESS**, 동일 HEAD 독립 Codex `completed/findings=false/unresolved=false` 및 미해결 Review Thread 0; 초기 `4b9d7cf` P1(위조 테스트의 no-op 교체)은 수정하고 해당 Thread 해결 후 새 HEAD 재리뷰 PASS. Rust D_027 전용
    **Debug 3/3
  - Release 3/3 PASS**, Linux Qt Offscreen **49/49 CTest PASS**, Windows x64/ARM64·macOS ARM64 Native Qt **SUCCESS**. PADS V9.4 MILS **254 nm Source Tick**, Layer-4 Main `(15000,25000)→(35000,25000)`,
    Branch `(15000,25000)→(5000,35000)`의 **Via-A (15000,25000)** 정확한 45° 외측 Source 끝점 접촉. Branch 역방향, 정수 i128 Dot/Cross 및 투영<0, 1-Tick Gap/Offset·각도 편차·직교/공선·Net/Layer/Source Inventory·Stale/위조 SHA
    Receipt·전기적 승격 Fail-Closed. **8 Source Nodes / 6 Layer-local Candidate Edges / 3 Local Components / 0 Proven Electrical Cross-layer Edges**; 접촉은 SOURCE_GEOMETRY_ONLY. 보호된 **Verified 단일 부모 Squash**
    [`main@2de28563231aeb15c92e695ebd17b5d73bfc200e`](https://github.com/july-rust/BoardSI/commit/2de28563231aeb15c92e695ebd17b5d73bfc200e), `verified=true/reason=valid`, sole parent
    `b66afa1342af54f542867b3c391deb8c4993f187`. 독립 **post-code-main 28/28 Checks
  - 8/8 event=push Workflows SUCCESS** ([Qt Runtime](https://github.com/july-rust/BoardSI/actions/runs/41764659506), [Numerical Core](https://github.com/july-rust/BoardSI/actions/runs/41764659200)). **실제
    제조 Copper/Via Barrel 전기 도통, 전역 Open/Short, P009/PH-08 Return Path, RLGC/S Solver, Physical Scenario/Cache 및 정식 U/T 미인수**. 부모 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN; 레거시
    [PR #35](https://github.com/july-rust/BoardSI/pull/35)는 2026-11-04 KST **Closed/Unmerged**, 기능 인수와 무관. 이번 문서 PR 자체의 정밀 CI·리뷰·병합 및 문서-main Push 증거와 Issue #421 행정 종료는 이후 별도로 확인한다.


<a id="d_028"></a>
### D_028 — PADS Same-Layer Exact 4:3 Non-45° Via-B Endpoint-to-Endpoint SOURCE Contact (한정 코드·main Push 인수 완료 · 2026-11-04 KST)

- [x] **D_028 Source-only Via-B 4:3 사선 Trace 끝점 접촉 후보 — 한정 코드 및 독립 main Push 검증 완료** · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST**
  - **최초 기록일: 2026-11-04 KST**
  - . [Issue #424](https://github.com/july-rust/BoardSI/issues/424), [코드 PR #425](https://github.com/july-rust/BoardSI/pull/425),
    [D_028 검증 원장](validation/d028-non45-oblique-via-b-endpoint-qualification.md). 정확한 최종 코드 HEAD
    [`8f6b690984be17be8392e0dc2102d8fd59e981b1`](https://github.com/july-rust/BoardSI/commit/8f6b690984be17be8392e0dc2102d8fd59e981b1/checks) **34/34 PR Check Runs
  - 11/11 PR Workflows SUCCESS**, 동일 HEAD 독립 Codex `completed/findings=false/unresolved=false`, 미해결 Review Thread 0. Rust D_028 Debug **3/3**·Release **3/3 PASS**, Linux Qt 6.8.3 Offscreen **50/50 CTest
    PASS**, Native Qt Windows x64·Windows ARM64·macOS ARM64 D_028 실제 실행과 회귀 SUCCESS. PADS V9.4 MILS 254 nm Source Tick, L4 Main `(15000,25000)→(35000,25000)`, Branch `(35000,25000)→(47000,34000)`, Via-B
    끝점에서 정확한 **4:3 비45°** Source 후보 접촉. Branch 역방향, `i128 4×|cross|=3×|dot|`, 1-Tick Gap/Offset·기울기 편차·45°/직각/공선 모드 혼동·Net/Layer/Via/Source Inventory·Stale/위조 SHA Receipt·전기적 승격 Fail-Closed. **8 Source
    Nodes / 6 Layer-local Candidate Edges / 3 Local Components / 0 Proven Electrical Cross-layer Edges** (SOURCE_GEOMETRY_ONLY). 초기 HEAD `78b4ae1`의 Cargo Rustfmt 실패는 최종 HEAD에서 형식 수정 후 전체 재검증했으며 원인을 기능 실패로
    과장하지 않음. 보호 **GitHub Verified 단일 부모 Squash** [`main@7c91dab8c735d7b9e59ed4352e3f3354dabe6188`](https://github.com/july-rust/BoardSI/commit/7c91dab8c735d7b9e59ed4352e3f3354dabe6188)
    `verified=true/reason=valid`, sole parent `4511e8d002bfa37f06ba3947b85c75d601cc2d7c`. 동일 코드-main SHA의 독립 **28/28 Check Runs
  - 8/8 event=push Workflows SUCCESS** ([Qt Runtime](https://github.com/july-rust/BoardSI/actions/runs/41773687613), [Numerical Core](https://github.com/july-rust/BoardSI/actions/runs/41773687542)). **실제
    제조 Copper/Plated Via Barrel 도통, Global Open/Short, P009/PH-08 Reference Return Path, RLGC/S Solver, Physical Scenario/Cache 및 정식 U/T 인수는 NOT_VERIFIED**. 부모
    [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN, 레거시 [PR #35](https://github.com/july-rust/BoardSI/pull/35) CLOSED/UNMERGED. 본 체크는 코드 한정 범위만 확인하며 문서 PR 자체의 CI·리뷰·병합 및 Issue #424 행정 종료는
    별도 완료 조건으로 기록한다.

<a id="d_029"></a>
### D_029 — PADS Same-Layer Exact 4:3 Non-45° Via-A Endpoint-to-Endpoint SOURCE Contact (한정 인수 완료 · 2026-11-04 KST)

- [x] **D_029 Source-only Via-A 정확한 4:3 비45° 사선 끝점 접속 후보 — 코드·문서 근거 한정 인수 완료** · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST**
  - **최초 기록일: 2026-11-04 KST**
  - . [Issue #427](https://github.com/july-rust/BoardSI/issues/427), [코드 PR #428](https://github.com/july-rust/BoardSI/pull/428), [문서 PR #429](https://github.com/july-rust/BoardSI/pull/429),
    [D_029 검증 원장](validation/d029-non45-oblique-via-a-endpoint-qualification.md). 정확한 최종 코드 PR HEAD `3f07ef8f0ee7ddcc7044f5ed3eb66acabcf698e0`의 **34/34 PR Check Runs
  - 11/11 PR Workflows SUCCESS**, 독립 동일 HEAD Codex `completed/findings=false/unresolved=false`, 미해결 리뷰 스레드 0. Rust D_029 **Debug 3/3
  - Release 3/3 PASS**, Linux Qt Offscreen **51/51 CTest PASS**, Windows x64·Windows ARM64·macOS ARM64 Native Qt 전용 D_029 및 기존 회귀 **SUCCESS**. PADS V9.4 MILS Source Tick 254 nm, L4 Main
    `(15000,25000)→(35000,25000)`, Branch `(15000,25000)→(3000,34000)`, Via-A `(15000,25000)`, 정확한 `i128 4×|cross|=3×|dot|`, Via-A 외측 음수 투영 및 Branch 역방향, 1 Tick Gap/Angle·모드 오인식·Net/Layer/Via·Source
    Inventory·Stale/Forged Receipt·전기적 승격 Fail-Closed를 재구성. **8 Source Nodes / 6 same-layer Candidate Edges / 3 local Components / 0 Proven Electrical Cross-layer Edges**, `SOURCE_GEOMETRY_ONLY`만 인정. 보호된
    GitHub **Verified 단일 부모 Squash 코드-main** [`54d345bfa5c4a60eb8d7c0dc2e9c3be7e7aeacf6`](https://github.com/july-rust/BoardSI/commit/54d345bfa5c4a60eb8d7c0dc2e9c3be7e7aeacf6), sole parent
    `bb689a47685018c1ee4eba54412eb596f0489f94`, `verified=true/reason=valid`; 정확한 코드-main SHA의 독립 **28/28 Check Runs
  - 8/8 event=push Workflows SUCCESS** ([Qt Runtime #41778597784](https://github.com/july-rust/BoardSI/actions/runs/41778597784),
    [Numerical #41778597746](https://github.com/july-rust/BoardSI/actions/runs/41778597746)). **과거 이력 보존:** 최초 코드-main Qt Push 실행은 Docs-main 동시 실행으로 인해 3개 Native Qt Job이 `CANCELLED`되었으며, 기존 실패 Job 재실행으로
    **attempt=2 모두 SUCCESS**; 처음부터 성공한 것으로 기록하지 않는다. 별도 2개 Markdown만 수정한 문서 PR #429의 최종 HEAD `5f7933a57175a10fcfb52a51dfc997b7c4c22a3d` **8/8 PR Workflows SUCCESS**, 동일 HEAD 독립 Codex
    findings=false/unresolved=false, 스레드 0. 보호 **Verified 단일 부모 docs-main**
    [`96cfe39bdd794a2c526c6c610a6e4c52dfe87a5c`](https://github.com/july-rust/BoardSI/commit/96cfe39bdd794a2c526c6c610a6e4c52dfe87a5c), sole parent `54d345bfa5c4a60eb8d7c0dc2e9c3be7e7aeacf6`,
    `verified=true/reason=valid`, 독립 **8/8 event=push Workflows SUCCESS**, **28/28 Check Runs 완료(12 SUCCESS / Docs-only 최적화에 따른 16 SKIPPED)**
    ([Qt Docs Push #41779146552](https://github.com/july-rust/BoardSI/actions/runs/41779146552)). **명시적 미인수:** 제조 Copper/Plated Via Barrel 실도통, Global Open/Short, P009·PH-08 Reference Return Path, RLGC/S,
    Physical Scenario/Cache, 정식 M1/M2 U/T 및 제품 릴리스는 **NOT_VERIFIED**. 부모 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN, 레거시 PR #35 CLOSED/UNMERGED. 본 한정 체크는 위에서 실측한 코드 및 최초 문서-main
    근거까지이며, 이 체크리스트를 업데이트하는 별도 마지막 Markdown-only 정리 PR 자체의 최종 CI·리뷰·보호 병합·post-main Push 결과는 **Issue #427에서 별도 실증 후 행정 종료**한다.


<a id="d_030"></a>
### D_030 — PADS Same-Layer Exact Steep 3:4 Non-45° Via-B Endpoint-to-Endpoint SOURCE Contact (코드-main 한정 인수 완료 · 2026-11-04 KST)

- [x] **D_030 Via-B exact steep 3:4 Source 끝점 접촉 후보 — 코드와 병합 Push 근거 한정 인수 완료** · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST**
  - **최초 기록일: 2026-11-04 KST**
  - . [Issue #431](https://github.com/july-rust/BoardSI/issues/431), [코드 PR #432](https://github.com/july-rust/BoardSI/pull/432), [검증 원장](validation/d030-non45-steep-via-b-endpoint-qualification.md). 정확한
    최종 코드 PR HEAD `c294d979e8ef1dc945be4a9174934ad6f655d565`: **34/34 PR Checks
  - 11/11 PR Workflows SUCCESS**, 독립 동일 HEAD Codex `completed/findings=false/unresolved=false`, 미해결 리뷰 스레드 0. Rust 전용 **Debug 3/3 및 Release 3/3 PASS**, Linux Qt Offscreen **52/52 CTest PASS**, Windows
    x64·Windows ARM64·macOS ARM64 D_030 Native Qt Smoke **SUCCESS**. PADS V9.4 MILS 254 nm Tick, NET_A L4, Main `(15000,25000)→(35000,25000)`, Branch `(35000,25000)→(44000,37000)`, Via-B `(35000,25000)`;
    Rust i128 `3×abs(cross)=4×abs(dot)` 및 Via-B 외측 투영, 역방향 Branch 허용, Qt 독립 Source 재구성. **8 Nodes / 6 Same-layer Candidate Edges / 3 Components / 0 Proven Electrical Cross-layer Edges**. 1-Tick
    Gap/Offset/Angle, 4:3/45°/직교/공선 모드, Net/Layer/Via, 추가·누락 Source, Stale/Forged Receipt, 전기적 증거 승격 Fail-Closed. 보호된 **Verified 단일 부모 Squash**
    [`main@f898ccc8ccedfca2be5876a986a521fb770d3dc1`](https://github.com/july-rust/BoardSI/commit/f898ccc8ccedfca2be5876a986a521fb770d3dc1) (`verified=true/reason=valid`, parent
    `beeb47e8e70301d12ae77e8ea978f52e13f4f474`) 및 정확한 code-main SHA 독립 **8/8 event=push Workflows
  - 28/28 Check Runs SUCCESS** ([Qt Runtime](https://github.com/july-rust/BoardSI/actions/runs/41795317730), [Numerical](https://github.com/july-rust/BoardSI/actions/runs/41795317722)). 초기 Rustfmt 실패·Codex
    P1(Qt CLI 옵션 불일치)는 최종 HEAD에서 수정·재검증하고 실패 이력 보존. **본 Docs-only PR 자체의 최종 CI·독립 리뷰·Verified Squash·post-doc-main Push 및 Issue #431 행정 종료는 별도로 확인**한다. SOURCE_GEOMETRY_ONLY이며 실제 Copper/Plated Via Barrel,
    Global Open/Short, P009/PH-08, RLGC/S, Physical Scenario/Cache, 정식 U/T·제품 릴리스 **NOT_VERIFIED**; 부모 Issue #12 OPEN, PR #35 CLOSED/UNMERGED.


<a id="d_031"></a>
### D_031 — PADS Same-Layer Exact 3:4 Steep Non-45° Via-A Endpoint-to-Endpoint SOURCE Contact (한정 인수 완료 · 2026-11-04 KST)

- [x] **D_031 Source-only Via-A 정확한 3:4 급경사 비45° 끝점 접촉 후보 — 코드·문서 및 main Push 한정 인수 완료** · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST**
  - **최초 기록일: 2026-11-04 KST**
  - . [Issue #434](https://github.com/july-rust/BoardSI/issues/434), [코드 PR #435](https://github.com/july-rust/BoardSI/pull/435), [D_031 검증 원장](validation/d031-non45-steep-via-a-endpoint-qualification.md).
    PADS V9.4 MILS, 254 nm Source Tick, L4 Main `(15000,25000)→(35000,25000)`, Via-A `(15000,25000)`, Branch `(15000,25000)→(6000,37000)`; signed i128 `3×|cross|=4×|dot|`와 Via-A 외측 음수 투영을 요구. 최종 코드 PR HEAD
    [`6381e86d8671a557fe80f8b85e0e0e5c7e48d2ad`](https://github.com/july-rust/BoardSI/commit/6381e86d8671a557fe80f8b85e0e0e5c7e48d2ad/checks)에서 **11/11 PR 워크플로 SUCCESS**, Rust Numerical Core/native Windows
    x64·Windows ARM64·macOS ARM64 SUCCESS, Linux Qt **53/53 CTest PASS**, D_031 전용 Native Qt Windows x64·ARM64·macOS ARM64 PASS, 독립 동일 HEAD Codex `completed/findings=false/unresolved=false`, 리뷰 Thread 0
    미해결. 최초 Superseded HEAD의 Rustfmt FAIL 및 첫 Codex P2(적대적 Qt 독립 검사 미흡)는 수정하고 재검증했으며 실패 이력 보존. 코드 PR 보호 Squash 병합 SHA
    [`main@2e5f203f82d6bf86846456bc8f89b85a50f5cbea`](https://github.com/july-rust/BoardSI/commit/2e5f203f82d6bf86846456bc8f89b85a50f5cbea) 확인, 기준 main과 비교 시 **1개 커밋**. **정확한 code-main Push 독립 검증 PASS:**
    Verified 단일 부모 Squash `2e5f203f82d6bf86846456bc8f89b85a50f5cbea`, `verified=true/reason=valid`, `event=push` **8/8 Workflow SUCCESS
  - 28/28 Check Runs SUCCESS**. 기존 [Numerical #41804760404](https://github.com/july-rust/BoardSI/actions/runs/41804760404) 및
    [Qt #41804760291](https://github.com/july-rust/BoardSI/actions/runs/41804760291)의 최초 `CANCELLED` 이력을 보존하고, 동일 SHA·동일 Push Event의 **2차 재실행에서 두 Run 모두 SUCCESS** 확인. **문서-main Push 독립 검증 PASS:**
    [PR #436](https://github.com/july-rust/BoardSI/pull/436) 최종 HEAD 8/8 PR Workflow SUCCESS, 독립 Codex findings=false/unresolved=false, 미해결 리뷰 0, Docs-only Native 16건 SKIPPED. Verified 단일 부모 Squash
    `3b9d4e65b6b17f2406e03bd67d697053139cba90`, `verified=true/reason=valid`, 정확한 `event=push` **8/8 Workflow SUCCESS
  - 28/28 Check Runs 정상(12 SUCCESS/16 SKIPPED)**. Rust Debug 3/3·Release 3/3도 [수치 코어 PR 로그](https://github.com/july-rust/BoardSI/actions/runs/41802722398)로 교차 검증. `SOURCE_GEOMETRY_ONLY`: 8 Source Nodes / 6
    layer-local Candidate Edges / 3 local Components / 0 proven Cross-layer Electrical Edges. 제조 Copper, Via Barrel 도통, 글로벌 Open/Short, P009/PH-08 Return Path, RLGC/S, Physical Scenario/Cache, 공식 U/T·제품
    릴리스 모두 **NOT_VERIFIED**. 부모 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN, 이전 PR #35 CLOSED/UNMERGED. **최종 문서 인수 PR 자체의 CI·독립 리뷰·보호 Squash 및 정확한 docs-main Push는
    [Issue #434](https://github.com/july-rust/BoardSI/issues/434)에 기록하고, 재귀 문서 PR 없이 행정 종결한다.**


<a id="d_032"></a>
### D_032 — PADS Same-Layer Mirrored Negative Steep 3:4 Non-45° Via-B SOURCE Contact (코드·문서·main Push 한정 인수 완료 · 2026-11-05 KST)

- [x] **D_032 Via-B 음의 기울기 3:4 정확한 Source 끝점 접촉 후보 — 코드·문서 및 main Push 한정 인수 완료** · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST**
  - **최초 기록일: 2026-11-05 KST**
  - . [Issue #438](https://github.com/july-rust/BoardSI/issues/438), [코드 PR #439](https://github.com/july-rust/BoardSI/pull/439),
    [D_032 검증 원장](validation/d032-non45-steep-mirrored-via-b-endpoint-qualification.md). PADS V9.4 MILS Source Tick 254 nm, NET_A L4, Main `(15000,25000)→(35000,25000)`, Via-B `(35000,25000)`, Branch
    `(35000,25000)→(44000,13000)`; Via→Exterior 기준 signed i128 `3×abs(cross)=4×abs(dot)`, 정규화 `cross<0` 및 Via-B 외측 투영 엄격 판정. Branch 역방향과 Source SHA/Receipt 독립 Rust·Qt 교차검증, 양의 D_030 모드 오인식 및 1-Tick
    Gap/Angle·Net/Layer/Via·추가/누락 Source·위조 Receipt·전기적 승격 Fail-Closed. 최종 코드 PR HEAD
    [`4ae76653fc94eeb870b314c9cb2e93e12402b6dc`](https://github.com/july-rust/BoardSI/commit/4ae76653fc94eeb870b314c9cb2e93e12402b6dc): **34/34 PR Check Runs
  - 11/11 Workflows SUCCESS**, Rust D_032 **Debug 3/3
  - Release 3/3 PASS** ([Numerical Core #41916995587](https://github.com/july-rust/BoardSI/actions/runs/41916995587)), Qt Linux **54/54 CTest PASS**, 실제 D_032 Native Qt Windows x64·Windows ARM64·macOS
    ARM64 SUCCESS ([Qt Runtime #41916995643](https://github.com/july-rust/BoardSI/actions/runs/41916995643)); 독립 동일 최종 HEAD Codex `completed/findings=false/unresolved=false`, 미해결 리뷰 0. 기존 초기 HEAD
    `369487bd...` Rustfmt FAIL/Qt `Receipt` 인수 누락, 중간 `a2b217a...` Rustfmt FAIL은 수정 후 최종 HEAD 완전 재검증했고 실패 이력을 보존. 보호 **Verified 단일 부모 Squash** 코드-main
    [`2afa5d3fccb4524037ca3f1ccf7e86b123d9a0e9`](https://github.com/july-rust/BoardSI/commit/2afa5d3fccb4524037ca3f1ccf7e86b123d9a0e9), parent `ebf039d25a7217aa87bff859ac8c0a03ed655090`,
    `verified=true/reason=valid`; **정확한 code-main `event=push` 독립 8/8 Workflows
  - 28/28 Check Runs SUCCESS** ([Qt Push #41919079083](https://github.com/july-rust/BoardSI/actions/runs/41919079083),
    [Numerical Push #41919079100](https://github.com/july-rust/BoardSI/actions/runs/41919079100)). **8 SOURCE Nodes / 6 Layer-local Candidate Edges / 3 Components / 0 Proven Electrical Cross-layer Edges**,
    `SOURCE_GEOMETRY_ONLY`. 실제 제조 Copper/Plated Barrel 도통, 전역 Open/Short, P009/PH-08 Return Path, RLGC/S, Physical Scenario/Cache, 정식 U/T 및 제품 릴리스는 **NOT_VERIFIED**. 부모 Issue #12 OPEN, 레거시 PR #35
    CLOSED/UNMERGED 유지. 별도 [문서 PR #440](https://github.com/july-rust/BoardSI/pull/440) 최종 HEAD `ed8af4c9fc268436fb303ff4e915d2f1ddfc5814`의 **8/8 Workflow SUCCESS
  - 29/29 Check Runs 완료(13 SUCCESS/16 문서 전용 SKIPPED)**, 동일 HEAD 독립 Codex `completed/findings=false/unresolved=false` 및 미해결 리뷰 0; 보호 Verified 단일 부모 Squash 문서-main
    [`b9546eb59d6b1c6f7d0e01ce01ca63603ac46b5c`](https://github.com/july-rust/BoardSI/commit/b9546eb59d6b1c6f7d0e01ce01ca63603ac46b5c), 정확한 `event=push` **8/8 Workflow SUCCESS
  - 28/28 Check Runs 완료(12 SUCCESS/16 문서 전용 SKIPPED)** 근거를 [Issue #438](https://github.com/july-rust/BoardSI/issues/438)에 기록했고 **Issue #438 CLOSED / completed** 확인. 정식 제품 U/T 인수는 계속 NOT_VERIFIED.

<a id="d_033"></a>
### D_033 — PADS Same-Layer Mirrored Negative Steep 3:4 Non-45° Via-A SOURCE Contact (코드·main Push 한정 인수 완료 · 2026-11-05 KST)

- [x] **D_033 Via-A 음의 기울기 3:4 정확한 SOURCE 끝점 접촉 후보 — Rust/Qt·코드-main Push 한정 검증 완료** · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST**
  - **최초 기록일: 2026-11-05 KST**
  - . [Issue #442](https://github.com/july-rust/BoardSI/issues/442), [코드 PR #443](https://github.com/july-rust/BoardSI/pull/443),
    [D_033 검증 원장](validation/d033-non45-steep-mirrored-via-a-endpoint-qualification.md). PADS V9.4 MILS Source Tick 254 nm, NET_A L4 Main `(15000,25000)→(35000,25000)`, Via-A `(15000,25000)`, Branch
    `(15000,25000)→(6000,13000)`, signed i128 `3×abs(cross)=4×abs(dot)`·Via-A→Exterior 정규화 **cross<0**·strict exterior projection<0 및 역방향 Branch SOURCE 허용. 양의 D_031 모드와 Via-B D_030/D_032 구분, 1 Tick
    Gap/Angle/Offset·4:3/45°/orthogonal/collinear·Net/Layer/Via·추가/누락 SOURCE·SHA/TRACE 위조/재Import stale·전기적 승격 Fail-Closed 및 Qt의 적대적 SOURCE 독립 재입력. 정확 최종 코드 HEAD
    [`39afdf94cb40cf8380b4d8d9be27ee0096093b8f`](https://github.com/july-rust/BoardSI/commit/39afdf94cb40cf8380b4d8d9be27ee0096093b8f/checks): **34/34 PR Check Runs
  - 11/11 PR Workflows SUCCESS**, 독립 동일 HEAD Codex `completed/findings=false/unresolved=false`, 리뷰 Thread 미해결 0. [Numerical #41933784234](https://github.com/july-rust/BoardSI/actions/runs/41933784234) Rust
    D_033 Debug **3/3**·Release **3/3 PASS**, Linux Qt Offscreen **55/55 CTest PASS**, Windows x64·Windows ARM64·macOS ARM64 Native Qt D_033 실제 단계 및 Qt Workflow SUCCESS
    ([Qt #41933784378](https://github.com/july-rust/BoardSI/actions/runs/41933784378)). 첫 HEAD `69dae644...` Rustfmt FAIL 및 취소 이력, 후속 `6affb1bc...` Codex P2(Hostile SOURCE/TRACE 위조 좌표 불일치)는 수정 후 최종 HEAD
    재검증 및 리뷰 Thread 해결; 이력 보존. 코드 PR 보호된 GitHub **Verified 단일 부모 Squash**
    [`main@91121f399b1d8b177a2c9ddaef92bd945488ec89`](https://github.com/july-rust/BoardSI/commit/91121f399b1d8b177a2c9ddaef92bd945488ec89), sole parent `1e465b76df4493b7a58d88c3e62a6dc3e280b058`,
    `verification.verified=true/reason=valid`. 정확 코드-main SHA에서 독립 `event=push` **8/8 Workflows SUCCESS
  - 28/28 Check Runs SUCCESS** 확인 후 문서 작업 착수. **SOURCE_GEOMETRY_ONLY: 8 Nodes / 6 same-layer Candidate Edges / 3 Components / 0 proven Cross-layer Electrical Edges**. 실제 제조 Copper/Plated Via Barrel 전기 도통,
    Global Open/Short, P009/PH-08 Return Path, RLGC/S Solver, Physical Scenario/Cache, 정식 M1/M2 U/T 및 제품 릴리스는 **NOT_VERIFIED**. 부모 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN, 레거시 PR
    #35 CLOSED/UNMERGED. 별도 Markdown-only 문서 PR의 CI·독립 Codex·Verified Squash 및 정확 docs-main Push가 최종 완료된 경우에만 Issue #442 행정 종료; 해당 마지막 증거는 이슈에 기록하고 재귀 문서 PR을 생성하지 않는다.

<a id="d_034"></a>
### D_034 — PADS Same-Layer Mirrored Negative Shallow 4:3 Non-45° Via-B SOURCE Contact (코드·main Push 한정 인수 완료 · 2026-11-05 KST)

- [x] **D_034 Via-B 음의 기울기 4:3 정확한 SOURCE 끝점 접촉 후보 — Rust/독립 Qt·코드-main Push 한정 검증 완료** · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST**
  - **최초 체크일: 2026-11-05 KST**
  - . [Issue #445](https://github.com/july-rust/BoardSI/issues/445)
  - [코드 PR #446](https://github.com/july-rust/BoardSI/pull/446)
  - [D_034 검증 원장](validation/d034-non45-oblique-mirrored-via-b-endpoint-qualification.md). PADS V9.4 MILS Source Tick 254 nm, NET_A L4 Main `(15000,25000)→(35000,25000)`, Via-B `(35000,25000)`, Branch
    `(35000,25000)→(47000,16000)`. 정수 i128 `4×abs(cross)=3×abs(dot)` 및 Via-B→Exterior 정규화 cross < 0, exterior projection > Main length²; Branch 역방향 허용. 양의 D_028은 cross > 0으로 분리하고 D_032 음의 3:4
    steep·Via-A·45°·직교·일직선·1 Tick Gap/Angle/Offset·Net/Layer/Via·추가/누락 SOURCE·SHA/TRACE 위조/재Import stale 및 전기적 승격은 Fail-Closed. **코드 PR #446 최종 HEAD**
    [`6f97dda85428cb26b0f75beb58685c676dbafb1d`](https://github.com/july-rust/BoardSI/commit/6f97dda85428cb26b0f75beb58685c676dbafb1d/checks): **34/34 Check Runs SUCCESS 및 11/11 pull_request Workflows
    SUCCESS**, 동일 HEAD 독립 Codex `completed`, `has_any_findings=false`, `has_unresolved_findings=false`, Review Thread 미해결 0.
    [Numerical PR #41941929339](https://github.com/july-rust/BoardSI/actions/runs/41941929339) Rust/Native 검증, [Qt PR #41941929384](https://github.com/july-rust/BoardSI/actions/runs/41941929384) Linux
    Offscreen 및 Windows x64·Windows ARM64·macOS ARM64 실제 Qt 검증 모두 SUCCESS. 이력: 초기 Rustfmt FAIL, 후속 Clippy `-D warnings` Dead Code FAIL, Native Qt 헤더명 오류 FAIL을 각각 수정했으며 실패한 이전 실행을 성공으로 재표기하지 않음. 보호된
    **Verified 단일 부모 Squash** [`main@6a72812d3070df068dd6f987503c90fd6505bd20`](https://github.com/july-rust/BoardSI/commit/6a72812d3070df068dd6f987503c90fd6505bd20), sole parent
    `3e9a0e24a44867c9f076f993c61a5bf171fcf2dc`, `verification.verified=true/reason=valid`; 정확 병합 코드-main `event=push`
    [**8/8 Workflow SUCCESS 및 28/28 Check Runs SUCCESS**](https://github.com/july-rust/BoardSI/commit/6a72812d3070df068dd6f987503c90fd6505bd20/checks) 확인 후 별도 Markdown-only 문서 작업 착수.
    **SOURCE_GEOMETRY_ONLY: 8 Nodes/6 same-layer Candidate Edges/3 Components/0 proven Electrical Cross-Layer Edges**. 제조 Copper/Plated Barrel 실제 전기 도통, 전역 Open/Short, P009/PH-08, RLGC/S Solver, 물리
    Scenario/Cache, 정식 M1/M2 U/T, 제품 릴리스는 **NOT_VERIFIED**. 상위 [Issue #12](https://github.com/july-rust/BoardSI/issues/12)는 OPEN, 구형 PR #35 CLOSED/UNMERGED. 별도 두 Markdown 전용 문서 PR은 독립 CI/Codex/Verified
    Squash 및 정확 docs-main Push 검증을 거쳐야 완료하며 최종 행정 증거는 Issue #445에 기록한다.


<a id="d_035"></a>
### D_035 — PADS Same-Layer Mirrored Negative Shallow 4:3 Non-45° Via-A SOURCE Contact (코드·main Push 한정 인수 완료 · 2026-11-05 KST)

- [x] **D_035 음의 기울기 4:3 Via-A 정확한 SOURCE 끝점 접촉 후보 — 코드·main Push 한정 검증 완료** · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST**
  - **최초 체크일: 2026-11-05 KST**
  - . [Issue #448](https://github.com/july-rust/BoardSI/issues/448), [코드 PR #449](https://github.com/july-rust/BoardSI/pull/449),
    [D_035 검증 원장](validation/d035-non45-oblique-mirrored-via-a-endpoint-qualification.md). PADS V9.4 MILS 254 nm Source Tick, NET_A L4 Main `(15000,25000)→(35000,25000)`, Via-A `(15000,25000)`, Branch
    `(15000,25000)→(3000,16000)`; Via-A→Exterior signed i128 `4×abs(cross)=3×abs(dot)`, `canonical_cross<0`, exterior projection `<0`, 반대 Branch SOURCE 기록 허용. D_029 양의 4:3 Via-A `canonical_cross>0`와 D_034
    음의 4:3 Via-B, D_033 음의 3:4 Via-A를 엄격히 분리; 1 Tick Gap/Offset/Angle, Wrong Net/Layer/Via/Mode, SOURCE Inventory, SHA/TRACE 위조, stale Receipt, 전기 증거 승격은 Fail-Closed. 최종 코드 PR HEAD
    [`29e3a32237b576053eb59caf7fada1a806afc7b4`](https://github.com/july-rust/BoardSI/commit/29e3a32237b576053eb59caf7fada1a806afc7b4/checks) **34/34 pull_request Check Runs SUCCESS 및 11/11 pull_request
    Workflows SUCCESS**; 동일 HEAD 독립 Codex `completed/findings=false/unresolved=false`, 미해결 Review Thread 0. [Numerical PR #41951441753](https://github.com/july-rust/BoardSI/actions/runs/41951441753) Rust
    Debug·Release 및 Windows x64/ARM64·macOS ARM64 Native Rust SUCCESS. [Qt PR #41951441750](https://github.com/july-rust/BoardSI/actions/runs/41951441750) Linux Offscreen **57/57 CTest PASS** 및 Windows
    x64·ARM64·macOS ARM64 Native Qt SUCCESS; D_035 Qt 자체 SOURCE 재입력 PASS. 초기 코드 HEAD Rustfmt 실패, 첫 독립 리뷰의 P1 두 건(CLI dispatch 누락, Qt 플래그 불일치)은 수정 후 새 HEAD에서 전체 재검증하고 당시 실패 이력을 보존. 보호 **GitHub Verified 단일
    부모 Squash** 코드-main [`51370cbb30cc5c9b3b8bf22ab243b367871bf486`](https://github.com/july-rust/BoardSI/commit/51370cbb30cc5c9b3b8bf22ab243b367871bf486), parent
    `810ba2f342baf1bec9a41569c15255ff9389c8e4`, `verification.verified=true/reason=valid`; 동일 code-main SHA에 대한 독립 **8/8 event=push Workflows SUCCESS
  - 28/28 Check Runs SUCCESS**. `SOURCE_GEOMETRY_ONLY`: **8 SOURCE Nodes / 6 Same-Layer Candidate Edges / 3 Local Components / 0 Proven Electrical Cross-Layer Edges**. 제조 Copper/Plated Via Barrel 도통, 전역
    Open/Short, P009/PH-08 Reference Return Path, RLGC/S Solver, Physical Scenario/Cache, 정식 M1/M2 U/T 및 제품 릴리스 **NOT_VERIFIED**. 상위 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN, 레거시
    [PR #35](https://github.com/july-rust/BoardSI/pull/35) CLOSED/UNMERGED. 본 체크는 **코드-main 검증 범위만** 확정하며, 별도의 두 Markdown 전용 문서 PR의 최종 CI·독립 Codex·보호 병합·정확 docs-main Push 및 Issue #448 최종 행정 종료는 완료 시 Issue
    #448 Comment에 실증한다. 재귀 문서 PR을 생성하지 않는다.

<a id="d_036"></a>
### D_036 — PADS Same-Layer Dual Mirrored Negative Shallow 4:3 Via-A/Via-B SOURCE Contact (코드·main Push 한정 인수 · 2026-11-05 KST)

- [x] **D_036 Via-A/B 양쪽 음의 기울기 4:3 복합 SOURCE 끝점 접촉 후보 — 코드·Verified Squash·main Push 한정 검증 완료** · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST**
  - **최초 체크일: 2026-11-05 KST**
  - . [Issue #451](https://github.com/july-rust/BoardSI/issues/451), [코드 PR #452](https://github.com/july-rust/BoardSI/pull/452), [D_036 검증 원장](validation/d036-dual-mirrored-via-endpoint-qualification.md).
    단일 PADS V9.4 MILS Source(254 nm/Tick, NET_A, 3 SIGNAL·2 Via·5 Trace)에서 L4 Main `(15000,25000)→(35000,25000)`, Via-A `(15000,25000)`, Via-B `(35000,25000)`, 양쪽 Branch `(15000,25000)→(3000,16000)` 및
    `(35000,25000)→(47000,16000)`을 한꺼번에 한정 판정. 양 branch 각각 signed i128 `4×abs(cross)=3×abs(dot)`, Via→Exterior 기준 `cross<0` 및 엄격 외측 투영; SOURCE 역방향 레코드 모두 허용. L1 두 Trace의 일반 정확 선분 교차, Via-on-arm, 180° 메인 반전
    테스트를 교차 검증하고 1 Tick 각도/좌표, Net/Layer/Via/Mode 오류, 누락·중복·미지원 Section, SHA/Trace/Edge 위조, stale Source, 전기적 승격을 Fail-Closed. **9 SOURCE Nodes / 8 Same-Layer Candidate Edges / 3 Local Components / 0
    Proven Electrical Cross-Layer Edges**. 최종 코드 PR HEAD [`59a6712267f4c96b5b9c6041fa6915112c9e60c3`](https://github.com/july-rust/BoardSI/commit/59a6712267f4c96b5b9c6041fa6915112c9e60c3/checks), **34/34
    PR Check Runs 및 11/11 Workflows SUCCESS**; 독립 동일 HEAD Codex `completed/findings=false/unresolved=false`, P2 두 건 수정·Review Thread 미해결 0.
    [Numerical #41960615047](https://github.com/july-rust/BoardSI/actions/runs/41960615047) Rust Debug **4/4**, Release **4/4**, Windows x64·ARM64, macOS ARM64, Rustfmt/Clippy SUCCESS;
    [Qt #41960615058](https://github.com/july-rust/BoardSI/actions/runs/41960615058) Linux Offscreen **58/58 CTest PASS**, Windows x64·ARM64, macOS ARM64 native Qt 및 독립 SOURCE 재구성 PASS. 초기
    Rustfmt/CRLF/Clippy·기존 D_006 Windows 임시파일 오류는 모두 수정 또는 동일 최종 HEAD 재실행 성공했고 최초 실패 이력을 보존한다. 보호 GitHub **Verified 단일 부모 Squash** 코드-main
    [`ed802d2de37b2a2701db333c625d10f916eeede0`](https://github.com/july-rust/BoardSI/commit/ed802d2de37b2a2701db333c625d10f916eeede0), parent `d00d866b0904a8c37dc2ef68fdcaf171c56a3ca1`, REST
    `verification.verified=true/reason=valid`; 정확 code-main SHA `ed802d2de37b2a2701db333c625d10f916eeede0`에서 **8/8 `event=push` Actions Workflows
  - 28/28 Check Runs SUCCESS**를 독립 확인([Qt Push #41963718339](https://github.com/july-rust/BoardSI/actions/runs/41963718339)
  - [Numerical Push #41963718255](https://github.com/july-rust/BoardSI/actions/runs/41963718255)); 동적 `Push on main` 개요는 8개 표준 Actions에 포함하지 않는다. **SOURCE_GEOMETRY_ONLY**, 제조 Copper/Plated Barrel 실제 도통, 전역
    Open/Short, P009/PH-08 Return Path, 물리 Scenario/Cache, RLGC/S Solver, 정식 U/T와 제품 릴리스는 **NOT_VERIFIED**. [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN, 구형 PR #35 CLOSED/UNMERGED. 본
    체크는 최종 제품 인수 완료를 뜻하지 않으며 별도 두 Markdown 전용 문서 PR의 CI·독립 Codex·Verified Squash·정확 docs-main Push와 Issue #451 종료 근거는 완료 시 Issue Comment에 기록한다.

<a id="d_037"></a>
### D_037 — PADS Same-Layer Mixed-Sign Shallow 4:3 Via-A Negative / Via-B Positive SOURCE Contact (코드·main Push 한정 인수 완료 · 2026-11-05 KST)

- [x] **D_037 Via-A 음의 기울기 4:3 + Via-B 양의 기울기 4:3 단일 PADS SOURCE 복합 접촉 후보 — 코드·Verified Squash·main Push 한정 검증 완료** · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST**
  - **최초 체크일: 2026-11-05 KST**
  - . [Issue #454](https://github.com/july-rust/BoardSI/issues/454), [코드 PR #455](https://github.com/july-rust/BoardSI/pull/455),
    [D_037 검증 원장](validation/d037-mixed-sign-dual-via-endpoint-qualification.md). PADS V9.4 MILS Source Tick 254 nm, NET_A L4 Main `(15000,25000)→(35000,25000)`, Via-A `(15000,25000)`, Via-B
    `(35000,25000)`, A Branch `(15000,25000)→(3000,16000)`, B Branch `(35000,25000)→(47000,34000)`. 두 Branch 모두 signed `i128` `4×abs(cross)=3×abs(dot)`, Via→Exterior 기준 **Via-A cross<0 / Via-B cross>0**,
    엄격 외측 투영 및 SOURCE 역방향 입력 허용. 한 개의 실제 재입력 SOURCE에서 3 SIGNAL·2 Via·5 Trace Inventory, 1 Tick Gap/Angle/Offset·잘못된 부호/Net/Layer/Via/모드·미지원/누락/중복 SOURCE·L1 교차·위조 SHA/Trace/Edge/Receipt·stale 재Import 및 전기적
    승격을 독립 Rust·Qt에서 Fail-Closed. **9 SOURCE Nodes / 8 Same-Layer Candidate Edges / 3 Local Components / 0 Proven Cross-Layer Electrical Edges**. 최종 코드 PR HEAD
    [`632fb034155f75defc7acc5c10ab4a2df61d2fee`](https://github.com/july-rust/BoardSI/commit/632fb034155f75defc7acc5c10ab4a2df61d2fee/checks) **34/34 Check Runs
  - 11/11 PR Workflows SUCCESS**; [Numerical #41969889694](https://github.com/july-rust/BoardSI/actions/runs/41969889694) D037 Rust Debug **4/4**·Release **4/4** PASS, Rustfmt/Clippy PASS;
    [Qt #41969889612](https://github.com/july-rust/BoardSI/actions/runs/41969889612) Linux Offscreen **59/59 CTest PASS** 및 Windows x64·ARM64/macOS ARM64 Native Qt SUCCESS. 독립 동일 최종 HEAD Codex
    `completed/findings=false/unresolved=false`, 미해결 Review Thread 0. 초반 Rustfmt 실패 및 독립 Codex P1 두 건(180° 회전 Branch 중복 치환, Qt Via-B 적대적 TRACE Y좌표 오류)은 수정 후 최신 HEAD 재검증했고 실패 이력을 성공으로 바꾸지 않았다. 보호된 GitHub
    **Verified 단일 부모 Squash** 코드-main [`09a407258fc330b6a17895e983b233612a1e07bf`](https://github.com/july-rust/BoardSI/commit/09a407258fc330b6a17895e983b233612a1e07bf), parent
    `61da3167db0b3239d1cbb1d29d963b6418977f28`, `verification.verified=true/reason=valid`. 정확한 code-main SHA의 독립 **8/8 `event=push` Actions Workflows
  - 28/28 Check Runs SUCCESS**, [Qt Push #41971330552](https://github.com/july-rust/BoardSI/actions/runs/41971330552)
  - [Numerical Push #41971330514](https://github.com/july-rust/BoardSI/actions/runs/41971330514). **SOURCE_GEOMETRY_ONLY**이므로 실제 Copper/Plated Barrel 전기 도통, Global Open/Short, P009/PH-08 Return Path,
    Physical Scenario/Cache, RLGC/S Solver, 정식 U/T 및 제품 릴리스는 **NOT_VERIFIED**. 부모 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN, 레거시
    [PR #35](https://github.com/july-rust/BoardSI/pull/35) CLOSED/UNMERGED. 본 체크는 **코드-main 한정 인수**이며 별도 두 Markdown 전용 문서 PR의 최종 CI·독립 Codex·보호 병합·정확 docs-main Push 및 Issue #454 행정 종결은 완료 후
    [Issue #454](https://github.com/july-rust/BoardSI/issues/454) Comment에 증거를 기록한다. 재귀 문서 PR을 생성하지 않는다.

<a id="d_038"></a>
### D_038 — PADS Same-Layer Reversed Mixed-Sign Shallow 4:3 Via-A Positive / Via-B Negative SOURCE Contact (코드·main Push 한정 인수 완료 · 2026-11-05 KST)

- [x] **D_038 Via-A 양(+) / Via-B 음(-) exact shallow 4:3 단일 SOURCE 복합 접촉 후보 — 코드·Verified Squash·정확한 main Push 한정 검증 완료** · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST**
  - **최초 체크일: 2026-11-05 KST**
  - . [Issue #457](https://github.com/july-rust/BoardSI/issues/457), [코드 PR #458](https://github.com/july-rust/BoardSI/pull/458),
    [D_038 검증 원장](validation/d038-mixed-sign-reversed-dual-via-endpoint-qualification.md). 한 개의 PADS V9.4 MILS SOURCE(254 nm/Tick, NET_A, 3 SIGNAL·2 Via·5 Trace), L4 Main `(15000,25000)→(35000,25000)`,
    Via-A branch `(15000,25000)→(3000,34000)`, Via-B branch `(35000,25000)→(47000,16000)`. Via→Exterior signed i128 exact `4×abs(cross)=3×abs(dot)`: **A dot=-240000000/cross=+180000000**, 외측
    projection=-240000000; **B dot=+240000000/cross=-180000000**, 외측 projection=640000000 (Main len²=400000000). Rust/Qt 독립 SOURCE 재Import/Exact Contact Candidate 검증, 1 Tick 기하·틀린
    부호/Net/Layer/Via·미지원/중복/누락 SOURCE·L1 교차·SHA/Trace/Edge/Receipt 위조·stale/전기적 승격 Fail-Closed, Branch 역전 및 180° 회전 정상 허용. **9 SOURCE Nodes / 8 Same-Layer Contact Candidate Edges / 3 Local Components / 0
    Proven Electrical Cross-Layer Edges**. 최종 코드 PR HEAD [`52d5ddc59e4878e9ebc3399a561eff1bbddfb734`](https://github.com/july-rust/BoardSI/commit/52d5ddc59e4878e9ebc3399a561eff1bbddfb734/checks) **34/34 PR
    Check Runs·11/11 PR Workflows SUCCESS**, [Numerical #41975273279](https://github.com/july-rust/BoardSI/actions/runs/41975273279) Rust D038 Debug **4/4**, Release **4/4**, Rustfmt/Clippy PASS;
    [Qt #41975273361](https://github.com/july-rust/BoardSI/actions/runs/41975273361) Linux Offscreen **60/60 CTest PASS** 및 Windows x64·ARM64/macOS ARM64 Native D038 Qt SUCCESS. 독립 동일 최종 HEAD Codex
    `completed/findings=false/unresolved=false`, 미해결 Thread 0. 과거 최초 HEAD Rustfmt FAIL와 Codex P1 두 건(Qt CLI Mode 불일치 및 뒤바뀐 TRACE hostile receipt 좌표)은 다음 커밋에서 수리하고 새 HEAD 전체 재검증, 실패 이력 보존. 보호된 **Verified 단일
    부모 Squash** 코드-main [`1de0cb2af0e7b0847441ea9fdca71ac413e0967e`](https://github.com/july-rust/BoardSI/commit/1de0cb2af0e7b0847441ea9fdca71ac413e0967e), sole parent
    `7096bc2f7611f9feb7f205c2b944d57f4f163664`, GitHub REST `verification.verified=true/reason=valid`; 정확한 코드-main SHA의 독립 **8/8 event=push Actions Workflows·28/28 Check Runs SUCCESS**
    ([Qt Push #41976797229](https://github.com/july-rust/BoardSI/actions/runs/41976797229)
  - [Numerical Push #41976797281](https://github.com/july-rust/BoardSI/actions/runs/41976797281)). **SOURCE_GEOMETRY_ONLY**: 실제 Copper/Pad/Via-Barrel 전기 도통, Global Open/Short, P009·PH-08 Return Path,
    Physical Scenario/Cache, RLGC/S, 정식 U/T 및 제품 릴리스는 **NOT_VERIFIED**. 부모 Issue #12 **OPEN**, 레거시 PR #35 **CLOSED/UNMERGED**. 본 체크는 **코드-main 한정 인수**이며, 별도 2개 Markdown 전용 문서 PR의 최신 HEAD 검증·Docs-only
    CI·Codex·Verified Squash·정확 docs-main Push·Issue #457 행정 종료는 [Issue #457](https://github.com/july-rust/BoardSI/issues/457) 최종 Comment에 기록한다. 재귀 문서 PR은 만들지 않는다.


<a id="d_039"></a>
### D_039 — PADS Same-Layer Copper Connectivity Foundation & Fail-Closed Qualification (코드·정확한 main Push 한정 인수 완료 · 2026-11-05 KST)

- [x] **D_039 SOURCE 소유 Same-Layer 2 SMD Pad / 2 Trace 국소 Copper 기하 Graph 및 Fail-Closed 검증 — 코드·Verified Squash·정확한 main Push 완료** · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST**
  - **최초 체크일: 2026-11-05 KST**
  - . [Issue #460](https://github.com/july-rust/BoardSI/issues/460), [코드 PR #461](https://github.com/july-rust/BoardSI/pull/461),
    [D_039 검증 원장](validation/d039-same-layer-copper-connectivity-qualification.md). 합성 PADS PowerPCB V9.4 MILS SOURCE(254 nm/Tick), Net NET_A, Layer 1 SMD Pad 2개와 직선 Trace 2개에서 엄격한 Pad 내부 접점·양의 Trace 폭·정확한
    Trace 공동 Endpoint 및 SOURCE·Net·Layer·Owner를 Rust와 실제 Qt 재Import로 독립 확인. **4 Nodes / 3 Local Copper Geometry Edges / 1 Local Component**, SHA-256 바인딩과 결정성; **LOCAL_COPPER_GEOMETRY_VERIFIED ≠ Global
    Electrical PASS**. 반전 SOURCE Trace, 헤더 4종, ASCII 탭·빈 줄 정상 처리; 1 Tick Gap, 다른 Net/Layer/폭, 중복·미지원·누락된 Decal/Part/Source, 위조 Edge/Receipt, stale 재Import, 전역 전기적 승격은 Fail-Closed. 최종 코드 PR HEAD
    [b8c9525ccbeb993525089a06ed9932849ba1a283](https://github.com/july-rust/BoardSI/commit/b8c9525ccbeb993525089a06ed9932849ba1a283/checks) **34/34 PR Check Runs·11/11 PR Workflows SUCCESS**.
    [Numerical #41993386759](https://github.com/july-rust/BoardSI/actions/runs/41993386759) Rust D039 Debug **5/5**, Release **5/5**, rustfmt/Clippy PASS;
    [Qt #41993386772](https://github.com/july-rust/BoardSI/actions/runs/41993386772) Linux Offscreen **61/61 CTest**, Windows x64·ARM64 및 macOS ARM64 Native Qt D039 PASS. 독립 최신 HEAD Codex 완료/추가 지적 0/미해결
    스레드 0. 과거 여러 HEAD의 Rustfmt·Qt 빌드·런타임 실패 및 Codex P2(중복 Pad Definition·실행 누락·빈 줄·헤더·탭 불일치)는 수정 후 새로운 HEAD 전체 재검증, 과거 실패 이력 보존. 보호된 **Verified 단일 부모 Squash** 코드-main
    [ad1f4f1992adb41b0ce47a54278ef8052b1b5909](https://github.com/july-rust/BoardSI/commit/ad1f4f1992adb41b0ce47a54278ef8052b1b5909), sole parent 69db0a263337f8e05c4be2df99f1120bb86badb1, GitHub REST
    verification.verified=true/reason=valid; 정확한 code-main에서 **8/8 event=push Workflows, 28/28 Check Runs SUCCESS** ([Qt Push #41995229580](https://github.com/july-rust/BoardSI/actions/runs/41995229580)
  - [Numerical Push #41995229505](https://github.com/july-rust/BoardSI/actions/runs/41995229505)). **제한 범위 외:** 제조 도통, 전역 Copper/Pad/Via-Barrel/Pour/Thermal Connectivity, Global Open/Short, P009·PH-08
    Return Path, Physical Scenario/Cache, RLGC/S Solver, 정식 U01–U48/T01–T54, 제품 릴리스는 **NOT_VERIFIED**. 부모 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. 문서 전용 PR은 정확히 이 체크리스트와 D039 검증 원장
    **두 Markdown 파일만 수정**, Docs-only CI·독립 Codex·Verified Squash·정확한 docs-main Push·[Issue #460](https://github.com/july-rust/BoardSI/issues/460) 행정 종료의 근거는 Issue 최종 댓글에 기록하며 재귀 문서 PR을 만들지 않는다.


<a id="d_040"></a>
### D_040 — PADS Same-Layer Multi-Component Copper Isolation & Net Separation Qualification (코드·정확한 main Push 한정 인수 완료 · 2026-11-05 KST)

- [x] **D_040 SOURCE 소유 2개 독립 동일 Layer Copper Component 구분 및 Fail-Closed 검증 — 코드·Verified Squash·정확한 code-main Push 완료** · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST**
  - **최초 체크일: 2026-11-05 KST**
  - . [Issue #463](https://github.com/july-rust/BoardSI/issues/463), [코드 PR #464](https://github.com/july-rust/BoardSI/pull/464),
    [D_040 검증 원장](validation/d040-multi-component-copper-isolation-qualification.md). 합성 PADS PowerPCB V9.4 MILS SOURCE(254 nm/Tick)의 Layer 1에서 U1·U2의 별도 SMD Pad 4개/직선 Trace 4개를 Rust와 실제 Qt
    ImportedArtwork로 독립 확인. 두 Pad-to-Pad 경로의 정확한 Pad 내부 Trace 접촉과 각 경로의 정확한 공동 Endpoint만 인정해 **8 Geometry Nodes / 6 Local Copper Edges / 2 Local Components**를 구성한다. **NET_A/NET_A** 동일 Net인 두 그룹과
    **NET_A/NET_B** 또는 **NET_B/NET_B** 사례도 별도 Component로 유지한다. SOURCE SHA-256, Owner/Pin/Net/Layer/폭/기하/Count 바인딩·결정적 Wire; 다른 Net 간 접촉·1 Tick Gap·중복/미지원 SOURCE·빈 SIGNAL·위조 Edge/Receipt·Stale 재Import·후행
    헤더를 Fail-Closed 거부. 서로 다른 Component의 Copper AABB가 겹치면 보수적으로 UNSUPPORTED 처리하며 이를 전기적 Short 증거로 선언하지 않는다. 최종 코드 PR HEAD
    [df45b9f467fa0dac29d7d3d09e38a8989cea508e](https://github.com/july-rust/BoardSI/commit/df45b9f467fa0dac29d7d3d09e38a8989cea508e/checks) **34/34 PR Check Runs·11/11 PR Workflows SUCCESS**,
    Rustfmt/Clippy/Debug/Release 및 실제 Linux/Windows x64/Windows ARM64/macOS ARM64 Qt Native PASS. 독립 최종 HEAD Codex 추가 주요 지적 0건, 과거 P1/P2 리뷰 3개 해결·미해결 Thread 0. 과거 Rustfmt·빈 SIGNAL Fail-Closed·Windows
    CRLF·NET_B-only Qt Parity·Qt 후행 Header 누락 오류는 각 HEAD에서 수정 후 새 HEAD 검증했으며 과거 성공 결과를 재사용하지 않았다. 보호된 **Verified 단일 부모 Squash**
    [code-main f65d6abbdb7e289a27e6b548798d5fcc4c03df9a](https://github.com/july-rust/BoardSI/commit/f65d6abbdb7e289a27e6b548798d5fcc4c03df9a), sole parent 804d5a8e3839683ff06dc6e68d891d3ea6830d44, REST
    verification.verified=true/reason=valid, main ref 일치. 독립 정확 code-main event=push **8/8 워크플로·28/28 Check Runs SUCCESS** ([Qt Push](https://github.com/july-rust/BoardSI/actions/runs/42026344040)
  - [Numerical Push](https://github.com/july-rust/BoardSI/actions/runs/42026344056)). **제한 범위 외:** 전역 Electrical Open/Short, 실물 도통, 모든 Copper/Pad/Via/Pour/Thermal 연결, P009·PH-08 Return Path, Physical
    Scenario/Cache, RLGC/S Solver, U01–U48/T01–T54 정식 인수 및 릴리스는 **NOT_VERIFIED**. 상위 [Issue #12](https://github.com/july-rust/BoardSI/issues/12)는 OPEN이다. 이 체크는 코드-main 한정 완료를 기록한다. 문서 전용 PR·Docs-only CI·독립
    Codex·Verified Squash·정확한 docs-main Push 및 Issue #463 최종 종료 증거는 Issue의 최종 Comment에 별도 기록한다. 재귀 문서 PR을 만들지 않는다.


<a id="d_041"></a>
### D_041 — PADS Same-Layer SOURCE Bridge Copper Qualification (코드·정확한 main Push 한정 인수 완료 · 2026-11-05 KST)

- [x] **D_041 SOURCE-owned 단일 Bridge Trace의 두 독립 Component 국소 병합 및 Fail-Closed 한정 검증** · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST**
  - **최초 체크일: 2026-11-05 KST**
  - . [Issue #466](https://github.com/july-rust/BoardSI/issues/466), [코드 PR #467](https://github.com/july-rust/BoardSI/pull/467), [D_041 검증 원장](validation/d041-same-layer-copper-bridge-qualification.md).
    공개 합성 PADS V9.4 MILS SOURCE(254 nm/Tick)에서 Layer 1 SMD Pad 4개·직선 Trace 5개, 기존 분리 Component 2개의 정확한 두 Junction을 제5 Bridge Trace로 연결: **9 Nodes / 8 SOURCE 국소 Edges / 1 Local Component**,
    Net/Layer/Pad/Trace/Width/Owner/Source SHA 바인딩. NET_A/NET_B, 반전 Trace 방향, 지원 헤더, CRLF/탭 변형 Rust/Qt 독립 검증; 1 Tick Gap·Wrong-Net/Layer/Width·위조 Receipt·stale Import·누락/중복·비지원 기하 Fail-Closed. Rust D041
    **3/3**, D040 **5/5**, Linux Qt **63/63 CTest**, Windows x64·ARM64/macOS ARM64 기존 Native CI PASS. 최종 코드 PR HEAD
    [56f9d8b75](https://github.com/july-rust/BoardSI/commit/56f9d8b75c79e471f0caf3a8f322747239775532/checks) **34/34 PR Checks / 11/11 Workflows SUCCESS**, 동일 HEAD 독립 Codex 검토 완료·신규 지적 0·미해결 리뷰 스레드 0. 보호
    **Verified 단일 부모 Squash** 코드-main [154e8e24556459044053a9b099ea9c3a5557d824](https://github.com/july-rust/BoardSI/commit/154e8e24556459044053a9b099ea9c3a5557d824), GitHub REST
    verified=true/reason=valid, 정확한 code-main **event=push 8/8 Workflows / 28/28 Check Runs SUCCESS**
    ([Qt Push](https://github.com/july-rust/BoardSI/actions/runs/42036225504)·[Numerical Push](https://github.com/july-rust/BoardSI/actions/runs/42036225380)). 코드 감사 한정 결과이며 제조 실물 도통, Global Electrical
    Open/Short, Cross-Layer Via/Pour/Thermal, P009·PH-08 Return Path, Physical Solver/Scenario/Cache, RLGC/S/DC–30 GHz, U01–U48/T01–T54 및 제품 릴리스는 **NOT_VERIFIED**. 상위
    [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. 본 체크의 문서 전용 최종 CI/보호 병합 및 Issue 행정 종료 결과는 Issue #466 최종 댓글에 기록하며 재귀 문서 PR을 만들지 않는다.


<a id="d_042"></a>
### D_042 — PADS Same-Layer Copper Closed-Loop / Redundant Path Connectivity & Fail-Closed Qualification (코드·정확한 main Push 한정 인수 완료 · 2026-11-06 KST)

- [x] **D_042 SOURCE 소유 동일 Layer 폐회로 및 Redundant Path 국소 Connectivity Graph 한정 검증 — 코드·Verified Squash·정확한 main Push 검증 완료** · **체크일: 2026-11-06 KST** · **근거일: 2026-11-06 KST**
  - **최초 체크일: 2026-11-06 KST**
  - . [Issue #469](https://github.com/july-rust/BoardSI/issues/469), [코드 PR #470](https://github.com/july-rust/BoardSI/pull/470), [D_042 검증 원장](validation/d042-same-layer-closed-loop-qualification.md). 공개
    합성 PADS V9.4 MILS SOURCE(254 nm/Tick), NET_A/NET_B, Layer 1의 기존 SMD Pad 4개/Trace 5개/D_041 Bridge에 SOURCE-owned 우회 직선 Trace 5개를 추가: **14 SOURCE 기하 Nodes / 14 Same-Layer 후보 Edges / 1 Local Component / 1
    Independent Local Cycle**, 실제 Cycle rank는 Graph Traversal 및 `E−V+C` 교차 확인. 정확히 지원하는 두 Junction의 연결과 10개 Trace SOURCE Pin-Pair 선언·좌표·소유권/Net/Layer/폭, SHA-256 Receipt를 Rust와 Native Qt ImportedArtwork에서
    독립 재구성; 반전 방향, V9.4 지원 Header, CRLF·탭, NET_B 회귀와 1 Tick Gap·Wrong-Net/Layer/Width·누락/중복·비지원 Geometry/Sections·위조 Cycle/Edge/Receipt·Stale Import·Pin-Pair 선언 Swap을 Fail-Closed 거부. **이 테스트는 고정된 합성 직선 폐회로
    사례이며 임의 실 PCB/복잡 폐회로 지원을 의미하지 않음.** 최종 코드 PR HEAD [`9cd97d4bc24deaf0c42a30e5c1a6d3999f1f6365`](https://github.com/july-rust/BoardSI/commit/9cd97d4bc24deaf0c42a30e5c1a6d3999f1f6365/checks): **34/34
    pull_request Check Runs
  - 11/11 Workflows SUCCESS**, Rustfmt/Clippy·Rust D_042 Debug **3/3** 및 Release **3/3** PASS, Linux Offscreen Qt **64/64 CTest PASS**, Windows x64/ARM64·macOS ARM64 실제 Native Qt CI SUCCESS. 최신 HEAD 독립
    Codex `completed/findings=false/unresolved=false`, 과거 P2 두 건(위조 Cycle 검증 시 stale Source, Qt SOURCE Pin-Pair Owner Swap 검증 누락) 수정 후 최신 HEAD 재검토 및 미해결 리뷰 Thread 0; 중간 Rustfmt/Clippy 실패 이력 보존. 보호된
    **GitHub Verified 단일 부모 Squash** 코드-main [`932e5bcd872fab20393b01cd46ace71a0c685633`](https://github.com/july-rust/BoardSI/commit/932e5bcd872fab20393b01cd46ace71a0c685633), parent
    `51a36f173a3ad785151df30c9635997445dc1eaa`, REST `verification.verified=true/reason=valid`, 정확 main-ref 일치. 정확한 code-main SHA의 독립 **8/8 `event=push` Workflows SUCCESS
  - 28/28 Check Runs SUCCESS**, [Qt Push #42130513630](https://github.com/july-rust/BoardSI/actions/runs/42130513630)
  - [Numerical Push #42130513899](https://github.com/july-rust/BoardSI/actions/runs/42130513899). **SOURCE_LOCAL_GEOMETRY_ONLY**: 제조 실물 Copper 도통, 전역 Electrical Open/Short, 전체 Layer/Via/Pour/Thermal,
    P009·PH-08 Return Path, Physical Scenario/Cache, RLGC/S Solver, U01–U48/T01–T54 및 제품 릴리스는 **NOT_VERIFIED**. 상위 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) **OPEN**. 본 체크는 **코드-main 한정
    인수**만 확정하며, 별도 두 Markdown 전용 문서 PR의 CI·독립 리뷰·Verified Squash·정확한 docs-main Push 및 Issue #469 행정 종료 증거는 [Issue #469](https://github.com/july-rust/BoardSI/issues/469) 최종 댓글에 기록한다. 재귀 문서 PR을 만들지 않는다.

<a id="d_043"></a>
### D_043 — PADS Same-Layer Copper Redundant Path Cut-Set / Single-Trace Failure & Fail-Closed Qualification (코드·정확한 main Push 한정 인수 완료 · 2026-11-06 KST)

- [x] **D_043 완전 SOURCE 검증 이후 가상 Trace Cut-Set 및 국소 Redundancy Graph 진단 — 코드·Verified Squash·main Push 한정 인수** · **체크일: 2026-11-06 KST** · **근거일: 2026-11-06 KST**
  - **최초 체크일: 2026-11-06 KST**
  - . [Issue #472](https://github.com/july-rust/BoardSI/issues/472), [코드 PR #473](https://github.com/july-rust/BoardSI/pull/473), [D_043 검증 원장](validation/d043-same-layer-cutset-qualification.md). D_042의
    공개 합성 PADS V9.4 MILS NET_A/NET_B Layer 1 (SMD Pad 4개/Trace 10개)을 먼저 완전 검증한 후, 원본 바이트·SHA를 보존하고 별도 `SOURCE_LOCAL_COUNTERFACTUAL_ONLY` 진단에서 Trace Node/Incident Edge만 제외. **12가지**(Baseline 14 Nodes/14
    Edges/1 Component/1 Cycle; T4 또는 T5–T9 단일 제외 13/12/1/0; T4+T5–T9 각각 이중 제외 12/10/2/0)의 Graph Traversal, `E−V+C`, U1/U2 국소 Reachability를 Rust·Native Qt에서 독립 계산. 임의 ID·중복/역순 Cut·누락/위조
    SOURCE·Pin-Pair/Owner 교환·Stale Import·위조 Receipt를 Fail-Closed. **정확 최종 코드 HEAD**
    [`ca46c3be13db0d88380d2cbc89c73c7921fa57ab`](https://github.com/july-rust/BoardSI/commit/ca46c3be13db0d88380d2cbc89c73c7921fa57ab/checks) **34/34 PR Check Runs
  - 11/11 PR Workflows SUCCESS**, Rustfmt/Clippy 및 Rust D_043 테스트, Linux Native Qt **65/65 CTest PASS**, Windows x64/ARM64·macOS ARM64 Qt SUCCESS; 동일 HEAD 독립 Codex
    `completed/findings=false/unresolved=false`, 미해결 Review Thread 0. 중간 Rust 빌드·Rustfmt·Clippy 실패는 후속 HEAD에서 수정·재검증됐으며 역사적 기록으로 유지. 보호된 **GitHub Verified 단일 부모 Squash**
    [코드-main `4dd458e5670dcb8fafd705c233bfedc25adfd517`](https://github.com/july-rust/BoardSI/commit/4dd458e5670dcb8fafd705c233bfedc25adfd517), parent `528c67fdb632b26c35f9ba956e582d5ca89e0a4e`, REST
    `verification.verified=true/reason=valid`, 정확 main-ref 일치. 코드-main 정확 SHA `event=push` **8/8 Workflows
  - 28/28 Check Runs SUCCESS**. 본 **SOURCE 국소 가상 단선** 검증은 실제 Copper 단선이나 제조 도통/전역 Electrical Open/Short, multilayer/Plane/Pour/Thermal, P009·PH-08 Return Path, Physical Scenario/Cache, RLGC/S Solver,
    U01–U48/T01–T54 및 제품 릴리스를 **NOT_VERIFIED**로 유지한다. 상위 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) **OPEN**. 별도의 Markdown-only 문서 PR 검증·병합·docs-main Push 및 Issue #472 최종 종료 증거는
    [Issue #472](https://github.com/july-rust/BoardSI/issues/472)의 마지막 댓글에 기록한다. **재귀 문서 PR을 만들지 않는다.**


<a id="d_044"></a>
### D_044 — PADS Same-Layer Parallel Finite-Width Trace Contact (코드 한정 검증 · 2026-11-06 KST)

- [x] **D_044 SOURCE 소유 동일 Layer 평행 Trace 선폭 접촉 후보의 정확한 둥근 끝단 접촉 및 Fail-Closed 코드 한정 인수** · **체크일: 2026-11-06 KST** · **근거일: 2026-11-06 KST**
  - **최초 체크일: 2026-11-06 KST**
  - . [Issue #475](https://github.com/july-rust/BoardSI/issues/475)
  - [코드 PR #476](https://github.com/july-rust/BoardSI/pull/476)
  - [검증 원장](validation/d044-parallel-finite-width-contact-qualification.md). 공개 합성 PADS V9.4 MILS, Layer 1, 254 nm/Tick, U1/U2 Pad 4개와 기존 Trace 4개 + 추가 52 mil Trace 하나의
    SOURCE/Owner/Pin-Pair/Net/Layer/Width/Geometry/SHA-256 완전 검증. 중심선 간격 30 mil, 합 반폭 31 mil의 엄격한 둥근 Copper 면적 접촉 후보만 인정하여 **9 Nodes / 10 국소 Edges / 1 Local Component** 구성. Rust i128와 Native Qt가 각각 정확한 선분
    최소거리 제곱 판정식을 구현하며 정상 SOURCE Receipt를 독립 재구성한다. Rust 회귀·CLI는 50 mil 접선, 48 mil Gap, 358–358.5 mil AABB 코너 Gap 및 Wrong Layer/Owner/Missing SOURCE를 거부하고, Qt Runner는 정상/반전/NET_B/CRLF/탭 재Import, 위조 Receipt,
    stale SHA 및 50 mil 접선을 직접 검증한다. **Qt에서 코너 Gap·48 mil Gap·Wrong Layer/Owner·Missing SOURCE를 별도로 재Import하는 음성 회귀는 아직 실행되지 않았으므로 Qt 독립 테스트 PASS로 표시하지 않는다.**
    [최종 코드 HEAD e4c944fe8d52cf3299d2e2252581fac97488a9aa](https://github.com/july-rust/BoardSI/commit/e4c944fe8d52cf3299d2e2252581fac97488a9aa/checks): **34개 PR Checks 중 33 SUCCESS·1 NEUTRAL·실패 0, 11/11 PR
    Workflows SUCCESS**, 동일 HEAD 독립 Codex 지적 없음·미해결 Review Thread 0. 초기 P2 AABB 허위 접촉, Rustfmt/소유권/CRLF, Qt 매크로 오류 및 기존 macOS D_006 간헐 임시파일 실패는 수정 또는 동일 코드 HEAD 재실행 성공으로 최종 검증. 보호
    [Verified 단일 부모 Squash 코드 main c46eaae7bc13cc224cc96f1b0d64172392c55bd5](https://github.com/july-rust/BoardSI/commit/c46eaae7bc13cc224cc96f1b0d64172392c55bd5), parent
    06f4433963a12522a0de68085df69f19b9420138, verification.verified=true. 코드-main 및 별도 문서-main의 정확한 독립 Push CI 최종 상태는 **검증 종료 후 Issue #475 최종 댓글로 기록**하며, 이 코드 인수 근거를 미래 결과로 소급하지 않는다.
    SOURCE_LOCAL_FINITE_WIDTH_ONLY: 제조 도통, Global Electrical Open/Short, 임의 PCB/Pours/Planes/Vias/Thermals, P009/PH-08 Return Path, 실제 Scenario/Cache 및 RLGC/S Solver, U01–U48/T01–T54 정식 인수·제품 출시
    **NOT_VERIFIED**. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. Issue #475는 코드 병합 시점 2026-11-06T02:35:04Z에 CLOSED/completed여서 문서 최종 검증과 시점 차이를 마지막 Issue 댓글에 명시한다. 재귀 문서 PR
    금지.

<a id="d_045"></a>
### D_045 — PADS Same-Layer Perpendicular Finite-Width Trace Contact & Fail-Closed Qualification (코드 한정 인수 · 2026-11-06 KST)

- [x] **D_045 SOURCE 소유 동일 Layer 직교 Trace의 실제 선폭·둥근 끝단 접촉 후보 및 D_044 Native Qt 적대적 재Import 보완 — 한정 코드·Verified Squash 검증 완료** · **체크일: 2026-11-06 KST** · **근거일: 2026-11-06 KST**
  - **최초 체크일: 2026-11-06 KST**
  - . [Issue #478](https://github.com/july-rust/BoardSI/issues/478)
  - [코드 PR #479](https://github.com/july-rust/BoardSI/pull/479)
  - [검증 원장](validation/d045-perpendicular-finite-width-contact-qualification.md). 공개 합성 PADS V9.4 MILS SOURCE 254 nm/Tick, NET_A/NET_B Layer 1의 SMD Pads **4개**/기존 수직 Traces **4개**와 추가 **수평 42 mil Bridge
    Trace** `(224,350)→(236,350)` mil을 검증. 중심선 간 최소거리 **24 mil**, 반폭 합 **26 mil**이므로 정확한 양의 면적 Round-Cap Copper 접촉만 인정: **9 Nodes / 10 same-layer Candidate Edges / 1 Local Component**, 교차 Layer 실제 도통 증거 0.
    Rust `i128` 및 Native Qt `qint64`의 독립 최소거리 제곱 검증, SOURCE Net/Layer/Pad/Owner/Pin-Pair/Trace Inventory 및 SHA-256-bound Receipt. 반전·CRLF·NET_B·지원 Header·탭을 검증하며 **38 mil 접선 / 36 mil Gap / y=361 mil AABB
    Corner 오판 / Wrong Layer·Owner / 누락·중복·위조 SOURCE / Stale SHA / Forged Receipt**는 Fail-Closed. D_044 Native Qt의 Corner Gap·48 mil Gap·Wrong Layer·Owner·Missing SOURCE 음성 회귀도 실제 Qt 재Import·Reconstruction을
    실행하도록 수정, Rust CLI의 거부만으로 Qt 독립 성공을 선언하지 않음. **최종 코드 HEAD** [`5d66d20e93eeb6ceffc84e217f30d786c1a4f240`](https://github.com/july-rust/BoardSI/commit/5d66d20e93eeb6ceffc84e217f30d786c1a4f240/checks)
    **34/34 PR Check Runs
  - 11/11 PR Workflows SUCCESS**, 동일 HEAD Codex `completed/findings=false/unresolved=false`, 미해결 Thread **0**. 초기 Rustfmt 및 Clippy CRLF 단일문자 경고 실패는 최종 HEAD에서 수정하고 새 CI로 재검증. 보호 **Verified 단일 부모 Squash**
    [code-main `99695309b1481c03a7e20480a5e97aa4677b18d6`](https://github.com/july-rust/BoardSI/commit/99695309b1481c03a7e20480a5e97aa4677b18d6), sole parent `0921df4c035ebb3643f4772d692b1fa69acdcfa0`,
    REST verified=true/reason=valid, 정확한 code-main ref 확인. **코드-main 독립 Push CI, 별도 Markdown-only 문서 PR 검증·서명 병합 및 docs-main Push의 최종 결과는 [Issue #478](https://github.com/july-rust/BoardSI/issues/478)의 최종
    증거 댓글로 별도 판정**하며 PR 통과를 Push 성공으로 소급하지 않는다. Issue #478은 코드 병합 시 **2026-11-06T03:36:37Z**에 자동 CLOSED 처리되어 문서 인수 시점과 다르므로 최종 댓글에 시간 순서를 기록한다. **SOURCE_LOCAL_FINITE_WIDTH_ONLY** 한정: 실제 제조 Copper/전역
    Electrical Open/Short/임의 PCB/Via/Pour/Thermal/P009·PH-08 Return Path/Physical Scenario·Cache/RLGC·S Solver/U01–U48·T01–T54/제품 출시 **NOT_VERIFIED**.
    [부모 Issue #12](https://github.com/july-rust/BoardSI/issues/12)는 OPEN. 재귀 문서 PR 금지.

<a id="d_046"></a>
### D_046 — PADS Same-Layer Oblique Finite-Width SOURCE Contact & Fail-Closed Qualification (코드·main Push 한정 인수 완료 · 2026-11-06 KST)

- [x] **D_046 비직교·비평행 SOURCE Trace의 Round-Cap 선폭 접촉 후보 — Rust·Native Qt·코드-main 검증 완료.** · **체크일: 2026-11-06 KST** · **근거일: 2026-11-06 KST**
  - **최초 체크일: 2026-11-06 KST**
  - . [Issue #481](https://github.com/july-rust/BoardSI/issues/481)
  - [코드 PR #482](https://github.com/july-rust/BoardSI/pull/482)
  - [D_046 독립 검증 원장](validation/d046-oblique-finite-width-contact-qualification.md).
  - **지원 사례:** 공개 합성 PADS V9.4 MILS (254 nm/SOURCE Tick), Layer 1, NET_A/NET_B, SMD Pad 4개·기존 직선 수직 Trace 4개·추가 Oblique Bridge 1개 (224,350)→(236,357) mil, 42 mil 폭. 중심선은 접촉하지 않고 좌측 상·하 및 우측 하부 24 mil, 우측
    상부 25 mil의 최소거리에서 반폭 합 26 mil로 양의 면적 Copper 후보 4개 생성. **9 Nodes / 10 Same-Layer Candidate Edges / 1 Local Component / 0 proven electrical Cross-Layer Edges.**
  - **기하 및 보안 경계:** Rust SOURCE Tick i128 및 독립 Native Qt SOURCE Tick qint64의 정확한 유리수 제곱 선분 거리 비교. 평행·직교·중심선 교차는 별도 범위. 40 mil 정확 접선, 38 mil Gap, AABB Corner 오판, Wrong Net/Layer/Owner/Pin-Pair, 추가·누락·위조
    Source/Receipt, Stale SHA 및 범위 밖 Geometry를 Fail-Closed. SOURCE SHA-256 바인딩 및 실제 Qt reImport 회귀.
  - **코드 검증:** 최종 PR HEAD [ea5225fe9886b998ca153f6023cf311b85f0e501](https://github.com/july-rust/BoardSI/commit/ea5225fe9886b998ca153f6023cf311b85f0e501/checks): **34/34 PR Check Runs·11/11 pull_request
    Workflows SUCCESS**. 독립 Codex 동일 HEAD 완료/findings=false/unresolved=false, 리뷰 스레드 0. Rustfmt/Clippy 및 Rust D_046 Debug/Release 회귀 PASS; Linux Offscreen Qt **68/68 CTest PASS**, Windows x64·ARM64/macOS
    ARM64 Native Qt 전용 회귀 SUCCESS. 초기 코드 HEAD의 Rustfmt 실패는 수정된 최종 HEAD에서 새로 재검증했으며 실패 이력을 소급 성공으로 쓰지 않음.
  - **보호 병합 및 Push:** GitHub **Verified 단일 부모 Squash** [code-main 11267c7494f9d7b82a06220402b4fa158fed4c29](https://github.com/july-rust/BoardSI/commit/11267c7494f9d7b82a06220402b4fa158fed4c29), parent
    b64f2e79fefeb4b255133552add112b3c6dd4d08, REST verification.verified=true/reason=valid, main HEAD 일치. 정확 code-main SHA에서 독립 **8/8 event=push Workflows·28/28 Check Runs SUCCESS** 확인. 별도 Markdown-only 문서
    PR 자체 검증·보호 병합·docs-main Push 및 Issue 행정 종료 근거는 [Issue #481](https://github.com/july-rust/BoardSI/issues/481)의 마지막 증거 댓글에서 확인하며 재귀 문서 PR을 만들지 않음.
  - **인수 경계:** **SOURCE_LOCAL_FINITE_WIDTH_ONLY**. 제조 Copper 도통·Global Electrical Open/Short·임의 PADS Artwork/Via/Pour/Thermal·P009/PH-08 Return Path·Physical Scenario/Cache·RLGC/S Solver·U01–U48/T01–T54 및
    제품 릴리스는 **NOT_VERIFIED**. 상위 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) **OPEN** 유지.

<a id="d_047"></a>
### D_047 — PADS Same-Layer Finite-Width Trace-to-SMD Square Pad Contact & Fail-Closed Qualification (코드-main Push 한정 인수 완료 · 2026-11-06 KST)

- [x] **D_047 SOURCE-owned 동일 Layer Trace-to-SMD Square Pad 유한 선폭 접촉 및 Fail-Closed Rust·Native Qt 독립 검증** · **체크일: 2026-11-06 KST** · **근거일: 2026-11-06 KST**
  - **최초 체크일: 2026-11-06 KST**
  - . [Issue #484](https://github.com/july-rust/BoardSI/issues/484)
  - [코드 PR #485](https://github.com/july-rust/BoardSI/pull/485)
  - [검증 원장](validation/d047-trace-pad-finite-width-contact-qualification.md). 공개 합성 PADS V9.4 MILS, 254 nm/Tick, NET_A/NET_B, Layer 1, D_040 기준 4 Pad·4 Trace·2 Local Component에 SOURCE Trace
    (217,390)→(217,410) mil, 폭 8 mil을 추가. 30 mil 정사각형 U1.2 Pad (200,400) mil의 오른쪽 변 x=215 mil과 Trace 중심선 x=217 mil 사이 **2 mil**, Trace 반폭 4 mil로 양의 면적 접촉 후보 1건을 엄격히 인정: **9 SOURCE Nodes / 7 Same-Layer
    Candidate Edges / 2 Local Components / 0 Proven Electrical Cross-Layer Edges**. Rust i128·독립 Native Qt qint64 SOURCE 정수/유리수 거리 판정과 원본 SHA-256·Net/Layer/Owner/Pin-Pair/Pad Shape/Full Inventory 바인딩.
    x=219 mil 접선, x=220 mil Gap, AABB 모서리 오판, 이동 U2, Wrong Net/Layer/Owner/Pad Shape/Width, 누락·중복·변조·Stale SOURCE/Receipt는 Fail-Closed, 실제 Qt reImport로 독립 검증. 최종 코드 HEAD
    [8797e34e39813060e59cc154cf835dbb4fc87c4b](https://github.com/july-rust/BoardSI/commit/8797e34e39813060e59cc154cf835dbb4fc87c4b/checks): **34/34 PR Checks, 11/11 PR Workflows SUCCESS**, 동일 HEAD Codex
    completed/findings=false/unresolved=false 및 Review Thread 0, Linux Qt **69/69 CTest PASS**, Windows x64/ARM64·macOS ARM64 Native Qt SUCCESS. 초기 Rustfmt·근거 없는 D_040 사전 가정 테스트와 기존 macOS D_006 간헐 임시파일 실패는
    과거 이력으로 보존, D_006 근본 원인 해결로 주장하지 않음. 보호 **GitHub Verified 단일 부모 Squash**
    [code-main 2e45fc5e8ea664956fdbdf240797025b12b8b61f](https://github.com/july-rust/BoardSI/commit/2e45fc5e8ea664956fdbdf240797025b12b8b61f), parent dc84008baca4ea1ab461d828af659cc91c9c082b, REST
    verified=true/reason=valid 및 main ref 일치. 독립 정확 code-main **8/8 event=push Workflows·28/28 Checks SUCCESS**. **SOURCE_LOCAL_TRACE_PAD_FINITE_WIDTH_ONLY:** 실제 제조 도통·전체 Open/Short·임의
    PADS/Via/Pour/Thermal/비정사각·회전 Pad·P009/PH-08·Physical Scenario/Cache·RLGC/S Solver·U01–U48/T01–T54·출시 모두 NOT_VERIFIED. 상위 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. 별도 문서 PR의 경량
    CI·독립 리뷰·Verified Squash·정확 docs-main Push 및 Issue #484 최종 종료 결과는 해당 Issue 댓글에 기록하며 재귀 문서 PR은 만들지 않음.

<a id="d_048"></a>
### D_048 — PADS Same-Layer Finite-Width Trace-to-SMD Round Pad Contact & Fail-Closed Qualification (코드·main Push 한정 인수 완료 · 2026-11-07 KST)

- [x] **D_048 SOURCE-owned 동일 Layer 원형 SMD Pad–유한 선폭 Trace의 정확한 접촉 후보·Fail-Closed Rust/Native Qt 독립 검증** · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST**
  - **최초 체크일: 2026-11-07 KST**
  - . [Issue #487](https://github.com/july-rust/BoardSI/issues/487)
  - [코드 PR #488](https://github.com/july-rust/BoardSI/pull/488)
  - [독립 검증 원장](validation/d048-trace-round-pad-finite-width-contact-qualification.md). D_040 공개 합성 PADS V9.4 MILS, Layer 1, 254 nm/SOURCE Tick, NET_A/NET_B, Pad 4개·원래 Trace 4개·독립 Component 2개를 완전 바인딩하고,
    U1.1 원형 SMD Pad(중심 200,300 mil, 직경 40 mil)에 8 mil 폭 SOURCE Trace (222,285)→(222,315) mil을 추가. 중심선 최단거리 22 mil, Pad 반경 20 mil+Trace 반폭 4 mil로 **20 < 22 < 24 mil** 엄격 양의 면적 접촉 후보를 승인하되 중심선 자체는 Pad 밖이어야
    한다. **9 SOURCE Nodes / 7 Same-Layer Candidate Edges / 2 Local Components / Cross-Layer Electrical Proven 0**. Rust i128·Native Qt qint64 독립 SOURCE 정수 유리수 제곱거리·실제 Qt 재Import/Receipt·원본
    SHA-256/Owner/Pin-Pair/Net/Layer/Shape/전체 Fixture 비교. 둥근 Trace 끝단 양의 접촉·1 Tick 경계/방향 반전/NET_B/CRLF/탭은 회귀, x=224 mil 접선·x=225 mil Gap·Pad 내부 중심선·AABB 허위 접촉·Wrong
    Layer/Owner/Shape/Width·Missing/Duplicate/Modified SOURCE·Forged Receipt·Stale SHA는 Fail-Closed. **최종 코드 PR HEAD**
    [`0b381936fc2415a209096583a2f86907911a8580`](https://github.com/july-rust/BoardSI/commit/0b381936fc2415a209096583a2f86907911a8580/checks) **34/34 PR Checks
  - 11/11 PR Workflows SUCCESS**, Rustfmt/Clippy 및 D_048 Rust Debug **3/3**·Release **3/3** PASS, Linux Native Qt **70/70 CTest PASS**, Windows x64/ARM64·macOS ARM64 Native Qt SUCCESS. 독립 Codex 해당 HEAD 👍
    반응·미해결 리뷰 Thread 0; 초기 Rustfmt 실패는 최종 HEAD에서 수정·재검증했고 이전 실패 이력을 성공으로 소급하지 않는다. 보호 **GitHub Verified 단일 부모 Squash**
    [code-main `19dcfe92af17392d2677ad02cf07c818c03266ee`](https://github.com/july-rust/BoardSI/commit/19dcfe92af17392d2677ad02cf07c818c03266ee), parent `884d7b522198612bd7a394a447db6ca115d877bc`, GitHub
    REST verified=true/reason=valid, main 참조 일치. 정확한 코드-main **8/8 event=push Workflows
  - 28/28 Checks SUCCESS**. **SOURCE_LOCAL_TRACE_ROUND_PAD_FINITE_WIDTH_ONLY**: 실물 제조 도통, 전역 Open/Short, 임의 PCB·복합 Pad/Via/Pour/Thermal, P009/PH-08 Return Path, Physical Scenario/Cache, RLGC/S Solver,
    U01–U48/T01–T54 및 제품 릴리스는 **NOT_VERIFIED**. 상위 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN. 별도 **Markdown-only 문서 PR의 CI·검토·Verified Squash·docs-main Push 및 Issue #487 최종 종료는
    Issue의 사후 증거 기록에서 판정**하며 재귀 문서 PR 금지.


<a id="d_049"></a>
### D_049 — PADS Same-Layer Mixed Round/Square SMD Pad Dual Finite-Width SOURCE Contact & Fail-Closed Qualification (코드-main 보호 병합 한정 검증 · 2026-11-07 KST)

- [x] **D_049 SOURCE-owned Round Pad + Square Pad 두 접촉의 동일 Layer 국소 Copper Geometry Graph 통합 한정 검증** · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST**
  - **최초 체크일: 2026-11-07 KST**
  - . [Issue #490](https://github.com/july-rust/BoardSI/issues/490)
  - [코드 PR #491](https://github.com/july-rust/BoardSI/pull/491)
  - [D_049 검증 원장](validation/d049-mixed-round-square-pad-contact-qualification.md). 공개 합성 PADS V9.4 MILS SOURCE 254 nm/Tick, NET_A/NET_B, Layer 1에서 D_040의 4개 SMD Pad·4개 기준 Trace·2개 독립 Component에 **폭 8 mil
    신규 Trace 두 개**를 동시에 추가: U1.1 원형 Pad(200,300) mil/지름 40 mil에 (222,285)→(222,315) mil, U1.2 정사각형 Pad(200,400) mil/변 30 mil에 (217,390)→(217,410) mil. 엄격한 양의 면적 Copper 후보 2개와 기존 6개 Edge로 **10 SOURCE Nodes
    / 8 Same-Layer Candidate Edges / 2 Local Components / 0 Proven Cross-Layer Electrical Edges**. Rust는 D_047/D_048의 완전 SOURCE·정확한 기하 인수를 독립 재검증하고 단일 SHA-256-bound 통합 Receipt를 새로 구성; Qt는 실제
    PreviewImportPanel reImport, ImportedArtwork의 4 Pad/6 Trace 및 서로 다른 Pad 형상별 정확한 간격·소유권·Net/Layer/Width/Pin-Pair·Receipt를 독립 재구성. Rust와 Qt에서 양쪽 접촉 동시 성공·방향 반전·Round-Cap 접촉·NET_B·CRLF·탭 및 지원된 SOURCE 숫자
    표기, 접선·Gap·AABB 허위 교차·Wrong Net/Layer/Owner/Shape/Width·누락/중복·위조 Receipt·Stale Source를 Fail-Closed 확인. 과거 Rustfmt 수정, Qt 헤더 누락 및 Codex P2 SOURCE 숫자 표기 정합성 문제, Windows CRLF Fixture 문자열 변형 실패는 후속 HEAD에서
    수정·재검증했고 과거 실패를 소급 성공으로 기록하지 않는다. 최종 [코드 PR HEAD `20f668f27dc2b56f45b0e020f06e2645dca1b4e1`](https://github.com/july-rust/BoardSI/commit/20f668f27dc2b56f45b0e020f06e2645dca1b4e1/checks) **11/11
    pull_request Workflows SUCCESS**, Rustfmt·Clippy·Rust Debug/Release, Native Rust Windows x64/ARM64/macOS ARM64 SUCCESS, Linux Qt **71/71 CTest PASS**, Windows x64/ARM64·macOS ARM64 Native Qt SUCCESS.
    동일 HEAD 독립 Codex 완료/findings=false/unresolved=false, 리뷰 스레드 0. 보호된
    [Verified 단일 부모 Squash code-main `8c69df61b605fe10b464fd87bedff81348d2b096`](https://github.com/july-rust/BoardSI/commit/8c69df61b605fe10b464fd87bedff81348d2b096) (parent
    `2e70337842471b2c81759ee7696994abf442eaf0`), REST verification.verified=true/reason=valid, 정확한 main ref 일치. 정확 code-main **event=push** 검사 및 문서 전용 검증·보호 병합/정확 docs-main Push는
    [Issue #490](https://github.com/july-rust/BoardSI/issues/490) 최종 증거 댓글에서 *별도로* 확인·판정한다. **SOURCE_LOCAL_MIXED_PAD_DUAL_FINITE_WIDTH_ONLY**: 실제 제조 Copper 도통, Global Electrical Open/Short, 임의
    PCB/Plane/Via/Pour/Thermal, P009·PH-08 Return Path, Physical Scenario/Cache, RLGC/S Solver, U01–U48/T01–T54 및 제품 릴리스는 **NOT_VERIFIED**. [상위 Issue #12](https://github.com/july-rust/BoardSI/issues/12)는
    **OPEN**. 문서 전용 변경은 위 두 Markdown 파일만 포함하고 불필요한 Native 플랫폼 재빌드를 실행하지 않으며, 재귀적인 문서 PR을 만들지 않는다.


<a id="d_050"></a>
### D_050 — PADS Same-Layer Mixed Round/Square SMD Pad Finite-Width Inter-Component Bridge & Fail-Closed Qualification (코드·main Push 한정 인수 완료 · 2026-11-07 KST)

- [x] **D_050 SOURCE-owned 동일 Layer 원형·정사각형 SMD Pad 혼합 국소 Component 사이의 유한 선폭 Bridge 접촉 후보 — Rust·독립 Native Qt·보호 code-main Push 한정 인수 완료** · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST**
  - **최초 체크일: 2026-11-07 KST**
  - . [Issue #493](https://github.com/july-rust/BoardSI/issues/493)
  - [코드 PR #494](https://github.com/july-rust/BoardSI/pull/494)
  - [D_050 검증 원장](validation/d050-finite-width-intercomponent-bridge-qualification.md). 공개 합성 PADS V9.4 MILS SOURCE 254 nm/Tick, Layer 1, 동일 NET_A(및 NET_B 지원 변형)에서 D_049의 4 Pad·6 Trace·2 분리 Component를 보존하고
    폭 8 mil의 명시적 새 Trace (226,300)→(378,300) mil 한 개를 추가한다. 이 Bridge는 U1.1의 기존 x=222 mil, 폭 8 mil 수직 보조 Trace와 4 mil 중심선 최소거리(합 반폭 8 mil)에서, U2.1 지름 40 mil 원형 Pad와 중심선 끝점 최소거리 22 mil(반경+반폭 24 mil)에서 각각
    **엄격한 양(+)의 Copper 면적 접촉 후보**를 가진다. 불필요한 직접 Pad 접촉이나 다른 Copper 접촉은 인수하지 않으며 **11 SOURCE Nodes(4 Pad/7 Trace) / 10 Same-Layer Candidate Edges / 1 Local Component / Proven Cross-Layer Electrical Edges
    0**을 확인한다. Rust는 SOURCE 전체 Inventory·D_049 감소 Fixture 재검증·정확한 두 접촉·원본 SHA-256을 바인딩한 통합 Receipt를 구성하고, Native Qt는 실제 PreviewImportPanel reImport의 ImportedArtwork에서 독립적으로 재구성하여 동일 Receipt와 Fail-Closed를
    검증한다. 방향 반전/지원 숫자 표기/CRLF/탭/NET_B 변형을 허용하고, 접선·Gap·AABB 허위 접촉·소유권·Pin-Pair·NET·Layer·Width·Shape 오류, 양쪽 접촉 중 어느 하나의 실패, SOURCE 누락·중복·추가·위조·Stale SHA/Receipt 및 혼합 NET을 거부한다. 초기 HEAD의 Rustfmt, Windows
    CRLF Fixture Mutation, Qt 선행 헤더 의존 오류는 수정된 최종 HEAD에서 재검증했으며 과거 실패는 소급 성공으로 보지 않는다. 최종
    [코드 PR HEAD `880f63b358de86f2f71d06e57e212711b0970c8f`](https://github.com/july-rust/BoardSI/commit/880f63b358de86f2f71d06e57e212711b0970c8f/checks): **11/11 pull_request Workflows·34/34 Checks
    SUCCESS**, Linux Native Qt **72/72 CTest PASS**(D_050 전용 진단 PASS), Windows x64/ARM64·macOS ARM64 Native Qt SUCCESS, Rustfmt/Clippy/Debug/Release SUCCESS. 정확한 HEAD Codex 독립 리뷰
    completed/findings=false/unresolved=false, 미해결 Review Thread 0. 보호된 **Verified 단일 부모 Squash
    [code-main `f1261fd7b8c8c1292ab1b25d6e54d590eadc9192`](https://github.com/july-rust/BoardSI/commit/f1261fd7b8c8c1292ab1b25d6e54d590eadc9192)**, 단독 parent `60facdc4015fd57a6e4b9eef58fbb9e6fe284b7f`,
    GitHub verification.verified=true/reason=valid 및 main ref 일치. 이 정확한 code-main SHA의 독립 **event=push 8/8 Workflows·28/28 Checks SUCCESS**를 확인했다. 별도 Markdown-only 문서 PR의 자체 CI·Codex·Verified Squash·정확
    docs-main Push는 Issue #493의 최종 증거 댓글로 별도 기록하며 재귀 문서 PR을 만들지 않는다. **SOURCE_LOCAL_MIXED_PAD_FINITE_WIDTH_BRIDGE_ONLY**: 제조 Copper 도통, Global Electrical Open/Short, 임의 PCB·Via·Plane·Pour·Thermal,
    P009/PH-08 Return Path, Physical Scenario/Cache, RLGC/S Solver, U01–U48/T01–T54 정식 인수 및 출시 모두 **NOT_VERIFIED**. 상위 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN 유지.

<a id="d_051"></a>
### D_051 — PADS Same-Layer Mixed Pad Dual Finite-Width Bridge Redundant Loop & Virtual Single-Bridge Failure Fail-Closed Qualification (코드-main 보호 병합 한정 · 2026-11-07 KST)

- [x] **D_051 공개 합성 PADS V9.4 SOURCE-Layer 1 혼합 Pad 두 Bridge의 중복 국소 경로 및 가상 단선 한정 검증** · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST**
  - **최초 체크일: 2026-11-07 KST**
  - . [Issue #497](https://github.com/july-rust/BoardSI/issues/497)
  - [코드 PR #498](https://github.com/july-rust/BoardSI/pull/498)
  - [검증 원장](validation/d051-dual-finite-width-bridge-loop-qualification.md). D_050의 원형·정사각형 Pad 4개 및 Trace 7개에 폭 8 mil 신규 직선 Trace `U1.2 U2.2` (222,400)→(382,400) mil을 추가하고, 기존 x=217 mil 수직 보조 Trace와 U2.2
    변 30 mil SMD 정사각형 Pad에 **엄격한 양의 Copper 면적 접촉 두 개**를 SOURCE Tick=254 nm 정수 기하로 검증. 12 Nodes(4 Pads/8 Traces) / 12 Same-Layer Candidate Edges / 1 Local Component / Cycle Rank 1, 각 Bridge 단독 가상 제거 시 11
    Nodes/10 Edges/1 Component/0 Cycle, 양 Bridge 제거 시 10 Nodes/8 Edges/2 Components/0 Cycle; 원본 전체 재구성·SOURCE SHA-256 결합 Receipt·Forged/Stale 및 접선/Gap/Wrong Net/Layer/Owner/Shape/Width·추가·누락·중복
    Fail-Closed. Rust와 실제 Qt PreviewImportPanel의 전체·감소 SOURCE 독립 재Import를 검증하고 기존 D_039–D_050 회귀를 보존함. 최종
    [코드 PR HEAD `c99415758e1a8e593eb384f81a64644074628047`](https://github.com/july-rust/BoardSI/commit/c99415758e1a8e593eb384f81a64644074628047/checks) **34/34 PR Checks SUCCESS**, Linux Qt **73/73 CTest
    PASS**, Native Windows x64/ARM64·macOS ARM64 CI SUCCESS, 정확한 HEAD Codex `completed/findings=false/unresolved=false`, 미해결 Review Thread 0. 보호된 단일 부모 Verified Squash
    [코드-main `28a6f2b43e49a06e4eff0da1d1e41a46eb0d8b92`](https://github.com/july-rust/BoardSI/commit/28a6f2b43e49a06e4eff0da1d1e41a46eb0d8b92), parent `7300bc3a759d41f858996a163302ab42e3061e9a`,
    verification.verified=true/reason=valid 및 코드 병합 직후 main ref 일치 확인. 별도 정확한 코드-main **event=push 검사 및 문서 전용 PR/Push 최종 인수는 [Issue #497](https://github.com/july-rust/BoardSI/issues/497)에서 실제 결과를 별도로 기록하여
    판정한다**. 과거 Rustfmt·Windows CRLF 테스트·Qt 타임아웃 수명 오류는 후속 HEAD에서 수정하고 재검증했으며 과거 실패를 소급 성공 처리하지 않는다. **SOURCE_LOCAL_DUAL_FINITE_WIDTH_REDUNDANCY_ONLY**: 제조 Copper 실제 도통, 글로벌 Electrical Open/Short, 임의 PCB,
    Via/Plane/Pour/Thermal, P009/PH-08, Scenario/Cache, RLGC/S, U/T 정식 인수 및 출시 모두 **NOT_VERIFIED**. 상위 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) **OPEN** 유지. 문서 전용 변경은 이 체크리스트 및 D_051 검증
    원장 Markdown 두 개만 허용하며 Native Runner 빌드를 실행하지 않고 재귀 문서 PR을 만들지 않는다.

<a id="d_052"></a>
### D_052 — PADS Same-Layer Dual Finite-Width Bridge Midspan Copper Break & Fail-Closed Qualification (코드-main 보호 병합 한정 · 2026-11-07 KST)

- [x] **D_052 공개 합성 PADS SOURCE의 실제 Midspan Copper Break 4상태 Rust·Native Qt 독립 국소 연결성 검증** · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST**
  - **최초 체크일: 2026-11-07 KST**
  - . [Issue #500](https://github.com/july-rust/BoardSI/issues/500)
  - [코드 PR #501](https://github.com/july-rust/BoardSI/pull/501)
  - [D_052 근거 원장](validation/d052-dual-bridge-midspan-break-qualification.md). D_051의 PADS V9.4 MILS/Layer 1/254 nm SOURCE Tick, 혼합 원형·정사각형 SMD Pad 4개·기본 8개 Trace·폭 8 mil Bridge A/B를 보존하고 실제 ASCII SOURCE의
    각 Bridge 중간을 2개의 명시적 선분으로 분할. A: (226,300)→(290,300)과 (310,300)→(378,300) mil, B: (222,400)→(290,400)과 (310,400)→(382,400) mil. 끝점 중심선 20 mil 간격, 둥근 끝단 고려 양의 Copper Separation 12 mil, 접선/겹침 경계 엄격 거부.
    **INTACT=12 Nodes/12 Edges/1 Component/Rank 1**, **BREAK_A/B=13/12/1/0**, **BREAK_BOTH=14/12/2/0**, 두 Bridge 모두 SOURCE에서 끊긴 경우에만 원래 두 Component 간 분리. 실제 SOURCE Inventory/Pin Pair/Pad
    Shape/Width/Layer/Net/Owner 일대일 바인딩·SOURCE SHA-256/Full Receipt·접선/Overlap/Extra/Missing/Duplicate/수정·Forged/Stale Fail-Closed. Rust는 D_051 기준 SOURCE 재자격화와 별도로 분할된 실제 SOURCE에 대한 DSU Graph/엄격 간격 검증을
    수행하고, Native Qt는 실제 PreviewImportPanel 복수 reImport와 ImportedArtwork/Receipt 독립 재구성으로 검증. 최초 Rustfmt 및 Windows CRLF 변형 테스트 실패는 수정한 최종 HEAD에서 재검증하였으며 과거 실패를 PASS로 소급하지 않음.
    [코드 PR 최종 HEAD 15d672c2d9d7bbc8e33d32786c5c062788c8246a](https://github.com/july-rust/BoardSI/commit/15d672c2d9d7bbc8e33d32786c5c062788c8246a/checks) **34/34 PR Checks·11/11 PR Workflows SUCCESS**,
    Rustfmt/Clippy/Rust Debug·Release, Native Rust Win x64/ARM64·macOS ARM64, Native Qt Linux **74/74 CTest PASS** 및 Win x64/ARM64·macOS ARM64 PASS. 동일 HEAD 독립 Codex
    completed/findings=false/unresolved=false 및 미해결 리뷰 0. 보호된 **Verified 단일 부모 Squash
    [code-main 310563cc375a899c96800136a19eed5ae93f667b](https://github.com/july-rust/BoardSI/commit/310563cc375a899c96800136a19eed5ae93f667b)**, parent f6e93642d60ea46f2c3f418999fcb1b11bf39411, GitHub
    REST verified=true/reason=valid, 병합 직후 정확한 main ref 일치. 독립 **code-main event=push 검사 및 docs-only PR/Push 최종 판정은 [Issue #500](https://github.com/july-rust/BoardSI/issues/500)의 별도 사후 근거**로 남기며 미완료 검사 결과를
    PASS로 선기록하지 않음. **SOURCE_LOCAL_DUAL_BRIDGE_MIDSPAN_BREAK_ONLY:** 실물 PCB Copper 실제 도통·전체 전기 Open/Short, 임의 PCB, Via/Plane/Pour/Thermal, P009/PH-08 Return Path, Physical Scenario/Cache, RLGC/S Solver, 정식
    U/T 인수 및 제품 릴리스 모두 **NOT_VERIFIED**. 상위 [Issue #12](https://github.com/july-rust/BoardSI/issues/12) **OPEN** 유지. 문서 PR은 이 체크리스트 및 D_052 검증 원장 Markdown 두 개만 수정, Docs-only CI 사용, Native 플랫폼 빌드 제외 및 재귀 문서
    PR 금지.

<a id="d_053"></a>
### D_053 — PADS Same-Layer Dual Finite-Width Bridge SOURCE Midspan Copper Reconnection & Fail-Closed Qualification (한정 코드 검증 · 2026-11-07 KST)

- [x] **D_053 공개 합성 PADS V9.4 SOURCE 실제 Repair Trace 6상태 Rust·Native Qt 독립 국소 접촉·연결성 검증** · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST**
  - **최초 체크일: 2026-11-07 KST**
  - . [Issue #503](https://github.com/july-rust/BoardSI/issues/503)
  - [코드 PR #504](https://github.com/july-rust/BoardSI/pull/504)
  - [검증 원장](validation/d053-dual-bridge-midspan-reconnection-qualification.md). D_052의 PADS V9.4 MILS, 254 nm SOURCE tick, Layer 1, 혼합 원형/정사각형 SMD Pad 4개, 기본 Trace 8개 및 중간 단선 두 개를 기반으로 폭 8 mil의 SOURCE 작성
    Repair A (288,300)→(312,300) mil 및 Repair B (288,400)→(312,400) mil을 **각각 별도 Trace 레코드**로 보존. 실제 SOURCE 재Import와 양쪽 양의 Copper 면적 중첩, Pad/Trace 1:1 Inventory, 독립 DSU 그래프 및 SHA-256 원본 결합 Receipt를 Rust와
    Native Qt에서 검증. **BREAK_BOTH N14/E12/C2/cycle0; REPAIR_A_ONLY, REPAIR_B_ONLY N15/E14/C1/cycle0; REPAIR_BOTH N16/E16/C1/cycle1; BREAK_A_REPAIR_A, BREAK_B_REPAIR_B N14/E14/C1/cycle1.**
    접선/간격/Owner/Shape/Layer/Width/Net/Extra/Duplicate/Forged/Stale 및 1-tick 양의 접촉을 Fail-Closed 회귀 시험. 코드 PR 최종 HEAD
    [944cd9d33dbf35d0354a957217f6c29d76b56c48](https://github.com/july-rust/BoardSI/commit/944cd9d33dbf35d0354a957217f6c29d76b56c48/checks): **34/34 PR Checks, 11/11 PR workflows SUCCESS**,
    Rustfmt/Clippy/Rust Debug/Release, Linux Qt **75/75 CTest PASS**, Windows x64/ARM64 및 macOS ARM64 Rust·Native Qt PASS; 동일 HEAD Codex `completed/findings=false/unresolved=false`, 미해결 Review Thread 0.
    보호된 단일 부모 Verified Squash [코드-main 2cbaac6af8845b89c087266ddcecab2824c75fa6](https://github.com/july-rust/BoardSI/commit/2cbaac6af8845b89c087266ddcecab2824c75fa6), parent
    30d46fa2f81a9e5ed6516aa667c1f59233553b08, GitHub verification.verified=true/reason=valid. 독립적인 **code-main event=push 검증과 docs-only PR·Push 결과는
    [Issue #503](https://github.com/july-rust/BoardSI/issues/503)에서 실제 종료 결과를 기록**하며 미완료 검사를 PASS로 선기록하지 않음. **SOURCE_LOCAL_DUAL_BRIDGE_REPAIR_ONLY:** 실물 PCB Copper 도통, 전역 Electrical Open/Short, 임의 PCB,
    Via/Plane/Pour/Thermal, P009/PH-08 Return Path, Physical Scenario/Cache, RLGC/S Solver, 정식 U/T 및 제품 Release **NOT_VERIFIED**. Parent [Issue #12](https://github.com/july-rust/BoardSI/issues/12)
    **OPEN**. 문서 변경은 이 체크리스트와 D_053 근거 원장 Markdown 두 파일로 제한; Docs-only Native 작업 SKIP, 문서 PR 재귀 생성 금지.

<a id="d_054"></a>
### D_054 — 범용 임의 각도 직선·원호 Trace SOURCE 국소 Copper 접촉 (한정 코드 검증 · 2026-11-07 KST)

- [x] **직선 모든 방향(0–359° 샘플 회귀는 검증용이며 각도 분기 구현이 아님)과 방향성 원호 CW/CCW, 유한 선폭 SOURCE 국소 접촉 공통 계산 및 Fail-Closed 검증** · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST**
  - **최초 체크일: 2026-11-07 KST**
  - . [Issue #506](https://github.com/july-rust/BoardSI/issues/506)
  - [코드 PR #507](https://github.com/july-rust/BoardSI/pull/507)
  - [코드 main 45878db](https://github.com/july-rust/BoardSI/commit/45878db712f2786c0852c2fb2b0153b584a4ce29)
  - [D_054–D_055 통합 증거](validation/d054-d055-general-trace-contact-qualification.md). 10개 한정 신규 Rust 테스트 Debug/Release PASS 및 필수 PR Workflow PASS, 독립 리뷰의 근접 접선 AABB 오진 수정 후 회귀 검증. 국소 SOURCE geometry-only,
    P009/전체 전기 연결·SI Solver/Qt 글로벌 그래프 공식 인수 **NOT_VERIFIED**.

<a id="d_055"></a>
### D_055 — Via 비중심 Annulus · SMD Pad 전체 면적 공통 Trace 접촉 (한정 코드 검증 · 2026-11-07 KST)

- [x] **Trace 중심/종점의 Via·Pad 중심 일치 가정을 제거한 재사용 가능한 Copper 겹침 분류 및 SOURCE Net/Layer/드릴/Shape/접선 Fail-Closed 검증** · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST**
  - **최초 체크일: 2026-11-07 KST**
  - . [Issue #508](https://github.com/july-rust/BoardSI/issues/508)
  - [코드 PR #509](https://github.com/july-rust/BoardSI/pull/509)
  - [코드 main 983f2b8](https://github.com/july-rust/BoardSI/commit/983f2b871d9a96b361899573207ca3a89c529c27)
  - [D_054–D_055 통합 증거](validation/d054-d055-general-trace-contact-qualification.md). 원본 PADS V9.4 MILS 실제 SOURCE 변형 2개(중심에서 벗어난 Via 접속, 정사각형 Pad 코너 접속) 포함 신규 Rust 10개 Debug/Release PASS. 필수 PR 9 Workflow
    모두 SUCCESS, Rust Linux/Windows x64/ARM64/macOS ARM64와 기존 Qt Linux/Windows x64/ARM64/macOS ARM64 회귀 PASS. Via 드릴 구멍만 중첩되면 접촉 미인정; Via의 명시적 도체 레이어만 인정; SMD R/S/RF Pad의 전체 구리 영역 접촉; 미확인 Net 및 서로 다른 Net은
    연결 확정이 아닌 별도 후보. 현재 opt-in \`--trace-pad-via-contacts-v1\`의 **SOURCE_LOCAL_ONLY**이며 Qt의 실제 전체 Connectivity/Port/SI, 제조 Copper 도통, 임의 형상·전 PADS 리비전, P009/PH-08 및 U/T 공식 인수는 **NOT_VERIFIED**. Docs-only
    PR 확인과 main Push 별도 인수는 실제 결과 확보 전 PASS로 단정하지 않음.

<a id="d_056"></a>
### D_056 — 범용 PCB 입력 정규화 및 실제 층수·Span 기반 Via 구조 (진행 중 · 2026-11-08 KST)

- [ ] **범용 입력 구조 최종 인수 미완료**
  - **최초 체크일: 2026-11-08 KST**
  - [Issue #520](https://github.com/july-rust/BoardSI/issues/520)
  - [구현 범위와 미완료 게이트](architecture/input-normalization.md). 첫 코드 단계는 별도의 비아 종류별 알고리즘을 나열하지 않고 `NormalizedVia`에 전체 Copper Layer 수, 실제 Start/End Layer, 명시적 ViaKind, Drill/Plating, Layer별 Copper Pad를 데이터로 전달하고
    검증한다. 2/4/6/8/64층 Through와 L2–L4·L2–L3 Buried, L1–L3·L3–L6 Blind, 인접층 Microvia를 **정규화 수준에서** 시험 대상으로 추가했으며 기존 PADS 제한 Through-Via Padstack 생성 경로에 공통 정규화기를 연결했다. **코드 첫 단계 검증 완료:**
    [PR #521](https://github.com/july-rust/BoardSI/pull/521) HEAD `ed3bbbe15d0934fb7b31db9e3e9ef606c5efb697`에 대해 **30/30 Checks
  - 10/10 Workflow SUCCESS** (Rust fmt/Clippy 및 Workspace Debug/Release, Native Rust/Qt, 문서·A032·M2 회귀), 보호된 단일 부모 Verified Squash
    [main `288b16e`](https://github.com/july-rust/BoardSI/commit/288b16e19054102e752d433ab60088e669ea235a) REST verification.verified=true/reason=valid 및 main ref 일치 확인. 독립 code-main Push 증거는 별도 검증 필요.
    **추가 코드 범위:** [PR #523](https://github.com/july-rust/BoardSI/pull/523)에서 PADS V9.4 공식 `drill_start/drill_end` 기반 부분 비아(Through/Blind/Buried)와 층별 **R 원형** 패드 정보를 opt-in 전용 경로로 해석하며 L2–L3/L2–L4,
    2/4/6/8층, 원본 선언과 모순되는 층·중복·미지원 패드에 Fail-Closed 테스트를 추가했다. 기존 Qt V1–V14 출력과 검증된 전기 연결 그래프는 그대로 유지하며, source-generalized via가 레거시 원형·4층 가정으로 허위 연결 처리되지 않도록 보호했다. **두 번째 코드 단계 검증 완료:**
    [PR #523](https://github.com/july-rust/BoardSI/pull/523) 최종 HEAD `b113e8a41b96d8d87e17f8d7cb6a91f9eddb8918`에 대해 **30/30 Checks
  - 10/10 PR Workflows SUCCESS**, Rust fmt/Clippy 및 Workspace Debug/Release/Native Rust Windows x64/ARM64·macOS ARM64, Qt Native Linux/Windows x64/ARM64·macOS ARM64, Docs·A032·M2 회귀. Codex P2의 동일 비아명
    Through/Buried 중복 정의 우회 지적을 수정한 최종 HEAD에서 Source Fail-Closed 회귀 테스트 추가 및 리뷰 Thread 해결 0건 잔여. 보호된 Verified 단일 부모 Squash
    [main `63183cb`](https://github.com/july-rust/BoardSI/commit/63183cbcf94e30134c08aee795b51ae9057bdc08), GitHub verification.verified=true/reason=valid 및 코드 main ref 일치 확인. 해당 Code-main Push는 PR-head와
    별도 감사한다. **미구현/미확인:** PADS RT/ST Thermal(Spoke)/비도금/Backdrill 및 미지원 비표준 Via 형상, Microvia 제조 공정 유형 증거, 실제 벤더 Export 상호 검증, Qt 다중 Layer 패드 UI/IPC, 기존 Reference/FEM 경로의 새 패드 형상 완전 정합, Canonical 전체 전기
    Connectivity와 Viewer/Solver, ODB++/IPC-2581/Gerber/KiCad의 독립 입력 검증 및 전체 입력 아키텍처 감사. 이러한 부분이 검증되기 전에는 D_056을 [x]로 변경하거나 전체 PCB 지원을 주장하지 않는다.

**세 번째 코드 단계 한정 인수 — 2026-11-08 KST:** [PR #526](https://github.com/july-rust/BoardSI/pull/526) 정확한 최종 HEAD `2f65d08321d183f4a446163f7b05705d5b5fb7f9`에서 **38/38 Checks · 13/13 PR Workflows SUCCESS**, 자동 코드
  리뷰 `completed/findings=false/unresolved=false`, 미해결 리뷰 Thread 0. PADS V9.4 R 원형/S 사각형/A 외경+별도 내경 환형은 `NormalizedViaShapes`와 Canonical `PadShape::{Circle,Rectangle,Annulus}`로, RA/SA는 전도성 Copper가 아닌
  `PadRole::AntiPad`로 정규화. Shared SOURCE-local 후보 엔진은 Square 드릴 구멍과 Annular 별도 Inner Void를 제외하며 RT/ST Thermal Spoke를 Fail-Closed한다. CLI 단독 Via 진단 V2는 Source 형상별 Layer 정보를 출력하며 기존 Qt V1–V14는 변경하지 않는다. 검증된
  보호된 단일 부모 **Verified Squash** `main` [`7e10d7b`](https://github.com/july-rust/BoardSI/commit/7e10d7ba07d6c9e0f959eb3e2e4be570af6dd88b) 반영(verification.verified=true/reason=valid). **PR-head CI와 코드-main
  Push CI는 별도 증거이며 Push CI 종료 전 PASS로 기재하지 않는다.**

**2026-11-09 D_056_06 후속 정정:** 위 최초 Stage 1/2에서 기술한 RT/ST 미구현·Qt 미통합은 해당 Stage의 날짜 고정 이력이다. 현재 Source-scope RT/ST Thermal/Qt 코드 구현은 [PR #553](https://github.com/july-rust/BoardSI/pull/553)으로 추가되었으며, 보호
  Squash SHA·정확한 PR CI·독립 Push 감사 현황은 [D_056_06](#d_056_06)에 별도로 기록한다. D_056 전체·타 CAD·제조 도통의 인수는 여전히 OPEN이다.

**D_056 부모 상태:** [Issue #520](https://github.com/july-rust/BoardSI/issues/520) OPEN, 통합 원장 이 항목 **[ ] 유지**. Native RT/ST 형상, Qt Layer별 Via UI/IPC, 글로벌 Source 기반 전체 전기 Connected Layers/Short/Open, 다중 포맷 입력
  어댑터 및 Vendor Export 독립 실물 검증은 제품 인수 요건으로 남음. 부분 SOURCE Geometry 처리 완료를 전체 PCB 입력/실물 SI 완료로 승격하지 않는다.


<a id="d_056_01"></a>
### D_056_01 — Existing Implementation Audit & Universal PCB Source Evidence Contract (2026-11-08 KST)

- [x] **D_056_01 기존 구현 감사·범용 PCB Source Evidence Contract 수립 (문서·CI 한정 완료)** · **체크일: 2026-11-08 KST** · **근거일: 2026-11-08 KST**
  - **최초 체크일: 2026-11-08 KST**
  - . 보호 `main@4118e2c0a525ba732e16ea9062d1b22e05c2af30`의 [코드별 구현 감사·Source/Layer/Padstack/Plating 증거 계약·음성/양성 테스트 매트릭스](architecture/d056-01-source-evidence-audit.md) 및
    [범용 입력 설계](architecture/input-normalization.md). [문서 PR #533](https://github.com/july-rust/BoardSI/pull/533) 최종 HEAD `80e05fd7cae211bd1345ca3c568d06710eb75824`에서 **29/29 Check runs 완료(13 SUCCESS/16
    SKIPPED), PR Workflow 8/8 SUCCESS, 미해결 Review Thread 0**, docs-only 분류 정상·Native 관련 무관 Runner SKIPPED. **보호·Verified 한 부모 Squash**
    [main `bd4c592`](https://github.com/july-rust/BoardSI/commit/bd4c592156c05cf56c94c6abcc9e0c1dc42bf059) (verification.verified=true/reason=valid, parent 1) 및 동일 SHA의 독립 main Push **8/8 Workflow
    SUCCESS** 확인. [부모 #520](https://github.com/july-rust/BoardSI/issues/520)와
    [#529](https://github.com/july-rust/BoardSI/issues/529)/[#530](https://github.com/july-rust/BoardSI/issues/530)/[#531](https://github.com/july-rust/BoardSI/issues/531)/[#532](https://github.com/july-rust/BoardSI/issues/532)는
    **OPEN**. 이 체크는 **감사·문서 계약 인수만** 의미하며 신규 Rust/Qt 기능 구현·물리 제조·실제 PCB 측정·SI 정식 인수 또는 D_056_02~12 개발 완료가 아니다. 이 체크리스트 변경 PR/메인 Push 검증은 별도 증거로 추후 기록한다.

<a id="d_056_02"></a>
### D_056_02 — Source-Evidenced Multilayer Via Copper Candidate Graph (2026-11-08 KST)

- [x] **D_056_02 — SOURCE 기반 다층 Via/Trace/SMD 국소 Copper 후보 Graph 구현 및 소프트웨어 검증 (한정 완료)** · **체크일: 2026-11-08 KST** · **근거일: 2026-11-08 KST**
  - **최초 체크일: 2026-11-08 KST**
  - . [Code PR #535](https://github.com/july-rust/BoardSI/pull/535) 최종 HEAD `c904fed9ea04142bdd298b17aff99346207d220b`: 보호 Ruleset의 필수 작업 포함 **PR Workflow 9/9 SUCCESS**, 미해결 Review Thread **0**; 신규 Rust
    Source 기반 테스트 **6개 PASS**(2/4/6/8/64층 Through/Partial, 잘못된 Span, Source SHA 변경, 중간층 패드 부재, S/RA 형상, 비중심·서로 다른 선언 Net 접촉, 도금 증거 미확인). 보호된 단일 부모·서명 Verified Squash
    [main `2eddeacbb985c14313544c958e2c5a0707a89aa0`](https://github.com/july-rust/BoardSI/commit/2eddeacbb985c14313544c958e2c5a0707a89aa0) 및 **해당 정확한 SHA의 독립 main Push Workflow 8/8 SUCCESS** 확인. 구현
    [Source Layer Copper Graph 계약](architecture/d056-02-source-layer-graph.md)은 Source SHA-256, 실제 Layer별 도전성 패드 Node, Straight/Arc Trace·SMD 국소 기하 접촉 후보, **Unverified Barrel**만 제공한다. **전기적 Verified Edge
    0**, 제조 도금·전체 Open/Short·Copper Pour/Thermal·SI 인수는 미완이며 후속 [#529](https://github.com/july-rust/BoardSI/issues/529), 상위 [#520](https://github.com/july-rust/BoardSI/issues/520) **OPEN** 유지. 별도 선택 CodeQL
    동적 분석 2개는 GitHub 배포 번들 다운로드 HTTP **504**로 FAIL (재실행 403), **필수 17개 Ruleset Context가 아니며 원인 미해결을 그대로 기록**한다. 문서 정리 [PR #536](https://github.com/july-rust/BoardSI/pull/536)의 별도 Docs-only PR/Push 결과는 이
    항목의 후속 문서 증거로 확인한다.

**D_056_02 CodeQL 별도 스캔 재검증 (2026-11-08 KST, 비집계 감사):** 초기 코드 PR #535 HEAD `c904fed9ea04142bdd298b17aff99346207d220b`의
  [동적 CodeQL 실행 #42464801639](https://github.com/july-rust/BoardSI/actions/runs/42464801639)에서는 Rust·Python이 CodeQL CLI 번들 다운로드 **HTTP 504**로 실패했다. 이 **원래 실행의 상태는 FAIL로 유지**하며, 실패 Job 직접 재실행 및 전체 실패 Job
  재실행은 GitHub가 **HTTP 403 ("cannot be re-run")**으로 거부했다. 그러나 **동일 구현 코드가 병합된 정확한 `main@2eddeacbb985c14313544c958e2c5a0707a89aa0`**에서 독립적으로 발생한
  [CodeQL Push 실행 #42465987816](https://github.com/july-rust/BoardSI/actions/runs/42465987816)은 Rust·Python·C/C++·Actions **4/4 SUCCESS**를 확인했다. 후속
  [문서 PR #536 HEAD `9c960772`의 CodeQL #42467019442](https://github.com/july-rust/BoardSI/actions/runs/42467019442) 역시 **4/4 SUCCESS**. 반면 더 최신 문서 병합 `main@5c9e3ee5be29d241666fa617eece37c7336e7194`의
  [CodeQL Push 실행 #42467102254](https://github.com/july-rust/BoardSI/actions/runs/42467102254)는 Rust·Actions 번들 다운로드 **HTTP 504**로 **2/4 FAILED**. **검증된 해당 SHA의 정상 스캔 성공과 다른 SHA의 다운로드 실패는 구분하며, 과거 FAIL을
  PASS로 재기록하지 않는다.** 보안 경고 0건/코드 취약점 부재 또는 최신 HEAD의 완전한 CodeQL PASS를 주장하지 않는다.

<a id="d_056_03"></a>
### D_056_03 — Source-Scoped Electrical Graph Qualification & Open/Short Witnesses (2026-11-08 KST)

- [x] **D_056_03 — PADS V9.4 지원 SOURCE 범위 다층 전기 그래프/Net 충돌·Open 후보·Cut-set 알고리즘 한정 인수 완료** · **체크일: 2026-11-09 KST** · **근거일: 2026-11-09 KST**
  - **최초 체크일: 2026-11-09 KST**
  - . 지원하는 명시적 SOURCE Copper/Pad/Via/Trace 형상에서 근거 있는 동일층 양성 접촉만 추가하며, 다층 연결은 외부 공정/Transition 증거 없이는 `UNKNOWN`으로 남긴다. Source SHA 재입력과 임의 층 수 2/4/6/8/64, 부분·Blind/Buried/Through Span, 누락 중간 Pad,
    Anti-Pad/Drill Void, Net 충돌·증거 경로·Connected Components, Cut-Set과 12,000 Node 반복 체인, Polygon 중간 교차·홀수 폭 SMD 0.5 Tick 비중심 접촉 및 COPCUT/접선 부정시험을 독립 검증했다. **총 45개 명시적 D_056_03 독립 Oracle PASS** (PR #538:10,
    #540:8, #542:8, #543:7, [#547:12](https://github.com/july-rust/BoardSI/pull/547)); 정확한 #547 PR HEAD `11122c7a9988ed17fcb102fe29a94296e93c3806` **9/9 워크플로 SUCCESS** (Numerical Core/Qt 각 5/5 Job), 실패했던
    Windows ARM64 D_006 기존 CLI 회귀 Job은 **동일 SHA 재실행 SUCCESS**, 최초 실패 로그는 보존.
    [단일 부모·서명 Verified Code Squash `f31052a8f7189b56037bf100905a74f84076a22e`](https://github.com/july-rust/BoardSI/commit/f31052a8f7189b56037bf100905a74f84076a22e). **동일 SHA의 독립 code-main Push 8/8
    Workflow SUCCESS** ([Numerical Core #42540363760](https://github.com/july-rust/BoardSI/actions/runs/42540363760) **5/5 Job SUCCESS**,
    [Qt Runtime #42540363813](https://github.com/july-rust/BoardSI/actions/runs/42540363813) **5/5 Job SUCCESS**, 기타 6개 Push Workflow SUCCESS), PR-head와 code-main Push 별도 증거 확인. **인수 범위는 SOURCE-국소 설계 그래프
    알고리즘 및 Fail-Closed 계약**이다. 제조된 PCB 도통·Global Open/Short·실제 PCB 전체 Netlist·P009/SI Solver는 인수하지 않는다. 추가 PADS RT/ST·Slot·Backdrill 문법은 [#530](https://github.com/july-rust/BoardSI/issues/530),
    Canonical/Qt 통합 [#531](https://github.com/july-rust/BoardSI/issues/531), 타 CAD Source 어댑터 [#532](https://github.com/july-rust/BoardSI/issues/532), D_056 전체
    [#520](https://github.com/july-rust/BoardSI/issues/520)에서 별도 인수한다. 이들은 D_056_03 코드 한정 인수와 혼합하지 않는다.


**진척 증거 (전체 인수 미완료 · 공식 체크 건수 비집계 · 근거일 2026-11-08 KST):** [Code PR #538](https://github.com/july-rust/BoardSI/pull/538)의 최종 HEAD `74cd67ccf9e6b3b5284a03f29391b21c681fc55a`에서 **9/9 PR Workflow SUCCESS**,
  미해결 리뷰 Thread **0**, 신규 Rust 독립 Oracle **10/10 PASS** (2/4/6/8/64층, 선택된 Blind/Buried/Through Span, Source SHA/공정 증거, Negative 증거, Open 후보/Net 충돌 경로, Tarjan Cut-Set, 12,000 Node 체인, 증거 오류, 충돌 수 제한).
  Numerical Core의 Rust 포맷/경고/회귀 및 Windows x64/ARM64, macOS ARM64, Qt 4개 Native/Provisional Job이 PR-head에서 정상 완료했다. [Source-scope 설계·시험 계약](architecture/d056-03-electrical-graph-qualification.md),
  [개발 범위 및 코드](../crates/boardsi-import/src/electrical_graph_qualification.rs). 보호된 단일 부모·서명 **Verified Squash**
  [main `ca87a4d6b08bc75ccff9827bcfdf6d0ffed64cc4`](https://github.com/july-rust/BoardSI/commit/ca87a4d6b08bc75ccff9827bcfdf6d0ffed64cc4) (verification.verified=true/reason=valid, parent 1). **동일 정확 SHA의
  독립 code-main Push 8/8 Workflow SUCCESS** ([Numerical Core #42470750025](https://github.com/july-rust/BoardSI/actions/runs/42470750025) 5/5 Job SUCCESS,
  [Qt runtime #42470750066](https://github.com/july-rust/BoardSI/actions/runs/42470750066) 5/5 Job SUCCESS, 기타 6개 Push Workflow SUCCESS). PR-head/Code-main Push를 별개 확인했다. 문서 전용 PR의 Markdown CI 및 병합 Push는
  이와 별도 증거로 확인한다.

**D_056_03 CopperArea 추가 검증 (근거일 2026-11-08 KST · 전체 인수 미완료 · 공식 완료 체크/분모 변경 없음):** [Code PR #540](https://github.com/july-rust/BoardSI/pull/540) 최종 HEAD `e81e4fdfb58157a468d8cfab17b5a6a43ca1d179`에서 **9/9
  PR 워크플로 SUCCESS**, Numerical Core **5/5 Job SUCCESS** ([#42475396805](https://github.com/july-rust/BoardSI/actions/runs/42475396805)), Qt Runtime **5/5 Job SUCCESS**
  ([#42475396842](https://github.com/july-rust/BoardSI/actions/runs/42475396842)), 신규 [SOURCE CopperArea 독립 Oracle](../crates/boardsi-import/tests/d056_03_source_area_graph.rs) **8/8 PASS**, 미해결 리뷰 스레드
  **0건**. Source 소유 Polygon/Circle/Cutout을 그래프 노드로 추가하고 국소 Trace/ViaPad/SMDPad/Area-to-Area **양성 증거 후보만** 연결한다. 원형 Hole 내부 Copper 거짓 Short, 다른 층 동일 XY 거짓 연결, 누락 접촉 기반 Open 확정, 대량 입력 접촉 수 100,000 초과 버그를
  Fail-Closed로 차단했다. 보호된 1-parent, 서명 **Verified** Squash로 [`main@1afbf1626d1adc93f6bb4ebedb028d65c0f86b39`](https://github.com/july-rust/BoardSI/commit/1afbf1626d1adc93f6bb4ebedb028d65c0f86b39) 병합 확인.
  **병합 후 동일 SHA의 독립 code-main Push 8/8 Workflow SUCCESS** ([Numerical Core #42479164786](https://github.com/july-rust/BoardSI/actions/runs/42479164786) 5/5 Job SUCCESS,
  [Qt Runtime #42479164930](https://github.com/july-rust/BoardSI/actions/runs/42479164930) 5/5 Job SUCCESS, 기타 6개 Push Workflow SUCCESS); PR-head와 code-main Push 결과를 별도 확인했다.

**D_056_03 추가 SOURCE 연결 구성·임의 각도 검증 (근거일 2026-11-09 KST, 공식 전체 인수 상태는 남은 관문 기준):** [Code PR #542](https://github.com/july-rust/BoardSI/pull/542) 최종 HEAD `15a708dc58b2eb2ed6df76c01836e36a0891206a`에서 **9/9
  PR Workflow SUCCESS**, Numerical Core **5/5 Job SUCCESS**, Qt Runtime **5/5 Job SUCCESS**, 신규 [직접 Pad 연결 독립 Oracle](../crates/boardsi-import/tests/d056_03_source_pad_pair_graph.rs) **8/8 PASS**, 미해결 리뷰
  0건. ViaPad–ViaPad/ViaPad–SMD Pad/SMD Pad–SMD Pad 동일층 양성 도체 접촉 후보를 합성 Node와 실제 PADS V9.4 SOURCE에서 독립적으로 검증했고, 동일 Via의 L1 R35 / L4 R40 Pad 크기에 따른 접촉/분리를 구별한다. Native Anti-pad, Drill Void, 동층 접선 및 이층 동일 XY의
  거짓 단락을 차단. 보호된 1-parent, 서명 **Verified** Squash [`main@0c45f5a3c361d1cf6aabc82b1ded911486aa90a7`](https://github.com/july-rust/BoardSI/commit/0c45f5a3c361d1cf6aabc82b1ded911486aa90a7) 병합 완료. **동일 SHA의 독립
  code-main Push 8/8 Workflow SUCCESS** ([Numerical Core #42533804238](https://github.com/july-rust/BoardSI/actions/runs/42533804238) 5/5 Job SUCCESS,
  [Qt Runtime #42533804450](https://github.com/july-rust/BoardSI/actions/runs/42533804450) 5/5 Job SUCCESS, 기타 6개 Push Workflow SUCCESS).

[Code PR #543](https://github.com/july-rust/BoardSI/pull/543) 최종 HEAD `49a7b2fbd1e5ba17c37cf3f551adf1507e3c62a3`: **9/9 PR Workflow SUCCESS**, Numerical Core **5/5 Job SUCCESS**, Qt Runtime **5/5 Job
  SUCCESS**, 신규 [임의 각도 Straight–Copper 경계 교차 Oracle](../crates/boardsi-import/tests/d056_03_straight_area_witness.rs) **7/7 PASS**, 미해결 리뷰 0건. Fixed 1/4-point 탐색의 좁은 중간 구간 누락을 SOURCE COPCLS/COPCIR 경계 교차
  이벤트로 보강하고 CUTOUT·접선·실제 Arc Chord가 아닌 도형을 거짓 연결하지 않도록 한다. 보호된 1-parent, 서명 **Verified** Squash
  [`main@d939409947e297cf334f8570087d9e1f5c3e231c`](https://github.com/july-rust/BoardSI/commit/d939409947e297cf334f8570087d9e1f5c3e231c) 병합 완료. **동일 SHA의 독립 code-main Push 8/8 Workflow SUCCESS**
  ([Numerical Core #42535600072](https://github.com/july-rust/BoardSI/actions/runs/42535600072) 5/5 Job SUCCESS, [Qt Runtime #42535600145](https://github.com/july-rust/BoardSI/actions/runs/42535600145) 5/5
  Job SUCCESS, 기타 6개 Push Workflow SUCCESS), PR-head와 code-main Push 각각 독립 확인. 33개 신규 독립 Oracle(기본 10 + Area 8 + Pad Pair 8 + Straight/Area 7)의 PASS와 실제 사용자 보드 전체 Source Coverage는 서로 다르다. **물리
  제조/Plating/전체 Open·Short/SI 인증을 주장하지 않는다.** [설계·시험 증거](architecture/d056-03-electrical-graph-qualification.md).

**문서 반영 최종 확인 (2026-11-09 KST):** [Docs PR #544](https://github.com/july-rust/BoardSI/pull/544) 정확 HEAD `4fd18131022e3b75b2a385a4f15027879cb0ddef` 문서 전용 8/8 Workflow SUCCESS, 미해결 Review 0건,
  [보호 `main` 서명 Verified·단일 부모 Squash `40ed93c2db37558d2ef2e353675498461c767ff0`](https://github.com/july-rust/BoardSI/commit/40ed93c2db37558d2ef2e353675498461c767ff0) 및
  [동일 SHA 문서 `main` Push #42536866564](https://github.com/july-rust/BoardSI/actions/runs/42536866564) 포함 **8/8 독립 Workflow SUCCESS** 확인. 서명 검증과 코드·문서 CI 감사는 각각 별도 기록했다. 이 완료는 **문서/코드 한정 증거의 마무리**이지 전체
  D_056_03 전기적 인수 완료가 아니다.

**지원 범위 외의 후속 관문 (D_056_03 소프트웨어 한정 인수와 구별):** 현재 `LayeredSourceGraph`는 제한 PADS V9.4 SOURCE-국소 Copper 후보 인벤토리다. PR #540/#542/#543으로 **일부** 명시적 Polygon/Circle Copper·Cutout, ViaPad/SMD Pad 직접 접촉, 임의 각도 직선의
  경계 교차 양성 접촉을 지원하지만 모든 면적 교차/접촉(특히 혼합 Round/Rectangle과 Copper Area, Trace Arc)에 대한 완전 탐색, 특수 Flood/Pour/Island, Thermal/Slot, 일부 Pad/Area 조합, 전체 CAD Source/Netlist Coverage, Plating/제조 공정 적격성은 미확인이다. 이
  모듈이 산출하는 충돌·분리 결과는 지원된 SOURCE 설계 그래프의 증거 또는 후보일 뿐 **실물 PCB Open/Short·SI Solver PASS·제조 도통 확정이 아니다**. D_056_03의 한정 소프트웨어 인수 완료 여부와 관계없이 [부모 #520](https://github.com/july-rust/BoardSI/issues/520)는
  **OPEN**이다. 미지원 Source 형상은 진짜 연결로 만들지 않고 `UNKNOWN`/명시적 거부하며 D_056_04~12는 별도 인수한다. 이전 날짜의 미완료 문단은 역사적 진행 기록이며 위 신규 체크가 최신 한정 인수 결론이다.

<a id="d_056_04"></a>
### D_056_04 — Versioned Canonical Padstack & Multilayer Via IPC (2026-11-09 KST)

- [x] **D_056_04 SOURCE 지원 범위 Canonical Padstack/Via → Versioned Rust/Qt IPC V1 독립 소프트웨어 한정 인수** · **체크일: 2026-11-09 KST** · **근거일: 2026-11-09 KST**
  - **최초 체크일: 2026-11-09 KST**
  - . [Issue #531](https://github.com/july-rust/BoardSI/issues/531) (D_056_05 완료 전 OPEN), [구현 PR #549](https://github.com/july-rust/BoardSI/pull/549),
    [IPC 구조·오류 처리·Native 검증 문서](architecture/d056-04-canonical-padstack-ipc.md). Rust `canonical_padstack_ipc.rs`와 전용 CLI, Qt 엄격 디코더/독립 테스트는 실제 SOURCE SHA-256과 전체 Body SHA-256, 지원 SOURCE/Frame 및 정확한
    Rational Dimension, 2/4/6/8/64층 물리 Layer, 임의 부분 Span, 층별 원형·사각·환형 R/S/A와 RA/SA AntiPad 구분, 패드 없는 내층, Via/Padstack/Drill 식별자, Drill Plating 선언의 **검증되지 않은 상태**, 경계 초과/변조/불일치 시 Fail-Closed를 확인한다. 실제 Rust
    CLI→Qt SOURCE Round-trip이 포함되며 기존 Artwork V1~V14는 변경하지 않는다. **정확한 최종 PR HEAD `5170669e0e71c3722bc225a35b65a2c44f2fdc13`: 36/36 GitHub Check SUCCESS**, Numerical/Qt Linux·Windows x64/ARM64·macOS
    ARM64/CodeQL을 포함한 PR 11개 워크플로 SUCCESS, 미해결 리뷰 스레드 **0**. 처음 Ubuntu apt 외부 타임아웃으로 FAIL이었던 F6 Job은 동일 SHA 재실행에서 SUCCESS; 원 실패 이력은 유지. 보호된 **Verified 1-parent Squash**
    [`main@6eb35680a6e7775f62cd0fd763b559955810e7d6`](https://github.com/july-rust/BoardSI/commit/6eb35680a6e7775f62cd0fd763b559955810e7d6) 반영·main ref 일치 확인. **이 정확한 SHA의 독립 code-main Push 결과는 별도 확인 후 아래
    후속 증거에 기록**하며 PR-head SUCCESS로 대체하지 않는다. `PadShape::Custom`, 다중 Source 파일, 지원 외 CAD/Frame, Thermal RT/ST, Slot/Backdrill, 제조 공정 Microvia, 전역 `connected_layers` 증거를 V1이 완전 전송한다고 주장하지 않는다. 전역 Qt Viewer
    CPU/RHI Layer/Picking/취소 통합은 **D_056_05 별도**, [부모 #520](https://github.com/july-rust/BoardSI/issues/520) **OPEN**. **실물 PCB 도통·전역 Open/Short·SI·제품 U/T는 NOT_VERIFIED**.

**D_056_04 독립 코드-main Push 최종 감사 (2026-11-09 KST):** Code PR #549의 PR-head **36/36 Checks SUCCESS**와 보호 Verified Squash에 더해, **동일 정확 SHA `6eb35680a6e7775f62cd0fd763b559955810e7d6`의 독립 main event=push 8/8
  Workflows SUCCESS**를 별도로 확인하였다. [Numerical Core #42555816109](https://github.com/july-rust/BoardSI/actions/runs/42555816109) (Rust fmt/Clippy, Workspace Debug/Release 및 Windows x64/ARM64/macOS ARM64),
  [Qt Runtime #42555816222](https://github.com/july-rust/BoardSI/actions/runs/42555816222) (Linux x86_64, Windows x64/ARM64, macOS ARM64 Native Qt), 기타 6개 Push Workflows 모두 SUCCESS. 코드 PR HEAD의 Linux Qt
  CTest **78/78 PASS**, 신규 `qt-d05604-canonical-padstack-ipc` 및 `qt-d05604-rust-canonical-roundtrip` 각각 PASS, Rust 신규 단위 테스트 **2/2 PASS**를 독립 실행 로그에서 확인하였다. **문서 전용
  [PR #550](https://github.com/july-rust/BoardSI/pull/550)과 문서 병합 후 별도 main Push는 코드-main Push와 구별**하고 실제 결과를 별도 확인한다.

<a id="d_056_05"></a>
### D_056_05 — Qt CPU/RHI Main Artwork & Legacy V1–V14 Compatibility (2026-11-09 KST)

- [x] **D_056_05 지원 PADS SOURCE 기반 Canonical Padstack/Via의 실제 메인 Qt CPU/RHI Artwork 통합과 Legacy V1~V14 소프트웨어 한정 코드 인수 완료** · **체크일: 2026-11-09 KST** · **근거일: 2026-11-09 KST**
  - **최초 체크일: 2026-11-09 KST**
  - . [Issue #531](https://github.com/july-rust/BoardSI/issues/531)
  - [코드 PR #551](https://github.com/july-rust/BoardSI/pull/551)
  - [Qt CPU/RHI 구현·검증 원장](architecture/d056-05-qt-main-artwork.md). 별도 실험용 뷰어가 아닌 실제 `ImportedArtworkPanel`/`PreviewImportPanel`에 D_056_04 엄격한 Source SHA-256 IPC를 연결하여 2/4/6/8/64층의 실제 Layer 순서,
    Through/Blind/Buried Span, 층별 원형·직사각·환형 Conductive Pad·Drill Void·AntiPad/내층 Pad 없음, Source 근거·Net UNKNOWN·레이어 Visibility·Fit·Picking·Hover·메인 Properties 표시를 한정 구현. CPU에서는 구멍 내부를 실수로 Stroke하지 않으며 RHI는
    보수적 Holes Mesh/정렬된 겹침 순서/CPU Fallback을 적용; 실제 GPU 픽셀·실물 Copper 도통 인증은 하지 않는다. QProcess 4 MiB Stdout/256B Stderr 상한, Source SHA 재검증, Cancel/Reimport, Windows x64 Qt 패키지의 별도 Canonical CLI 검사를 포함한다.
    Legacy Artwork V1~V14의 기존 경로는 변경하지 않고 비균일·부분 Via를 구형 균일 Through Via로 허위 변환하지 않는다. **정확한 코드 최종 PR HEAD `7f251a2d802cad92f84c1b28f4906ec6ba59a958` 35/35 Check SUCCESS
  - 11/11 Workflow SUCCESS**, Linux Qt CTest **81/81 PASS**, Native Qt Linux x64·Windows x64/ARM64·macOS ARM64 **4플랫폼 SUCCESS**, 이전 Codex P2 결함 4건 수정 및 미해결 리뷰 Thread 0(최종 HEAD에 대한 신규 Codex 독립 리뷰 완료라고 단정하지
    않음). 보호된 단일 부모 서명 Verified Squash [코드-main `6103a1b6d6f1068b548f4cbb13d7c9cfff6544dd`](https://github.com/july-rust/BoardSI/commit/6103a1b6d6f1068b548f4cbb13d7c9cfff6544dd), GitHub
    `verified=true/reason=valid`, main ref 일치 확인. **정확한 코드-main event=push 8개 워크플로 결과는 아래 후속 인수 감사에서 PR 결과와 분리하여 기록한다.** 상위 [#520](https://github.com/july-rust/BoardSI/issues/520)은 D_056_06~12 때문에
    **OPEN**. 아직 미지원 PADS Thermal/Slot/Backdrill/다중 CAD, 전체 Source Coverage, 글로벌 제조 전기 Open/Short, Return Path, RLGC/S, 물리 측정, 정식 제품 U/T·출시는 **NOT_VERIFIED**.

**D_056_05 코드-main Push 별도 최종 감사 (2026-11-09 KST):** 코드 PR #551의 HEAD 35/35 PASS 및 서명 Verified Squash와 별개로, **정확한 코드-main `6103a1b6d6f1068b548f4cbb13d7c9cfff6544dd`의 독립 `event=push` 8/8 Workflow 모두
  SUCCESS** 확인. [Qt Runtime Push #42570405620](https://github.com/july-rust/BoardSI/actions/runs/42570405620) **Linux x64·Windows x64/ARM64·macOS ARM64 Native Qt 4개 SUCCESS** 및
  [Numerical Core Push #42570405681](https://github.com/july-rust/BoardSI/actions/runs/42570405681) **Linux numerics 및 Windows x64/ARM64·macOS ARM64 Native Rust 4개 SUCCESS**. 나머지 6개 `event=push` 문서/합성
  FEM/PH-06/PH-07/Native FEM 워크플로도 모두 SUCCESS. 코드 PR·코드-main Push의 결과는 서로 다른 SHA와 이벤트로 구분하며, **문서 전용 후속 PR #552/문서-main Push 결과는 또 별도 인수**한다. 검증 성공은 지원 PADS SOURCE 형상의 Qt 소프트웨어 한정 인수이지 실물 PCB 전기 연결성이나 SI
  제품 인수가 아니다.

<a id="d_056_06"></a>
### D_056_06 — PADS RT/ST Thermal Spoke SOURCE / Canonical V2 / Main Qt (2026-11-09 KST)

- [x] **D_056_06 공식 PADS RT/ST Thermal Spoke의 SOURCE 한정 소프트웨어 구현·코드 PR 검증·보호 Squash 반영 완료** · **체크일: 2026-11-09 KST** · **근거일: 2026-11-09 KST**
  - **최초 체크일: 2026-11-09 KST**
  - . [상위 #520](https://github.com/july-rust/BoardSI/issues/520) / [PADS 제조 #530](https://github.com/july-rust/BoardSI/issues/530)
  - [구현·원본 문법·독립 검증 원장](architecture/d056-06-thermal-source.md)
  - [코드 PR #553](https://github.com/july-rust/BoardSI/pull/553) 정확한 최종 HEAD `3eceac04b3131d0ebb83501f37aadea002a5aaf9` 기준 **12/12 Workflow SUCCESS**, Rust/Qt Linux x64·Windows x64/ARM64·macOS ARM64 Native
    PASS, 리뷰 지적 3건 수정, 미해결 Review Thread 0. 원본 PADS V9.4 RT/ST Spoke/Relief 문법·SOURCE SHA와 Via Layer Span·중앙 Pad/Spoke Copper·Drill/Plane Void를 보존하고, Canonical `PadShape::Thermal` 및 **V1 Byte-compatible V2
    IPC**, 실제 `ImportedArtworkPanel` CPU Geometry·Picking 및 안전 CPU Fallback, 좁은 의미의 양성 Source-local Trace/Area 접촉 후보를 구현했다. 오류 Source/부족한 제조·Plane 증거는 **UNKNOWN/UNSUPPORTED로 Fail-Closed**한다. 특히 (1) 겹치는
    Spoke의 winding-fill union, (2) Spoke 외곽 크기를 포함하는 contact broadphase, (3) RHI 실패 뒤 GPU 빈 캔버스로 잘못 되돌아가는 문제를 수정하고 테스트로 검증했다. 보호 Squash 병합
    [code-main `e034335b055f068edcd68a17676c1f8f3e161fd8`](https://github.com/july-rust/BoardSI/commit/e034335b055f068edcd68a17676c1f8f3e161fd8) 및 현재 main ref 일치를 확인했다. 초기 Windows x64 Qt Native 시도는 오류 코드
    127로 실패했으나 **같은 HEAD의 실패 Job 재실행 성공**으로 최종 12/12 Workflow 성공을 확인했으며 실패 이력을 지우지 않았다.
- [x] **D_056_06 코드·문서 독립 main Push 검증 완료** · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST**
  - . [코드-main `e034335b055f068edcd68a17676c1f8f3e161fd8`](https://github.com/july-rust/BoardSI/commit/e034335b055f068edcd68a17676c1f8f3e161fd8)의 정확한 `event=push`는 **8/8 SUCCESS**
    ([Qt Runtime #42614036173](https://github.com/july-rust/BoardSI/actions/runs/42614036173) attempt 2 성공).
    [문서-main `04374fe3ae0f60436822e18ff3255911d881425b`](https://github.com/july-rust/BoardSI/commit/04374fe3ae0f60436822e18ff3255911d881425b)의 독립 `event=push`도 **8/8 SUCCESS** 확인. PR-head CI와 두 개의 독립 main
    Push 검증을 분리 기록하며, 제조·전역 전기·SI 판정까지 확대하지 않는다.
- [ ] **제조·전체 PCB·SI 제품 인수**
  - D_056_06 자체 범위가 아니다. 실제 CAM Thermal 생성·Plane Fill·Plating/Barrel 도통·전역 Open/Short·다중 포맷 CAD·RLGC/S 파라미터와 실물 측정은 **NOT_VERIFIED**. [#530](https://github.com/july-rust/BoardSI/issues/530)은 2026-11-10 조회
    시 **CLOSED** 상태이나 Native Backdrill/Stacked Microvia 전체 구현의 기술적 인수를 뜻하지 않는다. 상위 [#520](https://github.com/july-rust/BoardSI/issues/520)은 **OPEN**.

<a id="d_056_07"></a>
### D_056_07 — PADS 제조 Slot/NPTH/Backdrill/Microvia SOURCE 확인 (2026-11-09 KST)

- [x] **D_056_07 PADS PARTDECAL 한정 Source/Canonical 계약과 Legacy IPC 정보 손실 차단 소프트웨어 검증** · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST**
  - **최초 체크일: 2026-11-10 KST**
  - . 원본 SHA-256·행 기반 NPTH/PTH/Slot Source 증거, 계층 독립 제조 의도·가변 Layer Span(2~64), 명시·생략형 plated 문법, Slot 각도·길이·Offset 검증, unsupported input fail-closed, 제조 도통을 증명하지 않는 API와 회귀 테스트를 구현했다.
    [PR #555](https://github.com/july-rust/BoardSI/pull/555) **38/38**, [PR #558](https://github.com/july-rust/BoardSI/pull/558) **30/30**, [PR #559](https://github.com/july-rust/BoardSI/pull/559)
    **30/30** PR Checks SUCCESS, 리뷰 미해결 0건, 세 코드 커밋 모두 Verified Squash; 독립 Push는 다음 원장에 별도 확인한다. **실제 제조·Qt 렌더링·전기 그래프·SI 인수 의미 아님**.
- [ ] **D_056_07 전체 기능 인수 미완료:**
  - [제조 Source Issue #530](https://github.com/july-rust/BoardSI/issues/530)은 **CLOSED**(2026-11-10 확인, 상태 변경일 2026-11-10 KST), 그러나 [상위 #520](https://github.com/july-rust/BoardSI/issues/520)은 **OPEN**이며
    Source 한정 코드 통과를 제조·Qt·글로벌 그래프 지원으로 오인하지 않는다. [코드 PR #555](https://github.com/july-rust/BoardSI/pull/555) / [안전성 PR #558](https://github.com/july-rust/BoardSI/pull/558) /
    [강화 PR #559](https://github.com/july-rust/BoardSI/pull/559) / [설계·미지원 원장](architecture/d056-07-fabrication-source.md)은 제한된 PADS V9.4 PARTDECAL Round PTH/NPTH/Slot Source 해석, SHA·행 증거, 정확한 Rational 변환,
    2/4/6/8/64층 Span과 명시적 legacy IPC 거부를 구현한 **소프트웨어 부분집합**이다. 전체 D_056_07 인수로 표시하지 않는다.
- [ ] **필수 후속 구현/증거 미확보:**
  - PADS 네이티브 VIA/ROUTE Slot/NPTH, Backdrill 및 Stacked/Staggered Microvia의 공개 검증된 Native Source 문법·실제 벤더 Export Fixture, 부품 위치에 결합된 Hole/Copper Boolean, 완전한 Versioned IPC, 메인 Qt CPU/RHI Picking/Geometry
    통합, Backdrill 제거 Barrel에 따른 SOURCE Scope Graph 연결 결과, 전역 제조 Open/Short, Si Solver는 **UNSUPPORTED 또는 NOT_VERIFIED**. PARTDECAL 문법의 N/P·Slot 필드를 VIA 행으로 잘못 승격하지 않으며 제조 공정 미확인 자료에서 Barrel 연결 Edge를 생성하지
    않는다. 기존 D_056_04~06의 V1/V2, Legacy Qt 경로 및 회귀 보존을 필수로 한다.
- [x] **D_056_07 SOURCE 범위 코드·문서 독립 main Push 감사 완료** · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST**
  - **최초 체크일: 2026-11-10 KST**
  - . [원본 Mentor PADS V9.4 ASCII 명세](https://www.freecalypso.org/pub/CAD/PADS/pdfdocs/Ppcb_ASCII.pdf) Part Decal pp.36–38와 Via Definition pp.39–40을 분리 참고. Source 한정 코드
    [#555](https://github.com/july-rust/BoardSI/pull/555): 최종 PR HEAD `6f4a5d9041ae8c934f3aabbd8dac67b5939d88f8`, **38/38 SUCCESS**, Verified Squash-main `3bda3f0b34c183359e3b5eef59b7edd94965207d`, 독립 Push
    **8/8 SUCCESS**. [#558](https://github.com/july-rust/BoardSI/pull/558): 최종 HEAD `9d0c333d8c9b03ed6bca85a071431b96811e6ab8`, **30/30 SUCCESS**, Verified Squash-main
    `656a3cb359654c5245d49c37e36b58591de7d458`, 독립 Push **8/8 SUCCESS**. [#559](https://github.com/july-rust/BoardSI/pull/559): 최종 HEAD `514cce09a13e19ebd4c1a9d237485b804db1ff0d`, **30/30 SUCCESS**,
    Verified Squash-main `3ab697f09482abf4f0a4eb1111e32c5d276a73d9` 확인.
    [#559 정확한 코드-main Push `3ab697f09482abf4f0a4eb1111e32c5d276a73d9`](https://github.com/july-rust/BoardSI/commit/3ab697f09482abf4f0a4eb1111e32c5d276a73d9) **8/8 SUCCESS** 확인 (취소된
    [Qt runtime #42722623172](https://github.com/july-rust/BoardSI/actions/runs/42722623172)는 attempt 2에서 성공). [문서 PR #557](https://github.com/july-rust/BoardSI/pull/557)은 Verified Squash
    `a395b015756c853622aceeb2f123e3d79c168a19` 및 독립 Push **8/8 SUCCESS**. 이 항목의 소프트웨어 CI/Push 감사만 완료되었고 Native Backdrill/Microvia 제조 문법, Qt Fabrication Artwork/IPC 및 전역 전기 Graph 인수는 여전히 미완료이다.

<a id="d_056_08"></a>
### D_056_08 — Native ODB++Design SOURCE Adapter (2026-11-10 KST)

- [x] **D_056_08 ODB++ SOURCE 한정 Parse→Canonical 소프트웨어 검증·보호 병합** · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST**
  - **최초 체크일: 2026-11-10 KST**
  - . 압축 해제된 ODB++Design Directory/File-map의 `matrix/matrix` STEP/Layer 순서와 선택 STEP의 제한된 양성 `L/P/A` Copper Record를 `Trace/Pad/Padstack/Arc`로 전환. 2/4/6/8/64층 공통 Canonical 및 `BoardModel::validate` 구조 검증,
    Exact Decimal/단위, SHA-256 및 원본 File/Line 증거, 미지원 Copper/Full-circle/Zero-length/불량 입력 Fail-Closed, 디렉터리 경로·Symlink·Size 안전 점검과 진단 CLI. [코드 PR #562](https://github.com/july-rust/BoardSI/pull/562)의 **정확한
    최종 HEAD `8502bc0fa8a3ed9ff92b6c55e365cf1b97a0c411`에서 30/30 Checks SUCCESS**, GitHub 코드리뷰 지적 4건 모두 해결, 미해결 0건. Verified single-parent Squash
    [main `1f33e75beced7115cb65453aca4c6e9fc1d6df36`](https://github.com/july-rust/BoardSI/commit/1f33e75beced7115cb65453aca4c6e9fc1d6df36). **별도 main Push 8/8 워크플로
    SUCCESS**([Numerical](https://github.com/july-rust/BoardSI/actions/runs/42733742478), [Qt](https://github.com/july-rust/BoardSI/actions/runs/42733742501),
    [Docs](https://github.com/july-rust/BoardSI/actions/runs/42733742895), [FEM](https://github.com/july-rust/BoardSI/actions/runs/42733742572) 및 독립 관련 4건). **테스트 범위는 합성 Grammar Fixture 및 기존 회귀이며 실제 벤더
    Export 전체 호환성 인수는 아님**.
- [x] **D_056_08 ODB++ 양성 S/OB/OS/OE/SE Copper Area·Cutout SOURCE 한정 구현·보호 Squash 인수** · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST**
  - **최초 체크일: 2026-11-10 KST**
  - . 선택한 실제 BOARD Copper Layer의 단일 Positive Surface Island + 직선 경계와 엄격하게 내포된 Hole 최대 32개를 정확한 Source 좌표계에서 `i128` 기하 검증하고 `CopperArea/Boundary/Cutout`으로 변환한다. Self-Intersection, Boundary Touch, 잘못된
    Hole/미종결/음성 Surface/곡선 OC/복수 Island는 Fail-Closed. 사용자 진단 CLI에 Area/Cutout Count 추가. [코드 PR #564](https://github.com/july-rust/BoardSI/pull/564)의 최종 HEAD `d1f2ce51ab21764de7ef5cac2698179a0755636a`
    **30/30 PR Checks SUCCESS**, 코드 리뷰 수정 1건 해결 및 미해결 0건, 보호 Verified single-parent Squash
    [main `0d832dd92d86ea1d6e375b89506cf74be9b252d0`](https://github.com/july-rust/BoardSI/commit/0d832dd92d86ea1d6e375b89506cf74be9b252d0) 확인. 별도로 같은 main SHA의 **독립 Push 8/8 Workflow SUCCESS**
    ([Numerical #42742249073](https://github.com/july-rust/BoardSI/actions/runs/42742249073), [Qt #42742249075](https://github.com/july-rust/BoardSI/actions/runs/42742249075),
    [문서 #42742249504](https://github.com/july-rust/BoardSI/actions/runs/42742249504), 독립 FEM 관련 5건도 SUCCESS) 확인. **벤더 Source Fixture 독립 검증, Main Qt Artwork, Net/Plating/전기 Graph/SI 인수 아님**.
- [x] **D_056_08 ODB++ EDA NET/SNT/FID 설계 Net 소유 관계 SOURCE 한정 검증·보호 Squash 인수** · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST**
  - **최초 체크일: 2026-11-10 KST**
  - . 압축 해제된 ODB++Design `steps/<step>/eda/data`의 HDR·**필수 UNITS=MM/INCH**·LYR·NET·SNT(TRC/VIA/PLN)·FID(C, 0-base 실제 Copper Feature Index)를 `--eda-nets` **명시적 선택 시에만** 해석, 원본 Source SHA 및 FID 행번호를 보존해
    Trace/Arc/Pad/CopperArea의 설계 `Net`을 정합한다. 중복 실명 Net·중복 FID·없는 Feature·잘못된 Layer/Type·미지원 PKG/PIN/TOP·잘못된 Units는 Fail-Closed, 반복 `$NONE$`은 허용하되 소유 Net/전기 접속 생성하지 않는다. 기존 Geometry-only
    Import·SourceIdentity 계약은 그대로 유지. [코드 PR #566](https://github.com/july-rust/BoardSI/pull/566) **정확 최종 HEAD `518eefc9aa9e5f5ae66d40b3c3d4228e3971290e` 30/30 Checks SUCCESS** 및 리뷰 P1/P2 2건 수정·미해결 0,
    GitHub Verified 1-parent Squash [main `ffdad981aa96a2a89eee9c8f569b3d7b74916f58`](https://github.com/july-rust/BoardSI/commit/ffdad981aa96a2a89eee9c8f569b3d7b74916f58). **제조 도금·실제 Net 연결·전역
    Graph/Open/Short·Main Qt·SI는 인수하지 않음**. **독립 post-merge main Push CI 8/8 SUCCESS** ([Docs #42749304686](https://github.com/july-rust/BoardSI/actions/runs/42749304686),
    [Numerical #42749304963](https://github.com/july-rust/BoardSI/actions/runs/42749304963), [Qt #42749304706](https://github.com/july-rust/BoardSI/actions/runs/42749304706), FEM·G0 관련 5개도 SUCCESS), 실패 0건을
    같은 SHA에서 검증했다.
- [x] **D_056_08 ODB++ Drill MATRIX Layer Span / Tool SOURCE 의도 감사 한정 인수** · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST**
  - **최초 체크일: 2026-11-10 KST**
  - . MATRIX `TYPE=DRILL` 및 명시적 `START_NAME/END_NAME`, `layer/tools` NUM/TYPE/TYPE2/FINISH_SIZE/DRILL_SIZE를 소스 순서·단위·행·SHA와 교차 검증한다. 6층 합성 Through/Backdrill/Partial Span 및 잘못된 Tool 번호·명세·반전 Span에
    Fail-Closed; 임의 중간층 도금·실제 제조 홀·구리 접촉은 **UNKNOWN**. [코드 PR #568](https://github.com/july-rust/BoardSI/pull/568) 최종 HEAD `8e09dd464ca9633b65afa4238f104e4f4568fd96` **30/30 PR Checks SUCCESS**, 보호
    Verified single-parent Squash [main `badc4fb23011aea5193a5d39e8401e120c451ac5`](https://github.com/july-rust/BoardSI/commit/badc4fb23011aea5193a5d39e8401e120c451ac5), 독립 main Push **8/8 Workflow
    SUCCESS** ([Qt #42760798834 attempt 2](https://github.com/july-rust/BoardSI/actions/runs/42760798834)). 이 항목은 Source Tool Intent **감사 한정**으로만 체크하며 P hit 위치/Canonical Drill/Via/메인 Qt/전기 Graph/SI 및 외부 벤더
    검증은 미완료.
- [x] **D_056_08 ODB++ Drill P Feature Hit→Tool NUM 독립 SOURCE 한정 인수** · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST**
  - **최초 체크일: 2026-11-10 KST**
  - . Drill Layer `features`의 Positive `P` 좌표와 명시적 0 초과 `dcode`를 `layer/tools` NUM과 교차 검증하고 File/Line/Feature Index·SHA-256·MATRIX 행·Tool 원본 행 번호를 소유 증거로 기록한다. Copper Layer의 미지원 그래픽 파싱과 독립된
    `boardsi-odb-drill-audit ... --hits`는 합성 6층 예제 및 Tool 누락·불명·Unit/개수/ID 모순에 Fail-Closed 동작한다. [코드 PR #569](https://github.com/july-rust/BoardSI/pull/569) 최종 HEAD
    `cd5e49a7a51289a9b2bd09fa30d9e7bc4cbc1861`: **30/30 Checks SUCCESS** (취소되었던 Linux Qt Provisional 재실행 성공), 리뷰 미해결 0.
    [보호 Verified 단일 부모 Squash `f07442a1d97d11b49419f32655c03b1714d4840f`](https://github.com/july-rust/BoardSI/commit/f07442a1d97d11b49419f32655c03b1714d4840f) 및 **별도 code-main Push 8/8 Workflow SUCCESS**
    ([Qt #42771233014](https://github.com/july-rust/BoardSI/actions/runs/42771233014), [Numerical #42771232754](https://github.com/july-rust/BoardSI/actions/runs/42771232754), 기타 6개 전부 SUCCESS) 확인. 이 체크는
    Source에 선언된 Hole 위치/Tool의 **관측 증거**에만 해당한다. **Canonical Drill/Via, 실제 가공 도금·전도성 Barrel, Main Qt Import/IPC, 전역 Graph/Open/Short, 벤더 CAM 검증, SI는 모두 미인수**.
- [x] **D_056_08 Uncompressed USTAR Native Source Reader 한정 인수** · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST**
  - **최초 체크일: 2026-11-10 KST**
  - . USTAR Header/Octal Size/Checksum/종료 Block 및 단일 Product-root를 검증하고 파일시스템 추출·외부 Shell 없이 bounded OdbFileMap으로 연결, 기존 Native Directory와 동일한 제한 Copper Canonical 경로 및 선택형 EDA FID를 재사용한다. 합성 2층 실제
    Canonical Trace 동등성, USTAR 경로/Link/중복/손상·복수 PCB Root 거부, 정상 STEP `steps`와 최상위 폴더명 `matrix`/`steps` 호환을 회귀 테스트로 고정. [코드 PR #571](https://github.com/july-rust/BoardSI/pull/571) 최종 HEAD
    `b7c4c14a73d82f7d068ecf827db2fb4b8b2e53bc` **30/30 PR Checks SUCCESS**, 미해결 리뷰 0건, 보호 GitHub Verified 단일 부모 Squash
    [main `b425beb457f3286ddef23e38aadcac288fa21804`](https://github.com/july-rust/BoardSI/commit/b425beb457f3286ddef23e38aadcac288fa21804). **별도 동일 Squash SHA의 code-main Push 8/8 Workflow SUCCESS**
    ([Qt #42807544148 재실행 attempt 2](https://github.com/july-rust/BoardSI/actions/runs/42807544148), [Native FEM #42807544267 재실행 attempt 2](https://github.com/july-rust/BoardSI/actions/runs/42807544267),
    [Numerical #42807544151](https://github.com/july-rust/BoardSI/actions/runs/42807544151) 및 나머지 5개도 성공). 초기 취소된 작업은 성공으로 간주하지 않았으며 재실행 완료 결과로 확인했다. **`.tgz`/`.tar.gz`/ZIP·`features.Z`, 벤더 내보내기 Oracle, 전체
    부품/Pin, Canonical Drill/Via·실제 도금·Qt Main IPC/Graph/Open/Short/SI는 여전히 NOT_VERIFIED**. Native archive *한정 transport* 완료를 전체 D_056_08 완료로 착각하지 않는다.
- [x] **D_056_08 USTAR/Directory Drill Tools·P Hit 단일 SOURCE 스냅샷 한정 인수** · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST**
  - **최초 체크일: 2026-11-10 KST**
  - . 공통 `audit_path_drill_source`는 실제 디렉터리 또는 제한된 비압축 USTAR를 하나의 검증된 File-map으로 읽고, 같은 스냅샷에서 MATRIX Drill Span·`layer/tools` 번호/규격 및 선택형 `features` Positive P Hit을 교차 검증한다. `SourceDrillAudit.hits=None`은
    미실행, `Some([])`는 실행 후 0건임을 구분한다. 합성 2층 Source→USTAR/Directory의 Tool/Hit/SHA 동등성 및 부정형 Tool 8/복수 PCB Root/TAR Header 변조는 Fail-Closed 테스트로 확인. [코드 PR #573](https://github.com/july-rust/BoardSI/pull/573)
    최종 HEAD `481429c4b8aa9f13bf2c84ade31265b0782ec236` **30/30 Checks SUCCESS**, 리뷰 미해결 0건, GitHub Verified 단일 부모 보호 Squash
    [main `c0c3d28293f5d43a8afca4239814632cb8eb7d89`](https://github.com/july-rust/BoardSI/commit/c0c3d28293f5d43a8afca4239814632cb8eb7d89). **독립 exact-code-main Push 8/8 워크플로 SUCCESS, 0 FAIL 확인** (SHA
    `c0c3d28293f5d43a8afca4239814632cb8eb7d89`). 이 체크는 **Source Drill 관측 증거 감사 한정**이다. 제조 도금/잔류 Barrel·전도층·Canonical Drill/Via·Qt Import/IPC·전기 Graph/Open/Short·SI·실제 벤더 Export 검증은 여전히 미인수.
- [x] **D_056_08 ODB++ Drill P-Hit → 제조 홀 SOURCE 관측형 데이터 계약 한정 인수** · **체크일: 2026-11-11 KST** · **근거일: 2026-11-11 KST**
  - **최초 체크일: 2026-11-11 KST**
  - . 이미 검증된 MATRIX Drill 명시적 구리층 Span, Layer Tool의 Tool 번호·규격·타입과 Positive P Hit의 원본 좌표·SHA·행 정보를 **단일 안전한 Directory/USTAR Source 스냅샷**에서 결합하여 새로운 `OdbSourceImport.source_holes: Vec<OdbSourceHole>` 및 공통
    `FabricationHole` 제조 *의도* 모델로 출력한다. 2/4/6/8/64층에서 exact rational metre 변환, Source-declared Plated / NonPlated / Unknown 삼중 상태, 원본 드릴/Hit/Matrix 증거, Legacy geometry-only 입력의 기존 `BoardModel` 완전 동일성, 미선언
    Span/Tool·Hole Diameter와 미지원 Special Backdrill 문법 Fail-Closed 합성 테스트를 실행했다. 진단 `boardsi-odb-import ... --source-holes`는 `--eda-nets`와 조합 가능하며 홀 개수와 미확인 제조 상태만 보고한다.
    [구현 PR #575](https://github.com/july-rust/BoardSI/pull/575) 정확 최종 HEAD `82785b24c6353a1a6cfce1b7a78751e9f004b7a5` **30/30 PR Checks SUCCESS**, Codex 리뷰 미해결 0, GitHub Verified 보호 단일 부모 Squash
    [code-main `63aa6e1509f4eb1648fdfe616100627bc3e64886`](https://github.com/july-rust/BoardSI/commit/63aa6e1509f4eb1648fdfe616100627bc3e64886) 직접 확인. **별도 정확한 code-main SHA
    `63aa6e1509f4eb1648fdfe616100627bc3e64886`의 독립 Push 8/8 워크플로 SUCCESS, 실패 0건 확인** ([Qt #42928135416](https://github.com/july-rust/BoardSI/actions/runs/42928135416),
    [Numerical #42928135470](https://github.com/july-rust/BoardSI/actions/runs/42928135470), [Docs #42928135434](https://github.com/july-rust/BoardSI/actions/runs/42928135434),
    [PH-06 #42928135464](https://github.com/july-rust/BoardSI/actions/runs/42928135464), [PH-07 #42928137126](https://github.com/july-rust/BoardSI/actions/runs/42928137126),
    [Native FEM #42928135544](https://github.com/july-rust/BoardSI/actions/runs/42928135544), [Lossless FEM #42928135469](https://github.com/july-rust/BoardSI/actions/runs/42928135469),
    [G0 #42928136207](https://github.com/july-rust/BoardSI/actions/runs/42928136207)). PR-head CI와 post-merge CI는 별도 이벤트·별도 검증으로 집계. 이 Source 관측 배열은 **Canonical BoardModel.drills/vias에 생성하지 않고**, 임의 전기
    Edge, 제조 도금·실물 Via 연결, Qt Artwork, 전역 Open/Short·SI를 일절 인증하지 않는다. [설계·제한 원장](architecture/d056-08-odb-source-adapter.md) 참조. 전체 D_056_08은 다음 미확인 체크 그대로 유지.
- [ ] **D_056_08 전체 인수 미완료:**
  - ODB++ 압축 `.tgz`/`tar.gz`/ZIP 및 `features.Z` (uncompressed USTAR 한정 별도 구현), 실제 Vendor Export 독립 Oracle, 복수 Island/곡선 OC/Negative 등 전체 Surface/Contour, **전체 EDA/Net/Component/Pin(선택형 FID 부분 이외)**,
    Via/Drill/Plating/Backdrill/Microvia, Stackup z·Material, Step/Repeat/다양한 Transform, 실시간 Qt Import→Versioned IPC→Main Artwork 및 본 입력의 전기 Graph/Open/Short와 SI는 **미구현 또는 NOT_VERIFIED**. 일부 정상 ODB++
    Full-Circle/Zero-Length record도 정확히 처리하는 대신 현재는 명시적으로 UNSUPPORTED. 전체 PCB/제조 도통의 성공으로 승격 금지. [제한 구현·미지원 명세](architecture/d056-08-odb-source-adapter.md)
  - [Issue #532](https://github.com/july-rust/BoardSI/issues/532)와 [상위 #520](https://github.com/july-rust/BoardSI/issues/520)은 계속 OPEN. D_056_09 진행 자체는 가능하지만 D_056_08을 DONE으로 판정하거나 네이티브 ODB++ 전체 지원으로 홍보하지
    않는다.

<a id="d_056_09"></a>
### D_056_09 — Native ODB++ 전체 통합 후속 및 압축 SOURCE Transport (2026-11-11 KST)

- [x] **ZIP STORED 컨테이너 SOURCE 한정 파서·CLI/Drill/Hole 공통 파일맵 경로 소프트웨어 인수** · **체크일: 2026-11-11 KST** · **근거일: 2026-11-11 KST**
  - **최초 체크일: 2026-11-11 KST**
  - . [코드 PR #579](https://github.com/july-rust/BoardSI/pull/579) 최종 정확 HEAD `7a086ab050c1347a6ab2305371a51c2a18300dab` **30/30 PR Checks SUCCESS**, 미해결 리뷰 0,
    [GitHub Verified 단일 부모 Squash `8069420d10403fb3a3f5faaab07df07ecd7bdb0f`](https://github.com/july-rust/BoardSI/commit/8069420d10403fb3a3f5faaab07df07ecd7bdb0f) 확인. 외부 압축 해제 없이 Local/Central/EOCD 및
    CRC-32·Path/Symlink·중복·크기·Root를 검증하는 **비압축 ZIP Method 0 STORED 한정** Parser와 공통 Directory/USTAR OdbFileMap·Source Hole·Drill Audit 경로를 연결했다. 정상 Synthetic 2-layer Source→Canonical 및 훼손 ZIP Fail-Closed
    테스트를 추가했다. 이 체크는 **Source Transport 소프트웨어 부분집합**만 해당한다. GZIP/TGZ/ZIP DEFLATE·features.Z·외부 벤더 Oracle·완전 CAD/Qt/전기·SI는 미인수. [설계·제한 문서](architecture/d056-09-odb-zip-transport.md) 참고.
- [x] **D_056_09 #579 코드-main Push 독립 CI 감사 완료** · **체크일: 2026-11-11 KST** · **근거일: 2026-11-11 KST**
  - **최초 체크일: 2026-11-11 KST**
  - . 정확한 보호 Squash SHA [`8069420d10403fb3a3f5faaab07df07ecd7bdb0f`](https://github.com/july-rust/BoardSI/commit/8069420d10403fb3a3f5faaab07df07ecd7bdb0f)의 독립 `event=push` 워크플로 **8/8 SUCCESS
  - 0 FAILED** 확인. [Qt runtime #42936772184](https://github.com/july-rust/BoardSI/actions/runs/42936772184)는 최초 취소된 Windows x64/ARM64 검사를 다시 실행하여 **최종 attempt 3 SUCCESS**, 별도
    [Numerical #42936771992](https://github.com/july-rust/BoardSI/actions/runs/42936771992), [Docs #42936772016](https://github.com/july-rust/BoardSI/actions/runs/42936772016),
    [PH-06 #42936772170](https://github.com/july-rust/BoardSI/actions/runs/42936772170), [PH-07 #42936772203](https://github.com/july-rust/BoardSI/actions/runs/42936772203),
    [Native FEM #42936772055](https://github.com/july-rust/BoardSI/actions/runs/42936772055), [Lossless FEM #42936771949](https://github.com/july-rust/BoardSI/actions/runs/42936771949),
    [G0 #42936772137](https://github.com/july-rust/BoardSI/actions/runs/42936772137) 모두 최종 SUCCESS. 취소된 초기 시도는 SUCCESS로 집계하지 않는다. 코드 PR HEAD 30/30과 코드-main Push 8/8은 별개의 증거이며, ZIP STORED SOURCE 한정 구현만의 CI
    인수이다. 문서 PR [#580](https://github.com/july-rust/BoardSI/pull/580)의 Verified Squash
    [docs-main `140349e29be84110a465d3ab95961264089dc849`](https://github.com/july-rust/BoardSI/commit/140349e29be84110a465d3ab95961264089dc849) 역시 독립 main Push **8/8 SUCCESS**.
- [ ] **D_056_09 전체 ODB++Design 인수 미완료:**
  - [전용 이슈 #578](https://github.com/july-rust/BoardSI/issues/578) OPEN. 압축 .tar.gz/.tgz/ZIP DEFLATE·features.Z, 실제 Vendor Export 독립 Fixture 및 교차 포맷 Oracle, 복수 Island/곡선/Negative/Step Repeat·Transform, EDA
    PKG/PIN/Component, Canonical Drill/Via·Padstack/Plating·Stackup Material/z, 메인 Qt Import→Versioned IPC→Artwork/Picking, Source 근거의 전기 Graph/Open/Short와 SI Solver 입력 적합성 **NOT_VERIFIED 또는 미구현**.
    D_056_08에 이미 체크된 부분집합을 전체 Stage 완료로 확장하지 않는다. 상위 [#10](https://github.com/july-rust/BoardSI/issues/10), [#532](https://github.com/july-rust/BoardSI/issues/532),
    [#520](https://github.com/july-rust/BoardSI/issues/520)도 열린 상태로 유지한다.

**D_056_01~13 후속 추적 (아래 표는 계획·상태 색인이며 별도 체크 항목/진행률 집계가 아님):**

| 개발 단계 | 기능별 작업 | 이슈 및 완료 관문 |
| :--- | :--- | :--- |
| D_056_01 | 기존 Source/Canonical/Qt 감사 및 증거 계약 | #520; 문서 PR #533 Verified Squash 및 Push 인수 완료 |
| D_056_02 | 제한 PADS Layer별 Copper 후보 Graph (#535) | 후보/Span 미검증 분리 및 2/4/6/8/64층 코드·Push 한정 완료; 후속 전기 Graph #529는 D_056_03에서 SOURCE-scope 소프트웨어 인수 |
| D_056_03 | SOURCE 기반 다층 전기 그래프·Net 충돌·Open 후보·Cut-set (한정 인수) | PR #538/#540/#542/#543/#547 코드 병합, **45개 독립 Oracle PASS**; 지원 Source의 양성 Copper 접촉·불확실 구간의 Fail-Closed, 코드/Qt CI 및 보호 Squash 인수. #529 SOURCE-범위 소프트웨어 인수 완료, 보드 전역·제조·타 CAD는 후속 #530/#531/#532/#520 |
| D_056_04 | Canonical Layer별 Padstack / Rust→Qt V1 IPC (SOURCE 한정) | 코드 PR #549 보호 Verified Squash; 36/36 PR Checks PASS; 층별 Pad/Drill/Source SHA / Fail-Closed 한정 인수. 메인 GUI/취소 검증은 D_056_05 #531 |
| D_056_05 | Qt CPU/RHI / Legacy V1~V14 통합 | [코드 PR #551](https://github.com/july-rust/BoardSI/pull/551) 35/35 Checks · 11/11 PR Workflow, Qt Linux 81/81 CTest 및 4플랫폼 Native PASS; **정확한 code-main Push 8/8 SUCCESS**; docs-only PR #552 및 Push 검증 후 #531 종료 |
| D_056_06 | 공식 PADS RT/ST Thermal Spokes + Canonical V2·Qt 메인 UI | [PR #553](https://github.com/july-rust/BoardSI/pull/553) 12/12 CI, 4플랫폼 Native PASS, 3개 리뷰 결함 수정, code-main `e034335b` Squash; 독립 code-main Push **8/8 SUCCESS** (Qt attempt 2) + 문서-main `04374fe3` Push **8/8 SUCCESS**; 제조 도통·전역 Open/Short/SI 별도 |
| D_056_07 | PARTDECAL 한정 PTH/NPTH/Slot Source·Canonical 제조 의도 모델 + Legacy IPC Fail-Closed | [PR #555](https://github.com/july-rust/BoardSI/pull/555) 38/38, [#558](https://github.com/july-rust/BoardSI/pull/558) 30/30, [#559](https://github.com/july-rust/BoardSI/pull/559) 30/30 · 모두 Verified Squash; #530 CLOSED(기술적 전체 인수 의미 아님), #520 OPEN · **전체 미인수**, Native Backdrill/Microvia·Qt IPC/Graph 미지원 |
| D_056_08 | ODB++ SOURCE subset: Directory·Matrix/Positive L/P/A/S·Hole + 선택형 EDA NET/SNT/FID → Canonical·SHA·Fail-Closed | [#562](https://github.com/july-rust/BoardSI/pull/562), [#564](https://github.com/july-rust/BoardSI/pull/564), [#566](https://github.com/july-rust/BoardSI/pull/566), [#568](https://github.com/july-rust/BoardSI/pull/568), [#569](https://github.com/july-rust/BoardSI/pull/569) 각 코드 PR 30/30 Checks·Verified Squash 및 각각 독립 code-main Push 8/8 성공; [#571](https://github.com/july-rust/BoardSI/pull/571), [#573](https://github.com/july-rust/BoardSI/pull/573) 역시 보호 병합·독립 Push 8/8 확인; [#575](https://github.com/july-rust/BoardSI/pull/575) 30/30 Checks·Verified Squash, 독립 정확 code-main `63aa6e15` Push 8/8 SUCCESS 확인; **8개 SOURCE 한정 코드 인수(#571 USTAR, #573 통합 Drill, #575 제조 홀 관측 포함) / 전체 ODB++·Qt·Graph 미인수**, #532 OPEN |
| D_056_09 | D_056_08 ODB++ 잔여 Native Import / Qt / Graph / SI 전체 인수 | [#578](https://github.com/july-rust/BoardSI/issues/578); [#579](https://github.com/july-rust/BoardSI/pull/579) ZIP STORED SOURCE 한정 30/30 Verified Squash · code-main Push 8/8 SUCCESS, 전체 단계 OPEN |
| D_056_10 | **역사적 예정 번호 (신규 D 실행 금지)** — IPC-2581 → E_025 | #532; 네이티브 형식 검증 |
| D_056_11 | **역사적 예정 번호 (신규 D 실행 금지)** — KiCad → E_026 | #532; 네이티브 형식 검증 |
| D_056_12 | **역사적 예정 번호 (신규 D 실행 금지)** — Gerber + Drill → E_027 | #532; Net/Stackup 불명은 UNKNOWN |
| D_056_13 | **역사적 예정 번호 (신규 D 실행 금지)** — 교차 형식 회귀 → E_028 | #532, #520; 모든 범위 검증 후 종료 |

**주의:** D_056_01 감사 계약·D_056_02 Copper 후보 Graph·D_056_03 SOURCE 범위 전기 설계 Graph·D_056_04 Canonical IPC·D_056_05 메인 Qt Artwork **5개 단계의 별도 인수**와, D_056_06 **SOURCE 한정 코드 구현·보호 병합 1개**를 구분한다. D_056_06 정확한
  code-main 및 docs-main Push가 각각 8/8 SUCCESS로 검증되었지만 제조/전역 Open/Short/SI와 별개의 범위이며, D_056_07 코드의 Source 한정 구현만으로 전체 D_056 인수를 표시하지 않는다. D_056_07 및 D_056_08은 SOURCE 범위 코드에 한정해 별도 증거를 확보했으나 전체 단계 인수는 아니며,
  D_056_07~13도 전체 미인수로서 상위 D_056 `- [ ]`는 유지한다. 특히 D_056_02의 기하 후보는 실제 제조된 Via의 도통, 글로벌 Short/Open 또는 SI Solver PASS를 뜻하지 않는다. 역사적 PR·CI 근거는 변경하지 않는다.
