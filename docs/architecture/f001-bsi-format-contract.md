# F_001 — BoardSI Native `.bsi` 파일 핵심 규격 계약

> **결정 상태:** F_001 핵심 아키텍처 규격(F_002 DDL 및 F_003 코드 구현의 기준). **작성·규격 검토:** 2026-11-15 KST.  
> **출발점:** E_030 종료 `main@eda743fa6d8d669d07fa160aaf83fa6c16245abc`. [E_030 인계](../validation/e030-e-stage-final-closure-audit.md) · [Canonical Board Model](canonical-board-model.md) · [단일 체크리스트](../development-checklist.md)  
> **증거 범위:** 규격 결정/정적 근거 대조. **SQLite/Rust/Qt .bsi 실제 저장·열기 테스트는 아직 NOT_RUN.**

## 1. 채택 결정 및 데이터 흐름

**정식 결정:** BoardSI 소유의 단일 `.bsi` 파일은 **표준 SQLite 3 데이터베이스**이다. SQLite 헤더 앞에 독자 바이너리 헤더, ZIP 래퍼 또는 별도 필수 사이드카를 두지 않는다. 텍스트/관계형 메타데이터와 정확한 정수 기반 바이너리 Geometry를 **동일 SQLite 컨테이너** 안에서 관리한다.

```text
PADS / ODB++ / IPC-2581 / Gerber+Drill / KiCad (실제 지원 부분집합만)
     → Source adapter → 검증된 Canonical BoardModel (Rust)
     → .bsi writer → 임시 SQLite 파일 검증 → 원자적 교체 → 단일 .bsi
     → .bsi reader → 독립 구조/출처 검증 → Canonical BoardModel
     → 기존 Qt/Worker IPC (저장 파일과 IPC wire version은 별개)
```

- Native 파일은 CAD *원본 그대로의 저장본*이나 물리 전기 검증 증명이 아니다. 현재 지원하지 않는 CAD 객체를 데이터 손실 없이 Canonical에 대응시키지 못하면 무조건 무손실 변환 성공으로 표시하지 않는다.
- 기존 `crates/boardsi-model`의 `BoardModel`, `SourceIdentity`, `ObjectId`, `GeometryProvenance`, `ImportCapability`와 의미상 동형인 데이터를 직렬화한다. 이 파일이 제품의 또 다른 정규 모델이 되어서는 안 된다.
- `geometry→solver` 변환은 독립 검증 과정이며, 저장/로드가 전기적 Open/Short·RLGC·30 GHz SI 성능을 증명하지 않는다.
- Rust Core가 직렬화/검증 소유권을 가진다. Qt는 명시적 사용자 작업과 상태 표시, 기존 비차단 IPC를 사용한다. 파일 로딩 중 UI 동결을 허용하지 않는다.

## 2. SQLite 컨테이너 불변 조건

| 구분 | 영구 계약 | F_002/F_003 구현 확인 |
| :--- | :--- | :--- |
| 파일 외형 | SQLite 표준 헤더 `SQLite format 3\0`, 확장자 `.bsi` | 헤더/파일 식별 검사 |
| Application ID | `PRAGMA application_id = 0x42534931` (ASCII `BSI1`), 향후 고정 | 최초 정식 발행 전 공개 ID 목록 충돌 확인; file(1) 등록은 선택 |
| Schema 버전 | `PRAGMA user_version` = 최초 `1`부터 단조 증가하는 **물리 DDL 버전** | DDL/사용 라이브러리 지원 범위와 대조 |
| 파일 의미 버전 | 메타데이터에 `format_major`, `format_minor`, `minimum_reader` | major/minor 지원 매트릭스 |
| 초기 파일 버전 | 최초 발행 예약값 `1.0`; 초기 Reader 최소 `1.0` | 발행 전 Golden Fixture 생성 및 발행 기록 |
| Feature 계약 | `required_features` / `optional_extensions`의 이름+버전+필수 여부 | 지원표 기준 사전 검사 |
| 진단/호환성 | `integrity_check`, `foreign_key_check`, 의미 검증 각각 수행 | 결함별 오류 코드/테스트 |
| 최소 SQLite | F_002에서 확정하고 빌드에 고정. `STRICT` 채택 시 **3.37.0 이상** 필요 | 세 플랫폼 동일 DDL 지원 및 실제 Library 버전 기록 |

