# BoardSI 개발 체크리스트 — 단일 정본

> **E_030 · 2026-11-15 KST** · [최종 감사](validation/e030-e-stage-final-closure-audit.md) · [사양서 v1.0](specification/BoardSI_Development_Spec_v1.0.md)
>
> 현재 상태는 **이 파일 하나**에 기록합니다. 기존 2,802줄의 상세 내용은 [변경 전 커밋의 불변 원본](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md)에서 확인하세요. 기존 Git 커밋을 직접 참조하므로 원본 내부 상대경로 증거 링크도 그대로 동작합니다. 과거 원장은 두 번째 현행 체크리스트가 아닙니다.

**바로가기:** [현황](#stage-overview) · [E 작업](#stage-e) · [F 작업](#stage-f) · [차단](#blocked) · [A~D](#stage-history) · [U/T](#stage-acceptance) · [항목별 체크 증거](#check-evidence)

<a id="stage-overview"></a>
## 1. 전체 현황

**전체 체크 상태: 166건 확인 · 128건 미확인.**

| 범위 | 확인 | 미확인 | 의미 |
| :--- | ---: | ---: | :--- |
| A | 12 | 5 | PADS/Qt Preview 부분집합 |
| B | 17 | 6 | Stackup/Reference 부분 증거 |
| C | 40 | 8 | Source/Qt 국소 회귀 |
| D | 95 | 8 | Copper/다층/CAD 부분 구현 |
| 보안 | 1 | 0 | CodeQL Alert #1 한정 Fixed |
| U01–U48 | 1 | 47 | 정식 요구사항 |
| T01–T54 | 0 | 54 | 정식 시험 인수 |

**주의:** A~D의 [x]는 해당 *한정 검증*만 확인했다는 뜻입니다. E 작업은 이 수치에 포함하지 않습니다. 실제 PCB 전기 접속·Return Path·RLGC/S Solver 및 PH-08 원본 Gate는 정식 인수가 아닙니다.

<a id="stage-e"></a>
## 2. E_001~E_030 정산

| 작업 | 핵심 결과 | 상태 | 근거 |
| :--- | :--- | :--- | :--- |
| E_001 | PH-07 사후 감사 증거 재검토와 안전한 #41 종결 판단 | 한정 완료 | [증거](e-stage-closure-plan.md) |
| E_002 | M0 F1 요구사항 U/T 추적성 누락·충돌 1묶음 정리 | 한정 완료 | [증거](e-stage-closure-plan.md) |
| E_003 | Rust/Qt/IPC 결정·버전 호환 근거 한 묶음 인수 | 한정 완료 | [증거](e-stage-closure-plan.md) |
| E_004 | Canonical Layer/Via Source provenance 한 계약 검증 | 한정 완료 | [증거](e-stage-closure-plan.md) |
| E_005 | 비차단 Worker의 취소 또는 late-response 한 경로 인수 | 한정 완료 | [증거](e-stage-closure-plan.md) |
| E_006 | 세 Native OS Toolchain·실행 근거 최신성 감사 | 증거 일부 미확인 | [증거](e-stage-closure-plan.md) |
| E_007 | 공개 Fixture 출처·해시·재배포 권한 한 묶음 정합화 | 증거 일부 미확인 | [증거](validation/e007-public-fixture-provenance-rights-qualification.md) |
| E_008 | 입력 판별 및 readiness 한 Source 종류 보강 | 한정 완료 | [증거](e-stage-closure-plan.md) |
| E_009 | PADS V9.4 ROUTE Glued Via 0x0800 Source 플래그 한정 지원 | 한정 완료 | [증거](validation/e009-pads-glued-via-grammar-qualification.md) |
| E_010 | 실제 메인 Qt Canonical Through Via Import/IPC/CPU Picking/재Import/Fail-Close… | 한정 완료 | [증거](validation/e010-main-qt-import-picking-qualification.md) |
| E_011 | Stackup 편집/저장/재Import 무효화 한 경로 검증 | 한정 완료 | [증거](validation/e011-stackup-edit-persistence-reimport-qualification.md) |
| E_012 | 실제 GPU 백엔드 1개 플랫폼 Native 원본 계측 감사 | 한정 완료 | [증거](validation/e012-actual-hardware-gpu-qualification.md) |
| E_013 | ODB++ ZIP DEFLATE Method 8 한 경로 구현 | 한정 완료 | [증거](validation/e013-odb-zip-deflate-qualification.md) |
| E_014 | ODB++ tar.gz/tgz 한 Source FileMap 경로 | 한정 완료 | [증거](validation/e014-odb-gzip-tar-qualification.md) |
| E_015 | UNIX Compress features.Z LZW Source 정규화 | 한정 완료 | [증거](validation/e015-odb-features-z-qualification.md) |
| E_016 | Positive Multi-Island Surface (Negative 제외) | 한정 완료 | [증거](validation/e016-odb-positive-multi-island-qualification.md) |
| E_017 | ODB++ 단일 Step Repeat 평행이동·교차 전송 검증 | 한정 완료 | [증거](validation/e017-odb-step-repeat-qualification.md) |
| E_018 | ODB++ EDA PKG/PIN RC Source 매핑 1종 | 한정 완료 | [증거](validation/e018-odb-eda-package-pin-qualification.md) |
| E_019 | ODB++ 제조 PLATED/STANDARD 원형 Hole→SOURCE-only Canonical Via 후보 1종 | 한정 완료 | [증거](validation/e019-odb-drill-via-candidate-qualification.md) |
| E_020 | ODB++ EDA matrix/stackup.xml Source Material/Thickness/relative z 1개 경로 | 한정 완료 | [증거](validation/e020-odb-stackup-material-z-qualification.md) |
| E_021 | 메인 Qt ODB++ 공개 합성 SOURCE 보드 1개 비차단 Import | 증거 일부 미확인 | [증거](validation/e021-odb-main-qt-import-qualification.md) |
| E_022 | ODB++ 동일 Copper Layer Straight Trace↔Round Pad 국소 SOURCE Graph 1종 | 증거 일부 미확인 | [증거](validation/e022-odb-local-electrical-graph-qualification.md) |
| E_023 | ODB++ 독립 Vendor Fixture와 Cross-format Oracle 1건 | BLOCKED · EB-01 | [증거](validation/e023-odb-vendor-cross-format-oracle-qualification.md) |
| E_024 | ODB++ 전체 15개 Gate / #10 DoD 및 정식 U/T 증거 재평가 | 감사 완료 · 제품 미인수 | [증거](validation/e024-odb-full-import-acceptance-gate.md) |
| E_025 | IPC-2581C 단일 Revision Layer/LogicalNet SOURCE Inventory 구현·한정 검증 | 한정 완료 | [증거](validation/e025-ipc2581c-layer-net-source-qualification.md) |
| E_026 | KiCad 9 PCB Layer/Via SOURCE 한정 | 한정 완료 | [증거](validation/e026-kicad-layer-via-source-qualification.md) |
| E_027 | Gerber+Drill 한 Layer/홀 분류 부분집합 | 증거 일부 미확인 | [증거](e-stage-closure-plan.md) |
| E_028 | PADS V9.4 / KiCad 9 공통 SOURCE Through Via 1건 | 한정 완료 | [증거](validation/e028-cross-format-canonical-equivalence-qualification.md) |
| E_029 | macOS Qt 타이머 간헐 지연 Native A/B 실험 | 실험 완료 · 원인 미확인 | [증거](validation/e029-macos-qt-timer-causal-qualification.md) |
| E_030 | E 종료 감사·30건 이슈 전체 재분류 | 최종 정산 · PR 병합 후 종결 | [증거](e-stage-closure-plan.md) |

> **E_030은 E 시리즈 마지막 작업입니다.** E 작업별 한정 결과는 전체 제품 인수가 아니며, E_030 PR 최종 보호 병합·사후 확인 뒤에 행정 종료합니다.

<a id="stage-f"></a>
## 3. F 단계 — Native `.bsi` 파일

| 작업 | 핵심 결과 | 상태 | 근거 |
| :--- | :--- | :--- | :--- |
| **F_001** | SQLite 단일 파일·i64/i128 Geometry·52 B BLOB envelope·영구 호환성 계약 | **100% COMPLETE (규격 한정)** · Writer/Reader NOT_RUN | [규격](architecture/f001-bsi-format-contract.md) · [검증](validation/f001-bsi-format-qualification.md) · [PR #647](https://github.com/july-rust/BoardSI/pull/647) |
| **F_002** | SQLite v1 STRICT DDL·BSIG 9종 byte layout·Native SQLite 검증 | **100% COMPLETE (스키마/규격 한정)** · F_003 Writer/Reader NOT_RUN | [스키마](architecture/f002-bsi-sqlite-schema.md) · [바이너리](architecture/f002-bsi-geometry-binary-layout.md) · [검증](validation/f002-bsi-schema-qualification.md) · [PR #649](https://github.com/july-rust/BoardSI/pull/649) |
| **F_003** | Native Rust BSIG 9종·SQLite Writer/Reader·정밀 Round-Trip | **CORE COMPLETE** · 강제 종료/ENOSPC/파일 잠금·10k Geometry PASS · Qt 파일 UI는 F_003_1에서 별도 PASS · 실제 정전 NOT_RUN | [검증](validation/f003-native-writer-reader-qualification.md) · [#651](https://github.com/july-rust/BoardSI/pull/651) · [#653](https://github.com/july-rust/BoardSI/pull/653) |
| **F_003_1** | 메인 Qt Native .bsi Open/Save/Save As·취소 및 Staging/Journal 복구 | **100% COMPLETE (범위 한정)** · 최초 완료/근거일 **2026-11-16 KST** · T01~T12 **12/12 PASS**, Qt **4 OS**, 최종 PR CI **13/13**, 독립 리뷰 0건, Verified Squash | [검증 원장](validation/f003-1-native-qt-recovery-qualification.md) · [#654](https://github.com/july-rust/BoardSI/pull/654) · [#655](https://github.com/july-rust/BoardSI/pull/655) |
| **F_004** | v1 Golden 2종·독립 BSIG Oracle·버전/확장 호환·안전 Migration/Worker·강제 종료 및 ENOSPC | **100% COMPLETE (F_004 범위 한정)** · 최초 완료/근거일 **2026-11-16 KST** · T01~T12 **12/12 PASS** · PR **57/57**, main Push **48/48**, 4 OS + ENOSPC **5/5**, 독립 리뷰 0건, Verified Squash · 미발행 v2 실제 변환 제외 | [검증 원장](validation/f004-bsi-compatibility-qualification.md) · [#658](https://github.com/july-rust/BoardSI/pull/658) · [병합 SHA](https://github.com/july-rust/BoardSI/commit/d594a565e9ccf34de343671ce77be624921a60ea) |
| **F_005** | 지원 PADS/ODB++ CAD→Canonical→`.bsi`→Canonical, 메인 Qt Save/Open 및 Source Fail-Closed | **100% COMPLETE (F_005 지원 범위 한정)** · 최초 완료/근거일 **2026-11-16 KST** · T01~T12 **12/12 PASS** · PR **58/58**, 구현 병합 후 main Push **48/48**, 4 OS Native·Qt, 독립 Codex 0건, Verified Squash · 전체 CAD/제조 연속성/SI 검증 제외 | [검증 원장](validation/f005-cad-bsi-roundtrip-qualification.md) · [PR #660](https://github.com/july-rust/BoardSI/pull/660) · [병합 SHA](https://github.com/july-rust/BoardSI/commit/fff7882840d9db0eefd6bd32c6039ef68e6d075d) · [문서 정산 #661](https://github.com/july-rust/BoardSI/pull/661) |
| **F_006** | KiCad 9 지원 Track/Through Via→Canonical→`.bsi`, 메인 Qt SOURCE Artwork·저장·복원·Fail-Closed | **100% COMPLETE (F_006 지원 범위 한정)** · 최초 완료/근거일 **2026-11-17 KST** · T01~T12 **12/12 PASS** · Native/Qt **4 OS**, 최종 `main` **12/12 워크플로·48/48 검사 PASS**, 독립 Codex 지적 0건 · Verified Squash · 전체 KiCad/제조·SI 제외 | [검증 원장](validation/f006-kicad-native-roundtrip-qualification.md) · [구현 #662](https://github.com/july-rust/BoardSI/pull/662) · [수정 #663](https://github.com/july-rust/BoardSI/pull/663) · [수정 #664](https://github.com/july-rust/BoardSI/pull/664) · [문서 #665](https://github.com/july-rust/BoardSI/pull/665) |
| **F_007** | IPC-2581C 지원 Line/Polyline Copper → Canonical → `.bsi` 및 Main Qt Source Artwork | **IMPLEMENTATION + PR ACCEPTANCE COMPLETE · 2026-11-17 KST** · 4 OS F004/Qt PASS, 전체 PR CI 15/15 PASS, Codex 3건 해결, 보호 Squash `c847132` 병합 · **별도 main Push CI 증거는 조회 불가(미검증)** · 전체 IPC/제조·전기 SI 제외 | [F_007 검증 원장](validation/f007-ipc2581c-native-roundtrip-qualification.md) · [PR #666](https://github.com/july-rust/BoardSI/pull/666) |
| **F_008** | Gerber X2 D03 Copper Flash + Excellon Drill Source Evidence → `.bsi` 및 Main Qt | **100% COMPLETE (F_008 지원 Source 범위 한정)** · 최초 완료/근거일 **2026-11-17 KST** · T01~T12 **12/12 PASS**, 실제 PR **15/15 워크플로 PASS**, 4 OS Native·Qt, 최종 HEAD 독립 Codex 재리뷰 완료·지적 0·미해결 0, 구현/문서 Verified Squash 병합, 최종 `main` **12/12 워크플로 SUCCESS**(28 Job PASS·문서 전용 16 Skip) · 실제 제조 Via/연결·SI 및 U/T 제외 | [검증 원장](validation/f008-gerber-excellon-native-roundtrip-qualification.md) · [코드 #668](https://github.com/july-rust/BoardSI/pull/668) · [문서 #669](https://github.com/july-rust/BoardSI/pull/669) · [최종 Codex](https://github.com/july-rust/BoardSI/pull/668#issuecomment-6935113160) |
| **F_009** | PADS/ODB++ Native `.bsi` 원본 없는 재열기 → 지원 Geometry의 Qt Artwork Replay 및 미지원 Fail-Closed | **IN_PROGRESS · 인수 증거 미확인** · F009-T01~T12/4 OS/Codex/보호 병합·main CI 확인 전 완료 금지 | [검증 원장](validation/f009-native-pads-odb-artwork-replay-qualification.md) |

> F_001 **최초 완료일·근거일: 2026-11-15 KST**. PR #647 최종 HEAD `989d3225` CI 8/8, 독립 Codex 0건, Verified Squash [`e8538172`](https://github.com/july-rust/BoardSI/commit/e8538172f7eb8d1559753fc9888371457026fde5), 정확한 `main` Push 8/8 SUCCESS. 기존 **166/128은 A~D·보안·U/T 집계**이며 F 규격과 제품 인수를 혼동하지 않습니다.

> F_003 **핵심 구현 최초 완료일·근거일: 2026-11-15 KST**. [PR #651](https://github.com/july-rust/BoardSI/pull/651) HEAD `5766601b` 47/47 SUCCESS, 독립 리뷰 스레드 미해결 0. 보호 Squash [main `cdbf57bd`](https://github.com/july-rust/BoardSI/commit/cdbf57bd4c596e2643c78b59eee122accfe123d3) Verified, 정확한 main Push **41/41 SUCCESS**. **완료: Native 저장/복원·BSIG·실행한 Fail-Closed·4 OS 시험.** 물리 전원 차단/디스크 고갈 및 시간·메모리 스트레스·취소/Qt 비차단 인수 미검증(PARTIAL), 정식 제품 U/T 승격 금지. A~D·보안·U/T 역사적 166/128 체크 유지.

> F_003_1 **최초 완료일·근거일: 2026-11-16 KST**. [#654](https://github.com/july-rust/BoardSI/pull/654) · [#655](https://github.com/july-rust/BoardSI/pull/655)에서 실제 메인 Qt `.bsi` 파일 동작, 실 프로세스 취소·타임아웃·지연 응답, 원본 무결성, OS Lock/Journal 복구를 **4 OS 실행 시험**으로 완료. 두 PR의 Verified Squash: [`73ba76df`](https://github.com/july-rust/BoardSI/commit/73ba76df8537d8c9ce22ce9671dcf170222bf0c5) · [`9e0b273a`](https://github.com/july-rust/BoardSI/commit/9e0b273a699674ebfd1e869bb8a75a6aefe9a3a4). 마지막 PR HEAD **13/13 CI**·독립 리뷰 0건; 상세 12개 시험은 [단일 검증 원장](validation/f003-1-native-qt-recovery-qualification.md). 실제 PCB 물리 SI·정전·제품 전체 U/T는 여전히 별도 미인수.

> F_003 **추가 검증일: 2026-11-16 KST**. [PR #653](https://github.com/july-rust/BoardSI/pull/653)에서 OS 4종 프로세스 강제 종료·SQLite 파일 잠금·실제 Linux ENOSPC(16 MiB tmpfs) 및 10,000개 Geometry 읽기/쓰기·바이너리 단독 최고 RSS를 재검증. **실제 전원 차단 및 Qt 파일 UI/이벤트 루프 연계는 미검증**이며, 기존 A~D·U/T 상태는 변동 없음. 상세 수치·CI 증거는 [F_003 검증 원장](validation/f003-native-writer-reader-qualification.md) 참조.

> F_002 **최초 완료일·근거일: 2026-11-15 KST**. [PR #649](https://github.com/july-rust/BoardSI/pull/649) 최종 HEAD `2872f463` 9/9 SUCCESS, 독립 Codex 지적·미해결 0, Native SQLite 네 OS 4/4 SUCCESS. 보호 Squash [main `3108587d`](https://github.com/july-rust/BoardSI/commit/3108587d363f671dfc0dd66fefefcdccabcd255f) Verified, 정확한 main Push 9/9 SUCCESS. **Schema/DDL/Byte Contract 한정 완료**이며 F_003 Native Writer/Reader는 NOT_RUN. A~D·보안·U/T 기존 166/128 체크 수 변동 없음.

<a id="blocked"></a>
## 4. 진행 차단·후속 의무

| 코드 | 핵심 차단 이유 | 소유 이슈 |
| :--- | :--- | :--- |
| EB-01 | ODB++ 실제 Vendor / 독립 동일 Revision Oracle 부재 | [#10](https://github.com/july-rust/BoardSI/issues/10) |
| EB-02 | macOS Qt 콜드스타트 근본 원인 미규명 | [#46](https://github.com/july-rust/BoardSI/issues/46) |
| EB-03 | 고객 GPU·Driver·VRAM·Reset 시험 미인수 | [#165](https://github.com/july-rust/BoardSI/issues/165) |
| EB-04 | PH-08 원본 Gate FAIL | [#16](https://github.com/july-rust/BoardSI/issues/16) |
| EB-05 | 공개 Fixture 권리·Clean Host·배포/기준 PC | [#3](https://github.com/july-rust/BoardSI/issues/3), [#6](https://github.com/july-rust/BoardSI/issues/6), [#7](https://github.com/july-rust/BoardSI/issues/7) |
| EB-06 | 전체 CAD/제조 Via·Stackup·전기 Source 인수 | [#520](https://github.com/july-rust/BoardSI/issues/520), [#532](https://github.com/july-rust/BoardSI/issues/532) |
| EB-07 | M0 U/T 정식 개별 시험·독립 승인 | [#2](https://github.com/july-rust/BoardSI/issues/2) |

[차단 상세 및 재개 조건](e-stage-blocked-register.md). 감사 시작 시 **Open Issue 28개, Open PR 0개**. 과거 #41/#578은 Closed이며, 나머지 제품 이슈는 거짓 종결하지 않습니다.

<a id="stage-history"></a><a id="stage-a"></a><a id="a-history-ledger"></a><a id="stage-b"></a><a id="stage-c"></a><a id="stage-d"></a><a id="stage-security"></a><a id="stage-milestones"></a>
## 5. A~D 작업 이력

| 단계 | 한정 확인된 구현 | 제품에서 남은 부분 |
| :--- | :--- | :--- |
| **A_001~A_036** | PADS V9.4 합성·Qt Preview | 실제 전체 PCB·배포 |
| **B_001~B_009** | Source Stackup/Reference 후보 | Return Path 전체 |
| **C_001~C_025_10** | Copper Cutout·Stitch·Qt Risk | 전역 전기 Net |
| **D_001~D_056_09** | PADS 국소 Graph·다층 Via·ODB++ | 전체 CAD/실물 Open/Short |

**번호별 상세:** [원본 A/B/C/D 전체 색인](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L369). A_001~A_036의 누락된 개별 완료 증거는 추정하지 않습니다. D_018의 docs-only 관리 이력과 C_025 하위 번호도 보존합니다.

<a id="d_056"></a>
### D_056 상세 — 기존 문서 링크 호환

**D_056 범용 입력 정규화는 부분 구현이며 제품 전체 인수는 미완입니다.** [원본 기술 원장](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2459)

- <a id="d_056_01"></a> **D_056_01** · 기존 Source 계약 감사 · [기존 검증 기록](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2492)
- <a id="d_056_02"></a> **D_056_02** · 다층 Copper 접점 후보 · [기존 검증 기록](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2505)
- <a id="d_056_03"></a> **D_056_03** · SOURCE 국소 전기 Graph · [기존 검증 기록](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2526)
- <a id="d_056_04"></a> **D_056_04** · Canonical Padstack·IPC · [기존 검증 기록](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2585)
- <a id="d_056_05"></a> **D_056_05** · 메인 Qt 다층 Via Artwork · [기존 검증 기록](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2605)
- <a id="d_056_06"></a> **D_056_06** · Thermal RT/ST SOURCE·Qt · [기존 검증 기록](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2627)
- <a id="d_056_07"></a> **D_056_07** · 제조 Hole/Slot 증거 · [기존 검증 기록](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2649)
- <a id="d_056_08"></a> **D_056_08** · ODB++ Source Adapter · [기존 검증 기록](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2677)
- <a id="d_056_09"></a> **D_056_09** · ODB++ 제한 범위 종료 · [기존 검증 기록](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2757)

<a id="stage-acceptance"></a>
## 6. 정식 U/T 인수

- **U01~U48:** 1건(U16 제품명 적용) 확인, 47건 정식 인수 미완.
- **T01~T54:** 54건 모두 정식 인수 미완.
- 각 U/T 원문은 [개발 사양 v1.0](specification/BoardSI_Development_Spec_v1.0.md)을 따릅니다.

<a id="check-evidence"></a>
## 7. 체크 항목 — 필요한 단계만 펼치기

기록 형식: **짧은 항목명 / 최초 체크일 / 근거일 / 상세 원본 링크**. 모든 날짜와 원본 PR·CI·SHA는 변경 전 불변 커밋에서 확인할 수 있습니다.

<details>
<summary><strong>A · Preview — 확인 12 / 미확인 5</strong></summary>

- [x] Qt Widgets Preview와 별도 Rust Worker / Versioned IPC의 제한 작동 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-19 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L421)
- [x] 공개 합성 PADS Layout ASCII V9.4 MILS의 명시된 subset만 파싱 및 비지원 입력 거절 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-22 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L423)
- [x] 지원 Straight Trace/Arc/Via/Component·RF Pad 및 Bottom Mirror 변환 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-23 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L425)
- [x] 지원 Board Outline과 다각형·곡선·복합 외곽 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L427)
- [x] 지원 Copper Area/원형·곡선·비직교 외곽, Cutout/Hole와 CPU/RHI 표시 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-25 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L429)
- [x] 원본 Source tick·좌표·원본 개체별 Provenance와 Artwork 독립 검증 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L431)
- [x] 제한 Layer·Net 표시, Filtering, Picking, Zoom/Fit 및 Qt Quick RHI Presented Frame · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L433)
- [x] 원본 정확히 일치하는 일부 Trace/Arc/Via 국소 접점만 그래프 근거 생성 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L435)
- [x] CPU/RHI 국소 접점 마커와 소스 정보/Visible Frame 정합성 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L437)
- [x] 연속 다른 PADS 합성 보드 재Import의 stale 객체·필터 상태 제거 · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L439)
- [x] 10K·100K·1M 합성 객체의 렌더/상호작용 회귀 시험(성능 보증 아님) · **체크일: 2026-10-26 KST** · **근거일: 2026-10-24 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L441)
- [x] Windows x64·Windows ARM64·macOS ARM64 Native CI 시험 환경 범위 PASS · **체크일: 2026-10-26 KST** · **근거일: 2026-10-26 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L443)
- [ ] 임의 상용 PADS 파일·버전의 전체 객체를 자격 검증 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L456)
- [ ] Whole-board Open/Short, Pad/Pin/Pour/Thermal 접속의 물리 판정 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L458)
- [ ] GPU VRAM/driver reset/실장 기기/1M@60 FPS를 정량 측정 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L460)
- [ ] 실제 PCB/Substrate 기반 RLGC/S·DC/30 GHz·Touchstone 검증 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L462)
- [ ] 정식 설치/저사양 실물 PC/재배포 라이선스 검증 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L464)

</details>

<details>
<summary><strong>B · Stackup — 확인 17 / 미확인 6</strong></summary>

- [x] B_001 제한 Qt Stackup/Reference Overlay UI · **체크일: 2026-10-27 KST** · **근거일: 2026-10-26 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L470)
- [x] B_001_2 Rust Canonical Stackup/Material의 제한된 사용자 가정 검증 · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L474)
- [x] B_001_3 제한 PADS 원본 Copper/Net Reference 후보 감사 · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L479)
- [ ] B_001 전체 Issue #12 완료 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L484)
- [x] B_002의 제한적 B_001 Qt 감사 최신성 보강 · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L490)
- [x] B_002 후속 — 제한적 Rust 감사 Scenario Revision 계약 · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L496)
- [x] B_003 지원 PADS V9.4 MILS Trace/Pad/Via 정확 좌표 접촉 그래프 · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L505)
- [ ] B_001 전체 Issue #12 정식 인수 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L514)
- [x] B_004 제한 PADS Source Copper Area 내부 접촉 · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L520)
- [x] B_005 제한 Source-bound Reference Plane Trace Centreline 기하 커버리지 감사 · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L531)
- [x] B_005 post-main Push CI 최종 확인 · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L543)
- [x] B_006 공개 합성 PADS V9.4 MILS Plated Through Via–단일 Straight Hole-free COPCLS Polygon의 제한 국소 접촉 · **체크일: 2026-10-27 KST** · **근거일: 2026-10-27 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L551)
- [x] B_006 protected main post-merge Push CI 및 최종 서명 Verified 확인 · **체크일: 2026-10-28 KST** · **근거일: 2026-10-27 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L560)
- [x] B_007 original PADS single source-owned polygon COPCUT-aware Trace/Pad/Plated Through Via centre local evidence … · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L570)
- [x] B_007 protected main post-merge Checks and Push on main · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L578)
- [ ] B_007 exact latest HEAD independent Codex final-review closure · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L583)
- [ ] B_001 전체 Issue #12 정식 인수 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L589)
- [x] B_008 제한 단일 COPCUT Reference 전체 Centreline 기하 검사 구현 및 보호된 병합 · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L595)
- [x] B_008 보호된 main 병합 후 독립 Push/Native CI 최종 확인 · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L601)
- [x] B_009 실행 전 보호 main 및 체크리스트 감사 · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L611)
- [x] B_009 B_001–B_008 제한 구현 통합 감사·C 단계 인계 및 보호된 main 최종 검증 · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L616)
- [ ] B_007 최종 Source HEAD 독립 Codex 리뷰 종결 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L623)
- [ ] B_001 Issue #12 전체 정식 인수 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L626)

