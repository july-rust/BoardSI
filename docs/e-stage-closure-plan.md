# BoardSI E 단계 — 잔여 작업 종결 계획

> 기준: **2026-11-11 KST**. 이 문서는 E_001 실행 전 작성한 계획을 유지하며, 아래에 E_001의 검증된 종결 이력도 추가한 **계획·인수 경계 설명서**다. 체크 상태의 유일한 정본은 [단일 개발 체크리스트](development-checklist.md)다. 진행 불가·외부 근거 부족은 [별도 의존성 원장](e-stage-blocked-register.md)에서 관리한다.

## 목적

**D 실행은 D_056_09를 끝으로 종료한다.** 기존 체크리스트에 남은 D_056_10~13은 과거에 만든 *후속 예정 번호*라는 역사적 기록만 보존한다. 실제 신규 실행은 E_001부터 시작한다. D_056 전체 기능이나 ODB++ 전체가 인수된 것으로 표기하지 않는다.

2026-11-11 E 계획 수립 당시 GitHub에서 **열린 이슈 30건·열린 PR 0건**을 확인했다(역사적 스냅샷). [D_056_09 관련 #578](https://github.com/july-rust/BoardSI/issues/578)은 여전히 Open이며, ZIP STORED 부분 기능의 코드 병합 [#579](https://github.com/july-rust/BoardSI/pull/579)은 전체 CAD/Qt/전기/SI 인수가 아니다.

**원칙:** 큰 상위 Issue를 하나의 E 작업으로 몰아넣지 않는다. 한 번호는 한 가지 검증 가능한 완결 산출물만 포함한다. 조건을 못 채우면 완료로 꾸미지 않고 정확한 차단 근거와 재개 조건을 기록한다. 닫을 수 없는 상위 이슈는 정상적인 추적 이슈로 남기되, **시작한 E 단위의 PR과 증거 정리는 반드시 마무리**한다.

## 각 E 번호의 공통 완료 정의

1. 시작 전: 최신 main SHA, 열린 이슈/PR, 원본 규격·Source 문법, 기존 A~D 테스트와 실제 미지원 범위를 확인한다.
2. 구현: **하나의 소형 기능/문서 감사**, 정/부정/UNKNOWN 반례, 같은 범위의 자동 검증을 완료한다. 임의 PCB 층수·비아 위치별 분기를 추가하지 않고 범용 Source/Canonical 알고리즘을 우선한다.
3. 증거: 정확한 **PR 최종 HEAD**의 필수 Checks/독립 리뷰/미해결 Threads 0을 확인하고, 기존 보호 규칙을 우회하지 않는다. **GitHub Verified Squash-only** 병합, **정확한 새 main SHA의 Push CI**를 PR CI와 구분하여 확인한다.
4. 문서: 해당 소형 작업에서 **하나의 체크리스트**와 직접 관련 문서를 함께 갱신한다. 작업 종료 뒤 문서 전용 미완 PR을 남기지 않는다. 최초 체크일·근거일·SHA·한계는 삭제하지 않는다.
5. 종료 판정: E 번호가 정한 **소형 범위**의 전 과정을 종료했을 때만 완료. 상위 Issue의 **전체** 수락 조건이 모두 충족될 때만 Issue Closed. 이슈를 닫으려고 근거 없는 PASS를 만들지 않는다.
6. 장애: 독립 Fixture·실기기·라이선스·물리 기준 등 필수 전제가 부재하면 [차단 원장](e-stage-blocked-register.md)에 상태와 다음 조건을 정리한다. 미완 기능/미통과 수락시험을 완료라고 쓰지 않는다. 기존 PR이 유효하지 않으면 리뷰 근거를 보존해 안전 종료하고 후속 새 단위로 재계획한다.

## E_001 실행 기록 — 2026-11-11 KST

- **단위 종결:** PH-07의 #41 사후 감사 자체는 이전 증거와 현재 Ruleset 검증으로 **한정 완료**. [독립 감사 근거](validation/e001-ph07-post-audit-closure.md), [PR #583](https://github.com/july-rust/BoardSI/pull/583), [단일 체크리스트](development-checklist.md).
- **서로 다른 의무 분리:** #43 보호 규칙 조치는 완료; #46 Qt 타이머 원인 분석은 **여전히 미해결**이며 [EB-02](e-stage-blocked-register.md)와 E_029에만 인계. #41 종결은 #46의 기술적 해결을 뜻하지 않는다.
- **증거 경계:** 기존 PH-07 정적 문서·과거 시험 결과는 기록으로 확인했으나 E_001에서 FEM/FVM을 신규 실행했다고 주장하지 않는다. 과거 원본 FAIL, PH-08 Gate Q FAIL, 정식 제품 U/T 미인수는 그대로 유지한다.
- **운영 완료 관문:** PR #583의 최종 HEAD, 리뷰 스레드, Verified Squash, main Push CI 증거는 [#41 종결 댓글](https://github.com/july-rust/BoardSI/issues/41)에 확정 기록한다.

## 작은 단위 실행 순서 — E_001 ~ E_030

**각 행은 단일 작업에서 닫을 수 있도록 설계한 범위**다. 실제 실행 시 더 큰 수정이 필요하면 그 번호의 작은 수락 계약만 끝내고, 나머지를 새 번호로 분리한다. 독립 E_001–E_012는 실행 가능성이 높은 항목 우선 선택 가능하다.

| 번호 | 단일 완료 산출물 | 관련 열린 이슈 | 해당 단위 종료 기준 |
| :--- | :--- | :--- | :--- |
| **E_001** | PH-07 사후 감사 증거 재검토와 안전한 #41 종결 판단 | [#41](https://github.com/july-rust/BoardSI/issues/41), [#46](https://github.com/july-rust/BoardSI/issues/46) | 필수 규칙 적용·후속 #46 구분·과거 PASS/FAIL 보존; 조건 충족 때만 #41 종료 |
| **E_002** | M0 F1 요구사항 U/T 추적성 누락·충돌 1묶음 정리 | [#2](https://github.com/july-rust/BoardSI/issues/2) | U01–U48/T01–T54 원본·테스트 대응표의 단일 정정과 근거 |
| **E_003** | Rust/Qt/IPC 결정·버전 호환 근거 한 묶음 인수 | [#3](https://github.com/july-rust/BoardSI/issues/3) | 실제 코드·반례·Native 증거로 확인/미확인 분리 |
| **E_004** | Canonical Layer/Via Source provenance 한 계약 검증 | [#4](https://github.com/july-rust/BoardSI/issues/4), [#520](https://github.com/july-rust/BoardSI/issues/520) | 층수 하드코딩 없이 Source 근거·Unknown 보존 회귀 |
| **E_005** | 비차단 Worker의 취소 또는 late-response 한 경로 인수 | [#5](https://github.com/july-rust/BoardSI/issues/5) | 격리·stale 거부·Timeout 반례·적용 Native CI |
| **E_006** | 세 Native OS Toolchain·실행 근거 최신성 감사 | [#6](https://github.com/july-rust/BoardSI/issues/6) | Windows x64/ARM64, macOS ARM64 실제 실행 SHA별 증거 |
| **E_007** | 공개 Fixture 출처·해시·재배포 권한 한 묶음 정합화 | [#7](https://github.com/july-rust/BoardSI/issues/7) | 증명 없는 상용 데이터는 공개/승인으로 표기하지 않음 |
| **E_008** | 입력 판별 및 readiness 한 Source 종류 보강 | [#8](https://github.com/july-rust/BoardSI/issues/8) | 미지원/누락/악성 소스 반례와 UNKNOWN 보존 |
| **E_009** | PADS 소스 문법 1가지 지원 공백 수정 | [#9](https://github.com/july-rust/BoardSI/issues/9) | 공개 출처·공식 문법·변이 반례·A–D 회귀 |
| **E_010** | Qt 메인 Import→IPC→Picking 한 종류 회귀 | [#11](https://github.com/july-rust/BoardSI/issues/11) | 실제 메인 Qt CPU/RHI·재Import·Source ID 무결성 |
| **E_011** | Stackup 편집/저장/재Import 무효화 한 경로 검증 | [#12](https://github.com/july-rust/BoardSI/issues/12) | Dk/Df/두께가 Source 미기재면 Unknown, FR4 암묵 대체 금지 |
| **E_012** | 실제 GPU 백엔드 1개 플랫폼 계측 | [#165](https://github.com/july-rust/BoardSI/issues/165) | 실기기 없으면 미착수·별도 차단; RHI synthetic은 대체 불가 |
| **E_013** | ODB++ ZIP DEFLATE Method 8 한 경로 구현 | [#578](https://github.com/july-rust/BoardSI/issues/578) | CRC·크기·ZIP 폭탄·Traversal 반례 및 STORED 회귀 |
| **E_014** | ODB++ tar.gz/tgz 한 Source FileMap 경로 | [#578](https://github.com/july-rust/BoardSI/issues/578) | Decompress 제한·USTAR/Directory 내용 동등성·오류 반례 |
| **E_015** | ODB++ features.Z 문법 한정 파서 | [#578](https://github.com/july-rust/BoardSI/issues/578) | 명세/공개 Fixture 선행, 미확인 압축은 Fail-Closed |
| **E_016** | ODB++ Copper Negative 또는 Island 규칙 하나 | [#578](https://github.com/july-rust/BoardSI/issues/578), [#10](https://github.com/july-rust/BoardSI/issues/10) | Source Polarity·Source ID·독립 도형 반례 |
| **E_017** | ODB++ Step Repeat/Transform 규칙 하나 | [#578](https://github.com/july-rust/BoardSI/issues/578) | 정확한 좌표·반복 원본·중복/오버플로 반례 |
| **E_018** | ODB++ EDA Package/Pin 매핑 한 종류 | [#578](https://github.com/july-rust/BoardSI/issues/578) | Pin 이름·Net 레이블만으로 전기 연결 추론 금지 |
| **E_019** | ODB++ 제조 홀에서 Canonical Via 후보 한 종류 | [#578](https://github.com/july-rust/BoardSI/issues/578) | Plating/Span 미확인은 UNKNOWN; Hole=도금 Via 단정 금지 |
| **E_020** | ODB++ Stackup Material/z provenance 1개 경로 | [#578](https://github.com/july-rust/BoardSI/issues/578) | 재료/두께 원본 미기재 시 임의 추정 금지 |
| **E_021** | 메인 Qt에 ODB++ 공개 합성 보드 1개 비차단 Import | [#578](https://github.com/july-rust/BoardSI/issues/578), [#11](https://github.com/july-rust/BoardSI/issues/11) | Source→Versioned IPC→Artwork/Picking·취소·ReImport |
| **E_022** | ODB++ SOURCE 한 종류 국소 전기 Graph 반례 | [#578](https://github.com/july-rust/BoardSI/issues/578), [#520](https://github.com/july-rust/BoardSI/issues/520) | 증명된 Copper만 edge, 불명 Net/도금은 UNKNOWN |
| **E_023** | ODB++ 독립 Vendor Fixture와 Cross-format Oracle 1건 | [#578](https://github.com/july-rust/BoardSI/issues/578), [#10](https://github.com/july-rust/BoardSI/issues/10) | 공개·배포권·버전·독립 비교 확보 전 미착수/차단 |
| **E_024** | ODB++ #578/#10 전체 수락 Gate 재평가 | [#578](https://github.com/july-rust/BoardSI/issues/578), [#10](https://github.com/july-rust/BoardSI/issues/10) | 전체 조건 실제 PASS면 이슈 종료, 아니면 미인수 근거 분리 |
| **E_025** | IPC-2581 단일 버전 Layer/Net Source 부분집합 | [#532](https://github.com/july-rust/BoardSI/issues/532), [#23](https://github.com/july-rust/BoardSI/issues/23) | 공개 규격·원본 ID·유효/변형/누락 반례 |
| **E_026** | KiCad PCB Layer/Via Source 부분집합 | [#532](https://github.com/july-rust/BoardSI/issues/532), [#23](https://github.com/july-rust/BoardSI/issues/23) | 버전 명시·Layer Span 원본 증명·부정 반례 |
| **E_027** | Gerber+Drill 한 Layer/홀 분류 부분집합 | [#532](https://github.com/july-rust/BoardSI/issues/532), [#23](https://github.com/july-rust/BoardSI/issues/23) | Net/Material 미지정 UNKNOWN, Solver 입력 승인 금지 |
| **E_028** | 두 Source의 Canonical 기하 등가성 한 케이스 | [#532](https://github.com/july-rust/BoardSI/issues/532), [#520](https://github.com/july-rust/BoardSI/issues/520) | 독립 Oracle·레이어/패드/비아 스팬 전이 검증 |
| **E_029** | macOS Qt 타이머 간헐 지연 재현 실험 1개 | [#46](https://github.com/july-rust/BoardSI/issues/46) | 과거 FAIL 보존·같은 SHA·반증; 원인 미확정 시 별도 원장 |
| **E_030** | E 종료 감사·30건 이슈 전체 재분류 | [#2](https://github.com/july-rust/BoardSI/issues/2), [#3](https://github.com/july-rust/BoardSI/issues/3), [#4](https://github.com/july-rust/BoardSI/issues/4), [#5](https://github.com/july-rust/BoardSI/issues/5), [#6](https://github.com/july-rust/BoardSI/issues/6), [#7](https://github.com/july-rust/BoardSI/issues/7), [#8](https://github.com/july-rust/BoardSI/issues/8), [#9](https://github.com/july-rust/BoardSI/issues/9), [#10](https://github.com/july-rust/BoardSI/issues/10), [#11](https://github.com/july-rust/BoardSI/issues/11), [#12](https://github.com/july-rust/BoardSI/issues/12), [#13](https://github.com/july-rust/BoardSI/issues/13), [#14](https://github.com/july-rust/BoardSI/issues/14), [#15](https://github.com/july-rust/BoardSI/issues/15), [#16](https://github.com/july-rust/BoardSI/issues/16), [#17](https://github.com/july-rust/BoardSI/issues/17), [#18](https://github.com/july-rust/BoardSI/issues/18), [#19](https://github.com/july-rust/BoardSI/issues/19), [#20](https://github.com/july-rust/BoardSI/issues/20), [#21](https://github.com/july-rust/BoardSI/issues/21), [#22](https://github.com/july-rust/BoardSI/issues/22), [#23](https://github.com/july-rust/BoardSI/issues/23), [#24](https://github.com/july-rust/BoardSI/issues/24), [#25](https://github.com/july-rust/BoardSI/issues/25), [#41](https://github.com/july-rust/BoardSI/issues/41), [#46](https://github.com/july-rust/BoardSI/issues/46), [#165](https://github.com/july-rust/BoardSI/issues/165), [#520](https://github.com/july-rust/BoardSI/issues/520), [#532](https://github.com/july-rust/BoardSI/issues/532), [#578](https://github.com/july-rust/BoardSI/issues/578) | 종결/차단/차기제품 구분·열린 PR 0·링크 확인 |

## 현재 30개 열린 이슈 전수 분류

동일 이슈를 아래 서로 다른 분류에 중복 배치하지 않았다. **범위가 계획/감사라는 사실 자체로 기술 인수나 Issue Closed를 의미하지 않는다.**

| 구분 | 열린 이슈 | 처리 원칙 |
| :--- | :--- | :--- |
| M0 기반·인수 (6건) | [#2](https://github.com/july-rust/BoardSI/issues/2), [#3](https://github.com/july-rust/BoardSI/issues/3), [#4](https://github.com/july-rust/BoardSI/issues/4), [#5](https://github.com/july-rust/BoardSI/issues/5), [#6](https://github.com/july-rust/BoardSI/issues/6), [#7](https://github.com/july-rust/BoardSI/issues/7) | E_002–E_007: 완료 근거 재감사·종결 후보 |
| M1 Import·Viewer·Stackup (5건) | [#8](https://github.com/july-rust/BoardSI/issues/8), [#9](https://github.com/july-rust/BoardSI/issues/9), [#10](https://github.com/july-rust/BoardSI/issues/10), [#11](https://github.com/july-rust/BoardSI/issues/11), [#12](https://github.com/july-rust/BoardSI/issues/12) | E_008–E_011 및 E_016–E_024 |
| M1a/M1b 제품 기능 (3건) | [#13](https://github.com/july-rust/BoardSI/issues/13), [#14](https://github.com/july-rust/BoardSI/issues/14), [#15](https://github.com/july-rust/BoardSI/issues/15) | DEFERRED_PRODUCT |
| M2/M2a/M2b/M2c (7건) | [#16](https://github.com/july-rust/BoardSI/issues/16), [#17](https://github.com/july-rust/BoardSI/issues/17), [#18](https://github.com/july-rust/BoardSI/issues/18), [#19](https://github.com/july-rust/BoardSI/issues/19), [#20](https://github.com/july-rust/BoardSI/issues/20), [#21](https://github.com/july-rust/BoardSI/issues/21), [#22](https://github.com/july-rust/BoardSI/issues/22) | DEFERRED_PRODUCT; PH-08 실패 별도 |
| M3/M4/M5 (3건) | [#23](https://github.com/july-rust/BoardSI/issues/23), [#24](https://github.com/july-rust/BoardSI/issues/24), [#25](https://github.com/july-rust/BoardSI/issues/25) | #23 제한 형식 E_025–E_028, #24/#25 DEFERRED_PRODUCT |
| CI·Qt·GPU (3건) | [#41](https://github.com/july-rust/BoardSI/issues/41), [#46](https://github.com/july-rust/BoardSI/issues/46), [#165](https://github.com/july-rust/BoardSI/issues/165) | E_001, E_012, E_029; 원인·실측 증거 필수 |
| D_056 상위 범위 (3건) | [#520](https://github.com/july-rust/BoardSI/issues/520), [#532](https://github.com/july-rust/BoardSI/issues/532), [#578](https://github.com/july-rust/BoardSI/issues/578) | E_004/E_013–E_028; 전체 인수 전 OPEN |

**검산:** 6 + 5 + 3 + 7 + 3 + 3 + 3 = **30개 고유 Open Issue**, Open PR **0개** (2026-11-11 당시). [실시간 Open Issue](https://github.com/july-rust/BoardSI/issues?q=is%3Aissue+is%3Aopen) · [Open PR](https://github.com/july-rust/BoardSI/pulls?q=is%3Apr+is%3Aopen). 이후 이슈가 바뀌면 스냅샷을 재조회한다.

## 이전 A/B/C/D 전수 인계

- **A_001–A_036:** [A 번호별 역사 색인](development-checklist.md#a-history-ledger). A_035 합성 Preview 종합 인수와 A_036 단일 체크리스트 도입은 다른 성격이다. 원본 [A_035 감사](validation/a035-p2z-final-preview-acceptance.md)와 [A 검증 파일 인덱스](validation/INDEX.md), Source 내 번호별 흔적을 대조하되 근거 없는 개별 완료를 만들지 않는다.
- **B_001–B_009:** [Stage B](development-checklist.md#stage-b), [B_009 감사](validation/b009-b-series-final-acceptance.md). 합성 Reference/Stackup 증거는 실제 Copper/Return Path 제품 인수와 다르다.
- **C_001–C_025_10:** [Stage C](development-checklist.md#stage-c)와 C_025의 행정 정리 항목. 독립 체크가 없는 C_025 항목을 완료 건수로 중복하지 않는다.
- **D_001–D_056_09:** [Stage D](development-checklist.md#stage-d)의 SOURCE 국소 Geometry/Graph, Canonical Via, ODB++ subset의 소프트웨어 부분 인수. **D_056 전체 미인수는 E에서 처리**하며, 역사적 최초 날짜/CI는 보존한다.
- **SEC_001/M0/M2/U/T:** [보안 한정 근거](validation/sec001-codeql-markdown-comment-scan.md), [M0 완료 Gate](validation/m0-completion-gate.md), [PH-08 원본 실패](validation/m2-g2-ph08-stage4-original-gateq-reexecution.md), [제품 v1.0 사양](specification/BoardSI_Development_Spec_v1.0.md)과 [U/T 체크](development-checklist.md#stage-acceptance). 한정 PASS가 정식 제품 인수를 대신하지 않는다.

## 정지/이관 기준

- 외부 허가가 필요한 실제 PCB/벤더 데이터는 임의로 생성하지 않는다. 공개 권한·SHA·Oracle 미확인일 때 E_023 또는 실기기 요구 작업을 실행 가능한 것으로 표시하지 않는다.
- 연구·출시 단계 전체를 E에서 한 번에 해결하겠다고 선언하지 않는다. #13–#22, #24–#25는 후속 제품 Backlog로 정확하게 분리한다. #23은 CAD 형식별 **부분집합 작업**만 E에 포함한다.
- 마지막 E_030은 A–D 근거 및 30개 Issue 상태를 최신 다시 읽어 **종결·차단·미완/차후 작업**을 구분하는 작업이다. **프로그램 전체 완료**를 의미하지 않는다.