`0x42534931`은 F_001의 **제안 고정 상수**이며 공개 ID 목록과의 충돌은 아직 **NOT_VERIFIED**다. 최초 정식 `.bsi` 배포 전 충돌을 확인한다. OS `file(1)`의 별도 파일 형식 인식이 필요하면 해당 유틸리티 관리 프로젝트에 선택적으로 등록을 요청할 수 있으며, **SQLite DB 사용에 등록이 필수는 아니다**. **정식 발행 이후 ID 변경은 금지**한다. `user_version`은 파일 기능/확장 의미 버전을 대신하지 않는다.

SQLite 페이지 크기·자동 vacuum·인덱스·`STRICT` 적용 범위는 F_002에 위임한다. 해당 선택은 반드시 F_001 읽기 호환 정책을 지켜야 한다.

### 파일 열기 단계: 순서 고정

1. 파일의 존재/일반 파일 여부/크기 상한/SQLite 헤더 확인. `application_id` 불일치 시 **BSI_NOT_NATIVE**.
2. Reader가 SQLite DB를 **기본 읽기 전용**으로 연다. SQLite 확장 로딩 금지, 외부 파일 참조 금지, `trusted_schema=OFF` 및 지원 환경의 방어 설정 사용.
3. **DDL 물리 버전, major/minor, minimum_reader, required feature**를 확인한다. 지원하지 않는 필수 기능 또는 미래 DDL이면 **BSI_UNSUPPORTED_REQUIRED** 또는 **BSI_FUTURE_SCHEMA**; 자동 편집/저장 금지.
4. 한도 내 DB 무결성 검사 → FK 검사 → Canonical 객체/좌표/레이어·Stackup·Net·Via 참조와 Source Provenance의 의미 검증. 어느 하나라도 오류면 **BSI_INVALID**.
5. 검증 통과 후에만 객체를 노출한다. 실제 물리 검증·SI 인수와 구별해 **structurally loaded** 상태로 표시한다.

`UNKNOWN`이나 누락 값은 실제 검증된 값을 대체할 수 없다. 미래 파일의 `minimum_reader`가 현재 Reader보다 높으면 지원 불가 처리한다.

## 3. 데이터 영역, 객체 ID, 출처 보존

| 논리 영역 | 저장 책임 | 의미상 의무 |
| :--- | :--- | :--- |
| File/Version/Feature | ID, 버전, 필수/선택 확장, 제작 프로그램 | 호환성의 단일 진입점 |
| ImportSources | Format/Revision, 파일별 논리 경로+검증 여부+SHA-256, Import 조건 | Source 내용과 선언의 구분 |
| Identities/Provenance | `SourceIdentity` + `ObjectKind/source_object/part`, 변환 품질/오차/이력 | ID의 deterministic 원문 튜플 보존 |
| CoordinateFrames | Source i64 tick, metre/tick i128 유리수, 원점/회전/미러/방향 | 원본 좌표 재구성 |
| Layers/Stackup/Materials | 순서/kind/z/두께/단위/물성, Missing/Assumed/Confirmed | 물성/제조 사실 임의 채우기 금지 |
| Net/Component/Pin | Source와 Canonical 관계/고유 참조 | 이름만으로 전기 연결 추정 금지 |
| Geometry | Trace/Arc/Copper/Outline/Pad/Padstack/Via/Drill/Voids | 정확한 정의 및 객체별 참조 |
| Connectivity/Diagnostics | 선언·근거·Capability·Unverified/Invalid | `CANDIDATE`를 `VERIFIED`로 승격 금지 |
| Extensions | 확장 ID·버전·필수 여부·Opaque payload | 선택 확장과 필수 기능 구별 |

**ID 원칙:** 객체 식별자는 잘린 해시, DB rowid, 삽입 순서, 현재 프로세스 해시를 영구 ID로 사용하지 않는다. 기존 `ObjectId`의 **전체 SourceIdentity + ObjectKind + source_object + part**를 의미 손실 없이 보존한다. SQL의 단기 surrogate key는 내부 FK/인덱스에만 사용하며 재정렬·마이그레이션에도 식별자를 변경하지 않는다. 편집 후 새 ID와 과거 ID 사이의 추적 관계는 별도 이력으로 기록해야 하고, 임의 재결합 금지다.