</details>

<details>
<summary><strong>C · Source/Qt — 확인 40 / 미확인 8</strong></summary>

- [x] C_001 제한 다중 Source-owned Straight Polygon COPCUT 및 분리 Copper Island 국소 접점 — 정식 한정 구현 인수 · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L633)
- [x] C_002 Source-backed Reference Gap/Cutout 및 Plated Through Via Transition Risk의 제한 Rust Audit·공개 합성 시험·보호 병합 및 독립… · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L645)
- [x] C_002 이후 Source-bound Qt Reference Split/Layer Transition 위험 표시의 한정 인수 확인 · **체크일: 2026-10-30 KST** · **근거일: 2026-10-29 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L653)
- [ ] B_001 Issue #12 및 정식 U09/U10/U39·T02/T04/T24/T52 전체 인수 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L659)
- [x] C_003 제한 Rust Immutable Scenario/ResultReceipt 계약의 정확한 Source HEAD 시험·독립 리뷰·보호 병합 및 독립 post-main CI · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L665)
- [ ] C_003 이후 실 PCB 완전 Solver Snapshot/Worker transport 및 실제 재사용 캐시 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L675)
- [x] C_004 Canonical Material/Layer의 제한 Rust 입력 계약 구현·최종 코드 PR 검증·보호 병합 · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L682)
- [x] C_004 코드 보호 병합 후 독립 main@02cc35b 검증 · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L689)
- [x] C_005 Source-bound Rust Reference 기하 위험 감사 → Qt CPU/RHI Warning Marker 및 Stale 입력 무효화 · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L699)
- [x] C_006 공개 합성 PADS Source Reference 미피복 구간의 정확한 유리수 교차 매개변수 계산·Qt CPU/RHI 구간 경고 및 Stale 무효화 · **체크일: 2026-10-28 KST** · **근거일: 2026-10-28 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L715)
- [x] C_007 2–16개 Source-order Straight Trace 경로의 Reference Gap Source Geometry 검사 및 Qt CPU/RHI 표시 — 한정 코드·Native 및 병합… · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L748)
- [x] C_008 한정 PADS Source Two-Layer Via 전이 경로·Layer별 Reference Gap 및 Qt CPU/RHI Risk 표시 · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L762)
- [x] C_009 공개 합성 PADS V9.4 MILS 두 Reference Layer의 Source-exact Through Stitch Via 후보 접점 증명·Qt CPU/RHI 위험 마커·Stale 무효… · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L775)
- [x] C_010 공개 합성 PADS V9.4 MILS 지원 Straight Signal 경로의 두 Reference Layer Copper Corridor Source Area와 C_009 Stitch Vi… · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L792)
- [x] C_011 한정 Source-exact 원형 Reference Stitch Via Conductive Pad 전체 면적 / COPCLS·COPCUT 외곽·Cutout 위험 판정 및 Qt CPU/RHI … · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L806)
- [x] C_012 이종 Net Signal/Reference Via 원형 Conductive Pad의 Source Tick 기준 겹침·접촉·분리 판정 및 Qt CPU/RHI 위험 표시 — 한정 Source-g… · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L816)
- [x] C_013 한정 Source COPCUT 직사각형과 Signal Via 원형 Pad Source Tick 간격 및 Qt CPU/RHI 마커 · **체크일: 2026-10-29 KST** · **근거일: 2026-10-29 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L828)
- [x] C_014 Source-owned Reference Area별 독립 직사각형 COPCUT 0–8개/레이어 및 원형 Signal Via Pad Source Tick 간격·Qt CPU/RHI 위험 표시(한… · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L840)
- [x] C_015 원형·직사각형 혼합 Cutout Source Geometry·Qt 독립 V11 한정 인수 완료 · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L854)
- [x] C_016 Source Reference Copper 외곽·COPCUT/COPCCO 대비 Reference Stitch Via 원형 Conductive Pad 전체의 경계·접촉·중첩 위험 및 Qt 독립… · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L868)
- [x] C_017 V1–V4 Source SHA·Gap·Plated Via Risk 한정 통합 회귀 및 기존 Qt Native 증거 정합성 인수 · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L883)
- [x] C_018 V5–V8 Source SHA/Area ID/Pad Footprint/Pad Clearance 한정 통합 회귀 및 Qt Native 기존 증거 정합성 인수 · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L893)
- [x] C_019 V9–V12 Source SHA·COPCUT/COPCCO·Signal/Reference Via Pad 위험 및 기존 Qt Native 회귀 한정 인수 · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L904)
- [x] C_020 B_007 원본 최종 HEAD Codex 상태/리뷰 스레드 및 Legacy B_003–B_008 보호 병합 증거 재감사 · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L917)
- [x] C_020 감사 PR의 정확한 최종 HEAD CI·독립 Codex·보호 Squash·독립 post-main 검증 제한 인수 · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L925)
- [x] C_020 최종 문서 PR #298 독립 최종 HEAD CI·Codex·보호 Signed Verified Squash·post-document-main 인수 · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L934)
- [x] C_022 3~6개 Source-owned Reference Copper 직사각형 간 양의 면적 중첩 전이적 후보 그룹과 분리 Island 위험 표시 · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L947)
- [x] C_022 문서 PR #307 독립 완료 게이트 · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L961)
- [x] C_021 두 Source-owned 직사각형 Reference Area 간 양의 면적 중첩·분리 Island 위험 감사 · **체크일: 2026-10-30 KST** · **근거일: 2026-10-30 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L974)
- [x] C_023 V15 PADS Multi-Area Reference Corridor Coverage/Gap 및 Qt Source-only 위험 표시 · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L992)
- [x] C_024 C_001–C_023 Source Geometry/Qt 통합 회귀 증거 감사 및 D단계 인계 — 한정 감사 인수만 완료 · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1007)
- [x] C_025_05 v1.0 사양서/추적 원장 U/T 정의와 그룹 ID 정합성 재확인(문서 검사만) · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1022)
- [ ] Issue #2 및 M0 F1 정식 종료 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1027)
- [x] C_025_04 PR #153·#223 정책 보존·기능 중복 비교 후 Closed/Unmerged 상태 확인 · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1034)
- [x] C_025_04 정책 문서 보호 병합의 제한적 인수 · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1040)
- [x] C_025_03 구형 PR #115·#117·#118의 근거 기반 분류 및 미병합 종료 · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1050)
- [x] C_025_03 감사 문서 PR 보호 Squash 병합 및 Post-main 검증 — 문서 정리 한정 · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1056)
- [x] C_025_09 기존 Native 실패/성공 근거 확인 및 진단 방향 확정 (현행 로그 읽기 한정) · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1379)
- [x] C_025_09 Qt dispatcher 계측 코드 최종 인수 및 보호된 main 병합 (진단만) · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1384)
- [ ] Issue #46 코드 원인 및 근본 해결 입증 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1391)
- [ ] Issue #41 종료 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1393)
- [x] 두 공개 KiCad 후보의 원본 바이트/SHA-256 및 Git Blob SHA-1 검증 근거 재조정 (기존 실행만) · **체크일: 2026-10-31 KST** · **근거일: 2026-10-21 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1399)
- [x] 두 후보의 제한적 공식 KiCad 외곽/객체 수 Native 증거 재조정 (기존 실행만) · **체크일: 2026-10-31 KST** · **근거일: 2026-10-21 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1403)
- [x] 공개 소스 라이선스 계열 제목 검사 계약 존재 확인 · **체크일: 2026-10-31 KST** · **근거일: 2026-10-21 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1407)
- [ ] Issue #7 F6 공개 Fixture 배포 법적 승인 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1410)
- [ ] Issue #7 F6 실제 약 8 GiB 기준 PC 인수 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1412)
- [ ] Issue #7 전체 DoD 및 정식 T14/T15/T49 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1414)
- [x] C_025_10 보호 브랜치 문서 감사 최종 제한 인수 · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1416)

