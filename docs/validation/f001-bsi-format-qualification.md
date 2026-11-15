# F_001 — `.bsi` 핵심 규격 독립 검토 경계 및 인수 기록

> 검토일: **2026-11-15 KST** · 기준: `main@eda743fa6d8d669d07fa160aaf83fa6c16245abc` · [핵심 규격](../architecture/f001-bsi-format-contract.md) · [E_030 인계](e030-e-stage-final-closure-audit.md)  
> **범위:** 기존 Canonical 모델/공식 SQLite 규격과 문서 설계의 **정적 대조**. 이 보고서는 실행되지 않은 SQLite Native Encoder/Reader, Windows/macOS GUI, 대형 PCB를 PASS로 만들지 않는다.

## 1. 입증된 기존 사실과 신규 설계의 구분

| 영역 | 확인한 출처 | 분류 |
| :--- | :--- | :--- |
| SQLite 파일 헤더, ID/버전, 트랜잭션/추가 journal | [SQLite file format](https://www.sqlite.org/fileformat2.html), [PRAGMA](https://www.sqlite.org/pragma.html) | **FACT** — SQLite의 공개 규격 |
| SQLite WAL sidecar, checkpoint, 휴대 파일 주의 | [SQLite WAL](https://www.sqlite.org/wal.html) | **FACT** |
| STRICT 도입 3.37.0 | [SQLite STRICT](https://www.sqlite.org/stricttables.html) | **FACT** |
| 현재 Canonical i64 Source ticks, i128 유리수, SourceIdentity/ObjectId, 명시적 전기 그래프 | [Canonical](../architecture/canonical-board-model.md), [D_056_01](../architecture/d056-01-source-evidence-audit.md) | **FACT** — 기존 문서/코드 계약 |
| SQLite + 관계형 메타데이터 + Geometry BLOB, envelope, version 정책 | [F_001 계약](../architecture/f001-bsi-format-contract.md) | **DESIGN DECISION** — 신규 제품 설계 |
| 실제 `.bsi` 파일 bytes 로드, round-trip, 손상 복구, 이종 OS 성능 | F_002~F_005 구현·시험 증거 필요 | **NOT_RUN** |

**불확실성:** `application_id=0x42534931`의 외부 등록·충돌 여부 **NOT_VERIFIED**, **첫 정식 배포 전에 반드시 확인**. 초기 정식 `.bsi` Golden Fixture는 아직 없으며 과거 파일 호환성은 **정책만 확정**됐다. 테스트하지 않은 정상화/무손실 왕복을 주장하지 않는다.

## 2. F001-T01~T08 명시적 검토

| Gate | 정적 기대 결과 및 반례 | 현재 규격 검토 | 실행 검증 |
| :--- | :--- | :--- | :--- |
| T01 | SQLite 표준 헤더·`application_id` 계약; 타 포맷 위장 거부 | **SPEC_PASS** (등록 여부 제외) | NOT_RUN |
| T02 | `user_version`/format/min_reader/feature 의미 분리; 미래 필수 버전 거부 | **SPEC_PASS** | NOT_RUN |
| T03 | i64 tick/i128 rational 무손실; float truncation, overflow 거부; BLOB endian/길이/hash 고정 | **SPEC_PASS** | NOT_RUN |
| T04 | 최초 정식 Golden Fixture부터 미래 Reader/원본 보존 migration; downgrade 무조건 허용 금지 | **SPEC_PASS** | NOT_RUN |
| T05 | Unknown optional 재저장 시 opaque 보존 미검증이면 read-only; Unknown required 거부 | **SPEC_PASS** | NOT_RUN |
| T06 | 입력 검증·무결성/FK/semantic 분리; 중단/저장 실패 시 원본 bytes 불변 | **SPEC_PASS** | NOT_RUN |
| T07 | 기존 SourceIdentity/ObjectId/Canonical/Rust·Qt 비차단 소유권과 일치 | **SPEC_PASS** | NOT_RUN |
| T08 | 확장은 추가 테이블/타입/버전; 과거 Reader/Golden 회귀 보존 | **SPEC_PASS** | NOT_RUN |

**SPEC_PASS**는 F_001 문서의 요구 항목이 명시됐음을 뜻한다. 독립 코드 리뷰/런타임 검증/전체 제품 승인과 다르다. 반례를 실제 악성 `.bsi` 파일로 주입한 시험은 수행하지 않았다.

### 검토 반례 — 구현 전에 반드시 시험화

| ID | 반례 입력 | 기대 |
| :--- | :--- | :--- |
| N01 | SQLite 헤더 정상, 다른 `application_id` | BSI_NOT_NATIVE |
| N02 | 미래 `user_version` / 요구 Reader 상향 | read-only도 의미 로드 금지, 원본 불변 |
| N03 | unknown required feature / unknown optional 편집 후 write | required 거부 / optional 비보존이면 저장 거부 |
| N04 | Rational 분모 0, i64/i128 overflow, 비정수 regrid | 명시적 정밀도·범위 오류 |
| N05 | Geometry BLOB magic/version/hash/length/flags 손상 | BSI_INVALID, 미표시/미저장 |
| N06 | Padstack/Layer/Via Span/Net FK 불일치 | 구조/의미 검사 거부 |
| N07 | 활성 WAL 상태에서 main DB만 복사, disk full/kill/retry | 불완전 파일 배포 금지, 마지막 정상 파일 불변 |
| N08 | 현재 Reader에 없는 확장을 포함한 신버전 파일 | 지원 행렬에 따른 명시적 거부, Silent downgrade 금지 |
| N09 | 외부 Source SHA는 있지만 plating/접속 근거 부재 | Source 선언만 유지, 전기 VERIFIED로 승격 금지 |

## 3. CAD 표현 비교 — 설계 판단

| 근거 | 확인 사실 | 이 계약에서의 결론 |
| :--- | :--- | :--- |
| [KiCad PCB 문서](https://dev-docs.kicad.org/en/file-formats/sexpr-pcb/) | Source 포맷이 Layer/Net/Footprint/Track/Zone을 독립 섹션으로 정의 | Native DB는 Layer/Net/Geometry/Source의 독립 관계를 보존 |
| [KiCad 좌표 규칙](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/) | KiCad PCB 좌표가 1 nm 최소 내부 단위로 기술됨 | 이를 **모든 CAD의 1 nm 공통 격자**로 강제하면 안 됨 |
| [Siemens ODB++ 8.1 사양](https://odbplusplus.com/wp-content/uploads/sites/2/2024/08/odb_spec_user.pdf) | Step/Layer/EDA 제조·설계 Source 구조 구별 | Native도 출처·Step·제조 속성의 미확인을 명시 |
| [E_025 IPC-2581 단일 Revision 검증](e025-ipc2581c-layer-net-source-qualification.md) | **Layer/LogicalNet SOURCE Inventory** 제한 증거만 있음 | IPC-2581 전체 형상/전기 인수를 선행하는 것처럼 기록 금지 |
| [SQLite app file format](https://www.sqlite.org/appfileformat.html) | SQLite는 문서형 단일 파일 DB로 사용할 수 있음 | SQLite 선정 유지, 자체 외부 헤더/ZIP 이중 래퍼 금지 |

**평가:** 원본 CAD 표현과 Native Canonical 표현은 1:1 byte-equivalence가 아니다. 향후 시험에서는 (a) 원본 bytes, (b) source-to-canonical 정보, (c) 저장 전후 Canonical 의미 등가성을 별도로 평가해야 한다. 미지원 Source 객체는 별도의 Explicit Unsupported로 남기고 무손실이라 표시하지 않는다.

## 4. F_002 상세 설계 인계

F_001에서 **추가 이슈를 생성하지 않고**, 이미 예정된 F_002 계약으로 인계한다.

1. 정규화된 DDL: FileMeta, Version/Features, SourceIdentity, CoordinateFrames, Objects/Geometry, Layer/Net/Component/Pin, Stackup/Material, Diagnostics, Extensions와 FK/index를 설계한다.
2. Geometry Payload **52 byte envelope는 변경 금지**. 타입별 Shape 레코드·참조/압축/수치 경계·canonical byte 순서를 규정한다.
3. SQLite runtime version/STRICT 선택, `application_id` 등록·충돌 검증, 안전한 SQLite 설정·최대 한도·DDL 호환성을 확정한다.
4. 최초 writer를 배포할 때 공식 Golden Fixture/해시, source fixture 출처·권리·실행 OS·SQLite version을 남긴다.
5. `BSI_INVALID`, `BSI_FUTURE_SCHEMA`, `BSI_UNSUPPORTED_REQUIRED`, `BSI_NOT_NATIVE`를 데이터 검사에서 별도 오류로 노출한다.
6. F_003에서 손상/중단/확장/마이그레이션 반례 N01~N09의 **실제 실행 증거**를 수집한다.

**별도 제품 이슈:** E_030의 EB-01~EB-07 / U/T 정식 시험 미인수 상태 유지. PR CI나 docs lint가 성공해도 이를 상위 제품 PASS로 승격하지 않는다.

## 5. GitHub 종료 증거

- **작성 시점:** F_001 문서 작성·정적 검토. 최종 PR 번호, 최종 HEAD, CI/리뷰/보호 Squash, 정확한 main Push workflow는 아직 **미확정**이며 실제 GitHub 결과를 검토해 최종 보고에서 기록한다.
- **종료 판정:** 문서 SPEC_PASS와 필수 protected PR/merge gate가 확인되면 **F_001 문서·아키텍처 작업만 한정 COMPLETE**. 실제 Native `.bsi` round-trip은 계속 NOT_RUN.
