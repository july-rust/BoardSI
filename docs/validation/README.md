# BoardSI 검증 원칙 및 결과 읽기 | Validation

> **검증 보고서 인벤토리:** **[전체 과거 검증 기록 158개 — 단계별 색인](INDEX.md)** · [공식 개발 체크리스트](../development-checklist.md) · [문서 홈](../README.md).
>
> 이 문서 하단의 M2/PH-08 수치는 **당시 특정 쿠폰·시험 계약의 증거**입니다. 최신 전체 기능 상태와 U01–U48/T01–T54의 인수 결과는 오직 [개발 체크리스트](../development-checklist.md)에서 확인합니다.

## 검증 대상과 주장 가능한 범위

| 시험 층위 | 검증 대상 | 통과하더라도 검증되지 않는 것 |
| :--- | :--- | :--- |
| **형식 파서·Source 품질** | PADS 형식 서명, 지오메트리, Net/Layer, 미지원 항목 추적 | 전체 PADS 리비전/ODB++, 보드 도통, 실제 임피던스 |
| **2D 구리 기하** | 동일 층 Trace–Trace·Trace–Via·Trace–Pad의 SOURCE 국소 후보 | 모든 Padstack/Pour/Thermal/Plane, 전체 전기 Open/Short |
| **Canonical/Qt·IPC** | 동기화, Snapshot, 렌더링, 취소·오류, 네이티브 플랫폼 회귀시험 | 제조 실물 구리 연결성, SI 물리 계산/배포 승인 |
| **수치 코어·연구 쿠폰** | 고정 RLGC/2-port 계산, 제한 독립 PEC/lossless FEM 수학 참조 | 기하→손실 RLGC 추출, 실측 PCB, DC–30 GHz 완제품 |
| **제품 인수 U/T** | 사양서 규정 전체 사용자 작업 흐름과 수치·성능·플랫폼 조건 | 이 Gate만이 별도 정식 PASS의 근거가 됨 |

### 최근 국소 Copper 연구의 대표 증거

- [D_054·D_055 공통 SOURCE 접촉 검증](d054-d055-general-trace-contact-qualification.md): 임의 각도 직선/원호, 비중심 Via 환형 패드, SMD Pad 모서리·변·내부, SOURCE 해시와 경계조건을 확인한 **한정 Rust 결과**입니다.
- [D_039–D_053 역사적 국소 연결 시험](INDEX.md): 이전 단계의 개별 기하/네트워크 Fixture는 그대로 보존합니다. 해당 파일들이 모든 각도에 별도 코드가 필요하다는 뜻은 아닙니다.
- [D_055 코드 계약](../architecture/trace-copper-contact.md): 전기적 실제 연결성으로 오해하지 않도록 수학적 전제·드릴 영역·Net 소유권을 설명합니다.

### PASS와 불확실성의 표기