</details>

<details>
<summary><strong>D · Copper/CAD — 확인 95 / 미확인 8</strong></summary>

- [x] D_002 제한 Source COPCUT-aware 두 Copper Area 접촉 후보 Rust/Qt 코드 및 Native 통합 인수 · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1425)
- [x] D_001 제한 Source-owned SMD Pad/Trace endpoint 후보 코드 및 Native 통합 인수 · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1437)
- [x] D_001 S/RF Pad 회전 형상·중복 소유자 보강 회귀시험과 보호 통합 인수 · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1445)
- [ ] M1 전체 및 물리 전기 접속, P009 인수 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1452)
- [x] D_004 정확히 3개 Source COPCLS/최대 1개 소유 COPCUT와 Straight Trace 끝점 2개의 Source 기하 접촉 후보 연결 및 Qt 독립 재검증 한정 인수 · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1466)
- [x] D_003 정확히 세 개 Source COPCLS 및 최대 한 개 소유 COPCUT의 Source 기하 접촉 후보 그래프 및 Qt 독립 재검증 한정 인수 · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1476)
- [x] D_005 제한 Source 기하 Pad–Trace–Three-Area 접촉 후보 코드·Qt Native 인수 (전기적 연결 인수 아님) · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1488)
- [x] D_005 보호된 코드 병합 후 정확한 main SHA 독립 검증 완료 · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1495)
- [ ] D_005 별도 문서 PR #347 보호 병합 및 Issue #345 종료 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1499)
- [x] D_006 한정 Source 도금 Through Via 원형 Pad–3개 COPCLS/최대 1개 소유 COPCUT 접촉 후보 Rust/Qt Native 코드 인수 (전기 접속 인수 아님) · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1506)
- [x] D_006 병합 후 정확 main@19d3a877 독립 Push CI 최종 인수 · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1513)
- [x] D_006 별도 문서 PR 보호 병합·병합 후 CI 및 Issue #348 종료 · **체크일: 2026-11-01 KST** · **근거일: 2026-11-01 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1517)
- [x] D_007 한정 Source 소유 도금 Through Via 원형 Pad–Straight Trace–서로 다른 COPCLS Area 후보 코드 및 Qt 독립 검증 · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1528)
- [x] D_007 실제 Windows x64·Windows ARM64·macOS ARM64 Qt Native Selftest 보강 및 보호 병합 · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1534)
- [x] D_007 Native 보강 병합 후 정확한 main@aa6ddf744622 독립 Push CI 인수 · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1540)
- [x] D_008 한정 Source 소유 도금 Through Via Pad–서로 다른 두 Copper Layer COPCLS/선택적 COPCUT 기하 후보 구현·검증 · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1549)
- [x] D_008 코드 보호 병합 후 동일 main@f088c5f2... 독립 Push CI · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1555)
- [x] D_008 문서 PR #358 독립 심사·보호 Squash 및 문서 병합 후 main Push CI·Issue #356 종료 · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1559)
- [x] D_009 Source-only dual-layer Through-Via Pad → Layer-B Straight Trace → third COPCLS geometry candidate 구현 및 검증 · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1568)
- [x] D_009 보호 코드 병합 및 독립 post-code-main Push CI 검증 · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1572)
- [x] D_010 제한 Source-only Plated Through-Via / 두 Layer Conductive Pad / Layer별 Trace 2개 / Source COPCLS Area 4개·선택적 C… · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1583)
- [x] D_011 Source-only Contact Candidate Graph 및 Native Qt 검증 · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1597)
- [x] D_011 보호 코드 병합 및 독립 post-code-main Push CI 검증 · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1604)
- [x] D_012 제한 PADS Source Via Span 및 Importer-assumed 도금 Provenance 구현 코드 PR 검증·보호 병합 · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1617)
- [x] D_012 정확한 code-main Push CI 최종 인수 · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1625)
- [x] D_012 별도 문서 PR 및 Issue #368 종료 · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1631)
- [x] D_013 제한 Source-only Legacy Barrel Projection 검증·보호 코드 병합 · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1642)
- [x] D_013 정확 code-main SHA 독립 Push CI 최종 인수 · **체크일: 2026-11-02 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1651)
- [x] D_013 별도 문서 PR 보호 병합·문서 main Push CI·Issue #371 종료 · **체크일: 2026-11-03 KST** · **근거일: 2026-11-02 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1654)
- [x] D_014 제한 2-Via/3-Trace Source-only Contact Candidate Graph 구현 및 보호 코드 병합 · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1663)
- [x] D_014 code-main 정확 SHA 독립 Push CI 최종 인수 · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1670)
- [x] D_014 별도 문서 PR 보호 병합·문서 main Push CI·Issue #374 종료 · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1674)
- [x] D_015 제한 Source-only T-Junction 검증 코드·Native Qt 및 보호 병합 · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1685)
- [x] D_015 code-main 독립 Push CI 최종 인수 · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1693)
- [x] D_015 별도 문서 PR 보호 병합·독립 Push CI·Issue #378 종료 · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1696)
- [x] D_016 limited code verification / protected merge · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1708)
- [x] D_016 code-main exact-SHA independent Push CI · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1714)
- [x] D_016 separate documentation PR, independent doc-main Push CI and Issue #382 state reconciliation · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1718)
- [x] D_017 제한 Source-only 비직각 X-Junction 코드/Qt 검증·보호 병합 · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1731)
- [x] D_017 코드 main 독립 Push CI 최종 검증 · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1739)
- [x] D_017 별도 문서 PR/문서 main Push CI/Issue #386 종료 — 최종 확인 · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1743)
- [x] D_019 비직교 T-Junction Source-only Rust/Qt 구현·보호 병합 한정 인수 · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1754)
- [x] D_020 Source-only 엄격 포함형 공선 Trace 중첩 Rust/Qt 구현·보호 병합 한정 인수 · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1768)
- [x] D_021 Source-only 부분 공선 Trace 중첩 Rust·Qt 검증 및 보호 병합 — 한정 완료 · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1784)
- [x] D_022 동일 층 공선 Trace 끝점과 Via Pad Source 접촉 후보 — 한정 완료 · **체크일: 2026-11-03 KST** · **근거일: 2026-11-03 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1801)
- [x] D_023 Source-only Via-A 공선 Trace 끝점 접촉 후보 — 한정 코드 검증 완료 · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1818)
- [x] D_024 Source-only Via-B 직교 Trace 끝점 접촉 후보 — 한정 코드 검증 완료 · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1834)
- [x] D_025 Source-only Via-A 직교 Trace 끝점 접촉 후보 — 한정 코드·main Push 검증 완료 · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1852)
- [x] D_026 Source-only Via-B 45° 사선 Trace 끝점 접촉 후보 — 한정 코드 및 main Push 검증 완료 · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1868)
- [x] D_027 Source-only Via-A 45° 사선 Trace 끝점 접촉 후보 — 한정 코드 및 main Push 검증 완료 · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1885)
- [x] D_028 Source-only Via-B 4:3 사선 Trace 끝점 접촉 후보 — 한정 코드 및 독립 main Push 검증 완료 · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1904)
- [x] D_029 Source-only Via-A 정확한 4:3 비45° 사선 끝점 접속 후보 — 코드·문서 근거 한정 인수 완료 · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1923)
- [x] D_030 Via-B exact steep 3:4 Source 끝점 접촉 후보 — 코드와 병합 Push 근거 한정 인수 완료 · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1947)
- [x] D_031 Source-only Via-A 정확한 3:4 급경사 비45° 끝점 접촉 후보 — 코드·문서 및 main Push 한정 인수 완료 · **체크일: 2026-11-04 KST** · **근거일: 2026-11-04 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1965)
- [x] D_032 Via-B 음의 기울기 3:4 정확한 Source 끝점 접촉 후보 — 코드·문서 및 main Push 한정 인수 완료 · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1987)
- [x] D_033 Via-A 음의 기울기 3:4 정확한 SOURCE 끝점 접촉 후보 — Rust/Qt·코드-main Push 한정 검증 완료 · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2011)
- [x] D_034 Via-B 음의 기울기 4:3 정확한 SOURCE 끝점 접촉 후보 — Rust/독립 Qt·코드-main Push 한정 검증 완료 · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2031)
- [x] D_035 음의 기울기 4:3 Via-A 정확한 SOURCE 끝점 접촉 후보 — 코드·main Push 한정 검증 완료 · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2053)
- [x] D_036 Via-A/B 양쪽 음의 기울기 4:3 복합 SOURCE 끝점 접촉 후보 — 코드·Verified Squash·main Push 한정 검증 완료 · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2073)
- [x] D_037 Via-A 음의 기울기 4:3 + Via-B 양의 기울기 4:3 단일 PADS SOURCE 복합 접촉 후보 — 코드·Verified Squash·main Push 한정 검증 완료 · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2094)
- [x] D_038 Via-A 양(+) / Via-B 음(-) exact shallow 4:3 단일 SOURCE 복합 접촉 후보 — 코드·Verified Squash·정확한 main Push 한정 검증 완료 · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2116)
- [x] D_039 SOURCE 소유 Same-Layer 2 SMD Pad / 2 Trace 국소 Copper 기하 Graph 및 Fail-Closed 검증 — 코드·Verified Squash·정확한 main… · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2138)
- [x] D_040 SOURCE 소유 2개 독립 동일 Layer Copper Component 구분 및 Fail-Closed 검증 — 코드·Verified Squash·정확한 code-main Push 완료 · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2158)
- [x] D_041 SOURCE-owned 단일 Bridge Trace의 두 독립 Component 국소 병합 및 Fail-Closed 한정 검증 · **체크일: 2026-11-05 KST** · **근거일: 2026-11-05 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2178)
- [x] D_042 SOURCE 소유 동일 Layer 폐회로 및 Redundant Path 국소 Connectivity Graph 한정 검증 — 코드·Verified Squash·정확한 main Push 검증 완료 · **체크일: 2026-11-06 KST** · **근거일: 2026-11-06 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2195)
- [x] D_043 완전 SOURCE 검증 이후 가상 Trace Cut-Set 및 국소 Redundancy Graph 진단 — 코드·Verified Squash·main Push 한정 인수 · **체크일: 2026-11-06 KST** · **근거일: 2026-11-06 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2215)
- [x] D_044 SOURCE 소유 동일 Layer 평행 Trace 선폭 접촉 후보의 정확한 둥근 끝단 접촉 및 Fail-Closed 코드 한정 인수 · **체크일: 2026-11-06 KST** · **근거일: 2026-11-06 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2234)
- [x] D_045 SOURCE 소유 동일 Layer 직교 Trace의 실제 선폭·둥근 끝단 접촉 후보 및 D_044 Native Qt 적대적 재Import 보완 — 한정 코드·Verified Squash 검증 완료 · **체크일: 2026-11-06 KST** · **근거일: 2026-11-06 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2253)
- [x] D_046 비직교·비평행 SOURCE Trace의 Round-Cap 선폭 접촉 후보 — Rust·Native Qt·코드-main 검증 완료. · **체크일: 2026-11-06 KST** · **근거일: 2026-11-06 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2273)
- [x] D_047 SOURCE-owned 동일 Layer Trace-to-SMD Square Pad 유한 선폭 접촉 및 Fail-Closed Rust·Native Qt 독립 검증 · **체크일: 2026-11-06 KST** · **근거일: 2026-11-06 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2294)
- [x] D_048 SOURCE-owned 동일 Layer 원형 SMD Pad–유한 선폭 Trace의 정확한 접촉 후보·Fail-Closed Rust/Native Qt 독립 검증 · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2313)
- [x] D_049 SOURCE-owned Round Pad + Square Pad 두 접촉의 동일 Layer 국소 Copper Geometry Graph 통합 한정 검증 · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2335)
- [x] D_050 SOURCE-owned 동일 Layer 원형·정사각형 SMD Pad 혼합 국소 Component 사이의 유한 선폭 Bridge 접촉 후보 — Rust·독립 Native Qt·보호 code-m… · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2357)
- [x] D_051 공개 합성 PADS V9.4 SOURCE-Layer 1 혼합 Pad 두 Bridge의 중복 국소 경로 및 가상 단선 한정 검증 · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2378)
- [x] D_052 공개 합성 PADS SOURCE의 실제 Midspan Copper Break 4상태 Rust·Native Qt 독립 국소 연결성 검증 · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2397)
- [x] D_053 공개 합성 PADS V9.4 SOURCE 실제 Repair Trace 6상태 Rust·Native Qt 독립 국소 접촉·연결성 검증 · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2418)
- [x] 직선 모든 방향(0–359° 샘플 회귀는 검증용이며 각도 분기 구현이 아님)과 방향성 원호 CW/CCW, 유한 선폭 SOURCE 국소 접촉 공통 계산 및 Fail-Closed 검증 · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2437)
- [x] Trace 중심/종점의 Via·Pad 중심 일치 가정을 제거한 재사용 가능한 Copper 겹침 분류 및 SOURCE Net/Layer/드릴/Shape/접선 Fail-Closed 검증 · **체크일: 2026-11-07 KST** · **근거일: 2026-11-07 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2448)
- [ ] 범용 입력 구조 최종 인수 미완료 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2461)
- [x] D_056_01 기존 구현 감사·범용 PCB Source Evidence Contract 수립 (문서·CI 한정 완료) · **체크일: 2026-11-08 KST** · **근거일: 2026-11-08 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2494)
- [x] D_056_02 — SOURCE 기반 다층 Via/Trace/SMD 국소 Copper 후보 Graph 구현 및 소프트웨어 검증 (한정 완료) · **체크일: 2026-11-08 KST** · **근거일: 2026-11-08 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2507)
- [x] D_056_03 — PADS V9.4 지원 SOURCE 범위 다층 전기 그래프/Net 충돌·Open 후보·Cut-set 알고리즘 한정 인수 완료 · **체크일: 2026-11-09 KST** · **근거일: 2026-11-09 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2528)
- [x] D_056_04 SOURCE 지원 범위 Canonical Padstack/Via → Versioned Rust/Qt IPC V1 독립 소프트웨어 한정 인수 · **체크일: 2026-11-09 KST** · **근거일: 2026-11-09 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2587)
- [x] D_056_05 지원 PADS SOURCE 기반 Canonical Padstack/Via의 실제 메인 Qt CPU/RHI Artwork 통합과 Legacy V1~V14 소프트웨어 한정 코드 인수 완료 · **체크일: 2026-11-09 KST** · **근거일: 2026-11-09 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2607)
- [x] D_056_06 공식 PADS RT/ST Thermal Spoke의 SOURCE 한정 소프트웨어 구현·코드 PR 검증·보호 Squash 반영 완료 · **체크일: 2026-11-09 KST** · **근거일: 2026-11-09 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2629)
- [x] D_056_06 코드·문서 독립 main Push 검증 완료 · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2639)
- [ ] 제조·전체 PCB·SI 제품 인수 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2644)
- [x] D_056_07 PADS PARTDECAL 한정 Source/Canonical 계약과 Legacy IPC 정보 손실 차단 소프트웨어 검증 · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2651)
- [ ] D_056_07 전체 기능 인수 미완료: · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2656)
- [ ] 필수 후속 구현/증거 미확보: · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2661)
- [x] D_056_07 SOURCE 범위 코드·문서 독립 main Push 감사 완료 · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2665)
- [x] D_056_08 ODB++ SOURCE 한정 Parse→Canonical 소프트웨어 검증·보호 병합 · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2679)
- [x] D_056_08 ODB++ 양성 S/OB/OS/OE/SE Copper Area·Cutout SOURCE 한정 구현·보호 Squash 인수 · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2688)
- [x] D_056_08 ODB++ EDA NET/SNT/FID 설계 Net 소유 관계 SOURCE 한정 검증·보호 Squash 인수 · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2696)
- [x] D_056_08 ODB++ Drill MATRIX Layer Span / Tool SOURCE 의도 감사 한정 인수 · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2705)
- [x] D_056_08 ODB++ Drill P Feature Hit→Tool NUM 독립 SOURCE 한정 인수 · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2712)
- [x] D_056_08 Uncompressed USTAR Native Source Reader 한정 인수 · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2720)
- [x] D_056_08 USTAR/Directory Drill Tools·P Hit 단일 SOURCE 스냅샷 한정 인수 · **체크일: 2026-11-10 KST** · **근거일: 2026-11-10 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2729)
- [x] D_056_08 ODB++ Drill P-Hit → 제조 홀 SOURCE 관측형 데이터 계약 한정 인수 · **체크일: 2026-11-11 KST** · **근거일: 2026-11-11 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2736)
- [ ] D_056_08 전체 인수 미완료: · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2749)
- [x] ZIP STORED 컨테이너 SOURCE 한정 파서·CLI/Drill/Hole 공통 파일맵 경로 소프트웨어 인수 · **체크일: 2026-11-11 KST** · **근거일: 2026-11-11 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2759)
- [x] D_056_09 #579 코드-main Push 독립 CI 감사 완료 · **체크일: 2026-11-11 KST** · **근거일: 2026-11-11 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2765)
- [ ] D_056_09 전체 ODB++Design 인수 미완료: · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L2775)

