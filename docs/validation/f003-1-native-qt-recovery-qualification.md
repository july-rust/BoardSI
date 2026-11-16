# F_003_1 — Native Qt/Worker Integration & Crash Recovery Qualification

> 기준: 보호 `main@83d6acc114071557e6480840df19bea192b48071` · [구현 PR #654](https://github.com/july-rust/BoardSI/pull/654) · 인수 상태 **IN_PROGRESS**. 실행하지 않은 시험을 PASS로 표시하지 않는다.

## 구현 범위

- 기존 M0 합성 SI Worker/IPC 유지. 별도 `boardsi-native-file` Rust subprocess가 `boardsi_native::save/load`를 호출하고 bounded `BSI_NATIVE_FILE_V1` JSON receipt를 반환한다.
- 메인 `boardsi-preview-p0`의 `Open .bsi`, `Save Project (.bsi)`, `Native Save As`, `Cancel File` 기능은 실제 Qt `QProcess`와 타임아웃을 사용한다. 화면의 Native 탭은 구조 검증된 Layer/Net/Geometry 개수만 표시하며 아직 Native Artwork 렌더링을 주장하지 않는다.
- Native Save는 Qt Canonical Import와 동일한 PADS Source Coverage Gate, SHA-256 일치 확인 후 실제 Rust `BoardModel`을 저장한다. 지원되지 않은 CAD/객체 및 부분 Source 완전 복원을 주장하지 않는다.
- Native Save As는 최초 Open의 파일 SHA-256을 Qt에 보관하고, 재저장 시 원본 SQLite Load 전후 SHA-256을 재검증하여 다른 정상 .bsi 파일로 바뀐 경우도 Fail-Closed한다.
- 저장 Staging에는 OS 전용 파일 잠금 `.owner` 기록을 유지한다. 정상 프로세스 진행 중 파일은 복구하지 않고, 강제 종료 시 잠금 해제된 소유권 확인 Staging DB와 정확한 SQLite `-journal/-wal/-shm` 파일만 복구·정리한다.

## F0031-T01–T12 / PASS 증거 대기

| ID | 인수 항목 | 상태 / 실제 증거 |
| :--- | :--- | :--- |
| T01 | Main Qt Open .bsi | NOT_VERIFIED · Qt Native 실행 시험 |
| T02 | Main Qt PADS → .bsi Save / Native Save As | NOT_VERIFIED · Canonical Source SHA 결합 |
| T03 | Native 의미·정밀도 보존 | NOT_VERIFIED · 기존 F_003 Round-Trip 회귀 |
| T04 | GUI Event Loop 비차단 | NOT_VERIFIED · 실제 QProcess/Qt timer |
| T05 | Cancel / timeout / crash 처리 | NOT_VERIFIED · Qt Worker 종료 |
| T06 | 이전 요청의 지연 응답 | NOT_VERIFIED · 프로세스 결과 수명/ID 분리 |
| T07 | Writer 실제 강제 종료 | NOT_VERIFIED · F003 Native Resilience 4 OS |
| T08 | 고아 Staging 자동 정리 | NOT_VERIFIED · 제품 `recover_owned_staging` 호출 |
| T09 | 활성 Worker 및 SQLite Journal 보호 | NOT_VERIFIED · 실제 트랜잭션 중 OS Kill |
| T10 | 손상/미지원/변경 Source 보호 | NOT_VERIFIED · Native CLI 부정시험 |
| T11 | Windows x64/ARM64, macOS ARM64, Linux x64 | NOT_VERIFIED · OS별 CI |
| T12 | 기존 Import/Qt/IPC/Native 회귀 | NOT_VERIFIED · 관련 CI |

## 종료 원칙

- PR 최종 HEAD 필수 체크 통과, 독립 Codex 리뷰 미해결 0, Squash-only Verified protected merge 및 병합된 정확한 `main` SHA CI 확인 전에는 `COMPLETE`를 선언하지 않는다.
- 구조적 Native Open/Save 테스트와 실제 PCB Open/Short·물성·SI 수치 정확성 인수는 별개다.
- **범위 외/잔여:** 실제 물리 정전·저장장치 Cache flush 영속성, 512 MiB 최대 실보드 처리, 미구현 Native Artwork 전체 표시, F_004 Golden/migration, F_005 Vendor CAD 동등성, EB-01~07, PH-08 및 공식 U/T는 독립적으로 검증되어야 한다.