`ImportCapability`의 G/N/P/S/R/M 각 차원은 독립 보존하고 `Complete/Partial/Missing/Conflicting`을 유지한다. 실제 Source 매핑이 확인되지 않은 단계는 `Unknown` 또는 `Missing`을 그대로 저장한다. 개인 로컬 절대경로, 회사 비공개 PCB/Stackup, 재배포 권한 미확인 Fixture는 공개 저장소에 넣지 않는다.

## 4. 좌표·Geometry 무손실 계약

**기존 Canonical 계약 그대로 유지:** 원본 좌표는 `i64` tick, metre/tick 정규화 분수는 `i128/i128`이고 분모는 양수다. 초기 공통 단위로 일괄 1 nm 격자를 강제하지 않는다. 예컨대 KiCad의 1 nm 내부 단위는 KiCad Source Frame의 선택일 뿐 모든 CAD의 정밀도 정책이 아니다.

- `i64`는 signed 64-bit 정수 원형, `i128`은 **정확히 16 byte 고정 길이 little-endian 2's complement**로 저장한다. SQLite의 `INTEGER`만으로는 i128을 표현할 수 없어 Float/REAL로 변환하거나 잘라내지 않는다.
- 유리수의 기약화·부호/분모 검증·`i128::MIN` 제외 정책, checked 산술, 불능 `regrid`와 좌표 범위 초과의 Fail-Closed는 기존 `canonical-board-model.md`와 동일하다.
- Arc의 방향·중심·반지름/각도·원본 속성은 **해석적**으로 보존한다. 렌더링 tessellation을 정식 저장 Geometry로 대체하지 않는다. Polygon의 외곽/홀, Copper positive/negative 극성, Pad role(Conductive/AntiPad/Clearance), Via Span/Drill/stackup 및 미확인 plating 근거를 보존한다.
- 원본 좌표를 변환한 경우 **Source Frame, 원본 값, 변환 연산, exact/approx/loss/pending 상태와 오차 정보**를 유지한다. 근사 변환을 무손실이라고 표시하는 것은 오류다.
- SI 수치해석용 `f64` 좌표와 GPU mesh는 파생 캐시다. 정본으로 저장하지 않는다. 그 파생값을 사용한 결과는 별도 모델/정확도 검증이 필요하다.

### Binary Geometry payload — v1 envelope 계약

Geometry 데이터의 논리 단위는 **객체 또는 독립 스트림 단위의 버전 관리 BLOB**이다. F_001은 바깥쪽 envelope 의미와 부호·엔디언을 고정하고, 형상별 레코드 순서/DDL은 F_002에서 최종 바이트 표로 확정한다.

| Field | 크기/표현 | 유효성 검사 |
| :--- | :--- | :--- |
| magic | 4 byte ASCII `BSIG` | 다른 Magic 거부 |
| codec_version | u16 little-endian, 최초 1 | 미지원 버전 거부 |
| geometry_kind | u16 little-endian | 정의되지 않은 필수 Kind 거부 |
| flags | u32 little-endian | 미정의 필수 bit 거부 |
| uncompressed_length | u64 little-endian | 명시된 상한/실제 길이 일치 |
| payload_sha256 | 정확히 32 byte | **압축 해제한 원본 payload** SHA-256 일치 |
| payload | length-bounded byte stream | 종류/필드별 버전·개수·수치/참조 유효성 검사 |

고정 envelope **52 byte**. v1 초기 payload는 **비압축**, 미지정 flags=0을 원칙으로 한다. 이후 압축을 도입하면 독립 codec/flag와 최대 확장 비율·압축 해제 한도·무결성 시험을 추가한다. 숨겨진 padding, 환경별 ABI 크기, endian 의존 복사를 금지한다. 저장 데이터의 순서와 직렬화가 결정적이어야 한다. **F_002는 이 envelope를 바꿀 수 없고**, 신규 레코드 타입·새 버전으로 확장해야 한다.

