# BoardSI 문서 안내 | Documentation home

> **이 문서부터 읽으세요.** BoardSI는 오프라인 PCB/BGA Substrate 신호 무결성 분석기를 목표로 개발 중인 **Rust + Qt 기반 프리뷰 프로젝트**입니다. 실행 가능한 제한적 GUI/Importer와 수치 연구 코드는 존재하지만, **실제 PCB의 전체 전기 연결성, 물리 RLGC·S-Parameter 제품 해석, 배포판은 아직 정식 검증되지 않았습니다.**
>
> **상태 기준:** 2026-11-07 KST의 GitHub 코드·이슈·검증 기록. 변경되는 완료 수치와 증거 SHA의 **유일한 공식 현황 원장**은 [개발 체크리스트](development-checklist.md)입니다. 이 페이지를 별도 두 번째 체크리스트로 사용하지 않습니다.

## 1. 원하는 작업에서 시작하기

| 읽는 사람 / 목적 | 먼저 읽을 문서 | 다음에 읽을 문서 |
| :--- | :--- | :--- |
| **프로그램을 처음 접하는 사람** | [프로젝트 소개](../README.md) → 이 페이지 | [현재 지원 범위](#2-현재-구현-상태와-검증-경계) |
| **실행·개발을 시작하는 사람** | [개발 환경·실행 안내](getting-started.md) | [Qt 프리뷰 상세 안내](../apps/boardsi-qt-prototype/README.md), [Importer 상세 안내](../crates/boardsi-import/README.md) |
| **기능·일정을 판단하는 사람** | [개발 체크리스트 — 공식 구현 현황](development-checklist.md) | [개발 사양서 v1.0](specification/BoardSI_Development_Spec_v1.0.md), [제품 전달 계획](preview-delivery-playbook.md) |
| **코드 구조를 이해하려는 개발자** | [아키텍처 지도](architecture/README.md) | [Canonical Board Model](architecture/canonical-board-model.md), [Trace–Via–Pad 접촉 규칙](architecture/trace-copper-contact.md) |
| **시험 결과를 검토하는 사람** | [검증 방법 및 상태 해석](validation/README.md) | [전체 검증 보고서 색인](validation/INDEX.md), [PH-08 물리 게이트 근거](validation/m2-g2-ph08-stage5b-original-replay-evidence.md) |
| **기여·문서 수정을 준비하는 사람** | [기여 지침](../CONTRIBUTING.md) | [문서 작성·증거 정책](documentation-standards.md), [보안 정책](../SECURITY.md) |
| **프로젝트 운영·변경 이력을 찾는 사람** | [프로젝트 관리](project-management.md) | [개발 이력](development-history.md), [2026-11-07 문서 감사](audits/2026-11-07-documentation-review.md) |

## 2. 현재 구현 상태와 검증 경계

아래는 **기능이 존재하는지**와 **제품 인수까지 완료됐는지**를 구분한 요약입니다.

| 영역 | 확인된 구현·한정 시험 | 제품 인수에서 남은 일 |
| :--- | :--- | :--- |
| **기본 프로그램·GUI** | 네이티브 Qt Widgets/Rust Worker 개발 프리뷰, 제한적 Qt CPU/RHI Artwork 및 IPC 회귀시험 | 완전한 사용자 작업 흐름, 배포·라이선스, 제품 응답성·장애 복구 게이트 |
| **PCB Import** | 공개 합성 PADS Layout ASCII **V9.4 MILS의 제한 서브셋**: Trace·원호·일부 Via·Pad·Copper 형상 | 실제 제조 CAD 파일의 광범위 호환성, ODB++ 등의 추가 어댑터, 전체 손실/미지원 보고 |
| **Copper 접촉** | [D_054/D_055](validation/d054-d055-general-trace-contact-qualification.md): 임의 방향 직선/원호 및 **중심이 아닌** Via·SMD Pad 접촉의 SOURCE 국소 후보 계산 | 모든 Padstack/Custom Shape/Pour/Thermal, Canonical **전체 전기 연결 그래프·Qt 연결성 표시**, 실제 Open/Short 판정 |
| **수치 코어** | 입력으로 제공된 균일 RLGC의 scalar 2-port 계산 프로토타입; 제한 합성 PEC/lossless FEM 쿠폰 | 보드 기하·재료→물리 RLGC 추출, 결합선/손실 모델, 검증된 DC–30 GHz S-parameter 제품 해석 |
| **검증·릴리즈** | 공개 합성 픽스처, 네이티브 Rust/Qt CI 및 일부 독립 수학 참조 비교 | 공식 사양 U01–U48/T01–T54 완료, 실물 보드 대조, 최종 패키징·릴리즈 허가 |

**변경하지 않는 핵심 사실:** M0 정식 완료 게이트는 **BLOCKED**입니다. M1은 제한 Preview 단계이고, M2의 일부 FEM **연구 쿠폰 PASS**는 최종 PCB SI Solver PASS가 아닙니다. **PH-08 원본 HMM Gate Q = FAIL**은 별도로 유지됩니다. 해당 연구 이슈가 행정적으로 종료됐어도 물리 실패 판정은 바뀌지 않습니다. 상태 근거: [공식 체크리스트](development-checklist.md), [M0 완료 게이트](validation/m0-completion-gate.md), [PH-08 원본 재검증](validation/m2-g2-ph08-stage5b-original-replay-evidence.md).

## 3. 문서의 우선순위와 역할

| 분류 | 정본(수정 원칙) | 의미 |
| :--- | :--- | :--- |
| **요구사항과 공식 인수 기준** | [개발 사양서 v1.0](specification/BoardSI_Development_Spec_v1.0.md) | **무엇을 만들어야 하는지**의 기준. 코드나 PR PASS만으로 기준 변경 불가 |
| **날짜별 기능 상태·미완료 항목** | [개발 체크리스트](development-checklist.md) | **무엇이 어디까지 검증됐는지**의 단일 현황 원장. 체크 날짜/근거 SHA 필요 |
| **설계와 구현 계약** | [아키텍처 문서](architecture/README.md) 및 실제 GitHub 소스 | **어떻게 동작하는지**. 설명·코드 불일치 시 코드를 확인하고 문서 정정 |
| **증거와 과거 결과** | [검증 원장](validation/INDEX.md), [역사적 감사](audits/) | **어떤 조건에서 무엇을 시험했는지**. 옛 FAIL·실패 원본을 삭제하거나 소급 PASS로 변경 금지 |
| **운영·릴리즈 규칙** | [개발·CI 정책](project-management.md), [기여 지침](../CONTRIBUTING.md) | **어떤 검증으로 병합/배포를 승인하는지**. CI 합격과 제품 배포 승인 별개 |

동일 사실을 여러 파일에 숫자로 복제하지 않습니다. 과거 보고서에 쓰인 '현재'는 그 문서 **작성 당시**의 상태입니다. 최신 상태는 위 단일 공식 원장에서 확인합니다.

## 4. 읽기 순서: 설계부터 기능 검증까지

1. [요구사항과 사용자 흐름](specification/BoardSI_Development_Spec_v1.0.md) — Import → Artwork → Stackup/Reference → Port/Scenario → Frequency → 계산 → 결과
2. [Canonical 데이터 구조](architecture/canonical-board-model.md) — 원본 좌표, 층, 재료, Net, Provenance 및 무결성
3. [Importer 설명](../crates/boardsi-import/README.md) 및 [Copper 기하 계약](architecture/trace-copper-contact.md)
4. [Qt/Worker 프로세스 계약](architecture/worker-ipc-qt-prototype.md)와 [개발자 실행 안내](getting-started.md)
5. [수치 계약](architecture/numerical-contract.md) → [독립 검증](validation/README.md)
6. [공식 미완료 항목](development-checklist.md) 확인 후 해당 [검증 보고서](validation/INDEX.md)의 파일·PR·SHA 재현

## 5. 주요 용어와 판단 규칙

- **SOURCE-local:** 입력 PADS 원본에 직접 결합된 *일부 기하 객체 간 접촉 후보*. 전체 PCB 전기적 Net 연결성이나 실물 제조 도통을 증명하지 않습니다.
- **Positive-area contact:** 동일 층의 지원 도체 형상이 양의 구리 면적으로 겹치는 경우. 경계만 맞닿거나 수치적으로 불확실한 접촉은 **Fail-Closed**입니다.
- **Preview PASS:** 공개 합성 데이터 또는 제한된 UI/IPC 시나리오에서 재현 가능하게 동작했다는 뜻입니다.
- **Research coupon PASS:** 수치 해석의 제한된 모델/경계조건에서의 시험 통과입니다.
- **Product acceptance PASS:** 공식 U/T 시험, 실제 유효성·배포 요건을 모두 별도로 충족한 경우에만 사용합니다.
- **NOT_RUN / NOT_VERIFIED / BLOCKED / FAIL:** 각각 실행하지 않음 / 확정 검증 근거 없음 / 선행 게이트 미충족 / 실제 실패. 서로 대체하지 않습니다.

상세 표기 규칙은 [문서·검증 작성 표준](documentation-standards.md)과 [프로젝트 보안](../SECURITY.md)을 따릅니다.

---

**문의와 작업 출처:** GitHub 저장소 [Issues](https://github.com/july-rust/BoardSI/issues)와 [Pull requests](https://github.com/july-rust/BoardSI/pulls). 회사 PCB·Netlist·Touchstone 원본은 Issue·Actions·첨부에 등록하지 마세요.
