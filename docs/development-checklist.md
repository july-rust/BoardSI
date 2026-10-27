# BoardSI 개발 사양서 이행 체크리스트 — A_036

> 작성: 2026-10-26 KST · 추적 [A_036 Issue #209](https://github.com/july-rust/BoardSI/issues/209)  
> 기준: [개발 사양서 v1.0](specification/BoardSI_Development_Spec_v1.0.md) (U01–U48 / T01–T54), 착수 시 보호 `main@8168298e5a2f64966220ead49017aeef2b4844ad`, [A_035 범위 감사](validation/a035-p2z-final-preview-acceptance.md).  
> **사양서의 요구사항 확정 ≠ 프로그램 구현 완료 ≠ 정식 인수시험 PASS.** 아래 상태는 검증된 범위만 표시하며 사양서·PH-08 물리 Gate·릴리즈 정책을 변경하지 않습니다.

## 목차

- [현재 현황](#1-현재-현황) · [A시리즈 확인 목록](#2-a시리즈-구현-확인-목록) · [후속 개발](#3-후속-개발-체크리스트)
- [U01–U48](#4-요구사항-u01u48-체크리스트) · [T01–T54](#5-인수시험-t01t54-체크리스트) · [관리 규칙](#6-갱신-및-완료-판정-규칙)

## 1. 현재 현황

| 영역 | 확보된 증거 | 정식 완료 여부 |
| --- | --- | --- |
| A_001–A_035 P2 Preview | 제한된 PADS V9.4 MILS 합성 원본 Import→Canonical/IPC→Qt CPU/RHI Artwork; 3 Native CI ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md)) | **부분 기능 PASS**, 범용 PADS/실 PCB 해석 아님 |
| M0 구조·기술 | Rust/Qt·IPC·Canonical·3 Native 프로토타입 ([M0 종료 게이트](validation/m0-completion-gate.md)) | **BLOCKED**: F1/F6/F7 등 독립 종료 게이트 남음 |
| M1 PADS/Artwork | PADS subset, Copper/Cutout, Source parity, 제한 국소 Route 접점 ([A_034 Native 통합](validation/a034-p2y-integration-stabilization.md)) | **PARTIAL**, ODB++ 및 전체 Connectivity 미완 |
| M1 Stackup/Reference (B_001) | Qt Overlay UI + Rust Canonical **사용자 가정 Material 전용** 감사 PASS ([#213](https://github.com/july-rust/BoardSI/pull/213), [#215](https://github.com/july-rust/BoardSI/pull/215)); 실제 Copper/Net/Return Path·물리 Solver 미검증 | **두 제한 기능 PASS / M1 정식 미완** |
| M1a / M1b | Via/Pad 일부 기하 연구, 실 Port 자동/수동 전체 범위 증빙 없음 | **미완** |
| M2 연구·제품 Solver | PEC/lossless FEM 합성 쿠폰 수학 검증만 존재 ([M2 검증 원장](validation/README.md)) | **제품 Solver 미완**; PH-08 원본 Gate Q **FAIL 유지** |
| M2a/M2b/M2c, M3–M5 | 단계별 요구 명세 있음; 일부 기술 시연만 존재 | **미완 / 후속** |
| GPU 렌더 | 제한 RHI Native Frame/Picking 회귀 | 고객 GPU 드라이버·VRAM·Reset·1M@60FPS **미검증** |
| Security / 출시 | SEC_001 코드 변경 및 CodeQL CI 이력 ([SEC_001 코드 검사](validation/sec001-codeql-markdown-comment-scan.md)) | 실제 Alert #1 상태 NOT_VERIFIED; 라이선스·배포 미완 |

**날짜 기준:** 모든 `[x]`는 `체크일`(처음 체크리스트에 기록된 날짜, KST)과 `근거일`(참조 PR의 병합일 또는 사양서 개정일, KST)을 별도로 표시합니다. 최초 체크 13개는 2026-10-26에 등록했습니다. 선행 근거일이 전체 제품 인수 완료를 의미하지는 않습니다.

**표시 기준:** `[x]`는 **체크된 문장에 명시된 한정된 범위만 완료**를 뜻합니다. `부분 증빙`은 최종 인수 미완입니다. `[ ]`를 `[x]`로 변경하려면 해당 범위의 실행 결과·원본·SHA·리뷰·미해결 문제를 대조해야 합니다. 사양서에서 `확정`된 요구도 개발 완료로 자동 처리하지 않습니다.

## 2. A시리즈 구현 확인 목록

### 실제 실행 증빙이 있는 제한 Preview

- [x] Qt Widgets Preview와 별도 Rust Worker / Versioned IPC의 제한 작동 — [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) · **체크일: 2026-10-26 KST** · **근거일: 2026-10-19 KST** ([PR #102](https://github.com/july-rust/BoardSI/pull/102))
- [x] 공개 합성 PADS Layout ASCII V9.4 MILS의 **명시된 subset만** 파싱 및 비지원 입력 거절 — [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) · **체크일: 2026-10-26 KST** · **근거일: 2026-10-22 KST** ([PR #159](https://github.com/july-rust/BoardSI/pull/159))
- [x] 지원 Straight Trace/Arc/Via/Component·RF Pad 및 Bottom Mirror 변환 — [A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md) · **체크일: 2026-10-26 KST** · **근거일: 2026-10-23 KST** ([PR #167](https://github.com/july-rust/BoardSI/pull/167))
- [x] 지원 Board Outline과 다각형·곡선·복합 외곽 — [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST** ([PR #208](https://github.com/july-rust/BoardSI/pull/208))
- [x] 지원 Copper Area/원형·곡선·비직교 외곽, Cutout/Hole와 CPU/RHI 표시 — [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) · **체크일: 2026-10-26 KST** · **근거일: 2026-10-25 KST** ([PR #192](https://github.com/july-rust/BoardSI/pull/192))
- [x] 원본 Source tick·좌표·원본 개체별 Provenance와 Artwork 독립 검증 — [A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md) · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST** ([PR #200](https://github.com/july-rust/BoardSI/pull/200))
- [x] 제한 Layer·Net 표시, Filtering, Picking, Zoom/Fit 및 Qt Quick RHI Presented Frame — [A_034 Native 통합](validation/a034-p2y-integration-stabilization.md) · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST** ([PR #203](https://github.com/july-rust/BoardSI/pull/203))
- [x] 원본 정확히 일치하는 일부 Trace/Arc/Via **국소 접점만** 그래프 근거 생성 — [A_032 국소 접점](validation/a032-p2w-local-connectivity.md) · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST** ([PR #196](https://github.com/july-rust/BoardSI/pull/196))
- [x] CPU/RHI 국소 접점 마커와 소스 정보/Visible Frame 정합성 — [A_034 Native 통합](validation/a034-p2y-integration-stabilization.md) · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST** ([PR #203](https://github.com/july-rust/BoardSI/pull/203))
- [x] 연속 다른 PADS 합성 보드 재Import의 stale 객체·필터 상태 제거 — [A_034 Native 통합](validation/a034-p2y-integration-stabilization.md) · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST** ([PR #202](https://github.com/july-rust/BoardSI/pull/202))
- [x] 10K·100K·1M **합성** 객체의 렌더/상호작용 회귀 시험(성능 보증 아님) — [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) · **체크일: 2026-10-26 KST** · **근거일: 2026-10-24 KST** ([PR #173](https://github.com/july-rust/BoardSI/pull/173))
- [x] Windows x64·Windows ARM64·macOS ARM64 **Native CI 시험 환경** 범위 PASS — [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST** ([PR #208](https://github.com/july-rust/BoardSI/pull/208))

### A_036 이후에도 반드시 남겨 두는 미완 항목

- [ ] 임의 상용 PADS 파일·버전의 전체 객체를 자격 검증 — [Issue #9](https://github.com/july-rust/BoardSI/issues/9)
- [ ] Whole-board Open/Short, Pad/Pin/Pour/Thermal 접속의 물리 판정 — [A_032 국소 접점](validation/a032-p2w-local-connectivity.md)
- [ ] GPU VRAM/driver reset/실장 기기/1M@60 FPS를 정량 측정 — [Issue #165](https://github.com/july-rust/BoardSI/issues/165)
- [ ] 실제 PCB/Substrate 기반 RLGC/S·DC/30 GHz·Touchstone 검증 — [M2 검증 원장](validation/README.md)
- [ ] 정식 설치/저사양 실물 PC/재배포 라이선스 검증 — [M0 종료 게이트](validation/m0-completion-gate.md)

### B_001 추가 검증 — 제품 물리 자격과 구별

- [x] **B_001 제한 Qt Stackup/Reference Overlay UI** — Import Layer 번호 기반 Unknown 초기화, 수동 Material/Dk/Df/두께 입력 검사, 원본 SHA-256에 바인딩된 JSON 저장·복원, ReImport 초기화 및 원본 파일 덮어쓰기 방지의 **공개 합성 소스 Native 테스트 범위만** 완료 · **체크일: 2026-10-27 KST** · **근거일: 2026-10-26 KST** ([PR #213](https://github.com/july-rust/BoardSI/pull/213), [exact-head 26/26 검사](https://github.com/july-rust/BoardSI/commit/e5f449398c9f22a02878e397d69bb8b34319b252/checks), [Qt 3 Native + Linux 증거](validation/b001-stackup-reference.md)). **물리 Stackup/Net Reference 및 U09/U10/T02/T04/T24/T52 전체 PASS가 아님.**
- [x] **B_001_2 Rust Canonical Stackup/Material의 제한된 사용자 가정 검증** — 명시적 Layer·Material·출처 Revision·측정 방법·기준/유효 주파수 → 실제 `boardsi-model::BoardModel`의 `Property::Assumed`와 연속 z/전체 두께 계산, 대역 밖 차단, 변경 시 감사 무효화, Windows x64·ARM64/macOS ARM64/Linux native Qt→Rust 경로 PASS. 별도의 검증된 **Reference Net/Copper 및 물리 SI 실행은 미구현** · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** ([PR #215](https://github.com/july-rust/BoardSI/pull/215), [native Qt CI](https://github.com/july-rust/BoardSI/actions/runs/40419571718), [Rust Numerical CI](https://github.com/july-rust/BoardSI/actions/runs/40419571744), [한정 범위·검증 원장](validation/b001-2-canonical-stackup-audit.md)). 정식 U09/U10/U39/T02/T04/T24/T52 `[x]` 아님.
- [x] **B_001_3 제한 PADS 원본 Copper/Net Reference 후보 감사** — 성공 Import의 source Area ID·Layer·Net 레이블(누락 `-`는 Unknown)·Cutout 목록을 Native Rust 감사로 전달. 동일 Net Signal/Reference 조합 거부, 분할/Cutout·Net 누락·불완전 목록은 전기적 Return Path를 **Unsupported로 차단**; 실제 PADS 공개 합성 Import/GPU Artwork 및 Native 3 OS+Linux Qt 시험 **코드 HEAD 27/27 PASS** · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** ([PR #217](https://github.com/july-rust/BoardSI/pull/217), [코드 CI](https://github.com/july-rust/BoardSI/commit/d15b86db4cc044da07930fbc9baad41ea0bc891a/checks), [원장](validation/b001-3-source-reference-inventory.md)). **Reference 전기적 접속/Return Path P009 및 정식 T04/T24 인수 완료 아님.**
- [ ] **B_001 전체 Issue #12 완료** — 제한 Qt Stackup UI [#213](https://github.com/july-rust/BoardSI/pull/213), Rust Canonical 가정 Material [#215](https://github.com/july-rust/BoardSI/pull/215), 원본 Copper/Net **음성·Unsupported 판정** [#217](https://github.com/july-rust/BoardSI/pull/217)만 검증 완료. **실제 Copper/Net/Area/Pad/Port의 완전한 Canonical 연결 및 전기적 Reference/Return Path, Scenario/Cache Revision, 손실·거칠기 모델과 전체 정식 U/T 시험은 미완** ([#12](https://github.com/july-rust/BoardSI/issues/12), [B_001_3 원장](validation/b001-3-source-reference-inventory.md)).

- [x] **B_002의 제한적 B_001 Qt 감사 최신성 보강** — Stackup/Material/주파수·Reference Net 및 원본 Copper/Net 정보 변경 뒤 과거 Rust 감사 결과 무효화, 입력 JSON SHA-256 비교, 검증 중 외부 PCB 변경 거절을 Qt Native 테스트로 확인. **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** ([PR #219](https://github.com/july-rust/BoardSI/pull/219), 최종 PR HEAD `f9deda898b97a42886cfbdbdae972924c3fcdc56` 26/26 검사 PASS, 동일 HEAD Codex 리뷰 주요 지적 0, [서명된 병합 `6e795049b6de920f99053ae6361d487a7caeec96`](https://github.com/july-rust/BoardSI/commit/6e795049b6de920f99053ae6361d487a7caeec96), [B_002 원장](validation/b002-b001-completion.md)). **UI 감사 세션 한정**이며 실제 물리 Scenario/Cache, Copper/Pad/Port 연결 및 Return Path, Material 손실/거칠기 모델과 정식 T02/T04/T24/T52는 아직 미검증. **B_001 전체 Issue #12는 미완료/OPEN 유지.**

## 3. 후속 개발 체크리스트

| 단계 | 상태 | 구체적 잔여 작업 (GitHub) |
| --- | --- | --- |
| M0 | ☐ BLOCKED | [M0 종료 게이트](validation/m0-completion-gate.md) — 사양/시험 개별 인수·실물 기준 PC/공개 샘플·라이선스/배포 |
| M1 Import/Artwork | ☐ PARTIAL | [#8](https://github.com/july-rust/BoardSI/issues/8), [#9](https://github.com/july-rust/BoardSI/issues/9), [#10](https://github.com/july-rust/BoardSI/issues/10), [#11](https://github.com/july-rust/BoardSI/issues/11): 포맷별 Import 완성도·ODB++·전체 연결 |
| **B_001 / M1 Stackup·Reference** | ◐ 제한 UI PASS / 전체 미완 | [PR #213](https://github.com/july-rust/BoardSI/pull/213) · [B_001 실행 근거](validation/b001-stackup-reference.md): 공개 합성 Source-bound 편집·검증·저장·재Import는 Native 검증 PASS; 물리 Material/Canonical 연결·Reference 도체·인수시험은 미완 |
| M1a | ☐ PARTIAL | [#13](https://github.com/july-rust/BoardSI/issues/13): Substrate Pad/Via/Anti-pad/Plane·Pad-to-Pad |
| M1b | ☐ 미완 | [#14](https://github.com/july-rust/BoardSI/issues/14), [#15](https://github.com/july-rust/BoardSI/issues/15): Port 후보/수동·Single/Differential·P001–P012 |
| M2 | ☐ 미완 | [#16](https://github.com/july-rust/BoardSI/issues/16), [#17](https://github.com/july-rust/BoardSI/issues/17): 물리 검증된 RLGC, Direct S, DC, Touchstone |
| M2a | ☐ 미완 | [#18](https://github.com/july-rust/BoardSI/issues/18), [#19](https://github.com/july-rust/BoardSI/issues/19): Progressive Sweep/Region·결과 Live |
| M2b | ☐ 미완 | [#20](https://github.com/july-rust/BoardSI/issues/20), [#21](https://github.com/july-rust/BoardSI/issues/21): RAM/SSD Cache·Snapshot·강제종료/복구 |
| M2c | ☐ 미완 | [#22](https://github.com/july-rust/BoardSI/issues/22): Via/Pad 불연속 독립 물리 검증 |
| M3 | ☐ 미착수 | [#23](https://github.com/july-rust/BoardSI/issues/23): IPC-2581/KiCad/Gerber/Drill/Netlist |
| M4 | ☐ 미착수 | [#24](https://github.com/july-rust/BoardSI/issues/24): Altium/Allegro/EAGLE/EasyEDA 평가 |
| M5 | ☐ 미착수 | [#25](https://github.com/july-rust/BoardSI/issues/25): 기타 CAD·성능·모델 확대 |

**주의:** M0/M1/M2는 사양서의 제품 개발 단계이며 A_001…A_036, B_001… 명령 번호와 다릅니다. 제한 P2 Preview를 마쳤다고 M0/M1/M2 전체의 정식 인수를 완료 처리하면 안 됩니다.

## 4. 요구사항 U01–U48 체크리스트

원문: [사양서 2장 요구사항 표](specification/BoardSI_Development_Spec_v1.0.md). `[ ]`는 전체 요구의 정식 인수 **미완료**. U16만 실제 제품명 적용 여부를 확인할 수 있어 체크하였으며 상표권·법률 검증은 별개입니다.

### 기본 기능 U01–U16

- [ ] **U01** — Full-wave 3D EM을 제외한 경량 RLGC·임피던스·S-parameter — **부분 증빙 / 최종 미완료**: 제한 수학/무손실 쿠폰만 검증; 실제 PCB SI 해석 미검증 ([M2 검증 원장](validation/README.md))
- [ ] **U02** — Rust 계산 코어 + Qt UI — **부분 증빙 / 최종 미완료**: Qt/Rust/Worker Preview 시험 PASS, 최종 배포·전체 제품 미검증 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **U03** — Windows x64·ARM64, macOS Apple Silicon — **부분 증빙 / 최종 미완료**: 3개 Native CI 환경에서 Preview 구동, Clean PC 배포 미검증 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **U04** — 저사양 PC에서도 가벼운 동작 — **부분 증빙 / 최종 미완료**: 8 GiB Linux 합성 실험, 기준 실물 PC 미검증 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **U05** — 공개 GitHub에서 개발, 사내 데이터 비공개 — **부분 증빙 / 최종 미완료**: 공개 GitHub 운영; 기밀 데이터 무유출 최종 인수는 별도 ([SEC_001 코드 검사](validation/sec001-codeql-markdown-comment-scan.md))
- [ ] **U06** — PADS Layout ASCII (*.ASC) 지원 — **부분 증빙 / 최종 미완료**: PADS V9.4 MILS 공개 합성 subset만 검증 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **U07** — 다양한 PCB 형식 확장(ODB++, IPC-2581, KiCad, Gerber 등) — **정식 인수 미검증**
- [ ] **U08** — 자체 2D Artwork, Layer/Net 확인 — **부분 증빙 / 최종 미완료**: CPU/Qt RHI Preview·Layer/Net 필터 제한 검증 ([A_034 Native 통합](validation/a034-p2y-integration-stabilization.md))
- [ ] **U09** — Stackup 두께, Copper, Dk/Df, Material 편집 — **제한 Qt + 실제 Rust Canonical Assumed Material 연동 PASS / 정식 인수 미완료**: Native 3 OS/Linux; 주파수 출처·유효 대역 검사 PASS ([PR #215](https://github.com/july-rust/BoardSI/pull/215)); 원본 전기적 Copper 전체 형상 및 해석 적용 미완 ([B_001 검증](validation/b001-stackup-reference.md), [PR #213](https://github.com/july-rust/BoardSI/pull/213))
- [ ] **U10** — GND·Power·Reference 지정 — **제한 Layer 역할 힌트+사용자 Net 선택 및 Import 기반 Copper 후보 음성 판정 PASS / 전체 인수 미완료**: Source Net 누락·Cutout·분할·동일 Net 설정은 차단; 실제 접속·전원 분류·완전한 Return Path 자격 미검증 ([B_001_3 원장](validation/b001-3-source-reference-inventory.md)) ([B_001 검증](validation/b001-stackup-reference.md))
- [ ] **U11** — Single/Differential 선택, 선폭·두께 가상 변경 — **정식 인수 미검증**
- [ ] **U12** — DC~30 GHz 등 사용자 주파수 범위 — **정식 인수 미검증**
- [ ] **U13** — CPU/GPU 등 계산 자원 선택 UI — **정식 인수 미검증**
- [ ] **U14** — 영어 기본, 한국어 옵션 — **정식 인수 미검증**
- [ ] **U15** — 공개 검증 샘플, 기밀 PCB 오프라인 검증 — **부분 증빙 / 최종 미완료**: 공개 합성 fixture 존재, 기밀 실보드 오프라인 전 경로 시험 미완 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [x] **U16** — 제품명 BoardSI 사용 — **현재 제품명 적용 확인** · **체크일: 2026-10-26 KST** · **근거일: 2026-10-14 KST** ([사양서 개정일](specification/BoardSI_Development_Spec_v1.0.md))

### 실행/기하/입력 구조 U17–U32

- [ ] **U17** — Progressive/Hierarchical Sweep: 초깃값 1 GHz 간격 예시 → 절반 구간 세분화 — **정식 인수 미검증**
- [ ] **U18** — 계산 완료 주파수 샘플 캐시 재사용, 동일 조건 재계산 방지 — **정식 인수 미검증**
- [ ] **U19** — 실행 중 새로운 계산 결과가 나오면 SI 그래프 실시간 갱신 — **정식 인수 미검증**
- [ ] **U20** — Group × Frequency Region별 독립적인 수렴·상태 판정 — **정식 인수 미검증**
- [ ] **U21** — 일부 그룹/대역 수렴만으로 전체 실행 조기 종료 금지 — **정식 인수 미검증**
- [ ] **U22** — 긴 작업 중 GUI 응답·진행률·취소/일시정지 유지 — **정식 인수 미검증**
- [ ] **U23** — 기능별 소형 모듈, UI와 Solver 분리(필요시 별도 프로세스) — **부분 증빙 / 최종 미완료**: 별도 Rust Worker와 versioned Qt IPC 프로토타입 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **U24** — RAM/SSD 하이브리드 캐시, 체크포인트, 장애 후 복구 — **정식 인수 미검증**
- [ ] **U25** — 분석 전 배선·리턴경로·모델 한계 위험 표시 — **정식 인수 미검증**
- [ ] **U26** — 일반 PCB 외 BGA/FC-BGA Substrate SI 대상 — **부분 증빙 / 최종 미완료**: Canonical/Substrate 일부 형상 연구; 전체 Pad-to-Pad 미구현 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **U27** — Substrate Trace, Pad/Land, Via 및 Anti-pad, Plane 포함 — **부분 증빙 / 최종 미완료**: Via/Pad/Copper 일부 기하 구현; 전 Substrate 전기적 검증 미완 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **U28** — Bump/Cu Pillar/Solder Ball/Wirebond 해석 영역에서 제외 — **정식 인수 미검증**
- [ ] **U29** — Substrate 분석 Port 기준은 양 끝 금속 Pad-to-Pad — **정식 인수 미검증**
- [ ] **U30** — 검증된 모델 없는 3D 불연속은 정확한 해석인 것처럼 계산 금지 — **부분 증빙 / 최종 미완료**: 미검증 Solver를 Validated로 광고하지 않는 정책; 제품 게이트 미완 ([M2 검증 원장](validation/README.md))
- [ ] **U31** — ODB++를 기준 내부 형식으로 사용하지 않고 자체 Canonical Model 사용 — **부분 증빙 / 최종 미완료**: Format-neutral Canonical과 PADS subset 경로 구현 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **U32** — Format-specific Importer를 분리하고 동일한 Viewer/Port/Solver 사용 — **부분 증빙 / 최종 미완료**: PADS importer/Canonical/Viewer 경계 구현; 다형식+Solver 통합 미완 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))

### Port·검증·복구 U33–U48

- [ ] **U33** — Altium, Allegro, EAGLE, EasyEDA, CADSTAR 등 Import 확대 검토 — **정식 인수 미검증**
- [ ] **U34** — Auto Port 후보 생성 + Manual 수정 + 공통 자동 검증 — **정식 인수 미검증**
- [ ] **U35** — Single 기본 2-Port, Differential 기본 물리적 4-Port — **정식 인수 미검증**
- [ ] **U36** — Reference 도체와 기준 임피던스 분리, Pair Mapping 명시 — **정식 인수 미검증**
- [ ] **U37** — Error Port는 Solver 차단, Warning은 표시 및 확인 — **정식 인수 미검증**
- [ ] **U38** — Format/Version/객체별 지원 수준과 누락 정보 명시 — **부분 증빙 / 최종 미완료**: PADS 제한 지원표·진단, 기타 파일 형식 미완 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **U39** — Import 원본 불변, 정규화 및 Provenance 보존 — **부분 증빙 / 최종 미완료**: subset Source grid·provenance 대조 ([A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md))
- [ ] **U40** — Import할 때 보조 데이터(Netlist/Stackup/Drill) 연계 — **정식 인수 미검증**
- [ ] **U41** — 고정 Linear 직접 Sweep과 수렴 검사의 독립 기준선 — **부분 증빙 / 최종 미완료**: 수학 직접 기준선 일부, 제품 Direct sweep 없음 ([M2 검증 원장](validation/README.md))
- [ ] **U42** — 전체 행렬 및 NEXT/FEXT의 방향·종단·결합 그룹 고정 — **정식 인수 미검증**
- [ ] **U43** — 실행·수렴·모델·Limit·입력 신뢰도 상태 분리 — **정식 인수 미검증**
- [ ] **U44** — 불변 Run Snapshot, IPC 버전과 늦은 응답 격리 — **부분 증빙 / 최종 미완료**: 제한 immutable Snapshot/Worker IPC/late-message proof ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **U45** — 프로젝트 스키마·이전 버전 이관·재개·저장 무결성 — **정식 인수 미검증**
- [ ] **U46** — Sweep만 변경 시 동일 물리 조건 직접 계산점 재사용 — **정식 인수 미검증**
- [ ] **U47** — 수치 정확도·GUI 응답 목표 및 단계별 측정 게이트 — **부분 증빙 / 최종 미완료**: 일부 수치·Native CI; 전체 제품 정량 인수 미완 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **U48** — Import 좌표 정밀도·ODB Step·보조 자료 충돌 통제 — **부분 증빙 / 최종 미완료**: PADS 좌표 grid fidelity 일부; ODB Step/파일충돌 미검증 ([A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md))

## 5. 인수시험 T01–T54 체크리스트

원문: [사양서 13장 인수시험](specification/BoardSI_Development_Spec_v1.0.md). 각 Txx의 *전체* 원문 기준과 실제 실행 자료가 일치하는 경우에만 체크. `정식 인수 미검증`은 아무 시험도 실행하지 않았다는 뜻이 아니라 **해당 T 전체 범위의 증거가 부족**하다는 뜻입니다. 특정 하위 테스트 PASS는 별도 표기하되 Txx를 체크하지 않습니다.

### 기본 시험 T01–T15

- [ ] **T01** — ASC 형식·버전·단위 — **부분 증빙 / 정식 인수 미완료**: 제한 PADS 버전/Signature fixture ([A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md))
- [ ] **T02** — 필수 물성·기하 누락 — **제한 Qt 필드 검사 PASS / 정식 인수 미완료**: 실제 PCB·Canonical 입력 전체 누락 검증 아님 ([B_001 검증](validation/b001-stackup-reference.md))
- [ ] **T03** — Layer·Via·Padstack·Copper — **부분 증빙 / 정식 인수 미완료**: 일부 Pad/Via/Copper와 국소 접점만 ([A_032 국소 접점](validation/a032-p2w-local-connectivity.md))
- [ ] **T04** — Reference 분할·전원 분류 — **정식 인수 미검증**: B_001 역할 힌트는 실제 Copper/Net/분할 판정이 아님 ([B_001 원장](validation/b001-stackup-reference.md))
- [ ] **T05** — P/N·Mixed-mode — **정식 인수 미검증**
- [ ] **T06** — 균일 선로 수치 정확도 — **부분 증빙 / 정식 인수 미완료**: 합성 수학/무손실 쿠폰 ([M2 검증 원장](validation/README.md))
- [ ] **T07** — 손실·수동성·상호성 — **부분 증빙 / 정식 인수 미완료**: 무손실 기준 검사만; 손실 미검증 ([M2 검증 원장](validation/README.md))
- [ ] **T08** — Scenario·다중 파일 — **정식 인수 미검증**
- [ ] **T09** — DC·Log·주파수 — **정식 인수 미검증**
- [ ] **T10** — 자원·취소·복구 — **부분 증빙 / 정식 인수 미완료**: 비차단 Worker/Qt 일부; 제품 Crash 복구 미완 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **T11** — GPU 선택 기능 — **정식 인수 미검증**
- [ ] **T12** — 프로젝트·Export — **정식 인수 미검증**
- [ ] **T13** — 3 플랫폼과 언어 — **부분 증빙 / 정식 인수 미완료**: 3 Native CI 시험; 실제 배포·언어 기능 미완 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **T14** — 오프라인·기밀 보호 — **정식 인수 미검증**
- [ ] **T15** — 패키지 일관성 — **정식 인수 미검증**

### Importer/Port T16–T28

- [ ] **T16** — ASC·ODB++ 등 동일 보드 Cross-Format — **정식 인수 미검증**
- [ ] **T17** — 동일 확장자 다른 형식 — **부분 증빙 / 정식 인수 미완료**: 지원 PADS Header 구분, 전체 .brd/.asc 형식 아님 ([A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md))
- [ ] **T18** — Gerber Only Import — **정식 인수 미검증**
- [ ] **T19** — Gerber+Drill+IPC-D-356B+Stackup — **정식 인수 미검증**
- [ ] **T20** — 미지원/유실 객체 강제 입력 — **부분 증빙 / 정식 인수 미완료**: 제한 비정상 입력 Fail-closed; 전체 객체 분류 미완 ([A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md))
- [ ] **T21** — Canonical Model 안정성 — **부분 증빙 / 정식 인수 미완료**: Canonical synthetic 구조검증; Cross-format ID 미완 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **T22** — Import 압축/비정상 파일 — **정식 인수 미검증**
- [ ] **T23** — 수동 Port Copper 외부 클릭 — **정식 인수 미검증**
- [ ] **T24** — GND Plane Cutout/분리 — **정식 인수 미검증**: 물리 Return 경로 검증 미실시 ([B_001 원장](validation/b001-stackup-reference.md))
- [ ] **T25** — Pair P/N 역전/Port 번호 변화 — **정식 인수 미검증**
- [ ] **T26** — Branch/Series R·C 포함 신호 — **정식 인수 미검증**
- [ ] **T27** — Reimport 후 Port 이동/Net 변경 — **정식 인수 미검증**
- [ ] **T28** — Near/Far 4-Port 행렬 — **정식 인수 미검증**

### Sweep·복구·플랫폼 T29–T40

- [ ] **T29** — 단순 Microstrip·Stripline 1차 Sweep — **정식 인수 미검증**
- [ ] **T30** — Group×Region 독립 수렴 — **정식 인수 미검증**
- [ ] **T31** — 노치·급변 응답 — **정식 인수 미검증**
- [ ] **T32** — Abort/Pause/Resume/Crash Restart — **정식 인수 미검증**
- [ ] **T33** — Stackup/Port/Load 변경 후 캐시 — **정식 인수 미검증**
- [ ] **T34** — GUI 스트레스(무거운 Import/해석) — **부분 증빙 / 정식 인수 미완료**: Artwork UI/Viewer 스트레스 일부 ([A_034 Native 통합](validation/a034-p2y-integration-stabilization.md))
- [ ] **T35** — RAM 부족 및 SSD Spill — **정식 인수 미검증**
- [ ] **T36** — Substrate Pad-to-Pad 모델 경계 — **정식 인수 미검증**
- [ ] **T37** — Microvia/IVH/Plane Anti-pad — **정식 인수 미검증**
- [ ] **T38** — 3D 구조의 모델 미지원 — **정식 인수 미검증**
- [ ] **T39** — 세 플랫폼 결과·Import 재현성 — **부분 증빙 / 정식 인수 미완료**: 3 Native subset 실행만; 실 PCB 동일 S 행렬 아님 ([A_035 감사](validation/a035-p2z-final-preview-acceptance.md))
- [ ] **T40** — 정식 공개 릴리즈 산출물 — **정식 인수 미검증**

### 고급 인수 T41–T54

- [ ] **T41** — 고정 Direct와 Progressive 교차 비교 — **정식 인수 미검증**
- [ ] **T42** — NEXT/FEXT와 결합 그룹 — **정식 인수 미검증**
- [ ] **T43** — 비수동·비상호·특이행렬 입력 — **정식 인수 미검증**
- [ ] **T44** — 불균등 격자·부분 Export·한글 Net — **정식 인수 미검증**
- [ ] **T45** — 완료·수렴·모델·Limit 상태 조합 — **정식 인수 미검증**
- [ ] **T46** — 늦은 IPC·중복 결과·편집 중 Run — **부분 증빙 / 정식 인수 미완료**: Worker stale reply 격리 일부; 제품 전체 Run 미완 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **T47** — 프로젝트 이관·동시 저장·경로 변경 — **정식 인수 미검증**
- [ ] **T48** — 세분화·Region 경계·재개 — **정식 인수 미검증**
- [ ] **T49** — 진단 출력·캐시·공개 패키지 — **정식 인수 미검증**
- [ ] **T50** — UI·Heartbeat·Cancel 시간 계측 — **부분 증빙 / 정식 인수 미완료**: Qt 타이밍 회귀 일부; 원인/출시 목표 미확정 ([M0 종료 게이트](validation/m0-completion-gate.md))
- [ ] **T51** — 미세 Substrate·ODB Step·극성 — **부분 증빙 / 정식 인수 미완료**: PADS source tick 좌표 확인; ODB Step/미세 Substrate 미완 ([A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md))
- [ ] **T52** — 치수·물성 범위와 외삽 — **제한 Rust Canonical Layer z/두께 및 Material 유효 대역 차단 PASS / 전체 인수 미완료**: 물성 모델별 외삽·손실/거칠기·실 PCB 기하/Scenario 재계산은 미검증 ([B_001_2 감사](validation/b001-2-canonical-stackup-audit.md)) ([B_001 검증](validation/b001-stackup-reference.md))
- [ ] **T53** — DC 미지원·깊은 노치·긴 지연 — **정식 인수 미검증**
- [ ] **T54** — SSD 부족·쓰기 중 강제 종료 — **정식 인수 미검증**

## 6. 갱신 및 완료 판정 규칙

1. **문서 관리 담당은 챙(ChatGPT)**입니다. 사용자는 일상적으로 문서를 열람만 하고, 체크/날짜/근거 갱신은 챙이 GitHub PR로 수행합니다. `B_001`을 포함한 모든 후속 기능·수정 PR에서 이 문서를 함께 검토하며, 해당 Uxx·Txx·단계·검증 한계·Issue/PR 링크를 **동일 PR**에서 최신화합니다. 변경이 없다면 PR 본문에 `체크리스트 변경 없음`과 범위·이유를 적습니다.
2. 체크 전 **사양서 요구와 검증 범위가 일치**하는지 확인합니다. 증거: 원본/공개 Fixture·PR·최종 Commit SHA·CI/실행 로그·OS·독립 검토 결과. Green CI/PR Merge만으로 실 PCB Solver/제품을 PASS로 승격하지 않습니다.
3. 판정: `완료`=해당 문장 전체 수용, `부분 증빙`=일부 구현/시험, `정식 인수 미검증`=제품 게이트 부족, `BLOCKED`=실패 또는 명시적 외부 조건 미해결. 제한 완료와 제품 전체 완료를 섞지 않습니다.
4. 물리 Gate Q **historical FAIL**, M0 **BLOCKED**, CodeQL Alert #1의 **실제 Open/Fixed 상태 NOT_VERIFIED**, GPU 측정/라이선스·배포 미완 상태는 별도 근거가 확인되기 전 변경하지 않습니다.
5. 체크를 새로 표시할 때는 반드시 실제 KST 날짜의 `**체크일: YYYY-MM-DD KST**`와 근거 링크·근거일을 기록합니다. 이미 체크된 항목을 재검증하더라도 최초 체크일은 유지하며, 정정은 변경 이력에 남깁니다. GitHub 문서 CI가 체크일 누락 및 날짜 형식 오류를 검사합니다.
6. 위 운영 방식은 GitHub 계정의 기술적 접근권한 변경이 아닙니다. 저장소 소유자 권한은 그대로 유지되며, 현재 동일 계정 인증으로 챙의 수정과 소유자의 직접 수정을 GitHub가 기술적으로 구별하지는 못합니다.
7. 각 단계 종료 후 아래 이력에 작업 번호, 변경된 U/T, 최종 `main` SHA, Native 테스트·Codex 리뷰 증빙 및 아직 미검증인 항목을 기록합니다.

### 변경 이력

| 날짜(KST) | 작업 | 기준/증거 | 변경 사항 |
| --- | --- | --- | --- |
| 2026-10-26 | A_036 | 사양서 v1.0, [PR #210](https://github.com/july-rust/BoardSI/pull/210), `main@d23f2d4001130cf654e3071cfeb01a7df473ac1d` | U01–U48 / T01–T54 목록 및 최초 체크 13개 생성. 병합 후 21/21 CI PASS |
| 2026-10-26 | A_036 후속 날짜 보강 | [Issue #211](https://github.com/july-rust/BoardSI/issues/211) | 기존 체크 13개에 최초 체크일 및 개별 선행 근거일 기록, 향후 체크 날짜 필수화, 챙 관리·사용자 열람 운영 원칙 추가 |
| 2026-10-26 | B_001 착수 (PR 검증 전) | [Issue #12](https://github.com/july-rust/BoardSI/issues/12), [검증 원장](validation/b001-stackup-reference.md) | Qt 사용자 Stackup Overlay 후보 코드. U09/U10/T02/T52 부분 증빙 후보; 정식 체크박스 변경 없음. Native CI/독립 리뷰/실제 물리 준비도 미확인 |
| 2026-10-27 | B_001_3 보호된 main 병합·검증 기록 | [병합 PR #217](https://github.com/july-rust/BoardSI/pull/217), 유효 서명 **`main@7448edfe68f5bf50af4d1f3e580f84fca98af3d2`** ([commit](https://github.com/july-rust/BoardSI/commit/7448edfe68f5bf50af4d1f3e580f84fca98af3d2)), [최종 PR HEAD `ac02dfe57f885d2d33b496bdc93ef061f99935f6` 27/27 PASS](https://github.com/july-rust/BoardSI/commit/ac02dfe57f885d2d33b496bdc93ef061f99935f6/checks), Codex 독립 검토 0 findings·리뷰 스레드 4건 해결 | 제한 원본 Copper/Net Reference **음성 판정만** 최초 체크일 **2026-10-27 KST 유지** (시험 근거 2026-10-27 KST); 전기적 연결, Return Path P009, 완전한 B_001 및 정식 U/T 인수 미완. 병합 후 main CI는 별도 판정 필요 |
| 2026-10-27 | B_001_3 제한 Import Reference 후보 안전감사 체크 추가 | [PR #217](https://github.com/july-rust/BoardSI/pull/217), [code SHA `d15b86db4cc044da07930fbc9baad41ea0bc891a` 27/27 CI](https://github.com/july-rust/BoardSI/commit/d15b86db4cc044da07930fbc9baad41ea0bc891a/checks), [Qt Native+Linux](https://github.com/july-rust/BoardSI/actions/runs/40428481598), [검증 원장](validation/b001-3-source-reference-inventory.md) | B_001_3 좁은 음성/Unsupported 검증만 **최초 체크일 2026-10-27 KST**, 시험일 2026-10-27 KST로 체크. 물리 접속, P009, 전원 Net 및 U09/U10/U39/T02/T04/T24/T52 전체 공식 인수는 미완; 문서 새 HEAD 보호 검사 후 병합 필요 |
| 2026-10-27 | B_001_3 PADS Copper/Net Source Inventory 착수 | [PR #217](https://github.com/july-rust/BoardSI/pull/217), [검증 원장](validation/b001-3-source-reference-inventory.md) | 실제 성공 Import의 Layer·Net·Area ID·Hole 목록을 Rust 후보 감사에 전달. PASS 전까지 신규 완료 체크 없음. Reference 전기적 접속·Return Path P009·정식 T04/T24는 미검증 |
| 2026-10-27 | B_001_2 보호된 main 병합 증빙 확정 | [병합 PR #215](https://github.com/july-rust/BoardSI/pull/215), 서명된 `main@5f795b7f4609304827d4d6ca6acb4ff7ecf73cc8`, [정확한 PR HEAD 28/28 CI](https://github.com/july-rust/BoardSI/commit/6ce79d685a26cd3b26f1cd9b9ca065850aea5ca5/checks), [Qt Native 3+Linux](https://github.com/july-rust/BoardSI/actions/runs/40420950767), 독립 Codex same-HEAD 0 findings | 제한 Stackup Material `Assumed` 변환의 최초 체크일 **2026-10-27 KST 유지** (근거일 2026-10-27 KST); 별도 물리 Reference/Net/Copper 미자격 및 Issue #12 OPEN. 최종 main merge SHA 보존; 정식 U/T 체크박스 미변경 |
| 2026-10-27 | B_001_2 제한 Rust Canonical 감사 체크 추가 | [PR #215](https://github.com/july-rust/BoardSI/pull/215), [Native Qt 검사](https://github.com/july-rust/BoardSI/actions/runs/40419571718), [Rust fmt/Clippy/모델 검사](https://github.com/july-rust/BoardSI/actions/runs/40419571744), [원장](validation/b001-2-canonical-stackup-audit.md) | **검증된 좁은 B_001_2 항목만 최초 체크일 2026-10-27 KST / 근거일 2026-10-27 KST로 [x] 추가**, 물리 Reference·실 Port·Solver/정식 인수 미완. 최종 HEAD 필수 CI 및 Codex 재검증 필요 |
| 2026-10-27 | B_001_2 추가 착수 / 검증 전 | [PR #215](https://github.com/july-rust/BoardSI/pull/215), [Rust Canonical 검증 원장](validation/b001-2-canonical-stackup-audit.md) | UI 출처·물성 대역 필드를 Rust Canonical Layer/Material로 연결하는 후보 코드. 이번 PR의 필수 CI/독립 리뷰 증빙 전이라 [x] 추가 없음. T04/T24/실제 PADS Copper Net/Return Path 계속 미완 |
| 2026-10-27 | B_001 제한 UI 증빙 확정 / #12 OPEN | [병합 PR #213](https://github.com/july-rust/BoardSI/pull/213), **`main@ed9fc66e5b0ab0072fcb93103adc18c2729ae0e3`** ([서명된 squash merge](https://github.com/july-rust/BoardSI/commit/ed9fc66e5b0ab0072fcb93103adc18c2729ae0e3)), 사전 병합 `head@a789ff22ad025c83ec6357a81ea2bbe9b3069b32` (26/26 검사 PASS, 17/17 필수, Codex same-head findings 0), [Qt Native 3+Linux](validation/b001-stackup-reference.md) | 제한 Qt Overlay [x] 1개만 추가 (최초 체크일 2026-10-27 KST / 시험 근거일 2026-10-26 KST); U09/U10/U39 및 T02/T04/T24/T52 전체 인수 체크는 미완. 병합된 main 신규 CI는 별도 게이트로 확인 |

### 필수 참고 근거

- [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) · [A_034 Native 통합](validation/a034-p2y-integration-stabilization.md) · [A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md) · [A_032 국소 접점](validation/a032-p2w-local-connectivity.md)
- [M0 종료 게이트](validation/m0-completion-gate.md) · [M0 추적 원장](audits/m0-requirements-traceability-ledger.md) · [M2 검증 원장](validation/README.md)
- [원본 PH-08 FAIL](validation/m2-g2-ph08-stage5b-original-replay-evidence.md) · [SEC_001 코드 검사](validation/sec001-codeql-markdown-comment-scan.md) · [Code scanning alert #1](https://github.com/july-rust/BoardSI/security/code-scanning/1)
