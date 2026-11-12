# BoardSI E 단계 — 진행 차단·외부 의존성 별도 관리

> 기준: **2026-11-11 KST**. 실행 계획은 [E 단계 계획](e-stage-closure-plan.md), 정식 체크 상태는 [단일 개발 체크리스트](development-checklist.md)만 사용한다. 이 문서는 **기술 완료 체크리스트가 아니라 차단·이관 원장**이다.

## 상태 판정 기준

- **FAIL:** 실제 수락시험이 실패. 기존 실패를 소급 PASS/미실행으로 바꾸지 않는다.
- **BLOCKED:** 필수 조건이 현재 미충족되어 다음 승인이 불가.
- **DEPENDENCY_UNVERIFIED:** 공개 Source·실측 장비·권한 등 선행 조건의 존재/사용 가능 여부를 확인하지 못했다. 영구 불가능이라는 뜻은 아니다.
- **NOT_VERIFIED:** 완료를 주장할 독립 근거가 부족하다. 일부 기능이 존재할 수 있다.
- **DEFERRED_PRODUCT:** 기능이 크거나 후속 마일스톤 범위로서 E 우선순위 밖에 둔다. 차단 실패와 혼동하지 않는다.

## 독립적으로 관리할 의존성

| ID | 연관 이슈 | 증거로 확인한 사실 / 부족한 점 | 분류 | 재개 조건 |
| :--- | :--- | :--- | :--- | :--- |
| EB-01 | [#578](https://github.com/july-rust/BoardSI/issues/578), [#10](https://github.com/july-rust/BoardSI/issues/10) | ODB++ ZIP STORED Source 부분 구현은 병합됐지만 **독립 공개 Vendor Export 및 다른 CAD 형식의 동일 보드 Oracle 확보는 미확인** | DEPENDENCY_UNVERIFIED | 배포 가능한 Fixture의 허가·파일 버전·SHA·실물 비교 근거 확보 후 E_023 인수 |
| EB-02 | [#46](https://github.com/july-rust/BoardSI/issues/46) · E_029 ([#41 감사 종결 근거](validation/e001-ph07-post-audit-closure.md)) | **Qt 성능 원인은 미규명**. 과거 1,000 ms 기준의 실제 FAIL 1,134/1,331 ms와 임시 M0 1,500 ms 기준의 2,930 ms FAIL은 보존한다. 간헐적 PASS·재실행 성공이나 #41 감사 종결은 근본 원인 해결 증명이 아니다 | NOT_VERIFIED | E_029에서 원인별 대조 실험, 실제 Native 반복 재현, 변경 전/후 계측, 원인·교정·회귀를 확보한 다음 #46만 독립 평가 |
| EB-03 | [#165](https://github.com/july-rust/BoardSI/issues/165) · [E_012 Native 실 GPU 감사](validation/e012-actual-hardware-gpu-qualification.md) | **2026-11-12 KST 확인:** 정확 code-main `da880d29`의 [Native Qt run #43150028282](https://github.com/july-rust/BoardSI/actions/runs/43150028282)에서 macOS ARM64 Metal, Windows x64/ARM64 D3D11 하드웨어 adapter true 및 10k/100k/1M 프레임 증거를 확보. 따라서 'GitHub Native 실제 GPU 1개' 전제는 충족. **하지만** 고객 실기기·GPU 모델/Driver 식별·GPU VRAM·1M@60 FPS·실제 device reset 및 전체 #165 제품 인수는 미확인. 과거 synthetic/offscreen CI와 하드웨어 검증을 혼동하지 않는다 | DEPENDENCY_UNVERIFIED / NOT_VERIFIED (고객 실기기 및 전체 제품 수락) | E_012 문서 PR #599 보호 Verified Squash `04fd35a9` 및 정확 main Push 8/8 SUCCESS로 소형 범위 종료; #165 전체 재개는 고객 실기기/Driver/VRAM 계측, 실제 reset·60 FPS 기준 및 독립 U/T 승인 증거 |
| EB-04 | [#16](https://github.com/july-rust/BoardSI/issues/16), [PH-08 원본 Gate Q](validation/m2-g2-ph08-stage4-original-gateq-reexecution.md) | PH-08 원본 HMM Gate Q는 **FAIL**. 합성 PEC/FEM 수학 결과는 실제 PCB 30 GHz 정확도 승인 근거가 아님 | FAIL / BLOCKED | 고정된 재검증 계약, 실패 원인 해결, 독립 Oracle 및 원본 Gate 재실행이 모두 충족될 때만 재평가 |
| EB-05 | [#3](https://github.com/july-rust/BoardSI/issues/3), [#6](https://github.com/july-rust/BoardSI/issues/6), [#7](https://github.com/july-rust/BoardSI/issues/7), [M0 완료 게이트](validation/m0-completion-gate.md) | [E_006 Native 감사](validation/e006-native-platform-toolchain-qualification.md)에서 세 Native Qt/Rust 합성 프로토타입 실행은 실제 로그로 확인. [E_007 공개 Fixture 제한 감사](validation/e007-public-fixture-provenance-rights-qualification.md)는 KiCad 후보 2종의 출처·기존 해시/Native 증거와 라이선스 원문을 구분하나, **두 Board 원본의 제3자 기여/고지·특정 배포물 사용 승인 및 정식 소형/중형 크기 계급은 미인수**. Qt 등 제3자 소프트웨어의 **최종 공개 재배포 권한, Clean-Host 설치 및 OS/실제 약 8 GiB 기준 PC·SSD/GUI 인수도 미검증**. PR F7 임시 Merge Checkout은 PR HEAD와 동일한 Tree SHA임을 확인했지만 정식 설치·라이선스의 대체 증거가 아님 | DEPENDENCY_UNVERIFIED / NOT_VERIFIED | E_007 두 공개 PCB의 전체 저작권·제3자 고지·특정 재배포 방식에 대한 독립 권리 확인, 실제 기준 PC/SSD/GUI 계측 및 소형/중형 기준 승인; F7 최종 라이선스/플러그인 의무·실행 가능한 패키징·3 Native 플랫폼의 독립 Clean-Host 설치 증거 |
| EB-07 | [#2](https://github.com/july-rust/BoardSI/issues/2) · E_002 ([추적성 정합화](validation/e002-m0-f1-traceability-status-reconciliation.md)) | U01–U48/T01–T54 메타데이터 매핑은 존재하지만 M0 F1의 개별 시험별 승인·원본 Fixture·실행 SHA/Job·독립 인수 심사는 미충족. T13/T39/T46의 제한 M0 프로토타입 PARTIAL 증거는 정식 전체 시험 `NOT_RUN`과 구분한다. 원본 10-case JSON은 변경하지 않는다 | BLOCKED / NOT_VERIFIED | M0 전용 적합성 판단 후 각각의 기대값·실제 결과·재현 Fixture·SHA·Job/Artifact·독립 승인일을 확인해야 Issue #2 전체 DoD를 재평가할 수 있음 |
| EB-06 | [#520](https://github.com/july-rust/BoardSI/issues/520), [#532](https://github.com/july-rust/BoardSI/issues/532) | 임의 ODB++/IPC-2581/KiCad/Gerber 및 제조 Plating/Barrel/Stackup의 전체 원본 근거 미인수 | NOT_VERIFIED | 각 네이티브 형식별 공개 규격·양/음성 Fixture·Source Provenance·독립 교차 Oracle |

## 장기 제품 개발 — E에서 억지로 종료하지 않을 이슈

| 유형 | 추적할 기존 열린 이슈 | 취급 |
| :--- | :--- | :--- |
| PCB/Substrate 구조와 Port/검증 | [#13](https://github.com/july-rust/BoardSI/issues/13), [#14](https://github.com/july-rust/BoardSI/issues/14), [#15](https://github.com/july-rust/BoardSI/issues/15) | DEFERRED_PRODUCT; E 범위 밖 |
| RLGC·S-파라미터·Sweep·Cache·Stress·Via 물리 검증 | [#16](https://github.com/july-rust/BoardSI/issues/16), [#17](https://github.com/july-rust/BoardSI/issues/17), [#18](https://github.com/july-rust/BoardSI/issues/18), [#19](https://github.com/july-rust/BoardSI/issues/19), [#20](https://github.com/july-rust/BoardSI/issues/20), [#21](https://github.com/july-rust/BoardSI/issues/21), [#22](https://github.com/july-rust/BoardSI/issues/22) | DEFERRED_PRODUCT; #16의 PH-08 FAIL은 EB-04에서 계속 추적 |
| 미래 CAD·성능 확대 | [#23](https://github.com/july-rust/BoardSI/issues/23), [#24](https://github.com/july-rust/BoardSI/issues/24), [#25](https://github.com/july-rust/BoardSI/issues/25) | #23의 단일 Source 부분 작업만 E_025–E_028; #24·#25는 DEFERRED_PRODUCT |

## 업데이트 원칙

차단 원장에 등재하는 것은 **완료 처리도, 자동 이슈 Close도 아니다**. 원인·증거·재개 조건을 바꿀 때 날짜/원본 SHA를 덧붙인다. 재개 가능해지면 가장 작은 다음 E 단위로 시작하고, 해당 작업의 코드·테스트·체크리스트·PR을 한 번에 종결한다.

기존 이슈를 **전혀 해결하지 못했는데 닫는 행위는 금지**한다. 열려 있어야 하는 상위/장기 이슈와, 시작한 E 번호의 실제 미완 PR을 구분해 관리한다.
