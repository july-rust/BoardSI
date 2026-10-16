# BoardSI PH-08 Stage 5C — 일반 Chat 실행 기록

**판정:** 독립 합성 patch-test 5항목 PASS. **원본 Stage 5B Gate Q FAIL 유지**, Stage 5C 원본 Gate Q **Not Run**. AC-P1 PENDING / Contract NOT FROZEN / Rust FEM NO-GO.

## 1. 입력 및 근거 구분

- GitHub 읽기 전용 직접 조회: `july-rust/BoardSI` `main@f9d1e860bed2856049e0faac6e9861e7ee0ae160`, 문서 브랜치 `docs/m2-g2-ph08-contract` 역시 같은 SHA. Issue #16/#46 open, PR #35 open, 기타 검색으로 Issue #41 open 확인. GitHub 내용 변경 없음.
- GitHub에서 `AGENTS.md`, 문서 브랜치의 예상 경로 `docs/validation/m2-g2-ph08-acceptance-contract.md`는 404였으므로 **그 경로에 있는 PH-08 Contract를 확인하지 못했다**. 오브젝트가 다른 경로에 없는지 전체 트리 검색까지는 수행하지 않았다.
- Stage 4 보고서의 원시 텍스트는 Project Files에서 직접 읽었다. 이 보고서의 Stage 4 qualification FAIL 및 bulk RMS `4.04394e-15`, `4.24113e-15`, `9.23665e-15`를 입력 증거로 사용했다. 이 데이터의 ZIP 내부 raw source와 숫자 재집계는 **Not Run**.
- Stage 5B의 앞선 실행 보고에서 Gate M 조건부 충분조건 및 Gate Q FAIL, Pilot 0회를 확인했다. Stage 5B 원시 ZIP `boardsi-ph08-stage5b-evidence.zip`은 현 컨테이너에 없으며 바이트 무결성·원본 소스 검증은 **Not Run**.
- Ruleset 25046595/필수 CI 17개는 Stage 4 이전 보고서에 기재돼 있으나 이번 세션에서 Ruleset 원시 JSON이나 17개 각각을 재검증하지 않았다. 새 CI는 **Not Run**.

## 2. Stage 5C에서 실제 수행한 독립 시험

실행: `python boardsi_ph08_stage5c_independent_patchcheck.py`  
구성: 정규화된 3x3 샘플 9개, 6항 2차 다항식 기저, 독립 6x6 정규방정식 유리수 Gauss-Jordan, 별도 NumPy `float64` 최소제곱, 4개 해상도 `n=8/16/32/64`.

### 실제 수치 결과

| n | Quadratic: 정확 산술 Gradient 오차 | Quadratic: float64 Gradient 오차 | Cubic: 정확 산술 Gradient 오차 |
|---:|---:|---:|---:|
| 8 | 0 | 4.9650683e-16 | 1.0481569e-1 |
| 16 | 0 | 4.9650683e-16 | 2.6203922e-2 |
| 32 | 0 | 1.6910413e-15 | 6.5509804e-3 |
| 64 | 0 | 1.4217792e-15 | 1.6377451e-3 |

- Quadratic exact arithmetic reproduction: **PASS**; 모든 해상도에서 gradient 오차·전위 재구성 잔차 정확히 0.
- Cubic의 실제 gradient truncation error: **PASS**; exact arithmetic 연속 감소배율 `4.0 / 4.0 / 4.0`.
- 3종 합성 변조 거부 **PASS**: 한 샘플에 `h/1000` 전위 변조(각 해상도에서 exact gradient 오류 `2.3570226e-4`), gradient 전체 부호 반전, x/z 축 매핑 교환. 원본 HMM end-to-end mutation test는 Not Run.
- float64에서 quadratic gradient error는 `4.9651e-16, 4.9651e-16, 1.6910e-15, 1.4218e-15`로 비단조였다. **그 원인이 BoardSI 코드에서도 동일하다고 증명된 것은 아니다.**
- `cond(A)*eps*max|phi|/h`는 진단 스케일로만 기록했고 엄밀한 rounding enclosure로 간주하지 않았다.

## 3. 물리·수학적 의미와 경계

2차 다항식이 사용 중인 6항 local reconstruction space 안에 있으면 정확 산술에서는 reconstruction gradient truncation error가 0이어야 한다. 그래서 오류가 이미 `1e-15` 수준이면 감소율을 기준으로 수렴을 판정하기 어려운 경우가 실제로 생긴다. 별도 cubic MMS에서는 본 진단 스텐실에 대해 명확한 `O(h^2)` 감쇠를 관측했다. 이 실험은 기존 Gate Q의 보고된 실패에 대한 **개연성 진단**일 뿐 원본 HMM의 좌표 변환, conditioning, 경계 처리, interface projection 결함을 배제하지 못한다.

Neumann 하한과 zero-extension Dirichlet 상한의 정확 산술 수학적 순서는 이번에 새로 증명/실험하지 않았다. 이 결과로 Stage 5B Gate M의 수치 인증이나 open-domain error bound를 승격할 수 없다.

## 4. 보존 판정 및 후속 필요 작업

- **PASS (새 합성 테스트만):** 유리수 2차 재현, 비다항 cubic 기울기 차수, 샘플 변조 검출.
- **Not Run (원본):** 원본 Stage 5B ZIP hash/338 manifest 재검사, 실제 HMM Gradient 경로 분석, 원본 raw 세 해상도 재집계, 실수 연산 엄밀 bound, 독립 reviewer 승인, 신규 물리 solve/CI.
- **유지:** Stage 4 qualification FAIL; Stage 5B Gate Q FAIL; AC-P1 PENDING; Contract NOT FROZEN; Rust FEM NO-GO.
- 후속에는 먼저 Stage 5B 원시 ZIP와 소스/manifest를 확보하고 실제 quadratic reconstruction 경로의 유리수/고정밀·binary64 비교 및 부호/좌표 점검을 진행해야 한다. 기준 `1.8` 사후 완화는 허용하지 않는다.

## 5. 참고

1. 사용자 제공 `m2-g2-ph08-stage4-evidence.md`, 보고서 '실패 이력과 수정', 'AC-P1 오차 예산과 결정'.
2. GitHub: https://github.com/july-rust/BoardSI/commit/f9d1e860bed2856049e0faac6e9861e7ee0ae160
3. Talischi, Paulino, *Addressing Integration Error for Polygonal Finite Elements Through Polynomial Projections: A Patch Test Connection* (2013), https://arxiv.org/abs/1307.4423 . 본 코드의 성능을 입증하는 출처가 아닌 polynomial consistency 문제의 일반 배경 자료.

**원격 변경:** 없음. **새 Dirichlet solve:** 0. **물리 Pilot:** 0. **새 GitHub PR/commit/merge:** 없음.