</details>

<details>
<summary><strong>SEC_001 · 보안 — 확인 1 / 미확인 0</strong></summary>

- [x] SEC_001 CodeQL py/bad-tag-filter Alert #1 실제 Fixed 검증 · **체크일: 2026-10-31 KST** · **근거일: 2026-10-31 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L448)

</details>

<details>
<summary><strong>U01~U48 · 요구 — 확인 1 / 미확인 47</strong></summary>

- [ ] U01 — Full-wave 3D EM을 제외한 경량 RLGC·임피던스·S-parameter — **부분 증빙 / 최종 미완료**: 제한 수학/무손실 쿠폰 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1073)
- [ ] U02 — Rust 계산 코어 + Qt UI — **부분 증빙 / 최종 미완료**: Qt/Rust/Worker Preview 시험 PASS, 최종 배포·전 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1075)
- [ ] U03 — Windows x64·ARM64, macOS Apple Silicon — **부분 증빙 / 최종 미완료**: 3개 Native CI 환경에서 P · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1077)
- [ ] U04 — 저사양 PC에서도 가벼운 동작 — **부분 증빙 / 최종 미완료**: 8 GiB Linux 합성 실험, 기준 실물 PC 미검증 ([M0 종료 게 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1079)
- [ ] U05 — 공개 GitHub에서 개발, 사내 데이터 비공개 — **부분 증빙 / 최종 미완료**: 공개 GitHub 운영; 기밀 데이터 무유출 최종 인수는 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1081)
- [ ] U06 — PADS Layout ASCII (*.ASC) 지원 — **부분 증빙 / 최종 미완료**: PADS V9.4 MILS 공개 합성 subset만  · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1083)
- [ ] U07 — 다양한 PCB 형식 확장(ODB++, IPC-2581, KiCad, Gerber 등) — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1085)
- [ ] U08 — 자체 2D Artwork, Layer/Net 확인 — **부분 증빙 / 최종 미완료**: CPU/Qt RHI Preview·Layer/Net 필 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1087)
- [ ] U09 — Stackup 두께, Copper, Dk/Df, Material 편집 — **제한 Qt + 실제 Rust Canonical Assumed Mat · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1089)
- [ ] U10 — GND·Power·Reference 지정 — **제한 Layer 역할 힌트+사용자 Net 선택 및 Import 기반 Copper 후보 음성 판정 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1092)
- [ ] U11 — Single/Differential 선택, 선폭·두께 가상 변경 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1095)
- [ ] U12 — DC~30 GHz 등 사용자 주파수 범위 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1097)
- [ ] U13 — CPU/GPU 등 계산 자원 선택 UI — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1099)
- [ ] U14 — 영어 기본, 한국어 옵션 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1101)
- [ ] U15 — 공개 검증 샘플, 기밀 PCB 오프라인 검증 — **부분 증빙 / 최종 미완료**: 공개 합성 fixture 존재, 기밀 실보드 오프라인 전 경 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1103)
- [x] U16 — 제품명 BoardSI 사용 — **현재 제품명 적용 확인** ([사양서 개정일](specification/BoardSI_Development_S · **체크일: 2026-10-26 KST** · **근거일: 2026-10-14 KST** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1105)
- [ ] U17 — Progressive/Hierarchical Sweep: 초깃값 1 GHz 간격 예시 → 절반 구간 세분화 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1110)
- [ ] U18 — 계산 완료 주파수 샘플 캐시 재사용, 동일 조건 재계산 방지 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1112)
- [ ] U19 — 실행 중 새로운 계산 결과가 나오면 SI 그래프 실시간 갱신 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1114)
- [ ] U20 — Group × Frequency Region별 독립적인 수렴·상태 판정 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1116)
- [ ] U21 — 일부 그룹/대역 수렴만으로 전체 실행 조기 종료 금지 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1118)
- [ ] U22 — 긴 작업 중 GUI 응답·진행률·취소/일시정지 유지 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1120)
- [ ] U23 — 기능별 소형 모듈, UI와 Solver 분리(필요시 별도 프로세스) — **부분 증빙 / 최종 미완료**: 별도 Rust Worker와 vers · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1122)
- [ ] U24 — RAM/SSD 하이브리드 캐시, 체크포인트, 장애 후 복구 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1124)
- [ ] U25 — 분석 전 배선·리턴경로·모델 한계 위험 표시 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1126)
- [ ] U26 — 일반 PCB 외 BGA/FC-BGA Substrate SI 대상 — **부분 증빙 / 최종 미완료**: Canonical/Substrate 일부 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1128)
- [ ] U27 — Substrate Trace, Pad/Land, Via 및 Anti-pad, Plane 포함 — **부분 증빙 / 최종 미완료**: Via/Pa · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1130)
- [ ] U28 — Bump/Cu Pillar/Solder Ball/Wirebond 해석 영역에서 제외 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1132)
- [ ] U29 — Substrate 분석 Port 기준은 양 끝 금속 Pad-to-Pad — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1134)
- [ ] U30 — 검증된 모델 없는 3D 불연속은 정확한 해석인 것처럼 계산 금지 — **부분 증빙 / 최종 미완료**: 미검증 Solver를 Validated로 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1136)
- [ ] U31 — ODB++를 기준 내부 형식으로 사용하지 않고 자체 Canonical Model 사용 — **부분 증빙 / 최종 미완료**: Format-neu · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1138)
- [ ] U32 — Format-specific Importer를 분리하고 동일한 Viewer/Port/Solver 사용 — **부분 증빙 / 최종 미완료**: P · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1140)
- [ ] U33 — Altium, Allegro, EAGLE, EasyEDA, CADSTAR 등 Import 확대 검토 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1145)
- [ ] U34 — Auto Port 후보 생성 + Manual 수정 + 공통 자동 검증 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1147)
- [ ] U35 — Single 기본 2-Port, Differential 기본 물리적 4-Port — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1149)
- [ ] U36 — Reference 도체와 기준 임피던스 분리, Pair Mapping 명시 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1151)
- [ ] U37 — Error Port는 Solver 차단, Warning은 표시 및 확인 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1153)
- [ ] U38 — Format/Version/객체별 지원 수준과 누락 정보 명시 — **부분 증빙 / 최종 미완료**: PADS 제한 지원표·진단, 기타 파일 형 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1155)
- [ ] U39 — Import 원본 불변, 정규화 및 Provenance 보존 — **부분 증빙 / 최종 미완료**: subset Source grid·prove · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1157)
- [ ] U40 — Import할 때 보조 데이터(Netlist/Stackup/Drill) 연계 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1159)
- [ ] U41 — 고정 Linear 직접 Sweep과 수렴 검사의 독립 기준선 — **부분 증빙 / 최종 미완료**: 수학 직접 기준선 일부, 제품 Direct  · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1161)
- [ ] U42 — 전체 행렬 및 NEXT/FEXT의 방향·종단·결합 그룹 고정 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1163)
- [ ] U43 — 실행·수렴·모델·Limit·입력 신뢰도 상태 분리 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1165)
- [ ] U44 — 불변 Run Snapshot, IPC 버전과 늦은 응답 격리 — **부분 증빙 / 최종 미완료**: 제한 immutable Snapshot/Wo · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1167)
- [ ] U45 — 프로젝트 스키마·이전 버전 이관·재개·저장 무결성 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1169)
- [ ] U46 — Sweep만 변경 시 동일 물리 조건 직접 계산점 재사용 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1171)
- [ ] U47 — 수치 정확도·GUI 응답 목표 및 단계별 측정 게이트 — **부분 증빙 / 최종 미완료**: 일부 수치·Native CI; 전체 제품 정량 인수 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1173)
- [ ] U48 — Import 좌표 정밀도·ODB Step·보조 자료 충돌 통제 — **부분 증빙 / 최종 미완료**: PADS 좌표 grid fidelity 일 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1175)

