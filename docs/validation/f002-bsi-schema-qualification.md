# F_002 — SQLite v1 Schema / BSIG Byte Contract Qualification

> 실행 기준: `main@a6b797887b7bc40f4961927bf90f1e38c55bffce` (F_001 최종 정산 후) · 2026-11-15 KST · [F_002 DDL](../../schemas/bsi/v1.sql) · [SQL 설계](../architecture/f002-bsi-sqlite-schema.md) · [BLOB 규격](../architecture/f002-bsi-geometry-binary-layout.md).

## 1. 사실 / 설계 / 미실행 구분

| 범위 | 사실 또는 증거 | 분류 |
| :--- | :--- | :--- |
| F_001 확정 SQLite 단일 파일 / 52B BSIG | [F_001 원문](../architecture/f001-bsi-format-contract.md) | 상위 규격 |
| 실제 CREATE TABLE, PK/FK/STRICT 및 PRAGMA | [schema v1](../../schemas/bsi/v1.sql) | 구현된 DDL, 실행 결과는 CI 확인 |
| 9종 Geometry TLV byte grammar | [Binary Layout](../architecture/f002-bsi-geometry-binary-layout.md) | 신규 규범적 설계 |
| T01~T10 Python sqlite3 + SHA/정수 Byte Vectors | [실행 코드](../../tools/validation/test_f002_schema.py) · [CI](../../.github/workflows/f002-native-sqlite-schema.yml) | 실제 결과는 Actions 로그에 종속 |
| SQLite ID 목록 | [SQLite upstream magic.txt](https://raw.githubusercontent.com/sqlite/sqlite/master/magic.txt) | 2026-11-15 조회 목록에 0x42534931 없음; 등록/독점 보증 아님 |
| Rust Writer/Reader, Save As 원자성, full CAD round-trip | F_003~F_005 | **NOT_RUN** |

## 2. F002-T01~T10 시험 계획 / 판정 계약

| ID | 검사 | 합격 판정 |
| :--- | :--- | :--- |
| T01 | DDL CREATE, 무결성 및 파일 재열기 | 실제 SQLite DB header 및 .bsi reopen 정상 |
| T02 | ObjectId 중복, FK, SHA 크기, Capability 상태, Feature bool 반례 | SQLite 예외 및 원본 정상 상태 유지 |
| T03 | application_id/user_version/FK/trusted_schema/foreign_key_check | 선언 수치와 일치 |
| T04 | 상향 minimum Reader 및 선택/필수 Feature | DDL 거부 또는 명시 required, 독립 Reader 정책 별도 |
| T05 | Trace/Arc BSIG Envelope, Byte Vector, 유효 SHA | deterministic bytes, 형상 필드 정확 |
| T06 | i64 extreme, i128 극값, 분모 0/음수, unreduced | 정확 인코딩 또는 예외 |
| T07 | Envelope 훼손, version/flags/hash/length/kind 불일치 | Fail-Closed |
| T08 | geometry index query plan 및 Payload size 상한 | 명시 index 및 한도 오류 |
| T09 | Canonical Source, Geometry Kind, Capability, ConversionQuality 대응 | 구조 mapping/미확인 상태 보존 |
| T10 | 플랫폼별 SQLite 버전/OS 식별 기록 | 3.37.0 이상, 각 Native Runner 테스트 통과 |

**실행:** `python -m unittest discover -s tools/validation -p 'test_f002_schema.py' -v`. 3계열 4 OS Runner: Ubuntu, Windows x64, Windows ARM64, macOS ARM64. Native OS에서 Python 내장 SQLite 실행은 **네이티브 Rust/Qt Writer/Reader 시험과 다르다**.

## 3. 검사 책임 경계

- **SQL 제약:** PK, FK, CHECK, UNIQUE, SQLITE STRICT, 문서상 1행 file_meta. SQLite의 FK는 연결마다 ON 필수. SQL CHECK의 NULL 3가 논리값 주의.
- **Reader 의미 검사(후속 F_003):** 고정 버전, 필수/선택 Feature, Source tuple 정렬/중복, Rational 양수·기약 여부, Padstack/Layer Span, net/Connectivity Evidence, Unknown/preservation, Envelope/Geometry Kind 교차 검사.
- **제품 범위 밖:** E_030 EB-01~07, 원본 PH-08 FAIL, U/T 정식 물리 인수. 개발 체크리스트 전체 `166/128` 그대로 유지.

## 4. PR/CI 및 최종 상태

개발 중인 이 문서의 내용 자체가 완료 증거가 아니다. PR 최종 HEAD CI, 독립 리뷰, 보호 Squash Merge, Verified 서명 및 정확한 main Push CI를 확인한 뒤 체크리스트에 **100% COMPLETE (F_002 한정)**를 기록한다.

**F_002 작업 내용:** DDL/Byte Contract/SQLite Matrix 테스트. **F_003 작업 내용:** Production Rust Serializer/Reader, Writer/Reader 전체 타입 검증, Qt 파일 열기, 실제 원자적 저장. 서로 혼동하지 않는다.
