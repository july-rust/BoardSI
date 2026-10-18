# BoardSI — PH-08 Stage 5D: 독립 Gate Q 수치 안정성 진단 보고서

**현재 판정:** Stage 5B 원본 Gate Q **FAIL 유지** / Stage 5D 원본 Gate Q **Not Run** / AC-P1 **PENDING** / Contract **NOT FROZEN** / Rust FEM **NO-GO**.  
**실행 범위:** 독립 합성 polynomial 재구성 시험만. 물리 PDE·Neumann·Dirichlet solve 0회, Rust/Native CI 0회, 원격 GitHub 변경 0회.

## 1. 데이터 출처와 실제 접근 범위

- 이 작업에서 GitHub Connector로 직접 재조회한 `main`은 `f9d1e860bed2856049e0faac6e9861e7ee0ae160`, `docs/m2-g2-ph08-contract` 브랜치 HEAD도 같은 SHA다. 파일 브랜치의 PH-08 Contract 동결을 뜻하지 않는다.
- 관련 Issue #16, #41, #46은 열린 상태이며, Draft PR #35가 열린 상태라는 이전 확인 내용 중 #35의 open 상태도 이 작업에서 재조회했다. GitHub 커넥터 `fetch_commit` 응답으로 현재 commit의 **Verified 서명 여부는 검증할 수 없었다**. Ruleset 25046595 원시 JSON과 17개 CI의 현재 성공 여부는 **Not Run**이다.
- Stage 5C ZIP `boardsi-ph08-stage5c-chat-evidence.zip`는 직접 열어 ZIP CRC 검사 PASS. ZIP SHA-256: `67069f3614f4c82297751bf47f501e26a62bc963deb780c0ab93969fdc37aaa5`. 이는 이번 신작 합성 시험의 선행 산출물이지 BoardSI 원본 HMM 소스가 아니다.
- 필수 Stage 5B 원본 `boardsi-ph08-stage5b-evidence.zip`과 Stage 4 `boardsi-ph08-stage4-qualification.zip` 실제 바이트는 현 실행 컨테이너에 없었고 Project/Library 검색에서도 확보되지 않았다. 이 Stage 5D 작업 당시 Stage 5B 외부 ZIP 자체의 36개 checksum 및 13개 source snapshot, 중첩된 Stage 4의 338개 checksum 및 20개 source snapshot 재검증과 원본 HMM gradient 세 해상도 재집계는 모두 **Not Run**이었다. Stage 4 ZIP의 알려진 SHA-256 `6ade7f1e42db9b52f6243929183f1004a04f3f07950f37c903f592e60c05cda9`는 **이전 보고서에 기록된 값**이며 이번에 재확인한 hash가 아니다.
- 개발 사양서 `docs/specification/BoardSI_Development_Spec_v1.0.md`는 GitHub에서 직접 읽었다. §14의 재현 가능한 수치 검증 및 독립 오차 근거 원칙을 유지한다.

## 2. 새 합성 시험 (원본 BoardSI HMM 미사용)

실행 명령: `python boardsi_ph08_stage5d_gateq_diagnostic.py`.

유리수 `fractions.Fraction`으로 9개 샘플 지점과 6항 2차 다항식 fitting operator `(A^T A)^{-1}A^T`를 구성하고, 이를 독립 NumPy `float64` 최소제곱과 비교했다. 격자 종류는 Uniform, Skew, Graded 3개이며 각 `n=8,16,32,64,128`; 단위는 m, 전위 V, gradient V/m. 각 기하의 샘플 위치는 스크립트에 정확히 고정했다. 원본 HMM/mesh/flux/경계 구현을 호출하지 않았다.

| 검증 | 결과 | 의미 |
|---|---|---|
| quadratic 정확 산술 재현 | **15/15 PASS** | 샘플 9개로 2차 다항식을 정확 재구성, 정확 gradient 오차 0 |
| cubic 정확 산술 truncation | **12/12 PASS** | 3 격자의 4개 이웃 해상도 배율 모두 4.0, 즉 `O(h^2)` |
| float64 입력 양자화 범위 | **30/30 PASS** | **정확 함수값을 binary64로 단 한 번 반올림하는 단계만** enclosure, solver 전체 X |
| 4종 변조 테스트 | **12/12 PASS** | 전위 단일 샘플 변경/gradient 부호/좌표 축 바꿈/단위 배율 각각 검출 |
| 퇴화 stencil | **PASS** | 고정 z 직선 샘플에서 정확 rank deficiency 거부 |
| 실제 원본 Gate Q | **Not Run** | Stage 5B 소스와 raw JSON 미확보 |

### 조건수와 binary64 오차: quadratic 재구성

| 형상 | n | 정규화 기저 cond2 | 물리 좌표 그대로 cond2 | 정규화 binary64 gradient 오차 (V/m) |
|---|---:|---:|---:|---:|
| Uniform | 8 | 4.24 | 1.36×10⁸ | 3.22×10⁻¹³ |
| Uniform | 32 | 4.24 | 2.17×10⁹ | 1.53×10⁻¹² |
| Uniform | 128 | 4.24 | 3.48×10¹⁰ | 4.87×10⁻¹² |
| Skew | 8 | 5.49 | 1.92×10⁸ | 1.14×10⁻¹² |
| Skew | 32 | 5.49 | 3.08×10⁹ | 1.73×10⁻¹² |
| Skew | 128 | 5.49 | 4.92×10¹⁰ | 3.97×10⁻¹² |
| Graded | 8 | 4.90 | 1.03×10⁸ | 1.61×10⁻¹² |
| Graded | 32 | 4.90 | 1.65×10⁹ | 3.75×10⁻¹² |
| Graded | 128 | 4.90 | 2.63×10¹⁰ | 1.04×10⁻¹¹ |