> **계약 경계:** F_001은 바이너리 bytes를 생성하는 Encoder가 아니다. BLOB 내 Shape별 field layout, 전체 key/index DDL, 한도 숫자는 F_002의 후속 결정이며 실제 합성 Fixture/대형 보드 데이터를 바탕으로 설정한다. 표현 불가능한 객체를 지원되는 것으로 조용히 변환하지 않는다.

## 5. 버전/마이그레이션/Unknown/장기 호환

**영구 정책:** 최초 공식 공개 이후 발행한 모든 **정상·지원 규격의 정식 `.bsi` 파일**은 이후 BoardSI Reader에서 열 수 있도록 기존 Reader, 버전별 마이그레이션 경로, Golden Fixture, 회귀 테스트를 유지한다. 임의의 손상 파일, 미발행 실험 파일, 미래 버전 역방향 읽기까지 무조건 보장하는 의미는 아니다.

- **새 Reader ↤ 구 정식 파일:** 지원 경로 필수. 각 정식 버전의 원본 샘플/해시/기대 구조 불변식 확보 후 배포.
- **구 Reader ↤ 미래 파일:** 보장하지 않음. 필수 기능/major/DDL 불일치 시 안전 거부, 절대 침묵 복구하지 않는다.
- **Minor 변경:** 의미를 바꾸지 않는 선택 확장만. 새 필수 해석이 필요한 변경은 `minimum_reader`와 `required_features` 갱신이 필요하다.
- **Major 변경:** 기존 구조의 의미를 변경해야 할 때만 도입. 이전 파일 Reader/마이그레이션 유지, 다른 major의 원본을 무조건 덮어쓰지 않는다.
- **Unknown optional:** 읽기만 가능한 범위에서 스킵할 수 있지만 **저장 시 선택 확장 payload·참조 관계를 그대로 보존할 수 있다고 검증되지 않으면 반드시 read-only**. 사본이 삭제되거나 기존 오브젝트가 변형되면 Fail-Closed.
- **Unknown required:** 파일 열기 거부. 저장 규격에서 의미적으로 필수 정보·제약·참조가 없으면 Fail-Closed.
- **Migration:** 원본 읽기 전용 → 별도 staging copy → 전체 트랜잭션 마이그레이션 → `integrity_check`/FK/Canonical/Feature 검사 → 안전 교체. 실패 시 **원본 bytes 불변**, 원본을 덮어쓰지 않는다. Migration은 멱등성/중단 회귀 필요.
- **한정 호환:** 기존 `boardsi-model`이 현재 표현하지 못하는 정식 확장이 있다면 구버전 모델을 강제로 끼워 넣지 않고 해당 필드를 Opaque로 안전 보존하거나 필수 확장으로 거부한다.

**과거 정식 파일의 현재 수:** F_001 시점에 공식 배포된 `.bsi` 버전/Golden Fixture **증거 없음**. 첫 정식 writer를 발행하는 F 단계에서 처음으로 Golden Fixture를 만들고 그때부터 이전 버전 호환 회귀를 고정한다. 현재 호환성은 **설계 의무**, 구현 시험 PASS가 아니다.

## 6. 저장 원자성, 단일 파일 배포, 보안

- 일반 편집 중 SQLite rollback journal 또는 WAL sidecar가 임시 존재할 수 있다. **완료된 공유·배포 결과**는 검증된 단일 `.bsi` 파일이어야 한다. **활성 WAL의 main DB 파일만 복사하는 것은 금지**한다.
- 정식 저장/Save As는 staging 데이터베이스를 SQLite Backup API 또는 이에 준하는 일관된 스냅샷으로 구성하고, 실제 참조 검증·무결성 확인 후 안전한 원자적 교체를 시도한다. OS별 directory fsync·Windows 파일 교체·Crash 경계는 후속 실측 검증 대상으로 한다.
- checkpoint가 성공했다는 추정만으로 삭제/교체를 하지 않는다. reader/writer 충돌, disk full, 다른 프로세스 lock, 정전/프로세스 종료 시 **마지막 정상 파일을 보존**한다.
- 기본 열기는 read-only. 저장 승인을 받은 경우에만 임시 파일에 write, SQLite `foreign_keys=ON`, 트랜잭션 상태/commit 성공 검증. `integrity_check` 통과만으로 Source/Layer/Net/Geometry 의미 정확도를 주장하지 않는다.
- 파일 크기, 페이지 수, BLOB 크기, 레코드 수, 재귀 깊이, 참조 수, 변환 비용, 압축 해제량에 **테스트된 구현 상한**을 설정한다. 한도 초과 → 명시적 오류, GUI 비차단.
- `.bsi`는 자체 암호화·전자서명·변조 방지 포맷이 아니다. SHA-256은 손상·동일성 검증에 사용하며 **발신자 진위나 물리 설계 정당성 증명은 아니다.** 민감 설계는 별도 디스크/파일 접근통제를 따른다.

