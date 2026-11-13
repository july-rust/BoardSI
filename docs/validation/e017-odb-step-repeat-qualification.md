# E_017 — ODB++ Single-Level Step Repeat, Exact Translation & Fail-Closed Qualification

**최초 체크일:** 2026-11-13 KST  
**코드 및 Source 회귀:** MERGED / VERIFIED  
**E_017 완료 판정:** COMPLETE — **단일 단계 Step Repeat SOURCE 범위 한정** (최초 체크일/코드 근거일 2026-11-13 KST). 보호 병합된 최종 코드 main 두 SHA의 Push CI 8/8 각각 성공. 본 문서 전용 PR #612의 보호 Verified Squash 및 정확 docs-main Push 8/8 성공을 최종 운영 종료 조건으로 하며, 이 조건은 #10 전체 ODB++ 제품 수락을 뜻하지 않는다.  
**전체 ODB++ 제품 수락:** NOT_VERIFIED. [전체 Importer #10](https://github.com/july-rust/BoardSI/issues/10)는 OPEN. [#578](https://github.com/july-rust/BoardSI/issues/578)은 E_016 이전 행정 종료 기록을 그대로 유지한다.

## 공식 규격·범위

- [Siemens ODB++Design Format Specification 8.1 Update 4](https://odbplusplus.com/wp-content/uploads/sites/2/2024/08/odb_spec_user.pdf), `stephdr` Step Repeat 규칙 pp.139–141.
- 한 부모 Step의 단일 자식 Step 한 개와 단일 `STEP-REPEAT` 직사각형 배열에 한정. `ANGLE=0`, `FLIP=NO`, `MIRROR=NO`, 양의 `NX×NY≤64`.
- `X/Y`, `DX/DY`, child `X_DATUM/Y_DATUM`을 정확한 `i128`/Rational 계열 연산 및 Canonical CoordinateFrame 원점 이동으로 해석. 부모 Datum 및 선언된 Origin은 0에 한정; 자식 `MM/INCH` 및 부모 `MM/INCH` 단위 차이를 정확하게 변환.
- 기존 E_016 Source parser의 Trace/Pad/Arc/Positive CopperArea/Multi-Island 및 Cutout 재사용. 반복 인스턴스마다 독립 결정적 ObjectId, 원본 파일 SHA-256, 파일 경로:행, Step/반복 인덱스를 SourceIdentity/ConversionStep witness에 기록.
- 기존 Source Geometry만 반복한다. **Net 이름, 반복 ID만으로 전기 Node/Edge를 생성하지 않으며**, Plating/Via/Material/SI/Qt 메인 ODB Import를 수락하지 않는다.

## Fail-Closed 계약

- Nested 또는 여러 Step Repeat, 자기 참조·누락 Child, 지원되지 않는 ANGLE/FLIP/MIRROR, 잘못된 횟수/필드, 선언 중복, Overflow, 동일 위치 인스턴스, Source coordinate/resource 예산 초과를 거부.
- 부모 자체 Copper Feature/EDA, unsupported EDA ownership, 비영(非零) Parent Datum/Origin, 자식 비영 Origin 및 지원되지 않는 Geometry는 거부. Invalid import는 부분 BoardModel을 반환하지 않는다.
- 1,000,000 확장 Geometry 비용 예산에는 최상위 객체뿐 아니라 Trace Points, Arc Points, Padstack Entries, Area 경계선, Cutout 및 Hole 경계 세그먼트까지 포함한다. 복제 전에 checked arithmetic으로 검사한다.
- 전체 Canonical 재검증의 quadratic Padstack 참조 탐색 문제를 제거하고, 반복된 Frame/Layer/Padstack 참조 및 ObjectId 중복을 정렬된 인덱스로 검사하여 O(N log N) 참조 정합성 경로를 사용한다. 원본 Child Source Geometry는 기존 엄격 Parser로 선검증한다.

## 자동 시험 증거

| 범위 | 실제 결과 |
|---|---|
| E_017 2×2 정확 좌표, 결정적 IDs, Datum·혼합 단위 | PASS |
| E_016 Multi-Island 반복·Hole 보존 | PASS |
| Malformed/중첩/회전/반전/중복/Overflow/부모 Artwork | Fail-Closed PASS |
| Nested Hole Expansion Budget 자원 초과 반례 | PASS |
| Directory vs USTAR vs GZIP/TGZ vs ZIP0 vs ZIP8 + child `features.Z` 전체 Canonical 모델 동등성 | PASS |
| 회전 ANGLE=45 Source 변조에 대한 Directory Fail-Closed | PASS |
| 상용 Vendor Export, Cross-CAD Oracle | NOT_VERIFIED, EB-01 |
| 전기 Connectivity/Manufactured Via/SI/실제 Qt 메인 ODB Import | NOT_ACCEPTED |

### 코드 PR #610 — 기본 Parser 및 정밀 좌표

- [PR #610](https://github.com/july-rust/BoardSI/pull/610) final HEAD `e20c323254f567ecf260169caaa0d85a09bc80ed`: **30/30 Required PR Checks SUCCESS**, Native Windows x64/ARM64/macOS ARM64 PASS.
- [Numerical Rust job](https://github.com/july-rust/BoardSI/actions/runs/43335304939/job/131590329251): `boardsi-import` 327 tests PASS, E_017 신규 Source 단위 테스트 6개 PASS, `cargo fmt` 및 `clippy -D warnings` PASS.
- 독립 Codex 최종 HEAD Review 지적 0/미해결 0. 과거 P2 두 건 (Hole/Boundary 예산 누락 및 O(N²) Padstack Validation) 수정 근거 회신 후 두 Review Thread 모두 RESOLVED.
- 보호된 GitHub Verified 단일 부모 Squash [`8717317387114a6001b553f1a40136e73a204be8`](https://github.com/july-rust/BoardSI/commit/8717317387114a6001b553f1a40136e73a204be8), 부모 `02b54c2b6227ebb2a5b45d74a35cd22d317a62c2`.
- 정확한 코드-main `87173173` **8/8 Push Workflow SUCCESS**, PR CI와 별도 검증: [Qt](https://github.com/july-rust/BoardSI/actions/runs/43337569125), [Numerical](https://github.com/july-rust/BoardSI/actions/runs/43337570434), [Docs](https://github.com/july-rust/BoardSI/actions/runs/43337568513), [PH-06](https://github.com/july-rust/BoardSI/actions/runs/43337568760), [PH-07](https://github.com/july-rust/BoardSI/actions/runs/43337568787), [Native FEM](https://github.com/july-rust/BoardSI/actions/runs/43337568821), [Lossless](https://github.com/july-rust/BoardSI/actions/runs/43337568777), [G0](https://github.com/july-rust/BoardSI/actions/runs/43337568761).

### 코드 PR #611 — 직접 Transport 동등성 증거

- [PR #611](https://github.com/july-rust/BoardSI/pull/611) final HEAD `bfdfe551324ecc3e308c43a9838d8311d90cc7ce`: **29/29 PR Checks SUCCESS**, `features.Z`와 Step Repeat의 Directory·USTAR·GZIP·ZIP0/8 1:1 전체 모델 동등성을 실제 Rust 자동 테스트에서 확인([Numerical Job](https://github.com/july-rust/BoardSI/actions/runs/43338216885/job/131599912511)).
- 독립 Codex final HEAD review 지적 0/미해결 0, 스레드 0.
- 보호된 GitHub Verified 단일 부모 Squash [`83f0871e2608a5b2e8336d6bbea81d6d08b5d7fe`](https://github.com/july-rust/BoardSI/commit/83f0871e2608a5b2e8336d6bbea81d6d08b5d7fe), 부모 `8717317387114a6001b553f1a40136e73a204be8`.
- 정확한 코드-main `83f0871e2608a5b2e8336d6bbea81d6d08b5d7fe`: **독립 event=push 8/8 Workflow SUCCESS**, 0 실패: [Qt](https://github.com/july-rust/BoardSI/actions/runs/43340500007), [Numerical](https://github.com/july-rust/BoardSI/actions/runs/43340500023), [Docs](https://github.com/july-rust/BoardSI/actions/runs/43340499925), [PH-06](https://github.com/july-rust/BoardSI/actions/runs/43340500044), [PH-07](https://github.com/july-rust/BoardSI/actions/runs/43340499908), [Native FEM](https://github.com/july-rust/BoardSI/actions/runs/43340500018), [Lossless](https://github.com/july-rust/BoardSI/actions/runs/43340500316), [G0](https://github.com/july-rust/BoardSI/actions/runs/43340500094). PR #611의 29/29 CI와 별개의 exact main 실행.

## 수락 경계와 후속 번호

E_017 소형 Step Repeat Source 지원 범위만 완결한다. 산업용/상용 ODB++ 전체 적합성은 별도 `#10`과 EB-01, E_018(Eda PKG/PIN), E_019(Drill→Via), E_020(Material z), E_021(Qt Main Import), E_022(Graph), E_023(Vendor Oracle), E_024(Whole Import Gate)의 수락 조건으로 관리한다. #578 과거 행정 CLOSED는 전체 기술 수락을 의미하지 않는다. 본 검증의 합성 Fixture는 실제 Vendor Oracle을 대체하지 않는다.
