# BoardSI 아키텍처 지도 | Architecture

> **프로젝트 코드 기준: 2026-11-07 KST.** Qt/Rust 네이티브 개발 프리뷰와 제한된 PADS SOURCE 기하 파서는 존재하지만, 실제 PCB의 완전한 Connectivity/Port/물리 SI 데이터 경로는 **아직 공식 완료되지 않았습니다**.
>
> 문서 홈: [전체 문서 안내](../README.md) · 사양: [v1.0](../specification/BoardSI_Development_Spec_v1.0.md) · 구현 상태: [단일 체크리스트](../development-checklist.md).

## 1. 한눈에 보는 모듈과 경계

| 모듈 | 역할 | 현재 검증된 범위와 한계 |
| :--- | :--- | :--- |
| [\`crates/boardsi-import\`](../../crates/boardsi-import/) | 입력 형식 식별, PADS V9.4 MILS 지원 서브셋의 SOURCE 형상·부품/Pad 추출 | 제한 Import, 실제 모든 벤더 파일/ODB++ 미지원 |
| [\`crates/boardsi-model\`](../../crates/boardsi-model/) | 형식 중립 객체, Source Provenance, Frame, Stackup, Net, 연결 근거 | 합성 모델 유효성·불변 소유권 시험; 전 보드 원본 지원 미완 |
| [\`crates/boardsi-numerics\`](../../crates/boardsi-numerics/) | 주어진 균일 RLGC의 scalar 2-port 수치 계산 | 기준 수학 계산. 실제 PCB 기하/재료→RLGC 유도 아님 |
| [\`crates/boardsi-ipc\`](../../crates/boardsi-ipc/) | Snapshot/Worker 프레임 통신 규격 | 버전형 로컬 IPC 및 프로토타입 무결성 |
| [\`apps/boardsi-worker\`](../../apps/boardsi-worker/) | GUI와 분리된 계산 프로세스 | 합성 입력 프로토타입, 전체 PCB SI Worker 미완 |
| [\`apps/boardsi-qt-prototype\`](../../apps/boardsi-qt-prototype/) | Qt Widgets GUI/제한 Artwork CPU·Qt Quick-RHI 렌더링 | 네이티브 실행/회귀시험. 제품 워크플로·UI 연결성/배포 완료 아님 |
| [\`prototypes/m2-lossless-fem\`](../../prototypes/m2-lossless-fem/) | 독립 G0/G1/PH-03–07 제한 모델 검증 | 수학·PEC/lossless 연구 쿠폰. 실제 PCB 전자기 정확도 보증 아님 |

**의도된 데이터 흐름:** \`별도 CAD Importer\` → \`Canonical Board Model\` → \`Viewer·Connectivity·Port/Scenario\` → \`검증된 2D RLGC/S Solver\` → \`그래프·Touchstone\`.

이 흐름의 **일부**만 코드로 입증됐습니다. 화살표로 이어져 있다는 설명이 구현 완료를 뜻하지 않습니다. 특히 SOURCE-local Copper 접촉 후보를 전체 Canonical 전기 그래프 Edge로 자동 승격하지 않습니다.

## 2. 중요한 설계 계약

| 주제 | 결정 및 근거 | 실무상 주의사항 |
| :--- | :--- | :--- |
| **원본 독립성** | [Canonical Board Model](canonical-board-model.md) | PADS/ODB++는 외부 *입력*입니다. ODB++가 내부 정식 포맷인 것은 아닙니다 |
| **정밀도·Provenance** | [Canonical Model](canonical-board-model.md) | SOURCE 좌표·단위/원점/Layer, 원본 해시, 지원하지 못하는 변환의 손실 상태 유지 |
| **구리 접촉** | [공통 Trace–Via–Pad 기하 계약](trace-copper-contact.md) | 임의 방향 직선·원호, 비중심 Via/Pad 접촉. **지원 SOURCE 국소 후보**, 전체 전기적 Open/Short가 아님 |
| **수치 계산** | [M0 RLGC 수치 계약](numerical-contract.md) | 공급된 scalar RLGC→2-port S와 **기하→물성→RLGC** 모델을 분리 |
| **GUI ↔ 계산** | [Qt Worker IPC](worker-ipc-qt-prototype.md) · [Qt 런타임 기록](qt-runtime-followup.md) | GUI 메인 스레드 비차단, 순서/취소/스냅샷 검사. 네이티브 테스트와 릴리즈 승인 분리 |
| **실험 FEM** | [M2 물리 모델 ADR](m2-rlgc-physical-model-adr.md) · [검증 개요](../validation/README.md) | 합성 PEC/lossless 수학 PASS와 실 PCB 손실·DC–30 GHz 제품 승인은 다름 |
| **실행/재현** | [개발 가이드](../getting-started.md) · [Qt Preview](../../apps/boardsi-qt-prototype/README.md) | 공개 합성 픽스처와 현재 빌드 설정에 한정 |

## 3. Trace의 임의 각도·비중심 접촉에 관한 확정 사항

D_014–D_053의 과거 실험에는 특정 경사·양/음 좌표·Via 중심점·Pad 모서리 조합이 많이 등장합니다. **이들은 과거 회귀시험 기록이지 생산 코드에 그 모든 각도를 계속 열거하라는 설계 규칙이 아닙니다.**

- **D_054:** 입력 좌표 자체로 모든 지원 각도의 직선·CW/CCW 원호 Trace 사이 유한 선폭 구리 접촉을 계산.
- **D_055:** Via의 실제 **바깥 Pad 구리와 안쪽 Drill**을 분리하고, 원형·사각형 SMD Pad의 **전체 도체 면적**을 대상으로 비중심 접촉을 계산.
- **출력 의미:** 동일 선언 Net/서로 다른 Net/Pad Net 미확인을 분리한 국소 기하 후보. **기존 Qt Artwork 원본 프로토콜은 변경하지 않음**.
- **여전히 남은 것:** 모든 복잡한 Padstack·Custom Shape·Pour/Thermal 및 **Canonical 전체 전기 연결 그래프·GUI 통합·실측 정확도 검증**.

기술 공식, 반례, 경계조건과 SOURCE 해시 근거: [공통 계산 계약](trace-copper-contact.md), [D_054/D_055 검증](../validation/d054-d055-general-trace-contact-qualification.md).

## 4. 미결정 사항과 공식 게이트

- **M0:** 실제 Worker·Qt·Canonical Snapshot 프로토타입은 있지만 [정식 M0 완료 Gate](../validation/m0-completion-gate.md)는 **BLOCKED**입니다.
- **M1:** 전체 PADS/ODB++ Import, 실용적인 Reference/Port, 완전 Connectivity 및 실제 Layout↔Solver 일관성은 미완료입니다.
- **M2:** 진짜 손실·분산·다도체/차동/리턴경로 등과 검증된 제품 결과는 미완료입니다. [수치 연구 결과](../validation/README.md)만으로 승격하지 않습니다.
- **PH-08:** Original HMM Gate Q는 역사적 **FAIL**이고 물리 Error Bound **BOUND_NOT_CERTIFIED**입니다. [원본 재실행 근거](../validation/m2-g2-ph08-stage5b-original-replay-evidence.md).
- **G5 설계 문서 PR #35:** 2026-11-07 확인 결과 **Closed, Not Merged**입니다. 과거 원장의 \`Draft PR #35\`는 **당시 상태**이지 현재 열린 PR이 아닙니다. G5 기능 구현 근거로도 사용하지 않습니다.
- **Qt event-loop Issue #46:** 2026-11-07 기준 **Open**; 1,000 ms 과거 실패 및 소유자 승인 **임시 M0 1,500 ms 규칙**을 구분합니다. [당시 정책 결정](../validation/m0-f8-provisional-1500ms-policy.md).

**주의:** 각 개별 Issue·PR의 상태는 시간이 흐르면 바뀔 수 있습니다. 위 상태는 기준일의 확인 결과이며 최신 공식 진행 상태는 [체크리스트](../development-checklist.md)와 해당 GitHub Issue/PR에서 재확인해야 합니다.

## 5. 과거 ADR과 증거 찾기

- [M0 Model-Gated Worker 인계](m0-model-gated-worker-handoff.md) 및 [Synthetic Model Snapshot v2](m0-synthetic-model-snapshot-v2.md)
- [초기 Qt Linux Runtime ADR](qt-native-runtime.md)와 [후속 네이티브 런타임 수정](qt-runtime-followup.md)
- [독립 검증 보고서 158개 색인](../validation/INDEX.md)
- [2026-10-18 프로젝트 원본 감사](../audits/2026-10-18-pre-ph08-project-audit.md) — **역사적 스냅샷**
