# BoardSI PH-08 Stage 5E — 실행 코드 기반 증거 패키지 감사

**판정:** ZIP 무결성 검증 도구 완성 및 합성 음성 테스트 PASS; 실제 PH-08 HMM Gate Q는 FAIL 유지. AC-P1 PENDING / Contract NOT FROZEN / Rust FEM NO-GO.

## 무엇을 실행했나

- 일반 Chat에서 순수 Python 표준 라이브러리 기반 `tools/validation/ph08_evidence_audit.py`를 새로 작성했다.
- 외부 SHA-256 일치, ZIP 엔트리 중복·상대경로 탈출·암호화·symlink·확장 크기·압축률 검사, 내부 `SHA256SUMS`의 항목별 검산, 소스 Manifest 개수·경로·SHA 검증을 구현했다.
- 독립된 Stage 5C/5D Chat-only Manifest 포맷도 명확히 구분하여 검사한다. **Chat-only ZIP PASS는 원본 HMM 검증 PASS가 아니다.**
- `unittest` 합성 정상·음성 테스트 **22개 PASS**. 실제 Stage 5C ZIP의 5/5, Stage 5D ZIP의 6/6 Manifest 항목 SHA-256이 일치하였다.
- Stage 5D의 기존 `boardsi_ph08_stage5d_gateq_diagnostic.py`를 재실행하여 기존 15/15 quadratic, 12/12 cubic ratio, 30/30 input enclosure, 12/12 mutation 결과를 재확인했다. 이 시험들은 *독립 아홉 점 polynomial 복원 테스트*로 원본 HMM/물리 솔버에 대한 end-to-end 검증이 아니다.

## 인용 가능한 입력 지문

| 아티팩트 | SHA-256 | 목적 |
|---|---|---|
| `boardsi-ph08-stage5c-chat-evidence.zip` | `67069f3614f4c82297751bf47f501e26a62bc963deb780c0ab93969fdc37aaa5` | 5/5 manifest 검증 |
| `boardsi-ph08-stage5d-diagnostic-package.zip` | `7cb7a2ee41490b86706a87a99a86377dc635541ed68d94c40ed9b4c88a0b5947` | 6/6 manifest 검증 |

위 해시 중 Stage 5D 해시는 현재 작업 컨테이너의 실제 ZIP으로부터 계산했다. 같은 파일에서 계산한 기대 해시와 재대조하는 시험은 자체 파일 무결성 확인이며, 독립적 서명·외부 공인 해시 확인을 뜻하지 않는다.

## GitHub 직접 확인

- 조회 당시 `july-rust/BoardSI` `main` HEAD: `f9d1e860bed2856049e0faac6e9861e7ee0ae160`; 마지막 커밋은 2026-10-15의 PR #49 Qt 진단 보완이다.
- `docs/m2-g2-ph08-contract` HEAD도 같은 SHA. 오늘 PH-08 Rust FEM의 원격 반영, 서명된 변경 HEAD, 새 CI 검증은 확인되지 않았다.
- Issue #16(물리 코어), #41(감사), #46(Qt) open. PR #35는 별도 open이며 병합하지 않는다.
- GitHub API로 Ruleset ID 25046595 세부 구성/전체 17개 필수 CI의 현재값은 이번 실행에서 재검증하지 않았다. 과거 PASS를 새로운 HEAD PASS로 대체하지 않는다.

## 원본 검증 차단 조건

`boardsi-ph08-stage5b-evidence.zip` 및 원본 `boardsi-ph08-stage4-qualification.zip` 바이트를 확보하지 못했다. 이전 Stage 4 기록에 따르면 후자의 SHA-256은 `6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9`; Stage 5B 예상 해시는 알 수 없다. 따라서 Stage 5B 원본 338개 `SHA256SUMS` 항목과 Manifest source 20개, HMM Gate Q의 exact source/line, 원시 gradient 재집계, open-domain certified error budget은 **Not Run**이다.

## 검증기 해석의 주의

- 외부에서 알려진 ZIP SHA-256 없이 내부 합치만 PASS로 판정하지 않고 `NOT_VERIFIED`로 구분한다.
- 도구는 SHA256SUMS 파일의 **디렉터리 기준 상대경로**를 해석한다. 원본 ZIP의 형식이 다르면 스키마를 조사해야 하며, 임의 추정이나 대조 포기 없이 `NOT_VERIFIED`/`FAIL`을 내야 한다.
- 소스 Manifest의 인식된 키는 `sources`, `source_files`, `source`의 **list** 형태다. 다른 스키마는 누락된 검증으로 보고한다. 338/20은 사전 기대값일 뿐, 실제 파일 내용 확인 전 확정값이 아니다.
- 검증기가 `PASS`를 반환하더라도 이는 **아카이브의 자체 무결성·출처 보조자료가 기대 해시와 일치한다는 뜻만** 한다. 물리적 정확성, 실험 환경 재현, 메쉬 오차, 메모리 부하, 논문 독립 검증 또는 GitHub CI 인증이 아니다.

## 실행 환경과 변경 범위

- 실행 코드: Python 표준 라이브러리만 사용한 ZIP 감사기와 unittest.
- 기존 Stage 5D 재확인은 NumPy 의존의 합성 복원 실험임.
- 실제 물리 PDE solve: **0회**. Rust 빌드: **Not Run** (현재 환경에 `rustc`/`cargo` 없음).
- 이 결과의 GitHub 제품 소스 변경: **없음**. 적용용 patch만 생성 및 `git apply --check` 독립 시험. Verified SSH 서명 가능 환경이 없으므로 GitHub push/PR/merge 수행하지 않음.
- 최초 Stage 5B Gate Q FAIL, AC-P1 PENDING, Contract NOT FROZEN, Rust FEM NO-GO는 변경하지 않았다.

## 다음 실제 Gate

1. Stage 5B 원본 ZIP 바이트 확보 및 외부 SHA 확인.
2. 이 도구로 실제 SHA256SUMS 338개/Source 20개와 모든 원시 자료 검사. 자체 Schema 미지원이면 별도 형식 분석 및 테스트 먼저 추가.
3. Stage 5B 원본 gradient 샘플과 exact MMS로 Gate Q 실패 원인을 source-level 재현 (오차 분해·정규화 및 엄밀한 단위 검증).
4. 사전 승인 없이 PASS로 소급 변경하거나 Rust FEM 개발을 시작하지 않는다.