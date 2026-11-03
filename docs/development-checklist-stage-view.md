# BoardSI — 스테이지별 개발 체크리스트 현황판

> **기준:** 2026-11-03 KST · GitHub `main@4ddacf2bb1bc8210fba90a60e6d798a169b7d27a` 문서의 체크 항목을 읽기 쉽게 재배치한 **조회용 현황판**입니다.  
> **정식 원장:** [개발 체크리스트 전체 원문](development-checklist.md). 원문의 체크 상태·최초 체크일·근거일·PR/CI/SHA·제한 조건은 이 문서로 대체하거나 삭제하지 않습니다.  
> 이 현황판은 문서 스냅샷이며 실시간으로 자동 갱신되지 않습니다.

**판정 주의:** 여기서 ‘확인’은 해당 항목에 명시된 합성 Source·Rust·Qt 등의 *한정 증거*를 뜻합니다. 제품 전체 인수, 제조 도체의 전기적 연속성, PH-08, 실제 RLGC/S Solver 자격 또는 정식 U/T 시험 통과와 동일하지 않습니다.

## 빠른 이동

[Stage A](#stage-a) · [Stage B](#stage-b) · [Stage C](#stage-c) · [Stage D](#stage-d) · [제품 마일스톤](#milestones) · [U/T 정식 인수](#formal-acceptance) · [아직 남은 핵심 작업](#remaining)

## 1. 단계별 상태 요약

| 구분 | 의미 | 한정 확인 | 남은 기록 |
| :--- | :--- | ---: | ---: |
| [**A**](#stage-a) | Preview · Import · Artwork | 12 | 5 |
| [**B**](#stage-b) | Stackup · Copper/Reference 후보 | 17 | 6 |
| [**C**](#stage-c) | Reference 기하 · Qt 통합 · 감사 | 40 | 8 |
| [**D**](#stage-d) | PADS Source 접촉 후보 · T/X Junction | 39 | 2 |
| [**U**](#formal-acceptance) | U01–U48 제품 요구사항 | 1 | 47 |
| [**T**](#formal-acceptance) | T01–T54 정식 인수시험 | 0 | 54 |

> **계산 기준:** 기존 파일의 정확한 `- [x]` / `- [ ]` 기록 개수. 퍼센트로 단계 완료율을 산출하지 않습니다. D_017 한정 Source 기하 검증 완료가 M1 전체 완료를 의미하지 않습니다.

---

<a id="stage-a"></a>
## Stage A — 제한 Preview / Import / Artwork

**기록:** 확인 12건 · 미확인 5건

Stage A는 공개 합성 PADS 데이터에 대한 Preview, 변환, Qt 표시 및 Native CI의 *지원된 부분*을 검증했습니다.

### A 단계에서 확인한 기능

<details>
<summary>제한 Preview 검증 12건 펼치기</summary>

- [Qt Widgets Preview와 별도 Rust Worker / Versioned IPC의 제한 작동](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L34)
- [공개 합성 PADS Layout ASCII V9.4 MILS의 **명시된 subset만** 파싱 및 비지원 입력 거절](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L35)
- [지원 Straight Trace/Arc/Via/Component·RF Pad 및 Bottom Mirror 변환](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L36)
- [지원 Board Outline과 다각형·곡선·복합 외곽](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L37)
- [지원 Copper Area/원형·곡선·비직교 외곽, Cutout/Hole와 CPU/RHI 표시](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L38)
- [원본 Source tick·좌표·원본 개체별 Provenance와 Artwork 독립 검증](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L39)
- [제한 Layer·Net 표시, Filtering, Picking, Zoom/Fit 및 Qt Quick RHI Presented Frame](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L40)
- [원본 정확히 일치하는 일부 Trace/Arc/Via **국소 접점만** 그래프 근거 생성](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L41)
- [CPU/RHI 국소 접점 마커와 소스 정보/Visible Frame 정합성](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L42)
- [연속 다른 PADS 합성 보드 재Import의 stale 객체·필터 상태 제거](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L43)
- [10K·100K·1M **합성** 객체의 렌더/상호작용 회귀 시험(성능 보증 아님)](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L44)
- [Windows x64·Windows ARM64·macOS ARM64 **Native CI 시험 환경** 범위 PASS](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L45)

</details>

### A 단계에서 남은 필수 작업

- [임의 상용 PADS 파일·버전의 전체 객체를 자격 검증](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L53)
- [Whole-board Open/Short, Pad/Pin/Pour/Thermal 접속의 물리 판정](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L54)
- [GPU VRAM/driver reset/실장 기기/1M@60 FPS를 정량 측정](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L55)
- [실제 PCB/Substrate 기반 RLGC/S·DC/30 GHz·Touchstone 검증](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L56)
- [정식 설치/저사양 실물 PC/재배포 라이선스 검증](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L57)

---

<a id="stage-b"></a>
## Stage B — Stackup / Copper / Reference

**기록:** 확인 17건 · 미확인 6건  
**원문:** [검증 상세·날짜·PR/CI 링크 확인](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L59)

| 작업 | 제한 검증 범위 | 확인 | 미확인 |
| :--- | :--- | ---: | ---: |
| [**B_001**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L59) | 추가 검증 — 제품 물리 자격과 구별 | 5 | 1 |
| [**B_003**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L70) | 제한 완료 — Trace/Pad/Via 원본 접촉 근거 (Issue #12 연속 작업) | 1 | 1 |
| [**B_004**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L76) | 제한 완료 — 단일 단순 Copper Area 국소 접촉 검증 | 1 | 0 |
| [**B_005**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L80) | 제한 완료 — Source-bound Reference 기하 커버리지 | 2 | 0 |
| [**B_006**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L86) | 제한 완료 — Source-bound Through Via/COPCLS 국소 접촉 감사 | 2 | 0 |
| [**B_007**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L91) | COPCUT-aware bounded local contact implementation and integration | 2 | 2 |
| [**B_008**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L100) | 단일 COPCUT Reference 경로 기하 감사 | 2 | 0 |
| [**B_009**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L106) | B-series 제한 범위 최종 감사 및 C 단계 인계 | 2 | 2 |

<details>
<summary>미확인 항목 6건 확인 (원문 근거로 이동)</summary>

- [**B_001 전체 Issue #12 완료**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L64) · B_001
- [**B_001 전체 Issue #12 정식 인수**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L74) · B_003
- [**B_007 exact latest HEAD independent Codex final-review closure**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L95) · B_007
- [**B_001 전체 Issue #12 정식 인수**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L98) · B_007
- [**B_007 최종 Source HEAD 독립 Codex 리뷰 종결**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L110) · B_009
- [**B_001 Issue #12 전체 정식 인수**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L111) · B_009

</details>

---

<a id="stage-c"></a>
## Stage C — Source Reference 경로 / Qt 안정화 / 통합 감사

**기록:** 확인 40건 · 미확인 8건  
**원문:** [검증 상세·날짜·PR/CI 링크 확인](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L113)

| 작업 | 제한 검증 범위 | 확인 | 미확인 |
| :--- | :--- | ---: | ---: |
| [**C_001**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L113) | 다중 Source-owned COPCUT / Disconnected Island | 1 | 0 |
| [**C_002**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L117) | Reference Split 및 Layer Transition Topology Risk | 2 | 1 |
| [**C_003**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L123) | Immutable Scenario and Stale Result | 1 | 1 |
| [**C_004**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L128) | Material Provenance / Loss Input Gate | 2 | 0 |
| [**C_005**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L133) | Qt Source Reference Risk 표시/무효화 제한 인수 | 1 | 0 |
| [**C_006**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L137) | Source-exact Reference Gap Interval / Qt CPU/RHI 제한 기능 완료 | 1 | 0 |
| [**C_007**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L160) | Multi-Segment Source Reference Route / Qt Risk | 1 | 0 |
| [**C_008**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L165) | Two-Layer Source Plated Via / Reference Gap Qt Risk | 1 | 0 |
| [**C_009**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L169) | Source Reference Stitching Via Candidate 및 Qt Risk 표시 | 1 | 0 |
| [**C_010**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L173) | Reference Copper Corridor / Stitch Source Area ID 정합성 | 1 | 0 |
| [**C_011**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L177) | Source Stitch Via Pad Footprint V7 (한정 코드 범위 완료) | 1 | 0 |
| [**C_012**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L181) | Source Signal/Stitch Via Pad Clearance V8 (한정 코드 범위 완료) | 1 | 0 |
| [**C_013**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L185) | Source Signal Via Pad / Reference COPCUT V9 | 1 | 0 |
| [**C_014**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L189) | Multi-COPCUT Signal Via / Reference Source geometry V10 | 1 | 0 |
| [**C_015**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L193) | Circular COPCCO / rectangular COPCUT mixed Source Geometry V11 | 1 | 0 |
| [**C_016**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L197) | Reference Stitch Via full circular Pad / mixed cutout Source V12 | 1 | 0 |
| [**C_017**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L201) | Reference Gap/Layer Transition V1–V4 한정 통합 안정화 | 1 | 0 |
| [**C_018**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L205) | Reference Stitch Via / Copper Corridor V5–V8 제한 통합 회귀 인수 | 1 | 0 |
| [**C_019**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L209) | PADS Mixed Cutout / Signal and Reference Via Pad V9–V12 한정 통합 회귀 인수 | 1 | 0 |
| [**C_020**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L213) | B_007 최종 리뷰 증거와 B_003–B_008 Connectivity 제한 감사 | 3 | 0 |
| [**C_021**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L226) | Source Reference Copper Area-to-Area 제한 감사 | 1 | 0 |
| [**C_022**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L220) | PADS Multi-Area Source Copper Component Candidates V14 | 2 | 0 |
| [**C_023**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L230) | Multi-Area Reference Corridor V15 | 1 | 0 |
| [**C_024**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L235) | C단계 통합 증거 감사 및 D단계 인계 | 1 | 0 |
| [**C_025_03**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L250) | 구형 PADS Signature/Qt Import PR 정합화 및 안전 종료 (기능 인수와 별개) | 2 | 0 |
| [**C_025_04**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L245) | 구형 정책·B_003 Draft PR 정합화 및 안전 종료 (문서·PR 관리 한정) | 2 | 0 |
| [**C_025_05**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L240) | M0 F1 사양서/U-T 추적성 및 Issue #2 완료 조건 감사 | 1 | 1 |
| [**C_025_09**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L450) | macOS Qt 글로벌 타이머 지연 분리 진단 및 #46/#41 종료 안전성 | 2 | 2 |
| [**C_025_10**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L457) | M0 F6 공개 Fixture 증거 및 Issue #7 안전 판정 | 4 | 3 |

<details>
<summary>미확인 항목 8건 확인 (원문 근거로 이동)</summary>

- [**B_001 Issue #12 및 정식 U09/U10/U39·T02/T04/T24/T52 전체 인수**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L121) · C_002
- [**C_003 이후 실 PCB 완전 Solver Snapshot/Worker transport 및 실제 재사용 캐시**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L126) · C_003
- [**Issue #2 및 M0 F1 정식 종료**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L243) · C_025_05
- [**Issue #46 코드 원인 및 근본 해결 입증**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L454) · C_025_09
- [**Issue #41 종료**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L455) · C_025_09
- [**Issue #7 F6 공개 Fixture 배포 법적 승인**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L462) · C_025_10
- [**Issue #7 F6 실제 약 8 GiB 기준 PC 인수**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L463) · C_025_10
- [**Issue #7 전체 DoD 및 정식 T14/T15/T49**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L464) · C_025_10

</details>

---

<a id="stage-d"></a>
## Stage D — PADS Source Contact / Junction

**기록:** 확인 39건 · 미확인 2건  
**원문:** [검증 상세·날짜·PR/CI 링크 확인](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L471)

| 작업 | 제한 검증 범위 | 확인 | 미확인 |
| :--- | :--- | ---: | ---: |
| [**D_001**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L471) | Trace endpoint / conductive SMD Pad footprint Source witness | 2 | 1 |
| [**D_002**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L467) | Source COPCUT-aware two-area hypothesis | 1 | 0 |
| [**D_003**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L487) | Three-Area COPCUT Source Geometry Candidate | 1 | 0 |
| [**D_004**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L483) | Source Trace Endpoint / Three-Area Candidate Attachment | 1 | 0 |
| [**D_005**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L492) | SMD Pad → Trace → Three-Area COPCUT Source candidate | 2 | 1 |
| [**D_006**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L499) | Plated Via Pad → Three-Area COPCUT Source contact candidate | 3 | 0 |
| [**D_007**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L505) | Plated Via Pad → Straight Trace → Three-Area COPCUT Source chain | 3 | 0 |
| [**D_008**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L511) | Plated Through Via Pad → Dual-Layer Source COPCLS/COPCUT contacts | 3 | 0 |
| [**D_009**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L517) | Plated Through-Via Dual-Layer Pad → Layer-B Straight Trace → Third COPCLS | 2 | 0 |
| [**D_010**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L523) | Plated Through-Via Dual-Layer / Two-Trace / Four-Area Source qualification | 1 | 0 |
| [**D_011**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L527) | PADS Dual-Layer Source Contact Candidate Graph | 2 | 0 |
| [**D_012**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L534) | Through-Via Source Span & Plating Provenance Witness | 3 | 0 |
| [**D_013**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L541) | PADS Legacy V1 Barrel Compatibility Provenance & Fail-Closed Qualification | 3 | 0 |
| [**D_014**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L547) | PADS Two-Via Source Contact Chain & Fail-Closed Qualification | 3 | 0 |
| [**D_015**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L553) | PADS Same-Layer Trace T-Junction Source Contact & Fail-Closed Qualification | 3 | 0 |
| [**D_016**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L560) | PADS Same-Layer Trace X-Junction SOURCE Qualification | 3 | 0 |
| [**D_017**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L567) | PADS Same-Layer Oblique Trace X-Junction SOURCE Qualification | 3 | 0 |

<details>
<summary>미확인 항목 2건 확인 (원문 근거로 이동)</summary>

- [**M1 전체 및 물리 전기 접속, P009 인수**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L475) · D_001
- [**D_005 별도 문서 PR #347 보호 병합 및 Issue #345 종료**](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L496) · D_005

</details>

---

<a id="milestones"></a>
## 제품 개발 마일스톤 (M0–M5)

| 마일스톤 | 현재 사양·인수 해석 |
| :--- | :--- |
| **M0** | 구조·기술 시연은 있으나 종료 게이트가 별도로 남음 (**BLOCKED**) |
| **M1 / M1a / M1b** | PADS 제한 Import·Reference 후보, Via/Pad 및 Port 일부만 증빙 (**정식 미완**) |
| **M2 / M2a–c** | 합성 연구 데이터가 제품 RLGC/S Solver·DC·Touchstone을 증명하지 않음 (**미완**) |
| **M3–M5** | CAD 확장·고급 기능은 후속 요구사항 (**미착수/미완**) |

정확한 세부 근거: [마일스톤과 후속 작업 원문](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L141-L159).

<a id="formal-acceptance"></a>
## 제품 요구사항 및 정식 인수시험

| 체계 | 체크한 범위 | 아직 체크되지 않은 범위 | 원문 |
| :--- | ---: | ---: | :--- |
| **U01–U48** | 1 | 47 | [U 체크리스트](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L255-L315) |
| **T01–T54** | 0 | 54 | [T 체크리스트](https://github.com/july-rust/BoardSI/blob/4ddacf2bb1bc8210fba90a60e6d798a169b7d27a/docs/development-checklist.md#L316-L385) |

실제 사양 전체를 만족한 결과만 정식 완료 체크 대상입니다. 제한 기능 테스트 PASS만으로 U/T 항목을 완료 처리하지 않습니다.

<a id="remaining"></a>
## 아직 남은 핵심 검증 범위

- **실 PCB 전기 접속:** 제조 Copper, Through-Via Barrel, Pad/Pour/Thermal 및 전체 Open/Short 검증
- **Reference Return Path:** 분할 Plane/Cutout 실제 연속성 및 P009
- **Solver·결과:** 검증된 물성·RLGC·S-Parameter, DC~30 GHz, PH-08 및 Solver/Cache
- **제품 인수:** 플랫폼 실기기, 배포/라이선스, 사양서 U/T의 정식 시험

정확한 미완 범위와 상태 변경 근거는 언제나 [정식 원장](development-checklist.md) 및 각 Issue/PR에서 확인해야 합니다. **Stage별 하위 작업 기록과 M0–M5 제품 마일스톤은 별개입니다.**
