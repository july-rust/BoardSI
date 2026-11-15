# F_002 — .bsi SQLite v1 물리 스키마 계약

> F_001 상위 계약: [f001-bsi-format-contract.md](f001-bsi-format-contract.md) · 실행 DDL: [schemas/bsi/v1.sql](../../schemas/bsi/v1.sql) · 검증: [F_002](../validation/f002-bsi-schema-qualification.md) · 기준일 2026-11-15 KST
>
> 범위: **실행 가능한 최초 DDL/관계형 저장 매핑**, 배포 Writer/Reader 구현 아님. 아래 수치는 제품 용량/성능 보증값이 아니라 안전한 최초 정책 상한이다.

## 1. 파일 정체성 및 초기화

* 외형: 표준 SQLite header (`SQLite format 3\0`), 독자 header/ZIP 없음. `application_id=0x42534931` (BSI1); `user_version=1`.
* `file_meta`: 정확히 1행 `singleton=1`, `format_major=1`, `format_minor=0`, `min_reader_major/minor=1/0`, UUID, BoardKind, 생성자 정보. 읽기 시 **0행/2행/값 부정합 모두 거부**.
* `features`: (name, version) PK, required 0/1, opaque BLOB. `extensions`도 별도 PK와 required 표기를 갖는다.
* `PRAGMA`: `foreign_keys=ON`, `trusted_schema=OFF`, `journal_mode=DELETE`, `page_size=4096`, `auto_vacuum=NONE`, UTF-8. `foreign_keys`는 **연결별 명시 활성화**; PRAGMA를 트랜잭션 안에서 바꾸지 않는다.
* 최소 SQLite **3.37.0**, 최초 스키마는 전 테이블 `STRICT`. Python/OS에 동봉된 SQLite를 제품 배포판과 동일하다고 가정하지 않는다. F_003에서 실제 Rust 번들 SQLite 버전/빌드 해시를 잠그고 각 OS에서 재검증한다.
* `magic.txt` [현재 공개 등록 목록](https://raw.githubusercontent.com/sqlite/sqlite/master/magic.txt)에 `0x42534931`이 **2026-11-15 조회 당시 없음**. 공개 파일에 미등재라는 사실은 영구 배타적 예약을 뜻하지 않는다. 공식 출시 전 재확인·등록 정책을 정한다.

## 2. DDL 관계 계약

| 도메인 | 핵심 테이블 | 중요한 FK/비손실 규칙 |
| :--- | :--- | :--- |
| 파일/호환 | `file_meta`, `features`, `extensions` | DDL `user_version`과 Format/Reader/Feature를 별도 비교 |
| 원본 CAD | `source_identity`, `source_files`, `source_conditions` | format/version/importer/원본 논리경로/SHA256/조건 순서 전체 유지 |
| 수입 범위 | `import_capability`, `capability_coverage` | G/N/P/S/R/M 각각 Complete/Partial/Missing/Conflicting; 미확인은 미확인 |
| 영구 객체 | `objects`, `conversion_history` | 전체 SourceIdentity + kind + source_object + part의 **의미 튜플**; 정수 키는 DB 내부 FK |
| 정밀 좌표 | `coordinate_frames` | metres-per-tick / origin은 각각 signed 16B 분자·분모; Point tick은 i64 |
| 물성/적층 | `materials`, `material_properties`, `layers` | Missing/Assumed/Confirmed 분리, Layer order 고유, thickness/z 각각 별도 상태 |
| 회로 논리 | `nets`, `components`, `pins` | Pin→Component/Pad/Net; 이름만으로 연결 승격 금지 |
| 기하/관계 | `geometry`, `geometry_refs` | 객체당 하나의 typed BSIG BLOB, layer/net/padstack/span 및 참조 사전 |
| 접속/증거 | `connectivity_nodes`, `connectivity_edges` | Graph Edge는 명시적 입력만; `Candidate`는 `Verified`가 아님 |
| 진단 | `diagnostics` | Error/Missing/Assumption/Information 및 Source evidence |

SQLite `CHECK`는 형식/길이/상위 Enum의 일부만 방어한다. 연결 관계와 부호/기약분수, 제조 출처/진실성은 반드시 **F_003 Reader의 별도 의미 검사**를 통과해야 한다.

### 정본 ID와 다중 출처

`objects.id` 및 `source_identity.id` 등 INTEGER는 동일 파일 내에서만 유효한 임시 참조다. 영구 ObjectId는 다음의 내용 동등성 기준으로 복원한다:

`(SourceFormat, FormatVersion, sorted[(logical_path,SHA256)], ImporterVersion, sorted[conditions], ObjectKind, source_object, part)`.

SourceIdentity 객체의 모든 정보는 SQL 정규화 행에 존재해야 한다. Hash 값만으로 동일성 판단 불가. 원본 파일의 절대경로를 기록하지 않는다. `source_files.sha_verified`는 0/1인 *입력 검증 사실*이지, 해시 BLOB이 존재한다는 이유로 1이 될 수 없다.

SQL은 `source_files` 순서의 유일성과 `source_conditions` 키 유일성을 보장하지만, **정렬/연속 ordinal·source_identity 전체 tuple 유일성은 Reader가 반드시 검사**한다. 빈 Source/중복 정체성도 Reader가 거부한다.

### 좌표 및 Unknown

* `i128`은 정확한 little-endian signed 16B 두 보수. 분모>0, 분자 != MIN, 기약분수여야 한다. `0/1`은 합법, `0/0`은 아님. SQLite BLOB CHECK 길이만으로는 이 조건을 강제할 수 없다.
* `material_properties.value_blob`는 **Base64 문자열이 아닌 원본 binary64 값 또는 FrequencyRange 두 binary64를 담는 BLOB**이다. F_003 구현 시 이 의미를 반드시 고수하고 인코딩된 ASCII Base64를 넣지 않는다.
* `PendingRotation`은 f64 radians를 손실 없이 저장하고, 모델이 변환 미지원인 경우 그 상태 유지. 무단 정수화 금지.
* `geometry_refs.ref_index`는 각 BLOB 안에서만 사용하는 **u32 로컬 순서형 참조**다. SQL row ID가 영구 ID가 아님. Writer는 참조 대상의 전체 ObjectId 의미 튜플을 정렬하여 0부터 연속 인덱스를 만들고, Reader는 대상 Kind/정렬/연속성 및 실제 Shape 요구사항을 재검증한다.
* `geometry.geometry_kind`는 `BSIG` envelope kind와 반드시 동일해야 한다. SQL은 값 범위만 제한하며 Reader가 일치 확인.
* 지지되지 않는 CAD Source 형상(예: 현재 Canonical이 정확히 표현하지 못하는 제조 특징)을 `Trace`/Pad 등으로 위장하지 않는다. 필요하면 required 확장으로 명시하고 호환 Reader는 거부한다.

### Source/Property/Contact 정확도 보강

- `layer_property_sources`는 `Layer.z_m`과 `Layer.thickness_m`의 `PropertySource`(document/revision/method/reference_frequency/temperature)를 별도로 보존한다. Missing에는 Source 없음, Assumed/Confirmed에는 각각의 Source 1건을 Reader가 필수 확인한다.
- `connectivity_nodes`는 `witness_kind`, `trace_vertex_index`, `witness_evidence`, frame별 정확 tick을 보존한다. `TraceInterior`/ `AreaWitness` 등의 의미를 단순 점 일치로 축소하지 않는다.
- `connectivity_edges`는 `evidence_kind`의 SameLayerContact/ViaBarrel/AreaContinuity, `witness_object_id`(Via/Area), evidence text를 구분한다. Witness 객체의 Kind 및 Layer/Net 구조 검사는 Reader 책임이며, 선언을 물리적 검증 사실로 자동 승격하지 않는다.
- `material_properties.value_blob`의 `Dk`, `Df`, `Conductivity`는 **IEEE-754 finite f64 Little-Endian 8 bytes**; `ValidBand`는 `f64 min_hz + f64 max_hz`의 16 bytes다. 상태/참고 Source를 동반하고 `NaN/Inf` 및 범위 부정합은 Reader가 거부한다. SQL의 REAL 자료형에는 자료 출처 참고치만 저장한다.

## 3. 열기/저장 순서

`BSI_NOT_NATIVE`: magic/application_id 불일치 → `BSI_FUTURE_SCHEMA`: 미래 DDL/major → `BSI_UNSUPPORTED_REQUIRED`: 지원하지 않는 required/min_reader/codec → `BSI_INVALID`: 무결성/FK/의미/Envelope 검증 오류.

Reader의 안전한 순서는 **파일 물리 검사 → SQLite readonly/open 방어 → 버전/Feature → integrity_check → foreign_key_check → 객체/Geometry/출처 의미 검사 → 제한된 구조 노출**. Unknown optional을 손실 없이 재직렬화할 수 없다면 **읽기 전용**이다.

Save/Save As는 원본 직접 편집 금지. 새 staging 파일에서 트랜잭션·검사 완료 후 OS별 안전한 교체. WAL main 파일만 따로 복사 금지. 이 실제 실패 원자성·동시 접근·Crash 시험은 **F_003 인수**다.

## 4. 초기 안전 정책 (제품 성능 실측 아님)

| 자원 | 최초 논리 한도 | 상태 |
| :--- | ---: | :--- |
| 입력 파일 길이 | 512 MiB | 정책값; 실제 대형 PCB 수용성 미검증 |
| Geometry Envelope payload | 64 MiB/객체 | Python 참조 Decoder에서 초과 거부 시험 |
| 전체 Objects | 1,000,000 | Writer/Reader 실제 부하 NOT_RUN |
| Geometry refs/객체 | 65,535 | 인덱스 검사 F_003 |
| Boundary segments/객체 | 1,000,000 | 데이터 크기 상한과 병행 |
| 임포트 소스 파일 | 100,000 | Path/Source validation F_003 |
| SQLite 페이지 | `512MiB/4096=131072` | 파일 용량 정책과 연동 |
| TLV field 수 | 최대 64 | Python 참조 Decoder 검사 |

이 값은 F_002 규격의 **보수적인 기본 거부 경계**다. 운영체제별 주소 공간·메모리·성능과 무관하게 위 숫자까지 무조건 지원된다는 뜻이 아니다. F_003에서 메모리 예산, 처리 시간, SQLite `max_page_count`, `sqlite3_limit`, 객체별 row 한도를 실제 적용/시험한다.

## 5. 인덱스·이후 마이그레이션

현재 Schema에는 `objects_source_kind`, `geometry_layer_net`, `geometry_padstack`, `geometry_refs_target`, `pins_component`, `connectivity_edges_nodes`, `diagnostics_object` 등 실 조회 경로의 인덱스를 정의한다. F_002 시험은 EXPLAIN QUERY PLAN으로 인덱스 사용 여부를 확인한다. **실 CAD 1M 속도/메모리 보증은 미인수**.

정식 v1 발행 후 DDL 변경은 `user_version`을 증가시키고, 불변 Golden Fixture와 버전별 Migration을 유지한다. 구 파일을 업데이트하는 동안 원본을 덮어쓰지 않는다. Schema v1의 필수 컬럼을 임의 삭제하거나 기존 BLOB 인코딩을 재해석하지 않는다.

### 검증 소스

- [SQLite Application File Format](https://www.sqlite.org/appfileformat.html)
- [SQLite STRICT tables](https://www.sqlite.org/stricttables.html)
- [SQLite Foreign Key Support](https://www.sqlite.org/foreignkeys.html)
- [SQLite PRAGMA](https://www.sqlite.org/pragma.html)
- [SQLite Limits](https://www.sqlite.org/limits.html)
- [F_001 계약](f001-bsi-format-contract.md) 및 [Canonical 모델](canonical-board-model.md)
