# F_003_1 — Native Qt Save/Open, Worker Lifecycle & Crash Recovery

> **100% COMPLETE — 한정된 F_003_1 범위.** 최초 완료/근거일 **2026-11-16 KST**. 구조적 .bsi 인수이며 실제 PCB 물리 SI·정전 검증이나 제품 U/T 전체 승인과 구분한다.

## 보호 병합 및 독립 검증

| 항목 | 실제 근거 |
| :--- | :--- |
| Native 통합 | [PR #654](https://github.com/july-rust/BoardSI/pull/654) → Verified Squash [`73ba76df`](https://github.com/july-rust/BoardSI/commit/73ba76df8537d8c9ce22ce9671dcf170222bf0c5), PR HEAD 14/14 CI 성공 |
| Qt 취소·타임아웃·지연 응답 | [PR #655](https://github.com/july-rust/BoardSI/pull/655) HEAD `f7435f264956b77174137615bfac70f07c3e0d16` → Verified Squash [`9e0b273a`](https://github.com/july-rust/BoardSI/commit/9e0b273a699674ebfd1e869bb8a75a6aefe9a3a4), PR HEAD **13/13 워크플로 PASS** |
| 실제 4 OS Qt 실행 | [Qt runtime #43795724257](https://github.com/july-rust/BoardSI/actions/runs/43795724257): Windows x64/ARM64·macOS ARM64·Linux x64 **4/4 PASS** |
| 독립 리뷰 | #654의 P1/P2/P2 3개 지적 수정·스레드 해결; #655 최종 HEAD Codex **미해결 0**, 신규 지적 없음 |
| 기존 회귀/안전성 | #655의 F003 Native Resilience, F003 Native BSI Rust, F002 Schema, Numerical Core, Qt runtime 등 **13/13 PASS** |

## F0031-T01~T12 (12/12 PASS)

| ID | 인수 조건 | 결과 및 증거 범위 |
| :--- | :--- | :--- |
| T01 | 메인 Qt `.bsi` Open | **PASS** — 실제 Worker·SQLite 검증·구조 개수, 4 OS |
| T02 | Canonical PADS Save / Native Save As | **PASS** — 원본 SHA 검증, 재열기 및 별도 대상 저장 |
| T03 | Native 의미·정밀도 Round-Trip | **PASS** — F003 Rust 원본/재로딩 모델 동등성, lossless Geometry |
| T04 | GUI 비차단 | **PASS** — 실제 QProcess 지연 중 Qt timer 진행, Linux 38 ticks / macOS 27 ticks |
| T05 | Cancel / Timeout / Worker Kill | **PASS** — 시험 전용 Rust 일시 정지 + 실제 OS 자식 프로세스 종료, 4 OS |
| T06 | 이전 요청 응답 차단·최종 정상 상태 보존 | **PASS** — 서로 다른 경로의 취소·타임아웃 요청 후 정상 파일 재열기; 마지막 정상 `.bsi` 요약 유지 |
| T07 | 저장 중 실제 프로세스 강제 종료 | **PASS** — 4 OS F003 Native Resilience |
| T08 | 잠금 해제된 고아 Staging 정리 | **PASS** — 실제 `recover_owned_staging`, 재실행 멱등 |
| T09 | 활성 Writer·SQLite Journal 보호 | **PASS** — 실제 트랜잭션 중 Kill, Lock·Journal/WAL/SHM 안전 정리 |
| T10 | 손상·부분지원·출처 변경 Fail-Closed | **PASS** — 잘못된 SQLite·SHA·WAL-only 변경 거부, last-good 보존 |
| T11 | 4 OS 실행 | **PASS** — 4 OS Qt lifecycle + Rust Native/복원력 CI |
| T12 | 기존 Import/Qt/IPC/수치 연산 회귀 | **PASS** — #655 HEAD 13/13 workflows |

**직접 확인한 실행 수치:** Linux `qt-f0031-real-native-lifecycle`은 `pass=true, cancel_ok=true, timeout_ok=true, stale_response_ok=true, event_loop_ticks=38`; macOS에서도 동일 세 불리언이 모두 true, 이벤트 루프 27 ticks. 이 값은 특정 CI 환경에서의 동작 증거이며 일반 사용 환경에 대한 지연 시간 보증이 아니다.

## 구현 경계 및 제외

- Rust `boardsi-native-file` 전용 QProcess 사용. 기존 합성 SI Worker·IPC·SQLite v1 Container/Geometry Layout 변경 없음.
- `f0031-qualification` Rust feature에서만 실제 자식 프로세스 pause fault hook 활성화. 기본 배포 빌드에는 포함하지 않음.
- `.bsi`는 현재 **구조적으로 읽기·저장** 가능하나 Native Artwork 전체 렌더링, 물리적 Return Path·전기적 연결성·RLGC/S-parameter 정확성은 **NOT_VERIFIED / NOT_RUN**.
- 실제 전원 차단 및 저장장치 Cache Flush 내구성, 최대 실보드 512 MiB 성능, 모든 CAD 포맷, 향후 F_004 호환성/Golden 및 F_005 Vendor CAD, EB-01~07, PH-08, 공식 제품 U/T 전체 승인도 **별개 범위**다.

**결론:** 본 검증 원장의 **F_003_1 12/12 PASS / 100% COMPLETE**는 위 기능과 시험 항목만의 종료 판정이다. 다른 개발 단계나 제품 전체의 완료 선언이 아니다.