</details>

<details>
<summary><strong>T01~T54 · 시험 — 확인 0 / 미확인 54</strong></summary>

- [ ] T01 — ASC 형식·버전·단위 — **부분 증빙 / 정식 인수 미완료**: 제한 PADS 버전/Signature fixture ([A_033 원본 정합 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1185)
- [ ] T02 — 필수 물성·기하 누락 — **제한 Qt 필드 검사 PASS / 정식 인수 미완료**: 실제 PCB·Canonical 입력 전체 누락 검증 아님  · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1187)
- [ ] T03 — Layer·Via·Padstack·Copper — **부분 증빙 / 정식 인수 미완료**: 일부 Pad/Via/Copper와 국소 접점만 ([A · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1189)
- [ ] T04 — Reference 분할·전원 분류 — **정식 인수 미검증**: B_001 역할 힌트는 실제 Copper/Net/분할 판정이 아님 ([B_001 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1191)
- [ ] T05 — P/N·Mixed-mode — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1193)
- [ ] T06 — 균일 선로 수치 정확도 — **부분 증빙 / 정식 인수 미완료**: 합성 수학/무손실 쿠폰 ([M2 검증 원장](validation/README · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1195)
- [ ] T07 — 손실·수동성·상호성 — **부분 증빙 / 정식 인수 미완료**: 무손실 기준 검사만; 손실 미검증 ([M2 검증 원장](validation/RE · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1197)
- [ ] T08 — Scenario·다중 파일 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1199)
- [ ] T09 — DC·Log·주파수 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1201)
- [ ] T10 — 자원·취소·복구 — **부분 증빙 / 정식 인수 미완료**: 비차단 Worker/Qt 일부; 제품 Crash 복구 미완 ([M0 종료 게이트]( · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1203)
- [ ] T11 — GPU 선택 기능 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1205)
- [ ] T12 — 프로젝트·Export — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1207)
- [ ] T13 — 3 플랫폼과 언어 — **부분 증빙 / 정식 인수 미완료**: 3 Native CI 시험; 실제 배포·언어 기능 미완 ([A_035 감사](va · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1209)
- [ ] T14 — 오프라인·기밀 보호 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1211)
- [ ] T15 — 패키지 일관성 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1213)
- [ ] T16 — ASC·ODB++ 등 동일 보드 Cross-Format — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1218)
- [ ] T17 — 동일 확장자 다른 형식 — **부분 증빙 / 정식 인수 미완료**: 지원 PADS Header 구분, 전체 .brd/.asc 형식 아님 ([A_ · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1220)
- [ ] T18 — Gerber Only Import — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1222)
- [ ] T19 — Gerber+Drill+IPC-D-356B+Stackup — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1224)
- [ ] T20 — 미지원/유실 객체 강제 입력 — **부분 증빙 / 정식 인수 미완료**: 제한 비정상 입력 Fail-closed; 전체 객체 분류 미완 ([A_ · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1226)
- [ ] T21 — Canonical Model 안정성 — **부분 증빙 / 정식 인수 미완료**: Canonical synthetic 구조검증; Cross-for · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1228)
- [ ] T22 — Import 압축/비정상 파일 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1230)
- [ ] T23 — 수동 Port Copper 외부 클릭 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1232)
- [ ] T24 — GND Plane Cutout/분리 — **정식 인수 미검증**: 물리 Return 경로 검증 미실시 ([B_001 원장](validation/ · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1234)
- [ ] T25 — Pair P/N 역전/Port 번호 변화 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1236)
- [ ] T26 — Branch/Series R·C 포함 신호 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1238)
- [ ] T27 — Reimport 후 Port 이동/Net 변경 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1240)
- [ ] T28 — Near/Far 4-Port 행렬 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1242)
- [ ] T29 — 단순 Microstrip·Stripline 1차 Sweep — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1247)
- [ ] T30 — Group×Region 독립 수렴 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1249)
- [ ] T31 — 노치·급변 응답 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1251)
- [ ] T32 — Abort/Pause/Resume/Crash Restart — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1253)
- [ ] T33 — Stackup/Port/Load 변경 후 캐시 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1255)
- [ ] T34 — GUI 스트레스(무거운 Import/해석) — **부분 증빙 / 정식 인수 미완료**: Artwork UI/Viewer 스트레스 일부 ([A_0 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1257)
- [ ] T35 — RAM 부족 및 SSD Spill — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1259)
- [ ] T36 — Substrate Pad-to-Pad 모델 경계 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1261)
- [ ] T37 — Microvia/IVH/Plane Anti-pad — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1263)
- [ ] T38 — 3D 구조의 모델 미지원 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1265)
- [ ] T39 — 세 플랫폼 결과·Import 재현성 — **부분 증빙 / 정식 인수 미완료**: 3 Native subset 실행만; 실 PCB 동일 S 행렬  · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1267)
- [ ] T40 — 정식 공개 릴리즈 산출물 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1269)
- [ ] T41 — 고정 Direct와 Progressive 교차 비교 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1274)
- [ ] T42 — NEXT/FEXT와 결합 그룹 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1276)
- [ ] T43 — 비수동·비상호·특이행렬 입력 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1278)
- [ ] T44 — 불균등 격자·부분 Export·한글 Net — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1280)
- [ ] T45 — 완료·수렴·모델·Limit 상태 조합 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1282)
- [ ] T46 — 늦은 IPC·중복 결과·편집 중 Run — **부분 증빙 / 정식 인수 미완료**: Worker stale reply 격리 일부; 제품 전체 R · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1284)
- [ ] T47 — 프로젝트 이관·동시 저장·경로 변경 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1286)
- [ ] T48 — 세분화·Region 경계·재개 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1288)
- [ ] T49 — 진단 출력·캐시·공개 패키지 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1290)
- [ ] T50 — UI·Heartbeat·Cancel 시간 계측 — **부분 증빙 / 정식 인수 미완료**: Qt 타이밍 회귀 일부; 원인/출시 목표 미확정 ([ · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1292)
- [ ] T51 — 미세 Substrate·ODB Step·극성 — **부분 증빙 / 정식 인수 미완료**: PADS source tick 좌표 확인; ODB St · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1294)
- [ ] T52 — 치수·물성 범위와 외삽 — **제한 Rust Canonical Layer z/두께 및 Material 유효 대역 차단 PASS / 전체 인수 미 · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1296)
- [ ] T53 — DC 미지원·깊은 노치·긴 지연 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1299)
- [ ] T54 — SSD 부족·쓰기 중 강제 종료 — **정식 인수 미검증** · [원문](https://github.com/july-rust/BoardSI/blob/d2e99bfde2c3bbd98fa6655a15ab8d1ad7731cf3/docs/development-checklist.md#L1301)

</details>

<a id="update-policy"></a><a id="documentation-ci-evidence"></a>
## 8. 일정한 문서 양식 유지

1. **현재 체크리스트는 이 파일 하나.** 챙이 GitHub PR로 갱신합니다.
2. 업무 요약은 위 표의 **해당 행 한 줄**만 고칩니다. 새 장문 보고서를 앞부분에 누적하지 않습니다.
3. 모든 신규 `[x]`는 **체크일·근거일(KST)**과 PR/SHA/원본 테스트를 명시합니다. 과거 최초 날짜를 고치지 않습니다.
4. 상세 실행·실패·리뷰·성능 자료는 [검증 보고서 색인](validation/INDEX.md)과 원본을 사용합니다.
5. Docs-only Native `SKIPPED`는 PASS가 아닙니다. PR HEAD, 보호 Squash, 독립 main Push를 구분합니다.

**E_030 (2026-11-15 KST):** 기존 체크 상태(166/128)와 최초 KST 체크 날짜를 유지하고 장문 중복 서술을 짧은 현황판·접힌 근거로 교체했습니다.
