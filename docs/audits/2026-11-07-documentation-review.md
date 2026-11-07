# BoardSI 저장소 전체 문서 감사 및 정리 보고서

**점검일:** 2026-11-07 KST  
**감사 대상 기준점:** GitHub \`main@0549393b8912fbe9bfcde265de4f58560c62e3ee\` (문서 정리 착수 시점)  
**추적:** [문서 품질 Issue #514](https://github.com/july-rust/BoardSI/issues/514)  
**원칙:** 과거 수치 결과·PH-08 FAIL·원본 계약·완료 근거는 변경하지 않고 **현재 읽는 방법과 오류 문구**를 수정합니다.

## 1. 검토 대상과 방법

| 인벤토리 | 대상 수 | 확인 방법 |
| :--- | ---: | :--- |
| 저장소 전체 Markdown (\`.md\`) | **182개** | GitHub recursive Git tree 전수 수집, 해당 파일 전체 Fetch, 구조·상대경로 링크 스캔 |
| \`docs/validation\` 내 Markdown | **159개** | 일반 안내서 \`README.md\` 1개 + **역사적 근거 파일 158개**, 새 [전체 색인](../validation/INDEX.md)으로 분류 |
| \`docs/\` 내 비-Markdown JSON | **7개** | 파일을 읽어 **JSON 구문 파싱 7/7 PASS**. 스키마 의미·수치 오라클 전체 재계산을 뜻하지 않음 |
| 핵심 안내서 | 루트/Importer/Qt/Architecture/Validation/Project/Contribution/Security/Spec/Checklist | 전문·현재 코드/이슈 대조, 오래된 설명·표현·탐색 경로·완료 Gate 확인 |
| 상대경로 대상 검사 | 기존 Markdown의 **일반 인라인 링크** | Git tree의 실제 Blob과 디렉터리 대상 존재를 대조. 일반 상대 링크에 대해 **확정된 누락 파일 없음** |

**범위와 주의:** 이 검사는 모든 파일의 모든 문장을 물리 시험으로 다시 입증한 것이 아닙니다. 일반 Markdown 인라인 \`[text](path)\` 링크의 경로·구조를 자동 점검하고, 주요 관리 문서를 의미적으로 깊이 검토했습니다. **GitHub 자동 생성 Heading Anchor, Reference-style 링크, 외부 URL의 현재 접속성, HTML 내부 링크, 이미지 라이선스와 모든 과거 수치의 재계산은 별도 전수 인증 대상이 아닙니다.** 과거 \`prototypes/m2-lossless-fem/\` 링크처럼 파일이 아닌 **디렉터리**를 향하는 링크도 존재하므로 Blob 목록만으로 오탐하지 않습니다.

## 2. 발견한 주요 문제와 처리

| 심각도/분류 | 확인된 문제 | 수행한 문서 정정 | 제품 기능 변경 여부 |
| :--- | :--- | :--- | :--- |
| **높음 — 사실 정확성** | [Importer README](../../crates/boardsi-import/README.md)가 \`signature probe\`의 '기하 파싱 불가'를 전체 Importer 상태처럼 제시 | \`probe\`와 제한 \`artwork\` 별도 명령, D_054/D_055 SOURCE-local 기하 기능, 초기 P2-A 역사적 한계 분리 | 없음 |
| **높음 — 오래된 상태** | 과거 G5 [PR #35](https://github.com/july-rust/BoardSI/pull/35)가 현재 열린 Draft처럼 해석될 수 있음 | [프로젝트 관리](../project-management.md)와 [아키텍처](../architecture/README.md)에 **2026-11-07 기준 Closed / Not Merged** 명시 | 없음 |
| **높음 — 완료 오해** | 합성 FEM PASS·SOURCE Copper 접촉·Qt 네이티브 테스트와 실제 PCB SI 완제품 검증의 경계가 분산 | [프로젝트 홈](../../README.md), [문서 홈](../README.md), [검증 정책](../validation/README.md)에서 동일한 상태 용어·승격 금지 규칙 통일 | 없음 |
| **중간 — 탐색성** | 검증 159개 Markdown 중 하나씩 검색해야 하는 높은 탐색 비용 | [158개 기록 색인](../validation/INDEX.md), 목적별 [문서 안내](../README.md), 모듈별 [아키텍처 지도](../architecture/README.md) | 없음 |
| **중간 — 실행 온보딩** | Rust/Qt/Importer 명령과 P0/P2-A 시점 차이가 하위 README에 분산 | [Getting started](../getting-started.md) 신설; [Qt README](../../apps/boardsi-qt-prototype/README.md) 역사적 P0 제한 표시 | 없음 |
| **중간 — 기술 설명 누락** | Via 비중심 도금 패드, Pad 변·모서리, 원호·직선 범용 SOURCE 접촉의 공통 수학적 의미 설명 부족 | [Trace–Via–Pad 기하 계약](../architecture/trace-copper-contact.md) 신설; Drill·Layer·Net·접선 Fail-Closed·미지원 범위 설명 | 없음 |
| **중간 — 문서 관리** | 단일 체크리스트 원칙은 있으나 작성/최신성/증거 SHA/역사적 상태의 공통 표준을 찾기 어려움 | [문서 표준](../documentation-standards.md) 신설; 기여 지침과 역사적 M2 이력 안내 보강 | 없음 |
| **중간 — 보안 안내 부족** | [Security](../../SECURITY.md)가 단일 짧은 단락에 그쳐 입력·데이터·신고 채널을 구분하기 어려움 | 보안 경계, 보고 원칙, 공격면, 라이선스/릴리즈 상태를 항목화. **전용 신고 이메일/SLA는 만들지 않음** | 없음 |

## 3. 보호한 정본과 역사적 증거

- **요구사항 v1.0:** [원본 개발 사양서](../specification/BoardSI_Development_Spec_v1.0.md)는 설계 목표와 U01–U48/T01–T54의 정본으로 보존. 문서 편집만으로 인수 조건을 변경하지 않습니다.
- **단일 체크리스트:** [A_036 현황 원장](../development-checklist.md)은 기존 최초 체크일·검증 근거·단계별 범위를 그대로 유지. 두 번째 상태표로 대체하지 않았습니다.
- **A/B/C/D 및 M0/M2/PH-08:** 기존 158개 개별 실험 보고서의 원문과 당시 FAIL 기록을 유지. 새 색인은 파일을 찾기 쉽게 연결할 뿐 시험 결과를 변조하지 않습니다.
- **PH-08 Original HMM:** Gate Q **FAIL**, AC-P1 PENDING, T/R PROPOSED_NOT_FROZEN, Physical BOUND_NOT_CERTIFIED를 **PASS로 승격하지 않습니다**.
- **실제 동작의 물리적 미완:** D_054/D_055 구현은 SOURCE-local Copper 접촉 **후보**이지 전체 Electrical Connectivity/제품 SI Solver 검증이 아닙니다.
- **배포/브랜딩:** 정식 릴리즈/설치 라이선스 승인·실제 제조 PCB 검증은 별도 Gate이며, 기존 승인 Option B SVG의 바이트나 디자인은 손대지 않았습니다.

## 4. 문서 체계의 읽기 경험을 검증하는 기준

| 독자의 질문 | 이제 확인할 곳 |
| :--- | :--- |
| **BoardSI는 무엇이고 어떤 것이 실제로 동작하나?** | [루트 README](../../README.md) → [문서 홈](../README.md) |
| **실행 파일을 어떻게 빌드하고 제한 PADS 진단을 돌리나?** | [개발 가이드](../getting-started.md) |
| **각 모듈이 어떻게 연결되나?** | [아키텍처 지도](../architecture/README.md) |
| **Trace가 Via/Pad의 중심에서 벗어나도 되는가?** | [기하 접촉 계약](../architecture/trace-copper-contact.md) |
| **무엇을 어디까지 검증했는가?** | [공식 체크리스트](../development-checklist.md) → [158개 검증 색인](../validation/INDEX.md) |
| **어느 테스트가 실제 PCB 물리 성능을 증명하는가?** | [검증 정책](../validation/README.md) → [PH-08 원본 증거](../validation/m2-g2-ph08-stage5b-original-replay-evidence.md) |
| **어떻게 새 보고서를 일관성 있게 추가하나?** | [작성 표준](../documentation-standards.md), [기여 지침](../../CONTRIBUTING.md) |

## 5. 검증 및 이번 수정에서 아직 확인되지 않은 것

1. **문서 전용 PR CI:** 필요 체크와 Docs-only classifier가 실행되고, 네이티브 Windows/macOS Rust/Qt/FEM Job이 SKIP되는지 **최종 PR HEAD의 실행 결과**로 판단합니다. 실행 전 PASS로 쓰지 않습니다.
2. **문서 경로·Anchor:** 추가된 문서를 포함한 **최종 PR Tree**에서 경로를 재검합니다. 외부 링크의 실시간 상태와 GitHub slug 자동 생성 Anchor는 별도 확인 전 \`NOT_VERIFIED\`입니다.
3. **코드 변경:** 이번 작업에서 Solver/Importer/Qt 구현은 변경하지 않습니다. 단순 문서 수정으로 새 제품 기능이나 공식 U/T 검증 결과를 만들지 않습니다.
4. **향후 유지관리:** 검증 보고서가 증가하면 **새 색인 1곳만 추가**하고, 같은 숫자/상태를 여러 독립 문서로 복제하지 않습니다. 현재 완료 수치는 단일 공식 체크리스트가 우선합니다.
5. **별도 기술 과제:** 정확한 전기 연결성·Custom Pad/Plane/Thermal·실측 PCB/제품 SI 검증은 문서 보완으로 해결되지 않으며, 실제 기능·독립 시험 작업이 필요합니다.

**결론:** 기존 기록의 양은 충분했으나, **탐색 계층과 시점/검증 범위 표기**가 분산된 것이 핵심 문제였습니다. 정보량을 무조건 늘리는 대신 한 개의 정본 현황·한 개의 문서 홈·한 개의 검증 색인·모듈별 기술 계약으로 역할을 명확히 했습니다.
