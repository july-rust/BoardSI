# F_005 — CAD Source → Canonical → Native .bsi → Canonical

> **상태: 구현·기능 인수 PASS / 정확한 main Push CI 최종 정산 대기.**
> 최초 인수 근거일: **2026-11-16 KST**. F_005 한정 범위 밖의 CAD 전체 지원과 전기적/SI 검증은 승격하지 않음.

## 인수 범위

- **PADS V9.4 지원 부분집합**: 실제 PADS 파일 → Canonical → 기존 `save-pads` Writer → 단일 파일 `.bsi` → 독립 Reader → Canonical 의미 비교.
- **ODB++ SOURCE 한정 Directory/Single STEP 부분집합**: 2~64 Copper Layer, 직선 Trace와 Circle/Rectangle Pad. 공통 `odb_qt_wire::render`는 Qt와 Native에 동일한 fail-closed 허용 조건, 정확한 Rational 치수 제한 및 Qt double 투영 유효성을 적용.
- **Source 변경 보호**: 논리 경로+파일 SHA-256 Manifest, 정확한 모델 비교. `save_guarded`가 SQLite Staging·sync·독립 재검증 **후, 기존 목적 파일 교체 바로 전** ODB++ 원본을 재읽어 불일치 시 `BSI_SOURCE_CHANGED`를 반환하며 기존 `.bsi` 보존.
- **실제 Qt 메인 GUI**: 승인된 ODB++ Root/STEP/Manifest 기반 비차단 `Save Project (.bsi)` → `Open`. 실패·취소·변경된 원본을 최신 승인 데이터로 오인하지 않음.
- **제외**: 임의 상용 ODB++ 전체 CAD, KiCad/IPC-2581/Gerber의 완전한 Native Import, 제조 수준 Via/접촉·Net 전기적 연속성, RLGC/S-Parameter/SI, 전체 U/T 인증. 기존 E_028의 KiCad/PADS 교차 형상 Oracle은 Native KiCad 승격의 근거가 아님.

## F_005 인수 원장

| 항목 | 검증 내용 | 결과 |
|:--|:--|:--|
| T01 | PADS 실 파일 `save-pads` → Native Reader | **PASS** |
| T02 | ODB++ 실제 Directory/STEP → `save-odb` Worker | **PASS** |
| T03 | PADS Canonical → `.bsi` → Canonical 구조 동등성 | **PASS** |
| T04 | ODB++ Canonical → Native 저장·복원·Save As 의미 동등성 | **PASS** |
| T05 | 1.5 mm 독립 실측 기준과 정확한 Rational/BSIG 좌표 검증 | **PASS** |
| T06 | Geometry·Layer·Pad·Net·Frame 전체 필드 의미 비교 + 기존 BSIG 9종 회귀 | **PASS** |
| T07 | SourceIdentity·SHA-256·Capability·읽기 전용 정책의 Round-Trip | **PASS** |
| T08 | 독립 Source SHA Manifest / SQLite BSIG Oracle / F_004 Python Golden | **PASS** |
| T09 | 기존 E_028 PADS–KiCad SOURCE Via Oracle 5종 Debug/Release 회귀, Native KiCad 제외 | **PASS (기존 범위)** |
| T10 | 원본 변경·오류 STEP·미지원 형상·Qt 수치 초과·투영 붕괴·Staging 중 변경: 기존 파일 보존 | **PASS** |
| T11 | 실제 Main Qt ODB Import→Native Save→Open→stale 거부, 4 OS | **PASS** |
| T12 | 이전 F_001~F_004 규격·Golden·Migration·강제 종료·ENOSPC·Qt 회귀 | **PASS** |

## 재현 가능한 실행 근거

- **구현 PR [#660](https://github.com/july-rust/BoardSI/pull/660)** — 마지막 코드 HEAD `e678638b43a7e016178a7f32704dcd0e87508763`, **58/58 CI SUCCESS**, 독립 Codex 최종 HEAD 리뷰 **0건**, 기존 P1/P2 스레드 **3/3 해결**.
- **보호된 Verified Squash** — `fff7882840d9db0eefd6bd32c6039ef68e6d075d` ([commit](https://github.com/july-rust/BoardSI/commit/fff7882840d9db0eefd6bd32c6039ef68e6d075d)).
- **4 OS Native Writer/Reader/원본 변경 중 Staging 시험** — [F004/F005 통합 실행](https://github.com/july-rust/BoardSI/actions/runs/43832486414), Windows x64·ARM64, macOS ARM64, Linux x64 각각 `f005_cad_roundtrip` **5/5**.
- **실제 Main Qt GUI** — [Qt Native 회귀](https://github.com/july-rust/BoardSI/actions/runs/43832486644), Windows x64·ARM64, macOS ARM64, Linux x64에서 `--f005-native-odb-selftest` 통과. Linux CTest `qt-f005-main-odb-native-roundtrip` 1/1 및 전체 91/91.
- **E_028 교차 포맷 형상 오라클** — [Numerical Core CI](https://github.com/july-rust/BoardSI/actions/runs/43832486519), `e028_cross_format_via_oracle` 5/5 Debug, 5/5 Release.
- **Native 4 OS 실행 명령**: `cargo +1.75.0 test -p boardsi-native-file --locked --test f005_cad_roundtrip --features f004-qualification`.
- **Qt 자체 시험**: `boardsi-preview-p0 --f005-native-odb-selftest <odb-qt-import> <native-file-worker>`.
- **병합 후 main CI**: [정확한 병합 커밋의 Checks](https://github.com/july-rust/BoardSI/commit/fff7882840d9db0eefd6bd32c6039ef68e6d075d/checks). 최종 집계 대기 중이며 PASS로 선기록하지 않음.

## 판정 및 향후 사용 제한

**기능 인수 T01~T12: 12/12 PASS.** 이 12개는 F_005의 *지원 부분집합*에 대한 결함 차단·저장 왕복·Qt 수용성과 기존 회귀를 의미한다. 상용 PCB 전체 변환·실물 기판 전기적 연결성·SI 분석 품질을 보증하지 않는다. `main` Push 검증 완료 후 F_005 체크리스트를 범위 한정 `100% COMPLETE`로 최종 정산한다.