Condition number는 **이 합성 Vandermonde-type 행렬의 진단치**이며 원본 HMM reconstruction 행렬의 상태를 증명하지 않는다. 정확 arithmetic에서 quadratic gradient 오차가 0이어도 binary64 error는 비단조였으므로 감소 배율만으로 수렴을 평가하는 데 한계가 있는 예를 관측했다. Stage 4 보고서의 `~1e-15` 오차는 **원래 정의·단위·정규화가 재확인되지 않았으므로**, 이 표의 V/m 오차와 직접 비교하지 않는다.

## 3. 반올림 enclosure의 엄격한 범위

정확 유리수 샘플 `v_i`를 IEEE-754 round-to-nearest binary64로 한 번 변환하는 경우, 합성 stencil의 gradient component에 관해 다음 조건부 입력 양자화 상한을 직접 계산했다.

`|δ(∂φ/∂x_k)| <= Σ_i |w_{k,i}| × 0.5 × ulp(v_i) / h`.

이때 `w`와 `h`는 정확 유리수이며, `ulp`는 해당 값 주변 binade를 고려한 상한으로 사용했다. 각 exact sample과 변환 후 binary64의 차이를 유리수로 되돌려 실제 component-wise enclosure를 검사했다. **30/30 PASS는 오직 sample conversion 단계만 포괄한다.**

예: Graded `n=128`의 input-only component 상한은 약 `[6.74×10⁻¹², 6.64×10⁻¹²] V/m`이며, NumPy 선형해 및 연산을 포함한 전체 관측 gradient 오차는 `1.04×10⁻¹¹ V/m`였다. **이 입력 상한을 전체 solver/HMM certified bound라고 주장할 수 없다.** 실제 BoardSI의 좌표/샘플 생성·전위 solve·제약 회복·gradient 평가·에너지/합산 오차를 포괄한 bound는 여전히 없다. 이 실험에서는 관측 solver-only 차이도 원시 JSON에 별도로 보존했다.

## 4. 수학적 해석 및 최종 판정

- **확인된 사실:** 독립 synthetic polynomial reconstruction은 정확 산술 quadratic 0 error 및 cubic 감소율 4를 재현하며, float64에서 비단조 오차와 물리 좌표 기반 ill-conditioning을 관측했다.
- **추론:** 기존 Gate Q의 quadratic 감소율 실패는 roundoff-saturated regime 또는 coordinate conditioning에 의해 발생할 수 있다. 그러나 Stage 5B 원본 소스/원시 결과 확인 없이는 실제 원인을 알 수 없다.
- **불확실:** 기존 HMM의 계면/경계·stencil·좌표 변환이 같은 원리로 작동하는지, 기존 `1.8` 게이트의 반올림 exception을 엄밀하게 정당화할 수 있는지, 무한영역 Neumann/Dirichlet energy bracket이 완전히 certified 가능한지는 **확실하지 않음**.
- **보존:** Stage 4 Qualification **FAIL**, Stage 5B Gate Q **FAIL**, AC-P1 **PENDING**, PH-08 Contract **NOT FROZEN**, Rust FEM **NO-GO**. Gate M의 조건부 수학 유도를 수치 인증으로 승격하지 않는다.

## 5. 다음 실행에 꼭 필요한 입력

`boardsi-ph08-stage5b-evidence.zip` (혹은 Stage 5B 실험을 직접 재현할 수 있는 동일 immutable snapshot·원시 JSON·SHA256SUMS) 및 Stage 4 원본 ZIP. **이 파일이 확보되기 전에는 원본 Gate Q 분석을 완료했다고 주장하거나 PR/Contract 동결을 진행할 수 없다.**

**기준 및 출처:**

1. [BoardSI GitHub](https://github.com/july-rust/BoardSI), 직접 조회 main SHA `f9d1e860bed2856049e0faac6e9861e7ee0ae160` 및 저장소 개발 사양서.
2. 사용자가 이전에 제공한 `m2-g2-ph08-stage4-evidence.md` 원문 및 Stage 5C 실행 기록(위 ZIP과 SHA-256).
3. 본 단계 `boardsi_ph08_stage5d_gateq_diagnostic.py`, 원시 JSON 및 run summary(첨부 ZIP의 manifest로 byte-by-byte 검증 가능).

## 2026-10-18 감사 주석 (원래 Stage 5D 결과를 변경하지 않음)

이 문서의 **Not Run**은 **Stage 5D 당시 해당 실행 환경**에서의 접근/재실행 상태다. 그 이후 2026-10-17에 승인된 별도 비공개 환경에서 원본 **Stage 5B HMM**을 재실행했고, 원래 Gate Q **FAIL**이 12/12 MMS 행 및 1761/1761 결정적 수치 필드 일치로 재현되었다. 공개 GitHub CI에서는 원본을 실행하지 않았다. Stage 5B 자체의 내부 감사 범위는 **36/36 checksum, 13 source snapshot**이며, Stage 5B ZIP 내부에 포함된 별도 **Stage 4 ZIP**의 범위가 **338/338 checksum, 20/20 source snapshot**이다. 과거 Stage 5D의 독립 합성 시험 결과와 원본 물리 Gate Q FAIL은 변경되지 않았다. [후속 원본 재실행 근거](m2-g2-ph08-stage5b-original-replay-evidence.md).
