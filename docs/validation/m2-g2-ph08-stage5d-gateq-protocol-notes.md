# BoardSI PH-08 Gate Q — Stage 5D 추가 수치 검토 노트

상태: **비승인 PROPOSAL**. Stage 4/5B 이전 FAIL 유지. 이 노트로 시험 게이트를 자동 수정할 권한은 없다.

1. **Q-POLY:** original reconstruction stencil을 독립 정확 산술로 복제해 2차 다항식의 algebraic polynomial consistency를 확인해야 한다. Stage 5D uniform/skew/graded 합성 stencil의 15/15 PASS는 surrogate 사례일 뿐이다.
2. **Q-TRUNC:** 진짜 절단 오차가 존재하는 cubic 이상의 MMS를 원본 accepted geometry에 적용한다. 독립 합성 환경은 cubic 12개 step에 대해 감소율 4.0을 보였다. 기존 threshold 1.8은 소급 변경 금지.
3. **Q-FLOAT:** 원본 pipeline을 sample generation, coordinate rounding, fitting solve, derivative recovery, aggregation으로 분해해야 한다. Stage 5D는 합성 원본 값의 binary64 한 번 변환에만 exact-rational component bound를 구했다. 이를 solver-rounding 또는 HMM certified floor로 확대하지 않는다.
4. **Q-MUT:** 4종 단일 변조를 original end-to-end 수치 규칙으로 거부해야 한다. 단위 1000배 착오는 특히 SI 단위 검사로 독립 확인한다. 합성 code의 12/12은 원본 구현 test가 아니다.
5. **Stop rule:** Stage 5B 원본 archive가 없거나 해시가 맞지 않으면 원본 코드 원인 확정 및 신규 Pilot를 시작하지 않는다. Gate Q FAIL / AC-P1 PENDING / Contract NOT FROZEN / Rust FEM NO-GO 유지.

Stage 5D 보고서와 실행 스크립트/원시 JSON에 수치 근거가 있다. 본 문서는 승인된 Contract나 Frozen Reference가 아니다.