검증 결론은 **입력 데이터/모델/조건·코드 SHA·OS/컴파일러·실제 결과·미확인 범위**가 있어야 합니다. \`NOT_RUN\`, \`NOT_VERIFIED\`, \`BLOCKED\`, \`FAIL\`은 다른 의미입니다. 과거 PH-08 Original HMM **Gate Q FAIL**은 synthetic CI PASS로 대체되지 않으며, 과거 Issue를 닫았다는 사실도 물리 Gate를 통과했다는 뜻이 아닙니다. [문서/증거 표준](../documentation-standards.md).

## 역사적 M2 독립 검증 보고

The v1.0 development specification contains T01–T54 acceptance test definitions. These tests are NOT implemented merely because IDs appear in documentation. Separate: format parsing fidelity, electrical/numerical reference solutions, coupling/port conventions, adaptive vs direct sweep, GUI responsiveness, crash recovery, confidentiality and native platform execution. Each future PASS must include sample provenance, reference, model validity, method/units, hash, OS/compiler and observed results.

## M2 PH-03 exact mathematical plate (2026-10-15)

- [Pre-registered PH-03 acceptance contract](m2-ph03-parallel-plate-acceptance-contract.md)
- [G1 FEM evidence and PH-03 test results](m2-g1-lossless-fem-evidence.md)
- [Independent M2 plan and G1→G2 order](m2-rlgc-independent-verification-plan.md)

PH-03 is a numerical mathematical anchor only; no real PCB or 30 GHz physical qualification is implied.

## M2 G2 PH-05 ideal symmetric stripline (2026-10-15)

- [Pre-registered PH-05 acceptance contract](m2-g2-ph05-acceptance-contract.md)
- [Independent FVM method, SHA-256 and numeric reference](m2-g2-ph05-independent-reference.md)
- [FEM PH-05 limited acceptance and regression report](m2-g2-ph05-fem-evidence.md)

The PH-05 class-F **limited synthetic PEC/lossless** coupon passed 81 independent acceptance checks in [CI 38666133102](https://github.com/july-rust/BoardSI/actions/runs/38666133102). It does NOT validate actual PCB loss/roughness, 30 GHz, discontinuities or overall M2.

## M2 G2 PH-06 asymmetric and two-region PEC stripline (2026-10-15)

- [PH-06 immutable acceptance contract](m2-g2-ph06-acceptance-contract.md) — preregistered SHA `15da48ef3753e361d009decd03dbd4516dfd9868`.
- [Independent cell-centred FVM, material interface, source/fixture SHA-256](m2-g2-ph06-independent-reference.md).
- [Measured FEM/FVM comparisons, independent upper/lower ground charge, mesh/geometry/material sweeps](m2-g2-ph06-fem-evidence.md).

PH-06 limited synthetic lossless 2D field acceptance recorded 201 numerical and 57 invariant PASS gates in [CI 38669970099](https://github.com/july-rust/BoardSI/actions/runs/38669970099). This **does not** validate a real PCB, 30 GHz, multilayer microstrip, loss or full M2. The existing PH-05 and PH-03 tests remain mandatory.

## M2 G2 PH-07 multilayer microstrip with mask — 2026-10-15

- [Pre-registered PH-07 acceptance contract](m2-g2-ph07-acceptance-contract.md) (first commit `c11440a1`)
- [Independent Python FVM reference and SHA-256](m2-g2-ph07-independent-reference.md)
- [Rust FEM actual calculation/mesh/boundary/charge evidence](m2-g2-ph07-fem-evidence.md)
- [PR #40](https://github.com/july-rust/BoardSI/pull/40), initial [PH-07 CI run 38674095933](https://github.com/july-rust/BoardSI/actions/runs/38674095933): 740 numerical checks PASS (8 C/C0 cross-method comparisons plus 8 derived Z0/delay comparisons), 0 FAIL.

Only restricted synthetic ideal PEC/quasi-TEM electrostatic class F is supported, with explicit rectangular mask/top/side and layered substrate. PR #40 merged at `c6dce055`; final HEAD `875f314a`, [CI 38674727369](https://github.com/july-rust/BoardSI/actions/runs/38674727369) PASS; real PCB, 30 GHz full-wave and R/G loss models remain Not Run.

## M2 validation audit hardening (#41)

[Executed audit, check taxonomy, native platform ledger and administrative limits](m2-validation-audit-hardening.md). Frozen acceptance contracts remain unchanged.

## PH-08 Stage 4 / Stage 5B (historical FAIL; no physical approval)

- [Original Stage 4 36-row HMM FAIL reproduction](m2-g2-ph08-stage4-original-gateq-reexecution.md): 36/36 rows reproduced, historical uniform quadratic gradient ratios below 1.8, FAIL unchanged.
- [Original Stage 4 Recovery operator selected-case audit](m2-g2-ph08-original-recovery-operator-evidence.md): selected captured binary64 matrices and exact rational algebra, not physical forward-error certification.
- [Gate Q captured binary64 T/R proposals](m2-g2-ph08-gateq-tr-exact-captured-proposed.md): proposed synthetic-only registry; original dataset/reduction proof and prospective policy NOT VERIFIED.
- [Stage 5B private original HMM replay runbook](m2-g2-ph08-stage5b-private-replay-runbook.md): SHA-pinned executable source runs only on an approved isolated private host, never in public GitHub CI; it reproduces the historical FAIL rather than physics approval.
- [Stage 5B archive preflight / original replay boundary](m2-g2-ph08-stage5b-original-replay-evidence.md): previous private archive digest record, GitHub public synthetic integrity tests only. **Original Stage 5B privately re-executed after initial preflight: historical FAIL reproduced with 12/12 rows and 1761/1761 deterministic numerical scalars; public CI remains synthetic only.**

Stage 5B archived and newly reproduced Gate Q remain FAIL; AC-P1 PENDING, PH-08 contract NOT FROZEN, Rust FEM NO-GO, Gate M NOT VERIFIED, physical bound BOUND_NOT_CERTIFIED. Synthetic GitHub CI does not change these statuses.
