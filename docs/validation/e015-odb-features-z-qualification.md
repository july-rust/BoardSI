# E_015 — ODB++ `features.Z` UNIX Compress LZW Source qualification

**검증 근거일:** 2026-11-12 KST. **상태:** 한정 SOURCE COMPLETE, 최종 문서 PR 보호 병합 및 정확한 docs-main Push 성공 조건. 상위 [Issue #578](https://github.com/july-rust/BoardSI/issues/578)은 전체 ODB++ 미인수로 OPEN.

## 구현과 경계

- [코드 PR #605](https://github.com/july-rust/BoardSI/pull/605): UNIX Compress magic `1F 9D`, 9–16-bit 가변 LZW, Block CLEAR 및 Non-block mode, dictionary bounds, 16 MiB 해제 출력 상한. MSRV Rust 1.75, 외부 Shell/압축 실행 파일/디스크 압축 해제 없음.
- `crates/boardsi-import/src/odb_source.rs`는 `steps/<step>/layers/<layer>/features.Z`만 기존 `features` Parser로 정규화한다. Source SHA-256은 원본 압축 파일과 파생 해제 파일을 각각 기록한다. 압축이 없는 Legacy FileMap/SourceIdentity는 변경하지 않는다.
- 실패 조건: 잘못된 헤더·Width·Code·Dictionary, 중복 `features` + `features.Z`, 지원하지 않는 `.Z` 파일 경로, 출력/누적 메모리 한도 초과, 유효하지 않은 Feature 문법. 전부 Fail-Closed로 처리한다.
- 기존 Directory, USTAR, ZIP Method 0/8, GZIP/TGZ 경로와 Canonical Geometry 및 Drill Source Audit를 재사용한다. 기존 128 MiB 논리 파일 누적·8,192 Entry 및 Archive별 상한을 유지한다.
- **물리 경계:** Canonical의 Source 기하 일부와 Drill 관측값만 확인한다. 도금된 Via, 전체 전기적 Connectivity/Open/Short, Material Stackup, Main Qt ODB Import 및 SI Solver는 검증하지 않았다.

## 독립 검증

| 구분 | 원본 검증 증거 |
| :--- | :--- |
| 정확한 코드 PR HEAD | `80bf903a76dd1df0712ee2f85e2ef90142eb9744`, **30/30 GitHub Checks SUCCESS** |
| 세 플랫폼 Native | Windows x64 / Windows ARM64 / macOS ARM64 Native Rust 및 Qt 필수 검사 SUCCESS |
| 독립 리뷰 | [Codex PR #605](https://github.com/july-rust/BoardSI/pull/605#issuecomment-6818003700) 최종 HEAD 검토, 지적 0건, 미해결 Review Thread 0건 |
| 보호 병합 | GitHub Verified 단일-parent Squash [c16d568013be9e5b2d146919d2374f1cad23fbf3](https://github.com/july-rust/BoardSI/commit/c16d568013be9e5b2d146919d2374f1cad23fbf3) |
| 정확한 코드-main Push | **8/8 Workflow SUCCESS**, [Qt](https://github.com/july-rust/BoardSI/actions/runs/43181991679) · [Numerical](https://github.com/july-rust/BoardSI/actions/runs/43181991594) 등 |
| LZW 양성 | GNU `gzip -dc`로 독립 확인된 UNIX Compress 테스트 벡터, 9→10→11-bit 폭 변경, KwKwK, CLEAR, Non-block Code 256 |
| Source 통합 | 압축/비압축 Geometry 동등성, 서로 다른 압축·해제 SHA의 구분, USTAR/ZIP0/ZIP8/GZIP/Directory, Drill Audit 비교 |
| 부정 검증 | Header·예약 Flags·손상 Code·중복 Path·임계 출력·미지원 Feature Grammar를 거부 |

**중요한 무결성 제한:** UNIX Compress `.Z`는 GZIP과 달리 의무 CRC32/ISIZE 및 확정적인 End-of-Stream 표식이 없다. 유효하게 보이는 일부 손상·절단 비트스트림은 검출할 수 없다. Source SHA는 실제 입력과 파생 데이터의 바이트 신원이지 Vendor 서명/원본 진위의 증명이 아니다.

## 최종 종료 게이트

- [x] 최종 코드 HEAD 필수 CI 및 세 Native 플랫폼
- [x] 독립 Codex Review, 미해결 스레드 0
- [x] 보호 Verified Squash 코드 병합
- [x] 정확한 코드-main SHA Push 8/8 Workflow SUCCESS
- [ ] 이 문서 PR 최종 HEAD 필수 CI/리뷰, Verified Squash, 정확 docs-main Push — 후행 확인 필요

유일한 `docs/development-checklist.md`의 E_015는 위 최종 문서 Gate 조건부 SOURCE 한정 COMPLETE. Stage E 13건 한정 COMPLETE, E_006~E_007 2건 EVIDENCE_QUALIFIED, E_016~E_030 15건 NOT_STARTED. A/B/C/D/SEC/U/T 종전 166/128 통계와 합산하지 않는다.

E_016 Negative/Island, E_017 Step Transform, E_018 EDA Package/Pin, E_019 Drill/Via, E_020 Stackup, E_021 Qt ODB Import, 상용 Vendor Oracle, 전기/SI 및 정식 U/T 미인수. 상위 [#578](https://github.com/july-rust/BoardSI/issues/578) OPEN 유지.
