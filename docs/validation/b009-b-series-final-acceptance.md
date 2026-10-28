# B_009 — B-series Final Integration Acceptance & C-stage Handoff

**기록일:** 2026-10-28 KST · **판정:** B_009 제한 기능 문서 감사/보호된 main 병합/병합 후 Native CI 완료, 정식 B_001 전체 인수는 미완  
**기준:** 보호된 [`main@972c6663a5d3cd0fd9d7f9cfe2d22c24dbc21ef3`](https://github.com/july-rust/BoardSI/commit/972c6663a5d3cd0fd9d7f9cfe2d22c24dbc21ef3) · 상위 [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) **OPEN**  
**필수 비교:** [개발 체크리스트](../development-checklist.md), [사양서 v1.0](../specification/BoardSI_Development_Spec_v1.0.md), [프로젝트 계획](../project-plan.json), [B_002](b002-b001-completion.md), [B_007](b007-copper-cutout-connectivity.md), [B_008](b008-reference-cutout-geometry.md).

## 1. 실행 전 실제 검증

2026-10-28 KST GitHub REST에서 기준 `main@972c6663` 서명 `verified=true, reason=valid` 확인. 같은 SHA에 대해 [21/21 Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/972c6663a5d3cd0fd9d7f9cfe2d22c24dbc21ef3/checks), **9/9 Workflow Runs SUCCESS**(Push on main, Numerical Core, Qt Runtime, Docs, 관련 수치 및 Native)가 기록되어 있다. 이는 **B_009 이전 이미 병합된 코드**에 대한 검증이지, 새 통합시험/실제 PCB 정확도 증거가 아니다.

B_007 [PR #231](https://github.com/july-rust/BoardSI/pull/231)의 최종 소스 `6a75eb5c5616d86407a04a946bc994958fb55ea6`는 **28/28 Check Runs SUCCESS**, 3개 과거 Codex 리뷰 Thread resolved, 서명된 main 병합 및 별도 병합 후 21/21 CI까지 확인되었다. 그러나 [최신 Codex 자동 리뷰 상태](https://github.com/july-rust/BoardSI/pull/231#issuecomment-6455692908)는 이 정확한 SHA에서 **`status=running`, findings=null**이다. 예전 SHA의 리뷰를 최신 HEAD 리뷰 완료로 바꾸지 않는다. **B_007 독립 최종 리뷰 종료 = NOT_VERIFIED**. [Draft #223](https://github.com/july-rust/BoardSI/pull/223)은 대체 미완료 브랜치이므로 병합 금지.

## 2. B_001–B_008 제한 구현 인수 범위

| 단계 | 이미 검증된 범위와 원장 | 여전히 미인수 |
| --- | --- | --- |
| B_001 | [#213](https://github.com/july-rust/BoardSI/pull/213) Qt 사용자 Stackup UI, [#215](https://github.com/july-rust/BoardSI/pull/215) Rust `Property::Assumed` Material, [#217](https://github.com/july-rust/BoardSI/pull/217) Source Copper/Net 후보의 엄격한 음성 감사 | 완전한 실물 Stackup/도체/전기 Reference 및 공식 U/T |
| B_002 | [#219](https://github.com/july-rust/BoardSI/pull/219) 감사 최신성 검사, [#221](https://github.com/july-rust/BoardSI/pull/221) 출처/Net/주파수에 따른 결정적 **감사 입력 Revision** | 물리 Solver Scenario / Result Cache |
| B_003 | [#224](https://github.com/july-rust/BoardSI/pull/224) 지원 Trace/Pad/Via의 Source Tick 정확 접점과 도금 Barrel, [원장](b003-b001-reference-topology.md) | Pad 몸체/Trace 폭 및 전체 Open/Short |
| B_004 | [#226](https://github.com/july-rust/BoardSI/pull/226) 단일 Hole-free Straight COPCLS 국소 AreaInterior/Continuity, [원장](b004-reference-copper-connectivity.md) | 임의 Pour 및 다중 Island/Cutout |
| B_005 | [#227](https://github.com/july-rust/BoardSI/pull/227) 한 개 Hole-free Reference Polygon의 전체 Straight Centreline **기하** 커버리지, [원장](b005-reference-geometry.md) | P009 물리 Return Current |
| B_006 | [#229](https://github.com/july-rust/BoardSI/pull/229) 제한적 Source-bound Through Via–Copper Area 동일 Net/Layer 접촉, [원장](b006-via-copper-contact.md) | Anti-pad/Thermal/복잡한 Via-to-Plane |
| B_007 | [#231](https://github.com/july-rust/BoardSI/pull/231) 단일 Source-owned Polygon COPCUT 안/경계 Trace/Pad/Via 접촉 거부, [원장](b007-copper-cutout-connectivity.md) | 독립 최신 HEAD Codex 종료 및 일반 Cutout |
| B_008 | [#233](https://github.com/july-rust/BoardSI/pull/233) Trace 중간 구간의 COPCUT 교차·접촉까지 막는 전체 경로 기하 감사, [원장](b008-reference-cutout-geometry.md) | Reference 전기적 연속성/Return Path |

기존 증거는 공개 합성 **PADS V9.4 MILS 제한 Subset**에 한한다. 기존 기능의 Native CI/수치 회귀 결과를 종합하되 **B_009 새 코드를 시험한 것처럼 표현하지 않는다**.

## 3. 통합 회귀의 핵심 불변 조건

1. Original Source Bytes/SHA, Source Area/Net/Pad/Padstack/Drill/Via/Trace/COPCUT ID 및 Layer Order가 전부 대조되어야 한다. 변조된 Canonical 증거는 반드시 Fail-closed.
2. Net 레이블만 같아서 도체를 단락하지 않는다. 실 Source 좌표·접촉·Layer·도금 Span을 각각 확인한다.
3. 분리된 Island/Cross-Net, 미지원 형상, 복수/곡선 Cutout, 모호한 Area 중첩은 거짓 연결로 처리하지 않는다.
4. Cutout 안과 경계의 접점뿐 아니라 **양 끝이 Copper 위에 있으나 중간이 Cutout을 통과하는 Trace**도 거부한다.
5. B_002의 감사 SHA는 Solver 물리 Cache 키가 아니다. Stackup, Material, Net, 원본 Source 변화는 결과를 무효화해야 한다.
6. Qt V1 IPC/CPU/RHI Artwork 동작과 Windows x64·ARM64, macOS ARM64, Linux Qt 회귀를 보존한다.

**현재 판정:** 과거 B_003–B_008 각각의 제한 기능 CI = 각 원장 기준 PASS; 현재 보호 `main` = 21/21 CI SUCCESS. **B_009 PR의 신규 최종 HEAD CI·Codex·병합 후 main CI는 아직 이 원장에서 확인되지 않았으므로 NOT_VERIFIED**. 기능 결함이 실제 발견되면 해당 제한 범위에서 수정하고 재시험한다. 넓은 신규 물리 기능은 이 감사 PR에 넣지 않는다.

## 4. B_001 원래 계획의 미완료 Gate — C단계 인계

| 원래 Issue #12 조건 | 현재 판정 | 후속 요구 |
| --- | --- | --- |
| Source-complete Copper/Pad/Via/Area 전기 Net 그래프 | **PARTIAL** | 다중 Copper/Cutout/Island·실접촉 판정 |
| 검증된 Reference Copper 및 Split/Transition | **PARTIAL GEOMETRY ONLY** | 실제 전기 연결 및 전환 자격, 미지원 경로 차단 |
| P009 Return Path와 Signal/Reference 전기 분리 | **UNSUPPORTED / NOT_VERIFIED** | 검증 가능한 모델 및 독립 시험 |
| 물리 Solver용 Immutable Scenario/결과 Cache | **NOT_IMPLEMENTED** | B_002 감사 Revision과 구분 |
| 주파수 의존 Conductivity/Loss/Roughness | **NOT_ACCEPTED** | 출처·유효 대역·독립 수치 검증 |
| 정식 U09/U10/U39 및 T02/T04/T24/T52 | **정식 인수 미완료** | 각 공식 시험의 원본, 실패/차단 및 Native 결과 |
| 실제 PCB 기반 DC~30 GHz RLGC/S | **NOT_RUN** | 별도 M2/제품 물리 검증 |

물리 Gate 및 공식 T24를 충족하기 전에는 다음 안전 조건을 유지한다.

```text
return_path_p009      = UNSUPPORTED
solver_ready          = false
physical_si           = NOT_RUN
cache_reuse_eligible  = false
```

**B 시리즈의 제한 구현 종료 ≠ Issue #12 정식 완료.** Issue #12 OPEN, 사양서 U09/U10/U39·T02/T04/T24/T52 체크리스트 `[ ]` 보존. M0/M1/M2/PH-08 Gate 및 Draft #35 정책도 변경하지 않는다.

## 5. C단계 권장 분할 (여기서 개발하지 않음)

- **C_001:** 단일 Source-backed 연결 그래프에서 **분리 Island 및 복수 Straight Cutout**에 대한 접촉 분류 확장. Source·Net·Layer 증명, 오류 Fixture와 Native CI로 제한 인수. Solver/P009 제외.
- **C_002:** Reference Split 및 Layer Transition의 Source-backed **Topology Risk/Unsupported 판정**. 실제 AC Return 효과 수치화 금지.
- **C_003:** B_002 Audit Revision과 구분되는 불변 **Solver 입력 Scenario / Stale Result 계약**. 캐시 결과 물리 정확도는 별도.
- **C_004:** Material Loss, Conductivity, Roughness 입력의 출처·주파수 유효 범위 및 독립적인 계산 기준 정립. 근거 없는 FR-4 기본값 금지.

각 명령은 하나의 리뷰 가능한 범위로 구현→시험→체크리스트→보호 병합까지 종료한다. 연결되는 Port/M1b, RLGC/M2, Cache/M2b 및 Issue #12 전체 정식 인수는 별도 게이트로 남긴다.

## 6. B_009 실제 최종 병합 및 완료 증거

- [x] **사전 감사:** 2026-10-28 KST 기준 `main@972c6663` 서명 Verified, 21/21 Checks 및 9/9 Workflows 조회. 원본 체크리스트 최초 날짜 보존.
- [x] **검증 원장과 C단계 인계:** 이 문서는 제한 B_001–B_008 기하 기능을 인정하되 B_001 Issue #12 전체 인수는 승인하지 않는다.
- [x] **PR 검사:** [PR #235](https://github.com/july-rust/BoardSI/pull/235) 최종 HEAD `a38facad54b457947009e3a364ce5bfdb3a6a37d`에서 **22/22 Checks SUCCESS**(Windows x64/ARM64, macOS ARM64 및 Linux Qt 포함).
- [x] **독립 리뷰:** 같은 HEAD Codex Review `completed`, `has_any_findings=false`, `has_unresolved_findings=false`, 미해결 Review Thread 0.
- [x] **보호 Squash Merge:** 2026-10-28 KST, [`main@47f4a6de7125eb49cb504c3b5d91aa59d908031f`](https://github.com/july-rust/BoardSI/commit/47f4a6de7125eb49cb504c3b5d91aa59d908031f), GitHub REST 서명 `verified=true, reason=valid`, branch protected.
- [x] **별도 병합 후 검증:** 정확히 `47f4a6de` SHA의 [21/21 Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/47f4a6de7125eb49cb504c3b5d91aa59d908031f/checks), **9/9 Workflow Runs SUCCESS**, [Push on main #40623750981](https://github.com/july-rust/BoardSI/actions/runs/40623750981), [Qt Runtime #40623751051](https://github.com/july-rust/BoardSI/actions/runs/40623751051).
- [ ] **B_007 final SOURCE HEAD Codex closure:** [PR #231](https://github.com/july-rust/BoardSI/pull/231) `6a75eb5c` review receipt remains `status=running` / NOT_VERIFIED despite its separate 28/28 CI and resolved prior comments.
- [ ] **B_001 full formal M1 acceptance:** [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN; actual P009/Return Path, physical Scenario/Cache, losses, T02/T04/T24/T52 and U09/U10/U39 remain unchecked.

**B_009 제한 범위 판정: PASS** — signed merge and post-main CI independently verified. **B_001 전체 인수: PARTIAL/OPEN**. This documentation closeout update itself is a separate PR and must not be reported as merged until its own CI/merge evidence exists.
