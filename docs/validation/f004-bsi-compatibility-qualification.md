# F_004 — Native .bsi Golden / 호환성 / 안전 Migration 검증 원장

> 상태: **구현됨 · 최종 Native CI/독립 리뷰/보호 병합 확인 전** (2026-11-16 KST). 실증 전 PASS나 완료를 선언하지 않는다.

## Golden 및 교차 검증

| 정본 | SHA-256 | 생성/교차 검증 |
|---|---|---|
| [빈 보드 v1](../../schemas/bsi/golden-v1-empty.bsi) | `584c0f0c454f2f02ed06670bda98d22591c7a5f0bd53f86a45ac01af2c1bea4c` | 독립 SQL [시드](../../schemas/bsi/golden-v1-empty.sql) → SQLite; Rust Reader와 Python 검증 |
| [9종 Geometry v1](../../schemas/bsi/golden-v1-nine-geometry.bsi) | `8c9ec5ce6a4fcd8bb76b07bdaf4cffb90c1458db3e9a7523e0be82f972dd64cd` | 기존 상세 Canonical 모델과 Rust Writer 동등성; 독립 Python BSIG TLV·좌표·참조·Envelope SHA 검사 |

SHA는 각 [Manifest](../../schemas/bsi/golden-v1-nine-geometry.sha256)에서 CI로 대조한다. 고정 바이너리를 CI에서 **재생성하지 않는다**. [Python 독립 Oracle](../../tools/validation/test_f004_golden_oracle.py)은 Writer/Reader를 호출하지 않고 SQLite 헤더, DDL/버전, FK, Geometry 9종·10객체의 BSIG 필드, i128 Rational, 좌표 tick, 참조·홀·Padstack, 음성 변조를 점검한다. Python Oracle 통과는 물리 연결/전송선로 정확성 증거가 아니다.

## 버전·Migration 계약

- 공식 저장 스키마는 **v1** 하나뿐이다. 과거에 존재하지 않은 공식 v0 또는 미래 v2를 생성하거나 지원된다고 주장하지 않는다.
- [Rust Migration Framework](../../crates/boardsi-native/src/migration.rs): 인접 버전별 명시 Registry/경로 검사, 전체 트랜잭션·Rollback 단위시험, 미등록/역방향/모호 경로 Fail-Closed. 정식 등록 변환 Step 현재 **0개**.
- [Worker `migrate-v1`](../../apps/boardsi-native-file/src/main.rs): 검증된 v1의 동일 경로 호출은 무쓰기 멱등. 다른 신규 경로는 원본 Read-Only→소유 잠금 Staging→원본·Staging SHA 재확인→Canonical 비교→파일 sync→동일 디렉터리 No-Clobber Hard-Link로 발행. 기존 대상 경로는 덮어쓰지 않는다.
- Unknown Optional / Future Minor는 Read-Only이며 자동 보존 불가 시 Migration 거부. Unknown Required·Future DDL·미발행 v0·변조 스키마는 거부. Source SHA/원본 파일의 바이트를 변경하지 않는다.
- 테스트 전용 Feature `f004-qualification`에서만 I/O failpoint·실제 Worker Kill 주입. 기본 배포 실행파일에는 Fault Hook이 없다.
- 실제 v2가 **규격과 Golden으로 발행된 뒤** v1→v2 Step 및 Reader 호환 검증을 추가한다. 현재 가상 1→2→3 트랜잭션 테스트는 제품 Migration 실증이 아니다.

## F004-T01~T12 인수 표

| ID | 인수 항목 | 증거 / 조건 |
|---|---|---|
| T01 | 영구 Golden + SHA | 위 불변 2개 파일, Manifest 및 CI digest |
| T02 | 역사적 v1 Reader | [Rust 호환성 테스트](../../crates/boardsi-native/tests/f004_compatibility.rs)에서 저장된 파일 직접 Load |
| T03 | Canonical/바이너리 동등성 | 9종 모델 자체 검증 + 독립 Python TLV 및 좌표 확인 |
| T04 | Cross-OS | Windows x64/ARM64, macOS ARM64, Linux x64 **동일 파일** |
| T05 | 버전 매트릭스 | v1/미래 Minor/미지원 Major·DDL/minimum reader |
| T06 | Required Feature | Unknown Required Fail-Closed 및 원본 SHA 불변 |
| T07 | Optional Extension | Read-Only, Save As·Migration 손실 방지 |
| T08 | Migration Registry | 인접 Step 순서, 미등록/중복 경로 거부, 트랜잭션 Rollback |
| T09 | 멱등성 | v1 동일 경로 무쓰기, 새 경로 동일 바이트, 기존 대상 No-Clobber |
| T10 | 장애·복구 | [Worker 통합 시험](../../apps/boardsi-native-file/tests/f004_migration.rs) 실 Kill/I/O 실패/읽기전용 파일, CI Linux 실제 ENOSPC |
| T11 | Worker/Qt 회귀 | 실제 File Worker CLI, 기존 [F_003_1 Qt](f003-1-native-qt-recovery-qualification.md) 테스트 재실행 |
| T12 | 전체 기존 회귀 | F003/F002, Qt, 수치 엔진 등 PR HEAD 필수 CI, 병합 main CI |

CI: [F004 전용 4OS+ENOSPC](../../.github/workflows/f004-compatibility.yml) · [PR #658](https://github.com/july-rust/BoardSI/pull/658).
**테스트 통과의 증거는 최종 PR HEAD/병합 main의 실제 GitHub Actions 실행과 Codex 리뷰 결론에서만 확정한다.**

## 제외·경계

실제 이전 정식 버전(현재 없음)의 마이그레이션, 모든 CAD Import 동등성(F_005), 실기기 정전과 512MiB 실 PCB 성능, 전기 연결·RLGC/S-parameter 검증, EB-01~07, PH-08 및 제품 U/T 전체 승인까지 완료한 것이 아니다. *F_004 100%* 선언은 이 범위의 T01~T12·4 OS/ENOSPC·독립 코드 리뷰·보호 Squash·정확한 main Push PASS에 한정한다.

근거: [F_001 규격](../architecture/f001-bsi-format-contract.md) · [F_002 구조](../architecture/f002-bsi-geometry-binary-layout.md) · [F_003_1 검증](f003-1-native-qt-recovery-qualification.md).
