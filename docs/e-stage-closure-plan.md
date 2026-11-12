# BoardSI E 단계 — 잔여 작업 종결 계획

> 기준: **2026-11-11 KST**. 이 문서는 E_001 실행 전 작성한 계획을 유지하며, 아래에 E_001의 검증된 종결 이력도 추가한 **계획·인수 경계 설명서**다. 체크 상태의 유일한 정본은 [단일 개발 체크리스트](development-checklist.md)다. 진행 불가·외부 근거 부족은 [별도 의존성 원장](e-stage-blocked-register.md)에서 관리한다.

## 목적

**D 실행은 D_056_09를 끝으로 종료한다.** 기존 체크리스트에 남은 D_056_10~13은 과거에 만든 *후속 예정 번호*라는 역사적 기록만 보존한다. 실제 신규 실행은 E_001부터 시작한다. D_056 전체 기능이나 ODB++ 전체가 인수된 것으로 표기하지 않는다.

2026-11-11 E 계획 수립 당시 GitHub에서 **열린 이슈 30건·열린 PR 0건**을 확인했다(역사적 스냅샷). [D_056_09 관련 #578](https://github.com/july-rust/BoardSI/issues/578)은 여전히 Open이며, ZIP STORED 부분 기능의 코드 병합 [#579](https://github.com/july-rust/BoardSI/pull/579)은 전체 CAD/Qt/전기/SI 인수가 아니다.

**원칙:** 큰 상위 Issue를 하나의 E 작업으로 몰아넣지 않는다. 한 번호는 한 가지 검증 가능한 완결 산출물만 포함한다. 조건을 못 채우면 완료로 꾸미지 않고 정확한 차단 근거와 재개 조건을 기록한다. 닫을 수 없는 상위 이슈는 정상적인 추적 이슈로 남기되, **시작한 E 단위의 PR과 증거 정리는 반드시 마무리**한다.

## 각 E 번호의 공통 완료 정의

1. 시작 전: 최신 main SHA, 열린 이슈/PR, 원본 규격·Source 문법, 기존 A~D 테스트와 실제 미지원 범위를 확인한다.
2. 구현: **하나의 소형 기능/문서 감사**, 정/부정/UNKNOWN 반례, 같은 범위의 자동 검증을 완료한다. 임의 PCB 층수·비아 위치별 분기를 추가하지 않고 범용 Source/Canonical 알고리즘을 우선한다.
3. 증거: 정확한 **PR 최종 HEAD**의 필수 Checks/독립 리뷰/미해결 Threads 0을 확인하고, 기존 보호 규칙을 우회하지 않는다. **GitHub Verified Squash-only** 병합, **정확한 새 main SHA의 Push CI**를 PR CI와 구분하여 확인한다.
4. 문서: 해당 소형 작업에서 **하나의 체크리스트**와 직접 관련 문서를 함께 갱신한다. 작업 종료 뒤 문서 전용 미완 PR을 남기지 않는다. 최초 체크일·근거일·SHA·한계는 삭제하지 않는다.
5. 종료 판정: E 번호가 정한 **소형 범위**의 전 과정을 종료했을 때만 완료. 상위 Issue의 **전체** 수락 조건이 모두 충족될 때만 Issue Closed. 이슈를 닫으려고 근거 없는 PASS를 만들지 않는다.
6. 장애: 독립 Fixture·실기기·라이선스·물리 기준 등 필수 전제가 부재하면 [차단 원장](e-stage-blocked-register.md)에 상태와 다음 조건을 정리한다. 미완 기능/미통과 수락시험을 완료라고 쓰지 않는다. 기존 PR이 유효하지 않으면 리뷰 근거를 보존해 안전 종료하고 후속 새 단위로 재계획한다.

## E_001 실행 기록 — 2026-11-11 KST

- **단위 종결:** PH-07의 #41 사후 감사 자체는 이전 증거와 현재 Ruleset 검증으로 **한정 완료**. [독립 감사 근거](validation/e001-ph07-post-audit-closure.md), [PR #583](https://github.com/july-rust/BoardSI/pull/583), [단일 체크리스트](development-checklist.md).
- **서로 다른 의무 분리:** #43 보호 규칙 조치는 완료; #46 Qt 타이머 원인 분석은 **여전히 미해결**이며 [EB-02](e-stage-blocked-register.md)와 E_029에만 인계. #41 종결은 #46의 기술적 해결을 뜻하지 않는다.
- **증거 경계:** 기존 PH-07 정적 문서·과거 시험 결과는 기록으로 확인했으나 E_001에서 FEM/FVM을 신규 실행했다고 주장하지 않는다. 과거 원본 FAIL, PH-08 Gate Q FAIL, 정식 제품 U/T 미인수는 그대로 유지한다.
- **운영 완료 관문:** PR #583의 최종 HEAD, 리뷰 스레드, Verified Squash, main Push CI 증거는 [#41 종결 댓글](https://github.com/july-rust/BoardSI/issues/41)에 확정 기록한다.

## E_003 실행 증거 경계 — 2026-11-11 KST

- **단위 목적:** 기존에 C_025_08에서 확정한 Rust/Qt/IPC 프로토타입 선택을 재설계하지 않고, 추가 Fail-Closed Wire 회귀를 Rust/Qt 양쪽에서 실행합니다. [검증 보고서](validation/e003-rust-qt-ipc-compatibility-qualification.md)와 [PR #585](https://github.com/july-rust/BoardSI/pull/585)를 사용합니다.
- **상위 이슈 분리:** [Issue #3](https://github.com/july-rust/BoardSI/issues/3)은 제품 정식 T13/T39/T46, 라이선스, 배포 등 본래의 수락 기준을 충족하지 않았으므로 별도 OPEN입니다. [#6](https://github.com/july-rust/BoardSI/issues/6)과 [#7](https://github.com/july-rust/BoardSI/issues/7)은 E_006/E_007에서 각각 처리합니다.
- **작업 완료 관문:** PR 최종 HEAD의 관련 Rust·Qt native 실행, 필수 CI, 독립 리뷰와 미해결 Threads 0, 보호 Verified Squash 및 정확한 main Push 확인 시에만 E_003 제한 완료가 유효하며 상세 운영 증거는 #3의 최종 댓글을 참조합니다.

## E_004 실행 종결 기록 — 2026-11-11 KST

- **한정 수락 완료:** 원본 Source SHA 및 Layer ID 바인딩, 임의 2/4/6/8/64층 Copper envelope(전기 연결 아님), Via Span 오류 차단, Plating Unknown/SourceDeclared 보존, Copper 사이 Dielectric의 z/두께/Material 누락·가정 상태를 확인하는 **비승격 Source 관측 계약**. [검증 보고서](validation/e004-canonical-layer-via-source-qualification.md).
- **코드·리뷰:** [PR #586](https://github.com/july-rust/BoardSI/pull/586) 최종 HEAD `85996aabb603fa2b63af098a9284e9a0a9e4eee9`, **38/38 Checks SUCCESS**, 독립 리뷰 지적 0·미해결 Thread 0. 신규 합성 회귀 6개 Debug/Release PASS; 리뷰에서 발견된 중간 Dielectric 누락 결함을 수정하고 음성 반례로 보호.
- **보호 병합:** Verified 단일 부모 Squash [`de87983a6af2f55423a49881eaacae690fe3b50f`](https://github.com/july-rust/BoardSI/commit/de87983a6af2f55423a49881eaacae690fe3b50f). **정확한 code-main SHA 독립 Push CI 8/8 Workflow SUCCESS**, 실패 0.
- **상위 인수 분리:** [#4](https://github.com/july-rust/BoardSI/issues/4)와 [#520](https://github.com/july-rust/BoardSI/issues/520)은 전체 CAD/제조 도금/전기 Graph/실물·제품 T21/T51이 별도이므로 **계속 OPEN**. E_019/E_022 및 형식별 후속 단계의 조건을 앞당겨 PASS 표시하지 않는다. E_004 자체에 미완 PR은 없다.

## E_005 한정 종결 기록 — 2026-11-11 KST

- **단위 목표 완료:** Rust `RunGuard`와 실제 Qt `Window/QProcess`에서 `CancelAcknowledged` 이후 늦은 합성 `Result`/`Progress`를 Fail-Closed 처리하고 이전 세션 응답·재실행 오염을 방지. 첫 원인 오류 메시지를 OS 종료 과정에서도 보존하며 ACK 전 비행 중 결과, Heartbeat, Cancelled, 실제 OS 종료의 기존 의미를 유지.
- **최종 코드/검증:** [PR #588](https://github.com/july-rust/BoardSI/pull/588), 최종 HEAD `a4c935ed74e4ce60fe1f4678a4dbbdb4a6f0175b`, Native 포함 **37/37 Checks SUCCESS**, 독립 코드 리뷰 지적/미해결 0, Verified Squash [`c229eccd`](https://github.com/july-rust/BoardSI/commit/c229eccdbbba41af5f4c4d622c7a1bb6421bdf02), 정확한 코드 `main` Push **8/8 Workflow SUCCESS**. [상세 검증 기록](validation/e005-worker-cancellation-stale-timeout-qualification.md).
- **종결 경계:** **E_005만 한정 COMPLETE**. [상위 Issue #5](https://github.com/july-rust/BoardSI/issues/5)의 제품 Run Snapshot, 실제 PCB 고부하 Scheduler, T10/T34/T46/T50 전체 및 실제 SI Physics는 별도 OPEN. 코드 PR은 병합되어 미완 상태가 아니며 정식 제품 검증을 앞당겨 PASS하지 않는다.

## E_006 Native Toolchain 근거 감사 — 2026-11-11 KST

- **기술 범위 완료:** [독립 E_006 검증 보고서](validation/e006-native-platform-toolchain-qualification.md)는 Windows x64/ARM64 및 macOS ARM64의 실제 Rust·Qt Widgets/QProcess synthetic native 실행을 각각의 SHA·Workflow·Job까지 검증했습니다. 새로운 기능/검사 프레임워크는 추가하지 않았습니다.
- **소스 동일성 검증:** 최신 감사 당시 `main@400fe8e`은 Native가 실제 실행된 `main@c229eccd`보다 문서 5개만 변경했습니다. [Qt 정확한 code-main Push](https://github.com/july-rust/BoardSI/actions/runs/42982428188)와 [Rust 정확한 code-main Push](https://github.com/july-rust/BoardSI/actions/runs/42982428141)의 3+3개 Native Job 성공이 직접 증거이고, 최신 docs-only Push의 SKIPPED Job은 새 Native PASS가 아닙니다.
- **F7 근거 별도:** [PR F7 Relocation](https://github.com/july-rust/BoardSI/actions/runs/42980487868) 세 Native Job은 PR 자동 Merge Checkout `3ddd02b5`에서 실행되었고 Git Tree SHA가 PR HEAD `a4c935ed` 및 코드-main `c229eccd`와 동일함을 재검증했습니다([E_006 상세 근거](validation/e006-native-platform-toolchain-qualification.md)). SDK 분리 후 synthetic selftest 성공은 clean-host 설치, 법적 라이선스 및 제품 배포 승인을 의미하지 않습니다.
- **운영 완료 경계:** E_006 문서 PR의 최종 필수 CI·리뷰·보호 Verified Squash 및 새 정확한 main Push 결과가 모두 확인되어야 E_006 COMPLETE로 종결합니다. [Issue #6](https://github.com/july-rust/BoardSI/issues/6) 전체는 F7/정식 T13/T39 및 설치 검증 누락으로 계속 OPEN; E_007 라이선스/Fixture 별도. 제품 전체 인수로 승격 금지.

## E_008 PADS 입력 판별·Source Readiness 실행 기록 — 2026-11-12 KST

- **작은 단위 범위:** PADS V9.4 MILS 정확한 Header/지원 버전 판별, 8 MiB 상한의 opt-in ROUTE→Canonical 제한 파서 검증, G/N/P/S/R/M Source 근거/UNKNOWN 보존, 악성·누락·미지원 입력 회귀. [기술 보고서](validation/e008-pads-readiness-qualification.md).
- **기존 기능 호환:** `boardsi-import-probe FILE`의 514-byte Signature Probe 동작을 유지. 신규 `--readiness`는 Source-only PARTIAL 판정이며 물리 Solver·Global Connectivity·새 포맷 문법·Qt UI 기능을 추가하지 않는다.
- **정식 인수 경계:** [Issue #8](https://github.com/july-rust/BoardSI/issues/8)은 U38–U40/U48, 실제 Vendor·형식 전반·Qt 실패 경로 및 정식 T17/T20/T22에 대한 근거가 없으므로 OPEN. E_009, E_013–E_015의 후속 원래 범위 유지.
- **운영 종료:** E_008 PR의 정확한 최종 HEAD 필수 Checks, 독립 리뷰, 0 unresolved threads, Verified Squash 및 정확한 main Push 결과가 확인되어야 **E_008 한정 COMPLETE**로 기록. 증거 URL·SHA는 [#8 종료 댓글](https://github.com/july-rust/BoardSI/issues/8)에 기재한다.

## E_009 PADS ROUTE glued-via grammar one-slice execution — 2026-11-12 KST

- **Selected actual parser gap:** Authoritative PADS V9.4 ROUTE FLAG bit `0x0800` marks a glued named via; the restricted importer previously rejected/skipped this variant even when the via was already declared and normalized.
- **Implementation boundary:** Accept a documented glued-via SOURCE annotation only on a validated named-via vertex. Preserve geometry plus explicit `ViaInstance.glued`, Canonical source provenance and source conditions, but do not claim placement lock, manufactured connectivity, real exporter coverage or SI.
- **Fail-closed:** Unsupported 0x0800 extensions, missing names/definitions and malformed records cannot be silently converted to copper. Both legacy and generalized parser modes plus 2/4/6/8 layers receive focused synthetic regressions.
- **Code evidence:** [PR #593](https://github.com/july-rust/BoardSI/pull/593) exact HEAD `d952110ea855d07c8c67219e466a1d0cf10373f0` received **30/30 Checks SUCCESS**, Codex final-HEAD review **COMPLETED (0 findings/0 unresolved)** and a protected [Verified single-parent Squash `bbc9c0ffecff2871fd91009bbe1e3ce8e804a920`](https://github.com/july-rust/BoardSI/commit/bbc9c0ffecff2871fd91009bbe1e3ce8e804a920). [E_009 qualification report](validation/e009-pads-glued-via-grammar-qualification.md) records restricted acceptance. Independent exact merged-main [Push workflow evidence](https://github.com/july-rust/BoardSI/commit/bbc9c0ffecff2871fd91009bbe1e3ce8e804a920) is **8/8 Workflow SUCCESS and 24/24 Job SUCCESS (0 skipped/failure/cancelled)** for code-main `bbc9c0ff` as of 2026-11-12 KST. **E_009 bounded COMPLETE**; parent #9 and its original U/T acceptance remain OPEN.

## E_010 Main Qt Import→IPC→Picking/ReImport bounded closeout — 2026-11-12 KST

- **One supported public PADS V9.4 Through Via:** actual main `PreviewImportPanel` → Rust Canonical IPC → `ImportedArtworkPanel` via `qt-e010-main-canonical-picking-reimport`. Verify source SHA, accepted scene copper annulus vs nonconductive drill void, Layer/Net filter, SOURCE-ID and Properties, same-controller distinct NET_A→NET_B reImport, unsupported-header fail-closed.
- **Historic failure retained:** first scene selection incorrectly targeted the synthetic tab, then PADS MILS input units were confused with Canonical centimil source ticks. Both corrected; no old FAIL relabeled PASS. Prior macOS #46 event-loop gap >1500 ms failed an earlier exact-head attempt; its failed job rerun passed, but #46 root cause remains UNRESOLVED and belongs to E_029.
- **Code gates COMPLETE:** [PR #595](https://github.com/july-rust/BoardSI/pull/595) exact HEAD `50dedc2e2659381bbeb0a7d53fef84c390e19267` finally all required workflows passed; final HEAD independent Codex found no major issues and 0 unresolved threads; protected Verified single-parent Squash [`0bc459ecfc5419c3eaef420c8f48dfa59fffa87e`](https://github.com/july-rust/BoardSI/commit/0bc459ecfc5419c3eaef420c8f48dfa59fffa87e); exact code-main Push **8/8 workflows SUCCESS**, including all native Qt targets. [Qualification report](validation/e010-main-qt-import-picking-qualification.md).
- **Product boundary:** E_010 software subset only; full [Issue #11](https://github.com/july-rust/BoardSI/issues/11) still OPEN. GPU hardware, real vendor PCB, global Open/Short, RLGC/S and U08/U22/U31/T03/T34 remain unaccepted. Docs-only PR #596 follows separately.

## E_011 Main Qt Canonical Stackup lifecycle — 2026-11-12 KST (code-main Push verified; docs closeout #598)

- **Narrow contract:** actual `PreviewWindow` accepts public 4-layer PADS V9.4 Canonical Source only after imported SHA/path/Layer count are pinned. B_001 local user Stackup JSON v1/v2 can be saved and explicitly restored; missing Dk/Df/thickness/material stay UNKNOWN until manually supplied as Assumed. A new Canonical PCB action preserves the existing legacy importer path.
- **Actual regression:** `qt-e011-main-stackup-persistence-reimport` exercises Rust importer and Stackup Validator through the real main Qt Controller, unknown Source, user edit/save/load/re-audit, malformed JSON, negative Df, same-layer-count different-SHA board, Source A→B→A reImport, on-disk mutation and unsupported header. Legacy B_001/B_002/E_010 regressions retained.
- **Verified code gates:** [PR #597](https://github.com/july-rust/BoardSI/pull/597) final HEAD `f1ae06e580d38ac9ee01f51926b024566542af6d` **35/35 Checks SUCCESS** (earlier unrelated D006 `D006_FILE` FAIL retained, same-head failed-job rerun succeeded), independent Codex 0 major findings/0 unresolved threads, [protected Verified single-parent Squash `da880d29`](https://github.com/july-rust/BoardSI/commit/da880d29a4ada85f7f1da77b744a3700b64c1e48). Exact code-main event=push [workflow evidence](https://github.com/july-rust/BoardSI/commit/da880d29a4ada85f7f1da77b744a3700b64c1e48/checks) **8/8 workflows SUCCESS, 28/28 checks SUCCESS** (Linux/Windows x64/Windows ARM64/macOS ARM64 Qt included). Final [docs PR #598](https://github.com/july-rust/BoardSI/pull/598) required CI, same-head independent review, protected Verified Squash and exact docs-main Push are the remaining bounded E_011 documentation gates.
- **Unaccepted product scope:** [#12](https://github.com/july-rust/BoardSI/issues/12) remains OPEN. Real material provenance, full Source layer mapping, electrical Copper/Reference continuity, P009, physical Scenario/cache, RLGC/S, formal U09/U10/U39 and T02/T04/T24/T52 are not qualified. [Evidence report](validation/e011-stackup-edit-persistence-reimport-qualification.md).

## E_012 Native GPU hardware evidence audit — 2026-11-12 KST

- **Bounded evidence established:** Already completed [exact code-main native Qt run #43150028282](https://github.com/july-rust/BoardSI/actions/runs/43150028282) (`da880d29`) has macOS ARM64 Metal and Windows x64/ARM64 D3D11 real-device checks, 10k/100k/1M synthetic frame receipt/interval data and native interaction/fallback results. This is historical log/Artifact audit, **not a new GPU run**.
- **Source exactness:** The later main `1f3cf704` differs from measured code-main only in four documentation files; docs-only Native SKIPPED is not counted as a GPU PASS.
- **Limits:** 1M macOS p50=210 ms (60 FPS **not verified**), GPU VRAM not measured, physical driver reset not run, detailed adapter/driver identifiers not recorded, full [#165](https://github.com/july-rust/BoardSI/issues/165) OPEN. [Validation detail](validation/e012-actual-hardware-gpu-qualification.md); remaining customer-hardware gate [EB-03](e-stage-blocked-register.md).
- **Closeout:** E_012 bounded COMPLETE requires this documentation PR's exact final HEAD required CI/independent review/unresolved threads 0, protected Verified Squash and exact merged-main Push success. E_013 does not depend on closing #165.

## 작은 단위 실행 순서 — E_001 ~ E_030

**각 행은 단일 작업에서 닫을 수 있도록 설계한 범위**다. 실제 실행 시 더 큰 수정이 필요하면 그 번호의 작은 수락 계약만 끝내고, 나머지를 새 번호로 분리한다. 독립 E_001–E_012는 실행 가능성이 높은 항목 우선 선택 가능하다.

| 번호 | 단일 완료 산출물 | 관련 열린 이슈 | 해당 단위 종료 기준 |
| :--- | :--- | :--- | :--- |
| **E_001** | PH-07 사후 감사 증거 재검토와 안전한 #41 종결 판단 | [#41](https://github.com/july-rust/BoardSI/issues/41), [#46](https://github.com/july-rust/BoardSI/issues/46) | 필수 규칙 적용·후속 #46 구분·과거 PASS/FAIL 보존; 조건 충족 때만 #41 종료 |
| **E_002** | M0 F1 요구사항 U/T 추적성 누락·충돌 1묶음 정리 | [#2](https://github.com/july-rust/BoardSI/issues/2) | U01–U48/T01–T54 원본·테스트 대응표의 단일 정정과 근거 |
| **E_003** | Rust/Qt/IPC 결정·버전 호환 근거 한 묶음 인수 | [#3](https://github.com/july-rust/BoardSI/issues/3) | 실제 코드·반례·Native 증거로 확인/미확인 분리 |
| **E_004** | Canonical Layer/Via Source provenance 한 계약 검증 | [#4](https://github.com/july-rust/BoardSI/issues/4), [#520](https://github.com/july-rust/BoardSI/issues/520) | 층수 하드코딩 없이 Source 근거·Unknown 보존 회귀 |
| **E_005** | 비차단 Worker의 취소 또는 late-response 한 경로 인수 | [#5](https://github.com/july-rust/BoardSI/issues/5) | 격리·stale 거부·Timeout 반례·적용 Native CI |
| **E_006** | 세 Native OS Toolchain·실행 근거 최신성 감사 | [#6](https://github.com/july-rust/BoardSI/issues/6) | Windows x64/ARM64, macOS ARM64 실제 실행 SHA별 증거 |
| **E_007** | 공개 Fixture 출처·해시·재배포 권한 한 묶음 정합화 | [#7](https://github.com/july-rust/BoardSI/issues/7) · [E_007 보고서](validation/e007-public-fixture-provenance-rights-qualification.md) | 고정 KiCad 소스 2종 원본/해시·부정 회귀·라이선스 원문을 재조정; 제3자 권리/실기기 미승인은 별도 BLOCKED, E_007 기술 범위와 #7 전체 인수 분리 |
| **E_008** | 입력 판별 및 readiness 한 Source 종류 보강 | [#8](https://github.com/july-rust/BoardSI/issues/8) | 미지원/누락/악성 소스 반례와 UNKNOWN 보존 |
| **E_009** | PADS V9.4 ROUTE Glued Via 0x0800 Source 플래그 한정 지원 | [#9](https://github.com/july-rust/BoardSI/issues/9) · [E_009 보고서](validation/e009-pads-glued-via-grammar-qualification.md) | 공개 공식 V9.4 pp.51–55 · 2/4/6/8층·malformed/UNKNOWN 회귀 · final CI/리뷰/Squash/main Push 후 한정 COMPLETE. 전체 #9 OPEN |
| **E_010** | 실제 메인 Qt Canonical Through Via Import/IPC/CPU Picking/재Import/Fail-Closed 한 종류 회귀 | [#11](https://github.com/july-rust/BoardSI/issues/11) · [E_010 보고서](validation/e010-main-qt-import-picking-qualification.md) | **한정 COMPLETE:** PR #595 exact HEAD CI/독립 리뷰·Verified Squash `0bc459ec`·정확 code-main Push 8/8 SUCCESS. 실제 GPU 하드웨어/전체 #11은 OPEN |
| **E_011** | Stackup 편집/저장/재Import 무효화 한 경로 검증 | [#12](https://github.com/july-rust/BoardSI/issues/12) · [검증 보고서](validation/e011-stackup-edit-persistence-reimport-qualification.md) | **한정 COMPLETE (docs #598 최종 CI·리뷰·병합·Push 확인 조건):** PR #597 HEAD 35/35, Codex 0/0, Verified Squash `da880d29`, 정확한 code-main Push 8/8 Workflow·28/28 Checks SUCCESS. Docs Closeout #598 검증 후 확정; Source Dk/Df/두께 Unknown 보존, 상위 #12 OPEN |
| **E_012** | 실제 GPU 백엔드 1개 플랫폼 Native 원본 실행 계측 감사 | [#165](https://github.com/july-rust/BoardSI/issues/165) · [검증 기록](validation/e012-actual-hardware-gpu-qualification.md) | **기술 근거 확인:** code-main `da880d29` Metal/D3D11 Native 실제 adapter 확인 3 플랫폼·10k/100k/1M 프레임 원본. 보호 문서 PR 완료 후 한정 COMPLETE; VRAM/1M@60 FPS/실제 device loss 미인수, #165 OPEN |
| **E_013** | ODB++ ZIP DEFLATE Method 8 한 경로 구현 | [#578](https://github.com/july-rust/BoardSI/issues/578) | CRC·크기·ZIP 폭탄·Traversal 반례 및 STORED 회귀 |
| **E_014** | ODB++ tar.gz/tgz 한 Source FileMap 경로 | [#578](https://github.com/july-rust/BoardSI/issues/578) | Decompress 제한·USTAR/Directory 내용 동등성·오류 반례 |
| **E_015** | ODB++ features.Z 문법 한정 파서 | [#578](https://github.com/july-rust/BoardSI/issues/578) | 명세/공개 Fixture 선행, 미확인 압축은 Fail-Closed |
| **E_016** | ODB++ Copper Negative 또는 Island 규칙 하나 | [#578](https://github.com/july-rust/BoardSI/issues/578), [#10](https://github.com/july-rust/BoardSI/issues/10) | Source Polarity·Source ID·독립 도형 반례 |
| **E_017** | ODB++ Step Repeat/Transform 규칙 하나 | [#578](https://github.com/july-rust/BoardSI/issues/578) | 정확한 좌표·반복 원본·중복/오버플로 반례 |
| **E_018** | ODB++ EDA Package/Pin 매핑 한 종류 | [#578](https://github.com/july-rust/BoardSI/issues/578) | Pin 이름·Net 레이블만으로 전기 연결 추론 금지 |
| **E_019** | ODB++ 제조 홀에서 Canonical Via 후보 한 종류 | [#578](https://github.com/july-rust/BoardSI/issues/578) | Plating/Span 미확인은 UNKNOWN; Hole=도금 Via 단정 금지 |
| **E_020** | ODB++ Stackup Material/z provenance 1개 경로 | [#578](https://github.com/july-rust/BoardSI/issues/578) | 재료/두께 원본 미기재 시 임의 추정 금지 |
| **E_021** | 메인 Qt에 ODB++ 공개 합성 보드 1개 비차단 Import | [#578](https://github.com/july-rust/BoardSI/issues/578), [#11](https://github.com/july-rust/BoardSI/issues/11) | Source→Versioned IPC→Artwork/Picking·취소·ReImport |
| **E_022** | ODB++ SOURCE 한 종류 국소 전기 Graph 반례 | [#578](https://github.com/july-rust/BoardSI/issues/578), [#520](https://github.com/july-rust/BoardSI/issues/520) | 증명된 Copper만 edge, 불명 Net/도금은 UNKNOWN |
| **E_023** | ODB++ 독립 Vendor Fixture와 Cross-format Oracle 1건 | [#578](https://github.com/july-rust/BoardSI/issues/578), [#10](https://github.com/july-rust/BoardSI/issues/10) | 공개·배포권·버전·독립 비교 확보 전 미착수/차단 |
| **E_024** | ODB++ #578/#10 전체 수락 Gate 재평가 | [#578](https://github.com/july-rust/BoardSI/issues/578), [#10](https://github.com/july-rust/BoardSI/issues/10) | 전체 조건 실제 PASS면 이슈 종료, 아니면 미인수 근거 분리 |
| **E_025** | IPC-2581 단일 버전 Layer/Net Source 부분집합 | [#532](https://github.com/july-rust/BoardSI/issues/532), [#23](https://github.com/july-rust/BoardSI/issues/23) | 공개 규격·원본 ID·유효/변형/누락 반례 |
| **E_026** | KiCad PCB Layer/Via Source 부분집합 | [#532](https://github.com/july-rust/BoardSI/issues/532), [#23](https://github.com/july-rust/BoardSI/issues/23) | 버전 명시·Layer Span 원본 증명·부정 반례 |
| **E_027** | Gerber+Drill 한 Layer/홀 분류 부분집합 | [#532](https://github.com/july-rust/BoardSI/issues/532), [#23](https://github.com/july-rust/BoardSI/issues/23) | Net/Material 미지정 UNKNOWN, Solver 입력 승인 금지 |
| **E_028** | 두 Source의 Canonical 기하 등가성 한 케이스 | [#532](https://github.com/july-rust/BoardSI/issues/532), [#520](https://github.com/july-rust/BoardSI/issues/520) | 독립 Oracle·레이어/패드/비아 스팬 전이 검증 |
| **E_029** | macOS Qt 타이머 간헐 지연 재현 실험 1개 | [#46](https://github.com/july-rust/BoardSI/issues/46) | 과거 FAIL 보존·같은 SHA·반증; 원인 미확정 시 별도 원장 |
| **E_030** | E 종료 감사·30건 이슈 전체 재분류 | [#2](https://github.com/july-rust/BoardSI/issues/2), [#3](https://github.com/july-rust/BoardSI/issues/3), [#4](https://github.com/july-rust/BoardSI/issues/4), [#5](https://github.com/july-rust/BoardSI/issues/5), [#6](https://github.com/july-rust/BoardSI/issues/6), [#7](https://github.com/july-rust/BoardSI/issues/7), [#8](https://github.com/july-rust/BoardSI/issues/8), [#9](https://github.com/july-rust/BoardSI/issues/9), [#10](https://github.com/july-rust/BoardSI/issues/10), [#11](https://github.com/july-rust/BoardSI/issues/11), [#12](https://github.com/july-rust/BoardSI/issues/12), [#13](https://github.com/july-rust/BoardSI/issues/13), [#14](https://github.com/july-rust/BoardSI/issues/14), [#15](https://github.com/july-rust/BoardSI/issues/15), [#16](https://github.com/july-rust/BoardSI/issues/16), [#17](https://github.com/july-rust/BoardSI/issues/17), [#18](https://github.com/july-rust/BoardSI/issues/18), [#19](https://github.com/july-rust/BoardSI/issues/19), [#20](https://github.com/july-rust/BoardSI/issues/20), [#21](https://github.com/july-rust/BoardSI/issues/21), [#22](https://github.com/july-rust/BoardSI/issues/22), [#23](https://github.com/july-rust/BoardSI/issues/23), [#24](https://github.com/july-rust/BoardSI/issues/24), [#25](https://github.com/july-rust/BoardSI/issues/25), [#41](https://github.com/july-rust/BoardSI/issues/41), [#46](https://github.com/july-rust/BoardSI/issues/46), [#165](https://github.com/july-rust/BoardSI/issues/165), [#520](https://github.com/july-rust/BoardSI/issues/520), [#532](https://github.com/july-rust/BoardSI/issues/532), [#578](https://github.com/july-rust/BoardSI/issues/578) | 종결/차단/차기제품 구분·열린 PR 0·링크 확인 |

## 현재 30개 열린 이슈 전수 분류

동일 이슈를 아래 서로 다른 분류에 중복 배치하지 않았다. **범위가 계획/감사라는 사실 자체로 기술 인수나 Issue Closed를 의미하지 않는다.**

| 구분 | 열린 이슈 | 처리 원칙 |
| :--- | :--- | :--- |
| M0 기반·인수 (6건) | [#2](https://github.com/july-rust/BoardSI/issues/2), [#3](https://github.com/july-rust/BoardSI/issues/3), [#4](https://github.com/july-rust/BoardSI/issues/4), [#5](https://github.com/july-rust/BoardSI/issues/5), [#6](https://github.com/july-rust/BoardSI/issues/6), [#7](https://github.com/july-rust/BoardSI/issues/7) | E_002–E_007: 완료 근거 재감사·종결 후보 |
| M1 Import·Viewer·Stackup (5건) | [#8](https://github.com/july-rust/BoardSI/issues/8), [#9](https://github.com/july-rust/BoardSI/issues/9), [#10](https://github.com/july-rust/BoardSI/issues/10), [#11](https://github.com/july-rust/BoardSI/issues/11), [#12](https://github.com/july-rust/BoardSI/issues/12) | E_008–E_011 및 E_016–E_024 |
| M1a/M1b 제품 기능 (3건) | [#13](https://github.com/july-rust/BoardSI/issues/13), [#14](https://github.com/july-rust/BoardSI/issues/14), [#15](https://github.com/july-rust/BoardSI/issues/15) | DEFERRED_PRODUCT |
| M2/M2a/M2b/M2c (7건) | [#16](https://github.com/july-rust/BoardSI/issues/16), [#17](https://github.com/july-rust/BoardSI/issues/17), [#18](https://github.com/july-rust/BoardSI/issues/18), [#19](https://github.com/july-rust/BoardSI/issues/19), [#20](https://github.com/july-rust/BoardSI/issues/20), [#21](https://github.com/july-rust/BoardSI/issues/21), [#22](https://github.com/july-rust/BoardSI/issues/22) | DEFERRED_PRODUCT; PH-08 실패 별도 |
| M3/M4/M5 (3건) | [#23](https://github.com/july-rust/BoardSI/issues/23), [#24](https://github.com/july-rust/BoardSI/issues/24), [#25](https://github.com/july-rust/BoardSI/issues/25) | #23 제한 형식 E_025–E_028, #24/#25 DEFERRED_PRODUCT |
| CI·Qt·GPU (3건) | [#41](https://github.com/july-rust/BoardSI/issues/41), [#46](https://github.com/july-rust/BoardSI/issues/46), [#165](https://github.com/july-rust/BoardSI/issues/165) | E_001, E_012, E_029; 원인·실측 증거 필수 |
| D_056 상위 범위 (3건) | [#520](https://github.com/july-rust/BoardSI/issues/520), [#532](https://github.com/july-rust/BoardSI/issues/532), [#578](https://github.com/july-rust/BoardSI/issues/578) | E_004/E_013–E_028; 전체 인수 전 OPEN |

**검산:** 6 + 5 + 3 + 7 + 3 + 3 + 3 = **30개 고유 Open Issue**, Open PR **0개** (2026-11-11 당시). [실시간 Open Issue](https://github.com/july-rust/BoardSI/issues?q=is%3Aissue+is%3Aopen) · [Open PR](https://github.com/july-rust/BoardSI/pulls?q=is%3Apr+is%3Aopen). 이후 이슈가 바뀌면 스냅샷을 재조회한다.

## 이전 A/B/C/D 전수 인계

- **A_001–A_036:** [A 번호별 역사 색인](development-checklist.md#a-history-ledger). A_035 합성 Preview 종합 인수와 A_036 단일 체크리스트 도입은 다른 성격이다. 원본 [A_035 감사](validation/a035-p2z-final-preview-acceptance.md)와 [A 검증 파일 인덱스](validation/INDEX.md), Source 내 번호별 흔적을 대조하되 근거 없는 개별 완료를 만들지 않는다.
- **B_001–B_009:** [Stage B](development-checklist.md#stage-b), [B_009 감사](validation/b009-b-series-final-acceptance.md). 합성 Reference/Stackup 증거는 실제 Copper/Return Path 제품 인수와 다르다.
- **C_001–C_025_10:** [Stage C](development-checklist.md#stage-c)와 C_025의 행정 정리 항목. 독립 체크가 없는 C_025 항목을 완료 건수로 중복하지 않는다.
- **D_001–D_056_09:** [Stage D](development-checklist.md#stage-d)의 SOURCE 국소 Geometry/Graph, Canonical Via, ODB++ subset의 소프트웨어 부분 인수. **D_056 전체 미인수는 E에서 처리**하며, 역사적 최초 날짜/CI는 보존한다.
- **SEC_001/M0/M2/U/T:** [보안 한정 근거](validation/sec001-codeql-markdown-comment-scan.md), [M0 완료 Gate](validation/m0-completion-gate.md), [PH-08 원본 실패](validation/m2-g2-ph08-stage4-original-gateq-reexecution.md), [제품 v1.0 사양](specification/BoardSI_Development_Spec_v1.0.md)과 [U/T 체크](development-checklist.md#stage-acceptance). 한정 PASS가 정식 제품 인수를 대신하지 않는다.

## 정지/이관 기준

- 외부 허가가 필요한 실제 PCB/벤더 데이터는 임의로 생성하지 않는다. 공개 권한·SHA·Oracle 미확인일 때 E_023 또는 실기기 요구 작업을 실행 가능한 것으로 표시하지 않는다.
- 연구·출시 단계 전체를 E에서 한 번에 해결하겠다고 선언하지 않는다. #13–#22, #24–#25는 후속 제품 Backlog로 정확하게 분리한다. #23은 CAD 형식별 **부분집합 작업**만 E에 포함한다.
- 마지막 E_030은 A–D 근거 및 30개 Issue 상태를 최신 다시 읽어 **종결·차단·미완/차후 작업**을 구분하는 작업이다. **프로그램 전체 완료**를 의미하지 않는다.
