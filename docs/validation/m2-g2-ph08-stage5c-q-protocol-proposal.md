# PH-08 Gate Q — 독립 검토용 사전 개정 제안 (미승인)

상태: **PROPOSAL / NOT APPROVED**. 기존 Stage 4 qualification FAIL, Stage 5B Gate Q FAIL을 절대 바꾸지 않는다. 본 문서는 PH-08 Contract가 아니며 Frozen SHA도 없다.

## 핵심 판정 분리

### Q-POLY — polynomial consistency
- quadratic 전위와 gradient를 똑같은 정확한 좌표·방향·단위에서 도출하고, 6항 기저의 rank와 interpolation/reconstruction 범위를 명시한다.
- 독립한 유리수 혹은 검증된 고정밀 oracle가 2차 polynomial에서 0의 truncation error(또는 증명된 입력 quantization bound)를 갖는지 검사한다.
- 이 검사는 truncation polynomial reproduction만 검증한다. 물리 C/C0 accuracy는 검증하지 않는다.

### Q-TRUNC — genuine asymptotic convergence
- quadratic에만 의존하지 않고 2차 재구성 공간에 속하지 않는 cubic 등 매끄러운 MMS를 미리 명세한다.
- 미리 선택한 해상도 구간에서 2배 refinement의 gradient-error 감소배율을 체크한다. **기존 1.8은 유지**하고, 모든 물리 지점과 경계/계면에서 적용 조건을 분리한다.
- 적절한 해상도 구간에 진입하지 못하면 인위적 PASS 대신 INCONCLUSIVE/FAIL을 보고한다.

### Q-FLOAT — floating error accounting
- `eps`, `cond(A)`, 잔차, 입력/좌표 반올림, 회귀 연산과 norm 누적을 포함한 forward-error enclosure를 사전 증명한다.
- `cond(A)*eps*max|phi|/h`는 민감도 진단일 뿐 **certified bound가 아니다**. 그 자체로 floor 예외를 승인하지 않는다.
- 충분히 증명한 floor 이하에서만 rate test가 의미 없다는 결론을 **독립 reviewer와 사전 변경 관리 후** 차기 qualification에 적용할 것을 제안한다. 이때도 floor 위의 변조는 반드시 검출해야 한다.

### Q-MUT — fail-closed
- 샘플 phi 변조, gradient 부호 반전, 축/기저 대응 오류 세 유형이 각각 FAIL을 일으켜야 한다. 오염이 큰데도 floor/ratio 예외로 덮이지 않도록 제한한다.
- 시험 코드는 원본 source와 별개이며, 실제 원본 pipeline의 end-to-end mutation은 따로 확인한다.

### Q-PHYSICAL / Gate M 연계
- Q-POLY/Q-TRUNC/Q-FLOAT/Q-MUT 통과 이후에만 별도 승인을 받고 Neumann/Dirichlet 물리 solve 및 open-boundary error budget을 실행한다.
- `E_N <= E_infinity <= A_D`는 정확 산술·적합성·올바른 물리 경계와 zero-extension 가능성을 요구한다. 수치 certified bracketing은 floating enclosure가 갖춰져야만 주장한다.

## 적용 판정 표

| 조건 | 기존 Stage 5B | 장래 승인된 프로토콜 |
|---|---|---|
| 원본 코드/원시 데이터 미확보 | Gate Q FAIL 유지 | 검증 Not Run |
| 정확 다항식 재현·비다항 수렴 확인, floating 인증 부족 | Gate Q FAIL 유지 | Q-FLOAT 미충족, Gate Q 승인 금지 |
| floating bound 및 3종 mutation 포함, 독립 reviewer 미승인 | Gate Q FAIL 유지 | PROPOSAL, NOT FROZEN |
| 독립 리뷰/사전 등록/전체 조건 충족 | 소급 변경 금지 | 신규 버전에서만 조건부 승인 검토 |

**서명·CI·PR:** 원본 저장소 변경하지 않음. 미서명 커밋, Ruleset 우회, Contract freeze, Rust FEM 구현 금지.
