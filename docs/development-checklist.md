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

- [x] **B_002 후속 — 제한적 Rust 감사 Scenario Revision 계약** — 원본 SHA-256, Stackup/Material 두께·Dk/Df·출처, Reference Net, Copper Inventory 및 주파수 변경을 구분하는 결정적 감사 Revision을 Rust Worker가 생성하며, Qt는 최신 감사 입력에만 이를 연결. 무관한 Zoom/Theme 변경은 Revision에 영향 없음. **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** ([PR #221](https://github.com/july-rust/BoardSI/pull/221), 최종 HEAD `dd5a2ac7efb0007e4624fb19a275a281b5381128` **27/27 CI PASS**, 동일 HEAD Codex 주요 지적 0·미해결 스레드 0, [GitHub 서명된 병합 `222552c9f92808e67ba06c80cb6de50756fcf3c2`](https://github.com/july-rust/BoardSI/commit/222552c9f92808e67ba06c80cb6de50756fcf3c2), [B_002 원장](validation/b002-b001-completion.md)). **감사 입력 전용**이며 `cache_reuse_eligible=false`; 실제 물리 Scenario/Cache, Reference/Return Path, Copper/Pad/Port 연결 및 정식 T02/T04/T24/T52는 미검증. **B_001 전체 Issue #12는 미완료/OPEN 유지.**

### B_003 제한 완료 — Trace/Pad/Via 원본 접촉 근거 (Issue #12 연속 작업)

- [x] **B_003 지원 PADS V9.4 MILS Trace/Pad/Via 정확 좌표 접촉 그래프** — 제한 Straight Trace 내부 T-Junction, PADS ROUTE/SIGNAL Pin-pair에서 명시된 SMD Pad/Pin Net, Pad centre와 Straight Trace centreline 동일 층·동일 선언 Net 접점, 명시된 도금 Through Via의 해당 Conductive Layer 접점과 검증된 Layer 간 Barrel Edge를 Canonical Rust 증거로 구성. 동일 Net 레이블만으로 일반 접촉을 가정하지 않고, 충돌/미확인 Net·미지원 형상은 Fail-closed. 기존 Qt V1 Source Endpoint Audit/Record 호환과 성능 회귀 유지. **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** · [PR #224](https://github.com/july-rust/BoardSI/pull/224), 최종 HEAD `e211b2e626afd89ac7a880eaeace40e91c8abc50` **28/28 GitHub Checks SUCCESS**, [Codex 무결점 코드 Tree 동일성 비교](https://github.com/july-rust/BoardSI/compare/e40d45d157277d81ac8f50afef4a6b038c63f7bc...e211b2e626afd89ac7a880eaeace40e91c8abc50) (검토된 `e40d45d1`과 최종 HEAD의 Git Tree SHA 둘 다 `aa5ac263f50044c79d97f27585e883010cab5633`; 리뷰 6개 스레드 해결), [Verified 서명된 Squash Merge `1c11d08be6e7c83d3f6b697f515f61a8c1a43ba8`](https://github.com/july-rust/BoardSI/commit/1c11d08be6e7c83d3f6b697f515f61a8c1a43ba8), [세부 검증 원장](validation/b003-b001-reference-topology.md). **일반 Pad 몸체·Trace 폭 겹침, 전체 Plane/Cutout/Reference Return Path, 물리 Solver/Cache, 정식 T04/T24 및 B_001 전체는 여전히 미완료.** 병합 후 `main` CI는 별도 확인 대상.

- [ ] **B_001 전체 Issue #12 정식 인수** — B_003 제한 접속 기능 병합이 곧 전체 Copper/Plane 연결과 고주파 Return Path 물리 검증을 의미하지 않음. U09/U10/U39, T02/T04/T24/T52 전체 승인 보류. Issue #12 OPEN 유지.

### B_004 제한 완료 — 단일 단순 Copper Area 국소 접촉 검증

- [x] **B_004 제한 PADS Source Copper Area 내부 접촉** — 공개 합성 PADS V9.4 MILS의 소스 소유 단일 Hole-free Straight COPCLS Polygon과 명시적 ROUTE Net이 일치할 때, 정수 Source Tick 내부의 Trace Endpoint/선언된 SMD Pad Centre 접촉을 Canonical `AreaInterior` 증거로 검증하고 같은 Area 안의 두 국소 접점만 `AreaContinuity`로 연결. 직선상 정방향 Collinear 꼭짓점 허용·경계/역방향 중첩 거부, 타 Layer/Cross-Net/분리 Island 거짓 연결 방지, 변조된 증거와 구형 Qt V1 호환을 교차 시험. **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** · [B_004 PR #226](https://github.com/july-rust/BoardSI/pull/226), [코드 HEAD `7c68f9523b65999271fff5b510e984ec30dac72a`의 12 워크플로·23 Job 전부 SUCCESS](https://github.com/july-rust/BoardSI/commit/7c68f9523b65999271fff5b510e984ec30dac72a/checks), 동일 HEAD Codex 추가 지적 0·과거 스레드 4건 해결, [검증 원장](validation/b004-reference-copper-connectivity.md). **이 체크는 제한된 Rust Source-Geometry 기능에만 적용**되며 Arc/Hole/Cutout·Via-to-Plane·Pad 몸체/Trace 폭·Plane Return Path/P009/물리 SI/Cache 및 정식 U09/U10/U39·T02/T04/T24/T52는 여전히 미검증. 보호된 Signed Squash Merge 및 병합 후 main CI는 별도 확인하여 기록한다.

### B_005 제한 완료 — Source-bound Reference 기하 커버리지

- [x] **B_005 제한 Source-bound Reference Plane Trace Centreline 기하 커버리지 감사** — 공개 합성 PADS V9.4 MILS의 지원 Straight/COPCLS Hole-free 단일 Polygon에 한정. 원본 파일 Source SHA·Net ID/이름·Layer 순서·Trace/Arc 원본 완전성·Copper Source ID/Net 및 원본 경계를 재파싱해 검증, 단일 Layer의 명시적 비분기 Trace 경로 전체가 하나의 Reference Polygon 엄격 내부인지 정수 기하로 판정. Backtrack/T/Arc/Via 분기, 분리 Island·Cutout·원본 변조는 거부하며 결과는 `SOURCE_ORDER_GEOMETRY_ONLY`일 뿐 실제 P009/Return Path 아님. [PR #227](https://github.com/july-rust/BoardSI/pull/227), [코드 HEAD `225489bedabc146e31b83137267aaa29d86f8503` 23/23 CI 성공](https://github.com/july-rust/BoardSI/commit/225489bedabc146e31b83137267aaa29d86f8503/checks), [Rust Debug/Release B_005 17/17 PASS 및 원장](validation/b005-reference-geometry.md) · **근거일: 2026-10-27 KST**. **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** (최초 체크일) 독립 Codex 최종 PR HEAD `284f5f04fe` 리뷰 주요 이슈 0; 역사적 지적 11건 해결·리뷰 스레드 0건. 보호 Squash Merge [PR #227](https://github.com/july-rust/BoardSI/pull/227), [Signed Verified `main@7a1ed48bf905df8357691a1f51b73e130a74537c`](https://github.com/july-rust/BoardSI/commit/7a1ed48bf905df8357691a1f51b73e130a74537c). 보호된 `main` 병합 후 Push CI **21/21 SUCCESS** 및 Push on main 워크플로 SUCCESS 확인 (별도 완료 체크 참조). `return_path_p009=UNSUPPORTED`, `solver_ready=false`, `physical_si=NOT_RUN`, `cache_reuse_eligible=false`. B_001 Issue #12 OPEN, 공식 U09/U10/U39 및 T02/T04/T24/T52 미완.

- [x] **B_005 post-main Push CI 최종 확인** — [Signed Squash Merge `7a1ed48`](https://github.com/july-rust/BoardSI/commit/7a1ed48bf905df8357691a1f51b73e130a74537c/checks)의 병합 후 GitHub Check **21/21 SUCCESS** 및 별도 **Push on main SUCCESS** 확인. **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST**. [메인 Push 검사 근거](https://github.com/july-rust/BoardSI/commit/7a1ed48bf905df8357691a1f51b73e130a74537c/checks) · [Push on main 워크플로 #40501517050](https://github.com/july-rust/BoardSI/actions/runs/40501517050). 코드·제한 B_005 구현 `[x]`와 구분한다.

### B_006 제한 완료 — Source-bound Through Via/COPCLS 국소 접촉 감사

- [x] **B_006 공개 합성 PADS V9.4 MILS Plated Through Via–단일 Straight Hole-free COPCLS Polygon의 제한 국소 접촉** — 원본 ROUTE/Net/Layer/Via/Drill/Padstack 및 Copper Source SHA·기하를 재파싱해 동일 원본과 비교하며, 명시적 도금 Through Via의 Conductive Padstack Layer상 Via Centre가 같은 선언 Net의 단일 Source Polygon 엄격 내부인 경우에만 `ViaCentre`–`AreaInterior` SameLayer 증거를 Canonical Graph에 추가. 기존 독립 접점과의 ViaBarrel·AreaContinuity를 검증하며 Source 변조/거짓 Layer/동일 위치 Edge 중복/잘못된 Net·Cutout·Boundary를 차단, Qt V1 회귀 보존. **체크일: 2026-10-27 KST** (최초) · **근거일: 2026-10-27 KST**. [구현 PR #229](https://github.com/july-rust/BoardSI/pull/229), [최종 코드 HEAD `68cd29313d4b3ccfd340321196e286af0ef4ff80`의 CI](https://github.com/july-rust/BoardSI/commit/68cd29313d4b3ccfd340321196e286af0ef4ff80/checks) **9개 워크플로 SUCCESS**, [Rust Numerical Debug/Release B_006 각각 11/11 PASS](https://github.com/july-rust/BoardSI/actions/runs/40510123992), [Qt Native Windows x64/ARM64·macOS ARM64·Linux SUCCESS](https://github.com/july-rust/BoardSI/actions/runs/40510123737), Codex 동일 HEAD findings 0·두 과거 P2 스레드 해결, GitHub 보호 Squash [`main@d7a22f8230086cdf8aeb326882708dfee48d9ee1`](https://github.com/july-rust/BoardSI/commit/d7a22f8230086cdf8aeb326882708dfee48d9ee1), [B_006 검증 원장](validation/b006-via-copper-contact.md). **이 체크는 좁은 원본 기하 국소 접촉 기능만** 의미함. 실제 Copper fill/전기적 Return Path/P009/물리 Solver·Cache 및 정식 U09/U10/U39/T02/T04/T24/T52 전부 미인수. Issue #12 OPEN.
- [x] **B_006 protected main post-merge Push CI 및 최종 서명 Verified 확인** — B_006 코드 [보호 Squash Merge `d7a22f8230086cdf8aeb326882708dfee48d9ee1`](https://github.com/july-rust/BoardSI/commit/d7a22f8230086cdf8aeb326882708dfee48d9ee1/checks)의 REST `commit.verification.verified=true`, `reason=valid`, `verified_at=2026-10-27T13:23:25Z`를 별도 확인. 해당 SHA의 [GitHub Check Runs](https://github.com/july-rust/BoardSI/commit/d7a22f8230086cdf8aeb326882708dfee48d9ee1/checks) **21/21 SUCCESS**, 병합 후 **9개 workflow runs 전부 SUCCESS** 및 별도 [Push on main #40511614420](https://github.com/july-rust/BoardSI/actions/runs/40511614420) SUCCESS (코드 PR HEAD 검증과 독립 확인). **체크일: 2026-10-28 KST** · **근거일: 2026-10-27 KST**. B_006의 제한 국소 접촉 기능에만 적용하며 B_001 Issue #12, 물리 Return Path/P009, Solver/Cache, 공식 U09/U10/U39/T02/T04/T24/T52는 미완.

### B_007 — COPCUT-aware bounded local contact implementation and integration

- [x] **B_007 original PADS single source-owned polygon COPCUT-aware Trace/Pad/Plated Through Via centre local evidence — limited implementation/test/merge only** — [PR #231](https://github.com/july-rust/BoardSI/pull/231), exact source HEAD [`6a75eb5c5616d86407a04a946bc994958fb55ea6`](https://github.com/july-rust/BoardSI/commit/6a75eb5c5616d86407a04a946bc994958fb55ea6/checks) **28/28 PR checks SUCCESS**, B_007 source/signed area Rust Numerical Debug **11/11 PASS** and Release **11/11 PASS**, Native Rust Windows x64/ARM64, macOS ARM64 and Qt Windows x64/ARM64/macOS ARM64/Linux CI SUCCESS; original PADS Source/Net/Pad/Padstack/Drill/Via/Area geometry and strict single Hole contour validated. Protect Signed Verified Squash merged to [`main@8aae7504417f6aa8d212a0906de658ed173c4ef9`](https://github.com/july-rust/BoardSI/commit/8aae7504417f6aa8d212a0906de658ed173c4ef9), GitHub REST `commit.verification.verified=true, reason=valid`. **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**. The latest-head automatic independent Codex review is separately pending; this check certifies bounded implementation/CI/merge, **not** final independent-review closure, physical electrical SI, general hole/cutout connectivity or full B_001 acceptance. [Validation ledger](validation/b007-copper-cutout-connectivity.md).
- [x] **B_007 protected main post-merge Checks and Push on main** — independently inspected exact merged [`main@8aae7504` Check runs](https://github.com/july-rust/BoardSI/commit/8aae7504417f6aa8d212a0906de658ed173c4ef9/checks): **21/21 SUCCESS**, all **9/9 post-main workflow runs SUCCESS**, including [Push on main #40602499013](https://github.com/july-rust/BoardSI/actions/runs/40602499013) and [Qt Runtime #40602499442](https://github.com/july-rust/BoardSI/actions/runs/40602499442). **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**. Tested synthetic source geometry subset only.
- [ ] **B_007 exact latest HEAD independent Codex final-review closure** — prior three P2 findings (source Pad spoof, two negative/positive fixture expectations) were fixed and all three PR threads resolved. Automatic latest-head review receipt for `6a75eb5c` still reports `status=running`; **do not claim completed/zero findings** until rechecked as `status=completed` with `has_any_findings=false`, `has_unresolved_findings=false` and zero unresolved PR threads. Parent [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN.


- [ ] **B_001 전체 Issue #12 정식 인수** — B_006이 제한적으로 성공하더라도 물리 Reference Return Path/P009, Scenario/Cache, Material loss/roughness 및 공식 U09/U10/U39/T02/T04/T24/T52는 남는다. 원래 OPEN 상태 유지.

### B_008 — 단일 COPCUT Reference 경로 기하 감사 (2026-10-28 KST)

- [x] **B_008 제한 단일 COPCUT Reference 전체 Centreline 기하 검사 구현 및 보호된 병합** — B_005 원본 검증과 B_007 단일 Source-owned Hole을 결합해 전 구간의 Cutout Boundary/내부 교차를 차단; 물리 Return Path 자격 아님. [PR #233](https://github.com/july-rust/BoardSI/pull/233), exact final Source HEAD `3dc3a681a2259369d3b1396a95f3d988fbad438c` **23/23 Checks, 9/9 workflows SUCCESS**, Numerical [#40610946199](https://github.com/july-rust/BoardSI/actions/runs/40610946199) B_008 10/10 PASS, Qt Native [#40610946377](https://github.com/july-rust/BoardSI/actions/runs/40610946377) SUCCESS, same-HEAD independent Codex Review completed/no findings or unresolved threads, [Signed Verified Squash main `9cba34f1`](https://github.com/july-rust/BoardSI/commit/9cba34f1eb1135da8e187c8ecb945ad90c78d073). **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**. 지원된 Source Geometry의 제한 기능만 [x]; P009/Return Path, 물리 Solver/Cache, 정식 U09/U10/U39/T02/T04/T24/T52 및 B_001 Issue #12 미완.
- [x] **B_008 보호된 main 병합 후 독립 Push/Native CI 최종 확인** — 정확한 [Signed Verified Squash `main@9cba34f1eb1135da8e187c8ecb945ad90c78d073`](https://github.com/july-rust/BoardSI/commit/9cba34f1eb1135da8e187c8ecb945ad90c78d073/checks)에서 **21/21 GitHub Check Runs SUCCESS**, 9/9 Workflow Runs SUCCESS, 별도 [Push on main #40612009982](https://github.com/july-rust/BoardSI/actions/runs/40612009982) 및 [Qt Runtime #40612010054](https://github.com/july-rust/BoardSI/actions/runs/40612010054) SUCCESS를 GitHub REST로 확인. **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**. 제한 Source Geometry 기능만 완료. [B_008 검증 원장](validation/b008-reference-cutout-geometry.md).


### B_009 — B-series 제한 범위 최종 감사 및 C 단계 인계 (2026-10-28 KST)

- [x] **B_009 실행 전 보호 main 및 체크리스트 감사** — 보호 `main@972c6663a5d3cd0fd9d7f9cfe2d22c24dbc21ef3` GitHub 서명 `verified=true, reason=valid`, [동일 SHA 검사](https://github.com/july-rust/BoardSI/commit/972c6663a5d3cd0fd9d7f9cfe2d22c24dbc21ef3/checks) **21/21 Check Runs SUCCESS**, 9/9 Workflow Runs SUCCESS를 실제 GitHub REST에서 조회하고 이 문서 및 [B_008 원장](validation/b008-reference-cutout-geometry.md), [B_007 원장](validation/b007-copper-cutout-connectivity.md), [Issue #12](https://github.com/july-rust/BoardSI/issues/12)를 대조했다. **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**. 이는 기존 main 증거이며 **B_009 신규 PR 시험 결과는 아님**.
- [x] **B_009 B_001–B_008 제한 구현 통합 감사·C 단계 인계 및 보호된 main 최종 검증** — [B_009 검증 원장](validation/b009-b-series-final-acceptance.md)과 C_001–C_004 소규모 개발 인계. 최종 [PR #235](https://github.com/july-rust/BoardSI/pull/235) 정확한 소스 HEAD `a38facad54b457947009e3a364ce5bfdb3a6a37d` **22/22 PR Checks SUCCESS** 및 동일 HEAD Codex 자동 리뷰 `completed`, `has_any_findings=false`, `has_unresolved_findings=false`, 리뷰 미해결 Thread 0. [보호된 Squash Merge `main@47f4a6de7125eb49cb504c3b5d91aa59d908031f`](https://github.com/july-rust/BoardSI/commit/47f4a6de7125eb49cb504c3b5d91aa59d908031f) 서명 `verified=true, reason=valid`; [main 동일 SHA 병합 후 21/21 Checks SUCCESS](https://github.com/july-rust/BoardSI/commit/47f4a6de7125eb49cb504c3b5d91aa59d908031f/checks), 9/9 Workflows SUCCESS ([Push on main #40623750981](https://github.com/july-rust/BoardSI/actions/runs/40623750981), [Qt Native #40623751051](https://github.com/july-rust/BoardSI/actions/runs/40623751051)). **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**. **제한 문서/원본 기하 개발 감사만 [x]**; B_007 최종 Source HEAD 리뷰 및 B_001 Issue #12 전체·물리 P009/정식 U/T 인수는 여전히 [ ]/OPEN.
- [ ] **B_007 최종 Source HEAD 독립 Codex 리뷰 종결** — 원본 PR [#231](https://github.com/july-rust/BoardSI/pull/231)의 정확한 `6a75eb5c5616d86407a04a946bc994958fb55ea6`에 대한 [Codex 최신 상태](https://github.com/july-rust/BoardSI/pull/231#issuecomment-6455692908)는 `status=running`. 3개 이전 스레드 resolved 및 28/28 Checks PASS는 확인되나 **최종 HEAD 리뷰 완료가 아님**. `completed`, 발견 사항/미해결 0을 확인하기 전 체크 금지.
- [ ] **B_001 Issue #12 전체 정식 인수** — 제한 Source Geometry와 실제 전기적 Reference/Return Path, Material Loss, 물리 Solver Scenario/Cache, U09/U10/U39 및 T02/T04/T24/T52 전체 시험을 혼동하지 않는다. [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN 유지. 미완 사항은 C단계 후보로 명확히 이관하지만 정식 M1 종료로 간주하지 않는다.

### C_001 — 다중 Source-owned COPCUT / Disconnected Island (제한 기능 완료 · 2026-10-28 KST)

- [x] **C_001 제한 다중 Source-owned Straight Polygon COPCUT 및 분리 Copper Island 국소 접점 — 정식 한정 구현 인수** — [C_001 검증 원장](validation/c001-multi-cutout-island-connectivity.md), [코드 PR #237](https://github.com/july-rust/BoardSI/pull/237), 최종 PR Source HEAD `1db862985505427f8f3fcd58b6ae6c9530cbac6a`의 **28/28 Checks SUCCESS**, Rust C_001 신규 Debug·Release **각 10/10 PASS**, 독립 Codex 동일 HEAD `completed`/findings 0/unresolved 0, 과거 P2 스레드 resolved; [Verified 보호된 Squash `main@df1fde8648fd48f9ee884a3d9ad68fdf266e5f35`](https://github.com/july-rust/BoardSI/commit/df1fde8648fd48f9ee884a3d9ad68fdf266e5f35/checks) `verified=true, reason=valid`, **독립 병합 후 21/21 Checks·9/9 Workflows SUCCESS** ([Push main #40629876739](https://github.com/july-rust/BoardSI/actions/runs/40629876739), [Qt Native #40629877037](https://github.com/july-rust/BoardSI/actions/runs/40629877037)). **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**. (최초 체크일) 지원된 Source Geometry/국소 접촉만 `[x]`; 분리 Island는 Net 레이블만으로 연결하지 않으며 모호한 중첩 Area는 Unsupported. **B_007 원본 최종 HEAD Codex 리뷰 및 B_001 Issue #12 전체/P009/Solver/Cache/정식 U09/U10/U39/T02/T04/T24/T52는 여전히 `[ ]`/OPEN.**

### C_002 — Reference Split 및 Layer Transition Topology Risk (Rust 제한 기능 검증 완료 · 2026-10-28 KST)

- [x] **C_002 Source-backed Reference Gap/Cutout 및 Plated Through Via Transition Risk의 제한 Rust Audit·공개 합성 시험·보호 병합 및 독립 post-main 검증** — [검증 원장](validation/c002-reference-split-transition-topology.md), [코드 PR #240](https://github.com/july-rust/BoardSI/pull/240), 최종 Source HEAD [`c6591c14842b55fb48ba070c9f719998dd37d195`](https://github.com/july-rust/BoardSI/commit/c6591c14842b55fb48ba070c9f719998dd37d195/checks) **23/23 PR Checks SUCCESS**, C_002 신규 Rust Debug **10/10**·Release **10/10 PASS**, 기존 B_003–B_008/C_001 회귀, Rustfmt·Clippy 및 Windows x64/Windows ARM64/macOS ARM64/Linux Qt Native PASS. 독립 Codex **동일 최종 HEAD `completed`, findings 0, unresolved 0**, Review Thread 미해결 0. 보호된 [Verified Squash `main@304015008112b082f96829f763ad4e77e109094c`](https://github.com/july-rust/BoardSI/commit/304015008112b082f96829f763ad4e77e109094c) `verified=true, reason=valid`; 독립 [post-main 21/21 Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/304015008112b082f96829f763ad4e77e109094c/checks), **9/9 Workflows SUCCESS** ([Push main #40640594957](https://github.com/july-rust/BoardSI/actions/runs/40640594957), [Qt runtime #40640594732](https://github.com/july-rust/BoardSI/actions/runs/40640594732)). **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** (최초). 지원된 Source 정확 기하/Topology Risk 판정만 `[x]`; 전기적 Return Path·P009 물리 자격 또는 공식 U/T 전체 인수가 아님.
- [ ] **C_002 Qt Viewer의 실제 Reference Split/Transition 마커·상태 표시 통합 및 Native UI 수용** — 이번 코드 PR은 Opt-in Rust 감사 API만 추가했고 Qt Viewer 연계 UI는 미구현. 별도 구현·시험 증빙 전까지 미완 유지.
- [ ] **B_001 Issue #12 및 정식 U09/U10/U39·T02/T04/T24/T52 전체 인수** — Reference 전기적 연속성, 실제 P009 Return Path, Solver/Cache/Material Loss는 모두 미승인. [Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN 유지. B_007 원본 마지막 HEAD Codex 리뷰도 별도 NOT_VERIFIED 상태 보존.

### C_003 — Immutable Scenario and Stale Result (Rust 제한 기능 검증 완료 · 2026-10-28 KST)

- [x] **C_003 제한 Rust Immutable Scenario/ResultReceipt 계약의 정확한 Source HEAD 시험·독립 리뷰·보호 병합 및 독립 post-main CI** — [Issue #242](https://github.com/july-rust/BoardSI/issues/242), [검증 원장](validation/c003-immutable-scenario-stale-result.md), [코드 PR #243](https://github.com/july-rust/BoardSI/pull/243), 최종 Source HEAD [`f1d9c2d66e94d7228326ee009c3c0ae902a1ba3f`](https://github.com/july-rust/BoardSI/commit/f1d9c2d66e94d7228326ee009c3c0ae902a1ba3f/checks) **9/9 GitHub Actions Workflows SUCCESS**, 신규 C_003 Synthetic 회귀 Rust **Debug 12/12·Release 12/12 PASS**, Rustfmt·Clippy 및 기존 B/C 회귀 PASS, Numerical Native Windows x64/ARM64·macOS ARM64와 Qt Native Windows x64/ARM64·macOS ARM64/Linux PASS. 독립 Codex **동일 최종 HEAD completed, findings 0, unresolved 0**, Review Thread 미해결 0. 보호된 **Signed Verified Squash** [`main@7edff0b9766a1ae70ab26e7010e356d64bad7046`](https://github.com/july-rust/BoardSI/commit/7edff0b9766a1ae70ab26e7010e356d64bad7046) **`verified=true, reason=valid`**, [동일 main SHA 독립 21/21 Check Runs SUCCESS](https://github.com/july-rust/BoardSI/commit/7edff0b9766a1ae70ab26e7010e356d64bad7046/checks) 및 **9/9 Workflow Runs SUCCESS** ([Push on main #40648767875](https://github.com/july-rust/BoardSI/actions/runs/40648767875), [Qt runtime #40648767924](https://github.com/july-rust/BoardSI/actions/runs/40648767924), [Numerical core #40648767849](https://github.com/july-rust/BoardSI/actions/runs/40648767849)). **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** (최초 기록). 이는 Caller-provided SHA 주장과 Synthetic Receipt의 **제한 입력 최신성 계약**만 완료한 것이며, 실 PCB 전체 Canonical 직렬화·실행 가능한 물리 Solver/Cache 재사용·P009, 정식 U09/U10/U39/T02/T04/T24/T52 인수와 동일하지 않다.
- [ ] **C_003 이후 실 PCB 완전 Solver Snapshot/Worker transport 및 실제 재사용 캐시** — 전 물리 입력 Canonical Serialize/Source Rehash·Port/Load·Material·엔진 호환성 검증과 물리 정확도/Cache 별도 Gate 필요. [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN, C_002 Qt Reference 위험 마커 미구현 및 B_007 원본 HEAD Codex 최종 미확인 유지.

### C_004 — Material Provenance / Loss Input Gate (제한 코드 및 post-main 검증 완료 · 2026-10-28 KST)

- [x] **C_004 Canonical Material/Layer의 제한 Rust 입력 계약 구현·최종 코드 PR 검증·보호 병합** — [Issue #245](https://github.com/july-rust/BoardSI/issues/245), [PR #246](https://github.com/july-rust/BoardSI/pull/246), [검증 원장](validation/c004-material-loss-conductivity-roughness.md). Dk/Df·σ·거칠기 출처/유효대역·복소 유전율 부호와 오버플로·Material Revision을 제한 Native Rust로 검증. 정확한 코드 HEAD [`3c17b723b5d2508e71808d186c78869ece56ee06`](https://github.com/july-rust/BoardSI/commit/3c17b723b5d2508e71808d186c78869ece56ee06/checks) **23/23 CI SUCCESS·9/9 Workflows SUCCESS**, 3 Native Rust/Qt + Linux Qt, 동일 HEAD 독립 Codex completed / 주요 지적 0 / 미해결 스레드 0. [Verified Squash `main@02cc35b51dfe4263f3e3d3f02f9151d47d696849`](https://github.com/july-rust/BoardSI/commit/02cc35b51dfe4263f3e3d3f02f9151d47d696849), GitHub `verified=true`, `reason=valid`. **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**. 이 체크는 제한 입력 계약/코드 병합만 의미하며 실제 물리 Loss/PH-12~16, C_003 물리 Solver/Cache, P009 및 정식 U09/U10/U39/T02/T04/T24/T52 인수는 미완, Issue #12 OPEN.
- [x] **C_004 코드 보호 병합 후 독립 `main@02cc35b` 검증** — [동일 코드 병합 SHA](https://github.com/july-rust/BoardSI/commit/02cc35b51dfe4263f3e3d3f02f9151d47d696849/checks) **21/21 Check Runs SUCCESS·9/9 Workflow Runs SUCCESS** ([Push on main #40673838011](https://github.com/july-rust/BoardSI/actions/runs/40673838011), [Qt runtime #40673838211](https://github.com/july-rust/BoardSI/actions/runs/40673838211), [Numerical core #40673838276](https://github.com/july-rust/BoardSI/actions/runs/40673838276), [FEM Native #40673838163](https://github.com/july-rust/BoardSI/actions/runs/40673838163)). FEM macOS ARM64 최초 취소 후 재실행 attempt 2 성공 포함. **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**. 문서 자체의 보호 병합/후속 CI는 별도 PR에서 검증. 이 체크는 PH-12~16이나 실제 PCB 물리 Loss/전체 M1 인수와 무관.

### C_005 — Qt Source Reference Risk 표시/무효화 제한 인수 (2026-10-28 KST)

- [x] **C_005 Source-bound Rust Reference 기하 위험 감사 → Qt CPU/RHI Warning Marker 및 Stale 입력 무효화** — 공개 합성 PADS V9.4 MILS의 C_002 지원 Straight Trace와 명시적 도금 Through Via 전이·Reference COPCLS/COPCUT 일부에 한정. Source SHA-256·원본 ObjectId·Layer/Net·정수 Endpoint 일치 및 명시 Reference 선택을 Rust/Qt 간 재검증하고 `REFERENCE_SPLIT_RISK`/`LAYER_TRANSITION_RISK` 주의 마커, 대표 Midpoint 표시(정확한 교차 위치 아님), CPU/RHI Filter/Picking/Properties, 원본 변경·ReImport·선택/Stackup 수정·지연 응답 Stale 차단을 구현. `SOURCE_GEOMETRY_ONLY`는 전기적 PASS 아님, `UNSUPPORTED`는 긍정 판정 금지. [Issue #248](https://github.com/july-rust/BoardSI/issues/248) · [검증 원장](validation/c005-qt-reference-risk-integration.md). 코드 [PR #249](https://github.com/july-rust/BoardSI/pull/249) (정확한 HEAD `43204eab6c6b204a93a7e36ad957bad772187fd1`: 11/11 Workflow/22/22 Job 성공, 동일 HEAD Codex 0 findings·미해결 0, 과거 지적 3건 해결), [Verified Squash `main@338f5c5f9d8590ba84d56803036590158a8f87c4`](https://github.com/july-rust/BoardSI/commit/338f5c5f9d8590ba84d56803036590158a8f87c4) + 독립 Main 21/21 Check 및 Push-on-main 성공. Native 전용 C_005 회귀시험 보강 [PR #250](https://github.com/july-rust/BoardSI/pull/250) (HEAD `b7f6e550f1cd9bf8d6ca32ad8bdf43ae7e8222f2`: 25/25 GitHub Checks, 9/9 Workflow, Windows x64/ARM64·macOS ARM64의 실제 C_005 test + Linux Qt CTest 성공, Codex 0 findings/미해결 0), [Verified Squash `main@0c653bb6c01eba10bc0d1e145bc27d5228bd24f3`](https://github.com/july-rust/BoardSI/commit/0c653bb6c01eba10bc0d1e145bc27d5228bd24f3). **후속 main@0c653bb의 독립 21/21 Checks SUCCESS 및 9/9 Main Workflows SUCCESS 확인** ([post-main 검사](https://github.com/july-rust/BoardSI/commit/0c653bb6c01eba10bc0d1e145bc27d5228bd24f3/checks), [Push on main #40686719331](https://github.com/july-rust/BoardSI/actions/runs/40686719331), [Qt Native #40686719649](https://github.com/july-rust/BoardSI/actions/runs/40686719649)). **체크일: 2026-10-28 KST** (최초) · **근거일: 2026-10-28 KST**. 기존 모든 최초 체크일 불변. **실제 Reference 전류·P009, 물리 RLGC/S Solver·Cache, 정식 U09/U10/U39 및 T02/T04/T24/T52는 미완; 상위 M1 [Issue #12](https://github.com/july-rust/BoardSI/issues/12)는 OPEN 유지.**

### C_006 — Source-exact Reference Gap Interval / Qt CPU/RHI 제한 기능 완료 (2026-10-28 KST)

- [x] **C_006 공개 합성 PADS Source Reference 미피복 구간의 정확한 유리수 교차 매개변수 계산·Qt CPU/RHI 구간 경고 및 Stale 무효화** — C_005의 단일 대표 Midpoint만 표시하던 지원 Straight Signal Trace에 대해, 명시 Reference Net/Layer의 단일 Source COPCLS 및 최대 8개 Polygon COPCUT과의 교차를 원본 정수 Source Tick 기반 유리수 t 구간으로 구하여, 0~32개 미피복 구간을 V2 계약으로 전달하고 CPU/RHI 선택/Properties·Source SHA/Trace/Net/Layer·기하·구간/비동기 무효화 검증. 화면 좌표와 길이는 **표시용 근삿값**이며 경계 접촉·다중 Source Reference Island·비지원 형상은 Fail-closed. 기존 C_005 V1 Plated Via 위험판정 및 Qt Regression 보존. [Issue #252](https://github.com/july-rust/BoardSI/issues/252), [검증 원장](validation/c006-reference-gap-interval.md), [코드 PR #253](https://github.com/july-rust/BoardSI/pull/253). **최종 HEAD [`ed5e137bbb8b1bebcfde832961038d43286a7753`](https://github.com/july-rust/BoardSI/commit/ed5e137bbb8b1bebcfde832961038d43286a7753/checks) 27/27 CI SUCCESS** (Rustfmt·Clippy -D warnings, Rust Debug/Release C_006 9/9 synthetic case 및 기존 B/C 회귀, 3 Native Windows x64/ARM64·macOS ARM64 Qt C_006 실프로세스 smoke와 Linux Qt), **동일 HEAD Codex completed·지적 0·미해결 0**; 보호된 [Signed Verified Squash `main@5e4de472a90539221f69e5cf11b88e20d9fd7361`](https://github.com/july-rust/BoardSI/commit/5e4de472a90539221f69e5cf11b88e20d9fd7361) (`verification.verified=true`, `reason=valid`). **독립 post-main [21/21 Checks SUCCESS](https://github.com/july-rust/BoardSI/commit/5e4de472a90539221f69e5cf11b88e20d9fd7361/checks) 및 9/9 Workflow SUCCESS** ([Push on main #40697041065](https://github.com/july-rust/BoardSI/actions/runs/40697041065), [Qt runtime #40697040984](https://github.com/july-rust/BoardSI/actions/runs/40697040984)). **체크일: 2026-10-28 KST** (최초) · **근거일: 2026-10-28 KST**. **상위 M1 Issue #12 OPEN**, 물리 P009/실제 Return Current·전체 전기적 Copper 접속, RLGC/S/제품 Solver/Cache, 정식 U09/U10/U39·T02/T04/T24/T52는 미검증 유지.

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

### C_007 — Multi-Segment Source Reference Route / Qt Risk (제한 코드 인수 완료 · 2026-10-29 KST)

- [x] **C_007 2–16개 Source-order Straight Trace 경로의 Reference Gap Source Geometry 검사 및 Qt CPU/RHI 표시 — 한정 코드·Native 및 병합 후 인수** — [Issue #255](https://github.com/july-rust/BoardSI/issues/255), [C_007 검증 원장](validation/c007-multi-segment-reference-coverage.md), [코드 PR #256](https://github.com/july-rust/BoardSI/pull/256). 동일 Signal Net/Layer, 정확한 Source Endpoint 연결·Source SHA와 C_006 V2 Rational 각 Segment 증명, 미접촉/분기/역행 겹침·Arc/Via 미지원 차단 및 Qt stale-result 제거. **최종 코드 HEAD** [`fdfad35931805e47d6658aeed7966ed3d778b741` 27/27 CI SUCCESS](https://github.com/july-rust/BoardSI/commit/fdfad35931805e47d6658aeed7966ed3d778b741/checks) (Rustfmt/Clippy, Debug/Release Rust 및 기존 B/C, Qt Windows x64/ARM64·macOS ARM64 Native 실프로세스 C_007 테스트, Linux Qt). **독립 Codex 동일 HEAD completed / findings 0 / unresolved 0**, 기존 역방향 중첩 리뷰 수정·스레드 해결. **보호된 Signed Verified Squash** [`main@08849e3fdf048f60a8a06265792acd4f51316b2f`](https://github.com/july-rust/BoardSI/commit/08849e3fdf048f60a8a06265792acd4f51316b2f) (`verified=true, reason=valid`), **독립 post-main [21/21 Checks SUCCESS](https://github.com/july-rust/BoardSI/commit/08849e3fdf048f60a8a06265792acd4f51316b2f/checks) 및 9/9 Workflow SUCCESS** ([Push main #40781834140](https://github.com/july-rust/BoardSI/actions/runs/40781834140), [Qt Native #40781834617](https://github.com/july-rust/BoardSI/actions/runs/40781834617)). **체크일: 2026-10-29 KST** (최초) · **근거일: 2026-10-29 KST**. **제한 Source Geometry와 Qt 위험 표시만 [x]**, 물리 Reference 전류·P009·RLGC/S·Solver/Cache·정식 U09/U10/U39·T02/T04/T24/T52는 [ ] 유지, 상위 [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN.


### C_008 — Two-Layer Source Plated Via / Reference Gap Qt Risk (한정 코드 및 병합 후 인수 완료 · 2026-10-29 KST)

- [x] **C_008 한정 PADS Source Two-Layer Via 전이 경로·Layer별 Reference Gap 및 Qt CPU/RHI Risk 표시** — [Issue #258](https://github.com/july-rust/BoardSI/issues/258), [C_008 원장](validation/c008-multilayer-reference-via-transition.md), [코드 PR #259](https://github.com/july-rust/BoardSI/pull/259), 최종 코드 HEAD [`ee765510b1b6f761c31adea8990eb5ef42405f18`](https://github.com/july-rust/BoardSI/commit/ee765510b1b6f761c31adea8990eb5ef42405f18/checks) **27/27 Check Runs SUCCESS**, 동일 HEAD Codex 완료·지적 0·미해결 0, 보호 Signed Verified Squash [`main@5db1840aa267624d20693aef4e48186383e04a57`](https://github.com/july-rust/BoardSI/commit/5db1840aa267624d20693aef4e48186383e04a57) 확인. Source SHA·Trace Net/Layer·연속 Endpoint와 단일 도금 Via Canonical Conductive Padstack, 기존 C_006 V2 Rational Gap, Source/Generation stale 차단 및 Qt CPU/RHI Marker를 **공개 합성 PADS V9.4 MILS 제한 범위만** 확인. **코드 post-main [`5db1840aa267624d20693aef4e48186383e04a57` 21/21 Checks SUCCESS](https://github.com/july-rust/BoardSI/commit/5db1840aa267624d20693aef4e48186383e04a57/checks) 및 9/9 Workflow SUCCESS** ([Push main #40791493502](https://github.com/july-rust/BoardSI/actions/runs/40791493502), [Qt Native #40791493610](https://github.com/july-rust/BoardSI/actions/runs/40791493610), [FEM Native #40791493531](https://github.com/july-rust/BoardSI/actions/runs/40791493531)) 독립 확인. **체크일: 2026-10-29 KST** (최초) · **근거일: 2026-10-29 KST** — 한정 코드·Qt 위험 표시만 완료. 문서 PR #260 별도 검증·병합 예정. 실제 Return Current/P009/RLGC/S·물리 Solver/Cache/전체 U09/U10/U39·T02/T04/T24/T52는 미승인, [M1 Issue #12](https://github.com/july-rust/BoardSI/issues/12) OPEN 유지.

### C_009 — Source Reference Stitching Via Candidate 및 Qt Risk 표시 (한정 코드·병합 후 인수 완료 · 2026-10-29 KST)

- [x] **C_009 공개 합성 PADS V9.4 MILS 두 Reference Layer의 Source-exact Through Stitch Via 후보 접점 증명·Qt CPU/RHI 위험 마커·Stale 무효화 — 제한 Source Geometry 기능 인수** — [Issue #261](https://github.com/july-rust/BoardSI/issues/261), [코드 PR #262](https://github.com/july-rust/BoardSI/pull/262), [검증 원장](validation/c009-reference-stitching-via-candidate.md). 같은 Source Reference Net의 두 Layer, 도금 Through Via, Canonical/Source 동일성, 각 Layer의 별도 COPCLS/COPCUT strict interior ViaCentre↔AreaInterior 접점, V4 및 V2 계약 불변, 정확한 Source Tick 거리²/표시용 mm 근삿값, Qt CPU/RHI 경고 및 Source SHA/선택/재Import/비동기 stale 차단만 포함. **최종 코드 HEAD** [`bdd29c061086b5d68742b3915dddd9e2bcf83033` 27/27 Check Runs·11/11 PR Workflows SUCCESS](https://github.com/july-rust/BoardSI/commit/bdd29c061086b5d68742b3915dddd9e2bcf83033/checks), 독립 Codex **동일 HEAD completed · findings 0 · unresolved 0**, 과거 지적 3건 수정 및 스레드 3/3 해결. **보호 Squash Signed Verified** [`main@a246bc9f15a3b27cf33cdd65370308ee6051fece`](https://github.com/july-rust/BoardSI/commit/a246bc9f15a3b27cf33cdd65370308ee6051fece) (`verified=true, reason=valid`), **독립 코드 post-main [21/21 Checks SUCCESS](https://github.com/july-rust/BoardSI/commit/a246bc9f15a3b27cf33cdd65370308ee6051fece/checks) · 8/8 Push Workflows SUCCESS** ([Qt Native #40814575323](https://github.com/july-rust/BoardSI/actions/runs/40814575323), [Numerical Core #40814574904](https://github.com/july-rust/BoardSI/actions/runs/40814574904)). **체크일: 2026-10-29 KST** (최초) · **근거일: 2026-10-29 KST**. 본 기록의 별도 문서 PR/병합 후 검증은 분리 게이트. 전기적 Return Current/P009/RLGC/S, 물리 Solver/Cache, 정식 U09/U10/U39/T02/T04/T24/T52 전체 인수는 **[ ]/미검증**; [M1 #12](https://github.com/july-rust/BoardSI/issues/12) OPEN.

### C_010 — Reference Copper Corridor / Stitch Source Area ID 정합성 (한정 코드·병합 후 인수 · 2026-10-29 KST)

- [x] **C_010 공개 합성 PADS V9.4 MILS 지원 Straight Signal 경로의 두 Reference Layer Copper Corridor Source Area와 C_009 Stitch Via 접촉 Source Area ID 동일성 검증 및 Qt CPU/RHI 위험 표시 — Source Geometry 한정 인수** — [Issue #264](https://github.com/july-rust/BoardSI/issues/264), [코드 PR #265](https://github.com/july-rust/BoardSI/pull/265), [C_010 검증 원장](validation/c010-reference-corridor-stitch-continuity.md). 전체 Signal leg 중심선의 Source-Owned Reference COPCLS/COPCUT Area 엄격 내부 포함, Cutout·Boundary·동일-Net 분리 Island·서로 다른 Area ID 및 변조 Source 거부, V5/V4/V2 기존 계약 불변 및 Qt 변경/재Import/비동기 stale 결과 무효화 **한정**. **최종 코드 HEAD** [`4e050874fbf621b2e8f28d50c3ab06ff93d77b05` **27/27 Checks·12/12 PR Workflows SUCCESS**](https://github.com/july-rust/BoardSI/commit/4e050874fbf621b2e8f28d50c3ab06ff93d77b05/checks), 동일 HEAD [Codex `completed`, findings 0, unresolved 0](https://github.com/july-rust/BoardSI/pull/265#issuecomment-6488503225), 기존 P1/P2 리뷰 스레드 2/2 해결. 보호된 Signed Verified Squash [`main@565e63cb2f3cc4deaca7f09543340747f6024bb0`](https://github.com/july-rust/BoardSI/commit/565e63cb2f3cc4deaca7f09543340747f6024bb0) (`verification.verified=true, reason=valid`, one-parent), **독립 post-code-main 21/21 Checks·9/9 Workflow Runs SUCCESS** ([Qt Native #40823686048](https://github.com/july-rust/BoardSI/actions/runs/40823686048), [Numerical Core #40823686042](https://github.com/july-rust/BoardSI/actions/runs/40823686042)). **체크일: 2026-10-29 KST** (최초) · **근거일: 2026-10-29 KST**. 문서 PR 자체의 동일 HEAD 리뷰·CI·보호 병합 및 post-doc-main 검증은 별도 게이트. **실제 전류/P009, RLGC/S, 물리 Solver/Cache 및 전체 U09/U10/U39/T02/T04/T24/T52는 미검증**. [상위 M1 #12](https://github.com/july-rust/BoardSI/issues/12)는 OPEN.

### C_011 — Source Stitch Via Pad Footprint V7 (한정 코드 범위 완료)

- [x] **C_011 한정 Source-exact 원형 Reference Stitch Via Conductive Pad 전체 면적 / COPCLS·COPCUT 외곽·Cutout 위험 판정 및 Qt CPU/RHI 원본 크기 외곽 표시·선택 검증** — **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST**. [Issue #267](https://github.com/july-rust/BoardSI/issues/267), [코드 PR #268](https://github.com/july-rust/BoardSI/pull/268), [V7 검증 원장](validation/c011-reference-stitch-pad-footprint.md). 최종 HEAD `e7f61e3984b9f84a7cfbd3fcf25480c423a41620` **27/27 PR Checks·11/11 PR Workflows SUCCESS**, 동일 HEAD Codex 리뷰 **completed / findings=0 / unresolved=0**, RHI P2 2건 해결. 서명된 Squash `main@32485198fbb24058435f8ef1821f561edef6a793` **Verified/valid**, 병합 후 독립 **21/21 Checks·9/9 Workflows SUCCESS**. *이 체크는 공개 합성 PADS V9.4 MILS Source 기하/Qt Preview 기능의 제한적 코드 인수만 표시한다.* 물리 Return Path/P009, 실제 Copper Fill·Thermal/Antipad, RLGC/S·제품 Solver/Cache, 정식 U09/U10/U39/T02/T04/T24/T52 및 상위 M1 [#12](https://github.com/july-rust/BoardSI/issues/12)는 모두 **미완료/OPEN**. C_011 **문서 PR 최종 보호 병합·post-doc-main 증거와 Issue 종료는 [#267](https://github.com/july-rust/BoardSI/issues/267)에서 별도로 판정**하며 코드 체크가 이를 대체하지 않는다.

### C_012 — Source Signal/Stitch Via Pad Clearance V8 (한정 코드 범위 완료)

- [x] **C_012 이종 Net Signal/Reference Via 원형 Conductive Pad의 Source Tick 기준 겹침·접촉·분리 판정 및 Qt CPU/RHI 위험 표시 — 한정 Source-geometry 코드 인수** · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST**. [Issue #270](https://github.com/july-rust/BoardSI/issues/270), [코드 PR #271](https://github.com/july-rust/BoardSI/pull/271), [V8 검증 원장](validation/c012-signal-reference-via-pad-clearance.md). 최종 코드 HEAD [`5874567053f2f7320e69fc674f0260808bc1deaa`](https://github.com/july-rust/BoardSI/commit/5874567053f2f7320e69fc674f0260808bc1deaa/checks) **27/27 PR Checks·11/11 PR Workflows SUCCESS**. 동일 HEAD Codex **completed / findings=0 / unresolved=0**, 기존 P1/P2 스레드 **2/2 해결**. 서명된 보호 Squash [`main@ae187f1f4437181676239106260b64238bc0afd8`](https://github.com/july-rust/BoardSI/commit/ae187f1f4437181676239106260b64238bc0afd8) **Verified/valid**·부모 1개, 독립 병합 후 **21/21 Checks·9/9 Workflows SUCCESS**, Ruleset #25046595 Strict/17개 필수/Bypass 없음. **이 체크는 공개 합성 PADS V9.4 MILS Source 기하 및 Qt Preview의 한정된 코드 완료만 표시한다.** 문서 PR [#272](https://github.com/july-rust/BoardSI/pull/272)의 최종 CI·Codex·서명된 병합·post-doc-main 및 Issue #270 종료는 **별도 미완료 게이트**로 유지한다. P009/Return Current, 제조 Copper·Antipad/Thermal, RLGC/S, 제품 Solver/Cache, U09/U10/U39/T02/T04/T24/T52 및 상위 M1 [#12](https://github.com/july-rust/BoardSI/issues/12)는 **미완료/OPEN**.

### C_013 — Source Signal Via Pad / Reference COPCUT V9 (제한 기능 PASS · 2026-10-29 KST)

- [x] **C_013 한정 Source COPCUT 직사각형과 Signal Via 원형 Pad Source Tick 간격 및 Qt CPU/RHI 마커** — **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST** [Issue #273](https://github.com/july-rust/BoardSI/issues/273) · [코드 PR #274](https://github.com/july-rust/BoardSI/pull/274) · [검증 원장](validation/c013-signal-via-reference-cutout-clearance.md). 최종 코드 HEAD `d15169e1c5b5367bc96ff9cb1cfbc37144db9d6e`의 **PR CI 27/27 SUCCESS**, 동일 HEAD Codex 완료·Findings 0·미해결 Thread 0(기존 P2 3건 모두 해결), Native Qt Windows x64/ARM64/macOS ARM64/Linux offscreen 및 Rust V9 공개 합성 테스트/기존 회귀. [보호 Squash Verified `main@96f49248`](https://github.com/july-rust/BoardSI/commit/96f49248d0cda95b729734683d9918cd9444a45a) GitHub 서명 `verified=true, reason=valid`, 독립 **post-main 21/21 Check Runs SUCCESS + 9/9 Push Workflows SUCCESS**. Source 원본 정수 Tick Gap/Tangency/Overlap/Missing 분류 및 기존 V8 Byte-exact Receipt 보존이라는 **한정 기능만 PASS**. P009 Return Current/제조 Copper/실제 DRC/실물 PCB SI/Solver/Cache/정식 U/T 및 M1 #12는 **미완료**. 문서 자체의 보호 병합·최종 `main` 검증과 Issue 종료는 [#273](https://github.com/july-rust/BoardSI/issues/273)에서 별도로 증빙.

### C_014 — Multi-COPCUT Signal Via / Reference Source geometry V10 (한정 코드 범위 PASS · 2026-10-30 KST)

- [x] **C_014 Source-owned Reference Area별 독립 직사각형 COPCUT 0–8개/레이어 및 원형 Signal Via Pad Source Tick 간격·Qt CPU/RHI 위험 표시(한정 코드 인수)** — **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST**. [Issue #276](https://github.com/july-rust/BoardSI/issues/276) · [코드 PR #277](https://github.com/july-rust/BoardSI/pull/277) · [검증 원장](validation/c014-multi-cutout-signal-via-clearance.md). 코드 최종 HEAD [`fd95810bb8e60ca57a0c42b59ba83c6297021bcb`](https://github.com/july-rust/BoardSI/commit/fd95810bb8e60ca57a0c42b59ba83c6297021bcb/checks) **PR 11/11 Workflow SUCCESS**, Native Qt Windows x64/ARM64·macOS ARM64·Linux offscreen, Rust Debug/Release/fmt/Clippy 및 이전 B/C 회귀 PASS. **독립 동일 HEAD Codex completed / findings 0 / unresolved 0**, 기존 P1/P2 2/2 스레드 해결. 보호 Squash [`main@83623c167c27cdb51c58d9a4feab434068d7ed40`](https://github.com/july-rust/BoardSI/commit/83623c167c27cdb51c58d9a4feab434068d7ed40)는 GitHub 서명 **Verified/valid · 부모 1개**. [독립 post-main 증거 #40965201300](https://github.com/july-rust/BoardSI/actions/runs/40965201300): **21/21 Check Runs·GitHub API `event=push` 8/8 Workflow Runs SUCCESS**. 0~1개 Cutout 시 V9, 다중 시 V8 Receipt Byte-exact 호환; Cutout 중첩/경계 접촉/Source 불명확 시 Fail-closed. **이 체크는 공개 합성 PADS V9.4 MILS Source Geometry·Qt 한정 인수만 뜻하며 문서 PR #278의 최종 병합·CI 검증은 별도이다.** 물리 Return Current P009, 제조 Copper/Antipad/Thermal, 실제 DRC·SI·Solver·Cache, 공식 U09/U10/U39/T02/T04/T24/T52와 상위 M1 [#12](https://github.com/july-rust/BoardSI/issues/12)는 **미완료/OPEN**.

### C_015 — Circular COPCCO / rectangular COPCUT mixed Source Geometry V11 (한정 인수 완료 · 2026-10-30 KST)

- [x] **C_015 원형·직사각형 혼합 Cutout Source Geometry·Qt 독립 V11 한정 인수 완료** — **최초 체크일: 2026-10-30 KST / 근거일: 2026-10-30 KST**. [Issue #280](https://github.com/july-rust/BoardSI/issues/280) · [Code PR #281](https://github.com/july-rust/BoardSI/pull/281) · [Verified protected Squash `273417a7555a1dddc5724b15b062029ddf3b6e52`](https://github.com/july-rust/BoardSI/commit/273417a7555a1dddc5724b15b062029ddf3b6e52) · [검증 원장](validation/c015-circular-mixed-cutout-source-geometry.md). 원본 PADS COPCCO/COPCUT 혼합 형상에 대한 i128 정수(Source Tick=254 nm) 검사, Canonical/cardinal-circle Source-local ViaCentre 접속, **진짜 원본 V8 필수 계보**, Qt 독립 Signal/Stitch Via/void·nested CONTACT ID 검증과 원래 크기의 CPU/RHI 위험 표시·캐시 무효화 구현. 정확한 코드 HEAD `95daabc02339aa37a3c5ddb84af7e4798c54e96b`에서 Rust Debug/Release 원형 V8 및 음성 fixture PASS, [Numerical run 40984559607](https://github.com/july-rust/BoardSI/actions/runs/40984559607), [Qt Native run 40984559702](https://github.com/july-rust/BoardSI/actions/runs/40984559702), **PR 12/12 Workflow SUCCESS**; Codex 해당 HEAD 리뷰 완료 findings=0·unresolved=0, P2 수정 스레드 해결; 서명 `verified=true, reason=valid`인 Squash 보호 병합. 원본 main code commit `273417a…`의 **사후 Checks 21/21 SUCCESS, Push Workflows 8/8 SUCCESS**. **C_015 제한적인 원본 형상/접속 후보 결과만 인수**: `return_path_p009=UNSUPPORTED`, `physical_si=NOT_RUN`, `solver_ready=false`, `cache_reuse_eligible=false`. 정식 **U09/U10/U39/T02/T04/T24/T52** 및 상위 M1 [#12](https://github.com/july-rust/BoardSI/issues/12)는 계속 미완료/OPEN; 어떠한 SI·리턴전류·실제 제조품 DRC 인수도 의미하지 않음.

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

| 2026-10-29 | C_009 Reference Stitching Via Source Contact·Qt Native 한정 인수 | [Issue #261](https://github.com/july-rust/BoardSI/issues/261), [코드 PR #262](https://github.com/july-rust/BoardSI/pull/262), [정확한 코드 HEAD `bdd29c0` 27/27](https://github.com/july-rust/BoardSI/commit/bdd29c061086b5d68742b3915dddd9e2bcf83033/checks), [Signed Verified `main@a246bc9` post-main 21/21](https://github.com/july-rust/BoardSI/commit/a246bc9f15a3b27cf33cdd65370308ee6051fece/checks), [Qt Native Push](https://github.com/july-rust/BoardSI/actions/runs/40814575323), [원장](validation/c009-reference-stitching-via-candidate.md) | **최초 체크일: 2026-10-29 KST · 근거일: 2026-10-29 KST.** PR Workflow 11/11, 동일 HEAD Codex 지적/미해결 0, 리뷰 스레드 3/3 해결, 독립 Main Push 8/8 워크플로 성공. **단일 Reference Via 후보/Source Geometry·Qt Risk만 [x]**, P009/실전 Return Path/물리 SI·Solver/Cache/Uxx·Txx 미완, M1 #12 OPEN. 문서 PR 자체 검증은 별도. |


| 2026-10-29 | C_007 다중 Straight Route Source Reference Gap / Qt Native 제한 인수 | [Issue #255](https://github.com/july-rust/BoardSI/issues/255), [PR #256](https://github.com/july-rust/BoardSI/pull/256), [정확한 코드 HEAD 27/27](https://github.com/july-rust/BoardSI/commit/fdfad35931805e47d6658aeed7966ed3d778b741/checks), [Signed Verified main@08849e3f, post-main 21/21 Checks](https://github.com/july-rust/BoardSI/commit/08849e3fdf048f60a8a06265792acd4f51316b2f/checks), [Push on main #40781834140](https://github.com/july-rust/BoardSI/actions/runs/40781834140), [Qt runtime #40781834617](https://github.com/july-rust/BoardSI/actions/runs/40781834617) | **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST**. 독립 Codex 최신 HEAD 지적 0·미해결 0, Windows x64/ARM64·macOS ARM64 실프로세스와 Linux Qt PASS, post-code-main 9/9 Workflow SUCCESS. **Source Geometry/Qt 한정**, P009/Return Current/물리 Solver 및 정식 U/T 불승인, M1 #12 OPEN. |

| 2026-10-28 | C_006 Source Reference gap intervals / Qt Native 제한 인수 | [Issue #252](https://github.com/july-rust/BoardSI/issues/252), [코드 PR #253](https://github.com/july-rust/BoardSI/pull/253), [exact HEAD 27/27 checks](https://github.com/july-rust/BoardSI/commit/ed5e137bbb8b1bebcfde832961038d43286a7753/checks), [Signed Verified main 5e4de47 + 21/21 post-main checks](https://github.com/july-rust/BoardSI/commit/5e4de472a90539221f69e5cf11b88e20d9fd7361/checks), [main Push](https://github.com/july-rust/BoardSI/actions/runs/40697041065) | **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST**. C_006 한정 Source-geometry interval 및 CPU/RHI·Native 3 OS + Linux Qt PASS. 동일 HEAD Codex 0 findings/0 unresolved; post-main 9/9 Workflow SUCCESS. M1 #12와 정식 U/T 및 P009·물리 Solver 미완. |

| 2026-10-27 | B_005 post-main Push CI 완료 체크 | [Verified main@`7a1ed48`](https://github.com/july-rust/BoardSI/commit/7a1ed48bf905df8357691a1f51b73e130a74537c/checks) 21/21 SUCCESS, [Push on main #40501517050](https://github.com/july-rust/BoardSI/actions/runs/40501517050) SUCCESS | **체크일: 2026-10-27 KST**, **근거일: 2026-10-27 KST**. B_005 제한 기능 병합 후 CI 최종 확인, Issue #12 OPEN과 P009/물리 SI 공식 U/T 미완 유지. |

| 2026-10-27 | B_005 제한 Source-bound Reference 기하 감사 최초 완료 체크 | [PR #227](https://github.com/july-rust/BoardSI/pull/227), [Verified Squash `7a1ed48`](https://github.com/july-rust/BoardSI/commit/7a1ed48bf905df8357691a1f51b73e130a74537c), [PR HEAD `284f5f04` 23/23 CI](https://github.com/july-rust/BoardSI/commit/284f5f04fe3e1d5a07826c0c158893a6c685ea87/checks), Codex 동일 HEAD 주요 지적 0, Rust Debug/Release 17/17 | 최초 체크일 **2026-10-27 KST**, 근거일 **2026-10-27 KST**. `SOURCE_ORDER_GEOMETRY_ONLY` 제한. Post-main CI 별도 확인 필요; B_001 Issue #12 OPEN, 물리 P009 및 U/T 항목은 미완. |

| 2026-10-27 | B_005 Source-bound Reference 기하 감사 코드 HEAD 검증(정식 체크 보류) | [PR #227](https://github.com/july-rust/BoardSI/pull/227), [`225489bedabc146e31b83137267aaa29d86f8503` 23/23 checks](https://github.com/july-rust/BoardSI/commit/225489bedabc146e31b83137267aaa29d86f8503/checks), Rust Debug/Release B_005 17/17 PASS | 원본 Net/Layer/Trace/Arc/Area 재검증, Return Path 기하 커버리지 한정 코드 PASS. 동일 코드 최신 Codex 후속 리뷰·보호 병합·main CI 확인 전에는 신규 [x] 및 최초 체크일 없음; B_001 Issue #12 OPEN/공식 U/T 미검증 유지. |

| 2026-10-27 | B_004 제한 Copper Area 국소 접점 코드 인수 | [PR #226](https://github.com/july-rust/BoardSI/pull/226), [정확한 코드 HEAD `7c68f9523b65999271fff5b510e984ec30dac72a` CI](https://github.com/july-rust/BoardSI/commit/7c68f9523b65999271fff5b510e984ec30dac72a/checks), [검증 원장](validation/b004-reference-copper-connectivity.md) | 공개 합성 Straight COPCLS 내부 Trace/Pad Centre 및 동일 소스 Area의 제한적 Graph `[x]` 최초 체크일·근거일 **2026-10-27 KST**. 12 workflow/23 job 모두 성공, 동일 HEAD Codex findings 0·기존 4건 해결. 보호 병합·Post-main SHA는 PR 완료 후 별도 확인; B_001 Issue #12 OPEN, 정식 U/T·P009·물리 Solver 체크 없음. |


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

| 2026-10-29 | C_008 두 Signal Layer 단일 도금 Via Source Reference 기하·Qt Risk 제한 인수 | [Issue #258](https://github.com/july-rust/BoardSI/issues/258), [PR #259](https://github.com/july-rust/BoardSI/pull/259), [정확한 코드 HEAD `ee765510` 27/27 CI SUCCESS](https://github.com/july-rust/BoardSI/commit/ee765510b1b6f761c31adea8990eb5ef42405f18/checks), [Signed Verified `main@5db1840a` post-main 21/21](https://github.com/july-rust/BoardSI/commit/5db1840aa267624d20693aef4e48186383e04a57/checks), [Qt Native](https://github.com/july-rust/BoardSI/actions/runs/40791493610), [FEM Native](https://github.com/july-rust/BoardSI/actions/runs/40791493531), [원장](validation/c008-multilayer-reference-via-transition.md) | **최초 체크일: 2026-10-29 KST · 근거일: 2026-10-29 KST**. Codex 동일 HEAD 지적/미해결 0, post-main 9/9 Workflow SUCCESS. **PADS 합성 Source Geometry 및 Qt 표시만 [x]**; 실제 Return Current/P009·물리 SI·Solver/Cache 및 정식 U/T 미승인. 상위 M1 #12 OPEN. |

### 필수 참고 근거

- [A_035 감사](validation/a035-p2z-final-preview-acceptance.md) · [A_034 Native 통합](validation/a034-p2y-integration-stabilization.md) · [A_033 원본 정합성](validation/a033-p2x-source-artwork-parity.md) · [A_032 국소 접점](validation/a032-p2w-local-connectivity.md)
- [M0 종료 게이트](validation/m0-completion-gate.md) · [M0 추적 원장](audits/m0-requirements-traceability-ledger.md) · [M2 검증 원장](validation/README.md)
- [원본 PH-08 FAIL](validation/m2-g2-ph08-stage5b-original-replay-evidence.md) · [SEC_001 코드 검사](validation/sec001-codeql-markdown-comment-scan.md) · [Code scanning alert #1](https://github.com/july-rust/BoardSI/security/code-scanning/1)