## 7. F_002–F_005 인수 경계

| 단계 | F_001 고정 계약에 따른 후속 작업 | 현재 상태 |
| :--- | :--- | :--- |
| F_002 | 실제 테이블 DDL, FK/index, envelope 내부 타입별 byte layout, `STRICT`/최소 SQLite 버전, limits, Fixture 정의 | NOT_RUN |
| F_003 | Rust Native Writer/Reader, 비차단 Qt 연계, 실패 원자성, 실제 `.bsi` round-trip | NOT_RUN |
| F_004 | 최초 Golden Fixture부터 확장·migration·future/unknown fail-closed 회귀 | NOT_RUN |
| F_005 | **독립 검증한 CAD Source 부분집합만** Import→Canonical→.bsi→Canonical 비교 | NOT_RUN |

**비범위/기존 차단:** EB-01 Vendor ODB++ Oracle, EB-02 macOS Qt 원인, EB-03 GPU 실기기, EB-04 PH-08 원본 FAIL, EB-05 배포/권리, EB-06 전체 CAD 및 제조 전기 사실, EB-07 U/T 공식 제품 인수. 차단 이슈를 이 규격 작성만으로 닫지 않는다.

## 8. 외부 사례 교차 검증과 근거

| 근거 | 확인 사실 | F_001 선택 |
| :--- | :--- | :--- |
| [SQLite application file format](https://www.sqlite.org/appfileformat.html) | 스키마를 가진 SQLite 파일은 단일 파일·트랜잭션·확장성을 제공 | 표준 SQLite를 네이티브 컨테이너로 채택 |
| [SQLite file format](https://www.sqlite.org/fileformat2.html), [PRAGMA](https://www.sqlite.org/pragma.html), [application-id 등록 논의](https://sqlite.org/forum/info/6a768e7dca11a7b2) | SQLite 헤더 및 application_id/user_version은 표준 구조 | 자체 외부 헤더 금지, 버전·식별 분리 |
| [SQLite WAL](https://www.sqlite.org/wal.html) | `-wal`, `-shm` 별도 파일과 checkpoint 주의 | 배포용 snapshot/단일 파일 확정 절차 |
| [SQLite STRICT](https://www.sqlite.org/stricttables.html) | STRICT는 SQLite 3.37.0 이후 | 최소 런타임 버전과 DDL 선택 함께 검증 |
| [KiCad PCB format](https://dev-docs.kicad.org/en/file-formats/sexpr-pcb/), [좌표 계약](https://dev-docs.kicad.org/en/file-formats/sexpr-intro/) | 문서 구조에서 Layer/Net/Footprint/Track/Zone을 명시하며 Source Frame 단위가 정의됨 | Native를 Source 구문과 독립시켜 Provenance 보존 |
| [Siemens ODB++ Design specification](https://odbplusplus.com/wp-content/uploads/sites/2/2024/08/odb_spec_user.pdf) | 별도 Step/Layer/EDA/제조 정보 표현 | Source 제조 주장과 Canonical 접속 검증 분리 |
| [BoardSI Canonical 모델](canonical-board-model.md), [D_056_01 감사](d056-01-source-evidence-audit.md) | i64/i128, ObjectId, Capability, 그래프 증거 경계 | 기존 의미와 양립하도록 설계 |

**검증 판정:** 위 선택은 **F_001 문서/정적 규격 결정**. OCI/IPC 표준 전체 포맷 인증이나 실제 SQLite/Rust/Qt 실행 결과를 주장하지 않는다. [F_001 검토 결과](../validation/f001-bsi-format-qualification.md)에서 시험/불확실성을 별도 기록한다.
